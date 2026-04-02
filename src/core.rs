use std::cell::RefCell;
use std::rc::Rc;

pub trait GraphNode {
    fn execute_node(&mut self);
    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>>;
}

pub struct GraphNodePort<T> {
    pub value: Option<T>,
}
