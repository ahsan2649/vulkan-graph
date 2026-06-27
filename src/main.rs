//! A classic Vulkan "hello triangle" — but every single Vulkan call is
//! wrapped as a node in a small dataflow graph, and the *graph* is what
//! actually builds and renders the triangle. There is no function in this
//! file that calls `ash` directly outside of a `GraphNode::run` body.
//!
//! Structure of this file:
//!   PART 0 — a tiny generic node-graph engine (knows nothing about Vulkan)
//!   PART 1 — the `Value` enum: every kind of thing that can travel on a wire
//!   PART 2 — one `GraphNode` impl per Vulkan setup step (instance, device,
//!            swapchain, render pass, pipeline, framebuffers, command pool...)
//!   PART 3 — one `GraphNode` impl per *per-frame* Vulkan call (acquire,
//!            begin/record/end command buffer, submit, present)
//!   PART 4 — `main()`: wires PART 2's nodes into a "setup graph" that runs
//!            once, then drives winit's event loop, re-running a small
//!            "frame graph" (PART 3's nodes) every redraw.
//!
//! Build/runtime deps: ash, ash-window, winit, raw-window-handle.
//! Needs a Vulkan-capable GPU + driver + a windowing system to actually run.

use ash::extensions::khr::{Surface, Swapchain};
use ash::{vk, Device, Entry, Instance};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::collections::HashMap;
use std::ffi::CStr;
use std::rc::Rc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

// Pre-compiled SPIR-V for the two shaders, embedded as u32 words so the
// whole program stays a single file with no external assets.
pub const VERT_SPV: [u32; 358] = [
    0x07230203, 0x00010000, 0x0008000b, 0x00000036, 0x00000000, 0x00020011, 0x00000001, 0x0006000b,
    0x00000001, 0x4c534c47, 0x6474732e, 0x3035342e, 0x00000000, 0x0003000e, 0x00000000, 0x00000001,
    0x0008000f, 0x00000000, 0x00000004, 0x6e69616d, 0x00000000, 0x00000022, 0x00000026, 0x00000031,
    0x00030003, 0x00000002, 0x000001c2, 0x00040005, 0x00000004, 0x6e69616d, 0x00000000, 0x00050005,
    0x0000000c, 0x69736f70, 0x6e6f6974, 0x00000073, 0x00040005, 0x00000017, 0x6f6c6f63, 0x00007372,
    0x00060005, 0x00000020, 0x505f6c67, 0x65567265, 0x78657472, 0x00000000, 0x00060006, 0x00000020,
    0x00000000, 0x505f6c67, 0x7469736f, 0x006e6f69, 0x00070006, 0x00000020, 0x00000001, 0x505f6c67,
    0x746e696f, 0x657a6953, 0x00000000, 0x00070006, 0x00000020, 0x00000002, 0x435f6c67, 0x4470696c,
    0x61747369, 0x0065636e, 0x00070006, 0x00000020, 0x00000003, 0x435f6c67, 0x446c6c75, 0x61747369,
    0x0065636e, 0x00030005, 0x00000022, 0x00000000, 0x00060005, 0x00000026, 0x565f6c67, 0x65747265,
    0x646e4978, 0x00007865, 0x00050005, 0x00000031, 0x67617266, 0x6f6c6f43, 0x00000072, 0x00030047,
    0x00000020, 0x00000002, 0x00050048, 0x00000020, 0x00000000, 0x0000000b, 0x00000000, 0x00050048,
    0x00000020, 0x00000001, 0x0000000b, 0x00000001, 0x00050048, 0x00000020, 0x00000002, 0x0000000b,
    0x00000003, 0x00050048, 0x00000020, 0x00000003, 0x0000000b, 0x00000004, 0x00040047, 0x00000026,
    0x0000000b, 0x0000002a, 0x00040047, 0x00000031, 0x0000001e, 0x00000000, 0x00020013, 0x00000002,
    0x00030021, 0x00000003, 0x00000002, 0x00030016, 0x00000006, 0x00000020, 0x00040017, 0x00000007,
    0x00000006, 0x00000002, 0x00040015, 0x00000008, 0x00000020, 0x00000000, 0x0004002b, 0x00000008,
    0x00000009, 0x00000003, 0x0004001c, 0x0000000a, 0x00000007, 0x00000009, 0x00040020, 0x0000000b,
    0x00000006, 0x0000000a, 0x0004003b, 0x0000000b, 0x0000000c, 0x00000006, 0x0004002b, 0x00000006,
    0x0000000d, 0x00000000, 0x0004002b, 0x00000006, 0x0000000e, 0xbf000000, 0x0005002c, 0x00000007,
    0x0000000f, 0x0000000d, 0x0000000e, 0x0004002b, 0x00000006, 0x00000010, 0x3f000000, 0x0005002c,
    0x00000007, 0x00000011, 0x00000010, 0x00000010, 0x0005002c, 0x00000007, 0x00000012, 0x0000000e,
    0x00000010, 0x0006002c, 0x0000000a, 0x00000013, 0x0000000f, 0x00000011, 0x00000012, 0x00040017,
    0x00000014, 0x00000006, 0x00000003, 0x0004001c, 0x00000015, 0x00000014, 0x00000009, 0x00040020,
    0x00000016, 0x00000006, 0x00000015, 0x0004003b, 0x00000016, 0x00000017, 0x00000006, 0x0004002b,
    0x00000006, 0x00000018, 0x3f800000, 0x0006002c, 0x00000014, 0x00000019, 0x00000018, 0x0000000d,
    0x0000000d, 0x0006002c, 0x00000014, 0x0000001a, 0x0000000d, 0x00000018, 0x0000000d, 0x0006002c,
    0x00000014, 0x0000001b, 0x0000000d, 0x0000000d, 0x00000018, 0x0006002c, 0x00000015, 0x0000001c,
    0x00000019, 0x0000001a, 0x0000001b, 0x00040017, 0x0000001d, 0x00000006, 0x00000004, 0x0004002b,
    0x00000008, 0x0000001e, 0x00000001, 0x0004001c, 0x0000001f, 0x00000006, 0x0000001e, 0x0006001e,
    0x00000020, 0x0000001d, 0x00000006, 0x0000001f, 0x0000001f, 0x00040020, 0x00000021, 0x00000003,
    0x00000020, 0x0004003b, 0x00000021, 0x00000022, 0x00000003, 0x00040015, 0x00000023, 0x00000020,
    0x00000001, 0x0004002b, 0x00000023, 0x00000024, 0x00000000, 0x00040020, 0x00000025, 0x00000001,
    0x00000023, 0x0004003b, 0x00000025, 0x00000026, 0x00000001, 0x00040020, 0x00000028, 0x00000006,
    0x00000007, 0x00040020, 0x0000002e, 0x00000003, 0x0000001d, 0x00040020, 0x00000030, 0x00000003,
    0x00000014, 0x0004003b, 0x00000030, 0x00000031, 0x00000003, 0x00040020, 0x00000033, 0x00000006,
    0x00000014, 0x00050036, 0x00000002, 0x00000004, 0x00000000, 0x00000003, 0x000200f8, 0x00000005,
    0x0003003e, 0x0000000c, 0x00000013, 0x0003003e, 0x00000017, 0x0000001c, 0x0004003d, 0x00000023,
    0x00000027, 0x00000026, 0x00050041, 0x00000028, 0x00000029, 0x0000000c, 0x00000027, 0x0004003d,
    0x00000007, 0x0000002a, 0x00000029, 0x00050051, 0x00000006, 0x0000002b, 0x0000002a, 0x00000000,
    0x00050051, 0x00000006, 0x0000002c, 0x0000002a, 0x00000001, 0x00070050, 0x0000001d, 0x0000002d,
    0x0000002b, 0x0000002c, 0x0000000d, 0x00000018, 0x00050041, 0x0000002e, 0x0000002f, 0x00000022,
    0x00000024, 0x0003003e, 0x0000002f, 0x0000002d, 0x0004003d, 0x00000023, 0x00000032, 0x00000026,
    0x00050041, 0x00000033, 0x00000034, 0x00000017, 0x00000032, 0x0004003d, 0x00000014, 0x00000035,
    0x00000034, 0x0003003e, 0x00000031, 0x00000035, 0x000100fd, 0x00010038,
];

