# Genera los personajes que se escogen en Space Adventure (los que se desbloquean con el pase):
# por cada uno, modelos/personaje_<id>_piloto.obj (sentado manejando, como miku_piloto.obj) y
# modelos/personaje_<id>_baile.obj (de pie, con los pivotes para bailar, como miku_baile.obj).
# Uso: python3 herramientas/generar_personajes.py
#
# Todos usan el MISMO esqueleto chibi que Miku (voxel de 0.04, cabeza de 14 x 12 x 14) y las
# mismas partes por nombre (torso*, brazo_*, manga_*, mano_*, muslo_*, bota_*, suela_*, coleta_*,
# cara_*): asi el juego los pone en cualquier escena donde iba Miku, los hace bailar y les cambia
# la cara sin saber quien es quien. Lo que cambia es la pintura de la cabeza, los accesorios y los
# colores del cuerpo.

import math
from modelo3d import *

U = 0.04
HI0, HI1, HJ0, HJ1, HK0, HK1 = -7, 6, 10, 21, -7, 6
DJ = 6  # de pie, la cabeza va DJ filas mas arriba que sentada


def mat(nombre, kd, ks=0.1, ns=8, ke=None, d=None):
    s = f"newmtl {nombre}\nKd {kd[0]:.2f} {kd[1]:.2f} {kd[2]:.2f}\nKs {ks} {ks} {ks}\nNs {ns}\n"
    if ke:
        s += f"Ke {ke[0]:.2f} {ke[1]:.2f} {ke[2]:.2f}\n"
    if d:
        s += f"d {d}\n"
    return s + "\n"


SILLA = mat("asiento", (0.45, 0.33, 0.62)) + mat("volante", (0.72, 0.58, 0.92), 0.5, 48) + mat("columna", (0.30, 0.30, 0.35), 0.6, 64)


def en_cabeza(i, j, k):
    """La cabeza: caja con las aristas redondeadas (como la de Miku)."""
    return (i in (HI0, HI1)) + (j in (HJ0, HJ1)) + (k in (HK0, HK1)) < 2


def cabeza(vx, dj, pintar, forma=en_cabeza):
    """Llena la cabeza; pintar(i, j, k) -> (parte, material) con j SIN dj (j de la cabeza sentada)."""
    for i in range(HI0, HI1 + 1):
        for j in range(HJ0, HJ1 + 1):
            for k in range(HK0, HK1 + 1):
                if forma(i, j, k):
                    r = pintar(i, j, k)
                    if r:
                        vx.put(i, j + dj, k, *r)


def cara_libre(i, j, k):
    """El frente de la cara donde van los ojos y la boca (tiene que ser piel)."""
    return k <= HK0 + 1 and -5 <= i <= 4 and 11 <= j <= 16


def pelo_como_miku(pelo="pelo", flequillo=17):
    """Pintura de cabeza con pelo atras, a los lados y arriba, y flequillo."""
    def pintar(i, j, k):
        if j >= 19 or k >= 3 or i in (HI0, HI1):
            return "pelo", pelo
        if k <= HK0 + 1 and j >= flequillo:
            return "pelo", pelo
        return "cabeza", "piel"
    return pintar


# ---------- cuerpos ----------
# c: materiales del cuerpo. Con falda=False, la "falda" es un short o pantalon (pegado al cuerpo).
# w: medio ancho extra del torso (1 = gordito); los brazos se corren con el.

def cuerpo_de_pie(vx, c, falda=True, w=0):
    vx.box("suela_{lado}", c["suela"], 0, 1, 0, 0, -3, 1, espejo=True)
    vx.box("bota_{lado}", c["bota"], 0, 1, 1, 4, -2, 1, espejo=True)
    vx.box("bota_{lado}", c["bota"], 0, 1, 1, 2, -3, -3, espejo=True)
    vx.box("bota_{lado}_borde", c.get("bota_borde", c["bota"]), 0, 1, 5, 5, -2, 1, espejo=True)
    vx.box("muslo_{lado}", c["muslo"], 0, 1, 6, 7, -1, 0, espejo=True)
    if falda:
        vx.box("falda_borde", c.get("falda_borde", c["falda"]), -5, 4, 7, 7, -3, 2)
        vx.box("falda", c["falda"], -4, 3, 8, 8, -3, 2)
        vx.box("falda", c["falda"], -3, 2, 9, 9, -2, 1)
    else:
        vx.box("falda", c["falda"], -3 - w, 2 + w, 7, 9, -2, 1)
    vx.box("torso", c["torso"], -3 - w, 2 + w, 10, 14, -2, 1)
    if c.get("corbata"):
        vx.box("corbata", c["corbata"], -1, 0, 12, 14, -3, -3)
    vx.box("cuello", c.get("cuello", "piel"), -1, 0, 15, 15, -1, 0)
    a = 3 + w
    vx.box("brazo_{lado}", c.get("brazo", c["manga"]), a, a, 13, 14, -1, 0, espejo=True)
    vx.box("manga_{lado}", c["manga"], a, a, 10, 12, -1, 0, espejo=True)
    vx.box("manga_{lado}_borde", c.get("manga_borde", c["manga"]), a, a, 9, 9, -1, 0, espejo=True)
    vx.box("mano_{lado}", c.get("mano", "piel"), a, a + 1, 7, 8, -1, 0, espejo=True)


def silla(vx):
    """El asiento y el volante (aro de 4 x 4 con hueco) con su columna."""
    vx.box("asiento", "asiento", -5, 4, -2, -1, -4, 3)
    vx.box("asiento", "asiento", -4, 3, 0, 8, 3, 4)
    vx.box("volante", "volante", -2, 1, 4, 7, -9, -9, si=lambda i, j, k: i in (-2, 1) or j in (4, 7))
    vx.box("volante_columna", "columna", -1, 0, 3, 4, -11, -10)
    vx.box("volante_columna", "columna", -1, 0, 1, 2, -12, -12)


def cuerpo_sentado(vx, c, falda=True, w=0):
    silla(vx)
    if falda:
        vx.box("falda_borde", c.get("falda_borde", c["falda"]), -5, 4, 0, 0, -5, 2)
        vx.box("falda", c["falda"], -4, 3, 1, 2, -5, 2)
        vx.box("falda", c["falda"], -3, 2, 3, 3, -4, 1)
    else:
        vx.box("falda", c["falda"], -3 - w, 2 + w, 0, 3, -5, 1)
    vx.box("torso", c["torso"], -3 - w, 2 + w, 4, 8, -2, 1)
    if c.get("corbata"):
        vx.box("corbata", c["corbata"], -1, 0, 6, 8, -3, -3)
    vx.box("cuello", c.get("cuello", "piel"), -1, 0, 9, 9, -1, 0)
    a = 3 + w
    vx.box("brazo_{lado}", c.get("brazo", c["manga"]), a, a, 7, 8, -1, 0, espejo=True)
    vx.box("manga_{lado}", c["manga"], a, a, 6, 7, -5, -2, espejo=True)
    vx.box("manga_{lado}_borde", c.get("manga_borde", c["manga"]), a, a, 6, 7, -6, -6, espejo=True)
    vx.box("mano_{lado}", c.get("mano", "piel"), 2, 2, 6, 7, -8, -7, espejo=True)
    vx.box("muslo_{lado}", c["muslo"], 1, 2, 0, 1, -6, -3, espejo=True)
    vx.box("bota_{lado}_borde", c.get("bota_borde", c["bota"]), 1, 2, 1, 1, -8, -7, espejo=True)
    vx.box("bota_{lado}", c["bota"], 1, 2, -3, 0, -8, -7, espejo=True)
    vx.box("bota_{lado}", c["bota"], 1, 2, -4, -4, -9, -7, espejo=True)
    vx.box("suela_{lado}", c["suela"], 1, 2, -5, -5, -9, -7, espejo=True)


# ---------- caras fijas (para los que no ponen caras de anime) ----------
def cara_fija(piezas):
    return {emo: piezas for emo in CARAS}


def rects(filas, x0, y_top):
    return pixeles(filas, x0, y_top)


# El Creeper: la cara de siempre, en pixeles de medio voxel (x = 0 al centro, y = 0 abajo de la cara).
CARA_CREEPER = cara_fija([
    ("ojos", "creeper_negro", espejo(rects(["####"] * 4, 2, 12))),
    ("boca", "creeper_negro", rects(["....####....", "....####....", "..########..", "..########..", "..##....##..", "..##....##.."], -6, 8)),
])
# Steve: ojos blanco y azul, nariz, boca y barbita.
CARA_STEVE = cara_fija([
    ("ojos", "steve_blanco", rects(["##......##", "##......##"], -5, 9) ),
    ("pupilas", "steve_azul", rects(["..##..##..", "..##..##.."], -5, 9)),
    ("nariz", "steve_cafe", rects(["##", "##"], -1, 7)),
    ("boca", "steve_barba", rects(["..######..", "##......##"], -5, 4)),
])
# El gato que grita: ojos redondos bien abiertos y la boca abierta, siempre.
CARA_GATO = cara_fija([
    ("ojos", "ojo", espejo(rects([".###.", "#####", "#####", "#####", ".###."], 2, 11))),
    ("brillo", "steve_blanco", espejo(rects(["#"], 3, 10))),
    ("boca", "gato_boca", rects(["..####..", ".######.", "########", "########", ".######."], -4, 6)),
    ("lengua", "gato_lengua", rects(["####"], -2, 2)),
])


# ---------- los personajes ----------
# Cada uno: id, materiales (texto .mtl), colores del cuerpo, falda, ancho, pintura de cabeza,
# extras(vx, dj, sentado) para accesorios, y caras (None = las de anime).
PERSONAJES = []


