
int overlap(Rect* r, Rect* s) {
  return r->x0 < s->x1 &&
         r->y0 < s->y1 &&
         r->x1 > s->x0 &&
         r->y1 > s->y0;
}

int32_t scale_to_width(int32_t x, Rect* bounds) {
  int32_t bwidth = bounds->x1 - bounds->x0;
  return (x - bounds->x0) / (bwidth / tft.width());
}

int32_t scale_to_height(int32_t y, Rect* bounds) {
  int32_t bheight = bounds->y1 - bounds->y0;
  int32_t height = (y - bounds->y0) / (bheight / tft.height());
  return -height + tft.height();
}

void draw_line(Rect* bounds, int32_t x0, int32_t y0, int32_t x1, int32_t y1, uint16_t color) {
  x0 = scale_to_width(x0, bounds);
  y0 = scale_to_height(y0, bounds);
  x1 = scale_to_width(x1, bounds);
  y1 = scale_to_height(y1, bounds);
  
  tft.drawLine(x0, y0, x1, y1, color);
}

void draw_name(char* wayname, Point* point, Rect* bounds) {
  int x = scale_to_width(point->x, bounds);
  int y = scale_to_height(point->y, bounds);
  tft.setCursor(x, y);
  tft.println(wayname);
}

void draw_points(Way* w, Rect* bounds) {
  // WARNIG: fence poles ahead.
  // the first points are in the node,
  // this means we can satisfy most without aditional reads
  // but outliers should be read point-by-point.
  // max |        avg         
  //-----+--------------------
  // 784 | 4.8219990810608790
  int len = w->pointlen - 1; // minus last node
  char* wayname = &w->wayname;
  Point* points = (Point*)(wayname + w->namelen + 1);
  uint16_t color = ILI9341_YELLOW;
  if (w->flags & 1) {
    color = ILI9341_GREEN;
  } else if (w->flags & 2) {
    color = ILI9341_RED;
  } else if (w->flags & 4) {
    color = ILI9341_BLUE;
  }

  for (int i = 0; i<len && i<5; i++) {
    draw_line(bounds,
      points[i].x, // pi
      points[i].y,
      points[i+1].x, //pi+1
      points[i+1].y,
      color);
  }
  
  if (streetnames) draw_name(wayname, &points[len/2], bounds);
}


void inner_lookup(Rect* bounds, int32_t index) {
  NodeBuffer nb;
  sd.card()->readBlock(bgnBlock + (index / 512), nb.buf);
  
  if (!nb.n.len) {
    draw_points(&nb.n.sub.way, bounds);
    return;
  }
  for (int i=0; i<nb.n.len; i++) {
    if (overlap(&nb.n.sub.nodes[i], bounds)) {
      inner_lookup(bounds, nb.n.sub.nodes[i].sub);
    }
  }
}

void rtree_lookup(Rect* bounds) {
  if (!file.open(sd.vwd(), "data.bin", O_READ)) {
    Serial.println("No data file");
    return;
  }
  // get the location of the file's blocks
  if (!file.contiguousRange(&bgnBlock, &endBlock)) {
    Serial.println("File not contiguous");
    return;
  }

  uint32_t index;
  file.read(&index, sizeof(index));
  
  inner_lookup(bounds, index);
  
  file.close();
}
