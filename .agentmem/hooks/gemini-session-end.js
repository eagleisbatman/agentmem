#!/usr/bin/env node
const { spawn } = require('child_process');

// Read JSON input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    // Sync to git (non-blocking)
    spawn('am', ['sync'], {
      detached: true,
      stdio: 'ignore'
    }).unref();

    console.log(JSON.stringify({ status: 'success' }));
  } catch (error) {
    console.log(JSON.stringify({ status: 'success' }));
  }
});
