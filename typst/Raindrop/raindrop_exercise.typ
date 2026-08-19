/** Settings */
#set page(paper: "a4")
#set text(size: 12pt)
#set text(lang: "zh", font: ((name: "Libertinus Serif", covers: "latin-in-cjk"), "Noto Serif SC"))

= Part.1 模式 习题

== 1. 标记模式

+ 欲穷千里目 \  更上一层楼
+ \*
+ ``` ` ```
+ ```` ``` ````
+ 你可以在Typst 内通过插件```typc plugin("typst.wasm") ```调用Typst 编译器。
+ \
  约法五章。
  1. 其一。
  + 其二。

  前两条不算。

  3. 其三。
  + 其四。
  + 其五。

== 2. 脚本模式

+ #underline(offset: -0.4em, evade: false)[吾輩は猫である]
+ #hide[I'm the flag]
+ 走#text(size: 1.5em)[走#text(size: 1.5em)[走#text(size: 1.5em)[走]]]
+ 走#text(size: 1.5em)[走#text(size: 1.5em)[走]]走#text(size: 1.5em)[走#text(size: 1.5em)[走]]
+
  #set text(size: 1.5em)
  #set text(size: 1.5em)
  走
  #set text(size: 0.67em)
  走
  #set text(size: 0.67em)
  走

#show heading.where(level: 1): head => {
  if head.body.func() == text and head.body.text.starts-with("Part") {
    pagebreak()
  }
  head
}

= Part.2 脚本 习题

== 1. 基本类型

+ #calc.pow(2, 32)
+
  #let uno = "一";
  #let yu = "渔"
  #let jiang = "江"
  #(uno)帆#(uno)桨#(uno)#(yu)舟，#(uno)个#(yu)翁#(uno)钓钩。
  #(uno)俯#(uno)仰#(uno)场笑，#(uno)#(jiang)明月#(uno)#(jiang)秋。
+
  #let fib(n) = if n == 0 {
    0
  } else if n == 1 {
    1
  } else if n > 0 {
    fib(n - 1) + fib(n - 2)
  }

  #fib(75)
+
#let matrix-fmt(..matrix) = {
  let rows = matrix.pos()
  table(
    columns: rows.first().len(),
    ..rows.flatten().map(x => [#x]),
  )
}

#let mat = ((1, 2, 3), (4, 5, 6), (7, 8, 9))
#matrix-fmt(mat)
#table(
  columns: 3,
  [1], [2], [3],
  [4], [5], [6],
  [7], [8], [9],
)