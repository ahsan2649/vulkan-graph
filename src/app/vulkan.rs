use crate::{
    app::App,
    definition::{NodeDefinition, PortDefinition, PortType},
};

impl App {
    pub fn register_vulkan_nodes(&mut self) {
        self.register_application_info_node();
        self.register_instance_create_info_node();

        self.register_create_instance_node();
        self.register_enumerate_physical_devices_node();
    }

    fn register_application_info_node(&mut self) {
        let application_info_node = NodeDefinition::new(
            "VkApplicationInfo".to_owned(),
            vec![
                PortDefinition::new("sType".to_owned(), PortType::VkStructureType),
                PortDefinition::new("Application Name".to_owned(), PortType::String),
                PortDefinition::new("Application Version".to_owned(), PortType::Int),
                PortDefinition::new("Engine Name".to_owned(), PortType::String),
                PortDefinition::new("Engine Version".to_owned(), PortType::Int),
                PortDefinition::new("API Version".to_owned(), PortType::Int),
            ],
            vec![PortDefinition::new(
                "VkApplicationInfo".to_owned(),
                PortType::VkApplicationInfo,
            )],
        );

        self.node_registry
            .node_definitions
            .insert(application_info_node.definition_id, application_info_node);
    }
    fn register_instance_create_info_node(&mut self) {
        let instance_create_info_node = NodeDefinition::new(
            "VkInstanceCreateInfo".to_owned(),
            vec![
                PortDefinition::new("sType".to_owned(), PortType::VkStructureType),
                PortDefinition::new("Application Info".to_owned(), PortType::VkApplicationInfo),
            ],
            vec![PortDefinition::new(
                "VkCreateInfo".to_owned(),
                PortType::VkCreateInfo,
            )],
        );

        self.node_registry.node_definitions.insert(
            instance_create_info_node.definition_id,
            instance_create_info_node,
        );
    }
    fn register_create_instance_node(&mut self) {
        let create_instance_node = NodeDefinition::new(
            "vkCreateInstance".to_owned(),
            vec![
                PortDefinition::exec(),
                PortDefinition::new("VkCreateInfo".to_owned(), PortType::VkCreateInfo),
            ],
            vec![
                PortDefinition::exec(),
                PortDefinition::new("VkInstance".to_owned(), PortType::VkInstance),
            ],
        );

        self.node_registry
            .node_definitions
            .insert(create_instance_node.definition_id, create_instance_node);
    }
    fn register_enumerate_physical_devices_node(&mut self) {
        let enumerate_physical_devices_node = NodeDefinition::new(
            "vkEnumeratePhysicalDevices".to_owned(),
            vec![
                PortDefinition::exec(),
                PortDefinition::new("VkInstance".to_owned(), PortType::VkInstance),
            ],
            vec![
                PortDefinition::exec(),
                PortDefinition::new(
                    "VkPhysicalDevices".to_owned(),
                    PortType::VecVkPhysicalDevice,
                ),
            ],
        );

        self.node_registry.node_definitions.insert(
            enumerate_physical_devices_node.definition_id,
            enumerate_physical_devices_node,
        );
    }
}
