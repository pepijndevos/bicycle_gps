#define DEGREE 25

typedef struct {
  int32_t x, y;
} Point;

typedef struct {
  uint32_t sub;
  int32_t x0, y0, x1, y1;
} Rect;

typedef struct {
  uint16_t pointlen;
  uint8_t namelen;
  uint8_t flags;
  char wayname;
} Way;

typedef union {
  Rect nodes[DEGREE];
  Way way;
} Spatial;

typedef struct {
  uint8_t len;
  Spatial sub;
} Node;

typedef union {
  Node n;
  uint8_t buf[512];
} NodeBuffer;
