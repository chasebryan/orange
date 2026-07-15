#!/usr/bin/env python3
"""Orange foundation validator."""

from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import hashlib
import json
import os
import re
import select
import stat
import subprocess
import sys
import tempfile
import time
import tomllib
import unicodedata
from pathlib import Path, PurePosixPath
from typing import Any, Iterable, Mapping, Sequence
from urllib.parse import unquote, urlsplit


POLICY_PATH = Path("policy/gate0-repository-policy.json")
VALIDATOR_REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
ORANGE_BOOK_PATH = Path("docs/THE_ORANGE_BOOK.md")
ORANGE_BOOK_VERSION = "0.2"
ORANGE_BOOK_MINIMUM_CHAPTER_WORDS = 1_200
ORANGE_BOOK_CHAPTERS = (
    "## Chapter 1: The Seams Are the System",
    "## Chapter 2: Claims, Not Labels",
)
ORANGE_BOOK_REQUIRED_SECTIONS = (
    "## Contents",
    "## Preface",
    *ORANGE_BOOK_CHAPTERS,
    "## Manuscript map",
    "## Sources and drafting disclosure",
)
ORANGE_BOOK_CONTENTS = (
    "- [Preface](#preface)",
    "- [Chapter 1: The Seams Are the System](#chapter-1-the-seams-are-the-system)",
    "- [Chapter 2: Claims, Not Labels](#chapter-2-claims-not-labels)",
    "- [Manuscript map](#manuscript-map)",
    "- [Sources and drafting disclosure](#sources-and-drafting-disclosure)",
)
IGNORED_PARTS = set(".git .agents .codex __pycache__".split())
BINARY_SUFFIXES = set(".gif .jpeg .jpg .png .wasm".split())
TEXT_TAB_FREE_SUFFIXES = set(".json .jsonc .or .py .rs .sh .toml .yaml .yml".split())
GATE0_IGNORE_PATTERNS = tuple(
    """.DS_Store
Thumbs.db
*.swp
*.swo
*~
.idea/
.vscode/
.cache/
.tools/
__pycache__/
*.py[cod]
coverage/
dist/
out/
tmp/
.env
.env.*
!.env.example
*.key
*.pem
compiler/target/""".splitlines()
)
SCHEMA_DIALECT = "https://json-schema.org/draft/2020-12/schema"
GATE0_MAXIMUM_JSON_NESTING_DEPTH = 64
_JM = "9007199254740991"
GATE0_MAXIMUM_TEXT_FILE_BYTES = 256 * 1024
GATE0_MAXIMUM_BINARY_FILE_BYTES = 2 * 1024 * 1024
GATE0_MAXIMUM_REPOSITORY_BYTES = 8 * 1024 * 1024
GATE0_GIT_EXECUTABLE = "/usr/bin/git"
GATE0_MAXIMUM_REPOSITORY_FILES = 512
GATE0_MAXIMUM_REPOSITORY_PATH_BYTES = 1024
GATE0_MAXIMUM_RAW_PATH_METADATA_BYTES = 1024 * 1024
GATE0_MAXIMUM_FALLBACK_DIRECTORY_ENTRIES = 4096
_MF = 4096
GATE0_MAXIMUM_FINDINGS = _MF
GATE0_MAXIMUM_FINDING_MESSAGE_CHARACTERS = 4096
GATE0_MAXIMUM_GIT_STAGE_PREFIX_BYTES = 128
_GC = 4096
_GT = 30.0
_IP = "resource.inventory_protocol"
_RI_READ = "resource.inventory_read"
_RI_GIT = "resource.inventory_git"
_IE = "resource.inventory_encoding"
_RU = "resource.unsupported_host"
_RC = "resource.concurrent_change"
_ABA = "allowed_binary_artifacts["
_X = "cross_invariant"
_G = "github_actions"
_AR = "allowed_action_repositories"
_WP = "allowed_write_permissions"
_PD = "protected_file_digests"
_RS = "required_status"
_T = "compiler.dependency_table"
_RA = "record.acceptance"
_D = "2026-07-11"
_WS = "workspace"
_AB = "allowed_binary_artifacts"
_CT = "compiler/Cargo.toml"
_AI = "allowed_container_images"
_OCM = "compiler/crates/orange-compiler/Cargo.toml"
_CCM = "compiler/crates/orangec/Cargo.toml"
_OC = "orange-compiler"
_DS = "dependencies"
_RP = "required_paths"
_TP = "allowed_top_level_paths"
_FS = "require_full_commit_sha"
_PB = "policy.binary"
_VC = "require_version_comment"
_SC = "scorecard.yml"
_DR = "dependency-review.yml"
_EL = "external-links.yml"
_O = "workflow-online-audit.yml"
_BF = "bounded repository read failed"
_E = "evidence_refs"
_FE = "forbidden_events"
_CO = "required_codeowners"
_FP = "forbidden_paths"
_PV = "policy_version"
_RW = "required_workflows"
_GATE0_GIT_FIXED_ENVIRONMENT = {
    "GIT_CONFIG_GLOBAL": os.devnull,
    "GIT_CONFIG_NOSYSTEM": "1",
    "GIT_NO_LAZY_FETCH": "1",
    "GIT_NO_REPLACE_OBJECTS": "1",
    "GIT_OPTIONAL_LOCKS": "0",
    "GIT_TERMINAL_PROMPT": "0",
    "LC_ALL": "C",
}
MINIMUM_REQUIRED_PATHS = set(
    """
.editorconfig
.gitattributes
.github/CODEOWNERS
.github/ISSUE_TEMPLATE/conduct-contact.yml
.github/ISSUE_TEMPLATE/config.yml
.github/ISSUE_TEMPLATE/oep-proposal.yml
.github/ISSUE_TEMPLATE/planning-defect.yml
.github/ISSUE_TEMPLATE/planning-question.yml
.github/ISSUE_TEMPLATE/research-evidence.yml
.github/dependabot.yml
.github/dependency-review-config.yml
.github/pull_request_template.md
.github/workflows/ci.yml
.github/workflows/dependency-review.yml
.github/workflows/external-links.yml
.github/workflows/scorecard.yml
.github/workflows/workflow-online-audit.yml
.gitignore
.markdownlint-cli2.jsonc
CODE_OF_CONDUCT.md
CONTRIBUTING.md
compiler/.gitignore
compiler/Cargo.lock
compiler/Cargo.toml
compiler/README.md
compiler/crates/orange-compiler/Cargo.toml
compiler/crates/orange-compiler/src/core.rs
compiler/crates/orange-compiler/src/diagnostic.rs
compiler/crates/orange-compiler/src/edition.rs
compiler/crates/orange-compiler/src/eval.rs
compiler/crates/orange-compiler/src/lexer.rs
compiler/crates/orange-compiler/src/lib.rs
compiler/crates/orange-compiler/src/parser.rs
compiler/crates/orange-compiler/src/semantics.rs
compiler/crates/orange-compiler/src/source.rs
compiler/crates/orangec/Cargo.toml
compiler/crates/orangec/src/main.rs
compiler/crates/orangec/tests/cli.rs
compiler/crates/orangec/tests/s3a_conformance.rs
compiler/fixtures/hello.or
compiler/fixtures/s3a/invalid-duplicate-spec.or
compiler/fixtures/s3a/invalid-int-magnitude.or
compiler/fixtures/s3a/invalid-negative-word.or
compiler/fixtures/s3a/invalid-typed-impl.or
compiler/fixtures/s3a/invalid-unsupported-type.or
compiler/fixtures/s3a/invalid-word-range.or
compiler/fixtures/s3a/invalid-word-width.or
compiler/fixtures/s3a/valid-empty-mixed.or
compiler/fixtures/s3a/valid-int-radices.or
compiler/fixtures/s3a/valid-word8-boundaries.or
compiler/fixtures/typed-answer.or
DEPENDENCY_POLICY.md
GOVERNANCE.md
Makefile
README.md
RELEASE_POLICY.md
rust-toolchain.toml
SECURITY.md
SUPPORT.md
assets/brand/README.md
assets/brand/manifest.json
assets/brand/orange-banner-jpeg.JPEG
assets/brand/orange-banner.png
assets/brand/orange-banner2-erased.PNG
assets/brand/orange-banner2.PNG
assets/brand/orange-erased.PNG
assets/brand/orange-handdrawn-marker-banner.png
assets/brand/orange.jpg
assets/brand/orange.png
assets/brand/orangePNG.PNG
conformance/foundation/manifest.json
conformance/foundation/README.md
docs/DECISIONS.md
docs/GATE0_TRACEABILITY.md
docs/GATE0_SUPPORT_ENVELOPES.md
docs/LANGUAGE_2026.md
docs/PRODUCT_FORM_DECISION_PACKET.md
docs/PROOF_FOUNDATION_DECISION_SUITE.md
docs/REPRODUCIBILITY.md
docs/USER_JOURNEYS.md
docs/ARCHITECTURE.md
docs/ASSURANCE.md
docs/PROJECT_CHARTER.md
docs/RESEARCH.md
docs/ROADMAP.md
docs/SEMANTIC_STRATA_DECISION_SUITE.md
docs/SEMANTICS_2026.md
docs/THE_ORANGE_BOOK.md
docs/governance/adrs/ADR-0000-template.md
docs/governance/adrs/README.md
docs/governance/oeps/OEP-0000-template.md
docs/governance/oeps/README.md
docs/operations/CI_DEPENDENCIES.md
docs/operations/GITHUB_CONTROLS.md
docs/security/OSPS_BASELINE.md
docs/security/SECRETS_AND_INCIDENTS.md
docs/security/THREAT_MODEL.md
policy/README.md
policy/gate0-repository-policy.json
schemas/README.md
schemas/gate0/claim-record-v0.1.schema.json
schemas/gate0/evidence-manifest-v0.1.schema.json
schemas/gate0/repository-control-snapshot-v0.1.schema.json
schemas/gate0/standards-provenance-v0.1.schema.json
schemas/gate0/trust-inventory-v0.1.schema.json
scripts/ci/check-repository
scripts/ci/check-external-links
scripts/ci/install-actionlint
scripts/ci/install-lychee
tools/validate_foundation.py
tools/tests/test_validate_foundation.py
tools/tests/test_validate_foundation_hardening.py
""".strip().splitlines()
)
MINIMUM_FORBIDDEN_PATHS = set("COPYING LICENSE crates crypto formal release spec stdlib targets".split())
MINIMUM_REQUIRED_WORKFLOWS = set("ci.yml dependency-review.yml scorecard.yml".split())
MINIMUM_ACTION_REPOSITORIES = set(
    """DavidAnson/markdownlint-cli2-action actions/checkout actions/dependency-review-action
actions/upload-artifact github/codeql-action/upload-sarif zizmorcore/zizmor-action""".split()
)
_BRAND_IMPORT = "Byte-for-byte import from the steward-supplied Orange-Assets collection on "
_BRAND_ROLE = "Official working Orange "
GATE0_ALLOWED_CONTAINER_IMAGES = {
    "ghcr.io/ossf/scorecard-action@sha256:"
    "2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941"
}
GATE0_ALLOWED_BINARY_ARTIFACTS = [
    {"path": path, "sha256": digest, "role": _BRAND_ROLE + role, "provenance": _BRAND_IMPORT + date}
    for path, digest, role, date in (
        ("assets/brand/orange-banner2.PNG", "3136916eab9747871324cf146158e8f3a16197dbf32e8a6ef995056705dd6e5b", "wordmark on a light background", _D),
        ("assets/brand/orangePNG.PNG", "64d2e78436586466f9c24fb844922e1d7b474e98a6023b44a5a481533300ec02", "emblem source variant on a light background", _D),
        ("assets/brand/orange-banner-jpeg.JPEG", "288070ed86afd83a2e41e25fb664ac3ef44029521055a6ca3f6b6223cc48d41a", "horizontal lockup JPEG", _D),
        ("assets/brand/orange-banner2-erased.PNG", "5941784f123c7a3fb7922d859098d43d5aee10dbd8db4c9283a32b5f93e8611c", "transparent wordmark", _D),
        ("assets/brand/orange-erased.PNG", "9f256a98c1cbe7345ab29372fdc15eb9475ce3b89c4278af503d167d4a91f2f2", "transparent emblem", _D),
        ("assets/brand/orange-banner.png", "41cffe77744da07b9fbf9bc46c009755522468bbbc53a3f3f9b1a867ae05e266", "primary horizontal lockup with embedded C2PA claim", _D),
        ("assets/brand/orange.jpg", "170c48ab4a32bea289099b9505569ada5b99cc6deae93ece8f59d5c2102f4888", "emblem JPEG on a light background", _D),
        ("assets/brand/orange.png", "c10ed0b2d79a1e9447e842fcb9eaa7ec8eeb850dd2873e87eefd54d7cdc14463", "primary emblem with embedded C2PA claim", _D),
        ("assets/brand/orange-handdrawn-marker-banner.png", "05578f7080c38ad03464c7e09678a42ef0a67af8c1e73f163637585e8bda1735", "hand-drawn README and Orange Book horizontal lockup on a light background", "2026-07-14"),
    )
]
GATE0_BRAND_ASSET_METADATA = {
    "orange-banner2.PNG": ("image/png", 2048, 683, False, False),
    "orangePNG.PNG": ("image/png", 1254, 1254, False, False),
    "orange-banner-jpeg.JPEG": ("image/jpeg", 2048, 683, False, False),
    "orange-banner2-erased.PNG": ("image/png", 1444, 683, True, False),
    "orange-erased.PNG": ("image/png", 1254, 1254, True, False),
    "orange-banner.png": ("image/png", 2172, 724, False, True),
    "orange.jpg": ("image/jpeg", 1254, 1254, False, False),
    "orange.png": ("image/png", 1254, 1254, False, True),
    "orange-handdrawn-marker-banner.png": ("image/png", 2048, 682, False, False),
}
GATE0_BRAND_SOURCE_FILENAMES = {
    "orange-banner2.PNG": "1131687B-1CF6-405A-ABC6-0AF8DA9EBAC9.PNG",
    "orangePNG.PNG": "4DB7A71A-8FF8-48B3-8243-1657017AD816.PNG",
    "orange-banner-jpeg.JPEG": "IMG_2760.JPEG",
    "orange-banner2-erased.PNG": "IMG_2766.PNG",
    "orange-erased.PNG": "IMG_2768.PNG",
    "orange-banner.png": "orange-banner.png",
    "orange.jpg": "orange.jpg",
    "orange.png": "orange.png",
    "orange-handdrawn-marker-banner.png": "orange-handdrawn-marker-banner.png",
}
GATE0_EXECUTABLE_PATHS = set(
    """scripts/ci/check-external-links scripts/ci/check-repository
scripts/ci/install-actionlint scripts/ci/install-lychee tools/validate_foundation.py""".split()
)
GATE0_ALLOWED_WRITE_PERMISSIONS = {_SC: {"security-events"}}
GATE0_HOSTED_REPOSITORY_CONTROLS = {
    "snapshot_date": _D,
    "review_due_date": "2026-10-11",
    "main_ruleset_id": 18810248,
    "required_checks": [
        {"context": "Required CI / docs-policy-workflows", "integration_id": 15368},
        {"context": "Dependency Review / policy", "integration_id": 15368},
    ],
}
_SP = set(
    """schemas/gate0/claim-record-v0.1.schema.json schemas/gate0/evidence-manifest-v0.1.schema.json
schemas/gate0/repository-control-snapshot-v0.1.schema.json
schemas/gate0/standards-provenance-v0.1.schema.json schemas/gate0/trust-inventory-v0.1.schema.json""".split()
)
_WI = set(
    "ci.yml dependency-review.yml external-links.yml scorecard.yml workflow-online-audit.yml".split()
)
_PHD = "422f495385bb05de1b1e60133b60233e0f57d9db20d3a6c9d2cad936f603e466"
_CR = (
    "run: /usr/bin/env -u BASH_ENV -u ENV -u GNUMAKEFLAGS -u MAKEFLAGS -u MAKEFILES "
    "-u MAKEOVERRIDES -u MFLAGS /usr/bin/make --no-builtin-rules --no-builtin-variables check-compiler"
)
_PTR = (
    "run: pycache=\"$(/usr/bin/mktemp -d -- \"$RUNNER_TEMP/orange-python-cache.XXXXXXXX\")\"; "
    "pycache=\"$(CDPATH= cd -- \"$pycache\" && pwd -P)\"; trap '/usr/bin/rm -rf -- \"$pycache\"' EXIT; "
    "/usr/bin/env -i HOME=\"$HOME\" LANG=C LC_ALL=C PATH=\"$PATH\" PYTHONHASHSEED=0 "
    "PYTHONPYCACHEPREFIX=\"$pycache\" TZ=UTC python3 -S -P -B -X utf8 -W error::ResourceWarning -c 'import sys, unittest; "
    "sys.path.insert(0, \".\"); unittest.main(module=None)' discover -s tools/tests -p 'test_*.py'"
)
_PR = (
    "run: /usr/bin/env -i HOME=\"$HOME\" LANG=C LC_ALL=C PATH=\"$PATH\" PYTHONHASHSEED=0 "
    "TZ=UTC python3 -S -P -B -X utf8 -W error::ResourceWarning tools/validate_foundation.py"
)
_CSH = "4537523a0e41cc55912ad1013e6a74777ffad8def7015c4ffd51cfc3aeae3c9f"
_FI = tuple(f"F-{index:02d}" for index in range(1, 15))
_PI = tuple(f"P-{index:02d}" for index in range(1, 6))
_JI = tuple(f"J-{index:02d}" for index in range(1, 9))
_OI = tuple("install specify implement prove build inspect integrate update revoke offline-replay".split())
_CCI = tuple(
    """claim-record-valid claim-record-assumption-only-satisfied evidence-manifest-valid
evidence-manifest-network-enabled evidence-manifest-path-escape
evidence-manifest-independent-without-review repository-control-snapshot-valid
repository-control-disabled-without-explanation repository-control-selected-actions-empty
standards-provenance-valid standards-provenance-malformed-digest
standards-provenance-reviewed-without-reference trust-inventory-valid
trust-inventory-authority-without-identity""".split()
)
_CIP = set(
    """conformance/foundation/invalid/claim-record-assumption-only.json
conformance/foundation/invalid/evidence-manifest-independent-without-review.json
conformance/foundation/invalid/evidence-manifest-network-enabled.json
conformance/foundation/invalid/evidence-manifest-path-escape.json
conformance/foundation/invalid/repository-control-missing-explanation.json
conformance/foundation/invalid/repository-control-selected-actions-empty.json
conformance/foundation/invalid/standards-provenance-bad-digest.json
conformance/foundation/invalid/standards-provenance-reviewed-without-reference.json
conformance/foundation/invalid/trust-inventory-missing-identity.json
conformance/foundation/valid/claim-record.json
conformance/foundation/valid/evidence-manifest.json
conformance/foundation/valid/repository-control-snapshot.json
conformance/foundation/valid/standards-provenance.json
conformance/foundation/valid/trust-inventory.json""".split()
)
GATE0_RUST_TOOLCHAIN = {
    "toolchain": {
        "channel": "1.96.1",
        "components": ["clippy", "rustfmt"],
        "profile": "minimal",
    },
}
GATE0_RUST_MANIFESTS = {
    _CT: {
        _WS: {
            "members": [
                "crates/orange-compiler",
                "crates/orangec",
            ],
            "resolver": "2",
            "package": {
                "version": "0.0.1",
                "edition": "2024",
                "rust-version": "1.96.1",
                "publish": False,
            },
            "lints": {
                "rust": {"missing_docs": "deny", "unsafe_code": "forbid"},
                "clippy": {"all": "deny"},
            },
        },
        "profile": {
            "release": {
                "debug-assertions": True,
                "overflow-checks": True,
            },
        },
    },
    _OCM: {
        "package": {
            "name": _OC,
            "description": "Permanent compiler foundations for the Orange language",
            "version": {_WS: True},
            "edition": {_WS: True},
            "rust-version": {_WS: True},
            "publish": {_WS: True},
        },
        "lints": {_WS: True},
    },
    _CCM: {
        "package": {
            "name": "orangec",
            "description": "Command-line frontend for the Orange compiler",
            "version": {_WS: True},
            "edition": {_WS: True},
            "rust-version": {_WS: True},
            "publish": {_WS: True},
        },
        _DS: {
            _OC: {"path": "../orange-compiler"},
        },
        "lints": {_WS: True},
    },
}
GATE0_RUST_MANIFEST_PACKAGES = {
    _CT: None,
    _OCM: _OC,
    _CCM: "orangec",
}
GATE0_RUST_WORKSPACE_MEMBERS = [
    "crates/orange-compiler",
    "crates/orangec",
]
GATE0_RUST_DEPENDENCY_TABLES = {
    _CT: {},
    _OCM: {},
    _CCM: {
        _DS: {
            _OC: {"path": "../orange-compiler"},
        },
    },
}
GATE0_RUST_LOCK = {
    "version": 4,
    "package": [
        {"name": _OC, "version": "0.0.1"},
        {
            "name": "orangec",
            "version": "0.0.1",
            _DS: [_OC],
        },
    ],
}
_RB = {
    "compiler/crates/orange-compiler/src/source.rs": {"MAX_SOURCE_BYTES": 16 * 1024 * 1024},
    "compiler/crates/orange-compiler/src/lexer.rs": {
        "MAX_TOKENS_PER_SOURCE": 262_144,
        "MAX_DIAGNOSTICS_PER_SOURCE": 100,
    },
    "compiler/crates/orange-compiler/src/parser.rs": {
        "MAX_PARSE_DIAGNOSTICS_PER_SOURCE": 100,
        "MAX_SYNTAX_NODES_PER_SOURCE": 262_144,
        "MAX_PARSE_EVENTS_PER_SOURCE": 1_048_576,
        "MAX_RECOVERY_DELIMITER_DEPTH": 64,
    },
    "compiler/crates/orange-compiler/src/semantics.rs": {
        "MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE": 100,
        "MAX_CORE_NODES_PER_SOURCE": 262_144,
        "MAX_SEMANTIC_EVENTS_PER_SOURCE": 1_048_576,
        "MAX_INTEGER_BITS": 16_384,
    },
    "compiler/crates/orange-compiler/src/eval.rs": {"MAX_EVALUATION_STEPS_PER_SOURCE": 1_048_576},
}
_RM = {
    "docs/LANGUAGE_2026.md": {
        "at most 16 MiB\n(`16 * 1024 * 1024` bytes)": 16 * 1024 * 1024,
        "At most 262,144 non-trivia tokens": 262_144,
        "At most 100 ordinary lexical diagnostics": 100,
        "262,144 syntax nodes": 262_144,
        "1,048,576 parser events or equivalent syntax elements": 1_048_576,
        "100 ordinary parse diagnostics plus at most one suppression diagnostic": 100,
        "recovery delimiter nesting depth 64": 64,
    },
    "docs/SEMANTICS_2026.md": {
        "100 ordinary semantic diagnostics followed by at most one suppression\n  diagnostic": 100,
        "262,144 Typed Reference Core nodes": 262_144,
        "1,048,576 semantic events": 1_048_576,
        "16,384 significant bits in any decoded integer magnitude": 16_384,
        "1,048,576 reference-evaluation steps": 1_048_576,
    },
}
_OB = {
    "compiler/crates/orangec/src/main.rs": {
        "MAX_SOURCES_PER_INVOCATION": 256,
        "MAX_ARGUMENT_BYTES_PER_INVOCATION": 4 * 1024 * 1024,
        "MAX_SOURCE_BYTES_PER_INVOCATION": 64 * 1024 * 1024,
        "MAX_STANDARD_OUTPUT_BYTES": 64 * 1024 * 1024,
        "MAX_STANDARD_ERROR_BYTES": 64 * 1024 * 1024,
    },
}
_OM = {
    "compiler/README.md": {
        "`orangec` accepts up to 256 source inputs in argument order": 256,
        "Argument parsing\ninspects at most 4 MiB (`4 * 1024 * 1024` bytes) of encoded command-line\narguments per invocation": 4 * 1024 * 1024,
        "`orangec` buffers at most\n64 MiB (`64 * 1024 * 1024` bytes) across all source operands per invocation": 64 * 1024 * 1024,
        "`orangec` caps standard output at 64 MiB (`64 * 1024 * 1024` bytes)": 64 * 1024 * 1024,
        "`orangec` caps standard error at 64 MiB (`64 * 1024 * 1024` bytes)": 64 * 1024 * 1024,
    },
}
MINIMUM_CODEOWNERS = set(
    """* @chasebryan
/.github/ @chasebryan
/assets/brand/ @chasebryan
/SECURITY.md @chasebryan
/GOVERNANCE.md @chasebryan
/docs/ASSURANCE.md @chasebryan
/docs/security/ @chasebryan
/policy/ @chasebryan
/scripts/ci/ @chasebryan
/tools/validate_foundation.py @chasebryan""".splitlines()
)
GATE0_ALLOWED_TOP_LEVEL = set(
    """.editorconfig .gitattributes .github .gitignore .markdownlint-cli2.jsonc
CODE_OF_CONDUCT.md CONTRIBUTING.md compiler DEPENDENCY_POLICY.md GOVERNANCE.md Makefile
README.md RELEASE_POLICY.md rust-toolchain.toml SECURITY.md SUPPORT.md assets conformance
docs policy schemas scripts tools""".split()
)
ACTION_RE = re.compile(
    r"^\s*(?:-\s*)?uses:\s*([^\s@#]+)@([^\s#]+)"
    r"(?:\s+#\s*([^\s]+)(?:\s+.*)?)?\s*$"
)
CONTAINER_ACTION_RE = re.compile(
    r"^\s*(?:-\s*)?uses:\s*(docker://[^\s#]+)"
    r"(?:\s+#\s*([^\s]+)(?:\s+.*)?)?\s*$"
)
MARKDOWN_REFERENCE_RE = re.compile(
    r"(?m)^ {0,3}\[("
    r"(?:\\[\x21-\x2f\x3a-\x40\x5b-\x60\x7b-\x7e]|"
    r"\\(?=[^\x21-\x2f\x3a-\x40\x5b-\x60\x7b-\x7e])|[^\[\]\\])+"
    r")\]:[ \t]*(?:(?:\r\n?|\n)[ \t]*)?"
    r"(<(?:\\[^\r\n]|[^\\<>\r\n])*>|[^\s]+)"
)
MARKDOWN_CONTINUED_TITLE_RE = re.compile(
    r"\]\([ \t]*(<(?:\\[^\r\n]|[^\\<>\r\n])*>|[^()\s]+)"
    r"[ \t]*(?:\r\n?|\n)[ \t]*(?=[\"'()])"
)
HEADING_RE = re.compile(r"^(#{1,6})\s+(.+?)\s*#*\s*$")
FRONT_MATTER_KEY_RE = re.compile(r"^([a-z][a-z0-9-]*):(?:\s*(.*))?$")
RECORD_FILENAME_RE = re.compile(r"^(?P<prefix>OEP|ADR)-(?P<number>[0-9]{4})-(?P<slug>[a-z0-9]+(?:-[a-z0-9]+)*)\.md$")


class DuplicateKeyError(ValueError):
    pass


@dataclasses.dataclass(frozen=True, order=True)
class Finding:
    code: str
    path: str
    message: str

    def as_dict(self) -> dict[str, str]:
        return dataclasses.asdict(self)


def _text_report_field(value: str) -> str:
    return "".join(
        c if " " <= c <= "~" and c not in "\\:" else f"\\U{ord(c):08x}" for c in value
    )


@dataclasses.dataclass(frozen=True)
class SchemaIssue:
    keyword: str
    instance_path: str
    message: str


class _BoundedSchemaIssues(list[SchemaIssue]):
    def append(self, issue: SchemaIssue) -> None:
        if len(self) < _MF:
            super().append(issue)

    def extend(self, issues: Iterable[SchemaIssue]) -> None:
        for issue in issues:
            self.append(issue)
            if len(self) == _MF:
                break


