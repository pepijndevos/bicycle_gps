#![feature(slicing_syntax)]
extern crate postgres;
extern crate libc;
 
use libc::c_void;
use std::cmp;
use std::default::Default;
use std::io::{Seek, Writer, IoResult, File, SeekSet};

use postgres::{Connection, Row, Statement, Transaction, NoSsl};

// as much as you can fit in a 512 byte block
static DEGREE: uint = 25;

fn split_at<T>(mut left: Vec<T>, at: uint) -> (Vec<T>, Vec<T>) {
    if at >= left.len() {
        panic!("eh?!");
    }
 
    let right_len = left.len() - at;
    let mut right = Vec::with_capacity(DEGREE);
    unsafe {
        right.set_len(right_len);
        std::ptr::copy_nonoverlapping_memory(right.as_mut_ptr() as *mut c_void,
                     left.as_ptr().offset(at as int) as *const c_void,
                     (right_len * std::mem::size_of::<T>()));
        left.set_len(at);
    }
    return (left, right);
}

#[deriving(Show)]
enum InsertResult<T> {
    Inserted(Node<T>),
    Full(Node<T>, Node<T>),
}

#[deriving(Show, Clone, Default)]
struct Point {
    x: int,
    y: int,
}

#[deriving(Show, Clone, Default)]
struct Rect {
    x0: int,
    y0: int,
    x1: int,
    y1: int,
}

#[deriving(Show)]
enum NodeData<T> {
    SubNodes(Vec<Node<T>>),
    Leaf(T),
}

#[deriving(Show)]
struct Node<T> {
    rect: Rect,
    sub: NodeData<T>,
}

impl Rect {
    fn height(&self) -> int {
        return self.y1 - self.y0;
    }

    fn width(&self) -> int {
        return self.x1 - self.x0;
    }

    fn center(&self) -> Point {
        return Point {
            x: self.x0 + (self.width() / 2),
            y: self.y0 + (self.height() / 2),
        };
    }

    fn needed_growth(&self, rect: &Rect) -> int {
        let newrect = self.grow(rect);
        let diff = (newrect.width() * newrect.height()) - (self.width() * self.height());
        return cmp::max(0, diff);
    }

    fn grow(&self, rect: &Rect) -> Rect {
        return Rect {
            x0: cmp::min(self.x0, rect.x0),
            y0: cmp::min(self.y0, rect.y0),
            x1: cmp::max(self.x1, rect.x1),
            y1: cmp::max(self.y1, rect.y1),
        }
    }
}

impl<T: TreeWriter> Node<T> {

    fn new(rect: Rect) -> Node<T> {
        return Node {
            rect: rect,
            sub: SubNodes(Vec::with_capacity(DEGREE))
        };
    }

    fn is_leaf(&self) -> bool {
        return match self.sub {
            Leaf(_) => true,
            SubNodes(_) => false,
        };
    }

    fn subnodes(&self) -> &Vec<Node<T>> {
        return match self.sub {
            Leaf(_) => panic!("leaf has no nodes"),
            SubNodes(ref n) => n,
        };
    }

    fn mut_subnodes(&mut self) -> &mut Vec<Node<T>> {
        return match self.sub {
            Leaf(_) => panic!("leaf has no nodes"),
            SubNodes(ref mut n) => n,
        };
    }

    fn move_subnodes(self) -> Vec<Node<T>> {
        return match self.sub {
            Leaf(_) => panic!("leaf has no nodes"),
            SubNodes(n) => n,
        };
    }

    fn mut_leaf(&mut self) -> &mut T {
        return match self.sub {
            Leaf(ref mut l) => l,
            SubNodes(_) => panic!("Not a leaf"),
        };
    }

    fn split(self) -> [Node<T>, ..2] {
        let rect = self.rect;
        let mut subnodes = self.move_subnodes();
        let len = subnodes.len();
        if rect.width() > rect.height() {
           subnodes.sort_by(|n, m| n.rect.center().x.cmp(&m.rect.center().x));
        } else {
           subnodes.sort_by(|n, m| n.rect.center().y.cmp(&m.rect.center().y));
        }
        let (a, b) = split_at(subnodes, len/2);
        let mut n1: Node<T> = Node::new(Default::default());
        let mut n2: Node<T> = Node::new(Default::default());
        n1.sub = SubNodes(a);
        n1.update_rect();
        n2.sub = SubNodes(b);
        n2.update_rect();
        return [n1, n2];
    }

    fn update_rect(&mut self) {
        let init: Option<Rect> = None;
        self.rect = self.subnodes().iter().fold(init, |i, n| match i {
            None => Some(n.rect),
            Some(r) => Some(r.grow(&n.rect)),
        }).unwrap();
    }

