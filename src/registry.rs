use std::collections::HashMap;

use crate::{
    definition::{NodeDefinition, NodeDefinitionId},
    instance::NodeInstance,
};

pub struct NodeRegistry {
    pub node_definitions: HashMap<NodeDefinitionId, NodeDefinition>,
}
impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            node_definitions: HashMap::new(),
        }
    }

    pub fn register_node(&mut self, node_definition: NodeDefinition) {
        self.node_definitions
            .insert(node_definition.node_definition_id.clone(), node_definition);
    }
}

pub struct EvaluationRegistry {
    pub evaluation_definitions: HashMap<NodeDefinitionId, fn(&mut NodeInstance)>,
}

impl EvaluationRegistry {
    pub fn new() -> Self {
        Self {
            evaluation_definitions: HashMap::new(),
        }
    }

    pub fn register_evaluation(
        &mut self,
        node_definition_id: NodeDefinitionId,
        evaluation_definition: fn(&mut NodeInstance),
    ) {
        self.evaluation_definitions
            .insert(node_definition_id, evaluation_definition);
    }
}
