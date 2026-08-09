# Headless x86 emulation harness for extracting Heaven 7's scene script WITHOUT
# a Windows box or the GUI: loads the unpacked PE into the Unicorn CPU emulator,
# stubs every OS import, and hooks the two stream decoders (read_varint @
# 0x4086b8, read_float @ 0x4086c8) to capture the scene stream as the parser
# decodes it. Run: H7_EXE not needed; place h7w_unpacked.exe alongside.
#
# STATUS: work in progress. The CPU + memory model, import stubs, GlobalAlloc
# bump-heap, and command-line parser all run correctly. To reach the scene
# parser the init path still needs faithful stubs for: the DirectDraw COM
# vtable methods (CreateSurface must return a usable surface pointer), the
# settings DialogBoxParamA callback (return the chosen quality), and the
# CreateThread worker entry (the parse may run there). Those are the remaining
# pieces; the decoder hooks below are ready to dump the stream the moment
# control reaches them. This is the no-Windows alternative to tools/extract_scene.js.

# Headless x86 emulation of heaven7w.exe just far enough to capture the scene
# script the parser decodes. DirectDraw/DirectSound/USER32 are stubbed; only the
# CPU + memory the parser needs are real.
import struct, sys
from unicorn import *
from unicorn.x86_const import *
import pefile

PE='h7w_unpacked.exe'
pe=pefile.PE(PE)
BASE=pe.OPTIONAL_HEADER.ImageBase
ENTRY=BASE+pe.OPTIONAL_HEADER.AddressOfEntryPoint

uc=Uc(UC_ARCH_X86, UC_MODE_32)

# --- memory map ---
IMG_LO=0x00400000; IMG_SZ=0x00050000     # covers .text..rsrc
uc.mem_map(IMG_LO, IMG_SZ)
for s in pe.sections:
    va=BASE+s.VirtualAddress
    data=s.get_data()
    uc.mem_write(va, data)

STACK=0x00200000; STACK_SZ=0x00100000
uc.mem_map(STACK, STACK_SZ)
uc.reg_write(UC_X86_REG_ESP, STACK+STACK_SZ-0x1000)
uc.reg_write(UC_X86_REG_EBP, STACK+STACK_SZ-0x1000)

HEAP=0x02000000; HEAP_SZ=0x04000000      # 64MB bump heap for GlobalAlloc
uc.mem_map(HEAP, HEAP_SZ)
heap_ptr=[HEAP+0x1000]
def halloc(n):
    n=(n+0x0f)&~0x0f
    p=heap_ptr[0]; heap_ptr[0]+=n
    return p

# HLE region: IAT entries + COM vtable method stubs point here; a code hook
# intercepts execution and emulates the call, then returns.
HLE=0x0F000000; HLE_SZ=0x00010000
uc.mem_map(HLE, HLE_SZ)

# fake COM object + big vtable of stub pointers
COM=0x0E000000; COM_SZ=0x00010000
uc.mem_map(COM, COM_SZ)
VTABLE=COM+0x100
FAKE_OBJ=COM+0x40
uc.mem_write(FAKE_OBJ, struct.pack('<I', VTABLE))
for i in range(256):
    uc.mem_write(VTABLE+i*4, struct.pack('<I', HLE+0x8000+i*0x10))  # each method -> a COM stub slot

# IAT thunk -> HLE slot
iat={}
slot=0
def hle_for(addr):
    global slot
    a=HLE+slot*0x10; slot+=1
    uc.mem_write(addr, struct.pack('<I', a))
    return a
imp_by_hle={}
for imp in pe.DIRECTORY_ENTRY_IMPORT:
    dll=imp.dll.decode()
    for f in imp.imports:
        nm=f.name.decode() if f.name else f'{dll}#{f.ordinal}'
        a=hle_for(f.address)
        imp_by_hle[a]=(dll,nm)

log=[]
decoder_hits=[]
STOP_AFTER=int(sys.argv[1]) if len(sys.argv)>1 else 4000

def pop_ret_and_args(nargs, retval, stdcall=True):
    esp=uc.reg_read(UC_X86_REG_ESP)
    ret=struct.unpack('<I', uc.mem_read(esp,4))[0]
    esp+=4
    if stdcall: esp+=nargs*4
    uc.reg_write(UC_X86_REG_ESP, esp)
    uc.reg_write(UC_X86_REG_EAX, retval & 0xffffffff)
    uc.reg_write(UC_X86_REG_EIP, ret)

