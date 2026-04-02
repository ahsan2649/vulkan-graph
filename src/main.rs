use crate::{
    math_nodes::{AddNode, SubtractNode},
    print_node::PrintNode,
    start_node::StartNode,
};
use core::*;
use std::{cell::RefCell, rc::Rc};

mod core;
mod print_node;
mod start_node;

mod math_nodes;

fn main() {
    // Create nodes
    let start_node = StartNode::new();
    let b = PrintNode::new();
    let c = PrintNode::new();
    let d = SubtractNode::new();
    let e = AddNode::new();
    let f = SubtractNode::new();

    // Set node values
    b.borrow_mut().input_value.borrow_mut().value = Some(String::from("Hello World!"));
    c.borrow_mut().input_value.borrow_mut().value = Some(String::from("Hello World Again!"));

    d.borrow_mut().input_a.borrow_mut().value = Some(125);
    d.borrow_mut().input_b.borrow_mut().value = Some(100);

    e.borrow_mut().input_a.borrow_mut().value = Some(30);
    e.borrow_mut().input_b.borrow_mut().value = Some(20);

    GraphPortEdge::connect(d.borrow().output.clone(), f.borrow().input_a.clone());
    GraphPortEdge::connect(e.borrow().output.clone(), f.borrow().input_b.clone());

    // Wire nodes
    start_node.borrow_mut().next = Some(b.clone());
    b.borrow_mut().next = Some(c.clone());
    c.borrow_mut().next = Some(d.clone());
    d.borrow_mut().next = Some(e.clone());
    e.borrow_mut().next = Some(f.clone());

    // Run execution chain
    run_execution_chain(start_node);

    println!("{}", f.borrow().output.borrow().value.unwrap())
}

fn run_execution_chain(start_node: Rc<RefCell<StartNode>>) {
    let mut current_node: Rc<RefCell<dyn GraphNode>> = start_node;
    loop {
        current_node.borrow_mut().execute_node();
        current_node.borrow_mut().propagate();

        let next = current_node.borrow().next_node().clone();
        match next {
            Some(next_node) => current_node = next_node,
            None => break,
        }
    }
}
