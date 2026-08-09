"""Minimal PE loader + capstone disassembler for the unpacked heaven7w.exe.

Set H7_EXE to the path of the UPX-unpacked Windows binary (upx -d heaven7w.exe).
The original release binary is not redistributed in this repo.
"""
import os, pefile
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

PE_PATH = os.environ.get('H7_EXE', 'h7w_unpacked.exe')
pe = pefile.PE(PE_PATH)
BASE = pe.OPTIONAL_HEADER.ImageBase
SECS = {}
for s in pe.sections:
    name = s.Name.decode(errors='replace').strip('\x00')
    SECS[name] = dict(va=BASE + s.VirtualAddress, vsize=s.Misc_VirtualSize, raw=s.get_data())
TEXT = SECS['.text']; CODE = TEXT['raw']; CVA = TEXT['va']

def va_to_off(va):
    for n, s in SECS.items():
        if s['va'] <= va < s['va'] + len(s['raw']):
            return n, va - s['va']
    return None, None

def read(va, n):
    nm, off = va_to_off(va)
    return None if nm is None else SECS[nm]['raw'][off:off+n]

_md = Cs(CS_ARCH_X86, CS_MODE_32); _md.detail = True; _md.skipdata = True

def disasm_range(va, size):
    nm, off = va_to_off(va)
    return list(_md.disasm(SECS[nm]['raw'][off:off+size], va))

def disasm_all():
    return list(_md.disasm(CODE, CVA))
