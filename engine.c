#include <stdio.h>


struct Str {
    char* rc;
    int len;
};


void print(struct Str ptr)
{
    for (int i = 0; i < ptr.len; i++)
    {
        printf("%c", ptr.rc[8+i]);
    }
}

void __initStartupSystems__();

int main()
{
    __initStartupSystems__();
    return 0;
}
