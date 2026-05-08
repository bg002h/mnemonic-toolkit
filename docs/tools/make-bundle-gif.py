#!/usr/bin/env python3
"""Generate `docs/m-format-bundle-demo.gif`: a typing animation of the
canonical `mnemonic bundle` invocation + its exact captured output.

Run from anywhere in the repo:

    python3 docs/tools/make-bundle-gif.py

The script also runs through ImageMagick at the end to optimize the
GIF in place. Requires:

    - python3 + Pillow
    - DejaVu Sans Mono (or override FONT_PATH below)
    - magick (ImageMagick 7) on PATH for the optimization pass; the
      script still produces a working but larger GIF without it.

Inputs (read from the repo, never modified):
    docs/manual/transcripts/22-first-bundle.out

Output:
    docs/m-format-bundle-demo.gif
"""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

REPO = Path(__file__).resolve().parent.parent.parent
OUT_PATH = REPO / "docs" / "m-format-bundle-demo.gif"
TRANSCRIPT = REPO / "docs/manual/transcripts/22-first-bundle.out"

# ---- Terminal styling ------------------------------------------------------

COLS, ROWS = 90, 40
FONT_PATH = "/usr/share/fonts/TTF/DejaVuSansMono.ttf"
FONT_SIZE = 13
PADDING = 10

BG_COLOR = (30, 30, 30)
FG_COLOR = (212, 212, 212)
PROMPT_COLOR = (86, 156, 214)
COMMENT_COLOR = (106, 153, 85)
WARN_COLOR = (215, 186, 125)

font = ImageFont.truetype(FONT_PATH, FONT_SIZE)

ascent, descent = font.getmetrics()
CELL_H = ascent + descent
bbox = font.getbbox("M")
CELL_W = bbox[2] - bbox[0]

W = COLS * CELL_W + 2 * PADDING
H = ROWS * CELL_H + 2 * PADDING

# ---- Inputs ---------------------------------------------------------------

CMD_LINES = [
    'mnemonic bundle \\',
    '    --network mainnet \\',
    '    --template bip84 \\',
    '    --slot @0.phrase="abandon abandon abandon abandon abandon abandon \\',
    '                        abandon abandon abandon abandon abandon about"',
]

OUTPUT = TRANSCRIPT.read_text()

# ---- Frame rendering ------------------------------------------------------


def color_for(line: str) -> tuple:
    if line.startswith("#"):
        return COMMENT_COLOR
    if line.startswith("warning:"):
        return WARN_COLOR
    return FG_COLOR


def render(
    typed_cmd: str,
    output_text: str,
    show_cursor: bool,
    end_prompt: bool,
    end_cursor: bool = True,
) -> Image.Image:
    """Render a single frame.

    `typed_cmd`   — the user-typed command so far (may be partial).
    `output_text` — the binary's output so far (may be partial; empty if cmd
                    not yet submitted).
    `show_cursor` — draw a block cursor at the end of the typed area while
                    typing the command.
    `end_prompt`  — draw a final `$ ` prompt under the output (post-completion).
    `end_cursor`  — draw a block cursor on the final prompt line. Toggled by
                    the post-output blink loop.
    """
    img = Image.new("RGB", (W, H), BG_COLOR)
    draw = ImageDraw.Draw(img)
    y = PADDING

    cmd_lines = typed_cmd.split("\n")
    for idx, line in enumerate(cmd_lines):
        if idx == 0:
            draw.text((PADDING, y), "$", fill=PROMPT_COLOR, font=font)
            draw.text((PADDING + CELL_W * 2, y), line, fill=FG_COLOR, font=font)
            cur_x = PADDING + CELL_W * (2 + len(line))
        else:
            draw.text((PADDING, y), line, fill=FG_COLOR, font=font)
            cur_x = PADDING + CELL_W * len(line)
        cur_y = y
        y += CELL_H

    if show_cursor and not output_text and not end_prompt:
        draw.rectangle(
            [(cur_x, cur_y + 2), (cur_x + CELL_W, cur_y + CELL_H - 1)],
            fill=FG_COLOR,
        )

    if output_text:
        for line in output_text.split("\n"):
            draw.text((PADDING, y), line, fill=color_for(line), font=font)
            y += CELL_H
            if y > H - PADDING - CELL_H:
                break

    if end_prompt:
        draw.text((PADDING, y), "$", fill=PROMPT_COLOR, font=font)
        if end_cursor:
            draw.rectangle(
                [
                    (PADDING + CELL_W * 2, y + 2),
                    (PADDING + CELL_W * 3, y + CELL_H - 1),
                ],
                fill=FG_COLOR,
            )

    return img


