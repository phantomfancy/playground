#!/usr/bin/env python3
"""抓取 Astro 官网中免费 Blog 主题及其源码仓库信息。"""

from __future__ import annotations

import argparse
import csv
import re
import sys
import threading
from collections.abc import Callable, Sequence
from concurrent.futures import ThreadPoolExecutor
from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, TypeVar
from urllib.parse import urljoin, urlparse

import requests
from bs4 import BeautifulSoup
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

BASE_URL = "https://astro.build"
PAGE_URL = (
    "https://astro.build/themes/{page}/?search=&categories%5B%5D=blog&price%5B%5D=free"
)
USER_AGENT = "blog-theme-query/1.0 (+https://astro.build/themes/)"
CSV_FIELDS = [
    "github_rank",
    "listing_order",
    "theme_name",
    "theme_description",
    "author",
    "price",
    "astro_theme_url",
    "image_url",
    "source_url",
    "live_demo_url",
    "source_platform",
    "repository_full_name",
    "github_repository_url",
    "github_stars",
    "source_http_status",
    "source_page_title",
    "scraped_at_utc",
    "scrape_error",
]
GITHUB_METADATA_FIELDS = [
    "repository_full_name",
    "github_repository_url",
    "github_stars",
    "source_http_status",
    "source_page_title",
    "scrape_error",
]

T = TypeVar("T")
R = TypeVar("R")
_thread_local = threading.local()


@dataclass
class Theme:
    listing_order: int
    theme_name: str
    theme_description: str
    author: str
    price: str
    astro_theme_url: str
    image_url: str
    source_url: str = ""
    live_demo_url: str = ""
    source_platform: str = ""
    repository_full_name: str = ""
    github_repository_url: str = ""
    github_stars: int | None = None
    source_http_status: int | None = None
    source_page_title: str = ""
    scrape_error: str = ""
    github_rank: int | None = field(default=None, init=False)


def session() -> requests.Session:
    # 每个工作线程复用自己的连接池，避免在线程间共享 Session 状态。
    if cached := getattr(_thread_local, "session", None):
        return cached
    # 只重试幂等 GET 和常见瞬时错误，业务类 4xx 直接返回。
    retry = Retry(
        total=4,
        backoff_factor=0.8,
        status_forcelist=(429, 500, 502, 503, 504),
        allowed_methods={"GET"},
        respect_retry_after_header=True,
    )
    cached = requests.Session()
    cached.headers["User-Agent"] = USER_AGENT
    cached.mount("https://", HTTPAdapter(max_retries=retry, pool_maxsize=16))
    _thread_local.session = cached
    return cached


def fetch_soup(url: str, timeout: float) -> tuple[requests.Response, BeautifulSoup]:
    response = session().get(url, timeout=timeout)
    response.raise_for_status()
    return response, BeautifulSoup(response.content, "html.parser")


def parallel_map(
    items: Sequence[T],
    worker: Callable[[T, float], R],
    workers: int,
    timeout: float,
) -> list[R]:
    if not items:
        return []
    # executor.map 保持输入顺序，便于后续恢复官网原始顺序。
    with ThreadPoolExecutor(max_workers=min(workers, len(items))) as executor:
        return list(executor.map(lambda item: worker(item, timeout), items))


def clean_text(node: Any) -> str:
    if node is None:
        return ""
    return re.sub(r"\s+", " ", " ".join(node.stripped_strings)).strip()


def absolute_url(value: str, base_url: str = BASE_URL) -> str:
    return urljoin(base_url, value) if value else ""


def discover_page_count(soup: BeautifulSoup) -> int:
    # 分页链接包含页码；没有分页控件时按单页处理。
    pages = [
        int(match.group(1))
        for anchor in soup.find_all("a", href=True)
        if (match := re.match(r"^/themes/(\d+)/", anchor["href"]))
    ]
    return max(pages, default=1)


def parse_theme_cards(soup: BeautifulSoup) -> list[Theme]:
    themes: list[Theme] = []
    # 详情页链接是主题卡片最稳定的入口，标题用于过滤无效容器。
    for card in soup.select('a[href^="/themes/details/"]'):
        title = card.find(["h2", "h3"])
        if title is None:
            continue
        footer = card.find("footer")
        image = card.find("img", src=True)
        themes.append(
            Theme(
                listing_order=0,
                theme_name=clean_text(title),
                theme_description=clean_text(card.find("p")),
                author=clean_text(footer.find("span") if footer else None),
                price=clean_text(footer.find("p") if footer else None),
                astro_theme_url=absolute_url(card.get("href", "")),
                image_url=absolute_url(image.get("src", "") if image else ""),
            )
        )
    return themes