def _object_without_duplicates(pairs: Sequence[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise DuplicateKeyError(f"duplicate object key: {key}")
        result[key] = value
    return result


def _reject_non_json_constant(value: str) -> Any:
    raise json.JSONDecodeError(f"non-finite number {value!r} is not valid JSON", value, 0)


def _reject_floating_point(value: str) -> Any:
    raise json.JSONDecodeError(f"floating-point number {value!r} is forbidden by the Gate 0 profile", value, 0)


def _parse_i_json_integer(value: str) -> int:
    magnitude = value[1:] if value.startswith("-") else value
    if len(magnitude) > len(_JM) or (
        len(magnitude) == len(_JM)
        and magnitude > _JM
    ):
        raise json.JSONDecodeError(
            "integer exceeds the I-JSON interoperable range",
            value,
            0,
        )
    return int(value)


def _require_unicode_scalars(value: Any, source: str) -> None:
    if isinstance(value, str):
        try:
            value.encode("utf-8", errors="strict")
        except UnicodeEncodeError as exc:
            raise json.JSONDecodeError("lone Unicode surrogate is forbidden by I-JSON", source, exc.start) from exc
    elif isinstance(value, list):
        for item in value:
            _require_unicode_scalars(item, source)
    elif isinstance(value, dict):
        for key, item in value.items():
            _require_unicode_scalars(key, source)
            _require_unicode_scalars(item, source)


def _require_bounded_json_nesting(source: str) -> None:
    depth = 0
    in_string = False
    escaped = False
    for index, character in enumerate(source):
        if in_string:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                in_string = False
            continue
        if character == '"':
            in_string = True
        elif character in "[{":
            depth += 1
            if depth > GATE0_MAXIMUM_JSON_NESTING_DEPTH:
                raise json.JSONDecodeError(
                    "JSON structural nesting exceeds the Gate 0 limit "
                    f"of {GATE0_MAXIMUM_JSON_NESTING_DEPTH}",
                    source,
                    index,
                )
        elif character in "]}" and depth:
            depth -= 1


def _load_json_bytes(data: bytes) -> Any:
    source = data.decode("utf-8")
    _require_bounded_json_nesting(source)
    try:
        result = json.loads(
            source,
            object_pairs_hook=_object_without_duplicates,
            parse_constant=_reject_non_json_constant,
            parse_float=_reject_floating_point,
            parse_int=_parse_i_json_integer,
        )
        _require_unicode_scalars(result, source)
    except RecursionError as exc:
        raise json.JSONDecodeError(
            "JSON structural nesting exceeds the Gate 0 limit "
            f"of {GATE0_MAXIMUM_JSON_NESTING_DEPTH}",
            source,
            0,
        ) from exc
    return result


def load_json(path: Path) -> Any:
    return _load_json_bytes(path.read_bytes())


def canonical_json_bytes(value: Any) -> bytes:
    _require_unicode_scalars(value, "")

    def serialize(item: Any) -> str:
        if item is None:
            return "null"
        if item is True:
            return "true"
        if item is False:
            return "false"
        if isinstance(item, int):
            if not -(2**53) + 1 <= item <= 2**53 - 1:
                raise ValueError("integer exceeds the I-JSON interoperable range")
            return str(item)
        if isinstance(item, float):
            raise ValueError("floating-point values are forbidden by the Gate 0 profile")
        if isinstance(item, str):
            return json.dumps(item, ensure_ascii=False, separators=(",", ":"))
        if isinstance(item, list):
            return "[" + ",".join(serialize(value) for value in item) + "]"
        if isinstance(item, dict):
            if not all(isinstance(key, str) for key in item):
                raise TypeError("JSON object names must be strings")
            keys = sorted(item, key=lambda key: key.encode("utf-16-be"))
            return "{" + ",".join(f"{serialize(key)}:{serialize(item[key])}" for key in keys) + "}"
        raise TypeError(f"unsupported JSON value {type(item).__name__}")

    return serialize(value).encode("utf-8")


def relative(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return path.as_posix()


def _secure_repository_reads_supported() -> bool:
    return (
        os.name == "posix"
        and all(
            isinstance(getattr(os, name, None), int)
            for name in ("O_DIRECTORY", "O_NOFOLLOW", "O_NONBLOCK", "SEEK_HOLE")
        )
        and os.open in os.supports_dir_fd
        and os.stat in os.supports_dir_fd
        and os.stat in os.supports_follow_symlinks
    )


def _secure_repository_discovery_supported() -> bool:
    return _secure_repository_reads_supported() and os.scandir in os.supports_fd


def _open_directory_descriptor(root: Path | bytes, parts: Sequence[str | bytes]) -> int:
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | os.O_DIRECTORY | os.O_NOFOLLOW
    descriptor = os.open(root, flags)
    try:
        for part in parts:
            next_descriptor = os.open(part, flags, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = next_descriptor
        return descriptor
    except (NotImplementedError, OSError):
        try:
            os.close(descriptor)
        except OSError:
            pass
        raise


def _repository_entry_metadata(root: Path, raw_path: bytes) -> os.stat_result | bool | None:
    if not _secure_repository_reads_supported():
        return None
    descriptor: int | None = None
    try:
        parts = raw_path.split(b"/")
        descriptor = _open_directory_descriptor(root, parts[:-1])
        return os.stat(parts[-1], dir_fd=descriptor, follow_symlinks=False)
    except FileNotFoundError:
        return False
    except (NotImplementedError, OSError):
        return None
    finally:
        if descriptor is not None:
            try:
                os.close(descriptor)
            except OSError:
                pass


def _repository_entry_presence(root: Path, raw_path: bytes) -> bool | None:
    metadata = _repository_entry_metadata(root, raw_path)
    return metadata if metadata is False or metadata is None else True


@dataclasses.dataclass(frozen=True)
class _GitRecordRead:
    records: tuple[bytes, ...] | None
    finding: Finding | None = None


def _sanitized_git_environment(root: Path) -> dict[str, str]:
    environment = {"PATH": "/usr/bin:/bin"}
    environment.update(_GATE0_GIT_FIXED_ENVIRONMENT)
    environment["GIT_DIR"] = str(root / ".git")
    environment["GIT_WORK_TREE"] = str(root)
    return environment


def _stop_git_process(process: subprocess.Popen[bytes], timeout: float = 0.0) -> None:
    try:
        process.kill()
    except OSError:
        pass
    if process.stdout is not None:
        try:
            process.stdout.close()
        except OSError:
            pass
    try:
        process.wait(timeout=max(0.0, timeout))
    except (OSError, subprocess.TimeoutExpired):
        pass


def _read_git_records(
    root: Path,
    arguments: Sequence[str],
    *,
    maximum_record_bytes: int,
    input_records: Sequence[bytes] = (),
    terminator: bytes = b"\0",
    flag: str | None = "-z",
) -> _GitRecordRead:
    command = [
        GATE0_GIT_EXECUTABLE,
        "-c",
        "core.fsmonitor=false",
        "-c",
        "core.ignoreCase=false",
        "-c",
        "core.precomposeUnicode=false",
        "-C",
        str(root),
        *arguments,
        *((flag,) if flag else ()),
    ]
    input_file = None
    if input_records:
        try:
            input_file = tempfile.TemporaryFile()
            input_bytes = b"\0".join(input_records) + b"\0"
            if input_file.write(input_bytes) != len(input_bytes):
                raise OSError("short Git input write")
            input_file.seek(0)
        except OSError:
            if input_file is not None:
                try:
                    input_file.close()
                except OSError:
                    pass
            return _GitRecordRead(None)
    try:
        process = subprocess.Popen(
            command,
            env=_sanitized_git_environment(root),
            stdin=input_file or subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
    except OSError:
        if input_file is not None:
            try:
                input_file.close()
            except OSError:
                pass
        return _GitRecordRead(None)
    cleanup_deadline = time.monotonic() + _GT
    deadline = cleanup_deadline - min(1.0, _GT / 2.0)
    if input_file is not None:
        try:
            input_file.close()
        except OSError as exc:
            _stop_git_process(process, cleanup_deadline - time.monotonic())
            return _GitRecordRead(
                None,
                Finding(_RI_READ, ".", f"cannot close Git input: {exc}"),
            )
    if process.stdout is None:
        _stop_git_process(process, cleanup_deadline - time.monotonic())
        return _GitRecordRead(
            None,
            Finding(_IP, ".", "Git inventory did not expose a stdout stream"),
        )

    records: list[bytes] = []
    pending = bytearray()
    raw_bytes = 0

    def reject(code: str, message: str) -> _GitRecordRead:
        _stop_git_process(process, cleanup_deadline - time.monotonic())
        return _GitRecordRead(None, Finding(code, ".", message))

    try:
        output_descriptor = process.stdout.fileno()
    except (AttributeError, OSError, ValueError):
        return reject(_IP, "Git inventory stdout has no usable descriptor")

    try:
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not select.select((process.stdout,), (), (), remaining)[0]:
                return reject("resource.inventory_timeout", "Git inventory exceeded its deadline")
            chunk = os.read(output_descriptor, _GC)
            if not chunk:
                break
            if raw_bytes + len(chunk) > GATE0_MAXIMUM_RAW_PATH_METADATA_BYTES:
                return reject(
                    "resource.inventory_metadata",
                    "Git path metadata exceeds the Gate 0 raw-byte limit",
                )
            raw_bytes += len(chunk)
            offset = 0
            while True:
                end = chunk.find(terminator, offset)
                if end < 0:
                    tail = chunk[offset:]
                    if len(pending) + len(tail) > maximum_record_bytes:
                        return reject(
                            "resource.inventory_path",
                            "Git inventory record exceeds its Gate 0 byte limit",
                        )
                    pending.extend(tail)
                    break
                segment = chunk[offset:end]
                if len(pending) + len(segment) > maximum_record_bytes:
                    return reject(
                        "resource.inventory_path",
                        "Git inventory record exceeds its Gate 0 byte limit",
                    )
                pending.extend(segment)
                if not pending:
                    return reject(
                        _IP,
                        "Git inventory contains an empty NUL-delimited record",
                    )
                records.append(bytes(pending))
                pending.clear()
                if len(records) > GATE0_MAXIMUM_REPOSITORY_FILES:
                    return reject(
                        "resource.inventory_count",
                        "Git inventory exceeds the Gate 0 repository-file limit",
                    )
                offset = end + len(terminator)
    except (OSError, ValueError) as exc:
        return reject(_RI_READ, f"cannot read bounded Git inventory: {exc}")

    try:
        process.stdout.close()
    except OSError as exc:
        _stop_git_process(process, cleanup_deadline - time.monotonic())
        return _GitRecordRead(
            None,
            Finding(_RI_READ, ".", f"cannot close bounded Git inventory: {exc}"),
        )
    try:
        return_code = process.wait(timeout=max(0.0, deadline - time.monotonic()))
    except subprocess.TimeoutExpired:
        return reject("resource.inventory_timeout", "Git inventory exceeded its deadline")
    except OSError as exc:
        _stop_git_process(process, cleanup_deadline - time.monotonic())
        return _GitRecordRead(
            None,
            Finding(_RI_READ, ".", f"cannot wait for bounded Git inventory: {exc}"),
        )
    if return_code != 0:
        return _GitRecordRead(None)
    if pending:
        return _GitRecordRead(
            None,
            Finding(
                _IP,
                ".",
                "Git inventory ended with an unterminated record",
            ),
        )
    return _GitRecordRead(tuple(records))


def _fallback_repository_files(root: Path, findings: list[Finding]) -> list[Path]:
    if not _secure_repository_discovery_supported():
        findings.append(
            Finding(
                _RU,
                ".",
                "host cannot provide component-relative no-follow repository discovery",
            )
        )
        return []
    raw_root = os.fsencode(root)
    ignored = {os.fsencode(part) for part in IGNORED_PARTS}
    stack: list[bytes] = [b""]
    raw_files: list[bytes] = []
    directory_entries = 0
    raw_metadata_bytes = 0
    while stack:
        relative_directory = stack.pop()
        descriptor: int | None = None
        try:
            parts = relative_directory.split(b"/") if relative_directory else ()
            descriptor = _open_directory_descriptor(raw_root, parts)
            iterator = os.scandir(descriptor)
        except OSError as exc:
            if descriptor is not None:
                try:
                    os.close(descriptor)
                except OSError:
                    pass
            findings.append(
                Finding(_RI_READ, ".", f"cannot scan repository inventory: {exc}")
            )
            return []
        try:
            with iterator:
                for entry in iterator:
                    directory_entries += 1
                    if directory_entries > GATE0_MAXIMUM_FALLBACK_DIRECTORY_ENTRIES:
                        findings.append(
                            Finding(
                                "resource.inventory_entries",
                                ".",
                                "filesystem inventory exceeds the Gate 0 directory-entry limit",
                            )
                        )
                        return []
                    name = os.fsencode(entry.name)
                    raw_path = name if not relative_directory else relative_directory + b"/" + name
                    if len(raw_path) > GATE0_MAXIMUM_REPOSITORY_PATH_BYTES:
                        findings.append(
                            Finding(
                                "resource.inventory_path",
                                ".",
                                "filesystem inventory path exceeds the Gate 0 byte limit",
                            )
                        )
                        return []
                    if raw_metadata_bytes + len(raw_path) + 1 > GATE0_MAXIMUM_RAW_PATH_METADATA_BYTES:
                        findings.append(
                            Finding(
                                "resource.inventory_metadata",
                                ".",
                                "filesystem path metadata exceeds the Gate 0 raw-byte limit",
                            )
                        )
                        return []
                    raw_metadata_bytes += len(raw_path) + 1
                    if name == b"__pycache__" or (not relative_directory and name in ignored):
                        continue
                    try:
                        is_directory = entry.is_dir(follow_symlinks=False)
                        is_file = entry.is_file(follow_symlinks=False)
                        is_symlink = entry.is_symlink()
                    except OSError as exc:
                        findings.append(
                            Finding(
                                _RI_READ,
                                ".",
                                f"cannot inspect filesystem inventory entry: {exc}",
                            )
                        )
                        return []
                    if is_directory:
                        stack.append(raw_path)
                    elif is_file or is_symlink or not is_directory:
                        raw_files.append(raw_path)
                        if len(raw_files) > GATE0_MAXIMUM_REPOSITORY_FILES:
                            findings.append(
                                Finding(
                                    "resource.inventory_count",
                                    ".",
                                    "filesystem inventory exceeds the Gate 0 repository-file limit",
                                )
                            )
                            return []
        except OSError as exc:
            findings.append(
                Finding(_RI_READ, ".", f"cannot scan repository inventory: {exc}")
            )
            return []
        finally:
            if descriptor is not None:
                try:
                    os.close(descriptor)
                except OSError:
                    pass
    try:
        return [root / raw_path.decode("utf-8") for raw_path in sorted(raw_files)]
    except UnicodeDecodeError:
        findings.append(
            Finding(
                _IE,
                ".",
                "filesystem inventory contains a path that is not valid UTF-8",
            )
        )
        return []


def _git_path_is_relative(raw_path: bytes) -> bool:
    return bool(raw_path) and not raw_path.startswith(b"/") and all(
        part not in {b"", b".", b".."} for part in raw_path.split(b"/")
    )


def _git_object_id_is_valid(value: bytes) -> bool:
    return len(value) in {40, 64} and all(byte in b"0123456789abcdef" for byte in value)


def _repository_file_inventory(root: Path, findings: list[Finding]) -> tuple[list[Path], bool]:
    if not _secure_repository_reads_supported():
        findings.append(
            Finding(_RU, ".", "host cannot securely inspect repository metadata")
        )
        return [], False
    git_metadata = _repository_entry_metadata(root, b".git")
    git_metadata_present = git_metadata if git_metadata is False or git_metadata is None else True
    if git_metadata_present and (
        git_metadata is None or not stat.S_ISDIR(git_metadata.st_mode)
    ):
        findings.append(
            Finding(
                _RI_GIT,
                ".git",
                "repository Git metadata must be a local directory",
            )
        )
        return [], False
    for raw_path, required in ((b".git/config", True), (b".git/index", False)):
        metadata = _repository_entry_metadata(root, raw_path)
        present = metadata if metadata is False or metadata is None else True
        if git_metadata_present and (
            (required and present is not True)
            or (not required and present is None)
            or (
                present is True
                and (not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1)
            )
        ):
            findings.append(
                Finding(
                    _RI_GIT,
                    os.fsdecode(raw_path),
                    "repository Git configuration and index must be local, singly linked regular files",
                )
            )
            return [], False
    worktree_config_present = False
    for raw_path in (b".git/commondir", b".git/config.worktree", b".git/objects/info/alternates"):
        metadata = _repository_entry_metadata(root, raw_path)
        local_config = raw_path == b".git/config.worktree" and isinstance(metadata, os.stat_result) and stat.S_ISREG(metadata.st_mode) and metadata.st_nlink == 1
        worktree_config_present |= local_config
        if git_metadata_present and metadata is not False and not local_config:
            findings.append(
                Finding(
                    _RI_GIT,
                    os.fsdecode(raw_path),
                    "repository Git metadata must not redirect shared state or objects",
                )
            )
            return [], False
    for arguments, terminator, expected in (
        ("rev-parse --shared-index-path".split(), b"\n", ((),)),
        (r"config -z --local --no-includes --get-regexp ^include(\.|if\.)".split(), b"\0", (None,)),
        ("config -z --file .git/config.worktree --list".split(), b"\0", (((), (b"core.sparsecheckout\nfalse", b"core.sparsecheckoutcone\nfalse", b"index.sparse\nfalse")) if worktree_config_present else (None,))),
    ):
        if not git_metadata_present:
            break
        indirect = _read_git_records(
            root,
            arguments,
            maximum_record_bytes=GATE0_MAXIMUM_REPOSITORY_PATH_BYTES,
            terminator=terminator,
            flag=None,
        )
        if indirect.finding is not None:
            findings.append(indirect.finding)
            return [], False
        if indirect.records not in expected:
            findings.append(
                Finding(_RI_GIT, ".git", "Git metadata indirection is not admitted")
            )
            return [], False
    result = _read_git_records(
        root,
        [
            "ls-files",
            "--cached",
            "--others",
            "-v",
            *(f"--exclude={pattern}" for pattern in GATE0_IGNORE_PATTERNS),
        ],
        maximum_record_bytes=GATE0_MAXIMUM_REPOSITORY_PATH_BYTES + 2,
    )
    if result.finding is not None:
        findings.append(result.finding)
        return [], False
    if result.records is None:
        git_metadata_present = _repository_entry_presence(root, b".git")
        if git_metadata_present is None:
            findings.append(
                Finding(
                    _RI_GIT,
                    ".",
                    "cannot securely inspect repository Git metadata after inventory failure",
                )
            )
            return [], False
        if git_metadata_present:
            findings.append(
                Finding(
                    _RI_GIT,
                    ".",
                    "Git inventory failed even though repository metadata is present",
                )
            )
            return [], False
        return _fallback_repository_files(root, findings), False
    if any(len(value) < 3 or value[1:2] != b" " for value in result.records):
        findings.append(Finding(_IP, ".", "Git file inventory tag is malformed"))
        return [], False
    if any(value[:1] not in {b"H", b"?"} for value in result.records):
        findings.append(Finding("git.index_flags", ".", "Git index contains hidden worktree state"))
        return [], False
    records = tuple(value[2:] for value in result.records)
    if any(not _git_path_is_relative(value) for value in records):
        findings.append(
            Finding(
                _IP,
                ".",
                "Git inventory contains a non-relative repository path",
            )
        )
        return [], False
    if len(set(records)) != len(records):
        findings.append(
            Finding(
                _IP,
                ".",
                "Git inventory contains a duplicate repository path",
            )
        )
        return [], False
    try:
        paths = [root / value.decode("utf-8") for value in sorted(records)]
    except UnicodeDecodeError:
        findings.append(
            Finding(
                _IE,
                ".",
                "Git inventory contains a repository path that is not valid UTF-8",
            )
        )
        return [], False
    missing = next(
        (
            path
            for raw_path, path in zip(sorted(records), paths, strict=True)
            if _repository_entry_presence(root, raw_path) is False
        ),
        None,
    )
    if missing is not None:
        findings.append(
            Finding(
                "resource.inventory_missing",
                relative(missing, root),
                "Git inventory names a repository entry that is absent from the worktree",
            )
        )
        return [], False
    return paths, True


def iter_repository_files(root: Path, findings: list[Finding] | None = None) -> Iterable[Path]:
    inventory_findings = findings if findings is not None else []
    paths, _ = _repository_file_inventory(root, inventory_findings)
    return paths


def git_index_entries(
    root: Path,
    findings: list[Finding] | None = None,
    *,
    required: bool = False,
) -> list[tuple[str, bytes, str]]:
    inventory_findings = findings if findings is not None else []
    result = _read_git_records(
        root,
        ["ls-files", "--format=%(objectmode) %(objectname) %(stage) %(objectsize)%x09%(path)"],
        maximum_record_bytes=(
            GATE0_MAXIMUM_GIT_STAGE_PREFIX_BYTES + 1 + GATE0_MAXIMUM_REPOSITORY_PATH_BYTES
        ),
    )
    if result.finding is not None:
        inventory_findings.append(result.finding)
        return []
    if result.records is None:
        if required:
            inventory_findings.append(
                Finding(
                    "resource.inventory_stage",
                    ".",
                    "Git stage inventory is unavailable after a successful file inventory",
                )
            )
        return []

    raw_entries: list[tuple[bytes, bytes, bytes]] = []
    seen_paths: set[bytes] = set()
    for record in result.records:
        metadata, separator, raw_path = record.partition(b"\t")
        fields = metadata.split()
        if (
            not separator
            or not _git_path_is_relative(raw_path)
            or len(metadata) > GATE0_MAXIMUM_GIT_STAGE_PREFIX_BYTES
            or len(raw_path) > GATE0_MAXIMUM_REPOSITORY_PATH_BYTES
            or len(fields) != 4
            or len(fields[0]) != 6
            or any(byte not in b"01234567" for byte in fields[0])
            or fields[2] != b"0"
            or not fields[3].isdigit()
            or not _git_object_id_is_valid(fields[1])
            or raw_path in seen_paths
        ):
            inventory_findings.append(
                Finding(
                    _IP,
                    ".",
                    "Git stage inventory contains a malformed metadata record",
                )
            )
            return []
        seen_paths.add(raw_path)
        raw_entries.append((fields[0], fields[1], raw_path))
    if not raw_entries:
        return []
    try:
        entries = [
            (mode.decode("ascii"), object_id, raw_path.decode("utf-8"))
            for mode, object_id, raw_path in sorted(raw_entries, key=lambda entry: (entry[2], entry[0]))
        ]
    except UnicodeDecodeError:
        inventory_findings.append(
            Finding(
                _IE,
                ".",
                "Git stage inventory contains a repository path that is not valid UTF-8",
            )
        )
        return []
    object_ids = tuple(dict.fromkeys(object_id for _, object_id, _ in raw_entries))
    types = _read_git_records(
        root,
        ["cat-file", "--batch-check=%(objectname) %(objecttype)"],
        maximum_record_bytes=GATE0_MAXIMUM_GIT_STAGE_PREFIX_BYTES,
        input_records=object_ids,
        flag="-Z",
    )
    if types.finding is not None:
        inventory_findings.append(types.finding)
        return []
    if types.records is None:
        if required:
            inventory_findings.append(
                Finding("resource.inventory_stage", ".", "Git object-type inventory is unavailable")
            )
        return []
    actual_types: dict[bytes, bytes] = {}
    for object_id, record in zip(object_ids, types.records, strict=False):
        reported_id, separator, object_type = record.partition(b" ")
        if (
            not separator
            or reported_id != object_id
            or object_type not in {b"blob", b"commit", b"tree", b"tag"}
        ):
            inventory_findings.append(
                Finding(_IP, ".", "Git object-type inventory is malformed")
            )
            return []
        actual_types[object_id] = object_type
    if len(types.records) != len(object_ids):
        inventory_findings.append(
            Finding(_IP, ".", "Git object-type inventory count is incorrect")
        )
        return []
    expected_types = {
        b"100644": b"blob",
        b"100755": b"blob",
        b"120000": b"blob",
        b"160000": b"commit",
    }
    for mode, object_id, path in entries:
        mode = mode.encode("ascii")
        if (expected := expected_types.get(mode)) is not None and actual_types[object_id] != expected:
            inventory_findings.append(
                Finding(
                    "git.index_object_type", path, "Git index mode and object type disagree"
                )
            )
            return []
    return entries


class FoundationValidator:
    def __init__(self, root: Path) -> None:
        self.root = root.resolve()
        self.policy_path = self.root / POLICY_PATH
        self.findings: list[Finding] = []
        self.policy: dict[str, Any] = {}
        self.repository_files, git_inventory_succeeded = _repository_file_inventory(
            self.root, self.findings
        )
        self.index_entries = (
            git_index_entries(self.root, self.findings, required=True)
            if not self.findings and git_inventory_succeeded
            else []
        )
        if not self.findings and git_inventory_succeeded:
            inventory_paths = {relative(path, self.root) for path in self.repository_files}
            stage_paths = {value for _, _, value in self.index_entries}
            unexpected_stage_paths = sorted(stage_paths - inventory_paths)
            if unexpected_stage_paths:
                self.findings.append(
                    Finding(
                        _IP,
                        unexpected_stage_paths[0],
                        "Git stage inventory path is absent from the file inventory",
                    )
                )
            untracked_paths = sorted(inventory_paths - stage_paths)
            if not unexpected_stage_paths and untracked_paths:
                self.findings.append(
                    Finding(
                        "git.untracked",
                        untracked_paths[0],
                        "Git file inventory path has no stage-zero index entry",
                    )
                )
        if not self.findings and git_inventory_succeeded:
            intent = _read_git_records(
                self.root,
                [
                    "diff-files",
                    *(
                        "--ita-invisible-in-index --no-ext-diff --no-textconv "
                        "--ignore-submodules=none --no-renames --name-only --diff-filter=A"
                    ).split(),
                ],
                maximum_record_bytes=GATE0_MAXIMUM_REPOSITORY_PATH_BYTES,
            )
            if intent.finding is not None:
                self.findings.append(intent.finding)
            elif intent.records is None:
                self.findings.append(
                    Finding("resource.inventory_intent", ".", "Git intent inventory is unavailable")
                )
            elif intent.records:
                if any(not _git_path_is_relative(value) for value in intent.records) or len(
                    set(intent.records)
                ) != len(intent.records):
                    self.findings.append(
                        Finding(
                            _IP,
                            ".",
                            "Git intent inventory has malformed paths",
                        )
                    )
                else:
                    try:
                        intent_path = min(intent.records).decode("utf-8")
                    except UnicodeDecodeError:
                        self.findings.append(
                            Finding(
                                _IE,
                                ".",
                                "Git intent path is not valid UTF-8",
                            )
                        )
                    else:
                        self.findings.append(
                            Finding("git.intent_to_add", intent_path, "Git index entry is intent-to-add")
                        )
        self._inventory_has_findings = bool(self.findings)
        self._authenticated_protected_bytes: dict[str, bytes | None] = {}
        self._repository_byte_cache: dict[str, bytes | None] = {}
        self._repository_read_bytes = 0
        self._resource_metadata: dict[str, tuple[int, ...]] = {}
        self._resource_issue_keys: set[tuple[str, str]] = set()
        self._resource_preflight_complete = False

    def add(self, code: str, path: str | Path, message: str) -> None:
        path_text = relative(path, self.root) if isinstance(path, Path) else path
        truncation = "... [truncated]"
        if len(message) > GATE0_MAXIMUM_FINDING_MESSAGE_CHARACTERS:
            message = message[: GATE0_MAXIMUM_FINDING_MESSAGE_CHARACTERS - len(truncation)] + truncation
        if len(self.findings) < _MF:
            self.findings.append(Finding(code, path_text, message))
        elif len(self.findings) == _MF:
            self.findings.append(
                Finding(
                    "resource.finding_count",
                    ".",
                    f"validation retained {_MF} detailed findings; further findings are suppressed",
                )
            )

    def _ri(self, code: str, path: str | Path, message: str) -> None:
        path_text = relative(path, self.root) if isinstance(path, Path) else path
        key = (code, path_text)
        if key not in self._resource_issue_keys:
            self._resource_issue_keys.add(key)
            self.add(code, path_text, message)

    def _inventory_files_in(self, directory: str, *, recursive: bool = False) -> list[Path]:
        prefix = PurePosixPath(directory).as_posix().rstrip("/") + "/"
        selected: list[tuple[str, Path]] = []
        for path in self.repository_files:
            value = relative(path, self.root)
            if not value.startswith(prefix):
                continue
            remainder = value[len(prefix) :]
            if recursive or "/" not in remainder:
                selected.append((value, path))
        return [path for _, path in sorted(selected)]

    def _hf(self, path: Path) -> bool:
        value = relative(path, self.root)
        return any(relative(candidate, self.root) == value for candidate in self.repository_files)

    def _inventory_has_path(self, path: Path) -> bool:
        value = relative(path, self.root).rstrip("/")
        prefix = value + "/"
        return any(
            candidate == value or candidate.startswith(prefix)
            for candidate in (relative(path, self.root) for path in self.repository_files)
        )

    def _lp(self, path: Path) -> tuple[str, Path] | None:
        unnormalized = path if path.is_absolute() else self.root / path
        candidate = Path(os.path.normpath(os.fspath(unnormalized)))
        try:
            lexical = candidate.relative_to(self.root)
        except ValueError:
            self._ri("resource.path_escape", candidate, "content read escapes the repository root")
            return None
        if not lexical.parts or ".." in lexical.parts:
            self._ri("resource.path_escape", candidate, "content read escapes the repository root")
            return None
        return lexical.as_posix(), self.root / lexical

    @staticmethod
    def _ms(metadata: os.stat_result) -> tuple[int, ...]:
        return (
            metadata.st_dev,
            metadata.st_ino,
            metadata.st_mode,
            metadata.st_size,
            metadata.st_mtime_ns,
            metadata.st_ctime_ns,
            metadata.st_nlink,
        )

    @staticmethod
    def _fl(path: Path) -> int:
        if path.suffix.lower() in BINARY_SUFFIXES:
            return GATE0_MAXIMUM_BINARY_FILE_BYTES
        return GATE0_MAXIMUM_TEXT_FILE_BYTES

    def _if(self, path: Path) -> tuple[str, Path, os.stat_result] | None:
        lexical_path = self._lp(path)
        if lexical_path is None:
            return None
        value, candidate = lexical_path
        if not _secure_repository_reads_supported():
            self._ri(
                _RU,
                candidate,
                "host cannot provide component-relative no-follow repository inspection",
            )
            return None
        close_on_exec = getattr(os, "O_CLOEXEC", 0)
        directory_flags = os.O_RDONLY | close_on_exec | os.O_DIRECTORY | os.O_NOFOLLOW
        directory_descriptor: int | None = None
        parts = PurePosixPath(value).parts
        try:
            directory_descriptor = os.open(self.root, directory_flags)
            for index, part in enumerate(parts):
                metadata = os.stat(
                    part,
                    dir_fd=directory_descriptor,
                    follow_symlinks=False,
                )
                current = self.root.joinpath(*parts[: index + 1])
                if stat.S_ISLNK(metadata.st_mode):
                    self._ri(
                        "resource.symlink",
                        candidate,
                        f"repository content traverses symlink {relative(current, self.root)!r}",
                    )
                    return None
                if index + 1 < len(parts):
                    if not stat.S_ISDIR(metadata.st_mode):
                        self._ri(
                            "resource.not_file",
                            candidate,
                            "repository content has a non-directory parent",
                        )
                        return None
                    next_descriptor = os.open(part, directory_flags, dir_fd=directory_descriptor)
                    previous_descriptor = directory_descriptor
                    directory_descriptor = next_descriptor
                    os.close(previous_descriptor)
                elif not stat.S_ISREG(metadata.st_mode):
                    self._ri(
                        "resource.not_file",
                        candidate,
                        "repository content is not a regular file",
                    )
                    return None
                else:
                    return value, candidate, metadata
        except (NotImplementedError, OSError) as exc:
            self._ri(
                "resource.unreadable",
                candidate,
                f"cannot inspect repository file securely: {exc}",
            )
            return None
        finally:
            if directory_descriptor is not None:
                try:
                    os.close(directory_descriptor)
                except OSError:
                    pass
        return None

    def _preflight_repository_resources(self) -> bool:
        if self._resource_preflight_complete:
            return not self._resource_issue_keys
        self._resource_preflight_complete = True
        if not _secure_repository_reads_supported():
            self._ri(
                _RU,
                ".",
                "host cannot provide component-relative no-follow repository reads",
            )
            return False
        total_bytes = 0
        seen: set[str] = set()
        for path in self.repository_files:
            inspected = self._if(path)
            if inspected is None:
                continue
            value, candidate, metadata = inspected
            if value in seen:
                continue
            seen.add(value)
            self._resource_metadata[value] = self._ms(metadata)
            if metadata.st_nlink != 1:
                self._ri(
                    "resource.hardlink",
                    candidate,
                    "repository files must have exactly one filesystem link",
                )
            limit = self._fl(candidate)
            if metadata.st_size > limit:
                kind = "binary" if candidate.suffix.lower() in BINARY_SUFFIXES else "text"
                self._ri(
                    "resource.file_size",
                    candidate,
                    f"{kind} file is {metadata.st_size} bytes; Gate 0 permits at most {limit} bytes",
                )
            descriptor = self._open_repository_descriptor(value, candidate)
            if descriptor is not None:
                try:
                    opened_metadata = os.fstat(descriptor)
                    if self._ms(opened_metadata) != self._ms(metadata):
                        self._ri(
                            _RC,
                            candidate,
                            "repository file changed during resource preflight",
                        )
                    elif metadata.st_size and self._descriptor_is_sparse(
                        descriptor, metadata.st_size
                    ):
                        self._ri(
                            "resource.sparse",
                            candidate,
                            "sparse repository files are not admitted",
                        )
                except OSError as exc:
                    self._ri(
                        _RU,
                        candidate,
                        f"cannot inspect repository file allocation: {exc}",
                    )
                finally:
                    try:
                        os.close(descriptor)
                    except OSError:
                        pass
            total_bytes += metadata.st_size
        if total_bytes > GATE0_MAXIMUM_REPOSITORY_BYTES:
            self._ri(
                "resource.aggregate_size",
                ".",
                f"repository files total {total_bytes} bytes; Gate 0 permits at most {GATE0_MAXIMUM_REPOSITORY_BYTES} bytes",
            )
        return not self._resource_issue_keys

    def _read_repository_bytes(self, path: Path) -> bytes | None:
        lexical_path = self._lp(path)
        if lexical_path is None:
            return None
        value, candidate = lexical_path
        if value in self._repository_byte_cache:
            return self._repository_byte_cache[value]
        self._repository_byte_cache[value] = None

        inspected = self._if(candidate)
        if inspected is None:
            return None
        _, _, metadata = inspected
        signature = self._ms(metadata)
        if self._resource_preflight_complete:
            expected_signature = self._resource_metadata.get(value)
            if expected_signature is None:
                self._ri(
                    "resource.post_preflight_addition",
                    candidate,
                    "repository file appeared after the resource preflight",
                )
                return None
            if signature != expected_signature:
                self._ri(
                    "resource.post_preflight_change",
                    candidate,
                    "repository file metadata changed after the resource preflight",
                )
                return None

        file_limit = self._fl(candidate)
        if metadata.st_size > file_limit:
            self._ri(
                "resource.file_size",
                candidate,
                f"file is {metadata.st_size} bytes; Gate 0 permits at most {file_limit} bytes",
            )
            return None
        aggregate_remaining = GATE0_MAXIMUM_REPOSITORY_BYTES - self._repository_read_bytes
        read_limit = min(file_limit, aggregate_remaining)
        if metadata.st_size > aggregate_remaining:
            self._ri(
                "resource.aggregate_size",
                candidate,
                "reading this file would exceed the Gate 0 aggregate repository byte limit",
            )
            return None

        descriptor = self._open_repository_descriptor(value, candidate)
        if descriptor is None:
            return None
        try:
            with os.fdopen(descriptor, "rb") as source:
                opened_metadata = os.fstat(source.fileno())
                if not stat.S_ISREG(opened_metadata.st_mode) or self._ms(opened_metadata) != signature:
                    self._ri(
                        _RC,
                        candidate,
                        "repository file changed while it was being opened",
                    )
                    return None
                data = source.read(read_limit + 1)
                self._repository_read_bytes += min(len(data), aggregate_remaining)
                closed_metadata = os.fstat(source.fileno())
        except OSError as exc:
            self._ri("resource.unreadable", candidate, f"cannot read repository file: {exc}")
            return None
        if self._ms(closed_metadata) != signature:
            self._ri(
                _RC,
                candidate,
                "repository file changed while it was being read",
            )
            return None
        final = self._if(candidate)
        if final is None:
            return None
        if self._ms(final[2]) != signature:
            self._ri(_RC, candidate, "repository path changed while its file was being read")
            return None
        if len(data) > read_limit:
            code = "resource.file_size" if read_limit == file_limit else "resource.aggregate_size"
            self._ri(code, candidate, "repository content exceeded its bounded read limit")
            return None
        if len(data) != opened_metadata.st_size:
            self._ri(
                _RC,
                candidate,
                "repository file produced a short or inconsistent snapshot read",
            )
            return None
        self._repository_byte_cache[value] = data
        return data

    def _open_repository_descriptor(self, value: str, candidate: Path) -> int | None:
        if not _secure_repository_reads_supported():
            self._ri(
                _RU,
                candidate,
                "host cannot provide component-relative no-follow repository reads",
            )
            return None
        close_on_exec = getattr(os, "O_CLOEXEC", 0)
        file_flags = os.O_RDONLY | close_on_exec | os.O_NOFOLLOW | os.O_NONBLOCK
        directory_descriptor: int | None = None
        try:
            parts = PurePosixPath(value).parts
            directory_descriptor = _open_directory_descriptor(self.root, parts[:-1])
            return os.open(parts[-1], file_flags, dir_fd=directory_descriptor)
        except (NotImplementedError, OSError) as exc:
            self._ri(
                "resource.unreadable",
                candidate,
                f"cannot securely open repository file: {exc}",
            )
            return None
        finally:
            if directory_descriptor is not None:
                try:
                    os.close(directory_descriptor)
                except OSError:
                    pass

    @staticmethod
    def _descriptor_is_sparse(descriptor: int, size: int) -> bool:
        first_hole = os.lseek(descriptor, 0, os.SEEK_HOLE)
        os.lseek(descriptor, 0, os.SEEK_SET)
        return first_hole < size

    def _rt(self, path: Path) -> str | None:
        data = self._read_repository_bytes(path)
        if data is None:
            return None
        try:
            return data.decode("utf-8")
        except UnicodeDecodeError as exc:
            self._ri("resource.utf8", path, f"repository text is not valid UTF-8: {exc}")
            return None

    def _load_repository_json(self, path: Path) -> Any:
        data = self._read_repository_bytes(path)
        if data is None:
            raise OSError("repository file was rejected by the bounded reader")
        return _load_json_bytes(data)

    def _load_repository_toml(self, path: Path) -> Any:
        text = self._rt(path)
        if text is None:
            raise OSError("repository file was rejected by the bounded reader")
        try:
            return tomllib.loads(text)
        except RecursionError as exc:
            raise tomllib.TOMLDecodeError(
                "TOML structural nesting exceeds parser capacity"
            ) from exc

    def run(self) -> list[Finding]:
        if self._inventory_has_findings:
            return sorted(set(self.findings))
        if not self._preflight_repository_resources():
            return sorted(set(self.findings))
        self._load_and_validate_policy()
        if not self.policy:
            return sorted(set(self.findings))
        self._validate_required_and_forbidden_paths()
        self._validate_makefile_entrypoint()
        self._validate_compiler_dependency_boundary()
        self._validate_compiler_language_boundary()
        self._validate_tree_encoding_and_format()
        self._validate_brand_assets()
        self._validate_protected_file_digests()
        self._validate_hosted_control_evidence()
        self._validate_markdown_links()
        self._validate_orange_book()
        self._validate_json_documents()
        self._validate_schema_fixtures()
        self._validate_workflows()
        self._validate_dependabot()
        self._validate_codeowners()
        self._validate_decision_gates()
        self._validate_traceability()
        self._validate_user_journeys()
        self._validate_proof_foundation_suite()
        self._validate_product_form_decision_packet()
        self._validate_semantic_strata_suite()
        self._validate_change_records()
        self._validate_repository_templates()
        self._end()
        return sorted(set(self.findings))

    def _end(self) -> None:
        final_findings: list[Finding] = []
        final_files, _ = _repository_file_inventory(self.root, final_findings)
        final_index = (
            git_index_entries(self.root, final_findings, required=True)
            if self.index_entries
            else []
        )
        for finding in final_findings:
            self.add(finding.code, finding.path, finding.message)
        if self.index_entries and final_index != self.index_entries:
            self._ri(_RC, ".git", "Git stage-zero path, mode, object identity, or type changed during validation")
        expected = {relative(path, self.root) for path in self.repository_files}
        observed = {relative(path, self.root) for path in final_files}
        if observed != expected:
            self._ri(_RC, ".", "repository inventory changed during validation")
            return
        for path in final_files:
            inspected = self._if(path)
            value = relative(path, self.root)
            if inspected is not None and self._ms(inspected[2]) != self._resource_metadata.get(value):
                self._ri(_RC, path, "repository file metadata changed during validation")

    def _load_and_validate_policy(self) -> None:
        finding_count = len(self.findings)
        if not self._hf(self.policy_path):
            self.add("policy.missing", self.policy_path, "solo-bootstrap repository policy is missing")
            return
        try:
            policy = self._load_repository_json(self.policy_path)
        except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError) as exc:
            self.add("policy.invalid_json", self.policy_path, str(exc))
            return
        required = {
            _PV: str,
            "repository": str,
            "stage": str,
            "status": str,
            "default_branch": str,
            "bootstrap_steward": str,
            _TP: list,
            _AB: list,
            _RP: list,
            _FP: list,
            _RW: list,
            "workflow_inventory": list,
            _PD: dict,
            "executable_paths": list,
            _G: dict,
            "hosted_repository_controls": dict,
            _CO: list,
            "decision_gates": dict,
            "temporary_constraints": dict,
        }
        if not isinstance(policy, dict):
            self.add("policy.type", self.policy_path, "policy root must be an object")
            return
        for key, expected_type in required.items():
            if key not in policy:
                self.add("policy.required", self.policy_path, f"missing required key {key!r}")
            elif not isinstance(policy[key], expected_type):
                self.add(
                    "policy.type",
                    self.policy_path,
                    f"{key!r} must be {expected_type.__name__}",
                )
        if len(self.findings) != finding_count:
            return
        string_list_keys = (
            _TP,
            _RP,
            _FP,
            _RW,
            "workflow_inventory",
            "executable_paths",
            _CO,
        )
        for key in string_list_keys:
            if not all(isinstance(value, str) and value for value in policy[key]):
                self.add("policy.value", self.policy_path, f"{key} must contain non-empty strings")
        for key in (_RP, _FP):
            if any(isinstance(value, str) and "\0" in value for value in policy[key]):
                self.add("policy.value", self.policy_path, f"{key} contains an invalid path string")

        expected_binary_fields = {"path", "sha256", "role", "provenance"}
        for index, artifact in enumerate(policy[_AB]):
            if not isinstance(artifact, dict):
                self.add(_PB, self.policy_path, f"{_ABA}{index}] must be an object")
                continue
            if set(artifact) != expected_binary_fields:
                self.add(_PB, self.policy_path, f"{_ABA}{index}] has invalid fields")
                continue
            if not all(isinstance(artifact[field], str) for field in expected_binary_fields):
                self.add(_PB, self.policy_path, f"{_ABA}{index}] fields must be strings")

        action_policy = policy[_G]
        action_field_types = {
            _AR: list,
            _AI: list,
            _WP: dict,
            _FE: list,
            _FS: bool,
            _VC: bool,
        }
        for key, expected_type in action_field_types.items():
            if not isinstance(action_policy.get(key), expected_type):
                self.add(
                    "policy.type",
                    self.policy_path,
                    f"github_actions.{key} must be {expected_type.__name__}",
                )
        for key in (_AR, _AI, _FE):
            values = action_policy.get(key)
            if isinstance(values, list) and not all(isinstance(value, str) and value for value in values):
                self.add(
                    "policy.value",
                    self.policy_path,
                    f"github_actions.{key} must contain non-empty strings",
                )
            elif isinstance(values, list) and len(values) != len(set(values)):
                self.add(
                    "policy.duplicate",
                    self.policy_path,
                    f"github_actions.{key} contains duplicate values",
                )
        write_permissions = action_policy.get(_WP)
        if isinstance(write_permissions, dict) and not all(
            isinstance(name, str)
            and name
            and isinstance(values, list)
            and all(isinstance(value, str) and value for value in values)
            for name, values in write_permissions.items()
        ):
            self.add(
                "policy.value",
                self.policy_path,
                "github_actions.allowed_write_permissions must map names to string arrays",
            )
        elif isinstance(write_permissions, dict):
            for values in write_permissions.values():
                if len(values) != len(set(values)):
                    self.add(
                        "policy.duplicate",
                        self.policy_path,
                        "github_actions.allowed_write_permissions contains duplicate values",
                    )
        protected_digests = policy[_PD]
        if not all(
            isinstance(path, str)
            and path
            and isinstance(digest, str)
            and re.fullmatch(r"[0-9a-f]{64}", digest)
            for path, digest in protected_digests.items()
        ):
            self.add(
                "policy.value",
                self.policy_path,
                "protected_file_digests must map non-empty paths to lowercase SHA-256 values",
            )
        if len(self.findings) != finding_count:
            return
        if policy["repository"] != "chasebryan/orange":
            self.add("policy.scope", self.policy_path, "repository must remain chasebryan/orange")
        if policy["stage"] != "solo-bootstrap" or policy["status"] != "enforced":
            self.add("policy.stage", self.policy_path, "this validator only accepts enforced solo-bootstrap policy")
        if policy["default_branch"] != "main":
            self.add("policy.default_branch", self.policy_path, "solo-bootstrap default branch must remain main")
        if policy["bootstrap_steward"] != "chasebryan":
            self.add("policy.steward", self.policy_path, "bootstrap steward must remain chasebryan")
        if not re.fullmatch(r"0\.[0-9]+\.[0-9]+", policy[_PV]):
            self.add("policy.version", self.policy_path, "solo-bootstrap policy version must be a 0.x semantic version")
        for key in string_list_keys:
            values = policy[key]
            if len(values) != len(set(values)):
                self.add("policy.duplicate", self.policy_path, f"{key} contains duplicate values")
        minimum_sets = {
            _RP: MINIMUM_REQUIRED_PATHS,
            _FP: MINIMUM_FORBIDDEN_PATHS,
            _RW: MINIMUM_REQUIRED_WORKFLOWS,
            _CO: MINIMUM_CODEOWNERS,
        }
        for key, minimum in minimum_sets.items():
            missing = sorted(minimum - set(policy[key]))
            if missing:
                self.add("policy.minimum", self.policy_path, f"{key} omits protected values: {', '.join(missing)}")
        if set(policy[_RP]) != MINIMUM_REQUIRED_PATHS:
            self.add("policy.required_inventory", self.policy_path, "solo-bootstrap required-path inventory must remain exact")
        top_level = set(policy[_TP])
        if top_level != GATE0_ALLOWED_TOP_LEVEL:
            missing = sorted(GATE0_ALLOWED_TOP_LEVEL - top_level)
            extra = sorted(top_level - GATE0_ALLOWED_TOP_LEVEL)
            self.add(
                "policy.top_level",
                self.policy_path,
                f"solo-bootstrap top-level allowlist drifted; missing={missing}, extra={extra}",
            )
        for index, artifact in enumerate(policy[_AB]):
            if not isinstance(artifact, dict):
                self.add(_PB, self.policy_path, f"{_ABA}{index}] must be an object")
                continue
            if set(artifact) != {"path", "sha256", "role", "provenance"}:
                self.add(_PB, self.policy_path, f"{_ABA}{index}] has invalid fields")
                continue
            if not isinstance(artifact["path"], str) or safe_manifest_path(self.root, artifact["path"]) is None:
                self.add(_PB, self.policy_path, f"{_ABA}{index}] has unsafe path")
            if not isinstance(artifact["sha256"], str) or not re.fullmatch(r"[0-9a-f]{64}", artifact["sha256"]):
                self.add(_PB, self.policy_path, f"{_ABA}{index}] has invalid SHA-256")
            for field in ("role", "provenance"):
                if not isinstance(artifact[field], str) or not artifact[field].strip():
                    self.add(_PB, self.policy_path, f"{_ABA}{index}] needs {field}")
        if policy[_AB] != GATE0_ALLOWED_BINARY_ARTIFACTS:
            self.add(
                "policy.binary_inventory",
                self.policy_path,
                "official binary artifact paths, digests, roles, and provenance must remain exact",
            )
        expected_action_policy_keys = {
            _AR,
            _AI,
            _WP,
            _FE,
            _FS,
            _VC,
        }
        observed_action_policy_keys = set(policy[_G])
        if observed_action_policy_keys != expected_action_policy_keys:
            self.add(
                "policy.action_fields",
                self.policy_path,
                "github_actions fields must remain exact; "
                f"missing={sorted(expected_action_policy_keys - observed_action_policy_keys)}, "
                f"extra={sorted(observed_action_policy_keys - expected_action_policy_keys)}",
            )
        action_repositories = set(policy[_G].get(_AR, []))
        if action_repositories != MINIMUM_ACTION_REPOSITORIES:
            self.add(
                "policy.action_allowlist",
                self.policy_path,
                f"Action identities must be exact; missing={sorted(MINIMUM_ACTION_REPOSITORIES - action_repositories)}, extra={sorted(action_repositories - MINIMUM_ACTION_REPOSITORIES)}",
            )
        container_images = set(policy[_G].get(_AI, []))
        if container_images != GATE0_ALLOWED_CONTAINER_IMAGES:
            self.add(
                "policy.container_allowlist",
                self.policy_path,
                f"container image identities must be exact; missing={sorted(GATE0_ALLOWED_CONTAINER_IMAGES - container_images)}, extra={sorted(container_images - GATE0_ALLOWED_CONTAINER_IMAGES)}",
            )
        if set(policy["executable_paths"]) != GATE0_EXECUTABLE_PATHS:
            self.add("policy.executables", self.policy_path, "solo-bootstrap executable allowlist must remain exact")
        if set(policy["workflow_inventory"]) != _WI:
            self.add("policy.workflow_inventory", self.policy_path, "solo-bootstrap workflow inventory must remain exact")
        protected_digest = hashlib.sha256(
            json.dumps(
                policy[_PD], sort_keys=True, separators=(",", ":")
            ).encode("utf-8")
        ).hexdigest()
        if protected_digest != _PHD:
            self.add(
                "policy.protected_file_digests",
                self.policy_path,
                "protected solo-bootstrap file digests must remain exact",
            )
        actual_writes = {
            name: set(values)
            for name, values in policy[_G].get(_WP, {}).items()
            if isinstance(values, list)
        }
        if actual_writes != GATE0_ALLOWED_WRITE_PERMISSIONS:
            self.add("policy.write_permissions", self.policy_path, "workflow write-permission exceptions must remain exact")
        if policy[_G].get(_FS) is not True:
            self.add("policy.action_sha", self.policy_path, "full Action commit SHA enforcement cannot be disabled")
        if policy[_G].get(_VC) is not True:
            self.add("policy.action_comment", self.policy_path, "Action version comments cannot be disabled")
        if "pull_request_target" not in policy[_G].get(_FE, []):
            self.add("policy.forbidden_event", self.policy_path, "pull_request_target must remain forbidden")
        if policy["hosted_repository_controls"] != GATE0_HOSTED_REPOSITORY_CONTROLS:
            self.add(
                "policy.hosted_repository_controls",
                self.policy_path,
                "hosted repository-control snapshot must remain exact",
            )
        constraints = policy["temporary_constraints"]
        expected_constraints = {
            "accept_third_party_pull_requests": False,
            "allow_product_implementation": True,
            "allow_product_releases": False,
            "claim_osps_level_3": False,
        }
        if constraints != expected_constraints:
            self.add(
                "policy.temporary_constraint",
                self.policy_path,
                f"solo-bootstrap constraints must remain exact: {expected_constraints}",
            )
        expected_decisions = {
            "implementation_language": {"decision": "D-008", _RS: "directed"},
            "project_name": {"decision": "D-017", _RS: "directed"},
            "licenses": {"decision": "D-018", _RS: "directed"},
            "governance": {"decision": "D-019", _RS: "directed"},
            "solo_project": {"decision": "D-023", _RS: "directed"},
            "compiler_foundation": {"decision": "D-024", _RS: "directed"},
            "edition_2026_parser": {"decision": "D-025", _RS: "directed"},
            "typed_literal_semantics": {"decision": "D-026", _RS: "directed"},
        }
        if policy["decision_gates"] != expected_decisions:
            self.add("policy.decision_gates", self.policy_path, "solo-bootstrap decision gates must remain exact")
        if len(self.findings) == finding_count:
            self.policy = policy

    def _validate_protected_file_digests(self) -> None:
        for value in sorted(self.policy[_PD]):
            self._read_authenticated_protected_file(value)

    def _read_authenticated_protected_file(self, value: str) -> bytes | None:
        if value in self._authenticated_protected_bytes:
            return self._authenticated_protected_bytes[value]
        self._authenticated_protected_bytes[value] = None
        expected = self.policy[_PD].get(value)
        if expected is None:
            return None
        path = self.root / value
        if not self._hf(path):
            return None
        data = self._read_repository_bytes(path)
        if data is None:
            self.add("protected_file.unreadable", path, _BF)
            return None
        observed = hashlib.sha256(data).hexdigest()
        if observed != expected:
            self.add(
                "protected_file.digest",
                path,
                f"reviewed SHA-256 changed: expected {expected}, observed {observed}",
            )
            return None
        self._authenticated_protected_bytes[value] = data
        return data

    def _validate_hosted_control_evidence(self, *, today: dt.date | None = None) -> None:
        snapshot_value = str(GATE0_HOSTED_REPOSITORY_CONTROLS["snapshot_date"])
        review_due_value = str(GATE0_HOSTED_REPOSITORY_CONTROLS["review_due_date"])
        ruleset_id = str(GATE0_HOSTED_REPOSITORY_CONTROLS["main_ruleset_id"])
        expected_snapshot = (
            "Hosted-control snapshot: `"
            f"snapshot_date={snapshot_value} review_due_date={review_due_value} "
            f"ruleset_id={ruleset_id}`"
        )
        expected_bindings = [
            (
                "Required-check binding: `"
                f"context=\"{item['context']}\" integration_id={item['integration_id']}`"
            )
            for item in GATE0_HOSTED_REPOSITORY_CONTROLS["required_checks"]
        ]
        expected_markers = [expected_snapshot, *expected_bindings]
        evidence_paths = (
            "docs/operations/GITHUB_CONTROLS.md",
            "docs/security/OSPS_BASELINE.md",
            "docs/security/THREAT_MODEL.md",
        )
        stale_claim_patterns = {
            r"(?i)\bunprotected default branch\b": "default branch described as unprotected",
            r"(?i)\bno (?:branch protection|repository ruleset)(?:\s+or\s+ruleset)?\b": (
                "branch protection or ruleset described as absent"
            ),
            r"(?i)\bdefault branch has no protection and no ruleset\b": (
                "default branch described as having no protection"
            ),
            r"(?i)\bone owner and no branch rule\b": "branch rule described as absent",
            (
                r"(?i)\b(?:candidate|required|PR) (?:checks?|workflows?)\b[^.\n]{0,80}"
                r"\b(?:has|have) not (?:yet )?run\b"
            ): "required or candidate workflow described as never run",
            (
                r"(?i)\b(?:candidate|required|PR) (?:checks?|workflows?)\b[^.\n]{0,80}"
                r"\b(?:is|are) not required\b"
            ): "required check described as not required",
        }
        try:
            snapshot_date = dt.date.fromisoformat(snapshot_value)
            review_due_date = dt.date.fromisoformat(review_due_value)
        except ValueError:
            self.add(
                "hosted_control.date",
                self.policy_path,
                "hosted-control snapshot and review-due dates must be ISO calendar dates",
            )
            return
        if snapshot_date > review_due_date:
            self.add(
                "hosted_control.date_order",
                self.policy_path,
                "hosted-control review due date precedes its snapshot date",
            )
        observed_today = today or dt.datetime.now(tz=dt.timezone.utc).date()
        if snapshot_date > observed_today:
            self.add(
                "hosted_control.future_snapshot",
                self.policy_path,
                f"hosted-control snapshot {snapshot_value} is later than {observed_today.isoformat()}",
            )
        if observed_today >= review_due_date:
            self.add(
                "hosted_control.expired",
                self.policy_path,
                f"hosted-control snapshot expired on {review_due_value}; refresh live readback and evidence",
            )
        for value in evidence_paths:
            path = self.root / value
            if not self._hf(path):
                self.add("hosted_control.missing", path, "hosted-control evidence document is missing")
                continue
            text = self._rt(path)
            if text is None:
                self.add("hosted_control.unreadable", path, _BF)
                continue
            marker_prefixes = ("Hosted-control snapshot:", "Required-check binding:")
            observed_markers = [line for line in text.splitlines() if line.startswith(marker_prefixes)]
            visible_markers = [
                line
                for line in markdown_without_fenced_blocks_and_comments(text).splitlines()
                if line.startswith(marker_prefixes)
            ]
            if observed_markers != expected_markers or visible_markers != expected_markers:
                self.add(
                    "hosted_control.markers",
                    path,
                    "visible hosted-control snapshot and producer-binding markers must match the exact canonical sequence",
                )
            for pattern, description in sorted(stale_claim_patterns.items()):
                if re.search(pattern, text):
                    self.add(
                        "hosted_control.contradiction",
                        path,
                        f"stale hosted-control claim remains: {description}",
                    )

    def _policy_path(self, value: str) -> Path | None:
        candidate = safe_manifest_path(self.root, value)
        if candidate is None:
            self.add("policy.unsafe_path", self.policy_path, "repository path is not a safe relative path")
            return None
        return candidate

    def _validate_required_and_forbidden_paths(self) -> None:
        actual_paths = {relative(path, self.root) for path in self.repository_files}
        actual_top_level = {PurePosixPath(value).parts[0] for value in actual_paths}
        for value in sorted(actual_top_level - set(self.policy[_TP])):
            self.add("path.top_level", value, "top-level path is not admitted during Gate 0")
        static_paths = MINIMUM_REQUIRED_PATHS | _CIP
        for value in sorted(actual_paths - static_paths):
            if re.fullmatch(
                r"docs/governance/(?:oeps/OEP|adrs/ADR)-[0-9]{4}-[a-z0-9]+(?:-[a-z0-9]+)*\.md",
                value,
            ):
                continue
            self.add("path.inventory", value, "path is not admitted by the exact solo-bootstrap inventory")
        for value in self.policy[_RP]:
            path = self._policy_path(value)
            if path is not None and not self._hf(path):
                self.add("path.required", value, "required permanent artifact is missing")
        for value in self.policy[_FP]:
            path = self._policy_path(value)
            if path is not None and self._inventory_has_path(path):
                self.add("path.forbidden", value, "path is forbidden until its dependent capability decision closes")

    def _validate_makefile_entrypoint(self) -> None:
        path = self.root / "Makefile"
        source = self._rt(path)
        if source is None:
            return
        required_lines = {
            ".DEFAULT_GOAL := check": "default check required",
            "override SHELL := /bin/bash": "Bash path fixed",
            "override .SHELLFLAGS := -p -c": "startup state suppressed",
            "unexport BASH_ENV ENV": "hooks unexported",
            ".NOTPARALLEL: check": "check serialized",
            "check: check-policy test-policy check-compiler": (
                "policy/tests precede Cargo"
            ),
        }
        lines = source.splitlines()
        for required, meaning in required_lines.items():
            if lines.count(required) != 1:
                self.add("make.entrypoint_contract", path, f"{meaning}: expected exactly {required!r}")
        required_compiler_fragments = {
            "umask 077;": "mask",
            '/usr/bin/mktemp -d -- "$${TMPDIR:-/tmp}/orange-cargo-home.XXXXXXXX"': "fresh state",
            'cargo_home="$$(CDPATH= cd -- "$$cargo_home" && pwd -P)"': "absolute home",
            "cd -- /;": "root cwd",
            'env -i \\\n\t\t\t\tCARGO_HOME="$$cargo_home"': "empty env",
            'CARGO_HOME="$$cargo_home"': "fresh home",
            "CARGO_NET_OFFLINE=true": "offline",
            'CARGO_TARGET_DIR="$$cargo_home/target"': "target",
            "RUSTUP_TOOLCHAIN=1.96.1": "toolchain",
            "--workspace --all-targets --release --locked --offline": "release",
            (
                "--workspace --lib --bins --locked --offline -- -D warnings "
                "-D clippy::arithmetic_side_effects -D clippy::as_conversions "
                "-D clippy::string_slice "
                "-D clippy::indexing_slicing -D clippy::unwrap_used "
                "-D clippy::expect_used -D clippy::panic"
            ): "lints",
            (
                'run_cargo /usr/bin/env CARGO_TARGET_DIR="$$cargo_home/repro-target-a" '
                'cargo build --manifest-path "$$cargo_home/repro-src-a/compiler/Cargo.toml" '
                "-p orangec --bin orangec "
                "--release --locked --offline"
            ): "first roots",
            (
                'run_cargo /usr/bin/env CARGO_TARGET_DIR="$$cargo_home/repro-target-b" '
                'cargo build --manifest-path "$$cargo_home/repro-src-b/compiler/Cargo.toml" '
                "-p orangec --bin orangec "
                "--release --locked --offline"
            ): "second roots",
            'copy_compiler_source "$$cargo_home/repro-src-a"': "first copy",
            'copy_compiler_source "$$cargo_home/repro-src-b"': "second copy",
            'copy_compiler_source "$$cargo_home/check-src"': "check copy",
            'manifest="$$cargo_home/check-src/compiler/Cargo.toml"': "manifest",
            '--create --file="$$repro_source_archive"': "archive",
            "--format=gnu --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --mode='u+rwX,go+rX,go-w,u-s,g-s,o-t'": "metadata",
            '/usr/bin/env -i PATH=/usr/bin:/bin GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 /usr/bin/git -C "$$repository_root" ls-files --cached -z > "$$repro_source_paths"': "tracked list",
            'ls-files --cached -z > "$$repro_source_paths_after"': "final list",
            '--hard-dereference --null --verbatim-files-from --no-recursion --directory="$$repository_root" --files-from="$$repro_source_paths"': "safe list",
            '--extract --file="$$repro_source_archive"': "extraction",
            '! -L "$$cargo_home/check-src/$$relative_path" ]]': "type",
            'live_executable="$$(( (8#$$live_mode & 0111) != 0 ))"': "mode class",
            '[[ "$$live_executable" == "$$snapshot_executable" ]]': "mode compare",
            '/usr/bin/cmp --silent -- "$$repository_root/$$relative_path" "$$cargo_home/check-src/$$relative_path"': "content",
            '/usr/bin/cmp --silent -- "$$repro_source_paths" "$$repro_source_paths_after"': "membership",
            "optimized orangec builds differ across source roots": "artifacts match",
            'repository_manifest="$(abspath $(dir $(lastword $(MAKEFILE_LIST))))/compiler/Cargo.toml"': "anchor",
        }
        for required, meaning in required_compiler_fragments.items():
            if source.count(required) != 1:
                self.add("make.compiler_environment_contract", path, f"{meaning}: expected exactly {required!r}")
        required_python_fragments = {
            "PYTHONHASHSEED=0": (3, "fixed Python hash seed"),
            "python3 -S -P -B -X utf8": (
                3,
                "isolated Python startup/path/bytecode/encoding",
            ),
            "-W error::ResourceWarning": (3, "resource leaks fail"),
        }
        for required, (expected_count, meaning) in required_python_fragments.items():
            if source.count(required) != expected_count:
                self.add(
                    "make.python_environment_contract",
                    path,
                    f"{meaning}: expected exactly {expected_count} {required!r} fragments",
                )
        required_test_fragments = {
            '/usr/bin/mktemp -d -- "$${TMPDIR:-/tmp}/orange-python-cache.XXXXXXXX"': (
                "foundation tests need a fresh bytecode lookup root"
            ),
            'pycache="$$(CDPATH= cd -- "$$pycache" && pwd -P)"': (
                "the foundation-test bytecode root must be canonical before cleanup"
            ),
            'PYTHONPYCACHEPREFIX="$$pycache"': (
                "foundation tests must not load ignored checkout bytecode"
            ),
        }
        for required, meaning in required_test_fragments.items():
            if source.count(required) != 1:
                self.add("make.python_cache_contract", path, f"{meaning}: expected exactly {required!r}")

    def _validate_compiler_dependency_boundary(self) -> None:
        toolchain_path = self.root / "rust-toolchain.toml"
        try:
            toolchain = self._load_repository_toml(toolchain_path)
        except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
            self.add("compiler.toolchain_toml", toolchain_path, f"Rust toolchain file is not valid TOML: {exc}")
        else:
            if toolchain != GATE0_RUST_TOOLCHAIN:
                self.add(
                    "compiler.toolchain_contract",
                    toolchain_path,
                    "Rust channel, components, and profile must match the exact S1 toolchain contract",
                )

        expected_manifest_paths = set(GATE0_RUST_MANIFESTS)
        observed_manifest_paths = {
            relative(path, self.root)
            for path in self.repository_files
            if path.name == "Cargo.toml"
        }
        if observed_manifest_paths != expected_manifest_paths:
            self.add(
                "compiler.manifest_inventory",
                "compiler",
                "Cargo manifest inventory must remain exact; "
                f"missing={sorted(expected_manifest_paths - observed_manifest_paths)}, "
                f"extra={sorted(observed_manifest_paths - expected_manifest_paths)}",
            )

        manifests: dict[str, dict[str, Any]] = {}
        for value in sorted(expected_manifest_paths):
            path = self.root / value
            try:
                manifest = self._load_repository_toml(path)
            except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
                self.add("compiler.manifest_toml", path, f"Cargo manifest is not valid TOML: {exc}")
                continue
            manifests[value] = manifest

            if manifest != GATE0_RUST_MANIFESTS[value]:
                self.add(
                    "compiler.manifest_contract",
                    path,
                    "parsed Cargo manifest differs from the exact S1 workspace/package contract",
                )

            expected_package = GATE0_RUST_MANIFEST_PACKAGES[value]
            package = manifest.get("package")
            observed_package = package.get("name") if isinstance(package, dict) else None
            package_is_exact = (
                package is None
                if expected_package is None
                else isinstance(package, dict)
                and observed_package == expected_package
                and _WS not in package
            )
            if not package_is_exact:
                self.add(
                    "compiler.package_inventory",
                    path,
                    "workspace package identity/ownership must remain exact; "
                    f"expected={expected_package!r}, observed={observed_package!r}",
                )

            observed_tables: dict[str, Any] = {}

            def record_table(label: str, table: Any) -> None:
                if not isinstance(table, dict):
                    self.add(
                        _T,
                        path,
                        f"Cargo dependency table {label!r} must be a table",
                    )
                    return
                if table:
                    observed_tables[label] = table

            for kind in (_DS, "dev-dependencies", "build-dependencies"):
                if kind in manifest:
                    record_table(kind, manifest[kind])

            workspace = manifest.get(_WS)
            if workspace is not None:
                if not isinstance(workspace, dict):
                    self.add("compiler.workspace", path, "Cargo workspace declaration must be a table")
                elif value != _CT:
                    self.add("compiler.workspace", path, "only the root manifest may declare a workspace")
                elif _DS in workspace:
                    record_table("workspace.dependencies", workspace[_DS])

            targets = manifest.get("target")
            if targets is not None:
                if not isinstance(targets, dict):
                    self.add(_T, path, "Cargo target declaration must be a table")
                else:
                    for target_name, target in sorted(targets.items()):
                        if not isinstance(target, dict):
                            self.add(
                                _T,
                                path,
                                f"Cargo target {target_name!r} must be a table",
                            )
                            continue
                        for kind in (_DS, "dev-dependencies", "build-dependencies"):
                            if kind in target:
                                record_table(f"target.{target_name}.{kind}", target[kind])

            patches = manifest.get("patch")
            if patches is not None:
                if not isinstance(patches, dict):
                    self.add(_T, path, "Cargo patch declaration must be a table")
                else:
                    for source_name, table in sorted(patches.items()):
                        record_table(f"patch.{source_name}", table)
            if "replace" in manifest:
                record_table("replace", manifest["replace"])

            expected_tables = GATE0_RUST_DEPENDENCY_TABLES[value]
            if observed_tables != expected_tables:
                self.add(
                    "compiler.dependencies",
                    path,
                    "dependency declarations must remain the exact admitted first-party path graph; "
                    f"expected={expected_tables!r}, observed={observed_tables!r}",
                )

        root_manifest = manifests.get(_CT)
        if root_manifest is not None:
            workspace = root_manifest.get(_WS)
            observed_members = workspace.get("members") if isinstance(workspace, dict) else None
            observed_excludes = workspace.get("exclude", []) if isinstance(workspace, dict) else None
            if observed_members != GATE0_RUST_WORKSPACE_MEMBERS or observed_excludes != []:
                self.add(
                    "compiler.workspace_members",
                    self.root / _CT,
                    "workspace members must remain the exact admitted package directories with no exclusions",
                )

        lock_path = self.root / "compiler/Cargo.lock"
        try:
            lock = self._load_repository_toml(lock_path)
        except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
            self.add("compiler.lock_toml", lock_path, f"Cargo lockfile is not valid TOML: {exc}")
            return
        if lock != GATE0_RUST_LOCK:
            self.add(
                "compiler.lock_graph",
                lock_path,
                "Cargo lockfile must contain only the exact two first-party workspace packages and edge",
            )

    def _validate_compiler_language_boundary(self) -> None:
        budget_groups = (
            (_RB, True, "compiler.language_budget"),
            (_OB, False, "compiler.cli_budget"),
        )
        for budgets, require_public, finding_code in budget_groups:
            visibility = r"pub\s+" if require_public else r"(?:pub\s+)?"
            for value, expected_constants in budgets.items():
                path = self.root / value
                text = self._rt(path)
                if text is None:
                    self.add(finding_code, path, "cannot read Rust budget source through bounded reader")
                    continue
                source = rust_code_without_comments_and_literals(text)
                declarations: dict[str, list[str]] = {}
                for match in re.finditer(
                    rf"(?m)^\s*{visibility}const\s+([A-Z][A-Z0-9_]*)\s*:\s*usize\s*=\s*([^;]+);",
                    source,
                ):
                    declarations.setdefault(match.group(1), []).append(match.group(2))
                for name, expected in expected_constants.items():
                    expressions = declarations.get(name, [])
                    if len(expressions) != 1:
                        self.add(
                            finding_code,
                            path,
                            f"{name} must have exactly one usize declaration; observed={len(expressions)}",
                        )
                        continue
                    observed = parse_rust_usize_product(expressions[0])
                    if observed != expected:
                        self.add(
                            finding_code,
                            path,
                            f"{name} must equal {expected}; observed={observed!r}",
                        )

        marker_groups = (
            (_RM, "compiler.language_spec_budget", "normative specification"),
            (_OM, "compiler.cli_spec_budget", "compiler contract"),
        )
        for markers, finding_code, description in marker_groups:
            for value, expected_markers in markers.items():
                specification = self.root / value
                text = self._rt(specification)
                if text is None:
                    self.add(
                        finding_code,
                        specification,
                        "cannot read budget documentation through bounded reader",
                    )
                    continue
                for marker, expected in expected_markers.items():
                    if marker not in text:
                        self.add(
                            finding_code,
                            specification,
                            f"{description} must state the exact {expected} budget marker {marker!r}",
                        )

    def _validate_tree_encoding_and_format(self) -> None:
        if not self._preflight_repository_resources():
            return
        files = self.repository_files
        casefolded: dict[str, str] = {}
        normalized: dict[str, str] = {}
        executable_paths = set(self.policy["executable_paths"])
        binary_artifacts = {
            artifact["path"]: artifact
            for artifact in self.policy[_AB]
            if isinstance(artifact, dict) and isinstance(artifact.get("path"), str)
        }
        for mode, _, value in self.index_entries:
            if mode == "160000":
                self.add("git.submodule", value, "gitlinks/submodules are not admitted during Gate 0")
            elif mode not in {"100644", "100755"}:
                self.add("git.mode", value, f"unsupported Git index mode {mode}")
            metadata_signature = self._resource_metadata.get(value)
            if metadata_signature is not None and mode in {"100644", "100755"}:
                worktree_executable = bool(metadata_signature[2] & 0o111)
                if worktree_executable != (mode == "100755"):
                    self.add("git.mode_mismatch", value, "Git index and worktree executable modes differ")
        for path in files:
            value = relative(path, self.root)
            metadata_signature = self._resource_metadata.get(value)
            if metadata_signature is None:
                self._ri(
                    "resource.post_preflight_addition",
                    path,
                    "repository file appeared after the resource preflight",
                )
                continue
            if re.match(r"^(?:LICENSE|COPYING)(?:\.|$)", path.name, re.IGNORECASE):
                self.add("file.unratified_license", path, "license/copying files are forbidden until D-018 closes")
            folded = value.casefold()
            nfc = unicodedata.normalize("NFC", value)
            if folded in casefolded and casefolded[folded] != value:
                self.add("path.case_collision", path, f"case-fold collision with {casefolded[folded]}")
            else:
                casefolded[folded] = value
            if nfc in normalized and normalized[nfc] != value:
                self.add("path.normalization_collision", path, f"Unicode NFC collision with {normalized[nfc]}")
            else:
                normalized[nfc] = value
            if value != nfc:
                self.add("path.not_nfc", path, "repository path must be Unicode NFC")
            is_executable = bool(metadata_signature[2] & 0o111)
            if is_executable and value not in executable_paths:
                self.add("file.unexpected_executable", path, "executable bit is not authorized by repository policy")
            if value in executable_paths and not is_executable:
                self.add("file.missing_executable", path, "repository policy requires the executable bit")
            data = self._read_repository_bytes(path)
            if data is None:
                continue
            if b"\x00" in data or path.suffix.lower() in BINARY_SUFFIXES:
                admission = binary_artifacts.get(value)
                if admission is None:
                    self.add("file.binary", path, "binary file has no explicit digest, role, and provenance admission")
                elif hashlib.sha256(data).hexdigest() != admission.get("sha256"):
                    self.add("file.binary_digest", path, "binary content does not match its admitted SHA-256")
                continue
            try:
                text = data.decode("utf-8")
            except UnicodeDecodeError as exc:
                self.add("file.utf8", path, f"text must be valid UTF-8: {exc}")
                continue
            if text.startswith("\ufeff"):
                self.add("file.bom", path, "UTF-8 byte-order marks are forbidden")
            if "\r" in text:
                self.add("file.line_endings", path, "text must use LF line endings")
            if text and not text.endswith("\n"):
                self.add("file.final_newline", path, "text file must end with one LF")
            for line_number, line in enumerate(text.splitlines(), start=1):
                if line.rstrip(" \t") != line:
                    self.add("file.trailing_whitespace", path, f"trailing whitespace on line {line_number}")
                if path.suffix.lower() in TEXT_TAB_FREE_SUFFIXES and "\t" in line:
                    self.add("file.tab", path, f"tab character on line {line_number}")
                if re.match(r"^\s*(?:<{7}|={7}|>{7})(?:\s|$)", line):
                    self.add("file.merge_marker", path, f"merge conflict marker on line {line_number}")
            if path.suffix.lower() == ".md":
                fence_error = markdown_fence_error(text)
                if fence_error:
                    self.add("markdown.fence", path, fence_error)
                comment_error = markdown_html_comment_error(text)
                if comment_error:
                    self.add("markdown.html_comment", path, comment_error)

    def _validate_brand_assets(self) -> None:
        manifest_path = self.root / "assets/brand/manifest.json"
        try:
            manifest = self._load_repository_json(manifest_path)
        except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError) as exc:
            self.add("brand.manifest", manifest_path, str(exc))
            return
        expected_header = {
            "schema_version": "orange-brand-assets/v1",
            "status": "official",
            "authority": "chasebryan",
            "designated_on": _D,
            "source_collection": "Orange-Assets",
            "import_mode": "byte-for-byte",
        }
        if not isinstance(manifest, dict):
            self.add("brand.manifest", manifest_path, "brand manifest root must be an object")
            return
        if {key: manifest.get(key) for key in expected_header} != expected_header:
            self.add("brand.manifest_header", manifest_path, "official brand manifest header must remain exact")
        if set(manifest) != {*expected_header, "assets"}:
            self.add("brand.manifest_fields", manifest_path, "official brand manifest fields must remain exact")
        assets = manifest.get("assets")
        if not isinstance(assets, list):
            self.add("brand.manifest", manifest_path, "assets must be an array")
            return
        expected_paths = list(GATE0_BRAND_ASSET_METADATA)
        observed_paths = [item.get("path") for item in assets if isinstance(item, dict)]
        if observed_paths != expected_paths or len(observed_paths) != len(assets):
            self.add("brand.manifest_inventory", manifest_path, "official brand asset order and inventory must remain exact")
        admissions = {
            PurePosixPath(item["path"]).name: item
            for item in GATE0_ALLOWED_BINARY_ARTIFACTS
        }
        for index, item in enumerate(assets):
            if not isinstance(item, dict):
                self.add("brand.manifest_item", manifest_path, f"assets[{index}] must be an object")
                continue
            name = item.get("path")
            if not isinstance(name, str) or name not in GATE0_BRAND_ASSET_METADATA:
                self.add("brand.manifest_item", manifest_path, f"assets[{index}] has an unadmitted path")
                continue
            media_type, width, height, alpha, has_c2pa = GATE0_BRAND_ASSET_METADATA[name]
            expected_fields = {
                "path",
                "source_filename",
                "media_type",
                "width",
                "height",
                "alpha",
                "role",
                "sha256",
            }
            if has_c2pa:
                expected_fields.add("content_credentials")
            if set(item) != expected_fields:
                self.add("brand.manifest_fields", manifest_path, f"assets[{index}] fields must remain exact")
            if (
                item.get("media_type") != media_type
                or item.get("width") != width
                or item.get("height") != height
                or item.get("alpha") is not alpha
                or not isinstance(item.get("role"), str)
                or not item.get("role")
            ):
                self.add("brand.manifest_metadata", manifest_path, f"assets[{index}] technical metadata is incorrect")
            if item.get("sha256") != admissions[name]["sha256"]:
                self.add("brand.manifest_digest", manifest_path, f"assets[{index}] digest disagrees with policy")
            if item.get("source_filename") != GATE0_BRAND_SOURCE_FILENAMES[name]:
                self.add(
                    "brand.manifest_provenance",
                    manifest_path,
                    f"assets[{index}] source filename is incorrect",
                )
            if has_c2pa and item.get("content_credentials") != (
                "embedded-c2pa-openai-trainedAlgorithmicMedia-unverified"
            ):
                self.add("brand.manifest_provenance", manifest_path, f"assets[{index}] C2PA status is incorrect")

            asset_path = manifest_path.parent / name
            data = self._read_repository_bytes(asset_path)
            if data is None:
                self.add("brand.asset", asset_path, _BF)
                continue
            if media_type == "image/png":
                if len(data) < 29 or data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
                    self.add("brand.asset_format", asset_path, "asset is not a canonical PNG stream")
                    continue
                observed_width = int.from_bytes(data[16:20], "big")
                observed_height = int.from_bytes(data[20:24], "big")
                observed_alpha = data[25] in {4, 6}
                if (observed_width, observed_height, observed_alpha) != (width, height, alpha):
                    self.add("brand.asset_metadata", asset_path, "PNG header disagrees with admitted metadata")
            elif not data.startswith(b"\xff\xd8\xff") or not data.endswith(b"\xff\xd9"):
                self.add("brand.asset_format", asset_path, "asset is not a complete JPEG stream")
            if has_c2pa and b"caBX" not in data:
                self.add("brand.c2pa", asset_path, "canonical source no longer carries its C2PA container")

    def _validate_markdown_links(self) -> None:
        inventory_files = {relative(path, self.root) for path in self.repository_files}
        inventory_directories = {"."}
        for value in inventory_files:
            parts = PurePosixPath(value).parts
            for depth in range(1, len(parts)):
                inventory_directories.add("/".join(parts[:depth]))

        anchor_cache: dict[str, set[str]] = {}
        for path in (path for path in self.repository_files if path.suffix.lower() == ".md"):
            text = self._rt(path)
            if text is None:
                continue
            text = markdown_without_fenced_blocks_and_comments(text)
            text = markdown_with_masked_inline_syntax(text, "[]()")
            targets = (
                target
                for candidates in (
                    markdown_inline_link_targets(text),
                    (match.group(1) for match in MARKDOWN_CONTINUED_TITLE_RE.finditer(text)),
                    (
                        match.group(2)
                        for match in MARKDOWN_REFERENCE_RE.finditer(text)
                        if len(match.group(1)) <= 999 and match.group(1).strip()
                    ),
                )
                for target in candidates
            )
            for value in targets:
                raw_target = value.strip()
                target = self._markdown_destination(raw_target)
                if not target:
                    continue
                uri_target = target.replace(" ", "%20") if raw_target.startswith("<") else target
                if not valid_format(uri_target, "uri-reference"):
                    self.add("markdown.link_invalid", path, "link target is not a valid URI reference")
                    continue
                try:
                    parsed = urlsplit(target)
                except ValueError:
                    self.add(
                        "markdown.link_invalid",
                        path,
                        "link target is not a valid URI reference",
                    )
                    continue
                if parsed.scheme or target.startswith("//"):
                    continue
                file_part = decode_uri_component(parsed.path)
                fragment = decode_uri_component(parsed.fragment)
                if file_part is None or fragment is None:
                    self.add("markdown.link_invalid", path, "link target has invalid percent encoding")
                    continue
                target_path = path if not file_part else (path.parent / file_part)
                lexical_target = self._lp(target_path)
                if lexical_target is None:
                    self.add("markdown.link_escape", path, f"link escapes repository: {raw_target}")
                    continue
                target_value, resolved = lexical_target
                if target_value not in inventory_files and target_value not in inventory_directories:
                    self.add("markdown.link_missing", path, f"local link target does not exist: {raw_target}")
                    continue
                if fragment and target_value in inventory_files and resolved.suffix.lower() == ".md":
                    if target_value not in anchor_cache:
                        target_text = self._rt(resolved)
                        if target_text is None:
                            continue
                        anchor_cache[target_value] = markdown_anchors(target_text)
                    anchors = anchor_cache[target_value]
                    if fragment not in anchors:
                        self.add("markdown.anchor_missing", path, f"anchor not found: {raw_target}")

    def _validate_orange_book(self) -> None:
        path = self.root / ORANGE_BOOK_PATH
        if not self._hf(path):
            self.add("book.missing", path, "the canonical Orange Book manuscript is missing")
            return
        text = self._rt(path)
        if text is None:
            self.add("book.unreadable", path, _BF)
            return

        visible_text = markdown_without_fenced_blocks_and_comments(text)
        lines = visible_text.splitlines()
        if not lines or lines[0] != "# The Orange Book":
            self.add("book.identity", path, "the manuscript must begin with '# The Orange Book'")
        bylines = [line for line in lines if re.fullmatch(r"By\s+\S.*", line)]
        if bylines != ["By Chase Bryan"]:
            self.add("book.identity", path, "the manuscript must contain only the exact byline 'By Chase Bryan'")
        if lines.count("Status: living pre-alpha reader guide") != 1:
            self.add("book.status", path, "the manuscript must retain its living pre-alpha reader-guide status")

        version_lines = [line for line in lines if line.startswith("Manuscript version:")]
        if version_lines != [f"Manuscript version: {ORANGE_BOOK_VERSION}"]:
            self.add(
                "book.version",
                path,
                f"the manuscript must contain only the exact version {ORANGE_BOOK_VERSION}",
            )

        snapshot_lines = [line for line in lines if line.startswith("Snapshot:")]
        snapshot_valid = len(snapshot_lines) == 1 and re.fullmatch(
            r"Snapshot: \d{4}-\d{2}-\d{2}", snapshot_lines[0]
        )
        if snapshot_valid:
            try:
                dt.date.fromisoformat(snapshot_lines[0].removeprefix("Snapshot: "))
            except ValueError:
                snapshot_valid = False
        if not snapshot_valid:
            self.add("book.snapshot", path, "the manuscript must contain exactly one ISO-date Snapshot line")

        section_positions: list[int] = []
        for heading in ORANGE_BOOK_REQUIRED_SECTIONS:
            positions = [index for index, line in enumerate(lines) if line == heading]
            if len(positions) != 1:
                self.add("book.structure", path, f"required section must occur exactly once: {heading}")
            else:
                section_positions.append(positions[0])
        if len(section_positions) == len(ORANGE_BOOK_REQUIRED_SECTIONS) and section_positions != sorted(
            section_positions
        ):
            self.add("book.structure", path, "required manuscript sections are out of order")

        try:
            contents_start = lines.index("## Contents") + 1
            contents_end = lines.index("## Preface")
        except ValueError:
            pass
        else:
            observed_contents = [line for line in lines[contents_start:contents_end] if line.startswith("- [")]
            if observed_contents != list(ORANGE_BOOK_CONTENTS):
                self.add("book.navigation", path, "contents must list the required manuscript destinations in order")

        for index, heading in enumerate(ORANGE_BOOK_CHAPTERS):
            following_heading = (
                ORANGE_BOOK_CHAPTERS[index + 1]
                if index + 1 < len(ORANGE_BOOK_CHAPTERS)
                else "## Manuscript map"
            )
            try:
                chapter_start = lines.index(heading) + 1
                chapter_end = lines.index(following_heading)
            except ValueError:
                continue
            chapter_text = "\n".join(lines[chapter_start:chapter_end])
            chapter_word_count = sum(1 for _ in re.finditer(r"[A-Za-z0-9][A-Za-z0-9'’-]*", chapter_text))
            if chapter_word_count < ORANGE_BOOK_MINIMUM_CHAPTER_WORDS:
                self.add(
                    "book.chapter_length",
                    path,
                    f"{heading.removeprefix('## ')} must contain at least "
                    f"{ORANGE_BOOK_MINIMUM_CHAPTER_WORDS} words; observed {chapter_word_count}",
                )

        boundary_text = re.sub(r"(?m)^>\s?", "", visible_text)
        if re.search(r"not a\s+normative language specification", boundary_text) is None:
            self.add("book.boundary", path, "the manuscript must state its non-normative boundary")
        disclosure_text = " ".join(visible_text.split())
        v01_disclosure = all(
            marker in disclosure_text
            for marker in ("OpenAI Codex", "GPT-5", "Chase Bryan is the named author")
        )
        v02_disclosure = all(
            marker in disclosure_text
            for marker in (
                "Manuscript version 0.2 added Chapter 2",
                "drafted with OpenAI Codex, based on GPT-5",
                "under Chase Bryan's direction on 2026-07-14",
            )
        )
        if not v01_disclosure or not v02_disclosure:
            self.add(
                "book.disclosure",
                path,
                "the versioned AI-assistance and author-accountability disclosure is incomplete",
            )

    @staticmethod
    def _markdown_destination(raw_target: str) -> str:
        if raw_target.startswith("<") and ">" in raw_target:
            target = raw_target[1 : raw_target.index(">")]
        elif raw_target.startswith("#"):
            target = raw_target
        else:
            target = raw_target.split(maxsplit=1)[0]
        return re.sub(r"\\([\x21-\x2f\x3a-\x40\x5b-\x60\x7b-\x7e])", r"\1", target)

    def _validate_json_documents(self) -> None:
        for path in (path for path in self.repository_files if path.suffix.lower() == ".json"):
            try:
                self._load_repository_json(path)
            except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError) as exc:
                self.add("json.invalid", path, str(exc))

    def _validate_schema_fixtures(self) -> None:
        schema_dir = self.root / "schemas/gate0"
        schemas: dict[Path, Mapping[str, Any]] = {}
        id_registry: dict[str, tuple[Path, Mapping[str, Any]]] = {}
        schemas_with_audit_errors: set[Path] = set()
        schema_entries = self._inventory_files_in("schemas/gate0")
        if not schema_entries:
            return
        schema_paths = [path for path in schema_entries if path.name.endswith(".schema.json")]
        observed_schema_paths = {relative(path, self.root) for path in schema_paths}
        if observed_schema_paths != _SP:
            self.add(
                "schema.inventory",
                schema_dir,
                f"Gate 0 schema inventory must be exact; missing={sorted(_SP - observed_schema_paths)}, extra={sorted(observed_schema_paths - _SP)}",
            )
        for path in schema_paths:
            schema_bytes = self._read_authenticated_protected_file(relative(path, self.root))
            if schema_bytes is None:
                continue
            try:
                schema = _load_json_bytes(schema_bytes)
            except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError):
                continue
            if not isinstance(schema, dict):
                self.add("schema.type", path, "schema root must be an object")
                continue
            if schema.get("$schema") != SCHEMA_DIALECT:
                self.add("schema.dialect", path, f"schema must pin {SCHEMA_DIALECT}")
            schema_id = schema.get("$id")
            if not isinstance(schema_id, str) or not schema_id.startswith("urn:orange:gate0:"):
                self.add("schema.id", path, "$id must use the provisional urn:orange:gate0 namespace")
            elif schema_id in id_registry:
                self.add("schema.id_duplicate", path, f"duplicate schema $id {schema_id}")
            else:
                id_registry[schema_id] = (path, schema)
            audit_issues = audit_schema_vocabulary(schema)
            if audit_issues:
                schemas_with_audit_errors.add(path)
            for issue in audit_issues:
                self.add("schema.unsupported_keyword", path, issue)
            schemas[path] = schema
        for schema_path, schema in schemas.items():
            for location, node in iter_schema_nodes(schema):
                reference = node.get("$ref") if isinstance(node, dict) else None
                if isinstance(reference, str) and resolve_schema_ref(
                    reference,
                    schema_path,
                    schema,
                    schemas,
                    id_registry,
                ) is None:
                    self.add("schema.unresolved_ref", schema_path, f"unresolved $ref at {location}: {reference}")

        manifest_path = self.root / "conformance/foundation/manifest.json"
        if not self._hf(manifest_path):
            return
        try:
            manifest = self._load_repository_json(manifest_path)
        except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError):
            return
        entries = None
        if isinstance(manifest, dict):
            if set(manifest) != {"manifest_version", "record_status", "non_product", "cases"}:
                self.add("fixture.manifest_shape", manifest_path, "manifest fields must remain exact")
            if manifest.get("manifest_version") != "0.1" or manifest.get("record_status") != "provisional_gate0":
                self.add("fixture.manifest_version", manifest_path, "manifest version and provisional status must remain exact")
            if manifest.get("non_product") is not True:
                self.add("fixture.manifest_boundary", manifest_path, "conformance manifest must remain explicitly non-product")
            entries = manifest.get("cases")
        else:
            entries = manifest
        if not isinstance(entries, list):
            self.add("fixture.manifest", manifest_path, "manifest must contain a cases array")
            return
        observed_case_ids = tuple(
            entry.get("case_id") if isinstance(entry, dict) else None
            for entry in entries
        )
        if observed_case_ids != _CCI:
            self.add("fixture.case_inventory", manifest_path, "conformance case IDs and ordering must remain exact")
        seen_paths: set[str] = set()
        seen_case_ids: set[str] = set()
        coverage: dict[str, set[bool]] = {}
        for index, entry in enumerate(entries):
            label = f"fixtures[{index}]"
            if not isinstance(entry, dict):
                self.add("fixture.entry", manifest_path, f"{label} must be an object")
                continue
            case_id = entry.get("case_id")
            if not isinstance(case_id, str) or not case_id:
                self.add("fixture.case_id", manifest_path, f"{label} requires a non-empty case_id")
            elif case_id in seen_case_ids:
                self.add("fixture.case_id_duplicate", manifest_path, f"duplicate case_id {case_id}")
            else:
                seen_case_ids.add(case_id)
            fixture_value = entry.get("path", entry.get("instance"))
            schema_value = entry.get("schema")
            expected_valid = entry.get("expected_valid")
            if not isinstance(fixture_value, str) or not isinstance(schema_value, str) or not isinstance(expected_valid, bool):
                self.add("fixture.entry", manifest_path, f"{label} requires string path/schema and Boolean expected_valid")
                continue
            expected_fields = {"case_id", "instance", "schema", "expected_valid"}
            if expected_valid is False:
                expected_fields.add("expected_error")
            if set(entry) != expected_fields:
                self.add("fixture.entry_shape", manifest_path, f"{label} fields must remain exact")
            if fixture_value in seen_paths:
                self.add("fixture.duplicate", manifest_path, f"duplicate fixture path {fixture_value}")
            seen_paths.add(fixture_value)
            coverage.setdefault(schema_value, set()).add(expected_valid)
            fixture_path = safe_manifest_path(self.root, fixture_value)
            schema_path = safe_manifest_path(self.root, schema_value)
            if fixture_path is None or not relative(fixture_path, self.root).startswith(
                ("conformance/foundation/valid/", "conformance/foundation/invalid/")
            ):
                self.add("fixture.unsafe_path", manifest_path, f"fixture path escapes its allowed directories: {fixture_value}")
                continue
            expected_directory = (
                "conformance/foundation/valid/" if expected_valid else "conformance/foundation/invalid/"
            )
            if not relative(fixture_path, self.root).startswith(expected_directory):
                self.add("fixture.directory", manifest_path, f"{label} validity disagrees with its fixture directory")
            if schema_path is None or not relative(schema_path, self.root).startswith("schemas/gate0/"):
                self.add("fixture.unsafe_schema", manifest_path, f"schema path escapes schemas/gate0: {schema_value}")
                continue
            if not self._hf(fixture_path):
                self.add("fixture.missing", manifest_path, f"fixture does not exist: {fixture_value}")
                continue
            if schema_path not in schemas:
                self.add("fixture.schema_missing", manifest_path, f"schema is not registered: {schema_value}")
                continue
            if schema_path in schemas_with_audit_errors:
                continue
            try:
                instance = self._load_repository_json(fixture_path)
            except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError):
                continue
            issues = validate_schema_instance(
                instance,
                schemas[schema_path],
                schema_path,
                schemas,
                id_registry,
            )
            if not issues:
                issues.extend(validate_cross_record_invariants(instance, schema_path.name))
            if expected_valid and issues:
                first = issues[0]
                self.add(
                    "fixture.unexpected_invalid",
                    fixture_path,
                    f"{first.keyword} at {first.instance_path}: {first.message}",
                )
            elif not expected_valid and not issues:
                self.add("fixture.unexpected_valid", fixture_path, "adversarial fixture unexpectedly validates")
            elif not expected_valid:
                if len(issues) != 1:
                    observed = [(issue.keyword, display_instance_path(issue.instance_path)) for issue in issues]
                    self.add(
                        "fixture.error_cardinality",
                        fixture_path,
                        f"invalid fixture must produce exactly one canonical issue; observed {observed}",
                    )
                expected_error = entry.get("expected_error")
                if not isinstance(expected_error, dict):
                    self.add("fixture.expected_error", manifest_path, f"{label} must declare expected_error")
                    continue
                if set(expected_error) != {"code", "keyword", "instance_path"}:
                    self.add("fixture.expected_error_shape", manifest_path, f"{label} expected_error fields must remain exact")
                keyword = expected_error.get("keyword")
                instance_path = expected_error.get("instance_path")
                expected_code = expected_error.get("code")
                normalized_path = normalize_instance_path(instance_path)
                matching = [
                    issue
                    for issue in issues
                    if issue.keyword == keyword and issue.instance_path == normalized_path
                ]
                if not matching:
                    observed = ", ".join(f"{issue.keyword}@{display_instance_path(issue.instance_path)}" for issue in issues[:5])
                    self.add(
                        "fixture.error_mismatch",
                        fixture_path,
                        f"expected {keyword}@{instance_path}; observed {observed}",
                    )
                elif not isinstance(expected_code, str) or expected_code_for_issue(schema_path.name, matching[0]) != expected_code:
                    self.add(
                        "fixture.error_code_mismatch",
                        fixture_path,
                        f"expected stable error code {expected_code!r}; observed {expected_code_for_issue(schema_path.name, matching[0])!r}",
                    )

        fixture_files = {
            relative(path, self.root)
            for directory in (
                "conformance/foundation/valid",
                "conformance/foundation/invalid",
            )
            for path in self._inventory_files_in(directory, recursive=True)
            if path.suffix.lower() == ".json"
        }
        omitted = fixture_files - seen_paths
        for value in sorted(omitted):
            self.add("fixture.unregistered", value, "fixture is not listed in the conformance manifest")
        schema_values = {relative(path, self.root) for path in schemas}
        for schema_value in sorted(schema_values):
            observed = coverage.get(schema_value, set())
            if observed != {False, True}:
                self.add(
                    "fixture.coverage",
                    manifest_path,
                    f"{schema_value} requires at least one valid and one invalid fixture",
                )

    def _validate_workflows(self) -> None:
        workflow_dir = self.root / ".github/workflows"
        required = set(self.policy[_RW])
        workflow_paths = [
            path
            for path in self._inventory_files_in(".github/workflows")
            if PurePosixPath(path.name).match("*.y*ml")
        ]
        actual = {path.name for path in workflow_paths}
        if actual != _WI:
            self.add(
                "workflow.inventory",
                workflow_dir,
                f"workflow inventory must be exact; missing={sorted(_WI - actual)}, extra={sorted(actual - _WI)}",
            )
        for name in sorted(required - actual):
            self.add("workflow.required", f".github/workflows/{name}", "required workflow is missing")
        actions_policy = self.policy[_G]
        allowed = set(actions_policy.get(_AR, []))
        forbidden_events = set(actions_policy.get(_FE, []))
        allowed_writes = {
            name: set(values)
            for name, values in actions_policy.get(_WP, {}).items()
        }
        for path in workflow_paths:
            n = path.name
            text = self._rt(path)
            if text is None:
                continue
            lines = text.splitlines()
            active_text = yaml_without_comments(text)
            active_lines = active_text.splitlines()
            if re.search(r"\\u[0-9A-Fa-f]{4}", active_text):
                self.add("workflow.escape", path, "Unicode escapes are forbidden in workflow source")
            if re.search(r"(?m)^\s*[\"'][^\"']+[\"']\s*:", active_text):
                self.add("workflow.quoted_key", path, "quoted workflow keys are forbidden by the canonical source dialect")
            if re.search(r"(?m)^\s*[A-Za-z_][A-Za-z0-9_-]*\s+:", active_text):
                self.add("workflow.key_spacing", path, "whitespace before a YAML mapping colon is forbidden")
            if re.search(r"(?m)^(?:env| {4}env):", active_text):
                self.add("workflow.ambient_env", path, "workflow and job env are forbidden")
            if re.search(r"(?m)^ {4}(?:concurrency|environment|needs|outputs|strategy|uses):", active_text):
                self.add("workflow.job_extension", path, "job extension is not reviewed")
            if re.search(r"(?m)(?:^|[\s:{}\[\],-])(?:[&*][A-Za-z0-9_-]+|![A-Za-z0-9_!-]+|!<[^>\n]+>|<<\s*:)", active_text):
                self.add("workflow.indirection", path, "YAML anchors, aliases, merge keys, and tags are forbidden")
            if duplicate_yaml_mapping_key(active_text):
                self.add("workflow.duplicate_key", path, "duplicate YAML mapping keys are forbidden")
            if re.search(r"(?m)^\s*on:\s*[\[{]", active_text) or re.search(r"(?m)^\s*jobs:\s*[\[{]", active_text):
                self.add("workflow.flow_style", path, "on and jobs must use block-style YAML")
            if re.search(r"(?m)^\s{2}(?:pull_request|push|merge_group|schedule|workflow_dispatch):\s*[\[{]", active_text):
                self.add("workflow.event_flow_style", path, "workflow events must use block-style YAML")
            if re.search(r"(?m)^\s+(?:container|services)\s*:", active_text):
                self.add("workflow.container", path, "job containers and services are not admitted in Gate 0")
            if "permissions: {}" not in active_lines:
                self.add("workflow.permissions", path, "workflow must declare top-level permissions")
            if re.search(r"(?m)^\s*permissions:\s*write-all\s*$", text):
                self.add("workflow.write_all", path, "write-all permissions are forbidden")
            for event in forbidden_events:
                if re.search(rf"(?m)^[^#\n]*\b{re.escape(event)}\b", text):
                    self.add("workflow.forbidden_event", path, f"forbidden event {event}")
            if n == "ci.yml":
                for event in ("pull_request", "push", "merge_group"):
                    if not re.search(rf"(?m)^\s{{2}}{event}\s*:", text):
                        self.add("workflow.ci_event", path, f"required CI event is missing: {event}")
            if re.search(r"\bpaths(?:-ignore)?\s*:", active_text):
                self.add("workflow.path_filter", path, "protected workflows must not use path filters")
            for line_number, line in enumerate(lines, start=1):
                container_match = CONTAINER_ACTION_RE.search(line)
                match = None if container_match else ACTION_RE.search(line)
                if (
                    "uses:" in line
                    and not line.lstrip().startswith("#")
                    and match is None
                    and container_match is None
                ):
                    self.add("workflow.uses_syntax", path, f"line {line_number}: uses must use canonical unquoted block syntax")
                if container_match:
                    container_action, _ = container_match.groups()
                    self.add(
                        "workflow.container_action",
                        path,
                        f"line {line_number}: direct container Action syntax is not admitted: {container_action}",
                    )
                elif match:
                    action, ref, version = match.groups()
                    if action.startswith("./"):
                        self.add("workflow.local_action", path, f"line {line_number}: local composite actions are not admitted in Gate 0")
                        continue
                    if action not in allowed:
                        self.add("workflow.action_allowlist", path, f"line {line_number}: action not allowed: {action}")
                    if actions_policy.get(_FS) and not re.fullmatch(r"[0-9a-f]{40}", ref):
                        self.add("workflow.mutable_action", path, f"line {line_number}: action ref must be a full commit SHA")
                    if actions_policy.get(_VC) and not version:
                        self.add("workflow.version_comment", path, f"line {line_number}: pinned action needs a version comment")
                    elif version and not re.fullmatch(r"v[0-9]+(?:\.[0-9]+){1,2}(?:[-+][0-9A-Za-z.-]+)?", version):
                        self.add("workflow.version_comment", path, f"line {line_number}: invalid action version comment {version!r}")
                write_match = re.match(r"^\s+([a-z][a-z0-9-]*)\s*:\s*[\"']?write[\"']?(?:\s+#.*)?\s*$", line)
                if write_match and write_match.group(1) not in allowed_writes.get(n, set()):
                    self.add(
                        "workflow.write_permission",
                        path,
                        f"line {line_number}: {write_match.group(1)}: write is not allowed in this workflow",
                    )
            minutes = {_DR: 10, _SC: 20}.get(n, 15)
            for job_name, block in workflow_jobs(active_lines):
                block_text = "\n".join(block)
                if block.count("    runs-on: ubuntu-24.04") != 1:
                    self.add("workflow.runner", path, f"job {job_name} runner drift")
                if not re.search(rf"(?m)^\s{{4}}timeout-minutes:\s*{minutes}\s*$", block_text):
                    self.add("workflow.timeout", path, f"job {job_name} timeout drift")
                q = "    permissions:\n      contents: read" + ("\n      security-events: write" if n == _SC else "")
                if f"{q}\n    steps:" not in block_text:
                    self.add("workflow.job_permissions", path, f"job {job_name} permission drift")
            jobs = workflow_jobs(lines)
            if not jobs:
                self.add("workflow.jobs", path, "workflow must contain canonical two-space-indented jobs")
            for line_number in unsafe_run_interpolations(lines):
                self.add("workflow.untrusted_interpolation", path, f"untrusted event data is interpolated into run near line {line_number}")
            concurrency = top_level_block(active_lines, "concurrency")
            p = {"ci.yml": "required-ci", _SC: "openssf-scorecard"}.get(n, path.stem)
            c = "github.event.pull_request.number || github.ref" if n in {"ci.yml", _DR} else "github.ref"
            reviewed = (f"  group: {p}-${{{{ {c} }}}}", "  cancel-in-progress: true")
            if tuple(concurrency) != reviewed:
                self.add("workflow.concurrency", path, "concurrency contract drift")
            self._validate_required_workflow_content(path, active_text)

    def _validate_dependabot(self) -> None:
        path = self.root / ".github/dependabot.yml"
        if not self._hf(path):
            return
        source = self._rt(path)
        if source is None:
            return
        text = yaml_without_comments(source)
        required_patterns = (
            r"package-ecosystem:\s*[\"']?github-actions[\"']?",
            r"directory:\s*[\"']?/[\"']?",
            r"interval:\s*[\"']?weekly[\"']?",
        )
        for pattern in required_patterns:
            if not re.search(pattern, text):
                self.add("dependabot.configuration", path, f"missing required setting matching {pattern}")
        review_path = self.root / ".github/dependency-review-config.yml"
        if self._hf(review_path):
            review_source = self._rt(review_path)
            if review_source is None:
                return
            review = yaml_without_comments(review_source)
            required_review_settings = {
                "fail_on_severity: moderate": "moderate vulnerability threshold",
                "comment_summary_in_pr: never": "no write-permission PR comment",
                "warn_only: false": "fail-closed dependency result",
                "license_check: true": "license observation without an unratified allowlist",
                "vulnerability_check: true": "vulnerability evaluation",
            }
            for setting, meaning in required_review_settings.items():
                if setting not in review:
                    self.add("dependency_review.configuration", review_path, f"missing {meaning}: {setting}")

    def _validate_required_workflow_content(self, path: Path, text: str) -> None:
        n = path.name
        push = "  push:\n    branches:\n      - main"
        pull = push.replace("push", "pull_request")
        merge = "  merge_group:\n    types:\n      - checks_requested"
        dispatch = "\n  workflow_dispatch:"
        event_contracts = {
            "ci.yml": f"{pull}\n{push}\n{merge}",
            _DR: f"{pull}\n{merge}",
            _SC: f'{push}\n  schedule:\n    - cron: "41 5 * * 6"',
            _EL: f'{push}\n  schedule:\n    - cron: "23 4 * * 1"{dispatch}',
            _O: f'{push}\n  schedule:\n    - cron: "17 6 * * 3"{dispatch}',
        }
        if "\n".join(top_level_block(text.splitlines(), "on")) != event_contracts.get(n):
            self.add("workflow.event_contract", path, "workflow triggers must match their reviewed contract")
        defaults = tuple(top_level_block(text.splitlines(), "defaults"))
        reviewed_defaults = (
            ("  run:", "    shell: /bin/bash -p -e -o pipefail {0}")
            if n in {"ci.yml", _EL} else ()
        )
        if defaults != reviewed_defaults or re.search(r"(?m)^ {4}defaults:", text):
            self.add("workflow.defaults_contract", path, "run defaults must match the reviewed workflow contract")
        required_name = {"ci.yml": "Required CI / docs-policy-workflows", _DR: "Dependency Review / policy", _SC: "OpenSSF Scorecard / analysis", _EL: "External Links / scheduled audit", _O: "Workflow Online Audit / upstream metadata"}.get(n)
        if required_name and f"name: {required_name.split(' /')[0]}" not in text.splitlines()[:1]:
            self.add("workflow.name_contract", path, "workflow name drift")
        if required_name and f"    name: {required_name}" not in text.splitlines():
            self.add("workflow.required_content", path, f"missing protected job name: {required_name}")
        if n == _SC and re.search(r"(?m)^\s{2}workflow_dispatch\s*:", text):
            self.add("workflow.privileged_dispatch", path, "Scorecard must not allow manual ref selection")
        if "continue-on-error:" in text:
            self.add("workflow.continue_on_error", path, "continue-on-error is forbidden in Gate 0 workflows")
        expected_steps: dict[str, tuple[str, tuple[str, ...]]] = {
            "ci.yml": (
                "required",
                (
                    "Checkout",
                    "Enforce solo contribution boundary",
                    "Validate solo-bootstrap repository policy",
                    "Run foundation validator unit tests",
                    "Install selected Rust components",
                    "Validate Rust compiler",
                    "Lint Markdown",
                    "Install actionlint",
                    "Validate GitHub Actions workflows",
                    "Audit GitHub Actions security",
                ),
            ),
            _DR: ("review", ("Checkout", "Review dependency changes")),
            _SC: (
                "analysis",
                ("Checkout", "Run OpenSSF Scorecard", "Preserve SARIF result", "Upload result to code scanning"),
            ),
            _EL: ("links", ("Checkout", "Install checksum-verified lychee", "Check external links")),
            _O: ("metadata", ("Checkout", "Audit workflow source and upstream metadata")),
        }
        if n in expected_steps:
            job_name, names = expected_steps[n]
            jobs = dict(workflow_jobs(text.splitlines()))
            if set(jobs) != {job_name}:
                self.add("workflow.job_contract", path, f"workflow job set must be exactly {{{job_name}}}")
            if job_name not in jobs:
                self.add("workflow.step_contract", path, f"missing protected job {job_name}")
            else:
                conditions = tuple(
                    line.strip() for line in jobs[job_name] if line.startswith("    if:")
                )
                expected_conditions = (
                    ("if: ${{ github.ref == 'refs/heads/main' }}",) if n == _SC else ()
                )
                if conditions != expected_conditions:
                    self.add("workflow.job_condition_contract", path, "job condition must match its reviewed contract")
                steps = workflow_steps(jobs[job_name])
                observed_names = tuple(name for name, _ in steps)
                if observed_names != names:
                    self.add("workflow.step_contract", path, f"step sequence must be exact: {names}")
                self._validate_step_details(path, job_name, dict(steps))

    def _validate_step_details(self, path: Path, job_name: str, steps: Mapping[str, list[str]]) -> None:
        n = path.name
        checkout = yaml_without_comments("\n".join(steps.get("Checkout", [])))
        expected_checkout = '''      - name: Checkout
        uses: actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0
        with:
          fetch-depth: 1
          persist-credentials: false'''
        if checkout != expected_checkout:
            self.add("workflow.checkout_contract", path, f"{job_name}/Checkout must match the reviewed revision-bound contract")

        if n == "ci.yml":
            boundary = yaml_without_comments("\n".join(steps.get("Enforce solo contribution boundary", [])))
            expected_boundary = '''      - name: Enforce solo contribution boundary
        if: ${{ github.event_name == 'pull_request' && github.event.pull_request.user.login != 'chasebryan' }}
        run: |
          echo "Solo mode does not accept third-party pull requests until D-018 selects contribution terms." >&2
          exit 1'''
            if boundary != expected_boundary:
                self.add("workflow.solo_boundary_contract", path, "the solo contribution guard must match its reviewed fail-closed contract")
            ci_runs = {
                "Validate solo-bootstrap repository policy": _PR,
                "Run foundation validator unit tests": _PTR,
                "Install selected Rust components": 'run: /usr/bin/env -i HOME="$HOME" LANG=C LC_ALL=C PATH="$PATH" TZ=UTC rustup toolchain install 1.96.1 --profile minimal --component clippy,rustfmt --no-self-update',
                "Validate Rust compiler": _CR,
            }
            for step_name, expected_run in ci_runs.items():
                block = yaml_without_comments("\n".join(steps.get(step_name, [])))
                if block != f"      - name: {step_name}\n        {expected_run}":
                    self.add("workflow.ci_gate_contract", path, f"{job_name}/{step_name} must match its reviewed fail-closed command")
            ci_tools = {
                "Lint Markdown": '''      - name: Lint Markdown
        uses: DavidAnson/markdownlint-cli2-action@8de2aa07cae85fd17c0b35642db70cf5495f1d25
        with:
          globs: |
            **/*.md
            .github/**/*.md''',
                "Install actionlint": '''      - name: Install actionlint
        run: ./scripts/ci/install-actionlint "$RUNNER_TEMP/actionlint"''',
                "Validate GitHub Actions workflows": '''      - name: Validate GitHub Actions workflows
        run: |
          "$RUNNER_TEMP/actionlint/actionlint" -color''',
                "Audit GitHub Actions security": '''      - name: Audit GitHub Actions security
        uses: zizmorcore/zizmor-action@192e21d79ab29983730a13d1382995c2307fbcaa
        with:
          advanced-security: false
          annotations: false
          online-audits: false
          persona: pedantic
          version: "1.26.1"''',
            }
            for step_name, expected in ci_tools.items():
                if yaml_without_comments("\n".join(steps.get(step_name, []))) != expected:
                    self.add("workflow.ci_tool_contract", path, f"{job_name}/{step_name} must match its reviewed tool contract")
        elif n == _DR:
            review = yaml_without_comments("\n".join(steps.get("Review dependency changes", [])))
            expected_review = '''      - name: Review dependency changes
        uses: actions/dependency-review-action@a1d282b36b6f3519aa1f3fc636f609c47dddb294
        with:
          base-ref: ${{ github.event_name == 'merge_group' && github.event.merge_group.base_sha || github.event.pull_request.base.sha }}
          config-file: ./.github/dependency-review-config.yml
          head-ref: ${{ github.event_name == 'merge_group' && github.event.merge_group.head_sha || github.event.pull_request.head.sha }}'''
            if review != expected_review:
                self.add("workflow.dependency_review_contract", path, "dependency review must use only the reviewed configuration file and revision inputs")
        elif n == _SC:
            scorecard_block = yaml_without_comments("\n".join(steps.get("Run OpenSSF Scorecard", [])))
            expected_scorecard_block = '''      - name: Run OpenSSF Scorecard
        shell: /bin/bash -p -e -o pipefail {0}
        env:
          INPUT_REPO_TOKEN: ${{ github.token }}
        run: |
          set -euo pipefail
          test -n "${INPUT_REPO_TOKEN:-}"
          test -r "$GITHUB_EVENT_PATH"
          test -d "$GITHUB_WORKSPACE"
          printf '::add-mask::%s\\n' "$INPUT_REPO_TOKEN"
          /usr/bin/rm -f -- "$GITHUB_WORKSPACE/results.sarif"
          /usr/bin/env -i \\
            DOCKER_HOST=unix:///var/run/docker.sock \\
            GITHUB_API_URL="$GITHUB_API_URL" \\
            GITHUB_EVENT_NAME="$GITHUB_EVENT_NAME" \\
            GITHUB_REF="$GITHUB_REF" \\
            GITHUB_REPOSITORY="$GITHUB_REPOSITORY" \\
            HOME="$RUNNER_TEMP" \\
            INPUT_REPO_TOKEN="$INPUT_REPO_TOKEN" \\
            /usr/bin/docker run --rm \\
            --read-only \\
            --tmpfs /tmp:rw,noexec,nosuid,nodev,size=1g,mode=1777 \\
            --cap-drop=ALL \\
            --cap-add=DAC_OVERRIDE \\
            --security-opt=no-new-privileges=true \\
            --pids-limit=256 \\
            --mount "type=bind,source=${GITHUB_EVENT_PATH},target=/github/workflow/event.json,readonly" \\
            --mount "type=bind,source=${GITHUB_WORKSPACE},target=/github/workspace" \\
            --workdir /github/workspace \\
            --env GITHUB_ACTIONS=true \\
            --env GITHUB_API_URL \\
            --env GITHUB_EVENT_NAME \\
            --env GITHUB_EVENT_PATH=/github/workflow/event.json \\
            --env GITHUB_REF \\
            --env GITHUB_REPOSITORY \\
            --env GITHUB_WORKSPACE=/github/workspace \\
            --env INPUT_FILE_MODE=archive \\
            --env INPUT_PUBLISH_RESULTS=false \\
            --env INPUT_REPO_TOKEN \\
            --env INPUT_RESULTS_FILE=results.sarif \\
            --env INPUT_RESULTS_FORMAT=sarif \\
            ghcr.io/ossf/scorecard-action@sha256:2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941
          test -s "$GITHUB_WORKSPACE/results.sarif"'''  # noqa: E501
            if scorecard_block != expected_scorecard_block:
                self.add(
                    "workflow.scorecard_contract",
                    path,
                    f"{job_name}/Run OpenSSF Scorecard must match the reviewed Docker runtime contract exactly",
                )
            uploads = {
                "Preserve SARIF result": '''      - name: Preserve SARIF result
        if: ${{ always() && hashFiles('results.sarif') != '' }}
        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
        with:
          if-no-files-found: error
          name: openssf-scorecard-sarif
          path: results.sarif
          retention-days: 14''',
                "Upload result to code scanning": '''      - name: Upload result to code scanning
        if: ${{ always() && hashFiles('results.sarif') != '' }}
        uses: github/codeql-action/upload-sarif@99df26d4f13ea111d4ec1a7dddef6063f76b97e9
        with:
          sarif_file: results.sarif''',
            }
            for step_name, expected in uploads.items():
                if yaml_without_comments("\n".join(steps.get(step_name, []))) != expected:
                    self.add("workflow.scorecard_upload_contract", path, f"{job_name}/{step_name} must match its reviewed SARIF upload contract")
        elif n == _EL:
            commands = {
                "Install checksum-verified lychee": 'run: ./scripts/ci/install-lychee "$RUNNER_TEMP/lychee"',
                "Check external links": 'run: ./scripts/ci/check-external-links "$RUNNER_TEMP/lychee/bin/lychee"',
            }
            for step_name, command in commands.items():
                expected = f"      - name: {step_name}\n        {command}"
                if yaml_without_comments("\n".join(steps.get(step_name, []))) != expected:
                    self.add("workflow.external_links_contract", path, f"{job_name}/{step_name} must match its reviewed link-audit command")
        elif n == _O:
            block = yaml_without_comments("\n".join(steps.get("Audit workflow source and upstream metadata", [])))
            expected = '''      - name: Audit workflow source and upstream metadata
        uses: zizmorcore/zizmor-action@192e21d79ab29983730a13d1382995c2307fbcaa
        with:
          advanced-security: false
          annotations: false
          online-audits: true
          persona: pedantic
          version: "1.26.1"'''
            if block != expected:
                self.add("workflow.online_audit_contract", path, f"{job_name}/Audit workflow source and upstream metadata must match its reviewed online-audit contract")

    def _validate_codeowners(self) -> None:
        path = self.root / ".github/CODEOWNERS"
        if not self._hf(path):
            return
        text = self._rt(path)
        if text is None:
            return
        active_lines = {
            line.strip()
            for line in text.splitlines()
            if line.strip() and not line.lstrip().startswith("#")
        }
        for required in self.policy[_CO]:
            if required not in active_lines:
                self.add("codeowners.required", path, f"missing critical ownership rule: {required}")
        for line in active_lines:
            owners = line.split()[1:]
            if owners != ["@chasebryan"]:
                self.add("codeowners.bootstrap_owner", path, f"unratified or invalid bootstrap owner in: {line}")

    def _validate_decision_gates(self) -> None:
        path = self.root / "docs/DECISIONS.md"
        if not self._hf(path):
            return
        source = self._rt(path)
        if source is None:
            return
        text = markdown_without_fenced_blocks_and_comments(source)
        for gate, rule in self.policy["decision_gates"].items():
            decision = re.escape(rule.get("decision", ""))
            expected = rule.get(_RS)
            sections = list(re.finditer(
                rf"(?ms)^##\s+{decision}\b[^\n]*\n(?P<body>.*?)(?=^##\s+|\Z)",
                text,
            ))
            if len(sections) != 1:
                self.add(
                    "decision.duplicate" if sections else "decision.missing",
                    path,
                    f"expected exactly one semantic section for {gate}; observed {len(sections)}",
                )
                continue
            statuses = re.findall(r"(?m)^Status:\s*([a-z][a-z-]*)\b[^\n]*$", sections[0].group("body"))
            if len(statuses) != 1:
                self.add("decision.missing", path, f"cannot find status for {gate}")
            elif statuses[0] != str(expected).lower():
                self.add(
                    "decision.gate_changed",
                    path,
                    f"{gate} must remain {expected!r} until policy changes through governance",
                )

    def _validate_traceability(self) -> None:
        path = self.root / "docs/GATE0_TRACEABILITY.md"
        charter_path = self.root / "docs/PROJECT_CHARTER.md"
        decisions_path = self.root / "docs/DECISIONS.md"
        if not all(
            self._hf(candidate)
            for candidate in (path, charter_path, decisions_path)
        ):
            return
        source = self._rt(path)
        charter = self._read_repository_bytes(charter_path)
        decisions_source = self._rt(decisions_path)
        if source is None or charter is None or decisions_source is None:
            return
        text = markdown_without_fenced_blocks_and_comments(source)
        start_marker = b"## 5. In scope for the 1.0 product\n"
        end_marker = b"## 6. Explicit non-goals for 1.0\n"
        start = charter.find(start_marker)
        end = charter.find(end_marker, start + len(start_marker)) if start >= 0 else -1
        if start < 0 or end < 0:
            self.add("traceability.charter_section", charter_path, "cannot isolate the section 5 feature source")
        else:
            observed = hashlib.sha256(charter[start:end]).hexdigest()
            if observed != _CSH:
                self.add(
                    "traceability.charter_digest",
                    charter_path,
                    f"section 5 changed: expected {_CSH}, observed {observed}",
                )
        recorded_digest_count = sum(
            match.group() == _CSH
            for match in re.finditer(r"\b[0-9a-f]{64}\b", text)
        )
        if recorded_digest_count != 1:
            self.add(
                "traceability.recorded_digest",
                path,
                "traceability must record the exact reviewed charter-section SHA-256 once",
            )

        feature_section = markdown_section(text, "## 4. Feature matrix")
        feature_rows = table_rows(feature_section, r"F-[0-9]{2}")
        feature_ids = tuple(row[0] for row in feature_rows)
        if feature_ids != _FI:
            self.add("traceability.feature_ids", path, f"feature rows must be exact and ordered: {_FI}")
        known_decisions = {
            match.group(1)
            for match in re.finditer(
                r"(?m)^##\s+(D-[0-9]{3})\b",
                markdown_without_fenced_blocks_and_comments(decisions_source),
            )
        }
        trace_states: dict[str, str] = {}
        allowed_decision_states = {"accepted", "directed", "proposed", "investigate", "blocked", "superseded"}
        for row in feature_rows:
            if len(row) != 9:
                self.add("traceability.feature_shape", path, f"{row[0]} must contain exactly nine table fields")
                continue
            feature_id, feature, accountability, prerequisites, evidence, exit_test, states, target, trace_state = row
            for field_name, value in (
                ("feature", feature),
                ("accountability", accountability),
                ("prerequisites", prerequisites),
                ("evidence", evidence),
                ("exit test", exit_test),
                ("target", target),
            ):
                if not value.strip():
                    self.add("traceability.feature_value", path, f"{feature_id} has an empty {field_name}")
            if not re.search(r"\bW[1-7]\b", accountability):
                self.add("traceability.accountability", path, f"{feature_id} must name a workstream")
            evidence_match = re.match(r"`([NMCAXO](?:/[NMCAXO])*)`:", evidence)
            if evidence_match is None or len(set(evidence_match.group(1).split("/"))) != len(evidence_match.group(1).split("/")):
                self.add("traceability.evidence", path, f"{feature_id} has an invalid evidence-class declaration")
            observed_states = re.findall(r"`([^`]+)`", states)
            if not observed_states or any(state not in allowed_decision_states for state in observed_states):
                self.add("traceability.decision_state", path, f"{feature_id} has invalid decision state values")
            trace_values = re.findall(r"`([^`]+)`", trace_state)
            if trace_values not in (["mapped"], ["reviewed"]):
                self.add("traceability.trace_state", path, f"{feature_id} must be exactly mapped or reviewed")
            else:
                trace_states[feature_id] = trace_values[0]
            missing_decisions = sorted(set(re.findall(r"\bD-[0-9]{3}\b", " | ".join(row))) - known_decisions)
            if missing_decisions:
                self.add("traceability.decision_ref", path, f"{feature_id} references missing decisions: {missing_decisions}")

        attestation_section = markdown_section(text, "## 5. Review attestations")
        attestation_rows = table_rows(attestation_section, r"F-[0-9]{2}")
        if tuple(row[0] for row in attestation_rows) != _FI:
            self.add("traceability.attestation_ids", path, "review attestations must cover F-01 through F-14 exactly once in order")
        for row in attestation_rows:
            if len(row) != 6:
                self.add("traceability.attestation_shape", path, f"{row[0]} must contain exactly six table fields")
                continue
            feature_id, accountable, reviewer, revision, review_date, outcome = row
            complete = all(value not in {"", "—", "Unassigned"} for value in (accountable, reviewer, revision, review_date))
            reviewed = outcome == "`reviewed`"
            if complete != reviewed or (trace_states.get(feature_id) == "reviewed") != reviewed:
                self.add("traceability.attestation_state", path, f"{feature_id} trace state and attestation completeness disagree")
            if reviewed and parse_iso_date(review_date) is None:
                self.add("traceability.attestation_date", path, f"{feature_id} review date is not ISO 8601")
        normalized_text = re.sub(r"\s+", " ", text)
        for assertion in (
            "| Charter feature groups represented | 14/14 |",
            "| Structurally mapped rows | 14/14 |",
            "| Accountable people appointed and accepting | 0/14 |",
            "| Independently reviewed row mappings | 0/14 |",
            "| Feature exit tests evidenced | 0/14 |",
            "| Gate 0 exit criteria closed | 0/7 |",
        ):
            if assertion not in normalized_text:
                self.add("traceability.coverage_assertion", path, f"missing reviewed baseline assertion: {assertion}")

    def _validate_user_journeys(self) -> None:
        path = self.root / "docs/USER_JOURNEYS.md"
        if not self._hf(path):
            return
        source = self._rt(path)
        if source is None:
            return
        text = markdown_without_fenced_blocks_and_comments(source)
        persona_intro = markdown_section(text, "## 1. Purpose and limits")
        persona_rows = table_rows(persona_intro, r"P-[0-9]{2}")
        if tuple(row[0] for row in persona_rows) != _PI:
            self.add("journey.persona_ids", path, "persona definitions must cover P-01 through P-05 exactly once in order")

        index_section = markdown_section(text, "## 2. Journey index")
        journey_rows = table_rows(index_section, r"J-[0-9]{2}")
        if tuple(row[0] for row in journey_rows) != _JI:
            self.add("journey.index_ids", path, "journey index must cover J-01 through J-08 exactly once in order")
        operation_owners: dict[str, set[str]] = {value: set() for value in _OI}
        feature_owners: dict[str, set[str]] = {value: set() for value in _FI}
        primary_owners: dict[str, set[str]] = {value: set() for value in _PI}
        for row in journey_rows:
            if len(row) != 7:
                self.add("journey.index_shape", path, f"{row[0]} must contain exactly seven table fields")
                continue
            journey_id, title, primary, supporting, operations, features, target = row
            if not title or not target:
                self.add("journey.index_value", path, f"{journey_id} has an empty title or target gate")
            primary_ids = set(re.findall(r"\bP-[0-9]{2}\b", primary))
            supporting_ids = set(re.findall(r"\bP-[0-9]{2}\b", supporting))
            if not primary_ids or not primary_ids <= set(_PI) or not supporting_ids <= set(_PI):
                self.add("journey.persona_ref", path, f"{journey_id} has an invalid persona reference")
            for persona in primary_ids & set(_PI):
                primary_owners[persona].add(journey_id)
            operation_ids = set(re.findall(r"`([a-z]+(?:-[a-z]+)*)`", operations))
            if not operation_ids or not operation_ids <= set(_OI):
                self.add("journey.operation_ref", path, f"{journey_id} has an invalid operation reference")
            for operation in operation_ids & set(_OI):
                operation_owners[operation].add(journey_id)
            feature_ids = set(re.findall(r"\bF-[0-9]{2}\b", features))
            if not feature_ids or not feature_ids <= set(_FI):
                self.add("journey.feature_ref", path, f"{journey_id} has an invalid feature reference")
            for feature_id in feature_ids & set(_FI):
                feature_owners[feature_id].add(journey_id)

        specs = markdown_section(text, "## 3. Journey specifications")
        headings = tuple(re.findall(r"(?m)^###\s+(J-[0-9]{2})\b", specs))
        if headings != _JI:
            self.add("journey.spec_ids", path, "journey specifications must cover J-01 through J-08 exactly once in order")
        required_labels = (
            "Actors and intent",
            "Entry conditions and trusted inputs",
            "Ordered flow",
            "Fail-closed outcomes",
            "Evidence outputs",
            "Non-goals",
            "Completion test",
        )
        for journey_id in _JI:
            body = markdown_section(specs, f"### {journey_id}", heading_level=3, prefix=True)
            for label in required_labels:
                if f"**{label}:**" not in body:
                    self.add("journey.spec_field", path, f"{journey_id} is missing {label}")
            flow = body.split("**Ordered flow:**", 1)[1].split("**Fail-closed outcomes:**", 1)[0] if "**Ordered flow:**" in body and "**Fail-closed outcomes:**" in body else ""
            numbers = re.findall(r"(?m)^([1-9][0-9]*)\.\s", flow)
            expected_numbers = [str(index) for index in range(1, len(numbers) + 1)]
            if not numbers or numbers != expected_numbers:
                self.add("journey.flow_order", path, f"{journey_id} needs a consecutive ordered flow")

        if any(not owners for owners in primary_owners.values()):
            self.add("journey.persona_coverage", path, "every persona must own at least one primary journey")
        if any(not owners for owners in operation_owners.values()):
            self.add("journey.operation_coverage", path, "all ten operations require an owning journey")
        if any(not owners for owners in feature_owners.values()):
            self.add("journey.feature_coverage", path, "all fourteen features require journey coverage")
        self._validate_coverage_table(path, text, "Persona coverage", _PI, primary_owners, "journey.persona_matrix")
        self._validate_coverage_table(path, text, "Operation coverage", _OI, operation_owners, "journey.operation_matrix")
        self._validate_coverage_table(path, text, "Feature coverage", _FI, feature_owners, "journey.feature_matrix")
        normalized_text = re.sub(r"\s+", " ", text)
        for assertion in (
            "Persona coverage is 5/5.",
            "Operation coverage is 10/10.",
            "Feature coverage is 14/14.",
            "8/8 structurally specified journeys, 0/8 complete journeys, and 0/8",
        ):
            if assertion not in normalized_text:
                self.add("journey.coverage_assertion", path, f"missing reviewed baseline assertion: {assertion}")

    def _validate_coverage_table(
        self,
        path: Path,
        text: str,
        heading: str,
        expected_ids: Sequence[str],
        expected_owners: Mapping[str, set[str]],
        code: str,
    ) -> None:
        section = markdown_section(text, f"### {heading}", heading_level=3)
        identity_pattern = r"(?:P|F)-[0-9]{2}" if heading != "Operation coverage" else r"[a-z]+(?:-[a-z]+)*"
        rows = table_rows(section, identity_pattern, allow_backticks=heading == "Operation coverage")
        if tuple(row[0].strip("`") for row in rows) != tuple(expected_ids):
            self.add(code, path, f"{heading} identities must remain exact and ordered")
            return
        for row in rows:
            count_index = 3 if heading == "Persona coverage" else 2
            expected_length = count_index + 1
            if len(row) != expected_length or row[count_index] != "1/1":
                self.add(code, path, f"{row[0]} coverage row has the wrong shape or count")
                continue
            identity = row[0].strip("`")
            observed = set(re.findall(r"\bJ-[0-9]{2}\b", row[1]))
            if observed != expected_owners.get(identity, set()):
                self.add(code, path, f"{identity} owning journeys disagree with the journey index")
            if heading == "Persona coverage":
                supporting = set(re.findall(r"\bJ-[0-9]{2}\b", row[2]))
                if not supporting <= set(_JI) or supporting & observed:
                    self.add(code, path, f"{identity} has invalid or overlapping supporting journeys")

    def _validate_proof_foundation_suite(self) -> None:
        path = self.root / "docs/PROOF_FOUNDATION_DECISION_SUITE.md"
        if not self._hf(path):
            return
        source = self._rt(path)
        if source is None:
            return
        text = markdown_without_fenced_blocks_and_comments(source)
        candidate_rows = table_rows(markdown_section(text, "## 2. Candidate parity and frozen inputs"), r"C-[0-9]{2}")
        if tuple(row[0] for row in candidate_rows) != ("C-01", "C-02"):
            self.add("proof_suite.candidates", path, "candidate table must contain C-01 and C-02 exactly once in order")
        elif any(len(row) != 4 for row in candidate_rows):
            self.add("proof_suite.candidates", path, "candidate rows must contain exactly four table fields")
        elif [row[1] for row in candidate_rows] != ["Rocq", "Lean 4"]:
            self.add("proof_suite.candidates", path, "C-01 and C-02 must remain Rocq and Lean 4")
        for row in candidate_rows:
            if len(row) != 4 or row[3] != "0/7 cases":
                self.add("proof_suite.candidate_state", path, f"{row[0]} must retain the honest zero-evidence baseline")

        case_ids = tuple(f"DS-{index:02d}" for index in range(1, 8))
        cases = markdown_section(text, "## 3. Required decision cases")
        headings = tuple(re.findall(r"(?m)^###\s+(DS-[0-9]{2})\b", cases))
        if headings != case_ids:
            self.add("proof_suite.case_ids", path, "decision cases must cover DS-01 through DS-07 exactly once in order")
        required_labels = (
            "Question",
            "Dependencies",
            "Shared inputs",
            "Candidate outputs",
            "Positive checks",
            "Mutation and negative checks",
            "Hard acceptance",
        )
        for case_id in case_ids:
            body = markdown_section(cases, f"### {case_id}", heading_level=3, prefix=True)
            for label in required_labels:
                if f"**{label}:**" not in body:
                    self.add("proof_suite.case_field", path, f"{case_id} is missing {label}")

        metric_rows = table_rows(markdown_section(text, "## 4. Comparable metrics"), r"M-[0-9]{2}")
        metric_ids = tuple(f"M-{index:02d}" for index in range(1, 19))
        if tuple(row[0] for row in metric_rows) != metric_ids or any(len(row) != 4 for row in metric_rows):
            self.add("proof_suite.metrics", path, "metrics must cover M-01 through M-18 exactly once with four fields")
        hard_gates = markdown_section(text, "## 5. Hard gates and anti-gaming rules")
        gate_numbers = re.findall(r"(?m)^([1-9][0-9]*)\.\s", hard_gates)
        if gate_numbers != [str(index) for index in range(1, 9)]:
            self.add("proof_suite.hard_gates", path, "hard gates must remain the ordered non-compensable set 1 through 8")
        normalized_text = re.sub(r"\s+", " ", text)
        for assertion in (
            "There is no weighted aggregate score.",
            "The suite conclusion is exactly `recommend_rocq`, `recommend_lean`, `tie`, or",
            "Execution evidence is currently 0/2 candidates and 0/7 cases.",
            "Independent review is currently absent.",
        ):
            if assertion not in normalized_text:
                self.add("proof_suite.assertion", path, f"missing decision-protocol invariant: {assertion}")

    def _validate_product_form_decision_packet(self) -> None:
        path = self.root / "docs/PRODUCT_FORM_DECISION_PACKET.md"
        if not self._hf(path):
            return
        source = self._rt(path)
        if source is None:
            return
        text = markdown_without_fenced_blocks_and_comments(source)

        required_headings = (
            "Abstract",
            "Motivation",
            "Scope and non-goals",
            "Specification",
            "Alternatives",
            "Compatibility and migration",
            "Semantic and claim effects",
            "TCB, axiom, and proof effects",
            "Threat, abuse, and leakage effects",
            "Target and ABI effects",
            "Standards, errata, and provenance",
            "Dependencies, licenses, and IP",
            "Conformance, tests, and evidence",
            "Operations, release, and recovery",
            "Support and deprecation",
            "Unresolved questions",
            "Current disposition",
        )
        for heading in required_headings:
            if len(markdown_section(text, f"## {heading}").strip()) < 20:
                self.add(
                    "product_form.section",
                    path,
                    f"missing or empty substantive section: {heading}",
                )

        decision_gates = markdown_section(text, "### Decision gates", heading_level=3)
        gate_rows = table_rows(decision_gates, r"PF-G[0-9A-Z]+")
        gate_ids = tuple(f"PF-G{index:02d}" for index in range(1, 9))
        if tuple(row[0] for row in gate_rows) != gate_ids or any(
            len(row) != 2 for row in gate_rows
        ):
            self.add(
                "product_form.hard_gates",
                path,
                "hard-gate table must retain PF-G01 through PF-G08 exactly once in order",
            )

        candidate_rows = table_rows(decision_gates, r"PF-[0-9A-Z]+:.*")
        expected_candidate_rows = (
            (
                "PF-01: standalone editioned Orange DSL",
                "Pass",
                "Pass",
                "Pass",
                "Pass",
                "Pass",
                "Pass",
                "Pass",
                "Pass",
                "Recommend",
            ),
            (
                "PF-02: manifest-only orchestration",
                "Fail",
                "Pass",
                "Pass",
                "Pass",
                "Unproven",
                "Pass",
                "Pass",
                "Unproven",
                "Reject as product form; retain orchestration techniques",
            ),
            (
                r"PF-03: DSL embedded in F\*, Lean, or Rocq",
                "Pass",
                "Pass",
                "Pass",
                "Unproven",
                "Unproven",
                "Pass",
                "Pass",
                "Unproven",
                "Reject as product form; retain proof adapters",
            ),
            (
                "PF-04: Rust subset with proof annotations",
                "Pass",
                "Pass",
                "Pass",
                "Unproven",
                "Unproven",
                "Pass",
                "Pass",
                "Unproven",
                "Reject as product form; retain Rust implementation and integration paths",
            ),
        )
        if tuple(tuple(row) for row in candidate_rows) != expected_candidate_rows:
            self.add(
                "product_form.candidates",
                path,
                "candidate matrix must retain the four exact design-level assessments",
            )

        journey_rows = table_rows(
            markdown_section(text, "### Journey coverage", heading_level=3),
            r"J-[0-9A-Z]+",
        )
        journey_ids = tuple(f"J-{index:02d}" for index in range(1, 9))
        if tuple(row[0] for row in journey_rows) != journey_ids or any(
            len(row) != 2 for row in journey_rows
        ):
            self.add(
                "product_form.journeys",
                path,
                "journey coverage must retain J-01 through J-08 exactly once in order",
            )

        normalized_text = re.sub(r"\s+", " ", text)
        for assertion in (
            "Status: draft owner-review packet; no product form selected",
            "There is no weighted score;",
            "8/8 structurally specified design journeys and 0/8 completed journeys",
            "This is design coverage, not journey completion or user validation.",
            "No owner review or approval is recorded.",
            "The packet has no OEP number, intake or discussion reference, decision date, decision revision, approval record, or change authority.",
            "This packet does not accept D-003 or authorize S3b implementation.",
        ):
            if assertion not in normalized_text:
                self.add(
                    "product_form.assertion",
                    path,
                    f"missing decision-packet invariant: {assertion}",
                )

    def _validate_semantic_strata_suite(self) -> None:
        path = self.root / "docs/SEMANTIC_STRATA_DECISION_SUITE.md"
        if not self._hf(path):
            return
        source = self._rt(path)
        if source is None:
            return
        text = markdown_without_fenced_blocks_and_comments(source)

        candidate_rows = table_rows(
            markdown_section(text, "## 2. Candidate architectures"),
            r"ST-[A-Z]+",
        )
        candidate_ids = ("ST-REL", "ST-UNI", "ST-DUAL", "ST-MIRROR", "ST-HOST")
        candidate_names = (
            "Role-oriented related family",
            "Universal Core",
            "Pure/effect pair",
            "Five mirrored Cores",
            "Host-delegated strata",
        )
        candidate_shapes_valid = all(len(row) == 4 for row in candidate_rows)
        if tuple(row[0] for row in candidate_rows) != candidate_ids:
            self.add(
                "semantic_strata.candidates",
                path,
                "candidate table must retain the five exact ordered candidate identities",
            )
        elif not candidate_shapes_valid:
            self.add(
                "semantic_strata.candidates",
                path,
                "every candidate row must retain four fields",
            )
        elif tuple(row[1] for row in candidate_rows) != candidate_names:
            self.add(
                "semantic_strata.candidates",
                path,
                "candidate identities must retain their exact architecture names",
            )
        for row in candidate_rows:
            if len(row) != 4 or row[3] != "0/5 cases":
                self.add(
                    "semantic_strata.candidate_state",
                    path,
                    f"{row[0]} must retain the honest zero-evidence baseline",
                )

        relationship_rows = table_rows(
            markdown_section(text, "## 4. Required relationship graph"),
            r"SR-[0-9A-Z]+",
        )
        relationship_ids = tuple(f"SR-{index:02d}" for index in range(1, 15))
        if tuple(row[0] for row in relationship_rows) != relationship_ids or any(
            len(row) != 3 for row in relationship_rows
        ):
            self.add(
                "semantic_strata.relationships",
                path,
                "relationship graph must retain SR-01 through SR-14 exactly once in order with three fields",
            )

        case_ids = tuple(f"SC-{index:02d}" for index in range(1, 6))
        cases = markdown_section(text, "## 5. Required decision cases")
        case_headings = tuple(re.findall(r"(?m)^###\s+(SC-[0-9A-Z]+)\b", cases))
        if case_headings != case_ids:
            self.add(
                "semantic_strata.case_ids",
                path,
                "decision cases must retain SC-01 through SC-05 exactly once in order",
            )
        required_labels = (
            "Question",
            "Dependencies",
            "Inputs",
            "Required boundary observations",
            "Positive case",
            "Mutation and negative case",
            "Resource bounds",
            "Non-claims",
            "Falsification",
        )
        for case_id in case_ids:
            body = markdown_section(cases, f"### {case_id}", heading_level=3, prefix=True)
            for label in required_labels:
                if f"**{label}:**" not in body:
                    self.add(
                        "semantic_strata.case_field",
                        path,
                        f"{case_id} is missing {label}",
                    )

        hard_gates = markdown_section(text, "## 6. Hard gates and anti-gaming rules")
        gate_rows = re.findall(
            r"(?m)^([1-9][0-9]*)\.\s+\*\*(SS-G[0-9A-Z]+)\b",
            hard_gates,
        )
        expected_gates = tuple((str(index), f"SS-G{index:02d}") for index in range(1, 11))
        if tuple(gate_rows) != expected_gates:
            self.add(
                "semantic_strata.hard_gates",
                path,
                "hard gates must retain ordered non-compensable SS-G01 through SS-G10",
            )

        normalized_text = re.sub(r"\s+", " ", text)
        for assertion in (
            "There is no weighted aggregate score.",
            "Execution evidence is currently 0/5 candidates and 0/5 cases.",
            "Independent review is currently absent.",
            "No semantic stratum is selected by this draft suite.",
            "This suite does not accept D-003 or authorize S3b implementation.",
        ):
            if assertion not in normalized_text:
                self.add(
                    "semantic_strata.assertion",
                    path,
                    f"missing decision-protocol invariant: {assertion}",
                )

    def _validate_change_records(self) -> None:
        specifications = (
            (
                self.root / "docs/governance/oeps",
                "OEP",
                {"Draft", "Review", "Provisional", "Accepted", "Rejected", "Withdrawn", "Superseded"},
                {"Standards", "Process", "Informational", "Emergency"},
            ),
            (
                self.root / "docs/governance/adrs",
                "ADR",
                {"Proposed", "Accepted", "Rejected", "Superseded"},
                None,
            ),
        )
        for directory, prefix, statuses, types in specifications:
            seen: set[str] = set()
            candidates: list[Path] = []
            directory_value = relative(directory, self.root)
            for path in self._inventory_files_in(directory_value):
                if path.suffix.lower() != ".md":
                    continue
                if path.name == "README.md":
                    continue
                filename = RECORD_FILENAME_RE.fullmatch(path.name)
                if filename is None or filename.group("prefix") != prefix:
                    self.add("record.filename", path, f"invalid {prefix} record filename")
                    continue
                if filename.group("number") == "0000" and path.name != f"{prefix}-0000-template.md":
                    self.add("record.template", path, f"{prefix}-0000 is reserved for the template")
                candidates.append(path)
            for path in candidates:
                record_source = self._rt(path)
                if record_source is None:
                    continue
                parsed = parse_front_matter(record_source)
                if parsed is None:
                    self.add("record.front_matter", path, "numbered record requires YAML front matter")
                    continue
                metadata, parse_errors = parsed
                for message in parse_errors:
                    self.add("record.front_matter", path, message)
                expected_number = path.name.split("-", 2)[:2]
                expected = "-".join(expected_number)
                number = metadata.get("number")
                if number != expected:
                    self.add("record.number", path, f"metadata number must match filename: {expected}")
                if number in seen:
                    self.add("record.duplicate", path, f"duplicate record number {number}")
                seen.add(number or "")
                if metadata.get("status") not in statuses:
                    self.add("record.status", path, f"invalid {prefix} status {metadata.get('status')!r}")
                if types is not None and metadata.get("type") not in types:
                    self.add("record.type", path, f"invalid {prefix} type {metadata.get('type')!r}")
                required_keys = {"number", "title", "status"}
                if prefix == "OEP":
                    required_keys |= {
                        "approval-records",
                        "authors",
                        "champion",
                        "created",
                        "decision-date",
                        "decision-revision",
                        "discussion",
                        "related-adrs",
                        "related-decisions",
                        "requires",
                        "review-authorities",
                        "superseded-by",
                        "supersedes",
                        "type",
                        "updated",
                    }
                else:
                    required_keys |= {
                        "approval-records",
                        "date",
                        "decision-revision",
                        "owners",
                        "related-decisions",
                        "related-oeps",
                        "reviewers",
                        "superseded-by",
                        "supersedes",
                    }
                missing = sorted(required_keys - metadata.keys())
                if missing:
                    self.add("record.required", path, f"missing metadata keys: {', '.join(missing)}")
                is_template = number == f"{prefix}-0000"
                if not nonempty_scalar(metadata.get("title")):
                    self.add("record.value", path, "title must be a non-empty scalar")
                list_fields = (
                    ("authors", "review-authorities")
                    if prefix == "OEP"
                    else ("owners", "reviewers", "related-oeps")
                )
                for field in list_fields:
                    value = metadata.get(field)
                    if not isinstance(value, list) or not value or not all(nonempty_scalar(item) for item in value):
                        self.add("record.value", path, f"{field} must be a non-empty list of non-empty scalars")
                if prefix == "OEP":
                    if not nonempty_scalar(metadata.get("champion")):
                        self.add("record.value", path, "champion must be a non-empty scalar")
                    if not is_template:
                        created = parse_iso_date(metadata.get("created"))
                        updated = parse_iso_date(metadata.get("updated"))
                        if created is None or updated is None:
                            self.add("record.date", path, "created and updated must be exact ISO 8601 calendar dates")
                        elif updated < created:
                            self.add("record.date_order", path, "updated cannot precede created")
                elif not is_template and parse_iso_date(metadata.get("date")) is None:
                    self.add("record.date", path, "date must be an exact ISO 8601 calendar date")
                if not is_template:
                    required_headings = (
                        (
                            "Abstract",
                            "Motivation",
                            "Scope and non-goals",
                            "Specification",
                            "Alternatives",
                            "Compatibility and migration",
                            "Semantic and claim effects",
                            "TCB, axiom, and proof effects",
                            "Threat, abuse, and leakage effects",
                            "Conformance, tests, and evidence",
                            "Unresolved questions",
                            "Decision record",
                        )
                        if prefix == "OEP"
                        else (
                            "Context and constraints",
                            "Options considered",
                            "Decision",
                            "Security and assurance impact",
                            "Consequences and tradeoffs",
                            "Verification evidence",
                            "Rollback and revisit triggers",
                        )
                    )
                    source = markdown_without_fenced_blocks_and_comments(record_source)
                    for heading in required_headings:
                        body = markdown_section(source, f"## {heading}")
                        if len(body.strip()) < 20:
                            self.add("record.section", path, f"missing or empty substantive section: {heading}")
                if metadata.get("status") == "Accepted":
                    decision_revision = metadata.get("decision-revision")
                    approval_records = metadata.get("approval-records")
                    if not isinstance(decision_revision, str) or not re.fullmatch(r"[0-9a-f]{40}", decision_revision):
                        self.add(_RA, path, "Accepted record needs a full reviewed commit in decision-revision")
                    if not isinstance(approval_records, list) or not approval_records or not all(
                        nonempty_scalar(item) for item in approval_records
                    ):
                        self.add(_RA, path, "Accepted record needs immutable approval-record references")
                    if prefix == "OEP":
                        if isinstance(decision_revision, str) and re.fullmatch(r"[0-9a-f]{40}", decision_revision):
                            bound_revision = re.compile(
                                rf"(?<![0-9a-f]){re.escape(decision_revision)}(?![0-9a-f])"
                            )
                            if not isinstance(approval_records, list) or not any(
                                isinstance(item, str) and bound_revision.search(item)
                                for item in approval_records
                            ):
                                self.add(
                                    _RA,
                                    path,
                                    "Accepted OEP approval-records must bind the exact decision-revision",
                                )
                        if parse_iso_date(metadata.get("decision-date")) is None:
                            self.add(_RA, path, "Accepted OEP needs an exact decision-date")
                        related = metadata.get("related-decisions")
                        if not isinstance(related, list) or not related:
                            self.add(_RA, path, "Accepted OEP needs at least one related decision")
                        authorities = metadata.get("review-authorities")
                        if authorities != ["Orange Project Owner"]:
                            self.add(
                                "record.independence",
                                path,
                                "Accepted solo-mode OEP review-authorities must be exactly Orange Project Owner",
                            )
                        if not isinstance(approval_records, list) or not any(
                            isinstance(item, str) and re.search(r"(?<![A-Za-z0-9_])solo-reviewed(?![A-Za-z0-9_])", item)
                            for item in approval_records
                        ):
                            self.add(
                                _RA,
                                path,
                                "Accepted solo-mode OEP needs a literal solo-reviewed approval record",
                            )
                        if isinstance(approval_records, list) and any(
                            isinstance(item, str) and approval_record_claims_independence(item)
                            for item in approval_records
                        ):
                            self.add(
                                "record.independence",
                                path,
                                "solo-mode OEP approval records cannot claim independent review",
                            )
                    else:
                        owners = set(metadata.get("owners", [])) if isinstance(metadata.get("owners"), list) else set()
                        reviewers = set(metadata.get("reviewers", [])) if isinstance(metadata.get("reviewers"), list) else set()
                        if owners & reviewers:
                            self.add("record.independence", path, "ADR owners and reviewers must be distinct")

    def _validate_repository_templates(self) -> None:
        security = self.root / "SECURITY.md"
        if self._hf(security):
            security_text = self._rt(security)
            if (
                security_text is not None
                and "https://github.com/chasebryan/orange/security/advisories/new" not in security_text
            ):
                self.add("security.private_reporting", security, "private vulnerability-reporting URL is missing")
        pr = self.root / ".github/pull_request_template.md"
        if self._hf(pr):
            text = self._rt(pr)
            if text is None:
                return
            for heading in (
                "## Summary",
                "## Boundary and non-goals",
                "## Evidence and provenance",
                "## Impact assessment",
                "## Validation",
                "## Risk and rollback",
                "## Generated or AI-assisted material",
            ):
                if heading not in text:
                    self.add("template.pr_heading", pr, f"missing heading: {heading}")
            for area in (
                "Language semantics",
                "Public claims and evidence",
                "TCB and axioms",
                "Threat model and attack surface",
                "Dependencies, licenses, and provenance",
                "CI, release system, and keys",
            ):
                if f"| {area} |" not in text:
                    self.add("template.pr_impact", pr, f"missing impact row: {area}")


