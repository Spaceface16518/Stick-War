#!/usr/bin/env python3
"""Normalize, recolor, pack, and preview every production character animation."""

from __future__ import annotations

import argparse
import colorsys
import json
from pathlib import Path

try:
    from PIL import Image, ImageDraw
except ImportError as exc:  # pragma: no cover
    raise SystemExit("Pillow is required. Run with `uv run --with pillow`.") from exc


ALPHA_THRESHOLD = 8
CONTENT_PADDING = 18
PREVIEW_BACKGROUND = (228, 216, 188, 255)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", default="art/animation-plan.json")
    parser.add_argument("--work-root", default="art/work/animations/blue")
    parser.add_argument("--assets-root", default="assets/characters")
    parser.add_argument("--normalized-root", default="art/work/normalized")
    parser.add_argument("--preview-root", default="docs/animation-previews")
    return parser.parse_args()


def content_bbox(image: Image.Image) -> tuple[int, int, int, int] | None:
    alpha = image.getchannel("A").point(lambda value: 255 if value > ALPHA_THRESHOLD else 0)
    return alpha.getbbox()


def crop_content(image: Image.Image) -> Image.Image | None:
    bbox = content_bbox(image)
    return image.crop(bbox) if bbox else None


def load_frames(root: Path, character: str, animation: str, count: int) -> list[Image.Image]:
    frame_dir = root / character / animation / "frames"
    paths = [frame_dir / f"{index:02d}.png" for index in range(1, count + 1)]
    missing = [str(path) for path in paths if not path.exists()]
    if missing:
        raise SystemExit("Missing extracted frames:\n" + "\n".join(missing))
    return [Image.open(path).convert("RGBA") for path in paths]


def normalize_frame(content: Image.Image | None, frame_size: int, scale: float) -> Image.Image:
    canvas = Image.new("RGBA", (frame_size, frame_size), (0, 0, 0, 0))
    if content is None:
        return canvas
    width = max(1, round(content.width * scale))
    height = max(1, round(content.height * scale))
    resized = content.resize((width, height), Image.Resampling.LANCZOS)
    canvas.alpha_composite(resized, ((frame_size - width) // 2, frame_size - CONTENT_PADDING - height))
    return canvas


def recolor_blue_to_red(image: Image.Image) -> Image.Image:
    result = image.copy()
    pixels = result.load()
    for y in range(result.height):
        for x in range(result.width):
            red, green, blue, alpha = pixels[x, y]
            if alpha == 0:
                continue
            hue, saturation, value = colorsys.rgb_to_hsv(red / 255, green / 255, blue / 255)
            degrees = hue * 360
            if 190 <= degrees <= 250 and saturation >= 0.28 and blue > red * 1.08:
                target_hue = 356 / 360
                target_saturation = min(1.0, max(0.55, saturation * 1.08))
                target_value = min(1.0, value * 1.02)
                nr, ng, nb = colorsys.hsv_to_rgb(target_hue, target_saturation, target_value)
                pixels[x, y] = (round(nr * 255), round(ng * 255), round(nb * 255), alpha)
    return result


def pack_strip(frames: list[Image.Image], frame_size: int) -> Image.Image:
    strip = Image.new("RGBA", (frame_size * len(frames), frame_size), (0, 0, 0, 0))
    for index, frame in enumerate(frames):
        strip.alpha_composite(frame, (index * frame_size, 0))
    return strip


def preview_frame(frame: Image.Image) -> Image.Image:
    canvas = Image.new("RGBA", frame.size, PREVIEW_BACKGROUND)
    draw = ImageDraw.Draw(canvas)
    tile = 24
    alternate = (214, 202, 176, 255)
    for top in range(0, frame.height, tile):
        for left in range(0, frame.width, tile):
            if (left // tile + top // tile) % 2:
                draw.rectangle((left, top, left + tile, top + tile), fill=alternate)
    canvas.alpha_composite(frame)
    return canvas.convert("P", palette=Image.Palette.ADAPTIVE)


def save_gif(frames: list[Image.Image], fps: int, path: Path) -> None:
    previews = [preview_frame(frame) for frame in frames]
    path.parent.mkdir(parents=True, exist_ok=True)
    previews[0].save(
        path,
        save_all=True,
        append_images=previews[1:],
        duration=round(1000 / fps),
        loop=0,
        disposal=2,
        optimize=False,
    )


def main() -> None:
    args = parse_args()
    manifest = json.loads(Path(args.manifest).read_text())
    frame_size = int(manifest["frame_size"])
    work_root = Path(args.work_root)
    assets_root = Path(args.assets_root)
    normalized_root = Path(args.normalized_root)
    preview_root = Path(args.preview_root)

    for character, character_data in manifest["characters"].items():
        raw_by_animation: dict[str, list[Image.Image]] = {}
        all_content: list[Image.Image] = []
        for animation, spec in character_data["animations"].items():
            frames = load_frames(work_root, character, animation, int(spec["frames"]))
            raw_by_animation[animation] = frames
            all_content.extend(content for frame in frames if (content := crop_content(frame)))

        if not all_content:
            raise SystemExit(f"No visible content found for {character}.")
        max_width = max(content.width for content in all_content)
        max_height = max(content.height for content in all_content)
        available = frame_size - CONTENT_PADDING * 2
        scale = min(available / max_width, available / max_height)

        for animation, spec in character_data["animations"].items():
            blue_frames = [normalize_frame(crop_content(frame), frame_size, scale) for frame in raw_by_animation[animation]]
            red_frames = [recolor_blue_to_red(frame) for frame in blue_frames]

            for team, frames in (("blue", blue_frames), ("red", red_frames)):
                frame_dir = normalized_root / team / character / animation
                frame_dir.mkdir(parents=True, exist_ok=True)
                for index, frame in enumerate(frames, start=1):
                    frame.save(frame_dir / f"{index:02d}.png", optimize=True)

                strip_path = assets_root / team / character / f"{animation}.png"
                strip_path.parent.mkdir(parents=True, exist_ok=True)
                pack_strip(frames, frame_size).save(strip_path, optimize=True)
                save_gif(
                    frames,
                    int(spec["fps"]),
                    preview_root / team / character / f"{animation}.gif",
                )


if __name__ == "__main__":
    main()
