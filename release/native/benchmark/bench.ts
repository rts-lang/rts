#!/usr/bin/env bun
/**
  RTS FFI vs Python ctypes vs TypeScript (bun:ffi) benchmark (Linux only)

  Usage (from release/):
  ./native/benchmark/bench.ts
  ./native/benchmark/bench.ts 100

  Requires: bun, clang|cc|gcc, python3, glibc.

  todo Еще надо комментарии обычные по стадиям.
*/
// =====================================================================================================================

import { $ } from "bun";
import { dlopen, FFIType, ptr } from "bun:ffi";
import { existsSync, rmSync } from "fs";
import { join, resolve } from "path";
import { platform, tmpdir } from "os";

// =====================================================================================================================

if (platform() !== "linux") {
  console.error("error: linux only");
  process.exit(1);
}

const arg: string | undefined = process.argv[2];
if (arg !== undefined && !/^\d+$/.test(arg)) {
  console.error("usage: ./native/benchmark/bench.ts [runs]");
  process.exit(1);
}
const Runs: number =
  arg !== undefined ? Number(arg) : Number(process.env.Runs ?? "100");

// =====================================================================================================================

const here: string = import.meta.dir;
const releaseDir: string = resolve(here, "../..");
const rtsBin: string = join(releaseDir, "rts");
const libOut: string = join(here, "libbench.so");
const mainRt: string = join(here, "main.rt");
const mainPy: string = join(here, "main.py");
const mainTs: string = join(here, "main.ts");
const benchC: string = join(here, "bench.c");
const probeSrc: string = join(tmpdir(), `rtsBenchProbe${process.pid}.c`);
const probeBin: string = join(tmpdir(), `rtsBenchProbe${process.pid}`);

// Tiny spawner: its own RSS is ~1 MiB, so wait4() reports a clean child peak.
// Prints ru_maxrss (KiB) of the child.
const probeC: string = `
#include <fcntl.h>
#include <spawn.h>
#include <stdio.h>
#include <sys/resource.h>
#include <sys/wait.h>
extern char **environ;
int main(int argc, char **argv) {
  posix_spawn_file_actions_t fa;
  posix_spawn_file_actions_init(&fa);
  posix_spawn_file_actions_addopen(&fa, 0, "/dev/null", O_RDONLY, 0);
  posix_spawn_file_actions_addopen(&fa, 1, "/dev/null", O_WRONLY, 0);
  posix_spawn_file_actions_addopen(&fa, 2, "/dev/null", O_WRONLY, 0);
  pid_t pid;
  if (argc < 2 || posix_spawnp(&pid, argv[1], &fa, 0, argv + 1, environ)) return 127;
  int st;
  struct rusage ru;
  wait4(pid, &st, 0, &ru);
  printf("%ld\\n", ru.ru_maxrss);
  return WIFEXITED(st) ? WEXITSTATUS(st) : 128;
}
`;

// =====================================================================================================================

type Series = {
  walls: number[];
  userCpu: number[]; // seconds
  systemCpu: number[];
  voluntaryCs: number[];
  involuntaryCs: number[];
  rssKb: number[];
  minorFaults: number[];
  majorFaults: number[];
};

/**
  Same approach as the old bench.py: getrusage(RUSAGE_CHILDREN) before/after
  each run, called through bun:ffi (no deps). Exact for CPU, ctx, page faults.

  RSS is NOT taken from it. ru_maxrss is a running max and, on Linux, survives
  exec, so a child spawned by Bun (or Python: old bench.py showed its ~11 MiB
  floor as "RTS RSS") reports max(spawner peak, own peak).
  RSS comes from an inline C spawner (probeC), see memory().
  https://github.com/oven-sh/bun/issues/43305
*/
const libc = dlopen("libc.so.6", {
  getrusage: { args: [FFIType.i32, FFIType.ptr], returns: FFIType.i32 },
});
const ruBuf: BigInt64Array = new BigInt64Array(18); // struct rusage, x86-64/aarch64 glibc
const RUSAGE_CHILDREN: number = -1;

function childrenUsage(): number[] {
  libc.symbols.getrusage(RUSAGE_CHILDREN, ptr(ruBuf));
  return Array.from(ruBuf, Number);
}

async function timed(
  cmd: string[],
  cwd: string
): Promise<{ wall: number; code: number; before: number[]; after: number[] }> {
  const before: number[] = childrenUsage();
  const t0: number = performance.now();
  const proc = Bun.spawn(cmd, {
    cwd,
    stdout: "ignore",
    stderr: "ignore",
    stdin: "ignore",
  });
  const code: number = await proc.exited;
  const wall: number = (performance.now() - t0) / 1000;
  return { wall, code, before, after: childrenUsage() };
}

