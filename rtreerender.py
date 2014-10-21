import sys
import time
import json
import random
import sqlite3
import pygame
pygame.init() 

window = pygame.display.set_mode((1000, 1000)) 

def scale(n):
    return n

def color():
    return (
        random.randint(50, 255),
        random.randint(50, 255),
        random.randint(50, 255),
    )

def draw(data):
    rect = data['rect']
    x0 = scale(rect['x0'])
    y0 = scale(rect['y0'])
    x1 = scale(rect['x1'])
    y1 = scale(rect['y1'])
    pygame.draw.rect(window, color(), (x0, y0, x1-x0, y1-y0), 1)
    pygame.display.flip() 
    try:
        for d in data['sub']['fields'][0]:
            draw(d)
    except (TypeError):
        pass
    

def main(path):
    window.fill((0, 0, 0))
    pygame.display.flip() 
    input()
    draw(json.load(open(path)))



if __name__ == "__main__":
    main(*sys.argv[1:])
