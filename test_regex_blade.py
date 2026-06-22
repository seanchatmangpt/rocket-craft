import re

txt = open('/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/usd/SM_Blade_Left.usda').read()
GROUP_RE = re.compile(r'def Xform "(prim_[^"]+)"\s*\{(.*?)\n        \}', re.DOTALL)
TRANS_RE = re.compile(r'double3 xformOp:translate = \(([^)]+)\)')
SCALE_RE = re.compile(r'double3 xformOp:scale = \(([^)]+)\)')

for m in GROUP_RE.finditer(txt):
    gblock = m.group(2)
    print("Group:", m.group(1))
    print("  gtr:", TRANS_RE.search(gblock).group(1) if TRANS_RE.search(gblock) else None)
    print("  gsc:", SCALE_RE.search(gblock).group(1) if SCALE_RE.search(gblock) else None)
