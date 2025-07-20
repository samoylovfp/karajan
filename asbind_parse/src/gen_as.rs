use std::fmt::Write;

use anyhow::Result;
use itertools::Itertools;

use crate::definitions::{KrjFile, Struct, Type};

pub trait GenAs {
    fn gen_as(&self) -> Result<String>;
}

impl GenAs for KrjFile {
    fn gen_as(&self) -> Result<String> {
        let r: Vec<String> = self.structs.iter().map(GenAs::gen_as).try_collect()?;

        Ok(r.join("\n".into()))
    }
}

impl GenAs for Struct {
    fn gen_as(&self) -> Result<String> {
        let mut res = String::with_capacity(1024);
        writeln!(&mut res, "class {} {{", self.name)?;
        for f in &self.fields {
            writeln!(
                &mut res,
                "  {}: {} {}",
                f.name,
                f.r#type.gen_as()?,
                if f.optional { " | null = null;" } else { "" }
            )?;
        }
        writeln!(&mut res, "}}")?;

        Ok(res)
    }
}

impl GenAs for Type {
    fn gen_as(&self) -> Result<String> {
        let res = match self {
            Type::Other(o) => o.clone(),
            Type::String => "string".into(),
            Type::I32 => "i32".into(),
            Type::I64 => "i64".into(),
        };
        Ok(res)
    }
}
