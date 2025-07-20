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
    let mut result: [Val; 1] = [0.into()];
    let func = instance
        .get_func(&mut store, "processUpdate")
        .unwrap()
        .call(&mut store, &[0.into()], &mut result);
    dbg!(result);

    // // let hello = instance.get_typed_func::<(), ()>(&mut store, "hello")?;
    //
    // // And finally we can call the wasm!
    // hello.call(&mut store, ())?;

    Ok(())
}
