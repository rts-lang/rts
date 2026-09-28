#!/usr/bin/env bun
/**
  RTS build script for x86-64
 
  Usage:
    bun run build.ts            # debug
    bun run build.ts --release  # release
*/
// =====================================================================================================================

import { $ } from "bun";
import { existsSync, unlinkSync, renameSync } from "fs";
import { join, resolve } from "path";

// =====================================================================================================================

const isRelease: boolean = process.argv.includes("--release");

console.clear();

const scriptDir: string = import.meta.dir;
const rootDir: string = resolve(scriptDir, "..");

process.chdir(rootDir);

console.log(`[build] ${isRelease ? "release" : "debug"}`);

const result = isRelease
	? await $`CFLAGS= CXXFLAGS= CPPFLAGS= cargo build --release`.quiet().nothrow()
	: await $`CFLAGS= CXXFLAGS= CPPFLAGS= cargo build`.quiet().nothrow();

if (result.exitCode !== 0) {
	console.log(result.stderr.toString() + result.stdout.toString());
	process.exit(1);
}

console.log(`[linux-x86-64] Everything is fine`);

const outputPath: string = isRelease
	? join(rootDir, "target/release/rts")
	: join(rootDir, "target/debug/rts");

if (!existsSync(outputPath)) {
	console.error(`[build] ERROR: binary not found at ${outputPath}`);
	process.exit(1);
}

// =====================================================================================================================

// optimize
await $`strip ${outputPath}`.nothrow();

// Здесь также могло быть сжатие - но оно режет скорость работы.

const dest: string = join(scriptDir, "rts");

if (existsSync(dest)) {
	unlinkSync(dest);
}

renameSync(outputPath, dest);
console.log(`[build] binary → ${dest}`);

// =====================================================================================================================
