use std::ops::Range;

#[cfg(feature = "3d_render")]
use glam::{Mat4, Vec3};

use crate::window::component::base::area::{Area, AreaMath};
#[cfg(feature = "3d_render")]
use crate::window::component::block_3d::model::model::Model;

#[cfg(feature = "3d_render")]
use crate::window::wgpu::block_3d::brick_uniform::BakePushConstants;
#[cfg(feature = "3d_render")]
use crate::window::wgpu::block_3d::{camera_uniform::CameraUniform, instance::Instance3DData};

use crate::window::{
    component::{managers::atlas_manager::AtlasManager, theme::border::Border},
    wgpu::{
        draw_args::DrawIndexedIndirectArgs,
        shape_vertex::{SHAPE_LINE, SHAPE_RECT, SHAPE_TEXT, ShapeVertex},
    },
};

pub struct GpuRenderContext {
    pub shape_vertices: Vec<ShapeVertex>,
    pub command_sections: Vec<GpuCommand>,
    #[cfg(feature = "3d_render")]
    pub bake_cmds: Vec<BakePushConstants>,
    #[cfg(feature = "3d_render")]
    pub instances_3d: Vec<Instance3DData>,
    #[cfg(feature = "3d_render")]
    pub camera_data: CameraUniform,

    pub indirect_cmd: Vec<DrawIndexedIndirectArgs>,
}

pub enum GpuCommand {
    Shape(Section),
    Text(Section),
    Unmask(Section),
    Instance(Section),
}

pub struct Section {
    pub level: u32,
    pub command_index: u32,
    pub is_mask: bool,
}

impl GpuRenderContext {
    pub fn new() -> Self {
        GpuRenderContext {
            shape_vertices: Vec::with_capacity(1024),
            command_sections: Vec::with_capacity(1024),

            indirect_cmd: Vec::with_capacity(1024),
            #[cfg(feature = "3d_render")]
            camera_data: CameraUniform::default(),
            #[cfg(feature = "3d_render")]
            bake_cmds: Vec::with_capacity(1024),
            #[cfg(feature = "3d_render")]
            instances_3d: Vec::with_capacity(1024),
        }
    }

    pub fn push_text(
        &mut self,
        atlas: &mut AtlasManager,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: u32,
        level: u32,
    ) {
        let current_vert_count = self.shape_vertices.len() as u32;
        let first_index = (current_vert_count / 4) * 6;

        let mut cur_x = x;
        let scale = size / 64.0;
        let mut char_count = 0;

        let line_metrics = atlas.font.horizontal_line_metrics(64.0).unwrap();

        let baseline = y + line_metrics.ascent * scale;

        for c in text.chars() {
            if c == '\n' {
                cur_x = x;

                continue;
            }

            let glyph = atlas.get_glyph(c);

            let x0 = cur_x + glyph.x_offset * scale;
            let y0 = baseline - (glyph.y_offset + glyph.height) * scale;
            let x1 = x0 + glyph.width * scale;
            let y1 = y0 + glyph.height * scale;

            let pos = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
            let uvs = [
                [glyph.uv_min[0], glyph.uv_min[1]],
                [glyph.uv_max[0], glyph.uv_min[1]],
                [glyph.uv_max[0], glyph.uv_max[1]],
                [glyph.uv_min[0], glyph.uv_max[1]],
            ];

            // ВСЕГДА пушим 4 вершины, чтобы сохранить шаг для статических индексов
            for i in 0..4 {
                self.shape_vertices.push(ShapeVertex {
                    position: pos[i],
                    color: color_to_gpu(color),
                    p_a: uvs[i],                         // Шейдер использует p_a как UV
                    p_b: [0.0, 0.0],                     // p_b Не используется пока
                    params: [0.0, SHAPE_TEXT, 1.0, 0.0], // тип 2.0 — Текст
                    border_color: color_to_gpu(color),
                });
            }

            cur_x += glyph.advance * scale;
            char_count += 1;
        }

        if char_count == 0 {
            return;
        }

        let cmd_idx = self.indirect_cmd.len() as u32;
        self.indirect_cmd.push(DrawIndexedIndirectArgs {
            index_count: char_count * 6, // Ровно 6 индексов на каждый символ из цикла
            instance_count: 1,
            first_index,
            base_vertex: 0,
            first_instance: 0,
        });

        self.command_sections.push(GpuCommand::Text(Section {
            level,
            command_index: cmd_idx,
            is_mask: false,
        }));
    }

