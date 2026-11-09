#!/usr/bin/env python3
"""Inspect the actual crate archive; no extraction or third-party Python modules."""
from pathlib import Path, PurePosixPath
import hashlib
import json
import os
import re
import subprocess
import sys
import tarfile
import tomllib
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
SOURCE_DIRS = {"src", "assets", "docs"}
UPLOAD_LIMIT = 10_000_000  # crates.io rejects larger archives
ROOT_FILES = {"Cargo.toml", "Cargo.toml.orig", "Cargo.lock", ".cargo_vcs_info.json",
              ".clippy.toml", "AGENTS.md", "README.md", "LICENSE", "THIRD_PARTY.md",
              "CHANGELOG.md", "scripts/prepare_fonts.py"}
GENERATED = {"Cargo.toml", "Cargo.toml.orig", "Cargo.lock", ".cargo_vcs_info.json"}
DIFFERENCES = (".diff.png", ".new.png", ".old.png")


def approved(path):
    parts = PurePosixPath(path).parts
    return (parts and (parts[0] in SOURCE_DIRS or path in ROOT_FILES)
            and not path.endswith(DIFFERENCES)
            and not any(part in {"..", "target", ".git", ".idea", ".claude", "storyboard"} for part in parts))


def read_archive(path, prefix):
    files = {}
    with tarfile.open(path, "r:gz") as archive:
        for member in archive.getmembers():
            if member.isdir():
                continue
            assert member.isfile(), f"Non-file archive entry: {member.name}"
            name = str(PurePosixPath(member.name).relative_to(prefix))
            assert name not in files and approved(name), f"Unexpected archive path: {name}"
            files[name] = archive.extractfile(member).read()
    return files


def check_links(files):
    for name, content in files.items():
        if not name.endswith(".md"):
            continue
        body = re.sub(r"```.*?```", "", content.decode(), flags=re.S)
        for link in re.findall(r"\]\(([^\s)]+)(?:\s+[^)]*)?\)", body):
            url = urlsplit(link.strip("<>"))
            if url.scheme or url.netloc or not url.path:
                continue
            target = os.path.normpath(str(PurePosixPath(name).parent / unquote(url.path))).replace(os.sep, "/")
            assert target in files or any(p.startswith(target.rstrip("/") + "/") for p in files), f"Broken packaged link: {name}: {link}"


def main():
    package = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]
    prefix = f"{package['name']}-{package['version']}"
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    archive = Path(sys.argv[1]) if len(sys.argv) > 1 else target / "package" / f"{prefix}.crate"
    files = read_archive(archive, prefix)
    size = archive.stat().st_size
    assert size <= UPLOAD_LIMIT, f"Archive is {size} bytes, over the crates.io limit of {UPLOAD_LIMIT}"
    assert not any(name.startswith(("tests/", "examples/")) for name in files), "Tests, baselines and examples stay out of the package"
    expected = {str(p.relative_to(ROOT)) for directory in SOURCE_DIRS
                for p in (ROOT / directory).rglob("*") if p.is_file() and approved(str(p.relative_to(ROOT)))}
    expected.update(ROOT_FILES - {"Cargo.toml.orig", ".cargo_vcs_info.json"})
    assert expected <= files.keys(), f"Missing files: {sorted(expected - files.keys())}"
    for name, content in files.items():
        if name not in GENERATED:
            assert content == (ROOT / name).read_bytes(), f"Source mismatch: {name}"
    for notice in ["LICENSE", "THIRD_PARTY.md", "assets/fonts/Body-OFL.txt", "assets/fonts/Display-OFL.txt", "assets/fonts/Mono-OFL.txt"]:
        assert notice in files, f"Missing licence: {notice}"
    assert files["Cargo.toml.orig"] == (ROOT / "Cargo.toml").read_bytes()
    normalized = tomllib.loads(files["Cargo.toml"].decode())["package"]
    for key in ["name", "version", "license", "rust-version"]:
        assert normalized[key] == package[key], f"Manifest mismatch: {key}"
    locked = {(p["name"], p["version"], p.get("source")) for p in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]}
    assert all((p["name"], p["version"], p.get("source")) in locked for p in tomllib.loads(files["Cargo.lock"].decode())["package"])
    vcs = json.loads(files[".cargo_vcs_info.json"])
    assert vcs["git"]["sha1"] == subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    check_links(files)
    print(json.dumps({"archive": str(archive), "files": len(files), "bytes": size, "sha256": hashlib.sha256(archive.read_bytes()).hexdigest(), "allowlist": "pass", "source_bytes": "pass", "licenses": "included", "packaged_links": "pass"}, indent=2))


if __name__ == "__main__":
    main()
