#include "raindrop_learning_3_ChinesePattern.typ"

== CJK测试

helloworld
*helloworld*
_helloworld_

你好世界
*你好世界*
_你好世界_

你好世界helloworld你好世界 \
*你好世界helloworld你好世界* \
_你好世界helloworld你好世界_ \

本当にありがとうございます \
*本当にありがとうございます* \
_本当にありがとうございます_ \

== 代码块测试

单行代码块：`code代码块0oijklxyz`，多行代码块：

```c
//这是一个C语言的“你好世界”代码示例。
#include<stdio.h>
int main()
{
    printf("hello world!\n");
    return 0;
}
```

== 乱数假文测试

Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Mauris porta quam dui, a sagittis ex volutpat et. Suspendisse lacinia dignissim massa et hendrerit.
Sed pharetra sapien consectetur, varius dolor sit amet, interdum felis.
Nulla vel orci et ligula suscipit placerat sed quis sem. Nam vitae faucibus enim, sed facilisis nisi.
Duis suscipit semper diam, ac gravida nulla. Nunc ultrices ipsum et lectus pharetra condimentum.
Etiam pellentesque eget mi convallis consequat.
Nulla ut arcu ex. Curabitur nisi nisi, fringilla nec lacus ut, egestas malesuada ex.
Ut quis nibh at diam ultrices accumsan id non risus. Nunc finibus eros turpis,
ut dignissim quam rhoncus vitae. Sed dictum sem vel orci viverra, id sollicitudin neque ultrices.
Phasellus auctor porta nulla, at condimentum mauris ultricies at.

畅、直观。当我们谈论用户体验时，我们实际上是在谈论如何减少用户的认知负担。
在一个快节奏的数字世界里，清晰、简洁的设计语言往往比复杂的视觉效果更有力。
色彩的使用、排版的节奏、动效的反馈，每一个细节都在向用户传递信息。
好的设计是看不见的，它自然而然地引导用户完成任务，而不会成为阻碍。
从某种意义上说，设计师就是信息世界的建筑师，构建着人们与数字内容交互的桥梁。
保持一致性，遵循标准，但也不要害怕打破常规。创新往往来自于对细节的极致追求和对人性的深刻理解。

== 语段测试

=== 1. merge 前先做“干净性检查”

不要在工作区有未提交改动时 merge。Git 文档明确说，如果 merge 开始前已有未提交改动，后续 `git merge --abort` 在某些情况下可能无法完整恢复 merge 前状态。([Git][1])

每次 merge 前先执行：

```bash
git status
```

理想状态是：

```text
nothing to commit, working tree clean
```

如果有未提交改动，先二选一：

```bash
git add .
git commit -m "WIP before merge"
```

或者：

```bash
git stash push -u -m "before merge"
```

=== 2. 永远确认“当前分支”和“目标 SVN 分支”

`git merge other` 的含义是：*把 other 合进当前分支*。Git 文档也说，merge 会把指定分支自分叉以来的改动合入当前分支。([Git][1])

所以合并前固定执行：

```bash
git branch --show-current
git svn info
```

特别是 git-svn 仓库，重点看：

```text
URL:
```

确保它是你准备 `dcommit` 的 SVN 分支。

例如你要把 `mtm-og` 合进 `mtm-rebuild`，应该是：

```bash
git switch mtm-rebuild
git svn rebase
git svn info

git merge --no-ff mtm-og
```

不要反过来。

=== 3. merge 前先预览会合入哪些 commit

合并前看清楚对方分支相对当前分支多了什么：

```bash
git log --oneline --graph --decorate HEAD..mtm-og
```

看文件级别变化：

```bash
git diff --stat HEAD...mtm-og
git diff --name-status HEAD...mtm-og
```

注意这里用的是三个点：

```bash
HEAD...mtm-og
```

它表示从共同祖先到 `mtm-og` 的变化，适合看“对方分支准备带进来什么”。

如果你看到一堆不该出现的提交，不要 merge，先整理分支。

=== 4. 先做测试合并，不急着提交

推荐用：

```bash
git merge --no-commit --no-ff mtm-og
```

`--no-commit` 会让 Git 在真正创建 merge commit 前停下来，给你检查结果的机会；Git 文档也说明这个选项会在创建 merge commit 前停止。([Git][1])

检查：

```bash
git status
git diff --stat
git diff --cached --stat
```

如果不满意，直接撤销：

```bash
git merge --abort
```

如果满意：

```bash
git commit
```

或者：

```bash
git merge --continue
```

=== 5. 开启更清楚的冲突显示：`zdiff3`

默认冲突标记只显示 “ours” 和 “theirs”，不显示共同祖先。Git 文档说明可以把 `merge.conflictStyle` 设为 `diff3` 或 `zdiff3`，这样冲突区域会包含 base 版本，能帮助判断双方各自改了什么。([Git][2])

