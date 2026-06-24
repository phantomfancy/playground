#include <stdio.h>
#include <stdint.h>

typedef struct
{
    unsigned char bit0 : 1;
    unsigned char bit1 : 1;
    unsigned char bit2 : 1;
    unsigned char bit3 : 1;
    unsigned char bit4 : 1;
    unsigned char bit5 : 1;
    unsigned char bit6 : 1;
    unsigned char bit7 : 1;
} BitFieldLsbFirst;

typedef union
{
    uint8_t ch;
    BitFieldLsbFirst bits;
} BitChar;

_Static_assert(sizeof(BitFieldLsbFirst) == 1, "BitFieldLsbFirst must be 1 byte");
_Static_assert(sizeof(BitChar) == 1, "BitChar must be 1 byte");

int main(int argc, char **argv) {
    puts("bitfield union test: ");

    BitChar u = { .ch = 0 };

    u.bits.bit0 = 1;
    u.bits.bit3 = 1;
    u.bits.bit7 = 1;

    // 在常见 x86/ STM32 / RV32 ABI 下，u.ch == 0x89
    printf("u.ch = 0x%x\n", u.ch);
    return 0;
}
