# Genera los modelos de ¡NEPTUNO!, el juego de cartas de Neptuno:
#   carta.obj:    una carta con todas sus variantes (5 colores, 15 simbolos); se dibuja ocultando
#                 las partes que no son. Atras: el reverso con un planetita.
#   diamante.obj: un diamante (en Neptuno llueven), para la lluvia de atras.
#   doom_slayer.obj, master_chief.obj, cloud.obj: los rivales, fan art voxel (modelos propios,
#                 sin nada de los juegos) del Doom Slayer (DOOM), el Master Chief (Halo) y Cloud
#                 Strife (Final Fantasy VII). Miran hacia -Z, origen en el piso.
# Uso: python3 herramientas/generar_neptuno.py

import math
from modelo3d import *

def mtl(*materiales):
    out = ["# Materiales"]
    for nombre, kd, ke in materiales:
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), "Ks 0.3 0.3 0.3", "Ns 24"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
    return "\n".join(out) + "\n"

def poligono(part, pts, adentro):
    """Un poligono convexo en abanico, con la cara hacia afuera de `adentro`."""
    ids = [part.vert(p) for p in pts]
    for k in range(1, len(ids) - 1):
        part.tri(ids[0], ids[k], ids[k + 1], adentro)

# ---------- la carta ----------
# Centrada en el origen, 0.5 x 0.75, el frente mira a -Z. Desde el frente, la derecha de quien
# la ve es -X: por eso los dibujos se espejean en X.
m = Modelo()
vx = Voxels(0.025)
for i in range(-10, 10):
    for j in range(-15, 15):
        # Esquinas redondeadas
        esquina = (i in (-10, 9)) and (j in (-15, 14))
        if not esquina:
            vx.put(i, j, 0, "marco", "carta_blanco")
vx.mesh(m)
COLORES = {"rojo": (0.92, 0.18, 0.24), "amarillo": (0.98, 0.78, 0.15), "verde": (0.20, 0.68, 0.32),
           "azul": (0.18, 0.42, 0.95), "negro": (0.12, 0.10, 0.16)}
Z = -0.001
for nombre in COLORES:
    m.part(f"color_{nombre}", f"carta_{nombre}").quad(
        [(-0.21, -0.335, Z - 0.001), (0.21, -0.335, Z - 0.001), (0.21, 0.335, Z - 0.001), (-0.21, 0.335, Z - 0.001)], (0, 0, 1))
# El rombo blanco (un diamante de Neptuno) al centro
poligono(m.part("rombo", "carta_blanco"), [(0, 0.27, Z - 0.002), (0.19, 0, Z - 0.002), (0, -0.27, Z - 0.002), (-0.19, 0, Z - 0.002)], (0, 0, 1))

