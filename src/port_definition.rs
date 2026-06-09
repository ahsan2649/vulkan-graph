#[derive(Debug)]
pub enum PortType {
    Float,
    UInt,
    Bool,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct PortDefId(pub &'static str);

#[derive(Debug)]
pub struct PortDefinition {
    pub id: PortDefId,
    pub name: String,
    pub port_type: PortType,
}