def markdown_slug(text: str) -> str:
    text = re.sub(r"<[^>]+>", "", text)
    text = re.sub(r"!?\[([^]]+)\]\([^)]+\)", r"\1", text)
    text = text.replace("`", "").strip().lower()
    return "".join(char for char in text if char.isalnum() or char in {" ", "-", "_"}).replace(" ", "-")


def markdown_anchors(text: str) -> set[str]:
    text = markdown_without_fenced_blocks_and_comments(text)
    anchors: set[str] = set()
    counts: dict[str, int] = {}
    for line in text.splitlines():
        match = HEADING_RE.match(line)
        if not match:
            continue
        base = markdown_slug(match.group(2))
        count = counts.get(base, 0)
        counts[base] = count + 1
        anchors.add(base if count == 0 else f"{base}-{count}")
    return anchors


def decode_uri_component(value: str) -> str | None:
    if re.search(r"%(?![0-9A-Fa-f]{2})", value):
        return None
    try:
        return unquote(value, errors="strict")
    except UnicodeDecodeError:
        return None


def markdown_fence_error(text: str) -> str | None:
    active_char = ""
    active_length = 0
    active_line = 0
    for line_number, line in enumerate(text.splitlines(), start=1):
        match = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
        if not match:
            continue
        marker, remainder = match.groups()
        char = marker[0]
        if not active_char:
            if char == "`" and "`" in remainder:
                continue
            active_char, active_length, active_line = char, len(marker), line_number
        elif char == active_char and len(marker) >= active_length and not remainder.strip():
            active_char, active_length, active_line = "", 0, 0
    if active_char:
        return f"unclosed {active_char * active_length} fence opened on line {active_line}"
    return None


