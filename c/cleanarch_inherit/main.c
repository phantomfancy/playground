#include "namedPoint.h"
#include "point.h"
#include <stdio.h>

int main(void)
{
    struct NamedPoint *origin = makeNamedPoint(0.0, 0.0, "origin");
    struct NamedPoint *upperRight = makeNamedPoint(1.0, 1.0, "upperRight");
    printf("distance=%f\n",
           distance((struct Point *)origin, (struct Point *)upperRight));
}
