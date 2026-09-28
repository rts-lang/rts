#!/usr/bin/env bun
/**
  RTS run script
 
  Usage:
    ./run.ts <file.rt>            # debug build + run
    ./run.ts --release <file.rt>  # release build + run
*/
// =====================================================================================================================

import { $ } from "bun";
import { existsSync, rmSync } from "fs";
import { join } from "path";

// =====================================================================================================================

const args: string[] = process.argv.slice(2);
const isRelease: boolean = args.includes("--release");
const fileArg: string | undefined = args.find((arg: string) => !arg.startsWith("-"));

// =====================================================================================================================

console.clear();

const releaseDir: string = import.meta.dir;
process.chdir(releaseDir);

const rtsBinary: string = join(releaseDir, "rts");

// =====================================================================================================================

// remove previous binary
if (existsSync(rtsBinary)) {
  rmSync(rtsBinary, { force: true });
}

// =====================================================================================================================
// build

const buildScript: string = join(releaseDir, "build.ts");
const buildArgs: string[] = isRelease ? ["--release"] : [];

const buildResult = await $`bun run ${buildScript} ${buildArgs}`.nothrow();

if (buildResult.exitCode !== 0) {
  console.log("[run] Skipped (build failed)");
  process.exit(1);
}

if (!existsSync(rtsBinary)) {
  console.error("[run] ERROR: binary was not created");
  process.exit(1);
}

// =====================================================================================================================

// always run
await $`./rts run ${fileArg}`;

// =====================================================================================================================
