use std::collections::HashMap;

use crate::core::instance::{NodeInstance, NodeInstanceId, PortInstance, PortInstanceId};

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

    pub fn insert(&mut self, node_instance: NodeInstance) {
        self.nodes
            .insert(node_instance.node_instance_id, node_instance);
    }

    pub fn connect_data(&mut self, from_port: &PortInstance, to_port: &PortInstance) {
        self.data_connections.0.insert(
            (from_port.node_instance_id, from_port.port_instance_id),
            (to_port.node_instance_id, to_port.port_instance_id),
        );
    }

    pub fn connect_exec(&mut self, from_port: &PortInstance, to_port: &PortInstance) {
        self.data_connections.0.insert(
            (from_port.node_instance_id, from_port.port_instance_id),
            (to_port.node_instance_id, to_port.port_instance_id),
        );
    }
}
