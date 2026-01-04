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
      const escapedPrompt = prompt.replace(/"/g, '\\"').replace(/`/g, '\\`').substring(0, 500);
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

    // Output combined context
    if (contextParts.length > 0) {
      console.log(contextParts.join('\n\n'));
    }

  } catch (error) {
    // Silently fail - don't block user prompt
  }
});