建议：

```bash
git config --global merge.conflictStyle zdiff3
```

如果你的编辑器对 `zdiff3` 支持不好，退而用：

```bash
git config --global merge.conflictStyle diff3
```

冲突中含义大致是：

```text
<<<<<<< 当前分支
你的版本
||||||| 共同祖先
原始版本
=======
对方分支版本
>>>>>>> 对方分支
```

这比只看两边版本可靠很多。

=== 6. 开启 `rerere`，重复冲突只解决一次

长期分支反复 merge 时，同一个冲突可能出现多次。`git rerere` 会记录一次手动解决结果，后面遇到相同冲突时自动复用；官方文档说它会记录 conflicted automerge result 和对应的 hand resolve result，并在之后套用。([Git][3])

建议全局开启：

```bash
git config --global rerere.enabled true
git config --global rerere.autoupdate true
```

冲突解决后正常：

```bash
git add .
git merge --continue
```

以后遇到相同冲突，Git 可能会自动帮你解决并暂存。仍然要检查：

```bash
git diff --cached
```

`rerere` 是减少重复劳动，不是替代审查。

=== 7. 使用 mergetool，而不是纯手工猜

Git 官方的 `git mergetool` 就是用于在 merge 后调用外部工具解决冲突；它会为工具提供 `BASE`、`LOCAL`、`REMOTE`、`MERGED` 等文件。([Git][4])

查看可用工具：

```bash
git mergetool --tool-help
```

例如配置 VS Code：

```bash
git config --global merge.tool vscode
git config --global mergetool.vscode.cmd 'code --wait "$MERGED"'
```

使用：

```bash
git mergetool
```

如果你用 Windows，也可以用 KDiff3、Meld、TortoiseMerge、Beyond Compare 等。

=== 8. 大规模移动/重命名要拆提交

避免把这几类操作混在一个 commit 里：

```text
移动文件
重命名文件
格式化代码
大规模重写
功能修改
merge
```

更稳的顺序是：

```text
commit 1：只 git mv，不改内容
commit 2：只格式化，不改逻辑
commit 3：功能修改
commit 4：merge
```

这样 Git 更容易识别 rename，也更容易解决冲突。你前面看的 Stack Overflow 那个问题，本质上就是“移动 + 大改”后 Git 无法可靠识别同一个文件。

如果已经遇到 rename 识别不佳，可以临时试：

```bash
git merge -X find-renames=20% other-branch
```

或更保守：

```bash
git merge -X find-renames=50% other-branch
```

阈值越低，越容易识别 rename，但也越容易误识别。

=== 9. 对特殊文件写 `.gitattributes`

有些文件不适合普通文本三方合并，例如：

```text
自动生成文件
工程文件
锁文件
二进制文件
版本号文件
某些 IDE 配置
```

Git 可以用 `.gitattributes` 给路径设置属性；官方文档说明 `.gitattributes` 是按 pathname 指定属性的文件，属性规则可以放在版本控制的 `.gitattributes`，也可以只放在 `$GIT_DIR/info/attributes` 给本仓库私用。([Git][5])

例如把某些文件视作二进制，避免乱合：

```gitattributes
*.uvprojx binary
*.uvoptx binary
*.sln text eol=crlf
*.vcxproj text eol=crlf
*.pbxproj text
```

对于只想保留当前分支版本的文件，可以配置 `ours` merge driver。但要谨慎，它会静默丢弃对方分支对这些文件的改动：

```gitattributes
path/to/local_config.ini merge=ours
```

配置 driver：

```bash
git config merge.ours.driver true
```

这适合本地配置、生成文件，不适合源代码。

=== 10. 处理冲突时按固定流程走

冲突后不要急着编辑。先看哪些文件冲突：

```bash
git status
git diff --name-only --diff-filter=U
```

看某个文件三方版本：

```bash
git show :1:path/to/file   # base
git show :2:path/to/file   # ours 当前分支
git show :3:path/to/file   # theirs 被合入分支
```

快速选择一边：

```bash
git checkout --ours path/to/file
git checkout --theirs path/to/file
git add path/to/file
```

注意：

```text
ours   = 当前分支
theirs = 被 merge 进来的分支
```

解决完：

```bash
git diff --check
git add .
git merge --continue
```

`git diff --check` 可以帮你发现冲突标记残留、空白问题等。

=== 11. merge 后立即验证，不要直接 dcommit

merge 成功不代表代码正确。至少做：

```bash
git status
git log --graph --oneline --decorate -20
git diff --check
```

然后构建：

```bash
# 例如
xmake build
# 或
cmake --build build
# 或你的实际构建命令
```

再看将要提交到 SVN 的内容：

```bash
git svn dcommit --dry-run
```

确认没问题再：

