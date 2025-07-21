use karajan::{asc_loader::AscModule, tg};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let wasm_file_name = std::env::args()
        .nth(1)
        .expect("Pass client path as first argument");
    let tg_key = std::env::var("TG_KEY").expect("TG_KEY should contain telegram key");
    let wasm_code = std::fs::read(wasm_file_name)?;

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async move {
            let module = AscModule::from_bytes(&wasm_code).await.unwrap();

            tg::serve(module, tg_key).await
        });

    Ok(())
}
