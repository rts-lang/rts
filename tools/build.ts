#!/usr/bin/env bun
/**
  Build WASM package for tools.

  Usage:
    bun run tools/build.ts            # debug
    bun run tools/build.ts --release  # release

  todo Не понятно по wasm-pack, соберётся ли он нормально вне Linux.
    Возможно ошибки путей или автоматизации. Надо подумать.

  todo Еще надо комментарии обычные по стадиям.
*/
// =====================================================================================================================

import { $ } from "bun";
import { existsSync, statSync } from "fs";
import { join, resolve } from "path";
import * as readline from "readline";

// =====================================================================================================================

const rootDir: string = resolve(import.meta.dir, "..");
const pkgDir: string = join(rootDir, "pkg");
const pkgJs: string = join(pkgDir, "rts.js");
const pkgWasm: string = join(pkgDir, "rts_bg.wasm");

const args: string[] = process.argv.slice(2);
const isRelease: boolean = args.includes("--release");

function ask(question: string): Promise<boolean> {
  if (!process.stdin.isTTY) return Promise.resolve(false);

  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  return new Promise((resolveAsk) => {
    rl.question(question, (answer: string) => {
      rl.close();
      const a: string = answer.trim().toLowerCase();
      resolveAsk(a === "y" || a === "yes");
    });
  });
}

/** Parse wasm-pack stdout/stderr for newer version warning */
function parseWasmPackUpdate(text: string): { latest: string; current: string } | null {
  const m = text.match(
    /newer version of wasm-pack available[^]*?new version is:\s*([\d.]+)[^]*?you are using:\s*([\d.]+)/i
  );
  if (!m) return null;
  return { latest: m[1], current: m[2] };
}

async function maybeUpdateWasmPack(buildOutput: string): Promise<void> {
  const info = parseWasmPackUpdate(buildOutput);
  if (!info) return;

  console.log(
    `[tools/build] wasm-pack ${info.current} → available ${info.latest}`
  );

  const ok: boolean = await ask(
    `[tools/build] Update wasm-pack to ${info.latest}? [y/N] `
  );

  if (!ok) {
    console.log("[tools/build] skip wasm-pack update");
    return;
  }

  console.log("[tools/build] cargo install wasm-pack --force ...");
  const install = await $`cargo install wasm-pack --force`.nothrow();
  if (install.exitCode !== 0) {
    console.error("[tools/build] wasm-pack update failed");
    console.error(install.stderr.toString());
    return;
  }
  console.log(`[tools/build] wasm-pack updated to ${info.latest}`);
}

/** Install wasm-pack through cargo if it is missing (no prompt, works without TTY) */
async function ensureWasmPack(): Promise<boolean> {
  // cargo bin dir may be absent from PATH of the current process
  const cargoBin: string = join(process.env.CARGO_HOME ?? join(process.env.HOME ?? "", ".cargo"), "bin");
  if (!(process.env.PATH ?? "").split(":").includes(cargoBin)) {
    process.env.PATH = `${cargoBin}:${process.env.PATH ?? ""}`;
  }

  const which = await $`which wasm-pack`.quiet().nothrow();
  if (which.exitCode === 0) return true;

  const cargo = await $`which cargo`.quiet().nothrow();
  if (cargo.exitCode !== 0) {
    console.error(
      "[tools/build] wasm-pack not found and cargo is missing.\n" +
        "  Install Rust first: https://rustup.rs"
    );
    return false;
  }

  console.log("[tools/build] wasm-pack not found, cargo install wasm-pack ...");
  const install = await $`cargo install wasm-pack`.nothrow();
  if (install.exitCode !== 0) {
    console.error("[tools/build] wasm-pack install failed");
    console.error(install.stderr.toString());
    console.error("  Manual install: cargo install wasm-pack");
    return false;
  }

  const recheck = await $`which wasm-pack`.quiet().nothrow();
  if (recheck.exitCode !== 0) {
    console.error(`[tools/build] wasm-pack installed, but not found in PATH (${cargoBin})`);
    return false;
  }
  console.log("[tools/build] wasm-pack installed");
  return true;
}

/** Make sure wasm32-unknown-unknown std is available (rustup, or a manually installed sysroot) */
async function ensureWasmTarget(): Promise<boolean> {
  const rustup = await $`which rustup`.quiet().nothrow();
  if (rustup.exitCode === 0) {
    const check = await $`rustup target list --installed`.quiet().nothrow();
    if (check.exitCode === 0 && check.stdout.toString().includes("wasm32-unknown-unknown")) return true;

    console.log("[tools/build] adding rustup target wasm32-unknown-unknown ...");
    const add = await $`rustup target add wasm32-unknown-unknown`.nothrow();
    if (add.exitCode !== 0) {
      console.error("[tools/build] failed to add wasm32-unknown-unknown");
      return false;
    }
    return true;
  }

  // No rustup: the target must already be in the sysroot
  const sysroot = await $`rustc --print sysroot`.quiet().nothrow();
  const root: string = sysroot.stdout.toString().trim();
  if (sysroot.exitCode === 0 && existsSync(join(root, "lib", "rustlib", "wasm32-unknown-unknown"))) return true;

  console.error(
    "[tools/build] rustup not found and wasm32-unknown-unknown is not in the rustc sysroot.\n" +
      "  Install rustup (https://rustup.rs) or add the wasm32-unknown-unknown rust-std component."
  );
  return false;
}

export async function ensureWasm(options?: {
  release?: boolean;
}): Promise<boolean> {
  const release: boolean = options?.release ?? isRelease;

  if (!(await ensureWasmPack())) return false;

  if (!(await ensureWasmTarget())) return false;

  process.chdir(rootDir);

  const profileFlag: string = release ? "--release" : "--dev";
  console.log(
    `[tools/build] wasm-pack build --target web ${profileFlag} --features analyzer --no-default-features`
  );

  const result =
    await $`wasm-pack build --target web ${profileFlag} --features analyzer --no-default-features`.nothrow();

  const combined: string =
    result.stdout.toString() + "\n" + result.stderr.toString();

  await maybeUpdateWasmPack(combined);

  if (result.exitCode !== 0) {
    console.error("[tools/build] wasm-pack failed");
    console.error(result.stderr.toString());
    console.error(result.stdout.toString());
    return false;
  }

  if (!existsSync(pkgJs) || !existsSync(pkgWasm)) {
    console.error(`[tools/build] expected ${pkgJs} and ${pkgWasm} after build`);
    return false;
  }

  const wasmSize: number = statSync(pkgWasm).size;
  console.log(`[tools/build] ok → ${pkgDir} (${wasmSize} bytes wasm)`);
  return true;
}

// =====================================================================================================================

if (import.meta.main) {
  const ok: boolean = await ensureWasm({ release: isRelease });
  if (!ok) process.exit(1);
}

// =====================================================================================================================
