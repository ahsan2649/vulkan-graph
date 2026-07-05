#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum PortKind {
    Exec,
    Data,
}

pub enum PortType {
    String,
    Int,
    Float,
    Bool,
    Exec,
}

pub struct PortDefinition {
    pub name: String,
    pub port_kind: PortKind,
    pub port_type: PortType,
}
impl PortDefinition {
    pub fn new(name: String, port_type: PortType, port_kind: PortKind) -> Self {
        Self {
            name,
            port_kind,
            port_type,
        }
    }
}

use uuid::Uuid;

pub enum NodeType {
    Function,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct NodeDefinitionId(Uuid);

pub struct NodeDefinition {
    pub name: String,
    pub node_type: NodeType,
    pub node_definition_id: NodeDefinitionId,
    pub input_ports: Vec<PortDefinition>,
    pub output_ports: Vec<PortDefinition>,
}

impl NodeDefinition {
    pub fn new(
        name: String,
        node_type: NodeType,
        input_ports: Vec<PortDefinition>,
        output_ports: Vec<PortDefinition>,
    ) -> Self {
        Self {
            name,
            node_type,
            node_definition_id: NodeDefinitionId(Uuid::new_v4()),
            input_ports,
            output_ports,
        }
    }
}
