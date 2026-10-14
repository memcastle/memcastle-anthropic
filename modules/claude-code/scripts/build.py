#!/usr/bin/env python3
"""Copy Claude's declarative plugin files into the package's runtime tree."""

from pathlib import Path
import shutil

root = Path(__file__).resolve().parents[1]
source = root / "plugin"
target = root / "dist"
# A stale command or hook must not survive an update to the authored plugin.
if target.exists():
    shutil.rmtree(target)
shutil.copytree(source, target)
