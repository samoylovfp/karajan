use std::path::Path;

use asbind_parse::{GenAs, GenRust};

fn main() {
    let definitions = std::fs::read_to_string("scheme/tg.krj").unwrap();
    let definitions = asbind_parse::parse(&definitions).unwrap();
    
    let gen_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("gen");

    let tg_rs = gen_dir.join("tg.rs");
    std::fs::write(tg_rs, definitions.gen_rust().unwrap()).unwrap();

    let tg_as = gen_dir.join("tg.ts");
    std::fs::write(tg_as, definitions.gen_as().unwrap()).unwrap();
}