# import behaviors (stdcall arg counts)
def do_import(dll,nm):
    if nm=='GetCommandLineA':
        p=halloc(8); uc.mem_write(p, b'\x00'); pop_ret_and_args(0,p); return
    if nm=='GlobalAlloc':
        esp=uc.reg_read(UC_X86_REG_ESP)
        flags,size=struct.unpack('<II', uc.mem_read(esp+4,8))
        p=halloc(max(size,16))
        uc.mem_write(p, b'\x00'*min(max(size,16),1<<20))
        pop_ret_and_args(2,p); return
    if nm=='GlobalFree': pop_ret_and_args(1,0); return
    if nm=='GetModuleHandleA': pop_ret_and_args(1,BASE); return
    if nm in ('InitializeCriticalSection','DeleteCriticalSection','EnterCriticalSection','LeaveCriticalSection'): pop_ret_and_args(1,0); return
    if nm=='SetThreadPriority': pop_ret_and_args(2,1); return
    if nm=='CreateThread': pop_ret_and_args(6, 0x1234); return   # don't spawn; return fake handle
    if nm=='TerminateThread': pop_ret_and_args(2,1); return
    if nm=='CloseHandle': pop_ret_and_args(1,1); return
    if nm=='Sleep': pop_ret_and_args(1,0); return
    if nm=='timeGetTime': pop_ret_and_args(0,0); return
    if nm=='ExitProcess': uc.emu_stop(); return
    if nm=='DirectDrawCreate':
        esp=uc.reg_read(UC_X86_REG_ESP)
        _,pp,_=struct.unpack('<III', uc.mem_read(esp+4,12))
        uc.mem_write(pp, struct.pack('<I', FAKE_OBJ))
        pop_ret_and_args(3,0); return
    if dll=='DSOUND.dll':
        esp=uc.reg_read(UC_X86_REG_ESP)
        # DirectSoundCreate(guid, ppDS, outer) -> write fake obj
        try:
            _,pp,_=struct.unpack('<III', uc.mem_read(esp+4,12))
            uc.mem_write(pp, struct.pack('<I', FAKE_OBJ))
        except: pass
        pop_ret_and_args(3,0); return
    # USER32: return benign
    if nm=='DialogBoxParamA': pop_ret_and_args(5,1); return   # 5 args; returns chosen quality
    if nm=='MessageBoxA': pop_ret_and_args(4,1); return
    if nm=='CreateWindowExA': pop_ret_and_args(12, 0x00CD0000); return
    if nm=='RegisterClassA': pop_ret_and_args(1,1); return
    if nm=='ShowWindow': pop_ret_and_args(2,1); return
    if nm=='DestroyWindow': pop_ret_and_args(1,1); return
    if nm=='EndDialog': pop_ret_and_args(2,1); return
    if nm=='SetCursor': pop_ret_and_args(1,0); return
    if nm=='DispatchMessageA': pop_ret_and_args(1,0); return
    if nm=='DefWindowProcA': pop_ret_and_args(4,0); return
    if nm=='PeekMessageA': pop_ret_and_args(5,0); return       # no messages
    if nm=='SendDlgItemMessageA': pop_ret_and_args(5,0); return
    # default: assume 0 args
    pop_ret_and_args(0,0)

def hook_code(uc,address,size,ud):
    if address in imp_by_hle:
        dll,nm=imp_by_hle[address]
        do_import(dll,nm)
        return
    if HLE+0x8000 <= address < HLE+0x8000+256*0x10:
        # a COM method stub: return S_OK. We don't know arg count; assume 1
        # (the 'this' + ...). Most DDraw methods are 1-3 args; use 3 to reduce
        # stack leak; scene parse doesn't use COM so drift here is harmless.
        pop_ret_and_args(3,0)
        return
    if address==0x4086b8 or address==0x4086c8:
        edi=uc.reg_read(UC_X86_REG_EDI)
        decoder_hits.append(('v' if address==0x4086b8 else 'f', edi))
        if len(decoder_hits)>=STOP_AFTER:
            uc.emu_stop()

uc.hook_add(UC_HOOK_CODE, hook_code)

# capture invalid mem to report where it stalls
faults=[]
def hook_mem_invalid(uc,acc,addr,size,val,ud):
    faults.append((uc.reg_read(UC_X86_REG_EIP), acc, addr, size))
    return False
uc.hook_add(UC_HOOK_MEM_READ_UNMAPPED|UC_HOOK_MEM_WRITE_UNMAPPED|UC_HOOK_MEM_FETCH_UNMAPPED, hook_mem_invalid)

try:
    uc.emu_start(ENTRY, 0, count=20_000_000)
except UcError as e:
    print('UcError:', e)

print('decoder hits:', len(decoder_hits))
if decoder_hits[:20]:
    print('first hits (kind, edi):', [(k,hex(p)) for k,p in decoder_hits[:20]])
if faults[:5]:
    print('faults (eip, acc, addr, size):', [(hex(a),b,hex(c),d) for a,b,c,d in faults[:5]])
print('final esp:', hex(uc.reg_read(UC_X86_REG_ESP)))
