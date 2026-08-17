/** 中文文档最小排版设置 by phantomfancy */
// a4 + 12pt字号，和小四号相同
#set page(paper: "a4")
#set text(size: 12pt)
// 可以通过pointless-size包设置中文字号
// #import "@preview/pointless-size:0.1.2": zh, zihao
// #set text(size: zh(-4))

// 中文语言+中文区域标识，
#set text(lang: "zh", region: "cn")

// 字体设置为：西文Libertinus Serif，中文Noto Serif SC
// 建议中文字体不要使用CJK字体，有时可能会出现字型错误，显示为日韩字形，这就不好了
// 西文字体：衬线体、无衬线体、斜体、花体、等宽字体
#let rmfamilyFont = "Libertinus Serif"
// #let rmfamilyFont = "TeX Gyre Termes"
#let sffamilyFont = "Noto Sans SC"
#let twfamilyFont = "Cascadia Mono"
// 中文字体：宋体、黑体、仿宋，楷体
#let zhSongFont = "Noto Serif SC"
#let zhHeiFont = "Noto Sans SC"
#let zhFangFont = "FangSong"
#let zhKaiFont = "KaiTi"

// 正文字体
#set text(
  lang: "zh",
  region: "cn",
  font: ((name: rmfamilyFont, covers: "latin-in-cjk"), zhSongFont),
)
// 代码块字体
#show raw: set text(
  font: ((name: twfamilyFont, covers: "latin-in-cjk"), zhHeiFont),
)

// 提供中文排版的加点函数，可以替换加粗或者倾斜效果
#let 加点(content) = {
  show regex("\p{Han}|\p{Hiragana}|\p{Katakana}|\p{Hangul}"): it => [#it#box(place(
      circle(fill: black, width: 0.2em),
      dx: -0.55em,
      dy: 0.2em,
    ))]
  content
}

// 中文排版的加粗强调样式，默认使用fontWeight 700字形
// alt1：用黑体表示
#show strong: it => {
  show regex("\p{Han}|\p{Hiragana}|\p{Katakana}|\p{Hangul}"): set text(font: zhHeiFont)
  it
}
// alt2：用加点替代
// #show strong: it => {
//   show regex("\p{Han}|\p{Hiragana}|\p{Katakana}|\p{Hangul}"): it => [#it#box(place(
//       circle(fill: black, width: 0.2em),
//       dx: -0.55em,
//       dy: 0.2em,
//     ))]
//   it
// }

// 中文排版的倾斜强调，默认使用几何变换倾斜字形
// alt1：用楷体表示
#show emph: it => {
  show regex("\p{Han}|\p{Hiragana}|\p{Katakana}|\p{Hangul}"): set text(font: zhKaiFont)
  it
}
// alt2：用加点替代
// #show emph: it => {
//   show regex("\p{Han}|\p{Hiragana}|\p{Katakana}|\p{Hangul}"): it => [#it#box(place(
//       circle(fill: black, width: 0.2em),
//       dx: -0.55em,
//       dy: 0.2em,
//     ))]
//   it
// }

// 段前缩进2字符
#set par(first-line-indent: (amount: 2em, all: true))

//智能引号
#show smartquote: set text(font: rmfamilyFont)
