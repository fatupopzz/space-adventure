# Genera los modelos de la Luna: el "set de filmacion" de la llegada a la Luna (una broma con la teoria
# de que fue falsa: al final se ve que el set esta en la Luna de verdad).
#   set_luna.obj:    el suelo lunar con crateres y huellas, el modulo lunar (el Eagle), el asta de la bandera,
#                    la camara de cine con sus rieles, la silla del director y las cosas del estudio que se
#                    pueden quitar (partes aparte): ventilador, cafe, reflector, microfono y letrero (SALIDA).
#   bandera.obj:     la tela de la bandera, con el origen arriba en el asta (para que se mueva con el ventilador).
#   astronauta.obj:  el actor de astronauta (chibi): traje blanco, mochila y visor dorado (sin cara).
#   director.obj:    el director: boina, bigote, bufanda roja y megafono. Con las caras de siempre.
# Todos miran hacia -Z, origen en el piso.
# Uso: python3 herramientas/generar_set_luna.py

import math, random
from modelo3d import *

random.seed(1969)

def mtl(*materiales):
    """Cada material: (nombre, Kd, Ke) o (nombre, Kd, Ke, d)."""
    out = ["# Materiales"]
    for nombre, kd, ke, *d in materiales:
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), "Ks 0.2 0.2 0.2", "Ns 24"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
        if d:
            out.append(f"d {d[0]:.2f}")
    return "\n".join(out) + "\n"

def caja_redonda(vx, parte, mat, i0, i1, j0, j1, k0, k1, r=1.0, espejo=False):
    for i in range(i0, i1 + 1):
        for j in range(j0, j1 + 1):
            for k in range(k0, k1 + 1):
                d = 0.0
                for c, a, b in ((i + 0.5, i0, i1 + 1), (j + 0.5, j0, j1 + 1), (k + 0.5, k0, k1 + 1)):
                    fuera = max(a + r - c, 0.0, c - (b - r))
                    d += fuera * fuera
                if d <= r * r:
                    vx.put(i, j, k, parte, mat, espejo)

def elipsoide(vx, parte, mat, c, r, si=None):
    for i in range(math.floor(c[0] - r[0]) - 1, math.ceil(c[0] + r[0]) + 1):
        for j in range(math.floor(c[1] - r[1]) - 1, math.ceil(c[1] + r[1]) + 1):
            for k in range(math.floor(c[2] - r[2]) - 1, math.ceil(c[2] + r[2]) + 1):
                if sum(((p + 0.5 - q) / s) ** 2 for p, q, s in zip((i, j, k), c, r)) <= 1:
                    m = si(i, j, k) if si else mat
                    if m:
                        vx.put(i, j, k, parte if m == mat else f"{parte}_{m}", m)

def palo(vx, parte, mat, a, b, grueso=0):
    """Una linea de voxels de `a` a `b` (en metros), de `grueso` voxels de radio."""
    u = vx.u
    pasos = int(max(abs(b[q] - a[q]) for q in range(3)) / u * 2) + 1
    for s in range(pasos + 1):
        t = s / pasos
        p = [a[q] + (b[q] - a[q]) * t for q in range(3)]
        i, j, k = (math.floor(p[q] / u) for q in range(3))
        for di in range(-grueso, grueso + 1):
            for dj in range(-grueso, grueso + 1):
                for dk in range(-grueso, grueso + 1):
                    vx.put(i + di, j + dj, k + dk, parte, mat)

def cilindro(vx, parte, mat, centro, radio, y0, y1):
    """Un cilindro vertical (en metros)."""
    u = vx.u
    cx, cz = centro
    for i in range(math.floor((cx - radio) / u) - 1, math.ceil((cx + radio) / u) + 1):
        for k in range(math.floor((cz - radio) / u) - 1, math.ceil((cz + radio) / u) + 1):
            if ((i + 0.5) * u - cx) ** 2 + ((k + 0.5) * u - cz) ** 2 <= radio * radio:
                for j in range(math.floor(y0 / u), math.ceil(y1 / u)):
                    vx.put(i, j, k, parte, mat)