    fn best_node(&mut self, rect: &Rect) -> Node<T> {
        let (index, _) = self.subnodes().iter().enumerate()
            .filter(|&(_, n)| !n.is_leaf())
            .min_by(|&(_, n)| n.rect.needed_growth(rect))
            .expect("no insertable subs");
        //println!("best is {} of {}", index, self.subnodes().len());
        return self.mut_subnodes().swap_remove(index).expect("node empty, huh?");
    }

    fn insert_(mut self, new: Node<T>) -> InsertResult<T> {
        if self.subnodes().iter().any(|n| !n.is_leaf()) {
            // ***************************
            // There are subnodes, descend.
            //println!("subs");
            let node = self.best_node(&new.rect);
            match node.insert_(new) {
                Inserted(newchild) => {
                    // ***************************
                    // Node inserted. Back out.
                    //println!("inserted, now {} children", self.subnodes().len());
                    assert!(self.subnodes().len() < DEGREE);
                    self.mut_subnodes().push(newchild);
                    self.update_rect();
                    return Inserted(self);
                },
                Full(mut newchild, new) => {
                    // ***************************
                    // Child full. Split.
                    // This is the tricky part.
                    //println!("child full");
                    newchild.mut_subnodes().push(new);
                    let [n1, n2] = newchild.split();
                    //println!("n1 = {}, n2 = {}", n1.subnodes().len(), n2.subnodes().len());
                    assert!(self.subnodes().len() < DEGREE);
                    self.mut_subnodes().push(n1);
                    if self.subnodes().len() < DEGREE {
                        self.mut_subnodes().push(n2);
                        self.update_rect();
                        //println!("inserted after split, now {} children", self.subnodes().len());
                        return Inserted(self);
                    } else {
                        self.update_rect();
                        //println!("full after split, now {} children", self.subnodes().len());
                        return Full(self, n2);
                    }
                }
            }
        } else if self.subnodes().len() < DEGREE {
            // ***************************
            // Add the new node at this level
            //println!("inserting in node with {} children", self.subnodes().len());
            self.rect = self.rect.grow(&new.rect);
            self.mut_subnodes().push(new);
            return Inserted(self);
        } else {
            // ***************************
            // This node is full
            //println!("full");
            return Full(self, new);
        }
    }

    fn insert(self, new: Node<T>) -> Node<T> {
        match self.insert_(new) {
            Inserted(node) => return node,
            Full(mut node, new) => {
                println!("root full");
                let mut newroot: Node<T> = Node::new(node.rect);
                node.mut_subnodes().push(new);
                let [n1, n2] = node.split();
                newroot.mut_subnodes().push(n1);
                newroot.mut_subnodes().push(n2);
                newroot.update_rect();
                return newroot;
            }
        }
    }
}

trait TreeWriter {
    fn write<U>(&self, w: &mut U) -> IoResult<u64> where U: Writer + Seek;
}

impl TreeWriter for Point {
    fn write<U>(&self, w: &mut U) -> IoResult<u64> where U: Writer + Seek {
        let offset = try!(w.tell());
        try!(w.write_le_i32(self.x as i32));
        try!(w.write_le_i32(self.y as i32));
        return Ok(offset);
    }
}

impl TreeWriter for Rect {
    fn write<U>(&self, w: &mut U) -> IoResult<u64> where U: Writer + Seek {
        let offset = try!(w.tell());
        try!(w.write_le_i32(self.x0 as i32));
        try!(w.write_le_i32(self.y0 as i32));
        try!(w.write_le_i32(self.x1 as i32));
        try!(w.write_le_i32(self.y1 as i32));
        return Ok(offset);
    }
}

fn seek_block(w: &mut Seek) -> IoResult<u64> {
    // seek forward to the nearest 512 bytes
    let offset = try!(w.tell()) + 511 & !511;
    assert!(offset % 512 == 0);
    try!(w.seek(offset as i64, SeekSet));
    return Ok(offset);
}

impl<T: TreeWriter> TreeWriter for Node<Vec<T>> {
    fn write<U>(&self, w: &mut U) -> IoResult<u64> where U: Writer + Seek {
        match self.sub {
            Leaf(ref data) => {
                let offset = try!(seek_block(w));
                try!(w.write_le_i32(-(data.len() as i32))); // leaf node
                try!(data.write(w));
                return Ok(offset);
            },
            SubNodes(ref subs) => {
                let offsets: Vec<u64> = subs.iter().filter_map(|sub| sub.write(w).ok()).collect();
                let offset = try!(seek_block(w));
                try!(w.write_le_i32(offsets.len() as i32));
                for (o, sub) in offsets.into_iter().zip(subs.iter()) {
                    try!(w.write_le_i32(o as i32));
                    try!(sub.rect.write(w));
                }
                return Ok(offset);
            },
        }
    }
}

