mod core;

use core::node_graph::NodeGraph;
use core::registry::EvaluationRegistry;
use core::registry::NodeRegistry;

use core::definition::NodeDefinition;
use core::definition::NodeDefinitionId;
use core::definition::PortDefinition;
use core::definition::PortType;
use core::instance::NodeInstance;
use core::instance::NodeInstanceId;
use core::instance::PortValue;

struct App {
    node_graph: NodeGraph,
    node_registy: NodeRegistry,
    evaluation_registry: EvaluationRegistry,
}

impl App {
    fn new() -> Self {
        Self {
            node_graph: NodeGraph::new(),
            node_registy: NodeRegistry::new(),
            evaluation_registry: EvaluationRegistry::new(),
        }
    }

    fn register_function_node(
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

    fn instantiate_node(&mut self, node_definition_id: NodeDefinitionId) -> NodeInstanceId {
        let mut node_instance = NodeInstance::from(
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

fn main() {
    let mut app = App::new();

    let add_node = app.register_function_node(
        NodeDefinition::new(
            "Add".to_owned(),
            vec![
                PortDefinition::new("A".to_owned(), PortType::Int),
                PortDefinition::new("B".to_owned(), PortType::Int),
            ],
            vec![PortDefinition::new("Out".to_owned(), PortType::Int)],
        ),
        |node_instance| {
            let PortValue::Int(a) = &node_instance.input_values[0].port_value else {
                panic!("Unexpected Variant!");
            };
            let PortValue::Int(b) = &node_instance.input_values[1].port_value else {
                panic!("Unexpected Variant!");
            };

            node_instance.output_values[0].port_value = PortValue::Int(a + b);
        },
    );

    let add_node_instance = app.instantiate_node(add_node);
    let add_function = app
        .evaluation_registry
        .evaluation_definitions
        .get(&add_node)
        .unwrap();

    let node_instance = app.node_graph.nodes.get_mut(&add_node_instance).unwrap();
    node_instance.input_values[0].port_value = PortValue::Int(5);
    node_instance.input_values[1].port_value = PortValue::Int(5);

    add_function(app.node_graph.nodes.get_mut(&add_node_instance).unwrap());

    println!("Done!");
}