pub const FRAG_SPV: [u32; 125] = [
    0x07230203, 0x00010000, 0x0008000b, 0x00000013, 0x00000000, 0x00020011, 0x00000001, 0x0006000b,
    0x00000001, 0x4c534c47, 0x6474732e, 0x3035342e, 0x00000000, 0x0003000e, 0x00000000, 0x00000001,
    0x0007000f, 0x00000004, 0x00000004, 0x6e69616d, 0x00000000, 0x00000009, 0x0000000c, 0x00030010,
    0x00000004, 0x00000007, 0x00030003, 0x00000002, 0x000001c2, 0x00040005, 0x00000004, 0x6e69616d,
    0x00000000, 0x00050005, 0x00000009, 0x4374756f, 0x726f6c6f, 0x00000000, 0x00050005, 0x0000000c,
    0x67617266, 0x6f6c6f43, 0x00000072, 0x00040047, 0x00000009, 0x0000001e, 0x00000000, 0x00040047,
    0x0000000c, 0x0000001e, 0x00000000, 0x00020013, 0x00000002, 0x00030021, 0x00000003, 0x00000002,
    0x00030016, 0x00000006, 0x00000020, 0x00040017, 0x00000007, 0x00000006, 0x00000004, 0x00040020,
    0x00000008, 0x00000003, 0x00000007, 0x0004003b, 0x00000008, 0x00000009, 0x00000003, 0x00040017,
    0x0000000a, 0x00000006, 0x00000003, 0x00040020, 0x0000000b, 0x00000001, 0x0000000a, 0x0004003b,
    0x0000000b, 0x0000000c, 0x00000001, 0x0004002b, 0x00000006, 0x0000000e, 0x3f800000, 0x00050036,
    0x00000002, 0x00000004, 0x00000000, 0x00000003, 0x000200f8, 0x00000005, 0x0004003d, 0x0000000a,
    0x0000000d, 0x0000000c, 0x00050051, 0x00000006, 0x0000000f, 0x0000000d, 0x00000000, 0x00050051,
    0x00000006, 0x00000010, 0x0000000d, 0x00000001, 0x00050051, 0x00000006, 0x00000011, 0x0000000d,
    0x00000002, 0x00070050, 0x00000007, 0x00000012, 0x0000000f, 0x00000010, 0x00000011, 0x0000000e,
    0x0003003e, 0x00000009, 0x00000012, 0x000100fd, 0x00010038,
];

const MAX_FRAMES_IN_FLIGHT: usize = 2;

// ============================================================================
// PART 0: GENERIC NODE-GRAPH EXECUTION ENGINE
// ============================================================================
// A minimal dataflow runtime. Nodes declare named input/output ports; edges
// connect an output port of one node to an input port of another; `execute`
// topologically sorts the graph by data dependency and runs each node once,
// feeding it whatever its upstream nodes produced. This part is completely
// generic — it has no idea what a `VkInstance` is.

