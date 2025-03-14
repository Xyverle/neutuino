#include <stdio.h>
#include <termios.h>
#include <sys/ioctl.h>

int main() {
  printf("ECHO: 0x%x\n", ECHO);
  printf("ICANON: 0x%x\n", ICANON);
  printf("ISIG: 0x%x\n", ISIG);

  printf("\nNCCS: 0x%x\n", NCCS);
  printf("TIOCFWINSZ: 0x%x\n", TIOCGWINSZ);
  return 0;
}
