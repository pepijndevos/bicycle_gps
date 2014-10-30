
Node read_node(File f) {
  int32_t data[5+DEGREE];
  f.read(data, sizeof(data));
  Node n;
  n.len = data[0];
  n.rect.x0 = data[1];
  n.rect.y0 = data[2];
  n.rect.x1 = data[3];
  n.rect.y1 = data[4];
  memcpy(n.subnodes, (void*)data[5], sizeof(int32_t)*DEGREE);
  return n;
}

void draw_points(File f, Node n, Point origin) {
  // WARNIG: fence poles ahead.
  // the first 15.5 points are in the node,
  // this means we can satisfy most without aditional reads
  // but outliers should be read point-by-point.
  // max |        avg         
  //-----+--------------------
  // 784 | 4.8219990810608790
  int len = n.subnodes[0] - 1; // minus last node
  int nodelen = (len < 14 ? len : 14);
  int i = 0;
  for (; i<nodelen; i++) {
    tft.drawLine(
      n.subnodes[i*2+1], // pi
      n.subnodes[i*2+2],
      n.subnodes[i*2+3], //pi+1
      n.subnodes[i*2+4],
      ILI9341_WHITE);
  }
  // we drew a line from p14 to p15 at this point
  if (len > 15) {
    int32_t data[2];
    f.read(data, sizeof(int32_t));
    tft.drawLine(
      n.subnodes[29],
      n.subnodes[30],
      n.subnodes[31],
      data[0],
      ILI9341_WHITE);
    Point last_point = {.x = n.subnodes[31], .y = data[0]}; // p16
    for (; i<len; i++) {
      f.read(data, sizeof(data));
      tft.drawLine(
        last_point.x,
        last_point.y,
        data[0],
        data[1],
        ILI9341_WHITE);
        last_point = {.x = data[0], .y = data[1]};
    }
  }
}

void rtree_lookup() {
  File dataFile = SD.open("data.bin");
  if (!dataFile) {
    return;
  }
  
  dataFile.close();
}
