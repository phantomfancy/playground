#!/usr/bin/env python3
"""查询 getzola/themes 收录主题的仓库信息并按 stars 降序导出。"""

from __future__ import annotations

import argparse
import configparser
import csv
import io
import re
import sys
import threading
from collections.abc import Sequence
from concurrent.futures import ThreadPoolExecutor
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from functools import partial
from pathlib import Path
from typing import Any
from urllib.parse import quote, urlparse

import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

THEMES_REPOSITORY = "getzola/themes"
THEMES_REPOSITORY_URL = f"https://github.com/{THEMES_REPOSITORY}"
USER_AGENT = "blog-theme-query/1.0"
CSV_FIELDS = [
    "rank",
    "theme",
    "path",
    "stars",
    "host",
    "repository_url",
    "query_status",
    "source_commit",
    "queried_at_utc",
]

_thread_local = threading.local()


class GitHubHtmlError(RuntimeError):
    """GitHub HTML 页面缺少必要信息。"""


@dataclass
class ThemeRow:
    theme: str
    path: str
    host: str
    repository_url: str
    source_commit: str
    queried_at_utc: str
    rank: int = 0
    stars: int | None = None
    query_status: str = "ok"


def session() -> requests.Session:
    # 每个工作线程复用自己的连接池，避免在线程间共享 Session 状态。
    if cached := getattr(_thread_local, "session", None):
        return cached
    # 只重试幂等 GET 和常见瞬时错误，业务类 4xx 直接返回。
    retry = Retry(
        total=3,
        backoff_factor=0.5,
        status_forcelist=(429, 500, 502, 503, 504),
        allowed_methods={"GET"},
        respect_retry_after_header=True,
    )
    cached = requests.Session()
    cached.headers["User-Agent"] = USER_AGENT
    cached.mount("https://", HTTPAdapter(max_retries=retry, pool_maxsize=8))
    _thread_local.session = cached
    return cached


def fetch(url: str, timeout: float) -> requests.Response:
    response = session().get(url, timeout=timeout)
    response.raise_for_status()
    return response


def fetch_json(url: str, timeout: float) -> Any:
    return fetch(url, timeout).json()


def get_host_name(url: str) -> str:
    if match := re.match(r"^git@(?P<host>[^:]+):", url):
        return match.group("host").casefold()
    return (urlparse(url).hostname or "unknown").casefold()


def get_repository_path(url: str) -> str | None:
    # .gitmodules 中同时存在 SSH 与 HTTPS 地址，统一提取 owner/repository。
    if match := re.match(r"^git@[^:]+:(?P<path>.+)$", url):
        path = match.group("path")
    else:
        parsed = urlparse(url)
        if parsed.scheme not in {"http", "https"} or not parsed.netloc:
            return None
        path = parsed.path.lstrip("/")
    return re.sub(r"\.git$", "", path).rstrip("/") or None


def match_int(pattern: str, text: str) -> int | None:
    return int(match.group(1)) if (match := re.search(pattern, text)) else None


def load_submodules(timeout: float) -> tuple[list[tuple[str, str, str]], str]:
    page = fetch(THEMES_REPOSITORY_URL, timeout).text
    # 用页面公布的提交固定清单版本，避免抓取期间默认分支发生变化。
    if not (commit := re.search(r'"currentOid":"([0-9a-f]{40})"', page)):
        raise GitHubHtmlError("无法从 getzola/themes 页面解析当前提交")
    source_commit = commit.group(1)
    modules_url = (
        f"https://raw.githubusercontent.com/{THEMES_REPOSITORY}/"
        f"{source_commit}/.gitmodules"
    )
    modules_text = fetch(modules_url, timeout).content.decode("utf-8")

    # .gitmodules 使用 INI 语法；关闭插值以原样读取 URL。
    parser = configparser.RawConfigParser(interpolation=None, strict=True)
    parser.read_file(io.StringIO(modules_text))
    modules: list[tuple[str, str, str]] = []
    for section in parser.sections():
        # 仅接收标准 submodule 段，忽略可能存在的其他配置段。
        if not (match := re.fullmatch(r'submodule "([^"]+)"', section)):
            continue
        try:
            modules.append(
                (
                    match.group(1),
                    parser.get(section, "path").strip(),
                    parser.get(section, "url").strip(),
                )
            )
        except configparser.Error as exc:
            raise RuntimeError(f"不完整的 submodule 配置: {section}") from exc
    return modules, source_commit


def empty_row(
    theme: str,
    path: str,
    repository_url: str,
    source_commit: str,
    queried_at_utc: str,
) -> ThemeRow:
    return ThemeRow(
        theme,
        path,
        get_host_name(repository_url),
        repository_url,
        source_commit,
        queried_at_utc,
    )


def row_from_module(
    module: tuple[str, str, str], source_commit: str, queried_at_utc: str
) -> ThemeRow:
    return empty_row(*module, source_commit, queried_at_utc)


def apply_github_html(row: ThemeRow, html: str) -> None:
    # GitHub 将 stars 写在页面内嵌数据中，无需调用 API。
    stars = match_int(r'"stargazerCount":(\d+)', html)
    if stars is None:
        raise GitHubHtmlError("GitHub 页面中未找到 stars")
    row.stars = stars


def query_github(row: ThemeRow, repository_path: str, timeout: float) -> None:
    apply_github_html(row, fetch(f"https://github.com/{repository_path}", timeout).text)


