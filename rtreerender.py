import sys
import time
import json
import random
import rtreeparse
from PIL import Image, ImageDraw

lat = 505000000
lon =  30000000
xscale = 5000
yscale = 3000

def color():
    return (
        random.randint(50, 255),
        random.randint(50, 255),
        random.randint(50, 255),
    )

def draw(drawctx, data):
    x0 = (data.x0 - lon) / xscale
    y0 = (data.y0 - lat) / yscale
    x1 = (data.x1 - lon) / xscale
    y1 = (data.y1 - lat) / yscale
    drawctx.rectangle((x0, y0, x1, y1), outline=color())
    #print(x0, y0, x1, y1)
    try:
        for d in data.sub:
            draw(drawctx, d)
    except TypeError:
        pass
    

def main(path):
    image = Image.new("RGB", (10000, 12000))
    drawctx = ImageDraw.Draw(image)
    draw(drawctx, rtreeparse.read(path))
    image.save("output.gif", "GIF")

if __name__ == "__main__":
    main(*sys.argv[1:])
