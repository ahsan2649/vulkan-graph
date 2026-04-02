use std::cell::RefCell;

use std::rc::Rc;

use crate::core::{GraphNode, GraphNodePort, GraphPortEdge};

pub struct SubtractNode {
    pub next: Option<Rc<RefCell<dyn GraphNode>>>,
    pub input_a: Rc<RefCell<GraphNodePort<i32>>>,
    pub input_b: Rc<RefCell<GraphNodePort<i32>>>,
    pub output: Rc<RefCell<GraphNodePort<i32>>>,
}

impl SubtractNode {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(SubtractNode {
            next: None,
            input_a: GraphNodePort::new(),
            input_b: GraphNodePort::new(),
            output: GraphNodePort::new(),
        }))
    }
}

impl GraphNode for SubtractNode {
    fn execute_node(&mut self) {
        let a = self.input_a.borrow().value;
        let b = self.input_b.borrow().value;

        if let (Some(a), Some(b)) = (a, b) {
            self.output.borrow_mut().value = Some(a - b);
        }
    }

    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>> {
        return &self.next;
    }

    fn propagate(&mut self) {
        if let Some(edge) = self.output.borrow().connected_edge.clone() {
            GraphPortEdge::propagate(&edge);
        }
    }
}

pub struct AddNode {
    pub next: Option<Rc<RefCell<dyn GraphNode>>>,
    pub input_a: Rc<RefCell<GraphNodePort<i32>>>,
    pub input_b: Rc<RefCell<GraphNodePort<i32>>>,
    pub output: Rc<RefCell<GraphNodePort<i32>>>,
}

impl AddNode {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(AddNode {
            next: None,
            input_a: GraphNodePort::new(),
            input_b: GraphNodePort::new(),
            output: GraphNodePort::new(),
        }))
    }
}

impl GraphNode for AddNode {
    fn execute_node(&mut self) {
        let a = self.input_a.borrow().value;
        let b = self.input_b.borrow().value;

        if let (Some(a), Some(b)) = (a, b) {
            self.output.borrow_mut().value = Some(a + b);
        }
    }

    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>> {
        return &self.next;
    }

    fn propagate(&mut self) {
        if let Some(edge) = self.output.borrow().connected_edge.clone() {
            GraphPortEdge::propagate(&edge);
        }
    }
}
