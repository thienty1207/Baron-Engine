# Java framework supplements

Load only when bounded build metadata or source imports provide matching
evidence; otherwise keep the framework unknown and apply Java rules.

| Evidence signal | Focused review |
|---|---|
| Spring / Spring Boot | Security filter chain, method security activation, object binding, CSRF, actuator exposure, transaction ownership |
| Jakarta Servlet / JAX-RS | Filter/interceptor order, authentication context, request limits, output encoding and error handling |
