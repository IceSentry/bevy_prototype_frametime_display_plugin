use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{
            encase, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
            BufferBindingType, BufferInitDescriptor, BufferUsages, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
    },
    sprite::{Material2d, Material2dPipeline},
};
const FRAMETIME_LEN: usize = 200;
const DT_MIN: f32 = 1. / 240.;
const DT_MAX: f32 = 1. / 15.;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct FrametimeMaterial {
    config: FrametimeConfig,
    pub(crate) frametimes: Frametimes,
}

impl Default for FrametimeMaterial {
    fn default() -> Self {
        info!(
            "testing FrametimeConfig {}",
            std::mem::size_of::<[f32; 4]>()
        );
        FrametimeConfig::assert_uniform_compat();
        info!("FrametimeConfig is a valid uniform");
        Self {
            config: Default::default(),
            frametimes: Default::default(),
        }
    }
}

#[derive(Debug, Clone, ShaderType)]
pub(crate) struct Frametimes {
    pub(crate) values: [f32; FRAMETIME_LEN],
}

impl Default for Frametimes {
    fn default() -> Self {
        Self {
            values: [0.0; FRAMETIME_LEN],
        }
    }
}

#[derive(Debug, Clone, ShaderType)]
struct FrametimeConfig {
    dt_min: f32,
    dt_max: f32,
    dt_min_log2: f32,
    dt_max_log2: f32,
    max_width: f32,
    len: i32,
    colors: Mat4,
    dts: Vec4,
}

impl Default for FrametimeConfig {
    fn default() -> Self {
        Self {
            dt_min: DT_MIN,
            dt_max: DT_MAX,
            dt_min_log2: DT_MIN.log2(),
            dt_max_log2: DT_MAX.log2(),
            // There's probably a better value for this
            max_width: FRAMETIME_LEN as f32,
            len: FRAMETIME_LEN as i32,
            colors: Mat4::from_cols_array_2d(&[
                Color::BLUE.as_linear_rgba_f32(),
                Color::GREEN.as_linear_rgba_f32(),
                Color::YELLOW.as_linear_rgba_f32(),
                Color::RED.as_linear_rgba_f32(),
            ]),
            #[rustfmt::skip]
            dts: Vec4::new(
                DT_MIN,
                1. / 60.,
                1. / 30.,
                DT_MAX,
            ),
        }
    }
}

#[derive(Clone)]
pub struct GpuFrametimeMaterial {
    _config_buffer: Buffer,
    _frametimes_buffer: Buffer,
    bind_group: BindGroup,
}

impl RenderAsset for FrametimeMaterial {
    type ExtractedAsset = FrametimeMaterial;
    type PreparedAsset = GpuFrametimeMaterial;
    type Param = (SRes<RenderDevice>, SRes<Material2dPipeline<Self>>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let config_buffer = {
            let mut buffer = encase::UniformBuffer::new(Vec::new());
            buffer.write(&extracted_asset.config).unwrap();

            render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("config buffer"),
                contents: buffer.as_ref(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let frametimes_buffer = {
            let mut buffer = encase::StorageBuffer::new(Vec::new());
            buffer.write(&extracted_asset.frametimes).unwrap();

            render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("frametimes buffer"),
                contents: buffer.as_ref(),
                usage: BufferUsages::STORAGE,
            })
        };

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("frametime bind group"),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: config_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: frametimes_buffer.as_entire_binding(),
                },
            ],
            layout: &material_pipeline.material2d_layout,
        });

        Ok(GpuFrametimeMaterial {
            _config_buffer: config_buffer,
            _frametimes_buffer: frametimes_buffer,
            bind_group,
        })
    }
}

impl Material2d for FrametimeMaterial {
    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/frametime_display.wgsl"))
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Frametime Display Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(FrametimeConfig::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(Frametimes::min_size()),
                    },
                    count: None,
                },
            ],
        })
    }
}
