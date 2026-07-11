# Support policy

| Product or version | Support state |
| --- | --- |
| Orange software releases | Not applicable; none exists |

This repository currently contains research and planning material, not a
compiler, library, package, or supported product. It provides no production
support or software-security guarantee.

Public planning questions and non-sensitive defects belong in the structured
issue forms. Sensitive vulnerabilities use [SECURITY.md](SECURITY.md), and
conduct concerns use [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Never put
credentials, private keys, embargoed vulnerabilities, or private cryptographic
material in an issue.

Decision D-022 proposes, but does not yet promise, five years of full LTS plus
two years of critical-security-only support for a future Language Edition 1
toolchain line. It also proposes at least twelve months for ordinary
deprecations when safety permits; security-driven algorithm or target-profile
withdrawal may be immediate. Funding, maintainers, rotations, archives, and
rebuild capacity are prerequisites. Each release must publish actual start and
end dates.

Future support attaches to the complete affected tuple, not SemVer alone:
language edition, core and evidence format editions, toolchain release,
cryptography profile, target/ABI/leakage profile, package/artifact digest, and
operating environment. Security maintenance, general community help, and paid
consulting are distinct services. No response-time SLA is currently offered.
