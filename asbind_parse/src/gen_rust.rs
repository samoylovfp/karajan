use std::fmt::Write;

use anyhow::Result;
use itertools::Itertools;

use crate::definitions::{KrjFile, Struct, Type};

pub trait GenRust {
    fn gen_rust(&self) -> Result<String>;
}

pub trait GenWrite {
    fn gen_write(&self) -> Result<String>;
}

impl GenRust for KrjFile {
    fn gen_rust(&self) -> Result<String> {
        let mut items: Vec<String> = self.structs.iter().map(GenRust::gen_rust).try_collect()?;
        let write_impls: Vec<String> =
            self.structs.iter().map(GenWrite::gen_write).try_collect()?;
        items.extend(write_impls);
        Ok(items.join("\n"))
    }
}

impl GenRust for Struct {
    fn gen_rust(&self) -> Result<String> {
        let mut res = String::with_capacity(1024);
        writeln!(&mut res, "pub struct {} {{", self.name)?;
        for f in &self.fields {
            write!(&mut res, "    pub {}: ", f.name)?;
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

impl GenWrite for Struct {
    fn gen_write(&self) -> Result<String> {
        let mut res = String::with_capacity(1024);
        writeln!(&mut res, "impl asbind::WhatToWrite for {} {{", self.name)?;
        writeln!(
            &mut res,
            "    fn write(&self, target: &mut impl asbind::AllocateAndWrite, mut ptr: i32) {{"
        )?;
        writeln!(
            &mut res,
            r#"
        if let Some(size_on_heap) = self.size_on_heap() {{
            let heap_ptr = target.allocate(size_on_heap);
            heap_ptr.write(target, ptr);
            ptr = heap_ptr;
        }}
        "#
        )?;
        writeln!(&mut res, "        let mut offset = 0;")?;
        for f in &self.fields {
            if f.optional {
                writeln!(
                    &mut res,
                    r#"
        if let Some(value) = &self.{field} {{
            value.write(target, ptr + offset);
        }} else {{
            // FIXME: check if __new returns nulled memory;
            0_i32.write(target, ptr + offset);
        }}
        // FIXME: should be message size on stack
        offset += 4;
                "#,
                    field = f.name
                )?;
            } else {
                writeln!(
                    &mut res,
                    "        self.{field}.write(target, ptr + offset); offset += self.{field}.size_on_stack();",
                    field = f.name
                )?;
            }
            // match &f.r#type {
            //     Type::Other(s) => {
            //         writeln!(&mut res, "    let field_ptr = target.allocate(todo!());")?;
            //         todo!("Write the structure bytes");
            //         writeln!(&mut res, "    target.write(ptr + offset, field_ptr);")?
            //     }
            //     Type::String => writeln!(
            //         &mut res,
            //         "    self.{}.write(ptr + offset); offset += 4",
            //         f.name
            //     )?,
            //     Type::I32 => writeln!(
            //         &mut res,
            //         "    target.write(ptr + offset, self.{}); offset += 4",
            //         f.name
            //     )?,
            //     Type::I64 => writeln!(
            //         &mut res,
            //         "    target.write(ptr + offset, self.{}); offset += 8",
            //         f.name
            //     )?,
            // }
        }

        writeln!(&mut res, "    }}")?;
        writeln!(
            &mut res,
            r#"
    fn size_on_stack(&self) -> i32 {{
        4
    }}
        "#
        )?;
        writeln!(
            &mut res,
            r#"
    fn size_on_heap(&self) -> Option<i32> {{
        todo!()
    }}
        "#
        )?;
        writeln!(&mut res, "}}")?;

        //       writeln!(
        //     &mut res,
        //     r#"
        // fn size_on_stack(&self) {{
        //     let heap_ptr = target.allocate(size_on_heap);
        //     heap_ptr.write(target, ptr);
        //     ptr = heap_ptr;
        // }}
        // "#
        // )?;

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
                fields: vec![
                    Field {
                        name: "field".into(),
                        r#type: Type::I32,
                        optional: false,
                    },
                    Field {
                        name: "field2".into(),
                        r#type: Type::Other("S2".into()),
                        optional: true,
                    },
                ],
            }],
        };
        assert_eq!(
            k.gen_rust().unwrap().trim(),
            r#"
pub struct S1 {
    pub field: i32,
    pub field2: Option<S2>,
}"#
            .trim()
        )
    }
}
