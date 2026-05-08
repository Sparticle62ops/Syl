#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Syl v0.1 C Transpiler Preamble
typedef char* String;
// Bring in auth.syl as auth
String process_login(String username, String password) {
    if (username == NULL || strlen(username) == 0) {
        fprintf(stderr, "Missing username\n");
        exit(1);
    }
    String user_role = auth_get_role(username);
    if (strcmp(user_role, "Admin") == 0) {
        // RUN BACKGROUND: auth_log_admin_login()
        auth_log_admin_login(); // simulated async
        return "Welcome Admin";
    } else {
        return "Welcome User";
    }
}

// --- Dummy dependencies for compilation ---
String auth_get_role(String username) { return "Admin"; }
void auth_log_admin_login() { printf("Admin logged in\n"); }

int main() {
    printf("%s\n", process_login("Alice", "pass123"));
    return 0;
}
