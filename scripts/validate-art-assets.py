#!/usr/bin/env python3
"""Validate shipped animation strips and their exact-frame GIF previews."""

from __future__ import annotations

import json
from pathlib import Path

try:
    from PIL import Image
except ImportError as exc:  # pragma: no cover
    raise SystemExit("Pillow is required. Run with `uv run --with pillow`.") from exc


def main() -> None:
    manifest = json.loads(Path("art/animation-plan.json").read_text())
    size = int(manifest["frame_size"])
    checked = 0
    for team in ("blue", "red"):
        for character, data in manifest["characters"].items():
            for animation, spec in data["animations"].items():
                frames = int(spec["frames"])
                atlas_path = Path("assets/characters") / team / character / f"{animation}.png"
                gif_path = Path("docs/animation-previews") / team / character / f"{animation}.gif"
                with Image.open(atlas_path) as atlas:
                    assert atlas.mode == "RGBA", f"{atlas_path}: expected RGBA, got {atlas.mode}"
                    assert atlas.size == (size * frames, size), (
                        f"{atlas_path}: expected {(size * frames, size)}, got {atlas.size}"
                    )
                    assert atlas.getchannel("A").getbbox(), f"{atlas_path}: atlas is transparent"
                with Image.open(gif_path) as preview:
                    assert preview.n_frames == frames, (
                        f"{gif_path}: expected {frames} frames, got {preview.n_frames}"
                    )
                    expected_duration = round((1000 / int(spec["fps"])) / 10) * 10
                    durations = {
                        preview.seek(index) or preview.info.get("duration")
                        for index in range(preview.n_frames)
                    }
                    assert durations == {expected_duration}, (
                        f"{gif_path}: expected {expected_duration}ms frames, got {durations}"
                    )
                checked += 1
    print(f"Validated {checked} atlases and {checked} exact-frame GIF previews.")


if __name__ == "__main__":
    main()