def personaje(id, mtl, c, pintar, extras=None, falda=False, w=0, caras=None, forma=en_cabeza, coletas=False,
              cuerpo=None, cara=None, hombro=None, cadera=None):
    """cuerpo(vx, sentado): arma TODO el personaje (para los que no son humanitos, como Kirby
    o el Creeper). cara = (z del frente, y de la cara sentado, y de pie) en voxels, si no es la
    de siempre. hombro / cadera = (x, y) en voxels de los pivotes del baile."""
    PERSONAJES.append(dict(id=id, mtl=mtl, c=c, pintar=pintar, extras=extras, falda=falda, w=w, caras=caras, forma=forma, coletas=coletas,
                           cuerpo=cuerpo, cara=cara, hombro=hombro, cadera=cadera))


# --- El Chavo del 8: gorro de orejeras a cuadros, camisa a rayas, tirantes y parche ---
def chavo_extras(vx, dj, sentado):
    for i in range(-8, 8):
        for j in range(19, 23):
            for k in range(-8, 8):
                if (i in (-8, 7)) + (k in (-8, 7)) + (j == 22) >= 2:
                    continue
                if j >= 19 + (1 if k < -5 else 0):
                    cuadro = (i + k + j) % 3 == 0
                    vx.put(i, j + dj, k, "gorro_cuadros" if cuadro else "gorro", "gorro_cuadros" if cuadro else "gorro")
    vx.box("gorro_orejeras", "gorro", -8, -8, 14 + dj, 18 + dj, -3, 2)
    vx.box("gorro_orejeras", "gorro", 7, 7, 14 + dj, 18 + dj, -3, 2)
    t = 4 if sentado else 10
    for j in range(t, t + 5, 2):
        vx.box("torso_rayas", "chavo_raya", -3, 2, j, j, -3, -3)
        vx.box("torso_rayas", "chavo_raya", -3, 2, j, j, 2, 2)
    vx.box("torso_tirantes", "tirantes", -2, -2, t, t + 4, -3, 2, si=lambda i, j, k: k in (-3, 2))
    vx.box("torso_tirantes", "tirantes", 1, 1, t, t + 4, -3, 2, si=lambda i, j, k: k in (-3, 2))
    if not sentado:
        vx.box("falda_parche", "parche", 0, 1, 8, 9, -3, -3)

personaje("chavo",
          mat("piel", (1.0, 0.84, 0.70)) + mat("pelo", (0.55, 0.28, 0.12)) + mat("gorro", (0.52, 0.42, 0.24)) + mat("gorro_cuadros", (0.30, 0.38, 0.22))
          + mat("camisa", (0.88, 0.82, 0.60)) + mat("chavo_raya", (0.62, 0.50, 0.30)) + mat("tirantes", (0.35, 0.24, 0.14))
          + mat("pantalon", (0.55, 0.40, 0.22)) + mat("parche", (0.30, 0.50, 0.25)) + mat("zapato", (0.30, 0.20, 0.12)) + mat("suela", (0.15, 0.10, 0.08)),
          dict(torso="camisa", manga="camisa", falda="pantalon", muslo="pantalon", bota="zapato", suela="suela"),
          pelo_como_miku("pelo", 18), chavo_extras)


# --- Doctor Simi: pelo y bigote blancos, bata blanca y corbata azul ---
def simi_pinta(i, j, k):
    if j >= 18 or k >= 2 or (i in (HI0, HI1) and j >= 13):
        return "pelo", "canas"
    return "cabeza", "piel"

def simi_extras(vx, dj, sentado):
    vx.box("bigote", "canas", -4, 3, 12 + dj, 12 + dj, -8, -8)
    vx.box("bigote", "canas", -5, 4, 11 + dj, 11 + dj, -8, -8, si=lambda i, j, k: i not in (-1, 0))
    t = 4 if sentado else 10
    vx.box("torso_logo", "simi_azul", 1, 2, t + 3, t + 3, -4, -4)
    # el estetoscopio colgado del cuello (una U gris con su campana)
    vx.box("torso_estetoscopio", "metal_gris", -3, -3, t + 2, t + 4, -4, -4)
    vx.box("torso_estetoscopio", "metal_gris", -3, 0, t + 1, t + 1, -4, -4)
    vx.box("torso_estetoscopio", "metal_gris", 0, 0, t + 2, t + 4, -4, -4)
    vx.box("torso_campana", "simi_azul", -2, -1, t, t, -5, -4)
    vx.box("torso_bolsillo", "bata_sombra", -4, -3, t, t + 1, -3, -3)

personaje("simi",
          mat("piel", (1.0, 0.82, 0.68)) + mat("canas", (0.95, 0.95, 0.97), 0.2, 16) + mat("lentes", (0.12, 0.12, 0.16), 0.5, 48)
          + mat("bata", (0.97, 0.97, 1.0), 0.2, 16) + mat("bata_sombra", (0.80, 0.82, 0.90)) + mat("metal_gris", (0.55, 0.58, 0.65), 0.6, 48) + mat("simi_azul", (0.16, 0.34, 0.75)) + mat("pantalon", (0.18, 0.22, 0.40)) + mat("zapato", (0.10, 0.10, 0.12)),
          dict(torso="bata", manga="bata", falda="bata", falda_borde="bata", muslo="pantalon", bota="zapato", suela="zapato", corbata="simi_azul"),
          simi_pinta, simi_extras, falda=True, w=1)


# --- La Chilindrina: trenzas pelirrojas con monos, lentes, pecas, sueter verde y falda roja ---
def chilindrina_pinta(i, j, k):
    if (i, j) in ((-5, 12), (-3, 12), (2, 12), (4, 12), (-4, 13), (3, 13)) and k == HK0:
        return "pecas", "pecas"
    return pelo_como_miku("pelo", 17)(i, j, k)

def chilindrina_extras(vx, dj, sentado):
    # lentes: dos marcos alrededor de los ojos
    for x0 in (-6, 0):
        vx.box("lentes", "lentes", x0, x0 + 5, 12 + dj, 17 + dj, -8, -8,
               si=lambda i, j, k, x0=x0: i in (x0, x0 + 5) or j in (12 + dj, 17 + dj))
    # trenzas (son coletas: se mueven igual que las de Miku)
    vx.box("liga_{lado}", "mono", 7, 8, 15 + dj, 16 + dj, -1, 1, espejo=True)
    vx.box("coleta_{lado}", "pelo", 7, 8, 8 + dj, 14 + dj, 0, 1, espejo=True)
    vx.box("coleta_{lado}_puntas", "mono", 7, 8, 7 + dj, 7 + dj, 0, 1, espejo=True)

personaje("chilindrina",
          mat("piel", (1.0, 0.86, 0.76)) + mat("pelo", (0.78, 0.34, 0.14)) + mat("pecas", (0.70, 0.40, 0.25)) + mat("lentes", (0.55, 0.24, 0.16), 0.5, 48)
          + mat("mono", (0.30, 0.70, 0.35)) + mat("sueter", (0.28, 0.62, 0.36)) + mat("falda_roja", (0.85, 0.20, 0.22)) + mat("calceta", (0.96, 0.96, 0.96))
          + mat("zapato", (0.12, 0.10, 0.10)),
          dict(torso="sueter", manga="sueter", falda="falda_roja", muslo="piel", bota="calceta", bota_borde="calceta", suela="zapato"),
          chilindrina_pinta, chilindrina_extras, falda=True, coletas=True)


# --- El Chapulin Colorado: capucha roja con antenitas, corazon amarillo y short amarillo ---
def chapulin_pinta(i, j, k):
    if cara_libre(i, j, k) or (k == HK0 + 2 and -5 <= i <= 4 and 11 <= j <= 16):
        return "cabeza", "piel"
    return "capucha", "chapulin_rojo"

def chapulin_extras(vx, dj, sentado):
    vx.box("antenitas", "negro_mate", -4, -4, 22 + dj, 25 + dj, -2, -2)
    vx.box("antenitas", "negro_mate", 3, 3, 22 + dj, 25 + dj, -2, -2)
    vx.box("antenitas_bolas", "chapulin_amarillo", -5, -3, 26 + dj, 27 + dj, -3, -1)
    vx.box("antenitas_bolas", "chapulin_amarillo", 2, 4, 26 + dj, 27 + dj, -3, -1)
    t = 4 if sentado else 10
    vx.pattern("torso_corazon", "chapulin_amarillo", ["##.##", "#####", ".###.", "..#.."], -3, t + 4, -3)

personaje("chapulin",
          mat("piel", (1.0, 0.84, 0.70)) + mat("chapulin_rojo", (0.88, 0.10, 0.12), 0.3, 24) + mat("chapulin_amarillo", (1.0, 0.85, 0.15), 0.3, 24, ke=(0.15, 0.12, 0.0))
          + mat("negro_mate", (0.10, 0.08, 0.08)),
          dict(torso="chapulin_rojo", manga="chapulin_rojo", falda="chapulin_amarillo", muslo="chapulin_rojo", bota="chapulin_amarillo", suela="chapulin_rojo"),
          chapulin_pinta, chapulin_extras)


