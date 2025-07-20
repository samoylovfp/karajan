use std::fmt::Write;

#[derive(Debug, PartialEq)]
pub struct KrjFile {
    pub structs: Vec<Struct>,
}

#[derive(Debug, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub r#type: Type,
    pub optional: bool,
}

#[derive(Debug, PartialEq)]
pub enum Type {
    /// struct name
    Other(String),
    String,
    I32,
    I64,
}
