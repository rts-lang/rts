#!/usr/bin/env bun
/**
 todo desc
*/
import init, { analyzeLines } from '../pkg/rts.js';

// ---------- ANSI-утилиты ----------
const Bold = '\x1b[1m';
const Reset = '\x1b[0m';
const FG = (color: string): string => `\x1b[38;2;${hexToRgb(color).join(';')}m`;

function hexToRgb(hex: string): [number, number, number] {
  const num: number = parseInt(hex.slice(1), 16);
  return [(num >> 16) & 255, (num >> 8) & 255, num & 255];
}
// -----------------------------------

const sampleCode = `"test", f"test2", f"{test3}", 't', f't', f'{t}', 'test' ok`;

// ---------- Утилиты для работы с байтами UTF-8 ----------
const encoder = new TextEncoder();
const decoder = new TextDecoder();

function getTokenText(tokenStart: number, tokenEnd: number): string {
  const bytes: Uint8Array<ArrayBuffer> = encoder.encode(sampleCode);
  const slice: Uint8Array<ArrayBuffer> = bytes.slice(tokenStart, tokenEnd);
  return decoder.decode(slice);
}
// --------------------------------------------------------

await init();
const resultJson: string = analyzeLines(sampleCode);
const lines: Line[] = JSON.parse(resultJson);

// Запуск вывода
outputLines(lines, 0);

// ----- Типы -----
interface Token {
  kind: string;
  start: number;
  end: number;
  data?: string; // Может быть (опционально).
  primitive?: boolean;
  lines?: Line[];
}

interface Line {
  indent: number;
  tokens?: Token[];
  lines?: Line[];
}

// ----- Функции вывода -----

function outputTokens(
  tokens: Token[],
  lineIndent: number,
  indent: number
): void {
  if (tokens.length === 0) return;

  const lineIndentString: string = ' '.repeat(lineIndent * 2 + 1);
  const identString: string = ' '.repeat(indent * 2 + 1);
  const tokenCount: number = tokens.length - 1;

  tokens.forEach((token: Token, i: number): void => {
    const isLast: number = i === tokenCount;
    const c: 'X'|'┃' = isLast ? 'X' : '┃';

    // Определяем текст токена: из data или вырезаем из исходника байтово-корректно
    const tokenText: string | undefined =
      token.data !== undefined
        ? token.data
        : token.start !== undefined && token.end !== undefined
          ? getTokenText(token.start, token.end)
          : undefined;

    const tokenType: string = token.kind;

    if (tokenText !== undefined) {
      // Токен с текстовыми данными – выводим с кавычками для особых типов
      let displayed: string;
      displayed = tokenText;
      console.log(
        `${lineIndentString}${Bold}${c}${Reset}${identString}${FG('#f0f8ff')}${displayed}${Reset} | ${tokenType}`
      );
    } else {
      // Токен только с типом (примитив)
      if (token.primitive) {
        console.log(
          `${lineIndentString}${Bold}${c}${Reset}${identString}|${tokenType}`
        );
      } else {
        console.log(
          `${lineIndentString}${Bold}${c}${Reset}${identString}${tokenType}`
        );
      }
    }

    // Рекурсия по вложенным линиям токена
    if (token.lines) {
      token.lines.forEach((line: Line, idx: number): void => {
        outputTokens(line.tokens ?? [], lineIndent, indent + 1);
        const notLast: boolean = idx !== token.lines!.length - 1;
        if (notLast) {
          console.log(`${lineIndentString}${Bold}┃${Reset}`);
        }
      });
    }
  });
}

function outputLines(lines: Line[], indent: number): void {
  const identStr1: string = ' '.repeat(indent * 2);
  const identStr2: string = identStr1 + ' ';

  lines.forEach((line: Line, i: number): void => {
    console.log(`${identStr1} ${i}`);

    if (!line.tokens) {
      console.log(`${identStr2}${Bold}┗${Reset} ${FG('#90df91')}Separator${Reset}`);
    } else {
      console.log(`${identStr2}${Bold}┣${Reset} ${FG('#90df91')}Tokens${Reset}`);
      outputTokens(line.tokens, indent, 1);
    }

    if (line.lines) {
      console.log(`${identStr2}${Bold}┗${Reset} ${FG('#90df91')}Lines${Reset}`);
      outputLines(line.lines, indent + 1);
    }
  });
}