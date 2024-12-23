# prost-lastfm

Service generator for protobuf descriptors.

This project is split into four parts:
* `extensions`: Independent protobuf extension code.
* `serde-macros`: Derive macros that apply serialization attributes to
  generated struct fields.
* `service-generator`: Code generator that slots into prost. Requires
  `extensions`.
* `prost-lastfm`: The full service definition. Generated code defines the
  library's API while handwritten code defines errors and de+serialization.

I want to move this out of this project, but it requires prost to support
extensions. It will likely not look like this if or when it becomes possible.