```bash
git svn dcommit
```

`git-svn` 官方文档把它描述为 Subversion 和 Git 之间的 changeset 桥接工具；它可以从 SVN fetch，也可以用 dcommit 把 Git changesets 提交回 SVN。([Git][6])

=== 12. git-svn 中不要让待 dcommit 历史太复杂

纯 Git 项目中 merge commit 很正常；但 git-svn 不是完整的 Git DAG <-> SVN mergeinfo 转换器。官方文档也把 `git svn` 定位为 changeset conduit，而不是完整替代 SVN merge 模型的工具。([Git][6])

实用策略：

```text
本地可以 merge
提交 SVN 前尽量让目标分支清楚、可检查、不要包含一堆临时 merge
```

如果 merge 结果很复杂，但你只想把最终结果交给 SVN，可以压成一个普通提交：

```bash
git switch mtm-rebuild
git svn rebase

# 假设 refs/remotes/svn/mtm-rebuild 是当前 SVN 分支
git reset --soft refs/remotes/svn/mtm-rebuild
git commit -m "Merge mtm-og changes into rebuild branch"

git svn dcommit --dry-run
git svn dcommit
```

老式 git-svn ref 可能是：

```bash
refs/remotes/mtm-rebuild
```

那就改成：

```bash
git reset --soft refs/remotes/mtm-rebuild
```

这个做法会牺牲 Git 本地 merge DAG，但对 SVN 端更稳。

=== 13. 使用临时 worktree 做危险合并

你可以在不污染当前工作区的情况下开一个临时目录测试 merge：

```bash
git worktree add ../merge-test mtm-rebuild
cd ../merge-test

git svn rebase
git merge --no-commit --no-ff mtm-og
```

如果结果很乱，直接删 worktree：

```bash
cd ../VCMToolKit
git worktree remove ../merge-test --force
```

这比在主目录里反复 abort 更安全。

=== 14. 用 `--squash` 做“拿结果，不拿分支历史”

如果你不想保留 merge commit，只想把另一个分支的最终改动拿过来：

```bash
git switch mtm-rebuild
git svn rebase

git merge --squash mtm-og
git diff --cached --stat
git commit -m "Apply mtm-og changes"
git svn dcommit
```

Git 文档说明 `--squash` 会产生类似 merge 后的工作树和 index 状态，但不记录真实 merge 信息，也不会移动 `HEAD` 或记录 `MERGE_HEAD` 让下一次提交变成 merge commit。([Git][2])

在 git-svn 中，`--squash` 往往比真实 merge commit 更容易被 SVN 端接受。

缺点是：Git 不会记录“已经 merge 过”，下次可能还要你手动判断哪些改动已包含。

=== 15. 推荐采用的固定工作流

对 git-svn 仓库，建议这样做：

```bash
# 1. 进入目标分支，也就是准备 dcommit 的 SVN 分支
git switch mtm-rebuild

# 2. 保证同步 SVN
git svn rebase

# 3. 确认目标 SVN URL
git svn info

# 4. 检查工作区干净
git status

# 5. 预览要合入的内容
git log --oneline --graph HEAD..mtm-og
git diff --stat HEAD...mtm-og
git diff --name-status HEAD...mtm-og

# 6. 测试合并，不立即提交
git merge --no-commit --no-ff mtm-og

# 7. 检查、构建、测试
git status
git diff --check

# 8. 确认结果后提交 merge
git commit

# 9. 提交 SVN 前预演
git svn dcommit --dry-run

# 10. 正式提交
git svn dcommit
```

如果第 6 步出问题：

```bash
git merge --abort
```

如果你不想保留 merge commit，用第 6 步替换为：

```bash
git merge --squash mtm-og
git commit -m "Apply mtm-og changes"
```

=== 总结

总结要点如下：

```text
merge 前必须 git status 干净
merge 前必须 git svn info 确认 SVN URL
先 git merge --no-commit --no-ff 测试
开启 merge.conflictStyle=zdiff3
开启 rerere
大规模移动、格式化、功能修改分开提交
git-svn dcommit 前先 --dry-run
复杂 merge 准备进 SVN 前可以 squash 或 soft reset 成普通提交
```

养成良好的merge习惯，能有效减少合并自己代码和别人代码的痛苦。

[1]: https://git-scm.com/docs/git-merge "Git - git-merge Documentation"
[2]: https://git-scm.com/docs/git-merge/2.38.0 "Git - git-merge Documentation"
[3]: https://git-scm.com/docs/git-rerere "Git - git-rerere Documentation"
[4]: https://git-scm.com/docs/git-mergetool "Git - git-mergetool Documentation"
[5]: https://git-scm.com/docs/gitattributes "Git - gitattributes Documentation"
[6]: https://git-scm.com/docs/git-svn "Git - git-svn Documentation"
