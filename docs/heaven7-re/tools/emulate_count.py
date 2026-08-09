#!/usr/bin/env python3
"""Headless extraction of Heaven 7's scene script — no Windows, no Wine, no GPU.

Loads the UPX-unpacked heaven7w.exe into the Unicorn CPU emulator, stubs the
Win32/DirectDraw/DirectSound surface, and runs the real code until the scene
script has been interpreted. Hooks the script VM's opcode fetch and the two
stream decoders to dump the decoded script.

  pip install unicorn pefile capstone
  upx -d heaven7w.exe -o h7w_unpacked.exe
  python3 emulate_extract.py            # writes script_dump.json

Findings this harness established (see ../README.md):
  * the scene script is CLEARTEXT in .data at 0x418dca (no depacking needed)
  * the interpreter is at .text:0x40a60e; opcode byte -> handler via the
    16-bit offset table at .data:0x440f67 (handler = 0x400000 + entry)
  * 201 opcodes / 1735 decoder tokens make up the intro's timeline

Emulation notes: DirectDraw v1 vtable arg counts must be exact (the code
pushes an extra register before SetDisplayMode and pops it after, so
counting pushes at the call site over-pops and corrupts ESP). GetSurfaceDesc
must report a real pixel format because the blitter mode is chosen from
dwRGBBitCount and the RGB masks. CreateThread returns a fake handle so the
render/mixer thread never starts, and timeGetTime returns 0 — after the
script is parsed the main loop therefore spins, which is why we stop early.
"""
import struct, sys, collections
from unicorn import *
from unicorn.x86_const import *
import pefile
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

PE='h7w_unpacked.exe'
pe=pefile.PE(PE); BASE=pe.OPTIONAL_HEADER.ImageBase
ENTRY=BASE+pe.OPTIONAL_HEADER.AddressOfEntryPoint
md=Cs(CS_ARCH_X86,CS_MODE_32)

uc=Uc(UC_ARCH_X86,UC_MODE_32)
uc.mem_map(0x00400000,0x00060000)
IMG={}
for s in pe.sections:
    va=BASE+s.VirtualAddress; d=s.get_data()
    uc.mem_write(va,d); IMG[s.Name.decode().strip('\x00')]=(va,len(d))

uc.mem_map(0x00200000,0x00100000)                 # stack
uc.reg_write(UC_X86_REG_ESP,0x002f0000)
uc.mem_map(0x00390000,0x00010000)                 # cmdline
uc.mem_write(0x00390000,b'heaven7w.exe 1\x00')    # '1' = a quality flag => skip dialog
uc.mem_map(0x02000000,0x08000000)                 # heap
hp=[0x02001000]
def halloc(n):
    n=(n+0xfff)&~0xfff; p=hp[0]; hp[0]+=n+0x1000; return p
uc.mem_map(0x0F000000,0x00010000)                 # HLE import slots
uc.mem_map(0x0E000000,0x00020000)                 # COM objects+vtables

# COM: four interfaces, each with its own vtable band so we know interface+slot.
# band base -> (name, object addr, vtable addr)
COM_STUB=0x0F008000
BANDS={}   # band_index -> name
IFACES={}  # name -> (obj, vt)
def mk_iface(name, band):
    obj=0x0E000000+0x1000*(band+1)
    vt =obj+0x100
    uc.mem_write(obj,struct.pack('<I',vt))
    for i in range(64):
        uc.mem_write(vt+i*4,struct.pack('<I',COM_STUB+band*0x800+i*8))
    BANDS[band]=name; IFACES[name]=(obj,vt)
    return obj
DDOBJ   = mk_iface('dd',0)
SURFOBJ = mk_iface('surf',1)
DSOBJ   = mk_iface('ds',2)
DSBOBJ  = mk_iface('dsb',3)
SURF_W, SURF_H, SURF_BPP = 640, 480, 2
SURF_MEM=halloc(SURF_W*SURF_H*SURF_BPP*4)
imp={}; slot=[0]
for i in pe.DIRECTORY_ENTRY_IMPORT:
    dll=i.dll.decode()
    for f in i.imports:
        a=0x0F000000+slot[0]*0x10; slot[0]+=1
        uc.mem_write(f.address,struct.pack('<I',a))
        imp[a]=(dll, f.name.decode() if f.name else f'{dll}#{f.ordinal}')

def ret_with(nargs, rv):
    esp=uc.reg_read(UC_X86_REG_ESP)
    r=struct.unpack('<I',uc.mem_read(esp,4))[0]
    uc.reg_write(UC_X86_REG_ESP,esp+4+nargs*4)
    uc.reg_write(UC_X86_REG_EAX,rv&0xffffffff)
    uc.reg_write(UC_X86_REG_EIP,r)

