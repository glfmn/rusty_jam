use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::{MaterialPipeline, SpecializedMaterial},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_asset::{PrepareAssetError, RenderAsset, RenderAssets},
        render_resource::*,
        renderer::RenderDevice,
    },
};

/// Setup custom materials
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        use bevy::render::{RenderApp, RenderStage};

        app.add_plugin(MaterialPlugin::<UnlitMaterial>::default())
            .init_resource::<DefaultTexture>();

        app.sub_app_mut(RenderApp)
            .add_system_to_stage(RenderStage::Extract, extract_default_texture);
    }
}

/// Fallback texture
#[derive(Clone)]
pub struct DefaultTexture {
    handle: Handle<Image>,
}

impl FromWorld for DefaultTexture {
    fn from_world(world: &mut World) -> Self {
        Self {
            handle: world
                .resource_mut::<AssetServer>()
                .load("textures/default_texture.png"),
        }
    }
}

fn extract_default_texture(
    texture: Res<DefaultTexture>,
    mut commands: Commands,
) {
    commands.insert_resource(texture.clone())
}

pub type UnlitMaterialBundle = MaterialMeshBundle<UnlitMaterial>;

/// Render flat material
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "f1aacff7-3eea-4a71-836a-efbcb11fe870"]
pub struct UnlitMaterial {
    texture: Option<Handle<Image>>,
}

impl UnlitMaterial {
    pub fn new(texture: Handle<Image>) -> Self {
        Self {
            texture: Some(texture),
        }
    }
}

impl Default for UnlitMaterial {
    fn default() -> Self {
        Self { texture: None }
    }
}

/// GPU representation of `[UnlitMaterial]`
#[derive(Clone)]
pub struct GpuUnlitMaterial {
    bind_group: BindGroup,
}

impl RenderAsset for UnlitMaterial {
    type ExtractedAsset = UnlitMaterial;
    type PreparedAsset = GpuUnlitMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<RenderAssets<Image>>,
        SRes<DefaultTexture>,
        SRes<MaterialPipeline<Self>>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        asset: Self::ExtractedAsset,
        (device, gpu_images, default_texture, pipeline): &mut SystemParamItem<
            Self::Param,
        >,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>>
    {
        let texture = match gpu_images.get(
            &asset
                .texture
                .clone()
                .unwrap_or(default_texture.handle.clone()),
        ) {
            Some(texture) => texture,
            // Try again if the image isn't loaded
            None => {
                debug!("Texture ({:?}) not yet loaded", asset.texture);
                return Err(PrepareAssetError::RetryNextUpdate(asset));
            }
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &texture.texture_view,
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("Unlit Texture Material Bind Group Layout"),
            layout: &pipeline.material_layout,
        });

        Ok(GpuUnlitMaterial { bind_group })
    }
}

impl SpecializedMaterial for UnlitMaterial {
    type Key = ();

    fn key(_: &<UnlitMaterial as RenderAsset>::PreparedAsset) -> Self::Key {}

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _: Self::Key,
        _layout: &MeshVertexBufferLayout,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();
        Ok(())
    }

    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/unlit_material.vert"))
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/unlit_material.frag"))
    }

    fn bind_group(
        render_asset: &<Self as RenderAsset>::PreparedAsset,
    ) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float {
                            filterable: false,
                        },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
            label: Some("Unlit Material Bind Group"),
        })
    }
}
