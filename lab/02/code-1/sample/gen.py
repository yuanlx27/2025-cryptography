input_hex = """
02 00 00 00

01
05 20 00 00 00 00 00 00
00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00
21 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00
04 00 00 00 00 00 00 00

01
D8 3C FE 2A 9D 8E 8F 8D
74 53 84 A7 AF D8 8A 94
04 00 00 00 00 00 00 00
D4 FB 85 B0 5E B5 7D 59
3B 4A 41 2D 54 3A 2B 3E
06 00 00 00 00 00 00 00
"""

answer_hex = """
AB 10 04 02 00 00 00 00
00 00 00 00 00 00 00 00
04 00 00 00 00 00 00 00

A9 23 11 91 DF 9F 7B 76
A7 C3 5A CD 55 CF 34 E7
01 00 00 00 00 00 00 00
"""


def dump(hex_str, filename):
    with open(filename, "wb") as file:
        file.write(bytes.fromhex("".join(hex_str.split())))


dump(input_hex, "sample/input.bin")
dump(answer_hex, "sample/answer.bin")
