extern crate serialize;

use std::cmp;

use std::rand::{task_rng, Rng};
use serialize::json;

static DEGREE: uint = 32;

#[deriving(Show)]
enum InsertResult<T> {
    Inserted(Node<T>),
    Full(Node<T>, Node<T>),
}

#[deriving(Show, Rand, Encodable)]
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
        // this number makes no sense, but I hope it works.
        // It's growth in any direction * area
        return (north + east + south + west) * self.height() * self.width();
    }

    fn grow(&self, rect: Rect) -> Rect {
        return Rect {
            x0: cmp::min(self.x0, rect.x0),
            y0: cmp::min(self.y0, rect.y0),
            x1: cmp::max(self.x1, rect.x1),
            y1: cmp::max(self.y1, rect.y1),
        }
    }

    fn split(&self) -> Vec<Rect> {
        let hw = self.width() / 2;
        let hh = self.height() / 2;
        return vec![
            Rect { x0: self.x0,    y0: self.y0,    x1: self.x0+hw, y1: self.y0+hh, },
            Rect { x0: self.x0+hw, y0: self.y0,    x1: self.x1,    y1: self.y0+hh, },
            Rect { x0: self.x0+hw, y0: self.y0+hh, x1: self.x1,    y1: self.y1,    },
            Rect { x0: self.x0,    y0: self.y0+hh, x1: self.x0+hw, y1: self.y1,    },
        ];
    }
}

impl<T> Node<T> {
    fn leaf(x0: int, y0: int, x1: int, y1: int, data: T) -> Node<T> {
        return Node {
            rect: Rect { x0: x0, y0: y0, x1: x1, y1: y1, },
            sub: Leaf(data),
        }
    }

    fn inter(x0: int, y0: int, x1: int, y1: int, sub: Vec<Node<T>>) -> Node<T> {
        return Node {
            rect: Rect { x0: x0, y0: y0, x1: x1, y1: y1, },
            sub: SubNodes(sub),
        }
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

    fn insert_(mut self, new: Node<T>) -> InsertResult<T> {
        let has_subs = self.subnodes().iter().any(|n| !n.is_leaf());
        let has_space = self.subnodes().len() < DEGREE;
        let rect = self.rect.grow(new.rect);
        
        if has_subs {
            // ***************************
            // There are subnodes, descend.
            println!("subs");
            let node = {
                let subnodes = self.subnodes();
                let (index, _) = subnodes.iter().enumerate()
                    .filter(|&(_, n)| !n.is_leaf())
                    .min_by(|&(_, n)| n.rect.needed_growth(new.rect))
                    .expect("no insertable subs");
                println!("best is {}", index);
                subnodes.swap_remove(index).expect("no node to remove")
            };
            match node.insert_(new) {
                Inserted(newchild) => {
                    // ***************************
                    // Node inserted. Back out.
                    println!("inserted");
                    let mut subnodes = self.move_subnodes();
                    subnodes.push(newchild);
                    return Inserted(Node {
                        rect: rect,
                        sub: SubNodes(subnodes),
                    });
                },
                Full(node, new) => {
                    // ***************************
                    // Child full. Split.
                    println!("child full");
                    let rects = rect.split();
                    let newsubs: Vec<Node<T>> = rects.into_iter().map(|r| Node {
                        rect: r,
                        sub: SubNodes(Vec::with_capacity(DEGREE))
                    }).collect();
                    if newsubs.len() + self.subnodes().len() > DEGREE {
                        self.subnodes().push(node);
                        return Full(self, new);
                    }
                    for n in newsubs.into_iter() {
                        self.subnodes().push(n);
                    }
                    let mut this = self;
                    for n in node.move_subnodes().into_iter() {
                        this = match this.insert_(n) {
                            Inserted(s) => s,
                            Full(_, _) => fail!("could not insert split node"),
                        }
                    }
                    return this.insert_(new);
                }
            }
        } else if has_space {
            // ***************************
            // Add the new node at this level
            println!("inserting");
            let mut subnodes = self.move_subnodes();
            subnodes.push(new);
            return Inserted(Node {
                rect: rect,
                sub: SubNodes(subnodes),
            });
        } else {
            // ***************************
            // This node is full
            println!("full");
            return Full(self, new);
        }
    }

    fn insert(self, new: Node<T>) -> Node<T> {
        match self.insert_(new) {
            Inserted(node) => return node,
            Full(mut node, new) => {
                let mut newroot: Node<T> = Node {
                    rect: node.rect,
                    sub: SubNodes(Vec::with_capacity(DEGREE))
                };
                let rects = node.rect.split();
                let newsubs: Vec<Node<T>> = rects.into_iter().map(|r| Node {
                    rect: r,
                    sub: SubNodes(Vec::with_capacity(DEGREE))
                }).collect();
                for n in newsubs.into_iter() {
                    newroot.subnodes().push(n);
                }
                let mut this = newroot;
                for n in node.move_subnodes().into_iter() {
                    this = match this.insert_(n) {
                        Inserted(s) => s,
                        Full(_, _) => fail!("could NOT insert split node!!"),
                    }
                }
                return this.insert(new);
            }
        }
    }
}

fn main() { 
    let mut root = Node::inter(0,0,100,100, vec![]);
    let mut rng = task_rng();
    for i in range(0u, 10000) {
        root = root.insert(Node {
            rect: rng.gen(),
            sub: Leaf(i),
        });
        println!("#########################");
    }
    println!("{}", json::encode(&root));
}
