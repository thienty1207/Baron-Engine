# Superpowers Attribution

The workflow skill directories in this folder are vendored from
`obra/superpowers` release `v6.2.0`, commit
`3dcbd5c4b48e02263fbf4a3c01e3fe4f81d584d9`.

Copyright (c) 2025 Jesse Vincent. The upstream work is distributed under the
MIT License in `LICENSE.txt`.

Baron applies one local hardening patch to
`brainstorming/scripts/server.cjs`: the remote brand image, telemetry signal,
and live branding link are removed, and the displayed version is read from
local `UPSTREAM.json`. The visual companion makes no outbound branding request.
The original and patched tree digests are recorded in `UPSTREAM.json`.

Baron owns the root `SKILL.md` and `README.md` routing wrappers. Those wrappers
keep Superpowers as Baron's only workflow core and connect it to Baron memory,
plan, Harness, proof, trace, and quality-gate contracts. Baron does not depend
on a live upstream URL at runtime.
