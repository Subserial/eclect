fn main() -> Result<(), String> {
    let mut registry = prost::ExtensionRegistry::new();
    extensions::register_extensions(&mut registry);

    let async_setting = if cfg!(feature = "async_default") {
        if cfg!(any(feature = "gen_sync", feature = "gen_async")) {
            return Err("async_default incompatible with other gen features".to_string());
        } else {
            service_generator::AsyncSetting::AsyncDefault
        }
    } else {
        match (cfg!(feature = "gen_sync"), cfg!(feature = "gen_async")) {
            (true, true) => service_generator::AsyncSetting::SyncAndAsync,
            (true, false) => service_generator::AsyncSetting::SyncOnly,
            (false, true) => service_generator::AsyncSetting::AsyncOnly,
            (false, false) => panic!(
                "enabled features requires one of 'gen_sync', 'gen_async' or 'async_default'"
            ),
        }
    };

    prost_build::Config::new()
        .field_attribute("text", "#[serde(rename = \"#text\")]")
        .field_attribute("attr", "#[serde(rename = \"@attr\")]")
        .type_attribute(".", "#[serde_macros::default_deserialize_with(u32 => crate::shim::parse_from_string)]")
        .type_attribute(".", "#[serde_macros::default_deserialize_with(::core::option::Option<u32> => crate::shim::parse_option_from_string)]")
        .type_attribute(".", "#[serde_macros::default_deserialize_with(bool => crate::shim::parse_bool)]")
        .type_attribute(".", "#[serde_macros::default_deserialize_with(::core::option::Option<bool> => crate::shim::parse_option_bool)]")
        .type_attribute(".", "#[derive(::serde::Deserialize, ::serde::Serialize)]")
        .type_attribute("lastfm.ListAttributes", "#[serde(rename_all = \"camelCase\")]")
        .type_attribute("lastfm.error.Error.Error", "#[serde(untagged)]")
        .extension_registry(registry)
        .service_generator(Box::new(service_generator::ServiceGeneratorMacroWrapper(
            service_generator::LastFMRPCGenerator { async_setting },
        )))
        .compile_protos(&["../proto/service.proto", "../proto/error.proto"], &[
            "../proto/",
        ])
        .map_err(|err| err.to_string())?;
    Ok(())
}
