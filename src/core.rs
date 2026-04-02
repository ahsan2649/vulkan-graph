use std::cell::RefCell;
use std::rc::Rc;

pub trait GraphNode {
    fn execute_node(&mut self);
    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>>;
    fn propagate(&mut self);
}

pub struct GraphNodePort<T> {
    pub value: Option<T>,
    pub connected_edge: Option<Rc<RefCell<GraphPortEdge<T>>>>,
}

impl<T> GraphNodePort<T> {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(GraphNodePort {
            value: None,
            connected_edge: None,
        }))
    }
}

pub struct GraphPortEdge<T> {
    pub from: Rc<RefCell<GraphNodePort<T>>>,
    pub to: Rc<RefCell<GraphNodePort<T>>>,
}

impl<T: Copy> GraphPortEdge<T> {
    pub fn connect(
        from: Rc<RefCell<GraphNodePort<T>>>,
        to: Rc<RefCell<GraphNodePort<T>>>,
    ) -> Rc<RefCell<Self>> {
        let edge = Rc::new(RefCell::new(GraphPortEdge {
            from: from.clone(),
            to: to.clone(),
        }));

        from.borrow_mut().connected_edge = Some(edge.clone());
        to.borrow_mut().connected_edge = Some(edge.clone());

        return edge;
    }

    pub fn propagate(edge: &Rc<RefCell<Self>>) {
        let edge = edge.borrow();
        let val = edge.from.borrow().value;
        if let Some(v) = val {
            edge.to.borrow_mut().value = Some(v)
        }
    }
}
