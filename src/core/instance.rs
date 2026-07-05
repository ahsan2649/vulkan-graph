use std::collections::HashMap;

use crate::core::definition::NodeDefinition;
use crate::core::definition::NodeDefinitionId;
use crate::core::definition::PortDefinition;
use crate::core::definition::PortKind;
use crate::core::definition::PortType;
use uuid::Uuid;

pub enum PortValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Exec,
}

impl From<&PortType> for PortValue {
    fn from(port_type: &PortType) -> Self {
        match port_type {
            PortType::String => Self::String("".to_owned()),
            PortType::Int => Self::Int(0),
            PortType::Float => Self::Float(0.0),
            PortType::Bool => Self::Bool(true),
            PortType::Exec => Self::Exec,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct PortInstanceId(Uuid);

pub struct PortInstance {
    pub name: String,
    pub port_kind: PortKind,
    pub node_instance_id: NodeInstanceId,
    pub port_instance_id: PortInstanceId,
    pub port_value: PortValue,
}

impl PortInstance {
    pub fn new(node_instance_id: NodeInstanceId, port_definition: &PortDefinition) -> Self {
        Self {
            name: port_definition.name.clone(),
            port_kind: port_definition.port_kind,
            node_instance_id: node_instance_id,
            port_instance_id: PortInstanceId(Uuid::new_v4()),
            port_value: PortValue::from(&port_definition.port_type),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct NodeInstanceId(Uuid);

pub struct NodeInstance {
    pub node_definition_id: NodeDefinitionId,
    pub node_instance_id: NodeInstanceId,
    pub input_values: HashMap<PortInstanceId, PortInstance>,
    pub output_values: HashMap<PortInstanceId, PortInstance>,
}

impl From<&NodeDefinition> for NodeInstance {
    fn from(node_definition: &NodeDefinition) -> Self {
        let node_instance_id = NodeInstanceId(Uuid::new_v4());
        let input_values = node_definition
            .input_ports
            .iter()
            .map(|f| {
                let port_instance = PortInstance::new(node_instance_id, f);
                (port_instance.port_instance_id, port_instance)
            })
            .collect();
        let output_values = node_definition
            .output_ports
            .iter()
            .map(|f| {
                let port_instance = PortInstance::new(node_instance_id, f);
                (port_instance.port_instance_id, port_instance)
            })
            .collect();

        Self {
            node_definition_id: node_definition.node_definition_id,
            node_instance_id,
            input_values,
            output_values,
        }
    }
}

impl NodeInstance {
    pub fn get_input_by_name(&self, name: String) -> Option<PortInstanceId> {
        let input = self
            .input_values
            .iter()
            .find(|(_, instance)| instance.name == name);

        match input {
            Some((id, _)) => Some(id.to_owned()),
            None => None,
        }
    }

    pub fn get_output_by_name(&self, name: String) -> Option<PortInstanceId> {
        let input = self
            .output_values
            .iter()
            .find(|(_, instance)| instance.name == name);

        match input {
            Some((id, _)) => Some(id.to_owned()),
            None => None,
        }
    }
}
