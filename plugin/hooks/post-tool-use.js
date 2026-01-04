#!/usr/bin/env node
const { spawn, execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Read input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const event = JSON.parse(input);
    const { tool_name, tool_input, tool_response, cwd } = event;

    // Detect plan file creation (Write tool to a plan file)
    if (tool_name === 'Write' && tool_input && tool_input.file_path) {
      const filePath = tool_input.file_path;
      const fileName = path.basename(filePath).toLowerCase();

      // Check if this looks like a plan file
      if (fileName.includes('plan') ||
          filePath.includes('/plans/') ||
          filePath.includes('.claude/plans')) {

        // Create plan in AgentMem and extract tasks
        try {
          const planTitle = path.basename(filePath, path.extname(filePath));

          // Create the plan record
          execSync(`am plan create "${planTitle}" --file "${filePath}"`, {
            encoding: 'utf-8',
            timeout: 5000,
            stdio: 'pipe',
            cwd: cwd || process.cwd()
          });

          // Extract tasks from plan content (non-blocking)
          if (tool_input.content) {
            // Write plan content to temp file for task extraction
            const tempFile = `/tmp/am-plan-${Date.now()}.md`;
            fs.writeFileSync(tempFile, tool_input.content);

            spawn('am', ['plan', 'extract-tasks', '--file', tempFile], {
              detached: true,
              stdio: 'ignore',
              cwd: cwd || process.cwd()
            }).unref();

            // Clean up
            setTimeout(() => {
              try { fs.unlinkSync(tempFile); } catch (e) {}
            }, 30000).unref();
          }
        } catch (e) {
          // Silently continue
        }
      }
    }

    // Detect TodoWrite usage - save state
    if (tool_name === 'TodoWrite' && tool_input && tool_input.todos) {
      try {
        const todosJson = JSON.stringify(tool_input.todos);
        execSync(`am session save-todos '${todosJson.replace(/'/g, "'\\''")}'`, {
          encoding: 'utf-8',
          timeout: 5000,
          stdio: 'pipe',
          cwd: cwd || process.cwd()
        });
      } catch (e) {
        // Silently continue
      }
    }

    // Output success (don't block tool execution)
    console.log(JSON.stringify({}));

  } catch (error) {
    // Silently fail
    console.log(JSON.stringify({}));
  }
});
