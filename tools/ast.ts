#!/usr/bin/env bun
/**
  RTS AST console viewer
 
  Uses WASM analyzer (pkg/rts.js) to tokenize a source file and print
  a tree similar to the old Rust debug output.
 
  Usage:
    bun run tools/ast.ts <file.rt>
 
  If pkg/ is missing, builds it via tools/build.ts first.
*/

import { readFileSync, existsSync } from "fs";
import { resolve } from "path";
import { ensureWasm } from "./build.ts";

const fileArg: string | undefined = process.argv[2];

if (!fileArg) {
  console.error("Usage: bun run tools/ast.ts <file.rt>");
  process.exit(1);
}

const filePath: string = resolve(fileArg);

if (!existsSync(filePath)) {
  console.error(`Unable to read file: ${filePath}`);
  process.exit(1);
}

const sourceCode: string = readFileSync(filePath, "utf8");

const wasmOk: boolean = await ensureWasm();
if (!wasmOk) {
  process.exit(1);
}

const { default: init, analyzeLines } = await import("../pkg/rts.js");

// ---------- ANSI ----------
const BOLD: string = "\x1b[1m";
const RESET: string = "\x1b[0m";

function hexToRgb(hex: string): [number, number, number] {
  const num: number = parseInt(hex.slice(1), 16);
  return [(num >> 16) & 255, (num >> 8) & 255, num & 255];
}

function FG(color: string): string {
  return `\x1b[38;2;${hexToRgb(color).join(";")}m`;
}

const COLOR_TOKEN: string = FG("#f0f8ff");
const COLOR_LABEL: string = FG("#90df91");

// ---------- Types ----------
interface Token {
  kind: string;
  start: number;
  end: number;
  data?: string;
  primitive?: boolean;
  lines?: Line[];
}

interface Line {
  indent: number;
  tokens?: Token[];
  lines?: Line[];
}

// ---------- Source helpers ----------
const encoder: TextEncoder = new TextEncoder();
const decoder: TextDecoder = new TextDecoder();

function getTokenText(start: number, end: number): string {
  const bytes: Uint8Array = encoder.encode(sourceCode);
  return decoder.decode(bytes.slice(start, end));
}

function resolveTokenText(token: Token): string | undefined {
  if (token.data !== undefined) return token.data;
  if (token.start !== undefined && token.end !== undefined && token.end > token.start) {
    return getTokenText(token.start, token.end);
  }
  return undefined;
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

// ---------- Tree print ----------

function outputTokens(tokens: Token[], lineIndent: number, indent: number): void {
  if (tokens.length === 0) return;

  const lineIndentString: string = " ".repeat(lineIndent * 2 + 1);
  const identString: string = " ".repeat(indent * 2 + 1);
  const tokenCount: number = tokens.length - 1;

  tokens.forEach((token: Token, i: number) => {
    const isLast: boolean = i === tokenCount;
    const c: string = isLast ? "X" : "┃";
    const tokenType: string = token.kind;
    const tokenText: string | undefined = resolveTokenText(token);

    if (tokenText !== undefined) {
      const displayed: string = formatTokenDisplay(tokenType, tokenText);
      console.log(
        `${lineIndentString}${BOLD}${c}${RESET}${identString}${COLOR_TOKEN}${displayed}${RESET}  |${tokenType}`
      );
    } else if (token.primitive) {
      console.log(
        `${lineIndentString}${BOLD}${c}${RESET}${identString}|${tokenType}`
      );
    } else {
      console.log(
        `${lineIndentString}${BOLD}${c}${RESET}${identString}${tokenType}`
      );
    }

    if (token.lines && token.lines.length > 0) {
      token.lines.forEach((line: Line, idx: number) => {
        outputTokens(line.tokens ?? [], lineIndent, indent + 1);
        if (idx !== token.lines!.length - 1) {
          console.log(`${lineIndentString}${BOLD}┃${RESET}`);
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
      console.log(`${identStr2}${BOLD}┗${RESET} ${COLOR_LABEL}Separator${RESET}`);
    } else {
      console.log(`${identStr2}${BOLD}┣${RESET} ${COLOR_LABEL}Tokens${RESET}`);
      outputTokens(line.tokens, indent, 1);
    }

    if (line.lines && line.lines.length > 0) {
      console.log(`${identStr2}${BOLD}┗${RESET} ${COLOR_LABEL}Lines${RESET}`);
      outputLines(line.lines, indent + 1);
    }
  });
}

// ---------- Main ----------

await init();

const resultJson: string = analyzeLines(sourceCode);
const lines: Line[] = JSON.parse(resultJson) as Line[];

outputLines(lines, 0);