def workflow_jobs(lines: Sequence[str]) -> list[tuple[str, list[str]]]:
    jobs_start = next((index for index, line in enumerate(lines) if line == "jobs:"), None)
    if jobs_start is None:
        return []
    result: list[tuple[str, list[str]]] = []
    current_name: str | None = None
    current: list[str] = []
    for line in lines[jobs_start + 1 :]:
        if line and not line.startswith((" ", "\t")):
            break
        match = re.match(r"^  ([A-Za-z0-9_-]+):\s*$", line)
        if match:
            if current_name is not None:
                result.append((current_name, current))
            current_name, current = match.group(1), [line]
        elif current_name is not None:
            current.append(line)
    if current_name is not None:
        result.append((current_name, current))
    return result


def workflow_steps(job_lines: Sequence[str]) -> list[tuple[str, list[str]]]:
    start = next((index for index, line in enumerate(job_lines) if line == "    steps:"), None)
    if start is None:
        return []
    result: list[tuple[str, list[str]]] = []
    current_name: str | None = None
    current: list[str] = []
    for line in job_lines[start + 1 :]:
        match = re.match(r"^\s{6}- name:\s+(.+?)\s*$", line)
        if match:
            if current_name is not None:
                result.append((current_name, current))
            current_name, current = match.group(1).strip("\"'"), [line]
        elif current_name is not None:
            current.append(line)
    if current_name is not None:
        result.append((current_name, current))
    return result