def find_action_url(soup: BeautifulSoup, label: str) -> str:
    # 详情页没有固定按钮 ID，因此按可见标签定位源码和演示链接。
    expected = label.casefold()
    return next(
        (
            absolute_url(anchor["href"])
            for anchor in soup.find_all("a", href=True)
            if clean_text(anchor).casefold() == expected
        ),
        "",
    )


def classify_platform(url: str) -> str:
    host = (urlparse(url).hostname or "").casefold().removeprefix("www.")
    if host == "github.com":
        return "GitHub"
    if host == "gitlab.com" or host.endswith(".gitlab.com"):
        return "GitLab"
    if host == "codeberg.org":
        return "Codeberg"
    return host or "Unknown"


def parse_theme_detail(theme: Theme, timeout: float) -> Theme:
    try:
        _, soup = fetch_soup(theme.astro_theme_url, timeout)
        theme.source_url = find_action_url(soup, "Get started")
        theme.live_demo_url = find_action_url(soup, "Live demo")
        theme.source_platform = classify_platform(theme.source_url)
    except requests.RequestException as exc:
        add_error(theme, f"Astro 详情页请求失败: {exc}")
    return theme


def parse_github_repository(url: str) -> tuple[str, str]:
    parsed = urlparse(url)
    if (parsed.hostname or "").casefold().removeprefix("www.") != "github.com":
        return "", ""
    parts = [part for part in parsed.path.split("/") if part]
    if len(parts) < 2:
        return "", ""
    # 只保留 owner/repository，忽略 /tree/... 等仓库内子路径。
    full_name = f"{parts[0]}/{parts[1].removesuffix('.git')}"
    return full_name, f"https://github.com/{full_name}"


def match_int(pattern: str, text: str) -> int | None:
    return int(match.group(1)) if (match := re.search(pattern, text)) else None


def scrape_github(theme: Theme, timeout: float) -> Theme:
    full_name, repository_url = parse_github_repository(theme.source_url)
    theme.repository_full_name = full_name
    theme.github_repository_url = repository_url
    if not repository_url:
        add_error(theme, "无法从 GitHub URL 解析 owner/repository")
        return theme

    try:
        response, soup = fetch_soup(repository_url, timeout)
        text = response.text
        theme.source_http_status = response.status_code
        theme.source_page_title = clean_text(soup.title)
        # GitHub 将 stars 写在页面内嵌数据中，无需调用 API。
        theme.github_stars = match_int(r'"stargazerCount":(\d+)', text)
        if theme.github_stars is None:
            add_error(theme, "GitHub 页面中未找到 stars")
    except requests.RequestException as exc:
        if isinstance(exc, requests.HTTPError) and exc.response is not None:
            theme.source_http_status = exc.response.status_code
        add_error(theme, f"GitHub 仓库请求失败: {exc}")
    return theme


def scrape_github_group(themes: list[Theme], timeout: float) -> list[Theme]:
    source = scrape_github(themes[0], timeout)
    # 同仓库的其他主题共享首次抓取到的仓库结果和错误状态。
    for theme in themes[1:]:
        for attribute in GITHUB_METADATA_FIELDS:
            setattr(theme, attribute, getattr(source, attribute))
    return themes


def scrape_non_github(theme: Theme, timeout: float) -> Theme:
    if not theme.source_url:
        add_error(theme, "详情页无 Get started 链接")
        return theme
    try:
        # 非 GitHub 源不参与 stars 排名，只记录页面是否可访问及标题。
        response, soup = fetch_soup(theme.source_url, timeout)
        theme.source_http_status = response.status_code
        theme.source_page_title = clean_text(soup.title)
    except requests.RequestException as exc:
        if isinstance(exc, requests.HTTPError) and exc.response is not None:
            theme.source_http_status = exc.response.status_code
        add_error(theme, f"非 GitHub 源页面请求失败: {exc}")
    return theme


def add_error(theme: Theme, message: str) -> None:
    theme.scrape_error = (
        f"{theme.scrape_error}; {message}" if theme.scrape_error else message
    )