    pub fn push_shape(
        &mut self,
        min_p: [f32; 2],
        max_p: [f32; 2],
        p_a: [f32; 2],
        p_b: [f32; 2],
        color: u32,
        params: [f32; 4],
        border_color: u32,
        level: u32,
        is_clip: bool,
        un_mask: bool,
    ) {
        let current_vert_count = self.shape_vertices.len() as u32;

        // first_index теперь всегда четко привязан к позиции в статическом Uber-буфере
        let first_index = (current_vert_count / 4) * 6;

        let aa_padding = 2.0;
        let x0 = min_p[0] - aa_padding;
        let y0 = min_p[1] - aa_padding;
        let x1 = max_p[0] + aa_padding;
        let y1 = max_p[1] + aa_padding;

        // ЕДИНЫЙ ПОРЯДОК: TL, TR, BR, BL
        let corners = [
            [x0, y0], // 0: TL
            [x1, y0], // 1: TR
            [x1, y1], // 2: BR
            [x0, y1], // 3: BL
        ];

        let v = corners.map(|pos| ShapeVertex {
            position: pos,
            color: color_to_gpu(color),
            p_a,
            p_b,
            params,
            border_color: color_to_gpu(border_color),
        });

        self.shape_vertices.extend_from_slice(&v);

        let cmd_idx = self.indirect_cmd.len() as u32;
        self.indirect_cmd.push(DrawIndexedIndirectArgs {
            index_count: 6,
            instance_count: 1,
            first_index,    // Ссылка на статический индексный буфер
            base_vertex: 0, // ОБЯЗАТЕЛЬНО 0, так как first_index уже абсолютный
            first_instance: 0,
        });

        let cmd = Section {
            level,
            command_index: cmd_idx,
            is_mask: is_clip,
        };

        if un_mask {
            self.command_sections.push(GpuCommand::Unmask(cmd));
        } else {
            self.command_sections.push(GpuCommand::Shape(cmd));
        }
    }

    pub fn push_rect_sdf(
        &mut self,
        rect: &Area,
        color: u32,
        border: &Border,
        level: u32,
        is_clip: bool,
        un_mask: bool,
    ) {
        let x1 = rect.x1;
        let y1 = rect.y1;
        let x2 = rect.get_x2();
        let y2 = rect.get_y2();

        // Параметры для SDF шейдера
        let width = x2 - x1;
        let height = y2 - y1;
        let center = [x1 + width * 0.5, y1 + height * 0.5];
        let size = [width, height];

        self.push_shape(
            [x1, y1],
            [x2, y2],
            center,
            size,
            color, // color_rgba,
            [border.radius, SHAPE_RECT, 1.0, border.width],
            border.color,
            level,
            is_clip,
            un_mask,
        );
    }

    // Рисует линию графика
    pub fn push_line(
        &mut self,
        start_p: [f32; 2],
        end_p: [f32; 2],
        thickness: f32,
        color: u32,
        border: &Border,
        level: u32,
        is_clip: bool,
    ) {
        let pad = thickness + 2.0;

        let x1 = start_p[0].min(end_p[0]) - pad;
        let y1 = start_p[1].min(end_p[1]) - pad;
        let x2 = start_p[0].max(end_p[0]) + pad;
        let y2 = start_p[1].max(end_p[1]) + pad;

        // params: [половина толщины, тип: 1.0 (LINE), сглаживание: 1.0, 0.0]
        self.push_shape(
            [x1, y1], // min_p
            [x2, y2], // max_p
            start_p,  // p_a
            end_p,    // p_b
            color,    // color_rgba,
            [thickness * 0.5, SHAPE_LINE, 1.0, border.width],
            border.color,
            level,
            is_clip,
            false,
        );
    }

    pub fn clear(&mut self) {
        self.shape_vertices.clear();
        self.command_sections.clear();
        #[cfg(feature = "3d_render")]
        self.bake_cmds.clear();
        #[cfg(feature = "3d_render")]
        self.instances_3d.clear();
        self.indirect_cmd.clear();
    }

    // #[cfg(feature = "3d_render")]
    // pub fn push_3d_viewport(&mut self, rect: &Area, models: &[Model], level: u32) {
    //     // Запоминаем индекс первого инстанса для этой группы моделей
    //     let first_instance = self.instances_3d.len() as u32;

