"""Runs the cocotb tests of `fifo.sv` on epitome-sv.

cocotb's Icarus runner starts `iverilog` and `vvp` from the PATH; this puts
links of those names to `epitome-iverilog` and `epitome-vvp` first on it.
The binaries are taken from `EPITOME_SV_BIN`, or else from the workspace's
`target/release` (build them with `cargo build --release -p epitome-sv`).

    python3 sv/examples/cocotb/run.py

The exit status is 0 when every test passes.
"""

import os
import sys
from pathlib import Path

from cocotb_tools.runner import get_results, get_runner

here = Path(__file__).resolve().parent
bin_dir = Path(os.environ.get("EPITOME_SV_BIN", here.parents[2] / "target" / "release"))
build_dir = Path(os.environ.get("EPITOME_COCOTB_BUILD", here / "build"))

shims = build_dir / "bin"
shims.mkdir(parents=True, exist_ok=True)
for name, target in [("iverilog", "epitome-iverilog"), ("vvp", "epitome-vvp")]:
    link = shims / name
    if link.is_symlink() or link.exists():
        link.unlink()
    link.symlink_to(bin_dir / target)
os.environ["PATH"] = f"{shims}{os.pathsep}{os.environ['PATH']}"
# Keep the source directory clean.
os.environ["PYTHONDONTWRITEBYTECODE"] = "1"

runner = get_runner("icarus")
runner.build(
    sources=[here / "fifo.sv"],
    hdl_toplevel="fifo",
    parameters={"DEPTH": 8, "WIDTH": 16},
    timescale=("1ns", "1ps"),
    build_dir=build_dir,
    always=True,
)
results = runner.test(
    hdl_toplevel="fifo",
    test_module="test_fifo",
    test_dir=here,
    build_dir=build_dir,
    results_xml=str(build_dir / "results.xml"),
)
tests, failed = get_results(results)
print(f"run.py: {tests} tests, {failed} failed")
sys.exit(1 if failed or not tests else 0)
