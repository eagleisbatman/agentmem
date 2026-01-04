#!/usr/bin/env node
const { spawn, execSync } = require('child_process');
const fs = require('fs');

// Read input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const event = JSON.parse(input);
    // Claude Code provides transcript_path (file path to JSONL transcript)
    const { transcript_path, session_id } = event;

    // Get our internal session ID for task release
    let agentId = session_id || 'unknown';
    try {
      // Try to get the AgentMem session ID
      const statusOutput = execSync('am session status 2>/dev/null || true', {
        encoding: 'utf-8',
        timeout: 3000,
        stdio: 'pipe'
      });
      // Parse session ID from output like "Current session: sess-abc123"
      const match = statusOutput.match(/session[:\s]+(\S+)/i);
      if (match) {
        agentId = match[1];
      }
    } catch (e) {
      // Use provided session_id or default
    }

    // Release all tasks claimed by this agent/session
    try {
      execSync(`am task release-all --agent "${agentId}"`, {
        encoding: 'utf-8',
        timeout: 5000,
        stdio: 'pipe'
      });
    } catch (e) {
      // Silently continue
    }

    // Cleanup stale claims from any crashed agents (30 min timeout)
    try {
      execSync('am task cleanup-stale --timeout 30', {
        encoding: 'utf-8',
        timeout: 5000,
        stdio: 'pipe'
      });
    } catch (e) {
      // Silently continue
    }

    // End session tracking
    try {
      execSync('am session end', { encoding: 'utf-8', timeout: 5000, stdio: 'pipe' });
    } catch (e) {
      // Silently continue
    }

    // Extract memories from transcript file if path provided
    if (transcript_path && fs.existsSync(transcript_path)) {
      // Extract memories (non-blocking) - reads directly from Claude's transcript file
      spawn('am', ['extract', '--transcript', transcript_path], {
        detached: true,
        stdio: 'ignore'
      }).unref();
    }

    // Sync to git (non-blocking)
    spawn('am', ['sync'], {
      detached: true,
      stdio: 'ignore'
    }).unref();

    // Output success
    console.log(JSON.stringify({}));

  } catch (error) {
    // Silently fail
    console.log(JSON.stringify({}));
  }
});
