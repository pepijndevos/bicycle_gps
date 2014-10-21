import sys
import time
import json
import random
import rtreeparse
from PIL import Image, ImageDraw

def color():
    return (
        random.randint(50, 255),
        random.randint(50, 255),
        random.randint(50, 255),
    )

def draw(drawctx, data):
    x0 = data.x0
    y0 = data.y0
    x1 = data.x1
    y1 = data.y1
    drawctx.rectangle((x0, y0, x1, y1), outline=color())
    try:
        for d in data.sub:
            draw(drawctx, d)
    except TypeError:
        pass
    

def main(path):
    image = Image.new("RGB", (1000, 1000))
    drawctx = ImageDraw.Draw(image)
    draw(drawctx, rtreeparse.read(path))
    image.save("output.png", "PNG")

if __name__ == "__main__":
    main(*sys.argv[1:])
