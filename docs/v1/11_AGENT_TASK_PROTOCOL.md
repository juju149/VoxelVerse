# VoxelVerse V1 Agent Task Protocol

This file is a copy-paste protocol for AI coding agents.
Use it to turn a vague request into safe implementation work.

## Protocol header

Before coding, prepare:

```md
Task:
Phase:
Player value:
Technical value:
Files read:
Files to change:
Files to delete:
Tests to run:
Manual smoke test:
Stop conditions:
```

## Standard prompt for coding agent

```md
You are coding VoxelVerse V1.
Read `AGENTS.md` and all relevant `docs/v1/*.md` first.
This project has no released V1, so do not preserve legacy, compatibility, aliases, dead code, or duplicate systems.
Before changing code, inspect existing files and direct callers.
Implement the smallest complete step for the current roadmap phase.
Do not move to the next phase until the current gate is perfect.
Run formatting, clippy, tests and relevant game/content validation.
Report changed files, deleted code, validation, manual test, risks and next step.
```

## Refactor prompt

```md
Refactor the target system toward VoxelVerse V1 architecture.
Do not add behavior unless needed to preserve current gameplay.
Delete old paths instead of wrapping them.
Split files by responsibility.
Keep public APIs narrow.
Add or update tests.
Reject runtime fallbacks that hide invalid content.
```

## Performance prompt

```md
Analyze and improve performance for the target system.
Use diagnostics and budgets, not vibes.
Identify CPU hot paths, GPU upload spikes, unbounded queues, overdraw, draw calls, allocations and duplicated work.
Implement the smallest fix that measurably improves stability.
Do not reduce visual quality unless the quality profile explicitly allows it.
```

## Gameplay feel prompt

```md
Improve game feel for the target action.
Preserve architecture boundaries: gameplay emits semantic feedback events, renderer animates, audio plays mapped sounds, UI shows notices.
Make the action responsive, readable and satisfying.
Tune with named constants or data fields.
Add tests for rules and manual smoke instructions for feel.
```

## Pack/content prompt

```md
Improve VoxelVerse V1 content pipeline or content definitions.
`.object.ron` is canonical for gameplay objects.
World placement rules stay under `defs/world`.
Do not preserve old folder layouts for compatibility.
Make Pack Doctor stricter and errors clearer.
The runtime must not repair broken content.
```

## Final response template

```md
## Done

## Files changed

## Deleted

## Validation

## Manual smoke test

## Risks

## Next step
```

## Agent self-check

Before final answer, verify:

- Did I read before editing?
- Did I remove old paths?
- Did I avoid duplicate systems?
- Did I keep files small?
- Did I run the right commands?
- Did I improve the V1 target?
- Did I leave the next step obvious?