async function memory(cmd: string[], cwd: string): Promise<number> {
  const proc = Bun.spawn([probeBin, ...cmd], {
    cwd,
    stdout: "pipe",
    stderr: "ignore",
    stdin: "ignore",
  });
  const text: string = await new Response(proc.stdout).text();
  const code: number = await proc.exited;
  const kb: number = Number(text.trim());
  if (code !== 0 || Number.isNaN(kb)) {
    console.error(`FAIL probe: exit ${code}`);
    process.exit(1);
  }
  return kb;
}

async function series(
  name: string,
  cmd: string[],
  cwd: string
): Promise<Series> {
  const out: Series = {
    walls: [],
    userCpu: [],
    systemCpu: [],
    voluntaryCs: [],
    involuntaryCs: [],
    rssKb: [],
    minorFaults: [],
    majorFaults: [],
  };

  await timed(cmd, cwd); // warmup: page cache, dynamic linker

  for (let i: number = 0; i < Runs; i++) {
    const { wall, code, before: a, after: b } = await timed(cmd, cwd);
    if (code !== 0) {
      console.error(`FAIL ${name} run ${i + 1}: exit ${code}`);
      process.exit(1);
    }
    out.walls.push(wall);
    // indexes: 0/1 utime s/us, 2/3 stime s/us, 8 minflt, 9 majflt, 16 nvcsw, 17 nivcsw
    out.userCpu.push(b[0]! - a[0]! + (b[1]! - a[1]!) / 1e6);
    out.systemCpu.push(b[2]! - a[2]! + (b[3]! - a[3]!) / 1e6);
    out.minorFaults.push(b[8]! - a[8]!);
    out.majorFaults.push(b[9]! - a[9]!);
    out.voluntaryCs.push(b[16]! - a[16]!);
    out.involuntaryCs.push(b[17]! - a[17]!);
  }

  for (let i: number = 0; i < Runs; i++) {
    out.rssKb.push(await memory(cmd, cwd));
  }

  return out;
}

function mean(values: number[]): number {
  return values.reduce((a, b) => a + b, 0) / values.length;
}

function median(values: number[]): number {
  const s: number[] = [...values].sort((a, b) => a - b);
  const mid: number = Math.floor(s.length / 2);
  return s.length % 2 === 0 ? (s[mid - 1]! + s[mid]!) / 2 : s[mid]!;
}

function stdev(values: number[]): number {
  if (values.length < 2) return 0;
  const m: number = mean(values);
  const v: number =
    values.reduce((acc, x) => acc + (x - m) ** 2, 0) / (values.length - 1);
  return Math.sqrt(v);
}

function fmtMs(values: number[]): string {
  const best: number = Math.min(...values) * 1000;
  const m: number = mean(values) * 1000;
  const s: number = stdev(values) * 1000;
  return `${best.toFixed(2).padStart(8)} / ${m.toFixed(2).padStart(8)} ± ${s.toFixed(2).padStart(6)}`;
}

function fmtMsValue(value: number): string {
  return `${(value * 1000).toFixed(2).padStart(8)}ms`;
}

function fmtMiB(value: number): string {
  return `${(value / 1024).toFixed(1).padStart(8)}M`;
}

function fmtCount(value: number): string {
  return `${Math.round(value).toString().padStart(8)}`;
}

function fmtRatio(base: number, value: number): string {
  if (base === 0 || Number.isNaN(base) || Number.isNaN(value)) return "-";
  return `${(value / base).toFixed(2)}x`;
}

function row(
  name: string,
  rts: string,
  py: string,
  pyRatio: string,
  ts: string,
  tsRatio: string
): string {
  return (
    `${name.padEnd(19)} | ${rts.padStart(10)} | ` +
    `${py.padStart(10)} ${pyRatio.padStart(8)} | ` +
    `${ts.padStart(10)} ${tsRatio.padStart(8)}`
  );
}

function printMetric(
  name: string,
  rtsValues: number[],
  pyValues: number[],
  tsValues: number[],
  formatter: (v: number) => string
): void {
  const r: number = median(rtsValues);
  const p: number = median(pyValues);
  const t: number = median(tsValues);
  console.log(
    row(name, formatter(r), formatter(p), fmtRatio(r, p), formatter(t), fmtRatio(r, t))
  );
}

async function compile(cmd: string[]): Promise<boolean> {
  const proc = Bun.spawn(cmd, { stdout: "pipe", stderr: "pipe" });
  const [out, err] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ]);
  if ((await proc.exited) !== 0) {
    console.error(out + err);
    return false;
  }
  return true;
}

