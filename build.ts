#!/usr/bin/env bun
/**
  Entry runner: builds what the entry needs, then runs it.

  Usage:
    bun run release [entry...] [--release] [-- programArgs]
    bun run tools   [entry...] [--release] [-- programArgs]

  No entry: only the build step (rts for release, wasm pkg for tools).

  release:
    folder      -> <folder>/main.rt   (rts run)
    file.rt     -> rts run file.rt
    file.ts     -> bun file.ts
    build       rts is built once, only if an .rt entry is present;
                every x.c in the entry folder becomes libx.so in the same folder;
                cwd = entry folder (importNative("./libx.so"))
    paths       from release/, then from repo root

  tools:
    folder      -> <folder>/main.ts
    file.ts     -> bun file.ts (--release is forwarded to it)
    build       wasm pkg via tools/build.ts before every run; cwd = repo root
    paths       from tools/, then from repo root

  Patterns (*, ?) are expanded by us, quote them or not:
    bun run release "native/*"
    bun run release native/types/*.rt
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

type Profile = {
  baseDir: string;
  folderEntry: string;
  runners: Record<string, Runner>;
  native: boolean; // x.c -> libx.so next to the entry
  forwardRelease: boolean; // pass --release to the entry
  entryCwd: (entry: Entry) => string;
};

const rootDir: string = import.meta.dir;
const releaseDir: string = join(rootDir, "release");
const toolsDir: string = join(rootDir, "tools");
const rtsBin: string = join(releaseDir, "rts");

const profiles: Record<string, Profile> = {
  release: {
    baseDir: releaseDir,
    folderEntry: "main.rt",
    runners: { ".rt": "rts", ".ts": "bun" },
    native: true,
    forwardRelease: false,
    entryCwd: (entry: Entry): string => entry.dir,
  },
  tools: {
    baseDir: toolsDir,
    folderEntry: "main.ts",
    runners: { ".ts": "bun" },
    native: false,
    forwardRelease: true,
    entryCwd: (): string => rootDir,
  },
};

const argv: string[] = process.argv.slice(2);
const profileName: string = argv[0] ?? "";
const profile: Profile | undefined = profiles[profileName];
if (!profile) {
  console.error("usage: bun ./build.ts <release|tools> [entry...] [--release] [-- programArgs]");
  process.exit(2);
}

const rest: string[] = argv.slice(1);
const dashIndex: number = rest.indexOf("--");
const ownArgs: string[] = dashIndex === -1 ? rest : rest.slice(0, dashIndex);
const programArgs: string[] = dashIndex === -1 ? [] : rest.slice(dashIndex + 1);

const isRelease: boolean = ownArgs.includes("--release");
const inputs: string[] = ownArgs.filter((a: string) => !a.startsWith("-"));
const unknownFlags: string[] = ownArgs.filter(
  (a: string) => a.startsWith("-") && a !== "--release"
);

if (unknownFlags.length > 0) {
  console.error(`error: unknown flag ${unknownFlags.join(" ")}`);
  console.error(`usage: bun run ${profileName} [entry...] [--release] [-- programArgs]`);
  process.exit(2);
}

// -------------------------------------------------------------------------------------------------

function isDir(path: string): boolean {
  return existsSync(path) && statSync(path).isDirectory();
}

/** Path from the profile folder (release/ or tools/), then from repo root */
function locate(input: string): string | null {
  for (const base of [profile!.baseDir, rootDir]) {
    const path: string = resolve(base, input);
    if (existsSync(path)) return path;
  }
  return null;
}

function expandPattern(pattern: string): string[] {
  for (const base of [profile!.baseDir, rootDir]) {
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
    file = join(path, profile!.folderEntry);
    if (!existsSync(file)) return `no ${profile!.folderEntry}, pass a file`;
  }

  const ext: string = extname(file);
  const runner: Runner | undefined = profile!.runners[ext];
  if (!runner) return `unsupported ${ext || "file"}`;

  return { name: relative(rootDir, file), file, dir: dirname(file), runner };
}

function collect(): Entry[] {
  const entries: Entry[] = [];
  let failed: boolean = false;

  for (const input of inputs) {
    if (/[*?]/.test(input)) {
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
  if (inputs.length > 0 && entries.length === 0) {
    console.error("error: no runnable entries");
    process.exit(1);
  }
  return entries;
}

async function runBuildScript(script: string, label: string): Promise<void> {
  const args: string[] = isRelease ? ["--release"] : [];
  const proc = Bun.spawn([process.execPath, script, ...args], {
    stdout: "inherit",
    stderr: "inherit",
  });
  if ((await proc.exited) !== 0) {
    console.error(`error: ${label} build failed`);
    process.exit(1);
  }
}

async function buildRts(): Promise<void> {
  await runBuildScript(join(releaseDir, "build.ts"), "rts");
  if (!existsSync(rtsBin)) {
    console.error("error: release/rts missing after build");
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

async function prepare(entries: Entry[]): Promise<void> {
  if (profileName === "tools") {
    await runBuildScript(join(toolsDir, "build.ts"), "wasm");
    return;
  }

  const needRts: boolean =
    entries.length === 0 || entries.some((e: Entry) => e.runner === "rts");
  if (needRts) await buildRts();

  if (profile!.native) {
    for (const dir of new Set(entries.map((e: Entry) => e.dir))) {
      await buildNative(dir);
    }
  }
}

async function run(entry: Entry): Promise<number> {
  const forwarded: string[] =
    profile!.forwardRelease && isRelease ? ["--release"] : [];

  const cmd: string[] =
    entry.runner === "rts"
      ? [rtsBin, "run", entry.file, ...programArgs]
      : [process.execPath, entry.file, ...forwarded, ...programArgs];

  const proc = Bun.spawn(cmd, {
    cwd: profile!.entryCwd(entry),
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  });
  return await proc.exited;
}

// -------------------------------------------------------------------------------------------------

const entries: Entry[] = collect();
await prepare(entries);

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
