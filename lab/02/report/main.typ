#import "@local/sysu-templates:0.1.0": report
#import "@preview/lovelace:0.3.0": *

#show: report.with(
  title: "实验报告",
  subtitle: "实验二：有限域的实现",
  student: (name: "元朗曦", id: "23336294"),
  institude: "计算机学院",
  major: "计算机科学与技术",
  class: "计八",
)

= 实验目的

通过实现有限域上的基础运算，加深对有限域理论的理解，为后续密码学算法的实现打下基础。

= 实验内容

- 用 Rust 实现有限域 $FF_(2^131)$ #h(-0.2em) 上的加法、乘法、平方和求逆运算，其中求逆运算包括基于扩展欧几里得算法和费马小定理的两种实现。

= 实验原理

有限域 $FF_(2^m)$ #h(-0.2em) 中的元素为 $m$ 次以下的多项式，系数为 $0$ 或 $1$，故可以用 $m$ 位二进制数表示。

== 加法

按位异或。

== 减法

模 $2$ 意义下等价于加法。

== 乘法

计算 $c(z) = a(z) b(z)$ 时，朴素的做法是按位相乘再累加异或和，时间复杂度较高。一种更高效的做法是结合加法和位运算：

#pseudocode(
  [INPUT: Binary polynomials $a(z)$ and $b(z)$ of degree at most m − 1.],
  [OUTPUT: $c(z) = a(z)b(z) mod f(z)$.],
  [$c <- 1$.],
  [*for* $i$ from $1$ to $m − 1$ *do*],
  indent(
    [*if* $a_i = 1$ *then*],
    indent(
      [$c <- c + b dot z^i$.],
    ),
    [*end*],
  ),
  [*end*],
  [*return* $c$.],
)

== 求逆

=== 基于扩展欧几里得算法

与一般形式略有不同。具体见 Guide to Elliptic Curve Cryptography 第 2.3.6 节。

=== 基于费马小定理

在 $FF_(2^m)$ #h(-0.2em) 中，任意非零元素 $a$ 满足 $a^(2^m - 1) = 1$，因此 $a^(-1) = a^(2^m - 2)$。利用快速幂算法可以高效地计算出 $a^(2^m - 2)$。

= 实验步骤

具体代码见#link("https://github.com/yuanlx27/2025-cryptography")[代码仓库]。

= 实验结果

#grid(
  columns: 2,
  figure(
    caption: "基于扩展欧几里得算法的实现",
    image("assets/images/20251029-055524.png"),
  ),
  figure(
    caption: "基于费马小定理的实现",
    image("assets/images/20251029-055532.png"),
  )
)

= 实验总结

通过实现有限域上的基础运算，加深了对有限域理论的理解，为后续密码学算法的实现打下了基础。
