/** Settings */
#set page(paper: "a4")
#set text(size: 11pt)
#set text(lang: "zh", font: ((name: "Libertinus Serif", covers: "latin-in-cjk"), "Noto Serif SC"))

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