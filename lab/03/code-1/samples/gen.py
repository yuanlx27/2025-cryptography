input_hex = """
81
00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F
00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F
20 00 00 00
76 D0 62 7D A1 D2 90 43 6E 21 A4 AF 7F CA 94 B7
17 7C 1F C9 41 73 D4 42 E3 6E E7 9D 7C A0 E4 61
"""

answer_hex = """
00 11 22 33 44 55 66 77 88 99 AA BB CC DD EE FF
"""


def dump(hex_str, filename):
    with open(filename, "wb") as file:
        file.write(bytes.fromhex("".join(hex_str.split())))


dump(input_hex, "samples/input.bin")
dump(answer_hex, "samples/answer.bin")
