mod app;
mod core;

use app::App;

fn main() {
    let mut app = App::new();

    app.register_math_nodes();
    let add = app.node_registy.find_by_name("Add".to_owned()).unwrap();

    let subtract = app
        .node_registy
        .find_by_name("Subtract".to_owned())
        .unwrap();

    let multiply = app
        .node_registy
        .find_by_name("Multiply".to_owned())
        .unwrap();

    app.instantiate_node(add.to_owned());
    app.instantiate_node(subtract.to_owned());
    app.instantiate_node(multiply.to_owned());

    println!("Done!");
}
