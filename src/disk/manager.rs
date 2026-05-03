use std::io::{Read, Write};

use rustc_hash::FxHashMap;

use crate::window::component::block_3d::model::sdf_command::SDFCommandRaw;

pub struct DiskManager {
    file: std::fs::File,
    // Карта: ID_модели -> (смещение_в_файле, количество_команд)
    index: FxHashMap<u32, (u64, u32)>,
    next_offset: u64,
}

impl DiskManager {
    pub fn save_model(&mut self, id: u32, commands: &[SDFCommandRaw]) {
        let offset = self.next_offset;
        let bytes: &[u8] = bytemuck::cast_slice(commands);

        // Пишем в конец файла (или ищем свободное место)
        self.file.write_all(bytes).unwrap();

        self.index.insert(id, (offset, commands.len() as u32));
        self.next_offset += bytes.len() as u64;
    }

    pub fn load_model_to_gpu(
        &mut self,
        id: u32,
        queue: &wgpu::Queue,
        buffer: &wgpu::Buffer,
        buffer_offset: u64,
    ) {
        if let Some((file_offset, count)) = self.index.get(&id) {
            let mut data = vec![0u8; (*count as usize) * 32];
            self.file.read_exact(&mut data).unwrap();

            // Заливаем напрямую в Storage Buffer видеокарты
            queue.write_buffer(buffer, buffer_offset, &data);
        }
    }
}
