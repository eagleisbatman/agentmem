#!/usr/bin/env node
const { spawn, execSync } = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');

// Read input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const event = JSON.parse(input);
    const { transcript, sessionId, tokensIn, tokensOut, model } = event;

    // End cloud session tracking
    try {
      let cmd = 'am session end';
      if (tokensIn) cmd += ` --tokens-in ${tokensIn}`;
      if (tokensOut) cmd += ` --tokens-out ${tokensOut}`;
      if (model) cmd += ` --model "${model}"`;
      execSync(cmd, { encoding: 'utf-8', timeout: 5000 });
    } catch (e) {
      // Silently continue
    }

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

    // Output success
    console.log(JSON.stringify({}));

  } catch (error) {
    // Silently fail
    console.log(JSON.stringify({}));
  }
});
