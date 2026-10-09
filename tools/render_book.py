#!/usr/bin/env python3
"""Render the in-progress Orange Book to static HTML.

Uses only the Python standard library. Reads ``docs/book/manifest.json``,
checks drafted files against planned chapters, and writes an index plus one
HTML page per manuscript file. Does not fetch, execute Orange, or claim review.
"""

from __future__ import annotations

import html
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = ROOT / "docs" / "book" / "manifest.json"
PART_TITLES = {
    "novice": "Part 1, The Novice",
    "journeyman": "Part 2, The Journeyman",
    "master": "Part 3, The Master",
    "original": "Original manuscript",
}
PART_ORDER = ("novice", "journeyman", "master", "original")
STATUSES = {"draft", "planned", "original"}
HEADING_RE = re.compile(r"^(#{1,6})[ \t]+(.+?)[ \t]*#*[ \t]*$")
FENCE_RE = re.compile(r"^ {0,3}(`{3,}|~{3,})(.*)$")


def heading_anchor(title: str) -> str:
    """Slug a heading the way the expansion index already links it."""
    return re.sub(r"[^\w\- ]", "", title.lower()).replace(" ", "-")


def load_manifest(root: Path = ROOT) -> dict:
    """Return the manuscript manifest after checking its files and statuses."""
    path = root / "docs" / "book" / "manifest.json"
    manifest = json.loads(path.read_text(encoding="utf-8"))
    if manifest.get("kind") != "orange-book-manuscript-manifest":
        raise ValueError("manifest kind is not the Orange Book manuscript manifest")
    if manifest.get("version") != 1:
        raise ValueError("manifest version must be 1")
    if manifest.get("status") != "in-progress":
        raise ValueError("manifest must say the book is in progress")
    chapters = manifest.get("chapters")
    if not isinstance(chapters, list) or not chapters:
        raise ValueError("manifest needs a chapter list")
    seen: set[str] = set()
    drafted_files: set[str] = set()
    for chapter in chapters:
        identity = chapter.get("id")
        if not isinstance(identity, str) or not identity or identity in seen:
            raise ValueError(f"chapter id is missing or repeated: {identity!r}")
        seen.add(identity)
        if chapter.get("part") not in PART_TITLES:
            raise ValueError(f"{identity} has an unknown part")
        status = chapter.get("status")
        if status not in STATUSES:
            raise ValueError(f"{identity} has an unknown status")
        title = chapter.get("title")
        if not isinstance(title, str) or not title.strip():
            raise ValueError(f"{identity} needs a title")
        source = chapter.get("path")
        if status == "planned":
            if source is not None:
                raise ValueError(f"planned chapter {identity} must not name a file")
            continue
        if not isinstance(source, str) or not source or source.startswith("/") or ".." in Path(source).parts:
            raise ValueError(f"{identity} needs a repository-relative path")
        file_path = root / source
        if not file_path.is_file():
            raise ValueError(f"{identity} is missing {source}")
        if status == "draft":
            if not source.startswith("docs/book/") or not source.endswith(".md"):
                raise ValueError(f"draft chapter {identity} must live under docs/book/")
            drafted_files.add(source)
        elif source != "docs/THE_ORANGE_BOOK.md":
            raise ValueError("the original manuscript entry must be docs/THE_ORANGE_BOOK.md")
        anchor = chapter.get("anchor")
        if anchor is not None:
            anchors = heading_anchors(file_path.read_text(encoding="utf-8"))
            if anchor not in anchors:
                raise ValueError(f"{identity} anchor {anchor} is not a heading in {source}")
    lesson_files = {
        relative(path, root)
        for path in (root / "docs" / "book").glob("NOVICE*.md")
    }
    if lesson_files != drafted_files:
        missing = sorted(lesson_files - drafted_files)
        extra = sorted(drafted_files - lesson_files)
        raise ValueError(f"draft files drifted; missing={missing}, extra={extra}")
    return manifest