impl<T: TreeWriter> TreeWriter for Vec<T> {
    fn write<U>(&self, w: &mut U) -> IoResult<u64> where U: Writer + Seek {
        let offset = try!(w.tell());
        for o in self.iter() {
            try!(o.write(w));
        }
        return Ok(offset);
    }
}

fn query<'a>(conn: &'a Transaction) -> Statement<'a> {
    return conn.prepare("SELECT w.id,
                                 ST_X(n.geom), ST_Y(n.geom),
                                 ST_XMIN(w.bbox), ST_YMIN(w.bbox),
                                 ST_XMAX(w.bbox), ST_YMAX(w.bbox),
                                 HSTORE_TO_JSON(tags)
                          FROM nodes as n
                          INNER JOIN way_nodes as wn ON n.id = wn.node_id
                          INNER JOIN ways as w ON w.id = wn.way_id
                          ORDER BY w.id, wn.sequence_id").unwrap();
}

fn get_fixedpoint(row: &Row, idx: uint) -> int {
    let fl: f64 = row.get(idx);
    return (fl * 10000000.0) as int;
}

fn main() { 
    let conn = Connection::connect("postgres://pepijndevos@localhost/osm", &NoSsl).unwrap();
    let trans = conn.transaction().unwrap();
    let mut root: Node<Vec<Point>> = Node::new(Default::default());
    let mut current_id: Option<i64> = None;
    let mut current_node: Option<Node<Vec<Point>>> = None;
    for rrow in trans.lazy_query(&query(&trans), &[], 2000).unwrap() {
        let row = rrow.unwrap();
        match current_id {
            Some(i) if i == row.get(0u) => {
                current_node = match current_node {
                    Some(mut node) => {
                        node.mut_leaf().push(Point {
                            x: get_fixedpoint(&row, 1),
                            y: get_fixedpoint(&row, 2),
                        });
                        Some(node)
                    }
                    None => None
                }
            }
            _ => {
                match current_node {
                    Some(node) => {
                        root = root.insert(node);
                        //println!("#########################");
                    }
                    None => ()
                }
                current_id = Some(row.get(0u));
                current_node = Some(Node {
                    rect: Rect {
                        x0: get_fixedpoint(&row, 3),
                        y0: get_fixedpoint(&row, 4),
                        x1: get_fixedpoint(&row, 5),
                        y1: get_fixedpoint(&row, 6),
                    },
                    sub: Leaf(vec![Point {
                        x: get_fixedpoint(&row, 1),
                        y: get_fixedpoint(&row, 2),
                    }]),
                });
            }
        }
    }
    let ref mut f = File::create(&Path::new("data.bin")).unwrap();
    f.seek(4, SeekSet).unwrap();
    let start = root.write(f).unwrap();
    f.seek(0, SeekSet).unwrap();
    f.write_le_i32(start as i32).unwrap();
}

#[test]
fn needed_growth_test() {
    let r1 = Rect { x0: 0, y0: 0, x1: 100, y1: 100, };
    let r2 = Rect { x0: 0, y0: 0, x1: 100, y1: 101, };
    let r3 = Rect { x0: 0, y0: 0, x1: 101, y1: 101, };
    let r4 = Rect { x0: -1, y0: -1, x1: 101, y1: 101, };
    assert_eq!(r1.needed_growth(&r2), 100);
    assert_eq!(r1.needed_growth(&r3), 201);
    assert_eq!(r1.needed_growth(&r4), 404);
}

#[test]
fn grow_test() {
    let r1 = Rect { x0: 0, y0: 0, x1: 100, y1: 100, };
    let r2 = Rect { x0: 200, y0: 200, x1: 200, y1: 200, };
    let g = r1.grow(&r2);
    assert_eq!(g.x0, 0);
    assert_eq!(g.y0, 0);
    assert_eq!(g.x1, 200);
    assert_eq!(g.y1, 200);
}

#[test]
fn split_test() {
    let root: Node<Point> = Node::new(Rect { x0: 0, y0: 0, x1: 100, y1: 100, });
    let newroot = root.insert(Node {
        rect: Rect { x0: 0, y0: 0, x1: 50, y1: 50, },
        sub: Leaf(Point { x: 0, y: 0}),
    }).insert(Node {
        rect: Rect { x0: 50, y0: 50, x1: 100, y1: 100, },
        sub: Leaf(Point { x: 0, y: 0}),
    });
    let [n1, n2] = newroot.split();
    assert_eq!(n1.subnodes().len(), 1);
    assert_eq!(n2.subnodes().len(), 1);
}

#[test]
fn split_at_test() {
    let v = vec![1u,2,3,4,5,6,7,8,9,0];
    assert_eq!(split_at(v, 5), (vec![1, 2, 3, 4, 5], vec![6, 7, 8, 9, 0]));
}
