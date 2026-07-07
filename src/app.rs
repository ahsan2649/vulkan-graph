use std::collections::HashMap;

use egui_node_graph2::{Graph, GraphEditorState, NodeResponse};

use crate::{
    GraphState, NodeDefinition, NodeInstance, PortInstance, UserResponse,
    definition::PortDefinition, registry::NodeRegistry,
};

pub type NodeGraph = Graph<NodeInstance, PortDefinition, PortInstance>;
pub type EditorState =
    GraphEditorState<NodeInstance, PortDefinition, PortInstance, NodeDefinition, GraphState>;

pub struct App {
    pub state: EditorState,
    pub user_state: GraphState,
    pub node_registry: NodeRegistry,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: EditorState::default(),
            user_state: GraphState { active_node: None },
            node_registry: NodeRegistry {
                node_definitions: HashMap::new(),
            },
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
            })
        });

        let graph_response = egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.state.draw_graph_editor(
                    ui,
                    self.node_registry.clone(),
                    &mut self.user_state,
                    Vec::default(),
                )
            })
            .inner;

        for node_response in graph_response.node_responses {
            if let NodeResponse::User(user_event) = node_response {
                match user_event {
                    UserResponse::SetActiveNode(node) => self.user_state.active_node = Some(node),
                    UserResponse::ClearActiveNode => self.user_state.active_node = None,
                }
            }
        }
    }
}
