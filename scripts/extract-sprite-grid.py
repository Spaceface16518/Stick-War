#!/usr/bin/env python3
"""Flatten an AI-generated pose grid into canonical transparent frame slots."""

from __future__ import annotations

import argparse
from collections import deque
import math
from pathlib import Path

try:
    from PIL import Image, ImageOps
except ImportError as exc:  # pragma: no cover
    raise SystemExit("Pillow is required. Run with `uv run --with pillow`.") from exc


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True)
    parser.add_argument("--out-dir", required=True)
    parser.add_argument("--frames", required=True, type=int)
    parser.add_argument("--columns", required=True, type=int)
    parser.add_argument("--canonical-size", default=1024, type=int)
    parser.add_argument(
        "--mode",
        choices=("components", "grid"),
        default="components",
        help="Extract isolated alpha components when possible; fall back to grid slots.",
    )
    return parser.parse_args()


def canonicalize(slot: Image.Image, size: int) -> Image.Image:
    contained = ImageOps.contain(slot, (size, size), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    left = (size - contained.width) // 2
    top = (size - contained.height) // 2
    canvas.alpha_composite(contained, (left, top))
    return canvas


def component_boxes(image: Image.Image, expected: int, columns: int) -> list[tuple[int, int, int, int]]:
    # A modest dilation joins small antialiased gaps within a pose while the
    # generous chroma spacing keeps neighboring poses separate.
    alpha = image.getchannel("A")
    mask = alpha.point(lambda value: 255 if value > 8 else 0)
    pixels = mask.load()
    width, height = mask.size
    visited = bytearray(width * height)
    boxes: list[tuple[int, int, int, int, int]] = []

    for y in range(height):
        for x in range(width):
            index = y * width + x
            if visited[index] or pixels[x, y] == 0:
                continue
            queue: deque[tuple[int, int]] = deque([(x, y)])
            visited[index] = 1
            left = right = x
            top = bottom = y
            area = 0
            while queue:
                current_x, current_y = queue.popleft()
                area += 1
                left = min(left, current_x)
                right = max(right, current_x)
                top = min(top, current_y)
                bottom = max(bottom, current_y)
                for next_x, next_y in (
                    (current_x - 1, current_y),
                    (current_x + 1, current_y),
                    (current_x, current_y - 1),
                    (current_x, current_y + 1),
                ):
                    if not (0 <= next_x < width and 0 <= next_y < height):
                        continue
                    next_index = next_y * width + next_x
                    if visited[next_index] or pixels[next_x, next_y] == 0:
                        continue
                    visited[next_index] = 1
                    queue.append((next_x, next_y))
            if area >= 500:
                boxes.append((left, top, right + 1, bottom + 1, area))

    boxes.sort(key=lambda box: box[4], reverse=True)
    selected = boxes[:expected]
    if len(selected) != expected:
        return []

    # Preserve animation order by clustering the expected number of poses into
    # visual rows, then sorting each row from left to right.
    rows = math.ceil(expected / columns)
    selected.sort(key=lambda box: (box[1] + box[3]) / 2)
    ordered: list[tuple[int, int, int, int]] = []
    cursor = 0
    for row in range(rows):
        remaining = expected - cursor
        count = min(columns, remaining)
        row_boxes = selected[cursor : cursor + count]
        row_boxes.sort(key=lambda box: (box[0] + box[2]) / 2)
        for left, top, right, bottom, _ in row_boxes:
            padding = 8
            ordered.append(
                (
                    max(0, left - padding),
                    max(0, top - padding),
                    min(width, right + padding),
                    min(height, bottom + padding),
                )
            )
        cursor += count
    return ordered


def main() -> None:
    args = parse_args()
    if args.frames < 1 or args.columns < 1 or args.canonical_size < 1:
        raise SystemExit("Frame, column, and size values must be positive.")

    image = Image.open(args.input).convert("RGBA")
    rows = math.ceil(args.frames / args.columns)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    boxes = component_boxes(image, args.frames, args.columns) if args.mode == "components" else []
    for index in range(args.frames):
        if boxes:
            crop_box = boxes[index]
        else:
            row, column = divmod(index, args.columns)
            crop_box = (
                round(column * image.width / args.columns),
                round(row * image.height / rows),
                round((column + 1) * image.width / args.columns),
                round((row + 1) * image.height / rows),
            )
        frame = canonicalize(image.crop(crop_box), args.canonical_size)
        frame.save(out_dir / f"{index + 1:02d}.png", optimize=True)


if __name__ == "__main__":
    main()
