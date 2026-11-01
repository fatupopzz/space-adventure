# Genera los modelos del juicio de divorcio de Saturno, en Venus (homenaje a Ace Attorney):
#   juzgado.obj:          el juzgado de lo familiar de Venus, entre sus nubes amarillas: piso de madera, alfombra roja,
#                         estrado del juez, banquillo de testigos, mesas de la defensa y de la parte demandante,
#                         baranda, bancas del publico y el escudo de Venus en la pared.
#   saturno_cliente.obj:  Saturno chiquito, con cara y sus tres anillos principales (C, B y A, con la division
#                         de Cassini entre B y A) como partes aparte: anillo_a, anillo_b, anillo_c.
#   sonic.obj, gollum.obj, sinluz.obj: los tres ex (fan art voxel, modelos propios): Sonic (Sonic the Hedgehog),
#                         Gollum (The Lord of the Rings: Gollum) y el Sinluz (Elden Ring, sin cara: casco cerrado).
#   ganso_juez.obj:       el juez: el ganso de Untitled Goose Game, con peluca de juez.
#   phoenix.obj:          Phoenix Wright (Ace Attorney), el abogado de la parte demandante.
# Todos miran hacia -Z, origen en el piso (Saturno: en su centro).
# Uso: python3 herramientas/generar_divorcio.py

import math
from modelo3d import *

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
    """Una caja (rangos inclusivos de voxels) con las esquinas y aristas redondeadas con radio r."""
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
    """Llena un elipsoide de centro c y radios r (en voxels). si(i, j, k) da el material (o None)."""
    for i in range(math.floor(c[0] - r[0]) - 1, math.ceil(c[0] + r[0]) + 1):
        for j in range(math.floor(c[1] - r[1]) - 1, math.ceil(c[1] + r[1]) + 1):
            for k in range(math.floor(c[2] - r[2]) - 1, math.ceil(c[2] + r[2]) + 1):
                if sum(((p + 0.5 - q) / s) ** 2 for p, q, s in zip((i, j, k), c, r)) <= 1:
                    m = si(i, j, k) if si else mat
                    if m:
                        vx.put(i, j, k, parte if m == mat else f"{parte}_{m}", m)

def espiga(vx, parte, mat, base, direccion, largo, r0):
    """Un cono (pua, pelo en punta) desde `base` hacia `direccion`, que se afila hasta la punta."""
    d = norm(direccion)
    pasos = int(largo * 2)
    for s in range(pasos + 1):
        t = s / pasos
        c = add(base, mul(d, largo * t))
        r = r0 * (1 - t) + 0.4
        for i in range(math.floor(c[0] - r) - 1, math.ceil(c[0] + r) + 1):
            for j in range(math.floor(c[1] - r) - 1, math.ceil(c[1] + r) + 1):
                for k in range(math.floor(c[2] - r) - 1, math.ceil(c[2] + r) + 1):
                    if (i + 0.5 - c[0]) ** 2 + (j + 0.5 - c[1]) ** 2 + (k + 0.5 - c[2]) ** 2 <= r * r:
                        vx.put(i, j, k, parte, mat)

def disco_frente(vx, parte, mat, k, cx, cy, r):
    """Un circulo plano en la capa k (ojos grandes)."""
    for i in range(math.floor(cx - r) - 1, math.ceil(cx + r) + 1):
        for j in range(math.floor(cy - r) - 1, math.ceil(cy + r) + 1):
            if (i + 0.5 - cx) ** 2 + (j + 0.5 - cy) ** 2 <= r * r:
                vx.put(i, j, k, parte, mat)

U = 0.04

