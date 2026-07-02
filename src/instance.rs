use crate::definition::{NodeDefinition, NodeDefinitionId, PortDefinition, PortType};
use uuid::Uuid;

pub enum PortValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
}

impl From<&PortType> for PortValue {
    fn from(port_type: &PortType) -> Self {
        match port_type {
            PortType::String => Self::String("".to_owned()),
            PortType::Int => Self::Int(0),
            PortType::Float => Self::Float(0.0),
            PortType::Bool => Self::Bool(true),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct PortInstanceId(Uuid);

pub struct PortInstance {
    name: String,
    node_instance_id: NodeInstanceId,
    port_instance_id: PortInstanceId,
    pub port_value: PortValue,
}

impl PortInstance {
    pub fn new(node_instance_id: NodeInstanceId, port_definition: &PortDefinition) -> Self {
        Self {
            name: port_definition.name.clone(),
            node_instance_id: node_instance_id,
            port_instance_id: PortInstanceId(Uuid::new_v4()),
            port_value: PortValue::from(&port_definition.port_type),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct NodeInstanceId(Uuid);

pub struct NodeInstance {
    node_definition_id: NodeDefinitionId,
    pub node_instance_id: NodeInstanceId,
    pub input_values: Vec<PortInstance>,
    pub output_values: Vec<PortInstance>,
}

impl From<&NodeDefinition> for NodeInstance {
    fn from(node_definition: &NodeDefinition) -> Self {
        let node_instance_id = NodeInstanceId(Uuid::new_v4());
        let input_values = node_definition
            .input_ports
            .iter()
            .map(|f| PortInstance::new(node_instance_id, f))
            .collect();
        let output_values = node_definition
            .output_ports
            .iter()
            .map(|f| PortInstance::new(node_instance_id, f))
            .collect();

        Self {
            node_definition_id: node_definition.node_definition_id,
            node_instance_id,
            input_values,
            output_values,
        }
    }
}
