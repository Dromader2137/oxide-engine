
use vulkano::{buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, pipeline::graphics::vertex_input::Vertex, sync::{now, GpuFuture}};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::Renderer, state::State, types::vectors::Vec2f};

use super::uibox::UiBox;

#[derive(BufferContents, Vertex, Clone, Copy, Debug)]
#[repr(C)]
pub struct UiVertexData {
    #[format(R32G32_SFLOAT)]
    pub position: Vec2f,
    #[format(R32G32_SFLOAT)]
    pub uv: Vec2f
}

pub struct UiStorage {
    pub vertices: Vec<UiVertexData>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Option<Subbuffer<[UiVertexData]>>,
    pub index_buffer: Option<Subbuffer<[u32]>>,
}

impl UiStorage {
    pub fn load(&mut self, renderer: &mut Renderer) {
        if self.indices.is_empty() || self.vertices.is_empty() { return; }        

        self.vertex_buffer = Some(
            Buffer::from_iter(
                renderer.memeory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                self.vertices.clone(),
            )
            .unwrap(),
        );
        self.index_buffer = Some(
            Buffer::from_iter(
                renderer.memeory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                self.indices.clone(),
            )
            .unwrap(),
        );
    }

    pub fn change_indices(&mut self, renderer: &Renderer, vec: Vec<u32>) {
        self.indices = vec;
        if self.indices.is_empty() || self.vertices.is_empty() { return; }        
        let mut builder = AutoCommandBufferBuilder::primary(
            &renderer.command_buffer_allocator,
            renderer.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        let temp_buffer = Buffer::from_iter(
            renderer.memeory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                    MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            self.indices.to_owned(),
        ).unwrap();

        builder
            .copy_buffer(
                CopyBufferInfo::buffers(temp_buffer, self.index_buffer.as_ref().unwrap().to_owned())
            )
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = now(renderer.device.clone())
            .then_execute(renderer.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }
    
    pub fn change_vertices(&mut self, renderer: &Renderer, vec: Vec<UiVertexData>) {
        self.vertices = vec;
        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            renderer.device.clone(),
            Default::default(),
        );

        if self.indices.is_empty() || self.vertices.is_empty() { return; }        
        let mut builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            renderer.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        let temp_buffer = Buffer::from_iter(
            renderer.memeory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                    MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            self.vertices.to_owned(),
        ).unwrap();

        builder
            .copy_buffer(
                CopyBufferInfo::buffers(temp_buffer, self.vertex_buffer.as_ref().unwrap().to_owned())
            )
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let _future = now(renderer.device.clone())
            .then_execute(renderer.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        // future.wait(None).unwrap();
    }
}

pub struct UiManager {}

impl System for UiManager {
    fn on_start(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {
        let mut ui_boxes = world.entities.query::<&UiBox>();
        let mut vertices: Vec<UiVertexData> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut ic: u32 = 0;
        for (_, ui_box) in ui_boxes.iter() {
            vertices.push(UiVertexData { position: Vec2f::new([ui_box.right, ui_box.up]) , 
                uv: Vec2f::new([1.0, 1.0])});
            vertices.push(UiVertexData { position: Vec2f::new([ui_box.right, ui_box.down]) , 
                uv: Vec2f::new([1.0, -1.0])});
            vertices.push(UiVertexData { position: Vec2f::new([ui_box.left, ui_box.down]) , 
                uv: Vec2f::new([-1.0, -1.0])});
            vertices.push(UiVertexData { position: Vec2f::new([ui_box.left, ui_box.up]) , 
                uv: Vec2f::new([-1.0, 1.0])});
            indices.append(&mut vec![ic, ic + 1, ic + 2, ic, ic + 2, ic + 3]);
            ic += 4;
        }
        
        state.ui.vertices = vertices;
        state.ui.indices = indices;
        state.ui.load(&mut state.renderer);
    }

    fn on_update(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {
        let mut ui_boxes = world.entities.query::<&UiBox>();
        let mut vertices: Vec<UiVertexData> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut ic: u32 = 0;
        for (_, ui_box) in ui_boxes.iter() {
            vertices.push(UiVertexData { position: Vec2f::new([ui_box.right, ui_box.up]) , 
                uv: Vec2f::new([1.0, 1.0])});
            vertices.push(UiVertexData { position: Vec2f::new([ui_box.right, ui_box.down]) , 
                uv: Vec2f::new([1.0, -1.0])});
            vertices.push(UiVertexData { position: Vec2f::new([ui_box.left, ui_box.down]) , 
                uv: Vec2f::new([-1.0, -1.0])});
            vertices.push(UiVertexData { position: Vec2f::new([ui_box.left, ui_box.up]) , 
                uv: Vec2f::new([-1.0, 1.0])});
            indices.append(&mut vec![ic, ic + 1, ic + 2, ic, ic + 2, ic + 3]);
            ic += 4;
        }

        state.ui.change_indices(&state.renderer, indices);
        state.ui.change_vertices(&state.renderer, vertices);
    }
}
