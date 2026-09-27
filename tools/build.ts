#!/usr/bin/env bun
/**
 * Build WASM package for tools (analyzer).
 *
 * From Cargo.toml:
 *   wasm-pack build --target web --features analyzer --no-default-features
 *
 * Usage:
 *   bun run tools/build.ts           # release wasm → ./pkg
 *   bun run tools/build.ts --dev     # dev profile (faster, larger)
 *   bun run tools/build.ts --force   # rebuild even if pkg looks ok
 *
 * Other tools can call ensureWasm() so pkg/rts.js exists before import.
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
const force: boolean = args.includes("--force");

function pkgLooksReady(): boolean {
  return existsSync(pkgJs) && existsSync(pkgWasm);
}

/**
 * Build wasm into ./pkg. Returns true on success.
 * Safe to call from other tools before importing ../pkg/rts.js.
 */
export async function ensureWasm(options?: {
  dev?: boolean;
  force?: boolean;
}): Promise<boolean> {
  const dev: boolean = options?.dev ?? false;
  const doForce: boolean = options?.force ?? false;

  if (!doForce && pkgLooksReady()) {
    return true;
  }

  // wasm-pack must be available
  const which = await $`which wasm-pack`.quiet().nothrow();
  if (which.exitCode !== 0) {
    console.error(
      "[tools/build] wasm-pack not found.\n" +
        "  Install: cargo install wasm-pack\n" +
        "  Or: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    );
    return false;
  }

  // target wasm32-unknown-unknown
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

  const result = await $`wasm-pack build --target web ${profileFlag} --features analyzer --no-default-features`.nothrow();

  if (result.exitCode !== 0) {
    console.error("[tools/build] wasm-pack failed");
    console.error(result.stderr.toString());
    return false;
  }

  if (!pkgLooksReady()) {
    console.error(`[tools/build] expected ${pkgJs} and ${pkgWasm} after build`);
    return false;
  }

  const wasmSize: number = statSync(pkgWasm).size;
  console.log(`[tools/build] ok → ${pkgDir} (${wasmSize} bytes wasm)`);
  return true;
}

// CLI entry
if (import.meta.main) {
  const ok: boolean = await ensureWasm({ dev: isDev, force: force || !pkgLooksReady() });
  if (!ok) process.exit(1);
}
