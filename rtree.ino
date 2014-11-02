
int overlap(Rect r, Rect s) {
  return r.x0 < s.x1 &&
         r.y0 < s.y1 &&
         r.x1 > s.x0 &&
         r.y1 > s.y0;
}

//radians = (degrees * 71) / 4068
//degrees = (radians * 4068) / 71
void draw_line(Point origin, int xscale, int yscale, int32_t x0, int32_t y0, int32_t x1, int32_t y1) {
  x0 = (x0 - origin.x) / xscale;
  y0 = (y0 - origin.y) / yscale;
  x1 = (x1 - origin.x) / xscale;
  y1 = (y1 - origin.y) / yscale;
  tft.drawLine(x0, y0, x1, y1, ILI9341_RED);
}

void draw_points(File f, Node* n, Point origin, int xscale, int yscale) {
  // WARNIG: fence poles ahead.
  // the first points are in the node,
  // this means we can satisfy most without aditional reads
  // but outliers should be read point-by-point.
  // max |        avg         
  //-----+--------------------
  // 784 | 4.8219990810608790
  int len = abs(n->len) - 1; // minus last node
  Serial.println(len);
  int nodelen = (len < (int)(DEGREE*2.5) ? len : (int)(DEGREE*2.5));
  for (int i = 0; i<nodelen; i++) {
    draw_line(origin, xscale, yscale,
      n->sub.points[i].x, // pi
      n->sub.points[i].y,
      n->sub.points[i+1].x, //pi+1
      n->sub.points[i+1].y);
  }
}


void inner_lookup(File datafile, Rect bounds, int32_t index) {
  Serial.println("Reading");
  unsigned long time = millis();
  datafile.seek(index);
  Serial.println(millis() - time);
  time = millis();
  // The SD card has 512 byte blocks.
  // By reading 512 bytes at a time,
  // we avoid buffering and copying in SdFatLib.
  NodeBuffer nb;
  datafile.read(&nb.buf, 512);
  Serial.println(millis() - time);
  time = millis();
  
  if (nb.n.len < 0) {
    Serial.println("reached leaf");
    Point origin = {.x = bounds.x0, .y = bounds.y0};
    draw_points(datafile, &nb.n, origin, 1000, 1000);
    return;
  }
  Serial.println(nb.n.len);
  for (int i=0; i<nb.n.len; i++) {
    if (overlap(nb.n.sub.nodes[i], bounds)) {
      inner_lookup(datafile, bounds, nb.n.sub.nodes[i].sub);
    }
  }
}

void rtree_lookup(Rect bounds) {
  File datafile = SD.open("data.bin");
  if (!datafile) {
    Serial.println("No data file");
    return;
  }
  int32_t index;
  datafile.read(&index, sizeof(index));
  Serial.println(index);
  
  inner_lookup(datafile, bounds, index);
  
  datafile.close();
}
