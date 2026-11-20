#!/usr/bin/env python3
"""Gate checks for Rotland: task scope, number sources, unused pub items.

  python3 -I tools/gate_checks.py --base main [--task tasks/X.md]
  python3 -I tools/gate_checks.py --selftest

New numbers in non-test game/, world/, humanoid/ code need a source tag on the
same line or the line above: `// godot: <file>.cs:<line>`, `// data: <f>.json <key>`
or `// new: DECISIONS <YYYY-MM-DD>`. See docs/PLAN_0.2.md ("Workflow v2").
"""

import argparse, fnmatch, os, re, subprocess, sys

GODOT_ROOT = "/home/aymen/Documents/zombi servival"
NUM_BASES = ("game/src/", "world/src/", "humanoid/src/")
EXEMPT = {0.0, 1.0, 2.0, 3.0, 4.0, 0.5, 0.25, 10.0, 100.0, 1000.0,
          0.001, 1e-3, 1e-4, 1e-5, 1e-6, 180.0, 360.0}
NUM_RE = re.compile(
    r"(?<![\w.\[])(?:\d[\d_]*(?:\.[\d_]+)?|\.\d+)(?:[eE][+-]?\d+)?"
    r"(?:_?(?:f32|f64|u8|u16|u32|u64|i8|i16|i32|i64|usize|isize))?(?![\w.])")
SUFFIX_RE = re.compile(
    r"_?(?:f32|f64|u8|u16|u32|u64|i8|i16|i32|i64|usize|isize)$")
PUB_RE = re.compile(
    r"^\s*pub\s+(?:async\s+)?(fn|struct|enum|const|trait|type|mod)\s+([A-Za-z_]\w*)")
TEST_CFG_RE = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
GODOT_NUM_RE = re.compile(r"-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?f?")

def repo():
    r = subprocess.run(["git", "rev-parse", "--show-toplevel"],
                       capture_output=True, text=True)
    return r.stdout.strip() if r.returncode == 0 and r.stdout.strip() else os.getcwd()

def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True, cwd=repo())

def read_text(path):
    try:
        with open(os.path.join(repo(), path), encoding="utf-8", errors="replace") as f:
            return f.read()
    except OSError:
        return ""

def task_field(task, name):
    """First `<name>: value` line of a task file, or None."""
    if not task:
        return None
    path = task if os.path.isabs(task) else os.path.join(repo(), task)
    rx = re.compile(rf"^\s*{name}:\s*(.*)$", re.I)
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            for line in f:
                m = rx.match(line)
                if m:
                    return m.group(1).strip()
    except OSError:
        pass
    return None

def list_field(task, name):
    raw = task_field(task, name)
    if raw is None:
        return None
    items = [p.strip().strip("`").strip() for p in raw.split(",")]
    return [p for p in items if p]

def path_matches(path, pattern):
    if fnmatch.fnmatchcase(path, pattern):
        return True
    if not any(ch in pattern for ch in "*?["):
        base = pattern.rstrip("/")
        return path == base or path.startswith(base + "/")
    return False

def task_rel(task):
    return os.path.relpath(os.path.abspath(task), repo()).replace(os.sep, "/") if task else None

def changed_paths(mb):
    """Committed + unstaged + untracked paths vs merge-base."""
    paths = set()
    for line in git("diff", "--name-status", "-M", mb).stdout.splitlines():
        parts = line.split("\t")
        if len(parts) >= 2 and parts[0][0] in "ACDMRTUXB":
            paths.add(parts[-1])
    paths.update(p for p in git("ls-files", "--others", "--exclude-standard").stdout.splitlines() if p)
    return sorted(paths)

def failures(head, items):
    out = [head] + items[:10]
    if len(items) > 10:
        out.append(f"  ... {len(items) - 10} more")
    return False, "\n".join(out)

def check_scope(task, paths):
    files = list_field(task, "Files")
    if files is None:
        return True, "scope: SKIP (no task Files: line)"
    allowed = list(files)
    own = task_rel(task)
    if own:
        allowed.append(own)
    bad = [p for p in paths if not any(path_matches(p, pat) for pat in allowed)]
    if not bad:
        return True, f"scope: PASS ({len(paths)} changed paths)"
    return failures(f"scope: FAIL ({len(bad)} outside task Files:)", [f"  {p}" for p in bad])

