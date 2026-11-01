# Genera los modelos de la Gran Fuente de las Hadas (el Proyecto 2, ahora en voxels y rasterizado):
#   fuente_hadas.obj: la isla flotante con la fuente encima. Origen en el centro de la plaza, el piso en y = 0.
#                     La piscina hexagonal esta adelante (+Z), la Trifuerza del piso atras (-Z), donde se para Miku.
#   luka.obj:         Megurine Luka en chibi (la otra Vocaloid que sale de la fuente), con las caras de siempre.
#   navi.obj:         la hadita: un centro brillante y cuatro alas.
#   contenedor.obj:   el Contenedor de Corazon.
# Homenaje a The Legend of Zelda: Ocarina of Time (Nintendo); los modelos son propios.
# Uso: python3 herramientas/generar_fuente.py

import math, random
from modelo3d import *

random.seed(3)

def mtl(*materiales):
    """Cada material: (nombre, Kd, Ke) o (nombre, Kd, Ke, d)."""
    out = ["# Materiales"]
    for nombre, kd, ke, *d in materiales:
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), "Ks 0.25 0.25 0.25", "Ns 32"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
        if d:
            out.append(f"d {d[0]:.2f}")
    return "\n".join(out) + "\n"

def ruido(x, z, s):
    return math.sin(x * 1.3 + s) * math.sin(z * 1.1 + 2 * s) + 0.5 * math.sin(x * 2.9 - z * 2.3 + s)

def hexagono(x, z, r):
    """Si (x, z) esta dentro de un hexagono de radio (al vertice) r."""
    ax, az = abs(x), abs(z)
    return az <= r * math.sqrt(3) / 2 and ax * math.sqrt(3) / 2 + az / 2 <= r * math.sqrt(3) / 2

m = Modelo()

# ---------- la isla (voxel de 0.25) ----------
isla = Voxels(0.25)
R = 7.2
FLORES = []
for i in range(-34, 34):
    for k in range(-34, 34):
        x, _, z = isla.centro(i, 0, k)
        r = math.hypot(x, z) + 0.6 * ruido(x, z, 1.0)
        if r > R:
            continue
        # Hacia abajo se angosta como un cono irregular: la parte de abajo de una isla flotante.
        hondo = 1.5 + (R - r) * 0.9 + 0.8 * ruido(x * 0.7, z * 0.7, 4.0)
        for j in range(-int(hondo / 0.25), 0):
            y = (j + 0.5) * 0.25
            mat = "pasto" if j == -1 else ("tierra" if y > -1.2 else "roca")
            if mat == "roca" and ruido(x * 2, y * 2 + z, 7.0) > 0.6:
                mat = "roca_oscura"
            isla.put(i, j, k, mat, mat)
        # Flores sobre el pasto, lejos de la plaza (chiquitas: van en la rejilla fina)
        if r > 5.0 and random.random() < 0.12:
            FLORES.append((x + random.uniform(-0.1, 0.1), z + random.uniform(-0.1, 0.1), random.choice(["flor_rosa", "flor_blanca"])))
isla.mesh(m)
# Los nombres de las partes de la isla salen de su material; las flores van todas en una parte por color.

