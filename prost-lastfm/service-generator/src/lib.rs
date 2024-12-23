mod service;
mod service_macro;

use prost_build::Service;

pub use service_macro::ServiceGeneratorMacroWrapper;

pub enum AsyncSetting {
    None,
    SyncOnly,
    AsyncOnly,
    SyncAndAsync,
    AsyncDefault,
}

pub struct LastFMRPCGenerator {
    pub async_setting: AsyncSetting,
}

impl service_macro::ServiceGeneratorMacro for LastFMRPCGenerator {
    fn generate(&mut self, service: Service) -> proc_macro2::TokenStream {
        let mut tokens = proc_macro2::TokenStream::new();
        if let AsyncSetting::SyncOnly | AsyncSetting::SyncAndAsync = &self.async_setting {
            tokens.extend(service::service_trait(&service, false, false));
            tokens.extend(service::reqwest_service_impl(&service, false, false));
        }
        if let AsyncSetting::AsyncOnly | AsyncSetting::SyncAndAsync = &self.async_setting {
            tokens.extend(service::service_trait(&service, true, true));
            tokens.extend(service::reqwest_service_impl(&service, true, true));
        }
        if let AsyncSetting::AsyncDefault = &self.async_setting {
            tokens.extend(service::service_trait(&service, true, false));
            tokens.extend(service::reqwest_service_impl(&service, true, false));
        }
        tokens
    }
}
