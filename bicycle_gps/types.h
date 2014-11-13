#define DEGREE 25

typedef struct {
  int32_t x, y;
} Point;

typedef struct {
  int32_t sub, x0, y0, x1, y1;
} Rect;

typedef union {
  Rect nodes[DEGREE];
  Point points[(int)(DEGREE*2.5)];
} Spatial;

typedef struct {
  int32_t len;
  Spatial sub;
} Node;

typedef union {
  Node n;
  uint8_t buf[512];
} NodeBuffer;