# =====================================================================
# El set
# =====================================================================
LEM = (2.8, 3.2)
BANDERA = (-1.2, 2.2)
ALTO_ASTA = 1.6
m = Modelo()

# El suelo lunar: una malla de alturas (no voxels: asi no quedan grietas entre caras de distinto tamano).
# Plano en el set y, desde 8 m, se curva hacia abajo como un planeta chiquito: el horizonte de la Luna esta
# cerca, y asi el borde del suelo nunca se ve. Con crateres (hondonada y borde levantado) lejos del set.
R_SUELO, CURVA, PLANO = 36.0, 30.0, 8.0
crateres = [(random.uniform(-30, 30), random.uniform(-30, 30), random.uniform(1.2, 4.0)) for _ in range(26)]
crateres = [c for c in crateres if math.hypot(c[0], c[1]) > PLANO + c[2] + 1]
def altura(x, z):
    r = math.hypot(x, z)
    y = -max(0.0, r - PLANO) ** 2 / (2 * CURVA)
    for (cx, cz, cr) in crateres:
        d = math.hypot(x - cx, z - cz) / cr
        if d < 1:
            y -= 0.3 * cr * (1 - d * d)
        y += 0.12 * cr * math.exp(-((d - 1) / 0.25) ** 2)
    return y
def material(x, z):
    for (cx, cz, cr) in crateres:
        d = math.hypot(x - cx, z - cz) / cr
        if d < 0.8:
            return "regolito_oscuro"
        if d < 1.2:
            return "regolito_claro"
    return "regolito"
PASO = 0.6
n = int(R_SUELO / PASO)
partes = {}
for a in range(-n, n):
    for b in range(-n, n):
        x0, z0, x1, z1 = a * PASO, b * PASO, (a + 1) * PASO, (b + 1) * PASO
        xc, zc = (x0 + x1) / 2, (z0 + z1) / 2
        if math.hypot(xc, zc) > R_SUELO:
            continue
        mat = material(xc, zc)
        if mat not in partes:
            partes[mat] = m.part(f"suelo_{mat}", mat)
        pts = [(x, altura(x, z), z) for x, z in ((x0, z0), (x1, z0), (x1, z1), (x0, z1))]
        partes[mat].quad(pts, (xc, altura(xc, zc) - 5.0, zc))

v = Voxels(0.05)
# Las huellas: del modulo lunar a donde para el astronauta y a la bandera (un rectangulito oscuro cada paso)
def huellas(desde, hasta, n):
    for s in range(n):
        t = (s + 0.5) / n
        x, z = desde[0] + (hasta[0] - desde[0]) * t, desde[1] + (hasta[1] - desde[1]) * t
        lado = 0.08 if s % 2 else -0.08
        nx, nz = -(hasta[1] - desde[1]), hasta[0] - desde[0]
        l = math.hypot(nx, nz)
        x, z = x + nx / l * lado, z + nz / l * lado
        i0, k0 = math.floor(x / 0.05), math.floor(z / 0.05)
        v.box("huellas", "huella", i0 - 1, i0, 0, 0, k0 - 2, k0 + 1)
huellas((2.4, 2.6), (0.4, 1.5), 12)
huellas((0.2, 1.6), (-0.9, 2.1), 7)

# El modulo lunar (el Eagle): patas con platos, la etapa de descenso dorada y la de ascenso gris
lx, lz = LEM
for sx in (-1, 1):
    for sz in (-1, 1):
        pie = (lx + sx * 1.15, 0.0, lz + sz * 1.15)
        cuerpo = (lx + sx * 0.6, 0.7, lz + sz * 0.6)
        palo(v, "lem_patas", "plata", cuerpo, (pie[0], 0.1, pie[2]), 1)
        cilindro(v, "lem_patas", "plata", (pie[0], pie[2]), 0.16, 0.0, 0.08)