    //     // Наполняем буфер данных моделей (инстансов)
    //     for (indx, model) in models.iter().enumerate() {
    //         let mut params = model.params;
    //         if indx == 0 {
    //             params[3] = models.len() as f32; // Передаем кол-во моделей в первом инстансе
    //         }
    //         self.instances_3d.push(Instance3DData {
    //             inv_transform: model.transform.to_inv_matrix(),
    //             color: color_to_gpu(model.color),
    //             params,
    //             entity_id: model.id_model.unwrap_or(0),
    //             _padding: [0; 2],
    //         });
    //     }

    //     let base_vertex = self.shape_vertices.len() as u32;
    //     let first_index = (base_vertex / 4) * 6;

    //     // TL, TR, BR, BL — используем тот же порядок, что в push_shape
    //     let corners = [
    //         [rect.x1, rect.y1],
    //         [rect.get_x2(), rect.y1],
    //         [rect.get_x2(), rect.get_y2()],
    //         [rect.x1, rect.get_y2()],
    //     ];

    //     for pos in corners {
    //         self.shape_vertices.push(ShapeVertex {
    //             position: pos,
    //             color: 0,
    //             p_a: [rect.x1, rect.y1], // Можно использовать как координаты клиппинга
    //             p_b: [rect.get_x2(), rect.get_y2()],
    //             params: [first_instance as f32, 0.0, 0.0, 0.0],
    //             border_color: 0,
    //         });
    //     }

    //     // Добавляем команду в ЕДИНЫЙ indirect_cmd буфер
    //     let cmd_idx = self.indirect_cmd.len() as u32;
    //     self.indirect_cmd.push(DrawIndexedIndirectArgs {
    //         index_count: 6,
    //         instance_count: models.len() as u32,
    //         first_index,
    //         base_vertex: 0,
    //         first_instance,
    //     });

    //     self.command_sections.push(GpuCommand::Instance(Section {
    //         level,
    //         command_index: cmd_idx,
    //         is_mask: false,
    //     }));
    // }

    // #[cfg(feature = "3d_render")]
    // pub fn push_model_instance(&mut self, model: &Model, rect: &Area, level: u32) {
    //     // Запоминаем текущий индекс в буфере инстансов
    //     let first_instance = self.instances_3d.len() as u32;

    //     let mut params = model.params;
    //     params[3] = 1.0;
    //     // Пушим данные трансформации (поворот, позиция, масштаб)
    //     self.instances_3d.push(Instance3DData {
    //         inv_transform: model.transform.to_inv_matrix(),
    //         color: color_to_gpu(model.color),
    //         params: params, // Здесь лежит tag (сфера/куб) и размеры
    //         entity_id: model.id_model.unwrap_or(0),
    //         _padding: [0; 2],
    //     });

    //     // Подготавливаем вершины "тесного" квада
    //     let base_vertex = self.shape_vertices.len() as u32;
    //     let first_index = (base_vertex / 4) * 6;

    //     let corners = [
    //         [rect.x1, rect.y1],
    //         [rect.get_x2(), rect.y1],
    //         [rect.get_x2(), rect.get_y2()],
    //         [rect.x1, rect.get_y2()],
    //     ];

    //     for pos in corners {
    //         self.shape_vertices.push(ShapeVertex {
    //             position: pos,
    //             color: 0,
    //             p_a: [rect.x1, rect.y1], // Viewport Min для шейдера
    //             p_b: [rect.get_x2(), rect.get_y2()],
    //             params: [first_instance as f32, 0.0, 0.0, 0.0], // Тип 3.0
    //             border_color: 0,
    //         });
    //     }

    //     // Команда отрисовки: 1 инстанс, но со смещением first_instance
    //     let cmd_idx = self.indirect_cmd.len() as u32;
    //     self.indirect_cmd.push(DrawIndexedIndirectArgs {
    //         index_count: 6,
    //         instance_count: 1,
    //         first_index,
    //         base_vertex: 0,
    //         first_instance: first_instance, // <--- Указываем на данные в Storage Buffer
    //     });

    //     self.command_sections.push(GpuCommand::Instance(Section {
    //         level,
    //         command_index: cmd_idx,
    //         is_mask: false,
    //     }));
    // }

