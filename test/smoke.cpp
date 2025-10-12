// Smoke test that matches include file and lib for AGC

#include <agc/agc-api.h>

int main() {
    agc_open((char *)"test", 0);
    return 0;
}
