use std::cell::RefCell;

use std::rc::Rc;

use crate::core::{GraphNode, GraphNodePort};

pub struct PrintNode {
    pub input_value: GraphNodePort<String>,
    pub next: Option<Rc<RefCell<dyn GraphNode>>>,
}

impl GraphNode for PrintNode {
    fn execute_node(&mut self) {
        if let Some(v) = &self.input_value.value {
            println!("{}", v)
        }
    }

    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>> {
        return &self.next;
    }
}