# =====================================================================
# Sonic
# =====================================================================
vx = Voxels(U)
caja_redonda(vx, "zapatos", "rojo_sonic", 1, 4, 0, 2, -6, 2, 1.0, espejo=True)
vx.box("correas", "blanco", 1, 4, 2, 2, -3, -2, espejo=True)
vx.box("suelas", "blanco", 1, 4, 0, 0, -6, 2, espejo=True)
vx.box("piernas", "azul_sonic", 1, 2, 3, 7, -1, 0, espejo=True)
caja_redonda(vx, "cuerpo", "azul_sonic", 0, 4, 8, 15, -3, 3, 1.5, espejo=True)
caja_redonda(vx, "panza", "durazno", 0, 2, 9, 14, -4, -4, 0.5, espejo=True)
vx.box("brazos", "durazno", 5, 6, 11, 14, -1, 0, espejo=True)
caja_redonda(vx, "guantes", "blanco", 5, 7, 8, 10, -2, 1, 0.8, espejo=True)
elipsoide(vx, "cabeza", "azul_sonic", (0.0, 22.0, 0.0), (7.5, 7.0, 7.0))
# Las puas para atras (la de en medio larga, las de los lados mas cortas)
for (dx, dy, largo) in ((0, 3, 10), (0, -1, 11), (0, -5, 9), (-4, 1, 8), (4, 1, 8), (-4, -3, 7), (4, -3, 7)):
    espiga(vx, "puas", "azul_sonic", (dx, 22 + dy, 3), (dx * 0.08, -0.35 + dy * 0.03, 1), largo, 3.2)
espiga(vx, "puas", "azul_sonic", (0, 13, 3), (0, -0.6, 1), 5, 2.0)  # la colita
# Las orejas, con el adentro durazno
for j, ancho in ((27, 2), (28, 2), (29, 1), (30, 1)):
    vx.box("orejas", "azul_sonic", 4, 3 + ancho, j, j, -1, 1, espejo=True)
    vx.box("orejas_adentro", "durazno", 4, 3 + ancho, j, j, -2, -2, espejo=True)
# El hocico, la nariz y los ojos blancos (una sola mancha, como en los juegos)
elipsoide(vx, "hocico", "durazno", (0.0, 19.0, -6.0), (4.0, 2.6, 2.2))
vx.box("nariz", "negro", -1, 0, 20, 21, -9, -9)
for i in range(-5, 5):
    for j in range(21, 27):
        dx = abs(i + 0.5) - 2.6
        if (dx / 2.6) ** 2 + ((j + 0.5 - 23.6) / 2.6) ** 2 <= 1 or (abs(i + 0.5) < 1 and 22 <= j <= 25):
            vx.put(i, j, -8, "ojos_blancos", "blanco")
m = Modelo()
vx.mesh(m)
CARAS_VERDES = {}
for emo, piezas in CARAS.items():
    CARAS_VERDES[emo] = [(p, "ojo_verde" if mat == "ojo" and p == "ojos" else mat, rects) for p, mat, rects in piezas]
expresiones(m, z_frente=-8 * U, px=0.02, x_centro=0.0, y_base=20.2 * U, caras=CARAS_VERDES)
m.write("sonic", """# Sonic chibi (voxel de 0.04), fan art propio: puas para atras, hocico durazno, guantes blancos y zapatos
# rojos. Ojos verdes con las caras de siempre. Mira hacia -Z, origen en el piso.
""", mtl(("azul_sonic", (0.10, 0.30, 0.95), None), ("durazno", (1.0, 0.80, 0.62), None), ("rojo_sonic", (0.90, 0.10, 0.12), None),
         ("blanco", (0.97, 0.97, 0.97), None), ("negro", (0.06, 0.05, 0.08), None), ("ojo_verde", (0.10, 0.65, 0.25), None)) + MTL_CARA)

# =====================================================================
# Gollum
# =====================================================================
vx = Voxels(U)
caja_redonda(vx, "pies", "piel_gollum", 1, 4, 0, 1, -6, 1, 0.8, espejo=True)
vx.box("piernas", "piel_gollum", 1, 2, 2, 7, -1, 0, espejo=True)
caja_redonda(vx, "taparrabo", "taparrabo", 0, 3, 7, 9, -3, 2, 0.8, espejo=True)
# El torso flaco y jorobado (se va para adelante al subir)
for j in range(10, 17):
    adelante = (j - 10) // 2
    vx.box("torso", "piel_gollum", 0, 2, j, j, -2 - adelante, 1 - adelante, espejo=True)
