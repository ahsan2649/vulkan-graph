use std::{cell::RefCell, rc::Rc};

pub trait GraphNode {
    fn execute_node(&self);
    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>>;
}

struct GraphNodePort<T> {
    value: Option<T>,
}

struct StartNode {
    next: Option<Rc<RefCell<dyn GraphNode>>>,
}

struct PrintNode {
    input_value: GraphNodePort<String>,
    next: Option<Rc<RefCell<dyn GraphNode>>>,
}

impl GraphNode for PrintNode {
    fn execute_node(&self) {
        if let Some(v) = &self.input_value.value {
            println!("{}", v)
        }
    }

    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>> {
        return &self.next;
    }
}

impl GraphNode for StartNode {
    fn execute_node(&self) {}

    fn next_node(&self) -> &Option<Rc<RefCell<dyn GraphNode>>> {
        return &self.next;
    }
}

fn main() {
    let a = Rc::new(RefCell::new(StartNode { next: None }));
    let b = Rc::new(RefCell::new(PrintNode {
        input_value: GraphNodePort { value: None },
        next: None,
    }));
    let c = Rc::new(RefCell::new(PrintNode {
        input_value: GraphNodePort { value: None },
        next: None,
    }));
    let d = Rc::new(RefCell::new(PrintNode {
        input_value: GraphNodePort { value: None },
        next: None,
    }));

    b.borrow_mut().input_value.value = Some(String::from("Hello World!"));
    c.borrow_mut().input_value.value = Some(String::from("Hello World Again!"));
    d.borrow_mut().input_value.value = Some(String::from("Hello World Once More!"));

    a.borrow_mut().next = Some(b.clone());
    b.borrow_mut().next = Some(c.clone());
    c.borrow_mut().next = Some(d.clone());

    let mut current_node: Rc<RefCell<dyn GraphNode>> = a;
    loop {
        current_node.borrow().execute_node();
        let next = current_node.borrow().next_node().clone();
        match next {
            Some(next_node) => current_node = next_node,
            None => break,
        }
    }
}