ARGS={'GetCommandLineA':0,'ExitProcess':1,'TerminateThread':2,'Sleep':1,'GetModuleHandleA':1,
 'LeaveCriticalSection':1,'CreateThread':6,'InitializeCriticalSection':1,'SetThreadPriority':2,
 'GlobalFree':1,'GlobalAlloc':2,'DeleteCriticalSection':1,'EnterCriticalSection':1,'CloseHandle':1,
 'DirectDrawCreate':3,'RegisterClassA':1,'DestroyWindow':1,'EndDialog':2,'SetCursor':1,
 'CreateWindowExA':12,'ShowWindow':2,'MessageBoxA':4,'DispatchMessageA':1,'DefWindowProcA':4,
 'PeekMessageA':5,'SendDlgItemMessageA':5,'DialogBoxParamA':5,'timeGetTime':0}
icount=collections.Counter()
CLK=[0]
def do_imp(dll,nm):
    icount[nm]+=1
    n=ARGS.get(nm,0)
    if nm=='GetCommandLineA': return ret_with(0,0x00390000)
    if nm=='GlobalAlloc':
        esp=uc.reg_read(UC_X86_REG_ESP); fl,sz=struct.unpack('<II',uc.mem_read(esp+4,8))
        return ret_with(2,halloc(max(sz,64)))
    if nm=='GetModuleHandleA': return ret_with(1,BASE)
    if nm=='DirectDrawCreate':
        esp=uc.reg_read(UC_X86_REG_ESP); _,pp,_=struct.unpack('<III',uc.mem_read(esp+4,12))
        if pp: uc.mem_write(pp,struct.pack('<I',DDOBJ))
        return ret_with(3,0)
    if dll=='DSOUND.dll':
        esp=uc.reg_read(UC_X86_REG_ESP)
        try:
            _,pp,_=struct.unpack('<III',uc.mem_read(esp+4,12))
            if pp: uc.mem_write(pp,struct.pack('<I',DDOBJ))
        except Exception: pass
        return ret_with(3,0)
    if nm=='CreateWindowExA': return ret_with(12,0x00CD0000)
    if nm=='CreateThread': return ret_with(6,0x1234)   # never start the render thread
    if nm=='PeekMessageA': return ret_with(5,0)
    if nm=='ExitProcess': uc.emu_stop(); return
    if nm=='timeGetTime':
        CLK[0]+=250
        return ret_with(0,CLK[0])
    if nm=='DialogBoxParamA': return ret_with(5,ord('1'))
    return ret_with(n,1 if nm in ('RegisterClassA','ShowWindow','CloseHandle') else 0)


# Real vtable arg counts (including 'this') for the v1 interfaces this binary uses.
VT_ARGS={
 'dd':{0x00:3,0x04:1,0x08:1,0x0c:1,0x10:4,0x14:5,0x18:4,0x1c:3,0x20:5,0x24:5,0x28:1,
       0x2c:3,0x30:2,0x34:3,0x38:2,0x3c:2,0x40:2,0x44:2,0x48:2,0x4c:1,0x50:3,0x54:4,0x58:3},
 'surf':{0x00:3,0x04:1,0x08:1,0x0c:2,0x10:2,0x14:6,0x18:4,0x1c:6,0x20:3,0x24:3,0x28:4,
       0x2c:3,0x30:3,0x34:2,0x38:2,0x3c:2,0x40:3,0x44:2,0x48:2,0x4c:3,0x50:2,0x54:2,
       0x58:2,0x5c:3,0x60:1,0x64:5,0x68:2,0x6c:1,0x70:2,0x74:3,0x78:3,0x7c:2,0x80:2,
       0x84:6,0x88:2,0x8c:3},
 'ds':{0x00:3,0x04:1,0x08:1,0x0c:4,0x10:2,0x14:3,0x18:3,0x1c:1,0x20:2,0x24:2,0x28:2},
 'dsb':{0x00:3,0x04:1,0x08:1,0x0c:2,0x10:3,0x14:4,0x18:2,0x1c:2,0x20:2,0x24:2,0x28:3,
       0x2c:8,0x30:4,0x34:2,0x38:2,0x3c:2,0x40:2,0x44:2,0x48:1,0x4c:5,0x50:1},
}

# Fallback: count pushes preceding the call site to get stdcall arg count
argcache={}
def com_args(ret):
    if ret in argcache: return argcache[ret]
    lo=max(BASE, ret-64)
    data=uc.mem_read(lo, ret-lo)
    ins=list(md.disasm(bytes(data), lo))
    n=0
    # find the call whose end == ret, then walk back over pushes
    for k in range(len(ins)-1,-1,-1):
        if ins[k].address+ins[k].size==ret and ins[k].mnemonic=='call':
            j=k-1
            while j>=0 and ins[j].mnemonic=='push':
                n+=1; j-=1
            break
    argcache[ret]=n
    return n

