#!/usr/bin/env node
const { execSync } = require('child_process');

// Read input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const event = JSON.parse(input);
    const prompt = event.prompt || '';

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
      console.log(JSON.stringify({
        contextPrefix: `\n---\n${injection}---\n`
      }));
    } else {
      console.log(JSON.stringify({}));
    }

  } catch (error) {
    // Silently fail - output empty result
    console.log(JSON.stringify({}));
  }
});
