# Rust framework supplements

Load only when Cargo dependencies or source imports provide matching evidence;
otherwise keep the framework unknown and apply Rust rules.

| Evidence signal | Focused review |
|---|---|
| Axum | Extractor validation, middleware/layer order, authorization, body limits and error conversion |
| Actix Web | Guard and middleware order, identity/session cookies, payload limits and handler state |
| Rocket | Request guards, fairings, managed state, upload limits and production secret configuration |
| SQLx / Diesel | Bound query parameters, transaction ownership, tenant filters, migration safety and error disclosure |