def added_lines(mb):
    """dict path -> set of added line numbers in the new file."""
    adds, cur, newno = {}, None, 0
    for line in git("diff", "-U0", "--no-color", mb).stdout.splitlines():
        if line.startswith("+++ "):
            p = line[4:].strip()
            cur = p[2:] if p != "/dev/null" and p.startswith("b/") else None
            if cur:
                adds.setdefault(cur, set())
        elif line.startswith("@@") and cur is not None:
            m = re.search(r"\+(\d+)", line)
            newno = int(m.group(1)) if m else 0
        elif cur is not None and line.startswith("+") and not line.startswith("+++"):
            adds[cur].add(newno)
            newno += 1
    for p in git("ls-files", "--others", "--exclude-standard").stdout.splitlines():
        if p.endswith(".rs") and os.path.isfile(os.path.join(repo(), p)):
            adds[p] = set(range(1, len(read_text(p).splitlines()) + 1))
    return adds

def is_test_path(path):
    parts = path.split("/")
    return "tests" in parts or parts[-1].endswith("_tests.rs") or parts[-1] == "tests.rs"

def code_only(line):
    """Drop string/char literal contents and // comments."""
    out, i, n, quote = [], 0, len(line), None
    while i < n:
        c = line[i]
        if quote is not None:
            if c == "\\":
                i += 2
                continue
            if c == quote:
                quote = None
            i += 1
            continue
        if c in "\"'":
            quote = c
            i += 1
            continue
        if c == "/" and i + 1 < n and line[i + 1] == "/":
            break
        out.append(c)
        i += 1
    return "".join(out)

def parse_number(tok):
    try:
        return float(SUFFIX_RE.sub("", tok.replace("_", "")))
    except ValueError:
        return None

def numbers_in(line):
    return [n for n in map(parse_number, NUM_RE.findall(code_only(line))) if n is not None]

def godot_numbers(path, line_no):
    """Numbers near `line_no` in a Godot C# file, or None if the file is missing."""
    found = next((c for c in (os.path.join(GODOT_ROOT, path),
                              os.path.join(GODOT_ROOT, "scripts", path))
                  if os.path.isfile(c)), None)
    if found is None:
        return None
    with open(found, encoding="utf-8", errors="replace") as f:
        lines = f.read().splitlines()
    nums = []
    for text in lines[max(0, line_no - 4):line_no + 3]:
        for raw in GODOT_NUM_RE.findall(text):
            nums.append(float(raw[:-1] if raw.endswith(("f", "F")) else raw))
    return nums

def parse_tag(text):
    """Return ('godot', path, line) / ('data', file, key) / ('new', date) or None."""
    m = re.search(r"//\s*godot:\s*([^\s:]+\.cs):(\d+)", text)
    if m:
        return ("godot", m.group(1), int(m.group(2)))
    m = re.search(r"//\s*data:\s*([^\s]+\.json)\s+(\S+)", text)
    if m:
        return ("data", m.group(1), m.group(2))
    m = re.search(r"//\s*new:\s*DECISIONS\s+(\d{4}-\d{2}-\d{2})", text)
    return ("new", m.group(1)) if m else None

def has_decisions_row(text, date):
    return re.search(r"^\|\s*" + re.escape(date) + r"\s*\|", text, re.M) is not None

def tag_ok(num, text):
    """(ok, reason) for the source tag found in `text` (or no tag)."""
    tag = parse_tag(text)
    if tag is None:
        return False, "no source tag"
    if tag[0] == "godot":
        nums = godot_numbers(tag[1], tag[2])
        if nums is None:
            return False, f"godot file missing: {tag[1]}"
        if any(abs(n - num) < 1e-9 for n in nums):
            return True, ""
        return False, f"number {num:g} not near {tag[1]}:{tag[2]}"
    if tag[0] == "data":
        text_json = read_text(os.path.join("data", tag[1]))
        if not text_json:
            return False, f"data file missing: data/{tag[1]}"
        if tag[2] in text_json:
            return True, ""
        return False, f"key {tag[2]} not in data/{tag[1]}"
    if has_decisions_row(read_text("docs/DECISIONS.md"), tag[1]):
        return True, ""
    return False, f"no DECISIONS row dated {tag[1]}"

