from PIL import Image
import sys

read_fname = sys.argv[1]
write_fname = sys.argv[2]

f_in = Image.open(read_fname)
f_out = open(write_fname, 'wb')

for x in range(f_in.width):
    for y in range(f_in.height):
        pixel = f_in.getpixel((x,y))

        f_out.write(pixel[0].to_bytes(1, byteorder='big'))
        f_out.write(pixel[1].to_bytes(1, byteorder='big'))
        f_out.write(pixel[2].to_bytes(1, byteorder='big'))
        f_out.write(pixel[3].to_bytes(1, byteorder='big'))

f_out.close()
