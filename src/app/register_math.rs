use crate::{
    app::App,
    core::{
        definition::{NodeDefinition, NodeType, PortDefinition, PortKind, PortType},
        instance::PortValue,
    },
};

impl App {
    pub fn register_math_nodes(&mut self) {
        let add_node = self.register_function_node(
            NodeDefinition::new(
                "Add".to_owned(),
                NodeType::Function,
                vec![
                    PortDefinition::new("A".to_owned(), PortType::Int, PortKind::Data),
                    PortDefinition::new("B".to_owned(), PortType::Int, PortKind::Data),
                ],
                vec![PortDefinition::new(
                    "Out".to_owned(),
                    PortType::Int,
                    PortKind::Data,
                )],
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

        let subtract_node = self.register_function_node(
            NodeDefinition::new(
                "Subtract".to_owned(),
                NodeType::Function,
                vec![
                    PortDefinition::new("A".to_owned(), PortType::Int, PortKind::Data),
                    PortDefinition::new("B".to_owned(), PortType::Int, PortKind::Data),
                ],
                vec![PortDefinition::new(
                    "Out".to_owned(),
                    PortType::Int,
                    PortKind::Data,
                )],
            ),
            |node_instance| {
                let PortValue::Int(a) = &node_instance.input_values[0].port_value else {
                    panic!("Unexpected Variant!");
                };
                let PortValue::Int(b) = &node_instance.input_values[1].port_value else {
                    panic!("Unexpected Variant!");
                };

                node_instance.output_values[0].port_value = PortValue::Int(a - b);
            },
        );

        let multiply = self.register_function_node(
            NodeDefinition::new(
                "Multiply".to_owned(),
                NodeType::Function,
                vec![
                    PortDefinition::new("A".to_owned(), PortType::Int, PortKind::Data),
                    PortDefinition::new("B".to_owned(), PortType::Int, PortKind::Data),
                ],
                vec![PortDefinition::new(
                    "Out".to_owned(),
                    PortType::Int,
                    PortKind::Data,
                )],
            ),
            |node_instance| {
                let PortValue::Int(a) = &node_instance.input_values[0].port_value else {
                    panic!("Unexpected Variant!");
                };
                let PortValue::Int(b) = &node_instance.input_values[1].port_value else {
                    panic!("Unexpected Variant!");
                };

                node_instance.output_values[0].port_value = PortValue::Int(a * b);
            },
        );

        let divide_node = self.register_function_node(
            NodeDefinition::new(
                "Divide".to_owned(),
                NodeType::Function,
                vec![
                    PortDefinition::new("A".to_owned(), PortType::Int, PortKind::Data),
                    PortDefinition::new("B".to_owned(), PortType::Int, PortKind::Data),
                ],
                vec![PortDefinition::new(
                    "Out".to_owned(),
                    PortType::Int,
                    PortKind::Data,
                )],
            ),
            |node_instance| {
                let PortValue::Int(a) = &node_instance.input_values[0].port_value else {
                    panic!("Unexpected Variant!");
                };
                let PortValue::Int(b) = &node_instance.input_values[1].port_value else {
                    panic!("Unexpected Variant!");
                };

                node_instance.output_values[0].port_value = PortValue::Int(a / b);
            },
        );
    }
}
