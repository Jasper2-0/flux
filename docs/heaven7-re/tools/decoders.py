"""Faithful ports of Heaven 7's two stream-decoder primitives, transcribed
instruction-by-instruction from the unpacked heaven7w.exe .text section.

  read_varint  @ 0x4086b8 : signed variable-length integer (7 or 15 bit)
  read_float   @ 0x4086c8 : variable-length compressed IEEE-754 float
"""
import struct

def read_varint(buf, p):
    """0x4086b8. Returns (value, new_p). Low bit of first byte selects width."""
    b = buf[p]; p += 1
    if (b & 1) == 0:            # shr eax,1 ; jae  (CF==0 -> 7-bit)
        return (b >> 1), p
    # 15-bit: movsx word[p-1]; sar 1  (signed)
    w = struct.unpack('<h', buf[p-1:p+1])[0]
    p += 1
    return (w >> 1), p

def _bits_to_float(exp_bits6, mant_bits, mant_shift):
    exp = ((exp_bits6 + 0xE0) & 0x1F)
    exp = (exp + 0x7A) & 0xFF
    val = (exp << 23) | ((mant_bits << mant_shift) & 0x7FFFFF)
    return struct.unpack('<f', struct.pack('<I', val & 0xFFFFFFFF))[0]

def read_float(buf, p):
    """0x4086c8. Returns (value, new_p). 1 byte=>0.0, else 2 or 3 byte forms."""
    if buf[p] & 1:                          # bit0==1 : 2-byte compact
        v = buf[p] | (buf[p+1] << 8)        # movzx word
        mant = (v & 0x3FE)                  # bits 1..9
        exp6 = (v >> 10) & 0x3F             # bits 10..15
        p += 2
        return _bits_to_float(exp6, mant, 13), p
    # bit0==0
    if buf[p] == 0:                         # single zero byte => 0.0
        return 0.0, p + 1
    v = buf[p] | (buf[p+1]<<8) | (buf[p+2]<<16) | (buf[p+3]<<24)  # mov dword (byte4 unused)
    mant = (v & 0x3FFFE)                    # bits 1..17
    exp6 = (v & 0xFF0000) >> 18             # bits 18..23
    p += 3
    return _bits_to_float(exp6, mant, 5), p

if __name__ == '__main__':
    # Round-trip sanity: a single 0x00 byte must decode to exactly 0.0 and
    # advance one byte, matching the je-to-0x40871a path.
    assert read_float(bytes([0x00]), 0) == (0.0, 1)
    # varint: 0x04 -> low bit 0 -> 4>>1 = 2 ; 0x05,0x00 -> 15bit signed
    assert read_varint(bytes([0x04]), 0) == (2, 1)
    v,_ = read_varint(bytes([0x03,0x00]),0)   # low bit 1 -> word 0x0003 >>1 = 1
    assert v == 1, v
    print('decoder self-checks passed')
    # Demonstrate the 2- and 3-byte float forms produce finite scene-scale values
    for raw in ([0x01,0x40],[0x03,0x20],[0xff,0x7f],[0x02,0x00,0x40,0x00],[0x02,0x10,0x30,0x00]):
        b=bytes(raw); f,np=read_float(b,0)
        print('bytes', b.hex(), '-> float', f, 'consumed', np)
