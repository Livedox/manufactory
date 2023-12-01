use crate::meshes::Meshes;

use super::State;

impl State {
    #[inline]
    pub(super) fn draw_all(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        rpass_color_attachment: wgpu::RenderPassColorAttachment,
        indices: &[usize],
        meshes: &Meshes,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(rpass_color_attachment)],
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

        self.draw_block(&mut render_pass, indices, meshes);
        self.draw_transport_belt(&mut render_pass, indices, meshes);
        self.draw_animated_model(&mut render_pass, indices, meshes);
        self.draw_model(&mut render_pass, indices, meshes);
        self.draw_selection(&mut render_pass);
        self.draw_crosshair(&mut render_pass);
    }

    /// set bind group = 1 (block_texutre_bg)
    #[inline]
    fn draw_block<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, indices: &[usize], meshes: &'a Meshes) {
        render_pass.set_pipeline(&self.pipelines.block);
        render_pass.set_bind_group(1, &self.block_texutre_bg, &[]);
        indices.iter().for_each(|i| {
            if let Some(Some(mesh)) = &meshes.meshes().get(*i) {
                render_pass.set_vertex_buffer(0, mesh.block_vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.block_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..mesh.block_index_count, 0, 0..1);
            }
        });
    }

    /// set bind group = 3 (time)
    #[inline]
    fn draw_transport_belt<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, indices: &[usize], meshes: &'a Meshes) {
        render_pass.set_pipeline(&self.pipelines.transport_belt);
        render_pass.set_bind_group(3, &self.bind_groups_buffers.time.bind_group, &[]);
        indices.iter().for_each(|i| {
            if let Some(Some(mesh)) = meshes.meshes().get(*i) {
                render_pass.set_vertex_buffer(0, mesh.transport_belt_vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.transport_belt_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..mesh.transport_belt_index_count, 0, 0..1);
            }
        });
    }

    /// set bind group = 3 (transformation_matrices)
    /// set bind group = 1 (animated_model.texture)
    #[inline]
    fn draw_animated_model<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, indices: &[usize], meshes: &'a Meshes) {
        render_pass.set_pipeline(&self.pipelines.animated_model);
        indices.iter().for_each(|i| {
            let Some(Some(mesh)) = meshes.meshes().get(*i) else {return};
            let Some(bind_group) = &mesh.transformation_matrices_bind_group else {return};

            render_pass.set_bind_group(3, bind_group, &[]);
            mesh.animated_models.iter().for_each(|(name, (instance, len))| {
                let Some(animated_model) = self.animated_models.get(name) else { return; };
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
    fn draw_model<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, indices: &[usize], meshes: &'a Meshes) {
        render_pass.set_pipeline(&self.pipelines.model);
        indices.iter().for_each(|i| {
            let Some(Some(mesh)) = &meshes.meshes().get(*i) else {return};

            mesh.models.iter().for_each(|(name, (instance, len))| {
                let Some(model) = self.models.get(name) else {return};

                render_pass.set_bind_group(1, &model.texture, &[]);

                render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance.slice(..));

                render_pass.draw(0..model.vertex_count as u32, 0..*len as u32);
            });
        });
    }

    /// set bind group = 0 (camera)
    #[inline]
    fn draw_selection<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if let Some(selection_vertex_buffer) = &self.selection_vertex_buffer {
            render_pass.set_pipeline(&self.pipelines.selection);
            render_pass.set_bind_group(0, &self.bind_groups_buffers.camera.bind_group, &[]);
            render_pass.set_vertex_buffer(0, selection_vertex_buffer.slice(..));
            render_pass.draw(0..24, 0..1);
        }
    }

    /// set bind group = 0 (crosshair_aspect_scale)
    #[inline]
    fn draw_crosshair<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.is_crosshair {
            render_pass.set_pipeline(&self.pipelines.crosshair);
            render_pass.set_bind_group(0, &self.bind_groups_buffers.crosshair_aspect_scale.bind_group, &[]);
            render_pass.draw(0..12, 0..1);
        }
    }
}