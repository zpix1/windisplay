#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path


PROJECT_ROOT = Path(__file__).parent.resolve()
PYPROJECT = PROJECT_ROOT / "pyproject.toml"
INIT_FILE = PROJECT_ROOT / "windisplay" / "__init__.py"


def read_version_from_pyproject() -> str:
    text = PYPROJECT.read_text(encoding="utf-8")
    m = re.search(r"^version\s*=\s*\"(\d+)\.(\d+)\.(\d+)\"\s*$", text, re.M)
    if not m:
        raise RuntimeError("Could not find version in pyproject.toml")
    return ".".join(m.groups())


def write_version_to_pyproject(new_version: str) -> None:
    text = PYPROJECT.read_text(encoding="utf-8")
    text = re.sub(
        r"^(version\s*=\s*)\"\d+\.\d+\.\d+\"\s*$",
        rf'\1"{new_version}"',
        text,
        flags=re.M,
    )
    PYPROJECT.write_text(text, encoding="utf-8", newline="\n")


def read_version_from_init() -> str | None:
    if not INIT_FILE.exists():
        return None
    text = INIT_FILE.read_text(encoding="utf-8")
    m = re.search(r"^__version__\s*=\s*\"(\d+\.\d+\.\d+)\"\s*$", text, re.M)
    return m.group(1) if m else None


def write_version_to_init(new_version: str) -> None:
    if not INIT_FILE.exists():
        return
    text = INIT_FILE.read_text(encoding="utf-8")
    if re.search(r"^__version__\s*=\s*\"\d+\.\d+\.\d+\"\s*$", text, re.M):
        text = re.sub(
            r"^__version__\s*=\s*\"\d+\.\d+\.\d+\"\s*$",
            f'__version__ = "{new_version}"',
            text,
            flags=re.M,
        )
    else:
        # Append if missing
        if not text.endswith("\n"):
            text += "\n"
        text += f'__version__ = "{new_version}"\n'
    INIT_FILE.write_text(text, encoding="utf-8", newline="\n")


def bump_version(ver: str, bump: str) -> str:
    major, minor, patch = map(int, ver.split("."))
    if bump == "major":
        return f"{major + 1}.0.0"
    if bump == "minor":
        return f"{major}.{minor + 1}.0"
    # default patch
    return f"{major}.{minor}.{patch + 1}"


def run(cmd: list[str]) -> None:
    print("$", " ".join(cmd))
    subprocess.check_call(cmd, cwd=str(PROJECT_ROOT))


def current_branch() -> str:
    try:
        out = subprocess.check_output(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=str(PROJECT_ROOT)
        )
        return out.decode().strip()
    except Exception:
        return "HEAD"


def main() -> int:
    parser = argparse.ArgumentParser(description="Bump version, tag, and push")
    parser.add_argument("--bump", choices=["patch", "minor", "major"], default="patch")
    parser.add_argument("--no-push", action="store_true", help="Do not push to origin")
    args = parser.parse_args()

    if not PYPROJECT.exists():
        print(f"pyproject.toml not found at {PYPROJECT}", file=sys.stderr)
        return 1

    old_ver = read_version_from_pyproject()
    new_ver = bump_version(old_ver, args.bump)

    print(f"Version: {old_ver} -> {new_ver}")
    write_version_to_pyproject(new_ver)
    write_version_to_init(new_ver)

    # Commit, tag, push
    run(["git", "add", "pyproject.toml", "windisplay/__init__.py"])
    run(["git", "commit", "-m", f"chore: bump version to {new_ver}"])
    run(["git", "tag", "-a", f"v{new_ver}", "-m", f"Release v{new_ver}"])

    if not args.no_push:
        branch = current_branch()
        run(["git", "push", "origin", branch])
        run(["git", "push", "origin", f"v{new_ver}"])

    print("Done.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
