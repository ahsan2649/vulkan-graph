use std::collections::HashMap;

use crate::node_definition::NodeDefId;
use crate::node_graph::NodeGraph;
use crate::node_instance;

pub struct EvalRegistry {
    pub exec_handlers: HashMap<NodeDefId, fn(&mut node_instance::NodeInstance, &NodeGraph)>,
}

impl EvalRegistry {
    pub fn new() -> Self {
        Self {
            exec_handlers: HashMap::new(),
        }
    }
}