def top_level_block(lines: Sequence[str], name: str) -> list[str]:
    start = next((index for index, line in enumerate(lines) if line == f"{name}:"), None)
    if start is None:
        return []
    result: list[str] = []
    for line in lines[start + 1 :]:
        if line and not line.startswith((" ", "\t")):
            break
        result.append(line)
    return result


def yaml_without_comments(text: str) -> str:
    return "\n".join(strip_yaml_comment(line).rstrip() for line in text.splitlines() if strip_yaml_comment(line).strip())


def duplicate_yaml_mapping_key(text: str) -> str | None:
    seen: dict[int, set[str]] = {}
    block_indent = -1
    for line in text.splitlines():
        indent = len(line) - len(line.lstrip())
        if indent > block_indent >= 0:
            continue
        block_indent = -1
        source = line.lstrip()
        sequence = source.startswith("- ")
        if sequence:
            source = source[2:].lstrip()
        depth = indent + 2 if sequence else indent
        for prior_depth in tuple(seen):
            if prior_depth > (indent if sequence else depth):
                del seen[prior_depth]
        match = re.match(r"([A-Za-z_][A-Za-z0-9_-]*):", source)
        if not match:
            continue
        key = match.group(1)
        keys = seen.setdefault(depth, set())
        if key in keys:
            return key
        keys.add(key)
        if re.search(r":\s*[|>](?:[1-9][+-]?|[+-][1-9]?)?\s*$", source):
            block_indent = depth
    return None


