// AST representations for Horizon Native Rust sources (inspired by rusty-ast)

use serde::{Deserialize, Serialize};

pub mod parser;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AstType {
    pub raw: String,
    pub is_optional: bool,
    pub inner_type: Option<String>,
}

impl AstType {
    pub fn new(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let is_optional = raw.starts_with("Option<") && raw.ends_with('>');
        let inner_type = if is_optional {
            Some(raw[7..raw.len() - 1].trim().to_string())
        } else {
            None
        };
        Self {
            raw,
            is_optional,
            inner_type,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AstField {
    pub name: String,
    pub ty: AstType,
    pub is_pub: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AstStruct {
    pub name: String,
    pub fields: Vec<AstField>,
    pub derives: Vec<String>,
    pub docs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AstVariant {
    pub name: String,
    pub fields: Vec<AstField>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AstEnum {
    pub name: String,
    pub variants: Vec<AstVariant>,
    pub derives: Vec<String>,
    pub docs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AstParam {
    pub name: String,
    pub ty: AstType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AstFn {
    pub name: String,
    pub params: Vec<AstParam>,
    pub return_ty: Option<AstType>,
    pub is_pub: bool,
    pub is_native_hook: bool,
    pub docs: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AstModule {
    pub source_path: String,
    pub structs: Vec<AstStruct>,
    pub enums: Vec<AstEnum>,
    pub functions: Vec<AstFn>,
}
