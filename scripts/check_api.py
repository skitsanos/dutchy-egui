#!/usr/bin/env python3
"""Fail when a public type could only grow in a major release.

docs/versioning.md keeps Dutchy on 1.x. A new variant on an exhaustive enum,
or a new field on a struct whose fields are all public, breaks host code, so
such types carry #[non_exhaustive] unless docs/plan-api-extensibility.md keeps
them exhaustive. Structs with a private field are already extensible.
"""
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
# Keep these lists equal to the keep lists in docs/plan-api-extensibility.md.
KEEP_ENUMS = {
    "AccordionState", "DateSegment", "FlexDirection", "Mode", "TableSortDirection",
    "ToolbarOrientation", "TrendDirection", "TrendSentiment", "Weekday",
}
KEEP_STRUCTS = {"TableSelection", "TableSort"}
ITEM = re.compile(r"^[ \t]*pub (enum|struct) (\w+)", re.M)
NAMED_FIELD = re.compile(r"^\s*(pub(?:\([^)]*\))?\s+)?(?:r#)?\w+\s*:(?!:)", re.M)


def attributes(text, start):
    """Attribute text directly above an item, skipping doc comments."""
    lines = text[:start].split("\n")[:-1]
    found, depth = [], 0
    for line in reversed(lines):
        stripped = line.strip()
        if depth == 0 and stripped.startswith("//"):
            continue
        if depth == 0 and not stripped.endswith("]"):
            break
        depth += stripped.count("]") - stripped.count("[")
        found.append(stripped)
        if depth <= 0 and stripped.startswith("#["):
            depth = 0
    return " ".join(found)


def enclosed(text, start, opener, closer):
    """Text between the first `opener` at or after `start` and its match."""
    begin = text.index(opener, start)
    depth = 0
    for index in range(begin, len(text)):
        depth += {opener: 1, closer: -1}.get(text[index], 0)
        if depth == 0:
            return text[begin + 1:index]
    raise ValueError("unbalanced item body")


def after_generics(text, start):
    """Index just past a generic parameter list such as `<'a, T = ()>`."""
    if not text.startswith("<", start):
        return start
    depth = 0
    for index in range(start, len(text)):
        char = text[index]
        if char == "<":
            depth += 1
        elif char == ">" and text[index - 1] != "-":
            depth -= 1
            if depth == 0:
                return index + 1
    raise ValueError("unbalanced generics")


def only_public_fields(text, match):
    """True when every field of the struct is plainly `pub`."""
    start = after_generics(text, match.end())
    head = re.match(r"\s*([{(;]|where)", text[start:])
    if head is None:
        raise ValueError(f"cannot parse struct {match.group(2)}")
    if head.group(1) == ";":
        return True
    if head.group(1) == "(":
        fields = [f.strip() for f in enclosed(text, start, "(", ")").split(",")]
        return all(f.startswith("pub ") for f in fields if f)
    body = enclosed(text, start, "{", "}")
    body = re.sub(r"//[^\n]*", "", body)
    return all(vis is not None and vis.strip() == "pub" for vis in NAMED_FIELD.findall(body))


def scan():
    """Yield (path, line, kind, name, marked) for enums and all-public structs."""
    for path in sorted((ROOT / "src").rglob("*.rs")):
        text = path.read_text()
        for match in ITEM.finditer(text):
            kind, name = match.groups()
            if kind == "struct" and not only_public_fields(text, match):
                continue
            line = text.count("\n", 0, match.start()) + 1
            marked = "non_exhaustive" in attributes(text, match.start())
            yield path.relative_to(ROOT), line, kind, name, marked


def main():
    items = list(scan())
    keep = {"enum": KEEP_ENUMS, "struct": KEEP_STRUCTS}
    errors = []
    for path, line, kind, name, marked in items:
        kept = name in keep[kind]
        if marked and kept:
            errors.append(f"{path}:{line}: {kind} {name} is kept exhaustive but marked")
        elif not marked and not kept:
            errors.append(f"{path}:{line}: {kind} {name} needs #[non_exhaustive] or a keep-list entry")
    seen = {(kind, name) for _, _, kind, name, _ in items}
    for kind, names in keep.items():
        errors += [f"keep list: no public {kind} {name}" for name in sorted(names) if (kind, name) not in seen]
    if errors:
        print("\n".join(errors))
        print("See docs/versioning.md and docs/plan-api-extensibility.md.")
        return 1
    for kind in ("enum", "struct"):
        total = sum(1 for item in items if item[2] == kind)
        marked = sum(1 for item in items if item[2] == kind and item[4])
        label = "enums" if kind == "enum" else "structs with only public fields"
        print(f"{total} public {label}: {marked} non-exhaustive, {total - marked} kept")
    return 0


if __name__ == "__main__":
    sys.exit(main())
