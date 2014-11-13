import sys
import serial
import struct
import os
import time

size = os.path.getsize(sys.argv[2])
written = 0

with serial.Serial(sys.argv[1], 4000000, timeout=1) as ser:
    ser.setDTR(False)
    time.sleep(1)
    ser.setDTR(True)
    time.sleep(1)
    ser.write(struct.pack("<I", size))
    with open(sys.argv[2], 'rb') as f:
        while True:
            data = f.read(1024 * 1024)
            if not data:
                break
            written += ser.write(data)
            waiting = ser.inWaiting()
            print(written, int((written / size) * 100), "%", ser.read(waiting).decode())
            ser.flush()
