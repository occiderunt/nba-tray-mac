#!/usr/bin/env python3
"""Draw the NBA Tracker app icon and menu-bar template icon (no deps)."""
import math
import struct
import zlib
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "src-tauri" / "icons"
SS = 4  # supersampling factor


def write_png(path, width, height, pixels):
    raw = b"".join(b"\x00" + bytes(row) for row in pixels)

    def chunk(tag, data):
        body = tag + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )
    path.write_bytes(png)


def rounded_square(x, y, inset, radius):
    lo, hi = inset, 1 - inset
    if not (lo <= x <= hi and lo <= y <= hi):
        return False
    cx = min(max(x, lo + radius), hi - radius)
    cy = min(max(y, lo + radius), hi - radius)
    return (x - cx) ** 2 + (y - cy) ** 2 <= radius ** 2


def ball_pixel(x, y, radius, seam_width):
    """Return 'ball', 'seam' or None for a unit-square coordinate."""
    if (x - 0.5) ** 2 + (y - 0.5) ** 2 > radius ** 2:
        return None

    dx, dy = x - 0.5, y - 0.5
    seams = [
        abs(dx) < seam_width,
        abs(dy) < seam_width,
        abs(math.hypot(dx - radius * 1.0, dy) - radius * 0.62) < seam_width,
        abs(math.hypot(dx + radius * 1.0, dy) - radius * 0.62) < seam_width,
    ]
    if any(seams):
        return "seam"
    return "ball"


def render(size, background, ball, seam, radius=0.34, seam_width=0.014, rounded=None, seam_alpha_zero=False):
    width = height = size
    big = size * SS
    rows = []
    for py in range(height):
        row = []
        for px in range(width):
            r = g = b = a = 0.0
            for sy in range(SS):
                for sx in range(SS):
                    u = (px * SS + sx + 0.5) / big
                    v = (py * SS + sy + 0.5) / big
                    kind = ball_pixel(u, v, radius, seam_width)
                    if kind is None:
                        if background is not None and rounded and rounded_square(u, v, 0.06, 0.22):
                            cr, cg, cb = background
                            ca = 1.0
                        else:
                            cr = cg = cb = ca = 0.0
                    elif kind == "seam":
                        cr, cg, cb, ca = (*seam, 0.0 if seam_alpha_zero else 1.0)
                    else:
                        cr, cg, cb, ca = (*ball, 1.0)
                    r += cr * ca
                    g += cg * ca
                    b += cb * ca
                    a += ca
            n = SS * SS
            alpha = a / n
            if alpha > 0:
                row.extend(
                    [
                        round(r / a * 255),
                        round(g / a * 255),
                        round(b / a * 255),
                        round(alpha * 255),
                    ]
                )
            else:
                row.extend([0, 0, 0, 0])
        rows.append(bytes(row))
    return rows


def main():
    OUT.mkdir(parents=True, exist_ok=True)

    cream = (0.957, 0.922, 0.851)  # #F4EBD9
    orange = (0.886, 0.400, 0.184)  # #E2662F
    ink = (0.090, 0.078, 0.059)  # #17140F

    write_png(
        OUT / "icon.png",
        1024,
        1024,
        render(1024, cream, orange, ink, radius=0.345, seam_width=0.011, rounded=True),
    )

    # macOS menu bar template: alpha only, seams punched through
    write_png(
        OUT / "tray.png",
        88,
        88,
        render(88, None, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), radius=0.42, seam_width=0.010, seam_alpha_zero=True),
    )
    print("wrote icon.png and tray.png")


if __name__ == "__main__":
    main()
