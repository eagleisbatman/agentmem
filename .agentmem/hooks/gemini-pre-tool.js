#!/usr/bin/env node
const { execSync } = require('child_process');

// Read JSON input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const event = JSON.parse(input);

    // Get context from AgentMem
    const result = execSync('am context --json', { encoding: 'utf-8', timeout: 5000 });
    const context = JSON.parse(result);

    // Output context as JSON (Gemini will include in prompt)
    const output = {
      status: 'continue',
      context: formatContext(context)
    };

    console.log(JSON.stringify(output));
  } catch (error) {
    // Don't block on errors
    console.log(JSON.stringify({ status: 'continue' }));
  }
});

function formatContext(context) {
  let text = '';

  if (context.protected?.length > 0) {
    text += '## Protected Files\n';
    context.protected.forEach(p => {
      text += `- \`${p.pattern}\` - ${p.reason || 'Protected'}\n`;
    });
    text += '\n';
  }

  if (context.memories?.length > 0) {
    text += '## Relevant Context\n';
    context.memories.forEach(m => {
      text += `- [${m.memory_type}] ${m.title}: ${m.content || ''}\n`;
    });
  }

  return text;
}
