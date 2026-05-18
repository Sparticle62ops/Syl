#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include <math.h>
#include <time.h>

#ifdef _WIN32
    #define NOGDI
    #define NOUSER
    #include <windows.h>
    #include <conio.h>
#else
    #include <termios.h>
#endif

#include "raylib.h"

// Syl v0.1 C Transpiler Preamble
typedef char* String;
String _syl_last_error = NULL;
void* syl_alloc(size_t size);
char* syl_strdup(const char* s);

typedef void* Database;


Database syl_db_connect(String path) {
    return NULL;
}
void syl_db_execute(Database db, String query) {
}

typedef int SOCKET;
#define INVALID_SOCKET -1
#define closesocket close


void syl_http_reply(SOCKET client, String content) {}
SOCKET _syl_current_client = 0;
String _syl_current_path = "";

double syl_time_now() {
    return (double)time(NULL);
}

String syl_date_now() {
    time_t t = time(NULL);
    struct tm *tm = localtime(&t);
    char* s = (char*)syl_alloc(64);
    strftime(s, 64, "%Y-%m-%d", tm);
    return s;
}


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
    if (!s) return "";
    size_t len = strlen(s);
    char* d = (char*)syl_alloc(len + 1);
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

typedef struct {
    String* keys;
    String* values;
    int count;
    int capacity;
} Dictionary;

Dictionary syl_dict_create() {
    Dictionary d;
    d.capacity = 128;
    d.count = 0;
    d.keys = (String*)syl_alloc(sizeof(String) * d.capacity);
    d.values = (String*)syl_alloc(sizeof(String) * d.capacity);
    return d;
}

void syl_dict_set(Dictionary* d, String key, String value) {
    for (int i = 0; i < d->count; i++) {
        if (strcmp(d->keys[i], key) == 0) {
            d->values[i] = value;
            return;
        }
    }
    if (d->count >= d->capacity) return;
    d->keys[d->count] = key;
    d->values[d->count] = value;
    d->count++;
}

String syl_dict_get(Dictionary d, String key) {
    for (int i = 0; i < d.count; i++) {
        if (strcmp(d.keys[i], key) == 0) {
            return d.values[i];
        }
    }
    return "";
}

String syl_json_from_dict(Dictionary d) {
    char* buffer = (char*)syl_alloc(8192);
    strcpy(buffer, "{");
    for (int i = 0; i < d.count; i++) {
        strcat(buffer, "\"");
        strcat(buffer, d.keys[i]);
        strcat(buffer, "\": \"");
        strcat(buffer, d.values[i]);
        strcat(buffer, "\"");
        if (i < d.count - 1) strcat(buffer, ", ");
    }
    strcat(buffer, "}");
    return buffer;
}

Dictionary syl_dict_from_json(String j) {
    Dictionary d = syl_dict_create();
    char* copy = strdup(j); // Use system strdup for scratch
    char* p = copy;
    while (*p) {
        if (*p == '"') {
            p++;
            char* key = p;
            while (*p && *p != '"') p++;
            if (*p) { *p = 0; p++; }
            while (*p && (*p == ':' || *p == ' ' || *p == '"')) p++;
            char* val = p;
            while (*p && *p != '"') p++;
            if (*p) { *p = 0; p++; }
            syl_dict_set(&d, syl_strdup(key), syl_strdup(val));
        } else p++;
    }
    free(copy);
    return d;
}

void syl_http_dispatch(String path, SOCKET client) {
}

int main() {
    syl_arena_init(1024 * 1024 * 10); // 10MB default arena
double game_over = 0;
double snake_x = 10;
double snake_y = 10;
double food_x = 15;
double food_y = 15;
printf("\x1B[2J\x1B[1;1H");
while ((game_over == 0)) {
    printf("\x1B[2J\x1B[1;1H");
    { int _bx = 0; int _by = 0; int _bw = 30; int _bh = 20;
    printf("\x1B[%d;%dH┌", _by + 1, _bx + 1);
    for(int i=1; i<_bw-1; i++) printf("─");
    printf("┐\n");
    for(int i=1; i<_bh-1; i++) {
      printf("\x1B[%d;%dH│", _by + 1 + i, _bx + 1);
      printf("\x1B[%d;%dH│", _by + 1 + i, _bx + _bw);
    }
    printf("\x1B[%d;%dH└", _by + _bh, _bx + 1);
    for(int i=1; i<_bw-1; i++) printf("─");
    printf("┘\n"); }
    printf("\x1B[%d;%dH", (int)(food_y) + 1, (int)(food_x) + 1);
    printf("\x1b[31m%s\x1b[0m\n", "@");
    printf("\x1B[%d;%dH", (int)(snake_y) + 1, (int)(snake_x) + 1);
    printf("\x1b[32m%s\x1b[0m\n", "O");
    String input_key = "";
    #ifdef _WIN32
    if (_kbhit()) {
        int c = _getch();
        if (c == 224 || c == 0) {
            c = _getch();
            if (c == 72) input_key = "w";
            else if (c == 80) input_key = "s";
            else if (c == 75) input_key = "a";
            else if (c == 77) input_key = "d";
        } else {
            char buf[2] = {c, 0};
            input_key = syl_strdup(buf);
        }
    }
    #else
    {
        struct termios oldt, newt;
        tcgetattr(STDIN_FILENO, &oldt);
        newt = oldt;
        newt.c_lflag &= ~(ICANON | ECHO);
        tcsetattr(STDIN_FILENO, TCSANOW, &newt);
        struct timeval tv = {0, 0};
        fd_set fds;
        FD_ZERO(&fds);
        FD_SET(STDIN_FILENO, &fds);
        int has_data = select(STDIN_FILENO + 1, &fds, NULL, NULL, &tv);
        if (has_data > 0) {
            int c = getchar();
            if (c == 27) {
                FD_ZERO(&fds);
                FD_SET(STDIN_FILENO, &fds);
                if (select(STDIN_FILENO + 1, &fds, NULL, NULL, &tv) > 0) {
                    int c2 = getchar();
                    if (c2 == '[') {
                        FD_ZERO(&fds);
                        FD_SET(STDIN_FILENO, &fds);
                        if (select(STDIN_FILENO + 1, &fds, NULL, NULL, &tv) > 0) {
                            int c3 = getchar();
                            if (c3 == 'A') input_key = "w";
                            else if (c3 == 'B') input_key = "s";
                            else if (c3 == 'D') input_key = "a";
                            else if (c3 == 'C') input_key = "d";
                        }
                    }
                }
            } else {
                char buf[2] = {c, 0};
                input_key = syl_strdup(buf);
            }
        }
        tcsetattr(STDIN_FILENO, TCSANOW, &oldt);
    }
    #endif
    // Unimplemented statement transpilation
    // Unimplemented statement transpilation
    // Unimplemented statement transpilation
    // Unimplemented statement transpilation
    // Unimplemented statement transpilation
    if (((snake_x == food_x) && (snake_y == food_y))) {
        food_x = (rand() % (28 - 1 + 1) + 1);
        food_y = (rand() % (18 - 1 + 1) + 1);
    }
    #ifdef _WIN32
    Sleep((DWORD)(100));
    #else
    usleep((useconds_t)((100) * 1000));
    #endif
}
printf("\x1B[2J\x1B[1;1H");
printf("\x1b[31m%s\x1b[0m\n", "Game Over!");

    return 0;
}
