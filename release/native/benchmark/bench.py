from __future__ import annotations

import os
import resource
import statistics
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Tuple

HERE = Path(__file__).resolve().parent
RELEASE = HERE.parent.parent
RUNS = int(os.environ.get("RUNS", "5"))
RTS_BIN = Path(os.environ.get("RTS", RELEASE / "rts"))

MetricMap = Dict[str, List[float]]

def run(cmd: List[str], cwd: Path) -> Tuple[float, int, Dict[str, float]]:
  before = resource.getrusage(resource.RUSAGE_CHILDREN)

  t0 = time.perf_counter()

  process = subprocess.run(
    cmd,
    cwd=str(cwd),
    stdout=subprocess.DEVNULL,
    stderr=subprocess.DEVNULL,
  )

  wall = time.perf_counter() - t0

  after = resource.getrusage(resource.RUSAGE_CHILDREN)

  metrics = {
    "userCpu": after.ru_utime - before.ru_utime,
    "systemCpu": after.ru_stime - before.ru_stime,
    "rssKb": after.ru_maxrss,
    "minorFaults": after.ru_minflt - before.ru_minflt,
    "majorFaults": after.ru_majflt - before.ru_majflt,
    "voluntaryCs": after.ru_nvcsw - before.ru_nvcsw,
    "involuntaryCs": after.ru_nivcsw - before.ru_nivcsw,
    "fsInput": after.ru_inblock - before.ru_inblock,
    "fsOutput": after.ru_oublock - before.ru_oublock,
    "signals": after.ru_nsignals - before.ru_nsignals
  }

  return wall, process.returncode, metrics

def series(
  name: str,
  cmd: List[str],
  cwd: Path
) -> Tuple[List[float], MetricMap]:
  walls: List[float] = []
  metrics: MetricMap = {}

  for index in range(RUNS):
    wall, code, runMetrics = run(cmd, cwd)

    if code != 0:
      print(
          f"FAIL {name} run {index + 1}: exit {code}",
          file=sys.stderr,
      )
      sys.exit(1)

    walls.append(wall)

    for key, value in runMetrics.items():
      metrics.setdefault(key, []).append(value)

  return walls, metrics

def median(values: List[float]) -> float:
  return statistics.median(values)

def fmtMs(values: List[float]) -> str:
  best = min(values) * 1000
  mean = statistics.mean(values) * 1000
  std = statistics.stdev(values) * 1000 if len(values) > 1 else 0.0

  return f"{best:8.2f} / {mean:8.2f} ± {std:6.2f}"

def fmtMsValue(value: float) -> str:
  return f"{value * 1000:8.2f}ms"

def fmtMiB(value: float) -> str:
  return f"{value / 1024:8.1f}M"

def fmtCount(value: float) -> str:
  return f"{value:8.0f}"

def fmtRatio(base: float, value: float) -> str:
  if base == 0:
      return "-"

  return f"{value / base:6.2f}x"

def printMetric(
  name: str,
  rtsValue: float,
  pyValue: float,
  formatter
) -> None:
  print(
    f"{name:24}"
    f"{formatter(rtsValue):>12}"
    f"{formatter(pyValue):>12}"
    f"{fmtRatio(rtsValue, pyValue):>10}"
  )

def main() -> int:
  if not RTS_BIN.is_file():
    print(f"rts not found: {RTS_BIN}", file=sys.stderr)
    return 1

  libbench = HERE / "libbench.so"

  if not libbench.is_file():
    print("libbench.so not found", file=sys.stderr)
    return 1

  rtsT, rtsMetrics = series(
    "RTS",
    [
      str(RTS_BIN),
      "run",
      str(HERE / "main.rt"),
    ],
    HERE
  )

  pyT, pyMetrics = series(
    "ctypes",
    [
      sys.executable,
      str(HERE / "main.py"),
    ],
    HERE
  )

  rtsMean = statistics.mean(rtsT)
  pyMean = statistics.mean(pyT)

  ratio = pyMean / rtsMean if rtsMean else 0.0
  overhead = ((pyMean / rtsMean) - 1.0) * 100 if rtsMean else 0.0

  print()
  print("1. WALL-CLOCK: real time of the complete startup and execution of the process.")
  print()
  print(
    f"{'':12} "
    f"{'best / mean ± std (ms)':>28}   "
    f"{'ratio':>8}"
  )
  print(
    f"{'-' * 12} "
    f"{'-' * 28}   "
    f"{'-' * 8}"
  )
  print(
    f"{'RTS FFI':12} "
    f"{fmtMs(rtsT):>28}   "
    f"{'1.00x':>8}"
  )
  print(
    f"{'ctypes':12} "
    f"{fmtMs(pyT):>28}   "
    f"{ratio:7.2f}x"
  )

  print()
  print(f"ctypes overhead: {overhead:+.1f}%")

  print()
  print()
  print("2. PROCESS RESOURCES: CPU and system resources of the child process;")
  print("CPU time does not include the time the process spends waiting.")
  print()

  print(
    f"{'metric':24}"
    f"{'RTS FFI':>12}"
    f"{'ctypes':>12}"
    f"{'ratio':>10}"
  )
  print("-" * 62)

  printMetric(
    "User CPU",
    median(rtsMetrics["userCpu"]),
    median(pyMetrics["userCpu"]),
    fmtMsValue
  )

  printMetric(
    "System CPU",
    median(rtsMetrics["systemCpu"]),
    median(pyMetrics["systemCpu"]),
    fmtMsValue
  )

  rtsTotalCpu = (
    median(rtsMetrics["userCpu"])
    + median(rtsMetrics["systemCpu"])
  )
  pyTotalCpu = (
    median(pyMetrics["userCpu"])
    + median(pyMetrics["systemCpu"])
  )

  printMetric(
    "Total CPU",
    rtsTotalCpu,
    pyTotalCpu,
    fmtMsValue
  )

  printMetric(
    "RSS",
    median(rtsMetrics["rssKb"]),
    median(pyMetrics["rssKb"]),
    fmtMiB
  )

  printMetric(
    "Minor page faults",
    median(rtsMetrics["minorFaults"]),
    median(pyMetrics["minorFaults"]),
    fmtCount
  )

  printMetric(
    "Major page faults",
    median(rtsMetrics["majorFaults"]),
    median(pyMetrics["majorFaults"]),
    fmtCount
  )

  printMetric(
    "Voluntary ctx",
    median(rtsMetrics["voluntaryCs"]),
    median(pyMetrics["voluntaryCs"]),
    fmtCount
  )

  printMetric(
    "Involuntary ctx",
    median(rtsMetrics["involuntaryCs"]),
    median(pyMetrics["involuntaryCs"]),
    fmtCount
  )

  print()
  return 0

if __name__ == "__main__":
  sys.exit(main())