# Blog Theme Query

统一查询 Astro 和 Zola 博客主题及其源码仓库信息，并生成CSV供筛选。

- `astro`：抓取 Astro 官网 `Blog + Free` 筛选结果、主题详情、源码入口和 GitHub stars。GitHub 主题按 stars 降序排列，非 GitHub 源保留在表尾。
- `zola`：以 `getzola/themes` 当前提交中的 `.gitmodules` 为主题清单，查询 GitHub、GitLab 和 Codeberg 等平台的仓库 stars，并按 stars 降序排列。

## 配置

安装依赖：需要 Python 3.11 或更高版本。

```bash
python -m pip install "beautifulsoup4>=4.12,<5" "requests>=2.32,<3"
```

查看帮助：

```bash
python .\query_themes.py --help
python .\query_themes.py astro --help
python .\query_themes.py zola --help
```

## 查询

使用方式：

```bash
python .\query_themes.py astro
python .\query_themes.py zola
```

Astro 查询默认输出 `astro_free_blog_themes.csv`。GitHub 主题按 stars 降序排列，并写入 `github_rank`；
非 GitHub 源仍会抓取 HTTP 状态和页面标题，但不参与 GitHub stars 排名，按 Astro 官网原始顺序追加在表尾。

Zola 查询以 `getzola/themes` 仓库当前提交中的 `.gitmodules` 为主题清单，查询各主题源码仓库的 stars。

默认输出：

- Astro：`astro_free_blog_themes.csv`
- Zola：`zola_theme_stars_YYYY-MM-DD.csv`

## 直接运行脚本

子查询脚本也可直接运行。

```powershell
python .\query_astro_theme\query_astro_themes.py
python .\query_zola_theme\query_zola_themes.py
```

## 测试和验证

```powershell
python -m unittest discover -s .\tests -v
python -m compileall -q .
```
