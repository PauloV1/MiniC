//! Function declaration parser for MiniC.

use crate::ir::ast::{FunDecl, Type, UncheckedFunDecl};
use crate::parser::identifiers::identifier;
use crate::parser::statements::statement;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

/// Parse a type name: int | float | bool | str | void | T[] | T[][] (C-style lowercase).
pub fn type_name(input: &str) -> IResult<&str, Type> {
    preceded(
        multispace0,
        alt((
            // 2D arrays must be tried before 1D (longer prefix first)
            map(tag("int[][]"), |_| Type::Array(Box::new(Type::Array(Box::new(Type::Int))))),
            map(tag("float[][]"), |_| Type::Array(Box::new(Type::Array(Box::new(Type::Float))))),
            map(tag("bool[][]"), |_| Type::Array(Box::new(Type::Array(Box::new(Type::Bool))))),
            map(tag("str[][]"), |_| Type::Array(Box::new(Type::Array(Box::new(Type::Str))))),
            map(tag("int[]"), |_| Type::Array(Box::new(Type::Int))),
            map(tag("float[]"), |_| Type::Array(Box::new(Type::Float))),
            map(tag("bool[]"), |_| Type::Array(Box::new(Type::Bool))),
            map(tag("str[]"), |_| Type::Array(Box::new(Type::Str))),
            map(tag("int"), |_| Type::Int),
            map(tag("float"), |_| Type::Float),
            map(tag("bool"), |_| Type::Bool),
            map(tag("str"), |_| Type::Str),
            map(tag("void"), |_| Type::Unit),
        )),
    )(input)
}

/// Parse a typed parameter (C-style): `Type name`.
fn param(input: &str) -> IResult<&str, (String, Type)> {
    map(
        tuple((
            preceded(multispace0, type_name),
            preceded(multispace1, identifier),
        )),
        |(ty, name)| -> (String, Type) { (name.to_string(), ty) },
    )(input)
}

/// Parse a function declaration (C-style): `ReturnType name(Type name, ...) body`.
/// Example: `int add(int x, int y) { ... }` or `void main() x = 1`.
pub fn fun_decl(input: &str) -> IResult<&str, UncheckedFunDecl> {
    let (rest, return_type) = preceded(multispace0, type_name)(input)?;
    let (rest, name) = preceded(multispace1, identifier)(rest)?;
    let (rest, params) = delimited(
        preceded(multispace0, tag("(")),
        separated_list0(preceded(multispace0, tag(",")), param),
        preceded(multispace0, tag(")")),
    )(rest)?;
    let (rest, body) = preceded(multispace0, statement)(rest)?;
    Ok((
        rest,
        FunDecl {
            name: name.to_string(),
            params,
            return_type,
            body: Box::new(body),
        },
    ))
}
