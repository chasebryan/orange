from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.validate_foundation import (
    DuplicateKeyError,
    FoundationValidator,
    audit_schema_vocabulary,
    checkout_disables_credentials,
    load_json,
    markdown_anchors,
    markdown_fence_error,
    unsafe_run_interpolations,
    validate_schema_instance,
    workflow_jobs,
)


class JsonParsingTests(unittest.TestCase):
    def test_duplicate_keys_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "duplicate.json"
            path.write_text('{"scope":"first","scope":"second"}\n', encoding="utf-8")
            with self.assertRaises(DuplicateKeyError):
                load_json(path)

    def test_unambiguous_json_loads(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "record.json"
            path.write_text('{"scope":"one","status":true}\n', encoding="utf-8")
            self.assertEqual(load_json(path), {"scope": "one", "status": True})


class MarkdownTests(unittest.TestCase):
    def test_heading_anchors_match_github_style_duplicates(self) -> None:
        anchors = markdown_anchors("# One heading\n\n## Repeated\n\n## Repeated\n")
        self.assertEqual(anchors, {"one-heading", "repeated", "repeated-1"})

    def test_missing_relative_link_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text("# Source\n\n[missing](absent.md)\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_markdown_links()
            self.assertIn("markdown.link_missing", {finding.code for finding in validator.findings})

    def test_existing_cross_file_anchor_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text("# Source\n\n[target](target.md#exact-heading)\n", encoding="utf-8")
            (root / "target.md").write_text("# Exact heading\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_markdown_links()
            self.assertEqual(validator.findings, [])

    def test_unclosed_fence_is_rejected(self) -> None:
        self.assertEqual(
            markdown_fence_error("# Document\n\n```text\nnot closed\n"),
            "unclosed ``` fence opened on line 3",
        )

    def test_longer_closing_fence_is_valid(self) -> None:
        self.assertIsNone(markdown_fence_error("~~~text\ncontent\n~~~~\n"))

    @staticmethod
    def _orange_book_text(*, byline: str = "By Chase Bryan", chapter_words: int = 1_200) -> str:
        chapter = " ".join("evidence" for _ in range(chapter_words))
        return f"""# The Orange Book

{byline}

Status: living pre-alpha reader guide

Snapshot: 2026-07-12

This is not a normative language specification.

## Contents

- [Preface](#preface)
- [Chapter 1: The Seams Are the System](#chapter-1-the-seams-are-the-system)
- [Manuscript map](#manuscript-map)
- [Sources and drafting disclosure](#sources-and-drafting-disclosure)

## Preface

Reader context.

## Chapter 1: The Seams Are the System

{chapter}

## Manuscript map

Future chapters.

## Sources and drafting disclosure

Drafted with OpenAI Codex, based on GPT-5. Chase Bryan is the named author.
"""

    def test_orange_book_contract_accepts_v01_structure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            path.write_text(self._orange_book_text(), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual(validator.findings, [])

    def test_orange_book_contract_rejects_wrong_byline_and_short_chapter(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            path.write_text(
                self._orange_book_text(byline="By Someone Else", chapter_words=20),
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual(
                {finding.code for finding in validator.findings},
                {"book.chapter_length", "book.identity"},
            )

    def test_orange_book_contract_rejects_reordered_contents(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            text = self._orange_book_text().replace(
                "- [Preface](#preface)\n"
                "- [Chapter 1: The Seams Are the System](#chapter-1-the-seams-are-the-system)",
                "- [Chapter 1: The Seams Are the System](#chapter-1-the-seams-are-the-system)\n"
                "- [Preface](#preface)",
            )
            path.write_text(text, encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual({finding.code for finding in validator.findings}, {"book.navigation"})

    def test_orange_book_contract_rejects_hidden_chapter_and_disclosure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            hidden_words = " ".join("evidence" for _ in range(1_200))
            text = self._orange_book_text(chapter_words=20).replace(
                "\n## Manuscript map",
                f"\n<!-- {hidden_words} -->\n\n## Manuscript map",
            ).replace(
                "Drafted with OpenAI Codex, based on GPT-5. Chase Bryan is the named author.",
                "<!-- OpenAI Codex GPT-5 Chase Bryan is the named author -->",
            )
            path.write_text(text, encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual(
                {finding.code for finding in validator.findings},
                {"book.chapter_length", "book.disclosure"},
            )

    def test_orange_book_contract_rejects_impossible_snapshot_date(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            text = self._orange_book_text().replace("Snapshot: 2026-07-12", "Snapshot: 2026-99-99")
            path.write_text(text, encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual({finding.code for finding in validator.findings}, {"book.snapshot"})

    def test_orange_book_contract_rejects_competing_byline(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            text = self._orange_book_text().replace(
                "By Chase Bryan\n",
                "By Chase Bryan\n\nBy Someone Else\n",
                1,
            )
            path.write_text(text, encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual({finding.code for finding in validator.findings}, {"book.identity"})


class WorkflowTests(unittest.TestCase):
    def test_checkout_credentials_must_be_explicitly_disabled(self) -> None:
        safe = [
            "      - name: Checkout",
            "        uses: actions/checkout@" + "a" * 40 + " # v1.0.0",
            "        with:",
            "          persist-credentials: false",
            "      - name: Next",
        ]
        unsafe = safe[:2] + safe[4:]
        self.assertTrue(checkout_disables_credentials(safe, 1))
        self.assertFalse(checkout_disables_credentials(unsafe, 1))

    def test_job_blocks_are_found_without_parsing_untrusted_yaml(self) -> None:
        jobs = workflow_jobs(
            [
                "name: Test",
                "jobs:",
                "  first:",
                "    timeout-minutes: 5",
                "  second:",
                "    timeout-minutes: 7",
            ]
        )
        self.assertEqual([name for name, _ in jobs], ["first", "second"])

    def test_untrusted_event_interpolation_in_run_is_rejected(self) -> None:
        lines = [
            "      - name: Unsafe",
            "        run: |",
            "          printf '%s' '${{ github.event.issue.title }}'",
        ]
        self.assertEqual(unsafe_run_interpolations(lines), [2])

    def test_mutable_action_ref_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflow_dir = root / ".github/workflows"
            workflow_dir.mkdir(parents=True)
            (workflow_dir / "ci.yml").write_text(
                """name: CI
on:
  pull_request:
  push:
  merge_group:
permissions: {}
concurrency:
  group: ci
  cancel-in-progress: true
jobs:
  check:
    timeout-minutes: 5
    permissions: {}
    steps:
      - uses: actions/checkout@v7 # v7.0.0
        with:
          persist-credentials: false
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = {
                "required_workflows": ["ci.yml"],
                "github_actions": {
                    "allowed_action_repositories": ["actions/checkout"],
                    "allowed_write_permissions": {},
                    "forbidden_events": ["pull_request_target"],
                    "require_full_commit_sha": True,
                    "require_version_comment": True,
                },
            }
            validator._validate_workflows()
            self.assertIn("workflow.mutable_action", {finding.code for finding in validator.findings})


class ProvisionalSchemaTests(unittest.TestCase):
    def setUp(self) -> None:
        self.schema_path = Path("/virtual/record.schema.json")
        self.schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "urn:orange:gate0:test:v0.1",
            "type": "object",
            "required": ["id", "status"],
            "properties": {
                "id": {"type": "string", "pattern": "^[a-z]+-[0-9]+$"},
                "status": {"enum": ["open", "closed"]},
            },
            "additionalProperties": False,
        }

    def validate(self, instance: object):
        return validate_schema_instance(
            instance,
            self.schema,
            self.schema_path,
            {self.schema_path: self.schema},
            {self.schema["$id"]: (self.schema_path, self.schema)},
        )

    def test_valid_instance_passes(self) -> None:
        self.assertEqual(self.validate({"id": "claim-1", "status": "open"}), [])

    def test_invalid_instance_fails_on_stable_keyword_and_path(self) -> None:
        issues = self.validate({"id": "INVALID", "status": "open"})
        self.assertIn(("pattern", "$/id"), {(issue.keyword, issue.instance_path) for issue in issues})

    def test_unknown_property_fails_closed(self) -> None:
        issues = self.validate({"id": "claim-1", "status": "open", "surprise": True})
        self.assertIn(
            ("additionalProperties", "$/surprise"),
            {(issue.keyword, issue.instance_path) for issue in issues},
        )

    def test_unsupported_schema_keyword_is_detected(self) -> None:
        schema = dict(self.schema)
        schema["if"] = {"required": ["x"]}
        findings = audit_schema_vocabulary(schema)
        self.assertTrue(any("unsupported keyword 'if'" in finding for finding in findings))


if __name__ == "__main__":
    unittest.main()
