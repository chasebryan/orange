#!/usr/bin/env python3
"""Render the in-progress Orange Book to static HTML.

Uses only the Python standard library. Reads ``docs/book/manifest.json``,
checks drafted files against planned chapters, and writes an index plus one
HTML page per manuscript file. Does not fetch, execute Orange, or claim review.
"""

from __future__ import annotations

import html
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY_URL = "https://github.com/chasebryan/orange"
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
    return set(heading_anchor_list(text))


def heading_anchor_list(text: str) -> list[str]:
    anchors: list[str] = []
    counts: dict[str, int] = {}
    for title in heading_titles(text):
        anchor = heading_anchor(title)
        count = counts.get(anchor, 0)
        counts[anchor] = count + 1
        anchors.append(anchor if count == 0 else f"{anchor}-{count}")
    return anchors


def heading_titles(text: str) -> list[str]:
    titles: list[str] = []
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
        if heading:
            titles.append(heading.group(2))
    return titles


def fence_is_unclosed(text: str) -> bool:
    fence: tuple[str, int] | None = None
    for line in text.splitlines():
        marker = FENCE_RE.match(line)
        if not marker:
            continue
        char, info = marker.group(1)[0], marker.group(2)
        if fence is None and not (char == "`" and "`" in info):
            fence = (char, len(marker.group(1)))
        elif fence and char == fence[0] and len(marker.group(1)) >= fence[1] and not info.strip():
            fence = None
    return fence is not None


def leading_front_matter(text: str) -> bool:
    body_text = text.lstrip("\ufeff")
    return body_text.startswith("---\n") or body_text.startswith("---\r\n")


INCLUDE_RE = re.compile(r"""\{%\s*include\s+(?:["']([^"']+)["']|(\S+))\s*%\}""")
LINK_RE = re.compile(r"\[([^\]]+)\]\(([^)\s]+)\)")
INLINE_CODE_RE = re.compile(r"`+")


def prose_lines(text: str) -> list[tuple[int, str]]:
    """Return one-based lines outside fenced code, with inline code removed."""
    lines: list[tuple[int, str]] = []
    fence: tuple[str, int] | None = None
    for number, line in enumerate(text.splitlines(), start=1):
        marker = FENCE_RE.match(line)
        if marker:
            char, info = marker.group(1)[0], marker.group(2)
            if fence is None and not (char == "`" and "`" in info):
                fence = (char, len(marker.group(1)))
                continue
            if fence and char == fence[0] and len(marker.group(1)) >= fence[1] and not info.strip():
                fence = None
                continue
        if fence:
            continue
        lines.append((number, strip_inline_code(line)))
    return lines


def strip_inline_code(line: str) -> str:
    pieces: list[str] = []
    cursor = 0
    while cursor < len(line):
        marker = INLINE_CODE_RE.search(line, cursor)
        if marker is None:
            pieces.append(line[cursor:])
            break
        pieces.append(line[cursor:marker.start()])
        close = line.find(marker.group(0), marker.end())
        if close < 0:
            break
        cursor = close + len(marker.group(0))
    return "".join(pieces)


def link_target(root: Path, source: Path, raw_path: str) -> Path | None:
    """Resolve a relative link to a file or directory inside ``root``."""
    root = root.resolve()
    name = raw_path.strip()
    if not name or name.startswith(("/", "\\")) or "\\" in name:
        return None
    candidate = (source.resolve().parent / name).resolve()
    try:
        candidate.relative_to(root)
    except ValueError:
        return None
    if os.path.commonpath((os.fspath(candidate), os.fspath(root))) != os.fspath(root):
        return None
    if candidate.is_file() or candidate.is_dir():
        return candidate
    return None


def include_target(root: Path, source: Path, raw_path: str) -> Path | None:
    target = link_target(root, source, raw_path)
    if target is not None and target.is_file():
        return target
    return None


