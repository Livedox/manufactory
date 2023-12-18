use crate::{meshes::Mesh, engine::{bind_group, texture::Texture}};

use super::State;

impl State {
    #[inline]
    pub(super) fn draw_all(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        rpass_color_attachment: wgpu::RenderPassColorAttachment,
        rpass_color_attachment2: wgpu::RenderPassColorAttachment,
        texture: &wgpu::Texture,
        meshes: &[&Mesh],
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass1"),
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

        self.draw_block(&mut render_pass, meshes);
        self.draw_transport_belt(&mut render_pass, meshes);
        self.draw_animated_model(&mut render_pass, meshes);
        self.draw_model(&mut render_pass, meshes);
        self.draw_selection(&mut render_pass);
        self.draw_crosshair(&mut render_pass);

        drop(render_pass);
        let size = wgpu::Extent3d {
            width: self.config.width,
            height: self.config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("copy color"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };
        let depth_desc = wgpu::TextureDescriptor {
            label: Some("copy depth"),
            size,
            mip_level_count: 1,
            sample_count: self.sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: Texture::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let textur_dst = self.device.create_texture(&desc);
        let depth_dst = self.device.create_texture(&depth_desc);
        encoder.copy_texture_to_texture(texture.as_image_copy(), textur_dst.as_image_copy(), size);
        encoder.copy_texture_to_texture(self.depth_texture.texture.as_image_copy(), depth_dst.as_image_copy(), size);
        let layout = if self.sample_count == 1 {
            &self.layouts.post_proccess_test
        } else {
            &self.layouts.multisampled_post_proccess
        };
        // let layout = &self.layouts.post_proccess_test;
        let post_proccess_bg = bind_group::post_proccess::get(
            &self.device,
            layout,
            &textur_dst.create_view(&wgpu::TextureViewDescriptor::default()),
            &self.depth_texture.view);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass2"),
            color_attachments: &[Some(rpass_color_attachment2)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        
        // self.draw_post_proccess_test(&mut render_pass, &post_proccess_bg);
        if self.sample_count == 1 {
            self.draw_post_proccess_test(&mut render_pass, &post_proccess_bg);
        } else {
            self.draw_multisampled_post_proccess_test(&mut render_pass, &post_proccess_bg);
        }
    }

    /// set bind group = 1 (block_texutre_bg)
    #[inline]
    fn draw_block<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, meshes: &[&'a Mesh]) {
        render_pass.set_pipeline(&self.pipelines.block);
        render_pass.set_bind_group(1, &self.block_texutre_bg, &[]);
        meshes.iter().for_each(|mesh| {
            render_pass.set_vertex_buffer(0, mesh.block_vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.block_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..mesh.block_index_count, 0, 0..1);
        });
    }

    /// set bind group = 3 (time)
    #[inline]
    fn draw_transport_belt<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, meshes: &[&'a Mesh]) {
        render_pass.set_pipeline(&self.pipelines.transport_belt);
        render_pass.set_bind_group(3, &self.bind_groups_buffers.time.bind_group, &[]);
        meshes.iter().for_each(|mesh| {
            render_pass.set_vertex_buffer(0, mesh.transport_belt_vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.transport_belt_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..mesh.transport_belt_index_count, 0, 0..1);
        });
    }

    /// set bind group = 3 (transformation_matrices)
    /// set bind group = 1 (animated_model.texture)
    #[inline]
    fn draw_animated_model<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, meshes: &[&'a Mesh]) {
        render_pass.set_pipeline(&self.pipelines.animated_model);
        meshes.iter().for_each(|mesh| {
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
    fn draw_model<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, meshes: &[&'a Mesh]) {
        render_pass.set_pipeline(&self.pipelines.model);
        meshes.iter().for_each(|mesh| {
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

    #[inline]
    fn draw_post_proccess_test<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, post_proccess_bg: &'a wgpu::BindGroup) {
        render_pass.set_bind_group(0, post_proccess_bg, &[]);
        render_pass.set_pipeline(&self.pipelines.post_proccess_test);
        render_pass.draw(0..3, 0..1);
    }

    #[inline]
    fn draw_multisampled_post_proccess_test<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, post_proccess_bg: &'a wgpu::BindGroup) {
        render_pass.set_bind_group(0, post_proccess_bg, &[]);
        render_pass.set_pipeline(&self.pipelines.multisampled_post_proccess_test);
        render_pass.draw(0..3, 0..1);
    }
}