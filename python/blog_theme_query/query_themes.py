#!/usr/bin/env python3
"""Blog Theme Query 统一命令入口。"""

from __future__ import annotations

import sys
from collections.abc import Callable, Sequence

from query_astro_theme.query_astro_themes import main as astro_main
from query_zola_theme.query_zola_themes import main as zola_main

Command = tuple[str, Callable[[Sequence[str] | None], int]]
COMMANDS: dict[str, Command] = {
    "astro": ("抓取 Astro 官网的 Free + Blog 主题", astro_main),
    "zola": ("查询 getzola/themes 收录主题的仓库信息", zola_main),
}


def print_help() -> None:
    print(
        "用法: python .\\query_themes.py <command> [options]\n"
        "\n"
        "统一查询 Astro 和 Zola 博客主题仓库信息。\n"
        "\n"
        "commands:"
    )
    width = max(len(name) for name in COMMANDS)
    for name, (description, _) in COMMANDS.items():
        print(f"  {name:<{width}}  {description}")
    print(
        "\n"
        "查看子命令参数:\n"
        "  python .\\query_themes.py astro --help\n"
        "  python .\\query_themes.py zola --help"
    )


def main(argv: Sequence[str] | None = None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    if not arguments or arguments[0] in {"-h", "--help"}:
        print_help()
        return 0

    command_name = arguments[0]
    command = COMMANDS.get(command_name)
    if command is None:
        choices = ", ".join(COMMANDS)
        print(
            f"未知命令: {command_name!r}；可用命令: {choices}",
            file=sys.stderr,
        )
        return 2

    _, handler = command
    return handler(arguments[1:])


if __name__ == "__main__":
    raise SystemExit(main())
