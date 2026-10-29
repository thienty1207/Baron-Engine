# Python framework supplements

Load a supplement only when a source import or bounded project manifest
provides matching evidence. If evidence is absent or ambiguous, use the Python
pack and report the framework as unknown.

| Evidence signal | Focused review |
|---|---|
| Django imports/dependency | URL-to-view authorization, ORM raw methods, CSRF, template autoescape, upload storage, secure session/cookie settings |
| FastAPI imports/dependency | Dependency ordering, route authorization, Pydantic runtime validation, response serialization, request/body limits |
| Flask imports/dependency | Blueprint guards, session signing/cookie flags, Jinja autoescape, upload paths, debug server disabled in production |