for i in range(math.floor((lx - 0.75) / 0.05), math.ceil((lx + 0.75) / 0.05)):
    for k in range(math.floor((lz - 0.75) / 0.05), math.ceil((lz + 0.75) / 0.05)):
        x, _, z = v.centro(i, 0, k)
        if abs(x - lx) + abs(z - lz) <= 1.05:
            for j in range(10, 25):
                arrugas = "oro_lem_oscuro" if (i * 3 + j * 5 + k) % 7 == 0 else "oro_lem"
                v.put(i, j, k, f"lem_descenso_{arrugas}", arrugas)
for i in range(math.floor((lx - 0.6) / 0.05), math.ceil((lx + 0.6) / 0.05)):
    for k in range(math.floor((lz - 0.5) / 0.05), math.ceil((lz + 0.5) / 0.05)):
        x, _, z = v.centro(i, 0, k)
        for j in range(25, 41):
            esquina = abs(x - lx) > 0.45 and abs(z - lz) > 0.35 and j > 36
            if not esquina:
                v.put(i, j, k, "lem_ascenso", "gris_lem")
# Las ventanitas triangulares del frente (-Z), la escotilla y la antena
k_frente = math.floor((lz - 0.5) / 0.05) - 1
for lado in (-1, 1):
    for r in range(4):
        for c in range(4 - r):
            v.put(math.floor((lx + lado * (0.2 + c * 0.05)) / 0.05), 37 - r, k_frente, "lem_ventanas", "vidrio_lem")
v.box("lem_escotilla", "gris_oscuro", math.floor((lx - 0.15) / 0.05), math.floor((lx + 0.15) / 0.05), 27, 33, k_frente, k_frente)
palo(v, "lem_antena", "plata", (lx + 0.3, 2.0, lz), (lx + 0.5, 2.45, lz + 0.1))
cilindro(v, "lem_antena", "plata", (lx + 0.5, lz + 0.1), 0.18, 2.45, 2.5)
# La escalera en la pata de enfrente
for j in range(2, 10):
    v.box("lem_escalera", "plata", math.floor((lx - 0.85) / 0.05), math.floor((lx - 0.7) / 0.05), j, j, math.floor((lz - 0.9) / 0.05), math.floor((lz - 0.9) / 0.05))

# El asta de la bandera, con la varilla de arriba (la de verdad la tenia: por eso la bandera se ve extendida)
bx, bz = BANDERA
cilindro(v, "asta", "plata", (bx, bz), 0.025, 0.0, ALTO_ASTA)
palo(v, "asta", "plata", (bx, ALTO_ASTA - 0.03, bz), (bx + 0.82, ALTO_ASTA - 0.03, bz))

# La camara de cine sobre su tripie, en rieles, viendo al set (+Z)
cx, cz = 0.0, -3.6
for a in (0.0, 2.1, 4.2):
    palo(v, "tripie", "negro_mate", (cx, 1.05, cz), (cx + 0.45 * math.sin(a), 0.0, cz + 0.45 * math.cos(a) - 0.1), 0)
v.box("camara_cine", "negro_mate", math.floor((cx - 0.15) / 0.05), math.floor((cx + 0.15) / 0.05) - 1, 21, 27, math.floor((cz - 0.3) / 0.05), math.floor((cz + 0.25) / 0.05))
cilindro(v, "camara_lente", "gris_oscuro", (cx, cz + 0.4), 0.09, 1.15, 1.32)
v.box("camara_lente", "gris_oscuro", -2, 1, 22, 26, math.floor((cz + 0.25) / 0.05), math.floor((cz + 0.45) / 0.05))
for dz in (-0.18, 0.12):
    for i in range(-1, 1):
        for j in range(27, 36):
            for k in range(math.floor((cz + dz - 0.2) / 0.05), math.floor((cz + dz + 0.2) / 0.05)):
                y, z = (j + 0.5) * 0.05, (k + 0.5) * 0.05
                if (y - 1.55) ** 2 + (z - (cz + dz)) ** 2 <= 0.17 ** 2:
                    v.put(i, j, k, "camara_rollos", "negro_mate")
