mod definition;
mod instance;
mod node_graph;
mod registry;

use node_graph::NodeGraph;
use registry::EvaluationRegistry;
use registry::NodeRegistry;

use crate::definition::NodeDefinition;
use crate::definition::PortDefinition;
use crate::instance::NodeInstance;
use crate::instance::PortValue;

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
}

fn main() {
    let mut app = App::new();

    let add_node = NodeDefinition::new(
        "Add".to_owned(),
        vec![
            PortDefinition::new("A".to_owned(), definition::PortType::Int),
            PortDefinition::new("B".to_owned(), definition::PortType::Int),
        ],
        vec![PortDefinition::new(
            "Out".to_owned(),
            definition::PortType::Int,
        )],
    );
    let add_node_id = add_node.node_definition_id;

    app.node_registy.register_node(add_node);
    app.evaluation_registry
        .register_evaluation(add_node_id, |node_instance| {
            let PortValue::Int(a) = &node_instance.input_values[0].port_value else {
                panic!("Unexpected Variant!");
            };
            let PortValue::Int(b) = &node_instance.input_values[1].port_value else {
                panic!("Unexpected Variant!");
            };

            node_instance.output_values[0].port_value = PortValue::Int(a + b);
        });

    let mut add_node_instance =
        NodeInstance::from(app.node_registy.node_definitions.get(&add_node_id).unwrap());
    add_node_instance.input_values[0].port_value = PortValue::Int(5);
    add_node_instance.input_values[1].port_value = PortValue::Int(5);
    let add_node_instance_id = add_node_instance.node_instance_id;

    app.node_graph.insert(add_node_instance);

    let add_function = app
        .evaluation_registry
        .evaluation_definitions
        .get(&add_node_id)
        .unwrap();

    add_function(app.node_graph.nodes.get_mut(&add_node_instance_id).unwrap());

    println!("Done!");
}
