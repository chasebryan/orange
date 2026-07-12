#!/usr/bin/env python3
"""Deterministically validate Orange's solo-bootstrap repository foundation.

This is repository-policy tooling, not the Orange product checker and not a
general JSON Schema implementation. It deliberately supports and audits the
small JSON Schema vocabulary used by the provisional Gate 0 evidence fixtures.
"""

from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import hashlib
import json
import os
import re
import subprocess
import sys
import tomllib
import unicodedata
from pathlib import Path, PurePosixPath
from typing import Any, Iterable, Mapping, Sequence
from urllib.parse import unquote, urlsplit


POLICY_PATH = Path("policy/gate0-repository-policy.json")
IGNORED_PARTS = {".git", ".agents", ".codex", "__pycache__"}
BINARY_SUFFIXES = {".gif", ".jpeg", ".jpg", ".png", ".wasm"}
TEXT_TAB_FREE_SUFFIXES = {".json", ".jsonc", ".or", ".py", ".rs", ".sh", ".toml", ".yaml", ".yml"}
SCHEMA_DIALECT = "https://json-schema.org/draft/2020-12/schema"
MINIMUM_REQUIRED_PATHS = {
    ".editorconfig",
    ".gitattributes",
    ".github/CODEOWNERS",
    ".github/ISSUE_TEMPLATE/conduct-contact.yml",
    ".github/ISSUE_TEMPLATE/config.yml",
    ".github/ISSUE_TEMPLATE/oep-proposal.yml",
    ".github/ISSUE_TEMPLATE/planning-defect.yml",
    ".github/ISSUE_TEMPLATE/planning-question.yml",
    ".github/ISSUE_TEMPLATE/research-evidence.yml",
    ".github/dependabot.yml",
    ".github/dependency-review-config.yml",
    ".github/pull_request_template.md",
    ".github/workflows/ci.yml",
    ".github/workflows/dependency-review.yml",
    ".github/workflows/external-links.yml",
    ".github/workflows/scorecard.yml",
    ".github/workflows/workflow-online-audit.yml",
    ".gitignore",
    ".markdownlint-cli2.jsonc",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "compiler/.gitignore",
    "compiler/Cargo.lock",
    "compiler/Cargo.toml",
    "compiler/README.md",
    "compiler/crates/orange-compiler/Cargo.toml",
    "compiler/crates/orange-compiler/src/diagnostic.rs",
    "compiler/crates/orange-compiler/src/edition.rs",
    "compiler/crates/orange-compiler/src/lexer.rs",
    "compiler/crates/orange-compiler/src/lib.rs",
    "compiler/crates/orange-compiler/src/parser.rs",
    "compiler/crates/orange-compiler/src/source.rs",
    "compiler/crates/orangec/Cargo.toml",
    "compiler/crates/orangec/src/main.rs",
    "compiler/crates/orangec/tests/cli.rs",
    "compiler/fixtures/hello.or",
    "DEPENDENCY_POLICY.md",
    "GOVERNANCE.md",
    "Makefile",
    "README.md",
    "RELEASE_POLICY.md",
    "rust-toolchain.toml",
    "SECURITY.md",
    "SUPPORT.md",
    "assets/brand/README.md",
    "assets/brand/manifest.json",
    "assets/brand/orange-banner-jpeg.JPEG",
    "assets/brand/orange-banner.png",
    "assets/brand/orange-banner2-erased.PNG",
    "assets/brand/orange-banner2.PNG",
    "assets/brand/orange-erased.PNG",
    "assets/brand/orange.jpg",
    "assets/brand/orange.png",
    "assets/brand/orangePNG.PNG",
    "conformance/foundation/manifest.json",
    "conformance/foundation/README.md",
    "docs/DECISIONS.md",
    "docs/GATE0_TRACEABILITY.md",
    "docs/GATE0_SUPPORT_ENVELOPES.md",
    "docs/LANGUAGE_2026.md",
    "docs/PROOF_FOUNDATION_DECISION_SUITE.md",
    "docs/REPRODUCIBILITY.md",
    "docs/USER_JOURNEYS.md",
    "docs/ARCHITECTURE.md",
    "docs/ASSURANCE.md",
    "docs/PROJECT_CHARTER.md",
    "docs/RESEARCH.md",
    "docs/ROADMAP.md",
    "docs/governance/adrs/ADR-0000-template.md",
    "docs/governance/adrs/README.md",
    "docs/governance/oeps/OEP-0000-template.md",
    "docs/governance/oeps/README.md",
    "docs/operations/CI_DEPENDENCIES.md",
    "docs/operations/GITHUB_CONTROLS.md",
    "docs/security/OSPS_BASELINE.md",
    "docs/security/SECRETS_AND_INCIDENTS.md",
    "docs/security/THREAT_MODEL.md",
    "policy/README.md",
    "policy/gate0-repository-policy.json",
    "schemas/README.md",
    "schemas/gate0/claim-record-v0.1.schema.json",
    "schemas/gate0/evidence-manifest-v0.1.schema.json",
    "schemas/gate0/repository-control-snapshot-v0.1.schema.json",
    "schemas/gate0/standards-provenance-v0.1.schema.json",
    "schemas/gate0/trust-inventory-v0.1.schema.json",
    "scripts/ci/check-repository",
    "scripts/ci/check-external-links",
    "scripts/ci/install-actionlint",
    "scripts/ci/install-lychee",
    "tools/validate_foundation.py",
    "tools/tests/test_validate_foundation.py",
    "tools/tests/test_validate_foundation_hardening.py",
}
MINIMUM_FORBIDDEN_PATHS = {"COPYING", "LICENSE", "crates", "crypto", "formal", "release", "spec", "stdlib", "targets"}
MINIMUM_REQUIRED_WORKFLOWS = {"ci.yml", "dependency-review.yml", "scorecard.yml"}
MINIMUM_ACTION_REPOSITORIES = {
    "DavidAnson/markdownlint-cli2-action",
    "actions/checkout",
    "actions/dependency-review-action",
    "actions/upload-artifact",
    "github/codeql-action/upload-sarif",
    "zizmorcore/zizmor-action",
}
GATE0_ALLOWED_CONTAINER_IMAGES = {
    "ghcr.io/ossf/scorecard-action@sha256:"
    "2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941"
}
GATE0_ALLOWED_BINARY_ARTIFACTS = [
    {
        "path": "assets/brand/orange-banner2.PNG",
        "sha256": "3136916eab9747871324cf146158e8f3a16197dbf32e8a6ef995056705dd6e5b",
        "role": "Official working Orange wordmark on a light background",
        "provenance": "Byte-for-byte import from the steward-supplied Orange-Assets collection on 2026-07-11",
    },
    {
        "path": "assets/brand/orangePNG.PNG",
        "sha256": "64d2e78436586466f9c24fb844922e1d7b474e98a6023b44a5a481533300ec02",
        "role": "Official working Orange emblem source variant on a light background",
        "provenance": "Byte-for-byte import from the steward-supplied Orange-Assets collection on 2026-07-11",
    },
    {
        "path": "assets/brand/orange-banner-jpeg.JPEG",
        "sha256": "288070ed86afd83a2e41e25fb664ac3ef44029521055a6ca3f6b6223cc48d41a",
        "role": "Official working Orange horizontal lockup JPEG",
        "provenance": "Byte-for-byte import from the steward-supplied Orange-Assets collection on 2026-07-11",
    },
    {
        "path": "assets/brand/orange-banner2-erased.PNG",
        "sha256": "5941784f123c7a3fb7922d859098d43d5aee10dbd8db4c9283a32b5f93e8611c",
        "role": "Official working Orange transparent wordmark",
        "provenance": "Byte-for-byte import from the steward-supplied Orange-Assets collection on 2026-07-11",
    },
    {
        "path": "assets/brand/orange-erased.PNG",
        "sha256": "9f256a98c1cbe7345ab29372fdc15eb9475ce3b89c4278af503d167d4a91f2f2",
        "role": "Official working Orange transparent emblem",
        "provenance": "Byte-for-byte import from the steward-supplied Orange-Assets collection on 2026-07-11",
    },
    {
        "path": "assets/brand/orange-banner.png",
        "sha256": "41cffe77744da07b9fbf9bc46c009755522468bbbc53a3f3f9b1a867ae05e266",
        "role": "Official working Orange primary horizontal lockup with embedded C2PA claim",
        "provenance": "Byte-for-byte import from the steward-supplied Orange-Assets collection on 2026-07-11",
    },
    {
        "path": "assets/brand/orange.jpg",
        "sha256": "170c48ab4a32bea289099b9505569ada5b99cc6deae93ece8f59d5c2102f4888",
        "role": "Official working Orange emblem JPEG on a light background",
        "provenance": "Byte-for-byte import from the steward-supplied Orange-Assets collection on 2026-07-11",
    },
    {
        "path": "assets/brand/orange.png",
        "sha256": "c10ed0b2d79a1e9447e842fcb9eaa7ec8eeb850dd2873e87eefd54d7cdc14463",
        "role": "Official working Orange primary emblem with embedded C2PA claim",
        "provenance": "Byte-for-byte import from the steward-supplied Orange-Assets collection on 2026-07-11",
    },
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
}
GATE0_EXECUTABLE_PATHS = {
    "scripts/ci/check-external-links",
    "scripts/ci/check-repository",
    "scripts/ci/install-actionlint",
    "scripts/ci/install-lychee",
    "tools/validate_foundation.py",
}
GATE0_ALLOWED_WRITE_PERMISSIONS = {"scorecard.yml": {"security-events"}}
GATE0_HOSTED_REPOSITORY_CONTROLS = {
    "snapshot_date": "2026-07-11",
    "review_due_date": "2026-10-11",
    "main_ruleset_id": 18810248,
    "required_checks": [
        {"context": "Required CI / docs-policy-workflows", "integration_id": 15368},
        {"context": "Dependency Review / policy", "integration_id": 15368},
    ],
}
GATE0_SCHEMA_PATHS = {
    "schemas/gate0/claim-record-v0.1.schema.json",
    "schemas/gate0/evidence-manifest-v0.1.schema.json",
    "schemas/gate0/repository-control-snapshot-v0.1.schema.json",
    "schemas/gate0/standards-provenance-v0.1.schema.json",
    "schemas/gate0/trust-inventory-v0.1.schema.json",
}
GATE0_WORKFLOW_INVENTORY = {
    "ci.yml",
    "dependency-review.yml",
    "external-links.yml",
    "scorecard.yml",
    "workflow-online-audit.yml",
}
GATE0_PROTECTED_FILE_DIGESTS = {
    ".editorconfig": "a3766d51a21a904a405f808017eeb34d5426558ad487803a5d4f39a854379ca9",
    ".gitattributes": "a5f4501e4eeea215d890813156f46edf1cd9dee83be968e5ff5edc6c136f111d",
    ".github/CODEOWNERS": "8038a3a61117a29c26bfdd7b66a9a5675cb779736bad2c8c1797e680d7484663",
    ".github/ISSUE_TEMPLATE/conduct-contact.yml": "93f6aeacff7e7fe45c94ee1f5fbaf95c1d49c90c11e5887fe955e3fd92915541",
    ".github/ISSUE_TEMPLATE/config.yml": "ff5a8f986c0a9902d402ac17eecd3fbea8783ad396ba0b33133650826054ffe3",
    ".github/ISSUE_TEMPLATE/oep-proposal.yml": "7fa038f4caf7efb85bb05a98bb180b3d160f205aa54a0ae32afe7805a55222f8",
    ".github/ISSUE_TEMPLATE/planning-defect.yml": "b190eccb90a1097bd18b53e114429a08c26c6a84bd9bc606789c5a38fe6952ec",
    ".github/ISSUE_TEMPLATE/planning-question.yml": "a2936886eb6f13e234eda5cf49923565fcd107539015df93c1245038534b9c2b",
    ".github/ISSUE_TEMPLATE/research-evidence.yml": "60fb04d67cb5acbc822fb9ea613ab0d3b8caebb35544daefe91cf0f59a408f7a",
    ".github/dependabot.yml": "7ff6d88203254cab787bde78ac277edcf21fd159a1f3e547102af7e2f163e268",
    ".github/dependency-review-config.yml": "66279d4dec898deb6e178692a949c0e48cd0daef7d5928ab415549518d6c8b09",
    ".github/pull_request_template.md": "52b5a877ad9360f8b6c6a8429e77f1c98cd48c54c093f312fb7fbb08fad4f82f",
    ".github/workflows/ci.yml": "1ff8f97eb5e6be559b8d592d6127b03b36ff69867bbd989fb8f3902d905faf73",
    ".github/workflows/dependency-review.yml": "5a6c0bf9f9bcc41b2e92fb01ac1972ea068406b1c49465290637a06574673e0a",
    ".github/workflows/external-links.yml": "38315cad7f3e8909bf6b63fa78ef06e2755f18229339719bdd633ea98bb097a2",
    ".github/workflows/scorecard.yml": "be2ff8f6d336bfb2002c1367b36dbb701c0faf30db19769038e6293a4a204f67",
    ".github/workflows/workflow-online-audit.yml": "c4ff593389d834d380dff4118afc7aca19dcd685faa4210cde30384c93845da0",
    ".gitignore": "0dc93ed8728b8eb9726b7461ef8fd42db8f366b07d72039ed421ed9357e4152d",
    ".markdownlint-cli2.jsonc": "abcacc70e3d54a4cbfc4a4d3cbfd92564f5fbbf3f408d0f61aae37af4ab781a5",
    "CODE_OF_CONDUCT.md": "24d9a184b30787622cdc31145924a9c38558e3a2b72ed3f47a1ae94e1010074a",
    "CONTRIBUTING.md": "ee6a23e1c2bca6f86f6a40e2511c4de4c253a77ac2b24d3ae3d975416055b86f",
    "DEPENDENCY_POLICY.md": "ae5e10534b9081c401d943a55fc85fb2aa4a284cc366129f6139eefdb8389438",
    "GOVERNANCE.md": "8cbf5da50c63908948d181b1525c86e0f8a554eaa71fc98cf2f0ec47f6776103",
    "Makefile": "d53d7d969b0e4371417d20be388090dfda950cb50e2b18bb303f5945608ce5c6",
    "README.md": "a4329d0464aaa06ddaba7bbf4aaa2046322cd32fecf10b0e22834cdd0649dfee",
    "RELEASE_POLICY.md": "f8a3f0fa3494eb28bdd9fc3e6d18ddc8df2fdf63a4c628a5f6c9d72762586e45",
    "SECURITY.md": "1a801158996153650a2d94a4dbf5043d0a08ce9b96e4aefa9abdcd66344a0ede",
    "SUPPORT.md": "2dd3aa1da7b190822118a83c86bd5de7baa3ae3c041acf9baba4308f029254db",
    "assets/brand/README.md": "40c7dcc00ad935e8e05ac3b937fedf17c8cc5ff9a25accaa3ac2227e9f653ff7",
    "assets/brand/manifest.json": "35c65a3e6850badca2b6fc421dcdc5e3f4e1ecb5a5c0fae8620348e915030769",
    "conformance/foundation/README.md": "18dfeb0a2156e571df6e592b8b38a908661bb4f61da3a84ac4de8a3039b19294",
    "conformance/foundation/invalid/claim-record-assumption-only.json": "2e8fa46cda4b814f8d2096d19c4e7fec83ae9f28cd355c5012948ce5980ca210",
    "conformance/foundation/invalid/evidence-manifest-independent-without-review.json": "b92882efaf1f36a5988a8c4c484e4d7e659219248a6ee287f5928bf2b853f16b",
    "conformance/foundation/invalid/evidence-manifest-network-enabled.json": "955f58b255f4776d1cf1cac730c1fa7f1ab32a9fc5c35919bd74b3d007fe7b85",
    "conformance/foundation/invalid/evidence-manifest-path-escape.json": "b376f5435b54c5578ddcdd56acc4f61883625638130d34ee1ab33530c19f6ae0",
    "conformance/foundation/invalid/repository-control-missing-explanation.json": "a8bc273991680f616ddf78756cf3a8ad4568e733fefa12dbd3637ece40c8b8c8",
    "conformance/foundation/invalid/repository-control-selected-actions-empty.json": "f84cfeefe8cfa73f466a973a54c87b6e207cca9f970bcec6c19b8ed2d10674e4",
    "conformance/foundation/invalid/standards-provenance-bad-digest.json": "1142e67079f9778ccabe497dfae8ca80a72f870f5f3712b0569bc904d449b0cb",
    "conformance/foundation/invalid/standards-provenance-reviewed-without-reference.json": "4d7c311caff8a0d3c68b102f0690e61cc4da8112dff9762f3e03d22be41c2514",
    "conformance/foundation/invalid/trust-inventory-missing-identity.json": "ea616685e11fa714b7e99b45dddc4773310d6f4644b9a42fea700fe4ae0cb5e5",
    "conformance/foundation/manifest.json": "07f1ef6e49d2c094793b46a9db5fc56f834f6ad6410caded43fd13ce1957595f",
    "conformance/foundation/valid/claim-record.json": "985c2d0fe14a2961618182e3dd341d1715a3eb0a130ba03c36bfb27fcbb35249",
    "conformance/foundation/valid/evidence-manifest.json": "2e3cde3d86f770894356d90bdaed088ae65162f6590dac46ccdc750d2c34c0a4",
    "conformance/foundation/valid/repository-control-snapshot.json": "c79ed2b11d550573fc39463c27ec8207b3b7811011fe6abb13573651d4c232f3",
    "conformance/foundation/valid/standards-provenance.json": "1cd82e177baef03e1d3f413c86705b18891239cea413f7881331ee4066daf413",
    "conformance/foundation/valid/trust-inventory.json": "edb467fb6843713fea4571bacedf27e6b1039f1871ed835bcc0766dfb728542f",
    "docs/DECISIONS.md": "9d818486fa2961cf0d271d3878d3eff2b35b88dfb6a1d04e51a573e7dabc5ff5",
    "docs/LANGUAGE_2026.md": "1f779d3927f3d07540ab4d03815d9657c8a95eff9d9849921bfc02acd0e28acf",
    "docs/operations/CI_DEPENDENCIES.md": "21a7ec854592247ec0b3b238136046ca5bf3e4ab78797d53c16cc11f97667309",
    "docs/operations/GITHUB_CONTROLS.md": "dc43c34a3b35021223ddfc08fee557bcb4901b1742b84b4e31b7f8a254daec4f",
    "docs/security/OSPS_BASELINE.md": "2ee2a0040ce222be796f8524d9f5a44ca46745c6b5c78a5886a58f7c33f67295",
    "docs/security/SECRETS_AND_INCIDENTS.md": "93332edb737f84c7a3f74f256b5fb603537bf6f524388f62013140cb9906f6a6",
    "docs/security/THREAT_MODEL.md": "4b71a02989e8dcabdddbc2da747030a18de680be8a2181ace5da0eca4f29d9da",
    "policy/README.md": "f437a7671de3596b9035d626c3e59d70f7fd6d039b098fab162134aea5493704",
    "schemas/README.md": "39a7b91e15a316c1221cfce5082608eb453f20ea58b5e1c5a0cf32a07a81d774",
    "schemas/gate0/claim-record-v0.1.schema.json": "a287dde9ddf114da30af61d050aa96406f23e480d62e0f796d66943489579131",
    "schemas/gate0/evidence-manifest-v0.1.schema.json": "987ba1cddb23aaaf67a1234456fbffde8f80d45678b9671b8df97ad256742efd",
    "schemas/gate0/repository-control-snapshot-v0.1.schema.json": "f4cfcab41639fac0a5c3f75a99cfd3bef0a30b57fc950109058f5006f40ed8b4",
    "schemas/gate0/standards-provenance-v0.1.schema.json": "9d663bce83d7068e1e0b762eb50338a473ff8416062598dcd756d8ebf98f78f2",
    "schemas/gate0/trust-inventory-v0.1.schema.json": "fa673ccd1fbdc85faa92ee02835282e454c076db01b373c781e05ec1bbd1c222",
    "scripts/ci/check-external-links": "da0b282b8e9710625bf323b485b65bb2d15090557c384cace13e90c1ab94dc5c",
    "scripts/ci/check-repository": "692b0a7b0571891e5dfec985bdfbec3f2e340f9545afccaa76a04b7433621c16",
    "scripts/ci/install-actionlint": "b27105dc84be9f15fad5a1de3decbe7b75adc3065d9779d20ee6ba730c6fba4a",
    "scripts/ci/install-lychee": "42c0cca2b7a448d3ce131315b2c515e0492c3ddb343149fe5ddeffaef29198ed",
    "tools/tests/test_validate_foundation.py": "67b6a5d5d2ad670002c0c2175c5c424f5a63737a3ed7042662bf87f074a40a56",
    "tools/tests/test_validate_foundation_hardening.py": "3e972197baeeb331c8949413dcb317f3f6c4c909eafda952a2f01fb3db034ccf",
}
GATE0_CHARTER_SECTION_SHA256 = "4537523a0e41cc55912ad1013e6a74777ffad8def7015c4ffd51cfc3aeae3c9f"
GATE0_FEATURE_IDS = tuple(f"F-{index:02d}" for index in range(1, 15))
GATE0_PERSONA_IDS = tuple(f"P-{index:02d}" for index in range(1, 6))
GATE0_JOURNEY_IDS = tuple(f"J-{index:02d}" for index in range(1, 9))
GATE0_OPERATION_IDS = (
    "install",
    "specify",
    "implement",
    "prove",
    "build",
    "inspect",
    "integrate",
    "update",
    "revoke",
    "offline-replay",
)
GATE0_CONFORMANCE_CASE_IDS = (
    "claim-record-valid",
    "claim-record-assumption-only-satisfied",
    "evidence-manifest-valid",
    "evidence-manifest-network-enabled",
    "evidence-manifest-path-escape",
    "evidence-manifest-independent-without-review",
    "repository-control-snapshot-valid",
    "repository-control-disabled-without-explanation",
    "repository-control-selected-actions-empty",
    "standards-provenance-valid",
    "standards-provenance-malformed-digest",
    "standards-provenance-reviewed-without-reference",
    "trust-inventory-valid",
    "trust-inventory-authority-without-identity",
)
GATE0_CONFORMANCE_INSTANCE_PATHS = {
    "conformance/foundation/invalid/claim-record-assumption-only.json",
    "conformance/foundation/invalid/evidence-manifest-independent-without-review.json",
    "conformance/foundation/invalid/evidence-manifest-network-enabled.json",
    "conformance/foundation/invalid/evidence-manifest-path-escape.json",
    "conformance/foundation/invalid/repository-control-missing-explanation.json",
    "conformance/foundation/invalid/repository-control-selected-actions-empty.json",
    "conformance/foundation/invalid/standards-provenance-bad-digest.json",
    "conformance/foundation/invalid/standards-provenance-reviewed-without-reference.json",
    "conformance/foundation/invalid/trust-inventory-missing-identity.json",
    "conformance/foundation/valid/claim-record.json",
    "conformance/foundation/valid/evidence-manifest.json",
    "conformance/foundation/valid/repository-control-snapshot.json",
    "conformance/foundation/valid/standards-provenance.json",
    "conformance/foundation/valid/trust-inventory.json",
}
GATE0_RUST_TOOLCHAIN = {
    "toolchain": {
        "channel": "1.96.1",
        "components": ["clippy", "rustfmt"],
        "profile": "minimal",
    },
}
GATE0_RUST_MANIFESTS = {
    "compiler/Cargo.toml": {
        "workspace": {
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
                "rust": {"unsafe_code": "forbid"},
                "clippy": {"all": "deny"},
            },
        },
    },
    "compiler/crates/orange-compiler/Cargo.toml": {
        "package": {
            "name": "orange-compiler",
            "description": "Permanent compiler foundations for the Orange language",
            "version": {"workspace": True},
            "edition": {"workspace": True},
            "rust-version": {"workspace": True},
            "publish": {"workspace": True},
        },
        "lints": {"workspace": True},
    },
    "compiler/crates/orangec/Cargo.toml": {
        "package": {
            "name": "orangec",
            "description": "Command-line frontend for the Orange compiler",
            "version": {"workspace": True},
            "edition": {"workspace": True},
            "rust-version": {"workspace": True},
            "publish": {"workspace": True},
        },
        "dependencies": {
            "orange-compiler": {"path": "../orange-compiler"},
        },
        "lints": {"workspace": True},
    },
}
GATE0_RUST_MANIFEST_PACKAGES = {
    "compiler/Cargo.toml": None,
    "compiler/crates/orange-compiler/Cargo.toml": "orange-compiler",
    "compiler/crates/orangec/Cargo.toml": "orangec",
}
GATE0_RUST_WORKSPACE_MEMBERS = [
    "crates/orange-compiler",
    "crates/orangec",
]
GATE0_RUST_DEPENDENCY_TABLES = {
    "compiler/Cargo.toml": {},
    "compiler/crates/orange-compiler/Cargo.toml": {},
    "compiler/crates/orangec/Cargo.toml": {
        "dependencies": {
            "orange-compiler": {"path": "../orange-compiler"},
        },
    },
}
GATE0_RUST_LOCK = {
    "version": 4,
    "package": [
        {"name": "orange-compiler", "version": "0.0.1"},
        {
            "name": "orangec",
            "version": "0.0.1",
            "dependencies": ["orange-compiler"],
        },
    ],
}
ORANGE_2026_RUST_BUDGETS = {
    "compiler/crates/orange-compiler/src/source.rs": {
        "MAX_SOURCE_BYTES": 16 * 1024 * 1024,
    },
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
}
ORANGE_2026_SPEC_BUDGET_MARKERS = {
    "at most 16 MiB\n(`16 * 1024 * 1024` bytes)": 16 * 1024 * 1024,
    "At most 262,144 non-trivia tokens": 262_144,
    "At most 100 ordinary lexical diagnostics": 100,
    "262,144 syntax nodes": 262_144,
    "1,048,576 parser events or equivalent syntax elements": 1_048_576,
    "100 ordinary parse diagnostics plus at most one suppression diagnostic": 100,
    "recovery delimiter nesting depth 64": 64,
}
MINIMUM_CODEOWNERS = {
    "* @chasebryan",
    "/.github/ @chasebryan",
    "/assets/brand/ @chasebryan",
    "/SECURITY.md @chasebryan",
    "/GOVERNANCE.md @chasebryan",
    "/docs/ASSURANCE.md @chasebryan",
    "/docs/security/ @chasebryan",
    "/policy/ @chasebryan",
    "/scripts/ci/ @chasebryan",
    "/tools/validate_foundation.py @chasebryan",
}
GATE0_ALLOWED_TOP_LEVEL = {
    ".editorconfig",
    ".gitattributes",
    ".github",
    ".gitignore",
    ".markdownlint-cli2.jsonc",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "compiler",
    "DEPENDENCY_POLICY.md",
    "GOVERNANCE.md",
    "Makefile",
    "README.md",
    "RELEASE_POLICY.md",
    "rust-toolchain.toml",
    "SECURITY.md",
    "SUPPORT.md",
    "assets",
    "conformance",
    "docs",
    "policy",
    "schemas",
    "scripts",
    "tools",
}
ACTION_RE = re.compile(
    r"^\s*(?:-\s*)?uses:\s*([^\s@#]+)@([^\s#]+)"
    r"(?:\s+#\s*([^\s]+)(?:\s+.*)?)?\s*$"
)
CONTAINER_ACTION_RE = re.compile(
    r"^\s*(?:-\s*)?uses:\s*(docker://[^\s#]+)"
    r"(?:\s+#\s*([^\s]+)(?:\s+.*)?)?\s*$"
)
MARKDOWN_LINK_RE = re.compile(r"!?\[[^\]\n]*\]\(([^)\n]+)\)")
HEADING_RE = re.compile(r"^(#{1,6})\s+(.+?)\s*#*\s*$")
FRONT_MATTER_KEY_RE = re.compile(r"^([a-z][a-z0-9-]*):(?:\s*(.*))?$")
RECORD_FILENAME_RE = re.compile(r"^(?P<prefix>OEP|ADR)-(?P<number>[0-9]{4})-(?P<slug>[a-z0-9]+(?:-[a-z0-9]+)*)\.md$")


