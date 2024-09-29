use std::{collections::HashMap, sync::Arc};

use vulkano::{
    device::DeviceOwned,
    render_pass::{
        RenderPass, RenderPassCreateInfo, Subpass, SubpassDependency, SubpassDescription,
    },
    swapchain::Swapchain,
    sync::PipelineStages,
};

use crate::{assets::asset_descriptions::AttachmentDescription, types::material::Attachment, vulkan::context::VulkanContext};

use super::renderer_node::RendererNode;

pub struct Connection {
    from_node: u32,
    from_output: u32,
    to_node: u32,
    to_input: u32,
}

impl Connection {
    pub fn new(from_node: u32, from_output: u32, to_node: u32, to_input: u32) -> Connection {
        Connection {
            from_node,
            from_output,
            to_node,
            to_input,
        }
    }
}

pub struct VirtualAttachment {}

pub struct RenderGraph {
    pub nodes: Vec<Box<dyn RendererNode>>,
    pub connections: Vec<Connection>,
    pub attachments: Vec<VirtualAttachment>,
}

impl RenderGraph {
    pub fn new() -> RenderGraph {
        RenderGraph {
            nodes: vec![],
            connections: vec![],
            attachments: vec![],
        }
    }

    pub fn add_node<T: RendererNode + 'static>(&mut self, node: T) -> u32 {
        self.nodes.push(Box::new(node));
        self.nodes.len() as u32 - 1
    }

    pub fn add_connection(&mut self, connection: Connection) -> Result<(), &'static str> {
        if connection.from_node >= self.nodes.len() as u32 {
            return Err("Connection from node out of bounds");
        }
        let from_node = self.nodes.get(connection.from_node as usize).unwrap();
        if from_node.get_outputs().len() as u32 <= connection.from_output {
            return Err("Connection from output out of bounds");
        }
        let from_output = *from_node
            .get_outputs()
            .get(connection.from_output as usize)
            .unwrap();

        if connection.to_node >= self.nodes.len() as u32 {
            return Err("Connection to node out of bounds");
        }
        let to_node = self.nodes.get(connection.to_node as usize).unwrap();
        if to_node.get_outputs().len() as u32 <= connection.to_input {
            return Err("Connection to input out of bounds");
        }
        let to_input = *to_node
            .get_inputs()
            .get(connection.to_input as usize)
            .unwrap();

        if from_output == to_input {
            return Err("Input output type does not match");
        }

        self.connections.push(connection);
        Ok(())
    }

    pub fn get_render_pass(
        &self,
        vulkan_context: &VulkanContext,
        swapchain: Arc<Swapchain>,
    ) -> Arc<RenderPass> {
        let mut attachments = vec![];
        let mut attachment_map: HashMap<(u32, u32), u32> = HashMap::new();
        let mut subpasses = vec![];
        let dependencies = vec![];
        for (i, node) in self.nodes.iter().enumerate().map(|(i, v)| (i as u32, v)) {
            let mut node_attachments = node.get_attachments();

            let incoming_connections: Vec<&Connection> = self.connections.iter().filter_map(|x| {
                if x.to_node == i {
                    Some(x)
                } else {
                    None
                }
            }).collect();

            for attachment in node_attachments.iter() {
                if incoming_connections.iter().any(|x| x)

            }

            subpasses.push(SubpassDescription { dep });
        }


        RenderPass::new(
            vulkan_context.device.clone(),
            RenderPassCreateInfo {
                attachments,
                subpasses,
                dependencies,
                ..Default::default()
            },
        )
        .unwrap()
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}
