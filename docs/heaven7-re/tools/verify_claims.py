#!/usr/bin/env python3
"""Re-verify this directory's BYTES claims directly against the binary.

    H7_EXE=h7w_unpacked.exe python3 verify_claims.py ../claims.yaml

Every claim whose provenance is BYTES carries a `check` block that is re-read
from the PE and compared. Claims in the other provenance classes (DISASM,
TRACE, CROSS, INFER) cannot be machine-checked; they are listed as UNCHECKED
with their locators so the unverified surface stays visible.

Exit code is non-zero if any BYTES claim fails, so this can gate a commit.
See ../METHODOLOGY.md for what the classes mean.
"""
import os, struct, sys

# --- minimal YAML subset reader -------------------------------------------
# claims.yaml uses only: a top-level list of maps, scalar values, and a nested
# `check:` map. Rather than add a dependency, parse exactly that.
def load_claims(path):
    claims, cur, incheck = [], None, False
    for raw in open(path):
        line = raw.rstrip('\n')
        if not line.strip() or line.lstrip().startswith('#'):
            continue
        indent = len(line) - len(line.lstrip())
        s = line.strip()
        if s.startswith('- '):
            cur = {}
            claims.append(cur)
            incheck = False
            s = s[2:].strip()
            if not s:
                continue
        if s == 'check:':
            cur['check'] = {}
            incheck = True
            continue
        if ':' not in s:
            continue
        k, v = s.split(':', 1)
        k, v = k.strip(), v.strip()
        if v.startswith('"') and v.endswith('"') and len(v) > 1:
            v = v[1:-1]
        elif v.startswith("'") and v.endswith("'") and len(v) > 1:
            v = v[1:-1]
        else:
            try:
                v = int(v, 16) if v.lower().startswith('0x') else int(v)
            except ValueError:
                try:
                    v = float(v)
                except ValueError:
                    pass
        if incheck and indent >= 4:
            cur['check'][k] = v
        else:
            incheck = False
            cur[k] = v
    return claims

# --- binary access --------------------------------------------------------
def open_image():
    try:
        import pefile
    except ImportError:
        sys.exit("need pefile:  pip install pefile")
    path = os.environ.get('H7_EXE', 'h7w_unpacked.exe')
    if not os.path.exists(path):
        sys.exit(f"binary not found: {path}\n"
                 f"Set H7_EXE to your UPX-unpacked heaven7w.exe "
                 f"(upx -d heaven7w.exe -o h7w_unpacked.exe).")
    pe = pefile.PE(path)
    base = pe.OPTIONAL_HEADER.ImageBase
    secs = [(base + s.VirtualAddress, s.get_data()) for s in pe.sections]
    def read(va, n):
        for lo, data in secs:
            if lo <= va < lo + len(data):
                off = va - lo
                if off + n <= len(data):
                    return data[off:off + n]
        return None
    return read, path

# --- checks ---------------------------------------------------------------
def do_check(read, c):
    """Return (ok, detail). Each kind re-reads the binary."""
    kind = c.get('kind')
    addr = c.get('addr')
    if kind == 'f32':
        b = read(addr, 4)
        if b is None: return False, 'address not mapped'
        got = struct.unpack('<f', b)[0]
        exp = float(c['expect'])
        tol = float(c.get('tol', 1e-6))
        return (abs(got - exp) <= tol), f'{got!r} vs {exp!r}'
    if kind == 'u32':
        b = read(addr, 4)
        if b is None: return False, 'address not mapped'
        got = struct.unpack('<I', b)[0]
        return got == int(c['expect']), f'{got:#x} vs {int(c["expect"]):#x}'
    if kind == 'u16':
        b = read(addr, 2)
        if b is None: return False, 'address not mapped'
        got = struct.unpack('<H', b)[0]
        return got == int(c['expect']), f'{got:#x} vs {int(c["expect"]):#x}'
    if kind == 'u8':
        b = read(addr, 1)
        if b is None: return False, 'address not mapped'
        return b[0] == int(c['expect']), f'{b[0]:#x} vs {int(c["expect"]):#x}'
    if kind == 'bytes':
        exp = bytes.fromhex(str(c['expect']).replace(' ', ''))
        got = read(addr, len(exp))
        if got is None: return False, 'address not mapped'
        return got == exp, f'{got.hex()} vs {exp.hex()}'
    if kind == 'table_count':
        # count non-zero-keyed entries until a zero key
        es = int(c.get('entry_size', 8))
        n = 0
        while n < 4096:
            b = read(addr + n * es, 4)
            if b is None: break
            if struct.unpack('<I', b)[0] == 0: break
            n += 1
        return n == int(c['expect']), f'{n} vs {c["expect"]}'
    if kind == 'table_entry':
        es = int(c.get('entry_size', 8))
        i = int(c['index'])
        b = read(addr + i * es, es)
        if b is None: return False, 'address not mapped'
        key, val = struct.unpack('<II', b[:8])
        ek, ev = int(c['expect_key']), int(c['expect_value'])
        return (key == ek and val == ev), f'({key:#x},{val:#x}) vs ({ek:#x},{ev:#x})'
    if kind == 'dispatch16':
        # 16-bit offset table: handler = image_base + word[addr + op*2]
        b = read(addr + int(c['op']) * 2, 2)
        if b is None: return False, 'address not mapped'
        off = struct.unpack('<H', b)[0]
        got = 0x400000 + off
        return got == int(c['expect']), f'{got:#x} vs {int(c["expect"]):#x}'
    return False, f'unknown check kind {kind!r}'

def main():
    path = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
        os.path.dirname(__file__), '..', 'claims.yaml')
    claims = load_claims(path)
    read, binpath = open_image()
    print(f"binary : {binpath}")
    print(f"claims : {path}  ({len(claims)} claims)\n")

    npass = nfail = 0
    unchecked = []
    for c in claims:
        cls = str(c.get('evidence', '?')).upper()
        cid = c.get('id', '<no id>')
        if cls == 'BYTES' and 'check' in c:
            ok, detail = do_check(read, c['check'])
            print(f"  [{'PASS' if ok else 'FAIL'}] {cid}")
            if not ok:
                print(f"         {c.get('statement','')}")
                print(f"         got {detail}")
                nfail += 1
            else:
                npass += 1
        else:
            unchecked.append((cls, cid, c.get('locator', '')))

    print(f"\nBYTES claims: {npass} passed, {nfail} failed")
    if unchecked:
        print(f"\nUNCHECKED ({len(unchecked)}) — not machine-verifiable, see METHODOLOGY.md:")
        for cls, cid, loc in unchecked:
            print(f"  [{cls:6}] {cid}{('  @ ' + str(loc)) if loc else ''}")
        infer = [u for u in unchecked if u[0] == 'INFER']
        if infer:
            print(f"\n  of which {len(infer)} are INFER (hypotheses, NOT evidence):")
            for _, cid, _ in infer:
                print(f"    - {cid}")
    return 1 if nfail else 0

if __name__ == '__main__':
    sys.exit(main())