def require_well_formed(text: str, source: Path, root: Path) -> None:
    """Reject front matter, an unclosed fence, or a missing include."""
    problems: list[str] = []
    if leading_front_matter(text):
        problems.append(f"{source}: leading front-matter block")
    if fence_is_unclosed(text):
        problems.append(f"{source}: unclosed code fence")
    for number, line in prose_lines(text):
        for match in INCLUDE_RE.finditer(line):
            raw_path = match.group(1) or match.group(2)
            if include_target(root, source, raw_path) is None:
                problems.append(f"{source}:{number}: include target does not exist: {raw_path}")
    if problems:
        raise ValueError("malformed chapter\n" + "\n".join(problems))


def source_link_errors(root: Path, sources: list[Path]) -> list[str]:
    """Return dead relative links as ``file:line: target`` errors."""
    root = root.resolve()
    errors: list[str] = []
    for source in sources:
        text = source.read_text(encoding="utf-8")
        anchors = heading_anchors(text)
        for number, line in prose_lines(text):
            for match in LINK_RE.finditer(line):
                url = match.group(2)
                problem = relative_link_problem(root, source, url, anchors)
                if problem is not None:
                    errors.append(f"{source.relative_to(root)}:{number}: {problem}")
    return errors


def relative_link_problem(
    root: Path, source: Path, url: str, own_anchors: set[str]
) -> str | None:
    if url.startswith(("https://", "http://", "mailto:")):
        return None
    if url.startswith(("//", "/")) or "\\" in url:
        return f"unresolved relative link {url}"
    path, _, fragment = url.partition("#")
    if not path:
        if fragment not in own_anchors:
            return f"missing anchor {url}"
        return None
    target = link_target(root, source, path)
    if target is None:
        return f"missing target {url}"
    if not fragment:
        return None
    if target.is_dir() or target.suffix.lower() != ".md":
        return f"missing anchor {url}"
    if fragment not in heading_anchors(target.read_text(encoding="utf-8")):
        return f"missing anchor {url}"
    return None


def book_output(root: Path) -> Path:
    """Return the only directory the renderer may write: ``<root>/build/book``.

    Refuse a symlink at ``build``, ``build/book``, or any component between
    the repository root and that directory. Each component is checked with
    ``is_symlink()``; the decision does not follow links.
    """
    root = root.resolve()
    current = root
    for part in ("build", "book"):
        current = current / part
        if current.is_symlink():
            raise ValueError(f"rendered book path is a symlink: {part}")
    output = root / "build" / "book"
    if output != current or output.parent != root / "build" or output.parent.parent != root:
        raise ValueError("rendered book must stay in build/book")
    return output


def remove_rendered_book(root: Path, output: Path) -> None:
    """Delete a previous render only when it is a real directory under ``root/build``."""
    build = root / "build"
    for candidate in (build, output):
        if candidate.is_symlink():
            raise ValueError("rendered book path is a symlink")
    if not output.exists():
        return
    if (
        output.is_symlink()
        or build.is_symlink()
        or not build.is_dir()
        or not output.is_dir()
        or output.parent != build
        or build.parent != root
    ):
        raise ValueError("rendered book must stay in build/book")
    shutil.rmtree(output)


def within_output(output: Path, relative_path: str) -> Path:
    """Resolve one render-relative path and refuse anything outside ``output``."""
    output = output.resolve()
    if relative_path.startswith("/") or ".." in Path(relative_path).parts:
        raise ValueError(f"render path escapes build/book: {relative_path}")
    target = (output / relative_path).resolve()
    target.relative_to(output)
    if os.path.commonpath((os.fspath(target), os.fspath(output))) != os.fspath(output):
        raise ValueError(f"render path escapes build/book: {relative_path}")
    return target