def strip_yaml_comment(line: str) -> str:
    single = False
    double = False
    escaped = False
    for index, char in enumerate(line):
        if escaped:
            escaped = False
            continue
        if char == "\\" and double:
            escaped = True
            continue
        if char == "'" and not double:
            single = not single
            continue
        if char == '"' and not single:
            double = not double
            continue
        if char == "#" and not single and not double and (index == 0 or line[index - 1].isspace()):
            return line[:index]
    return line


def safe_manifest_path(root: Path, value: str) -> Path | None:
    if not isinstance(value, str) or not value:
        return None
    parts = value.split("/")
    if (
        re.match(r"^[A-Za-z]:", value)
        or "\\" in value
        or any(part in {"", ".", ".."} for part in parts)
        or any(ord(character) < 32 or ord(character) == 127 for character in value)
    ):
        return None
    try:
        pure = PurePosixPath(value)
        if pure.is_absolute():
            return None
        lexical_root = Path(os.path.normpath(os.fspath(root)))
        candidate = Path(os.path.normpath(os.fspath(lexical_root / pure)))
        candidate.relative_to(lexical_root)
    except (TypeError, ValueError):
        return None
    return candidate


def unsafe_run_interpolations(lines: Sequence[str]) -> list[int]:
    unsafe_fields = re.compile(
        r"\$\{\{\s*(?:github\.(?:event\.|(?:base_ref|head_ref|ref|ref_name)\b)|inputs\.)"
    )
    result: list[int] = []
    run_indent: int | None = None
    run_start = 0
    for index, line in enumerate(lines, start=1):
        indent = len(line) - len(line.lstrip())
        if run_indent is not None and line.strip() and indent <= run_indent:
            run_indent = None
        if re.match(r"^\s*run:\s*", line):
            run_indent, run_start = indent, index
            if unsafe_fields.search(line):
                result.append(index)
        elif run_indent is not None and unsafe_fields.search(line):
            result.append(run_start)
    return sorted(set(result))


