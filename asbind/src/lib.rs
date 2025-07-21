pub trait Memory {
    fn allocate(&mut self, size: i32) -> i32;
    fn write(&mut self, ptr: i32, data: &[u8]);
}

pub trait WhatToWrite {
    /// write the contents
    fn write(&self, target: &mut impl Memory, ptr: i32);
    /// size of the target payload
    fn size(&self) -> i32;
}

impl WhatToWrite for i32 {
    fn write(&self, memory: &mut impl Memory, ptr: i32) {
        memory.write(ptr, &self.to_le_bytes())
    }

    fn size(&self) -> i32 {
        4
    }
}

impl WhatToWrite for i64 {
    fn write(&self, memory: &mut impl Memory, ptr: i32) {
        memory.write(ptr, &self.to_le_bytes())
    }

    fn size(&self) -> i32 {
        8
    }
}

impl WhatToWrite for String {
    fn write(&self, memory: &mut impl Memory, ptr: i32) {
        let input_utf16: Vec<u16> = self.encode_utf16().collect();
        memory.write(ptr, bytemuck::cast_slice(&input_utf16));
    }

    fn size(&self) -> i32 {
        let input_utf16: Vec<u16> = self.encode_utf16().collect();
        (input_utf16.len() * 2) as i32
    }
}
