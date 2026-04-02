use std::cell::RefCell;

use std::rc::Rc;

use crate::core::{GraphNode, GraphNodePort};

pub struct PrintNode {
    pub input_value: Rc<RefCell<GraphNodePort<String>>>,
    pub next: Option<Rc<RefCell<dyn GraphNode>>>,
}

impl PrintNode {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(PrintNode {
            next: None,
            input_value: GraphNodePort::new(),
        }))
    }
}

impl GraphNode for PrintNode {
    fn execute_node(&mut self) {
        if let Some(v) = &self.input_value.borrow().value {
            println!("{}", v)
        }
    }

    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>> {
        return &self.next;
    }

    fn propagate(&mut self) {}
}
