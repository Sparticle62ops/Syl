#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include "raylib.h"

// Syl v0.8 C Transpiler Preamble
typedef char* String;

int ends_with(char* str, char* suffix) {
    if (!str || !suffix) return 0;
    size_t lenstr = strlen(str);
    size_t lensuffix = strlen(suffix);
    if (lensuffix > lenstr) return 0;
    return strncmp(str + lenstr - lensuffix, suffix, lensuffix) == 0;
}

int main() {
    // [ SYL STUDIO ] Initializing variables...
    int count = 0;
    
    // [ SYL STUDIO ] Bridging to Raylib...
    InitWindow(400, 300, "Syl Counter");
    SetTargetFPS(60);

    // [ SYL STUDIO ] Main Interaction Loop
    while (!WindowShouldClose()) {
        // Handle Key Input
        if (IsKeyPressed(KEY_SPACE)) {
            count += 1;
        }

        // Handle Rendering
        BeginDrawing();
        ClearBackground(BLACK);
        
        // Human-friendly dynamic text formatting
        DrawText(TextFormat("Count: %d", count), 50, 100, 30, RAYWHITE);
        
        EndDrawing();
    }
    
    // [ SYL STUDIO ] Cleanup and Exit
    CloseWindow();
    printf("Final count was: %d\n", count);
    
    return 0;
}
