use std::cmp;

static DEGREE: uint = 32;

#[deriving(Show)]
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

    fn split(&self) -> Vec<Node<T>> {
        let ref rect = self.rect;
        let hw = rect.width() / 2;
        let hh = rect.height() / 2;
        let rects = vec![
            Rect { x0: rect.x0,    y0: rect.y0,    x1: rect.x0+hw, y1: rect.y0+hh, },
            Rect { x0: rect.x0+hw, y0: rect.y0,    x1: rect.x1,    y1: rect.y0+hh, },
            Rect { x0: rect.x0+hw, y0: rect.y0+hh, x1: rect.x1,    y1: rect.y1,    },
            Rect { x0: rect.x0,    y0: rect.y0+hh, x1: rect.x0+hw, y1: rect.y1,    },
        ];
        return rects.iter().map(|r| Node { rect: *r, sub: SubNodes(Vec::new()) }).collect();
    }

    fn insert(self, new: Node<T>) {
        let subnodes = match self.sub {
            Leaf(_) => fail!("dafuq"),
            SubNodes(n) => n,
        };
        let has_subs = subnodes.iter().any(|n| !n.is_leaf());
        
        if subnodes.len() < DEGREE {
        } else {
        }
    }
}

fn main() { 
    let leafs = vec![Node::leaf(0,0,50,50, "aap"), Node::leaf(50,50,100,100, "noot")];
    let root = Node::inter(0,0,100,100, leafs);
    println!("Hello, world! {}", root)
    root.insert(Node::leaf(25, 25, 75, 75, "mies"));
}