for x in (-0.35, 0.35):
    v.box("rieles", "plata", math.floor(x / 0.05), math.floor(x / 0.05), 0, 0, math.floor((cz - 1.0) / 0.05), math.floor((cz + 1.0) / 0.05))
for z in range(-9, 10, 3):
    v.box("rieles_durmientes", "madera_set", -8, 7, 0, 0, math.floor((cz + z * 0.1) / 0.05), math.floor((cz + z * 0.1) / 0.05))

# La silla del director (con su claqueta en el asiento)
sx, sz = -2.3, -3.6
for dx in (-0.22, 0.22):
    palo(v, "silla", "madera_set", (sx + dx, 0.0, sz - 0.2), (sx + dx, 0.95, sz + 0.2), 0)
    palo(v, "silla", "madera_set", (sx + dx, 0.0, sz + 0.2), (sx + dx, 0.5, sz - 0.2), 0)
v.box("silla_lona", "lona_roja", math.floor((sx - 0.22) / 0.05), math.floor((sx + 0.22) / 0.05), 10, 10, math.floor((sz - 0.2) / 0.05), math.floor((sz + 0.15) / 0.05))
v.box("silla_lona", "lona_roja", math.floor((sx - 0.22) / 0.05), math.floor((sx + 0.22) / 0.05), 13, 18, math.floor((sz + 0.2) / 0.05), math.floor((sz + 0.2) / 0.05))
v.box("claqueta", "negro_mate", math.floor((sx - 0.12) / 0.05), math.floor((sx + 0.1) / 0.05), 11, 11, math.floor((sz - 0.15) / 0.05), math.floor((sz + 0.05) / 0.05))
for i in range(math.floor((sx - 0.12) / 0.05), math.floor((sx + 0.1) / 0.05) + 1):
    if i % 2 == 0:
        v.put(i, 12, math.floor((sz - 0.15) / 0.05), "claqueta_rayas", "blanco_set")

# ---------- las cosas del estudio que se pueden quitar ----------
# El ventilador, soplandole a la bandera (+X)
fx, fz = -2.3, 2.1
cilindro(v, "ventilador", "gris_oscuro", (fx, fz), 0.2, 0.0, 0.05)
cilindro(v, "ventilador", "gris_oscuro", (fx, fz), 0.03, 0.05, 1.0)
for j in range(16, 28):
    for k in range(math.floor((fz - 0.32) / 0.05), math.ceil((fz + 0.32) / 0.05)):
        y, z = (j + 0.5) * 0.05, (k + 0.5) * 0.05
        r = math.hypot(y - 1.1, z - fz)
        if r <= 0.3:
            i = math.floor(fx / 0.05) + 1
            if r > 0.26 or abs(y - 1.1) < 0.025 or abs(z - fz) < 0.025:
                v.put(i + 1, j, k, "ventilador_rejilla", "plata")
            if r < 0.24 and (math.atan2(y - 1.1, z - fz) * 3 / math.pi) % 2 < 1:
                v.put(i, j, k, "ventilador_aspas", "aspas")
# El cafe: una mesita plegable con un termo y una taza
tx, tz = 1.0, 0.7
cilindro(v, "cafe_mesa_pata", "plata", (tx, tz), 0.03, 0.0, 0.55)
v.box("cafe_mesa", "madera_set", math.floor((tx - 0.25) / 0.05), math.floor((tx + 0.25) / 0.05), 11, 11, math.floor((tz - 0.2) / 0.05), math.floor((tz + 0.2) / 0.05))
cilindro(v, "cafe_taza", "blanco_set", (tx + 0.1, tz - 0.05), 0.05, 0.6, 0.72)
cilindro(v, "cafe_liquido", "cafe", (tx + 0.1, tz - 0.05), 0.035, 0.72, 0.73)
cilindro(v, "cafe_termo", "termo", (tx - 0.1, tz), 0.06, 0.6, 0.9)
# El reflector del estudio: tripie y una lampara enorme viendo al set
rx, rz = 3.8, -0.8
for a in (0.0, 2.1, 4.2):
    palo(v, "reflector", "negro_mate", (rx, 1.4, rz), (rx + 0.5 * math.sin(a), 0.0, rz + 0.5 * math.cos(a)), 0)