def sort_themes(themes: list[Theme]) -> list[Theme]:
    github = [theme for theme in themes if theme.source_platform == "GitHub"]
    others = [theme for theme in themes if theme.source_platform != "GitHub"]
    # stars 缺失的 GitHub 主题排在已有 stars 的主题之后。
    github.sort(
        key=lambda theme: (
            -(theme.github_stars if theme.github_stars is not None else -1),
            theme.theme_name.casefold(),
            theme.listing_order,
        )
    )
    others.sort(key=lambda theme: theme.listing_order)
    for rank, theme in enumerate(github, 1):
        theme.github_rank = rank
    return github + others


def write_csv(themes: list[Theme], output: Path, scraped_at: str) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, CSV_FIELDS, lineterminator="\r\n")
        writer.writeheader()
        for theme in themes:
            row = asdict(theme)
            row["scraped_at_utc"] = scraped_at
            writer.writerow(
                {key: "" if value is None else value for key, value in row.items()}
            )


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="抓取 Astro 官网 Free + Blog 主题，并按 GitHub stars 降序导出。"
    )
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=Path("astro_free_blog_themes.csv"),
        help="CSV 输出路径（默认: astro_free_blog_themes.csv）",
    )
    parser.add_argument("--workers", type=int, default=8, help="并发请求数（默认: 8）")
    parser.add_argument(
        "--timeout", type=float, default=30.0, help="HTTP 请求超时秒数（默认: 30）"
    )
    parser.add_argument(
        "--max-pages",
        type=int,
        help="只抓取前 N 页，供调试使用；默认抓取全部页",
    )
    args = parser.parse_args(argv)
    if args.workers < 1 or args.timeout <= 0:
        parser.error("--workers 和 --timeout 必须大于 0")
    if args.max_pages is not None and args.max_pages < 1:
        parser.error("--max-pages 必须大于 0")
    return args


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    scraped_at = datetime.now(UTC).replace(microsecond=0).isoformat()
    try:
        _, first_soup = fetch_soup(PAGE_URL.format(page=1), args.timeout)
        page_count = discover_page_count(first_soup)
        if args.max_pages:
            page_count = min(page_count, args.max_pages)
        page_urls = [PAGE_URL.format(page=page) for page in range(2, page_count + 1)]
        pages = [first_soup] + [
            soup
            for _, soup in parallel_map(
                page_urls, fetch_soup, args.workers, args.timeout
            )
        ]
    except requests.RequestException as exc:
        print(f"筛选页请求失败: {exc}", file=sys.stderr)
        return 1

    unique: dict[str, Theme] = {}
    for soup in pages:
        for theme in parse_theme_cards(soup):
            unique.setdefault(theme.astro_theme_url, theme)
    themes = list(unique.values())
    for order, theme in enumerate(themes, 1):
        theme.listing_order = order
    print(f"筛选结果: {page_count} 页，{len(themes)} 个主题", file=sys.stderr)

    parallel_map(themes, parse_theme_detail, args.workers, args.timeout)
    github = [theme for theme in themes if theme.source_platform == "GitHub"]
    others = [theme for theme in themes if theme.source_platform != "GitHub"]
    # 多个 Astro 条目可能指向同一仓库，只抓取一次并复用结果。
    groups: dict[str, list[Theme]] = {}
    for theme in github:
        _, repository_url = parse_github_repository(theme.source_url)
        groups.setdefault((repository_url or theme.source_url).casefold(), []).append(
            theme
        )
    print(
        (
            f"GitHub 主题: {len(github)}；GitHub 仓库: {len(groups)}；"
            f"非 GitHub 源: {len(others)}"
        ),
        file=sys.stderr,
    )
    parallel_map(list(groups.values()), scrape_github_group, args.workers, args.timeout)
    parallel_map(others, scrape_non_github, min(args.workers, 4), args.timeout)

    write_csv(sort_themes(themes), args.output, scraped_at)
    stars_count = sum(theme.github_stars is not None for theme in github)
    errors = sum(bool(theme.scrape_error) for theme in themes)
    print(
        (
            f"已写入 {args.output.resolve()}：{len(themes)} 行；"
            f"取得 stars {stars_count}/{len(github)}；错误 {errors} 行"
        ),
        file=sys.stderr,
    )
    return 0 if errors == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
