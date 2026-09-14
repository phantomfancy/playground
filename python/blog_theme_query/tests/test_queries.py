from __future__ import annotations

import unittest
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path
from tempfile import TemporaryDirectory

from query_astro_theme.query_astro_themes import (
    CSV_FIELDS as ASTRO_CSV_FIELDS,
)
from query_astro_theme.query_astro_themes import (
    Theme,
    parse_github_repository,
    sort_themes,
)
from query_astro_theme.query_astro_themes import (
    write_csv as write_astro_csv,
)
from query_themes import main as unified_main
from query_zola_theme.query_zola_themes import (
    CSV_FIELDS as ZOLA_CSV_FIELDS,
)
from query_zola_theme.query_zola_themes import (
    apply_github_html,
    empty_row,
    get_host_name,
    get_repository_path,
    sort_and_rank,
)
from query_zola_theme.query_zola_themes import (
    write_csv as write_zola_csv,
)


class UnifiedCliTests(unittest.TestCase):
    def test_help_succeeds(self) -> None:
        with redirect_stdout(StringIO()):
            self.assertEqual(unified_main(["--help"]), 0)

    def test_unknown_command_fails(self) -> None:
        with redirect_stderr(StringIO()):
            self.assertEqual(unified_main(["unknown"]), 2)


class AstroTests(unittest.TestCase):
    def test_parse_github_repository_ignores_subpath(self) -> None:
        full_name, url = parse_github_repository(
            "https://github.com/example/theme/tree/main/demo"
        )
        self.assertEqual(full_name, "example/theme")
        self.assertEqual(url, "https://github.com/example/theme")

    def test_non_github_themes_do_not_participate_in_ranking(self) -> None:
        high = self.make_theme(1, "High", "GitHub", 20)
        other = self.make_theme(2, "Other", "Codeberg", None)
        low = self.make_theme(3, "Low", "GitHub", 5)

        rows = sort_themes([low, other, high])

        self.assertEqual(
            [row.theme_name for row in rows],
            ["High", "Low", "Other"],
        )
        self.assertEqual([row.github_rank for row in rows], [1, 2, None])

    def test_csv_omits_non_core_github_metadata(self) -> None:
        self.assertTrue(
            {
                "github_forks",
                "github_watchers",
                "github_default_branch",
                "github_license",
                "github_created_at",
                "github_is_archived",
            }.isdisjoint(ASTRO_CSV_FIELDS)
        )

    @staticmethod
    def make_theme(
        order: int,
        name: str,
        platform: str,
        stars: int | None,
    ) -> Theme:
        theme = Theme(
            listing_order=order,
            theme_name=name,
            theme_description="",
            author="",
            price="Free",
            astro_theme_url=f"https://astro.build/themes/details/{name}/",
            image_url="",
        )
        theme.source_platform = platform
        theme.github_stars = stars
        return theme


class ZolaTests(unittest.TestCase):
    def test_repository_url_parsing(self) -> None:
        self.assertEqual(
            get_repository_path("git@github.com:owner/theme.git"),
            "owner/theme",
        )
        self.assertEqual(
            get_repository_path("https://codeberg.org/owner/theme.git"),
            "owner/theme",
        )
        self.assertEqual(
            get_host_name("git@github.com:owner/theme.git"),
            "github.com",
        )

    def test_github_stars_are_parsed_without_api(self) -> None:
        row = empty_row(
            "theme",
            "theme",
            "https://github.com/example/theme.git",
            "abc",
            "2026-07-30T00:00:00Z",
        )
        html = '<script>{"stargazerCount":42,"forksCount":7}</script>'

        apply_github_html(row, html)

        self.assertEqual(row.stars, 42)

    def test_csv_omits_non_core_repository_metadata(self) -> None:
        self.assertTrue(
            {
                "forks",
                "open_issues",
                "archived",
                "default_branch",
                "language",
                "license",
                "repository_updated_at",
                "last_pushed_at",
                "description",
                "api_url",
            }.isdisjoint(ZOLA_CSV_FIELDS)
        )

    def test_rows_without_stars_sort_last(self) -> None:
        queried_at = "2026-07-30T00:00:00Z"
        commit = "abc"
        high = empty_row(
            "high",
            "high",
            "https://github.com/example/high.git",
            commit,
            queried_at,
        )
        high.stars = 10
        low = empty_row(
            "low",
            "low",
            "https://github.com/example/low.git",
            commit,
            queried_at,
        )
        low.stars = 2
        unavailable = empty_row(
            "unavailable",
            "unavailable",
            "https://git.sr.ht/~example/unavailable",
            commit,
            queried_at,
        )

        rows = sort_and_rank([low, unavailable, high])

        self.assertEqual(
            [row.theme for row in rows],
            ["high", "low", "unavailable"],
        )
        self.assertEqual([row.rank for row in rows], [1, 2, 3])


class CsvEncodingTests(unittest.TestCase):
    def test_outputs_are_utf8_without_bom(self) -> None:
        astro = AstroTests.make_theme(1, "Astro", "GitHub", 1)
        zola = empty_row(
            "Zola",
            "zola",
            "https://github.com/example/zola.git",
            "abc",
            "2026-07-30T00:00:00Z",
        )
        with TemporaryDirectory() as directory:
            astro_path = Path(directory, "astro.csv")
            zola_path = Path(directory, "zola.csv")
            write_astro_csv([astro], astro_path, "2026-07-30T00:00:00Z")
            write_zola_csv([zola], zola_path)
            for path in (astro_path, zola_path):
                data = path.read_bytes()
                self.assertFalse(data.startswith(b"\xef\xbb\xbf"))
                data.decode("utf-8")
                self.assertIn(b"\r\n", data)


if __name__ == "__main__":
    unittest.main()
