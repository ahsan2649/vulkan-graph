mod eval_registry;
mod node_definition;
mod node_graph;
mod node_instance;
mod node_registry;
mod port_definition;
mod port_instance;

struct App {
    node_graph: node_graph::NodeGraph,
    node_registry: node_registry::NodeRegistry,
    eval_registry: eval_registry::EvalRegistry,
}

impl App {
    fn new() -> Self {
        let node_registry = node_registry::NodeRegistry::new();
        let eval_registry = eval_registry::EvalRegistry::new();

        App {
            node_graph: node_graph::NodeGraph::new(),
            node_registry: node_registry,
            eval_registry: eval_registry,
        }
    }
    fn init(&mut self) {
        // Register "Start" Node
        self.node_registry.register_node(node_definition::NodeDefinition {
            id: node_definition::NodeDefId("Start"),
            name: "Start".to_owned(),
            kind: node_definition::NodeKind::Exec,
            input_ports: vec![],
            output_ports: vec![],
            exec_in_count: 0,
            exec_out_count: 1,
        });

        // Register "Add" Node
        self.node_registry.register_node(node_definition::NodeDefinition {
            id: node_definition::NodeDefId("Add"),
            name: "Add".to_owned(),
            kind: node_definition::NodeKind::Exec,
            input_ports: vec![
                port_definition::PortDefinition {
                    id: port_definition::PortDefId("i32"),
                    name: "A".to_owned(),
                    port_type: port_definition::PortType::UInt,
                },
                port_definition::PortDefinition {
                    id: port_definition::PortDefId("i32"),
                    name: "B".to_owned(),
                    port_type: port_definition::PortType::UInt,
                },
            ],
            output_ports: vec![port_definition::PortDefinition {
                id: port_definition::PortDefId("i32"),
                name: "Out".to_owned(),
                port_type: port_definition::PortType::UInt,
            }],
            exec_in_count: 1,
            exec_out_count: 1,
        });

        // Add "Start" Node to Graph
        self.node_graph
            .add_node_by_def_id(&node_definition::NodeDefId("Start"), &self.node_registry);
    }
}

fn main() {
    let mut app = App::new();
    app.init();
}
