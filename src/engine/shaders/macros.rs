#[macro_export]
macro_rules! shaders {
    ( $($name:ident),+ ) => {
        pub(crate) struct Shaders {
            $(
                pub(crate) $name: wgpu::ShaderModule,
            )*
        }

        impl Shaders {
            pub(crate) fn new(device: &wgpu::Device) -> Self {
                Self {
                    $(
                        $name: device.create_shader_module(
                            wgpu::include_wgsl!(concat!(stringify!($name), ".wgsl"))),
                    )*
                }
            }
        }
    };
}