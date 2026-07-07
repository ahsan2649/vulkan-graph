use ash::vk::{PhysicalDevice, StructureType};
use egui::{DragValue, accesskit::Role::ComboBox};
use egui_node_graph2::{Graph, NodeDataTrait, NodeId, UserResponseTrait, WidgetValueTrait};

use crate::{
    GraphState, UserResponse,
    definition::{NodeDefinitionId, PortDefinition, PortType},
    instance::PortValue::{VkApplicationInfo, VkStructureType},
};

pub struct NodeInstance {
    pub name: String,
    pub definition_id: NodeDefinitionId,
}

impl NodeDataTrait for NodeInstance {
    type Response = UserResponse;

    type UserState = GraphState;

    type DataType = PortDefinition;

    type ValueType = PortInstance;

    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<egui_node_graph2::NodeResponse<Self::Response, Self>>
    where
        Self::Response: UserResponseTrait,
    {
        let mut responses = vec![];
        let is_active = user_state
            .active_node
            .map(|id| id == node_id)
            .unwrap_or(false);

        // Pressing the button will emit a custom user response to either set,
        // or clear the active node. These responses do nothing by themselves,
        // the library only makes the responses available to you after the graph
        // has been drawn. See below at the update method for an example.
        if !is_active {
            if ui.button("👁 Set active").clicked() {
                responses.push(egui_node_graph2::NodeResponse::User(
                    UserResponse::SetActiveNode(node_id),
                ));
            }
        } else {
            let button =
                egui::Button::new(egui::RichText::new("👁 Active").color(egui::Color32::BLACK))
                    .fill(egui::Color32::GOLD);
            if ui.add(button).clicked() {
                responses.push(egui_node_graph2::NodeResponse::User(
                    UserResponse::ClearActiveNode,
                ));
            }
        }

        responses
    }
}

pub enum PortValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    None,
    VkCreateInfo,
    VkInstance,
    VkStructureType(StructureType),
    VkApplicationInfo,
    VkPhysicalDevice,
    VecVkPhysicalDevice(Vec<PhysicalDevice>),
}

pub struct PortInstance {
    pub name: String,
    pub port_value: PortValue,
}

impl From<&PortDefinition> for PortInstance {
    fn from(value: &PortDefinition) -> Self {
        Self {
            name: value.name.to_owned(),
            port_value: match value.port_type {
                PortType::String => PortValue::String("".to_owned()),
                PortType::Int => PortValue::Int(0),
                PortType::Float => PortValue::Float(0.0),
                PortType::Bool => PortValue::Bool(false),
                PortType::Exec => PortValue::None,
                PortType::VkCreateInfo => PortValue::VkCreateInfo,
                PortType::VkInstance => PortValue::VkInstance,
                PortType::VkStructureType => PortValue::VkStructureType(StructureType::default()),
                PortType::VkApplicationInfo => PortValue::VkApplicationInfo,
                PortType::VkPhysicalDevice => PortValue::VkPhysicalDevice,
                PortType::VecVkPhysicalDevice => PortValue::VecVkPhysicalDevice(vec![]),
            },
        }
    }
}

impl Default for PortInstance {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
            port_value: PortValue::None,
        }
    }
}

impl WidgetValueTrait for PortInstance {
    type Response = UserResponse;
    type UserState = GraphState;
    type NodeData = NodeInstance;

    fn value_widget(
        &mut self,
        param_name: &str,
        node_id: NodeId,
        ui: &mut egui::Ui,
        user_state: &mut Self::UserState,
        node_data: &Self::NodeData,
    ) -> Vec<Self::Response> {
        ui.label(self.name.to_owned());
        match &mut self.port_value {
            PortValue::String(value) => {
                ui.text_edit_singleline(value);
            }
            PortValue::Int(value) => {
                ui.add(DragValue::new(value));
            }
            PortValue::Float(value) => {
                ui.add(DragValue::new(value));
            }
            PortValue::Bool(value) => {
                ui.checkbox(value, "");
            }
            PortValue::VkStructureType(value) => {
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", value))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            value,
                            StructureType::INSTANCE_CREATE_INFO,
                            "Instance Create Info",
                        );
                        ui.selectable_value(
                            value,
                            StructureType::DEVICE_CREATE_INFO,
                            "Device Create Info",
                        );
                        ui.selectable_value(
                            value,
                            StructureType::APPLICATION_INFO,
                            "Application Info",
                        );
                    });
            }
            _ => {}
        }
        Vec::new()
    }
}
