#!/usr/bin/env python3
"""Generate app icons from the idle neko sprite.

Writes assets/icon-1024.png, assets/icon.ico and (on macOS) assets/AppIcon.icns.
Needs Pillow; iconutil is used for the icns.
"""
import os, shutil, subprocess, sys, tempfile
from PIL import Image, ImageDraw

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS = os.path.join(ROOT, "assets")
SIZE = 1024
BG = (247, 231, 196, 255)       # warm cream
BG_SHADE = (214, 190, 146, 255) # bottom edge
IDLE = (3, 3)                   # idle cell in the sheet (col, row)


def sprite():
    sheet = Image.open(os.path.join(ASSETS, "neko.png")).convert("RGBA")
    c, r = IDLE
    return sheet.crop((c * 32, r * 32, c * 32 + 32, r * 32 + 32))


def base():
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # macOS-style rounded square, inset ~10% like Apple's template
    inset = int(SIZE * 0.1)
    radius = int(SIZE * 0.18)
    d.rounded_rectangle((inset, inset, SIZE - inset, SIZE - inset), radius, fill=BG_SHADE)
    d.rounded_rectangle((inset, inset, SIZE - inset, SIZE - inset - int(SIZE * 0.02)), radius, fill=BG)
    cat = sprite()
    # trim transparent border so the cat fills the square nicely
    bbox = cat.getbbox()
    cat = cat.crop(bbox)
    scale = int((SIZE - 2 * inset) * 0.72 / max(cat.size))
    cat = cat.resize((cat.width * scale, cat.height * scale), Image.NEAREST)
    x = (SIZE - cat.width) // 2
    y = (SIZE - cat.height) // 2 + int(SIZE * 0.01)
    img.alpha_composite(cat, (x, y))
    return img


def main():
    img = base()
    img.save(os.path.join(ASSETS, "icon-1024.png"))

    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    img.save(os.path.join(ASSETS, "icon.ico"), sizes=[(s, s) for s in ico_sizes])

    if sys.platform == "darwin" and shutil.which("iconutil"):
        with tempfile.TemporaryDirectory() as tmp:
            iconset = os.path.join(tmp, "AppIcon.iconset")
            os.mkdir(iconset)
            for s in [16, 32, 128, 256, 512]:
                img.resize((s, s), Image.LANCZOS).save(os.path.join(iconset, f"icon_{s}x{s}.png"))
                img.resize((s * 2, s * 2), Image.LANCZOS).save(os.path.join(iconset, f"icon_{s}x{s}@2x.png"))
            subprocess.run(["iconutil", "-c", "icns", iconset, "-o", os.path.join(ASSETS, "AppIcon.icns")], check=True)
    print("icons written to", ASSETS)


if __name__ == "__main__":
    main()
