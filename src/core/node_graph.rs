use std::{borrow::Cow, collections::HashMap};

use egui_node_graph2::DataTypeTrait;

use crate::core::{
    definition::PortType,
    instance::{NodeInstance, NodeInstanceId, PortInstance, PortInstanceId},
};

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

impl DataTypeTrait<NodeGraph> for PortType {
    fn data_type_color(&self, user_state: &mut NodeGraph) -> egui::Color32 {
        match self {
            PortType::String => egui::Color32::from_rgb(38, 109, 211),
            PortType::Int => egui::Color32::from_rgb(38, 109, 211),
            PortType::Float => egui::Color32::from_rgb(38, 109, 211),
            PortType::Bool => egui::Color32::from_rgb(38, 109, 211),
            PortType::Exec => egui::Color32::from_rgb(38, 109, 211),
        }
    }

    fn name(&self) -> std::borrow::Cow<str> {
        match self {
            PortType::String => Cow::Borrowed("String"),
            PortType::Int => Cow::Borrowed("Int"),
            PortType::Float => Cow::Borrowed("Float"),
            PortType::Bool => Cow::Borrowed("Bool"),
            PortType::Exec => Cow::Borrowed("Exec"),
        }
    }
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
