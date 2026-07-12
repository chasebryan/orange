# Support policy

| Product or version | Support state |
| --- | --- |
| Orange compiler source | Pre-alpha, best-effort owner support |
| Orange software releases | Not applicable; none exists |

This repository contains an incomplete Rust compiler foundation alongside
research and planning material. It is not a supported product and provides no
production, compatibility, cryptographic, or software-security guarantee.

Public planning questions and non-sensitive defects belong in their structured
issue forms. Sensitive vulnerabilities use [SECURITY.md](SECURITY.md), and
conduct concerns use [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Never put
credentials, private keys, embargoed vulnerabilities, or private cryptographic
material in an issue.

Decision D-022 directs best-effort solo support with no SLA, LTS, compatibility,
or migration promise. Security-driven algorithm or target-profile withdrawal
may be immediate. Each release, if one is later authorized, must publish its
actual start and end dates and single-maintainer risk.

Future support attaches to the complete affected tuple, not SemVer alone:
language edition, core and evidence format editions, toolchain release,
cryptography profile, target/ABI/leakage profile, package/artifact digest, and
operating environment. Security maintenance, general community help, and paid
consulting are distinct services. No response-time SLA is currently offered.
