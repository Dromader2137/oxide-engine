use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        BufferUsage, Subbuffer,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    memory::allocator::MemoryTypeFilter,
    pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint},
};

use crate::{
    asset_library::AssetLibrary,
    state::State,
    types::{
        material::{Attachment, Material},
        mesh::DynamicMesh,
        model::ModelComponent,
        transform::{ModelData, Transform},
    },
    vulkan::memory::MemoryAllocators,
};

use super::{rendering_component::RenderingComponent, Matrix4f, PipelineIdentifier};

pub struct MeshRenderingComponent {
    model_allocator: SubbufferAllocator,
}

impl MeshRenderingComponent {
    pub fn new(allocators: &MemoryAllocators) -> MeshRenderingComponent {
        MeshRenderingComponent {
            model_allocator: SubbufferAllocator::new(
                allocators.standard_memory_allocator.clone(),
                SubbufferAllocatorCreateInfo {
                    buffer_usage: BufferUsage::STORAGE_BUFFER,
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
            ),
        }
    }
}

fn get_descriptor_sets(
    state: &State,
    assets: &AssetLibrary,
    material: &Material,
    pipeline: &GraphicsPipeline,
    model: &Subbuffer<ModelData>,
    image_id: usize,
) -> Vec<std::sync::Arc<PersistentDescriptorSet>> {
    let vp_set = PersistentDescriptorSet::new(
        state.memory_allocators.descriptor_set_allocator.as_ref(),
        pipeline.layout().set_layouts().first().unwrap().clone(),
        [WriteDescriptorSet::buffer(
            0,
            state.renderer.vp_buffers.get(image_id).unwrap().clone(),
        )],
        [],
    )
    .unwrap();

    let m_set = PersistentDescriptorSet::new(
        state.memory_allocators.descriptor_set_allocator.as_ref(),
        pipeline.layout().set_layouts().get(1).unwrap().clone(),
        [WriteDescriptorSet::buffer(0, model.clone())],
        [],
    )
    .unwrap();

    let attachment_set = if !material.attachments.is_empty() {
        Some({
            PersistentDescriptorSet::new(
                state.memory_allocators.descriptor_set_allocator.as_ref(),
                pipeline.layout().set_layouts().get(2).unwrap().clone(),
                material
                    .attachments
                    .iter()
                    .enumerate()
                    .map(|(id, attachment)| match attachment {
                        Attachment::Texture(uuid) => {
                            let tex = assets.textures.get(uuid).unwrap();
                            WriteDescriptorSet::image_view_sampler(
                                id as u32,
                                tex.image_view.as_ref().unwrap().clone(),
                                tex.sampler.as_ref().unwrap().clone(),
                            )
                        }
                        Attachment::DefaultTexture => {
                            let (_, tex) = assets
                                .textures
                                .iter()
                                .find(|(_, x)| x.name == *"default")
                                .unwrap();
                            WriteDescriptorSet::image_view_sampler(
                                id as u32,
                                tex.image_view.as_ref().unwrap().clone(),
                                tex.sampler.as_ref().unwrap().clone(),
                            )
                        }
                    })
                    .collect::<Vec<_>>(),
                [],
            )
            .unwrap()
        })
    } else {
        None
    };

    let material_set = if material.parameter_buffer.is_some() {
        Some(
            PersistentDescriptorSet::new(
                state.memory_allocators.descriptor_set_allocator.as_ref(),
                pipeline
                    .layout()
                    .set_layouts()
                    .get({
                        if attachment_set.is_some() {
                            3
                        } else {
                            2
                        }
                    })
                    .unwrap()
                    .clone(),
                [WriteDescriptorSet::buffer(
                    0,
                    material.parameter_buffer.as_ref().unwrap().clone(),
                )],
                [],
            )
            .unwrap(),
        )
    } else {
        None
    };

    let mut sets = vec![vp_set, m_set];
    if let Some(attachment_set) = attachment_set {
        sets.push(attachment_set);
    }
    if let Some(material_set) = material_set {
        sets.push(material_set);
    }

    sets
}

impl RenderingComponent for MeshRenderingComponent {
    fn render(
        &self,
        mut builder: vulkano::command_buffer::AutoCommandBufferBuilder<
            vulkano::command_buffer::PrimaryAutoCommandBuffer<
                vulkano::command_buffer::allocator::StandardCommandBufferAllocator,
            >,
            vulkano::command_buffer::allocator::StandardCommandBufferAllocator,
        >,
        world: &crate::ecs::World,
        assets: &crate::asset_library::AssetLibrary,
        state: &crate::state::State,
        image_id: usize,
    ) -> vulkano::command_buffer::AutoCommandBufferBuilder<
        vulkano::command_buffer::PrimaryAutoCommandBuffer<
            vulkano::command_buffer::allocator::StandardCommandBufferAllocator,
        >,
        vulkano::command_buffer::allocator::StandardCommandBufferAllocator,
    > {
        let camera_pos = state.renderer.vp_pos;
        let entities = world.entities.borrow();
        for (_, (dyn_mesh, transform)) in entities.query::<(&DynamicMesh, &Transform)>().iter() {
            let model = ModelData {
                translation: Matrix4f::translation((transform.position - camera_pos).to_vec3f()),
                rotation: transform.rotation.to_matrix(),
                scale: Matrix4f::scale(transform.scale),
            };
            let model_buffer = self.model_allocator.allocate_sized().unwrap();
            *model_buffer.write().unwrap() = model;

            let mesh = assets
                .meshes
                .get(&dyn_mesh.mesh.expect("Mesh not set"))
                .expect("Mesh not found");
            let vertex_buffer = mesh
                .vertex_buffer
                .as_ref()
                .expect("Vertex buffer not found")
                .as_ref();
            let index_buffer = mesh
                .index_buffer
                .as_ref()
                .expect("Index buffer not found")
                .as_ref();
            let material = assets
                .materials
                .get(&dyn_mesh.material)
                .expect("Material not found");
            let pipeline = state
                .renderer
                .pipelines
                .get(&PipelineIdentifier::new(
                    material.vertex_shader,
                    material.fragment_shader,
                    material.rendering_type,
                ))
                .unwrap();

            let descriptor_sets =
                get_descriptor_sets(state, assets, material, pipeline, &model_buffer, image_id);

            builder
                .bind_pipeline_graphics(pipeline.clone())
                .expect("GP bind faild");
            builder
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline.layout().clone(),
                    0,
                    descriptor_sets,
                )
                .unwrap();
            builder
                .bind_index_buffer(index_buffer.clone())
                .expect("Index buffer bind failed");
            builder
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .expect("Vertex buffer bind failed");
            builder
                .draw_indexed(mesh.indices.len() as u32, 1, 0, 0, 0)
                .expect("Draw failed");
        }

