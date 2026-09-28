#!/usr/bin/env bun
/**
  RTS entry runner: builds what the entry needs, then runs it.

  Usage:
    bun run release <entry>... [--release] [-- programArgs]

  Entry:
    folder      -> <folder>/main.rt   (rts run)
    file.rt     -> rts run file.rt
    file.ts     -> bun file.ts

  Paths are resolved from the repo root, then from release/.
  Patterns (*, ?) are expanded by us, quote them or not:
    bun run release "native/*"
    bun run release native/types/*.rt

  Build convention:
    rts is built once, only if an .rt entry is present (--release for release);
    every x.c in the entry folder becomes libx.so in the same folder.
*/

import { existsSync, readdirSync, statSync } from "fs";
import { basename, dirname, extname, join, relative, resolve } from "path";

type Runner = "rts" | "bun";

type Entry = {
  name: string;
  file: string;
  dir: string;
  runner: Runner;
};

const rootDir: string = import.meta.dir;
const releaseDir: string = join(rootDir, "release");
const rtsBin: string = join(releaseDir, "rts");
const buildScript: string = join(releaseDir, "build.ts");

const argv: string[] = process.argv.slice(2);
const dashIndex: number = argv.indexOf("--");
const ownArgs: string[] = dashIndex === -1 ? argv : argv.slice(0, dashIndex);
const programArgs: string[] = dashIndex === -1 ? [] : argv.slice(dashIndex + 1);

const isRelease: boolean = ownArgs.includes("--release");
const inputs: string[] = ownArgs.filter((a: string) => !a.startsWith("-"));
const unknownFlags: string[] = ownArgs.filter(
  (a: string) => a.startsWith("-") && a !== "--release"
);

if (inputs.length === 0 || unknownFlags.length > 0) {
  if (unknownFlags.length > 0) {
    console.error(`error: unknown flag ${unknownFlags.join(" ")}`);
  }
  console.error("usage: bun run release <entry>... [--release] [-- programArgs]");
  process.exit(2);
}

// -------------------------------------------------------------------------------------------------

function isDir(path: string): boolean {
  return existsSync(path) && statSync(path).isDirectory();
}

/** Path from repo root, then from release/ */
function locate(input: string): string | null {
  for (const base of [rootDir, releaseDir]) {
    const path: string = resolve(base, input);
    if (existsSync(path)) return path;
  }
  return null;
}

function expandPattern(pattern: string): string[] {
  for (const base of [rootDir, releaseDir]) {
    const glob = new Bun.Glob(pattern);
    const found: string[] = [...glob.scanSync({ cwd: base, onlyFiles: false })];
    if (found.length > 0) return found.sort().map((p: string) => resolve(base, p));
  }
  return [];
}

/** Returns Entry, or a string with the reason it is not runnable */
function toEntry(path: string): Entry | string {
  let file: string = path;
  if (isDir(path)) {
    file = join(path, "main.rt");
    if (!existsSync(file)) return "no main.rt, pass a file";
  }

  const ext: string = extname(file);
  if (ext !== ".rt" && ext !== ".ts") return `unsupported ${ext || "file"}`;

  return {
    name: relative(rootDir, file),
    file,
    dir: dirname(file),
    runner: ext === ".rt" ? "rts" : "bun",
  };
}

function collect(): Entry[] {
  const entries: Entry[] = [];
  let failed: boolean = false;

  for (const input of inputs) {
    const isPattern: boolean = /[*?]/.test(input);

    if (isPattern) {
      const matches: string[] = expandPattern(input);
      if (matches.length === 0) {
        console.error(`error: nothing matches ${input}`);
        failed = true;
      }
      for (const path of matches) {
        const entry: Entry | string = toEntry(path);
        // patterns are lenient: not runnable matches (print.c, libx.so) are skipped
        if (typeof entry !== "string") entries.push(entry);
      }
      continue;
    }

    const path: string | null = locate(input);
    if (path === null) {
      console.error(`error: not found: ${input}`);
      failed = true;
      continue;
    }
    const entry: Entry | string = toEntry(path);
    if (typeof entry === "string") {
      console.error(`error: ${input}: ${entry}`);
      failed = true;
      continue;
    }
    entries.push(entry);
  }

  if (failed) process.exit(1);
  if (entries.length === 0) {
    console.error("error: no runnable entries");
    process.exit(1);
  }
  return entries;
}

async function buildRts(): Promise<void> {
  const args: string[] = isRelease ? ["--release"] : [];
  const proc = Bun.spawn([process.execPath, buildScript, ...args], {
    stdout: "inherit",
    stderr: "inherit",
  });
  if ((await proc.exited) !== 0 || !existsSync(rtsBin)) {
    console.error("error: rts build failed");
    process.exit(1);
  }
}

/** every x.c in dir -> libx.so in dir, only if the .c is newer */
async function buildNative(dir: string): Promise<void> {
  const sources: string[] = readdirSync(dir).filter((f: string) => f.endsWith(".c"));
  if (sources.length === 0) return;

  const cc: string | null =
    Bun.which("clang") ?? Bun.which("cc") ?? Bun.which("gcc");
  if (!cc) {
    console.error("error: clang/cc/gcc not found");
    process.exit(1);
  }

  for (const source of sources) {
    const src: string = join(dir, source);
    const out: string = join(dir, `lib${basename(source, ".c")}.so`);
    if (existsSync(out) && statSync(out).mtimeMs >= statSync(src).mtimeMs) continue;

    const proc = Bun.spawn([cc, "-shared", "-fPIC", "-O2", "-o", out, src], {
      stdout: "inherit",
      stderr: "inherit",
    });
    if ((await proc.exited) !== 0) {
      console.error(`error: ${relative(rootDir, src)} build failed`);
      process.exit(1);
    }
    console.log(`[native] ${relative(rootDir, src)} -> ${basename(out)}`);
  }
}

async function run(entry: Entry): Promise<number> {
  const cmd: string[] =
    entry.runner === "rts"
      ? [rtsBin, "run", entry.file, ...programArgs]
      : [process.execPath, entry.file, ...programArgs];

  const proc = Bun.spawn(cmd, {
    cwd: entry.dir, // relative importNative("./libx.so") resolves here
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  });
  return await proc.exited;
}

// -------------------------------------------------------------------------------------------------

const entries: Entry[] = collect();

if (entries.some((e: Entry) => e.runner === "rts")) {
  await buildRts();
}
for (const dir of new Set(entries.map((e: Entry) => e.dir))) {
  await buildNative(dir);
}

const failedNames: string[] = [];
for (const entry of entries) {
  if (entries.length > 1) console.log(`\n[run] ${entry.name}`);
  const code: number = await run(entry);
  if (code !== 0) {
    failedNames.push(entry.name);
    if (entries.length === 1) process.exit(code);
  }
}

if (entries.length > 1) {
  console.log(`\n[done] ${entries.length - failedNames.length}/${entries.length} ok`);
  for (const name of failedNames) console.log(`  FAIL ${name}`);
}
process.exit(failedNames.length > 0 ? 1 : 0);
