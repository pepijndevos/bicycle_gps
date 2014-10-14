use std::cmp;

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
    sub: Box<NodeData<T>>,
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
            sub: box Leaf(data),
        }
    }

    fn inter(x0: int, y0: int, x1: int, y1: int, sub: Vec<Node<T>>) -> Node<T> {
        return Node {
            rect: Rect { x0: x0, y0: y0, x1: x1, y1: y1, },
            sub: box SubNodes(sub),
        }
    }
}

fn main() { 
    let leafs = vec![Node::leaf(0,0,50,50, "aap"), Node::leaf(50,50,100,100, "noot")];
    let root = Node::inter(0,0,100,100, leafs);
    println!("Hello, world! {}", root)
}
