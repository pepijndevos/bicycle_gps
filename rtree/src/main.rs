#![feature(slicing_syntax)]
extern crate serialize;

use std::cmp;
use std::default::Default;
use std::io::{Seek, Writer, IoResult, File};

use std::rand::{task_rng};
use std::rand::distributions::{IndependentSample, Range};
use serialize::json;

static DEGREE: uint = 32;

#[deriving(Show)]
enum InsertResult<T> {
    Inserted(Node<T>),
    Full(Node<T>, Node<T>),
}

#[deriving(Show, Rand, Default, Encodable, Clone)]
struct Rect {
    x0: int,
    y0: int,
    x1: int,
    y1: int,
}

#[deriving(Show, Encodable, Clone)]
enum NodeData<T> {
    SubNodes(Vec<Node<T>>),
    Leaf(T),
}

#[deriving(Show, Encodable, Clone)]
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

impl<T: Clone + TreeWriter> Node<T> {

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
            Leaf(_) => fail!("leaf has no nodes"),
            SubNodes(ref n) => n,
        };
    }

    fn mut_subnodes(&mut self) -> &mut Vec<Node<T>> {
        return match self.sub {
            Leaf(_) => fail!("leaf has no nodes"),
            SubNodes(ref mut n) => n,
        };
    }

    fn move_subnodes(self) -> Vec<Node<T>> {
        return match self.sub {
            Leaf(_) => fail!("leaf has no nodes"),
            SubNodes(n) => n,
        };
    }

    fn split(self) -> (Node<T>, Node<T>) {
        let mut sub1 = Node::new(self.subnodes()[0].rect.clone());
        let mut sub2 = Node::new(self.subnodes()[1].rect.clone());
        for n in self.move_subnodes().into_iter() {
            if sub1.rect.needed_growth(&n.rect) > sub2.rect.needed_growth(&n.rect) {
                sub2.rect = sub2.rect.grow(&n.rect);
                sub2.mut_subnodes().push(n);
            } else {
                sub1.rect = sub1.rect.grow(&n.rect);
                sub1.mut_subnodes().push(n);
            }
        }
        return (sub1, sub2);
    }

    fn best_node(&self, rect: &Rect) -> (uint, Node<T>) {
        let (index, noderef) = self.subnodes().iter().enumerate()
            .filter(|&(_, n)| !n.is_leaf())
            .min_by(|&(_, n)| n.rect.needed_growth(rect))
            .expect("no insertable subs");
        let node = noderef.clone();
        return (index, node);
    }

    fn insert_(self, new: Node<T>) -> InsertResult<T> {
        let has_subs = self.subnodes().iter().any(|n| !n.is_leaf());
        let has_space = self.subnodes().len() < DEGREE;
        // the rect for this node after inserting new
        let rect = self.rect.grow(&new.rect);
        
        if has_subs {
            // ***************************
            // There are subnodes, descend.
            //println!("subs");
            let (index, node) = self.best_node(&new.rect);
            //println!("best is {} of {}", index, self.subnodes().len());
            match node.insert_(new) {
                Inserted(newchild) => {
                    // ***************************
                    // Node inserted. Back out.
                    //println!("inserted");
                    let mut subnodes = self.move_subnodes();
                    subnodes[mut][index] = newchild;
                    return Inserted(Node {
                        rect: rect,
                        sub: SubNodes(subnodes),
                    });
                },
                Full(mut node, new) => {
                    // ***************************
                    // Child full. Split.
                    //println!("child full");
                    // add new anyway, then split.
                    node.mut_subnodes().push(new);
                    let (n1, n2) = node.split();
                    let mut subnodes = self.move_subnodes();
                    subnodes[mut][index] = n1;
                    if subnodes.len() < DEGREE {
                        subnodes.push(n2);
                        //println!("inserted after split");
                        return Inserted(Node {
                            rect: rect,
                            sub: SubNodes(subnodes),
                        });
                    } else {
                        //println!("full after split");
                        return Full(Node {
                            rect: rect,
                            sub: SubNodes(subnodes),
                        }, n2);
                    }
                }
            }
        } else if has_space {
            // ***************************
            // Add the new node at this level
            //println!("inserting");
            let mut subnodes = self.move_subnodes();
            subnodes.push(new);
            return Inserted(Node {
                rect: rect,
                sub: SubNodes(subnodes),
            });
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
                //println!("root full");
                let mut newroot: Node<T> = Node::new(node.rect);
                // add new anyway, then split.
                node.mut_subnodes().push(new);
                let (n1, n2) = node.split();
                newroot.mut_subnodes().push(n1);
                newroot.mut_subnodes().push(n2);
                return newroot;
            }
        }
    }
}

trait TreeWriter {
    fn write<U>(&self, w: &mut U) -> IoResult<u64> where U: Writer + Seek;
}

impl TreeWriter for uint {
    fn write<U>(&self, w: &mut U) -> IoResult<u64> where U: Writer + Seek {
        let offset = try!(w.tell());
        try!(w.write_be_uint(*self));
        return Ok(offset);
    }
}

impl<T: TreeWriter>  TreeWriter for Node<T> {
    fn write<U>(&self, w: &mut U) -> IoResult<u64> where U: Writer + Seek {
        match self.sub {
            Leaf(ref data) => {
                let offset = try!(w.tell());
                try!(w.write_u8(0)); // leaf node
                try!(w.write_be_i32(self.rect.x0 as i32));
                try!(w.write_be_i32(self.rect.y0 as i32));
                try!(w.write_be_i32(self.rect.x1 as i32));
                try!(w.write_be_i32(self.rect.y1 as i32));
                try!(data.write(w));
                return Ok(offset);
            },
            SubNodes(ref subs) => {
                let offsets: Vec<u64> = subs.iter().filter_map(|sub| sub.write(w).ok()).collect();
                let offset = try!(w.tell());
                try!(w.write_u8(offsets.len() as u8));
                try!(w.write_be_i32(self.rect.x0 as i32));
                try!(w.write_be_i32(self.rect.y0 as i32));
                try!(w.write_be_i32(self.rect.x1 as i32));
                try!(w.write_be_i32(self.rect.y1 as i32));
                for o in offsets.into_iter() {
                    try!(w.write_be_u32(o as u32));
                }
                return Ok(offset);
            },
        }
    }
}

fn main() { 
    let mut root: Node<uint> = Node::new(Default::default());
    let between = Range::new(0, 995i);
    let mut rng = task_rng();
    for i in range(0u, 10000) {
        let x = between.ind_sample(&mut rng);
        let y = between.ind_sample(&mut rng);
        let r = Rect { x0: x, y0: y, x1: x+5, y1: y+5, };
        root = root.insert(Node {
            rect: r,
            sub: Leaf(i),
        });
        //println!("#########################");
    }
    let ref mut f = File::create(&Path::new("data.bin")).ok().expect("no file");
    root.write(f);
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
    let root: Node<uint> = Node::new(Rect { x0: 0, y0: 0, x1: 100, y1: 100, });
    let newroot = root.insert(Node {
        rect: Rect { x0: 0, y0: 0, x1: 50, y1: 50, },
        sub: Leaf(1),
    }).insert(Node {
        rect: Rect { x0: 50, y0: 50, x1: 100, y1: 100, },
        sub: Leaf(1),
    });
    let (n1, n2) = newroot.split();
    assert_eq!(n1.subnodes().len(), 1);
    assert_eq!(n2.subnodes().len(), 1);
}
