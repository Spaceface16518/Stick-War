#!/usr/bin/env python3
"""Turn approved chroma-key references into normalized transparent game assets."""

from __future__ import annotations

import argparse
import colorsys
from pathlib import Path

try:
    from PIL import Image
except ImportError as exc:  # pragma: no cover
    raise SystemExit("Pillow is required. Run with `uv run --with pillow`.") from exc


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--statue", required=True)
    parser.add_argument("--gold", required=True)
    parser.add_argument("--output", default="assets/props")
    return parser.parse_args()


def remove_green(image: Image.Image) -> Image.Image:
    result = image.convert("RGBA")
    pixels = result.load()
    for y in range(result.height):
        for x in range(result.width):
            red, green, blue, _ = pixels[x, y]
            dominance = green - max(red, blue)
            if dominance >= 8 and green >= 45:
                # A green-screen composite adds roughly `(1 - alpha) * 255`
                # to the green channel. Recover that coverage and un-premultiply
                # the foreground so fine ink edges stay dark instead of green.
                alpha = max(0, 255 - dominance)
                if alpha == 0:
                    pixels[x, y] = (0, 0, 0, 0)
                    continue
                scale = 255 / alpha
                red = min(255, round(red * scale))
                blue = min(255, round(blue * scale))
                green = max(red, blue)
                pixels[x, y] = (red, green, blue, alpha)
    return result


def normalize(image: Image.Image, size: tuple[int, int], padding: int = 12) -> Image.Image:
    bbox = image.getchannel("A").getbbox()
    if bbox is None:
        raise SystemExit("No visible content remained after chroma-key removal.")
    content = image.crop(bbox)
    scale = min((size[0] - padding * 2) / content.width, (size[1] - padding * 2) / content.height)
    resized = content.resize(
        (max(1, round(content.width * scale)), max(1, round(content.height * scale))),
        Image.Resampling.LANCZOS,
    )
    canvas = Image.new("RGBA", size, (0, 0, 0, 0))
    canvas.alpha_composite(resized, ((size[0] - resized.width) // 2, size[1] - padding - resized.height))
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
            if 190 <= hue * 360 <= 250 and saturation >= 0.28 and blue > red * 1.08:
                nr, ng, nb = colorsys.hsv_to_rgb(356 / 360, min(1.0, max(0.55, saturation * 1.08)), min(1.0, value * 1.02))
                pixels[x, y] = (round(nr * 255), round(ng * 255), round(nb * 255), alpha)
    return result


def main() -> None:
    args = parse_args()
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    statue = normalize(remove_green(Image.open(args.statue)), (512, 768))
    statue.save(output / "statue-blue.png", optimize=True)
    recolor_blue_to_red(statue).save(output / "statue-red.png", optimize=True)

    gold = normalize(remove_green(Image.open(args.gold)), (768, 384))
    gold.save(output / "gold-deposit.png", optimize=True)


if __name__ == "__main__":
    main()
