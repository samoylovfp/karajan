use wasmtime::{Engine, Linker, Module, Store, Val};

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
    // linker.func_wrap("host", "host_func", |caller: Caller<'_, u32>, param: i32| {
    //     println!("Got {} from WebAssembly", param);
    //     println!("my host state is: {}", caller.data());
    // })?;

    // linker.func_wrap("env", "abort", |caller: Caller<'_, u32>, param: (i32,i32,i32,i32)| {
    //     println!("Got {:?} from WebAssembly", param);
    //     println!("my host state is: {}", caller.data());
    // })?;

    // All wasm objects operate within the context of a "store". Each
    // `Store` has a type parameter to store host-specific data, which in
    // this case we're using `4` for.
    let mut store: Store<u32> = Store::new(&engine, 4);

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

    // Write the UTF-16 bytes into the guest's memory.
    memory.write(&mut store, input_ptr as usize, bytemuck::cast_slice(&input_utf16))?;

    let func = instance
        .get_typed_func::<u32, u32>(&mut store, "processUpdate").unwrap();
    let result = func.call(&mut store, input_ptr).unwrap();

    // --- 3. READ the result string FROM the guest ---

    // To read the string, we need its length. AssemblyScript stores this
    // in a 4-byte header right before the string's content pointer.
    let mut length_buffer = [0; 4];
    memory.read(&store, (result - 4) as usize, &mut length_buffer)?;
    let result_len = u32::from_le_bytes(length_buffer);

    let mut result_buf = vec![0; (result_len) as usize];
    memory.read(&mut store, result as usize, &mut result_buf).unwrap();

    let result_u16 = result_buf.chunks(2).map(|w|u16::from_le_bytes(w.try_into().unwrap())).collect::<Vec<_>>();

    dbg!(String::from_utf16(&result_u16).unwrap());

    // let response = read_string_from_guest(&self.store, &self.memory, result_ptr, result_len);

    Ok(())
}