def relative(path: Path, root: Path) -> str:
    return path.resolve().relative_to(root.resolve()).as_posix()


def heading_anchors(text: str) -> set[str]:
    anchors: set[str] = set()
    counts: dict[str, int] = {}
    fence: tuple[str, int] | None = None
    for line in text.splitlines():
        marker = FENCE_RE.match(line)
        if marker:
            char, info = marker.group(1)[0], marker.group(2)
            if fence is None and not (char == "`" and "`" in info):
                fence = (char, len(marker.group(1)))
            elif fence and char == fence[0] and len(marker.group(1)) >= fence[1] and not info.strip():
                fence = None
            continue
        if fence:
            continue
        heading = HEADING_RE.match(line)
        if not heading:
            continue
        anchor = heading_anchor(heading.group(2))
        count = counts.get(anchor, 0)
        counts[anchor] = count + 1
        anchors.add(anchor if count == 0 else f"{anchor}-{count}")
    return anchors


def render(root: Path, output: Path) -> None:
    """Write the rendered book under ``output``, which must be outside ``root``."""
    output = output.resolve()
    root = root.resolve()
    if output == root or root in output.parents:
        raise ValueError("refusing to write the rendered book inside the repository")
    manifest = load_manifest(root)
    output.mkdir(parents=True, exist_ok=True)
    (output / "manifest.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    pages: dict[str, str] = {}
    for source in manuscript_files(manifest):
        text = (root / source).read_text(encoding="utf-8")
        destination = html_path(source)
        pages[source] = destination
        target = output / destination
        target.parent.mkdir(parents=True, exist_ok=True)
        banner = page_banner(manifest, source)
        target.write_text(
            page(title_of(text, source), body(text, source), banner, index_href(destination)),
            encoding="utf-8",
        )
    (output / "index.html").write_text(
        page("The Orange Book", index_body(manifest, pages), "", ""),
        encoding="utf-8",
    )


def manuscript_files(manifest: dict) -> list[str]:
    files: list[str] = []
    for chapter in manifest["chapters"]:
        source = chapter.get("path")
        if source and source not in files:
            files.append(source)
    return files


def html_path(source: str) -> str:
    return str(Path(source).with_suffix(".html")).replace("\\", "/")


def title_of(text: str, source: str) -> str:
    for line in text.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return Path(source).stem


def page_banner(manifest: dict, source: str) -> str:
    states = {
        chapter["review_state"]
        for chapter in manifest["chapters"]
        if chapter.get("path") == source and chapter["status"] == "draft"
    }
    if not states:
        return ""
    if states == {"owner-approved-with-unreviewed-corrections"}:
        note = (
            "Draft. The owner approved the opening before the continuation. "
            "Later corrections in this file have not had a separate review."
        )
    else:
        note = "Draft. Owner review is pending. This page is not a finished chapter."
    return f'<p class="banner">{html.escape(note)}</p>'


def index_href(destination: str) -> str:
    depth = destination.count("/")
    return f'{"../" * depth}index.html'


def index_body(manifest: dict, pages: dict[str, str]) -> str:
    parts = [
        "<h1>The Orange Book</h1>",
        '<p class="banner">Living, in-progress manuscript. '
        "Draft lessons are not a finished book. Planned lessons are not written. "
        "This rendering is not an assurance claim or an independent review.</p>",
        f"<p>{html.escape(manifest['review'])}</p>",
    ]
    for part in PART_ORDER:
        chapters = [chapter for chapter in manifest["chapters"] if chapter["part"] == part]
        parts.append(f"<h2>{html.escape(PART_TITLES[part])}</h2>")
        parts.append("<ul>")
        for chapter in chapters:
            label = f'{html.escape(chapter["title"])} — {html.escape(chapter["status"])}'
            source = chapter.get("path")
            if source:
                href = html.escape(pages[source], quote=True)
                anchor = chapter.get("anchor")
                if anchor:
                    href = f"{href}#{html.escape(anchor, quote=True)}"
                item = f'<a href="{href}">{label}</a>'
            else:
                item = f'<span class="planned">{label}</span>'
            summary = chapter.get("summary")
            if summary:
                item += f"<br>{html.escape(summary)}"
            parts.append(f"<li>{item}</li>")
        parts.append("</ul>")
    return "\n".join(parts)


def page(title: str, article: str, banner: str, index: str) -> str:
    home = f'<p><a href="{html.escape(index, quote=True)}">Manuscript status</a></p>' if index else ""
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{html.escape(title)}</title>
<style>
body {{ font-family: Georgia, serif; max-width: 44rem; margin: 2rem auto; padding: 0 1rem; line-height: 1.55; color: #23180f; }}
code, pre {{ font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }}
pre {{ background: #f6f3ee; padding: 1rem; overflow: auto; }}
table {{ border-collapse: collapse; }}
td, th {{ border: 1px solid #d9cfc2; padding: 0.3rem 0.5rem; vertical-align: top; }}
.banner {{ background: #fff4e5; border: 1px solid #e6c48a; padding: 0.8rem 1rem; }}
.planned {{ color: #6b4f2a; }}
blockquote {{ border-left: 3px solid #e6c48a; margin-left: 0; padding-left: 1rem; }}
</style>
</head>
<body>
{banner}
{home}
<article>
{article}
</article>
{home}
</body>
</html>
"""


def body(text: str, source: str) -> str:
    lines = text.splitlines()
    blocks: list[str] = []
    paragraph: list[str] = []
    index = 0

    def flush() -> None:
        if paragraph:
            blocks.append(f"<p>{inline(' '.join(paragraph), source)}</p>")
            paragraph.clear()

    while index < len(lines):
        line = lines[index]
        fence = FENCE_RE.match(line)
        if fence and not (fence.group(1)[0] == "`" and "`" in fence.group(2)):
            flush()
            marker, length = fence.group(1)[0], len(fence.group(1))
            code: list[str] = []
            index += 1
            while index < len(lines):
                close = FENCE_RE.match(lines[index])
                if (
                    close
                    and close.group(1)[0] == marker
                    and len(close.group(1)) >= length
                    and not close.group(2).strip()
                ):
                    break
                code.append(lines[index])
                index += 1
            blocks.append(f"<pre><code>{html.escape(chr(10).join(code))}</code></pre>")
            index += 1
            continue
        heading = HEADING_RE.match(line)
        if heading:
            flush()
            level = len(heading.group(1))
            title = heading.group(2).strip()
            blocks.append(
                f'<h{level} id="{html.escape(heading_anchor(title), quote=True)}">'
                f"{inline(title, source)}</h{level}>"
            )
            index += 1
            continue
        if is_table_start(lines, index):
            flush()
            rows, index = table_rows(lines, index)
            blocks.append(render_table(rows, source))
            continue
        if re.match(r"^ {0,3}[-*+][ \t]+", line) or re.match(r"^ {0,3}\d+[.)][ \t]+", line):
            flush()
            items, index, ordered = list_items(lines, index)
            tag = "ol" if ordered else "ul"
            rendered = "".join(f"<li>{inline(item, source)}</li>" for item in items)
            blocks.append(f"<{tag}>{rendered}</{tag}>")
            continue
        if line.startswith(">"):
            flush()
            quoted: list[str] = []
            while index < len(lines) and lines[index].startswith(">"):
                quoted.append(re.sub(r"^>\s?", "", lines[index]))
                index += 1
            blocks.append(f"<blockquote><p>{inline(' '.join(quoted), source)}</p></blockquote>")
            continue
        if not line.strip():
            flush()
            index += 1
            continue
        paragraph.append(line.strip())
        index += 1
    flush()
    return "\n".join(blocks)


def is_table_start(lines: list[str], index: int) -> bool:
    if index + 1 >= len(lines) or not lines[index].lstrip().startswith("|"):
        return False
    return bool(re.match(r"^\s*\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)+\|?\s*$", lines[index + 1]))


def table_rows(lines: list[str], index: int) -> tuple[list[list[str]], int]:
    rows: list[list[str]] = []
    while index < len(lines) and lines[index].lstrip().startswith("|"):
        if not re.match(r"^\s*\|?\s*:?-{3,}", lines[index]):
            cells = [cell.strip() for cell in lines[index].strip().strip("|").split("|")]
            rows.append(cells)
        index += 1
    return rows, index


def render_table(rows: list[list[str]], source: str) -> str:
    if not rows:
        return ""
    head = "".join(f"<th>{inline(cell, source)}</th>" for cell in rows[0])
    body_rows = []
    for row in rows[1:]:
        body_rows.append("<tr>" + "".join(f"<td>{inline(cell, source)}</td>" for cell in row) + "</tr>")
    return f"<table><thead><tr>{head}</tr></thead><tbody>{''.join(body_rows)}</tbody></table>"


def list_items(lines: list[str], index: int) -> tuple[list[str], int, bool]:
    ordered = bool(re.match(r"^ {0,3}\d+[.)][ \t]+", lines[index]))
    pattern = r"^ {0,3}\d+[.)][ \t]+" if ordered else r"^ {0,3}[-*+][ \t]+"
    items: list[str] = []
    while index < len(lines) and re.match(pattern, lines[index]):
        items.append(re.sub(pattern, "", lines[index]).strip())
        index += 1
    return items, index, ordered


def inline(text: str, source: str) -> str:
    pieces: list[str] = []
    cursor = 0
    pattern = re.compile(
        r"(`+)([^`]*)\1"
        r"|!\[([^\]]*)\]\(([^)\s]+)\)"
        r"|\[([^\]]+)\]\(([^)\s]+)\)"
        r"|\*\*([^*]+)\*\*"
        r"|(?<!\*)\*([^*]+)\*(?!\*)"
    )
    for match in pattern.finditer(text):
        pieces.append(html.escape(text[cursor:match.start()]))
        if match.group(2) is not None:
            pieces.append(f"<code>{html.escape(match.group(2))}</code>")
        elif match.group(4) is not None:
            pieces.append(image(match.group(3), match.group(4)))
        elif match.group(6) is not None:
            pieces.append(link(match.group(5), match.group(6), source))
        elif match.group(7) is not None:
            pieces.append(f"<strong>{html.escape(match.group(7))}</strong>")
        else:
            pieces.append(f"<em>{html.escape(match.group(8))}</em>")
        cursor = match.end()
    pieces.append(html.escape(text[cursor:]))
    return "".join(pieces)


def safe_url(url: str, source: str) -> str | None:
    if url.startswith(("https://", "http://", "mailto:")):
        return url
    if url.startswith(("#", "/")) or "\\" in url or url.startswith("//"):
        return url if url.startswith("#") else None
    if ".." in Path(url.split("#", 1)[0]).parts:
        return None
    rewritten = url
    path, _, fragment = url.partition("#")
    if path.endswith(".md"):
        rewritten = str(Path(path).with_suffix(".html")).replace("\\", "/")
        if fragment:
            rewritten += f"#{fragment}"
    base = Path(source).parent
    target = (base / rewritten.split("#", 1)[0]).as_posix()
    if target.startswith("..") and "docs/" not in target and not target.endswith(".html"):
        return None
    return rewritten


def link(label: str, url: str, source: str) -> str:
    target = safe_url(url, source)
    if target is None:
        return html.escape(label)
    return f'<a href="{html.escape(target, quote=True)}">{html.escape(label)}</a>'


def image(alt: str, url: str) -> str:
    if not url.startswith(("https://", "http://")) and (url.startswith("/") or ".." in Path(url).parts):
        return html.escape(alt)
    return f'<img alt="{html.escape(alt, quote=True)}" src="{html.escape(url, quote=True)}">'


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: render_book.py OUTPUT_DIR", file=sys.stderr)
        return 2
    try:
        render(ROOT, Path(argv[1]))
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"orange book render failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
