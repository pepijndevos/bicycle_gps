import sys
import sqlite3
import pygame
pygame.init() 

window = pygame.display.set_mode((800, 800)) 
clock = pygame.time.Clock()

scale = 10000
yoffset = -52.1
xoffset = -5.9

def get(conn):
    return conn.execute('''
        SELECT w.id, n.lat, n.lon, t.val
        FROM nodes as n
        INNER JOIN junct as j
            ON n.id = j.node_id
        INNER JOIN ways as w
            ON w.id = j.way_id
        INNER JOIN tags as t
            ON w.id = t.way_id
        WHERE
            t.key = 'highway'
        OR
            t.key = 'bicycle'
        ORDER BY w.id, j.pathindex;
        ''')

def draw(nodes):
    window.fill((0, 0, 0))
    prev = None
    for way, lat, lon, roadtype in nodes:
        if prev and prev[0] == way:
            if roadtype == 'yes':
                color = (0, 0, 255)
            elif roadtype == 'cycleway':
                color = (0, 255, 0)
            elif roadtype == 'no':
                color = (255, 0, 0)
            else:
                color = (255, 255, 255)
            pygame.draw.line(window,
            #print(
                color,
                ((prev[2] + xoffset) * scale, (prev[1] + yoffset) * -scale),
                ((lon + xoffset) * scale, (lat + yoffset) * -scale))
        prev = way, lat, lon
    pygame.display.flip() 

def main(dbpath):
    global xoffset, yoffset, scale
    with sqlite3.connect(dbpath) as conn:
        nodes = get(conn).fetchall()
        draw(nodes)
        while True:
            clock.tick(30)
            for event in pygame.event.get():
                if event.type == pygame.MOUSEMOTION and event.buttons[0]:
                    print(event.type, event.buttons, event.rel)
                    xoffset += event.rel[0] / scale
                    yoffset -= event.rel[1] / scale
                    draw(nodes)
                if event.type == pygame.MOUSEBUTTONUP and event.button == 4:
                    scale *= 1.1
                    draw(nodes)
                if event.type == pygame.MOUSEBUTTONUP and event.button == 5:
                    scale *= 0.9
                    draw(nodes)



if __name__ == "__main__":
    main(*sys.argv[1:])
