mod app;
mod definition;
mod instance;
mod registry;

use egui::Visuals;
use egui_node_graph2::{NodeId, UserResponseTrait};

use crate::{
    app::App,
    definition::{NodeDefinition, PortDefinition},
    instance::{NodeInstance, PortInstance},
};

#[derive(Debug, Clone)]
pub enum UserResponse {
    SetActiveNode(NodeId),
    ClearActiveNode,
}

pub struct GraphState {
    pub active_node: Option<NodeId>,
}

impl UserResponseTrait for UserResponse {}

fn main() {
    let mut app = App::default();

    eframe::run_native(
        "Vulkan-Graph",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::dark());
            Ok(Box::new(app))
        }),
    )
    .expect("Failed to run native example");
}
