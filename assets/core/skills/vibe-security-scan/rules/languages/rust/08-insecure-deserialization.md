---
id: INSECURE-DESERIALIZATION
severity_max: HIGH
applies_to: rust
---

# Rust Insecure Deserialization

## Intent

Rust deserialization is usually safer than dynamic-language object deserialization, but untrusted binary formats, unsafe custom visitors, and unbounded payloads can still cause denial of service, logic bypass, or unsafe-code exposure.

## Search patterns

```text
bincode::deserialize
serde_yaml::from_str
ron::from_str
postcard::from_bytes
MessagePack|rmp_serde::from
unsafe.*Deserialize|deserialize_any
```

## Flag when

- untrusted request body, upload, webhook, queue message, or cookie data is deserialized with no size/depth/type limits
- deserialized data drives auth, role, filesystem, command, or payment decisions without validation
- unsafe custom deserialization accepts attacker-controlled structure

## Do not flag when

- payload size is bounded, schema is narrow, and values are validated before use
- deserialization only handles trusted local config files

## Fix recommendation

Bound payload size, use narrow structs, reject unknown fields where possible, validate after parsing, and keep unsafe custom deserializers out of request paths.
