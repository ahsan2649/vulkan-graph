use std::borrow::Cow;

use egui_node_graph2::{DataTypeTrait, InputParamKind, NodeId, NodeTemplateTrait};
use uuid::Uuid;

use crate::{
    GraphState,
    instance::{NodeInstance, PortInstance},
};

pub type NodeDefinitionId = Uuid;

#[derive(PartialEq, Eq, Clone)]
pub enum PortType {
    String,
    Int,
    Float,
    Bool,
    Exec,
    VkCreateInfo,
    VkInstance,
    VkStructureType,
    VkApplicationInfo,
    VkPhysicalDevice,
    VecVkPhysicalDevice,
}

#[derive(Eq, Clone)]
pub struct PortDefinition {
    pub name: String,
    pub port_type: PortType,
}

impl PortDefinition {
    pub fn new(name: String, port_type: PortType) -> Self {
        Self { name, port_type }
    }
    pub fn exec() -> Self {
        Self {
            name: "".to_owned(),
            port_type: PortType::Exec,
        }
    }
}

impl PartialEq for PortDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.port_type == other.port_type
    }
}

impl DataTypeTrait<GraphState> for PortDefinition {
    fn data_type_color(&self, user_state: &mut GraphState) -> egui::Color32 {
        match self.port_type {
            PortType::String => egui::Color32::from_rgb(38, 109, 211),
            PortType::Int => egui::Color32::from_rgb(38, 109, 211),
            PortType::Float => egui::Color32::from_rgb(38, 109, 211),
            PortType::Bool => egui::Color32::from_rgb(38, 109, 211),
            PortType::Exec => egui::Color32::from_rgb(255, 255, 255),
            PortType::VkCreateInfo => egui::Color32::from_rgb(109, 38, 211),
            PortType::VkInstance => egui::Color32::from_rgb(211, 38, 211),
            PortType::VkStructureType => egui::Color32::from_rgb(100, 50, 211),
            PortType::VkApplicationInfo => egui::Color32::from_rgb(123, 45, 67),
            PortType::VkPhysicalDevice => egui::Color32::from_rgb(89, 101, 112),
            PortType::VecVkPhysicalDevice => egui::Color32::from_rgb(131, 41, 51),
        }
    }

    fn name(&self) -> std::borrow::Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }
}

#[derive(Clone)]
pub struct NodeDefinition {
    pub name: String,
    pub definition_id: NodeDefinitionId,
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
            name: name,
            definition_id: Uuid::new_v4(),
            input_ports: input_ports,
            output_ports: output_ports,
        }
    }
}

impl NodeTemplateTrait for NodeDefinition {
    type NodeData = NodeInstance;

    type DataType = PortDefinition;

    type ValueType = PortInstance;

    type UserState = GraphState;

    type CategoryType = &'static str;

    fn node_finder_label(&self, user_state: &mut Self::UserState) -> std::borrow::Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, user_state: &mut Self::UserState) -> Self::NodeData {
        NodeInstance {
            name: self.name.clone(),
            definition_id: self.definition_id,
        }
    }

    fn build_node(
        &self,
        graph: &mut egui_node_graph2::Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        self.input_ports.iter().for_each(|port| {
            graph.add_input_param(
                node_id,
                port.name.to_string(),
                port.to_owned(),
                PortInstance::from(port),
                egui_node_graph2::InputParamKind::ConnectionOrConstant,
                true,
            );
        });

        self.output_ports.iter().for_each(|port| {
            graph.add_output_param(node_id, port.name.to_string(), port.to_owned());
        });
    }
}
