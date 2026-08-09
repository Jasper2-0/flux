"""Print annotated disassembly of the key scene-script routines.
Run:  H7_EXE=/path/to/unpacked.exe python3 annotate.py
"""
from analib import disasm_range
def dump(va, size, title):
    print(f"\n; ===== {title}  (.text:{va:#x}) =====")
    for i in disasm_range(va, size):
        if i.id == 0: break
        b = ' '.join('%02x' % x for x in i.bytes)
        print('%08x  %-20s %-9s %s' % (i.address, b, i.mnemonic, i.op_str))
dump(0x4086b8, 16,  'read_varint  -> signed 7/15-bit varint in EAX, cursor in EDI')
dump(0x4086c8, 88,  'read_float   -> compressed IEEE-754 float in EAX (1/2/3-byte forms)')
dump(0x406a39, 90,  'parse_object_record  (ESI=object, EDI=stream): 3 varints + N*6 floats + 4 floats')
dump(0x4023b7, 170, 'shade_tree_eval  (recursive, ESI=32-byte node): texture sample + color modulate')
