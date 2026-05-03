use std::time::Instant;

pub struct GPUTextVertex {
    pub text_bind_group_layout: wgpu::BindGroupLayout,
    pub text_bind_group: wgpu::BindGroup,
    pub last_defrag_time: Instant,
    pub atlas: AtlasManager,
}

use wgpu::Device;

use crate::window::component::managers::atlas_manager::AtlasManager;

impl GPUTextVertex {
    pub fn new(device: &Device) -> Self {
        let font = include_bytes!("../component/base/Fonts/Roboto-Black.ttf");

        let atlas_manager = AtlasManager::new(&device, font, 2048);

        let glyph_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let text_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Текстура атласа (@binding(0))
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // Самплер (@binding(1))
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("text_bind_group_layout"),
            });

        let text_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &text_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_manager.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&glyph_sampler),
                },
            ],
            label: Some("Text Bind Group"),
        });

        Self {
            atlas: atlas_manager,
            //section_hashes: Vec::with_capacity(1024),
            text_bind_group_layout: text_bind_group_layout,
            text_bind_group: text_bind_group,
            last_defrag_time: Instant::now(),
            //temp_verts: Vec::with_capacity(1024),
            //last_base_idx: 0,
        }
    }
}
