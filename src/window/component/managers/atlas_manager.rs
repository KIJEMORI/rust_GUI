use std::collections::HashMap;

use fontdue::{Font, FontSettings};
use wgpu::{Device, Extent3d, Texture, TextureDescriptor, TextureUsages};

use crate::window::component::glyph::glyph_info::GlyphInfo;

pub struct AtlasManager {
    pub texture: Texture,
    pub view: wgpu::TextureView,
    pub glyphs: HashMap<char, GlyphInfo>,
    pub font: Font,

    // Состояние упаковщика
    current_x: u32,
    current_y: u32,
    row_height: u32,
    atlas_size: u32,
    padding: u32,

    pending_metadata: Vec<PendingGlyphInfo>,
    pending_pixels: Vec<u8>,
}

struct PendingGlyphInfo {
    origin: [u32; 2],
    size: [u32; 2],
}

impl AtlasManager {
    pub fn new(device: &Device, font_data: &[u8], size: u32) -> Self {
        let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("SDF Atlas"),
            size: Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            glyphs: HashMap::new(),
            font,
            current_x: 2, // Небольшой отступ от края
            current_y: 2,
            row_height: 0,
            atlas_size: size,
            padding: 4,
            pending_metadata: Vec::new(),
            pending_pixels: Vec::new(),
        }
    }

    #[inline]
    pub fn get_glyph(&mut self, c: char) -> &GlyphInfo {
        if self.glyphs.contains_key(&c) {
            return self.glyphs.get(&c).unwrap();
        }

        let (metrics, pixels) = self.font.rasterize(c, 64.0);

        self.add_to_pending(c, metrics, pixels)
    }

    fn add_to_pending(
        &mut self,
        c: char,
        metrics: fontdue::Metrics,
        pixels: Vec<u8>,
    ) -> &GlyphInfo {
        // Расчет упаковки (остается на CPU)
        if self.current_x + metrics.width as u32 > self.atlas_size {
            self.current_y += self.row_height + self.padding;
            self.current_x = 2;
            self.row_height = 0;
        }

        let inv_size = 1.0 / self.atlas_size as f32;
        let info = GlyphInfo {
            uv_min: [
                self.current_x as f32 * inv_size,
                self.current_y as f32 * inv_size,
            ],
            uv_max: [
                (self.current_x + metrics.width as u32) as f32 * inv_size,
                (self.current_y + metrics.height as u32) as f32 * inv_size,
            ],
            width: metrics.width as f32,
            height: metrics.height as f32,
            x_offset: metrics.xmin as f32,
            y_offset: metrics.ymin as f32,
            advance: metrics.advance_width,
        };

        if metrics.width > 0 && metrics.height > 0 {
            // Добавляем в пакет на загрузку
            self.pending_metadata.push(PendingGlyphInfo {
                origin: [self.current_x, self.current_y],
                size: [metrics.width as u32, metrics.height as u32],
            });
            self.pending_pixels.extend_from_slice(&pixels);
        }

        self.row_height = self.row_height.max(metrics.height as u32);
        self.current_x += metrics.width as u32 + self.padding;

        self.glyphs.insert(c, info);
        self.glyphs.get(&c).unwrap()
    }

    /// Вызывать ОДИН раз перед рендерингом
    pub fn update_atlas(&mut self, queue: &wgpu::Queue) {
        if self.pending_metadata.is_empty() {
            return;
        }

        let mut offset = 0;
        // Используем обычный итератор, drain сделаем в конце
        for meta in &self.pending_metadata {
            let data_size = (meta.size[0] * meta.size[1]) as usize;

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: meta.origin[0],
                        y: meta.origin[1],
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &self.pending_pixels[offset..offset + data_size],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(meta.size[0]), // Для R8Unorm это просто ширина
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: meta.size[0],
                    height: meta.size[1],
                    depth_or_array_layers: 1,
                },
            );
            offset += data_size;
        }

        // Очищаем всё ПОСЛЕ того как всё отправили
        self.pending_metadata.clear();
        self.pending_pixels.clear();
    }
}
