#import "@local/sysu-templates:0.2.0": report

#show: report.with(
  title: "实验五：RSA 的实现",
  subtitle: "现代密码学实验报告",
  student: (name: "元朗曦", id: "23336294"),
  institude: "计算机学院",
  major: "计算机科学与技术",
  class: "计八",
)

= 实验目的

+ 深入理解 RSA 算法原理：掌握 RSA 公钥密码体制的数学基础，包括大数模幂运算、欧拉定理以及公私钥的生成与使用。

+ 掌握 OAEP 填充机制：理解直接 RSA 加密的安全性缺陷（如确定性加密），通过实现 OAEP（Optimal Asymmetric Encryption Padding）填充方案，增强 RSA 加密的语义安全性（IND-CCA2）。

+ 实现大数运算优化：通过实现*蒙哥马利归约（Montgomery Reduction）*算法，优化大整数模幂运算的性能，避免频繁的高精度除法操作。

+ 应用中国剩余定理（CRT）加速解密：利用 RSA 私钥中的 $p$，$q$ 等参数，通过 CRT 将模 $n$ 的运算分解为模 $p$ 和模 $q$ 的运算，显著提升解密速度。

+ 熟悉密码学安全编程：使用密码学安全的随机数生成器（CSPRNG），并处理侧信道攻击风险（如解密失败时的静默处理）。

= 实验原理

== RSA 算法基础

RSA 的安全性基于大整数分解难题。

加密：给定公钥 $(n, e)$ 和明文 $m$，密文 $c = m^e mod n$。

解密：给定私钥 $(n, d)$ 和密文 $c$，明文 $m = c^d mod n$。

其中 $n = p q, phi(n) = (p − 1)(q − 1), e d equiv 1 (mod phi(n))$。

== OAEP 填充 (PKCS\#1 v2.2)

为了防止选择密文攻击和通过密文推导明文统计信息，在加密前对消息 M 进行填充。

编码过程：

+ 生成随机种子 seed。

+ 构建数据块 $"DB" = "LHash" || "PS" || "0x01" || "M"$，其中 LHash 是标签 L 的 SHA-256 哈希（本实验 L 为空），PS 为全 0 填充串。

+ 使用掩码生成函数 MGF1（基于 SHA-256）对 DB 和 seed 进行异或掩码操作，生成 maskedDB 和 maskedSeed。

+ 最终编码块 $"EM" = "0x00" || "maskedSeed" || "maskedDB"$。

+ 解码过程：为编码的逆过程，需严格校验首字节 0x00、LHash 的正确性以及 0x01 分隔符的存在，校验失败则视为密文无效。

== 蒙哥马利归约 (Montgomery Reduction)

大数模幂运算 $x^k mod n$ 是 RSA 的核心瓶颈。标准的模乘 $a b mod n$ 需要高代价的除法。 蒙哥马利乘法将数转换到蒙哥马利域（$A' = A R mod n$），利用位移和加法代替除法计算 $A B R^(−1) (mod n)$。

+ 选取 $R = 2^k > n$（本实验中根据 $n$ 的位数动态选取）。

+ 预计算 $n' = −n^(−1) mod R$。

+ 整个模幂过程中，数值保持在蒙哥马利域中运算，仅在最后转换回普通域。

== 中国剩余定理 (CRT) 加速解密

直接计算 $m = c^d (mod n)$ 需要处理 2048 位的指数运算。利用 CRT，可以将解密分解为两个 1024 位的运算：

$
  m_1 = c^(d mod (p − 1)) (mod p) \
  m_2 = c^(d mod (q − 1)) (mod q)
$

利用 Garner 算法合并结果：$h = (m_1 − m_2) q^(−1) mod p$，则 $m = m_2 + h q$。 理论上，CRT 模式比直接解密快约 4 倍（两个半规模的幂运算）。

= 实验内容

== 加密

#raw(block: true, lang: "rust", read("code-1/src/main.rs"))

== 解密

#raw(block: true, lang: "rust", read("code-2/src/main.rs"))

= 实验结果

#grid(
  columns: 2,
  figure(
    caption: [实验 5-1 评测结果],
    image("assets/images/20251202-155558.png"),
  ),
  figure(
    caption: [实验 5-2 评测结果],
    image("assets/images/20251202-155609.png"),
  ),
)
