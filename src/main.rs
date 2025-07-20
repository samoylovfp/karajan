use std::{cell::Cell, sync::{Arc, Mutex}};

use wasmtime::{AsContext, Caller, Engine, Linker, Memory, Module, Store, Val};

struct Mem(Option<Memory>);

fn main() -> anyhow::Result<()> {
    let engine = Engine::default();
    let wasm_file_name = std::env::args()
        .nth(1)
        .expect("Pass client path as first argument");
    let wasm_code = std::fs::read(wasm_file_name)?;
    let module = Module::new(&engine, wasm_code)?;

    // Host functionality can be arbitrary Rust functions and is provided
    // to guests through a `Linker`.
    let mut linker = Linker::new(&engine);
    // linker.func_wrap("env", "abort", |caller: Caller<'_, u32>, param: i32| {
    //     println!("Got {} from WebAssembly", param);
    //     println!("my host state is: {}", caller.data());
    // })?;

    linker.func_wrap(
        "env",
        "abort",
        |caller: Caller<'_, Mem>, message_ptr: i32, filename: i32, ine: i32, col: i32| {
            let mem = caller.data();
            let mut buf = vec![0; 1024];
            let msg = read_string(&mem.0.unwrap(), caller, message_ptr);
            println!("Got error {msg:?}");
            // println!("Got {p1} {p2} {p3} {p4} from WebAssembly");
            // println!("my host state is: {}", caller.data());
        },
    )?;

    // All wasm objects operate within the context of a "store". Each
    // `Store` has a type parameter to store host-specific data, which in
    // this case we're using `4` for.
    let mut store: Store<Mem> = Store::new(&engine, Mem(None));

    // Instantiation of a module requires specifying its imports and then
    // afterwards we can fetch exports by name, as well as asserting the
    // type signature of the function with `get_typed_func`.
    let instance = linker.instantiate(&mut store, &module)?;

    // Get the guest's allocation function.
    let alloc_func = instance.get_typed_func::<(u32, u32), u32>(&mut store, "__new")?;

    // Encode the input string as UTF-16, which AssemblyScript expects.
    let input_string = "Хакатон".to_string();
    let input_utf16: Vec<u16> = input_string.encode_utf16().collect();
    let input_size_bytes = (input_utf16.len() * 2) as u32;

    // Allocate memory in the guest for the input string.
    let input_ptr = alloc_func.call(&mut store, (input_size_bytes, 123))?;

    let memory = instance.get_memory(&mut store, "memory").unwrap();
    *store.data_mut() = Mem(Some(memory.clone()));

    // Write the UTF-16 bytes into the guest's memory.
    memory.write(
        &mut store,
        input_ptr as usize,
        bytemuck::cast_slice(&input_utf16),
    )?;

    let func = instance
        .get_typed_func::<u32, u32>(&mut store, "processUpdate")
        .unwrap();
    let result = func.call(&mut store, input_ptr).unwrap();

    Ok(())
}


fn read_string(memory: &Memory, mut context: impl AsContext, ptr: i32) -> String {
    let mut length_buffer = [0; 4];
    memory.read(&context, (ptr - 4) as usize, &mut length_buffer).unwrap();
    let length = u32::from_le_bytes(length_buffer);

    let mut result_buf = vec![0; (length) as usize];
    memory
        .read(&mut context, ptr as usize, &mut result_buf)
        .unwrap();

    let result_u16 = result_buf
        .chunks(2)
        .map(|w| u16::from_le_bytes(w.try_into().unwrap()))
        .collect::<Vec<_>>();
    return String::from_utf16(&result_u16).unwrap()
}