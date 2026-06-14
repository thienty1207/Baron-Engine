# Superpowers Workflow Routing Index

Use this file to choose one Superpowers workflow skill. Superpowers controls process discipline; specialist knowledge should come from repo context, current docs, or targeted subagents.

## Routing

| Task shape | Load |
| --- | --- |
| Feature or bugfix, refactor, or behavior change | `test-driven-development/SKILL.md` |
| Unexpected behavior or failing test | `systematic-debugging/SKILL.md` |
| Multi-step requirements need an implementation plan | `writing-plans/SKILL.md` |
| Executing an existing written plan | `executing-plans/SKILL.md` |
| Independent tasks can run in the same session | `subagent-driven-development/SKILL.md` |
| Need isolated feature workspace | `using-git-worktrees/SKILL.md` |
| Completing major work or preparing integration | `finishing-a-development-branch/SKILL.md` |
| Before claiming completion, fixed, or passing | `verification-before-completion/SKILL.md` |
| Requesting review before merge or after major change | `requesting-code-review/SKILL.md` |
| Receiving review feedback | `receiving-code-review/SKILL.md` |
| Creating or editing skills | `writing-skills/SKILL.md` |
| Starting a conversation in a Superpowers-native agent | `using-superpowers/SKILL.md` |

## Priority

- Use Superpowers first when the task involves planning, debugging, coding, review, verification, or branch finishing.
- Follow the coding discipline guardrails in `AGENTS.md` for simplicity, surgical edits, assumptions, and verification criteria.
- Load another skill only when it is installed locally and `.codex/skills/INDEX.md` routes to it.
- Do not load every Superpowers skill. Pick the one row that matches the immediate workflow.

## Included Skills

- brainstorming
- dispatching-parallel-agents
- executing-plans
- finishing-a-development-branch
- receiving-code-review
- requesting-code-review
- subagent-driven-development
- systematic-debugging
- test-driven-development
- using-git-worktrees
- using-superpowers
- verification-before-completion
- writing-plans
- writing-skills

The upstream attribution readme is source attribution only; do not load it during normal task execution.
