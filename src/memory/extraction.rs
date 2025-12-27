use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::embedding::openai::chat_completion;
use crate::memory::service::add_memory_with_embedding;
use crate::retrieval::search::semantic_search;

/// Extracted memory from transcript analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedMemory {
    #[serde(rename = "type")]
    pub memory_type: String,
    pub title: String,
    pub content: String,
    pub source_chunk: String,
    pub confidence: i32,
    #[serde(default)]
    pub entities: Vec<ExtractedEntity>,
    pub reasoning: String,
}

/// Entity extracted from transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub name: String,
}

/// Result of extraction operation
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub memories: Vec<ExtractedMemory>,
    pub tokens_used: u32,
    pub model: String,
}

/// Result of extraction with deduplication stats
#[derive(Debug)]
pub struct ExtractionStats {
    pub extracted: usize,
    pub stored: usize,
    pub duplicates: usize,
}

/// The extraction prompt for GPT-4o
const EXTRACTION_PROMPT: &str = r#"You are a memory extraction assistant. Analyze this session transcript and extract learnings that would be valuable for future AI agent sessions.

## Memory Types to Extract:
1. **correction** (confidence 90+): User corrected agent behavior or pointed out mistakes
2. **infrastructure** (85+): URLs, endpoints, API keys, database configs, service locations
3. **tool** (85+): Existing scripts, utilities, or commands the user mentioned
4. **protected** (95+): Files or paths user said not to modify
5. **decision** (70+): Architectural choices with reasoning
6. **gotcha** (75+): Things that broke, surprised, or caused issues
7. **pattern** (60+): Repeated behaviors or preferences (mentioned 2+ times)
8. **insight** (60+): Non-obvious discoveries about the codebase or project

## Rules:
- Only extract SURPRISING or NON-OBVIOUS information
- Do NOT extract general programming knowledge
- Do NOT extract information that would be obvious from reading the code
- Focus on corrections, gotchas, and user preferences
- Include exact source_chunk quotes from the transcript
- Higher confidence = more certain this is worth remembering

## Output Format (JSON):
{
  "memories": [
    {
      "type": "correction",
      "title": "Brief title (5-10 words)",
      "content": "Full description of what was learned",
      "source_chunk": "Exact transcript excerpt that led to this",
      "confidence": 90,
      "entities": [{"type": "person|service|file", "name": "..."}],
      "reasoning": "Why this is worth remembering"
    }
  ]
}

If no memories are worth extracting, return: {"memories": []}

## Transcript to Analyze:
"#;

/// Extract memories from a transcript
pub async fn extract_from_transcript(
    transcript: &str,
    model: &str,
) -> Result<ExtractionResult> {
    // Build the full prompt
    let user_prompt = format!("{}\n\n{}", EXTRACTION_PROMPT, transcript);

    // Call GPT-4o
    let response = chat_completion(
        model,
        "You are a memory extraction assistant. Output valid JSON only.",
        &user_prompt,
    ).await?;

    // Parse the response
    let extraction: ExtractionResponse = parse_extraction_response(&response)?;

    Ok(ExtractionResult {
        memories: extraction.memories,
        tokens_used: 0, // We don't have token count from current API
        model: model.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct ExtractionResponse {
    memories: Vec<ExtractedMemory>,
}

/// Parse the GPT response, handling potential JSON issues
fn parse_extraction_response(response: &str) -> Result<ExtractionResponse> {
    // Try direct parse first
    if let Ok(result) = serde_json::from_str::<ExtractionResponse>(response) {
        return Ok(result);
    }

    // Try to extract JSON from markdown code blocks
    let json_str = if response.contains("```json") {
        response
            .split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .unwrap_or(response)
            .trim()
    } else if response.contains("```") {
        response
            .split("```")
            .nth(1)
            .unwrap_or(response)
            .trim()
    } else {
        response.trim()
    };

    serde_json::from_str::<ExtractionResponse>(json_str)
        .context("Failed to parse extraction response as JSON")
}

/// Extract memories and store them (with optional deduplication)
pub async fn extract_and_store(
    conn: &Connection,
    transcript: &str,
    model: &str,
    dedupe: bool,
) -> Result<ExtractionStats> {
    // Extract memories from transcript
    let result = extract_from_transcript(transcript, model).await?;

    let extracted = result.memories.len();
    let mut stored = 0;
    let mut duplicates = 0;

    for memory in result.memories {
        // Check for duplicates if deduplication is enabled
        if dedupe {
            if is_duplicate(conn, &memory).await? {
                duplicates += 1;
                continue;
            }
        }

        // Store the memory
        store_extracted_memory(conn, &memory).await?;
        stored += 1;
    }

    Ok(ExtractionStats {
        extracted,
        stored,
        duplicates,
    })
}

/// Check if a memory is a duplicate of an existing one
async fn is_duplicate(conn: &Connection, memory: &ExtractedMemory) -> Result<bool> {
    // Try semantic search for similar memories
    let search_text = format!("{}: {}", memory.title, memory.content);

    match semantic_search(conn, &search_text, 3).await {
        Ok(results) => {
            // If any result has score > 0.9, consider it a duplicate
            for r in results {
                if r.score > 0.9 {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Err(_) => {
            // If semantic search fails, check by exact title match
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM memories WHERE title = ?1"
            )?;
            let count: i64 = stmt.query_row([&memory.title], |row| row.get(0))?;
            Ok(count > 0)
        }
    }
}

/// Store an extracted memory in the database
async fn store_extracted_memory(conn: &Connection, memory: &ExtractedMemory) -> Result<Uuid> {
    // Add memory with embedding
    let (id, _embedded) = add_memory_with_embedding(
        conn,
        &memory.memory_type,
        &memory.title,
        Some(&memory.content),
    ).await?;

    // Update additional fields (source_chunk, confidence)
    conn.execute(
        "UPDATE memories SET source_chunk = ?1, confidence = ?2 WHERE id = ?3",
        rusqlite::params![memory.source_chunk, memory.confidence, id.to_string()],
    )?;

    Ok(id)
}

/// Read transcript from file (supports JSONL and plain text)
pub fn read_transcript_file(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read transcript file: {}", path))?;

    // If it looks like JSONL, parse and concatenate messages
    if content.trim_start().starts_with('{') {
        let mut transcript = String::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(role) = msg.get("role").and_then(|r| r.as_str()) {
                    if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                        transcript.push_str(&format!("{}: {}\n\n", role, content));
                    }
                }
            }
        }
        if !transcript.is_empty() {
            return Ok(transcript);
        }
    }

    // Otherwise return as-is
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extraction_response() {
        let json = r#"{"memories": [{"type": "correction", "title": "Test", "content": "Content", "source_chunk": "chunk", "confidence": 90, "entities": [], "reasoning": "test"}]}"#;
        let result = parse_extraction_response(json).unwrap();
        assert_eq!(result.memories.len(), 1);
        assert_eq!(result.memories[0].title, "Test");
    }

    #[test]
    fn test_parse_extraction_with_code_block() {
        let json = "Here is the result:\n```json\n{\"memories\": []}\n```\n";
        let result = parse_extraction_response(json).unwrap();
        assert_eq!(result.memories.len(), 0);
    }
}
