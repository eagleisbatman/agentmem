---
name: plan-to-tasks
description: Convert implementation plans into trackable tasks. Use after creating a plan to ensure systematic execution.
---

# Plan to Tasks Skill

When you create an implementation plan, convert it into trackable tasks for systematic execution.

## When to Trigger

### After Plan Mode
When you exit plan mode after creating an implementation plan:
1. The plan file has been written
2. The user has approved the plan
3. You're ready to start implementation

### After Creating a Plan
When you've outlined steps to accomplish a task:
- "Here's my plan: 1. Do X, 2. Do Y, 3. Do Z"
- Written a plan to a file
- Received approval on an approach

## How to Convert Plan to Tasks

### Step 1: Identify Discrete Tasks
Break the plan into independent, actionable tasks:
- Each task should be completable in one focused effort
- Tasks should have clear completion criteria
- Order tasks by dependencies (blockers first)

### Step 2: Create Tasks in AgentMem
For each task, run:
```bash
am task create "<task title>" --description "<details from plan>" --priority <0-4> --type <type>
```

**Priority levels**:
- 0: Critical (blockers)
- 1: High (core functionality)
- 2: Medium (default)
- 3: Low (nice to have)
- 4: Backlog

**Task types**: bug, feature, task, epic, chore

### Step 3: Report Created Tasks
After creating tasks, run:
```bash
am task list
```

Show the user what was created.

### Step 4: Start First Task
Begin working on the highest priority unblocked task:
```bash
am task ready
```

Use TodoWrite to track sub-steps within the task.

## Example Workflow

```
Plan: Implement user authentication
1. Set up database schema for users
2. Create registration endpoint
3. Create login endpoint
4. Add JWT token generation
5. Add middleware for protected routes
6. Write tests

Convert to tasks:
am task create "Set up users database schema" --priority 1 --type task
am task create "Create registration endpoint" --priority 1 --type feature
am task create "Create login endpoint" --priority 1 --type feature
am task create "Add JWT token generation" --priority 1 --type feature
am task create "Add auth middleware" --priority 2 --type feature
am task create "Write authentication tests" --priority 2 --type task

Report: "Created 6 tasks from the plan. Starting with: Set up users database schema"
```

## Task Progression

When you complete a task:

1. Mark it done: Update your TodoWrite
2. The memory-persistence skill will capture the completion
3. Check for next task: `am task ready`
4. Start the next unblocked task

## Linking to Plans (Future)

When database support is added:
- Tasks will link to their source plan
- You can view all tasks for a plan
- Plan completion status will be tracked

For now, include a reference to the plan in task descriptions.

## Best Practices

- Create tasks immediately after plan approval (don't delay)
- Keep task titles short and action-oriented
- Include enough description to understand the task later
- Set realistic priorities
- Don't create tasks that are too granular (use TodoWrite for sub-steps)
