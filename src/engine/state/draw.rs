use std::sync::Arc;



use crate::{meshes::Mesh, engine::{bind_group, texture::Texture}};

use super::State;

impl<'a> State<'a> {
    #[inline]
    pub(super) fn draw_all(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        output_texture: &wgpu::Texture,
        view: &wgpu::TextureView,
        meshes: &[Arc<Mesh>],
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass Start"),
            color_attachments: &[Some(self.get_rpass_color_attachment(view, wgpu::StoreOp::Discard))],
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

        self.draw_block(&mut render_pass, meshes);
        self.draw_transport_belt(&mut render_pass, meshes);
        self.draw_animated_model(&mut render_pass, meshes);
        self.draw_model(&mut render_pass, meshes);
        self.draw_selection(&mut render_pass);
        drop(render_pass);
        let texture_dst = Texture::create_copy_dst_texture(&self.device, &self.config);
        encoder.copy_texture_to_texture(output_texture.as_image_copy(),
            texture_dst.as_image_copy(), Texture::get_screen_size(&self.config));
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass Start"),
            color_attachments: &[Some(self.get_clear_rpass_color_attachment(view, wgpu::StoreOp::Discard))],
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
        let glass_dst = Texture::create_copy_dst_texture(&self.device, &self.config);
        encoder.copy_texture_to_texture(output_texture.as_image_copy(),
        glass_dst.as_image_copy(), Texture::get_screen_size(&self.config));

        let layout = if self.sample_count == 1 {
            &self.layouts.post_process
        } else {
            &self.layouts.multisampled_post_process
        };
        let post_process_bg = bind_group::post_process::get(
            &self.device, layout,
            &texture_dst.create_view(&wgpu::TextureViewDescriptor::default()),
            &glass_dst.create_view(&wgpu::TextureViewDescriptor::default()),
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
    }

    /// set bind group = 1 (block_texutre_bg)
    #[inline]
    fn draw_block<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>, meshes: &'b [Arc<Mesh>]) {
        render_pass.set_pipeline(&self.pipelines.block);
        render_pass.set_bind_group(1, &self.block_texutre_bg, &[]);
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
        render_pass.set_bind_group(1, &self.block_texutre_bg, &[]);
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
        render_pass.set_bind_group(3, &self.bind_groups_buffers.time.bind_group, &[]);
        meshes.iter().filter(|m| m.transport_belt_index_count > 0).for_each(|mesh| {
            render_pass.set_vertex_buffer(0, mesh.transport_belt_vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.transport_belt_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.transport_belt_index_count, 0, 0..1);
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
            mesh.animated_models.iter().for_each(|(id, (instance, len))| {
                let Some(animated_model) = self.animated_models.get(*id as usize) else { return; };
                if !mesh.animated_models.is_empty() {
                    render_pass.set_bind_group(1, &animated_model.texture, &[]);
                    render_pass.set_vertex_buffer(0, animated_model.vertex_buffer.slice(..));

                    render_pass.set_vertex_buffer(1, instance.slice(..));
                    render_pass.draw(0..animated_model.vertex_count as u32, 0..*len as u32);
                }
            });
        });
    }

    /// set bind group = 1 (model.texture)
    #[inline]
    fn draw_model<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>, meshes: &'b [Arc<Mesh>]) {
        render_pass.set_pipeline(&self.pipelines.model);
        meshes.iter().for_each(|mesh| {
            mesh.models.iter().for_each(|(name, (instance, len))| {
                let Some(model) = self.models.get(*name as usize) else {return};

                render_pass.set_bind_group(1, &model.texture, &[]);

                render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance.slice(..));

                render_pass.draw(0..model.vertex_count as u32, 0..*len as u32);
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
}