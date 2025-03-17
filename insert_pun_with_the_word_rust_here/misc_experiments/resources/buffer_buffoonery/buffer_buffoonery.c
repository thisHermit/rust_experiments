#include <string.h>

void stack_to_heap_pumping_1() {
  int written = 0;

  char chunk[] = "chunkongoongoongoongoongoongong";
  char stack_buffer[(1024 * 32) + 1];
  for (int i = 0; i < 1024; ++i) {
    memcpy(stack_buffer + written, chunk, sizeof(chunk));
    stack_buffer[written + 31] = (char)i;
    written += sizeof(chunk) + 1;
  }
}

int main() { stack_to_heap_pumping_1(); }
