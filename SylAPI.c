#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include <math.h>

#ifdef _WIN32
    #include <winsock2.h>
    #pragma comment(lib, "ws2_32.lib")
#else
    #include <sys/socket.h>
    #include <netinet/in.h>
    #include <arpa/inet.h>
    #include <unistd.h>
    typedef int SOCKET;
    #define INVALID_SOCKET -1
    #define closesocket close
#endif

#include "raylib.h"

// Syl v0.1 C Transpiler Preamble
typedef char* String;
String _syl_last_error = NULL;

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

void syl_http_reply(SOCKET client, String content) {
    char header[512];
    sprintf(header, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: %d\r\n\r\n", (int)strlen(content));
    send(client, header, strlen(header), 0);
    send(client, content, strlen(content), 0);
}

SOCKET _syl_current_client;
String _syl_current_path;

void syl_http_dispatch(String path, SOCKET client) {
if (strcmp(path, "/api/user") == 0) {
    printf("%s\n", "Received API request for /api/user");
    _syl_last_error = NULL;
    String raw_data = "";
    { FILE *f = fopen("user_db.json", "r"); if (f) {
        _syl_last_error = NULL;
        fseek(f, 0, SEEK_END); long fsize = ftell(f); fseek(f, 0, SEEK_SET);
        raw_data = syl_alloc(fsize + 1);
        fread(raw_data, 1, fsize, f); raw_data[fsize] = 0;
        fclose(f);
    } else { _syl_last_error = "File not found"; } }
    if (_syl_last_error != NULL) {
        printf("%s\n", "Warning: user_db.json not found. Returning error response.");
        dictionary error_resp = {0};
        syl_dict_set(&error_resp, "error", "Database missing.");
        syl_dict_set(&error_resp, "suggestion", "Please create user_db.json");
        double json_output = syl_json_from_dict(error_resp);
        syl_http_reply(_syl_current_client, json_output);
    }
    if (_syl_last_error == NULL) {
        printf("%s\n", "Database loaded successfully.");
        double user_dict = syl_dict_from_json(raw_data);
        syl_dict_set(&user_dict, "status", "Active");
        double final_json = syl_json_from_dict(user_dict);
        syl_http_reply(_syl_current_client, final_json);
    }
}
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include <math.h>

#ifdef _WIN32
    #include <winsock2.h>
    #pragma comment(lib, "ws2_32.lib")
#else
    #include <sys/socket.h>
    #include <netinet/in.h>
    #include <arpa/inet.h>
    #include <unistd.h>
    typedef int SOCKET;
    #define INVALID_SOCKET -1
    #define closesocket close
#endif

#include "raylib.h"

// Syl v0.1 C Transpiler Preamble
typedef char* String;
String _syl_last_error = NULL;

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

void syl_http_reply(SOCKET client, String content) {
    char header[512];
    sprintf(header, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: %d\r\n\r\n", (int)strlen(content));
    send(client, header, strlen(header), 0);
    send(client, content, strlen(content), 0);
}

SOCKET _syl_current_client;
String _syl_current_path;
}

int main() {
    syl_arena_init(1024 * 1024 * 10); // 10MB default arena
printf("%s\n", "Syl Enterprise API starting on http://localhost:8080");
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
