import sys
import struct
from collections import namedtuple

Node = namedtuple('Node', ['x0', 'y0', 'x1', 'y1', 'sub'])

def read_fmt(f, fmt):
    return struct.unpack(fmt, f.read(struct.calcsize(fmt)))


def read_node(f, idx):
    f.seek(idx)
    fmt = "!BIIII"
    count, x0, y0, x1, y1 = struct.unpack(fmt, f.read(struct.calcsize(fmt)))
    if count:
        children = read_fmt(f, '!' + 'I' * count)
        return Node(x0, y0, x1, y1, [read_node(f, i) for i in children])
    else:
        return Node(x0, y0, x1, y1, read_fmt(f, '!I')[0])

def read(path):
    with open(path, "rb") as f:
        idx = read_fmt(f, '!I')[0]
        return read_node(f, idx)

