# Piezas comunes para generar los .obj voxel del proyecto (Miku y la nave).
# Todos los modelos: Y arriba, miran hacia -Z, caras CCW vistas desde afuera, formato "f v//vn".
# El voxel (i, j, k) ocupa [i*u, (i+1)*u] x [j*u, (j+1)*u] x [k*u, (k+1)*u]; el espejo de i es -1-i.

import os

MODELOS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "modelos")


# ---------- vectores ----------
def add(a, b): return (a[0]+b[0], a[1]+b[1], a[2]+b[2])
def sub(a, b): return (a[0]-b[0], a[1]-b[1], a[2]-b[2])
def mul(a, s): return (a[0]*s, a[1]*s, a[2]*s)
def dot(a, b): return a[0]*b[0]+a[1]*b[1]+a[2]*b[2]
def cross(a, b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])
def norm(a):
    l = dot(a, a) ** 0.5
    return (a[0]/l, a[1]/l, a[2]/l) if l > 1e-12 else (0.0, 1.0, 0.0)


class Part:
    def __init__(self, name, mat, comment=None):
        self.name, self.mat, self.comment = name, mat, comment
        self.v, self.f = [], []
        self.uv = []  # una coordenada de textura por vertice, si la parte tiene textura

    def vert(self, p, uv=None):
        self.v.append(p)
        if uv is not None:
            self.uv.append(uv)
        return len(self.v) - 1

    def tri(self, a, b, c, inside):
        """Agrega un triangulo con la cara hacia afuera de `inside` (CCW visto desde afuera)."""
        pa, pb, pc = self.v[a], self.v[b], self.v[c]
        fn = cross(sub(pb, pa), sub(pc, pa))
        if dot(fn, fn) < 1e-16:
            return
        cen = mul(add(add(pa, pb), pc), 1/3)
        if dot(fn, sub(cen, inside)) < 0:
            b, c = c, b
        self.f.append((a, b, c))

    def quad(self, pts, inside, uvs=None):
        ids = [self.vert(p, uvs[k] if uvs else None) for k, p in enumerate(pts)]
        self.tri(ids[0], ids[1], ids[2], inside); self.tri(ids[0], ids[2], ids[3], inside)


class Modelo:
    def __init__(self):
        self.parts = []

    def part(self, *a, **k):
        p = Part(*a, **k); self.parts.append(p); return p

    def write(self, nombre, header, mtl):
        """Normales planas (una por cara), como saturno_escenario.obj. Las partes con
        coordenadas de textura escriben `vt` y caras `v/vt/vn`."""
        lines = [header.rstrip("\n"), f"mtllib {nombre}.mtl", ""]
        vbase, nbase, tbase = 0, 0, 0
        for p in self.parts:
            if not p.f:
                continue
            con_uv = len(p.uv) == len(p.v)
            lines += [f"o {p.name}"] + ([f"# {p.comment}"] if p.comment else []) + [f"usemtl {p.mat}", "s off"]
            lines += [f"v {x:.4f} {y:.4f} {z:.4f}" for x, y, z in p.v]
            if con_uv:
                lines += [f"vt {u:.4f} {v:.4f}" for u, v in p.uv]
            for a, b, c in p.f:
                fn = norm(cross(sub(p.v[b], p.v[a]), sub(p.v[c], p.v[a])))
                lines.append(f"vn {fn[0]:.4f} {fn[1]:.4f} {fn[2]:.4f}")
            for k, (a, b, c) in enumerate(p.f):
                n = nbase + k + 1
                if con_uv:
                    lines.append(f"f {a+1+vbase}/{a+1+tbase}/{n} {b+1+vbase}/{b+1+tbase}/{n} {c+1+vbase}/{c+1+tbase}/{n}")
                else:
                    lines.append(f"f {a+1+vbase}//{n} {b+1+vbase}//{n} {c+1+vbase}//{n}")
            nbase += len(p.f); vbase += len(p.v)
            if con_uv:
                tbase += len(p.uv)
            lines.append("")
        os.makedirs(MODELOS, exist_ok=True)
        open(os.path.join(MODELOS, nombre + ".obj"), "w").write("\n".join(lines))
        open(os.path.join(MODELOS, nombre + ".mtl"), "w").write(mtl)
        caras = {}
        for p in self.parts:
            if p.name.startswith("cara_"):
                emo = p.name.split("_")[1]
                caras[emo] = caras.get(emo, 0) + len(p.f)
        cuerpo = sum(len(p.f) for p in self.parts if not p.name.startswith("cara_"))
        print(f"{nombre}: {len([p for p in self.parts if p.f])} partes, {sum(len(p.f) for p in self.parts)} triangulos "
              f"(cuerpo {cuerpo}, max en pantalla {cuerpo + max(caras.values(), default=0)})")


# ---------- voxels ----------
DIRS = [(0, -1), (0, 1), (1, -1), (1, 1), (2, -1), (2, 1)]  # (eje, signo)

