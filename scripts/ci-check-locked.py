#!/usr/bin/env python3
# Copyright (c) 2025 - 2026 Haixing Hu.
# SPDX-License-Identifier: Apache-2.0

"""Serialize CI runs, including bootstrap's exit-time artifact cleanup."""

import os
from pathlib import Path
import subprocess
import sys
import time


def main():
    root = Path(__file__).resolve().parents[1]
    target = root / "target"
    target.mkdir(exist_ok=True)
    # Keep this file outside the directories removed by bootstrap cleanup.
    # Never unlink it: waiters must continue locking the same file.
    with (target / "ci-check.lock").open("a+b") as lock:
        if os.name == "nt":
            import msvcrt

            lock.write(b"\0")
            lock.flush()
            while True:
                lock.seek(0)
                try:
                    msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
                    break
                except OSError:
                    time.sleep(0.1)
        else:
            import fcntl

            fcntl.flock(lock, fcntl.LOCK_EX)
            # Exec preserves signals and the exact bootstrap exit status. The
            # descriptor remains open until bootstrap finishes its EXIT trap.
            os.set_inheritable(lock.fileno(), True)
            os.execvp("bash", ["bash", str(root / ".infra/bootstrap.sh"), ".infra/bin/ci-check.sh", *sys.argv[1:]])
        return subprocess.call(["bash", str(root / ".infra/bootstrap.sh"), ".infra/bin/ci-check.sh", *sys.argv[1:]])


if __name__ == "__main__":
    sys.exit(main())
