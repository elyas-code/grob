# Security Policy

## Supported Versions

Grob is currently in active development (Alpha phase). Security updates and patches will be provided according to the following schedule:

| Version | Status | Supported |
| ------- | ------ | --------- |
| 0.0.1 | Initial Release (Alpha) | Full Support |

**Note**: Grob is at its initial release (0.0.1). As the project develops, new versions will be released with new features, improvements, and security patches.

### Version Support Details

- **0.0.1**: Current and only version - all security patches and bug fixes are applied to this version
- Future releases will follow semantic versioning (0.1.0, 0.2.0, etc.)

---

## Security Considerations

### Current Status

Grob is in **Alpha development** and should **not be used in production environments** without thorough security review and testing. The following security features are either incomplete or planned:

#### Implemented Security Features
- Memory safety via Rust's ownership system
- Safe concurrency primitives
- Input validation in parsers (HTML/CSS)
- Basic error handling and recovery

#### Planned Security Features
- HTTPS/TLS support (currently HTTP only)
- Content Security Policy (CSP) enforcement
- CORS (Cross-Origin Resource Sharing) support
- XSS (Cross-Site Scripting) protection
- CSRF (Cross-Site Request Forgery) tokens
- Secure cookie handling
- Sandbox/isolation for JavaScript execution
- Safe resource loading with origin verification

#### Known Security Limitations
- No HTTPS support - all connections are unencrypted HTTP
- Limited input validation for malicious content
- JavaScript execution not sandboxed (in development)
- No authentication or authorization framework
- File system access not restricted
- No protection against malicious stylesheets or scripts

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

### Phase 1 (Prealpha - Current)
- Focus: Core engine stability and correctness
- Security: Basic input validation, memory safety
- Internal testing only
- Target: Q1 2026

### Phase 2 (Alpha)
- Initial HTTPS/TLS implementation
- Early JavaScript sandbox prototype
- Basic same-origin policy enforcement
- Initial Content Security Policy parsing
- Fuzz testing for parser and rendering engine
- Target: Q2 2026

### Phase 3 (Beta)
- Full HTTPS/TLS support
- Stable JavaScript sandbox
- Content Security Policy enforcement
- Initial CORS support
- Security bug bounty program
- Target: Q3 2026

### Phase 4 (1.0 Release)
- Full CORS implementation
- Comprehensive XSS protection
- Secure cookie handling
- Hardened sandboxing
- Security audit by external firm
- Target: Q4 2026

### Phase 5 (Post 1.0)
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
- PGP Key: 

-----BEGIN PGP PUBLIC KEY BLOCK-----

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
GULOdPTuuycMtNiuuXGvvwJ3Ew==
=yVD+
-----END PGP PUBLIC KEY BLOCK-----


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

**Last Updated**: March 6, 2026

**Status**: Prealpha - Security features still under development
