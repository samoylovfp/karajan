//! Wrapper for working with assemblyscript wasm modules

use anyhow::{anyhow, bail};
use asbind::{Memory, WhatToWrite};
use wasmtime::*;

use crate::tg::send_msg;

type Ptr = i32;

#[expect(dead_code)]
pub struct AscModule {
    module: Module,
    memory: wasmtime::Memory,
    instance: Instance,
    store: Store<ModuleData>,
    /// (length, type) -> ptr
    alloc_func: TypedFunc<(i32, i32), Ptr>,
    pin_func: TypedFunc<Ptr, Ptr>,
    unpin_func: TypedFunc<Ptr, ()>,
    process_update_func: TypedFunc<Ptr, ()>,
}

/// name of the wasm export of main memory
const MEMORY_NAME: &str = "memory";

// a structure to pass data to callbacks
struct ModuleData {
    limits: StoreLimits,
}

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
    pub async fn from_bytes(bytes: &[u8]) -> anyhow::Result<AscModule> {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Engine::new(&config)?;
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

        linker.func_wrap_async(
            "host",
            "sendMessage",
            move |caller: Caller<'_, ModuleData>, (chat_id, msg_ptr): (i64, i32)| {
                let message = read_asc_string_from_caller(caller, memory, msg_ptr).unwrap();
                Box::new(async move {
                    // FIXME: how to separate out provided functions?
                    let tg_key = std::env::var("TG_KEY").unwrap();
                    send_msg(&reqwest::Client::new(), &tg_key, chat_id, message)
                        .await
                        .inspect_err(|e| tracing::error!(?e, "sending response"))
                })
            },
        )?;

        let mut store: Store<ModuleData> = Store::new(
            &engine,
            ModuleData {
                limits: StoreLimitsBuilder::new()
                    .memory_size(1 << 20 /* 1 MB */)
                    .instances(1)
                    .build(),
            },
        );
        store.limiter(|data| &mut data.limits);

        // Instantiation of a module requires specifying its imports and then
        // afterwards we can fetch exports by name, as well as asserting the
        // type signature of the function with `get_typed_func`.
        let instance = linker.instantiate_async(&mut store, &module).await?;

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

    pub async fn call_process_updates(&mut self, update: String) -> anyhow::Result<()> {
        let ptr = self
            .alloc_func
            .call_async(&mut self.store, (update.size(), 0))
            .await?;
        let ptr = self.pin_func.call_async(&mut self.store, ptr).await?;
        update.write(self, ptr);

        let res = self
            .process_update_func
            .call_async(&mut self.store, ptr)
            .await;
        self.unpin_func.call_async(&mut self.store, ptr).await?;
        res
    }
}

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

fn read_asc_string_from_caller(
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