class DuplicateKeyError(ValueError):
    """Raised when JSON contains an ambiguous duplicate object key."""


@dataclasses.dataclass(frozen=True, order=True)
class Finding:
    code: str
    path: str
    message: str

    def as_dict(self) -> dict[str, str]:
        return dataclasses.asdict(self)


@dataclasses.dataclass(frozen=True)
class SchemaIssue:
    keyword: str
    instance_path: str
    message: str


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
    parsed = int(value)
    if not -(2**53) + 1 <= parsed <= 2**53 - 1:
        raise json.JSONDecodeError(f"integer {value!r} exceeds the I-JSON interoperable range", value, 0)
    return parsed


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


def load_json(path: Path) -> Any:
    source = path.read_text(encoding="utf-8")
    result = json.loads(
        source,
        object_pairs_hook=_object_without_duplicates,
        parse_constant=_reject_non_json_constant,
        parse_float=_reject_floating_point,
        parse_int=_parse_i_json_integer,
    )
    _require_unicode_scalars(result, source)
    return result


def canonical_json_bytes(value: Any) -> bytes:
    """Serialize the integer-only Gate 0 JSON profile using RFC 8785 rules.

    Gate 0 rejects floating-point values, unsafe integers, duplicate names, and
    lone surrogates before this function is called. That narrower domain avoids
    the ECMAScript floating-point formatting edge cases in full RFC 8785.
    """

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
    """Format a lexical repository-relative path without following symlinks."""

    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return path.as_posix()


