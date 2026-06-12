import init, { analyzeLines } from '../pkg/rts.js';

// ---------- ANSI-утилиты ----------
const BOLD = '\x1b[1m';
const RESET = '\x1b[0m';
const FG = (color: string) => `\x1b[38;2;${hexToRgb(color).join(';')}m`;

function hexToRgb(hex: string): [number, number, number] {
  const num = parseInt(hex.slice(1), 16);
  return [(num >> 16) & 255, (num >> 8) & 255, num & 255];
}
// -----------------------------------

const sampleCode = `"test", f"test2", f"{test3}", 't', f't', f'{t}', 'test' ok`;

// ---------- Утилиты для работы с байтами UTF-8 ----------
const encoder = new TextEncoder();
const decoder = new TextDecoder();

function getTokenText(tokenStart: number, tokenEnd: number): string {
  const bytes = encoder.encode(sampleCode);
  const slice = bytes.slice(tokenStart, tokenEnd);
  return decoder.decode(slice);
}
// --------------------------------------------------------

await init();
const resultJson = analyzeLines(sampleCode);
const lines: Line[] = JSON.parse(resultJson);

// Запуск вывода
outputLines(lines, 0);

// ----- Типы -----
interface Token {
  kind: string;
  start: number;
  end: number;
  data?: string;        // может быть (опционально)
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

  const lineIndentString = ' '.repeat(lineIndent * 2 + 1);
  const identString = ' '.repeat(indent * 2 + 1);
  const tokenCount = tokens.length - 1;

  tokens.forEach((token, i) => {
    const isLast = i === tokenCount;
    const c = isLast ? 'X' : '┃';

    // Определяем текст токена: из data или вырезаем из исходника байтово-корректно
    const tokenText: string | undefined =
      token.data !== undefined
        ? token.data
        : token.start !== undefined && token.end !== undefined
          ? getTokenText(token.start, token.end)
          : undefined;

    const tokenType = token.kind;

    if (tokenText !== undefined) {
      // Токен с текстовыми данными – выводим с кавычками для особых типов
      let displayed: string;
      displayed = tokenText;
      console.log(
        `${lineIndentString}${BOLD}${c}${RESET}${identString}${FG('#f0f8ff')}${displayed}${RESET} | ${tokenType}`
      );
    } else {
      // Токен только с типом (примитив)
      if (token.primitive) {
        console.log(
          `${lineIndentString}${BOLD}${c}${RESET}${identString}|${tokenType}`
        );
      } else {
        console.log(
          `${lineIndentString}${BOLD}${c}${RESET}${identString}${tokenType}`
        );
      }
    }

    // Рекурсия по вложенным линиям токена
    if (token.lines) {
      token.lines.forEach((line, idx) => {
        outputTokens(line.tokens ?? [], lineIndent, indent + 1);
        const notLast = idx !== token.lines!.length - 1;
        if (notLast) {
          console.log(`${lineIndentString}${BOLD}┃${RESET}`);
        }
      });
    }
  });
}

function outputLines(lines: Line[], indent: number): void {
  const identStr1 = ' '.repeat(indent * 2);
  const identStr2 = identStr1 + ' ';

  lines.forEach((line, i) => {
    console.log(`${identStr1} ${i}`);

    if (!line.tokens) {
      console.log(`${identStr2}${BOLD}┗${RESET} ${FG('#90df91')}Separator${RESET}`);
    } else {
      console.log(`${identStr2}${BOLD}┣${RESET} ${FG('#90df91')}Tokens${RESET}`);
      outputTokens(line.tokens, indent, 1);
    }

    if (line.lines) {
      console.log(`${identStr2}${BOLD}┗${RESET} ${FG('#90df91')}Lines${RESET}`);
      outputLines(line.lines, indent + 1);
    }
  });
}