comcount=collections.Counter()
def do_com(addr):
    band=(addr-COM_STUB)//0x800
    slotidx=((addr-COM_STUB)%0x800)//8
    off=slotidx*4
    name=BANDS.get(band,'?')
    comcount[f'{name}+{off:#x}']+=1
    esp=uc.reg_read(UC_X86_REG_ESP)
    ret=struct.unpack('<I',uc.mem_read(esp,4))[0]
    n=VT_ARGS.get(name,{}).get(off)
    if n is None: n=com_args(ret)
    def arg(k):
        return struct.unpack('<I',uc.mem_read(esp+4+4*k,4))[0]
    try:
        if name=='dd' and off==0x18:            # CreateSurface(this,desc,out,outer)
            uc.mem_write(arg(2),struct.pack('<I',SURFOBJ))
        elif name=='surf' and off==0x30:        # GetAttachedSurface(this,caps,out)
            uc.mem_write(arg(2),struct.pack('<I',SURFOBJ))
        elif name=='surf' and off==0x64:        # Lock(this,rect,desc,flags,hEvent)
            d=arg(2)
            uc.mem_write(d+0x08,struct.pack('<I',SURF_H))
            uc.mem_write(d+0x0c,struct.pack('<I',SURF_W))
            uc.mem_write(d+0x10,struct.pack('<I',SURF_W*SURF_BPP))
            uc.mem_write(d+0x24,struct.pack('<I',SURF_MEM))
        elif name=='surf' and off==0x58:        # GetSurfaceDesc(this,desc)
            d=arg(1)
            uc.mem_write(d+0x08,struct.pack('<I',SURF_H))
            uc.mem_write(d+0x0c,struct.pack('<I',SURF_W))
            uc.mem_write(d+0x10,struct.pack('<I',SURF_W*SURF_BPP))
            uc.mem_write(d+0x24,struct.pack('<I',SURF_MEM))
            # DDPIXELFORMAT at +0x48: 16-bit RGB 565
            uc.mem_write(d+0x48,struct.pack('<I',32))       # dwSize
            uc.mem_write(d+0x4c,struct.pack('<I',0x40))     # DDPF_RGB
            uc.mem_write(d+0x50,struct.pack('<I',0))        # dwFourCC
            uc.mem_write(d+0x54,struct.pack('<I',16))       # dwRGBBitCount
            uc.mem_write(d+0x58,struct.pack('<I',0xF800))   # R
            uc.mem_write(d+0x5c,struct.pack('<I',0x07E0))   # G
            uc.mem_write(d+0x60,struct.pack('<I',0x001F))   # B
        elif name=='ds' and off==0x0c:          # CreateSoundBuffer(this,desc,out,outer)
            uc.mem_write(arg(2),struct.pack('<I',DSBOBJ))
        elif name=='dsb' and off==0x2c:         # Lock(this,off,bytes,pp1,pb1,pp2,pb2,flags)
            uc.mem_write(arg(3),struct.pack('<I',SURF_MEM))
            uc.mem_write(arg(4),struct.pack('<I',0x8000))
            if arg(5): uc.mem_write(arg(5),struct.pack('<I',0))
            if arg(6): uc.mem_write(arg(6),struct.pack('<I',0))
    except Exception:
        pass
    uc.reg_write(UC_X86_REG_ESP,esp+4+n*4)
    uc.reg_write(UC_X86_REG_EAX,0)
    uc.reg_write(UC_X86_REG_EIP,ret)

VM_FETCH=0x0040a624
ops=[]; toks=[]; STOP=int(sys.argv[1]) if len(sys.argv)>1 else 20000
def hook(uc,addr,size,ud):
    if addr in imp:
        d,n=imp[addr]; do_imp(d,n); return
    if COM_STUB<=addr<COM_STUB+4*0x800: do_com(addr); return
    if addr==VM_FETCH:
        edi=uc.reg_read(UC_X86_REG_EDI)
        try: op=uc.mem_read(edi,1)[0]
        except Exception: op=-1
        ops.append((edi,op))
        if len(ops)>=STOP: uc.emu_stop()
        return
    if addr==0x4086b8 or addr==0x4086c8:
        toks.append(('v' if addr==0x4086b8 else 'f', uc.reg_read(UC_X86_REG_EDI)))
def h_imp(uc,addr,size,ud):
    d,n=imp[addr]; do_imp(d,n)
for a in imp: uc.hook_add(UC_HOOK_CODE,h_imp,begin=a,end=a)

def h_com(uc,addr,size,ud): do_com(addr)
uc.hook_add(UC_HOOK_CODE,h_com,begin=COM_STUB,end=COM_STUB+4*0x800-1)

