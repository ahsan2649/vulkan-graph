use std::collections::HashMap;

use egui_node_graph2::NodeTemplateIter;

use crate::definition::{NodeDefinition, NodeDefinitionId};

#[derive(Clone)]
pub struct NodeRegistry {
    pub node_definitions: HashMap<NodeDefinitionId, NodeDefinition>,
}

impl NodeTemplateIter for NodeRegistry {
    type Item = NodeDefinition;

    fn all_kinds(&self) -> Vec<Self::Item> {
        self.node_definitions
            .clone()
            .values()
            .map(|item| item.to_owned())
            .collect()
    }
}