# --- Shrek: verde, calvo, orejas de trompeta, chaleco cafe ---
def shrek_extras(vx, dj, sentado):
    for lado, i0, i1 in (("der", 7, 8), ("izq", -9, -8)):
        vx.box("orejas", "piel", i0, i1, 16 + dj, 17 + dj, -1, 0)
    vx.box("orejas", "piel", 9, 9, 15 + dj, 18 + dj, -2, 1)
    vx.box("orejas", "piel", -10, -10, 15 + dj, 18 + dj, -2, 1)
    vx.box("cejas", "shrek_ceja", -5, -2, 17 + dj, 17 + dj, -8, -8)
    vx.box("cejas", "shrek_ceja", 1, 4, 17 + dj, 17 + dj, -8, -8)
    t = 4 if sentado else 10
    vx.box("torso_chaleco", "chaleco", -4, 3, t + 1, t + 4, -3, 2, si=lambda i, j, k: i in (-4, -3, 2, 3) or k == 2)
    vx.box("torso_cinturon", "cinturon", -4, 3, t, t, -3, 2, si=lambda i, j, k: k in (-3, 2) or i in (-4, 3))
    vx.box("torso_hebilla", "hebilla", -1, 0, t, t, -4, -4)
    vx.box("torso_cordones", "cinturon", -1, 0, t + 3, t + 4, -3, -3, si=lambda i, j, k: (i + j) % 2 == 0)

personaje("shrek",
          mat("piel", (0.56, 0.72, 0.22)) + mat("shrek_ceja", (0.30, 0.38, 0.10)) + mat("camisa", (0.92, 0.88, 0.72)) + mat("chaleco", (0.42, 0.28, 0.14))
          + mat("cinturon", (0.30, 0.20, 0.10)) + mat("hebilla", (0.85, 0.70, 0.30), 0.6, 48) + mat("pantalon", (0.38, 0.28, 0.16)) + mat("bota", (0.24, 0.16, 0.10)),
          dict(torso="camisa", manga="camisa", mano="piel", falda="pantalon", muslo="pantalon", bota="bota", suela="bota", cuello="piel"),
          lambda i, j, k: ("cabeza", "piel"), shrek_extras, w=1)


# --- Doge: shiba color miel, hocico blanco, orejitas, cola enroscada ---
def doge_pinta(i, j, k):
    if k <= HK0 + 1 and j <= 13 and -4 <= i <= 3:
        return "hocico", "blanco_perro"
    if k <= HK0 + 1 and j in (14, 15) and i in (-6, -5, 4, 5):
        return "hocico", "blanco_perro"
    return "cabeza", "piel"

def doge_extras(vx, dj, sentado):
    vx.box("nariz", "ojo", -1, 0, 13 + dj, 13 + dj, -8, -8)
    for i0 in (-6, 3):
        vx.box("orejas", "piel", i0, i0 + 2, 22 + dj, 22 + dj, -2, 0)
        vx.box("orejas", "piel", i0 + (0 if i0 < 0 else 1), i0 + (1 if i0 < 0 else 2), 23 + dj, 23 + dj, -1, 0)
        vx.box("orejas_dentro", "blanco_perro", i0 + 1, i0 + 1, 22 + dj, 22 + dj, -3, -3)
    t = 4 if sentado else 10
    vx.box("torso_panza", "blanco_perro", -2, 1, t, t + 3, -3, -3)
    vx.box("torso_cola", "piel", -1, 0, t + 1, t + 3, 2, 3)
    vx.box("torso_cola_punta", "blanco_perro", -1, 0, t + 3, t + 4, 4, 4)

personaje("doge",
          mat("piel", (0.88, 0.62, 0.30)) + mat("blanco_perro", (0.98, 0.95, 0.88)),
          dict(torso="piel", manga="piel", mano="blanco_perro", falda="piel", muslo="piel", bota="piel", suela="blanco_perro", cuello="piel"),
          doge_pinta, doge_extras)


# --- El gato que grita: gris, orejas con rosa adentro, la boca abierta siempre ---
def gato_pinta(i, j, k):
    if k <= HK0 + 1 and j <= 12 and -3 <= i <= 2:
        return "hocico", "gato_blanco"
    return "cabeza", "piel"

def gato_extras(vx, dj, sentado):
    for i0 in (-6, 3):
        vx.box("orejas", "piel", i0, i0 + 2, 22 + dj, 22 + dj, -2, 0)
        vx.box("orejas", "piel", i0 + (0 if i0 < 0 else 1), i0 + (1 if i0 < 0 else 2), 23 + dj, 24 + dj, -1, 0)
        vx.box("orejas_dentro", "gato_rosa", i0 + 1, i0 + 1, 22 + dj, 23 + dj, -3, -3)
    vx.box("bigotes", "gato_blanco", -9, -6, 12 + dj, 12 + dj, -8, -8)
    vx.box("bigotes", "gato_blanco", 5, 8, 12 + dj, 12 + dj, -8, -8)
    t = 4 if sentado else 10
    vx.box("torso_panza", "gato_blanco", -2, 1, t, t + 3, -3, -3)
    vx.box("torso_cola", "piel", -1, 0, t, t + 5, 2, 2)
    vx.box("torso_cola", "piel", -1, 0, t + 5, t + 6, 3, 4)

personaje("gato",
          mat("piel", (0.62, 0.62, 0.66)) + mat("gato_blanco", (0.96, 0.96, 0.98)) + mat("gato_rosa", (1.0, 0.62, 0.72))
          + mat("gato_boca", (0.35, 0.05, 0.10)) + mat("gato_lengua", (1.0, 0.45, 0.55)) + mat("steve_blanco", (1.0, 1.0, 1.0)),
          dict(torso="piel", manga="piel", mano="gato_blanco", falda="piel", muslo="piel", bota="piel", suela="gato_blanco", cuello="piel"),
          gato_pinta, gato_extras, caras=CARA_GATO)


# --- Patricio Estrella: rosa, cabeza de punta, short verde con flores ---
def patricio_medio(j):
    """Medio ancho de la cabeza en la fila j: se angosta hacia arriba hasta la punta."""
    return 7.0 if j < 15 else 7.0 - (j - 14) * 0.62

def patricio_forma(i, j, k):
    if not en_cabeza(i, j, k):
        return False
    m = patricio_medio(j)
    return abs(i + 0.5) <= m and abs(k + 0.5) <= m * 0.85 + 0.6

def patricio_extras(vx, dj, sentado):
    # la punta sigue arriba de la caja de la cabeza
    for j in range(22, 26):
        m = patricio_medio(j)
        vx.box("cabeza", "piel", -8, 7, j + dj, j + dj, -8, 7,
               si=lambda i, jj, k, m=m: abs(i + 0.5) <= m and abs(k + 0.5) <= m * 0.85 + 0.6)
    vx.box("cejas", "ojo", -5, -3, 17 + dj, 17 + dj, -8, -8)
    vx.box("cejas", "ojo", 2, 4, 17 + dj, 17 + dj, -8, -8)
    if not sentado:
        for (i, j, k) in ((-2, 8, -3), (1, 9, -3), (2, 7, -3), (-3, 7, 2), (0, 8, 2)):
            vx.put(i, j, k, "falda_flores", "flor")

personaje("patricio",
          mat("piel", (1.0, 0.58, 0.68)) + mat("short", (0.42, 0.78, 0.30)) + mat("flor", (0.62, 0.30, 0.80)),
          dict(torso="piel", manga="piel", mano="piel", falda="short", muslo="piel", bota="piel", suela="piel", cuello="piel"),
          lambda i, j, k: ("cabeza", "piel"), patricio_extras, forma=patricio_forma)


# --- Ugandan Knuckles: rojo, boca beige gigante, rastas, media luna en el pecho ---
def knuckles_extras(vx, dj, sentado):
    vx.box("boca_grande", "beige", -4, 3, 10 + dj, 12 + dj, -9, -8)
    for i in range(-6, 6, 2):
        vx.box("rastas", "rojo_oscuro", i, i, 8 + dj, 18 + dj, 7, 7)
    vx.box("rastas", "rojo_oscuro", -7, 6, 16 + dj, 18 + dj, 7, 7)
    t = 4 if sentado else 10
    vx.pattern("torso_luna", "blanco", [".####.", "##..##", "#....#"], -3, t + 3, -3)

personaje("knuckles",
          mat("piel", (0.82, 0.10, 0.12), 0.2, 16) + mat("rojo_oscuro", (0.55, 0.05, 0.08)) + mat("beige", (0.96, 0.82, 0.62))
          + mat("blanco", (0.97, 0.97, 0.97)) + mat("zapato", (0.20, 0.55, 0.30)),
          dict(torso="piel", manga="piel", mano="blanco", falda="piel", muslo="piel", bota="piel", bota_borde="blanco", suela="zapato", cuello="piel"),
          lambda i, j, k: ("cabeza", "piel"), knuckles_extras)


# --- Mario: gorra roja, bigote, overol azul con botones amarillos, guantes ---
def mario_pinta(i, j, k):
    if j >= 18:
        return None  # arriba va la gorra
    if (k >= 3 or i in (HI0, HI1)) and j >= 13:
        return "pelo", "mario_pelo"
    return "cabeza", "piel"

