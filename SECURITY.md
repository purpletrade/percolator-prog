Purple Security Policy (Draft)

Contact
- Primary: tradeonpurple@proton.me
- Alternate: https://purple.trade/security (responsible disclosure page)
- PGP: Optional — provide a public key on the security page if you prefer encrypted reports.

Scope
- On-chain programs and SDKs maintained by Purple, including the Percolator program and associated matching components.
- Production deployments on Solana mainnet-beta and devnet (clearly identify addresses).

Out of Scope (examples)
- Social engineering of Purple staff or partners.
- Denial-of-service attacks that disrupt network or validator operations.
- Automated scanners reporting missing headers, clickjacking, etc., without demonstrable impact.

Disclosure Process
- Please email a report with a proof-of-concept, impact, and affected program IDs.
- We will acknowledge within 72 hours, triage and reproduce within 7 business days.
- We will propose remediation and coordinate a disclosure timeline. For critical issues, we may ask for additional time to safely deploy fixes and/or pause functionality.

Safe Harbor
- As long as you comply with this policy and make a good-faith effort to avoid privacy violations, data destruction, or service disruption, we will not initiate legal action against you.

Testing Guidelines
- Use devnet where possible; for mainnet, only minimal test transactions with negligible value.
- Do not attempt to exfiltrate user data or funds.
- Do not publicly disclose vulnerabilities prior to an agreed remediation timeline.

Recognition / Bounties
- If you operate a formal bug bounty (e.g., via Immunefi or self-hosted), link here. Otherwise, we will credit researchers in release notes where appropriate.

Versioning
- Keep this file aligned with the solana-security-txt macro values embedded in the program binary.
