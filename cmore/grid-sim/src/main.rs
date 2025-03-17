use std::cell::RefCell;
use std::rc::Rc;

pub struct Node {
    id: String,
    power: i32,
    connections: Vec<Rc<RefCell<Node>>>,
}

pub struct Wire {
    source: Node,
    sink: Node,
}

impl Node {
    fn new(id: &str, power: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            id: id.to_string(),
            power,
            connections: Vec::new(),
        }))
    }
    fn connect(&mut self, other: &Rc<RefCell<Node>>) {
        self.connections.push(other.clone());
    }
}

fn main() {
    let node1 = Node::new("node1", 10);
    let node2 = Node::new("node2", 20);

    node1.borrow_mut().connect(&node2);
    node2.borrow_mut().connect(&node1);

    println!("Hello, node1 power {}", node1.borrow().power);
    println!("Hello, node2 power {}", node2.borrow().power);
}
