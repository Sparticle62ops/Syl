#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include <math.h>
#include <winsock2.h>
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
    if (!s) return "";
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

void syl_http_reply(SOCKET client, String content) {
    char header[256];
    sprintf(header, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: %d\r\n\r\n", (int)strlen(content));
    send(client, header, strlen(header), 0);
    send(client, content, strlen(content), 0);
}

SOCKET _syl_current_client;
String _syl_current_path;

void syl_http_dispatch(String path, SOCKET client) {
if (strcmp(path, "/api/status") == 0) {
    printf("%s\n", "Received request for API status.");
    dictionary response_data = {0};
    syl_dict_set(&response_data, "status", "Online");
    syl_dict_set(&response_data, "speed", "Native");
    String output = ({ char* res = syl_alloc(strlen("{ status: ") + strlen(syl_dict_get(response_data, "status")) + 1); strcpy(res, "{ status: "); strcat(res, syl_dict_get(response_data, "status")); res; });
    output = ({ char* res = syl_alloc(strlen(output) + strlen(", speed: ") + 1); strcpy(res, output); strcat(res, ", speed: "); res; });
    output = ({ char* res = syl_alloc(strlen(output) + strlen(syl_dict_get(response_data, "speed")) + 1); strcpy(res, output); strcat(res, syl_dict_get(response_data, "speed")); res; });
    output = ({ char* res = syl_alloc(strlen(output) + strlen(" }") + 1); strcpy(res, output); strcat(res, " }"); res; });
    syl_http_reply(_syl_current_client, output);
}
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include <math.h>
#include <winsock2.h>
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
    if (!s) return "";
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

void syl_http_reply(SOCKET client, String content) {
    char header[256];
    sprintf(header, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: %d\r\n\r\n", (int)strlen(content));
    send(client, header, strlen(header), 0);
    send(client, content, strlen(content), 0);
}

SOCKET _syl_current_client;
String _syl_current_path;
}

int main() {
    syl_arena_init(1024 * 1024 * 10); // 10MB default arena
printf("%s\n", "Syl Web Server starting on http://localhost:8080");
// SYL HTTP SERVER BOILERPLATE
{
    WSADATA wsa; WSAStartup(MAKEWORD(2,2), &wsa);
    SOCKET server = socket(AF_INET, SOCK_STREAM, 0);
    struct sockaddr_in saddr; saddr.sin_family = AF_INET; saddr.sin_addr.s_addr = INADDR_ANY;
    saddr.sin_port = htons(8080);
    bind(server, (struct sockaddr *)&saddr, sizeof(saddr));
    listen(server, 3);
    while(1) {
        struct sockaddr_in client; int csize = sizeof(client);
        _syl_current_client = accept(server, (struct sockaddr *)&client, &csize);
        if (_syl_current_client != INVALID_SOCKET) {
            char buffer[4096] = {0}; recv(_syl_current_client, buffer, 4096, 0);
            char m[16], p[256], pr[16]; sscanf(buffer, "%s %s %s", m, p, pr);
            _syl_current_path = syl_strdup(p);
            syl_http_dispatch(_syl_current_path, _syl_current_client);
            closesocket(_syl_current_client);
        }
    }
}

    return 0;
}