    #[cfg(feature = "3d_render")]
    pub fn push_bake_commands(&mut self, rect: &Area, model: &mut Model, level: u32) {
        // Наполняем буфер данных моделей (инстансов)
        let (bake_cmds, instance_cmds) = model.render();
        self.bake_cmds = bake_cmds;
        self.instances_3d = instance_cmds;

        let base_vertex = self.shape_vertices.len() as u32;
        let first_index = (base_vertex / 4) * 6;

        // TL, TR, BR, BL — используем тот же порядок, что в push_shape
        let corners = [
            [rect.x1, rect.y1],
            [rect.get_x2(), rect.y1],
            [rect.get_x2(), rect.get_y2()],
            [rect.x1, rect.get_y2()],
        ];

        for pos in corners {
            self.shape_vertices.push(ShapeVertex {
                position: pos,
                color: 0,
                p_a: [rect.x1, rect.y1],
                p_b: [rect.get_x2(), rect.get_y2()],
                params: [0.0, 0.0, 0.0, 0.0],
                border_color: 0,
            });
        }

        // Добавляем команду в ЕДИНЫЙ indirect_cmd буфер
        let cmd_idx = self.indirect_cmd.len() as u32;
        self.indirect_cmd.push(DrawIndexedIndirectArgs {
            index_count: 6,
            instance_count: 1,
            first_index,
            base_vertex: 0,
            first_instance: 0,
        });

        self.command_sections.push(GpuCommand::Instance(Section {
            level,
            command_index: cmd_idx,
            is_mask: false,
        }));
    }

    // pub fn push_voxel_instance(
    //     &mut self,
    //     model: &Model,
    //     rect: &Area,
    //     level: u32,
    //     brick_id: u32,
    //     brick_manager: &mut BrickManager,
    // ) {
    //     let first_instance = self.instances_3d.len() as u32;

    //     // Получаем UV координаты блока из менеджера атласа
    //     let (uv_min, uv_max) = brick_manager.get_uv_range(brick_id);

    //     // Матрица трансформации
    //     // Важно: SDF воксели работают в локальном кубе 0..16.
    //     // Нам нужна матрица, которая превращает мир в эти 0..16.
    //     let inv_transform = model.transform.to_inv_matrix_voxel();

    //     self.instances_3d.push(Instance3DData {
    //         inv_transform,
    //         color: color_to_gpu(model.color),
    //         params: [6.0, brick_id as f32, 0.5, 1.0], // 6.0 - тег вокселя, brick_id, k-сглаживание
    //         entity_id: model.id_model,
    //         _padding: [0; 2],
    //     });

    //     // Вершины квада (используем p_a и p_b для передачи UV атласа)
    //     let base_vertex = self.shape_vertices.len() as u32;
    //     let first_index = (base_vertex / 4) * 6;

    //     let corners = [
    //         [rect.x1, rect.y1],
    //         [rect.get_x2(), rect.y1],
    //         [rect.get_x2(), rect.get_y2()],
    //         [rect.x1, rect.get_y2()],
    //     ];

    //     for pos in corners {
    //         self.shape_vertices.push(ShapeVertex {
    //             position: pos,
    //             color: 0,
    //             p_a: uv_min, // ТЕПЕРЬ ЭТО UV СТАРТ БЛОКА В АТЛАСЕ
    //             p_b: uv_max, // ТЕПЕРЬ ЭТО UV КОНЕЦ БЛОКА В АТЛАСЕ
    //             params: [0.0, 6.0, 0.0, first_instance as f32], // тип 6.0 и ссылка на данные инстанса
    //             border_color: 0,
    //         });
    //     }

    //     let cmd_idx = self.indirect_cmd.len() as u32;
    //     self.indirect_cmd.push(DrawIndexedIndirectArgs {
    //         index_count: 6,
    //         instance_count: 1,
    //         first_index,
    //         base_vertex: 0,
    //         first_instance: 0, // Мы используем индексацию через params.w в шейдере
    //     });

    //     self.command_sections.push(GpuCommand::Instance(Section {
    //         level,
    //         command_index: cmd_idx,
    //         is_mask: false,
    //     }));
    // }
}

pub fn color_to_gpu(color: u32) -> u32 {
    let a = (color >> 24) & 0xFF;
    let r = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = color & 0xFF;

    // Собираем в RGBA (порядок байтов для Unorm8x4 в wgpu)
    (a << 24) | (b << 16) | (g << 8) | r
}
