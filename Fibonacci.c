#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include <math.h>
#include "raylib.h"

// Syl v0.1 C Transpiler Preamble
typedef char* String;

typedef struct {
    uint8_t* buffer;
    size_t offset;
    size_t capacity;
} SylArena;

SylArena global_arena;

void syl_arena_init(size_t capacity) {
    global_arena.buffer = (uint8_t*)malloc(capacity);
    global_arena.offset = 0;
    global_arena.capacity = capacity;
}

void* syl_alloc(size_t size) {
    if (global_arena.offset + size > global_arena.capacity) {
        fprintf(stderr, "[ HELIX FATAL ] Arena Out of Memory.\n");
        exit(1);
    }
    void* ptr = global_arena.buffer + global_arena.offset;
    global_arena.offset += size;
    return ptr;
}

char* syl_strdup(const char* s) {
    size_t len = strlen(s);
    char* d = syl_alloc(len + 1);
    if (d) {
        memcpy(d, s, len + 1);
    }
    return d;
}

int ends_with(char* str, char* suffix) {
    if (!str || !suffix) return 0;
    size_t lenstr = strlen(str);
    size_t lensuffix = strlen(suffix);
    if (lensuffix > lenstr) return 0;
    return strncmp(str + lenstr - lensuffix, suffix, lensuffix) == 0;
}
int get_fib(int n) {
    size_t _arena_save = global_arena.offset;
    if ((n < 2)) {
        return n;
    }
    int a = get_fib((n - 1));
    int b = get_fib((n - 2));
    return (a + b);
    global_arena.offset = _arena_save;
}

int main() {
    syl_arena_init(1024 * 1024 * 10); // 10MB default arena
int result = get_fib(30);
printf("%d\n", result);

    return 0;
}
