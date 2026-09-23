#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

// Сборка: clang -shared -fPIC -o libprint.so print.c

// Функция, которую вызовет RTS после importNative("libprint.so")
// Сигнатура: extern "C" fn(*const u8, usize) -> *mut u8
// Принимает сырые байты аргументов (склеенные подряд), печатает их как строку,
// возвращает NULL (означает Token::None).
uint8_t* print(const uint8_t* data, size_t len) {
    fwrite(data, 1, len, stdout);
    fflush(stdout);
    return NULL; // нет возвращаемого значения
}