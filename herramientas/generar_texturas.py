# Genera las TEXTURAS del proyecto (PNG en modelos/texturas/) y los modelos con coordenadas UV
# que las usan, para el muestreo de texturas del fragment shader:
#   texturas/pizarra.png:  lo escrito con tiza en el pizarron del salon de la UVG (el automata).
#   texturas/dimension.png: el cielo de la "dimension de bolsillo" del Planeta Hada (equirectangular).
#   portal.obj:    el portal de la cueva del Planeta Hada: la mascara (para el stencil) y su borde.
#   dimension.obj: una esfera con UV y las caras hacia adentro: el cielo que se ve por el portal.
# El rectangulo con UV del pizarron lo pone generar_aula.py.
# Uso: python3 herramientas/generar_texturas.py   (necesita Pillow)

import math, os, random
from PIL import Image, ImageDraw, ImageFont
from modelo3d import *

AQUI = os.path.dirname(os.path.abspath(__file__))
TEXTURAS = os.path.join(MODELOS, "texturas")
FUENTE = os.path.join(AQUI, "..", "assets", "fuentes", "PixelifySans.ttf")
os.makedirs(TEXTURAS, exist_ok=True)
random.seed(7)

def fuente(tam):
    return ImageFont.truetype(FUENTE, tam)

