use uuid::Uuid;

use crate::{
    node_definition::{NodeDefId, NodeDefinition},
    port_instance::{DataPortInstance, ExecPortInstance},
};

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct NodeInstanceId(Uuid);

impl NodeInstanceId {
    fn new() -> Self {
        NodeInstanceId(Uuid::new_v4())
    }
}

#[derive(Debug)]
pub struct NodeInstance {
    pub id: NodeInstanceId,
    pub def_id: NodeDefId,
    pub input_ports: Vec<DataPortInstance>,
    pub output_ports: Vec<DataPortInstance>,
    pub exec_in_ports: Vec<ExecPortInstance>,
    pub exec_out_ports: Vec<ExecPortInstance>,
}

impl NodeInstance {
    pub fn from_def(def: &NodeDefinition) -> Self {
        NodeInstance {
            id: NodeInstanceId::new(),
            def_id: def.id.clone(),
            input_ports: def
                .input_ports
                .iter()
                .map(|p| DataPortInstance::new(p.id.clone()))
                .collect(),
            output_ports: def
                .output_ports
                .iter()
                .map(|p| DataPortInstance::new(p.id.clone()))
                .collect(),
            exec_in_ports: (0..def.exec_in_count)
                .map(|_| ExecPortInstance::new())
                .collect(),
            exec_out_ports: (0..def.exec_out_count)
                .map(|_| ExecPortInstance::new())
                .collect(),
        }
    }
}
