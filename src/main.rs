use bevy::{
    asset::AssetServerSettings,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    math::vec2,
    pbr::MaterialPipeline,
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
};

const FRAMETIME_LEN: usize = 240;
const DT_MIN: f32 = 1. / 144.;
const DT_MAX: f32 = 1. / 15.;

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(MaterialPlugin::<FrametimeMaterial>::default())
        .add_startup_system(setup)
        .add_system(update_frametimes)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FrametimeMaterial>>,
) {
    // TODO attach it to camera and display on top
    // maybe use a 2d material instead?
    commands.spawn().insert_bundle(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(vec2(4.0, 4.0)))),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: materials.add(FrametimeMaterial {
            dt_min: DT_MIN,
            dt_max: DT_MAX,
            dt_min_log2: DT_MIN.log2(),
            dt_max_log2: DT_MAX.log2(),
            // There's probably a better value for this
            max_width: FRAMETIME_LEN as f32,
            len: FRAMETIME_LEN as i32,
            frametimes: [0.0; FRAMETIME_LEN],
        }),
        ..default()
    });

    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn update_frametimes(
    diagnostics: Res<Diagnostics>,
    mut materials: ResMut<Assets<FrametimeMaterial>>,
    mut materials_query: Query<&Handle<FrametimeMaterial>>,
) {
    if let Some(frame_time_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        for material_handle in &mut materials_query {
            if let Some(material) = materials.get_mut(material_handle) {
                material.frametimes.rotate_left(1);
                let dt = frame_time_diagnostic.value();
                material.frametimes[FRAMETIME_LEN - 1] = dt.unwrap_or(0.0) as f32;
            }
        }
    }
}

// This is the struct that will be passed to your shader
#[derive(Debug, Clone, TypeUuid, ShaderType)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct FrametimeMaterial {
    dt_min: f32,
    dt_max: f32,
    dt_min_log2: f32,
    dt_max_log2: f32,
    max_width: f32,
    len: i32,
    frametimes: [f32; FRAMETIME_LEN],
}

#[derive(Clone)]
pub struct GpuFrametimeMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

// The implementation of [`Material`] needs this impl to work properly.
impl RenderAsset for FrametimeMaterial {
    type ExtractedAsset = FrametimeMaterial;
    type PreparedAsset = GpuFrametimeMaterial;
    type Param = (SRes<RenderDevice>, SRes<MaterialPipeline<Self>>);
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
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuFrametimeMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}

impl Material for FrametimeMaterial {
    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/frametime_display.wgsl"))
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn alpha_mode(_material: &<Self as RenderAsset>::PreparedAsset) -> AlphaMode {
        AlphaMode::Blend
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
