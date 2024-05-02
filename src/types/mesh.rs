use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, sync::{now, GpuFuture}};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::{Renderer, VertexData}, state::State};

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub material: String,
    pub vertex_buffer: Option<Subbuffer<[VertexData]>>,
    pub index_buffer: Option<Subbuffer<[u32]>>,
}

impl Mesh {
    pub fn load(&mut self, renderer: &mut Renderer) {
        self.vertex_buffer = Some(
            Buffer::from_iter(
                renderer.memeory_allocator.as_ref().unwrap().clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
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
                renderer.memeory_allocator.as_ref().unwrap().clone(),
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
}

pub struct MeshLoader {}

impl System for MeshLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        for mesh in assets.meshes.iter_mut() {
            mesh.load(&mut state.renderer);
        }
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}

#[derive(Debug, Clone)]
pub struct DynamicMesh {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub material: String,
    pub vertex_buffer: Option<Subbuffer<[VertexData]>>,
    pub index_buffer: Option<Subbuffer<[u32]>>,
}

impl DynamicMesh {
    pub fn from_mesh(mesh: String, assets: &AssetLibrary) -> DynamicMesh {
        let mesh = assets.meshes.iter().find(|x| x.name == mesh).unwrap();
        DynamicMesh {
            vertices: mesh.vertices.clone(),
            indices: mesh.indices.clone(),
            material: mesh.material.clone(),
            vertex_buffer: None,
            index_buffer: None
        }
    }

    pub fn load(&mut self, renderer: &mut Renderer) {
        self.vertex_buffer = Some(
            Buffer::from_iter(
                renderer.memeory_allocator.as_ref().unwrap().clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
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
                renderer.memeory_allocator.as_ref().unwrap().clone(),
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
        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            renderer.device.as_ref().unwrap().clone(),
            Default::default(),
        );

        let mut builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            renderer.queue.as_ref().unwrap().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        let temp_buffer = Buffer::from_iter(
            renderer.memeory_allocator.as_ref().unwrap().clone(),
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

        let future = now(renderer.device.as_ref().unwrap().clone())
            .then_execute(renderer.queue.as_ref().unwrap().clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }
    
    pub fn change_vertices(&mut self, renderer: &Renderer, vec: Vec<VertexData>) {
        self.vertices = vec;
        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            renderer.device.as_ref().unwrap().clone(),
            Default::default(),
        );

        let mut builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            renderer.queue.as_ref().unwrap().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        let temp_buffer = Buffer::from_iter(
            renderer.memeory_allocator.as_ref().unwrap().clone(),
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

        let future = now(renderer.device.as_ref().unwrap().clone())
            .then_execute(renderer.queue.as_ref().unwrap().clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        // future.wait(None).unwrap();
    }
}

pub struct DynamicMeshLoader {}

impl System for DynamicMeshLoader {
    fn on_start(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {
        for (_, mesh) in world.entities.query::<&mut DynamicMesh>().iter() {
            mesh.load(&mut state.renderer);
        }
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
