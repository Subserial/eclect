use proc_macro2::TokenStream;
use prost_build::{Service, ServiceGenerator};

pub fn append_pretty(tokens: TokenStream, buf: &mut String) {
    if tokens.is_empty() {
        return;
    }
    let file = syn::parse2(tokens).expect("pretty print failure");
    buf.push_str(&prettyplease::unparse(&file));
}

pub trait ServiceGeneratorMacro {
    fn generate(&mut self, service: Service) -> TokenStream;

    fn finalize(&mut self) -> TokenStream {
        TokenStream::new()
    }

    fn finalize_package(&mut self, _package: &str) -> TokenStream {
        TokenStream::new()
    }
}

pub struct ServiceGeneratorMacroWrapper<T: ServiceGeneratorMacro>(pub T);

impl<T: ServiceGeneratorMacro> ServiceGenerator for ServiceGeneratorMacroWrapper<T> {
    fn generate(&mut self, service: Service, buf: &mut String) {
        append_pretty(self.0.generate(service), buf)
    }
    fn finalize(&mut self, buf: &mut String) {
        append_pretty(self.0.finalize(), buf)
    }
    fn finalize_package(&mut self, package: &str, buf: &mut String) {
        append_pretty(self.0.finalize_package(package), buf)
    }
}