DIGITOS = {
    "0": [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."],
    "1": ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###."],
    "2": [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"],
    "3": ["####.", "....#", "....#", ".###.", "....#", "....#", "####."],
    "4": ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#."],
    "5": ["#####", "#....", "####.", "....#", "....#", "#...#", ".###."],
    "6": [".###.", "#....", "#....", "####.", "#...#", "#...#", ".###."],
    "7": ["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."],
    "8": [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."],
    "9": [".###.", "#...#", "#...#", ".####", "....#", "....#", ".###."],
}
MAS = ["...", ".#.", "###", ".#.", "..."]
def con_mas(digito):
    filas = []
    for r in range(7):
        izq = MAS[r - 1] if 1 <= r <= 5 else "..."
        filas.append(izq + "." + DIGITOS[digito][r])
    return filas
SIMBOLOS = {f"s_{d}": DIGITOS[d] for d in DIGITOS}
SIMBOLOS["s_salta"] = ["..###..", ".#...#.", "#...#.#", "#..#..#", "#.#...#", ".#...#.", "..###.."]
SIMBOLOS["s_reversa"] = ["..#....", ".######", "..#....", ".......", "....#..", "######.", "....#.."]
SIMBOLOS["s_mas2"] = con_mas("2")
SIMBOLOS["s_mas4"] = con_mas("4")
PX = 0.026
for nombre, filas in SIMBOLOS.items():
    part = m.part(nombre, "tinta")
    w, h = len(filas[0]), len(filas)
    for r, fila in enumerate(filas):
        for c, ch in enumerate(fila):
            if ch == "#":
                # Espejo en X: la columna 0 queda a la izquierda de quien ve el frente.
                x0 = (w / 2 - c - 1) * PX
                y0 = (h / 2 - r - 1) * PX
                z = Z - 0.003
                part.quad([(x0, y0, z), (x0 + PX, y0, z), (x0 + PX, y0 + PX, z), (x0, y0 + PX, z)], (0, 0, 1))
# El comodin: cuatro cuadritos de colores en el rombo
for (nombre, dx, dy) in (("rojo", 1, 1), ("amarillo", -1, 1), ("verde", 1, -1), ("azul", -1, -1)):
    x0, y0 = (0.005 if dx > 0 else -0.075), (0.005 if dy > 0 else -0.075)
    m.part(f"s_comodin_{nombre}", f"carta_{nombre}").quad(
        [(x0, y0, Z - 0.003), (x0 + 0.07, y0, Z - 0.003), (x0 + 0.07, y0 + 0.07, Z - 0.003), (x0, y0 + 0.07, Z - 0.003)], (0, 0, 1))
# El reverso: azul noche con un planetita con anillo
ZR = 0.025 + 0.001
m.part("reverso", "reverso").quad([(-0.21, -0.335, ZR), (0.21, -0.335, ZR), (0.21, 0.335, ZR), (-0.21, 0.335, ZR)], (0, 0, 0))
poligono(m.part("reverso_planeta", "reverso_planeta"),
         [(0.12 * math.cos(a * math.pi / 8), 0.12 * math.sin(a * math.pi / 8), ZR + 0.001) for a in range(16)], (0, 0, 0))
anillo = m.part("reverso_anillo", "reverso_anillo")
for a in range(24):
    t0, t1 = a * math.pi / 12, (a + 1) * math.pi / 12
    pts = [(0.2 * math.cos(t0), 0.06 * math.sin(t0) - 0.02 * math.cos(t0), ZR + 0.002), (0.2 * math.cos(t1), 0.06 * math.sin(t1) - 0.02 * math.cos(t1), ZR + 0.002),
           (0.16 * math.cos(t1), 0.04 * math.sin(t1) - 0.015 * math.cos(t1), ZR + 0.002), (0.16 * math.cos(t0), 0.04 * math.sin(t0) - 0.015 * math.cos(t0), ZR + 0.002)]
    if math.sin((t0 + t1) / 2) < 0.2:  # solo la parte de enfrente del anillo, encima del planeta
        anillo.quad(pts, (0, 0, 0))
m.write("carta", """# Carta de ¡NEPTUNO! (0.5 x 0.75, centrada, el frente mira a -Z). Trae todas las variantes:
# color_<rojo|amarillo|verde|azul|negro>, simbolos s_0..s_9, s_salta, s_reversa, s_mas2, s_mas4,
# s_comodin_*; se dibuja ocultando las que no son. Atras (+Z): reverso con un planetita.
""", mtl(("carta_blanco", (0.97, 0.97, 0.98), None), ("tinta", (0.08, 0.06, 0.12), None),
         *[(f"carta_{n}", c, tuple(0.18 * x for x in c)) for n, c in COLORES.items()],
         ("reverso", (0.08, 0.10, 0.32), None), ("reverso_planeta", (0.25, 0.50, 1.0), (0.08, 0.18, 0.4)),
         ("reverso_anillo", (0.85, 0.80, 1.0), (0.2, 0.2, 0.3))).replace("Ks 0.3 0.3 0.3", "Ks 0.03 0.03 0.03"))
# (Las cartas son planas: con brillo especular, toda la cara se lavaba a blanco cuando miraba a la luz.)

# ---------- el diamante ----------
vx = Voxels(0.05)
for i in range(-5, 5):
    for j in range(-6, 5):
        for k in range(-5, 5):
            x, y, z = vx.centro(i, j, k)
            dentro = abs(x) + abs(z) <= (0.25 - y * 0.5 if y >= 0 else 0.25 + y * 0.85)
            if dentro:
                mat = "diamante_brillo" if (i + j + k) % 3 == 0 else "diamante"
                vx.put(i, j, k, mat, mat)
m = Modelo()
vx.mesh(m)
m.write("diamante", """# Diamante de Neptuno (voxel de 0.05), centrado en el origen, ~0.5 de ancho.
""", mtl(("diamante", (0.75, 0.95, 1.00), (0.25, 0.45, 0.60)), ("diamante_brillo", (1.0, 1.0, 1.0), (0.6, 0.8, 1.0))))

# ---------- los rivales ----------
# Chibis como Miku: cabeza (o casco) enorme, cuerpo chiquito, voxel de 0.04. Se arman con cajas de
# esquinas redondeadas, casi siempre espejeadas (espejo=True: se da el lado +X y sale el otro).
U = 0.04

def caja_redonda(vx, parte, mat, i0, i1, j0, j1, k0, k1, r=1.0, espejo=False, si=None):
    """Una caja (rangos inclusivos de voxels) con las esquinas y aristas redondeadas con radio r."""
    for i in range(i0, i1 + 1):
        for j in range(j0, j1 + 1):
            for k in range(k0, k1 + 1):
                d = 0.0
                for c, a, b in ((i + 0.5, i0, i1 + 1), (j + 0.5, j0, j1 + 1), (k + 0.5, k0, k1 + 1)):
                    fuera = max(a + r - c, 0.0, c - (b - r))
                    d += fuera * fuera
                if d <= r * r and (si is None or si(i, j, k)):
                    vx.put(i, j, k, parte, mat, espejo)

def elipse_frente(vx, parte, mat, k, cx, cy, rx, ry, espejo=True):
    """Un ovalo plano en la capa k (para visores y ojos). cx es 0 al centro."""
    for i in range(0, math.ceil(rx) + 1):
        for j in range(math.floor(cy - ry), math.ceil(cy + ry) + 1):
            if ((i + 0.5 - cx) / rx) ** 2 + ((j + 0.5 - cy) / ry) ** 2 <= 1:
                vx.put(i, j, k, parte, mat, espejo)

def piernas_y_cuerpo(vx, botas, piernas, cintura, torso):
    caja_redonda(vx, "botas", botas, 1, 4, 0, 2, -4, 2, 1.0, espejo=True)
    caja_redonda(vx, "piernas", piernas, 1, 4, 3, 7, -2, 2, 0.8, espejo=True)
    caja_redonda(vx, "cintura", cintura, 0, 4, 8, 9, -3, 3, 0.8, espejo=True)
    caja_redonda(vx, "torso", torso, 0, 5, 10, 17, -3, 3, 1.2, espejo=True)

# --- Doom Slayer ---
vx = Voxels(U)
piernas_y_cuerpo(vx, "botas", "verde", "cinturon", "verde")
vx.box("rodilleras", "gris", 1, 4, 5, 6, -3, -3, espejo=True)
vx.box("bolsas", "cinturon", 2, 3, 8, 9, -4, -4, espejo=True)
vx.box("pecho", "gris", 0, 3, 12, 16, -4, -4, espejo=True)
vx.box("pecho_linea", "verde_oscuro", 0, 0, 12, 16, -5, -5, espejo=True)
caja_redonda(vx, "hombreras", "verde", 5, 8, 14, 18, -3, 3, 1.5, espejo=True)
vx.box("hombreras_filo", "verde_oscuro", 5, 8, 14, 14, -3, 3, espejo=True)
caja_redonda(vx, "brazos", "gris", 6, 7, 9, 13, -2, 1, 0.6, espejo=True)
caja_redonda(vx, "guantes", "verde_oscuro", 6, 7, 7, 8, -2, 1, 0.6, espejo=True)
caja_redonda(vx, "casco", "verde", 0, 7, 17, 32, -7, 7, 3.0, espejo=True)
vx.box("visor", "visor", 0, 5, 23, 26, -8, -8, espejo=True)
vx.box("visor_brillo", "visor_brillo", -5, -3, 26, 26, -9, -9)
vx.box("casco_cresta", "verde_oscuro", 0, 0, 27, 32, -8, -8, espejo=True)
vx.box("bozal", "gris", 0, 4, 18, 21, -8, -8, espejo=True)
for i in (1, 3):
    vx.box("bozal_rejilla", "verde_oscuro", i, i, 19, 20, -9, -9, espejo=True)
caja_redonda(vx, "orejeras", "gris", 8, 8, 21, 27, -3, 2, 0.6, espejo=True)
# La escopeta de dos canones, en la mano izquierda (-X), apuntando al frente
vx.box("escopeta", "arma", -10, -9, 8, 9, -13, 1)
vx.box("escopeta_madera", "madera", -10, -9, 6, 9, 2, 5)
m = Modelo()
vx.mesh(m)
m.write("doom_slayer", """# Doom Slayer chibi (voxel de 0.04), fan art propio del de DOOM: casco enorme con visor y bozal,
# hombreras redondas y la escopeta de dos canones. Mira hacia -Z, origen en el piso, mide ~1.32.
""", mtl(("verde", (0.28, 0.48, 0.22), None), ("verde_oscuro", (0.16, 0.30, 0.14), None), ("gris", (0.55, 0.55, 0.52), None),
         ("cinturon", (0.42, 0.28, 0.15), None), ("botas", (0.22, 0.20, 0.20), None),
         ("visor", (0.20, 0.12, 0.04), (0.55, 0.30, 0.04)), ("visor_brillo", (1.0, 0.9, 0.6), (0.9, 0.7, 0.3)),
         ("arma", (0.28, 0.28, 0.32), None), ("madera", (0.48, 0.30, 0.16), None)))

# --- Master Chief ---
vx = Voxels(U)
piernas_y_cuerpo(vx, "olivo_oscuro", "olivo", "traje", "olivo")
vx.box("rodillas", "traje", 1, 4, 5, 5, -3, -3, espejo=True)
vx.box("pecho", "olivo_oscuro", 0, 4, 12, 16, -4, -4, espejo=True)
vx.box("cuello", "traje", 0, 3, 17, 17, -4, 3, espejo=True)
caja_redonda(vx, "hombreras", "olivo", 5, 8, 14, 17, -3, 3, 1.5, espejo=True)
caja_redonda(vx, "brazos", "traje", 6, 7, 11, 13, -2, 1, 0.6, espejo=True)
caja_redonda(vx, "antebrazos", "olivo", 6, 7, 7, 10, -2, 1, 0.6, espejo=True)
caja_redonda(vx, "mochila", "olivo_oscuro", 0, 4, 11, 17, 4, 6, 1.0, espejo=True)
caja_redonda(vx, "casco", "olivo", 0, 7, 17, 32, -7, 7, 4.0, espejo=True)
elipse_frente(vx, "visor", "visor_dorado", -8, 0.0, 24.5, 6.2, 3.2)
vx.box("visor_brillo", "visor_brillo", 2, 4, 26, 26, -9, -9)
vx.box("visor_brillo", "visor_brillo", 4, 5, 25, 25, -9, -9)
vx.box("barbilla", "olivo_oscuro", 0, 3, 17, 19, -8, -8, espejo=True)
caja_redonda(vx, "orejeras", "traje", 8, 8, 21, 26, -3, 2, 0.6, espejo=True)
m = Modelo()
vx.mesh(m)
m.write("master_chief", """# Master Chief chibi (voxel de 0.04), fan art propio del de Halo: casco redondo enorme verde olivo
# con el visor dorado (y su brillo), hombreras y mochila. Mira hacia -Z, origen en el piso, mide ~1.32.
""", mtl(("olivo", (0.40, 0.46, 0.26), None), ("olivo_oscuro", (0.28, 0.33, 0.18), None), ("traje", (0.12, 0.12, 0.14), None),
         ("visor_dorado", (1.0, 0.70, 0.15), (0.50, 0.30, 0.03)), ("visor_brillo", (1.0, 0.97, 0.85), (0.9, 0.85, 0.6))))

# --- Cloud Strife ---
vx = Voxels(U)
piernas_y_cuerpo(vx, "cuero", "ropa", "cuero", "ropa")
vx.box("tirantes", "metal", 2, 2, 11, 17, -4, -4, espejo=True)
vx.box("tirantes", "metal", 0, 2, 14, 14, -4, -4, espejo=True)
caja_redonda(vx, "brazos", "piel", 6, 7, 10, 15, -2, 1, 0.6, espejo=True)
caja_redonda(vx, "guantes", "cuero", 6, 7, 8, 9, -2, 1, 0.6, espejo=True)
# La hombrera solo en el hombro izquierdo (-X)
caja_redonda(vx, "hombrera", "metal", -10, -5, 15, 18, -4, 3, 1.6)
vx.box("hombrera_correa", "cuero", -6, -5, 13, 14, -4, 3)
# La cabeza y el pelo
caja_redonda(vx, "cabeza", "piel", 0, 6, 18, 30, -6, 6, 3.0, espejo=True)
caja_redonda(vx, "pelo", "rubio", 0, 7, 26, 33, -5, 8, 3.0, espejo=True)
caja_redonda(vx, "pelo", "rubio", 0, 7, 20, 30, 3, 8, 2.0, espejo=True)
# Fleco en picos sobre la frente
for i, largo in ((0, 2), (1, 1), (2, 3), (3, 1), (4, 2), (5, 1), (6, 3)):
    for j in range(29 - largo, 31):
        vx.put(i, j, -7, "fleco", "rubio", espejo=True)
# Los picos grandes: hacia arriba, a los lados y hacia atras
picos = [((0, 34, 0), (0, 1, 0), 4), ((3, 34, -1), (1, 1, 0), 4), ((2, 34, 3), (0, 1, 1), 4), ((6, 31, 2), (1, 1, 0), 4),
         ((7, 28, 4), (1, 0, 1), 4), ((4, 31, 8), (0, 1, 1), 4), ((0, 30, 9), (0, 0, 1), 4), ((6, 25, 7), (1, -1, 1), 3)]
for (i, j, k), (di, dj, dk), largo in picos:
    for t in range(largo):
        for (a, b) in ((0, 0), (0, -1), (-1, 0)) if t < largo - 1 else ((0, 0),):
            vx.put(i + di * t + (a if di == 0 else 0), j + dj * t + b, k + dk * t, "picos", "rubio", espejo=True)
# La Buster Sword en la espalda, en diagonal, con sus dos agujeros
for t in range(0, 30):
    i = -6 + t // 3
    j = 4 + t
    for w in range(4):
        hueco = t in (24, 26) and w in (1, 2)
        if not hueco:
            vx.put(i + w, j, 9, "espada" if w > 0 else "espada_filo", "hoja" if w > 0 else "filo")
vx.box("espada_guarda", "metal", 2, 7, 34, 35, 8, 10)
vx.box("espada_mango", "cuero", 4, 5, 36, 40, 9, 9)
m = Modelo()
vx.mesh(m)
# Ojos chibi como los de Miku, con todas sus expresiones
expresiones(m, z_frente=-6 * U, px=0.022, x_centro=0.0, y_base=19.5 * U)
m.write("cloud", """# Cloud Strife chibi (voxel de 0.04), fan art propio del de Final Fantasy VII: pelo rubio en picos,
# ojos chibi con expresiones, hombrera en el hombro izquierdo y la Buster Sword en la espalda.
# Mira hacia -Z, origen en el piso, mide ~1.36 (con el pelo).
""", mtl(("ropa", (0.20, 0.22, 0.40), None), ("cuero", (0.40, 0.26, 0.15), None), ("metal", (0.75, 0.77, 0.83), None),
         ("piel", (1.0, 0.84, 0.72), None), ("rubio", (1.0, 0.85, 0.35), None), ("hoja", (0.72, 0.75, 0.82), None),
         ("filo", (0.88, 0.90, 0.95), None)) + MTL_CARA)

def elipsoide(vx, parte, mat, c, r):
    """Llena un elipsoide de centro c y radios r (en voxels)."""
    for i in range(math.floor(c[0] - r[0]) - 1, math.ceil(c[0] + r[0]) + 1):
        for j in range(math.floor(c[1] - r[1]) - 1, math.ceil(c[1] + r[1]) + 1):
            for k in range(math.floor(c[2] - r[2]) - 1, math.ceil(c[2] + r[2]) + 1):
                if sum(((p + 0.5 - q) / s) ** 2 for p, q, s in zip((i, j, k), c, r)) <= 1:
                    vx.put(i, j, k, parte, mat)

# --- Canela (Animal Crossing) ---
# Perrita shih tzu amarilla: chongo con liga roja y cascabel, orejas largas con las puntas naranjas,
# hocico blanco, chaleco verde a cuadros sobre camisa blanca, mono rojo, falda azul y colita.
vx = Voxels(U)
caja_redonda(vx, "botas", "pelo_canela", 1, 3, 0, 1, -2, 1, 0.8, espejo=True)
caja_redonda(vx, "piernas", "pelo_canela", 1, 3, 2, 5, -2, 1, 0.8, espejo=True)
caja_redonda(vx, "falda", "falda", 0, 5, 6, 10, -4, 3, 1.5, espejo=True)
caja_redonda(vx, "torso", "chaleco", 0, 4, 11, 17, -3, 3, 1.2, espejo=True)
for j in range(11, 18):
    for i in range(0, 5):
        if (i + j) % 3 == 0:
            vx.put(i, j, -4, "cuadros", "chaleco_claro", espejo=True)
vx.box("botones", "camisa", 0, 0, 12, 16, -5, -5, espejo=True)
vx.box("cuello_camisa", "camisa", 0, 2, 17, 17, -4, 3, espejo=True)
vx.box("mono", "mono", 0, 1, 16, 17, -5, -5, espejo=True)
caja_redonda(vx, "mangas", "camisa", 5, 6, 14, 17, -2, 1, 0.6, espejo=True)
caja_redonda(vx, "brazos", "pelo_canela", 5, 6, 10, 13, -2, 1, 0.6, espejo=True)
# La cabeza grande, el fleco y el hocico
caja_redonda(vx, "cabeza", "pelo_canela", 0, 7, 18, 30, -6, 6, 3.5, espejo=True)
caja_redonda(vx, "fleco", "pelo_canela", 0, 7, 27, 31, -7, 4, 2.0, espejo=True)
caja_redonda(vx, "hocico", "hocico", 0, 3, 18, 22, -8, -6, 1.2, espejo=True)
vx.box("nariz", "nariz", 0, 0, 22, 22, -9, -9, espejo=True)
# Orejas largas que cuelgan, con la punta naranja
for j in range(17, 29):
    mat = "oreja_punta" if j < 21 else "pelo_canela"
    parte = "orejas_punta" if j < 21 else "orejas"
    for k in range(-2, 3):
        vx.put(8, j, k, parte, mat, espejo=True)
        if j < 26:
            vx.put(9, j, k, parte, mat, espejo=True)
# El chongo con su liga y su cascabel
elipsoide(vx, "chongo", "pelo_canela", (-0.5, 35.5, 0.5), (3.2, 3.2, 3.2))
vx.box("liga", "mono", -2, 1, 32, 32, -2, 2)
vx.box("cascabel", "cascabel", 2, 2, 32, 33, -3, -3)
# La colita
for (y, z, mat) in ((7, 4, "pelo_canela"), (8, 5, "pelo_canela"), (9, 6, "pelo_canela"), (10, 6, "hocico")):
    vx.box("colita_punta" if mat == "hocico" else "colita", mat, -1, 0, y, y, z, z)
m = Modelo()
vx.mesh(m)
# Ojitos negros arriba del hocico, con todas las expresiones
expresiones(m, z_frente=-6 * U, px=0.018, x_centro=0.0, y_base=22.5 * U)
m.write("canela", """# Canela chibi (voxel de 0.04), fan art propio de la de Animal Crossing: chongo con liga y cascabel,
# orejas largas, hocico blanco, chaleco verde a cuadros y falda azul. Mira hacia -Z, origen en el piso.
""", mtl(("pelo_canela", (1.0, 0.80, 0.28), None), ("oreja_punta", (0.98, 0.55, 0.18), None), ("hocico", (1.0, 0.96, 0.85), None),
         ("nariz", (0.12, 0.08, 0.06), None), ("chaleco", (0.45, 0.62, 0.40), None), ("chaleco_claro", (0.62, 0.78, 0.55), None),
         ("camisa", (0.97, 0.97, 0.97), None), ("mono", (0.85, 0.15, 0.25), None), ("falda", (0.18, 0.24, 0.35), None),
         ("cascabel", (1.0, 0.92, 0.5), (0.2, 0.15, 0.05))) + MTL_CARA)
