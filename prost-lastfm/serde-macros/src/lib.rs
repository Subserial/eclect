extern crate proc_macro;

use quote::ToTokens;

struct AttributeRule {
    ty_from: syn::Type,
    _equals_token: syn::token::FatArrow,
    fn_using: syn::ExprPath,
}

impl syn::parse::Parse for AttributeRule {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ty_from: input.parse()?,
            _equals_token: input.parse()?,
            fn_using: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn default_deserialize_with(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let rule = syn::parse_macro_input!(attrs as AttributeRule);
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    let res = match &mut input.data {
        syn::Data::Struct(data) => struct_apply_deserialize_with(rule, data),
        syn::Data::Enum(data) => enum_apply_deserialize_with(rule, data),
        _ => {
            return syn::Error::new(input.ident.span(), "expected struct")
                .to_compile_error()
                .into();
        }
    };
    match res {
        Ok(_) => input.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn struct_apply_deserialize_with(
    rule: AttributeRule,
    data: &mut syn::DataStruct,
) -> syn::Result<()> {
    for field in &mut data.fields {
        if field.ty.eq(&rule.ty_from) {
            apply_deserialize_with(&rule.fn_using, &mut field.attrs)?
        }
    }
    Ok(())
}

fn enum_apply_deserialize_with(rule: AttributeRule, data: &mut syn::DataEnum) -> syn::Result<()> {
    for variant in &mut data.variants {
        for field in &mut variant.fields {
            if field.ty.eq(&rule.ty_from) {
                continue;
            }
            apply_deserialize_with(&rule.fn_using, &mut field.attrs)?
        }
    }
    Ok(())
}

fn apply_deserialize_with(
    path: &syn::ExprPath,
    attrs: &mut Vec<syn::Attribute>,
) -> syn::Result<()> {
    for attr in &*attrs {
        if attr.path().is_ident("serde") {
            let mut has_deserialize_with = false;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("deserialize_with") {
                    has_deserialize_with = true;
                }
                Ok(())
            })?;
            if has_deserialize_with {
                return Ok(());
            }
        }
    }
    let path_str = path.to_token_stream().to_string();
    attrs.push(syn::parse_quote!(#[serde(deserialize_with = #path_str)]));
    Ok(())
}
