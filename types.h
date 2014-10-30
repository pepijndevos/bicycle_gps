#define DEGREE 32

typedef struct {
  int32_t x, y;
} Point;

typedef struct {
  int32_t x0, y0, x1, y1;
} Rect;

typedef struct {
  Rect rect;
  int len;
  int32_t subnodes[DEGREE];
} Node;
