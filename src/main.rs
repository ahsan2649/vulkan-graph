mod app;
mod core;

use app::App;
use egui::Visuals;

fn main() {
    let mut app = App::new();

    app.register_math_nodes();

    eframe::run_native(
        "Vulkan-Graph",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(app))),
    );

    println!("Done!");
}