def mario_extras(vx, dj, sentado):
    vx.box("gorra", "mario_rojo", -7, 6, 18 + dj, 21 + dj, -7, 6, si=lambda i, j, k: en_cabeza(i, j - dj, k) or j < 21 + dj)
    vx.box("gorra", "mario_rojo", -6, 5, 18 + dj, 18 + dj, -10, -8)
    vx.box("gorra_logo", "blanco", -2, 1, 19 + dj, 20 + dj, -8, -8)
    vx.box("bigote", "mario_pelo", -4, 3, 12 + dj, 12 + dj, -8, -8)
    vx.box("bigote", "mario_pelo", -5, 4, 11 + dj, 11 + dj, -8, -8, si=lambda i, j, k: i not in (-1, 0))
    vx.box("nariz", "piel", -1, 0, 13 + dj, 14 + dj, -9, -8)
    vx.box("orejas", "piel", -8, -8, 13 + dj, 15 + dj, -1, 1)
    vx.box("orejas", "piel", 7, 7, 13 + dj, 15 + dj, -1, 1)
    vx.box("patillas", "mario_pelo", -7, -7, 13 + dj, 17 + dj, -6, -4)
    vx.box("patillas", "mario_pelo", 6, 6, 13 + dj, 17 + dj, -6, -4)
    t = 4 if sentado else 10
    vx.box("torso_overol", "overol", -2, 1, t, t + 3, -3, -3)
    vx.box("torso_overol", "overol", -3, 2, t, t, -3, 2)
    vx.box("torso_botones", "boton", -2, -2, t + 3, t + 3, -4, -4)
    vx.box("torso_botones", "boton", 1, 1, t + 3, t + 3, -4, -4)

personaje("mario",
          mat("piel", (1.0, 0.80, 0.64)) + mat("mario_pelo", (0.28, 0.16, 0.08)) + mat("mario_rojo", (0.90, 0.10, 0.10), 0.3, 24)
          + mat("blanco", (0.98, 0.98, 0.98)) + mat("overol", (0.15, 0.30, 0.85), 0.2, 16) + mat("boton", (1.0, 0.85, 0.15), 0.5, 48)
          + mat("zapato", (0.42, 0.24, 0.10)),
          dict(torso="mario_rojo", manga="mario_rojo", mano="blanco", falda="overol", muslo="overol", bota="zapato", suela="zapato"),
          mario_pinta, mario_extras, w=1)


# --- Kirby: una bola rosa, con bracitos a los lados y sus pies rojos grandotes ---
def kirby_cuerpo(vx, sentado):
    # La bola (cy = centro, r = radio, en voxels) y los pies abajo, un poco hacia el frente.
    cy, r = (8.0, 7.6) if sentado else (10.5, 8.2)
    bola = lambda i, j, k: (i + 0.5) ** 2 + (j + 0.5 - cy) ** 2 + (k + 0.5) ** 2 <= r * r
    vx.box("cabeza", "piel", -9, 8, int(cy - r) - 1, int(cy + r) + 1, -9, 8, si=bola)
    if sentado:
        silla(vx)
        vx.box("bota_{lado}", "pie_rojo", 1, 4, -1, 1, -9, -5, espejo=True,
               si=lambda i, j, k: not (i in (1, 4) and k in (-9, -5)))
        vx.box("manga_{lado}", "piel", 7, 8, 5, 7, -6, -3, espejo=True)
        vx.box("mano_{lado}", "piel", 5, 6, 5, 6, -8, -7, espejo=True)
    else:
        vx.box("bota_{lado}", "pie_rojo", 1, 5, 0, 2, -4, 2, espejo=True,
               si=lambda i, j, k: not (i in (1, 5) and k in (-4, 2)) and not (j == 2 and (i in (1, 5) or k in (-4, 2))))
        vx.box("brazo_{lado}", "piel", 8, 9, 9, 11, -1, 1, espejo=True,
               si=lambda i, j, k: not (i == 9 and j in (9, 11)))
    # rubor siempre (aunque la cara cambie), como en los juegos
    vx.box("cachetes", "rubor_kirby", 3, 5, (7 if sentado else 9), (7 if sentado else 9), -9, -9, espejo=True,
           si=lambda i, j, k: bola(i, j, k + 1))

personaje("kirby",
          mat("piel", (1.0, 0.64, 0.76), 0.25, 24) + mat("pie_rojo", (0.86, 0.10, 0.26), 0.3, 24) + mat("rubor_kirby", (1.0, 0.42, 0.58)),
          {}, None, cuerpo=kirby_cuerpo,
          # la cara va pegada al frente de la bola, a media altura
          cara=(-9, 8.5, 11.0), hombro=(8.5, 10.0), cadera=(3.0, 2.0))


# --- El Creeper: cabeza cubica con manchas verdes, cuerpo alto, cuatro patas y sin brazos ---
VERDES = ["creeper_verde", "creeper_claro", "creeper_oscuro", "creeper_gris"]

