# Claude Code Agents Tutorial

## Table of Contents
1. [What are Agents?](#what-are-agents)
2. [How Agent Selection Works](#how-agent-selection-works)
3. [Creating Your First Agent](#creating-your-first-agent)
4. [Common Agent Types & Examples](#common-agent-types--examples)
5. [Advanced Agent Patterns](#advanced-agent-patterns)
6. [Hierarchical Agent Systems](#hierarchical-agent-systems)
7. [Debugging Agent Issues](#debugging-agent-issues)
8. [Best Practices](#best-practices)

## What are Agents?

Agents (officially called "subagents") are specialized AI assistants in Claude Code that have:

- **Focused expertise** in specific domains
- **Custom system prompts** that guide their behavior
- **Specific tool permissions** (Read, Write, Bash, etc.)
- **Separate context windows** from the main Claude instance

### Key Benefits
- **Context preservation**: Each agent maintains its own conversation space
- **Specialized expertise**: Tailored for specific tasks and domains
- **Reusability**: Can be used across different projects and sessions
- **Flexible permissions**: Each agent only gets the tools it needs

### Storage Locations
- **Project agents**: `.claude/agents/` (version controlled with your project)
- **User agents**: `~/.claude/agents/` (personal, across all projects)

## How Agent Selection Works

Claude automatically chooses agents by analyzing:

1. **Your request keywords** - what you're asking for
2. **Agent descriptions** - what each agent claims to do
3. **Available tools** - whether the agent can actually perform the task
4. **Current context** - what files/project you're working with

### Selection Examples

**Request**: "Review this pull request for security issues"
```markdown
Available agents:
- code-reviewer: "Expert code review specialist for quality, security, and maintainability"
- test-writer: "Creates comprehensive unit and integration tests" 
- debugger: "Fixes runtime errors and bugs"

Claude picks: code-reviewer (matches "review" and "security")
```

**Request**: "My tests are failing with a weird error"
```markdown  
Available agents:
- debugger: "Debugging specialist for errors, test failures, and unexpected behavior"
- code-reviewer: "Reviews code for quality and security"

Claude picks: debugger (mentions "test failures" and "errors")
```

### Making Agents More Discoverable

Use **keyword-rich descriptions**:
```markdown
# ❌ Too generic
"General helper for code tasks"

# ✅ Specific with trigger words  
"React testing specialist for Jest, React Testing Library, component tests, hooks testing, and UI testing"
```

Add **proactive phrases**:
```markdown
"Security specialist. Use PROACTIVELY when reviewing authentication, authorization, or handling sensitive data"
```

## Creating Your First Agent

### Step 1: Use the `/agents` Command
```bash
/agents
```
This opens the agent creation interface where you can choose project-level or user-level agents.

### Step 2: Basic Agent Structure
Every agent file has this structure:
```markdown
---
name: agent-name
description: What this agent does and when to use it
tools: Read, Write, Bash, Grep
---

System prompt content goes here...
```

### Step 3: Simple Example - Code Formatter
```markdown
---
name: rust-formatter
description: Rust code formatting specialist using rustfmt and cargo fmt. Use for ANY Rust formatting tasks.
tools: Read, Edit, Bash
---

You are a Rust code formatting expert.

When invoked:
1. Always run `cargo fmt` to format all Rust code
2. Run `cargo clippy --fix` to auto-fix linting issues
3. Check for any remaining style issues
4. Report what was changed

Process any Rust files (.rs) for proper formatting and style.
Always verify changes don't break compilation with `cargo check`.
```

## Common Agent Types & Examples

### 1. Code Review Agent
```markdown
---
name: code-reviewer
description: Expert code review specialist. Proactively reviews code for quality, security, and maintainability.
tools: Read, Grep, Glob, Bash
---

You are a senior code reviewer ensuring high standards.

When invoked:
1. Run `git diff` to see recent changes
2. Focus on modified files  
3. Begin review immediately

Review checklist:
- Code is simple and readable
- Functions and variables are well-named
- No duplicated code
- Proper error handling
- No exposed secrets or API keys
- Input validation implemented
- Good test coverage
- Performance considerations

Provide feedback organized by priority:
- **Critical issues** (must fix)
- **Warnings** (should fix)  
- **Suggestions** (consider improving)

Include specific examples of how to fix issues.
```

### 2. Test Writer Agent
```markdown
---
name: rust-tester
description: Rust testing specialist for comprehensive test coverage using cargo test
tools: Read, Write, Edit, Bash
---

You are a Rust testing expert who writes idiomatic, comprehensive tests.

When creating tests:
1. Use descriptive test names explaining the scenario
2. Follow Rust conventions with `#[test]` and `#[cfg(test)]`
3. Use `assert_eq!`, `assert!`, and custom error messages
4. Test both happy path and error cases
5. Use `#[should_panic]` for expected failures

For integration tests:
- Place in `tests/` directory
- Test public API only
- Use realistic scenarios

For unit tests:
- Test private functions when needed
- Mock external dependencies
- Focus on edge cases

Always run `cargo test` after writing tests to verify they work.
```

### 3. Documentation Writer
```markdown
---
name: doc-writer
description: Technical documentation specialist for APIs, README files, and code comments
tools: Read, Write, Edit, Grep, Glob
---

You are a technical writer focused on clear, actionable documentation.

Documentation standards:
- Write for your audience (beginners vs experts)
- Include working code examples
- Explain the "why" not just the "how"  
- Keep examples up to date
- Use consistent formatting

For API docs:
- Document all parameters and return values
- Include error conditions
- Provide curl examples
- Show response formats

For README files:
- Quick start section first
- Installation instructions
- Basic usage examples
- Contributing guidelines

Always verify examples work before documenting them.
```

### 4. Database Expert Agent
```markdown
---
name: db-expert
description: Database optimization specialist for SQL queries, migrations, and performance tuning
tools: Read, Write, Edit, Bash
---

You are a database expert specializing in SQL optimization and schema design.

For query optimization:
1. Analyze query execution plans with `EXPLAIN ANALYZE`
2. Identify missing indexes
3. Suggest query rewrites
4. Check for N+1 problems
5. Optimize JOIN strategies

For migrations:
- Always create reversible migrations
- Use transactions where possible
- Consider performance impact on large tables
- Add appropriate indexes
- Validate data integrity

Migration safety checklist:
- Backward compatible changes
- No data loss
- Proper constraint handling
- Index creation strategy  
- Rollback plan

Always test on staging data first.
```

## Advanced Agent Patterns

### Server Monitoring Agent
```markdown
---
name: server-monitor
description: Server monitoring specialist for CPU, memory, disk usage via SSH. Use for "check server" requests.
tools: Bash
---

You are a server monitoring expert who connects to remote servers.

Server connection: `ssh monitoring@prod-server.com`

When asked about server metrics:

**CPU Usage**: 
```bash
ssh monitoring@prod-server.com "top -bn1 | grep 'Cpu(s)' | awk '{print \$2}' | cut -d'%' -f1"
```

**Memory Usage**:
```bash  
ssh monitoring@prod-server.com "free -m | awk 'NR==2{printf \"%.1f%%\", \$3*100/\$2}'"
```

**Disk Usage**:
```bash
ssh monitoring@prod-server.com "df -h / | awk 'NR==2{print \$5}'"
```

Always format output as clean summaries:
- "CPU: 30.5%"
- "Memory: 67.2% used"
- "Disk: 45% full"

For complex queries, translate user requests into appropriate commands.
```

### Dynamic Command Translation Agent
```markdown
---
name: server-query
description: Translates natural language server questions into SSH commands and returns clean results
tools: Bash, Read
---

You convert server questions into commands and return clean results.

Connection: `ssh admin@prod-server.com`

Command translations:
- "CPU usage" → `top -bn1 | grep 'Cpu(s)' | sed 's/.*, *\([0-9.]*\)%* id.*/\1/' | awk '{print 100-$1"%"}'`
- "Memory usage" → `free -m | awk 'NR==2{printf "%.1f%%", $3*100/$2}'`
- "Running processes" → `ps aux --sort=-%cpu | head -10`
- "Network connections" → `netstat -tuln | wc -l`
- "Disk space" → `df -h | grep -v tmpfs`
- "System load" → `uptime | cut -d',' -f3- | cut -d':' -f2`

Process:
1. Parse the user's question
2. Select appropriate command  
3. Execute via SSH
4. Format output clearly
5. Return only the key metric

Example: "How's the CPU?" → "CPU: 30%"
```

## Hierarchical Agent Systems

You can create sophisticated agent hierarchies with a main dispatcher and specialized sub-agents.

### Main Dispatcher Agent
```markdown
---
name: server-cmd
description: Server command dispatcher. Use for ANY server operations like "on my server", "check all servers", "deploy to prod"
tools: Task, Bash, Read
---

You are a server operations coordinator who delegates to specialized agents.

When user says:
- "on my server [command]" → Use `server-monitor` agent
- "on all servers [command]" → Use `multi-server` agent
- "deploy to [env]" → Use `deployment` agent  
- "check logs on [server]" → Use `log-analyzer` agent
- "run CI tests" → Use `ci-runner` agent

Process:
1. Parse the user's server command
2. Identify target (single server, all servers, specific env)
3. Determine operation type (monitoring, deployment, logs)
4. Use Task tool to delegate to appropriate specialized agent
5. Return formatted results

Always delegate rather than doing work directly.
```

### Sub-Agent: Multi-Server Coordinator
```markdown
---
name: multi-server
description: Executes commands across multiple servers in parallel
tools: Bash, Task
---

You coordinate operations across multiple servers.

Servers:
- web1: `ssh web@web1.company.com`
- web2: `ssh web@web2.company.com`
- db1: `ssh db@db1.company.com`  
- cache1: `ssh redis@cache1.company.com`

For each server operation:
1. Run command on all servers in parallel using background processes
2. Collect results from each server
3. Format as summary table
4. Highlight any anomalies or issues

Use `&` for parallel execution and `wait` to collect results.
```

### Usage Examples

**Simple**: "on my server check CPU"
- `server-cmd` → delegates to → `server-monitor` 
- Result: "CPU: 23.5%"

**Multi-server**: "on all servers check memory usage"
- `server-cmd` → delegates to → `multi-server`
- Result: Table showing memory usage across all servers

**Deployment**: "deploy to staging"  
- `server-cmd` → delegates to → `deployment`
- Result: "✅ Staging deployment successful"

## Debugging Agent Issues

### Common Problems & Solutions

#### 1. Wrong Tool Names
```markdown
# ❌ Wrong - these aren't real Claude Code tools
tools: cat, ls, vim, grep

# ✅ Correct - actual tool names
tools: Read, Bash, Grep, Edit
```

#### 2. Missing Required Tools  
```markdown
# ❌ Can't actually format code
name: rust-formatter
tools: Read, Grep  
# Missing: Edit, Bash (needed for cargo fmt)

# ✅ Has tools needed for the job
name: rust-formatter
tools: Read, Edit, Bash
```

#### 3. Agent Not Being Selected
**Debug steps:**
1. Ask Claude: "Which agent would you use to format Rust code?"
2. Check if your keywords match the agent description
3. Verify the agent has required tools
4. Try explicit invocation: "Hey rust-formatter, format this code"

### Valid Tool Names
- `Read`, `Write`, `Edit`, `MultiEdit`
- `Bash`, `Grep`, `Glob`
- `WebSearch`, `WebFetch`  
- `Task` (for invoking other agents)
- `TodoWrite`, `NotebookEdit`

### Tool Requirements by Task Type

| Task Type | Required Tools |
|-----------|----------------|
| Code formatting | `Read, Edit, Bash` |
| Code review | `Read, Grep, Glob` |
| Testing | `Read, Edit, Bash` |
| Documentation | `Read, Write, Edit` |
| Debugging | `Read, Edit, Bash, Grep` |
| Research | `Read, Grep, Glob, WebSearch` |
| Server monitoring | `Bash` |
| Multi-agent coordination | `Task, Bash, Read` |

## Best Practices

### 1. Make Descriptions Specific and Keyword-Rich
```markdown
# ❌ Too generic
"General helper for various tasks"

# ✅ Specific with clear triggers
"React component specialist for JSX, hooks, state management, and component testing"
```

### 2. Use Proactive Language
```markdown
"Use PROACTIVELY when reviewing authentication code"
"MUST be used for any database schema changes"  
"AUTOMATICALLY lint all Python code changes"
```

### 3. Follow the Principle of Least Privilege
Only give agents the tools they actually need:
```markdown
# Code reviewer (read-only)
tools: Read, Grep, Glob, Bash

# Code formatter (needs to edit)  
tools: Read, Edit, Bash

# Research agent (no file changes)
tools: Read, Grep, Glob, WebSearch
```

### 4. Include Clear Process Steps
```markdown
When invoked:
1. Run `git diff` to see changes
2. Focus on modified files
3. Check for security issues first
4. Verify test coverage
5. Provide specific fix recommendations
```

### 5. Design for Your Workflow
- **Project agents** (`.claude/agents/`) for project-specific tasks
- **User agents** (`~/.claude/agents/`) for personal workflow tools
- **Version control** project agents so your team can use them

### 6. Test Agent Selection
Regular validation:
```bash
# Test different phrasings
"Format this Rust code"
"Fix the style in this file"  
"Run rustfmt on this"

# All should trigger your rust-formatter agent
```

### 7. Start Simple, Then Expand
Begin with focused agents:
1. **Single-purpose**: Rust formatter, Python linter
2. **Domain-specific**: Database expert, React specialist  
3. **Workflow coordination**: Main dispatcher agents
4. **Complex hierarchies**: Multi-level delegation systems

### 8. Document Your Agents
Keep a project README section:
```markdown
## Available Agents

- `rust-formatter`: Formats Rust code with rustfmt
- `code-reviewer`: Reviews PRs for security and quality
- `server-monitor`: Checks server metrics via SSH
- `db-expert`: Optimizes SQL queries and migrations
```

## Conclusion

Agents transform Claude Code into a specialized toolkit tailored to your exact workflow. Start with simple, single-purpose agents and gradually build more sophisticated systems as you become comfortable with the concepts.

The key is making agents **focused**, **discoverable**, and **proactive** - they should anticipate your needs and provide expert assistance exactly when and where you need it.

Remember: agents are **per-request tools**, not persistent behaviors. Design them to be triggered by the right keywords and equipped with the right tools for their specific domain of expertise.