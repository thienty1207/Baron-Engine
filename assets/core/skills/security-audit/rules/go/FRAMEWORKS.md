# Go framework supplements

Load only when module or source import evidence identifies the framework;
otherwise keep the framework unknown and apply Go rules.

| Evidence signal | Focused review |
|---|---|
| `net/http` | Handler wrapping/order, request body cap, deadlines, TLS and redirect policy |
| Gin / Echo / Fiber / Chi | Middleware order, route groups, authentication context, recovery behavior and body limits |
