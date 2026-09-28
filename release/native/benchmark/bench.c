#include <stdio.h>
#include <stdint.h>
#include <stddef.h>

uint8_t *print(const uint8_t *data, size_t length) {
  fwrite(data, 1, length, stdout);
  fflush(stdout);
  return NULL;
}
