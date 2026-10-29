# .NET framework supplements

Load only when project references or source imports provide matching evidence;
otherwise keep the framework unknown and apply C# rules.

| Evidence signal | Focused review |
|---|---|
| ASP.NET Core | Middleware and fallback policy, endpoint authorization, antiforgery, model binding, upload limits and forwarded headers |
| Entity Framework Core | Raw SQL parameterization, tenant query filters, transaction ownership and lazy-loading exposure |
