characters = {
    "a": " # "
         "# #"
         "###"
         "# #",

    "b": "###"
         "## "
         "# #"
         "## ",

    "c": " ##"
         "#  "
         "#  "
         " ##",

    "d": "## "
         "# #"
         "# #"
         "## ",

    "e": "###"
         "## "
         "#  "
         "###",

    "f": "###"
         "## "
         "#  "
         "#  ",

    "g": " ##"
         "#  "
         "# #"
         " ##",

    "h": "# #"
         "###"
         "# #"
         "# #",

    "i": "###"
         " # "
         " # "
         "###",

    "j": "###"
         "  #"
         "# #"
         " # ",

    "k": "# #"
         "## "
         "# #"
         "# #",


    "l": "#  "
         "#  "
         "#  "
         "###",

    "m": "###"
         "###"
         "# #"
         "# #",

    "n": "## "
         "# #"
         "# #"
         "# #",

    "o": " # "
         "# #"
         "# #"
         " # ",

    "p": "###"
         "# #"
         "###"
         "#  ",

    "q": " # "
         "# #"
         "###"
         " ##",

    "r": "## "
         "# #"
         "## "
         "# #",

    "s": "###"
         "## "
         "  #"
         "###",

    "t": "###"
         " # "
         " # "
         " # ",

    "u": "# #"
         "# #"
         "# #"
         " # ",

    "v": "# #"
         "# #"
         "## "
         "#  ",

    "w": "# #"
         "# #"
         "###"
         "###",

    "x": "# #"
         " # "
         "# #"
         "# #",

    "y": "# #"
         " # "
         " # "
         " # ",

    "z": "###"
         " # "
         "#  "
         "###",

    "0": " # "
         "###"
         "# #"
         " # ",

    "1": "## "
         " # "
         " # "
         "###",

    "2": "## "
         "  #"
         " # "
         "###",

    "3": "## "
         " ##"
         "  #"
         "## ",

    "4": "# #"
         "###"
         "  #"
         "  #",

    "5": "###"
         "## "
         "  #"
         " # ",

    "6": " ##"
         "#  "
         "###"
         "## ",

    "7": "###"
         "  #"
         " # "
         "#  ",

    "8": "###"
         "###"
         "# #"
         "###",

    "9": " ##"
         "###"
         "  #"
         "## ",

    ":": "#  "
         "   "
         "#  "
         "   ",
}

width = 3
height = 4

for c, data in characters.items():
    clist = []

    with open(f"{c}.bin", "wb") as f:
        it = iter(range(width * height))

        written_bytes = 0

        for index in range(width * height):
            if data[index] == "#":
                clist.append(index)

            if len(clist) == 2:
                p1 = ((clist[0] % width) << 2) | (clist[0] // width)
                p2 = ((clist[1] % width) << 2) | (clist[1] // width)

                byte = (p1 << 4) | p2

                print(f"{c}: writing byte {byte:08b}")
                f.write(byte.to_bytes(1, 'big'))

                written_bytes += 1

                clist.clear()

        if len(clist) != 0:
            p1 = ((clist[0] % width) << 2) | (clist[0] // width)

            byte = (p1 << 4) | 0b1111

            f.write(byte.to_bytes(1, 'big'))

            written_bytes += 1

        while written_bytes != 12:
            f.write(0b11111111.to_bytes(1, 'big'))

            written_bytes += 1
