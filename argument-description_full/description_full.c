#include <stdio.h>
#include <stdlib.h>

int main() {
  FILE* f =
      fopen("../../all_5_states_undecided_machines_with_global_header", "r");

  unsigned char buffer[30];
  fread(buffer, 30, 1, f);

  for (int i = 0; i < 88664064; i += 1) {
    fread(buffer, 30, 1, f);
    int description_full = 1;
    for (int j = 0; j < 10; j += 1) {
      if (buffer[3 * j] == 0) {
        description_full = 0;
        break;
      }
    }

    if (description_full) {
      printf("%d\n", i);
    }
  }
  fclose(f);
  return 0;
}