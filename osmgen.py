import sqlite3
import xml.etree.ElementTree as etree
import sys

import pdb

def setup(db):
    db.execute('''CREATE TABLE ways (
                  id INTEGER PRIMARY KEY
                  )''')
    db.execute('''CREATE TABLE nodes (
                  id INTEGER PRIMARY KEY,
                  lat real,
                  lon real
                  )''')
    db.execute('''CREATE TABLE junct (
                  way_id INTEGER,
                  node_id INTEGER,
                  pathindex INTEGER,
                  FOREIGN KEY(way_id) REFERENCES ways(id),
                  FOREIGN KEY(node_id) REFERENCES nodes(id)
                  )''')
    db.execute('''CREATE INDEX nodeindex ON junct(node_id)''')
    db.execute('''CREATE INDEX wayindex ON junct(way_id)''')
    db.execute('''CREATE TABLE tags (
                  way_id INTEGER,
                  key text,
                  val text,
                  FOREIGN KEY(way_id) REFERENCES ways(id)
                  )''')
    db.execute('''CREATE INDEX tagindex ON tags(way_id)''')


def parse(xmlpath, db):
    for event, elem in etree.iterparse(xmlpath):
        if elem.tag == 'node':
            db.execute("INSERT INTO nodes VALUES (?, ?, ?)", (
                elem.attrib['id'],
                elem.attrib['lat'],
                elem.attrib['lon'],
            ))
        elif elem.tag == 'way':
            db.execute("INSERT INTO ways VALUES (?)", (elem.attrib['id'],))
            order = 0
            for nd in elem:
                if nd.tag == 'nd':
                    order += 1
                    db.execute("INSERT INTO junct VALUES (?, ?, ?)", (
                        elem.attrib['id'],
                        nd.attrib['ref'],
                        order,
                    ))
                elif nd.tag == 'tag':
                    db.execute("INSERT INTO tags VALUES (?, ?, ?)", (
                        elem.attrib['id'],
                        nd.attrib['k'],
                        nd.attrib['v'],
                    ))

def main(xmlpath, dbpath):
    with sqlite3.connect(dbpath) as conn:
        setup(conn)
        parse(xmlpath, conn)

if __name__ == "__main__":
    main(*sys.argv[1:])