# ---- Frame timeline -------------------------------------------------------

frames: list[Image.Image] = []
durations: list[int] = []

# Frame 0 — empty prompt with cursor (1.0s hold so the GIF loop has a clear start).
frames.append(render("", "", show_cursor=True, end_prompt=False))
durations.append(1000)

# Type the command in 2-char batches at 90ms/frame (~22 chars/sec — human-fast).
typed = ""
joined_cmd = "\n".join(CMD_LINES)
typing_batch = 2
for i in range(typing_batch, len(joined_cmd) + 1, typing_batch):
    typed = joined_cmd[:i]
    frames.append(render(typed, "", show_cursor=True, end_prompt=False))
    durations.append(90)
if len(joined_cmd) % typing_batch != 0:
    typed = joined_cmd
    frames.append(render(typed, "", show_cursor=True, end_prompt=False))
    durations.append(90)

# Brief processing pause after Enter.
frames.append(render(typed, "", show_cursor=False, end_prompt=False))
durations.append(450)

# Output emerges in 12-char batches at 50ms — ~240 chars/sec, visibly
# "the computer is typing it" but tight enough to keep frame count
# manageable.
out_batch = 12
for i in range(out_batch, len(OUTPUT) + 1, out_batch):
    frames.append(render(typed, OUTPUT[:i], show_cursor=False, end_prompt=False))
    durations.append(50)
if len(OUTPUT) % out_batch != 0:
    frames.append(render(typed, OUTPUT, show_cursor=False, end_prompt=False))
    durations.append(50)

# Final hold with blinking cursor on the new prompt. 12 cursor-on / cursor-off
# alternations at 600ms each = 7.2s total — long enough to read the full
# bundle output before the loop restarts.
final_on = render(typed, OUTPUT, show_cursor=False, end_prompt=True, end_cursor=True)
final_off = render(typed, OUTPUT, show_cursor=False, end_prompt=True, end_cursor=False)

BLINK_CYCLES = 12
BLINK_MS = 600
for i in range(BLINK_CYCLES):
    frames.append(final_on if i % 2 == 0 else final_off)
    durations.append(BLINK_MS)

# ---- Save ----------------------------------------------------------------

print(
    f"frames: {len(frames)}, total duration: {sum(durations)/1000:.1f}s, "
    f"image: {W}x{H}"
)

OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
frames[0].save(
    OUT_PATH,
    save_all=True,
    append_images=frames[1:],
    duration=durations,
    loop=0,
    optimize=True,
    disposal=2,
)

raw_kb = OUT_PATH.stat().st_size / 1024
print(f"wrote {OUT_PATH} ({raw_kb:.0f} KB raw)")

# Optimization pass — ImageMagick reduces by 60-75% on per-frame deltas.
magick = shutil.which("magick")
if magick:
    subprocess.run(
        [magick, str(OUT_PATH), "-layers", "OptimizePlus", "+remap", str(OUT_PATH)],
        check=True,
    )
    opt_kb = OUT_PATH.stat().st_size / 1024
    print(f"optimized: {opt_kb:.0f} KB ({100 * opt_kb / raw_kb:.0f}% of raw)")
else:
    print(
        "note: `magick` not on PATH — skipping optimization. "
        "Install ImageMagick to compress the output further.",
        file=sys.stderr,
    )
