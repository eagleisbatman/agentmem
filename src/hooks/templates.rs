/// Pre-prompt hook template for Claude Code
/// This hook runs before each user prompt and injects AgentMem context
pub const PRE_PROMPT_HOOK: &str = r#"const { execSync } = require('child_process');

module.exports = {
  event: 'UserPromptSubmit',

  async handler({ prompt }) {
    try {
      // Escape the prompt for shell
      const escapedPrompt = prompt.replace(/"/g, '\\"').replace(/\n/g, ' ');

      // Get context from AgentMem CLI
      const result = execSync(
        `am context --query "${escapedPrompt}" --json`,
        { encoding: 'utf-8', timeout: 5000 }
      );

      const context = JSON.parse(result);

      // Format as markdown
      let injection = '';

      if (context.protected && context.protected.length > 0) {
        injection += '## Protected Files\n';
        injection += 'Ask before modifying:\n';
        context.protected.forEach(p => {
          injection += `- \`${p.pattern}\` - ${p.reason || 'No reason provided'}\n`;
        });
        injection += '\n';
      }

      if (context.tasks && context.tasks.length > 0) {
        injection += '## Current Tasks\n';
        context.tasks.forEach(t => {
          injection += `- [P${t.priority}] ${t.id}: ${t.title} (${t.status})\n`;
        });
        injection += '\n';
      }

      if (context.memories && context.memories.length > 0) {
        injection += '## Relevant Context\n';
        context.memories.forEach(m => {
          injection += `- [${m.memory_type}] ${m.title}: ${m.content || ''}\n`;
        });
        injection += '\n';
      }

      if (context.tools && context.tools.length > 0) {
        injection += '## Available Tools\n';
        context.tools.forEach(t => {
          injection += `- \`${t.location}\` - ${t.description || ''}\n`;
        });
      }

      if (injection.trim()) {
        return {
          contextPrefix: `\n---\n${injection}---\n`
        };
      }

      return {};

    } catch (error) {
      // Silently fail - don't block the user's prompt
      console.error('AgentMem hook error:', error.message);
      return {};
    }
  }
};
"#;

/// Post-session hook template for Claude Code
/// This hook runs after session end for memory extraction and sync
pub const POST_SESSION_HOOK: &str = r#"const { spawn } = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');

module.exports = {
  event: 'SessionEnd',

  async handler({ transcript, sessionId }) {
    try {
      // Write transcript to temp file if provided
      if (transcript && Array.isArray(transcript) && transcript.length > 0) {
        const tempDir = os.tmpdir();
        const transcriptFile = path.join(tempDir, `am-transcript-${sessionId || Date.now()}.jsonl`);

        // Write transcript as JSONL
        const content = transcript
          .map(msg => JSON.stringify(msg))
          .join('\n');
        fs.writeFileSync(transcriptFile, content);

        // Extract memories (non-blocking)
        spawn('am', ['extract', '--transcript', transcriptFile], {
          detached: true,
          stdio: 'ignore'
        }).unref();

        // Clean up temp file after a delay
        setTimeout(() => {
          try { fs.unlinkSync(transcriptFile); } catch (e) {}
        }, 60000);
      }

      // Sync to git (non-blocking)
      spawn('am', ['sync'], {
        detached: true,
        stdio: 'ignore'
      }).unref();

    } catch (error) {
      console.error('AgentMem post-session error:', error.message);
    }
  }
};
"#;

/// CLAUDE.md section to append when installing hooks
pub const CLAUDE_MD_SECTION: &str = r#"
## AgentMem Integration

This project uses AgentMem for persistent context across sessions.

**Quick commands:**
- `am task ready` - See unblocked tasks
- `am context` - Get relevant memories
- `am mem add <type> <title>` - Add a memory
- `am protect <file>` - Mark file as protected
- `am sync` - Sync to git

Protected files require approval before modification.
"#;
