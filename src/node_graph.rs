use std::collections::HashMap;

use crate::instance::{NodeInstance, NodeInstanceId, PortInstanceId};

pub struct DataConnection(
    HashMap<(NodeInstanceId, PortInstanceId), (NodeInstanceId, PortInstanceId)>,
);
pub struct ExecConnection(
    HashMap<(NodeInstanceId, PortInstanceId), (NodeInstanceId, PortInstanceId)>,
);

pub struct NodeGraph {
    pub nodes: HashMap<NodeInstanceId, NodeInstance>,
    pub data_connections: DataConnection,
    pub exec_connections: ExecConnection,
}
impl NodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            data_connections: DataConnection(HashMap::new()),
            exec_connections: ExecConnection(HashMap::new()),
        }
    }
}
