use crate::node_definition::{NodeDefId, NodeDefinition, NodeKind};
use crate::port_definition::PortDefinition;
use crate::port_definition::{PortDefId, PortType};

use std::collections::HashMap;

#[derive(Debug)]
pub struct NodeRegistry {
    pub node_definitions: HashMap<NodeDefId, NodeDefinition>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        NodeRegistry {
            node_definitions: HashMap::new(),
        }
    }

    pub fn register_node(&mut self, node_definition: NodeDefinition) {
        self.node_definitions
            .insert(node_definition.id.clone(), node_definition);
    }
}
