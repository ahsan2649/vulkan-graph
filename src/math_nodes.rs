use std::cell::RefCell;

use std::rc::Rc;

use crate::core::{GraphNode, GraphNodePort};

pub struct SubtractNode {
    pub next: Option<Rc<RefCell<dyn GraphNode>>>,
    pub input_a: GraphNodePort<i32>,
    pub input_b: GraphNodePort<i32>,
    pub output: GraphNodePort<i32>,
}

impl GraphNode for SubtractNode {
    fn execute_node(&mut self) {
        if self.input_a.value != None && self.input_b.value != None {
            self.output.value = Some(self.input_a.value.unwrap() - self.input_b.value.unwrap())
        }
    }

    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>> {
        return &self.next;
    }
}

pub struct AddNode {
    pub next: Option<Rc<RefCell<dyn GraphNode>>>,
    pub input_a: GraphNodePort<i32>,
    pub input_b: GraphNodePort<i32>,
    pub output: GraphNodePort<i32>,
}

impl GraphNode for AddNode {
    fn execute_node(&mut self) {
        if self.input_a.value != None && self.input_b.value != None {
            self.output.value = Some(self.input_a.value.unwrap() + self.input_b.value.unwrap())
        }
    }

    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>> {
        return &self.next;
    }
}