class Voxels:
    def __init__(self, u, origen=(0.0, 0.0, 0.0)):
        self.u, self.o = u, origen
        self.v = {}  # (i, j, k) -> (parte, material)

    def put(self, i, j, k, parte, mat, espejo=False):
        """parte puede llevar {lado}: der para +X, izq para el espejo."""
        self.v[(i, j, k)] = (parte.format(lado="der"), mat)
        if espejo:
            self.v[(-1 - i, j, k)] = (parte.format(lado="izq"), mat)

    def box(self, parte, mat, i0, i1, j0, j1, k0, k1, espejo=False, si=None):
        """Llena la caja (rangos inclusivos). si(i, j, k) filtra voxels."""
        for i in range(i0, i1 + 1):
            for j in range(j0, j1 + 1):
                for k in range(k0, k1 + 1):
                    if si is None or si(i, j, k):
                        self.put(i, j, k, parte, mat, espejo)

    def pattern(self, parte, mat, filas, i0, j_top, k, espejo=False):
        """Dibujo de pixeles ('#') en el plano XY, filas de arriba hacia abajo, en la capa k."""
        for r, fila in enumerate(filas):
            for c, ch in enumerate(fila):
                if ch == "#":
                    self.put(i0 + c, j_top - r, k, parte, mat, espejo)

    def remove(self, si):
        for key in [key for key in self.v if si(*key)]:
            del self.v[key]

    def centro(self, i, j, k):
        """Centro del voxel en el mundo."""
        u = self.u
        return (self.o[0] + (i + 0.5)*u, self.o[1] + (j + 0.5)*u, self.o[2] + (k + 0.5)*u)

    def centro_de(self, parte):
        cs = [self.centro(*key) for key, (p, _) in self.v.items() if p == parte]
        return tuple(sum(c[q] for c in cs) / len(cs) for q in range(3))

    def mesh(self, modelo, animadas=(), comentarios=None, fusionar=True, transform=None):
        """Convierte los voxels en partes del modelo: solo caras expuestas, y con fusionar=True las
        caras vecinas iguales se juntan en rectangulos (greedy meshing).
        Las partes animadas solo se tapan entre si, para que no queden huecos cuando se mueven.
        transform(p) mueve cada vertice (por ejemplo rot_y) para poner piezas giradas."""
        comentarios = comentarios or {}
        tf = transform or (lambda p: p)
        partes = {}
        for key, (p, mat) in self.v.items():
            info = partes.setdefault(p, [mat, set()])
            assert info[0] == mat, f"la parte {p} tiene dos materiales"
            info[1].add(key)
        estaticos = {key for key, (p, _) in self.v.items() if p not in animadas}
        u, o = self.u, self.o
        for nombre, (mat, celdas) in partes.items():
            part = modelo.part(nombre, mat, comentarios.get(nombre))
            ocup = celdas if nombre in animadas else estaticos
            for eje, s in DIRS:
                b_eje, c_eje = [e for e in range(3) if e != eje]
                capas = {}
                for cel in celdas:
                    vec = list(cel); vec[eje] += s
                    if tuple(vec) not in ocup:
                        capas.setdefault(cel[eje], set()).add((cel[b_eje], cel[c_eje]))
                for capa, pts in capas.items():
                    for b0, b1, c0, c1 in rectangulos(pts, fusionar):
                        plano = (capa + (1 if s > 0 else 0)) * u
                        esquinas = []
                        for bb, cc in ((b0, c0), (b1 + 1, c0), (b1 + 1, c1 + 1), (b0, c1 + 1)):
                            q = [0.0, 0.0, 0.0]
                            q[eje], q[b_eje], q[c_eje] = plano, bb*u, cc*u
                            esquinas.append(add(o, tuple(q)))
                        dentro = [0.0, 0.0, 0.0]
                        dentro[eje] = plano - s*u*0.5
                        dentro[b_eje], dentro[c_eje] = (b0 + b1 + 1)*u/2, (c0 + c1 + 1)*u/2
                        part.quad([tf(q) for q in esquinas], tf(add(o, tuple(dentro))))


def rot_y(phi, t=(0.0, 0.0, 0.0)):
    """Transformacion: gira phi radianes alrededor de Y y luego traslada a t."""
    import math
    c, s = math.cos(phi), math.sin(phi)
    return lambda p: (p[0]*c + p[2]*s + t[0], p[1] + t[1], -p[0]*s + p[2]*c + t[2])


def rectangulos(pts, fusionar=True):
    """Cubre un conjunto de celdas 2D con rectangulos (b0, b1, c0, c1), inclusivos."""
    if not fusionar:
        return [(b, b, c, c) for b, c in pts]
    libres, out = set(pts), []
    for b, c in sorted(pts, key=lambda q: (q[1], q[0])):
        if (b, c) not in libres:
            continue
        b1 = b
        while (b1 + 1, c) in libres:
            b1 += 1
        c1 = c
        while all((x, c1 + 1) in libres for x in range(b, b1 + 1)):
            c1 += 1
        for x in range(b, b1 + 1):
            for y in range(c, c1 + 1):
                libres.discard((x, y))
        out.append((b, b1, c, c1))
    return out


