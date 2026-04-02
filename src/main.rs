use crate::{math_nodes::AddNode, start_node::StartNode};
use core::*;
use std::{cell::RefCell, rc::Rc};

mod core;
mod print_node;
mod start_node;

mod math_nodes;

fn main() {
    // Create nodes
    let start_node = Rc::new(RefCell::new(StartNode { next: None }));
    let b = Rc::new(RefCell::new(print_node::PrintNode {
        input_value: core::GraphNodePort { value: None },
        next: None,
    }));
    let c = Rc::new(RefCell::new(print_node::PrintNode {
        input_value: core::GraphNodePort { value: None },
        next: None,
    }));

    let d = Rc::new(RefCell::new(AddNode {
        next: None,
        input_a: GraphNodePort { value: None },
        input_b: GraphNodePort { value: None },
        output: GraphNodePort { value: None },
    }));

    // Set node values
    b.borrow_mut().input_value.value = Some(String::from("Hello World!"));
    c.borrow_mut().input_value.value = Some(String::from("Hello World Again!"));
    {
        let mut d = d.borrow_mut();
        d.input_a.value = Some(5);
        d.input_b.value = Some(6)
    }

    // Wire nodes
    start_node.borrow_mut().next = Some(b.clone());
    b.borrow_mut().next = Some(c.clone());
    c.borrow_mut().next = Some(d.clone());
    // Run execution chain
    run_execution_chain(start_node);

    println!(
        "The value of Add is {}",
        d.borrow_mut().output.value.unwrap()
    )
}

fn run_execution_chain(start_node: Rc<RefCell<StartNode>>) {
    let mut current_node: Rc<RefCell<dyn GraphNode>> = start_node;
    loop {
        current_node.borrow_mut().execute_node();
        let next = current_node.borrow().next_node().clone();
        match next {
            Some(next_node) => current_node = next_node,
            None => break,
        }
    }
}
