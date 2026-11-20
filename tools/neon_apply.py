#!/usr/bin/env python3
"""Apply a Gemini Neon answer (=== FILE: path === blocks) into the repo.

Usage: python3 -I tools/neon_apply.py neon/in/X.md --task tasks/X.md [--dry-run]

Refuses any path that is not in the task's `Files:` list, is absolute, or
contains `..`. If any path is refused nothing is written (exit 1).
Also appends the `=== NOTES ===` block under `## Neon notes` in the task file.
"""

import argparse
import fnmatch
import os
import re
import subprocess
import sys

FILE_HEAD = re.compile(r"^=== FILE:\s*(.+?)\s*===\s*$")
END = re.compile(r"^=== END ===\s*$")
NOTES_HEAD = re.compile(r"^=== NOTES ===\s*$")
FENCE_OPEN = re.compile(r"^\s*(?:```|~~~)\w*\s*$")
FENCE_CLOSE = re.compile(r"^\s*(?:```|~~~)\s*$")
HEADING = "## Neon notes"


def repo():
    r = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True, text=True, cwd=os.getcwd(),
    )
    return r.stdout.strip() if r.returncode == 0 and r.stdout.strip() else os.getcwd()


def read(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return f.read()


def show(path, base):
    rel = os.path.relpath(path, base)
    return path if rel.startswith("..") else rel


def task_files(task_text):
    """File patterns from the first `Files:` line, or None."""
    for line in task_text.splitlines():
        m = re.match(r"^\s*Files:\s*(.*)$", line, re.I)
        if m:
            items = []
            for part in m.group(1).split(","):
                p = part.strip().strip("`").strip()
                if p:
                    items.append(p)
            return items
    return None


def path_allowed(path, patterns):
    for pat in patterns:
        if fnmatch.fnmatchcase(path, pat):
            return True
        if not any(ch in pat for ch in "*?["):
            base = pat.rstrip("/")
            if path == base or path.startswith(base + "/"):
                return True
    return False


def strip_fence(body):
    """Remove one surrounding ``` fence (with optional language word)."""
    lines = body.split("\n")
    first = next((i for i, l in enumerate(lines) if l.strip()), None)
    last = next((i for i in range(len(lines) - 1, -1, -1) if lines[i].strip()), None)
    if first is None or last is None:
        return ""
    if FENCE_OPEN.match(lines[first]) and first != last and FENCE_CLOSE.match(lines[last]):
        lines = lines[first + 1:last]
    elif FENCE_OPEN.match(lines[first]) and first != last:
        lines = lines[first + 1:]
    else:
        lines = lines[first:last + 1]
    return "\n".join(lines).strip("\n") + "\n"


def parse_answer(text):
    """Return ([(path, content)], notes_text)."""
    blocks = []
    notes = []
    cur_path = None
    cur = []
    mode = None
    for line in text.splitlines():
        m = FILE_HEAD.match(line)
        if m:
            if cur_path is not None:
                blocks.append((cur_path, "\n".join(cur)))
            cur_path, cur, mode = m.group(1), [], "file"
            continue
        if NOTES_HEAD.match(line):
            if cur_path is not None:
                blocks.append((cur_path, "\n".join(cur)))
            cur_path, cur, mode = None, [], "notes"
            continue
        if END.match(line):
            if cur_path is not None:
                blocks.append((cur_path, "\n".join(cur)))
            cur_path, cur, mode = None, [], None
            continue
        if mode == "file":
            cur.append(line)
        elif mode == "notes":
            notes.append(line)
    if cur_path is not None:
        blocks.append((cur_path, "\n".join(cur)))

    cleaned = [(path, strip_fence(body)) for path, body in blocks]
    return cleaned, "\n".join(notes).strip()


def write_notes(task_path, notes):
    lines = read(task_path).splitlines()
    keep = []
    for line in lines:
        if line.strip() == HEADING:
            break
        keep.append(line)
    while keep and not keep[-1].strip():
        keep.pop()
    out = keep + ["", HEADING, "", notes, ""]
    with open(task_path, "w", encoding="utf-8") as f:
        f.write("\n".join(out))


def main():
    ap = argparse.ArgumentParser(description="Apply a Neon answer")
    ap.add_argument("answer")
    ap.add_argument("--task", required=True)
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    root = repo()

    def resolve(p):
        return p if os.path.isabs(p) else os.path.join(root, p)

    task_path = resolve(args.task)
    answer_path = resolve(args.answer)
    for p in (task_path, answer_path):
        if not os.path.isfile(p):
            print(f"neon_apply: no such file: {show(p, root)}", file=sys.stderr)
            return 1

    task_text = read(task_path)
    files = task_files(task_text)
    if files is None:
        print("neon_apply: task has no Files: line", file=sys.stderr)
        return 1

    blocks, notes = parse_answer(read(answer_path))
    if not blocks:
        print("neon_apply: no === FILE: ... === blocks found", file=sys.stderr)
        return 1

    refused = []
    accepted = []
    for raw, content in blocks:
        path = raw.strip().strip("`").strip()
        if not path:
            refused.append(("<empty>", "empty path"))
            continue
        if os.path.isabs(path) or path.startswith("~"):
            refused.append((path, "absolute path"))
            continue
        if ".." in path.replace("\\", "/").split("/"):
            refused.append((path, "contains .."))
            continue
        if not path_allowed(path, files):
            refused.append((path, "not in task Files: list"))
            continue
        accepted.append((path, content))

    for path, why in refused:
        print(f"refused: {path}  ({why})")
    if refused:
        print(f"neon_apply: {len(refused)} refused path(s); nothing written")
        return 1

    for path, content in accepted:
        n = len(content.splitlines())
        if args.dry_run:
            print(f"would write {path} ({n} lines)")
            continue
        full = os.path.join(root, path)
        os.makedirs(os.path.dirname(full) or ".", exist_ok=True)
        with open(full, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"wrote {path} ({n} lines)")

    if notes:
        if args.dry_run:
            print(f"would append {len(notes.splitlines())} note line(s) to {args.task}")
        else:
            write_notes(task_path, notes)
            print(f"appended notes to {args.task}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
