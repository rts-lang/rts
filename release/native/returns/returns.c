#include <stdint.h>
#include <stddef.h>

// clang -shared -fPIC -o libreturns.so returns.c

// Функции без аргументов — по одной на каждый ABI-тип результата
uint8_t  retU8(void)    { return 200; }
uint16_t retU16(void)   { return 60000; }
uint32_t retU32(void)   { return 4000000000u; }
uint64_t retU64(void)   { return 18446744073709551615ull; }
size_t   retUsize(void) { return 123456789; }
int8_t   retI8(void)    { return -100; }
int16_t  retI16(void)   { return -30000; }
int32_t  retI32(void)   { return -2000000000; }
int64_t  retI64(void)   { return -9000000000000000000ll; }
float    retF32(void)   { return -1.5f; }
double   retF64(void)   { return 2.5; }
void     retVoid(void)  {}

// Аргументы и результат
uint16_t addU8(uint8_t a, uint8_t b)  { return (uint16_t)(a + b); }
double   mulF64(double a, double b)   { return a * b; }
