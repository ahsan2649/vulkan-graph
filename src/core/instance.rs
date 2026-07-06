use std::collections::HashMap;

use crate::core::definition::NodeDefinition;
use crate::core::definition::NodeDefinitionId;
use crate::core::definition::PortDefinition;
use crate::core::definition::PortKind;
use crate::core::definition::PortType;
use crate::core::node_graph::NodeGraph;
use egui::Checkbox;
use egui::DragValue;
use egui_node_graph2::Graph;
use egui_node_graph2::GraphEditorState;
use egui_node_graph2::NodeDataTrait;
use egui_node_graph2::NodeId;
use egui_node_graph2::UserResponseTrait;
use egui_node_graph2::WidgetValueTrait;
use uuid::Uuid;

pub enum PortValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Exec,
    None,
}

impl Default for PortValue {
    fn default() -> Self {
        PortValue::None
    }
}

impl From<&PortType> for PortValue {
    fn from(port_type: &PortType) -> Self {
        match port_type {
            PortType::String => Self::String("".to_owned()),
            PortType::Int => Self::Int(0),
            PortType::Float => Self::Float(0.0),
            PortType::Bool => Self::Bool(true),
            PortType::Exec => Self::Exec,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct PortInstanceId(Uuid);

pub struct PortInstance {
    pub name: String,
    pub port_kind: PortKind,
    pub node_instance_id: NodeInstanceId,
    pub port_instance_id: PortInstanceId,
    pub port_value: PortValue,
}

impl PortInstance {
    pub fn new(node_instance_id: NodeInstanceId, port_definition: &PortDefinition) -> Self {
        Self {
            name: port_definition.name.clone(),
            port_kind: port_definition.port_kind,
            node_instance_id: node_instance_id,
            port_instance_id: PortInstanceId(Uuid::new_v4()),
            port_value: PortValue::from(&port_definition.port_type),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct NodeInstanceId(Uuid);

pub struct NodeInstance {
    pub name: String,
    pub node_definition_id: NodeDefinitionId,
    pub node_instance_id: NodeInstanceId,
    pub input_values: HashMap<PortInstanceId, PortInstance>,
    pub output_values: HashMap<PortInstanceId, PortInstance>,
}

impl From<&NodeDefinition> for NodeInstance {
    fn from(node_definition: &NodeDefinition) -> Self {
        let node_instance_id = NodeInstanceId(Uuid::new_v4());
        let input_values = node_definition
            .input_ports
            .iter()
            .map(|f| {
                let port_instance = PortInstance::new(node_instance_id, f);
                (port_instance.port_instance_id, port_instance)
            })
            .collect();
        let output_values = node_definition
            .output_ports
            .iter()
            .map(|f| {
                let port_instance = PortInstance::new(node_instance_id, f);
                (port_instance.port_instance_id, port_instance)
            })
            .collect();

        Self {
            name: node_definition.name.clone(),
            node_definition_id: node_definition.node_definition_id,
            node_instance_id,
            input_values,
            output_values,
        }
    }
}

impl NodeInstance {
    pub fn get_input_by_name(&self, name: String) -> Option<PortInstanceId> {
        let input = self
            .input_values
            .iter()
            .find(|(_, instance)| instance.name == name);

        match input {
            Some((id, _)) => Some(id.to_owned()),
            None => None,
        }
    }

    pub fn get_output_by_name(&self, name: String) -> Option<PortInstanceId> {
        let input = self
            .output_values
            .iter()
            .find(|(_, instance)| instance.name == name);

        match input {
            Some((id, _)) => Some(id.to_owned()),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MyResponse {
    SetActiveNode(NodeId),
    ClearActiveNode,
}

impl WidgetValueTrait for PortValue {
    type Response = MyResponse;
    type UserState = NodeGraph;
    type NodeData = NodeInstance;

    fn value_widget(
        &mut self,
        param_name: &str,
        node_id: NodeId,
        ui: &mut egui::Ui,
        user_state: &mut Self::UserState,
        node_data: &Self::NodeData,
    ) -> Vec<Self::Response> {
        match self {
            PortValue::String(value) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.text_edit_singleline(value);
                });
            }
            PortValue::Int(value) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(value));
                });
            }
            PortValue::Float(value) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(value));
                });
            }
            PortValue::Bool(value) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(Checkbox::new(value, param_name));
                });
            }
            PortValue::Exec => {}
            PortValue::None => {}
        }
        Vec::new()
    }
}

impl UserResponseTrait for MyResponse {}
impl NodeDataTrait for NodeInstance {
    type Response = MyResponse;
    type UserState = NodeGraph;
    type DataType = PortType;
    type ValueType = PortValue;

    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &egui_node_graph2::Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<egui_node_graph2::NodeResponse<Self::Response, Self>>
    where
        Self::Response: UserResponseTrait,
    {
        let mut responses = vec![];

        responses
    }
}

pub type MyGraph = Graph<NodeInstance, PortType, PortValue>;
pub type MyEditorState =
    GraphEditorState<NodeInstance, PortType, PortValue, NodeDefinition, NodeGraph>;
