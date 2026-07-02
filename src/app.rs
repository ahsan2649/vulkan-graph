pub mod register_math;

use crate::core::{
    definition::{NodeDefinition, NodeDefinitionId},
    instance::{NodeInstance, NodeInstanceId},
    node_graph::NodeGraph,
    registry::{EvaluationRegistry, NodeRegistry},
};

pub struct App {
    pub node_graph: NodeGraph,
    pub node_registy: NodeRegistry,
    pub evaluation_registry: EvaluationRegistry,
}

impl App {
    pub fn new() -> Self {
        Self {
            node_graph: NodeGraph::new(),
            node_registy: NodeRegistry::new(),
            evaluation_registry: EvaluationRegistry::new(),
        }
    }

    pub fn register_function_node(
        &mut self,
        node_definition: NodeDefinition,
        evaluation_definition: fn(&mut NodeInstance),
    ) -> NodeDefinitionId {
        let node_definition_id = node_definition.node_definition_id;
        self.node_registy.register_node(node_definition);
        self.evaluation_registry
            .register_evaluation(node_definition_id, evaluation_definition);
        return node_definition_id;
    }

    pub fn instantiate_node(&mut self, node_definition_id: NodeDefinitionId) -> NodeInstanceId {
        let node_instance = NodeInstance::from(
            self.node_registy
                .node_definitions
                .get(&node_definition_id)
                .unwrap(),
        );

        let node_instance_id = node_instance.node_instance_id;

        self.node_graph.insert(node_instance);

        return node_instance_id;
    }
}
