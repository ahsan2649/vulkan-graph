mod app;
mod core;

use core::definition::NodeDefinition;
use core::definition::PortDefinition;
use core::definition::PortType;
use core::instance::PortValue;

use app::App;

fn main() {
    let mut app = App::new();

    app.register_math_nodes();
    let (add, _) = app.node_registy.find_by_name("Add".to_owned()).unwrap();
    app.instantiate_node(add.to_owned());

    let (subtract, _) = app
        .node_registy
        .find_by_name("Subtract".to_owned())
        .unwrap();
    app.instantiate_node(subtract.to_owned());

    let (multiply, _) = app
        .node_registy
        .find_by_name("Multiply".to_owned())
        .unwrap();

    app.instantiate_node(multiply.to_owned());

    println!("Done!");
}
