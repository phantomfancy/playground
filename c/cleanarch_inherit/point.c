#include "point.h"
#include <math.h>
#include <stdlib.h>

struct Point
{
    double x, y;
};

struct Point *makePoint(double x, double y)
{
    struct Point *p = (struct Point *)malloc(sizeof(struct Point));
    p->x = x;
    p->y = y;
    return p;
}

double distance(struct Point *p1, struct Point *p2)
{
    return sqrt(pow(p2->x - p1->x, 2) + pow(p2->y - p1->y, 2));
}
