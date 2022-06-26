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
        app.add_plugin(MaterialPlugin::<UnlitMaterial>::default());
    }
}

pub type UnlitMaterialBundle = MaterialMeshBundle<UnlitMaterial>;

/// Render flat material
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "f1aacff7-3eea-4a71-836a-efbcb11fe870"]
pub struct UnlitMaterial {
    /// Handle to the full sprite sheet
    pub sprite_sheet: Handle<Image>,
    /// Specific sprite in the sprite sheet
    pub sprite: Rect<f32>,
}

impl UnlitMaterial {
    /// Render the entire sprite sheet
    #[allow(unused)]
    pub const FULL_SHEET: Rect<f32> = Rect {
        // WGPU/Bevy uses (0,0) as top left, (1,1) as bottom right
        top: 0.,
        left: 0.,
        right: 1.,
        bottom: 1.,
    };

    /// Create a new unlit material
    pub fn new(sprite_sheet: Handle<Image>, sprite: Rect<f32>) -> Self {
        Self {
            sprite_sheet,
            sprite,
        }
    }

    /// Render the entire texture, unaltered
    #[allow(unused)]
    pub fn full_sheet(sprite_sheet: Handle<Image>) -> Self {
        Self {
            sprite_sheet,
            sprite: Self::FULL_SHEET,
        }
    }
}

/// GPU representation of `[UnlitMaterial]`
#[derive(Clone)]
pub struct GpuUnlitMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

impl RenderAsset for UnlitMaterial {
    type ExtractedAsset = UnlitMaterial;
    type PreparedAsset = GpuUnlitMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<RenderAssets<Image>>,
        SRes<MaterialPipeline<Self>>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        asset: Self::ExtractedAsset,
        (device, gpu_images, pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>>
    {
        let texture = gpu_images
            .get(&asset.sprite_sheet.clone())
            .ok_or_else(|| PrepareAssetError::RetryNextUpdate(asset.clone()))?;

        // Pack UV min and UV max into a vec4 where min: (x,y) max: (z,w)
        // Uniform data padding requirements are pretty strict, this lets
        // us save some memory and simplifies our buffer creation code a bit.
        //
        // UV coordinate system in bevy uses (0,0) as the top left and (1,1) as
        // the bottom right coordinate.
        let data = Vec4::new(
            asset.sprite.left,
            asset.sprite.top,
            asset.sprite.right,
            asset.sprite.bottom,
        );

        // Traits to convert data to uniform buffer memory layout (Std140)
        use bevy::render::render_resource::std140::{AsStd140, Std140};

        let buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Sprite UV Offset"),
            contents: data.as_std140().as_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

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
                BindGroupEntry {
                    binding: 2,
                    resource: buffer.as_entire_binding(),
                },
            ],
            label: Some("Unlit Texture Material Bind Group Layout"),
            layout: &pipeline.material_layout,
        });

        Ok(GpuUnlitMaterial {
            _buffer: buffer,
            bind_group,
        })
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
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Unlit Material Bind Group"),
        })
    }
}
