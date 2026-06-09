use crate::node_definition::NodeDefId;
use crate::node_instance;
use crate::node_instance::NodeInstanceId;
use crate::node_registry;
use crate::port_instance::PortInstanceId;

use std::collections::HashMap;

#[derive(Debug)]
pub struct NodeGraph {
    nodes: HashMap<NodeInstanceId, node_instance::NodeInstance>,
    /// data wire:  output PortInstanceId  →  input PortInstanceId
    /// (one output can feed many inputs, so Vec on the right)
    data_connections: HashMap<PortInstanceId, Vec<PortInstanceId>>,
    /// exec wire:  exec-out PortInstanceId  →  exec-in PortInstanceId
    exec_connections: HashMap<PortInstanceId, PortInstanceId>,
}

impl NodeGraph {
    pub fn new() -> Self {
        NodeGraph {
            nodes: HashMap::new(),
            data_connections: HashMap::new(),
            exec_connections: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, instance: node_instance::NodeInstance) -> NodeInstanceId {
        let id = instance.id.clone();
        self.nodes.insert(id.clone(), instance);
        id
    }

    pub fn add_node_by_def_id(
        &mut self,
        def_id: &NodeDefId,
        registry: &node_registry::NodeRegistry,
    ) {
        self.add_node(node_instance::NodeInstance::from_def(
            registry
                .node_definitions
                .get(def_id)
                .expect("Node Registry has no Add definition"),
        ));
    }

    /// Connect an output data port to an input data port.
    pub fn connect_data(&mut self, from: PortInstanceId, to: PortInstanceId) {
        self.data_connections.entry(from).or_default().push(to);
    }

    /// Connect an exec-out port to an exec-in port (one-to-one).
    pub fn connect_exec(&mut self, from: PortInstanceId, to: PortInstanceId) {
        self.exec_connections.insert(from, to);
    }
}