# ---------- el pizarron: tiza sobre verde ----------
W, H = 1024, 288
img = Image.new("RGB", (W, H), (34, 62, 48))
d = ImageDraw.Draw(img)
# Manchas de borrador y polvo de tiza
for _ in range(90):
    x, y, r = random.randrange(W), random.randrange(H), random.randrange(10, 60)
    c = 34 + random.randrange(-4, 10)
    d.ellipse([x - r, y - r // 2, x + r, y + r // 2], fill=(c, 62 + random.randrange(-3, 10), 48 + random.randrange(-3, 8)))
TIZA, AZUL, ROJA, VERDE = (236, 238, 230), (150, 205, 255), (255, 140, 150), (170, 240, 160)

def centrado(texto, x, y, tam, color):
    f = fuente(tam)
    w = d.textlength(texto, font=f)
    d.text((x - w / 2, y), texto, font=f, fill=color)

centrado("L = (a|b)* abb", W / 2, 10, 40, ROJA)
ESTADOS = [190, 410, 630, 850]
Y, R = 150, 42
for n, x in enumerate(ESTADOS):
    d.ellipse([x - R, Y - R, x + R, Y + R], outline=AZUL, width=4)
    centrado(f"q{n}", x, Y - 18, 30, TIZA)
d.ellipse([ESTADOS[3] - R + 8, Y - R + 8, ESTADOS[3] + R - 8, Y + R - 8], outline=AZUL, width=3)  # de aceptacion

def flecha(x0, y0, x1, y1, color):
    d.line([x0, y0, x1, y1], fill=color, width=4)
    a = math.atan2(y1 - y0, x1 - x0)
    for s in (-1, 1):
        d.line([x1, y1, x1 - 16 * math.cos(a + s * 0.5), y1 - 16 * math.sin(a + s * 0.5)], fill=color, width=4)

flecha(60, Y, ESTADOS[0] - R - 4, Y, TIZA)
centrado("inicio", 80, Y + 10, 20, TIZA)
for n, letra in enumerate("abb"):
    flecha(ESTADOS[n] + R + 4, Y, ESTADOS[n + 1] - R - 4, Y, TIZA)
    centrado(letra, (ESTADOS[n] + ESTADOS[n + 1]) / 2, Y - 46, 30, ROJA)
# Lazos y regresos
for n, letra in ((0, "b"), (1, "a")):
    x = ESTADOS[n]
    d.arc([x - 26, Y - R - 50, x + 26, Y - R + 6], 200, 340, fill=TIZA, width=4)
    centrado(letra, x, Y - R - 86, 28, ROJA)
d.arc([ESTADOS[0], Y + 10, ESTADOS[2], Y + 120], 15, 165, fill=TIZA, width=3)
centrado("b", (ESTADOS[0] + ESTADOS[2]) / 2, Y + 96, 24, ROJA)
centrado("ACEPTA CADENAS QUE TERMINAN EN abb", W / 2, 246, 26, VERDE)
# Garabatos de estudiantes: una estrellita y "EXAMEN HOY"
d.text((870, 12), "EXAMEN HOY", font=fuente(22), fill=TIZA)
for k in range(5):
    a0, a1 = k * 2 * math.pi * 2 / 5 - math.pi / 2, (k + 1) * 2 * math.pi * 2 / 5 - math.pi / 2
    d.line([40 + 22 * math.cos(a0), 40 + 22 * math.sin(a0), 40 + 22 * math.cos(a1), 40 + 22 * math.sin(a1)], fill=(255, 230, 120), width=3)
img.save(os.path.join(TEXTURAS, "pizarra.png"))

# ---------- el cielo de la dimension de bolsillo (equirectangular) ----------
W, H = 512, 256
img = Image.new("RGB", (W, H))
px = img.load()
for y in range(H):
    for x in range(W):
        u, v = x / W, y / H
        a = u * 2 * math.pi
        remolino = math.sin(a * 3 + v * 12 + math.sin(a * 2) * 2) * 0.5 + 0.5
        r = int(60 + 150 * remolino * (1 - v * 0.5))
        g = int(20 + 60 * (1 - remolino) + 80 * v)
        b = int(110 + 120 * (1 - remolino * 0.5))
        px[x, y] = (min(r, 255), min(g, 255), min(b, 255))
d = ImageDraw.Draw(img)
for _ in range(260):
    x, y = random.randrange(W), random.randrange(H)
    c = random.choice([(255, 255, 255), (255, 220, 250), (200, 250, 255)])
    d.point((x, y), fill=c)
    if random.random() < 0.15:
        d.point([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)], fill=c)
for x in (256,):  # u = 0.5: la pared de enfrente del portal
    texto = "PROYECTO 2"
    f = fuente(36)
    w = d.textlength(texto, font=f)
    d.text((x - w / 2 + 2, 102), texto, font=f, fill=(60, 0, 60))
    d.text((x - w / 2, 100), texto, font=f, fill=(255, 240, 120))
    nota = "calificación: pendiente"
    f = fuente(16)
    d.text((x - d.textlength(nota, font=f) / 2, 146), nota, font=f, fill=(230, 220, 255))
img.save(os.path.join(TEXTURAS, "dimension.png"))

# ---------- el portal: un casquete de esfera sobre la cueva (la cueva mira a -Z) ----------
def en_esfera(radio, lon, lat):
    """Punto de una esfera de centro el origen; lon 0, lat 0 es (0, 0, -radio)."""
    return (radio * math.cos(lat) * math.sin(lon), radio * math.sin(lat), -radio * math.cos(lat) * math.cos(lon))

m = Modelo()
mascara = m.part("portal_mascara", "portal_mascara")
borde = m.part("portal_borde", "portal_borde")
ABERTURA = 0.27  # radio angular del portal
N = 28
for k in range(N):
    a0, a1 = 2 * math.pi * k / N, 2 * math.pi * (k + 1) / N
    # Un poco afuera de la superficie: los voxels de la cueva sobresalen hasta ~1.05.
    for (parte, r0, r1, rad) in ((mascara, 0.0, ABERTURA, 1.065), (borde, ABERTURA, ABERTURA + 0.045, 1.07)):
        pts = [en_esfera(rad, r * math.cos(a), r * math.sin(a)) for (r, a) in ((r0, a0), (r1, a0), (r1, a1), (r0, a1))]
        if r0 == 0.0:
            ids = [parte.vert(p) for p in pts[:3]]
            parte.tri(ids[0], ids[1], ids[2], (0, 0, 0))
        else:
            parte.quad(pts, (0, 0, 0))
m.write("portal", """# El portal de la cueva del Planeta Hada (en coordenadas del planeta: radio 1, la cueva mira a -Z).
# portal_mascara: el casquete que se dibuja solo en el STENCIL (marca la ventana); portal_borde: el aro que brilla.
""", """# Materiales

newmtl portal_mascara
Kd 0 0 0

newmtl portal_borde
Kd 0.9 0.6 1.0
Ke 0.9 0.5 1.0
""")

# ---------- el cielo de adentro: esfera con UV y las caras hacia adentro ----------
m = Modelo()
cielo = m.part("dimension_cielo", "dimension_cielo")
RADIO, CENTRO = 0.85, (0.0, 0.0, 0.1)
NL, NA = 32, 16
def punto(i, j):
    lon = 2 * math.pi * i / NL
    lat = math.pi * (j / NA - 0.5)
    return (CENTRO[0] + RADIO * math.cos(lat) * math.sin(lon), CENTRO[1] + RADIO * math.sin(lat), CENTRO[2] - RADIO * math.cos(lat) * math.cos(lon))
for i in range(NL):
    for j in range(NA):
        pts = [punto(i, j), punto(i + 1, j), punto(i + 1, j + 1), punto(i, j + 1)]
        uvs = [(i / NL, j / NA), ((i + 1) / NL, j / NA), ((i + 1) / NL, (j + 1) / NA), (i / NL, (j + 1) / NA)]
        # "inside" afuera de la esfera: las caras miran hacia el centro.
        cx = sum(p[0] for p in pts) / 4; cy = sum(p[1] for p in pts) / 4; cz = sum(p[2] for p in pts) / 4
        afuera = (CENTRO[0] + (cx - CENTRO[0]) * 2, CENTRO[1] + (cy - CENTRO[1]) * 2, CENTRO[2] + (cz - CENTRO[2]) * 2)
        cielo.quad(pts, afuera, uvs)
m.write("dimension", """# La dimension de bolsillo del Planeta Hada: esfera con coordenadas UV (latitud y longitud) y las caras
# hacia adentro, con la textura texturas/dimension.png. Solo se ve a traves del portal (stencil test).
""", """# Materiales

newmtl dimension_cielo
Kd 1.0 1.0 1.0
Ke 0.8 0.8 0.8
map_Kd texturas/dimension.png
""")