def query_gitlab(row: ThemeRow, repository_path: str, timeout: float) -> None:
    # GitLab 与旧的 42l 实例使用相同项目 API，只读取 stars。
    origin = "https://gitlab.com" if row.host == "gitlab.com" else "https://git.42l.fr"
    data = fetch_json(
        f"{origin}/api/v4/projects/{quote(repository_path, safe='')}", timeout
    )
    row.stars = int(data.get("star_count") or 0)


def query_codeberg(row: ThemeRow, repository_path: str, timeout: float) -> None:
    data = fetch_json(f"https://codeberg.org/api/v1/repos/{repository_path}", timeout)
    row.stars = int(data.get("stars_count") or 0)


def query_theme(
    module: tuple[str, str, str],
    source_commit: str,
    queried_at_utc: str,
    timeout: float,
) -> ThemeRow:
    row = row_from_module(module, source_commit, queried_at_utc)
    repository_path = get_repository_path(row.repository_url)
    if not repository_path:
        row.query_status = "error:InvalidRepositoryUrl"
        return row
    try:
        if row.host == "github.com":
            query_github(row, repository_path, timeout)
        elif row.host in {"gitlab.com", "git.42l.fr"}:
            query_gitlab(row, repository_path, timeout)
        elif row.host == "codeberg.org":
            query_codeberg(row, repository_path, timeout)
        elif row.host == "git.sr.ht":
            # SourceHut 没有 stars 指标，保留条目但不伪造数值。
            row.query_status = "not_supported:no_star_metric"
        else:
            row.query_status = "not_supported:no_known_repository_api"
    except GitHubHtmlError:
        row.query_status = "error:GitHubHtmlError"
    except requests.HTTPError:
        row.query_status = "error:HTTPError"
    except requests.RequestException:
        row.query_status = "error:RequestException"
    except (KeyError, TypeError, ValueError):
        row.query_status = "error:MetadataParseError"
    return row


def query_all_themes(
    modules: list[tuple[str, str, str]],
    source_commit: str,
    queried_at_utc: str,
    workers: int,
    timeout: float,
) -> list[ThemeRow]:
    # 每个仓库独立查询；executor.map 保持主题清单的输入顺序。
    with ThreadPoolExecutor(max_workers=max(1, workers)) as executor:
        rows = list(
            executor.map(
                partial(
                    query_theme,
                    source_commit=source_commit,
                    queried_at_utc=queried_at_utc,
                    timeout=timeout,
                ),
                modules,
            )
        )
    print(
        f"仓库信息: {len(rows)}/{len(modules)}；并发数: {max(1, workers)}",
        file=sys.stderr,
    )
    return rows


def sort_and_rank(rows: list[ThemeRow]) -> list[ThemeRow]:
    # stars 不可用的仓库保留在结果末尾，再用主题名稳定排序。
    rows.sort(
        key=lambda row: (
            row.stars is None,
            -(row.stars or 0),
            row.theme.casefold(),
        )
    )
    for rank, row in enumerate(rows, 1):
        row.rank = rank
    return rows


def write_csv(rows: list[ThemeRow], output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    # newline="" 交给 csv 模块统一输出显式 CRLF，避免产生空行。
    with output.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, CSV_FIELDS, lineterminator="\r\n")
        writer.writeheader()
        for row in rows:
            writer.writerow(
                {
                    key: "" if value is None else value
                    for key, value in asdict(row).items()
                }
            )


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    today = datetime.now(UTC).date().isoformat()
    parser = argparse.ArgumentParser(
        description="查询 getzola/themes 主题仓库信息，按 stars 降序导出 CSV。"
    )
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=Path(f"zola_theme_stars_{today}.csv"),
        help=f"CSV 输出路径（默认: zola_theme_stars_{today}.csv）",
    )
    parser.add_argument("--workers", type=int, default=8, help="并发查询数（默认: 8）")
    parser.add_argument(
        "--timeout",
        type=float,
        default=30.0,
        help="HTTP 请求超时秒数（默认: 30）",
    )
    args = parser.parse_args(argv)
    if args.workers < 1 or args.timeout <= 0:
        parser.error("--workers 和 --timeout 必须大于 0")
    return args


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    queried_at = datetime.now(UTC).replace(microsecond=0).strftime("%Y-%m-%dT%H:%M:%SZ")
    try:
        modules, source_commit = load_submodules(args.timeout)
    except (
        GitHubHtmlError,
        RuntimeError,
        UnicodeDecodeError,
        requests.RequestException,
    ) as exc:
        print(f"无法读取主题清单: {exc}", file=sys.stderr)
        return 1

    print(
        f"主题清单: {len(modules)}；source_commit: {source_commit}",
        file=sys.stderr,
    )
    rows = sort_and_rank(
        query_all_themes(modules, source_commit, queried_at, args.workers, args.timeout)
    )
    write_csv(rows, args.output)

    ok_count = sum(row.query_status == "ok" for row in rows)
    stars_count = sum(row.stars is not None for row in rows)
    print(
        (
            f"已写入 {args.output.resolve()}：{len(rows)} 行；"
            f"stars 可用 {stars_count}；查询成功 {ok_count}；"
            f"不可用 {len(rows) - ok_count}"
        ),
        file=sys.stderr,
    )
    for row in rows:
        if row.query_status != "ok":
            print(f"  {row.theme}: {row.query_status} ({row.host})", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