for i in range(math.floor((rx - 0.35) / 0.05), math.ceil((rx + 0.35) / 0.05)):
    for j in range(28, 42):
        for k in range(math.floor((rz - 0.35) / 0.05), math.ceil((rz + 0.35) / 0.05)):
            x, y, z = v.centro(i, j, k)
            dx, dz = x - rx, z - rz
            # La cabeza de la lampara (un cubo) inclinada hacia el set: su cara de luz mira a (-X, +Z).
            frente = -dx * 0.75 + dz * 0.66
            if abs(y - 1.75) < 0.33 and abs(dx * 0.66 + dz * 0.75) < 0.33 and abs(frente) < 0.25:
                v.put(i, j, k, "reflector_lampara" if frente > 0.18 else "reflector", "foco" if frente > 0.18 else "negro_mate")
# El microfono de jirafa, colgando sobre el astronauta
palo(v, "microfono", "negro_mate", (-0.9, 0.0, -1.0), (-0.9, 2.2, -1.0), 0)
palo(v, "microfono_jirafa", "plata", (-1.1, 2.2, -1.2), (0.1, 2.35, 1.3), 0)
palo(v, "microfono", "negro_mate", (0.1, 2.35, 1.3), (0.1, 2.1, 1.3), 0)
for i in range(-1, 4):
    for j in range(37, 43):
        for k in range(24, 29):
            if (i + 0.5 - 2.5) ** 2 / 4 + (j + 0.5 - 40) ** 2 / 9 + (k + 0.5 - 26.5) ** 2 / 4 <= 1:
                v.put(i, j, k, "microfono_esponja", "esponja")
# El letrero de SALIDA (verde, con letras blancas que se leen desde la camara)
LETRAS = {
    "S": ["###", "#..", "###", "..#", "###"], "A": [".#.", "#.#", "###", "#.#", "#.#"], "L": ["#..", "#..", "#..", "#..", "###"],
    "I": ["###", ".#.", ".#.", ".#.", "###"], "D": ["##.", "#.#", "#.#", "#.#", "##."],
}
ex_, ez_ = -3.6, 3.8
i_centro, k_letrero = math.floor(ex_ / 0.05), math.floor(ez_ / 0.05)
for dx in (-0.55, 0.55):
    palo(v, "letrero_postes", "gris_oscuro", (ex_ + dx, 0.0, ez_ + 0.05), (ex_ + dx, 1.7, ez_ + 0.05), 0)
v.box("letrero", "verde_salida", i_centro - 13, i_centro + 12, 33, 41, k_letrero, k_letrero + 1)
for c, letra in enumerate("SALIDA"):
    for fila, renglon in enumerate(LETRAS[letra]):
        for col, ch in enumerate(renglon):
            if ch == "#":
                # Visto desde -Z, la derecha es -X: las letras van de +X a -X.
                v.put(i_centro + 11 - (c * 4 + col), 39 - fila, k_letrero - 1, "letrero_letras", "blanco_luz")