/// A single node: a function from named input ports to named output ports.
trait GraphNode {
    fn label(&self) -> &str;
    fn input_ports(&self) -> &[&'static str];
    fn output_ports(&self) -> &[&'static str];
    /// `inputs` contains exactly the ports declared by `input_ports`.
    /// Must return exactly the ports declared by `output_ports`.
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value>;
}

type NodeId = usize;

/// "Input port `dst_port` of node `dst` is fed by output port `src_port`
/// of node `src`."
struct Edge {
    src: NodeId,
    src_port: &'static str,
    dst: NodeId,
    dst_port: &'static str,
}

struct Graph {
    nodes: Vec<Box<dyn GraphNode>>,
    edges: Vec<Edge>,
}

impl Graph {
    fn new() -> Self {
        Graph { nodes: Vec::new(), edges: Vec::new() }
    }

    /// Add a node to the graph, returning its id for use with `connect`.
    fn add(&mut self, node: impl GraphNode + 'static) -> NodeId {
        self.nodes.push(Box::new(node));
        self.nodes.len() - 1
    }

    /// Wire an output port of `src` to an input port of `dst`.
    fn connect(&mut self, src: NodeId, src_port: &'static str, dst: NodeId, dst_port: &'static str) {
        self.edges.push(Edge { src, src_port, dst, dst_port });
    }

    /// Feed an externally-supplied value directly into a node's input port,
    /// bypassing the rest of the graph (used for per-frame inputs like
    /// "which frame-in-flight index is this" that come from the outer event
    /// loop, not from another node). Implemented as a synthetic
    /// constant-producing node so it still goes through the normal
    /// port/edge machinery rather than being a special case in `execute`.
    fn feed(&mut self, value: Value, dst: NodeId, dst_port: &'static str) {
        struct Const(Option<Value>);
        impl GraphNode for Const {
            fn label(&self) -> &str { "ExternalInput" }
            fn input_ports(&self) -> &[&'static str] { &[] }
            fn output_ports(&self) -> &[&'static str] { &["value"] }
            fn run(&mut self, _inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
                let mut m = HashMap::new();
                m.insert("value", self.0.take().expect("Const node run twice"));
                m
            }
        }
        let src = self.add(Const(Some(value)));
        self.connect(src, "value", dst, dst_port);
    }

    /// Topologically sort by data dependency (Kahn's algorithm), run each
    /// node in order threading values along edges, and return every output
    /// ever produced, keyed by (node id, port name).
    fn execute(&mut self) -> HashMap<(NodeId, &'static str), Value> {
        let n = self.nodes.len();

        let mut depends_on: Vec<Vec<NodeId>> = vec![Vec::new(); n];
        for e in &self.edges {
            if !depends_on[e.dst].contains(&e.src) {
                depends_on[e.dst].push(e.src);
            }
        }
        let mut remaining: Vec<usize> = depends_on.iter().map(|d| d.len()).collect();
        let mut dependents: Vec<Vec<NodeId>> = vec![Vec::new(); n];
        for i in 0..n {
            for &dep in &depends_on[i] {
                dependents[dep].push(i);
            }
        }

        let mut ready: Vec<NodeId> = (0..n).filter(|&i| remaining[i] == 0).collect();
        let mut order: Vec<NodeId> = Vec::with_capacity(n);
        while let Some(id) = ready.pop() {
            order.push(id);
            for &dep_node in &dependents[id] {
                remaining[dep_node] -= 1;
                if remaining[dep_node] == 0 {
                    ready.push(dep_node);
                }
            }
        }
        assert_eq!(order.len(), n, "node graph has a cycle");

        let mut results: HashMap<(NodeId, &'static str), Value> = HashMap::new();

        for id in order {
            let mut inputs: HashMap<&'static str, Value> = HashMap::new();
            for e in &self.edges {
                if e.dst == id {
                    let v = results
                        .get(&(e.src, e.src_port))
                        .unwrap_or_else(|| panic!(
                            "node '{}' needs port '{}' before node '{}' produced it",
                            self.nodes[id].label(), e.dst_port, self.nodes[e.src].label()
                        ))
                        .clone();
                    inputs.insert(e.dst_port, v);
                }
            }
            let label = self.nodes[id].label().to_string();
            let outputs = self.nodes[id].run(&inputs);
            for port in self.nodes[id].output_ports() {
                let v = outputs.get(port).unwrap_or_else(|| {
                    panic!("node '{}' did not produce declared output port '{}'", label, port)
                });
                results.insert((id, *port), v.clone());
            }
        }

        results
    }
}

/// Convenience: pull a single named output of a single node out of a
/// finished graph's results.
fn output_of(results: &HashMap<(NodeId, &'static str), Value>, id: NodeId, port: &'static str) -> Value {
    results[&(id, port)].clone()
}

// ============================================================================
// PART 1: THE VALUE ENUM
// ============================================================================
// Everything that travels along a wire in this graph is a `Value`. Most
// Vulkan handles are plain `Copy` types (just integers/pointers under the
// hood) so they're cheap to clone directly. A few outputs are naturally a
// *bundle* of related things produced together (e.g. the swapchain comes
// bundled with its images, format and extent) — those are wrapped in `Rc`
// so cloning a `Value` stays cheap even for bundles containing `Vec`s.

#[derive(Clone)]
struct SwapchainBundle {
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    format: vk::Format,
    extent: vk::Extent2D,
}

#[derive(Clone)]
struct SyncBundle {
    image_available: Vec<vk::Semaphore>,
    render_finished: Vec<vk::Semaphore>,
    in_flight: Vec<vk::Fence>,
}

#[derive(Clone)]
enum Value {
    Unit,
    Usize(usize),
    Extent(vk::Extent2D),
    PresentMode(vk::PresentModeKHR),
    SurfaceFormat(vk::SurfaceFormatKHR),
    SurfaceCaps(vk::SurfaceCapabilitiesKHR),
    QueueFamilyIndices(Rc<QueueFamilyIndices>),

    Window(Rc<Window>),
    Entry(Rc<Entry>),
    Instance(Rc<Instance>),
    SurfaceLoader(Rc<Surface>),
    SwapchainLoader(Rc<Swapchain>),
    SurfaceKHR(vk::SurfaceKHR),
    PhysicalDevice(vk::PhysicalDevice),
    Device(Rc<Device>),
    Queue(vk::Queue),

    SwapchainBundle(Rc<SwapchainBundle>),
    ImageViews(Rc<Vec<vk::ImageView>>),
    RenderPass(vk::RenderPass),
    ShaderModule(vk::ShaderModule),
    PipelineLayout(vk::PipelineLayout),
    Pipeline(vk::Pipeline),
    Framebuffers(Rc<Vec<vk::Framebuffer>>),
    CommandPool(vk::CommandPool),
    CommandBuffers(Rc<Vec<vk::CommandBuffer>>),
    SyncBundle(Rc<SyncBundle>),

    ImageIndex(u32),
    CommandBuffer(vk::CommandBuffer),
}

#[derive(Clone, Copy)]
struct QueueFamilyIndices {
    graphics: u32,
    present: u32,
}

// `unwrap_*` accessors keep node bodies free of `match ... else panic`
// boilerplate. A wrong-variant access is a programming error in how the
// graph was wired, so panicking with a clear message is the right thing —
// equivalent to a type error you'd get for free with statically-typed
// ports, just paid at graph-construction time instead of compile time.
impl Value {
    fn unwrap_usize(&self) -> usize { match self { Value::Usize(v) => *v, _ => panic!("expected Usize") } }
    fn unwrap_extent(&self) -> vk::Extent2D { match self { Value::Extent(v) => *v, _ => panic!("expected Extent") } }
    fn unwrap_present_mode(&self) -> vk::PresentModeKHR { match self { Value::PresentMode(v) => *v, _ => panic!("expected PresentMode") } }
    fn unwrap_surface_format(&self) -> vk::SurfaceFormatKHR { match self { Value::SurfaceFormat(v) => *v, _ => panic!("expected SurfaceFormat") } }
    fn unwrap_surface_caps(&self) -> vk::SurfaceCapabilitiesKHR { match self { Value::SurfaceCaps(v) => *v, _ => panic!("expected SurfaceCaps") } }
    fn unwrap_qfi(&self) -> Rc<QueueFamilyIndices> { match self { Value::QueueFamilyIndices(v) => v.clone(), _ => panic!("expected QueueFamilyIndices") } }
    fn unwrap_window(&self) -> Rc<Window> { match self { Value::Window(v) => v.clone(), _ => panic!("expected Window") } }
    fn unwrap_entry(&self) -> Rc<Entry> { match self { Value::Entry(v) => v.clone(), _ => panic!("expected Entry") } }
    fn unwrap_instance(&self) -> Rc<Instance> { match self { Value::Instance(v) => v.clone(), _ => panic!("expected Instance") } }
    fn unwrap_surface_loader(&self) -> Rc<Surface> { match self { Value::SurfaceLoader(v) => v.clone(), _ => panic!("expected SurfaceLoader") } }
    fn unwrap_swapchain_loader(&self) -> Rc<Swapchain> { match self { Value::SwapchainLoader(v) => v.clone(), _ => panic!("expected SwapchainLoader") } }
    fn unwrap_surface(&self) -> vk::SurfaceKHR { match self { Value::SurfaceKHR(v) => *v, _ => panic!("expected SurfaceKHR") } }
    fn unwrap_physical_device(&self) -> vk::PhysicalDevice { match self { Value::PhysicalDevice(v) => *v, _ => panic!("expected PhysicalDevice") } }
    fn unwrap_device(&self) -> Rc<Device> { match self { Value::Device(v) => v.clone(), _ => panic!("expected Device") } }
    fn unwrap_queue(&self) -> vk::Queue { match self { Value::Queue(v) => *v, _ => panic!("expected Queue") } }
    fn unwrap_swapchain_bundle(&self) -> Rc<SwapchainBundle> { match self { Value::SwapchainBundle(v) => v.clone(), _ => panic!("expected SwapchainBundle") } }
    fn unwrap_image_views(&self) -> Rc<Vec<vk::ImageView>> { match self { Value::ImageViews(v) => v.clone(), _ => panic!("expected ImageViews") } }
    fn unwrap_render_pass(&self) -> vk::RenderPass { match self { Value::RenderPass(v) => *v, _ => panic!("expected RenderPass") } }
    fn unwrap_shader_module(&self) -> vk::ShaderModule { match self { Value::ShaderModule(v) => *v, _ => panic!("expected ShaderModule") } }
    fn unwrap_pipeline_layout(&self) -> vk::PipelineLayout { match self { Value::PipelineLayout(v) => *v, _ => panic!("expected PipelineLayout") } }
    fn unwrap_pipeline(&self) -> vk::Pipeline { match self { Value::Pipeline(v) => *v, _ => panic!("expected Pipeline") } }
    fn unwrap_framebuffers(&self) -> Rc<Vec<vk::Framebuffer>> { match self { Value::Framebuffers(v) => v.clone(), _ => panic!("expected Framebuffers") } }
    fn unwrap_command_pool(&self) -> vk::CommandPool { match self { Value::CommandPool(v) => *v, _ => panic!("expected CommandPool") } }
    fn unwrap_command_buffers(&self) -> Rc<Vec<vk::CommandBuffer>> { match self { Value::CommandBuffers(v) => v.clone(), _ => panic!("expected CommandBuffers") } }
    fn unwrap_sync_bundle(&self) -> Rc<SyncBundle> { match self { Value::SyncBundle(v) => v.clone(), _ => panic!("expected SyncBundle") } }
    fn unwrap_image_index(&self) -> u32 { match self { Value::ImageIndex(v) => *v, _ => panic!("expected ImageIndex") } }
    fn unwrap_command_buffer(&self) -> vk::CommandBuffer { match self { Value::CommandBuffer(v) => *v, _ => panic!("expected CommandBuffer") } }
}

// ============================================================================
// PART 2: SETUP NODES (run once, build everything up to "ready to render")
// ============================================================================
// Each node below corresponds to one (or a tight cluster of) Vulkan calls.
// They are deliberately *not* called directly from `main` — `main` only
// builds a `Graph` out of these and calls `execute()`. The dependency
// edges between nodes (wired in `main`) are exactly the data dependencies
// a normal imperative ash program would express via local variables and
// call order; here they're explicit wires instead of implicit sequencing.

/// Wraps the already-created `winit::window::Window` as a graph source
/// node, so every later node that needs window/display handles gets them
/// through a port rather than through an ambient variable.
struct WindowNode(Rc<Window>);
impl GraphNode for WindowNode {
    fn label(&self) -> &str { "Window" }
    fn input_ports(&self) -> &[&'static str] { &[] }
    fn output_ports(&self) -> &[&'static str] { &["window"] }
    fn run(&mut self, _inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let mut m = HashMap::new();
        m.insert("window", Value::Window(self.0.clone()));
        m
    }
}

/// `ash::Entry::linked()` — loads the Vulkan loader.
struct CreateEntryNode;
impl GraphNode for CreateEntryNode {
    fn label(&self) -> &str { "CreateEntry" }
    fn input_ports(&self) -> &[&'static str] { &[] }
    fn output_ports(&self) -> &[&'static str] { &["entry"] }
    fn run(&mut self, _inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let entry = Entry::linked();
        let mut m = HashMap::new();
        m.insert("entry", Value::Entry(Rc::new(entry)));
        m
    }
}

/// `vkCreateInstance` — needs the entry point and the window (to ask winit
/// which platform surface extensions to enable).
struct CreateInstanceNode;
impl GraphNode for CreateInstanceNode {
    fn label(&self) -> &str { "CreateInstance" }
    fn input_ports(&self) -> &[&'static str] { &["entry", "window"] }
    fn output_ports(&self) -> &[&'static str] { &["instance"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let entry = inputs["entry"].unwrap_entry();
        let window = inputs["window"].unwrap_window();

        let app_name = CStr::from_bytes_with_nul(b"node-graph-triangle\0").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name)
            .application_version(0)
            .engine_name(app_name)
            .engine_version(0)
            .api_version(vk::API_VERSION_1_0);

        let mut extension_names =
            ash_window::enumerate_required_extensions(window.raw_display_handle())
                .unwrap()
                .to_vec();
        extension_names.push(ash::extensions::ext::DebugUtils::name().as_ptr());

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        let instance = unsafe {
            entry.create_instance(&create_info, None).expect("failed to create instance")
        };
        let mut m = HashMap::new();
        m.insert("instance", Value::Instance(Rc::new(instance)));
        m
    }
}

/// `ash_window::create_surface` — the one step that's inherently platform
/// glue rather than a "pure" Vulkan call, but it still goes through a node
/// exactly like everything else, taking `instance` + `window` and producing
/// the `VkSurfaceKHR`.
struct CreateSurfaceNode;
impl GraphNode for CreateSurfaceNode {
    fn label(&self) -> &str { "CreateSurface" }
    fn input_ports(&self) -> &[&'static str] { &["entry", "instance", "window"] }
    fn output_ports(&self) -> &[&'static str] { &["surface"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let entry = inputs["entry"].unwrap_entry();
        let instance = inputs["instance"].unwrap_instance();
        let window = inputs["window"].unwrap_window();
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
            .expect("failed to create surface")
        };
        let mut m = HashMap::new();
        m.insert("surface", Value::SurfaceKHR(surface));
        m
    }
}

/// Creates the `khr::Surface` extension loader (the set of host-side
/// functions for querying a `VkSurfaceKHR`, e.g. capabilities/formats).
struct CreateSurfaceLoaderNode;
impl GraphNode for CreateSurfaceLoaderNode {
    fn label(&self) -> &str { "CreateSurfaceLoader" }
    fn input_ports(&self) -> &[&'static str] { &["entry", "instance"] }
    fn output_ports(&self) -> &[&'static str] { &["surface_loader"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let entry = inputs["entry"].unwrap_entry();
        let instance = inputs["instance"].unwrap_instance();
        let loader = Surface::new(&entry, &instance);
        let mut m = HashMap::new();
        m.insert("surface_loader", Value::SurfaceLoader(Rc::new(loader)));
        m
    }
}

/// `vkEnumeratePhysicalDevices` + picking one, fused with finding its
/// graphics+present queue families (`vkGetPhysicalDeviceQueueFamilyProperties`
/// and `vkGetPhysicalDeviceSurfaceSupportKHR`). In a finer-grained graph
/// these could be three separate nodes; they're fused here because "pick a
/// device" and "does it support what we need" are the same decision in a
/// single-GPU-system tutorial like this one.
struct PickPhysicalDeviceNode;
impl GraphNode for PickPhysicalDeviceNode {
    fn label(&self) -> &str { "PickPhysicalDevice" }
    fn input_ports(&self) -> &[&'static str] { &["instance", "surface", "surface_loader"] }
    fn output_ports(&self) -> &[&'static str] { &["physical_device", "queue_family_indices"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let instance = inputs["instance"].unwrap_instance();
        let surface = inputs["surface"].unwrap_surface();
        let surface_loader = inputs["surface_loader"].unwrap_surface_loader();

        let physical_devices =
            unsafe { instance.enumerate_physical_devices() }.expect("failed to enumerate GPUs");

        let mut chosen: Option<(vk::PhysicalDevice, QueueFamilyIndices)> = None;
        for &pd in &physical_devices {
            let queue_families = unsafe { instance.get_physical_device_queue_family_properties(pd) };
            let mut graphics: Option<u32> = None;
            let mut present: Option<u32> = None;
            for (i, qf) in queue_families.iter().enumerate() {
                let i = i as u32;
                if qf.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    graphics = Some(i);
                }
                let supports_present = unsafe {
                    surface_loader.get_physical_device_surface_support(pd, i, surface)
                }
                .unwrap_or(false);
                if supports_present {
                    present = Some(i);
                }
            }
            if let (Some(g), Some(p)) = (graphics, present) {
                chosen = Some((pd, QueueFamilyIndices { graphics: g, present: p }));
                break;
            }
        }
        let (pd, qfi) = chosen.expect("no suitable physical device found");

        let mut m = HashMap::new();
        m.insert("physical_device", Value::PhysicalDevice(pd));
        m.insert("queue_family_indices", Value::QueueFamilyIndices(Rc::new(qfi)));
        m
    }
}

/// `vkCreateDevice` — creates the logical device and requests the
/// graphics + present queues (which may be the same queue family).
struct CreateDeviceNode;
impl GraphNode for CreateDeviceNode {
    fn label(&self) -> &str { "CreateDevice" }
    fn input_ports(&self) -> &[&'static str] { &["instance", "physical_device", "queue_family_indices"] }
    fn output_ports(&self) -> &[&'static str] { &["device"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let instance = inputs["instance"].unwrap_instance();
        let pd = inputs["physical_device"].unwrap_physical_device();
        let qfi = inputs["queue_family_indices"].unwrap_qfi();

        let mut unique_families = vec![qfi.graphics, qfi.present];
        unique_families.sort_unstable();
        unique_families.dedup();

        let priorities = [1.0_f32];
        let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_families
            .iter()
            .map(|&family| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(family)
                    .queue_priorities(&priorities)
                    .build()
            })
            .collect();

        let device_extensions = [Swapchain::name().as_ptr()];
        let features = vk::PhysicalDeviceFeatures::default();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions)
            .enabled_features(&features);

        let device = unsafe {
            instance.create_device(pd, &create_info, None).expect("failed to create device")
        };
        let mut m = HashMap::new();
        m.insert("device", Value::Device(Rc::new(device)));
        m
    }
}

/// `vkGetDeviceQueue` for the graphics queue.
struct GetGraphicsQueueNode;
impl GraphNode for GetGraphicsQueueNode {
    fn label(&self) -> &str { "GetGraphicsQueue" }
    fn input_ports(&self) -> &[&'static str] { &["device", "queue_family_indices"] }
    fn output_ports(&self) -> &[&'static str] { &["queue"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let qfi = inputs["queue_family_indices"].unwrap_qfi();
        let queue = unsafe { device.get_device_queue(qfi.graphics, 0) };
        let mut m = HashMap::new();
        m.insert("queue", Value::Queue(queue));
        m
    }
}

/// `vkGetDeviceQueue` for the present queue (separate node since on some
/// GPUs this is genuinely a different queue/family than graphics).
struct GetPresentQueueNode;
impl GraphNode for GetPresentQueueNode {
    fn label(&self) -> &str { "GetPresentQueue" }
    fn input_ports(&self) -> &[&'static str] { &["device", "queue_family_indices"] }
    fn output_ports(&self) -> &[&'static str] { &["queue"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let qfi = inputs["queue_family_indices"].unwrap_qfi();
        let queue = unsafe { device.get_device_queue(qfi.present, 0) };
        let mut m = HashMap::new();
        m.insert("queue", Value::Queue(queue));
        m
    }
}

/// Creates the `khr::Swapchain` extension loader.
struct CreateSwapchainLoaderNode;
impl GraphNode for CreateSwapchainLoaderNode {
    fn label(&self) -> &str { "CreateSwapchainLoader" }
    fn input_ports(&self) -> &[&'static str] { &["instance", "device"] }
    fn output_ports(&self) -> &[&'static str] { &["swapchain_loader"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let instance = inputs["instance"].unwrap_instance();
        let device = inputs["device"].unwrap_device();
        let loader = Swapchain::new(&*instance, &*device);
        let mut m = HashMap::new();
        m.insert("swapchain_loader", Value::SwapchainLoader(Rc::new(loader)));
        m
    }
}

/// `vkGetPhysicalDeviceSurfaceCapabilitiesKHR`.
struct QuerySurfaceCapsNode;
impl GraphNode for QuerySurfaceCapsNode {
    fn label(&self) -> &str { "QuerySurfaceCaps" }
    fn input_ports(&self) -> &[&'static str] { &["surface_loader", "physical_device", "surface"] }
    fn output_ports(&self) -> &[&'static str] { &["caps"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let surface_loader = inputs["surface_loader"].unwrap_surface_loader();
        let pd = inputs["physical_device"].unwrap_physical_device();
        let surface = inputs["surface"].unwrap_surface();
        let caps = unsafe {
            surface_loader.get_physical_device_surface_capabilities(pd, surface)
        }
        .expect("failed to query surface capabilities");
        let mut m = HashMap::new();
        m.insert("caps", Value::SurfaceCaps(caps));
        m
    }
}

/// `vkGetPhysicalDeviceSurfaceFormatsKHR` + picking a preferred format
/// (BGRA8 sRGB if available, else whatever's first).
struct ChooseSurfaceFormatNode;
impl GraphNode for ChooseSurfaceFormatNode {
    fn label(&self) -> &str { "ChooseSurfaceFormat" }
    fn input_ports(&self) -> &[&'static str] { &["surface_loader", "physical_device", "surface"] }
    fn output_ports(&self) -> &[&'static str] { &["surface_format"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let surface_loader = inputs["surface_loader"].unwrap_surface_loader();
        let pd = inputs["physical_device"].unwrap_physical_device();
        let surface = inputs["surface"].unwrap_surface();
        let formats = unsafe {
            surface_loader.get_physical_device_surface_formats(pd, surface)
        }
        .expect("failed to query surface formats");
        let chosen = formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .copied()
            .unwrap_or(formats[0]);
        let mut m = HashMap::new();
        m.insert("surface_format", Value::SurfaceFormat(chosen));
        m
    }
}

/// `vkGetPhysicalDeviceSurfacePresentModesKHR` + picking a preferred mode
/// (MAILBOX if available, else the universally-supported FIFO).
struct ChoosePresentModeNode;
impl GraphNode for ChoosePresentModeNode {
    fn label(&self) -> &str { "ChoosePresentMode" }
    fn input_ports(&self) -> &[&'static str] { &["surface_loader", "physical_device", "surface"] }
    fn output_ports(&self) -> &[&'static str] { &["present_mode"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let surface_loader = inputs["surface_loader"].unwrap_surface_loader();
        let pd = inputs["physical_device"].unwrap_physical_device();
        let surface = inputs["surface"].unwrap_surface();
        let modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(pd, surface)
        }
        .expect("failed to query present modes");
        let chosen = if modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else {
            vk::PresentModeKHR::FIFO
        };
        let mut m = HashMap::new();
        m.insert("present_mode", Value::PresentMode(chosen));
        m
    }
}

/// Picks the swap extent from surface capabilities + the current window
/// size (handling the `u32::MAX` "must match window" sentinel some
/// platforms report).
struct ChooseExtentNode;
impl GraphNode for ChooseExtentNode {
    fn label(&self) -> &str { "ChooseExtent" }
    fn input_ports(&self) -> &[&'static str] { &["caps", "window"] }
    fn output_ports(&self) -> &[&'static str] { &["extent"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let caps = inputs["caps"].unwrap_surface_caps();
        let window = inputs["window"].unwrap_window();
        let extent = if caps.current_extent.width != u32::MAX {
            caps.current_extent
        } else {
            let size = window.inner_size();
            vk::Extent2D {
                width: size.width.clamp(caps.min_image_extent.width, caps.max_image_extent.width),
                height: size.height.clamp(caps.min_image_extent.height, caps.max_image_extent.height),
            }
        };
        let mut m = HashMap::new();
        m.insert("extent", Value::Extent(extent));
        m
    }
}

/// `vkCreateSwapchainKHR` + `vkGetSwapchainImagesKHR`, fused into one node
/// since the resulting images are intrinsically part of "the swapchain"
/// as a unit (mirroring how the rest of Vulkan treats them).
struct CreateSwapchainNode;
impl GraphNode for CreateSwapchainNode {
    fn label(&self) -> &str { "CreateSwapchain" }
    fn input_ports(&self) -> &[&'static str] {
        &["swapchain_loader", "surface", "caps", "surface_format", "present_mode", "extent", "queue_family_indices"]
    }
    fn output_ports(&self) -> &[&'static str] { &["swapchain_bundle"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let loader = inputs["swapchain_loader"].unwrap_swapchain_loader();
        let surface = inputs["surface"].unwrap_surface();
        let caps = inputs["caps"].unwrap_surface_caps();
        let surface_format = inputs["surface_format"].unwrap_surface_format();
        let present_mode = inputs["present_mode"].unwrap_present_mode();
        let extent = inputs["extent"].unwrap_extent();
        let qfi = inputs["queue_family_indices"].unwrap_qfi();

        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count > 0 && image_count > caps.max_image_count {
            image_count = caps.max_image_count;
        }

        let families = [qfi.graphics, qfi.present];
        let (sharing_mode, family_slice): (vk::SharingMode, &[u32]) = if qfi.graphics != qfi.present {
            (vk::SharingMode::CONCURRENT, &families)
        } else {
            (vk::SharingMode::EXCLUSIVE, &[])
        };

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(sharing_mode)
            .queue_family_indices(family_slice)
            .pre_transform(caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe { loader.create_swapchain(&create_info, None) }
            .expect("failed to create swapchain");
        let images = unsafe { loader.get_swapchain_images(swapchain) }
            .expect("failed to get swapchain images");

        let bundle = SwapchainBundle { swapchain, images, format: surface_format.format, extent };
        let mut m = HashMap::new();
        m.insert("swapchain_bundle", Value::SwapchainBundle(Rc::new(bundle)));
        m
    }
}

/// `vkCreateImageView`, once per swapchain image.
struct CreateImageViewsNode;
impl GraphNode for CreateImageViewsNode {
    fn label(&self) -> &str { "CreateImageViews" }
    fn input_ports(&self) -> &[&'static str] { &["device", "swapchain_bundle"] }
    fn output_ports(&self) -> &[&'static str] { &["image_views"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let bundle = inputs["swapchain_bundle"].unwrap_swapchain_bundle();

        let views: Vec<vk::ImageView> = bundle
            .images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(bundle.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe { device.create_image_view(&create_info, None) }
                    .expect("failed to create image view")
            })
            .collect();

        let mut m = HashMap::new();
        m.insert("image_views", Value::ImageViews(Rc::new(views)));
        m
    }
}

/// `vkCreateRenderPass` — one color attachment, one subpass, with the
/// usual "wait for the swapchain image to be available before writing"
/// subpass dependency.
struct CreateRenderPassNode;
impl GraphNode for CreateRenderPassNode {
    fn label(&self) -> &str { "CreateRenderPass" }
    fn input_ports(&self) -> &[&'static str] { &["device", "swapchain_bundle"] }
    fn output_ports(&self) -> &[&'static str] { &["render_pass"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let bundle = inputs["swapchain_bundle"].unwrap_swapchain_bundle();

        let color_attachment = vk::AttachmentDescription::builder()
            .format(bundle.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        let color_refs = [color_attachment_ref];

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_refs)
            .build();

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        let attachments = [color_attachment];
        let subpasses = [subpass];
        let dependencies = [dependency];
        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let render_pass = unsafe { device.create_render_pass(&create_info, None) }
            .expect("failed to create render pass");
        let mut m = HashMap::new();
        m.insert("render_pass", Value::RenderPass(render_pass));
        m
    }
}

/// `vkCreateShaderModule` for the vertex shader. Takes a `Bool` input port
/// only nominally — it has no real dependency on anything upstream — but
/// every node still goes through the same `device`-in, `shader_module`-out
/// shape for consistency.
struct CreateVertexShaderModuleNode;
impl GraphNode for CreateVertexShaderModuleNode {
    fn label(&self) -> &str { "CreateVertexShaderModule" }
    fn input_ports(&self) -> &[&'static str] { &["device"] }
    fn output_ports(&self) -> &[&'static str] { &["shader_module"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let create_info = vk::ShaderModuleCreateInfo::builder().code(&VERT_SPV);
        let module = unsafe { device.create_shader_module(&create_info, None) }
            .expect("failed to create vertex shader module");
        let mut m = HashMap::new();
        m.insert("shader_module", Value::ShaderModule(module));
        m
    }
}

/// `vkCreateShaderModule` for the fragment shader.
struct CreateFragmentShaderModuleNode;
impl GraphNode for CreateFragmentShaderModuleNode {
    fn label(&self) -> &str { "CreateFragmentShaderModule" }
    fn input_ports(&self) -> &[&'static str] { &["device"] }
    fn output_ports(&self) -> &[&'static str] { &["shader_module"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let create_info = vk::ShaderModuleCreateInfo::builder().code(&FRAG_SPV);
        let module = unsafe { device.create_shader_module(&create_info, None) }
            .expect("failed to create fragment shader module");
        let mut m = HashMap::new();
        m.insert("shader_module", Value::ShaderModule(module));
        m
    }
}

/// `vkCreatePipelineLayout` — empty layout, since this triangle has no
/// descriptor sets or push constants.
struct CreatePipelineLayoutNode;
impl GraphNode for CreatePipelineLayoutNode {
    fn label(&self) -> &str { "CreatePipelineLayout" }
    fn input_ports(&self) -> &[&'static str] { &["device"] }
    fn output_ports(&self) -> &[&'static str] { &["pipeline_layout"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let create_info = vk::PipelineLayoutCreateInfo::builder();
        let layout = unsafe { device.create_pipeline_layout(&create_info, None) }
            .expect("failed to create pipeline layout");
        let mut m = HashMap::new();
        m.insert("pipeline_layout", Value::PipelineLayout(layout));
        m
    }
}

/// `vkCreateGraphicsPipelines` — the big one. Fixed-function state
/// (vertex input is empty since positions/colors are hardcoded in the
/// vertex shader; viewport+scissor are dynamic so the pipeline doesn't
/// need to be rebuilt on resize) plus the two shader stages.
struct CreateGraphicsPipelineNode;
impl GraphNode for CreateGraphicsPipelineNode {
    fn label(&self) -> &str { "CreateGraphicsPipeline" }
    fn input_ports(&self) -> &[&'static str] {
        &["device", "vertex_shader_module", "fragment_shader_module", "pipeline_layout", "render_pass"]
    }
    fn output_ports(&self) -> &[&'static str] { &["pipeline"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let vert_module = inputs["vertex_shader_module"].unwrap_shader_module();
        let frag_module = inputs["fragment_shader_module"].unwrap_shader_module();
        let layout = inputs["pipeline_layout"].unwrap_pipeline_layout();
        let render_pass = inputs["render_pass"].unwrap_render_pass();

        let entry_point = CStr::from_bytes_with_nul(b"main\0").unwrap();
        let stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_module)
                .name(entry_point)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_module)
                .name(entry_point)
                .build(),
        ];

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::builder();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)
            .build();
        let attachments = [color_blend_attachment];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&attachments);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);

        let create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state)
            .layout(layout)
            .render_pass(render_pass)
            .subpass(0)
            .build();

        let pipelines = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
        }
        .expect("failed to create graphics pipeline");

        let mut m = HashMap::new();
        m.insert("pipeline", Value::Pipeline(pipelines[0]));
        m
    }
}

/// `vkCreateFramebuffer`, once per swapchain image view.
struct CreateFramebuffersNode;
impl GraphNode for CreateFramebuffersNode {
    fn label(&self) -> &str { "CreateFramebuffers" }
    fn input_ports(&self) -> &[&'static str] { &["device", "render_pass", "image_views", "swapchain_bundle"] }
    fn output_ports(&self) -> &[&'static str] { &["framebuffers"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let render_pass = inputs["render_pass"].unwrap_render_pass();
        let image_views = inputs["image_views"].unwrap_image_views();
        let bundle = inputs["swapchain_bundle"].unwrap_swapchain_bundle();

        let framebuffers: Vec<vk::Framebuffer> = image_views
            .iter()
            .map(|&view| {
                let attachments = [view];
                let create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(bundle.extent.width)
                    .height(bundle.extent.height)
                    .layers(1);
                unsafe { device.create_framebuffer(&create_info, None) }
                    .expect("failed to create framebuffer")
            })
            .collect();

        let mut m = HashMap::new();
        m.insert("framebuffers", Value::Framebuffers(Rc::new(framebuffers)));
        m
    }
}

/// `vkCreateCommandPool`, bound to the graphics queue family.
struct CreateCommandPoolNode;
impl GraphNode for CreateCommandPoolNode {
    fn label(&self) -> &str { "CreateCommandPool" }
    fn input_ports(&self) -> &[&'static str] { &["device", "queue_family_indices"] }
    fn output_ports(&self) -> &[&'static str] { &["command_pool"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let qfi = inputs["queue_family_indices"].unwrap_qfi();
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(qfi.graphics);
        let pool = unsafe { device.create_command_pool(&create_info, None) }
            .expect("failed to create command pool");
        let mut m = HashMap::new();
        m.insert("command_pool", Value::CommandPool(pool));
        m
    }
}

/// `vkAllocateCommandBuffers` — one primary command buffer per
/// frame-in-flight slot.
struct AllocateCommandBuffersNode;
impl GraphNode for AllocateCommandBuffersNode {
    fn label(&self) -> &str { "AllocateCommandBuffers" }
    fn input_ports(&self) -> &[&'static str] { &["device", "command_pool"] }
    fn output_ports(&self) -> &[&'static str] { &["command_buffers"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let pool = inputs["command_pool"].unwrap_command_pool();
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
        let buffers = unsafe { device.allocate_command_buffers(&alloc_info) }
            .expect("failed to allocate command buffers");
        let mut m = HashMap::new();
        m.insert("command_buffers", Value::CommandBuffers(Rc::new(buffers)));
        m
    }
}

/// `vkCreateSemaphore` x2 + `vkCreateFence` x1, per frame-in-flight slot —
/// the synchronization primitives that pace the render loop.
struct CreateSyncObjectsNode;
impl GraphNode for CreateSyncObjectsNode {
    fn label(&self) -> &str { "CreateSyncObjects" }
    fn input_ports(&self) -> &[&'static str] { &["device"] }
    fn output_ports(&self) -> &[&'static str] { &["sync_bundle"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();

        let sem_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available.push(unsafe { device.create_semaphore(&sem_info, None) }.unwrap());
            render_finished.push(unsafe { device.create_semaphore(&sem_info, None) }.unwrap());
            in_flight.push(unsafe { device.create_fence(&fence_info, None) }.unwrap());
        }

        let bundle = SyncBundle { image_available, render_finished, in_flight };
        let mut m = HashMap::new();
        m.insert("sync_bundle", Value::SyncBundle(Rc::new(bundle)));
        m
    }
}

// ============================================================================
// PART 3: PER-FRAME NODES
// ============================================================================
// These run every redrawn frame rather than once at startup. They're driven
// by a small "frame graph" rebuilt and executed on each `RedrawRequested`
// event in `main`'s event loop (see PART 4). Per-frame *state* that isn't a
// Vulkan handle — which frame-in-flight slot this is — comes in via
// `Graph::feed` rather than from another node, since it originates from the
// event loop's own counter, not from a Vulkan call.

/// `vkWaitForFences` — waits for the GPU to finish with the command buffer
/// (and associated resources) belonging to this frame-in-flight slot before
/// we're allowed to record into it again.
struct WaitForFenceNode;
impl GraphNode for WaitForFenceNode {
    fn label(&self) -> &str { "WaitForFence" }
    fn input_ports(&self) -> &[&'static str] { &["device", "sync_bundle", "frame_index"] }
    fn output_ports(&self) -> &[&'static str] { &["done"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let sync = inputs["sync_bundle"].unwrap_sync_bundle();
        let frame = inputs["frame_index"].unwrap_usize();
        let fence = sync.in_flight[frame];
        unsafe {
            device.wait_for_fences(&[fence], true, u64::MAX).expect("wait_for_fences failed");
        }
        let mut m = HashMap::new();
        m.insert("done", Value::Unit);
        m
    }
}

/// `vkAcquireNextImageKHR` — gets the index of the swapchain image to
/// render into this frame, signaling `image_available[frame]` once it's
/// actually ready to be written.
struct AcquireImageNode;
impl GraphNode for AcquireImageNode {
    fn label(&self) -> &str { "AcquireImage" }
    fn input_ports(&self) -> &[&'static str] { &["swapchain_loader", "swapchain_bundle", "sync_bundle", "frame_index", "wait_for_fence"] }
    fn output_ports(&self) -> &[&'static str] { &["image_index"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let loader = inputs["swapchain_loader"].unwrap_swapchain_loader();
        let bundle = inputs["swapchain_bundle"].unwrap_swapchain_bundle();
        let sync = inputs["sync_bundle"].unwrap_sync_bundle();
        let frame = inputs["frame_index"].unwrap_usize();
        let semaphore = sync.image_available[frame];

        let (image_index, _suboptimal) = unsafe {
            loader.acquire_next_image(bundle.swapchain, u64::MAX, semaphore, vk::Fence::null())
        }
        .expect("failed to acquire swapchain image");

        let mut m = HashMap::new();
        m.insert("image_index", Value::ImageIndex(image_index));
        m
    }
}

/// `vkResetFences` — re-arms the in-flight fence for this frame slot now
/// that we know we're about to submit new work for it.
struct ResetFenceNode;
impl GraphNode for ResetFenceNode {
    fn label(&self) -> &str { "ResetFence" }
    fn input_ports(&self) -> &[&'static str] { &["device", "sync_bundle", "frame_index", "image_index"] }
    fn output_ports(&self) -> &[&'static str] { &["done"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let sync = inputs["sync_bundle"].unwrap_sync_bundle();
        let frame = inputs["frame_index"].unwrap_usize();
        let fence = sync.in_flight[frame];
        unsafe { device.reset_fences(&[fence]).expect("reset_fences failed") };
        let mut m = HashMap::new();
        m.insert("done", Value::Unit);
        m
    }
}

/// `vkResetCommandBuffer` then `vkBeginCommandBuffer`.
struct BeginCommandBufferNode;
impl GraphNode for BeginCommandBufferNode {
    fn label(&self) -> &str { "BeginCommandBuffer" }
    fn input_ports(&self) -> &[&'static str] { &["device", "command_buffers", "frame_index", "reset_fence"] }
    fn output_ports(&self) -> &[&'static str] { &["command_buffer"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let buffers = inputs["command_buffers"].unwrap_command_buffers();
        let frame = inputs["frame_index"].unwrap_usize();
        let cmd = buffers[frame];

        unsafe {
            device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
                .expect("reset_command_buffer failed");
            let begin_info = vk::CommandBufferBeginInfo::builder();
            device.begin_command_buffer(cmd, &begin_info).expect("begin_command_buffer failed");
        }

        let mut m = HashMap::new();
        m.insert("command_buffer", Value::CommandBuffer(cmd));
        m
    }
}

/// `vkCmdBeginRenderPass` — clears to a dark blue-grey and starts the
/// single subpass.
struct CmdBeginRenderPassNode;
impl GraphNode for CmdBeginRenderPassNode {
    fn label(&self) -> &str { "CmdBeginRenderPass" }
    fn input_ports(&self) -> &[&'static str] {
        &["device", "command_buffer", "render_pass", "framebuffers", "image_index", "swapchain_bundle"]
    }
    fn output_ports(&self) -> &[&'static str] { &["done"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let cmd = inputs["command_buffer"].unwrap_command_buffer();
        let render_pass = inputs["render_pass"].unwrap_render_pass();
        let framebuffers = inputs["framebuffers"].unwrap_framebuffers();
        let image_index = inputs["image_index"].unwrap_image_index();
        let bundle = inputs["swapchain_bundle"].unwrap_swapchain_bundle();

        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.01, 0.01, 0.03, 1.0] },
        };
        let clear_values = [clear_value];
        let render_area = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: bundle.extent };
        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(framebuffers[image_index as usize])
            .render_area(render_area)
            .clear_values(&clear_values);

        unsafe { device.cmd_begin_render_pass(cmd, &begin_info, vk::SubpassContents::INLINE) };
        let mut m = HashMap::new();
        m.insert("done", Value::Unit);
        m
    }
}

/// `vkCmdBindPipeline`.
struct CmdBindPipelineNode;
impl GraphNode for CmdBindPipelineNode {
    fn label(&self) -> &str { "CmdBindPipeline" }
    fn input_ports(&self) -> &[&'static str] { &["device", "command_buffer", "pipeline", "begin_render_pass"] }
    fn output_ports(&self) -> &[&'static str] { &["done"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let cmd = inputs["command_buffer"].unwrap_command_buffer();
        let pipeline = inputs["pipeline"].unwrap_pipeline();
        unsafe { device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline) };
        let mut m = HashMap::new();
        m.insert("done", Value::Unit);
        m
    }
}

/// `vkCmdSetViewport` — dynamic viewport, sized to the current swapchain
/// extent (this is what lets the pipeline skip rebuilding on resize).
struct CmdSetViewportNode;
impl GraphNode for CmdSetViewportNode {
    fn label(&self) -> &str { "CmdSetViewport" }
    fn input_ports(&self) -> &[&'static str] { &["device", "command_buffer", "swapchain_bundle", "bind_pipeline"] }
    fn output_ports(&self) -> &[&'static str] { &["done"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let cmd = inputs["command_buffer"].unwrap_command_buffer();
        let bundle = inputs["swapchain_bundle"].unwrap_swapchain_bundle();
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: bundle.extent.width as f32,
            height: bundle.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        unsafe { device.cmd_set_viewport(cmd, 0, &[viewport]) };
        let mut m = HashMap::new();
        m.insert("done", Value::Unit);
        m
    }
}

/// `vkCmdSetScissor` — dynamic scissor, matching the full extent.
struct CmdSetScissorNode;
impl GraphNode for CmdSetScissorNode {
    fn label(&self) -> &str { "CmdSetScissor" }
    fn input_ports(&self) -> &[&'static str] { &["device", "command_buffer", "swapchain_bundle", "set_viewport"] }
    fn output_ports(&self) -> &[&'static str] { &["done"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let cmd = inputs["command_buffer"].unwrap_command_buffer();
        let bundle = inputs["swapchain_bundle"].unwrap_swapchain_bundle();
        let scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: bundle.extent };
        unsafe { device.cmd_set_scissor(cmd, 0, &[scissor]) };
        let mut m = HashMap::new();
        m.insert("done", Value::Unit);
        m
    }
}

/// `vkCmdDraw` — 3 vertices, 1 instance. The positions/colors are
/// hardcoded in the vertex shader, so there's no vertex buffer to bind.
struct CmdDrawNode;
impl GraphNode for CmdDrawNode {
    fn label(&self) -> &str { "CmdDraw" }
    fn input_ports(&self) -> &[&'static str] { &["device", "command_buffer", "set_scissor"] }
    fn output_ports(&self) -> &[&'static str] { &["done"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let cmd = inputs["command_buffer"].unwrap_command_buffer();
        unsafe { device.cmd_draw(cmd, 3, 1, 0, 0) };
        let mut m = HashMap::new();
        m.insert("done", Value::Unit);
        m
    }
}

/// `vkCmdEndRenderPass`.
struct CmdEndRenderPassNode;
impl GraphNode for CmdEndRenderPassNode {
    fn label(&self) -> &str { "CmdEndRenderPass" }
    fn input_ports(&self) -> &[&'static str] { &["device", "command_buffer", "draw"] }
    fn output_ports(&self) -> &[&'static str] { &["done"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let cmd = inputs["command_buffer"].unwrap_command_buffer();
        unsafe { device.cmd_end_render_pass(cmd) };
        let mut m = HashMap::new();
        m.insert("done", Value::Unit);
        m
    }
}

/// `vkEndCommandBuffer`.
struct EndCommandBufferNode;
impl GraphNode for EndCommandBufferNode {
    fn label(&self) -> &str { "EndCommandBuffer" }
    fn input_ports(&self) -> &[&'static str] { &["device", "command_buffer", "end_render_pass"] }
    fn output_ports(&self) -> &[&'static str] { &["done"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let cmd = inputs["command_buffer"].unwrap_command_buffer();
        unsafe { device.end_command_buffer(cmd).expect("end_command_buffer failed") };
        let mut m = HashMap::new();
        m.insert("done", Value::Unit);
        m
    }
}

/// `vkQueueSubmit` — waits on `image_available`, signals `render_finished`
/// and the in-flight fence once the GPU is done.
struct QueueSubmitNode;
impl GraphNode for QueueSubmitNode {
    fn label(&self) -> &str { "QueueSubmit" }
    fn input_ports(&self) -> &[&'static str] {
        &["device", "queue", "command_buffer", "sync_bundle", "frame_index", "end_command_buffer"]
    }
    fn output_ports(&self) -> &[&'static str] { &["done"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let device = inputs["device"].unwrap_device();
        let queue = inputs["queue"].unwrap_queue();
        let cmd = inputs["command_buffer"].unwrap_command_buffer();
        let sync = inputs["sync_bundle"].unwrap_sync_bundle();
        let frame = inputs["frame_index"].unwrap_usize();

        let wait_semaphores = [sync.image_available[frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [sync.render_finished[frame]];
        let command_buffers = [cmd];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build();

        unsafe {
            device.queue_submit(queue, &[submit_info], sync.in_flight[frame])
                .expect("queue_submit failed")
        };

        let mut m = HashMap::new();
        m.insert("done", Value::Unit);
        m
    }
}

/// `vkQueuePresentKHR` — waits on `render_finished`, presents the image
/// that was just rendered into.
struct QueuePresentNode;
impl GraphNode for QueuePresentNode {
    fn label(&self) -> &str { "QueuePresent" }
    fn input_ports(&self) -> &[&'static str] {
        &["swapchain_loader", "queue", "swapchain_bundle", "sync_bundle", "frame_index", "image_index", "queue_submit"]
    }
    fn output_ports(&self) -> &[&'static str] { &["done"] }
    fn run(&mut self, inputs: &HashMap<&'static str, Value>) -> HashMap<&'static str, Value> {
        let loader = inputs["swapchain_loader"].unwrap_swapchain_loader();
        let queue = inputs["queue"].unwrap_queue();
        let bundle = inputs["swapchain_bundle"].unwrap_swapchain_bundle();
        let sync = inputs["sync_bundle"].unwrap_sync_bundle();
        let frame = inputs["frame_index"].unwrap_usize();
        let image_index = inputs["image_index"].unwrap_image_index();

        let wait_semaphores = [sync.render_finished[frame]];
        let swapchains = [bundle.swapchain];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            loader.queue_present(queue, &present_info).expect("queue_present failed");
        }

        let mut m = HashMap::new();
        m.insert("done", Value::Unit);
        m
    }
}

// ============================================================================
// PART 4: MAIN — wires the nodes above into a setup graph (run once) and a
// per-frame graph (rebuilt and run on every redraw).
// ============================================================================
// Two things in this program are deliberately *not* nodes:
//   - winit's `Window`/`EventLoop` construction, since a window isn't the
//     output of a Vulkan call; the `WindowNode` above wraps the already-
//     created window as a graph *source* so everything downstream still
//     receives it through a port like everything else.
//   - The event loop's `run()` call itself, which is the thing that decides
//     *when* to execute the frame graph (on each redraw) — analogous to how
//     a graph engine's own scheduler invocation isn't a node within the
//     graph it's running.
// Every actual Vulkan API call lives inside a `GraphNode::run` body.

/// Handles pulled out of the one-shot setup graph's results, kept around so
/// the per-frame graph (rebuilt every redraw) can be wired against them
/// without re-running setup.
struct RenderState {
    device: Rc<Device>,
    swapchain_loader: Rc<Swapchain>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain_bundle: Rc<SwapchainBundle>,
    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
    framebuffers: Rc<Vec<vk::Framebuffer>>,
    command_buffers: Rc<Vec<vk::CommandBuffer>>,
    sync_bundle: Rc<SyncBundle>,
    current_frame: usize,
}

/// Builds the setup graph: every node from PART 2, wired exactly along the
/// data dependencies a normal imperative ash program would express through
/// local variables and call order. Runs it once and returns a `RenderState`.
fn run_setup_graph(window: Rc<Window>) -> RenderState {
    let mut g = Graph::new();

    let window_node = g.add(WindowNode(window));
    let entry = g.add(CreateEntryNode);
    let instance = g.add(CreateInstanceNode);
    g.connect(entry, "entry", instance, "entry");
    g.connect(window_node, "window", instance, "window");

    let surface = g.add(CreateSurfaceNode);
    g.connect(entry, "entry", surface, "entry");
    g.connect(instance, "instance", surface, "instance");
    g.connect(window_node, "window", surface, "window");

    let surface_loader = g.add(CreateSurfaceLoaderNode);
    g.connect(entry, "entry", surface_loader, "entry");
    g.connect(instance, "instance", surface_loader, "instance");

    let physical_device = g.add(PickPhysicalDeviceNode);
    g.connect(instance, "instance", physical_device, "instance");
    g.connect(surface, "surface", physical_device, "surface");
    g.connect(surface_loader, "surface_loader", physical_device, "surface_loader");

    let device = g.add(CreateDeviceNode);
    g.connect(instance, "instance", device, "instance");
    g.connect(physical_device, "physical_device", device, "physical_device");
    g.connect(physical_device, "queue_family_indices", device, "queue_family_indices");

    let graphics_queue = g.add(GetGraphicsQueueNode);
    g.connect(device, "device", graphics_queue, "device");
    g.connect(physical_device, "queue_family_indices", graphics_queue, "queue_family_indices");

    let present_queue = g.add(GetPresentQueueNode);
    g.connect(device, "device", present_queue, "device");
    g.connect(physical_device, "queue_family_indices", present_queue, "queue_family_indices");

    let swapchain_loader = g.add(CreateSwapchainLoaderNode);
    g.connect(instance, "instance", swapchain_loader, "instance");
    g.connect(device, "device", swapchain_loader, "device");

    let surface_caps = g.add(QuerySurfaceCapsNode);
    g.connect(surface_loader, "surface_loader", surface_caps, "surface_loader");
    g.connect(physical_device, "physical_device", surface_caps, "physical_device");
    g.connect(surface, "surface", surface_caps, "surface");

    let surface_format = g.add(ChooseSurfaceFormatNode);
    g.connect(surface_loader, "surface_loader", surface_format, "surface_loader");
    g.connect(physical_device, "physical_device", surface_format, "physical_device");
    g.connect(surface, "surface", surface_format, "surface");

    let present_mode = g.add(ChoosePresentModeNode);
    g.connect(surface_loader, "surface_loader", present_mode, "surface_loader");
    g.connect(physical_device, "physical_device", present_mode, "physical_device");
    g.connect(surface, "surface", present_mode, "surface");

    let extent = g.add(ChooseExtentNode);
    g.connect(surface_caps, "caps", extent, "caps");
    g.connect(window_node, "window", extent, "window");

    let swapchain = g.add(CreateSwapchainNode);
    g.connect(swapchain_loader, "swapchain_loader", swapchain, "swapchain_loader");
    g.connect(surface, "surface", swapchain, "surface");
    g.connect(surface_caps, "caps", swapchain, "caps");
    g.connect(surface_format, "surface_format", swapchain, "surface_format");
    g.connect(present_mode, "present_mode", swapchain, "present_mode");
    g.connect(extent, "extent", swapchain, "extent");
    g.connect(physical_device, "queue_family_indices", swapchain, "queue_family_indices");

    let image_views = g.add(CreateImageViewsNode);
    g.connect(device, "device", image_views, "device");
    g.connect(swapchain, "swapchain_bundle", image_views, "swapchain_bundle");

    let render_pass = g.add(CreateRenderPassNode);
    g.connect(device, "device", render_pass, "device");
    g.connect(swapchain, "swapchain_bundle", render_pass, "swapchain_bundle");

    let vert_module = g.add(CreateVertexShaderModuleNode);
    g.connect(device, "device", vert_module, "device");

    let frag_module = g.add(CreateFragmentShaderModuleNode);
    g.connect(device, "device", frag_module, "device");

    let pipeline_layout = g.add(CreatePipelineLayoutNode);
    g.connect(device, "device", pipeline_layout, "device");

    let pipeline = g.add(CreateGraphicsPipelineNode);
    g.connect(device, "device", pipeline, "device");
    g.connect(vert_module, "shader_module", pipeline, "vertex_shader_module");
    g.connect(frag_module, "shader_module", pipeline, "fragment_shader_module");
    g.connect(pipeline_layout, "pipeline_layout", pipeline, "pipeline_layout");
    g.connect(render_pass, "render_pass", pipeline, "render_pass");

    let framebuffers = g.add(CreateFramebuffersNode);
    g.connect(device, "device", framebuffers, "device");
    g.connect(render_pass, "render_pass", framebuffers, "render_pass");
    g.connect(image_views, "image_views", framebuffers, "image_views");
    g.connect(swapchain, "swapchain_bundle", framebuffers, "swapchain_bundle");

    let command_pool = g.add(CreateCommandPoolNode);
    g.connect(device, "device", command_pool, "device");
    g.connect(physical_device, "queue_family_indices", command_pool, "queue_family_indices");

    let command_buffers = g.add(AllocateCommandBuffersNode);
    g.connect(device, "device", command_buffers, "device");
    g.connect(command_pool, "command_pool", command_buffers, "command_pool");

    let sync_objects = g.add(CreateSyncObjectsNode);
    g.connect(device, "device", sync_objects, "device");

    let results = g.execute();

    RenderState {
        device: output_of(&results, device, "device").unwrap_device(),
        swapchain_loader: output_of(&results, swapchain_loader, "swapchain_loader").unwrap_swapchain_loader(),
        graphics_queue: output_of(&results, graphics_queue, "queue").unwrap_queue(),
        present_queue: output_of(&results, present_queue, "queue").unwrap_queue(),
        swapchain_bundle: output_of(&results, swapchain, "swapchain_bundle").unwrap_swapchain_bundle(),
        render_pass: output_of(&results, render_pass, "render_pass").unwrap_render_pass(),
        pipeline: output_of(&results, pipeline, "pipeline").unwrap_pipeline(),
        framebuffers: output_of(&results, framebuffers, "framebuffers").unwrap_framebuffers(),
        command_buffers: output_of(&results, command_buffers, "command_buffers").unwrap_command_buffers(),
        sync_bundle: output_of(&results, sync_objects, "sync_bundle").unwrap_sync_bundle(),
        current_frame: 0,
    }
}

/// Builds the per-frame graph: every node from PART 3, wired along the
/// dependency chain wait-fence -> acquire -> reset-fence -> begin -> ... ->
/// present. Runs it once for the current frame-in-flight slot.
fn run_frame_graph(state: &mut RenderState) {
    let mut g = Graph::new();

    let wait_fence = g.add(WaitForFenceNode);
    g.feed(Value::Device(state.device.clone()), wait_fence, "device");
    g.feed(Value::SyncBundle(state.sync_bundle.clone()), wait_fence, "sync_bundle");
    g.feed(Value::Usize(state.current_frame), wait_fence, "frame_index");

    let acquire = g.add(AcquireImageNode);
    g.feed(Value::SwapchainLoader(state.swapchain_loader.clone()), acquire, "swapchain_loader");
    g.feed(Value::SwapchainBundle(state.swapchain_bundle.clone()), acquire, "swapchain_bundle");
    g.feed(Value::SyncBundle(state.sync_bundle.clone()), acquire, "sync_bundle");
    g.feed(Value::Usize(state.current_frame), acquire, "frame_index");
    g.connect(wait_fence, "done", acquire, "wait_for_fence");

    let reset_fence = g.add(ResetFenceNode);
    g.feed(Value::Device(state.device.clone()), reset_fence, "device");
    g.feed(Value::SyncBundle(state.sync_bundle.clone()), reset_fence, "sync_bundle");
    g.feed(Value::Usize(state.current_frame), reset_fence, "frame_index");
    g.connect(acquire, "image_index", reset_fence, "image_index");

    let begin_cmd = g.add(BeginCommandBufferNode);
    g.feed(Value::Device(state.device.clone()), begin_cmd, "device");
    g.feed(Value::CommandBuffers(state.command_buffers.clone()), begin_cmd, "command_buffers");
    g.feed(Value::Usize(state.current_frame), begin_cmd, "frame_index");
    g.connect(reset_fence, "done", begin_cmd, "reset_fence");

    let begin_rp = g.add(CmdBeginRenderPassNode);
    g.feed(Value::Device(state.device.clone()), begin_rp, "device");
    g.feed(Value::RenderPass(state.render_pass), begin_rp, "render_pass");
    g.feed(Value::Framebuffers(state.framebuffers.clone()), begin_rp, "framebuffers");
    g.feed(Value::SwapchainBundle(state.swapchain_bundle.clone()), begin_rp, "swapchain_bundle");
    g.connect(begin_cmd, "command_buffer", begin_rp, "command_buffer");
    g.connect(acquire, "image_index", begin_rp, "image_index");

    let bind_pipeline = g.add(CmdBindPipelineNode);
    g.feed(Value::Device(state.device.clone()), bind_pipeline, "device");
    g.feed(Value::Pipeline(state.pipeline), bind_pipeline, "pipeline");
    g.connect(begin_cmd, "command_buffer", bind_pipeline, "command_buffer");
    g.connect(begin_rp, "done", bind_pipeline, "begin_render_pass");

    let set_viewport = g.add(CmdSetViewportNode);
    g.feed(Value::Device(state.device.clone()), set_viewport, "device");
    g.feed(Value::SwapchainBundle(state.swapchain_bundle.clone()), set_viewport, "swapchain_bundle");
    g.connect(begin_cmd, "command_buffer", set_viewport, "command_buffer");
    g.connect(bind_pipeline, "done", set_viewport, "bind_pipeline");

    let set_scissor = g.add(CmdSetScissorNode);
    g.feed(Value::Device(state.device.clone()), set_scissor, "device");
    g.feed(Value::SwapchainBundle(state.swapchain_bundle.clone()), set_scissor, "swapchain_bundle");
    g.connect(begin_cmd, "command_buffer", set_scissor, "command_buffer");
    g.connect(set_viewport, "done", set_scissor, "set_viewport");

    let draw = g.add(CmdDrawNode);
    g.feed(Value::Device(state.device.clone()), draw, "device");
    g.connect(begin_cmd, "command_buffer", draw, "command_buffer");
    g.connect(set_scissor, "done", draw, "set_scissor");

    let end_rp = g.add(CmdEndRenderPassNode);
    g.feed(Value::Device(state.device.clone()), end_rp, "device");
    g.connect(begin_cmd, "command_buffer", end_rp, "command_buffer");
    g.connect(draw, "done", end_rp, "draw");

    let end_cmd = g.add(EndCommandBufferNode);
    g.feed(Value::Device(state.device.clone()), end_cmd, "device");
    g.connect(begin_cmd, "command_buffer", end_cmd, "command_buffer");
    g.connect(end_rp, "done", end_cmd, "end_render_pass");

    let submit = g.add(QueueSubmitNode);
    g.feed(Value::Device(state.device.clone()), submit, "device");
    g.feed(Value::Queue(state.graphics_queue), submit, "queue");
    g.feed(Value::SyncBundle(state.sync_bundle.clone()), submit, "sync_bundle");
    g.feed(Value::Usize(state.current_frame), submit, "frame_index");
    g.connect(begin_cmd, "command_buffer", submit, "command_buffer");
    g.connect(end_cmd, "done", submit, "end_command_buffer");

    let present = g.add(QueuePresentNode);
    g.feed(Value::SwapchainLoader(state.swapchain_loader.clone()), present, "swapchain_loader");
    g.feed(Value::Queue(state.present_queue), present, "queue");
    g.feed(Value::SwapchainBundle(state.swapchain_bundle.clone()), present, "swapchain_bundle");
    g.feed(Value::SyncBundle(state.sync_bundle.clone()), present, "sync_bundle");
    g.feed(Value::Usize(state.current_frame), present, "frame_index");
    g.connect(acquire, "image_index", present, "image_index");
    g.connect(submit, "done", present, "queue_submit");

    g.execute();

    state.current_frame = (state.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
}

/// Destroys every Vulkan object created by the setup graph, in reverse
/// dependency order. Not itself a node — teardown is a side effect of the
/// program ending, not a data-producing step the graph needs to schedule.
unsafe fn cleanup(state: &RenderState) {
    state.device.device_wait_idle().ok();
    for &fence in &state.sync_bundle.in_flight {
        state.device.destroy_fence(fence, None);
    }
    for &sem in &state.sync_bundle.render_finished {
        state.device.destroy_semaphore(sem, None);
    }
    for &sem in &state.sync_bundle.image_available {
        state.device.destroy_semaphore(sem, None);
    }
    for &fb in state.framebuffers.iter() {
        state.device.destroy_framebuffer(fb, None);
    }
    state.device.destroy_pipeline(state.pipeline, None);
    state.device.destroy_render_pass(state.render_pass, None);
    state.swapchain_loader.destroy_swapchain(state.swapchain_bundle.swapchain, None);
}

fn main() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Node-Graph Vulkan Triangle")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .build(&event_loop)
            .expect("failed to create window"),
    );

    let mut state = run_setup_graph(window.clone());

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    elwt.exit();
                }
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    run_frame_graph(&mut state);
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                Event::LoopExiting => {
                    unsafe { cleanup(&state) };
                }
                _ => {}
            }
        })
        .expect("event loop exited with an error");
}