def markdown_inline_link_targets(text: str) -> Iterable[str]:
    text_length = len(text)
    offset = 0
    line_end = -1
    last_delimiter: dict[str, int] = {}
    label_depth = 0
    escaped = False
    while offset < text_length:
        start = None
        for index in range(offset, text_length):
            character = text[index]
            if character in "\r\n":
                next_line = index + 1
                if character == "\r" and text.startswith("\n", next_line):
                    next_line += 1
                while next_line < text_length and text[next_line] in " \t":
                    next_line += 1
                if next_line >= text_length or text[next_line] in "\r\n":
                    label_depth = 0
                escaped = False
            elif escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == "[":
                label_depth += 1
            elif character == "]":
                if label_depth and text.startswith("(", index + 1):
                    label_depth -= 1
                    start = index + 2
                    break
                label_depth = max(0, label_depth - 1)
        if start is None:
            return
        for continuation in range(2):
            if start > line_end:
                line_end = text_length
                for marker in "\r\n":
                    position = text.find(marker, start)
                    if position >= 0:
                        line_end = min(line_end, position)
                last_delimiter = {
                    marker: text.rfind(marker, start, line_end) for marker in ">\"'"
                }
            if continuation or line_end == text_length or text[start:line_end].strip(" \t"):
                break
            start = line_end + 1
            if text[line_end] == "\r" and text.startswith("\n", start):
                start += 1
            while start < text_length and text[start] in " \t":
                start += 1
        depth = 0
        quote: str | None = None
        angle = text.startswith("<", start) and start < last_delimiter[">"]
        escaped = False
        for index in range(start, line_end):
            character = text[index]
            if escaped:
                escaped = False
                continue
            if character == "\\":
                escaped = True
                continue
            if quote is not None:
                if character == quote:
                    quote = None
                continue
            if angle:
                if character == ">":
                    angle = False
                continue
            if (
                character in "\"'"
                and (index == start or text[index - 1].isspace())
                and index < last_delimiter[character]
            ):
                quote = character
            elif character == "(":
                depth += 1
            elif character == ")":
                if depth:
                    depth -= 1
                else:
                    yield text[start:index]
                    offset = index + 1
                    break
        else:
            offset = line_end + 1
            label_depth = 0
            escaped = False
            continue


def markdown_without_fenced_blocks(text: str) -> str:
    result: list[str] = []
    fence_char: str | None = None
    fence_length = 0
    for line in text.splitlines():
        match = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
        if match:
            marker, remainder = match.groups()
            if fence_char is None:
                if marker[0] == "`" and "`" in remainder:
                    result.append(line)
                    continue
                fence_char, fence_length = marker[0], len(marker)
            elif marker[0] == fence_char and len(marker) >= fence_length and not remainder.strip():
                fence_char, fence_length = None, 0
            continue
        if fence_char is None:
            result.append(line)
    return "\n".join(result)


def markdown_with_masked_inline_syntax(text: str, syntax: str) -> str:
    result = list(text)
    runs = list(re.finditer(r"`+", text))
    opening_index = 0
    while opening_index < len(runs):
        opening = runs[opening_index]
        closing_index = opening_index + 1
        while closing_index < len(runs) and len(runs[closing_index].group()) != len(opening.group()):
            closing_index += 1
        if closing_index >= len(runs):
            opening_index += 1
            continue
        closing = runs[closing_index]
        for index in range(opening.start(), closing.end()):
            if result[index] in syntax:
                result[index] = " "
        opening_index = closing_index + 1
    return "".join(result)


def markdown_without_fenced_blocks_and_comments(text: str) -> str:
    prose = markdown_with_masked_inline_syntax(markdown_without_fenced_blocks(text), "<>")
    uncommented: list[str] = []
    offset = 0
    while offset < len(prose):
        opening = prose.find("<!--", offset)
        closing = prose.find("-->", offset)
        if closing >= 0 and (opening < 0 or closing < opening):
            uncommented.append(prose[offset:closing])
            offset = closing + 3
            continue
        if opening < 0:
            uncommented.append(prose[offset:])
            break
        uncommented.append(prose[offset:opening])
        closing = prose.find("-->", opening + 4)
        if closing < 0:
            break
        offset = closing + 3
    return "".join(uncommented)


def markdown_html_comment_error(text: str) -> str | None:
    open_ = False
    body = markdown_with_masked_inline_syntax(markdown_without_fenced_blocks(text), "<>")
    at = 0
    while at < len(body):
        begin = body.startswith("<!--", at)
        end = body.startswith("-->", at)
        if begin == open_ and (begin or end):
            return ("HTML comment closer without opener", "nested HTML comment opener")[open_]
        if begin or end:
            open_ = begin
        at += 4 if begin else 3 if end else 1
    return "unclosed HTML comment" if open_ else None


def markdown_section(
    text: str,
    heading: str,
    *,
    heading_level: int = 2,
    prefix: bool = False,
) -> str:
    suffix = r"[^\n]*" if prefix else ""
    match = re.search(rf"(?m)^{re.escape(heading)}{suffix}\s*$", text)
    if match is None:
        return ""
    next_heading = re.search(rf"(?m)^#{{1,{heading_level}}}\s+", text[match.end() :])
    end = match.end() + next_heading.start() if next_heading is not None else len(text)
    return text[match.end() : end]


def table_rows(section: str, identity_pattern: str, *, allow_backticks: bool = False) -> list[list[str]]:
    wrapper = "`?" if allow_backticks else ""
    pattern = re.compile(rf"^{wrapper}(?:{identity_pattern}){wrapper}$")
    result: list[list[str]] = []
    for line in section.splitlines():
        if not line.startswith("|") or not line.endswith("|"):
            continue
        fields = [field.strip() for field in line[1:-1].split("|")]
        if fields and pattern.fullmatch(fields[0]):
            result.append(fields)
    return result


def parse_front_matter_value(value: str) -> Any:
    if value == "[]":
        return []
    if value in {"null", "~"}:
        return None
    return value.strip("\"'")


def parse_front_matter(text: str) -> tuple[dict[str, Any], list[str]] | None:
    lines = text.splitlines()
    if not lines or lines[0] != "---":
        return None
    result: dict[str, Any] = {}
    errors: list[str] = []
    current_list: str | None = None
    for line_number, line in enumerate(lines[1:], start=2):
        if line == "---":
            return result, errors
        match = FRONT_MATTER_KEY_RE.match(line)
        if match:
            key, raw = match.group(1), (match.group(2) or "").strip()
            if key in result and len(errors) < _MF:
                errors.append(f"duplicate metadata key {key!r} on line {line_number}")
            result[key] = parse_front_matter_value(raw) if raw else []
            current_list = key if not raw else None
            continue
        list_item = re.fullmatch(r"  -\s+(.+?)\s*", line)
        if list_item and current_list is not None:
            value = parse_front_matter_value(list_item.group(1))
            if isinstance(result.get(current_list), list):
                result[current_list].append(value)
            continue
        if line.strip() and len(errors) < _MF:
            errors.append(f"unsupported metadata syntax on line {line_number}")
        current_list = None
    if len(errors) < _MF:
        errors.append("front matter is not closed")
    return result, errors