v.mesh(m)
m.write("set_luna", """# El "set de filmacion" de la llegada a la Luna (una broma: esta en la Luna de verdad). Origen al centro del set,
# piso en y = 0. Modulo lunar en (2.8, 3.2), asta de la bandera en (-1.2, 2.2) de 1.6 de alto (la tela es bandera.obj),
# camara de cine en (0, -3.6) viendo a +Z, silla del director en (-2.3, -3.6) y huellas en el piso.
# Cosas del estudio que se pueden ocultar: ventilador (-2.3, 2.1), cafe (1.0, 0.7), reflector (3.8, -0.8),
# microfono (jirafa sobre (0.1, 1.3)) y letrero de SALIDA (-3.6, 3.8).
""", mtl(("regolito", (0.62, 0.62, 0.64), None), ("regolito_claro", (0.74, 0.74, 0.76), None), ("regolito_oscuro", (0.42, 0.42, 0.45), None),
         ("huella", (0.34, 0.34, 0.37), None), ("oro_lem", (1.0, 0.76, 0.25), (0.08, 0.05, 0.0)), ("oro_lem_oscuro", (0.80, 0.55, 0.15), None),
         ("gris_lem", (0.72, 0.72, 0.75), None), ("gris_oscuro", (0.30, 0.30, 0.34), None), ("vidrio_lem", (0.10, 0.12, 0.20), None),
         ("plata", (0.85, 0.86, 0.90), None), ("negro_mate", (0.10, 0.10, 0.12), None), ("madera_set", (0.55, 0.38, 0.22), None),
         ("lona_roja", (0.80, 0.12, 0.15), None), ("blanco_set", (0.95, 0.95, 0.95), None), ("aspas", (0.55, 0.75, 0.90), None),
         ("cafe", (0.30, 0.16, 0.08), None), ("termo", (0.25, 0.45, 0.80), None), ("foco", (1.0, 0.97, 0.85), (1.0, 0.95, 0.8)),
         ("esponja", (0.45, 0.45, 0.48), None), ("verde_salida", (0.10, 0.60, 0.25), (0.03, 0.2, 0.06)),
         ("blanco_luz", (1.0, 1.0, 1.0), (0.8, 0.8, 0.8))))

# ---------- la tela de la bandera ----------
# Origen arriba, en el asta; cuelga hacia -Y y se extiende hacia +X, en el plano z = 0 (de frente a -Z).
v = Voxels(0.05, origen=(0.0, -0.5, -0.01))
for i in range(0, 16):
    for j in range(0, 10):
        canton = i < 7 and j >= 5
        if canton:
            mat = "estrella_bandera" if (i + j) % 2 == 0 else "azul_bandera"
        else:
            mat = "rojo_bandera" if j % 2 == 0 else "blanco_bandera"
        v.put(i, j, 0, f"tela_{mat}", mat)
m = Modelo()
v.mesh(m)
m.write("bandera", """# La tela de la bandera (voxel de 0.05): franjas rojas y blancas y el canton azul. Origen arriba, junto al asta;
# cuelga hacia -Y y se extiende hacia +X.
""", mtl(("rojo_bandera", (0.80, 0.12, 0.16), None), ("blanco_bandera", (0.95, 0.95, 0.95), None),
         ("azul_bandera", (0.12, 0.18, 0.48), None), ("estrella_bandera", (0.85, 0.88, 0.95), None)))

# =====================================================================
# El astronauta (el actor)
# =====================================================================
U = 0.04
vx = Voxels(U)
caja_redonda(vx, "botas", "gris_traje", 1, 4, 0, 2, -4, 2, 1.0, espejo=True)
caja_redonda(vx, "piernas", "traje", 1, 4, 3, 8, -3, 2, 1.2, espejo=True)
caja_redonda(vx, "torso", "traje", 0, 5, 9, 17, -4, 3, 1.6, espejo=True)
vx.box("panel", "gris_traje", -2, 1, 11, 14, -5, -5)
vx.box("panel_luces", "rojo_luz", -2, -2, 13, 13, -6, -6)
vx.box("panel_luces", "rojo_luz", 0, 0, 12, 12, -6, -6)
caja_redonda(vx, "mochila", "traje_sucio", 0, 5, 9, 19, 4, 8, 1.2, espejo=True)
caja_redonda(vx, "brazos", "traje", 6, 7, 9, 16, -2, 1, 0.8, espejo=True)
caja_redonda(vx, "guantes", "gris_traje", 6, 7, 7, 9, -2, 1, 0.6, espejo=True)
vx.box("parche", "rojo_luz", 6, 7, 15, 16, -3, -3)
def casco(i, j, k):
    if k + 0.5 < -3.5 and abs(j + 0.5 - 23.5) < 4.5 and abs(i + 0.5) < 5.5:
        return "visor"
    return "traje"
