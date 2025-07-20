mod definitions;
mod parse;
mod gen_rust;

pub use parse::parse;
pub use gen_rust::GenRust;

#[cfg(test)]
mod tests {
    use crate::definitions::{Field, KrjFile, Struct};

    use super::parse;
    #[test]
    fn smoke_test() {
        let r = parse(
            "struct Update {
                field: i32,
                field2: i64,
                field3: User?
            }",
        )
        .unwrap();
        assert_eq!(
            r,
            KrjFile {
                structs: vec![Struct {
                    name: "Update".into(),
                    fields: vec![
                        Field {
                            name: "field".into(),
                            r#type: crate::definitions::Type::I32,
                            optional: false
                        },
                        Field {
                            name: "field2".into(),
                            r#type: crate::definitions::Type::I64,
                            optional: false
                        },
                        Field {
                            name: "field3".into(),
                            r#type: crate::definitions::Type::Other("User".into()),
                            optional: true
                        }
                    ]
                }]
            }
        )
    }
    #[test]
    fn trailing_comma() {
           let r = parse(
            "struct Update {
                field: User?,
            }",
        )
        .unwrap();
    }
}
