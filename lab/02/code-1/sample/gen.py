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
05 20 00 00 00 00 00 00
00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00
21 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00
04 00 00 00 00 00 00 00
"""

answer_hex = """
AB 10 04 02 00 00 00 00
00 00 00 00 00 00 00 00
04 00 00 00 00 00 00 00

AB 10 04 02 00 00 00 00
00 00 00 00 00 00 00 00
04 00 00 00 00 00 00 00
"""


def dump(hex_str, filename):
    with open(filename, "wb") as file:
        file.write(bytes.fromhex("".join(hex_str.split())))


dump(input_hex, "sample/input.bin")
dump(answer_hex, "sample/answer.bin")
