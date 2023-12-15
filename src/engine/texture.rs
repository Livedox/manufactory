use image::DynamicImage;


pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        src: &str,
    ) -> Result<Self, ()> {
        let img = image::open(src).expect(src);
        let (width, height) = (img.width(), img.height());
        
        if width != height { panic!("Use square textures") }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture object"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            img.as_flat_samples_u8().unwrap().samples,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }


    pub fn image_array(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        srcs: &[&str],
        label: Option<&str>
    ) -> Result<Self, ()> {
        const BASE_SIZE: u32 = 32;
        //Maximum mipmap_count is BASE_SIZE.ilog2() + 1 (img size 1px) but it's too small
        let mipmap_count = BASE_SIZE.ilog2();
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: BASE_SIZE,
                height: BASE_SIZE,
                depth_or_array_layers: srcs.len() as u32,
            },
            mip_level_count: mipmap_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let mut images: Vec<DynamicImage> = vec![];
        srcs.iter().for_each(|src| { images.push(image::open(src).expect(src)) });
        (0..mipmap_count).for_each(|mipmap| {
            let mut data: Vec<u8> = vec![];
            let img_size = BASE_SIZE / 2u32.pow(mipmap);
            (0..srcs.len()).for_each(|i| {
                let rgba = match mipmap {
                    0 => images[i].to_rgba8(),
                    _ => images[i].resize(img_size, img_size, image::imageops::FilterType::Triangle).to_rgba8(),
                };
                data.extend_from_slice(&rgba);
            });

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: mipmap,
                    origin: wgpu::Origin3d::ZERO,
                },
                &data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * img_size),
                    rows_per_image: Some(img_size),
                },
                wgpu::Extent3d {
                    width: img_size,
                    height: img_size,
                    depth_or_array_layers: srcs.len() as u32,
                }
            );
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }


    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    
    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str, sample_count: u32) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }


    pub fn create_multisampled_framebuffer(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> wgpu::TextureView {
        let multisampled_texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            size: multisampled_texture_extent,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: config.view_formats[0],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };

        device
            .create_texture(multisampled_frame_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}


pub struct TextureAtlas {
    pub texture_id: egui::TextureId,
    pub uv_size: f32,
    pub row_count: f32,
}


impl TextureAtlas {
    pub fn new(
        render_pass: &mut egui_wgpu_backend::RenderPass,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        src: &str,
        row_count: u32
    ) -> Self {
        let img = image::open(src).expect(src);
        let (width, height) = (img.width(), img.height());
        
        if width != height { panic!("Use square textures") }
        if width & (width-1) != 0 { panic!("Image resolution must be a multiple of a power of two")}

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture atlas"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            img.as_flat_samples_u8().unwrap().samples,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_id = render_pass.egui_texture_from_wgpu_texture(device, &view, wgpu::FilterMode::Nearest);
        Self {
            texture_id,
            uv_size: 1.0 / row_count as f32,
            row_count: row_count as f32,
        }
    }


    pub fn u(&self, id: u32) -> f32 {
        (id as f32 % self.row_count).trunc()*self.uv_size
    }

    pub fn v(&self, id: u32) -> f32 {
        (id as f32 / self.row_count).trunc()*self.uv_size
    }

    pub fn uv(&self, id: u32) -> (f32, f32) {
        (self.u(id), self.v(id))
    }


    pub fn uv_rect(&self, id: u32) -> (f32, f32, f32, f32) {
        let uv = self.uv(id);
        (uv.0, uv.1, uv.0+self.uv_size, uv.1+self.uv_size)
    }
}
