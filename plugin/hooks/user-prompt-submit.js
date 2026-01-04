#!/usr/bin/env node
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Check if AgentMem is initialized in this project
function findAgentMemDir() {
  let dir = process.cwd();
  while (dir !== '/') {
    if (fs.existsSync(path.join(dir, '.agentmem'))) {
      return path.join(dir, '.agentmem');
    }
    dir = path.dirname(dir);
  }
  return null;
}

// Message counter for periodic reminders
function getMessageCount(amDir) {
  const counterFile = path.join(os.tmpdir(), `agentmem-counter-${Buffer.from(amDir).toString('base64').slice(0, 20)}`);
  try {
    return parseInt(fs.readFileSync(counterFile, 'utf8')) || 0;
  } catch {
    return 0;
  }
}

function incrementMessageCount(amDir) {
  const counterFile = path.join(os.tmpdir(), `agentmem-counter-${Buffer.from(amDir).toString('base64').slice(0, 20)}`);
  const count = getMessageCount(amDir) + 1;
  fs.writeFileSync(counterFile, count.toString());
  return count;
}

// Read input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const event = JSON.parse(input);
    const prompt = event.prompt || '';

    // Check if AgentMem is initialized
    const amDir = findAgentMemDir();
    if (!amDir) {
      // Not initialized - output setup instructions
      console.log(`## AgentMem Not Initialized

This project doesn't have AgentMem set up yet. To enable persistent memory:

\`\`\`bash
am init
\`\`\`

This will:
- Create .agentmem/ directory
- Set up SQLite database
- Optionally configure semantic search (requires Docker + OpenAI key)
- Install Claude Code plugin

Run \`am doctor\` to check system requirements.
`);
      return;
    }

    // Track message count for periodic reminders
    const messageCount = incrementMessageCount(amDir);

    // Start session
    try {
      execSync('am session start --agent claude-code', {
        encoding: 'utf-8',
        timeout: 5000,
        stdio: 'pipe'
      });
    } catch (e) {
      // Silently continue
    }

    let contextParts = [];

    // Get saved TodoWrite state from previous session
    try {
      const todosJson = execSync('am session get-todos --json', {
        encoding: 'utf-8',
        timeout: 5000,
        stdio: 'pipe'
      }).trim();

      if (todosJson && todosJson !== 'null' && todosJson !== '[]') {
        const todos = JSON.parse(todosJson);
        if (Array.isArray(todos) && todos.length > 0) {
          let todoList = '## Continuing from previous session\n\n';
          todoList += 'You have pending tasks from the last session. Use TodoWrite to restore this state:\n\n';
          todoList += '```json\n' + JSON.stringify(todos, null, 2) + '\n```\n\n';
          todoList += 'Please restore these tasks using the TodoWrite tool before proceeding.\n';
          contextParts.push(todoList);
        }
      }
    } catch (e) {
      // No saved todos or error - continue
    }

    // Check for active plan with tasks
    try {
      const planJson = execSync('am plan active --json', {
        encoding: 'utf-8',
        timeout: 5000,
        stdio: 'pipe'
      }).trim();

      if (planJson && planJson !== 'null') {
        const plan = JSON.parse(planJson);
        if (plan && plan.id) {
          // Get tasks linked to this plan
          const tasksOutput = execSync('am task list --json', {
            encoding: 'utf-8',
            timeout: 5000,
            stdio: 'pipe'
          }).trim();

          const allTasks = JSON.parse(tasksOutput || '[]');
          const openTasks = allTasks.filter(t => t.status === 'open' || t.status === 'in_progress');

          if (openTasks.length > 0) {
            let planContext = `## Active Plan: ${plan.title}\n\n`;
            planContext += `You have ${openTasks.length} tasks to complete:\n\n`;
            openTasks.forEach((t, i) => {
              const status = t.status === 'in_progress' ? '🔄' : '⬜';
              planContext += `${i + 1}. ${status} [${t.id}] ${t.title}\n`;
            });
            planContext += '\nWork through these tasks in order. Mark each complete with `am task update <id> closed`.\n';
            contextParts.push(planContext);
          }
        }
      }
    } catch (e) {
      // No active plan - continue
    }

    // Get relevant context based on prompt
    try {
      const escapedPrompt = prompt
        .replace(/\\/g, '\\\\')   // Escape backslashes first
        .replace(/"/g, '\\"')     // Escape double quotes
        .replace(/`/g, '\\`')     // Escape backticks
        .replace(/\$/g, '\\$')    // Escape dollar signs (prevents $HOME, $(cmd) expansion)
        .replace(/!/g, '\\!')     // Escape history expansion
        .substring(0, 500);
      const context = execSync(`am context --query "${escapedPrompt}" --format markdown`, {
        encoding: 'utf-8',
        timeout: 5000,
        stdio: 'pipe'
      }).trim();

      if (context) {
        contextParts.push(context);
      }
    } catch (e) {
      // No context or error - continue
    }

    // Periodic memory checkpoint reminder (every 10 messages)
    if (messageCount > 0 && messageCount % 10 === 0) {
      contextParts.push(`## Memory Checkpoint (Message ${messageCount})

**Pause and save any learnings before continuing.** Ask yourself:
- Did I make any decisions worth remembering?
- Did I fix any bugs or discover gotchas?
- Did the user correct me on anything?

If yes, run \`am mem add <type> "<title>" --content "<details>"\` now.
`);
    }

    // Output combined context
    if (contextParts.length > 0) {
      console.log(contextParts.join('\n\n'));
    }

  } catch (error) {
    // Silently fail - don't block user prompt
  }
});
