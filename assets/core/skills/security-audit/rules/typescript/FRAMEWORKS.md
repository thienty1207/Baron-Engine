# TypeScript framework supplements

Load only when a dependency manifest or source import provides matching
evidence; otherwise keep the framework unknown and apply TypeScript rules.

| Evidence signal | Focused review |
|---|---|
| Express / Fastify / Koa | Middleware order, validation, session/CSRF behavior, body limits and error responses |
| NestJS | Guard/interceptor activation, decorator metadata, validation pipes and provider scope |
| Next.js | Server/client boundary, route handlers, middleware, server actions, redirect and cache isolation |
