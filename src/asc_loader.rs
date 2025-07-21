//! Wrapper for working with assemblyscript wasm modules

use anyhow::{anyhow, bail};
use asbind::{Memory, WhatToWrite};
use wasmtime::*;

type Ptr = i32;
pub struct AscModule {
    module: Module,
    memory: wasmtime::Memory,
    instance: Instance,
    store: Store<ModuleData>,
    /// (length, type) -> ptr
    alloc_func: TypedFunc<(i32, i32), Ptr>,
    pin_func: TypedFunc<Ptr, Ptr>,
    unpin_func: TypedFunc<Ptr, ()>,
    // FIXME:
    process_update_func: TypedFunc<Ptr, ()>,
}

/// name of the wasm export of main memory
const MEMORY_NAME: &str = "memory";

// a structure to pass data to callbacks
struct ModuleData {}

impl Memory for AscModule {
    fn allocate(&mut self, size: i32) -> i32 {
        let ptr = self.alloc_func.call(&mut self.store, (size, 0)).unwrap();
        self.pin_func.call(&mut self.store, ptr).unwrap()
    }

    fn write(&mut self, ptr: i32, data: &[u8]) {
        self.memory
            .write(&mut self.store, ptr as usize, data)
            .unwrap()
    }
}

impl AscModule {
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<AscModule> {
        let engine = Engine::default();
        let module = Module::new(&engine, bytes)?;
        let memory = module
            .get_export_index(MEMORY_NAME)
            .ok_or(anyhow!("No memory in the module"))?;

        let mut linker = Linker::new(&engine);

        let (err_sender, err_receiver) = tokio::sync::mpsc::channel(128);

        // TODO: send error to the author
        drop(err_receiver);

        // Assembly script requires the "abort" function
        // we provide it via the linker
        linker.func_wrap(
            "env",
            "abort",
            move |caller: Caller<'_, ModuleData>,
                  message_ptr: i32,
                  filename_ptr: i32,
                  line: i32,
                  col: i32| {
                let err = generate_err(caller, memory, message_ptr, filename_ptr, line, col);
                if let Err(e) = err_sender.try_send(dbg!(err)) {
                    tracing::error!(?e, "sending error to author pipe");
                }
            },
        )?;

        linker.func_wrap(
            "host",
            "sendMessage",
            move |caller: Caller<'_, ModuleData>, chat_id: i64, message: i32| -> () {
                let message = get_str_from_caller(caller, memory, message).unwrap();
                println!("Send message called with {chat_id:?}: {message}")
            },
        )?;

        let mut store: Store<ModuleData> = Store::new(&engine, ModuleData {});

        // Instantiation of a module requires specifying its imports and then
        // afterwards we can fetch exports by name, as well as asserting the
        // type signature of the function with `get_typed_func`.
        let instance = linker.instantiate(&mut store, &module)?;

        // Get the guest's allocation function.
        let alloc_func = instance.get_typed_func(&mut store, "__new")?;
        // FIXME: memory is extracted in two places
        let memory = instance.get_memory(&mut store, MEMORY_NAME).unwrap();

        // read_type_info(memory, &instance, &mut store)?;