elipsoide(vx, "casco", "traje", (0.0, 23.0, 0.0), (7.0, 7.0, 7.0), si=casco)
m = Modelo()
vx.mesh(m)
m.write("astronauta", """# El actor de astronauta (voxel de 0.04): traje blanco, mochila, panel del pecho y visor dorado (no se le ve la
# cara). Mira hacia -Z, origen en el piso.
""", mtl(("traje", (0.92, 0.92, 0.90), None), ("traje_sucio", (0.80, 0.80, 0.78), None), ("gris_traje", (0.55, 0.56, 0.60), None),
         ("rojo_luz", (0.9, 0.2, 0.2), (0.4, 0.05, 0.05)), ("visor", (1.0, 0.75, 0.25), (0.25, 0.16, 0.02))))

# =====================================================================
# El director
# =====================================================================
vx = Voxels(U)
caja_redonda(vx, "zapatos", "cafe_zapato", 1, 3, 0, 1, -4, 1, 0.7, espejo=True)
vx.box("pantalon", "pantalon", 1, 3, 2, 8, -2, 1, espejo=True)
caja_redonda(vx, "camisa", "camisa_director", 0, 5, 9, 17, -3, 3, 1.4, espejo=True)
caja_redonda(vx, "bufanda", "bufanda", 0, 5, 16, 18, -4, 3, 1.0, espejo=True)
vx.box("bufanda", "bufanda", 1, 2, 11, 15, -5, -5)
vx.box("brazos", "camisa_director", 6, 7, 10, 16, -2, 1, espejo=True)
caja_redonda(vx, "manos", "piel", 6, 7, 8, 9, -2, 0, 0.6, espejo=True)
caja_redonda(vx, "cabeza", "piel", 0, 6, 19, 30, -6, 5, 3.0, espejo=True)
caja_redonda(vx, "pelo", "pelo_cafe", 6, 7, 21, 27, -2, 5, 1.0, espejo=True)
vx.box("pelo", "pelo_cafe", 0, 6, 21, 27, 6, 6, espejo=True)
# La boina negra, ladeada, con su colita
elipsoide(vx, "boina", "boina", (1.0, 31.0, 0.0), (8.0, 2.0, 7.5))
vx.box("boina", "boina", 0, 0, 33, 34, 0, 0)
# El bigote
vx.box("bigote", "pelo_cafe", 0, 3, 22, 22, -7, -7, espejo=True)
vx.box("bigote", "pelo_cafe", 3, 3, 21, 21, -7, -7, espejo=True)
# El megafono en la mano derecha (-X), apuntando al frente
for t in range(9):
    r = 1 + t // 3
    for di in range(-r, r + 1):
        for dj in range(-r, r + 1):
            if di * di + dj * dj <= r * r:
                vx.put(-7 + di, 10 + dj, -3 - t, "megafono" if t < 8 else "megafono_borde", "blanco_set" if t < 8 else "rojo_megafono")
m = Modelo()
vx.mesh(m)
expresiones(m, z_frente=-6 * U, px=0.021, x_centro=0.0, y_base=22.6 * U)
m.write("director", """# El director (voxel de 0.04): boina negra ladeada, bigote, bufanda roja y megafono en la mano derecha. Caras de
# siempre. Mira hacia -Z, origen en el piso.
""", mtl(("cafe_zapato", (0.35, 0.22, 0.12), None), ("pantalon", (0.30, 0.26, 0.22), None), ("camisa_director", (0.55, 0.70, 0.88), None),
         ("bufanda", (0.85, 0.15, 0.18), None), ("piel", (1.0, 0.84, 0.72), None), ("pelo_cafe", (0.35, 0.22, 0.14), None),
         ("boina", (0.10, 0.10, 0.12), None), ("blanco_set", (0.95, 0.95, 0.95), None), ("rojo_megafono", (0.85, 0.15, 0.18), None)) + MTL_CARA)
