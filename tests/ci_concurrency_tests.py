#!/usr/bin/env python3
"""CI runs must keep exclusive ownership through bootstrap cleanup."""

import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class CiConcurrencyTests(unittest.TestCase):
    def test_concurrent_runs_do_not_overlap_and_forward_exit_status(self):
        with tempfile.TemporaryDirectory(prefix="model ci lock ") as temporary:
            root = Path(temporary)
            (root / ".infra/bin").mkdir(parents=True)
            (root / "scripts").mkdir()
            shutil.copy(ROOT / ".infra/bin/ci-check.sh", root / ".infra/bin/ci-check.sh")
            helper = ROOT / "scripts/ci-check-locked.py"
            if helper.exists():
                shutil.copy(helper, root / "scripts/ci-check-locked.py")
            bootstrap = root / ".infra/bootstrap.sh"
            bootstrap.write_text(
                '#!/usr/bin/env bash\nset -eu\ncd "$(dirname "$0")/.."\n'
                'mkdir active || exit 91\n'
                'trap "rmdir active" EXIT\n'
                'sleep 0.3\nexit "$2"\n'
            )
            bootstrap.chmod(0o755)
            processes = [
                subprocess.Popen(["bash", str(root / ".infra/bin/ci-check.sh"), str(code)],
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                for code in (0, 23)
            ]
            for process, expected in zip(processes, (0, 23)):
                _, stderr = process.communicate(timeout=10)
                self.assertEqual(process.returncode, expected, stderr.decode())
            self.assertFalse((root / "active").exists())


if __name__ == "__main__":
    unittest.main()
