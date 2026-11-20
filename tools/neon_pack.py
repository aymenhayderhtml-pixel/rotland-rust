#!/usr/bin/env python3
"""Build one paste-ready task pack for Gemini Neon (chat only, no file access).

Usage: python3 -I tools/neon_pack.py tasks/X.md [--out neon/out/X.prompt.md]

Reads these optional task-file lines (comma separated paths/globs):
  Read:  full text       API:  pub signatures only
  Data:  full JSON       Godot: full C# from /home/aymen/Documents/zombi servival
  Files: files Neon may write (echoed so the answer paths match)

Writes a single markdown file and prints its size in chars and rough tokens.
"""

import argparse
import glob
import os
import re
import subprocess
import sys

GODOT_ROOT = "/home/aymen/Documents/zombi servival"
FIELDS = ("Read", "API", "Data", "Godot", "Files")
SIG_RE = re.compile(
    r"^\s*(pub\s+(?:async\s+)?(?:fn|struct|enum|trait|const|type|mod|union)\b|impl\b)"
)
PUB_FIELD_RE = re.compile(r"^pub\s+[A-Za-z_]\w*\s*:")
MAX_CHARS = 400_000

RULES = """# Rotland task pack for Gemini Neon

You are writing Rust for **Rotland**: a solo, offline, 3D open-world zombie survival
game with a custom wgpu engine (Rust 2024 edition, cargo workspace).

## Rules (follow all of them)

- Rust 2024 edition. Math with glam 0.34, UI with egui 0.37.
- Do not invent APIs. Use only what appears under "Read", "API signatures",
  "Data" and "Godot source" below. If something is missing, say so in NOTES.
- No unused code, no placeholders, no TODOs.
- Every game number must come from the data or Godot source shown here and carry a
  source tag comment on its line (or the line above):
  `// godot: <file>.cs:<line>` or `// data: <file>.json <key>` or
  `// new: DECISIONS <YYYY-MM-DD>`.
- Godot yaw convention: forward = `(-sin yaw, 0, -cos yaw)`. Facing east (+X) is yaw `-PI/2`.
- Code must be clippy-clean with `cargo clippy --all-targets -- -D warnings`:
  use let-chains instead of nested `if`s, never hand-write constants like `0.7071`
  or `3.14` (use `std::f32::consts`), keep files small (~300 lines), one topic per file.
- Pure, testable logic goes in the `game/` and `world/` crates with unit tests;
  wiring into the game loop stays in `rotland/`.
"""

ANSWER = """## Required answer format

For every file you write, output exactly this, with the **complete** file
(never a diff, never a fragment):

=== FILE: <path> ===
```rust
<complete file content>
```
=== END ===

The code block fences are REQUIRED (the answer is copied from a chat window;
text outside a code block gets re-wrapped and loses `*` characters).

After all files, add one notes block:

=== NOTES ===
<short wiring notes: what you wired, what the caller must do, known risks>
=== END ===
"""


def repo():
    r = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True, text=True, cwd=os.getcwd(),
    )
    return r.stdout.strip() if r.returncode == 0 and r.stdout.strip() else os.getcwd()


def task_fields(text):
    """First value of each `Read:`/`API:`/`Data:`/`Godot:`/`Files:` line."""
    fields = {}
    for line in text.splitlines():
        m = re.match(r"^\s*(Read|API|Data|Godot|Files)\s*:\s*(.*)$", line)
        if m and m.group(1) not in fields:
            raw = m.group(2)
            fields[m.group(1)] = [
                p.strip().strip("`") for p in raw.split(",") if p.strip().strip("`")
            ]
    return fields


def expand(patterns, base):
    """Expand paths/globs relative to `base`, in order, without duplicates."""
    out = []
    for pat in patterns:
        matches = sorted(glob.glob(os.path.join(base, pat), recursive=True))
        if not matches:
            print(f"neon_pack: missing: {pat}", file=sys.stderr)
        for m in matches:
            if os.path.isfile(m) and m not in out:
                out.append(m)
    return out


def read(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return f.read().rstrip("\n")


def show(path, base):
    """Repo-relative path when inside base, else the absolute path."""
    rel = os.path.relpath(path, base)
    return path if rel.startswith("..") else rel


def fence(path, text):
    return f"### `{path}`\n````\n{text}\n````\n"


def api_lines(text):
    """(line_no, text) for pub items, impl lines, and pub struct fields."""
    out = []
    in_struct = False
    for i, line in enumerate(text.splitlines(), 1):
        s = line.strip()
        if not s or s.startswith("//"):
            continue
        if SIG_RE.match(line):
            out.append((i, s))
            if re.match(r"pub\s+struct\b", s) and ";" not in s:
                in_struct = True
            continue
        if in_struct:
            if s.startswith("}"):
                in_struct = False
            elif PUB_FIELD_RE.match(s):
                out.append((i, s))
    return out


def section(title, blocks):
    if not blocks:
        return ""
    return f"## {title}\n\n" + "\n".join(blocks)


def main():
    ap = argparse.ArgumentParser(description="Build a Neon task pack")
    ap.add_argument("task")
    ap.add_argument("--out")
    args = ap.parse_args()

    root = repo()
    task_path = args.task if os.path.isabs(args.task) else os.path.join(root, args.task)
    if not os.path.isfile(task_path):
        print(f"neon_pack: no such task file: {args.task}", file=sys.stderr)
        return 1
    task_text = read(task_path)
    fields = task_fields(task_text)

    out_path = args.out
    if not out_path:
        stem = os.path.splitext(os.path.basename(task_path))[0]
        out_path = os.path.join("neon", "out", f"{stem}.prompt.md")
    if not os.path.isabs(out_path):
        out_path = os.path.join(root, out_path)

    rel_task = show(task_path, root)

    parts = [RULES, "", "## The task", "", f"Source: `{rel_task}`", "", task_text, ""]

    files = fields.get("Files") or []
    if files:
        parts.append("## Files you may write\n")
        parts.append("\n".join(f"- `{p}`" for p in files) + "\n")

    read_blocks = [
        fence(show(p, root), read(p))
        for p in expand(fields.get("Read") or [], root)
    ]
    parts.append(section("Read", read_blocks))

    api_blocks = []
    for p in expand(fields.get("API") or [], root):
        lines = api_lines(read(p))
        body = "\n".join(f"{no:5d}: {s}" for no, s in lines)
        api_blocks.append(f"### `{show(p, root)}`\n````\n{body}\n````\n")
    parts.append(section("API signatures (no bodies; do not invent more)", api_blocks))

    data_blocks = [
        fence(show(p, root), read(p))
        for p in expand(fields.get("Data") or [], root)
    ]
    parts.append(section("Data", data_blocks))

    godot_blocks = [
        fence(os.path.relpath(p, GODOT_ROOT), read(p))
        for p in expand(fields.get("Godot") or [], GODOT_ROOT)
    ]
    parts.append(section("Godot source (port behaviour and numbers from here)", godot_blocks))

    parts.append(ANSWER)

    text = "\n".join(p for p in parts if p is not None) + "\n"
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(text)

    size = len(text)
    tokens = size // 4
    print(f"wrote {show(out_path, root)}: {size} chars (~{tokens} tokens)")
    if size > MAX_CHARS:
        print(
            f"neon_pack: warning: output is {size} chars, over ~{MAX_CHARS}; "
            "trim Read:/API:/Data: lists",
            file=sys.stderr,
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
