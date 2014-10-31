
int overlap(Rect r, Rect s) {
  return r.x0 < s.x1 &&
         r.y0 < s.y1 &&
         r.x1 > s.x0 &&
         r.y1 > s.y0;
}

Node read_node(File f) {
  int32_t data[5+DEGREE];
  f.read(data, sizeof(data));
  Node n;
  n.len = data[0];
  n.rect.x0 = data[1];
  n.rect.y0 = data[2];
  n.rect.x1 = data[3];
  n.rect.y1 = data[4];
  //for (int i=0; i<5+DEGREE; i++) {
  //  Serial.println(data[i]);
  //}
  memcpy(&n.subnodes, (void*)&data[5], sizeof(n.subnodes));
  return n;
}

//radians = (degrees * 71) / 4068
//degrees = (radians * 4068) / 71
void draw_line(Point origin, int xscale, int yscale, int32_t x0, int32_t y0, int32_t x1, int32_t y1) {
  x0 = (x0 - origin.x) / xscale;
  y0 = (y0 - origin.y) / yscale;
  x1 = (x1 - origin.x) / xscale;
  y1 = (y1 - origin.y) / yscale;
  Serial.print(x0);
  Serial.print(" ");
  Serial.print(y0);
  Serial.print(" ");
  Serial.print(x1);
  Serial.print(" ");
  Serial.println(y1);
  tft.drawLine(x0, y0, x1, y1, ILI9341_RED);
}

void draw_points(File f, Node n, Point origin, int xscale, int yscale) {
  // WARNIG: fence poles ahead.
  // the first 15.5 points are in the node,
  // this means we can satisfy most without aditional reads
  // but outliers should be read point-by-point.
  // max |        avg         
  //-----+--------------------
  // 784 | 4.8219990810608790
  int len = n.subnodes[0] - 1; // minus last node
  Serial.println(len);
  int nodelen = (len < 14 ? len : 14);
  int i = 0;
  for (; i<nodelen; i++) {
    draw_line(origin, xscale, yscale,
      n.subnodes[i*2+1], // pi
      n.subnodes[i*2+2],
      n.subnodes[i*2+3], //pi+1
      n.subnodes[i*2+4]);
  }
  // we drew a line from p14 to p15 at this point
  /*if (len > 15) {
    int32_t data[2];
    f.read(data, sizeof(int32_t));
    draw_line(origin, xscale, yscale,
      n.subnodes[29],
      n.subnodes[30],
      n.subnodes[31],
      data[0]);
    Point last_point = {.x = n.subnodes[31], .y = data[0]}; // p16
    for (; i<len; i++) {
      f.read(data, sizeof(data));
      draw_line(origin, xscale, yscale,
        last_point.x,
        last_point.y,
        data[0],
        data[1]);
        last_point = {.x = data[0], .y = data[1]};
    }
  }*/
}

void print_node(Node* n) {
  Serial.println("Node");
  Serial.print("Rect: ");
  Serial.print(n->rect.x0);
  Serial.print(" ");
  Serial.print(n->rect.y0);
  Serial.print(" ");
  Serial.print(n->rect.x1);
  Serial.print(" ");
  Serial.println(n->rect.y1);
  Serial.print("Len: ");
  Serial.println(n->len);
  //for (int i=0; i<n->len; i++) {
  //  Serial.println(n->subnodes[i]);
  //}
}

void inner_lookup(File datafile, Rect bounds, int32_t index) {
  datafile.seek(index);
  Node n = read_node(datafile);
  //print_node(&n);
  if (!overlap(n.rect, bounds)) {
    return;
  }
  if (!n.len) {
    Serial.println("reached leaf");
    Point origin = {.x = bounds.x0, .y = bounds.y0};
    draw_points(datafile, n, origin, 1000, 1000);
    return;
  }
  Serial.println(n.len);
  for (int i=0; i<n.len; i++) {
    inner_lookup(datafile, bounds, n.subnodes[i]);
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
