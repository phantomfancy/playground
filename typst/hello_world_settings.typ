//! Settings
//! 页面设置
#set page(paper: "a4")
#set text(size: 11pt)
//! 字体和样式
// 单独设置英文和中文字体
// typst的原版英文字体是"Libertinus Serif"
#set text(lang: "zh", font: ((name: "Libertinus Serif", covers: "latin-in-cjk"), "Noto Serif SC"))
// 中文排版的加粗语义用加点或黑体表示
#let 着重号(body) = {
  show regex("\p{sc=Han}"): it => {
    box(
      grid(
        columns: 1,
        rows: (auto, -0.2em),
        row-gutter: 0.2em,
        align(center, it),
        align(center, circle(fill: black, width: 0.2em)),
      )
    )
  }
  body
}
// 中文排版的加粗强调，用加点表示
#show strong: content => {
  let base-weight = text.weight
  show regex("\p{Han}|\p{Hiragana}|\p{Katakana}|\p{Hangul}"): it => [#text(weight: base-weight)[#it#box(place(
      circle(fill: black, width: 0.2em),
      dx: -0.55em,
      dy: 0.2em,
    ))]]
  content
}
// 中文排版的倾斜强调，用楷体表示
#show emph: it => {
  show regex("\p{Han}"): set text(font: "KaiTi")
  it
}
// 中文排版的倾斜强调，用加点表示(alt)
// #show emph: content => {
//   show regex("\p{Han}|\p{Hiragana}|\p{Katakana}|\p{Hangul}"): it => [#it#box(place(circle(fill: black, width: 0.2em), dx: -0.55em, dy: 0.2em))]
//   content
// }
// show regex("\p{Han}")覆盖的样式，不会应用到假名和谚文等CJK中的非汉字文字上，
// 我认为这会让覆盖后的日韩文字样式看起来很奇怪，因为其中的汉字是覆盖后的样式（例如倾斜变楷体），假名和谚文却还是默认样式
// 使用show regex("\p{Han}|\p{Hiragana}|\p{Katakana}|\p{Hangul}")，可以使得强调样式应用到所有中日韩(CJK)文字

helloworld
*helloworld*
_helloworld_
#着重号[helloworld]

你好世界
*你好世界*
_你好世界_
#着重号[你好世界]

你好世界helloworld你好世界

*你好世界helloworld你好世界*

_你好世界helloworld你好世界_

#着重号[你h好e世ll界helloworld你oo好hello世world界]


本当にありがとうございます

*本当にありがとうございます*

_本当にありがとうございます_

#着重号[本当にありがとうございます]

*Strong强壮*

