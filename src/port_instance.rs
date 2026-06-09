use crate::port_definition::PortDefId;
use uuid::Uuid;

#[derive(Debug)]
pub enum PortValue {
    Float(f32),
    UInt(u32),
    Bool(bool),
}

#[derive(Debug)]
pub struct DataPortInstance {
    pub id: PortInstanceId,
    pub def_id: PortDefId,
    pub value: Option<PortValue>,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct PortInstanceId(Uuid);

impl PortInstanceId {
    fn new() -> Self {
        PortInstanceId(Uuid::new_v4())
    }
}

impl DataPortInstance {
    pub fn new(def_id: PortDefId) -> Self {
        DataPortInstance {
            id: PortInstanceId::new(),
            def_id,
            value: None,
        }
    }
}

#[derive(Debug)]
pub struct ExecPortInstance {
    id: PortInstanceId,
}

impl ExecPortInstance {
    pub fn new() -> Self {
        ExecPortInstance {
            id: PortInstanceId::new(),
        }
    }
}
