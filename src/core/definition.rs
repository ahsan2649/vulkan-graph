pub enum PortType {
    String,
    Int,
    Float,
    Bool,
}

pub struct PortDefinition {
    pub name: String,
    pub port_type: PortType,
}
impl PortDefinition {
    pub fn new(name: String, port_type: PortType) -> Self {
        Self { name, port_type }
    }
}

use uuid::Uuid;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct NodeDefinitionId(Uuid);

pub struct NodeDefinition {
    pub name: String,
    pub node_definition_id: NodeDefinitionId,
    pub input_ports: Vec<PortDefinition>,
    pub output_ports: Vec<PortDefinition>,
}

impl NodeDefinition {
    pub fn new(
        name: String,
        input_ports: Vec<PortDefinition>,
        output_ports: Vec<PortDefinition>,
    ) -> Self {
        Self {
            name,
            node_definition_id: NodeDefinitionId(Uuid::new_v4()),
            input_ports,
            output_ports,
        }
    }
}
