use super::*;
use quote::ToTokens;
use syn::{
    Attribute, Fields, File, FnArg, Item, ItemEnum, ItemFn, ItemImpl, ItemStruct, Pat, ReturnType,
    Type, Visibility,
};

pub fn parse_rust_source(source: &str, path: &str) -> Result<AstModule, String> {
    let file: File = syn::parse_str(source).map_err(|e| format!("syn parse error: {e}"))?;
    let mut module = AstModule {
        source_path: path.to_string(),
        structs: Vec::new(),
        enums: Vec::new(),
        functions: Vec::new(),
    };

    for item in file.items {
        match item {
            Item::Struct(s) => {
                module.structs.push(parse_struct(&s));
            }
            Item::Enum(e) => {
                module.enums.push(parse_enum(&e));
            }
            Item::Fn(f) => {
                module.functions.push(parse_fn(&f));
            }
            Item::Impl(i) => {
                parse_impl(&i, &mut module);
            }
            _ => {}
        }
    }

    Ok(module)
}

fn extract_docs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            return Some(lit_str.value().trim().to_string());
                        }
                    }
                }
            }
            None
        })
        .collect()
}

fn extract_derives(attrs: &[Attribute]) -> Vec<String> {
    let mut derives = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("derive") {
            let _ = attr.parse_nested_meta(|meta| {
                if let Some(ident) = meta.path.get_ident() {
                    derives.push(ident.to_string());
                }
                Ok(())
            });
        }
    }
    derives
}

fn parse_type(ty: &Type) -> AstType {
    let s = ty.to_token_stream().to_string();
    AstType::new(s)
}

fn parse_struct(s: &ItemStruct) -> AstStruct {
    let name = s.ident.to_string();
    let docs = extract_docs(&s.attrs);
    let derives = extract_derives(&s.attrs);

    let fields = match &s.fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|f| AstField {
                name: f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default(),
                ty: parse_type(&f.ty),
                is_pub: matches!(f.vis, Visibility::Public(_)),
            })
            .collect(),
        Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(idx, f)| AstField {
                name: format!("_{idx}"),
                ty: parse_type(&f.ty),
                is_pub: matches!(f.vis, Visibility::Public(_)),
            })
            .collect(),
        Fields::Unit => Vec::new(),
    };

    AstStruct {
        name,
        fields,
        derives,
        docs,
    }
}

fn parse_enum(e: &ItemEnum) -> AstEnum {
    let name = e.ident.to_string();
    let docs = extract_docs(&e.attrs);
    let derives = extract_derives(&e.attrs);

    let variants = e
        .variants
        .iter()
        .map(|v| {
            let v_name = v.ident.to_string();
            let fields = match &v.fields {
                Fields::Named(named) => named
                    .named
                    .iter()
                    .map(|f| AstField {
                        name: f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default(),
                        ty: parse_type(&f.ty),
                        is_pub: true,
                    })
                    .collect(),
                Fields::Unnamed(unnamed) => unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(idx, f)| AstField {
                        name: format!("_{idx}"),
                        ty: parse_type(&f.ty),
                        is_pub: true,
                    })
                    .collect(),
                Fields::Unit => Vec::new(),
            };
            AstVariant {
                name: v_name,
                fields,
            }
        })
        .collect();

    AstEnum {
        name,
        variants,
        derives,
        docs,
    }
}

fn parse_fn(f: &ItemFn) -> AstFn {
    let name = f.sig.ident.to_string();
    let docs = extract_docs(&f.attrs);
    let is_pub = matches!(f.vis, Visibility::Public(_));

    let params = f
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => Some(AstParam {
                name: "self".to_string(),
                ty: AstType::new("&mut self"),
            }),
            FnArg::Typed(pat_type) => {
                let param_name = match &*pat_type.pat {
                    Pat::Ident(pi) => pi.ident.to_string(),
                    _ => pat_type.pat.to_token_stream().to_string(),
                };
                Some(AstParam {
                    name: param_name,
                    ty: parse_type(&pat_type.ty),
                })
            }
        })
        .collect::<Vec<_>>();

    let is_native_hook = name.starts_with("native_")
        || name.contains("_tick")
        || name.contains("_interact")
        || name.contains("_hud")
        || name.contains("_key")
        || params
            .iter()
            .any(|p| p.ty.raw.contains("Game") || p.ty.raw.contains("HudBatch"));

    let return_ty = match &f.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(parse_type(ty)),
    };

    AstFn {
        name,
        params,
        return_ty,
        is_pub,
        is_native_hook,
        docs,
    }
}

fn parse_impl(i: &ItemImpl, module: &mut AstModule) {
    for item in &i.items {
        if let syn::ImplItem::Fn(f) = item {
            let mut parsed = parse_fn(&ItemFn {
                attrs: f.attrs.clone(),
                vis: f.vis.clone(),
                sig: f.sig.clone(),
                block: Box::new(f.block.clone()),
            });
            let type_name = i.self_ty.to_token_stream().to_string();
            if type_name.contains("Game") {
                parsed.is_native_hook = true;
            }
            module.functions.push(parsed);
        }
    }
}
