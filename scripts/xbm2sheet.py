#!/usr/bin/env python3
"""Convert the original oneko XBM bitmaps + masks into 256x128 RGBA PNG sprite sheets.

Usage: xbm2sheet.py <oneko-source-dir> <out-dir>

Layout matches oneko.js (8 cols x 4 rows of 32px): see CELLS below.
Pure Python, no PIL needed.
"""
import os, re, struct, sys, zlib

CELL, COLS, ROWS = 32, 8, 4
# (col, row) -> original frame name
CELLS = {
    (3, 3): "mati2",  # idle
    (7, 3): "awake",  # alert
    (3, 2): "mati3",  # tired / yawn
    (2, 0): "sleep1", (2, 1): "sleep2",
    (5, 0): "kaki1", (6, 0): "kaki2", (7, 0): "jare2",  # scratch self
    (0, 0): "utogi1", (0, 1): "utogi2",  # scratch wall N
    (7, 1): "dtogi1", (6, 2): "dtogi2",  # S
    (2, 2): "rtogi1", (2, 3): "rtogi2",  # E
    (4, 0): "ltogi1", (4, 1): "ltogi2",  # W
    (1, 2): "up1", (1, 3): "up2",
    (0, 2): "upright1", (0, 3): "upright2",
    (3, 0): "right1", (3, 1): "right2",
    (5, 1): "dwright1", (5, 2): "dwright2",
    (6, 3): "down1", (7, 2): "down2",
    (5, 3): "dwleft1", (6, 1): "dwleft2",
    (4, 2): "left1", (4, 3): "left2",
    (1, 0): "upleft1", (1, 1): "upleft2",
}
assert len(CELLS) == COLS * ROWS

# character -> (bitmap dir, bitmap suffix, mask dir, mask suffix)
CHARACTERS = {
    "neko": ("neko", "", "neko", "_mask"),
    "tora": ("tora", "_tora", "neko", "_mask"),  # tora reuses neko's masks (see oneko.c)
    "dog": ("dog", "_dog", "dog", "_dog_mask"),
    "sakura": ("sakura", "_sakura", "sakura", "_sakura_mask"),
    "tomoyo": ("tomoyo", "_tomoyo", "tomoyo", "_tomoyo_mask"),
}


def read_xbm(path):
    text = open(path, encoding="latin-1").read()
    w = int(re.search(r"_width\s+(\d+)", text).group(1))
    h = int(re.search(r"_height\s+(\d+)", text).group(1))
    data = re.search(r"\{(.*)\}", text, re.S).group(1)
    bytes_ = [int(b, 16) for b in re.findall(r"0x[0-9a-fA-F]+", data)]
    stride = (w + 7) // 8
    assert len(bytes_) >= stride * h, path
    bits = [[(bytes_[y * stride + x // 8] >> (x % 8)) & 1 for x in range(w)] for y in range(h)]
    return w, h, bits


def write_png(path, w, h, rgba):
    raw = b"".join(b"\x00" + bytes(rgba[y * w * 4:(y + 1) * w * 4]) for y in range(h))

    def chunk(tag, body):
        c = tag + body
        return struct.pack(">I", len(body)) + c + struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)

    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(raw, 9))
    png += chunk(b"IEND", b"")
    open(path, "wb").write(png)


def main(src, out):
    W, H = COLS * CELL, ROWS * CELL
    for name, (bdir, bsuf, mdir, msuf) in CHARACTERS.items():
        sheet = bytearray(W * H * 4)
        for (col, row), frame in CELLS.items():
            bw, bh, bits = read_xbm(os.path.join(src, "bitmaps", bdir, f"{frame}{bsuf}.xbm"))
            mw, mh, mask = read_xbm(os.path.join(src, "bitmasks", mdir, f"{frame}{msuf}.xbm"))
            assert (bw, bh, mw, mh) == (CELL,) * 4, (name, frame)
            for y in range(CELL):
                for x in range(CELL):
                    if not mask[y][x]:
                        continue
                    v = 0 if bits[y][x] else 255
                    i = ((row * CELL + y) * W + col * CELL + x) * 4
                    sheet[i:i + 4] = bytes((v, v, v, 255))
        write_png(os.path.join(out, f"{name}.png"), W, H, sheet)
        print(f"wrote {name}.png")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