def mancha(i, j, k):
    """Manchas de dos en dos voxels (como los pixeles grandes de Minecraft), siempre las mismas."""
    h = ((i + 40) // 2 * 73856093) ^ ((j + 40) // 2 * 19349663) ^ ((k + 40) // 2 * 83492791)
    return VERDES[[0, 0, 0, 1, 1, 2, 2, 3][h % 8]]

def creeper_cara(i, j):
    """La cara de siempre, en el frente de la cabeza (j de la cabeza sentada: 10 a 21)."""
    ojos = j in (16, 17, 18) and (-5 <= i <= -2 or 1 <= i <= 4)
    boca = (j in (12, 13, 14, 15) and -2 <= i <= 1) or (j in (10, 11, 12, 13) and (i in (-4, -3) or i in (2, 3)))
    return ojos or boca

def creeper_cuerpo(vx, sentado):
    dj = 0 if sentado else DJ
    # la cabeza: un cubo con manchas y la cara negra
    for i in range(HI0, HI1 + 1):
        for j in range(HJ0, HJ1 + 1):
            for k in range(HK0, HK1 + 1):
                if k == HK0 and creeper_cara(i, j):
                    vx.put(i, j + dj, k, "cabeza_cara", "creeper_negro")
                else:
                    m = mancha(i, j, k)
                    vx.put(i, j + dj, k, "cabeza" if m == VERDES[0] else "cabeza_" + m, m)
    def bloque(nombre, i0, i1, j0, j1, k0, k1, espejo=False):
        for i in range(i0, i1 + 1):
            for j in range(j0, j1 + 1):
                for k in range(k0, k1 + 1):
                    m = mancha(i, j, k)
                    sufijo = "" if m == VERDES[0] else "_" + m
                    vx.put(i, j, k, nombre.replace("{lado}", "{lado}" + sufijo) if "{lado}" in nombre else nombre + sufijo, m, espejo)
    if sentado:
        silla(vx)
        bloque("torso", -4, 3, 0, 9, -2, 1)
        # las patas de adelante, estiradas hacia el volante
        bloque("muslo_{lado}", 0, 3, 0, 2, -6, -3, espejo=True)
        bloque("bota_{lado}", 0, 3, -3, 2, -9, -7, espejo=True)
    else:
        bloque("torso", -4, 3, 6, 15, -2, 1)
        # cuatro patas: dos adelante y dos atras (las de cada lado se mueven juntas)
        for k0 in (-6, 2):
            bloque("muslo_{lado}", 0, 3, 3, 5, k0, k0 + 3, espejo=True)
            bloque("bota_{lado}", 0, 3, 0, 2, k0, k0 + 3, espejo=True)

personaje("creeper",
          mat("creeper_verde", (0.32, 0.70, 0.30)) + mat("creeper_claro", (0.52, 0.84, 0.46)) + mat("creeper_oscuro", (0.18, 0.48, 0.18))
          + mat("creeper_gris", (0.55, 0.62, 0.52)) + mat("creeper_negro", (0.05, 0.07, 0.05)),
          {}, None, cuerpo=creeper_cuerpo, caras=cara_fija([]), cadera=(2.0, 6.0))


# --- Steve: cabeza cuadrada, pelo cafe, playera cian, pantalon azul ---
def steve_pinta(i, j, k):
    if j >= 18 or k >= 2 or (i in (HI0, HI1) and j >= 15):
        return "pelo", "steve_pelo"
    return "cabeza", "piel"

personaje("steve",
          mat("piel", (0.85, 0.62, 0.48)) + mat("steve_pelo", (0.30, 0.20, 0.12)) + mat("steve_playera", (0.20, 0.72, 0.75))
          + mat("steve_pantalon", (0.22, 0.24, 0.62)) + mat("steve_zapato", (0.42, 0.42, 0.45))
          + mat("steve_blanco", (1.0, 1.0, 1.0)) + mat("steve_azul", (0.30, 0.25, 0.70)) + mat("steve_cafe", (0.55, 0.35, 0.25)) + mat("steve_barba", (0.40, 0.25, 0.15)),
          dict(torso="steve_playera", manga="steve_playera", brazo="piel", falda="steve_pantalon", muslo="steve_pantalon", bota="steve_pantalon", suela="steve_zapato"),
          steve_pinta, None, caras=CARA_STEVE, forma=lambda i, j, k: True)


# --- Pikachu: amarillo, orejas largas de punta negra, cachetes rojos, cola de rayo ---
def pikachu_pinta(i, j, k):
    if k == HK0 and j in (11, 12) and i in (-7, -6, 5, 6):
        return "cachetes", "cachete"
    return "cabeza", "piel"

def pikachu_extras(vx, dj, sentado):
    for i0 in (-6, 4):
        vx.box("orejas", "piel", i0, i0 + 1, 22 + dj, 26 + dj, -1, 0)
        vx.box("orejas_puntas", "ojo", i0, i0 + 1, 27 + dj, 28 + dj, -1, 0)
    t = 4 if sentado else 10
    vx.box("torso_rayas", "pika_cafe", -3, 2, t + 2, t + 2, 2, 2)
    vx.box("torso_rayas", "pika_cafe", -3, 2, t + 4, t + 4, 2, 2)
    vx.box("torso_cola", "pika_cafe", -1, 0, t, t + 1, 3, 3)
    vx.box("torso_cola_rayo", "piel", -1, 0, t + 1, t + 3, 4, 4)
    vx.box("torso_cola_rayo", "piel", -1, 0, t + 3, t + 4, 5, 6)
    vx.box("torso_cola_rayo", "piel", -1, 0, t + 4, t + 7, 7, 8)

personaje("pikachu",
          mat("piel", (1.0, 0.86, 0.15), 0.2, 16) + mat("cachete", (0.95, 0.20, 0.18)) + mat("pika_cafe", (0.55, 0.32, 0.12)),
          dict(torso="piel", manga="piel", mano="piel", falda="piel", muslo="piel", bota="piel", suela="piel", cuello="piel"),
          pikachu_pinta, pikachu_extras)


# --- El profe Koch: pelo negro corto, lentes rectangulares, chumpa negra de cuello alto y camisa verde ---
def koch_pinta(i, j, k):
    if j >= 19 or k >= 3 or (i in (HI0, HI1) and j >= 15):
        return "pelo", "pelo"
    if k <= HK0 + 1 and j >= 18 and -6 <= i <= 4:
        return "pelo", "pelo"  # el copete
    return "cabeza", "piel"

def koch_extras(vx, dj, sentado):
    # lentes: marcos negros rectangulares (arriba gruesos) y el puente
    for x0 in (-6, 0):
        vx.box("lentes", "lentes", x0, x0 + 5, 12 + dj, 16 + dj, -8, -8,
               si=lambda i, j, k, x0=x0: i in (x0, x0 + 5) or j in (12 + dj, 16 + dj))
    vx.box("lentes", "lentes", -1, 0, 15 + dj, 15 + dj, -8, -8)
    vx.box("lentes", "lentes", -7, -7, 15 + dj, 15 + dj, -7, 0)
    vx.box("lentes", "lentes", 6, 6, 15 + dj, 15 + dj, -7, 0)
    # la sonrisa de la foto (siempre, aunque cambie la cara)
    vx.box("sonrisa", "boca", -2, 1, 11 + dj, 11 + dj, -8, -8)
    vx.box("sonrisa", "boca", -3, -3, 12 + dj, 12 + dj, -8, -8)
    vx.box("sonrisa", "boca", 2, 2, 12 + dj, 12 + dj, -8, -8)
    t = 4 if sentado else 10
    # el cuello alto de la chumpa y el cierre
    vx.box("torso_cuello_alto", "chumpa", -3, 2, t + 4, t + 5, -3, 2, si=lambda i, j, k: not (-1 <= i <= 0 and k <= -2))
    vx.box("torso_cierre", "cierre", -1, -1, t, t + 2, -3, -3)

personaje("koch",
          mat("piel", (0.92, 0.74, 0.60)) + mat("pelo", (0.09, 0.07, 0.06), 0.3, 24) + mat("lentes", (0.06, 0.06, 0.08), 0.6, 64)
          + mat("chumpa", (0.10, 0.10, 0.13), 0.25, 20) + mat("camisa_verde", (0.20, 0.32, 0.24)) + mat("cierre", (0.45, 0.45, 0.50), 0.6, 48)
          + mat("pantalon", (0.14, 0.14, 0.17)) + mat("zapato", (0.08, 0.07, 0.07), 0.5, 32),
          dict(torso="chumpa", manga="chumpa", falda="pantalon", muslo="pantalon", bota="zapato", suela="zapato", corbata="camisa_verde", cuello="camisa_verde"),
          koch_pinta, koch_extras)


# --- Vegetta777 (su skin de siempre): pelo cafe en picos, el visor morado sobre un ojo, la
#     armadura morada de saiyajin con el peto blanco, hombreras y cinturon dorados, guantes y
#     botas blancas ---
def vegetta_pinta(i, j, k):
    if j >= 19 or k >= 3 or (i in (HI0, HI1) and j >= 15):
        return "pelo", "pelo"
    if k <= HK0 + 1 and j >= 18 and not (i in (-2, -1) and j == 18):
        return "pelo", "pelo"
    return "cabeza", "piel"

def vegetta_extras(vx, dj, sentado):
    # Los picos del pelo: mechones hacia arriba y hacia atras, de distinto alto.
    for n, (i0, alto, k0) in enumerate([(-7, 2, -6), (-4, 3, -7), (-1, 4, -5), (2, 3, -6), (5, 2, -4), (-6, 2, 0), (-2, 3, 1), (3, 2, 2)]):
        for h in range(alto):
            brillo = (n + h) % 3 == 0
            vx.box("pelo_brillo" if brillo else "pelo", "pelo_brillo" if brillo else "pelo", i0 + h // 2, i0 + 2 - h // 2, 22 + dj + h, 22 + dj + h, k0 + h, k0 + 3)
    # El visor morado sobre el ojo derecho (el de la izquierda de quien lo ve), con su brillo.
    vx.box("visor", "visor", 0, 4, 12 + dj, 16 + dj, HK0 - 1, HK0 - 1)
    vx.box("visor_brillo", "visor_brillo", 1, 2, 15 + dj, 15 + dj, HK0 - 1, HK0 - 1)
    vx.box("visor_arco", "dorado", 5, 6, 14 + dj, 16 + dj, HK0 - 1, HK0 + 2)
    t = 4 if sentado else 10
    # El peto blanco (adelante y atras), con su sombra gris en medio.
    vx.box("torso_peto", "blanco", -3, 2, t + 1, t + 4, -3, -3)
    vx.box("torso_peto_sombra", "gris", -1, 0, t + 1, t + 3, -3, -3)
    vx.box("torso_peto", "blanco", -3, 2, t + 2, t + 4, 2, 2)
    # El cinturon dorado con su hebilla grande.
    vx.box("torso_cinturon", "dorado", -3, 2, t, t, -3, 2, si=lambda i, j, k: k in (-3, 2) or i in (-3, 2))
    vx.box("torso_hebilla", "dorado_oscuro", -1, 0, t, t + 1, -4, -4)
    # Las hombreras doradas (se mueven con los brazos).
    a = 3
    vx.box("brazo_{lado}_hombrera", "dorado", a, a + 1, t + 4, t + 4, -2, 1, espejo=True)
    vx.box("brazo_{lado}_hombrera_borde", "dorado_oscuro", a + 1, a + 1, t + 3, t + 3, -2, 1, espejo=True)

personaje("vegetta",
          mat("piel", (0.96, 0.78, 0.64)) + mat("pelo", (0.24, 0.15, 0.10), 0.3, 24) + mat("pelo_brillo", (0.36, 0.24, 0.16), 0.3, 24)
          + mat("morado", (0.55, 0.16, 0.80), 0.25, 20) + mat("morado_oscuro", (0.36, 0.10, 0.55)) + mat("blanco", (0.96, 0.96, 0.98), 0.3, 24)
          + mat("gris", (0.70, 0.70, 0.78)) + mat("dorado", (1.00, 0.70, 0.18), 0.6, 48) + mat("dorado_oscuro", (0.86, 0.48, 0.10), 0.6, 48)
          + mat("visor", (0.62, 0.18, 0.86), 0.7, 64, d=0.85) + mat("visor_brillo", (0.92, 0.70, 1.00), 0.7, 64, ke=(0.25, 0.10, 0.35)),
          dict(torso="morado", manga="morado", falda="morado_oscuro", muslo="morado", bota="blanco", suela="gris", mano="blanco",
               manga_borde="blanco", bota_borde="dorado", cuello="morado"),
          vegetta_pinta, vegetta_extras, forma=lambda i, j, k: True)


# --- Willyrex (su skin de siempre): pelo verde con un mechon amarillo, el fleco rubio, ojos
#     verdes, la chumpa verde con la franja amarilla, pantalon cafe y el pico de diamante ---
def willy_pinta(i, j, k):
    if j >= 19 or k >= 3 or (i in (HI0, HI1) and j >= 14):
        # el mechon amarillo de arriba
        if j >= 20 and i in (-1, 0) and k <= 0:
            return "pelo_mechon", "mechon"
        return "pelo", "pelo"
    # el fleco rubio, con picos que bajan por la frente
    if k <= HK0 + 1 and (j >= 19 or (j == 18 and i in (-7, -6, -4, -3, 0, 1, 3, 4, 6)) or (j == 17 and i in (-6, -3, 1, 4))):
        return "pelo_fleco", "fleco"
    return "cabeza", "piel"

def willy_extras(vx, dj, sentado):
    # Los mechones de los lados (rubios, como en la skin).
    vx.box("pelo_fleco", "fleco", -8, -8, 15 + dj, 19 + dj, -7, -4)
    vx.box("pelo_fleco", "fleco", 7, 7, 15 + dj, 19 + dj, -7, -4)
    t = 4 if sentado else 10
    # La franja amarilla de la chumpa (adelante, del cuello al cinturon) y el cuello abierto.
    vx.box("torso_franja", "amarillo", -1, 0, t, t + 4, -3, -3)
    vx.box("torso_cuello", "verde_oscuro", -3, 2, t + 4, t + 4, -3, 2, si=lambda i, j, k: k in (-3, 2) or i in (-3, 2))
    # El pico de diamante en la espalda: el mango cafe en diagonal y la punta celeste asomando por
    # encima del hombro derecho, junto a la cabeza (como en su skin).
    for n in range(12):
        vx.box("torso_pico", "mango", -2 + n, -2 + n, t - 1 + n, t - 1 + n, 3, 3)
    punta = [(5, t + 13), (6, t + 13), (7, t + 13), (8, t + 12), (9, t + 11), (9, t + 10), (10, t + 10), (10, t + 9), (4, t + 12), (3, t + 12), (11, t + 8)]
    for (i, j) in punta:
        vx.box("torso_pico_punta", "diamante", i, i, j, j, 2, 4)
    vx.box("torso_pico_brillo", "diamante_claro", 7, 8, t + 12, t + 12, 2, 2)

# La cara de Willy: los ojos de anime, pero verdes.
CARAS_WILLY = {emo: [(p, "ojo_verde" if p == "ojos" else m, r) for (p, m, r) in piezas] for emo, piezas in CARAS.items()}

personaje("willyrex",
          mat("piel", (0.94, 0.76, 0.62)) + mat("pelo", (0.30, 0.52, 0.14), 0.3, 24) + mat("mechon", (0.98, 0.86, 0.36), 0.3, 24)
          + mat("fleco", (0.98, 0.76, 0.28), 0.3, 24) + mat("verde", (0.36, 0.60, 0.18), 0.2, 16) + mat("verde_oscuro", (0.22, 0.40, 0.10))
          + mat("amarillo", (0.98, 0.88, 0.36)) + mat("pantalon", (0.42, 0.28, 0.16)) + mat("zapato", (0.24, 0.16, 0.10), 0.4, 24)
          + mat("mango", (0.45, 0.30, 0.16)) + mat("diamante", (0.30, 0.90, 0.85), 0.8, 64, ke=(0.06, 0.22, 0.20))
          + mat("diamante_claro", (0.75, 1.00, 0.98), 0.8, 64, ke=(0.2, 0.4, 0.4)) + mat("ojo_verde", (0.25, 0.65, 0.18), 0.4, 32),
          dict(torso="verde", manga="verde", falda="pantalon", muslo="pantalon", bota="zapato", suela="zapato", cuello="piel",
               manga_borde="verde_oscuro"),
          willy_pinta, willy_extras, forma=lambda i, j, k: True, caras=CARAS_WILLY)


# --- Bill Nye (no se escoge: cuenta los datos del universo): bata azul, corbatin rojo, pelo de lado ---
def bill_pinta(i, j, k):
    if j >= 19 or k >= 3 or (i in (HI0, HI1) and j >= 14):
        return "pelo", "pelo"
    if k <= HK0 + 1 and j >= 18 and i <= 1:
        return "pelo", "pelo"  # la raya de lado
    return "cabeza", "piel"

def bill_extras(vx, dj, sentado):
    t = 4 if sentado else 10
    vx.pattern("torso_corbatin", "corbatin", ["##.##", "#####", "##.##"], -3, t + 5, -3)
    vx.box("torso_camisa", "camisa", -1, 0, t + 1, t + 3, -3, -3)

personaje("bill",
          mat("piel", (0.96, 0.80, 0.68)) + mat("pelo", (0.62, 0.48, 0.32), 0.3, 24) + mat("bata_azul", (0.55, 0.75, 0.95), 0.2, 16)
          + mat("corbatin", (0.85, 0.12, 0.18), 0.3, 24) + mat("camisa", (0.96, 0.96, 0.98)) + mat("pantalon", (0.30, 0.30, 0.36))
          + mat("zapato", (0.12, 0.10, 0.10), 0.5, 32),
          dict(torso="bata_azul", manga="bata_azul", falda="bata_azul", muslo="pantalon", bota="zapato", suela="zapato", corbata="camisa"),
          bill_pinta, bill_extras, falda=True)


# --- El guardia del CIT (no se escoge: hace su ronda por el lab la noche de la entrega y entra
#     con su linterna cuando se va la luz): gorra azul con placa, bigote y silbato ---
def guardia_pinta(i, j, k):
    if j >= 18:
        return None  # arriba va la gorra
    if (k >= 3 or i in (HI0, HI1)) and j >= 14:
        return "pelo", "pelo"
    return "cabeza", "piel"

def guardia_extras(vx, dj, sentado):
    vx.box("gorra", "gorra", -7, 6, 18 + dj, 21 + dj, -7, 6, si=lambda i, j, k: en_cabeza(i, j - dj, k) or j < 21 + dj)
    vx.box("gorra", "gorra", -7, 6, 22 + dj, 22 + dj, -7, 6, si=lambda i, j, k: abs(i + 0.5) < 6 and abs(k + 0.5) < 6)
    vx.box("gorra_visera", "visera", -6, 5, 18 + dj, 18 + dj, -10, -8)
    vx.box("gorra_placa", "placa", -1, 0, 19 + dj, 21 + dj, -8, -8)
    vx.box("bigote", "pelo", -4, 3, 12 + dj, 12 + dj, -8, -8)
    vx.box("bigote", "pelo", -5, 4, 11 + dj, 11 + dj, -8, -8, si=lambda i, j, k: i not in (-1, 0))
    vx.box("orejas", "piel", -8, -8, 13 + dj, 15 + dj, -1, 1)
    vx.box("orejas", "piel", 7, 7, 13 + dj, 15 + dj, -1, 1)
    t = 4 if sentado else 10
    vx.box("torso_placa", "placa", 1, 2, t + 3, t + 4, -4, -4)
    vx.box("torso_silbato", "silbato", -3, -2, t + 2, t + 2, -4, -4)
    vx.box("torso_cordon", "visera", -2, 1, t + 4, t + 5, -4, -4, si=lambda i, j, k: (i + j) % 2 == 0)
    vx.box("torso_cincho", "visera", -3, 2, t, t, -3, 2)

personaje("guardia",
          mat("piel", (0.86, 0.66, 0.50)) + mat("pelo", (0.10, 0.08, 0.07), 0.3, 24) + mat("gorra", (0.12, 0.16, 0.34), 0.25, 20)
          + mat("visera", (0.05, 0.05, 0.07), 0.5, 40) + mat("placa", (0.95, 0.78, 0.30), 0.8, 64, ke=(0.15, 0.12, 0.02))
          + mat("silbato", (0.80, 0.80, 0.85), 0.8, 64) + mat("uniforme", (0.16, 0.22, 0.40), 0.2, 16)
          + mat("pantalon", (0.10, 0.12, 0.20)) + mat("zapato", (0.06, 0.06, 0.06), 0.6, 48),
          dict(torso="uniforme", manga="uniforme", falda="pantalon", muslo="pantalon", bota="zapato", suela="zapato", corbata="uniforme", cuello="uniforme"),
          guardia_pinta, guardia_extras, w=1)


# --- Rocky (Project Hail Mary): el ingeniero eridiano. Un caparazon de roca en forma de
#     pentagono, sin ojos (oye con todo el cuerpo), un respiradero arriba y cinco patas con su
#     rodilla. Las dos de adelante son "brazos" (manejan y saludan en el baile), dos son piernas y
#     la quinta va con el cuerpo. No tiene cara: habla con acordes (GLADAS traduce). ---
def rocky_pentagono(x, z, r):
    a = math.atan2(z, x)
    sector = 2 * math.pi / 5
    b = (a % sector) - sector / 2
    return math.hypot(x, z) * math.cos(b) <= r * math.cos(sector / 2)

def rocky_pata(vx, puntos, alto, bajo, corte=0.62):
    """Una pata por una linea quebrada (en voxels), de dos voxels de grueso: la parte de arriba con
    el nombre/material `alto` y la punta con `bajo`."""
    tramos = len(puntos) - 1
    for s in range(tramos):
        a, b = puntos[s], puntos[s + 1]
        pasos = int(max(abs(b[q] - a[q]) for q in range(3)) * 2) + 1
        for t in range(pasos + 1):
            u = t / pasos
            p = [a[q] + (b[q] - a[q]) * u for q in range(3)]
            nombre, mat_ = bajo if (s + u) / tramos > corte else alto
            for dy in (0, 1):
                vx.put(math.floor(p[0]), math.floor(p[1]) + dy, math.floor(p[2]), nombre, mat_)

def rocky_cuerpo(vx, sentado):
    y0 = 1 if sentado else 9
    if sentado:
        silla(vx)
    # El caparazon: siete capas de pentagono que se van cerrando, con vetas oscuras.
    for n in range(7):
        r = 6.6 - n * 0.75
        for i in range(-8, 8):
            for k in range(-8, 8):
                if rocky_pentagono(i + 0.5, k + 0.5, r):
                    veta = ((i // 2) * 7 + (k // 2) * 13 + n * 5) % 9 == 0
                    vx.put(i, y0 + n, k, "torso_veta" if veta else "torso", "roca_oscura" if veta else "roca")
    vx.box("torso_respiradero", "respiradero", -1, 0, y0 + 7, y0 + 7, -1, 0)
    sale = lambda a, r, y: (r * math.cos(a), y, r * math.sin(a))
    frente, atras = -math.pi / 2, math.pi / 2
    if sentado:
        # Las de adelante agarran el volante; las de atras cuelgan hacia el frente; la quinta, abajo.
        for lado, x in (("der", 1), ("izq", -1)):
            rocky_pata(vx, [(x * 3.5, y0 + 2, -4.5), (x * 4.5, y0 + 6, -6.5), (x * 2.2, y0 + 5, -8.5)], (f"brazo_{lado}", "pata"), (f"mano_{lado}", "mano"))
            rocky_pata(vx, [(x * 4.0, y0 + 1, -3.0), (x * 4.0, y0 - 2, -6.0), (x * 3.0, -5.0, -8.5)], (f"muslo_{lado}", "pata"), (f"bota_{lado}", "mano"))
        rocky_pata(vx, [(0.0, y0 + 1, -5.0), (0.0, -2.0, -7.0), (0.0, -5.0, -9.0)], ("torso_pata", "pata"), ("torso_pata_punta", "mano"))
    else:
        # De pie, sobre sus cinco patas: suben a la rodilla y bajan al piso.
        for lado, signo in (("der", 1), ("izq", -1)):
            a = frente + signo * 0.65
            rocky_pata(vx, [sale(a, 5.0, y0 + 2), sale(a, 8.5, y0 + 6), sale(a, 11.0, 0.0)], (f"brazo_{lado}", "pata"), (f"mano_{lado}", "mano"))
            a = atras - signo * 0.95
            rocky_pata(vx, [sale(a, 5.0, y0 + 2), sale(a, 8.5, y0 + 6), sale(a, 11.0, 0.0)], (f"muslo_{lado}", "pata"), (f"bota_{lado}", "mano"))
        rocky_pata(vx, [sale(atras, 5.0, y0 + 2), sale(atras, 8.5, y0 + 6), sale(atras, 11.0, 0.0)], ("torso_pata", "pata"), ("torso_pata_punta", "mano"))

personaje("rocky",
          mat("roca", (0.50, 0.44, 0.40), 0.15, 12) + mat("roca_oscura", (0.34, 0.30, 0.28)) + mat("respiradero", (0.15, 0.12, 0.12))
          + mat("pata", (0.42, 0.37, 0.34), 0.15, 12) + mat("mano", (0.30, 0.27, 0.25)),
          {}, None, cuerpo=rocky_cuerpo, caras=cara_fija([]), cara=(-7, 4.0, 12.0), hombro=(4.0, 11.0), cadera=(3.5, 11.0))


# ======================== Pilotos de memes (se compran en la tienda) ========================

# --- La capibara: cafe, hocico ancho, orejitas y su mandarina en la cabeza. Siempre en paz. ---
CARA_CAPIBARA = cara_fija([
    ("ojos", "ojo", espejo(rects(["####"], 2, 8))),
    ("nariz", "capi_nariz", espejo(rects(["##"], 1, 3))),
])

def capibara_pinta(i, j, k):
    return "cabeza", "piel"

def capibara_extras(vx, dj, sentado):
    vx.box("hocico", "capi_oscuro", -5, 4, 10 + dj, 14 + dj, -9, -8)
    vx.box("hocico", "capi_oscuro", -4, 3, 10 + dj, 13 + dj, -10, -10)
    for i0 in (-7, 5):
        vx.box("orejas", "capi_oscuro", i0, i0 + 1, 21 + dj, 22 + dj, -1, 0)
    # La mandarina con su hojita.
    vx.box("mandarina", "mandarina", -2, 1, 22 + dj, 24 + dj, -2, 1, si=lambda i, j, k: not (i in (-2, 1) and k in (-2, 1) and j in (22 + dj, 24 + dj)))
    vx.box("hojita", "hojita", 0, 1, 25 + dj, 25 + dj, -1, -1)

personaje("capibara",
          mat("piel", (0.62, 0.44, 0.28)) + mat("capi_oscuro", (0.45, 0.30, 0.18)) + mat("capi_nariz", (0.18, 0.12, 0.08))
          + mat("mandarina", (1.0, 0.58, 0.10), 0.3, 24) + mat("hojita", (0.25, 0.70, 0.25)) + mat("toalla", (0.95, 0.80, 0.45)),
          dict(torso="piel", manga="piel", falda="toalla", muslo="piel", bota="capi_oscuro", suela="capi_oscuro", cuello="piel"),
          capibara_pinta, capibara_extras, caras=CARA_CAPIBARA, forma=lambda i, j, k: True)


# --- Moo Deng: la hipopotamo bebe, gris y brillosa, cachetes rosas y la boca abierta. ---
CARA_MOODENG = cara_fija([
    ("ojos", "ojo", espejo(rects(["###", "###"], 3, 11))),
    ("boca", "moo_boca", rects(["..######..", ".########.", "##########", ".########."], -5, 6)),
    ("rubor", "rubor_moo", espejo(rects(["###"], 6, 6))),
])

def moodeng_pinta(i, j, k):
    return "cabeza", "piel"

def moodeng_extras(vx, dj, sentado):
    vx.box("hocico", "piel", -5, 4, 10 + dj, 14 + dj, -9, -8)
    vx.box("nariz", "moo_boca", -3, -2, 14 + dj, 14 + dj, -10, -10)
    vx.box("nariz", "moo_boca", 1, 2, 14 + dj, 14 + dj, -10, -10)
    for i0 in (-6, 4):
        vx.box("orejas", "rubor_moo", i0, i0 + 1, 22 + dj, 22 + dj, -1, 0)
    t = 4 if sentado else 10
    vx.box("torso_panza", "rubor_moo", -2, 1, t, t + 3, -3, -3)

personaje("moodeng",
          mat("piel", (0.42, 0.40, 0.46), 0.6, 48) + mat("moo_boca", (0.55, 0.15, 0.22)) + mat("rubor_moo", (1.0, 0.55, 0.62)),
          dict(torso="piel", manga="piel", falda="piel", muslo="piel", bota="piel", suela="moo_boca", cuello="piel"),
          moodeng_pinta, moodeng_extras, caras=CARA_MOODENG, w=1)


# --- Pedro, el mapache que baila: gris, antifaz negro, hocico blanco y la cola a rayas. ---
def pedro_pinta(i, j, k):
    if k <= HK0 + 1 and j in (14, 15, 16) and -6 <= i <= 5:
        return "antifaz", "antifaz"
    if k <= HK0 + 1 and j <= 12 and -3 <= i <= 2:
        return "hocico", "blanco_mapache"
    return "cabeza", "piel"

def pedro_extras(vx, dj, sentado):
    vx.box("nariz", "antifaz", -1, 0, 12 + dj, 12 + dj, -8, -8)
    for i0 in (-6, 3):
        vx.box("orejas", "piel", i0, i0 + 2, 22 + dj, 23 + dj, -1, 0)
        vx.box("orejas_dentro", "blanco_mapache", i0 + 1, i0 + 1, 22 + dj, 22 + dj, -2, -2)
    t = 4 if sentado else 10
    vx.box("torso_panza", "blanco_mapache", -2, 1, t, t + 3, -3, -3)
    for n in range(7):
        m_ = "antifaz" if n % 2 == 0 else "piel"
        vx.box("torso_cola" + ("" if m_ == "piel" else "_raya"), m_, -1, 0, t + n // 2, t + n // 2 + 1, 2 + n, 2 + n)

CARA_PEDRO = {emo: [(p_, "ojo_mapache" if p_ == "ojos" else m_, r) for (p_, m_, r) in piezas] for emo, piezas in CARAS.items()}

personaje("pedro",
          mat("piel", (0.58, 0.58, 0.62)) + mat("antifaz", (0.12, 0.12, 0.14)) + mat("blanco_mapache", (0.95, 0.95, 0.95))
          + mat("ojo_mapache", (0.95, 0.95, 1.0), 0.6, 48),
          dict(torso="piel", manga="piel", mano="antifaz", falda="piel", muslo="piel", bota="antifaz", suela="antifaz", cuello="piel"),
          pedro_pinta, pedro_extras, caras=CARA_PEDRO)


# --- Chill Guy: el perrito tranquilo, sueter gris, jeans arremangados y tenis rojos. ---
CARA_CHILL = cara_fija([
    ("ojos", "ojo", espejo(rects(["####", ".##."], 2, 10))),
    ("sonrisa", "ojo", rects(["......##", "######.."], -4, 5)),
])

def chill_pinta(i, j, k):
    if k <= HK0 + 1 and j <= 12 and -4 <= i <= 3:
        return "hocico", "chill_claro"
    return "cabeza", "piel"

def chill_extras(vx, dj, sentado):
    vx.box("nariz", "ojo", -1, 0, 12 + dj, 12 + dj, -8, -8)
    # Las orejas largas y caidas a los lados.
    for i0 in (-8, 7):
        vx.box("orejas", "chill_oreja", i0, i0, 13 + dj, 20 + dj, -3, 1)
    t = 4 if sentado else 10
    vx.box("torso_cuello", "sueter_oscuro", -3, 2, t + 4, t + 4, -2, 1)

personaje("chillguy",
          mat("piel", (0.90, 0.84, 0.76)) + mat("chill_claro", (0.98, 0.95, 0.90)) + mat("chill_oreja", (0.72, 0.60, 0.50))
          + mat("sueter", (0.55, 0.57, 0.62)) + mat("sueter_oscuro", (0.40, 0.42, 0.47)) + mat("jeans", (0.25, 0.40, 0.72))
          + mat("tenis_rojos", (0.90, 0.15, 0.15), 0.3, 24) + mat("suela_blanca", (0.97, 0.97, 0.97)),
          dict(torso="sueter", manga="sueter", mano="piel", falda="jeans", muslo="jeans", bota="tenis_rojos", suela="suela_blanca",
               cuello="sueter", manga_borde="sueter_oscuro", bota_borde="jeans"),
          chill_pinta, chill_extras, caras=CARA_CHILL)


# --- Labubu: el monstruito de peluche, orejas de conejo, carita crema y la sonrisa de nueve dientes. ---
CARA_LABUBU = cara_fija([
    ("ojos", "ojo", espejo(rects([".##.", "####", "####", ".##."], 2, 12))),
    ("brillo", "steve_blanco", espejo(rects(["#"], 3, 11))),
    ("boca", "labubu_boca", rects(["##########", ".########."], -5, 6)),
    ("dientes", "steve_blanco", rects(["#.#.#.#.#.", "#.#.#.#.#."], -5, 6)),
])

def labubu_pinta(i, j, k):
    # La carita crema, ovalada, al frente; lo demas es peluche cafe.
    if k <= HK0 + 1 and ((i + 0.5) / 6.5) ** 2 + ((j - 14.5) / 5.0) ** 2 <= 1.0:
        return "cabeza", "piel"
    return "pelo", "peluche"

def labubu_extras(vx, dj, sentado):
    for i0 in (-6, 3):
        vx.box("orejas", "peluche", i0, i0 + 2, 22 + dj, 28 + dj, -1, 0)
        vx.box("orejas_dentro", "rosa_labubu", i0 + 1, i0 + 1, 23 + dj, 27 + dj, -2, -2)
    t = 4 if sentado else 10
    vx.box("torso_panza", "piel", -2, 1, t, t + 3, -3, -3)

personaje("labubu",
          mat("piel", (0.98, 0.90, 0.80)) + mat("peluche", (0.62, 0.42, 0.28)) + mat("rosa_labubu", (1.0, 0.62, 0.70))
          + mat("labubu_boca", (0.25, 0.08, 0.10)) + mat("steve_blanco", (1.0, 1.0, 1.0)),
          dict(torso="peluche", manga="peluche", mano="peluche", falda="peluche", muslo="peluche", bota="peluche", suela="rosa_labubu", cuello="peluche"),
          labubu_pinta, labubu_extras, caras=CARA_LABUBU, forma=lambda i, j, k: True)


# --- Tung Tung Tung Sahur: la cabeza es un tronco (con anillos arriba), ojos grandes y su bate. ---
CARA_TUNG = cara_fija([
    ("ojos", "steve_blanco", espejo(rects(["####", "####", "####", "####"], 1, 12))),
    ("pupilas", "ojo", espejo(rects(["##", "##"], 2, 11))),
    ("cejas", "ojo", espejo(rects(["####"], 1, 14))),
    ("boca", "ojo", rects(["########"], -4, 4)),
])

def tung_pinta(i, j, k):
    if j == HJ1:
        return ("anillos", "anillos") if (abs(i + 0.5) + abs(k + 0.5)) % 4 < 2 else ("anillos_claro", "madera_clara")
    if (i * 3 + j * 7 + k * 5) % 11 == 0:
        return "cabeza_veta", "madera_oscura"
    return "cabeza", "piel"

def tung_extras(vx, dj, sentado):
    # El bate, en la mano derecha (+X): parado junto al cuerpo.
    if sentado:
        for n in range(12):
            vx.box("bate", "bate", 6, 6, 4 + n, 4 + n, -2 - n // 3, -2 - n // 3)
    else:
        for n in range(14):
            vx.box("bate", "bate", 5, 5, 7 + n, 7 + n, -2 + n // 5, -2 + n // 5)

personaje("tungtung",
          mat("piel", (0.72, 0.50, 0.30)) + mat("madera_oscura", (0.48, 0.32, 0.18)) + mat("madera_clara", (0.88, 0.70, 0.45))
          + mat("anillos", (0.80, 0.60, 0.36)) + mat("bate", (0.85, 0.70, 0.45), 0.3, 24) + mat("steve_blanco", (1.0, 1.0, 1.0)),
          dict(torso="piel", manga="piel", mano="madera_oscura", falda="madera_oscura", muslo="piel", bota="madera_oscura", suela="madera_oscura", cuello="piel"),
          tung_pinta, tung_extras, caras=CARA_TUNG, forma=lambda i, j, k: True)


# --- Chimpanzini Bananini (premio del nivel 40 del pase): un changuito adentro de una banana
#     pelada, con los petalos de cascara alrededor. ---
def bananini_pinta(i, j, k):
    if k <= HK0 + 1 and j <= 16 and -5 <= i <= 4:
        return "cabeza", "piel"
    return "pelo", "mono"

def bananini_extras(vx, dj, sentado):
    for i0 in (-8, 7):
        vx.box("orejas", "piel", i0, i0, 14 + dj, 16 + dj, -1, 0)
    # Los petalos de la cascara: suben desde la cintura alrededor del cuerpo, con la puntita verde.
    t = 4 if sentado else 10
    for (i0, k0) in ((-5, -4), (4, -4), (-5, 3), (4, 3), (-1, -5), (0, 4)):
        alto = 9
        vx.box("torso_cascara", "banana", i0, i0, t - 2, t + alto, k0, k0)
        vx.box("torso_cascara_punta", "banana_verde", i0, i0, t + alto + 1, t + alto + 1, k0, k0)
    vx.box("torso_cascara", "banana", -5, 4, t - 2, t - 1, -4, 3, si=lambda i, j, k: i in (-5, 4) or k in (-4, 3))

personaje("bananini",
          mat("piel", (0.95, 0.80, 0.62)) + mat("mono", (0.45, 0.28, 0.15)) + mat("banana", (1.0, 0.86, 0.20), 0.3, 24)
          + mat("banana_verde", (0.45, 0.70, 0.20)),
          dict(torso="mono", manga="mono", mano="piel", falda="banana", muslo="mono", bota="mono", suela="piel", cuello="mono"),
          bananini_pinta, bananini_extras, forma=lambda i, j, k: True)


# ---------- escribir ----------
PIERNAS_Y_BRAZOS = {"brazo_der", "brazo_izq", "manga_der", "manga_izq", "manga_der_borde", "manga_izq_borde", "mano_der", "mano_izq",
                    "muslo_der", "muslo_izq", "bota_der", "bota_izq", "bota_der_borde", "bota_izq_borde", "suela_der", "suela_izq"}
COLETAS = {"coleta_der", "coleta_izq", "coleta_der_puntas", "coleta_izq_puntas"}

for p in PERSONAJES:
    mtl = f"# Materiales de {p['id']} (voxel)\n\n" + p["mtl"] + SILLA + MTL_CARA
    # Sentado manejando.
    vx = Voxels(U)
    if p["cuerpo"]:
        p["cuerpo"](vx, True)
    else:
        cuerpo_sentado(vx, p["c"], p["falda"], p["w"])
        cabeza(vx, 0, p["pintar"], p["forma"])
        if p["extras"]:
            p["extras"](vx, 0, True)
    z_cara, y_sentado, y_de_pie = p["cara"] or (HK0, HJ0 + 1, HJ0 + DJ + 1)
    m = Modelo()
    comentarios = {}
    if p["coletas"]:
        piv = vx.centro_de("liga_der")
        comentarios = {"coleta_der": f"pivote: {piv[0]:.3f} {piv[1]:.3f} {piv[2]:.3f}",
                       "coleta_izq": f"pivote: {-piv[0]:.3f} {piv[1]:.3f} {piv[2]:.3f}"}
    vol = vx.centro_de("volante")
    comentarios["volante"] = f"gira alrededor del eje Z que pasa por {vol[0]:.3f} {vol[1]:.3f} {vol[2]:.3f}"
    vx.mesh(m, COLETAS | {"volante"}, comentarios)
    expresiones(m, z_frente=z_cara*U, px=U/2, x_centro=0.0, y_base=y_sentado*U, caras=p["caras"])
    m.write(f"personaje_{p['id']}_piloto", f"""# {p['id']} piloto (chibi voxel, voxel de 0.04): el mismo esqueleto que miku_piloto.obj.
# Y arriba, mira hacia -Z, origen en el centro del asiento. izq/der desde su punto de vista: izq = -X.
# Expresiones: partes cara_<emocion>_*. Animables: volante y coleta_* (si tiene).
""", mtl)

    # De pie, para bailar.
    vb = Voxels(U)
    if p["cuerpo"]:
        p["cuerpo"](vb, False)
    else:
        cuerpo_de_pie(vb, p["c"], p["falda"], p["w"])
        cabeza(vb, DJ, p["pintar"], p["forma"])
        if p["extras"]:
            p["extras"](vb, DJ, False)
    mb = Modelo()
    animadas = {n for (n, _) in vb.v.values() if n.startswith(("brazo_", "manga_", "mano_", "muslo_", "bota_", "suela_"))}
    vb.mesh(mb, PIERNAS_Y_BRAZOS | COLETAS | animadas)
    expresiones(mb, z_frente=z_cara*U, px=U/2, x_centro=0.0, y_base=y_de_pie*U, caras=p["caras"])
    a, ya = p["hombro"] or (3.5 + p["w"], 14.5)
    xc, yc = p["cadera"] or (1.0, 8.0)
    coletas = ""
    if p["coletas"]:
        liga = vb.centro_de("liga_der")
        coletas = (f"# pivote coleta_der: {liga[0]:.3f} {liga[1]:.3f} {liga[2]:.3f}\n"
                   f"# pivote coleta_izq: {-liga[0]:.3f} {liga[1]:.3f} {liga[2]:.3f}\n")
    mb.write(f"personaje_{p['id']}_baile", f"""# {p['id']} de pie para bailar (chibi voxel, voxel de 0.04): el mismo esqueleto que miku_baile.obj.
# Y arriba, mira hacia -Z, origen en el piso. izq/der desde su punto de vista: izq = -X.
# Grupos (por nombre): brazo_/manga_/mano_ = brazo; muslo_/bota_/suela_ = pierna; torso*/corbata/falda*/cuello = cuerpo;
#   coleta_* = coleta; lo demas = cabeza.
# pivote hombro_der: {a*U:.3f} {ya*U:.3f} 0.000
# pivote hombro_izq: {-a*U:.3f} {ya*U:.3f} 0.000
# pivote cadera_der: {xc*U:.3f} {yc*U:.3f} 0.000
# pivote cadera_izq: {-xc*U:.3f} {yc*U:.3f} 0.000
# pivote cuello: 0.000 {15.5*U:.3f} 0.000
{coletas}""", mtl)
