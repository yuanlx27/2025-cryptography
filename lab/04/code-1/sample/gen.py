file_name = "input-4.bin"
hex_string = "".join("61" for i in range(1000000))


def dump(file_name, hex_string):
    with open(file_name, "wb") as file:
        file.write(bytes.fromhex("".join(hex_string.split())))


dump(file_name, hex_string)