def check_numbers(adds):
    offenders, checked = [], 0
    for path in sorted(adds):
        if not path.startswith(NUM_BASES) or not path.endswith(".rs") or is_test_path(path):
            continue
        lines = read_text(path).splitlines()
        test_at = next((i + 1 for i, ln in enumerate(lines) if TEST_CFG_RE.search(ln)), None)
        for no in sorted(adds[path]):
            if no < 1 or no > len(lines) or (test_at is not None and no >= test_at):
                continue
            line = lines[no - 1]
            prev = lines[no - 2] if no >= 2 else ""
            for num in numbers_in(line):
                if num in EXEMPT:
                    continue
                checked += 1
                reason = "no source tag"
                for text in (line, prev):
                    if parse_tag(text) is None:
                        continue
                    ok, why = tag_ok(num, text)
                    if ok:
                        reason = None
                        break
                    reason = why
                if reason is not None:
                    offenders.append(f"{path}:{no}  {num:g}  {reason}")
    if not offenders:
        return True, f"numbers: PASS ({checked} checked)"
    return failures(f"numbers: FAIL ({len(offenders)} offenders)", offenders)

def check_unused(adds):
    offenders = []
    for path in sorted(adds):
        if not path.endswith(".rs"):
            continue
        lines = read_text(path).splitlines()
        for no in sorted(adds[path]):
            if no < 1 or no > len(lines):
                continue
            m = PUB_RE.match(lines[no - 1])
            if not m or m.group(2) in ("new", "default", "from", "fmt", "drop"):
                continue
            name = m.group(2)
            uses = 0
            for hit in git("grep", "--untracked", "-n", "-w", "-e", name, "--", "*.rs").stdout.splitlines():
                parts = hit.split(":", 2)
                if len(parts) >= 2 and not (parts[0] == path and parts[1] == str(no)):
                    uses += 1
            if uses == 0:
                offenders.append(f"{path}:{no}  pub item `{name}` is never used")
    if not offenders:
        return True, "unused: PASS"
    return failures(f"unused: FAIL ({len(offenders)})", offenders)

def selftest():
    n = 0
    cases = [
        ("let x = 2.5_f32;", [2.5]), ("let x = 1e-4;", [1e-4]),
        ('let s = "42";', []), ("let a = v[3];", []), ("let t = p.0;", []),
        ("let w = 1920; // 999", [1920.0]), ("let d = 0x1F;", []),
        ("let e = -0.75;", [0.75]), ("let r = 0.7071;", [0.7071]),
    ]
    for src, want in cases:
        got = numbers_in(src)
        assert got == want, f"{src!r}: {got} != {want}"
        n += 1
    assert parse_number("12.0_f32") == 12.0 and parse_number("7u32") == 7.0
    assert 1e-3 in EXEMPT and 0.7071 not in EXEMPT
    n += 3
    assert parse_tag("// godot: Player.cs:120") == ("godot", "Player.cs", 120)
    assert parse_tag("// data: zombies.json walker_speed") == ("data", "zombies.json", "walker_speed")
    assert parse_tag("// new: DECISIONS 2026-11-20") == ("new", "2026-11-20")
    assert parse_tag("// plain comment") is None
    n += 4
    assert has_decisions_row("| 2026-11-20 | x | y |", "2026-11-20")
    assert not has_decisions_row("| 2026-11-19 | x | y |", "2026-11-20")
    n += 2
    assert path_matches("game/src/a.rs", "game/src/*")
    assert path_matches("world/src/city/roads.rs", "world/src/city/*")
    assert path_matches("tools/gate.sh", "tools") and not path_matches("rotland/src/main.rs", "game/src/*")
    n += 4
    print(f"selftest: {n} ok")

def main():
    ap = argparse.ArgumentParser(description="Rotland gate checks")
    ap.add_argument("--base", default="main")
    ap.add_argument("--task")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        selftest()
        return 0
    mb = merge_base(args.base)
    adds = added_lines(mb)
    results = [check_scope(args.task, changed_paths(mb)),
               check_numbers(adds),
               check_unused(adds)]
    for ok, line in results:
        print(line)
    return 0 if all(ok for ok, _ in results) else 1

def merge_base(base):
    r = git("merge-base", base, "HEAD")
    return r.stdout.strip() if r.returncode == 0 and r.stdout.strip() else base


if __name__ == "__main__":
    sys.exit(main())
