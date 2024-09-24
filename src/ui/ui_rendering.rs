use vulkano::{pipeline::{Pipeline, PipelineBindPoint}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}};
 
use crate::{asset_library::AssetLibrary, ecs::World, rendering::{rendering_component::RenderingComponent, PipelineIdentifier}, state::State, types::material::Attachment};

pub struct UiRenderingComponent {}

impl RenderingComponent for UiRenderingComponent {
    fn render(
            &self,
            mut builder:
                AutoCommandBufferBuilder<
                    PrimaryAutoCommandBuffer<StandardCommandBufferAllocator>, 
                    StandardCommandBufferAllocator
                >,
            _world: &World,
            assets: &AssetLibrary,
            state: &State,
            _image_id: usize
            ) -> AutoCommandBufferBuilder<
                    PrimaryAutoCommandBuffer<StandardCommandBufferAllocator>, 
                    StandardCommandBufferAllocator
        > {
            
        for (_, ui_layout) in assets.ui.iter() {
            let material = assets.materials.get(&ui_layout.material).unwrap();
            let pipeline = state.renderer.pipelines.get(&PipelineIdentifier::new(material.vertex_shader, material.fragment_shader, material.rendering_type)).unwrap().clone();
            let material_set = PersistentDescriptorSet::new(
                state.memory_allocators.descriptor_set_allocator.as_ref(),
                pipeline.layout().set_layouts().first().unwrap().clone(),
                [WriteDescriptorSet::buffer(
                    0,
                    material.parameter_buffer.as_ref().unwrap().clone(),
                )],
                [],
            ).unwrap();
            
            let attachment_set = if !material.attachments.is_empty() {
                Some({
                    PersistentDescriptorSet::new(
                        state.memory_allocators.descriptor_set_allocator.as_ref(),
                        pipeline.layout().set_layouts().get(1).unwrap().clone(),
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
                            },
                            Attachment::DefaultTexture => {
                                let (_, tex) = assets.textures.iter().find(|(_, x)| x.name == *"default").unwrap();
                                WriteDescriptorSet::image_view_sampler(
                                    id as u32,
                                    tex.image_view.as_ref().unwrap().clone(),
                                    tex.sampler.as_ref().unwrap().clone(),
                                )
                            }
                        })
                        .collect::<Vec<_>>(),
                        [],
                    ).unwrap()
                })
            } else {
                None
            };
            
            let mut sets = vec![material_set];
            if let Some(attachment_set) = attachment_set {
                sets.push(attachment_set);
            }

            builder.bind_pipeline_graphics(pipeline.clone()).unwrap();
            builder.bind_descriptor_sets(PipelineBindPoint::Graphics, pipeline.layout().clone(), 0, sets).unwrap();
            builder.bind_index_buffer(ui_layout.mesh.as_ref().unwrap().index_buffer.as_ref().unwrap().clone()).unwrap();
            builder.bind_vertex_buffers(0, ui_layout.mesh.as_ref().unwrap().vertex_buffer.as_ref().unwrap().clone()).unwrap();
            builder.draw_indexed(ui_layout.mesh.as_ref().unwrap().indices.len() as u32, 1, 0, 0, 0).unwrap();
        }

        builder
    }
    
}
