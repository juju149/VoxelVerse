# Texture Checklist

Use this list when authoring or reviewing a texture.

Reference rules: [`docs/content_rules.md`](../../../../../../docs/content_rules.md)
section 7 (textures) and [`content_pipeline.md`](../../../../../../docs/CONTENT_PIPELINE.md)
section 5.

---

## 1. Brief

- [ ] A prompt or visual brief is captured in
      `source/production/briefs/<stem>.md` (mandatory for AI-generated
      textures so future agents can iterate; optional for purely manual work).

## 2. File

- [ ] PNG, RGBA or RGB.
- [ ] **256x256** for block surface textures unless documented otherwise.
- [ ] Filename is lowercase, snake_case, ends with `_albedo.png`,
      `_normal.png`, or `_roughness.png` for material-driven textures.

## 3. Naming

- [ ] Path is `media/textures/<category>/<material_stem>/<file>.png`.
- [ ] Filename includes the role (face, map) when relevant:
      `oak_log_side_albedo.png`, not `oak_log_side1.png`.

## 4. Consistency

- [ ] Albedo, normal, and roughness depict the same surface.
- [ ] The visual style matches the VoxelVerse art direction
      (stylized 2D, clean edges, no photorealistic detail).

## 5. Usage

- [ ] At least one material under `defs/materials/` references this texture,
      **or** the texture is listed in
      `source/production/allowed_unused.ron`.

## 6. Validation

- [ ] `tools/validate_content.ps1` passes.
- [ ] `tools/pack_doctor.ps1` passes; the texture is not in
      `unused.textures` (unless intentionally allowed) and not in
      `missing.textures`.

If any box stays unchecked, the texture is not done.