def render(root: Path = ROOT) -> Path:
    """Write the rendered book under ``<root>/build/book`` and return that directory."""
    root = root.resolve()
    output = book_output(root)
    remove_rendered_book(root, output)
    output.mkdir(parents=True)
    manifest = load_manifest(root)
    sources = manuscript_files(manifest)
    texts = {}
    for source in sources:
        path = root / source
        text = path.read_text(encoding="utf-8")
        require_well_formed(text, path, root)
        texts[source] = text
    within_output(output, "manifest.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    pages: dict[str, str] = {}
    manuscript = set(sources)
    for source in sources:
        text = texts[source]
        destination = html_path(source)
        pages[source] = destination
        target = within_output(output, destination)
        target.parent.mkdir(parents=True, exist_ok=True)
        banner = page_banner(manifest, source)
        target.write_text(
            page(
                title_of(text, source),
                body(text, source, root, manuscript),
                banner,
                index_href(destination),
            ),
            encoding="utf-8",
        )
    within_output(output, "index.html").write_text(
        page("The Orange Book", index_body(manifest, pages), "", ""),
        encoding="utf-8",
    )
    require_complete_output(output, list(pages.values()))
    link_errors = source_link_errors(root, [root / source for source in sources])
    link_errors.extend(written_href_errors(output, root, pages))
    if link_errors:
        raise ValueError("dead links\n" + "\n".join(link_errors))
    return output


def require_complete_output(output: Path, destinations: list[str]) -> None:
    """Require one non-empty HTML page per manuscript file, plus the index."""
    problems: list[str] = []
    for relative_path in [*destinations, "index.html"]:
        path = within_output(output, relative_path)
        if not path.is_file() or not path.read_text(encoding="utf-8").strip():
            problems.append(f"missing non-empty page {relative_path}")
            continue
        article = re.search(r"<article>(.*)</article>", path.read_text(encoding="utf-8"), re.S)
        if article is None or not article.group(1).strip():
            problems.append(f"hollow page {relative_path}")
    if problems:
        raise ValueError("hollow rendered book\n" + "\n".join(problems))


def written_href_errors(output: Path, root: Path, pages: dict[str, str]) -> list[str]:
    """Resolve every relative href in the written pages."""
    output = output.resolve()
    root = root.resolve()
    by_destination = {destination: source for source, destination in pages.items()}
    errors: list[str] = []
    for destination in [*pages.values(), "index.html"]:
        page_path = within_output(output, destination)
        for href in re.findall(r'href="([^"]*)"', page_path.read_text(encoding="utf-8")):
            problem = written_href_problem(output, root, page_path, href, by_destination)
            if problem is not None:
                errors.append(f"{destination}: {problem}")
    return errors


def written_href_problem(
    output: Path,
    root: Path,
    page_path: Path,
    href: str,
    by_destination: dict[str, str],
) -> str | None:
    if href.startswith(("https://", "http://", "mailto:")):
        return None
    if href.startswith(("//", "/")) or "\\" in href:
        return f"local href does not resolve to a file in the artifact: {href}"
    path, _, fragment = href.partition("#")
    if not path:
        return fragment_problem(root, page_path, output, by_destination, fragment, href)
    try:
        resolved = (page_path.parent / path).resolve()
        relative = resolved.relative_to(output)
    except ValueError:
        return f"local href does not resolve to a file in the artifact: {href}"
    if os.path.commonpath((os.fspath(resolved), os.fspath(output))) != os.fspath(output):
        return f"local href does not resolve to a file in the artifact: {href}"
    if not resolved.is_file():
        return f"local href does not resolve to a file in the artifact: {href}"
    relative_path = relative.as_posix()
    if not fragment:
        return None
    if relative_path == "index.html":
        return None if fragment in html_ids(resolved.read_text(encoding="utf-8")) else f"missing anchor {href}"
    source_name = by_destination.get(relative_path)
    if source_name is None:
        return f"missing anchor {href}"
    source_path = root / source_name
    if fragment not in heading_anchors(source_path.read_text(encoding="utf-8")):
        return f"missing anchor {href}"
    return None


def fragment_problem(
    root: Path,
    page_path: Path,
    output: Path,
    by_destination: dict[str, str],
    fragment: str,
    href: str,
) -> str | None:
    if not fragment:
        return f"unresolved relative link {href}"
    relative = page_path.resolve().relative_to(output).as_posix()
    if relative == "index.html":
        if fragment not in html_ids(page_path.read_text(encoding="utf-8")):
            return f"missing anchor {href}"
        return None
    source = by_destination.get(relative)
    if source is None:
        return f"missing anchor {href}"
    if fragment not in heading_anchors((root / source).read_text(encoding="utf-8")):
        return f"missing anchor {href}"
    return None


def html_ids(text: str) -> set[str]:
    return set(re.findall(r'\sid="([^"]+)"', text))


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


def body(text: str, source: str, root: Path = ROOT, manuscript: set[str] | None = None) -> str:
    lines = text.splitlines()
    blocks: list[str] = []
    paragraph: list[str] = []
    heading_counts: dict[str, int] = {}
    index = 0

    def flush() -> None:
        if paragraph:
            blocks.append(f"<p>{inline(' '.join(paragraph), source, root, manuscript)}</p>")
            paragraph.clear()

    while index < len(lines):
        line = lines[index]
        fence = FENCE_RE.match(line)
        if fence and not (fence.group(1)[0] == "`" and "`" in fence.group(2)):
            flush()
            marker, length = fence.group(1)[0], len(fence.group(1))
            code: list[str] = []
            index += 1
            closed = False
            while index < len(lines):
                close = FENCE_RE.match(lines[index])
                if (
                    close
                    and close.group(1)[0] == marker
                    and len(close.group(1)) >= length
                    and not close.group(2).strip()
                ):
                    closed = True
                    break
                code.append(lines[index])
                index += 1
            if not closed:
                raise ValueError(f"{source}: unclosed code fence")
            blocks.append(f"<pre><code>{html.escape(chr(10).join(code))}</code></pre>")
            index += 1
            continue
        heading = HEADING_RE.match(line)
        if heading:
            flush()
            level = len(heading.group(1))
            title = heading.group(2).strip()
            anchor = heading_anchor(title)
            count = heading_counts.get(anchor, 0)
            heading_counts[anchor] = count + 1
            ident = anchor if count == 0 else f"{anchor}-{count}"
            blocks.append(
                f'<h{level} id="{html.escape(ident, quote=True)}">'
                f"{inline(title, source, root, manuscript)}</h{level}>"
            )
            index += 1
            continue
        if is_table_start(lines, index):
            flush()
            rows, index = table_rows(lines, index)
            blocks.append(render_table(rows, source, root, manuscript))
            continue
        ordered_start = _ordered_start(line)
        if re.match(r"^ {0,3}[-*+][ \t]+", line) or ordered_start is not None:
            # CommonMark: an ordered list interrupts a paragraph only when it starts at 1.
            if ordered_start is not None and paragraph and not _ordered_list_interrupts(ordered_start):
                paragraph.append(line.strip())
                index += 1
                continue
            flush()
            items, index, ordered, start = list_items(lines, index)
            if ordered:
                open_tag = "ol" if start == 1 else f'ol start="{start}"'
                close_tag = "ol"
            else:
                open_tag = close_tag = "ul"
            rendered = "".join(
                f"<li>{inline(item, source, root, manuscript)}</li>" for item in items
            )
            blocks.append(f"<{open_tag}>{rendered}</{close_tag}>")
            continue
        if line.startswith(">"):
            flush()
            quoted: list[str] = []
            while index < len(lines) and lines[index].startswith(">"):
                quoted.append(re.sub(r"^>\s?", "", lines[index]))
                index += 1
            blocks.append(
                f"<blockquote><p>{inline(' '.join(quoted), source, root, manuscript)}</p></blockquote>"
            )
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
            rows.append(split_table_cells(lines[index]))
        index += 1
    return rows, index


def split_table_cells(line: str) -> list[str]:
    """Split a row on pipes that are outside code spans and unescaped.

    A pipe is escaped only when an odd number of backslashes precedes it,
    the same rule as ``_ends_with_unescaped_pipe``. The escaping backslash
    is dropped and the pipe stays in the cell. A pipe inside an inline code
    span is part of that span, not a column boundary.
    """
    text = line.strip()
    if text.startswith("|"):
        text = text[1:]
    if _ends_with_unescaped_pipe(text):
        text = text[:-1]
    cells: list[str] = []
    buf: list[str] = []
    index = 0
    while index < len(text):
        if text[index] == "`":
            end = index + 1
            while end < len(text) and text[end] == "`":
                end += 1
            ticks = text[index:end]
            close = text.find(ticks, end)
            if close < 0:
                buf.append(text[index])
                index += 1
                continue
            content = text[end:close].replace("\\|", "|")
            buf.append(f"{ticks}{content}{ticks}")
            index = close + len(ticks)
            continue
        if text[index] == "|":
            slashes = 0
            probe = len(buf) - 1
            while probe >= 0 and buf[probe] == "\\":
                slashes += 1
                probe -= 1
            if slashes % 2 == 1:
                buf.pop()
                buf.append("|")
                index += 1
                continue
            cells.append("".join(buf).strip())
            buf = []
            index += 1
            continue
        buf.append(text[index])
        index += 1
    cells.append("".join(buf).strip())
    return cells


def _ends_with_unescaped_pipe(text: str) -> bool:
    if not text.endswith("|"):
        return False
    slashes = 0
    index = len(text) - 2
    while index >= 0 and text[index] == "\\":
        slashes += 1
        index -= 1
    return slashes % 2 == 0


def render_table(
    rows: list[list[str]],
    source: str,
    root: Path = ROOT,
    manuscript: set[str] | None = None,
) -> str:
    if not rows:
        return ""
    head = "".join(f"<th>{inline(cell, source, root, manuscript)}</th>" for cell in rows[0])
    body_rows = []
    for row in rows[1:]:
        body_rows.append(
            "<tr>" + "".join(f"<td>{inline(cell, source, root, manuscript)}</td>" for cell in row) + "</tr>"
        )
    return f"<table><thead><tr>{head}</tr></thead><tbody>{''.join(body_rows)}</tbody></table>"


def _ordered_start(line: str) -> int | None:
    """Return the marker number, or None when the line is not an ordered marker."""
    match = re.match(r"^ {0,3}(\d+)[.)][ \t]+", line)
    if match is None:
        return None
    return int(match.group(1))


def _ordered_list_interrupts(start: int) -> bool:
    """An ordered list may interrupt a paragraph only when it starts at 1."""
    return start == 1


def list_items(lines: list[str], index: int) -> tuple[list[str], int, bool, int]:
    start_number = _ordered_start(lines[index])
    ordered = start_number is not None
    start = 1 if start_number is None else start_number
    pattern = r"^ {0,3}\d+[.)][ \t]+" if ordered else r"^ {0,3}[-*+][ \t]+"
    items: list[str] = []
    while index < len(lines):
        if not re.match(pattern, lines[index]):
            if lines[index].strip():
                break
            nxt = _next_nonblank(lines, index + 1)
            if nxt is None or not re.match(pattern, lines[nxt]):
                break
            index = nxt
            continue
        parts = [re.sub(pattern, "", lines[index]).strip()]
        index += 1
        while index < len(lines):
            if _is_list_continuation(lines[index]):
                parts.append(lines[index].strip())
                index += 1
                continue
            if lines[index].strip():
                break
            nxt = _next_nonblank(lines, index + 1)
            if nxt is not None and _is_list_continuation(lines[nxt]):
                index += 1
                continue
            break
        items.append(" ".join(part for part in parts if part))
    return items, index, ordered, start


def _is_list_continuation(line: str) -> bool:
    """An indented line that stays in the current item, not a new marker."""
    if not line or line[0] not in " \t" or not line.strip():
        return False
    if re.match(r"^ {0,3}([-*+]|\d+[.)])[ \t]+", line):
        return False
    return True


def _next_nonblank(lines: list[str], index: int) -> int | None:
    while index < len(lines) and not lines[index].strip():
        index += 1
    if index >= len(lines):
        return None
    return index


def inline(text: str, source: str, root: Path = ROOT, manuscript: set[str] | None = None) -> str:
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
            pieces.append(link(match.group(5), match.group(6), source, root, manuscript))
        elif match.group(7) is not None:
            pieces.append(f"<strong>{html.escape(match.group(7))}</strong>")
        else:
            pieces.append(f"<em>{html.escape(match.group(8))}</em>")
        cursor = match.end()
    pieces.append(html.escape(text[cursor:]))
    return "".join(pieces)


def safe_url(
    url: str,
    source: str,
    root: Path = ROOT,
    manuscript: set[str] | None = None,
) -> str | None:
    """Rewrite a link for the standalone book.

    ``.md`` targets that are in ``manuscript_files()`` become sibling ``.html``
    pages. Every other repository file or directory is pinned to the rendered
    commit on GitHub (``blob`` for a file, ``tree`` for a directory).
    """
    if url.startswith(("https://", "http://", "mailto:")):
        return url
    if url.startswith("#"):
        return url
    if url.startswith(("//", "/")) or "\\" in url:
        return None
    path, _, fragment = url.partition("#")
    root = root.resolve()
    target = link_target(root, root / source, path)
    if target is None:
        return None
    suffix = f"#{fragment}" if fragment else ""
    repo_path = target.relative_to(root).as_posix()
    if target.is_file() and path.endswith(".md") and repo_path in _manuscript_paths(root, manuscript):
        rewritten = str(Path(path).with_suffix(".html")).replace("\\", "/")
        return f"{rewritten}{suffix}"
    kind = "tree" if target.is_dir() else "blob"
    if target.is_dir():
        repo_path = repo_path.rstrip("/")
    commit = rendered_commit(root)
    return f"{REPOSITORY_URL}/{kind}/{commit}/{repo_path}{suffix}"


def _manuscript_paths(root: Path, manuscript: set[str] | None) -> set[str]:
    if manuscript is not None:
        return manuscript
    return set(manuscript_files(load_manifest(root)))


_commit_cache: dict[tuple[str, str], str] = {}


def rendered_commit(root: Path) -> str:
    """Return the commit this render is pinned to.

    ``GITHUB_SHA`` wins when it is set. Otherwise ``git rev-parse HEAD`` is
    run in ``root``. Fail when neither yields a 40-character commit.
    """
    env_sha = os.environ.get("GITHUB_SHA", "").strip()
    key = (os.fspath(root.resolve()), env_sha)
    cached = _commit_cache.get(key)
    if cached is not None:
        return cached
    sha = env_sha
    if not sha:
        try:
            completed = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=root,
                check=False,
                capture_output=True,
                text=True,
            )
        except OSError as error:
            raise ValueError(
                "rendered commit is unavailable: set GITHUB_SHA or run inside a git checkout "
                f"({error})"
            ) from error
        if completed.returncode != 0 or not completed.stdout.strip():
            detail = completed.stderr.strip() or "git rev-parse HEAD failed"
            raise ValueError(
                "rendered commit is unavailable: set GITHUB_SHA or run inside a git checkout "
                f"({detail})"
            )
        sha = completed.stdout.strip()
    if not re.fullmatch(r"[0-9a-fA-F]{40}", sha):
        raise ValueError(
            "rendered commit is unavailable: GITHUB_SHA or git rev-parse HEAD "
            f"must be a 40-character commit ({sha!r})"
        )
    pinned = sha.lower()
    _commit_cache[key] = pinned
    return pinned


def link(
    label: str,
    url: str,
    source: str,
    root: Path = ROOT,
    manuscript: set[str] | None = None,
) -> str:
    target = safe_url(url, source, root, manuscript)
    if target is None:
        return html.escape(label)
    return f'<a href="{html.escape(target, quote=True)}">{html.escape(label)}</a>'


def image(alt: str, url: str) -> str:
    if not url.startswith(("https://", "http://")) and (url.startswith("/") or ".." in Path(url).parts):
        return html.escape(alt)
    return f'<img alt="{html.escape(alt, quote=True)}" src="{html.escape(url, quote=True)}">'


def main(argv: list[str]) -> int:
    if argv:
        print("usage: render_book.py", file=sys.stderr)
        return 2
    try:
        render(ROOT)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"orange book render failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
