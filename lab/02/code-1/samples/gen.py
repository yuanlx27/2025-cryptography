# Hex bytes as a multiline string
hex_str = '''
02 00 00 00

00
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
'''

# Remove whitespace and line breaks, keep only hex numbers
hex_bytes = []
for line in hex_str.strip().splitlines():
    # Ignore empty lines
    line = line.strip()
    if line == "":
        continue
    hex_bytes.extend(line.split())

# Convert hex strings to bytes
data = bytes(int(b, 16) for b in hex_bytes)

# Write to file
with open("sample3_in.bin", "wb") as f:
    f.write(data)
