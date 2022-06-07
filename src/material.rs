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

#[derive(Debug, Clone, TypeUuid, ShaderType)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct FrametimeMaterial {
    pub dt_min: f32,
    pub dt_max: f32,
    pub dt_min_log2: f32,
    pub dt_max_log2: f32,
    pub max_width: f32,
    pub len: i32,
    pub frametimes: [f32; FRAMETIME_LEN],
}

impl Default for FrametimeMaterial {
    fn default() -> Self {
        Self {
            dt_min: DT_MIN,
            dt_max: DT_MAX,
            dt_min_log2: DT_MIN.log2(),
            dt_max_log2: DT_MAX.log2(),
            // There's probably a better value for this
            max_width: FRAMETIME_LEN as f32,
            len: FRAMETIME_LEN as i32,
            frametimes: [0.0; FRAMETIME_LEN],
        }
    }
}

#[derive(Clone)]
pub struct GpuFrametimeMaterial {
    _buffer: Buffer,
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
        let mut buffer = encase::StorageBuffer::new(Vec::new());
        buffer.write(&extracted_asset).unwrap();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("frametime buffer"),
            contents: buffer.as_ref(),
            usage: BufferUsages::STORAGE,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("frametime bind group"),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            layout: &material_pipeline.material2d_layout,
        });

        Ok(GpuFrametimeMaterial {
            _buffer: buffer,
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
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(FrametimeMaterial::min_size()),
                },
                count: None,
            }],
        })
    }
}
