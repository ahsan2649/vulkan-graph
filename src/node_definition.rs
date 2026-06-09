use crate::port_definition::PortDefinition;

#[derive(Debug)]
pub enum NodeKind {
    Exec,
    Pure,
    Variable,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct NodeDefId(pub &'static str);

#[derive(Debug)]
pub struct NodeDefinition {
    pub id: NodeDefId,
    pub name: String,
    pub kind: NodeKind,
    /// Data ports
    pub input_ports: Vec<PortDefinition>,
    pub output_ports: Vec<PortDefinition>,
    /// Exec ports carry no value — just a count / name for wiring
    pub exec_in_count: usize,
    pub exec_out_count: usize,
}
