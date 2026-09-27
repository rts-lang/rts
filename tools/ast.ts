#!/usr/bin/env bun
/**
  RTS AST console viewer.

  Usage:
    bun run tools/ast.ts <file.rt>            # debug
    bun run tools/ast.ts --release <file.rt>  # release
    bun run tools/ast.ts <file.rt> --release  # release

  // todo Нужны комментарии обычные.
*/

import { readFileSync, existsSync } from "fs";
import { resolve } from "path";
import { ensureWasm } from "./build.ts";

const args: string[] = process.argv.slice(2);
const isRelease: boolean = args.includes("--release");
const fileArg: string | undefined = args.find((arg: string) => !arg.startsWith("-"));

if (!fileArg) {
  console.error("Usage: bun run tools/ast.ts [--release] <file.rt>");
  process.exit(1);
}

const filePath: string = resolve(fileArg);

if (!existsSync(filePath)) {
  console.error(`Unable to read file: ${filePath}`);
  process.exit(1);
}

const sourceCode: string = readFileSync(filePath, "utf8");

// build.ts: dev=true → --dev, dev=false → --release
const wasmOk: boolean = await ensureWasm({ dev: !isRelease });
if (!wasmOk) {
  process.exit(1);
}

const pkgMod = await import(`../pkg/rts.js?t=${Date.now()}`);
const init = pkgMod.default as (module?: unknown) => Promise<void>;
const analyzeLinesTree = pkgMod.analyzeLinesTree as
  | ((code: string) => string)
  | undefined;

if (typeof analyzeLinesTree !== "function") {
  console.error(
    "[ast] analyzeLinesTree is not exported from pkg/rts.js\n" +
      "  Run: bun run tools/build.ts --force"
  );
  process.exit(1);
}

// ---------- ANSI ----------
const Bold: string = "\x1b[1m";
const Reset: string = "\x1b[0m";

function hexToRgb(hex: string): [number, number, number] {
  const num: number = parseInt(hex.slice(1), 16);
  return [(num >> 16) & 255, (num >> 8) & 255, num & 255];
}

function FG(color: string): string {
  return `\x1b[38;2;${hexToRgb(color).join(";")}m`;
}

const ColorToken: string = FG("#f0f8ff");
const ColorLabel: string = FG("#90df91");

interface Token {
  kind: string;
  start: number;
  end: number;
  data?: string;
  lines?: Line[];
}

interface Line {
  indent: number;
  tokens?: Token[] | null;
  lines?: Line[] | null;
}

function formatTokenDisplay(kind: string, text: string): string {
  switch (kind) {
    case "Char":
    case "FormattedChar":
      return `'${text}'`;
    case "String":
    case "FormattedString":
      return `"${text}"`;
    case "RawString":
    case "FormattedRawString":
      return `\`${text}\``;
    default:
      return text;
  }
}

function outputTokens(tokens: Token[], lineIndent: number, indent: number): void {
  if (tokens.length === 0) return;

  const lineIndentString: string = " ".repeat(lineIndent * 2 + 1);
  const identString: string = " ".repeat(indent * 2 + 1);
  const tokenCount: number = tokens.length - 1;

  tokens.forEach((token: Token, i: number) => {
    const isLast: boolean = i === tokenCount;
    const c: string = isLast ? "X" : "┃";
    const tokenType: string = token.kind;

    if (token.data !== undefined && token.data !== "") {
      const displayed: string = formatTokenDisplay(tokenType, token.data);
      // data совпадает с kind (например ":")
      if (token.data === tokenType || displayed === tokenType) {
        console.log(
          `${lineIndentString}${Bold}${c}${Reset}${identString}${ColorToken}${displayed}${Reset}`
        );
      } else {
        // `data  Type`
        console.log(
          `${lineIndentString}${Bold}${c}${Reset}${identString}${ColorToken}${displayed}${Reset}  ${tokenType}`
        );
      }
    } else {
      // Comment, без data — только тип
      console.log(
        `${lineIndentString}${Bold}${c}${Reset}${identString}${tokenType}`
      );
    }

    if (token.lines && token.lines.length > 0) {
      token.lines.forEach((line: Line, idx: number) => {
        if (line.tokens && line.tokens.length > 0) {
          outputTokens(line.tokens, lineIndent, indent + 1);
        }
        if (idx !== token.lines!.length - 1) {
          console.log(`${lineIndentString}${Bold}┃${Reset}`);
        }
      });
    }
  });
}

function outputLines(lines: Line[], indent: number): void {
  const identStr1: string = " ".repeat(indent * 2);
  const identStr2: string = identStr1 + " ";

  lines.forEach((line: Line, i: number) => {
    console.log(`${identStr1} ${i}`);

    if (!line.tokens || line.tokens.length === 0) {
      console.log(`${identStr2}${Bold}┗${Reset} ${ColorLabel}Separator${Reset}`);
    } else {
      console.log(`${identStr2}${Bold}┣${Reset} ${ColorLabel}Tokens${Reset}`);
      outputTokens(line.tokens, indent, 1);
    }

    if (line.lines && line.lines.length > 0) {
      console.log(`${identStr2}${Bold}┗${Reset} ${ColorLabel}Lines${Reset}`);
      outputLines(line.lines, indent + 1);
    }
  });
}

await init();

const resultJson: string = analyzeLinesTree(sourceCode);
const lines: Line[] = JSON.parse(resultJson) as Line[];

outputLines(lines, 0);
