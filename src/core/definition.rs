#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum PortKind {
    Exec,
    Data,
}

#[derive(Clone, Eq, PartialEq, Copy)]
pub enum PortType {
    String,
    Int,
    Float,
    Bool,
    Exec,
}

#[derive(Clone)]
pub struct PortDefinition {
    pub name: String,
    pub port_kind: PortKind,
    pub port_type: PortType,
}
impl PortDefinition {
    pub fn new(name: String, port_type: PortType, port_kind: PortKind) -> Self {
        Self {
            name,
            port_kind,
            port_type,
        }
    }
}

use std::borrow::Cow;

use egui_node_graph2::{NodeTemplateIter, NodeTemplateTrait};
use uuid::Uuid;

use crate::core::{
    instance::{NodeInstance, PortValue},
    node_graph::NodeGraph,
    registry::NodeRegistry,
};

#[derive(Clone)]
pub enum NodeType {
    Function,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct NodeDefinitionId(Uuid);

#[derive(Clone)]
pub struct NodeDefinition {
    pub name: String,
    pub node_type: NodeType,
    pub node_definition_id: NodeDefinitionId,
    pub input_ports: Vec<PortDefinition>,
    pub output_ports: Vec<PortDefinition>,
}

impl NodeDefinition {
    pub fn new(
        name: String,
        node_type: NodeType,
        input_ports: Vec<PortDefinition>,
        output_ports: Vec<PortDefinition>,
    ) -> Self {
        Self {
            name,
            node_type,
            node_definition_id: NodeDefinitionId(Uuid::new_v4()),
            input_ports,
            output_ports,
        }
    }
}

impl NodeTemplateTrait for NodeDefinition {
    type NodeData = NodeInstance;

    type DataType = PortType;

    type ValueType = PortValue;

    type UserState = NodeGraph;

    type CategoryType = &'static str;

    fn node_finder_label(&self, user_state: &mut Self::UserState) -> std::borrow::Cow<str> {
        Cow::Borrowed(self.name.as_str())
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, user_state: &mut Self::UserState) -> Self::NodeData {
        NodeInstance::from(self)
    }

    fn build_node(
        &self,
        graph: &mut egui_node_graph2::Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        node_id: egui_node_graph2::NodeId,
    ) {
        self.input_ports.iter().for_each(|port| {
            graph.add_input_param(
                node_id,
                port.name.clone(),
                port.port_type,
                PortValue::from(&port.port_type),
                egui_node_graph2::InputParamKind::ConnectionOnly,
                true,
            );
        });
        self.output_ports.iter().for_each(|port| {
            graph.add_output_param(node_id, port.name.clone(), port.port_type);
        });
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct VariableDefinitionId(Uuid);

pub struct VariableDefinition {
    pub name: String,
    pub variable_definition_id: VariableDefinitionId,
    pub variable_type: PortType,
    pub variable_value: PortValue,
}

impl NodeTemplateIter for NodeRegistry {
    type Item = NodeDefinition;

    fn all_kinds(&self) -> Vec<Self::Item> {
        self.node_definitions
            .values()
            .cloned()
            .map(|item| item)
            .collect()
    }
}
