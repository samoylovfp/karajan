use std::path::Path;

fn main() {
    let definitions = std::fs::read_to_string("scheme/tg.krj").unwrap();
    let definitions = asbind_parse::parse(&definitions).unwrap();
    let tg_rs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("gen")
        .join("tg.rs");
    std::fs::write(tg_rs, definitions.gen_rust().unwrap()).unwrap();
}