def h_vm(uc,addr,size,ud):
    edi=uc.reg_read(UC_X86_REG_EDI)
    try: op=uc.mem_read(edi,1)[0]
    except Exception: op=-1
    ops.append({'p':edi,'op':op,'depth':depth[0],'ntok':len(toks)})
    if len(ops)>=STOP: uc.emu_stop()
uc.hook_add(UC_HOOK_CODE,h_vm,begin=VM_FETCH,end=VM_FETCH)

def h_dec(uc,addr,size,ud):
    toks.append({'k':'v' if addr==0x4086b8 else 'f','p':uc.reg_read(UC_X86_REG_EDI),'v':None})
uc.hook_add(UC_HOOK_CODE,h_dec,begin=0x4086b8,end=0x4086b8)
uc.hook_add(UC_HOOK_CODE,h_dec,begin=0x4086c8,end=0x4086c8)

def h_decret(uc,addr,size,ud):
    if not toks: return
    eax=uc.reg_read(UC_X86_REG_EAX)
    t=toks[-1]
    if t['v'] is not None: return
    if addr==0x4086c7:
        t['v']=struct.unpack('<i',struct.pack('<I',eax))[0]
    else:
        t['v']=struct.unpack('<f',struct.pack('<I',eax))[0]
    t['end']=uc.reg_read(UC_X86_REG_EDI)
uc.hook_add(UC_HOOK_CODE,h_decret,begin=0x4086c7,end=0x4086c7)
uc.hook_add(UC_HOOK_CODE,h_decret,begin=0x40871e,end=0x40871e)

depth=[0]
def h_vmenter(uc,addr,size,ud): depth[0]+=1
uc.hook_add(UC_HOOK_CODE,h_vmenter,begin=0x40a60e,end=0x40a60e)
def h_vmexit(uc,addr,size,ud):
    depth[0]-=1
uc.hook_add(UC_HOOK_CODE,h_vmexit,begin=0x40a63b,end=0x40a63b)

# DEFINITIVE COUNT: every texture allocation and every texture-VM program
allocs=[]
def h_texalloc(uc,addr,size,ud):
    allocs.append((uc.reg_read(UC_X86_REG_EAX), uc.reg_read(UC_X86_REG_EDX)))
uc.hook_add(UC_HOOK_CODE,h_texalloc,begin=0x4088c9,end=0x4088c9)
texentry=[]
def h_texvm(uc,addr,size,ud):
    texentry.append(uc.reg_read(UC_X86_REG_EDI))
uc.hook_add(UC_HOOK_CODE,h_texvm,begin=0x44189f,end=0x44189f)
scriptmax=[0]
def h_smax(uc,addr,size,ud):
    e=uc.reg_read(UC_X86_REG_EDI)
    if e>scriptmax[0]: scriptmax[0]=e
uc.hook_add(UC_HOOK_CODE,h_smax,begin=0x40a624,end=0x40a624)

blocks=collections.Counter()
tail=collections.deque(maxlen=40)
def h_blk(uc,addr,size,ud):
    blocks[addr]+=1
    tail.append(addr)
uc.hook_add(UC_HOOK_BLOCK,h_blk)

faults=[]
def mem_bad(uc,acc,ad,sz,val,ud):
    faults.append((uc.reg_read(UC_X86_REG_EIP),acc,ad)); return False
uc.hook_add(UC_HOOK_MEM_READ_UNMAPPED|UC_HOOK_MEM_WRITE_UNMAPPED|UC_HOOK_MEM_FETCH_UNMAPPED,mem_bad)

try:
    uc.emu_start(ENTRY,0,count=3_000_000_000)
except UcError as e:
    print('UcError:',e,'at eip',hex(uc.reg_read(UC_X86_REG_EIP)))
print('imports called:',dict(icount))
print('com calls:',dict(comcount))
print('VM opcode fetches:',len(ops))
print('script opcodes:',len(ops))
import json
json.dump({'ops':ops,'toks':toks},open('script_dump.json','w'))
print('wrote script_dump.json')
print('decoder tokens:',len(toks))
print('faults:',[(hex(a),b,hex(c)) for a,b,c in faults[:4]])
print('final eip:',hex(uc.reg_read(UC_X86_REG_EIP)))
print('hot blocks:',[(hex(a),n) for a,n in blocks.most_common(6)])
print('clock reached (ms):',CLK[0])
print('TEXTURE ALLOCATIONS:',len(allocs),'sizes:',sorted(set(allocs)))
print('TEXTURE-VM PROGRAMS:',len(texentry),'ptrs:',[hex(x) for x in texentry])
print('script cursor max:',hex(scriptmax[0]))
import json; json.dump({'allocs':allocs,'texentry':texentry,'smax':scriptmax[0]},open('texcount.json','w'))
