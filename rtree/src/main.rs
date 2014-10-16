extern crate serialize;

use std::cmp;
use std::default::Default;

use std::rand::{task_rng, Rng};
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

#[deriving(Show, Encodable)]
enum NodeData<T> {
    SubNodes(Vec<Node<T>>),
    Leaf(T),
}

#[deriving(Show, Encodable)]
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

    fn needed_growth(&self, rect: Rect) -> int {
        let north = cmp::max(0, self.y0 - rect.y0);
        let east  = cmp::max(0, rect.x1 - self.x1);
        let south = cmp::max(0, rect.y1 - self.y1);
        let west  = cmp::max(0, self.x0 - rect.x0);
        return (north + south) * self.height() + (east + west) * self.width();
    }

    fn grow(&self, rect: Rect) -> Rect {
        return Rect {
            x0: cmp::min(self.x0, rect.x0),
            y0: cmp::min(self.y0, rect.y0),
            x1: cmp::max(self.x1, rect.x1),
            y1: cmp::max(self.y1, rect.y1),
        }
    }
}

impl<T> Node<T> {

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

    fn subnodes(&mut self) -> &mut Vec<Node<T>> {
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

    fn split(mut self) -> (Node<T>, Node<T>) {
        let mut sub1 = Node::new((*self.subnodes())[0].rect.clone());
        let mut sub2 = Node::new((*self.subnodes())[1].rect.clone());
        for n in self.move_subnodes().into_iter() {
            if sub1.rect.needed_growth(n.rect) > sub2.rect.needed_growth(n.rect) {
                sub2.rect = sub2.rect.grow(n.rect);
                sub2.subnodes().push(n);
            } else {
                sub1.rect = sub1.rect.grow(n.rect);
                sub1.subnodes().push(n);
            }
        }
        return (sub1, sub2);
    }

    fn insert_(mut self, new: Node<T>) -> InsertResult<T> {
        let has_subs = self.subnodes().iter().any(|n| !n.is_leaf());
        let has_space = self.subnodes().len() < DEGREE;
        let rect = self.rect.grow(new.rect);
        
        if has_subs {
            // ***************************
            // There are subnodes, descend.
            //println!("subs");
            let node = {
                let subnodes = self.subnodes();
                let (index, _) = subnodes.iter().enumerate()
                    .filter(|&(_, n)| !n.is_leaf())
                    .min_by(|&(_, n)| n.rect.needed_growth(new.rect))
                    .expect("no insertable subs");
                //println!("best is {}", index);
                subnodes.swap_remove(index).expect("no node to remove")
            };
            match node.insert_(new) {
                Inserted(newchild) => {
                    // ***************************
                    // Node inserted. Back out.
                    //println!("inserted");
                    let mut subnodes = self.move_subnodes();
                    subnodes.push(newchild);
                    return Inserted(Node {
                        rect: rect,
                        sub: SubNodes(subnodes),
                    });
                },
                Full(mut node, new) => {
                    // ***************************
                    // Child full. Split.
                    //println!("child full");
                    node.subnodes().push(new);
                    let (n1, n2) = node.split();
                    self.subnodes().push(n1);
                    if self.subnodes().len() < DEGREE {
                        self.subnodes().push(n2);
                        return Inserted(self);
                    } else {
                        return Full(self, n2);
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
            Full(node, new) => {
                let mut newroot: Node<T> = Node::new(node.rect);
                let (n1, n2) = node.split();
                newroot.subnodes().push(n1);
                newroot.subnodes().push(n2);
                return newroot.insert(new);
            }
        }
    }
}

fn main() { 
    let mut root: Node<uint> = Node::new(Default::default());
    let mut rng = task_rng();
    let mut i = 0u;
    while i < 10000 {
        let r: Rect = rng.gen();
        if r.x1 > r.x0 && r.y1 > r.y0 {
            i = i + 1;
            root = root.insert(Node {
                rect: r,
                sub: Leaf(i),
            });
            //println!("#########################");
        }
    }
    println!("{}", json::encode(&root));
}
