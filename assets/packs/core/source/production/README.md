# source/production

This folder is for **humans and AI agents**, not for the runtime.

Anything under `source/production/` is design material:

- `core_pack_mvp.md` - the minimum content the core pack must ship.
- `briefs/` - per-asset design briefs and AI prompts.
- `checklists/` - per-asset checklists used during authoring and review.
- `draft/` - work-in-progress content whose schema is not yet stable.
- `allowed_unused.ron` - declared exceptions for orphan content.

The pack loader does not parse anything in here. The compiler does not parse
anything in here. The runtime never sees anything in here. Pack Doctor reads
`allowed_unused.ron` to suppress justified warnings - that is the only
machine-visible file in this folder.

See [`docs/content_pipeline.md`](../../../../../docs/CONTENT_PIPELINE.md) and
[`docs/content_rules.md`](../../../../../docs/content_rules.md).
