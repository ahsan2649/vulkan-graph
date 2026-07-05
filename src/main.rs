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

    let add_node = app.instantiate_node(add);
    let subtract_node = app.instantiate_node(subtract);
    let multiply_node = app.instantiate_node(multiply);

    let add_evaluate = app
        .evaluation_registry
        .evaluation_definitions
        .get(&add)
        .unwrap();
    let subtract_evaluate = app
        .evaluation_registry
        .evaluation_definitions
        .get(&subtract)
        .unwrap();

    add_evaluate(app.node_graph.nodes.get_mut(&add_node).unwrap());
    subtract_evaluate(app.node_graph.nodes.get_mut(&subtract_node).unwrap());

    println!("Done!");
}
