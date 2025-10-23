input_hex = """
01 00 00 00

00
05 20 00 00 00 00 00 00
00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00
21 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00
04 00 00 00 00 00 00 00
"""

answer_hex = """
24 20 00 00 00 00 00 00
00 00 00 00 00 00 00 00
04 00 00 00 00 00 00 00
"""


def dump(hex_str, filename):
    with open(filename, "wb") as file:
        file.write(bytes.fromhex("".join(hex_str.split())))


dump(input_hex, "sample/input.bin")
dump(answer_hex, "sample/answer.bin")