        for (_, (model_comp, transform)) in entities.query::<(&ModelComponent, &Transform)>().iter() {
            let model = assets.models.get(&model_comp.model_uuid).unwrap();
            for (mesh_uuid, material_uuid) in model.meshes_and_materials.iter() {
                let model = ModelData {
                    translation: Matrix4f::translation(
                        (transform.position - camera_pos).to_vec3f(),
                    ),
                    rotation: transform.rotation.to_matrix(),
                    scale: Matrix4f::scale(transform.scale),
                };
                let model_buffer = self.model_allocator.allocate_sized().unwrap();
                *model_buffer.write().unwrap() = model;

                let mesh = assets.meshes.get(mesh_uuid).expect("Mesh not found");
                let vertex_buffer = mesh
                    .vertex_buffer
                    .as_ref()
                    .expect("Vertex buffer not found")
                    .as_ref();
                let index_buffer = mesh
                    .index_buffer
                    .as_ref()
                    .expect("Index buffer not found")
                    .as_ref();
                let material = assets
                    .materials
                    .get(material_uuid)
                    .expect("Material not found");
                let pipeline = state
                    .renderer
                    .pipelines
                    .get(&PipelineIdentifier::new(
                        material.vertex_shader,
                        material.fragment_shader,
                        material.rendering_type,
                    ))
                    .unwrap();

                let descriptor_sets =
                    get_descriptor_sets(state, assets, material, pipeline, &model_buffer, image_id);

                builder
                    .bind_pipeline_graphics(pipeline.clone())
                    .expect("GP bind faild");
                builder
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipeline.layout().clone(),
                        0,
                        descriptor_sets,
                    )
                    .unwrap();
                builder
                    .bind_index_buffer(index_buffer.clone())
                    .expect("Index buffer bind failed");
                builder
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .expect("Vertex buffer bind failed");
                builder
                    .draw_indexed(mesh.indices.len() as u32, 1, 0, 0, 0)
                    .expect("Draw failed");
            }
        }

        builder
    }
}