// =====================================================================================================================

async function main(): Promise<number> {
  process.chdir(releaseDir);

  console.log("[1/3] build rts");
  const buildScript: string = join(releaseDir, "build.ts");
  if (!existsSync(buildScript)) {
    console.error("error: release/build.ts missing");
    return 1;
  }
  const buildResult = await $`bun run ${buildScript} --release`.nothrow();
  if (buildResult.exitCode !== 0 || !existsSync(rtsBin)) {
    console.error("error: release/rts missing after build.ts");
    return 1;
  }

  console.log("[2/3] build libbench.so + rss probe");
  const cc: string | null =
    Bun.which("clang") ?? Bun.which("cc") ?? Bun.which("gcc");
  if (!cc) {
    console.error("error: clang/cc/gcc not found");
    return 1;
  }
  if (!(await compile([cc, "-shared", "-fPIC", "-O2", "-o", libOut, benchC]))) {
    console.error("error: libbench.so build failed");
    return 1;
  }

  await Bun.write(probeSrc, probeC);
  if (!(await compile([cc, "-O2", "-o", probeBin, probeSrc]))) {
    console.error("error: rss probe build failed");
    return 1;
  }

  console.log("[3/3] measure");
  console.log();
  console.log(`Runs: ${Runs}`);

  const python: string =
    Bun.which("python3") ?? Bun.which("python") ?? "python3";

  const rts: Series = await series("RTS", [rtsBin, "run", mainRt], here);
  const py: Series = await series("ctypes", [python, mainPy], here);
  const ts: Series = await series("TS FFI", [process.execPath, mainTs], here);
  rmSync(probeSrc, { force: true });
  rmSync(probeBin, { force: true });

  const rtsMean: number = mean(rts.walls);
  const rows: [string, Series][] = [
    ["RTS FFI", rts],
    ["ctypes", py],
    ["TS FFI", ts],
  ];

  console.log();
  console.log(
    "1. WALL-CLOCK: real time of the complete startup and execution of the process."
  );
  console.log();
  console.log(
    `${"".padEnd(12)} ${"best / mean ± std (ms)".padStart(28)}   ${"ratio".padStart(8)}`
  );
  console.log(`${"-".repeat(12)} ${"-".repeat(28)}   ${"-".repeat(8)}`);
  for (const [name, s] of rows) {
    const r: number = rtsMean ? mean(s.walls) / rtsMean : 0;
    console.log(
      `${name.padEnd(12)} ${fmtMs(s.walls).padStart(28)}   ${(r.toFixed(2) + "x").padStart(8)}`
    );
  }
  console.log();
  for (const [name, s] of rows.slice(1)) {
    const overhead: number = rtsMean ? (mean(s.walls) / rtsMean - 1) * 100 : 0;
    console.log(
      `${name} overhead: ${overhead >= 0 ? "+" : ""}${overhead.toFixed(1)}%`
    );
  }
  console.log();
  console.log();
  console.log(
    "2. PROCESS RESOURCES: CPU and system resources of the child process;"
  );
  console.log("CPU time does not include the time the process spends waiting.");
  console.log();
  console.log(row("metric", "RTS FFI", "ctypes", "ratio", "TS FFI", "ratio"));
  console.log(
    `${"-".repeat(20)}+${"-".repeat(12)}+${"-".repeat(21)}+${"-".repeat(20)}`
  );

  const total = (s: Series): number[] =>
    s.userCpu.map((v, i) => v + s.systemCpu[i]!);

  printMetric("User CPU", rts.userCpu, py.userCpu, ts.userCpu, fmtMsValue);
  printMetric("System CPU", rts.systemCpu, py.systemCpu, ts.systemCpu, fmtMsValue);
  printMetric("Total CPU", total(rts), total(py), total(ts), fmtMsValue);
  printMetric("RSS", rts.rssKb, py.rssKb, ts.rssKb, fmtMiB);
  printMetric("Minor page faults", rts.minorFaults, py.minorFaults, ts.minorFaults, fmtCount);
  printMetric("Major page faults", rts.majorFaults, py.majorFaults, ts.majorFaults, fmtCount);
  printMetric("Voluntary ctx", rts.voluntaryCs, py.voluntaryCs, ts.voluntaryCs, fmtCount);
  printMetric("Involuntary ctx", rts.involuntaryCs, py.involuntaryCs, ts.involuntaryCs, fmtCount);
  console.log();

  return 0;
}

// =====================================================================================================================

process.exit(await main());

// =====================================================================================================================
