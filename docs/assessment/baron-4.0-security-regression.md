# Baron 4.0 Security Routing Regression

- Project: `59b68ba36271a308126ddf49e5a8891fb6475665a0875b1a21fb094c990b46aa`
- Source revision: `85e1ba7181924d58e207839ce5c0a70f96e3d334aea1739d98e4c06919205c69`
- Score: **100/100**
- Passed: `true`

- `source-appsec`: allowed `true` (expected `true`), authorization `false` (expected `false`), route `SourceAppSec`, passed `true`
- `reverse-static`: allowed `true` (expected `true`), authorization `false` (expected `false`), route `ReverseAnalysis`, passed `true`
- `missing-authorization`: allowed `false` (expected `false`), authorization `true` (expected `true`), route `AuthorizedAdversary`, passed `true`
  - hard failures: missing authorization
- `confirmed-authorized`: allowed `true` (expected `true`), authorization `true` (expected `true`), route `Mixed`, passed `true`
- `project-mismatch`: allowed `false` (expected `false`), authorization `true` (expected `true`), route `Mixed`, passed `true`
  - hard failures: authorization project ID does not match current project
- `allowlist-mismatch`: allowed `false` (expected `false`), authorization `true` (expected `true`), route `Mixed`, passed `true`
  - hard failures: target is outside the authorization allowlist
- `offensive-intent`: allowed `false` (expected `false`), authorization `true` (expected `true`), route `Unsupported`, passed `true`
  - hard failures: blocked offensive or destructive request; create a defensive remediation plan instead
- `unsupported-task`: allowed `false` (expected `false`), authorization `false` (expected `false`), route `Unsupported`, passed `true`
  - hard failures: security task does not match a supported safe route
- `mixed-without-scope`: allowed `false` (expected `false`), authorization `true` (expected `true`), route `Mixed`, passed `true`
  - hard failures: missing authorization
