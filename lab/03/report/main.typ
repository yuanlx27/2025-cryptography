#import "@local/sysu-templates:0.1.0": report

#show: report.with(
  title: "实验报告",
  subtitle: "实验三：AES-128 的实现",
  student: (name: "元朗曦", id: "23336294"),
  institude: "计算机学院",
  major: "计算机科学与技术",
  class: "计八",
)

= 实验目的

通过实现 AES-128 CBC 工作模式的加密和解密，加深对对称加密算法的理解。

= 实验内容

用 Rust 实现 AES-128 CBC 工作模式的加密和解密。

= 实验原理

== AES 算法

#quote(attribution: [Cryptography: Theory and Practice], block: true)[
  We first give a high-level description of AES.
  The algorithm proceeds as follows:

  + Given a plaintext $x$, initialize *State* to be $x$ and perform an operation AddRoundKey, which x-ors the *RoundKey* with *State*.

  + For each of the first $N - 1$ rounds, perform a substitution operation called #smallcaps[SubBytes] on *State* using an S-box;
    perform a permutation #smallcaps[ShiftRows] on *State*;
    perform an operation #smallcaps[MixColumns] on *State*;
    and perform #smallcaps[AddRoundKey].

  + Perform #smallcaps[SubBytes]\;
    perform #smallcaps[ShiftRows]\;
    and perform #smallcaps[AddRoundKey].

  + Define the ciphertext $y$ to be *State*.
]

本实验使用 AES-128 算法，数据块和密钥均为 128 位，轮数 $N = 10$。

我们需要实现字节代换（#smallcaps[SubBytes]）、行移位（#smallcaps[ShiftRows]）、列混淆（#smallcaps[MixColumns]）和轮密钥加（#smallcaps[AddRoundKey]）。

== CBC 工作模式

在 CBC 工作模式下，我们有一个初始向量 IV，每个数据块在加密前会与前一个密文块进行异或操作，第一个数据块则与 IV 进行异或操作。

== PKCS\#7 填充

我们使用 PKCS\#7 填充，使得数据长度为块大小（16 Bytes）的倍数。设原长度为 $n$ 字节，则填充长度为 $k = 16 - (n mod 16)$ 字节，填充值均为 $k$。

= 实验步骤

具体代码见#link("https://github.com/yuanlx27/2025-cryptography")[代码仓库]。

= 实验结果

#figure(image(width: 80%, "assets/images/20251105-024820.png"))

= 实验总结

通过实现 AES-128 CBC 工作模式的加密和解密，加深了对对称加密算法的理解。
