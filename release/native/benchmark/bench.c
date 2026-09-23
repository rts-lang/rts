#include <stdio.h>
#include <stdint.h>
#include <stddef.h>

// clang -shared -fPIC -o libbench.so bench.c

uint8_t *print(const uint8_t *data, size_t length) {
  fwrite(data, 1, length, stdout);
  fflush(stdout);
  return NULL;
}
