# Security Policy

## Supported Versions

Atlas is currently in active development. Security fixes are applied to the latest version only.

| Version | Supported |
|---------|-----------|
| latest  | ✅ Yes    |
| older   | ❌ No     |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

To report a security vulnerability, open a [GitHub Security Advisory](https://github.com/atl-lang/atlas/security/advisories/new).

You can expect:
- Acknowledgement within 48 hours
- A status update within 7 days
- Credit in the release notes if you wish

## Scope

Security issues in the following areas are in scope:

- **Atlas compiler** — code execution, sandbox escapes, path traversal
- **Atlas CLI** — privilege escalation, unsafe file operations
- **Atlas stdlib** — memory safety, information disclosure
- **Atlas LSP** — malicious workspace attacks

## Out of Scope

- Vulnerabilities in user-written Atlas programs (not a compiler bug)
- Issues requiring physical access to the machine
- Social engineering attacks
