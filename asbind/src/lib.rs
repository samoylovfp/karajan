pub trait AllocateAndWrite {
    fn allocate(&mut self, size: i32) -> i32;
    fn write(&mut self, ptr: i32, data: &[u8]);
}

pub trait WhatToWrite {
    fn write(&self, target: &mut impl AllocateAndWrite, ptr: i32);
    /// FIXME: not on stack, but in current contiguous memory fragment
    fn size_on_stack(&self) -> i32;
    fn size_on_heap(&self) -> Option<i32>;
}

impl WhatToWrite for i32 {
    fn write(&self, target: &mut impl AllocateAndWrite, ptr: i32) {
        target.write(ptr, &self.to_le_bytes())
    }

    fn size_on_stack(&self) -> i32 {
        4
    }

    fn size_on_heap(&self) -> Option<i32> {
        None
    }
}

impl WhatToWrite for i64 {
    fn write(&self, target: &mut impl AllocateAndWrite, ptr: i32) {
        target.write(ptr, &self.to_le_bytes())
    }

    fn size_on_stack(&self) -> i32 {
        8
    }

    fn size_on_heap(&self) -> Option<i32> {
        None
    }
}

impl WhatToWrite for &str {
    fn write(&self, target: &mut impl AllocateAndWrite, ptr: i32) {
        let input_utf16: Vec<u16> = self.encode_utf16().collect();
        let input_size_bytes = (input_utf16.len() * 2) as i32;

        let str_ptr = target.allocate(input_size_bytes);
        target.write(str_ptr, bytemuck::cast_slice(&input_utf16));
        target.write(ptr, &str_ptr.to_le_bytes());
    }

    fn size_on_stack(&self) -> i32 {
        let input_utf16: Vec<u16> = self.encode_utf16().collect();
        (input_utf16.len() * 2) as i32
    }

    fn size_on_heap(&self) -> Option<i32> {
        Some(self.bytes().count() as i32 * 2)
    }
}
