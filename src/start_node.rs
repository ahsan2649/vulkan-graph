use std::{cell::RefCell, rc::Rc};

use crate::core::GraphNode;

pub struct StartNode {
    pub next: Option<Rc<RefCell<dyn GraphNode>>>,
}

impl GraphNode for StartNode {
    fn execute_node(&mut self) {}

    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>> {
        return &self.next;
    }
}