def iter_repository_files(root: Path) -> Iterable[Path]:
    try:
        result = subprocess.run(
            ["git", "-C", str(root), "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        values = [value for value in result.stdout.split(b"\0") if value]
        for value in sorted(values):
            path = root / os.fsdecode(value)
            if path.is_file() or path.is_symlink():
                yield path
        return
    except (OSError, subprocess.CalledProcessError):
        pass
    for path in sorted(root.rglob("*")):
        if any(part in IGNORED_PARTS for part in path.relative_to(root).parts):
            continue
        if path.is_file() or path.is_symlink():
            yield path


def git_index_entries(root: Path) -> list[tuple[str, str]]:
    try:
        result = subprocess.run(
            ["git", "-C", str(root), "ls-files", "--stage", "-z"],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except (OSError, subprocess.CalledProcessError):
        return []
    entries: list[tuple[str, str]] = []
    for record in (value for value in result.stdout.split(b"\0") if value):
        metadata, separator, raw_path = record.partition(b"\t")
        fields = metadata.split()
        if not separator or len(fields) != 3:
            continue
        entries.append((os.fsdecode(fields[0]), os.fsdecode(raw_path)))
    return entries


class FoundationValidator:
    def __init__(self, root: Path) -> None:
        self.root = root.resolve()
        self.policy_path = self.root / POLICY_PATH
        self.findings: list[Finding] = []
        self.policy: dict[str, Any] = {}
        self.repository_files = list(iter_repository_files(self.root))
        self.index_entries = git_index_entries(self.root)

    def add(self, code: str, path: str | Path, message: str) -> None:
        path_text = relative(path, self.root) if isinstance(path, Path) else path
        self.findings.append(Finding(code, path_text, message))

    def run(self) -> list[Finding]:
        self._load_and_validate_policy()
        if not self.policy:
            return sorted(set(self.findings))
        self._validate_required_and_forbidden_paths()
        self._validate_compiler_dependency_boundary()
        self._validate_compiler_language_boundary()
        self._validate_tree_encoding_and_format()
        self._validate_brand_assets()
        self._validate_protected_file_digests()
        self._validate_hosted_control_evidence()
        self._validate_markdown_links()
        self._validate_json_documents()
        self._validate_schema_fixtures()
        self._validate_workflows()
        self._validate_dependabot()
        self._validate_codeowners()
        self._validate_decision_gates()
        self._validate_traceability()
        self._validate_user_journeys()
        self._validate_proof_foundation_suite()
        self._validate_change_records()
        self._validate_repository_templates()
        return sorted(set(self.findings))

    def _load_and_validate_policy(self) -> None:
        if not self.policy_path.is_file():
            self.add("policy.missing", self.policy_path, "solo-bootstrap repository policy is missing")
            return
        try:
            policy = load_json(self.policy_path)
        except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError) as exc:
            self.add("policy.invalid_json", self.policy_path, str(exc))
            return
        required = {
            "policy_version": str,
            "repository": str,
            "stage": str,
            "status": str,
            "default_branch": str,
            "bootstrap_steward": str,
            "allowed_top_level_paths": list,
            "allowed_binary_artifacts": list,
            "required_paths": list,
            "forbidden_paths": list,
            "required_workflows": list,
            "workflow_inventory": list,
            "protected_file_digests": dict,
            "executable_paths": list,
            "github_actions": dict,
            "hosted_repository_controls": dict,
            "required_codeowners": list,
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
        if any(f.code.startswith("policy.") and f.code != "policy.missing" for f in self.findings):
            return
        if policy["repository"] != "chasebryan/orange":
            self.add("policy.scope", self.policy_path, "repository must remain chasebryan/orange")
        if policy["stage"] != "solo-bootstrap" or policy["status"] != "enforced":
            self.add("policy.stage", self.policy_path, "this validator only accepts enforced solo-bootstrap policy")
        if policy["default_branch"] != "main":
            self.add("policy.default_branch", self.policy_path, "solo-bootstrap default branch must remain main")
        if policy["bootstrap_steward"] != "chasebryan":
            self.add("policy.steward", self.policy_path, "bootstrap steward must remain chasebryan")
        if not re.fullmatch(r"0\.[0-9]+\.[0-9]+", policy["policy_version"]):
            self.add("policy.version", self.policy_path, "solo-bootstrap policy version must be a 0.x semantic version")
        for key in (
            "required_paths",
            "forbidden_paths",
            "required_workflows",
            "workflow_inventory",
            "executable_paths",
            "required_codeowners",
        ):
            values = policy[key]
            if not all(isinstance(value, str) and value for value in values):
                self.add("policy.value", self.policy_path, f"{key} must contain non-empty strings")
            if len(values) != len(set(values)):
                self.add("policy.duplicate", self.policy_path, f"{key} contains duplicate values")
        minimum_sets = {
            "required_paths": MINIMUM_REQUIRED_PATHS,
            "forbidden_paths": MINIMUM_FORBIDDEN_PATHS,
            "required_workflows": MINIMUM_REQUIRED_WORKFLOWS,
            "required_codeowners": MINIMUM_CODEOWNERS,
        }
        for key, minimum in minimum_sets.items():
            missing = sorted(minimum - set(policy[key]))
            if missing:
                self.add("policy.minimum", self.policy_path, f"{key} omits protected values: {', '.join(missing)}")
        if set(policy["required_paths"]) != MINIMUM_REQUIRED_PATHS:
            self.add("policy.required_inventory", self.policy_path, "solo-bootstrap required-path inventory must remain exact")
        top_level = set(policy["allowed_top_level_paths"])
        if top_level != GATE0_ALLOWED_TOP_LEVEL:
            missing = sorted(GATE0_ALLOWED_TOP_LEVEL - top_level)
            extra = sorted(top_level - GATE0_ALLOWED_TOP_LEVEL)
            self.add(
                "policy.top_level",
                self.policy_path,
                f"solo-bootstrap top-level allowlist drifted; missing={missing}, extra={extra}",
            )
        for index, artifact in enumerate(policy["allowed_binary_artifacts"]):
            if not isinstance(artifact, dict):
                self.add("policy.binary", self.policy_path, f"allowed_binary_artifacts[{index}] must be an object")
                continue
            if set(artifact) != {"path", "sha256", "role", "provenance"}:
                self.add("policy.binary", self.policy_path, f"allowed_binary_artifacts[{index}] has invalid fields")
                continue
            if not isinstance(artifact["path"], str) or safe_manifest_path(self.root, artifact["path"]) is None:
                self.add("policy.binary", self.policy_path, f"allowed_binary_artifacts[{index}] has unsafe path")
            if not isinstance(artifact["sha256"], str) or not re.fullmatch(r"[0-9a-f]{64}", artifact["sha256"]):
                self.add("policy.binary", self.policy_path, f"allowed_binary_artifacts[{index}] has invalid SHA-256")
            for field in ("role", "provenance"):
                if not isinstance(artifact[field], str) or not artifact[field].strip():
                    self.add("policy.binary", self.policy_path, f"allowed_binary_artifacts[{index}] needs {field}")
        if policy["allowed_binary_artifacts"] != GATE0_ALLOWED_BINARY_ARTIFACTS:
            self.add(
                "policy.binary_inventory",
                self.policy_path,
                "official binary artifact paths, digests, roles, and provenance must remain exact",
            )
        expected_action_policy_keys = {
            "allowed_action_repositories",
            "allowed_container_images",
            "allowed_write_permissions",
            "forbidden_events",
            "require_full_commit_sha",
            "require_version_comment",
        }
        observed_action_policy_keys = set(policy["github_actions"])
        if observed_action_policy_keys != expected_action_policy_keys:
            self.add(
                "policy.action_fields",
                self.policy_path,
                "github_actions fields must remain exact; "
                f"missing={sorted(expected_action_policy_keys - observed_action_policy_keys)}, "
                f"extra={sorted(observed_action_policy_keys - expected_action_policy_keys)}",
            )
        action_repositories = set(policy["github_actions"].get("allowed_action_repositories", []))
        if action_repositories != MINIMUM_ACTION_REPOSITORIES:
            self.add(
                "policy.action_allowlist",
                self.policy_path,
                f"Action identities must be exact; missing={sorted(MINIMUM_ACTION_REPOSITORIES - action_repositories)}, extra={sorted(action_repositories - MINIMUM_ACTION_REPOSITORIES)}",
            )
        container_images = set(policy["github_actions"].get("allowed_container_images", []))
        if container_images != GATE0_ALLOWED_CONTAINER_IMAGES:
            self.add(
                "policy.container_allowlist",
                self.policy_path,
                f"container image identities must be exact; missing={sorted(GATE0_ALLOWED_CONTAINER_IMAGES - container_images)}, extra={sorted(container_images - GATE0_ALLOWED_CONTAINER_IMAGES)}",
            )
        if set(policy["executable_paths"]) != GATE0_EXECUTABLE_PATHS:
            self.add("policy.executables", self.policy_path, "solo-bootstrap executable allowlist must remain exact")
        if set(policy["workflow_inventory"]) != GATE0_WORKFLOW_INVENTORY:
            self.add("policy.workflow_inventory", self.policy_path, "solo-bootstrap workflow inventory must remain exact")
        if policy["protected_file_digests"] != GATE0_PROTECTED_FILE_DIGESTS:
            self.add(
                "policy.protected_file_digests",
                self.policy_path,
                "protected solo-bootstrap file digests must remain exact",
            )
        actual_writes = {
            name: set(values)
            for name, values in policy["github_actions"].get("allowed_write_permissions", {}).items()
            if isinstance(values, list)
        }
        if actual_writes != GATE0_ALLOWED_WRITE_PERMISSIONS:
            self.add("policy.write_permissions", self.policy_path, "workflow write-permission exceptions must remain exact")
        if policy["github_actions"].get("require_full_commit_sha") is not True:
            self.add("policy.action_sha", self.policy_path, "full Action commit SHA enforcement cannot be disabled")
        if policy["github_actions"].get("require_version_comment") is not True:
            self.add("policy.action_comment", self.policy_path, "Action version comments cannot be disabled")
        if "pull_request_target" not in policy["github_actions"].get("forbidden_events", []):
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
            "implementation_language": {"decision": "D-008", "required_status": "directed"},
            "project_name": {"decision": "D-017", "required_status": "directed"},
            "licenses": {"decision": "D-018", "required_status": "directed"},
            "governance": {"decision": "D-019", "required_status": "directed"},
            "solo_project": {"decision": "D-023", "required_status": "directed"},
            "compiler_foundation": {"decision": "D-024", "required_status": "directed"},
            "edition_2026_parser": {"decision": "D-025", "required_status": "directed"},
        }
        if policy["decision_gates"] != expected_decisions:
            self.add("policy.decision_gates", self.policy_path, "solo-bootstrap decision gates must remain exact")
        self.policy = policy

    def _validate_protected_file_digests(self) -> None:
        """Fail closed if a security-critical Gate 0 file differs from reviewed bytes."""

        for value, expected in sorted(GATE0_PROTECTED_FILE_DIGESTS.items()):
            path = self.root / value
            if not path.is_file():
                continue
            try:
                observed = hashlib.sha256(path.read_bytes()).hexdigest()
            except OSError as exc:
                self.add("protected_file.unreadable", path, str(exc))
                continue
            if observed != expected:
                self.add(
                    "protected_file.digest",
                    path,
                    f"reviewed SHA-256 changed: expected {expected}, observed {observed}",
                )

    def _validate_hosted_control_evidence(self, *, today: dt.date | None = None) -> None:
        """Keep the current hosted-control snapshot internally coherent."""

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
            if not path.is_file():
                self.add("hosted_control.missing", path, "hosted-control evidence document is missing")
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except (OSError, UnicodeError) as exc:
                self.add("hosted_control.unreadable", path, str(exc))
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
        pure = PurePosixPath(value)
        if pure.is_absolute() or ".." in pure.parts:
            self.add("policy.unsafe_path", self.policy_path, f"unsafe repository path {value!r}")
            return None
        resolved = (self.root / pure).resolve()
        try:
            resolved.relative_to(self.root)
        except ValueError:
            self.add("policy.unsafe_path", self.policy_path, f"path escapes repository: {value!r}")
            return None
        return resolved

    def _validate_required_and_forbidden_paths(self) -> None:
        actual_paths = {relative(path, self.root) for path in self.repository_files}
        actual_top_level = {PurePosixPath(value).parts[0] for value in actual_paths}
        for value in sorted(actual_top_level - set(self.policy["allowed_top_level_paths"])):
            self.add("path.top_level", value, "top-level path is not admitted during Gate 0")
        static_paths = MINIMUM_REQUIRED_PATHS | GATE0_CONFORMANCE_INSTANCE_PATHS
        for value in sorted(actual_paths - static_paths):
            if re.fullmatch(
                r"docs/governance/(?:oeps/OEP|adrs/ADR)-[0-9]{4}-[a-z0-9]+(?:-[a-z0-9]+)*\.md",
                value,
            ):
                continue
            self.add("path.inventory", value, "path is not admitted by the exact solo-bootstrap inventory")
        for value in self.policy["required_paths"]:
            path = self._policy_path(value)
            if path is not None and not path.is_file():
                self.add("path.required", value, "required permanent artifact is missing")
        for value in self.policy["forbidden_paths"]:
            path = self._policy_path(value)
            if path is not None and path.exists():
                self.add("path.forbidden", value, "path is forbidden until its dependent capability decision closes")

    def _validate_compiler_dependency_boundary(self) -> None:
        """Require the exact pinned, safe, first-party-only Rust foundation."""

        toolchain_path = self.root / "rust-toolchain.toml"
        try:
            with toolchain_path.open("rb") as source:
                toolchain = tomllib.load(source)
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
                with path.open("rb") as source:
                    manifest = tomllib.load(source)
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
                and "workspace" not in package
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
                        "compiler.dependency_table",
                        path,
                        f"Cargo dependency table {label!r} must be a table",
                    )
                    return
                if table:
                    observed_tables[label] = table

            for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
                if kind in manifest:
                    record_table(kind, manifest[kind])

            workspace = manifest.get("workspace")
            if workspace is not None:
                if not isinstance(workspace, dict):
                    self.add("compiler.workspace", path, "Cargo workspace declaration must be a table")
                elif value != "compiler/Cargo.toml":
                    self.add("compiler.workspace", path, "only the root manifest may declare a workspace")
                elif "dependencies" in workspace:
                    record_table("workspace.dependencies", workspace["dependencies"])

            targets = manifest.get("target")
            if targets is not None:
                if not isinstance(targets, dict):
                    self.add("compiler.dependency_table", path, "Cargo target declaration must be a table")
                else:
                    for target_name, target in sorted(targets.items()):
                        if not isinstance(target, dict):
                            self.add(
                                "compiler.dependency_table",
                                path,
                                f"Cargo target {target_name!r} must be a table",
                            )
                            continue
                        for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
                            if kind in target:
                                record_table(f"target.{target_name}.{kind}", target[kind])

            patches = manifest.get("patch")
            if patches is not None:
                if not isinstance(patches, dict):
                    self.add("compiler.dependency_table", path, "Cargo patch declaration must be a table")
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

        root_manifest = manifests.get("compiler/Cargo.toml")
        if root_manifest is not None:
            workspace = root_manifest.get("workspace")
            observed_members = workspace.get("members") if isinstance(workspace, dict) else None
            observed_excludes = workspace.get("exclude", []) if isinstance(workspace, dict) else None
            if observed_members != GATE0_RUST_WORKSPACE_MEMBERS or observed_excludes != []:
                self.add(
                    "compiler.workspace_members",
                    self.root / "compiler/Cargo.toml",
                    "workspace members must remain the exact admitted package directories with no exclusions",
                )

        lock_path = self.root / "compiler/Cargo.lock"
        try:
            with lock_path.open("rb") as source:
                lock = tomllib.load(source)
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
        """Bind normative Orange 2026 resource budgets to compiled constants."""

        for value, expected_constants in ORANGE_2026_RUST_BUDGETS.items():
            path = self.root / value
            try:
                source = rust_code_without_comments_and_literals(path.read_text(encoding="utf-8"))
            except (OSError, UnicodeError) as exc:
                self.add("compiler.language_budget", path, f"cannot read Rust budget source: {exc}")
                continue
            declarations: dict[str, list[str]] = {}
            for match in re.finditer(
                r"(?m)^\s*pub\s+const\s+([A-Z][A-Z0-9_]*)\s*:\s*usize\s*=\s*([^;]+);",
                source,
            ):
                declarations.setdefault(match.group(1), []).append(match.group(2))
            for name, expected in expected_constants.items():
                expressions = declarations.get(name, [])
                if len(expressions) != 1:
                    self.add(
                        "compiler.language_budget",
                        path,
                        f"{name} must have exactly one public usize declaration; observed={len(expressions)}",
                    )
                    continue
                observed = parse_rust_usize_product(expressions[0])
                if observed != expected:
                    self.add(
                        "compiler.language_budget",
                        path,
                        f"{name} must equal {expected}; observed={observed!r}",
                    )

        specification = self.root / "docs/LANGUAGE_2026.md"
        try:
            text = specification.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            self.add("compiler.language_spec_budget", specification, f"cannot read normative budget specification: {exc}")
            return
        for marker, expected in ORANGE_2026_SPEC_BUDGET_MARKERS.items():
            if marker not in text:
                self.add(
                    "compiler.language_spec_budget",
                    specification,
                    f"normative specification must state the exact {expected} budget marker {marker!r}",
                )

    def _validate_tree_encoding_and_format(self) -> None:
        files = self.repository_files
        casefolded: dict[str, str] = {}
        normalized: dict[str, str] = {}
        executable_paths = set(self.policy["executable_paths"])
        binary_artifacts = {
            artifact["path"]: artifact
            for artifact in self.policy["allowed_binary_artifacts"]
            if isinstance(artifact, dict) and isinstance(artifact.get("path"), str)
        }
        for mode, value in self.index_entries:
            if mode == "160000":
                self.add("git.submodule", value, "gitlinks/submodules are not admitted during Gate 0")
            elif mode not in {"100644", "100755"}:
                self.add("git.mode", value, f"unsupported Git index mode {mode}")
            path = self.root / value
            if path.is_file() and mode in {"100644", "100755"}:
                worktree_executable = bool(path.stat().st_mode & 0o111)
                if worktree_executable != (mode == "100755"):
                    self.add("git.mode_mismatch", value, "Git index and worktree executable modes differ")
        for path in files:
            value = relative(path, self.root)
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
            if path.is_symlink():
                self.add("file.symlink", path, "symlinks are not permitted in the solo-bootstrap repository tree")
                continue
            is_executable = bool(path.stat().st_mode & 0o111)
            if is_executable and value not in executable_paths:
                self.add("file.unexpected_executable", path, "executable bit is not authorized by repository policy")
            if value in executable_paths and not is_executable:
                self.add("file.missing_executable", path, "repository policy requires the executable bit")
            data = path.read_bytes()
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
            manifest = load_json(manifest_path)
        except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError) as exc:
            self.add("brand.manifest", manifest_path, str(exc))
            return
        expected_header = {
            "schema_version": "orange-brand-assets/v1",
            "status": "official",
            "authority": "chasebryan",
            "designated_on": "2026-07-11",
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
            if name not in GATE0_BRAND_ASSET_METADATA:
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
            try:
                data = asset_path.read_bytes()
            except OSError as exc:
                self.add("brand.asset", asset_path, str(exc))
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
        for path in (path for path in self.repository_files if path.suffix.lower() == ".md"):
            try:
                text = path.read_text(encoding="utf-8")
            except UnicodeError:
                continue
            for match in MARKDOWN_LINK_RE.finditer(text):
                raw_target = match.group(1).strip()
                target = self._markdown_destination(raw_target)
                if not target:
                    continue
                parsed = urlsplit(target)
                if parsed.scheme or target.startswith("//"):
                    continue
                file_part = unquote(parsed.path)
                fragment = unquote(parsed.fragment)
                target_path = path if not file_part else (path.parent / file_part)
                resolved = target_path.resolve()
                try:
                    resolved.relative_to(self.root)
                except ValueError:
                    self.add("markdown.link_escape", path, f"link escapes repository: {raw_target}")
                    continue
                if not resolved.exists():
                    self.add("markdown.link_missing", path, f"local link target does not exist: {raw_target}")
                    continue
                if fragment and resolved.is_file() and resolved.suffix.lower() == ".md":
                    anchors = markdown_anchors(resolved.read_text(encoding="utf-8"))
                    if fragment not in anchors:
                        self.add("markdown.anchor_missing", path, f"anchor not found: {raw_target}")

    @staticmethod
    def _markdown_destination(raw_target: str) -> str:
        if raw_target.startswith("<") and ">" in raw_target:
            return raw_target[1 : raw_target.index(">")]
        if raw_target.startswith("#"):
            return raw_target
        # Markdown titles follow the destination after whitespace. Local spaces
        # must be percent-encoded or enclosed in angle brackets.
        return raw_target.split(maxsplit=1)[0]

    def _validate_json_documents(self) -> None:
        for path in (path for path in self.repository_files if path.suffix.lower() == ".json"):
            try:
                load_json(path)
            except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError) as exc:
                self.add("json.invalid", path, str(exc))

    def _validate_schema_fixtures(self) -> None:
        schema_dir = self.root / "schemas/gate0"
        schemas: dict[Path, Mapping[str, Any]] = {}
        id_registry: dict[str, tuple[Path, Mapping[str, Any]]] = {}
        if not schema_dir.is_dir():
            return
        for path in sorted(schema_dir.glob("*.schema.json")):
            try:
                schema = load_json(path)
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
            for issue in audit_schema_vocabulary(schema):
                self.add("schema.unsupported_keyword", path, issue)
            schemas[path.resolve()] = schema
        observed_schema_paths = {relative(path, self.root) for path in schemas}
        if observed_schema_paths != GATE0_SCHEMA_PATHS:
            self.add(
                "schema.inventory",
                schema_dir,
                f"Gate 0 schema inventory must be exact; missing={sorted(GATE0_SCHEMA_PATHS - observed_schema_paths)}, extra={sorted(observed_schema_paths - GATE0_SCHEMA_PATHS)}",
            )
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
        if not manifest_path.is_file():
            return
        try:
            manifest = load_json(manifest_path)
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
        if observed_case_ids != GATE0_CONFORMANCE_CASE_IDS:
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
            if not fixture_path.is_file():
                self.add("fixture.missing", manifest_path, f"fixture does not exist: {fixture_value}")
                continue
            if schema_path not in schemas:
                self.add("fixture.schema_missing", manifest_path, f"schema is not registered: {schema_value}")
                continue
            try:
                instance = load_json(fixture_path)
            except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError):
                continue
            issues = validate_schema_instance(
                instance,
                schemas[schema_path],
                schema_path,
                schemas,
                id_registry,
            )
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
            for directory in (self.root / "conformance/foundation/valid", self.root / "conformance/foundation/invalid")
            if directory.is_dir()
            for path in directory.rglob("*.json")
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
        required = set(self.policy["required_workflows"])
        actual = {path.name for path in workflow_dir.glob("*.y*ml")} if workflow_dir.is_dir() else set()
        if actual != GATE0_WORKFLOW_INVENTORY:
            self.add(
                "workflow.inventory",
                workflow_dir,
                f"workflow inventory must be exact; missing={sorted(GATE0_WORKFLOW_INVENTORY - actual)}, extra={sorted(actual - GATE0_WORKFLOW_INVENTORY)}",
            )
        for name in sorted(required - actual):
            self.add("workflow.required", f".github/workflows/{name}", "required workflow is missing")
        actions_policy = self.policy["github_actions"]
        allowed = set(actions_policy.get("allowed_action_repositories", []))
        forbidden_events = set(actions_policy.get("forbidden_events", []))
        allowed_writes = {
            name: set(values)
            for name, values in actions_policy.get("allowed_write_permissions", {}).items()
        }
        for path in sorted(workflow_dir.glob("*.y*ml")) if workflow_dir.is_dir() else []:
            text = path.read_text(encoding="utf-8")
            lines = text.splitlines()
            active_text = yaml_without_comments(text)
            active_lines = active_text.splitlines()
            if re.search(r"\\u[0-9A-Fa-f]{4}", active_text):
                self.add("workflow.escape", path, "Unicode escapes are forbidden in workflow source")
            if re.search(r"(?m)^\s*[\"'][^\"']+[\"']\s*:", active_text):
                self.add("workflow.quoted_key", path, "quoted workflow keys are forbidden by the canonical source dialect")
            if re.search(r"(?m)^\s*[A-Za-z_][A-Za-z0-9_-]*\s+:", active_text):
                self.add("workflow.key_spacing", path, "whitespace before a YAML mapping colon is forbidden")
            if re.search(r"(?m)^\s*on:\s*[\[{]", active_text) or re.search(r"(?m)^\s*jobs:\s*[\[{]", active_text):
                self.add("workflow.flow_style", path, "on and jobs must use block-style YAML")
            if re.search(r"(?m)^\s{2}(?:pull_request|push|merge_group|schedule|workflow_dispatch):\s*[\[{]", active_text):
                self.add("workflow.event_flow_style", path, "workflow events must use block-style YAML")
            if re.search(r"(?m)^\s+(?:container|services)\s*:", active_text):
                self.add("workflow.container", path, "job containers and services are not admitted in Gate 0")
            if not re.search(r"(?m)^permissions:\s*(?:\{\})?\s*$", text):
                self.add("workflow.permissions", path, "workflow must declare top-level permissions")
            if re.search(r"(?m)^\s*permissions:\s*write-all\s*$", text):
                self.add("workflow.write_all", path, "write-all permissions are forbidden")
            if not re.search(r"(?m)^concurrency:\s*$", text) or not re.search(
                r"(?m)^\s+cancel-in-progress:\s*true\s*$", text
            ):
                self.add("workflow.concurrency", path, "workflow must cancel superseded concurrent runs")
            for event in forbidden_events:
                if re.search(rf"(?m)^[^#\n]*\b{re.escape(event)}\b", text):
                    self.add("workflow.forbidden_event", path, f"forbidden event {event}")
            if path.name == "ci.yml":
                for event in ("pull_request", "push", "merge_group"):
                    if not re.search(rf"(?m)^\s{{2}}{event}\s*:", text):
                        self.add("workflow.ci_event", path, f"required CI event is missing: {event}")
                if re.search(r"\bpaths(?:-ignore)?\s*:", active_text):
                    self.add("workflow.path_filter", path, "required CI must not use path filters")
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
                    if actions_policy.get("require_full_commit_sha") and not re.fullmatch(r"[0-9a-f]{40}", ref):
                        self.add("workflow.mutable_action", path, f"line {line_number}: action ref must be a full commit SHA")
                    if actions_policy.get("require_version_comment") and not version:
                        self.add("workflow.version_comment", path, f"line {line_number}: pinned action needs a version comment")
                    elif version and not re.fullmatch(r"v[0-9]+(?:\.[0-9]+){1,2}(?:[-+][0-9A-Za-z.-]+)?", version):
                        self.add("workflow.version_comment", path, f"line {line_number}: invalid action version comment {version!r}")
                    if action == "actions/checkout" and not checkout_disables_credentials(lines, line_number - 1):
                        self.add("workflow.checkout_credentials", path, f"line {line_number}: checkout must set persist-credentials: false")
                write_match = re.match(r"^\s+([a-z][a-z0-9-]*)\s*:\s*[\"']?write[\"']?(?:\s+#.*)?\s*$", line)
                if write_match and write_match.group(1) not in allowed_writes.get(path.name, set()):
                    self.add(
                        "workflow.write_permission",
                        path,
                        f"line {line_number}: {write_match.group(1)}: write is not allowed in this workflow",
                    )
                runner_match = re.match(r"^\s+runs-on:\s*(.+?)\s*$", line)
                if runner_match:
                    runner = runner_match.group(1).strip().strip("\"'")
                    if "${{" in runner or "self-hosted" in runner or "latest" in runner:
                        self.add("workflow.runner", path, f"line {line_number}: runner must be a fixed GitHub-hosted image")
            for job_name, block in workflow_jobs(lines):
                block_text = "\n".join(block)
                if not re.search(r"(?m)^\s{4}timeout-minutes:\s*[1-9][0-9]*\s*$", block_text):
                    self.add("workflow.timeout", path, f"job {job_name} needs timeout-minutes")
                if not re.search(r"(?m)^\s{4}permissions:\s*(?:\{\})?\s*$", block_text):
                    self.add("workflow.job_permissions", path, f"job {job_name} needs explicit permissions")
            jobs = workflow_jobs(lines)
            if not jobs:
                self.add("workflow.jobs", path, "workflow must contain canonical two-space-indented jobs")
            for line_number in unsafe_run_interpolations(lines):
                self.add("workflow.untrusted_interpolation", path, f"untrusted event data is interpolated into run near line {line_number}")
            concurrency = top_level_block(active_lines, "concurrency")
            if not any(re.fullmatch(r"\s{2}cancel-in-progress:\s*true\s*", line) for line in concurrency):
                self.add("workflow.concurrency", path, "top-level concurrency must set cancel-in-progress: true")
            if any(re.search(r"cancel-in-progress:\s*false", line) for line in active_lines):
                self.add("workflow.concurrency_false", path, "cancel-in-progress: false is forbidden")
            self._validate_required_workflow_content(path, active_text)

    def _validate_dependabot(self) -> None:
        path = self.root / ".github/dependabot.yml"
        if not path.is_file():
            return
        text = yaml_without_comments(path.read_text(encoding="utf-8"))
        required_patterns = (
            r"package-ecosystem:\s*[\"']?github-actions[\"']?",
            r"directory:\s*[\"']?/[\"']?",
            r"interval:\s*[\"']?weekly[\"']?",
        )
        for pattern in required_patterns:
            if not re.search(pattern, text):
                self.add("dependabot.configuration", path, f"missing required setting matching {pattern}")
        review_path = self.root / ".github/dependency-review-config.yml"
        if review_path.is_file():
            review = yaml_without_comments(review_path.read_text(encoding="utf-8"))
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
        requirements: dict[str, tuple[str, ...]] = {
            "ci.yml": (
                "name: Required CI / docs-policy-workflows",
                "name: Enforce solo contribution boundary",
                "github.event.pull_request.user.login != 'chasebryan'",
                "Solo mode does not accept third-party pull requests until D-018 selects contribution terms.",
                "run: make check-compiler",
                "run: python3 -m unittest discover -s tools/tests -p 'test_*.py'",
                "run: python3 tools/validate_foundation.py",
                "DavidAnson/markdownlint-cli2-action@",
                "run: ./scripts/ci/install-actionlint",
                '"$RUNNER_TEMP/actionlint/actionlint" -color',
                "zizmorcore/zizmor-action@",
                "online-audits: false",
            ),
            "dependency-review.yml": (
                "name: Dependency Review / policy",
                "actions/dependency-review-action@",
                "base-ref: ${{ github.event_name == 'merge_group' && github.event.merge_group.base_sha || github.event.pull_request.base.sha }}",
                "config-file: ./.github/dependency-review-config.yml",
                "head-ref: ${{ github.event_name == 'merge_group' && github.event.merge_group.head_sha || github.event.pull_request.head.sha }}",
            ),
            "scorecard.yml": (
                "name: OpenSSF Scorecard / analysis",
                "if: ${{ github.ref == 'refs/heads/main' }}",
                "docker run --rm",
                "ghcr.io/ossf/scorecard-action@sha256:",
                "github/codeql-action/upload-sarif@",
            ),
        }
        for required in requirements.get(path.name, ()):
            if required not in text:
                self.add("workflow.required_content", path, f"missing protected workflow content: {required}")
        if path.name == "scorecard.yml" and re.search(r"(?m)^\s{2}workflow_dispatch\s*:", text):
            self.add("workflow.privileged_dispatch", path, "Scorecard must not allow manual ref selection")
        if "continue-on-error:" in text:
            self.add("workflow.continue_on_error", path, "continue-on-error is forbidden in Gate 0 workflows")
        if path.name == "ci.yml":
            for job_name, block in workflow_jobs(text.splitlines()):
                if job_name == "required" and any(re.match(r"^\s{4}if:\s*", line) for line in block):
                    self.add("workflow.required_job_condition", path, "required CI job must not have a job-level condition")
        expected_steps: dict[str, tuple[str, tuple[str, ...]]] = {
            "ci.yml": (
                "required",
                (
                    "Checkout",
                    "Enforce solo contribution boundary",
                    "Validate Rust compiler",
                    "Run foundation validator unit tests",
                    "Validate solo-bootstrap repository policy",
                    "Lint Markdown",
                    "Install actionlint",
                    "Validate GitHub Actions workflows",
                    "Audit GitHub Actions security",
                ),
            ),
            "dependency-review.yml": ("review", ("Checkout", "Review dependency changes")),
            "scorecard.yml": (
                "analysis",
                ("Checkout", "Run OpenSSF Scorecard", "Preserve SARIF result", "Upload result to code scanning"),
            ),
            "external-links.yml": ("links", ("Checkout", "Install checksum-verified lychee", "Check external links")),
            "workflow-online-audit.yml": ("metadata", ("Checkout", "Audit workflow source and upstream metadata")),
        }
        if path.name in expected_steps:
            job_name, names = expected_steps[path.name]
            jobs = dict(workflow_jobs(text.splitlines()))
            if set(jobs) != {job_name}:
                self.add("workflow.job_contract", path, f"workflow job set must be exactly {{{job_name}}}")
            if job_name not in jobs:
                self.add("workflow.step_contract", path, f"missing protected job {job_name}")
            else:
                steps = workflow_steps(jobs[job_name])
                observed_names = tuple(name for name, _ in steps)
                if observed_names != names:
                    self.add("workflow.step_contract", path, f"step sequence must be exact: {names}")
                self._validate_step_details(path, job_name, dict(steps))

    def _validate_step_details(self, path: Path, job_name: str, steps: Mapping[str, list[str]]) -> None:
        def require(step_name: str, values: Sequence[str]) -> None:
            block = yaml_without_comments("\n".join(steps.get(step_name, [])))
            for value in values:
                if value not in block:
                    self.add("workflow.step_contract", path, f"{job_name}/{step_name} is missing {value!r}")

        allowed_step_conditions: dict[str, set[str]] = {
            "ci.yml": {"Enforce solo contribution boundary"},
            "scorecard.yml": {"Preserve SARIF result", "Upload result to code scanning"},
        }
        for step_name, lines in steps.items():
            active_lines = yaml_without_comments("\n".join(lines)).splitlines()
            keys = [
                match.group(1)
                for line in active_lines
                if (match := re.match(r"^\s{8}([a-z][a-z0-9-]*):", line))
            ]
            for key in set(keys):
                if keys.count(key) > 1:
                    self.add("workflow.step_duplicate_key", path, f"{job_name}/{step_name} repeats {key}")
            execution_keys = keys.count("run") + keys.count("uses")
            if execution_keys != 1:
                self.add("workflow.step_execution", path, f"{job_name}/{step_name} needs exactly one run or uses key")
            if "if" in keys and step_name not in allowed_step_conditions.get(path.name, set()):
                self.add("workflow.step_condition", path, f"{job_name}/{step_name} must not be conditional")
            block = "\n".join(active_lines)
            for bypass in ("|| true", "set +e", "continue-on-error:"):
                if bypass in block:
                    self.add("workflow.fail_open", path, f"{job_name}/{step_name} contains fail-open construct {bypass!r}")

        if path.name == "ci.yml":
            require("Checkout", ("uses: actions/checkout@", "persist-credentials: false"))
            require(
                "Enforce solo contribution boundary",
                (
                    "if: ${{ github.event_name == 'pull_request' && github.event.pull_request.user.login != 'chasebryan' }}",
                    'echo "Solo mode does not accept third-party pull requests until D-018 selects contribution terms." >&2',
                    "exit 1",
                ),
            )
            require(
                "Validate Rust compiler",
                ("run: make check-compiler",),
            )
            require(
                "Run foundation validator unit tests",
                ("run: python3 -m unittest discover -s tools/tests -p 'test_*.py'",),
            )
            require(
                "Validate solo-bootstrap repository policy",
                ("run: python3 tools/validate_foundation.py",),
            )
            require("Lint Markdown", ("uses: DavidAnson/markdownlint-cli2-action@",))
            require("Install actionlint", ("run: ./scripts/ci/install-actionlint",))
            require("Validate GitHub Actions workflows", ('"$RUNNER_TEMP/actionlint/actionlint" -color',))
            require("Audit GitHub Actions security", ("uses: zizmorcore/zizmor-action@", "online-audits: false", "persona: pedantic"))
        elif path.name == "dependency-review.yml":
            require("Review dependency changes", ("uses: actions/dependency-review-action@", "base-ref:", "head-ref:", "config-file:"))
        elif path.name == "scorecard.yml":
            require(
                "Run OpenSSF Scorecard",
                (
                    "shell: bash",
                    "run: |",
                    "set -euo pipefail",
                    'test -n "${INPUT_REPO_TOKEN:-}"',
                    'test -r "$GITHUB_EVENT_PATH"',
                    'test -d "$GITHUB_WORKSPACE"',
                    "printf '::add-mask::%s\\n' \"$INPUT_REPO_TOKEN\"",
                    'rm -f -- "$GITHUB_WORKSPACE/results.sarif"',
                    "docker run --rm",
                    "--read-only",
                    "--tmpfs /tmp:rw,noexec,nosuid,nodev,size=1g,mode=1777",
                    "--cap-drop=ALL",
                    "--cap-add=DAC_OVERRIDE",
                    "--security-opt=no-new-privileges=true",
                    "--pids-limit=256",
                    '--mount "type=bind,source=${GITHUB_EVENT_PATH},target=/github/workflow/event.json,readonly"',
                    '--mount "type=bind,source=${GITHUB_WORKSPACE},target=/github/workspace"',
                    "--workdir /github/workspace",
                    "--env GITHUB_ACTIONS=true",
                    "--env GITHUB_API_URL",
                    "--env GITHUB_EVENT_NAME",
                    "--env GITHUB_EVENT_PATH=/github/workflow/event.json",
                    "--env GITHUB_REF",
                    "--env GITHUB_REPOSITORY",
                    "--env GITHUB_WORKSPACE=/github/workspace",
                    "--env INPUT_FILE_MODE=archive",
                    "--env INPUT_PUBLISH_RESULTS=false",
                    "--env INPUT_REPO_TOKEN",
                    "--env INPUT_RESULTS_FILE=results.sarif",
                    "--env INPUT_RESULTS_FORMAT=sarif",
                    "ghcr.io/ossf/scorecard-action@sha256:2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941",
                    "INPUT_REPO_TOKEN: ${{ github.token }}",
                    'test -s "$GITHUB_WORKSPACE/results.sarif"',
                ),
            )
            scorecard_block = yaml_without_comments("\n".join(steps.get("Run OpenSSF Scorecard", [])))
            expected_scorecard_block = '''      - name: Run OpenSSF Scorecard
        shell: bash
        env:
          INPUT_REPO_TOKEN: ${{ github.token }}
        run: |
          set -euo pipefail
          test -n "${INPUT_REPO_TOKEN:-}"
          test -r "$GITHUB_EVENT_PATH"
          test -d "$GITHUB_WORKSPACE"
          printf '::add-mask::%s\\n' "$INPUT_REPO_TOKEN"
          rm -f -- "$GITHUB_WORKSPACE/results.sarif"
          docker run --rm \\
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
            image = next(iter(GATE0_ALLOWED_CONTAINER_IMAGES))
            if scorecard_block.count(image) != 1:
                self.add(
                    "workflow.scorecard_image",
                    path,
                    f"{job_name}/Run OpenSSF Scorecard must invoke the one admitted image exactly once",
                )
            if "docker://" in scorecard_block:
                self.add(
                    "workflow.scorecard_runtime",
                    path,
                    f"{job_name}/Run OpenSSF Scorecard must use the hosted runner Docker CLI",
                )
            for forbidden in (
                'INPUT_PUBLISH_RESULTS: "true"',
                "INPUT_PUBLISH_RESULTS=true",
                "INPUT_INTERNAL_PUBLISH_BASE_URL",
                "INPUT_INTERNAL_DEFAULT_TOKEN",
            ):
                if forbidden in scorecard_block:
                    self.add(
                        "workflow.scorecard_publication",
                        path,
                        f"{job_name}/Run OpenSSF Scorecard contains forbidden public publication setting {forbidden!r}",
                    )
            for forbidden in (
                "--privileged",
                "--cap-add=ALL",
                "--cap-add=SYS_ADMIN",
                "--device",
                "--entrypoint",
                "--network=host",
                "/var/run/docker.sock",
            ):
                if forbidden in scorecard_block:
                    self.add(
                        "workflow.scorecard_runtime",
                        path,
                        f"{job_name}/Run OpenSSF Scorecard contains forbidden Docker option {forbidden!r}",
                    )
            require("Upload result to code scanning", ("uses: github/codeql-action/upload-sarif@",))

    def _validate_codeowners(self) -> None:
        path = self.root / ".github/CODEOWNERS"
        if not path.is_file():
            return
        active_lines = {
            line.strip()
            for line in path.read_text(encoding="utf-8").splitlines()
            if line.strip() and not line.lstrip().startswith("#")
        }
        for required in self.policy["required_codeowners"]:
            if required not in active_lines:
                self.add("codeowners.required", path, f"missing critical ownership rule: {required}")
        for line in active_lines:
            owners = line.split()[1:]
            if owners != ["@chasebryan"]:
                self.add("codeowners.bootstrap_owner", path, f"unratified or invalid bootstrap owner in: {line}")

    def _validate_decision_gates(self) -> None:
        path = self.root / "docs/DECISIONS.md"
        if not path.is_file():
            return
        text = markdown_without_fenced_blocks_and_comments(path.read_text(encoding="utf-8"))
        for gate, rule in self.policy["decision_gates"].items():
            decision = re.escape(rule.get("decision", ""))
            expected = rule.get("required_status")
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
        if not path.is_file() or not charter_path.is_file() or not decisions_path.is_file():
            return
        text = markdown_without_fenced_blocks_and_comments(path.read_text(encoding="utf-8"))
        charter = charter_path.read_bytes()
        start_marker = b"## 5. In scope for the 1.0 product\n"
        end_marker = b"## 6. Explicit non-goals for 1.0\n"
        start = charter.find(start_marker)
        end = charter.find(end_marker, start + len(start_marker)) if start >= 0 else -1
        if start < 0 or end < 0:
            self.add("traceability.charter_section", charter_path, "cannot isolate the section 5 feature source")
        else:
            observed = hashlib.sha256(charter[start:end]).hexdigest()
            if observed != GATE0_CHARTER_SECTION_SHA256:
                self.add(
                    "traceability.charter_digest",
                    charter_path,
                    f"section 5 changed: expected {GATE0_CHARTER_SECTION_SHA256}, observed {observed}",
                )
        recorded_digests = re.findall(r"\b[0-9a-f]{64}\b", text)
        if recorded_digests.count(GATE0_CHARTER_SECTION_SHA256) != 1:
            self.add(
                "traceability.recorded_digest",
                path,
                "traceability must record the exact reviewed charter-section SHA-256 once",
            )

        feature_section = markdown_section(text, "## 4. Feature matrix")
        feature_rows = table_rows(feature_section, r"F-[0-9]{2}")
        feature_ids = tuple(row[0] for row in feature_rows)
        if feature_ids != GATE0_FEATURE_IDS:
            self.add("traceability.feature_ids", path, f"feature rows must be exact and ordered: {GATE0_FEATURE_IDS}")
        known_decisions = set(
            re.findall(
                r"(?m)^##\s+(D-[0-9]{3})\b",
                markdown_without_fenced_blocks_and_comments(decisions_path.read_text(encoding="utf-8")),
            )
        )
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
        if tuple(row[0] for row in attestation_rows) != GATE0_FEATURE_IDS:
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
        if not path.is_file():
            return
        text = markdown_without_fenced_blocks_and_comments(path.read_text(encoding="utf-8"))
        persona_intro = markdown_section(text, "## 1. Purpose and limits")
        persona_rows = table_rows(persona_intro, r"P-[0-9]{2}")
        if tuple(row[0] for row in persona_rows) != GATE0_PERSONA_IDS:
            self.add("journey.persona_ids", path, "persona definitions must cover P-01 through P-05 exactly once in order")

        index_section = markdown_section(text, "## 2. Journey index")
        journey_rows = table_rows(index_section, r"J-[0-9]{2}")
        if tuple(row[0] for row in journey_rows) != GATE0_JOURNEY_IDS:
            self.add("journey.index_ids", path, "journey index must cover J-01 through J-08 exactly once in order")
        operation_owners: dict[str, set[str]] = {value: set() for value in GATE0_OPERATION_IDS}
        feature_owners: dict[str, set[str]] = {value: set() for value in GATE0_FEATURE_IDS}
        primary_owners: dict[str, set[str]] = {value: set() for value in GATE0_PERSONA_IDS}
        for row in journey_rows:
            if len(row) != 7:
                self.add("journey.index_shape", path, f"{row[0]} must contain exactly seven table fields")
                continue
            journey_id, title, primary, supporting, operations, features, target = row
            if not title or not target:
                self.add("journey.index_value", path, f"{journey_id} has an empty title or target gate")
            primary_ids = set(re.findall(r"\bP-[0-9]{2}\b", primary))
            supporting_ids = set(re.findall(r"\bP-[0-9]{2}\b", supporting))
            if not primary_ids or not primary_ids <= set(GATE0_PERSONA_IDS) or not supporting_ids <= set(GATE0_PERSONA_IDS):
                self.add("journey.persona_ref", path, f"{journey_id} has an invalid persona reference")
            for persona in primary_ids & set(GATE0_PERSONA_IDS):
                primary_owners[persona].add(journey_id)
            operation_ids = set(re.findall(r"`([a-z]+(?:-[a-z]+)*)`", operations))
            if not operation_ids or not operation_ids <= set(GATE0_OPERATION_IDS):
                self.add("journey.operation_ref", path, f"{journey_id} has an invalid operation reference")
            for operation in operation_ids & set(GATE0_OPERATION_IDS):
                operation_owners[operation].add(journey_id)
            feature_ids = set(re.findall(r"\bF-[0-9]{2}\b", features))
            if not feature_ids or not feature_ids <= set(GATE0_FEATURE_IDS):
                self.add("journey.feature_ref", path, f"{journey_id} has an invalid feature reference")
            for feature_id in feature_ids & set(GATE0_FEATURE_IDS):
                feature_owners[feature_id].add(journey_id)

        specs = markdown_section(text, "## 3. Journey specifications")
        headings = tuple(re.findall(r"(?m)^###\s+(J-[0-9]{2})\b", specs))
        if headings != GATE0_JOURNEY_IDS:
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
        for journey_id in GATE0_JOURNEY_IDS:
            body = markdown_section(specs, f"### {journey_id}", heading_level=3, prefix=True)
            for label in required_labels:
                if f"**{label}:**" not in body:
                    self.add("journey.spec_field", path, f"{journey_id} is missing {label}")
            flow = body.split("**Ordered flow:**", 1)[1].split("**Fail-closed outcomes:**", 1)[0] if "**Ordered flow:**" in body and "**Fail-closed outcomes:**" in body else ""
            numbers = [int(value) for value in re.findall(r"(?m)^([1-9][0-9]*)\.\s", flow)]
            if not numbers or numbers != list(range(1, len(numbers) + 1)):
                self.add("journey.flow_order", path, f"{journey_id} needs a consecutive ordered flow")

        if any(not owners for owners in primary_owners.values()):
            self.add("journey.persona_coverage", path, "every persona must own at least one primary journey")
        if any(not owners for owners in operation_owners.values()):
            self.add("journey.operation_coverage", path, "all ten operations require an owning journey")
        if any(not owners for owners in feature_owners.values()):
            self.add("journey.feature_coverage", path, "all fourteen features require journey coverage")
        self._validate_coverage_table(path, text, "Persona coverage", GATE0_PERSONA_IDS, primary_owners, "journey.persona_matrix")
        self._validate_coverage_table(path, text, "Operation coverage", GATE0_OPERATION_IDS, operation_owners, "journey.operation_matrix")
        self._validate_coverage_table(path, text, "Feature coverage", GATE0_FEATURE_IDS, feature_owners, "journey.feature_matrix")
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
                if not supporting <= set(GATE0_JOURNEY_IDS) or supporting & observed:
                    self.add(code, path, f"{identity} has invalid or overlapping supporting journeys")

    def _validate_proof_foundation_suite(self) -> None:
        path = self.root / "docs/PROOF_FOUNDATION_DECISION_SUITE.md"
        if not path.is_file():
            return
        text = markdown_without_fenced_blocks_and_comments(path.read_text(encoding="utf-8"))
        candidate_rows = table_rows(markdown_section(text, "## 2. Candidate parity and frozen inputs"), r"C-[0-9]{2}")
        if tuple(row[0] for row in candidate_rows) != ("C-01", "C-02"):
            self.add("proof_suite.candidates", path, "candidate table must contain C-01 and C-02 exactly once in order")
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
        gate_numbers = [int(value) for value in re.findall(r"(?m)^([1-9][0-9]*)\.\s", hard_gates)]
        if gate_numbers != list(range(1, 9)):
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
            if not directory.is_dir():
                continue
            seen: set[str] = set()
            candidates: list[Path] = []
            for path in sorted(directory.glob("*.md")):
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
                parsed = front_matter(path)
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
                    source = markdown_without_fenced_blocks_and_comments(path.read_text(encoding="utf-8"))
                    for heading in required_headings:
                        body = markdown_section(source, f"## {heading}")
                        if len(body.strip()) < 20:
                            self.add("record.section", path, f"missing or empty substantive section: {heading}")
                if metadata.get("status") == "Accepted":
                    decision_revision = metadata.get("decision-revision")
                    approval_records = metadata.get("approval-records")
                    if not isinstance(decision_revision, str) or not re.fullmatch(r"[0-9a-f]{40}", decision_revision):
                        self.add("record.acceptance", path, "Accepted record needs a full reviewed commit in decision-revision")
                    if not isinstance(approval_records, list) or not approval_records or not all(
                        nonempty_scalar(item) for item in approval_records
                    ):
                        self.add("record.acceptance", path, "Accepted record needs immutable approval-record references")
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
                                    "record.acceptance",
                                    path,
                                    "Accepted OEP approval-records must bind the exact decision-revision",
                                )
                        if parse_iso_date(metadata.get("decision-date")) is None:
                            self.add("record.acceptance", path, "Accepted OEP needs an exact decision-date")
                        related = metadata.get("related-decisions")
                        if not isinstance(related, list) or not related:
                            self.add("record.acceptance", path, "Accepted OEP needs at least one related decision")
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
                                "record.acceptance",
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
        if security.is_file() and "https://github.com/chasebryan/orange/security/advisories/new" not in security.read_text(encoding="utf-8"):
            self.add("security.private_reporting", security, "private vulnerability-reporting URL is missing")
        pr = self.root / ".github/pull_request_template.md"
        if pr.is_file():
            text = pr.read_text(encoding="utf-8")
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


def markdown_fence_error(text: str) -> str | None:
    active_char = ""
    active_length = 0
    active_line = 0
    for line_number, line in enumerate(text.splitlines(), start=1):
        match = re.match(r"^\s{0,3}(`{3,}|~{3,})(.*)$", line)
        if not match:
            continue
        marker, remainder = match.groups()
        char = marker[0]
        if not active_char:
            active_char, active_length, active_line = char, len(marker), line_number
        elif char == active_char and len(marker) >= active_length and not remainder.strip():
            active_char, active_length, active_line = "", 0, 0
    if active_char:
        return f"unclosed {active_char * active_length} fence opened on line {active_line}"
    return None


def checkout_disables_credentials(lines: Sequence[str], index: int) -> bool:
    uses_column = lines[index].find("uses:")
    if uses_column < 0:
        return False
    with_indent: int | None = None
    for line in lines[index + 1 :]:
        current_indent = len(line) - len(line.lstrip())
        stripped = line.strip()
        if stripped.startswith("-") and current_indent < uses_column:
            break
        if not stripped or stripped.startswith("#"):
            continue
        if current_indent == uses_column and re.fullmatch(r"with:\s*(?:#.*)?", stripped):
            with_indent = current_indent
            continue
        if with_indent is not None and current_indent <= with_indent:
            with_indent = None
        if with_indent is not None and re.fullmatch(
            r"persist-credentials:\s*(?:false|\"false\"|'false')(?:\s+#.*)?",
            stripped,
        ):
            return True
    return False


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
    pure = PurePosixPath(value)
    if pure.is_absolute() or ".." in pure.parts:
        return None
    candidate = (root / pure).resolve()
    try:
        candidate.relative_to(root.resolve())
    except ValueError:
        return None
    return candidate


def unsafe_run_interpolations(lines: Sequence[str]) -> list[int]:
    unsafe_fields = re.compile(
        r"\$\{\{\s*github\.event\.(?:comment|discussion|head_commit|issue|pull_request|review|workflow_run)\b"
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


def markdown_without_fenced_blocks_and_comments(text: str) -> str:
    """Return Markdown prose with code fences and HTML comments removed."""

    result: list[str] = []
    fence_char: str | None = None
    fence_length = 0
    for line in text.splitlines():
        match = re.match(r"^ {0,3}(`{3,}|~{3,})(?:[^`~].*)?$", line)
        if match:
            marker = match.group(1)
            if fence_char is None:
                fence_char, fence_length = marker[0], len(marker)
            elif marker[0] == fence_char and len(marker) >= fence_length:
                fence_char, fence_length = None, 0
            continue
        if fence_char is None:
            result.append(line)
    prose = "\n".join(result)
    uncommented: list[str] = []
    offset = 0
    while offset < len(prose):
        opening = prose.find("<!--", offset)
        closing = prose.find("-->", offset)
        if closing >= 0 and (opening < 0 or closing < opening):
            # A stray closer is invalid source, but it is not an opening that
            # can make later headings semantic; omit it from semantic parsing.
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
    """Detect unbalanced HTML comments outside fenced code blocks."""

    # Reuse semantic preprocessing's fence recognition while retaining comment
    # markers by temporarily replacing them with inert sentinels.
    protected = text.replace("<!--", "OPEN_COMMENT_SENTINEL").replace("-->", "CLOSE_COMMENT_SENTINEL")
    unfenced = markdown_without_fenced_blocks_and_comments(protected)
    tokens = re.findall(r"OPEN_COMMENT_SENTINEL|CLOSE_COMMENT_SENTINEL", unfenced)
    open_comment = False
    for token in tokens:
        if token == "OPEN_COMMENT_SENTINEL":
            if open_comment:
                return "nested HTML comment opener"
            open_comment = True
        elif not open_comment:
            return "HTML comment closer without opener"
        else:
            open_comment = False
    return "unclosed HTML comment" if open_comment else None


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


def front_matter(path: Path) -> tuple[dict[str, Any], list[str]] | None:
    lines = path.read_text(encoding="utf-8").splitlines()
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
            if key in result:
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
        if line.strip():
            errors.append(f"unsupported metadata syntax on line {line_number}")
        current_list = None
    errors.append("front matter is not closed")
    return result, errors


def nonempty_scalar(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def parse_rust_usize_product(value: str) -> int | None:
    """Parse the deliberately tiny integer-product form used by Rust budgets."""

    if re.fullmatch(r"\s*[0-9][0-9_]*(?:\s*\*\s*[0-9][0-9_]*)*\s*", value) is None:
        return None
    result = 1
    for factor in value.split("*"):
        result *= int(factor.strip().replace("_", ""), 10)
    return result


def rust_code_without_comments_and_literals(value: str) -> str:
    """Blank Rust comments and strings while preserving lines and offsets."""

    result = list(value)
    index = 0
    state = "code"
    block_depth = 0
    raw_closer = ""
    while index < len(value):
        if state == "code":
            raw = re.match(r'r(#{0,255})"', value[index:])
            if raw is not None:
                raw_closer = '"' + raw.group(1)
                length = raw.end()
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
    """Return whether an approval record positively claims a second reviewer."""

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


SUPPORTED_SCHEMA_KEYWORDS = {
    "$comment",
    "$defs",
    "$id",
    "$ref",
    "$schema",
    "additionalProperties",
    "allOf",
    "anyOf",
    "const",
    "default",
    "deprecated",
    "description",
    "enum",
    "examples",
    "exclusiveMaximum",
    "exclusiveMinimum",
    "format",
    "items",
    "maxItems",
    "maxLength",
    "maxProperties",
    "maximum",
    "minItems",
    "minLength",
    "minProperties",
    "minimum",
    "multipleOf",
    "not",
    "oneOf",
    "pattern",
    "patternProperties",
    "prefixItems",
    "properties",
    "readOnly",
    "required",
    "title",
    "type",
    "uniqueItems",
    "writeOnly",
}


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
    issues: list[SchemaIssue] = []
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
        if "pattern" in schema and re.search(schema["pattern"], instance) is None:
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
    document_ref, marker, fragment = reference.partition("#")
    target_path = schema_path
    target_root = root_schema
    if document_ref:
        if document_ref in id_registry:
            target_path, target_root = id_registry[document_ref]
        else:
            candidate = (schema_path.parent / document_ref).resolve()
            if candidate not in schemas:
                return None
            target_path, target_root = candidate, schemas[candidate]
    target: Any = target_root
    if marker and fragment:
        if not fragment.startswith("/"):
            return None
        for token in fragment[1:].split("/"):
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


def valid_format(value: str, format_name: str) -> bool:
    try:
        if format_name == "date":
            dt.date.fromisoformat(value)
            return bool(re.fullmatch(r"\d{4}-\d{2}-\d{2}", value))
        if format_name == "date-time":
            parsed = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
            return parsed.tzinfo is not None
        if format_name == "uri":
            parsed = urlsplit(value)
            return bool(parsed.scheme)
        if format_name == "uri-reference":
            urlsplit(value)
            return True
    except (TypeError, ValueError):
        return False
    # The Gate 0 schemas may annotate unfamiliar formats, but cannot use them as
    # validation assertions without adding deterministic support here.
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
                        "cross_invariant",
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
                            "cross_invariant",
                            f"$/basis/{index}/assumption_ref",
                            "assumption basis reference does not resolve",
                        )
                    )
    elif schema_name == "evidence-manifest-v0.1.schema.json":
        files = instance.get("files", [])
        file_paths = identifiers(files, "path", "$/files", issues)
        if isinstance(files, list) and [item.get("path") for item in files if isinstance(item, dict)] != sorted(file_paths):
            issues.append(SchemaIssue("cross_invariant", "$/files", "file records must be ordered by path"))
        external = instance.get("external_sources", [])
        if isinstance(external, list):
            identifiers(external, "source_id", "$/external_sources", issues, optional=True)
        replay = instance.get("replay")
        if isinstance(replay, dict):
            toolchains = replay.get("toolchains", [])
            names = identifiers(toolchains, "name", "$/replay/toolchains", issues)
            if isinstance(toolchains, list) and [item.get("name") for item in toolchains if isinstance(item, dict)] != sorted(names):
                issues.append(SchemaIssue("cross_invariant", "$/replay/toolchains", "toolchains must be ordered by name"))
    elif schema_name == "repository-control-snapshot-v0.1.schema.json":
        sources = identifiers(instance.get("evidence_sources"), "evidence_id", "$/evidence_sources", issues)
        for path, references in repository_control_evidence_refs(instance):
            for reference in references:
                if reference not in sources:
                    issues.append(
                        SchemaIssue(
                            "cross_invariant",
                            path,
                            f"repository-control evidence reference does not resolve: {reference}",
                        )
                    )
    elif schema_name == "standards-provenance-v0.1.schema.json":
        standards = instance.get("standards", [])
        standard_ids = identifiers(standards, "standard_id", "$/standards", issues)
        if isinstance(standards, list) and [item.get("standard_id") for item in standards if isinstance(item, dict)] != sorted(standard_ids):
            issues.append(SchemaIssue("cross_invariant", "$/standards", "standards must be ordered by standard_id"))
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
                            "cross_invariant",
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
                                "cross_invariant",
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
                    "cross_invariant",
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
            if isinstance(control, dict) and isinstance(control.get("evidence_refs"), list):
                result.append((f"$/security_features/{pointer_escape(name)}/evidence_refs", control["evidence_refs"]))
    actions = instance.get("actions")
    if isinstance(actions, dict):
        enabled = actions.get("enabled")
        if isinstance(enabled, dict) and isinstance(enabled.get("evidence_refs"), list):
            result.append(("$/actions/enabled/evidence_refs", enabled["evidence_refs"]))
    for field in ("default_branch_policy", "merge_policy"):
        value = instance.get(field)
        if isinstance(value, dict) and isinstance(value.get("evidence_refs"), list):
            result.append((f"$/{field}/evidence_refs", value["evidence_refs"]))
    return result


def expected_code_for_issue(schema_name: str, issue: SchemaIssue) -> str:
    if issue.keyword == "cross_invariant" and schema_name.startswith("claim-record-"):
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


def parse_arguments(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=("text", "json"), default="text", help="output format")
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    arguments = parse_arguments(sys.argv[1:] if argv is None else argv)
    # This repository-owned validator always binds its filesystem scope to the
    # checkout containing the script. Do not reintroduce caller-selected roots
    # or policy paths without a new trust-boundary design and containment tests.
    repository_root = Path(__file__).resolve().parents[1]
    validator = FoundationValidator(repository_root)
    findings = validator.run()
    if arguments.format == "json":
        output = {
            "schema_version": "0.1.0",
            "repository": validator.policy.get("repository", "unknown"),
            "policy_version": validator.policy.get("policy_version", "unknown"),
            "valid": not findings,
            "findings": [finding.as_dict() for finding in findings],
        }
        print(json.dumps(output, sort_keys=True, separators=(",", ":")))
    elif findings:
        for finding in findings:
            print(f"{finding.path}: {finding.code}: {finding.message}")
        print(f"Solo-bootstrap repository policy failed with {len(findings)} finding(s).")
    else:
        print(
            "Solo-bootstrap repository policy passed "
            f"({validator.policy['repository']} policy {validator.policy['policy_version']})."
        )
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
