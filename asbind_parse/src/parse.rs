use anyhow::anyhow;
use winnow::{
    Parser, Result,
    ascii::{alphanumeric1, multispace0, space0, space1},
    combinator::{alt, opt, repeat, separated, seq},
    error::{
        StrContext::{Expected, Label},
        StrContextValue,
    },
    token::take_while,
};

use crate::definitions::{Field, KrjFile, Struct, Type};

pub fn parse(s: &str) -> anyhow::Result<KrjFile> {
    Ok(KrjFile {
        structs: repeat(0.., struct_def)
            .context(Label("Repeated struct definition"))
            .parse(s)
            .map_err(|e| anyhow!("{e:?}"))?,
    })
}

fn struct_def(input: &mut &str) -> Result<Struct> {
    seq!(
        Struct {
            _: ("struct", space1).context(Label("struct keyword")),
            name: ident.context(Label("struct name")),
            _: (multispace0, '{', multispace0).context(Label("struct name")),
            fields: separated(0.., field_def, (multispace0, ',', multispace0)).context(Label("comma separated list of fields")),
            _: opt((multispace0, ',')),
            _: (multispace0, '}', multispace0).context(Expected(StrContextValue::CharLiteral('}')))
        }
    )
    .parse_next(input)
}

fn ident<'s>(input: &mut &'s str) -> Result<String> {
    take_while(1.., |c: char| c.is_alphanumeric() || c == '_')
        .map(String::from)
        .parse_next(input)
}

fn field_def(input: &mut &str) -> Result<Field> {
    seq!(
        Field{
            name: ident.context(Label("field name")),
            _ : (space0, ':', space0),
            r#type: field_type,
            optional: opt('?').map(|v|v.is_some())
        }
    )
    .parse_next(input)
}

fn field_type(input: &mut &str) -> Result<Type> {
    alt((
        "i32".map(|_| Type::I32),
        "i64".map(|_| Type::I64),
        "string".map(|_| Type::String),
        alphanumeric1.map(ToString::to_string).map(Type::Other),
    ))
    .context(Expected(StrContextValue::Description("field type")))
    .parse_next(input)
}