        Ok(AscModule {
            memory,
            module,
            alloc_func,
            pin_func: instance.get_typed_func(&mut store, "__pin")?,
            unpin_func: instance.get_typed_func(&mut store, "__unpin")?,
            instance,
            process_update_func: instance
                .get_typed_func(&mut store, "processUpdate")
                .unwrap(),
            store,
        })
    }

    pub fn call_process_updates(&mut self, update: String) -> anyhow::Result<()> {
        let ptr = self
            .alloc_func
            .call(&mut self.store, (update.size_on_heap().unwrap(), 0))?;
        let ptr = self.pin_func.call(&mut self.store, ptr).unwrap();
        update.write(self, ptr);

        self.process_update_func.call(&mut self.store, ptr).unwrap();
        // // Encode the input string as UTF-16, which AssemblyScript expects.

        // let input_utf16: Vec<u16> = update.encode_utf16().collect();
        // let input_size_bytes = (input_utf16.len() * 2) as i32;

        // // Allocate memory in the guest for the input string.

        // let input_ptr = self
        //     .alloc_func
        //     .call(&mut self.store, (input_size_bytes, 0))?;

        // let input_ptr_pinned = self.pin_func.call(&mut self.store, (input_ptr))?;

        // // Write the UTF-16 bytes into the guest's memory.
        // self.memory.write(
        //     &mut self.store,
        //     input_ptr_pinned as usize,
        //     bytemuck::cast_slice(&input_utf16),
        // )?;

        // update.write(&mut self, ptr);

        // // do the call
        // let result = self.process_update_func.call(&mut self.store, input_ptr)?;
        // self.unpin_func.call(&mut self.store, input_ptr_pinned)?;

        Ok(())

        // Ok(read_asc_string(&self.memory, &mut self.store, result))
    }
}

// fn read_type_info(
//     memory: Memory,
//     instance: &Instance,
//     store: &mut Store<ModuleData>,
// ) -> anyhow::Result<()> {
//     let rtti = instance
//         .get_global(&mut *store, "__rtti_base")
//         .ok_or(anyhow!("No rtti base"))?;
//     let rtti = match rtti.get(&mut *store) {
//         Val::I32(rtti) => rtti,
//         _ => bail!("Rtti is not i32"),
//     };
//     let count = read_i32(&memory, &mut *store, rtti);

//     for offset in 1..=count {
//         println!("rtti {}", offset - 1);
//         let data = read_i32(&memory, &mut *store, rtti + offset * 4);
//         println!("{data::>32b}");
//         for flag in enum_iterator::all::<RttiFlags>() {
//             let flag_i32 = flag as i32;
//             if (flag_i32 & data) != 0 {
//                 println!("{flag:?}")
//             }
//         }
//     }

//     Ok(())
// }

fn read_i32(memory: &wasmtime::Memory, context: impl AsContext, ptr: i32) -> i32 {
    let mut val_buf = [0; 4];
    memory.read(&context, ptr as usize, &mut val_buf).unwrap();
    i32::from_le_bytes(val_buf)
}

fn read_asc_string(memory: &wasmtime::Memory, mut context: impl AsContext, ptr: i32) -> String {
    let length = read_i32(memory, &mut context, ptr - 4);

    let mut result_buf = vec![0; (length) as usize];
    memory
        .read(&mut context, ptr as usize, &mut result_buf)
        .unwrap();

    let result_u16 = result_buf
        .chunks(2)
        .map(|w| u16::from_le_bytes(w.try_into().unwrap()))
        .collect::<Vec<_>>();
    return String::from_utf16(&result_u16).unwrap();
}

fn get_str_from_caller(
    mut caller: Caller<'_, ModuleData>,
    memory: ModuleExport,
    message_ptr: i32,
) -> anyhow::Result<String> {
    let mem = caller.get_module_export(&memory).unwrap();
    let mem = match mem {
        Extern::Memory(mem) => mem,
        o => {
            bail!("Expected to have a memory under 'memory' name, got {o:?}");
        }
    };
    Ok(read_asc_string(&mem, &mut caller, message_ptr))
}

fn generate_err(
    mut caller: Caller<'_, ModuleData>,
    memory: ModuleExport,
    message_ptr: i32,
    filename_ptr: i32,
    line: i32,
    col: i32,
) -> anyhow::Error {
    let mem = caller.get_module_export(&memory).unwrap();
    let mem = match mem {
        Extern::Memory(mem) => mem,
        o => {
            return anyhow!("Expected to have a memory under 'memory' name, got {o:?}");
        }
    };

    let msg = read_asc_string(&mem, &mut caller, message_ptr);
    let filename = read_asc_string(&mem, &mut caller, filename_ptr);

    anyhow!("Got error {msg:?} at {filename}:{line}:{col}")
}
