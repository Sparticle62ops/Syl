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

int main() {
    syl_arena_init(1024 * 1024 * 10); // 10MB default arena
printf("%s\n", "Running Helix Test Suite...");
double result = (5 * 5);
if (result != 25) {
    printf("[ TEST FAILED ] Expected %g, got %g\n", (double)(25), (double)(result));
    exit(1);
} else {
    printf("✔ Test passed.\n");
}
double root = sqrt(64);
if (root != 8) {
    printf("[ TEST FAILED ] Expected %g, got %g\n", (double)(8), (double)(root));
    exit(1);
} else {
    printf("✔ Test passed.\n");
}
String my_string = ({ char* res = syl_alloc(strlen("Hello") + strlen(" World") + 1); strcpy(res, "Hello"); strcat(res, " World"); res; });
if (strcmp(my_string, "Hello World") != 0) {
    printf("[ TEST FAILED ] Expected %s, got %s\n", "Hello World", my_string);
    exit(1);
} else {
    printf("✔ Test passed.\n");
}
printf("%s\n", "All tests passed successfully!");

    return 0;
}
