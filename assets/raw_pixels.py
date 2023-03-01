from PIL import Image
import sys

read_fname = sys.argv[1]
write_fname = sys.argv[2]

f_in = Image.open(read_fname)
f_out = open(write_fname, 'wb')

pallette_mapping = {
    (0xf8, 0xfc, 0x00, 0xff): 0,
    (0xff, 0xff, 0x00, 0xff): 0,
    (0xfb, 0xf2, 0x36, 0xff): 0,
    (0xa8, 0x54, 0x50, 0xff): 1,
    (0xf8, 0x54, 0x00, 0xff): 2,
    (0xff, 0xff, 0xff, 0xff): 3,
    (0x0, 0x0, 0x0, 0x0): 255,
}

for x in range(f_in.width):
    for y in range(f_in.height):
        pixel = f_in.getpixel((x,y))

        f_out.write(pallette_mapping[pixel].to_bytes(1, byteorder='big'))
        # print(pixel)

        # f_out.write(pixel[0].to_bytes(1, byteorder='big'))
        # f_out.write(pixel[1].to_bytes(1, byteorder='big'))
        # f_out.write(pixel[2].to_bytes(1, byteorder='big'))
        # f_out.write(pixel[3].to_bytes(1, byteorder='big'))

f_out.close()
