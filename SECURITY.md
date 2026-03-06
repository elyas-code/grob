# Security Policy

## Supported Versions

Grob is currently in active development (Alpha phase). Security updates and patches will be provided according to the following schedule:

| Version | Status | Supported |
| ------- | ------ | --------- |
| 0.0.1 | Initial Release (Alpha) | ✅ Full Support |

**Note**: Grob is at its initial release (0.0.1). As the project develops, new versions will be released with new features, improvements, and security patches.

### Version Support Details

- **0.0.1**: Current and only version - all security patches and bug fixes are applied to this version
- Future releases will follow semantic versioning (0.1.0, 0.2.0, etc.)

---

## Security Considerations

### Current Status

Grob is in **Alpha development** and should **not be used in production environments** without thorough security review and testing. The following security features are either incomplete or planned:

#### Implemented Security Features
- ✅ Memory safety via Rust's ownership system
- ✅ Safe concurrency primitives
- ✅ Input validation in parsers (HTML/CSS)
- ✅ Basic error handling and recovery

#### Planned Security Features
- 🚧 HTTPS/TLS support (currently HTTP only)
- 🚧 Content Security Policy (CSP) enforcement
- 🚧 CORS (Cross-Origin Resource Sharing) support
- 🚧 XSS (Cross-Site Scripting) protection
- 🚧 CSRF (Cross-Site Request Forgery) tokens
- 🚧 Secure cookie handling
- 🚧 Sandbox/isolation for JavaScript execution
- 🚧 Safe resource loading with origin verification

#### Known Security Limitations
- ⚠️ No HTTPS support - all connections are unencrypted HTTP
- ⚠️ Limited input validation for malicious content
- ⚠️ JavaScript execution not sandboxed (in development)
- ⚠️ No authentication or authorization framework
- ⚠️ File system access not restricted
- ⚠️ No protection against malicious stylesheets or scripts

---

## Reporting a Vulnerability

We take security seriously and appreciate responsible disclosure of security vulnerabilities.

### How to Report

**Please DO NOT open a public GitHub issue for security vulnerabilities.**

Instead, email your report to: **elyas@albahrani.org**

Include the following information:

1. **Description**: Clear description of the vulnerability
2. **Impact**: Potential impact and severity (Critical, High, Medium, Low)
3. **Affected Component**: Which module(s) are affected (Parser, Layout, Network, etc.)
4. **Affected Versions**: Which versions are vulnerable
5. **Steps to Reproduce**: Detailed steps or proof-of-concept code
6. **Suggested Fix**: If you have any suggestions for fixing the vulnerability

### Response Timeline

- **Acknowledgment**: We will acknowledge receipt of your report within 48 hours
- **Investigation**: Initial investigation will be completed within 7 days
- **Update Frequency**: We will provide status updates every 7 days while the issue is being addressed
- **Fix/Patch**: Critical vulnerabilities will be patched within 30 days when possible
- **Disclosure**: Coordinated disclosure will be arranged 30 days after a patch is released

### What to Expect

#### If the Vulnerability is Accepted

1. We will confirm the vulnerability and assess its severity
2. We will work on a fix or mitigation strategy
3. A security advisory will be released alongside the patch
4. You will be credited in the security advisory (unless you prefer anonymity)
5. The fix will be released as a patch update

#### If the Vulnerability is Declined

1. We will explain why the report is not considered a security issue
2. Suggestions for alternative approaches may be provided
3. The issue may be converted to a regular bug report if applicable

#### Disclosure Policy

- **30-day coordinated disclosure**: We ask reporters to give us 30 days to develop and release a fix before public disclosure
- **Public notification**: A security advisory will be published once a patch is available
- **Embargo**: We request that reporters maintain confidentiality during the 30-day period

---

## Security Best Practices for Grob Users

### While Using Grob (Alpha)

1. **Do not use in production**: Grob is under active development and not suitable for production use
2. **Monitor updates**: Check for security updates regularly
3. **Isolated environment**: Run Grob in an isolated environment if testing untrusted content
4. **No sensitive data**: Do not access sensitive information through Grob
5. **Report issues**: Report any suspicious behavior to the security contact

### For Local Development

1. **Review code**: Review any code you add to Grob
2. **Limit network access**: Restrict which sites Grob can access if possible
3. **Validate input**: If accepting input to feed to Grob, validate it thoroughly
4. **Keep updated**: Use the latest version of Grob from the main branch

### Content Security

1. **Assume untrusted content is unsafe**: Do not trust website content
2. **Limited JavaScript**: JavaScript execution is not sandboxed; be cautious
3. **CSS attacks**: Malicious CSS could potentially cause issues
4. **Network interception**: HTTP connections are unencrypted; assume content could be intercepted

---

## Security Roadmap

### Phase 1 (Current - Alpha)
- Focus: Core engine stability and correctness
- Security: Basic input validation, memory safety
- Target: Q1 2026

### Phase 2 (Beta)
- HTTPS/TLS support
- Basic sandbox for JavaScript
- Content Security Policy support
- Target: Q2 2026

### Phase 3 (1.0 Release)
- Full CORS implementation
- Comprehensive XSS protection
- Secure cookie handling
- Security audit by external firm
- Target: Q4 2026

### Phase 4 (Post 1.0)
- Advanced sandboxing
- Service Worker security
- Enhanced authentication support
- Ongoing security hardening

---

## Dependencies and Supply Chain

Grob uses the following major dependencies:

- **winit**: Window management and event handling
- **pixels**: Graphics rendering
- **rusttype**: Font rendering
- **Other Rust ecosystem crates**: See Cargo.toml

### Dependency Security

- Regular dependency updates for security patches
- `cargo audit` used to check for known vulnerabilities
- Minimal dependencies to reduce attack surface
- Review of new dependencies for security implications

---

## Compliance

Grob aims to comply with:

- **Rust Security Guidelines**: Following Rust best practices
- **OWASP Top 10**: Addressing common web vulnerabilities (as applicable)
- **HTML5 Security**: Following HTML5 security specifications
- **CSS Security**: Following CSS security guidelines

**Note**: Compliance is an ongoing effort and not yet complete.

---

## Security Contacts

For security matters only (not general support):
- Email: **elyas@albahrani.org**
- PGP Key: [To be added when available]

For general support and bug reports:
- GitHub Issues: https://github.com/elyas-code/grob/issues
- Discussions: https://github.com/elyas-code/grob/discussions

---

## Acknowledgments

We thank all security researchers and community members who responsibly report vulnerabilities and help improve Grob's security.

---

## Additional Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Mozilla Security Guidelines](https://infosec.mozilla.org/)
- [Rust Security](https://www.rust-lang.org/governance/security-policy)
- [Web Security Academy](https://portswigger.net/web-security)

---

**Last Updated**: January 22, 2026

**Status**: Alpha - Security features still under development
