---
id: COMMAND-INJECTION
severity_max: CRITICAL
applies_to: rust
---

# Rust Command Injection

## Intent

`std::process::Command` is safe when command and args are fixed and separated. Risk appears when code invokes a shell or lets L1 input control command names, shell fragments, or dangerous args.

## Search patterns

```text
Command::new\(\s*"sh"
Command::new\(\s*"bash"
Command::new\(\s*"cmd"
\.arg\(\s*format!
\.args\(
duct::cmd
xshell
```

## Flag when

- user input reaches `sh -c`, `bash -c`, `cmd /C`, or PowerShell command strings
- user input selects executable names or unvalidated flags
- uploaded filenames, URLs, archive paths, or repo paths become shell arguments with dangerous tools

## Do not flag when

- executable is fixed and user input is passed as a single argument to a non-shell command
- user choices are mapped through an enum/allowlist

## Fix recommendation

Avoid shells, use fixed executables, pass arguments separately, validate flags through allowlists, and run commands with least privilege in a constrained working directory.
