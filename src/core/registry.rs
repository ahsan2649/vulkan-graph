use std::{collections::HashMap, ops::Deref};

use crate::core::{
    definition::{NodeDefinition, NodeDefinitionId, VariableDefinition, VariableDefinitionId},
    instance::NodeInstance,
};

#[derive(Clone)]
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

    pub fn find_by_name(&self, name: String) -> Option<NodeDefinitionId> {
        let value = self
            .node_definitions
            .iter()
            .find(|(_, value)| value.name == name);
        match value {
            Some((node_definition, _)) => return Some(node_definition.clone()),
            None => return None,
        }
    }
}

pub struct EvaluationRegistry {
    pub evaluation_definitions: HashMap<
        NodeDefinitionId,
        fn(&mut NodeInstance, &mut HashMap<VariableDefinitionId, VariableDefinition>),
    >,
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
        evaluation_definition: fn(
            &mut NodeInstance,
            &mut HashMap<VariableDefinitionId, VariableDefinition>,
        ),
    ) {
        self.evaluation_definitions
            .insert(node_definition_id, evaluation_definition);
    }
}
