#include <stdio.h>
#include <stdint.h>
#include <stddef.h>

// Сборка: clang -shared -fPIC -o libprint.so print.c

// Сигнатура одинаково подходит и для (str.pointer, str.length) вручную,
// и для целого str через .arg::<String>() — chillffi в обоих случаях
// кладёт на ABI-границу (const char* data, size_t len).
uint8_t* print(const uint8_t* data, size_t length) {
  fwrite(data, 1, length, stdout);
  fflush(stdout);
  return NULL;
}
