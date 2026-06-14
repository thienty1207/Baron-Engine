---
id: PATH-TRAVERSAL
severity_max: HIGH
applies_to: rust
---

# Rust Path Traversal

## Intent

`PathBuf` does not make user-provided paths safe by itself. Pushing `../` segments or absolute paths can escape an intended base directory.

## Search patterns

```text
PathBuf::from
\.push\(
std::fs::read
std::fs::write
tokio::fs::read
tokio::fs::write
File::open
ZipArchive|tar::Archive
```

## Flag when

- filename/path from route, query, body, upload, archive entry, or header is joined to a server path
- code does not canonicalize and verify the result remains under the allowed base directory
- uploaded filenames become storage paths without normalization

## Do not flag when

- the app maps user IDs to server-generated filenames
- canonicalized target path is checked with `starts_with(base)`

## Fix recommendation

Use server-generated names, normalize/canonicalize paths, reject absolute paths and parent traversal, and verify the final path stays inside the allowed base directory.