# ---------- caras: pixeles pegados al frente de la cabeza ----------
def pixeles(filas, x0, y_top):
    """Dibujo de pixeles ('#'), filas de arriba hacia abajo -> rectangulos (x0, y0, x1, y1) en pixeles."""
    out = []
    for r, fila in enumerate(filas):
        c = 0
        while c < len(fila):
            if fila[c] == "#":
                c1 = c
                while c1 + 1 < len(fila) and fila[c1 + 1] == "#":
                    c1 += 1
                out.append((x0 + c, y_top - r - 1, x0 + c1 + 1, y_top - r))
                c = c1 + 1
            else:
                c += 1
    return out

def espejo(rects):
    return rects + [(-x1, y0, -x0, y1) for x0, y0, x1, y1 in rects]

OJO = ["####"] * 6

CARAS = {
    # cada pieza: (nombre, material, rectangulos en pixeles; x = 0 al centro de la cara, y = 0 abajo de la cara)
    "feliz": [
        ("ojos", "ojo", espejo(pixeles(OJO, 2, 10))),
        ("rubor", "rubor", espejo(pixeles(["###", "###"], 5, 3))),
    ],
    "asustada": [
        ("ojos", "ojo", espejo(pixeles(["##", "##"], 3, 8))),
        ("boca", "boca", pixeles(["##", "##"], -1, 3)),
        ("sudor", "sudor", pixeles([".#", "##", "##"], 7, 11)),
    ],
    "espiral": [
        ("ojos", "ojo", espejo(pixeles(["#####", "....#", ".##.#", ".#..#", ".####"], 2, 10))),
        ("boca", "ojo", pixeles(["#.#.#", ".#.#."], -2, 3)),
    ],
    "enamorada": [
        ("corazon", "corazon", espejo(pixeles([".#.#.", "#####", "#####", ".###.", "..#.."], 2, 10))),
        ("rubor", "rubor", espejo(pixeles(["###", "###"], 5, 3))),
    ],
    "dormida": [
        ("ojos", "ojo", espejo(pixeles(["#...#", ".###."], 2, 7))),
        ("rubor", "rubor", espejo(pixeles(["###", "###"], 5, 3))),
    ],
    "enojado": [
        ("ojos", "ojo", espejo(pixeles(["####"] * 4, 2, 8))),
        ("cejas", "ojo", espejo(pixeles(["....##", "..##..", "##...."], 1, 12))),
        ("boca", "ojo", pixeles([".####.", "#....#"], -3, 3)),
    ],
    "guino": [
        # Un ojo abierto y el otro cerrado en arquito, con sonrisa y rubor.
        ("ojos", "ojo", pixeles(OJO, 2, 10) + [(-x1, y0, -x0, y1) for x0, y0, x1, y1 in pixeles(["#...#", ".###."], 1, 7)]),
        ("boca", "boca", pixeles(["#..#", ".##."], -2, 3)),
        ("rubor", "rubor", espejo(pixeles(["###", "###"], 5, 3))),
    ],
    "serio": [
        ("ojos", "ojo", espejo(pixeles(["####", "####", "####"], 2, 7))),
        ("boca", "ojo", pixeles(["####"], -2, 3)),
    ],
}

def expresiones(modelo, z_frente, px, x_centro, y_base, separacion=0.002, caras=None):
    """Pega las caras en el plano z = z_frente (el frente de la cabeza), un poco hacia -Z.
    px = tamano del pixel de la cara (medio voxel se ve bien). `caras` reemplaza el juego
    de expresiones chibi por otro con las mismas emociones."""
    for emo, piezas in (caras or CARAS).items():
        for k, (pieza, mat, rects) in enumerate(piezas):
            part = modelo.part(f"cara_{emo}_{pieza}", mat)
            z = z_frente - separacion*(k + 1)
            for x0, y0, x1, y1 in rects:
                X0, X1 = x_centro + x0*px, x_centro + x1*px
                Y0, Y1 = y_base + y0*px, y_base + y1*px
                part.quad([(X0, Y0, z), (X1, Y0, z), (X1, Y1, z), (X0, Y1, z)], (0.0, (Y0 + Y1)/2, z + 1.0))


MTL_CARA = """
newmtl ojo
Kd 0.05 0.04 0.07
Ks 0.3 0.3 0.3
Ns 32

newmtl boca
Kd 0.62 0.16 0.28
Ks 0 0 0
Ns 1

newmtl corazon
Kd 1.00 0.30 0.55
Ks 0 0 0
Ns 1
Ke 0.50 0.10 0.25

newmtl rubor
Kd 1.00 0.48 0.62
Ks 0 0 0
Ns 1
d 0.60

newmtl sudor
Kd 0.40 0.78 1.00
Ks 0.9 0.9 0.9
Ns 96
d 0.85
"""