# ---------- cristales que cuelgan y cascadas (voxel de 0.1) ----------
fino = Voxels(0.1)
for n in range(9):
    a = n / 9 * 2 * math.pi + 0.3
    x, z = 5.6 * math.cos(a), 5.6 * math.sin(a)
    largo = 8 + (n * 5) % 9
    for j in range(largo):
        ancho = max(0, 2 - j // 4)
        for di in range(-ancho, ancho + 1):
            for dk in range(-ancho, ancho + 1):
                fino.put(round(x / 0.1) + di, -15 - j, round(z / 0.1) + dk, "cristales_colgantes", "cristal")
for n, a in enumerate((0.9, 2.5, 4.4)):
    x, z = 6.6 * math.cos(a), 6.6 * math.sin(a)
    for j in range(-40, 1):
        for di in range(-2, 2):
            fino.put(round(x / 0.1) + di, j, round(z / 0.1), "cascadas", "agua_cascada")
# Cristales parados en las orillas de la plaza, como en el Proyecto 2
for (x, z, alto) in ((-4.6, 2.6, 12), (4.4, 2.9, 10), (-4.2, -3.4, 9), (4.7, -3.0, 13), (-1.5, 4.9, 8), (2.0, 5.0, 11)):
    i0, k0 = round(x / 0.1), round(z / 0.1)
    for j in range(alto):
        ancho = 2 if j < alto - 3 else (1 if j < alto - 1 else 0)
        for di in range(-ancho, ancho + 1):
            for dk in range(-ancho, ancho + 1):
                fino.put(i0 + di, j, k0 + dk, "cristales", "cristal")
for (x, z, flor) in FLORES:
    i0, k0 = round(x / 0.1), round(z / 0.1)
    fino.put(i0, 0, k0, "tallos", "pasto")
    for di, dk in ((0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)):
        fino.put(i0 + di, 1, k0 + dk, flor, flor)
fino.mesh(m)

# ---------- la fuente (voxel de 0.1) ----------
f = Voxels(0.1)
PZ = 1.2       # centro de la piscina
for i in range(-45, 45):
    for k in range(-45, 45):
        x, _, z = f.centro(i, 0, k)
        dentro_plaza = abs(x) <= 4.2 and abs(z) <= 4.2
        if not dentro_plaza:
            continue
        en_piscina = hexagono(x, z - PZ, 2.3)
        borde = hexagono(x, z - PZ, 2.55) and not en_piscina
        if en_piscina:
            # El fondo de la piscina, mas abajo, y el agua arriba (transparente).
            f.put(i, -3, k, "fondo_piscina", "fondo_piscina")
            f.put(i, 0, k, "agua", "agua")
        elif borde:
            f.put(i, 0, k, "borde_oro", "oro")
            f.put(i, 1, k, "borde_oro", "oro")
        else:
            mat = "baldosa_a" if (i // 6 + k // 6) % 2 == 0 else "baldosa_b"
            f.put(i, 0, k, mat, mat)
# El estrado en terrazas en medio de la piscina, la flor de loto y el cuenco
for nivel, (r, alto) in enumerate(((1.1, 2), (0.8, 4), (0.55, 6))):
    for i in range(-12, 12):
        for k in range(-12, 12):
            x, _, z = f.centro(i, 0, k)
            if hexagono(x, z, r):
                for j in range(-2, alto):
                    mat = "oro" if j == alto - 1 and not hexagono(x, z, r - 0.1) else "baldosa_a"
                    f.put(i, j, k + round(PZ / 0.1), "estrado" if mat == "baldosa_a" else "estrado_oro", mat)
for n in range(8):
    a = n / 8 * 2 * math.pi
    for t in range(4):
        x, z = (0.25 + t * 0.07) * math.cos(a), (0.25 + t * 0.07) * math.sin(a)
        f.put(round(x / 0.1), 6 + (t // 2), round((z + PZ) / 0.1), "loto", "loto")
for i in range(-2, 2):
    for k in range(-2, 2):
        f.put(i, 6, k + round(PZ / 0.1), "cuenco", "obsidiana")
# La Trifuerza en el piso, atras, donde se para Miku: tres triangulos (apagados y encendidos)
TZ, LADO = -2.6, 0.9
def triangulo(cx, cz, lado, parte, mat, j):
    h = lado * math.sqrt(3) / 2
    for i in range(round(cx / 0.1) - 8, round(cx / 0.1) + 8):
        for k in range(round(cz / 0.1) - 8, round(cz / 0.1) + 8):
            x, _, z = f.centro(i, 0, k)
            dz = z - (cz - h / 3)  # desde la base
            if 0 <= dz <= h and abs(x - cx) <= (lado / 2) * (1 - dz / h):
                f.put(i, j, k, parte, mat)
centros = [(0.0, TZ + LADO * math.sqrt(3) / 4), (-LADO / 2, TZ - LADO * math.sqrt(3) / 4), (LADO / 2, TZ - LADO * math.sqrt(3) / 4)]
for n, (cx, cz) in enumerate(centros):
    triangulo(cx, cz, LADO, f"trifuerza_{n}", "oro", 1)
# (las encendidas van en otro Voxels: ocupan las mismas celdas, un poquito mas arriba)
# Las antorchas de cono, a los lados de la Trifuerza
for lado in (-1, 1):
    x0, z0 = lado * 3.3, -2.0
    for j in range(0, 12):
        r = 0.32 - j * 0.02
        for i in range(-4, 4):
            for k in range(-4, 4):
                x, _, z = f.centro(i, 0, k)
                if math.hypot(x, z) <= r:
                    f.put(i + round(x0 / 0.1), j, k + round(z0 / 0.1), "antorchas" if j < 10 else "antorchas_oro", "baldosa_a" if j < 10 else "oro")
# Las seis columnas en los vertices de un hexagono alrededor de la piscina, y el techo
# Los vertices en 0, 60, 120... grados (los mismos del techo): asi ninguna queda entre la Trifuerza y
# la piscina, y las dos de enfrente quedan a los lados de Miku.
for n in range(6):
    a = n / 6 * 2 * math.pi
    x0, z0 = 3.5 * math.cos(a), PZ + 3.5 * math.sin(a)
    for j in range(0, 34):
        r = 0.28 if 1 < j < 32 else 0.38
        for i in range(-4, 4):
            for k in range(-4, 4):
                x, _, z = f.centro(i, 0, k)
                if math.hypot(x, z) <= r:
                    mat = "oro" if j >= 32 or j <= 1 else "marmol_perla"
                    f.put(i + round(x0 / 0.1), j, k + round(z0 / 0.1), "columnas" if mat != "oro" else "capiteles", mat)
for i in range(-45, 45):
    for k in range(-40, 50):
        x, _, z = f.centro(i, 0, k)
        anillo = hexagono(x, z - PZ, 4.1) and not hexagono(x, z - PZ, 3.0)
        if anillo:
            f.put(i, 34, k, "techo", "marmol_perla")
            if not hexagono(x, z - PZ, 3.95):
                f.put(i, 35, k, "techo_oro", "oro")
f.mesh(m)

# Las encendidas (Trifuerza con luz) y el fuego de las antorchas, aparte
luz = Voxels(0.1, origen=(0.0, 0.012, 0.0))
def triangulo_luz(cx, cz, lado, parte):
    h = lado * math.sqrt(3) / 2
    for i in range(round(cx / 0.1) - 8, round(cx / 0.1) + 8):
        for k in range(round(cz / 0.1) - 8, round(cz / 0.1) + 8):
            x, _, z = luz.centro(i, 0, k)
            dz = z - (cz - h / 3)
            if 0 <= dz <= h and abs(x - cx) <= (lado / 2) * (1 - dz / h):
                luz.put(i, 1, k, parte, "oro_luz")
for n, (cx, cz) in enumerate(centros):
    triangulo_luz(cx, cz, LADO, f"trifuerza_luz_{n}")
for lado in (-1, 1):
    x0, z0 = lado * 3.3, -2.0
    for j in range(12, 16):
        r = 0.18 - (j - 12) * 0.04
        for i in range(-2, 2):
            for k in range(-2, 2):
                x, _, z = luz.centro(i, 0, k)
                if math.hypot(x, z) <= r + 0.05:
                    luz.put(i + round(x0 / 0.1), j, k + round(z0 / 0.1), "fuego", "fuego")
luz.mesh(m)

m.write("fuente_hadas", """# La Gran Fuente de las Hadas sobre su isla flotante (homenaje a Ocarina of Time; el Proyecto 2 en voxels).
# Origen en el centro de la plaza, piso en y = 0. Piscina hexagonal en (0, 0, 1.2) con el estrado y el cuenco
# (la Gran Hada sale de ahi, a y ~ 0.6). Trifuerza del piso en (0, 0, -2.6): partes trifuerza_N (apagada) y
# trifuerza_luz_N (encendida). Antorchas en (+-3.3, 0, -2.0) con su fuego. Seis columnas y techo.
""", mtl(("pasto", (0.35, 0.72, 0.38), None), ("tierra", (0.55, 0.40, 0.28), None), ("roca", (0.45, 0.42, 0.50), None),
         ("roca_oscura", (0.32, 0.30, 0.40), None), ("flor_rosa", (1.0, 0.55, 0.8), (0.25, 0.1, 0.2)), ("flor_blanca", (1.0, 1.0, 1.0), (0.2, 0.2, 0.2)),
         ("cristal", (0.75, 0.55, 1.0), (0.35, 0.18, 0.55)), ("agua_cascada", (0.6, 0.85, 1.0), (0.15, 0.3, 0.45), 0.7),
         ("baldosa_a", (0.86, 0.92, 1.0), None), ("baldosa_b", (0.72, 0.82, 0.98), None), ("oro", (1.0, 0.78, 0.30), (0.12, 0.08, 0.0)),
         ("oro_luz", (1.0, 0.9, 0.5), (1.0, 0.75, 0.25)), ("fondo_piscina", (0.15, 0.35, 0.75), (0.02, 0.06, 0.15)),
         ("agua", (0.35, 0.75, 1.0), (0.08, 0.22, 0.4), 0.55), ("loto", (1.0, 0.6, 0.85), (0.3, 0.1, 0.25)),
         ("obsidiana", (0.12, 0.08, 0.2), (0.1, 0.05, 0.2)), ("marmol_perla", (0.82, 0.78, 0.95), None),
         ("fuego", (1.0, 0.6, 0.2), (1.0, 0.5, 0.12))))

# ---------- Megurine Luka (chibi, voxel de 0.04) ----------
# Pelo rosa larguisimo, audifonos dorados, top negro sin mangas con detalles dorados, mangas sueltas
# negras con punos dorados y falda negra larga con orilla dorada. Su izquierda es -X.
U = 0.04
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
def elipsoide(vx, parte, mat, c, r):
    for i in range(math.floor(c[0] - r[0]) - 1, math.ceil(c[0] + r[0]) + 1):
        for j in range(math.floor(c[1] - r[1]) - 1, math.ceil(c[1] + r[1]) + 1):
            for k in range(math.floor(c[2] - r[2]) - 1, math.ceil(c[2] + r[2]) + 1):
                if sum(((p + 0.5 - q) / s) ** 2 for p, q, s in zip((i, j, k), c, r)) <= 1:
                    vx.put(i, j, k, parte, mat)
vx = Voxels(U)
caja_redonda(vx, "botas", "negro_luka", 1, 3, 0, 3, -2, 1, 0.8, espejo=True)
caja_redonda(vx, "falda", "negro_luka", 0, 5, 4, 14, -4, 3, 1.5, espejo=True)
vx.box("falda_orilla", "oro_luka", 0, 5, 4, 4, -5, -5, espejo=True)
vx.box("falda_orilla", "oro_luka", 0, 5, 14, 14, -5, -5, espejo=True)
caja_redonda(vx, "torso", "negro_luka", 0, 4, 15, 21, -3, 2, 1.2, espejo=True)
vx.box("adorno", "oro_luka", 0, 0, 16, 20, -4, -4, espejo=True)
vx.box("cuello_luka", "oro_luka", 0, 3, 21, 21, -4, 2, espejo=True)
caja_redonda(vx, "brazos", "piel_luka", 5, 6, 16, 21, -2, 1, 0.6, espejo=True)
caja_redonda(vx, "mangas", "negro_luka", 5, 7, 10, 15, -3, 2, 0.8, espejo=True)
vx.box("punos", "oro_luka", 5, 7, 10, 10, -3, 2, espejo=True)
caja_redonda(vx, "cabeza", "piel_luka", 0, 6, 22, 33, -6, 6, 3.0, espejo=True)
# El pelo: gorrito arriba, cortina larga por la espalda (hasta las rodillas), mechones a los lados y fleco.
caja_redonda(vx, "pelo", "rosa_luka", 0, 7, 28, 36, -5, 7, 3.0, espejo=True)
caja_redonda(vx, "pelo", "rosa_luka", 0, 7, 6, 32, 3, 8, 2.0, espejo=True)
caja_redonda(vx, "pelo", "rosa_luka", 6, 7, 14, 30, -4, 2, 1.0, espejo=True)
for i, largo in ((0, 3), (1, 2), (2, 4), (3, 2), (4, 3), (5, 2), (6, 4)):
    for j in range(32 - largo, 34):
        vx.put(i, j, -7, "fleco", "rosa_luka", espejo=True)
# Los audifonos dorados con sus aletitas
vx.box("audifonos", "oro_luka", 8, 8, 25, 29, -2, 1, espejo=True)
vx.box("audifonos", "oro_luka", 9, 9, 27, 31, 0, 1, espejo=True)
m = Modelo()
vx.mesh(m)
expresiones(m, z_frente=-6 * U, px=0.021, x_centro=0.0, y_base=23.5 * U)
m.write("luka", """# Megurine Luka chibi (voxel de 0.04), modelo propio: pelo rosa larguisimo, audifonos dorados, top negro
# sin mangas, mangas sueltas y falda larga negras con orillas doradas. Caras de siempre. Mira hacia -Z,
# origen en el piso, mide ~1.48.
""", mtl(("piel_luka", (1.0, 0.86, 0.78), None), ("rosa_luka", (1.0, 0.55, 0.72), (0.12, 0.03, 0.06)),
         ("negro_luka", (0.14, 0.12, 0.16), None), ("oro_luka", (1.0, 0.8, 0.35), (0.15, 0.1, 0.0))) + MTL_CARA)

# ---------- Navi ----------
vx = Voxels(0.03)
elipsoide(vx, "centro", "luz_navi", (-0.5, -0.5, -0.5), (3.0, 3.0, 3.0))
for lado in (-1, 1):
    for (dy, largo) in ((2, 6), (-2, 5)):
        for t in range(largo):
            for w in range(-1, 2):
                vx.put(lado * (3 + t) - (1 if lado < 0 else 0), dy + w + t // 3 * (1 if dy > 0 else -1), 0, "alas", "ala")
m = Modelo()
vx.mesh(m)
m.write("navi", """# Navi (voxel de 0.03): un centro de luz y cuatro alitas transparentes. Centrada en el origen.
""", mtl(("luz_navi", (0.9, 0.95, 1.0), (0.8, 0.95, 1.0)), ("ala", (0.7, 0.9, 1.0), (0.3, 0.45, 0.6), 0.55)))

# ---------- el Contenedor de Corazon ----------
vx = Voxels(0.04)
FORMA = ["..##...##..",
         ".####.####.",
         "###########",
         "###########",
         ".#########.",
         "..#######..",
         "...#####...",
         "....###....",
         ".....#....."]
for r, fila in enumerate(FORMA):
    for c, ch in enumerate(fila):
        if ch == "#":
            borde = (r > 0 and FORMA[r - 1][c] != "#") or c == 0 or c == len(fila) - 1 or fila[c - 1] != "#" or fila[c + 1] != "#" or r == len(FORMA) - 1 or FORMA[r + 1][c] != "#"
            for k in (-1, 0) if not borde else (-2, -1, 0, 1):
                vx.put(c - 5, 4 - r, k, "corazon_borde" if borde else "corazon", "corazon_borde" if borde else "rojo_corazon")
m = Modelo()
vx.mesh(m)
m.write("contenedor", """# El Contenedor de Corazon (voxel de 0.04): corazon rojo brillante con borde claro. Centrado, mira a -Z.
""", mtl(("rojo_corazon", (1.0, 0.15, 0.25), (0.75, 0.05, 0.12)), ("corazon_borde", (1.0, 0.85, 0.85), (0.5, 0.35, 0.35))))
