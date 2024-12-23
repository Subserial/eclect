use convert_case::{Case, Casing};
use extensions::LastFmIdent;
use proc_macro2::TokenStream;
use quote::quote;
use std::str::FromStr;

pub fn arg_name(full_rusty_arg: &str) -> String {
    full_rusty_arg.replace("::", "_").to_case(Case::Snake)
}

pub fn arg_type(full_rusty_arg: &str) -> String {
    if let Some((pre, post)) = full_rusty_arg.rsplit_once("::") {
        format!("{}::{}", pre, post.to_case(Case::UpperCamel))
    } else {
        full_rusty_arg.to_case(Case::UpperCamel)
    }
}

fn ext_auth(base: &prost_build::Method) -> LastFmIdent {
    let data = *base
        .options
        .extension_set
        .extension_data(extensions::IDENT)
        .unwrap_or(&0);
    LastFmIdent::from_i32(data).unwrap_or(LastFmIdent::IdentUnknown)
}

fn ext_method_name(base: &prost_build::Method) -> String {
    base.options
        .extension_set
        .extension_data(extensions::METHOD_NAME)
        .map(String::clone)
        .unwrap_or_else(|_| String::new())
}

fn async_tokens(write_async: bool) -> (TokenStream, TokenStream) {
    if write_async {
        (quote!(async), quote!(.await))
    } else {
        (TokenStream::new(), TokenStream::new())
    }
}

pub fn agent_trait(method: &prost_build::Method, write_async: bool) -> TokenStream {
    let (token_async, _) = async_tokens(write_async);

    let name = TokenStream::from_str(&arg_name(&method.name)).unwrap();
    let base_arg = TokenStream::from_str(&arg_name(&method.input_type)).unwrap();
    let base_in_ty = TokenStream::from_str(&arg_type(&method.input_type)).unwrap();
    let base_out_ty = TokenStream::from_str(&arg_type(&method.output_type)).unwrap();

    let mut args = vec![quote!(#base_arg: #base_in_ty)];

    match ext_auth(&method) {
        LastFmIdent::IdentUnknown
        | LastFmIdent::IdentStandard
        | LastFmIdent::IdentSignatureOnly => {}
        LastFmIdent::IdentSessionToken => args.push(quote!(session_token: &str)),
        LastFmIdent::IdentSessionOptional => args.push(quote!(session_token: Option<&str>)),
    }

    quote! {
        #token_async fn #name(&self, #(#args),*) -> Result<#base_out_ty, Self::Error>;
    }
}

pub fn reqwest_agent_impl(method: &prost_build::Method, write_async: bool) -> TokenStream {
    let (token_async, token_await) = async_tokens(write_async);

    let name = TokenStream::from_str(&arg_name(&method.name)).unwrap();
    let base_arg =
        TokenStream::from_str(&arg_name(&method.input_type.to_case(Case::Snake))).unwrap();
    let base_in_ty = TokenStream::from_str(&arg_type(&method.input_type)).unwrap();
    let base_out_ty = TokenStream::from_str(&arg_type(&method.output_type)).unwrap();

    let mut args = vec![quote!(#base_arg: #base_in_ty)];

    let (session_key, append_signature) = match ext_auth(&method) {
        LastFmIdent::IdentUnknown | LastFmIdent::IdentStandard => (quote!(None), false),
        LastFmIdent::IdentSignatureOnly => (quote!(None), true),
        LastFmIdent::IdentSessionToken => {
            args.push(quote!(session_token: &str));
            (quote!(Some(session_token)), true)
        }
        LastFmIdent::IdentSessionOptional => {
            args.push(quote!(session_token: Option<&str>));
            (quote!(session_token), false)
        }
    };

    let method = &ext_method_name(method);
    let req_method = quote!(get);
    let expect_msg = format!("invalid message type {}", name);

    quote! {
        #token_async fn #name(&self, #(#args),*) -> Result<#base_out_ty, Self::Error> {
            let url = crate::api::ApiCall::new(
                &self.api_key,
                #method,
                #session_key
            )
            .struct_params(#base_arg).expect(#expect_msg)
            .to_url(self.secret.as_bytes(), &self.endpoint, #append_signature);
            let resp = self.client.#req_method(url).send()
                #token_await?
                .json::<crate::api::Response<#base_out_ty>>()
                #token_await?
                .to_result()?;
            Ok(resp)
        }
    }
}

pub fn service_trait(
    service: &prost_build::Service,
    write_async: bool,
    name_async: bool,
) -> TokenStream {
    let methods = service
        .methods
        .iter()
        .map(|method| agent_trait(method, write_async))
        .collect::<Vec<_>>();
    let name = if name_async {
        TokenStream::from_str(&format!("{}Async", &service.name)).unwrap()
    } else {
        TokenStream::from_str(&service.name).unwrap()
    };
    quote! {
        pub trait #name {
            type Error: From<reqwest::Error> + From<crate::error::LastFMError>;
            #(#methods)*
        }
    }
}

pub fn reqwest_service_impl(
    service: &prost_build::Service,
    write_async: bool,
    name_async: bool,
) -> TokenStream {
    let methods = service
        .methods
        .iter()
        .map(|method| reqwest_agent_impl(method, write_async))
        .collect::<Vec<_>>();

    let (name, agent) = if name_async {
        (
            TokenStream::from_str(&format!("{}Async", &service.name)).unwrap(),
            TokenStream::from_str(&format!("{}AgentAsync", &service.name)).unwrap(),
        )
    } else {
        (
            TokenStream::from_str(&service.name).unwrap(),
            TokenStream::from_str(&format!("{}Agent", &service.name)).unwrap(),
        )
    };
    let client = if write_async {
        quote!(reqwest::Client)
    } else {
        quote!(reqwest::blocking::Client)
    };
    quote! {
        pub struct #agent {
            client: #client,
            api_key: String,
            secret: String,
            endpoint: String,
        }
        impl #agent {
            pub fn new(client: #client, api_key: String, secret: String, endpoint: String) -> Self {
                Self { client, api_key, secret, endpoint }
            }
        }
        impl #name for #agent {
            type Error = crate::error::Error;
            #(#methods)*
        }
    }
}