def nonempty_scalar(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


RUST_USIZE_MAXIMUM = (1 << 64) - 1
RUST_USIZE_MAXIMUM_DECIMAL_DIGITS = len(str(RUST_USIZE_MAXIMUM))
RUST_RAW_STRING_PREFIX_RE = re.compile(r'r(#{0,255})"')


def parse_rust_usize_product(value: str) -> int | None:
    if re.fullmatch(r"\s*[0-9][0-9_]*(?:\s*\*\s*[0-9][0-9_]*)*\s*", value) is None:
        return None
    result = 1
    for factor in value.split("*"):
        digits = factor.strip().replace("_", "")
        significant_digits = digits.lstrip("0") or "0"
        if len(significant_digits) > RUST_USIZE_MAXIMUM_DECIMAL_DIGITS:
            return None
        parsed = int(significant_digits, 10)
        if parsed > RUST_USIZE_MAXIMUM:
            return None
        if parsed != 0 and result > RUST_USIZE_MAXIMUM // parsed:
            return None
        result *= parsed
    return result


def rust_code_without_comments_and_literals(value: str) -> str:
    result = list(value)
    index = 0
    state = "code"
    block_depth = 0
    raw_closer = ""
    while index < len(value):
        if state == "code":
            raw = RUST_RAW_STRING_PREFIX_RE.match(value, index)
            if raw is not None:
                raw_closer = '"' + raw.group(1)
                length = raw.end() - index
                for offset in range(index, index + length):
                    result[offset] = " "
                index += length
                state = "raw"
            elif value.startswith("//", index):
                result[index : index + 2] = [" ", " "]
                index += 2
                state = "line"
            elif value.startswith("/*", index):
                result[index : index + 2] = [" ", " "]
                index += 2
                block_depth = 1
                state = "block"
            elif value[index] == '"':
                result[index] = " "
                index += 1
                state = "string"
            else:
                index += 1
        elif state == "line":
            if value[index] == "\n":
                state = "code"
            else:
                result[index] = " "
            index += 1
        elif state == "block":
            if value.startswith("/*", index):
                result[index : index + 2] = [" ", " "]
                index += 2
                block_depth += 1
            elif value.startswith("*/", index):
                result[index : index + 2] = [" ", " "]
                index += 2
                block_depth -= 1
                if block_depth == 0:
                    state = "code"
            else:
                if value[index] != "\n":
                    result[index] = " "
                index += 1
        elif state == "string":
            if value[index] == "\\" and index + 1 < len(value):
                result[index] = " "
                if value[index + 1] != "\n":
                    result[index + 1] = " "
                index += 2
            else:
                character = value[index]
                if character != "\n":
                    result[index] = " "
                index += 1
                if character == '"':
                    state = "code"
        else:
            if value.startswith(raw_closer, index):
                for offset in range(index, index + len(raw_closer)):
                    result[offset] = " "
                index += len(raw_closer)
                state = "code"
            else:
                if value[index] != "\n":
                    result[index] = " "
                index += 1
    return "".join(result)


def approval_record_claims_independence(value: str) -> bool:
    normalized = re.sub(r"[_-]+", " ", value.casefold())
    for claim in re.finditer(r"\bindependen(?:t|ce|tly)\b", normalized):
        prefix = normalized[max(0, claim.start() - 32) : claim.start()]
        suffix = normalized[claim.end() : claim.end() + 40]
        negated_before = re.search(
            r"\b(?:no|not|non|without|absent|missing|unavailable)(?:\s+(?:a|an|any))?\s*$",
            prefix,
        )
        negated_after = re.match(
            r"\s+(?:review\s+)?(?:was\s+|is\s+)?(?:absent|missing|unavailable|not available)\b",
            suffix,
        )
        if negated_before is None and negated_after is None:
            return True
    return False


def parse_iso_date(value: Any) -> dt.date | None:
    if not isinstance(value, str) or not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", value):
        return None
    try:
        return dt.date.fromisoformat(value)
    except ValueError:
        return None


SUPPORTED_SCHEMA_KEYWORDS = set(
    """$comment $defs $id $ref $schema additionalProperties allOf anyOf const
default deprecated description enum examples exclusiveMaximum exclusiveMinimum format
items maxItems maxLength maxProperties maximum minItems minLength minProperties minimum
multipleOf not oneOf pattern patternProperties prefixItems properties readOnly required title
type uniqueItems writeOnly""".split()
)


def audit_schema_vocabulary(schema: Any, location: str = "$") -> list[str]:
    issues: list[str] = []
    if isinstance(schema, bool):
        return issues
    if not isinstance(schema, dict):
        return [f"{location} is not a schema object or Boolean"]
    for key in schema:
        if key not in SUPPORTED_SCHEMA_KEYWORDS:
            issues.append(f"unsupported keyword {key!r} at {location}")
    string_keywords = ("$comment", "$id", "$ref", "$schema", "description", "format", "pattern", "title")
    for key in string_keywords:
        if key in schema and not isinstance(schema[key], str):
            issues.append(f"{key} must be a string at {location}")
    pattern = schema.get("pattern")
    if isinstance(pattern, str):
        try:
            re.compile(pattern)
        except re.error:
            issues.append(f"invalid pattern expression {pattern!r} at {location}")
    value = schema.get("format")
    if isinstance(value, str) and value not in ("date", "date-time", "uri", "uri-reference"):
        issues.append(f"unsupported format {value!r} at {location}")
    value = schema.get("$id")
    if isinstance(value, str) and (location != "$" or "#" in value or not valid_format(value, "uri")):
        issues.append(f"unsupported $id at {location}")
    if "$schema" in schema and location != "$":
        issues.append(f"unsupported nested $schema at {location}")
    type_value = schema.get("type")
    allowed_types = {"array", "boolean", "integer", "null", "number", "object", "string"}
    if type_value is not None:
        values = [type_value] if isinstance(type_value, str) else type_value
        if (
            not isinstance(values, list)
            or not values
            or not all(isinstance(value, str) and value in allowed_types for value in values)
            or len(values) != len(set(values))
        ):
            issues.append(f"type must name one or more unique JSON types at {location}")
    required = schema.get("required")
    if required is not None and (
        not isinstance(required, list)
        or not all(isinstance(value, str) and value for value in required)
        or len(required) != len(set(required))
    ):
        issues.append(f"required must be a unique string array at {location}")
    enum = schema.get("enum")
    if enum is not None:
        if not isinstance(enum, list) or not enum:
            issues.append(f"enum must be a non-empty array at {location}")
        elif len({canonical_json_bytes(value) for value in enum}) != len(enum):
            issues.append(f"enum values must be unique at {location}")
    for key in ("maxItems", "maxLength", "maxProperties", "minItems", "minLength", "minProperties"):
        if key in schema and (not isinstance(schema[key], int) or isinstance(schema[key], bool) or schema[key] < 0):
            issues.append(f"{key} must be a non-negative integer at {location}")
    for key in ("exclusiveMaximum", "exclusiveMinimum", "maximum", "minimum"):
        if key in schema and (not isinstance(schema[key], int) or isinstance(schema[key], bool)):
            issues.append(f"{key} must be an integer in the Gate 0 JSON profile at {location}")
    if "multipleOf" in schema and (
        not isinstance(schema["multipleOf"], int) or isinstance(schema["multipleOf"], bool) or schema["multipleOf"] <= 0
    ):
        issues.append(f"multipleOf must be a positive integer at {location}")
    for key in ("deprecated", "readOnly", "uniqueItems", "writeOnly"):
        if key in schema and not isinstance(schema[key], bool):
            issues.append(f"{key} must be Boolean at {location}")
    if "examples" in schema and not isinstance(schema["examples"], list):
        issues.append(f"examples must be an array at {location}")
    for key in ("$defs", "properties", "patternProperties"):
        value = schema.get(key)
        if value is not None and not isinstance(value, dict):
            issues.append(f"{key} must be an object at {location}")
        elif isinstance(value, dict):
            for child_name, child in value.items():
                if key == "patternProperties":
                    try:
                        re.compile(child_name)
                    except re.error as exc:
                        issues.append(f"invalid patternProperties expression {child_name!r} at {location}: {exc}")
                issues.extend(audit_schema_vocabulary(child, f"{location}/{key}/{child_name}"))
    for key in ("additionalProperties", "items", "not"):
        value = schema.get(key)
        if value is not None and not isinstance(value, (dict, bool)):
            issues.append(f"{key} must be a schema object or Boolean at {location}")
        elif isinstance(value, (dict, bool)):
            issues.extend(audit_schema_vocabulary(value, f"{location}/{key}"))
    for key in ("allOf", "anyOf", "oneOf", "prefixItems"):
        value = schema.get(key)
        if value is not None and (not isinstance(value, list) or not value):
            issues.append(f"{key} must be a non-empty schema array at {location}")
        elif isinstance(value, list):
            for index, child in enumerate(value):
                issues.extend(audit_schema_vocabulary(child, f"{location}/{key}/{index}"))
    return issues


def iter_schema_nodes(schema: Mapping[str, Any] | bool, location: str = "$") -> Iterable[tuple[str, Mapping[str, Any] | bool]]:
    yield location, schema
    if not isinstance(schema, dict):
        return
    for key in ("$defs", "properties", "patternProperties"):
        value = schema.get(key)
        if isinstance(value, dict):
            for name, child in value.items():
                if isinstance(child, (dict, bool)):
                    yield from iter_schema_nodes(child, f"{location}/{key}/{name}")
    for key in ("additionalProperties", "items", "not"):
        child = schema.get(key)
        if isinstance(child, (dict, bool)):
            yield from iter_schema_nodes(child, f"{location}/{key}")
    for key in ("allOf", "anyOf", "oneOf", "prefixItems"):
        value = schema.get(key)
        if isinstance(value, list):
            for index, child in enumerate(value):
                if isinstance(child, (dict, bool)):
                    yield from iter_schema_nodes(child, f"{location}/{key}/{index}")


def validate_schema_instance(
    instance: Any,
    schema: Mapping[str, Any] | bool,
    schema_path: Path,
    schemas: Mapping[Path, Mapping[str, Any]],
    id_registry: Mapping[str, tuple[Path, Mapping[str, Any]]],
    instance_path: str = "$",
    root_schema: Mapping[str, Any] | None = None,
    depth: int = 0,
) -> list[SchemaIssue]:
    if depth > 100:
        return [SchemaIssue("$ref", instance_path, "schema recursion limit exceeded")]
    if schema is False:
        return [SchemaIssue("falseSchema", instance_path, "Boolean false schema rejects the instance")]
    if schema is True:
        return []
    if root_schema is None:
        root_schema = schema
    issues: list[SchemaIssue] = _BoundedSchemaIssues()
    if "$ref" in schema:
        resolved = resolve_schema_ref(str(schema["$ref"]), schema_path, root_schema, schemas, id_registry)
        if resolved is None:
            return [SchemaIssue("$ref", instance_path, f"unresolved schema reference {schema['$ref']}")]
        target_path, target_schema, target_root = resolved
        issues.extend(
            validate_schema_instance(
                instance,
                target_schema,
                target_path,
                schemas,
                id_registry,
                instance_path,
                target_root,
                depth + 1,
            )
        )
    expected_type = schema.get("type")
    if expected_type is not None:
        expected = [expected_type] if isinstance(expected_type, str) else expected_type
        if not isinstance(expected, list) or not any(json_type_matches(instance, value) for value in expected):
            return [SchemaIssue("type", instance_path, f"expected JSON type {expected_type!r}")]
    if "const" in schema and canonical_json_bytes(instance) != canonical_json_bytes(schema["const"]):
        issues.append(SchemaIssue("const", instance_path, "value does not equal const"))
    if "enum" in schema and all(
        canonical_json_bytes(instance) != canonical_json_bytes(candidate) for candidate in schema["enum"]
    ):
        issues.append(SchemaIssue("enum", instance_path, "value is not in enum"))

    for keyword in ("allOf", "anyOf", "oneOf"):
        branches = schema.get(keyword)
        if not isinstance(branches, list):
            continue
        results = [
            validate_schema_instance(
                instance,
                branch,
                schema_path,
                schemas,
                id_registry,
                instance_path,
                root_schema,
                depth + 1,
            )
            for branch in branches
        ]
        valid_count = sum(not result for result in results)
        if keyword == "allOf":
            for result in results:
                issues.extend(result)
        elif keyword == "anyOf" and valid_count == 0:
            issues.append(SchemaIssue("anyOf", instance_path, "no anyOf branch matched"))
        elif keyword == "oneOf" and valid_count != 1:
            issues.append(SchemaIssue("oneOf", instance_path, f"expected one matching branch, observed {valid_count}"))
    if "not" in schema:
        not_issues = validate_schema_instance(
            instance,
            schema["not"],
            schema_path,
            schemas,
            id_registry,
            instance_path,
            root_schema,
            depth + 1,
        )
        if not not_issues:
            issues.append(SchemaIssue("not", instance_path, "instance matches forbidden schema"))

    if isinstance(instance, str):
        if "minLength" in schema and len(instance) < schema["minLength"]:
            issues.append(SchemaIssue("minLength", instance_path, "string is too short"))
        if "maxLength" in schema and len(instance) > schema["maxLength"]:
            issues.append(SchemaIssue("maxLength", instance_path, "string is too long"))
        if "pattern" in schema:
            try:
                pattern = re.compile(schema["pattern"])
            except (re.error, TypeError):
                issues.append(SchemaIssue("pattern", instance_path, "schema pattern expression is invalid"))
            else:
                if pattern.search(instance) is None:
                    issues.append(SchemaIssue("pattern", instance_path, "string does not match pattern"))
        if "format" in schema and not valid_format(instance, schema["format"]):
            issues.append(SchemaIssue("format", instance_path, f"invalid {schema['format']} value"))
    if isinstance(instance, (int, float)) and not isinstance(instance, bool):
        if "minimum" in schema and instance < schema["minimum"]:
            issues.append(SchemaIssue("minimum", instance_path, "number is below minimum"))
        if "maximum" in schema and instance > schema["maximum"]:
            issues.append(SchemaIssue("maximum", instance_path, "number is above maximum"))
        if "exclusiveMinimum" in schema and instance <= schema["exclusiveMinimum"]:
            issues.append(SchemaIssue("exclusiveMinimum", instance_path, "number is not above exclusive minimum"))
        if "exclusiveMaximum" in schema and instance >= schema["exclusiveMaximum"]:
            issues.append(SchemaIssue("exclusiveMaximum", instance_path, "number is not below exclusive maximum"))
        if "multipleOf" in schema and instance % schema["multipleOf"] != 0:
            issues.append(SchemaIssue("multipleOf", instance_path, "number is not a required multiple"))
    if isinstance(instance, list):
        if "minItems" in schema and len(instance) < schema["minItems"]:
            issues.append(SchemaIssue("minItems", instance_path, "array has too few items"))
        if "maxItems" in schema and len(instance) > schema["maxItems"]:
            issues.append(SchemaIssue("maxItems", instance_path, "array has too many items"))
        if schema.get("uniqueItems"):
            canonical = [json.dumps(value, sort_keys=True, separators=(",", ":")) for value in instance]
            if len(canonical) != len(set(canonical)):
                issues.append(SchemaIssue("uniqueItems", instance_path, "array items are not unique"))
        prefix_items = schema.get("prefixItems", [])
        if isinstance(prefix_items, list):
            for index, child_schema in enumerate(prefix_items[: len(instance)]):
                issues.extend(
                    validate_schema_instance(
                        instance[index],
                        child_schema,
                        schema_path,
                        schemas,
                        id_registry,
                        f"{instance_path}/{index}",
                        root_schema,
                        depth + 1,
                    )
                )
        item_schema = schema.get("items")
        if isinstance(item_schema, (dict, bool)):
            start = len(prefix_items) if isinstance(prefix_items, list) else 0
            for index, value in enumerate(instance[start:], start=start):
                issues.extend(
                    validate_schema_instance(
                        value,
                        item_schema,
                        schema_path,
                        schemas,
                        id_registry,
                        f"{instance_path}/{index}",
                        root_schema,
                        depth + 1,
                    )
                )
    if isinstance(instance, dict):
        if "minProperties" in schema and len(instance) < schema["minProperties"]:
            issues.append(SchemaIssue("minProperties", instance_path, "object has too few properties"))
        if "maxProperties" in schema and len(instance) > schema["maxProperties"]:
            issues.append(SchemaIssue("maxProperties", instance_path, "object has too many properties"))
        for name in schema.get("required", []):
            if name not in instance:
                issues.append(SchemaIssue("required", instance_path, f"missing required property {name!r}"))
        properties = schema.get("properties", {})
        pattern_properties = schema.get("patternProperties", {})
        evaluated: set[str] = set()
        if isinstance(properties, dict):
            for name, child_schema in properties.items():
                if name in instance:
                    evaluated.add(name)
                    issues.extend(
                        validate_schema_instance(
                            instance[name],
                            child_schema,
                            schema_path,
                            schemas,
                            id_registry,
                            f"{instance_path}/{pointer_escape(name)}",
                            root_schema,
                            depth + 1,
                        )
                    )
        if isinstance(pattern_properties, dict):
            for pattern, child_schema in pattern_properties.items():
                for name, value in instance.items():
                    if re.search(pattern, name):
                        evaluated.add(name)
                        issues.extend(
                            validate_schema_instance(
                                value,
                                child_schema,
                                schema_path,
                                schemas,
                                id_registry,
                                f"{instance_path}/{pointer_escape(name)}",
                                root_schema,
                                depth + 1,
                            )
                        )
        additional = schema.get("additionalProperties", True)
        for name in sorted(instance.keys() - evaluated):
            child_path = f"{instance_path}/{pointer_escape(name)}"
            if additional is False:
                issues.append(SchemaIssue("additionalProperties", child_path, "unexpected property"))
            elif isinstance(additional, dict):
                issues.extend(
                    validate_schema_instance(
                        instance[name],
                        additional,
                        schema_path,
                        schemas,
                        id_registry,
                        child_path,
                        root_schema,
                        depth + 1,
                    )
                )
    return issues


def resolve_schema_ref(
    reference: str,
    schema_path: Path,
    root_schema: Mapping[str, Any],
    schemas: Mapping[Path, Mapping[str, Any]],
    id_registry: Mapping[str, tuple[Path, Mapping[str, Any]]],
) -> tuple[Path, Mapping[str, Any] | bool, Mapping[str, Any]] | None:
    if not has_valid_rfc3986_lexical_form(reference):
        return None
    document_ref, marker, fragment = reference.partition("#")
    if document_ref.startswith("/"):
        return None
    target_path = schema_path
    target_root = root_schema
    if document_ref:
        if document_ref in id_registry:
            target_path, target_root = id_registry[document_ref]
        else:
            candidate = Path(
                os.path.normpath(os.fspath(schema_path.parent / document_ref))
            )
            if candidate not in schemas:
                return None
            target_path, target_root = candidate, schemas[candidate]
    target: Any = target_root
    if marker and fragment:
        try:
            fragment = unquote(fragment, errors="strict")
        except UnicodeDecodeError:
            return None
        if not fragment.startswith("/"):
            return None
        for token in fragment[1:].split("/"):
            if re.search(r"~(?:[^01]|$)", token):
                return None
            token = token.replace("~1", "/").replace("~0", "~")
            if not isinstance(target, dict) or token not in target:
                return None
            target = target[token]
    if not isinstance(target, (dict, bool)):
        return None
    return target_path, target, target_root


def json_type_matches(value: Any, expected: Any) -> bool:
    return {
        "null": value is None,
        "boolean": isinstance(value, bool),
        "object": isinstance(value, dict),
        "array": isinstance(value, list),
        "number": isinstance(value, (int, float)) and not isinstance(value, bool),
        "integer": isinstance(value, int) and not isinstance(value, bool),
        "string": isinstance(value, str),
    }.get(expected, False)


RFC3986_LITERAL_CHARACTERS = frozenset(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
    "0123456789-._~:/?#[]@!$&'()*+,;="
)
RFC3986_HEX_DIGITS = frozenset("0123456789ABCDEFabcdef")


def has_valid_rfc3986_lexical_form(value: str) -> bool:
    if not value.isascii():
        return False
    for index, character in enumerate(value):
        if character == "%":
            if index + 2 >= len(value):
                return False
            if value[index + 1] not in RFC3986_HEX_DIGITS:
                return False
            if value[index + 2] not in RFC3986_HEX_DIGITS:
                return False
        elif character not in RFC3986_LITERAL_CHARACTERS:
            return False
    return True


def valid_format(value: str, format_name: str) -> bool:
    try:
        if format_name == "date":
            dt.date.fromisoformat(value)
            return bool(re.fullmatch(r"\d{4}-\d{2}-\d{2}", value))
        if format_name == "date-time":
            if re.fullmatch(
                r"[0-9]{4}-[0-9]{2}-[0-9]{2}[Tt][0-9]{2}:[0-9]{2}:[0-9]{2}(?:\.[0-9]+)?(?:[Zz]|[+-][0-9]{2}:[0-9]{2})",
                value,
            ) is None:
                return False
            parsed = dt.datetime.fromisoformat(value[:-1] + "+00:00" if value[-1] in "Zz" else value)
            return parsed.tzinfo is not None
        if format_name in ("uri", "uri-reference"):
            if not has_valid_rfc3986_lexical_form(value):
                return False
            parsed = urlsplit(value)
            if "#" in parsed.fragment or any(
                "[" in component or "]" in component
                for component in (parsed.path, parsed.query, parsed.fragment)
            ):
                return False
            if re.fullmatch(
                r"(?:[^@\[\]]*@)?(?:\[[^\[\]]+\]|[^:@\[\]]*)(?::[0-9]*)?",
                parsed.netloc,
            ) is None:
                return False
            return format_name == "uri-reference" or bool(parsed.scheme)
    except (TypeError, ValueError):
        return False
    return False


def pointer_escape(value: str) -> str:
    return value.replace("~", "~0").replace("/", "~1")


def normalize_instance_path(value: Any) -> str:
    if value in (None, "", "/"):
        return "$" if value in (None, "") else "$/"
    if isinstance(value, str) and value.startswith("$"):
        return value
    if isinstance(value, str) and value.startswith("/"):
        return "$" + value
    return str(value)


def display_instance_path(value: str) -> str:
    return value[1:] if value.startswith("$") else value


def validate_cross_record_invariants(instance: Any, schema_name: str) -> list[SchemaIssue]:
    issues: list[SchemaIssue] = []
    if not isinstance(instance, dict):
        return issues
    if schema_name == "claim-record-v0.1.schema.json":
        if instance.get("outcome") == "satisfied":
            basis = instance.get("basis", [])
            has_checked_basis = isinstance(basis, list) and any(
                isinstance(item, dict)
                and item.get("type") != "assumption"
                and item.get("verification_state") == "checked"
                for item in basis
            )
            if not has_checked_basis:
                issues.append(
                    SchemaIssue(
                        _X,
                        "$/basis",
                        "a satisfied claim requires a checked non-assumption basis",
                    )
                )
        assumptions = identifiers(instance.get("assumptions"), "assumption_id", "$/assumptions", issues)
        identifiers(instance.get("basis"), "basis_id", "$/basis", issues)
        for index, basis in enumerate(instance.get("basis", [])):
            if isinstance(basis, dict) and basis.get("type") == "assumption":
                reference = basis.get("assumption_ref")
                if reference not in assumptions:
                    issues.append(
                        SchemaIssue(
                            _X,
                            f"$/basis/{index}/assumption_ref",
                            "assumption basis reference does not resolve",
                        )
                    )
    elif schema_name == "evidence-manifest-v0.1.schema.json":
        files = instance.get("files", [])
        file_paths = identifiers(files, "path", "$/files", issues)
        if isinstance(files, list) and [item.get("path") for item in files if isinstance(item, dict)] != sorted(file_paths):
            issues.append(SchemaIssue(_X, "$/files", "file records must be ordered by path"))
        external = instance.get("external_sources", [])
        if isinstance(external, list):
            identifiers(external, "source_id", "$/external_sources", issues, optional=True)
        replay = instance.get("replay")
        if isinstance(replay, dict):
            toolchains = replay.get("toolchains", [])
            names = identifiers(toolchains, "name", "$/replay/toolchains", issues)
            if isinstance(toolchains, list) and [item.get("name") for item in toolchains if isinstance(item, dict)] != sorted(names):
                issues.append(SchemaIssue(_X, "$/replay/toolchains", "toolchains must be ordered by name"))
    elif schema_name == "repository-control-snapshot-v0.1.schema.json":
        sources = identifiers(instance.get("evidence_sources"), "evidence_id", "$/evidence_sources", issues)
        for path, references in repository_control_evidence_refs(instance):
            for reference in references:
                if reference not in sources:
                    issues.append(
                        SchemaIssue(
                            _X,
                            path,
                            f"repository-control evidence reference does not resolve: {reference}",
                        )
                    )
    elif schema_name == "standards-provenance-v0.1.schema.json":
        standards = instance.get("standards", [])
        standard_ids = identifiers(standards, "standard_id", "$/standards", issues)
        if isinstance(standards, list) and [item.get("standard_id") for item in standards if isinstance(item, dict)] != sorted(standard_ids):
            issues.append(SchemaIssue(_X, "$/standards", "standards must be ordered by standard_id"))
        for index, standard in enumerate(standards if isinstance(standards, list) else []):
            if not isinstance(standard, dict):
                continue
            identifiers(standard.get("errata"), "erratum_id", f"$/standards/{index}/errata", issues)
            identifiers(standard.get("normative_clauses"), "clause_id", f"$/standards/{index}/normative_clauses", issues)
            identifiers(standard.get("vector_sources"), "vector_id", f"$/standards/{index}/vector_sources", issues)
    elif schema_name == "trust-inventory-v0.1.schema.json":
        components = identifiers(instance.get("components"), "component_id", "$/components", issues)
        axioms = identifiers(instance.get("axioms"), "axiom_id", "$/axioms", issues)
        models = identifiers(instance.get("trusted_models"), "model_id", "$/trusted_models", issues)
        contracts = identifiers(instance.get("external_contracts"), "contract_id", "$/external_contracts", issues)
        for index, component in enumerate(instance.get("components", [])):
            if isinstance(component, dict) and component.get("trust_role") == "assumed":
                if component.get("assumption_ref") not in axioms:
                    issues.append(
                        SchemaIssue(
                            _X,
                            f"$/components/{index}/assumption_ref",
                            "assumed component reference does not resolve to an axiom",
                        )
                    )
        closure_specs = (
            ("component_refs", components),
            ("axiom_refs", axioms),
            ("model_refs", models),
            ("contract_refs", contracts),
        )
        for index, closure in enumerate(instance.get("claim_closures", [])):
            if not isinstance(closure, dict):
                continue
            for field, valid_ids in closure_specs:
                for reference in closure.get(field, []):
                    if reference not in valid_ids:
                        issues.append(
                            SchemaIssue(
                                _X,
                                f"$/claim_closures/{index}/{field}",
                                f"trust-closure reference does not resolve: {reference}",
                            )
                        )
    return issues


def identifiers(
    value: Any,
    field: str,
    instance_path: str,
    issues: list[SchemaIssue],
    optional: bool = False,
) -> set[str]:
    result: set[str] = set()
    if not isinstance(value, list):
        return result
    for index, item in enumerate(value):
        if not isinstance(item, dict):
            continue
        identifier = item.get(field)
        if identifier is None and optional:
            continue
        if not isinstance(identifier, str):
            continue
        if identifier in result:
            issues.append(
                SchemaIssue(
                    _X,
                    f"{instance_path}/{index}/{field}",
                    f"duplicate identifier {identifier}",
                )
            )
        result.add(identifier)
    return result


def repository_control_evidence_refs(instance: Mapping[str, Any]) -> list[tuple[str, list[str]]]:
    result: list[tuple[str, list[str]]] = []
    security = instance.get("security_features")
    if isinstance(security, dict):
        for name, control in security.items():
            if isinstance(control, dict) and isinstance(control.get(_E), list):
                result.append((f"$/security_features/{pointer_escape(name)}/evidence_refs", control[_E]))
    actions = instance.get("actions")
    if isinstance(actions, dict):
        enabled = actions.get("enabled")
        if isinstance(enabled, dict) and isinstance(enabled.get(_E), list):
            result.append(("$/actions/enabled/evidence_refs", enabled[_E]))
    for field in ("default_branch_policy", "merge_policy"):
        value = instance.get(field)
        if isinstance(value, dict) and isinstance(value.get(_E), list):
            result.append((f"$/{field}/evidence_refs", value[_E]))
    return result


def expected_code_for_issue(schema_name: str, issue: SchemaIssue) -> str:
    if issue.keyword == _X and schema_name.startswith("claim-record-"):
        return "CLAIM_SATISFIED_WITHOUT_CHECKED_BASIS"
    if issue.keyword == "const":
        return "SCHEMA_CONST"
    if issue.keyword == "pattern":
        return "SCHEMA_PATTERN"
    if issue.keyword == "oneOf" and issue.instance_path.startswith("$/security_features/"):
        return "CONTROL_STATE_REQUIRES_EXPLANATION"
    if issue.keyword == "oneOf" and issue.instance_path.startswith("$/components/"):
        return "TRUST_ROLE_REQUIRES_IDENTITY"
    return f"SCHEMA_{issue.keyword.upper()}"


def asserted_repository_root(value: str) -> Path:
    try:
        candidate = Path(value).resolve(strict=True)
    except (OSError, RuntimeError):
        raise argparse.ArgumentTypeError(
            "must resolve to the checkout containing this validator"
        ) from None
    if candidate != VALIDATOR_REPOSITORY_ROOT:
        raise argparse.ArgumentTypeError(
            "must resolve to the checkout containing this validator"
        )
    return VALIDATOR_REPOSITORY_ROOT


def parse_arguments(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=asserted_repository_root,
        default=VALIDATOR_REPOSITORY_ROOT,
        metavar="PATH",
        help="assert the checkout containing this validator; cannot redirect validation",
    )
    parser.add_argument("--format", choices=("text", "json"), default="text", help="output format")
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    arguments = parse_arguments(sys.argv[1:] if argv is None else argv)
    repository_root = arguments.root
    validator = FoundationValidator(repository_root)
    findings = validator.run()
    if arguments.format == "json":
        output = {
            "schema_version": "0.1.0",
            "repository": validator.policy.get("repository", "unknown"),
            _PV: validator.policy.get(_PV, "unknown"),
            "valid": not findings,
            "findings": [finding.as_dict() for finding in findings],
        }
        json.dump(output, sys.stdout, sort_keys=True, separators=(",", ":"))
        sys.stdout.write("\n")
    elif findings:
        for finding in findings:
            print(
                f"{_text_report_field(finding.path)}: {finding.code}: "
                f"{_text_report_field(finding.message)}"
            )
        print(f"Solo-bootstrap repository policy failed with {len(findings)} finding(s).")
    else:
        print(
            "Solo-bootstrap repository policy passed "
            f"({validator.policy['repository']} policy {validator.policy['policy_version']})."
        )
    return 1 if findings else 0


def _cli() -> int:
    try:
        try:
            status = main()
        except SystemExit as error:
            status = error.code
        sys.stdout.flush()
        sys.stderr.flush()
    except OSError:
        fd = os.open(os.devnull, os.O_WRONLY)
        for stream in (sys.stdout, sys.stderr):
            try:
                os.dup2(fd, stream.fileno())
            except OSError:
                pass
        os.close(fd)
        return 1
    return status


if __name__ == "__main__":
    raise SystemExit(_cli())
