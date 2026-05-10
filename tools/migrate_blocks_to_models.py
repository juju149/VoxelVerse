"""Mechanical migration of *.block.ron files to the RawBlockModel architecture.

Reads each block file in assets/packs/core/defs/blocks/, derives:
- the appropriate model (core:block_model/<name>)
- the materials map matching the model's face_layers
and writes back the migrated file.

Run from repo root: `python3 tools/migrate_blocks_to_models.py`
"""

from __future__ import annotations
import re
from pathlib import Path

ROOT = Path("assets/packs/core/defs/blocks")

# Regexes anchored on the legacy fields. We don't try to be a full RON parser
# because we know the exact layout produced by the previous schema.
SHAPE_RE = re.compile(r"^\s*shape:\s*(\w+)\s*,\s*$", re.MULTILINE)
COLLISION_RE = re.compile(r"^\s*collision:\s*(\w+)\s*,\s*$", re.MULTILINE)
AO_RE = re.compile(r"^\s*ambient_occlusion:\s*\w+\s*,\s*\n", re.MULTILINE)
SHADOW_RE = re.compile(r"^\s*casts_shadow:\s*\w+\s*,\s*\n", re.MULTILINE)
MAT_NONE_RE = re.compile(r"^\s*materials:\s*None\s*,\s*$", re.MULTILINE)
MAT_ALL_RE = re.compile(r'^\s*materials:\s*All\("([^"]+)"\)\s*,\s*$', re.MULTILINE)
MAT_FACES_RE = re.compile(
    r'^\s*materials:\s*Faces\(\(top:\s*"([^"]+)"\s*,\s*sides:\s*"([^"]+)"\s*,\s*bottom:\s*"([^"]+)"\)\)\s*,\s*$',
    re.MULTILINE,
)


def model_for(shape: str, collision: str) -> str:
    if shape == "none":
        return "core:block_model/air"
    if shape == "cross_plane":
        return "core:block_model/cross_plane"
    if shape == "cube":
        if collision == "leaf_volume":
            return "core:block_model/leaf_block"
        if collision == "soft_cube":
            return "core:block_model/soft_cube"
        return "core:block_model/cube"
    raise ValueError(f"unknown shape: {shape}")


def materials_block(model_key: str, mat_kind: str, payload) -> str:
    """Returns the new materials entry as a string."""
    if model_key == "core:block_model/air":
        return "        materials: {},\n"
    if model_key == "core:block_model/cross_plane":
        assert mat_kind == "all"
        return f'        materials: {{\n            "plane": "{payload}",\n        }},\n'
    if model_key == "core:block_model/cube_column":
        # Only used for logs (Faces with top == bottom).
        top, sides, bottom = payload
        assert top == bottom, "cube_column requires top == bottom"
        return (
            "        materials: {\n"
            f'            "end": "{top}",\n'
            f'            "side": "{sides}",\n'
            "        },\n"
        )
    # Cube / leaf_block / soft_cube — all use the 6-slot py/ny/pz/nz/px/nx convention.
    if mat_kind == "all":
        m = payload
        return (
            "        materials: {\n"
            f'            "py": "{m}",\n'
            f'            "ny": "{m}",\n'
            f'            "pz": "{m}",\n'
            f'            "nz": "{m}",\n'
            f'            "px": "{m}",\n'
            f'            "nx": "{m}",\n'
            "        },\n"
        )
    if mat_kind == "faces":
        top, sides, bottom = payload
        return (
            "        materials: {\n"
            f'            "py": "{top}",\n'
            f'            "ny": "{bottom}",\n'
            f'            "pz": "{sides}",\n'
            f'            "nz": "{sides}",\n'
            f'            "px": "{sides}",\n'
            f'            "nx": "{sides}",\n'
            "        },\n"
        )
    raise ValueError(mat_kind)


def migrate(path: Path) -> None:
    text = path.read_text(encoding="utf-8-sig")  # strip BOM if present

    shape_m = SHAPE_RE.search(text)
    collision_m = COLLISION_RE.search(text)
    if not shape_m or not collision_m:
        print(f"SKIP {path}: shape/collision not found")
        return

    shape = shape_m.group(1)
    collision = collision_m.group(1)

    if MAT_NONE_RE.search(text):
        mat_kind, payload = "none", None
        # Logs and natural blocks shouldn't be `None`. Air is.
    elif (m := MAT_ALL_RE.search(text)):
        mat_kind, payload = "all", m.group(1)
    elif (m := MAT_FACES_RE.search(text)):
        mat_kind, payload = "faces", (m.group(1), m.group(2), m.group(3))
    else:
        print(f"SKIP {path}: materials pattern not recognised")
        return

    # Special model resolution for cube_column (logs): faces with top == bottom.
    base_model = model_for(shape, collision)
    if (
        mat_kind == "faces"
        and base_model == "core:block_model/cube"
        and payload[0] == payload[2]  # top == bottom
        and "/logs/" in str(path).replace("\\", "/")
    ):
        base_model = "core:block_model/cube_column"

    new_materials_text = materials_block(base_model, mat_kind, payload)

    # Remove dead fields
    text = SHAPE_RE.sub("", text)
    text = COLLISION_RE.sub("", text)
    text = AO_RE.sub("", text)
    text = SHADOW_RE.sub("", text)

    # Replace materials line(s)
    if MAT_NONE_RE.search(text):
        text = MAT_NONE_RE.sub(new_materials_text.rstrip("\n"), text, count=1)
    elif MAT_ALL_RE.search(text):
        text = MAT_ALL_RE.sub(new_materials_text.rstrip("\n"), text, count=1)
    elif MAT_FACES_RE.search(text):
        text = MAT_FACES_RE.sub(new_materials_text.rstrip("\n"), text, count=1)

    # Insert `format_version: 1,` and `model: "..."` right after `BlockDef(`.
    text = re.sub(
        r"^BlockDef\(\s*\n",
        f'BlockDef(\n    format_version: 1,\n    model: "{base_model}",\n',
        text,
        count=1,
    )

    # Move `model:` and `format_version:` up just under display_name for readability.
    # Also clean up extra blank lines.
    lines = [ln for ln in text.splitlines()]
    # Collapse triple+ blank lines into a single blank.
    cleaned = []
    blank = 0
    for ln in lines:
        if ln.strip() == "":
            blank += 1
            if blank > 1:
                continue
        else:
            blank = 0
        cleaned.append(ln)
    # Strip trailing blank lines.
    while cleaned and cleaned[-1].strip() == "":
        cleaned.pop()
    text = "\n".join(cleaned) + "\n"

    path.write_text(text, encoding="utf-8")
    print(f"  migrated  {path} -> {base_model} ({mat_kind})")


def main() -> int:
    files = sorted(ROOT.rglob("*.block.ron"))
    print(f"Found {len(files)} block files under {ROOT}")
    for f in files:
        migrate(f)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
