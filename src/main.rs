use karajan::asc_loader::AscModule;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let wasm_file_name = std::env::args()
        .nth(1)
        .expect("Pass client path as first argument");
    let wasm_code = std::fs::read(wasm_file_name)?;
    let mut module = AscModule::from_bytes(&wasm_code)?;
    _ = dbg!(module.call_process_updates("test".into()));
    _ = dbg!(module.call_process_updates("".into()));
    Ok(())
}