for j in range(11, 15):
    vx.put(0, j, -3 - (j - 10) // 2, "costillas", "piel_sombra", espejo=True)
# Los brazos larguisimos y las manos
vx.box("brazos", "piel_gollum", 3, 3, 6, 15, -4, -3, espejo=True)
caja_redonda(vx, "manos", "piel_gollum", 3, 4, 4, 6, -5, -3, 0.6, espejo=True)
# La cabeza grande, adelante del cuerpo, con las orejas puntiagudas y tres pelos
elipsoide(vx, "cabeza", "piel_gollum", (0.0, 21.0, -4.0), (6.5, 5.5, 5.5))
for (j, ancho) in ((20, 4), (21, 4), (22, 3), (23, 2), (24, 1)):
    vx.box("orejas", "piel_gollum", 6, 5 + ancho, j, j, -4, -3, espejo=True)
for (i, k, largo) in ((-1, -3, 3), (1, -5, 2), (2, -2, 4)):
    vx.box("pelos", "negro", i, i, 27, 26 + largo, k, k)
# Los ojos enormes, palidos y brillosos
for lado in (-1, 1):
    disco_frente(vx, "ojos_grandes", "ojo_gollum", -10, lado * 2.8, 21.8, 2.3)
vx.box("boca_gollum", "piel_sombra", -2, 1, 18, 18, -10, -10)
m = Modelo()
vx.mesh(m)
# Las pupilas chiquitas al centro de cada ojo (sus propias caras)
CARAS_GOLLUM = {
    "feliz": [("pupilas", "ojo", espejo(pixeles(["##", "##"], 2, 23)))],
    "asustada": [("pupilas", "ojo", espejo(pixeles(["#"], 3, 22))), ("sudor", "sudor", pixeles([".#", "##"], 6, 27))],
    "enojado": [("pupilas", "ojo", espejo(pixeles(["##"], 2, 22))), ("cejas", "ojo", espejo(pixeles(["..##", "##.."], 1, 26)))],
    "espiral": [("pupilas", "ojo", espejo(pixeles(["###", "#.#", "###"], 2, 23)))],
    "serio": [("pupilas", "ojo", espejo(pixeles(["##"], 2, 22))), ("parpados", "piel_sombra", espejo(pixeles(["#####"], 1, 24)))],
    "enamorada": [("corazon", "corazon", espejo(pixeles(["#.#", "###", ".#."], 2, 23)))],
}
expresiones(m, z_frente=-10 * U, px=U, x_centro=0.0, y_base=0.0, caras=CARAS_GOLLUM)
m.write("gollum", """# Gollum chibi (voxel de 0.04), fan art propio: flaco y jorobado, cabeza enorme con ojos palidos,
# orejas puntiagudas, tres pelos y taparrabo. Sus caras son las pupilas. Mira hacia -Z, origen en el piso.
""", mtl(("piel_gollum", (0.80, 0.78, 0.66), None), ("piel_sombra", (0.55, 0.52, 0.42), None), ("taparrabo", (0.40, 0.30, 0.20), None),
         ("negro", (0.06, 0.05, 0.08), None), ("ojo_gollum", (0.75, 0.90, 1.0), (0.25, 0.32, 0.4))) + MTL_CARA)

# =====================================================================
# El Sinluz (Elden Ring)
# =====================================================================
vx = Voxels(U)
caja_redonda(vx, "botas", "metal_oscuro", 1, 4, 0, 2, -4, 2, 0.8, espejo=True)
caja_redonda(vx, "piernas", "metal", 1, 3, 3, 8, -2, 1, 0.8, espejo=True)
caja_redonda(vx, "torso", "metal", 0, 4, 9, 16, -3, 3, 1.3, espejo=True)
# La sobreveste azul con orilla dorada, larga y rota abajo
for j in range(4, 15):
    for i in range(0, 4):
        if j > 5 or (i + j) % 3 != 0:
            vx.put(i, j, -4, "sobreveste", "tela_sinluz", espejo=True)
vx.box("sobreveste_orilla", "oro_sinluz", 0, 3, 15, 15, -4, -4, espejo=True)
vx.box("cinturon", "cuero", 0, 4, 9, 9, -5, -4, espejo=True)
# La capa oscura y rasgada atras
for j in range(3, 17):
    for i in range(0, 6):
        if j > 4 or i % 2 == 0:
            vx.put(i, j, 4, "capa", "capa", espejo=True)
# Hombreras, brazos y guanteletes
caja_redonda(vx, "hombreras", "metal", 4, 7, 14, 17, -3, 2, 1.2, espejo=True)
vx.box("brazos", "metal_oscuro", 5, 6, 9, 13, -2, 0, espejo=True)
caja_redonda(vx, "guanteletes", "metal", 5, 7, 7, 9, -3, 0, 0.6, espejo=True)
# El casco cerrado con la cruz de la visera
elipsoide(vx, "casco", "metal", (0.0, 21.5, 0.0), (5.5, 5.5, 5.5))
vx.box("visera", "negro", -3, 2, 21, 22, -6, -6)
vx.box("visera", "negro", -1, 0, 18, 24, -6, -6)
vx.box("cresta", "metal_oscuro", -1, 0, 26, 28, -3, 4)
# La espada recta en la mano derecha, con la punta en el piso
vx.box("hoja", "acero", 7, 7, 0, 10, -2, -2)
vx.box("guarda", "oro_sinluz", 6, 8, 11, 11, -2, -2)
vx.box("puno", "cuero", 7, 7, 12, 13, -2, -2)
m = Modelo()
vx.mesh(m)
m.write("sinluz", """# El Sinluz chibi (voxel de 0.04), fan art propio de Elden Ring: armadura, casco cerrado (no tiene cara),
# sobreveste azul con orilla dorada, capa rasgada y espada recta. Mira hacia -Z, origen en el piso.
""", mtl(("metal", (0.72, 0.74, 0.80), None), ("metal_oscuro", (0.38, 0.40, 0.46), None), ("acero", (0.88, 0.90, 0.95), None),
         ("tela_sinluz", (0.22, 0.30, 0.50), None), ("oro_sinluz", (1.0, 0.78, 0.30), (0.12, 0.08, 0.0)), ("cuero", (0.35, 0.22, 0.12), None),
         ("capa", (0.16, 0.15, 0.20), None), ("negro", (0.03, 0.03, 0.05), None)))

# =====================================================================
# El ganso juez (Untitled Goose Game)
# =====================================================================
vx = Voxels(U)
for lado in (-1, 1):
    vx.box("patas", "naranja", 1 if lado > 0 else -2, 1 if lado > 0 else -2, 1, 5, 0, 0)
vx.box("pies", "naranja", 0, 2, 0, 0, -3, 0, espejo=True)
elipsoide(vx, "cuerpo", "pluma", (0.0, 9.0, 1.0), (4.5, 4.0, 6.5))
vx.box("cola", "pluma", -1, 0, 10, 11, 7, 8)
vx.box("alas", "pluma_gris", 4, 4, 8, 11, -2, 5, espejo=True)
# El cuello largo, un poco para adelante, y la cabeza
for j in range(11, 21):
    k = -3 - (j - 11) // 4
    for i in range(-2, 2):
        for kk in range(k - 1, k + 2):
            if (i + 0.5) ** 2 + (kk + 0.5 - k) ** 2 <= 2.4:
                vx.put(i, j, kk, "cuello", "pluma")
elipsoide(vx, "cabeza", "pluma", (0.0, 21.5, -5.0), (2.4, 2.3, 3.0))
vx.box("pico", "naranja", -1, 0, 20, 21, -10, -8)
vx.box("ojos", "negro", 2, 2, 22, 22, -6, -6, espejo=True)
# La peluca de juez: blanca, con rizos a los lados
elipsoide(vx, "peluca", "peluca", (0.0, 23.6, -4.8), (3.0, 1.6, 3.4))
for j in range(18, 23):
    for k in range(-7, -2):
        if (j + k) % 2 == 0:
            vx.put(3, j, k, "rizos", "peluca", espejo=True)
m = Modelo()
vx.mesh(m)
m.write("ganso_juez", """# El ganso juez (voxel de 0.04), fan art propio de Untitled Goose Game: ganso blanco con pico y patas
# naranjas y peluca de juez con rizos. No tiene caras (es un ganso). Mira hacia -Z, origen en el piso.
""", mtl(("pluma", (0.97, 0.97, 0.95), None), ("pluma_gris", (0.80, 0.80, 0.82), None), ("naranja", (1.0, 0.55, 0.10), None),
         ("negro", (0.04, 0.04, 0.05), None), ("peluca", (0.90, 0.88, 0.82), None)))

# =====================================================================
# Phoenix Wright (Ace Attorney)
# =====================================================================
vx = Voxels(U)
caja_redonda(vx, "zapatos", "negro", 1, 3, 0, 1, -4, 1, 0.7, espejo=True)
vx.box("piernas", "traje", 1, 3, 2, 8, -2, 1, espejo=True)
caja_redonda(vx, "saco", "traje", 0, 5, 9, 17, -3, 3, 1.4, espejo=True)
# La camisa blanca en V, la corbata roja y el pin de abogado
for j in range(12, 18):
    ancho = (j - 11) // 2
    vx.box("camisa", "camisa", 0, ancho, j, j, -4, -4, espejo=True)
vx.box("corbata", "corbata", -1, 0, 11, 16, -5, -5)
vx.put(3, 15, -4, "pin", "oro_pin")
vx.box("brazos", "traje", 6, 7, 10, 16, -2, 1, espejo=True)
caja_redonda(vx, "manos", "piel", 6, 7, 8, 9, -2, 0, 0.6, espejo=True)
caja_redonda(vx, "cabeza", "piel", 0, 6, 18, 29, -6, 5, 3.0, espejo=True)
# El pelo negro en puntas para atras (el peinado mas famoso de los juzgados)
caja_redonda(vx, "pelo", "pelo_negro", 0, 7, 26, 32, -5, 6, 2.5, espejo=True)
for (dy, dx, largo) in ((5, 0, 11), (1, 0, 12), (-3, 0, 10), (3, 4, 8), (-1, 4, 8), (3, -4, 8), (-1, -4, 8)):
    espiga(vx, "puntas", "pelo_negro", (dx, 26 + dy, 4), (dx * 0.07, 0.15 + dy * 0.04, 1), largo, 3.0)
for i, largo in ((0, 3), (1, 2), (2, 3), (3, 2), (4, 3), (5, 2), (6, 2)):
    for j in range(31 - largo, 31):
        vx.put(i, j, -7, "fleco", "pelo_negro", espejo=True)
m = Modelo()
vx.mesh(m)
expresiones(m, z_frente=-6 * U, px=0.021, x_centro=0.0, y_base=19.5 * U)
m.write("phoenix", """# Phoenix Wright chibi (voxel de 0.04), fan art propio de Ace Attorney: pelo negro en puntas para atras,
# traje azul, camisa blanca, corbata roja y pin de abogado. Caras de siempre (suda con "asustada").
# Mira hacia -Z, origen en el piso.
""", mtl(("traje", (0.16, 0.24, 0.58), None), ("camisa", (0.97, 0.97, 0.97), None), ("corbata", (0.85, 0.10, 0.15), None),
         ("oro_pin", (1.0, 0.80, 0.30), (0.2, 0.15, 0.0)), ("piel", (1.0, 0.85, 0.74), None), ("pelo_negro", (0.10, 0.09, 0.14), None),
         ("negro", (0.06, 0.05, 0.08), None)) + MTL_CARA)

# =====================================================================
# Saturno, el cliente
# =====================================================================
S = 0.03
vx = Voxels(S)
R = 9.0
def banda(i, j, k):
    y = j + 0.5
    return "banda_clara" if (y > 2 or y < -1.5) and int(abs(y) // 2.5) % 2 == 0 else ("banda" if abs(y) < 6 else "banda_oscura")
elipsoide(vx, "planeta", "banda", (0.0, 0.0, 0.0), (R, R * 0.92, R), si=banda)
for i in range(-6, 6):
    for j in range(-3, 8):
        if (i + 0.5) ** 2 + (j - 2) ** 2 <= 5.5 ** 2:
            for k in (-9, -8):
                vx.put(i, j, k, "rostro", "banda_clara")
m = Modelo()
vx.mesh(m)
# Los anillos (planos, de un voxel de grueso, abajo de la boca): C adentro y oscuro, B ancho y brillante,
# la division de Cassini (hueco) y A afuera.
anillos = Voxels(S)
for i in range(-23, 23):
    for k in range(-23, 23):
        r = math.hypot(i + 0.5, k + 0.5)
        cual = "c" if 11 <= r < 13.5 else "b" if 13.5 <= r < 17.5 else "a" if 18.5 <= r < 21.5 else None
        if cual:
            mat = {"c": "anillo_oscuro", "b": "anillo_brillante", "a": "anillo_medio"}[cual]
            if cual == "b" and int(r * 2) % 3 == 0:
                mat = "anillo_medio"
            vx_parte = f"anillo_{cual}"
            anillos.put(i, -2, k, vx_parte if mat == {"c": "anillo_oscuro", "b": "anillo_brillante", "a": "anillo_medio"}[cual] else f"{vx_parte}_surco", mat)
anillos.mesh(m)
expresiones(m, z_frente=-9 * S, px=0.018, x_centro=0.0, y_base=-1.2 * S)
m.write("saturno_cliente", """# Saturno chiquito (voxel de 0.03), el cliente del juicio: con bandas, cara y sus tres anillos principales
# como partes aparte (anillo_c, anillo_b con sus surcos, la division de Cassini y anillo_a), para ocultar los
# que pierda. Centrado en el origen, mira hacia -Z.
""", mtl(("banda_clara", (0.90, 0.78, 0.54), None), ("banda", (0.82, 0.66, 0.42), None), ("banda_oscura", (0.70, 0.52, 0.34), None),
         ("anillo_oscuro", (0.55, 0.48, 0.42), None), ("anillo_brillante", (1.0, 0.94, 0.78), (0.12, 0.1, 0.06)),
         ("anillo_medio", (0.86, 0.78, 0.60), None)) + MTL_CARA)

# =====================================================================
# El juzgado
# =====================================================================
# Origen al centro de la sala, piso en y = 0. El juez al fondo (+Z), el publico adelante (-Z).
# Defensa en +X y la parte demandante en -X: vistas desde el publico (camara hacia +Z), la defensa
# queda a la izquierda de la pantalla, como en Ace Attorney.
m = Modelo()
v = Voxels(0.1)
for i in range(-50, 50):
    for k in range(-45, 56):
        if -6 <= i <= 5 and k < 28:
            v.put(i, 0, k, "alfombra", "alfombra_roja")
        else:
            mat = "madera" if (i // 3) % 2 == 0 else "madera_clara"
            v.put(i, 0, k, f"piso_{mat}", mat)
# La pared del fondo y las de los lados (con ventanas a las nubes de Venus)
for i in range(-51, 51):
    for j in range(1, 46):
        mat = "madera_oscura" if j <= 12 else "pared"
        v.put(i, j, 56, f"pared_{mat}", mat)
for lado in (-51, 50):
    for k in range(-45, 56):
        for j in range(1, 46):
            ventana = 16 <= j <= 36 and (-32 <= k <= -14 or 6 <= k <= 24)
            if ventana:
                if j in (16, 36) or k in (-32, -14, 6, 24) or j == 26 or k in (-23, 15):
                    v.put(lado, j, k, "marcos", "oro_juzgado")
                continue
            mat = "madera_oscura" if j <= 12 else "pared"
            v.put(lado, j, k, f"pared_{mat}", mat)
# Columnas pegadas a las paredes
for (x, z) in ((-4.6, -4.0), (-4.6, 0.0), (-4.6, 5.2), (4.6, -4.0), (4.6, 0.0), (4.6, 5.2)):
    for j in range(1, 46):
        for i in range(-3, 3):
            for k in range(-3, 3):
                if (i + 0.5) ** 2 + (k + 0.5) ** 2 <= 7.5:
                    mat = "oro_juzgado" if j <= 2 or j >= 44 else "marmol"
                    v.put(i + round(x / 0.1), j, k + round(z / 0.1), "capiteles" if mat == "oro_juzgado" else "columnas", mat)
# El escudo de Venus en la pared (el juzgado esta en Venus): el simbolo ♀, en oro
for i in range(-10, 10):
    for j in range(18, 46):
        x, y = i + 0.5, j + 0.5 - 37
        circulo = 25 <= x * x + y * y <= 56
        cruz = (abs(x) < 1.1 and 20 <= j <= 29) or (abs(x) < 4.6 and 24 <= j <= 25)
        if circulo or cruz:
            v.put(i, j, 55, "escudo", "oro_juzgado")
# El estrado del juez, con su orilla dorada
for i in range(-16, 16):
    for k in range(30, 41):
        for j in range(1, 13):
            borde = j == 12 or i in (-16, 15) or k == 30 and j in (1, 6, 11)
            v.put(i, j, k, "estrado_oro" if borde else "estrado", "oro_juzgado" if borde else "madera_oscura")
# El banquillo de los testigos
for i in range(-5, 5):
    for k in range(6, 10):
        for j in range(1, 6):
            v.put(i, j, k, "banquillo_tapa" if j == 5 else "banquillo", "madera_oscura" if j == 5 else "madera")
# Las mesas: defensa (-X) y parte demandante (+X)
for (i0, i1) in ((-28, -25), (24, 27)):
    for i in range(i0, i1 + 1):
        for k in range(-6, 13):
            for j in range(1, 6):
                tapa = j == 5
                v.put(i, j, k, "mesas_tapa" if tapa else "mesas", "madera_oscura" if tapa else "madera")
# El escaloncito de la defensa (para que Miku alcance a ver por encima de la mesa)
for i in range(28, 36):
    for k in range(-5, 12):
        for j in (1, 2):
            v.put(i, j, k, "escalon", "madera_oscura")
# La baranda entre la sala y el publico
for i in range(-45, 45):
    if -7 <= i <= 6:
        continue
    for j in range(1, 9):
        if j == 8 or i % 5 == 0:
            v.put(i, j, -16, "baranda", "madera_oscura")
# Las bancas del publico (dos filas a cada lado del pasillo)
for k0 in (-26, -36):
    for i in list(range(-42, -8)) + list(range(8, 42)):
        for k in range(k0, k0 + 3):
            v.put(i, 4, k, "bancas", "madera")
        for j in range(1, 4):
            if i % 8 == 0:
                v.put(i, j, k0 + 1, "bancas", "madera")
        for j in range(5, 9):
            v.put(i, j, k0 - 1, "bancas_respaldo", "madera_oscura")
v.mesh(m)
# El mazo del juez y su base (voxel de 0.03)
w = Voxels(0.03, origen=(0.0, 1.3, 0.0))
w.box("mazo_base", "madera_clara", 30, 36, 0, 1, 104, 110)
w.box("mazo", "madera_oscura", 32, 34, 2, 4, 101, 113)
w.box("mazo_mango", "madera_clara", 33, 33, 3, 3, 113, 122)
w.mesh(m)
m.write("juzgado", """# El juzgado de lo familiar de Venus (voxel de 0.1), homenaje a Ace Attorney. Origen al centro de la
# sala, piso en y = 0. Estrado del juez en z 3.0..4.1 (arriba en y = 1.3), banquillo de los testigos en
# z 0.6..1.0 (alto 0.6), mesas en x -2.8..-2.5 (demandante) y 2.4..2.8 (defensa), de 0.6 de alto (z -0.6..1.3),
# baranda en z = -1.6 y bancas del publico atras. Ventanas a las nubes en las paredes de los lados.
""", mtl(("madera", (0.62, 0.38, 0.20), None), ("madera_clara", (0.72, 0.47, 0.26), None), ("madera_oscura", (0.40, 0.22, 0.12), None),
         ("alfombra_roja", (0.70, 0.10, 0.14), None), ("pared", (0.78, 0.68, 0.54), None), ("marmol", (0.86, 0.83, 0.77), None),
         ("oro_juzgado", (1.0, 0.78, 0.30), (0.12, 0.08, 0.0))))
