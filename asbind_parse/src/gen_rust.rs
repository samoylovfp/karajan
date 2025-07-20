use std::fmt::Write;

use anyhow::Result;
use itertools::Itertools;

use crate::definitions::{KrjFile, Struct, Type};

pub trait GenRust {
    fn gen_rust(&self) -> Result<String>;
}

impl GenRust for KrjFile {
    fn gen_rust(&self) -> Result<String> {
        let struct_defs = self
            .structs
            .iter()
            .map(GenRust::gen_rust)
            .try_collect::<_, Vec<String>, _>()?;
        Ok(struct_defs.join("\n"))
    }
}

impl GenRust for Struct {
    fn gen_rust(&self) -> Result<String> {
        let mut res = String::with_capacity(1024);
        writeln!(&mut res, "pub struct {} {{", self.name)?;
        for f in &self.fields {
            write!(&mut res, "    {}: ", f.name)?;
            if f.optional {
                writeln!(&mut res, "Option<{}>,", f.r#type.gen_rust()?)?
            } else {
                writeln!(&mut res, "{},", f.r#type.gen_rust()?)?
            }
        }
        writeln!(&mut res, "}}")?;
        Ok(res)
    }
}

impl GenRust for Type {
    fn gen_rust(&self) -> Result<String> {
        let res = match self {
            Type::Other(s) => s.clone(),
            Type::String => "String".into(),
            Type::I32 => "i32".into(),
            Type::I64 => "i64".into(),
        };
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::definitions::Field;

    use super::*;

    #[test]
    fn test_gen() {
        let k = KrjFile {
            structs: vec![Struct {
                name: "S1".into(),
                fields: vec![Field {
                    name: "field".into(),
                    r#type: Type::I32,
                    optional: false,
                },Field {
                    name: "field2".into(),
                    r#type: Type::Other("S2".into()),
                    optional: true,
                }],
            }],
        };
        assert_eq!(k.gen_rust().unwrap().trim(), r#"
pub struct S1 {
    field: i32,
    field2: Option<S2>,
}"#.trim())
    }
}
