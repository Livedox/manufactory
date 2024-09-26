use std::sync::Arc;



use crate::{bind_group, mesh::{Mesh, MeshBuffer}, player_mesh::PlayerMesh, texture::Texture};

use super::State;

impl<'a> State<'a> {
    #[inline]
    pub(super) fn draw_all(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        output_texture: &wgpu::Texture,
        view: &wgpu::TextureView,
        meshes: &[Arc<Mesh>],
        players: &[PlayerMesh]
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass Start"),
            color_attachments: &[Some(self.get_rpass_color_attachment(view, wgpu::StoreOp::Store))],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Set sun
        render_pass.set_bind_group(0, &self.bind_groups_buffers.sun.bind_group, &[]);

        // Set camera
        render_pass.set_bind_group(2, &self.bind_groups_buffers.camera.bind_group, &[]);

        self.draw_players(&mut render_pass, players);
        self.draw_block(&mut render_pass, meshes);
        self.draw_transport_belt(&mut render_pass, meshes);
        self.draw_animated_model(&mut render_pass, meshes);
        self.draw_model(&mut render_pass, meshes);
        drop(render_pass);
        
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass Glass"),
            color_attachments: &[
                Some(
                    if self.sample_count == 1 {
                        wgpu::RenderPassColorAttachment {
                            view: &self.accum_texture.view, 
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color { 
                                    r: 0.0, 
                                    g: 0.0, 
                                    b: 0.0, 
                                    a: 0.0, 
                                }), 
                                store: wgpu::StoreOp::Store,
                            },
                            resolve_target: None,
                        }
                    } else {
                        wgpu::RenderPassColorAttachment {
                            view: &self.multisampled_glass_framebuffer, 
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color { 
                                    r: 0.0, 
                                    g: 0.0, 
                                    b: 0.0, 
                                    a: 0.0, 
                                }), 
                                store: wgpu::StoreOp::Store,
                            },
                            resolve_target: Some(&self.accum_texture.view),
                        }
                    }
                ),
                Some(
                    if self.sample_count == 1 {
                        wgpu::RenderPassColorAttachment {
                            view: &self.reveal_texture.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color { 
                                    r: 1.0, 
                                    g: 1.0, 
                                    b: 1.0, 
                                    a: 1.0, 
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        }
                    } else {
                        wgpu::RenderPassColorAttachment {
                            view: &self.multisampled_reveal_framebuffer,
                            resolve_target: Some(&self.reveal_texture.view),
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color { 
                                    r: 1.0, 
                                    g: 1.0, 
                                    b: 1.0, 
                                    a: 1.0, 
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        }
                    }
                )
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_bind_group(0, &self.bind_groups_buffers.sun.bind_group, &[]);
        render_pass.set_bind_group(2, &self.bind_groups_buffers.camera.bind_group, &[]);
        self.draw_glass(&mut render_pass, meshes);
        drop(render_pass);

        let composite_bg = bind_group::oit::get(&self.device, &self.layouts.oit,
            &self.accum_texture.view, &self.reveal_texture.view);
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Composite"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        self.draw_composite(&mut render_pass, &composite_bg);
        drop(render_pass);

        let texture_dst = Texture::create_copy_dst_texture(&self.device, &self.config);
        encoder.copy_texture_to_texture(output_texture.as_image_copy(),
            texture_dst.as_image_copy(), Texture::get_screen_size(&self.config));

        let layout = if self.sample_count == 1 {
            &self.layouts.post_process
        } else {
            &self.layouts.multisampled_post_process
        };
        let post_process_bg = bind_group::post_process::get(
            &self.device, layout,
            &texture_dst.create_view(&wgpu::TextureViewDescriptor::default()),
            &self.depth_texture.view);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass Post Proccess"),
            color_attachments: &[Some(self.get_rpass_color_attachment(view, wgpu::StoreOp::Store))],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        
        self.draw_post_process(&mut render_pass, &post_process_bg);
        self.draw_crosshair(&mut render_pass);
        drop(render_pass);
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass Selection"),
            color_attachments: &[Some(if self.sample_count == 1 {
                wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                }
            } else {
                wgpu::RenderPassColorAttachment {
                    view: &self.multisampled_framebuffer,
                    resolve_target: Some(view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                }
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        self.draw_selection(&mut render_pass);
    }

    /// set bind group = 1 (block_texutre_bg)
    #[inline]
    fn draw_block<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>, meshes: &'b [Arc<Mesh>]) {
        render_pass.set_pipeline(&self.pipelines.block);
        render_pass.set_bind_group(1, self.resources().block_bind_group(), &[]);
        meshes.iter().filter(|m| m.block_index_count > 0).for_each(|mesh| {
            render_pass.set_vertex_buffer(0, mesh.block_vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.block_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.block_index_count, 0, 0..1);
        });
    }

    /// set bind group = 1 (block_texutre_bg)
    #[inline]
    fn draw_glass<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>, meshes: &'b [Arc<Mesh>]) {
        render_pass.set_pipeline(&self.pipelines.glass);
        render_pass.set_bind_group(1, self.resources().block_bind_group(), &[]);
        meshes.iter().filter(|m| m.glass_index_count > 0).for_each(|mesh| {
            render_pass.set_vertex_buffer(0, mesh.glass_vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.glass_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.glass_index_count, 0, 0..1);
        });
    }

    /// set bind group = 3 (time)
    #[inline]
    fn draw_transport_belt<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>, meshes: &'b [Arc<Mesh>]) {
        render_pass.set_pipeline(&self.pipelines.transport_belt);
        render_pass.set_bind_group(1, self.resources().block_bind_group(), &[]);
        render_pass.set_bind_group(3, &self.bind_groups_buffers.time.bind_group, &[]);
        meshes.iter().filter(|m| m.belt_index_count > 0).for_each(|mesh| {
            render_pass.set_vertex_buffer(0, mesh.belt_vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.belt_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.belt_index_count, 0, 0..1);
        });
    }

    #[inline]
    fn draw_players<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>, players: &'b [PlayerMesh]) {
        render_pass.set_pipeline(&self.pipelines.player);
        render_pass.set_bind_group(1, self.resources().player_bind_group(), &[]);
        players.iter().for_each(|mesh| {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.draw(0..mesh.vertex_count, 0..1);
        });
    }

    /// set bind group = 3 (transformation_matrices)
    /// set bind group = 1 (animated_model.texture)
    #[inline]
    fn draw_animated_model<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>, meshes: &'b [Arc<Mesh>]) {
        render_pass.set_pipeline(&self.pipelines.animated_model);
        meshes.iter().for_each(|mesh| {
            let Some(bind_group) = &mesh.transformation_matrices_bind_group else {return};

            render_pass.set_bind_group(3, bind_group, &[]);
            mesh.animated_models.iter().for_each(|MeshBuffer {id, size, buffer}| {
                let Some(animated_model) = self.resources().animated_models().get(*id as usize) else { return; };
                if !mesh.animated_models.is_empty() {
                    render_pass.set_bind_group(1, &animated_model.texture, &[]);
                    render_pass.set_vertex_buffer(0, animated_model.vertex_buffer.slice(..));

                    render_pass.set_vertex_buffer(1, buffer.slice(..));
                    render_pass.draw(0..animated_model.vertex_count as u32, 0..*size as u32);
                }
            });
        });
    }

    /// set bind group = 1 (model.texture)
    #[inline]
    fn draw_model<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>, meshes: &'b [Arc<Mesh>]) {
        render_pass.set_pipeline(&self.pipelines.model);
        meshes.iter().for_each(|mesh| {
            mesh.models.iter().for_each(|MeshBuffer {id, size, buffer}| {
                let Some(model) = self.resources().models().get(*id as usize) else {return};

                render_pass.set_bind_group(1, &model.texture, &[]);

                render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, buffer.slice(..));

                render_pass.draw(0..model.vertex_count as u32, 0..*size as u32);
            });
        });
    }

    /// set bind group = 0 (camera)
    #[inline]
    fn draw_selection<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
        if let Some(selection_vertex_buffer) = &self.selection_vertex_buffer {
            render_pass.set_pipeline(&self.pipelines.selection);
            render_pass.set_bind_group(0, &self.bind_groups_buffers.camera.bind_group, &[]);
            render_pass.set_vertex_buffer(0, selection_vertex_buffer.slice(..));
            render_pass.draw(0..24, 0..1);
        }
    }

    /// set bind group = 0 (crosshair_aspect_scale)
    #[inline]
    fn draw_crosshair<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
        if self.is_crosshair {
            render_pass.set_pipeline(&self.pipelines.crosshair);
            render_pass.set_bind_group(0, &self.bind_groups_buffers.crosshair_aspect_scale.bind_group, &[]);
            render_pass.draw(0..18, 0..1);
        }
    }

    #[inline]
    fn draw_post_process<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>, post_process_bg: &'b wgpu::BindGroup) {
        render_pass.set_bind_group(0, post_process_bg, &[]);
        if self.sample_count == 1 {
           render_pass.set_pipeline(&self.pipelines.post_process); 
        } else {
            render_pass.set_pipeline(&self.pipelines.multisampled_post_process);
        }
        render_pass.draw(0..3, 0..1);
    }

    #[inline]
    fn draw_composite<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>, composite_bg: &'b wgpu::BindGroup) {
        render_pass.set_bind_group(0, composite_bg, &[]);
        render_pass.set_pipeline(&self.pipelines.composite); 
        render_pass.draw(0..3, 0..1);
    }
}