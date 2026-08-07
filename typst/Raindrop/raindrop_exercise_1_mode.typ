/** Settings */
#set page(paper: "a4")
#set text(size: 11pt)
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