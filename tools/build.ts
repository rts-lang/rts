#!/usr/bin/env bun
/**
  Build WASM package for tools (analyzer).
 
  From Cargo.toml:
    wasm-pack build --target web --features analyzer --no-default-features
 
  Usage:
    bun run tools/build.ts           # release wasm → ./pkg
    bun run tools/build.ts --dev     # dev profile (faster, larger)
    bun run tools/build.ts --force   # rebuild even if pkg looks ok

  todo Не понятно по wasm-pack, соберётся ли он нормально вне Linux.
    Возможно ошибки путей или автоматизации. Надо подумать.
 
  todo Еще надо комментарии обычные по стадиям.
*/

import { $ } from "bun";
import { existsSync, statSync } from "fs";
import { join, resolve } from "path";

const rootDir: string = resolve(import.meta.dir, "..");
const pkgDir: string = join(rootDir, "pkg");
const pkgJs: string = join(pkgDir, "rts.js");
const pkgWasm: string = join(pkgDir, "rts_bg.wasm");

const args: string[] = process.argv.slice(2);
const isDev: boolean = args.includes("--dev");

export async function ensureWasm(options?: {
  dev?: boolean;
}): Promise<boolean> {
  const dev: boolean = options?.dev ?? isDev;

  const which = await $`which wasm-pack`.quiet().nothrow();
  if (which.exitCode !== 0) {
    console.error(
      "[tools/build] wasm-pack not found.\n" +
      "  Install: cargo install wasm-pack"
    );
    return false;
  }

  const targetCheck = await $`rustup target list --installed`.quiet().nothrow();
  const hasWasmTarget: boolean =
    targetCheck.exitCode === 0 &&
    targetCheck.stdout.toString().includes("wasm32-unknown-unknown");

  if (!hasWasmTarget) {
    console.log("[tools/build] adding rustup target wasm32-unknown-unknown ...");
    const add = await $`rustup target add wasm32-unknown-unknown`.nothrow();
    if (add.exitCode !== 0) {
      console.error("[tools/build] failed to add wasm32-unknown-unknown");
      return false;
    }
  }

  process.chdir(rootDir);

  const profileFlag: string = dev ? "--dev" : "--release";
  console.log(
    `[tools/build] wasm-pack build --target web ${profileFlag} --features analyzer --no-default-features`
  );

  const result =
    await $`wasm-pack build --target web ${profileFlag} --features analyzer --no-default-features`.nothrow();

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

if (import.meta.main) {
  const ok: boolean = await ensureWasm({ dev: isDev });
  if (!ok) process.exit(1);
}