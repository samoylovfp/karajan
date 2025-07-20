use anyhow::anyhow;
use wasmtime::*;

pub struct AscModule {
    memory: Memory,
    store: Store<ModuleData>,
    /// (length, type) -> ptr
    alloc_func: TypedFunc<(i32, i32), i32>,
    // FIXME:
    process_update_func: TypedFunc<i32, i32>,
}

/// name of the wasm export of main memory
const MEMORY_NAME: &str = "memory";

// a structure to pass data to callbacks
struct ModuleData {}

impl AscModule {
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<AscModule> {
        let engine = Engine::default();
        let module = Module::new(&engine, bytes)?;
        let memory = module
            .get_export_index(MEMORY_NAME)
            .expect("module should export memory");

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
                if let Err(e) = err_sender.try_send(err) {
                    tracing::error!(?e, "sending error to author pipe");
                }
            },
        )?;

        let mut store: Store<ModuleData> = Store::new(&engine, ModuleData {});

        // Instantiation of a module requires specifying its imports and then
        // afterwards we can fetch exports by name, as well as asserting the
        // type signature of the function with `get_typed_func`.
        let instance = linker.instantiate(&mut store, &module)?;

        // Get the guest's allocation function.
        let alloc_func = instance.get_typed_func::<(i32, i32), i32>(&mut store, "__new")?;

        // FIXME: memory is extracted in two places
        let memory = instance.get_memory(&mut store, MEMORY_NAME).unwrap();

        let process_update_func = instance
            .get_typed_func::<i32, i32>(&mut store, "processUpdate")
            .unwrap();

        Ok(AscModule {
            memory,
            store,
            alloc_func,
            process_update_func,
        })
    }

    pub fn call_process_updates(&mut self, update: String) -> anyhow::Result<String> {
        // Encode the input string as UTF-16, which AssemblyScript expects.

        let input_utf16: Vec<u16> = update.encode_utf16().collect();
        let input_size_bytes = (input_utf16.len() * 2) as i32;

        // Allocate memory in the guest for the input string.
        let input_ptr = self
            .alloc_func
            .call(&mut self.store, (input_size_bytes, 123))?;

        // Write the UTF-16 bytes into the guest's memory.
        self.memory.write(
            &mut self.store,
            input_ptr as usize,
            bytemuck::cast_slice(&input_utf16),
        )?;

        // do the call
        let result = self.process_update_func.call(&mut self.store, input_ptr)?;

        Ok(read_asc_string(&self.memory, &mut self.store, result))
    }
}

fn read_asc_string(memory: &Memory, mut context: impl AsContext, ptr: i32) -> String {
    let mut length_buffer = [0; 4];
    memory
        .read(&context, (ptr - 4) as usize, &mut length_buffer)
        .unwrap();
    let length = u32::from_le_bytes(length_buffer);

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
