#include <stdio.h>
#include <unistd.h>
#include <time.h>

// Сборка: clang -shared -fPIC -o libhello.so hello.c

void hello(int number) {
    printf("hello from FFI block: %d\n", number);
    fflush(stdout);
}

long mypid() {
    return (long)getpid();
}

long mytime() {
    return (long)time(NULL);
}