#include <stdio.h>

int main()
{
    printf("Hello, World!\n");

    auto ver = __STDC_VERSION__;
    printf("c version: %ld\n", ver);

    #ifdef __ZIG__
        printf("Compiled by Zig CC (__ZIG__ defined)\n");
    #endif

    #ifdef __clang__
        // Zig CC 底层基于 Clang，所以这个也会存在，但可以打印版本
        printf("Clang version (zig wrapper): %s\n", __clang_version__);
    #endif

    #ifdef __GNUC__
        // 注意：zig cc 会定义 __GNUC__ 来模拟 GCC，所以看到这个不代表是 GCC
        printf("GCC compatibility macro defined (simulated by zig)\n");
    #endif
    return 0;
}
