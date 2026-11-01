# El arte de las cartas del tarot de Madame Mercurio (lo usa generar_mercurio.py).
# Cada carta se pinta 4 veces mas grande y se reduce con Lanczos: asi las lineas y las curvas
# quedan suaves. Todas siguen el mismo diseno: marco crema con doble filo dorado y adornos en las
# esquinas, el numero romano en una plaquita arriba, un fondo con degradado, rayos y estrellitas
# (cada carta con sus colores), la ilustracion con sombras y brillos, y el nombre en un liston.

import math, random
from PIL import Image, ImageDraw, ImageFilter, ImageFont

S = 4  # supermuestreo
TINTA = (40, 22, 60)
ORO = (214, 168, 70)
ORO_CLARO = (250, 220, 130)
CREMA = (250, 238, 210)
MORADO = (70, 35, 110)


def mezcla(a, b, k):
    return tuple(int(a[i] + (b[i] - a[i]) * k) for i in range(len(a)))


class Pincel:
    """Dibuja en coordenadas de la carta final (floats) sobre un lienzo S veces mas grande."""

    def __init__(self, img, fuente):
        self.img = img
        self.d = ImageDraw.Draw(img, "RGBA")
        self.fuente_ruta = fuente

    def f(self, tam):
        return ImageFont.truetype(self.fuente_ruta, int(tam * S))

    def _p(self, pts):
        return [(x * S, y * S) for x, y in pts]

    def elipse(self, cx, cy, rx, ry, fill=None, outline=TINTA, w=1.6):
        self.d.ellipse([(cx - rx) * S, (cy - ry) * S, (cx + rx) * S, (cy + ry) * S], fill=fill, outline=outline, width=int(w * S) if outline else 0)

    def circulo(self, cx, cy, r, **k):
        self.elipse(cx, cy, r, r, **k)

    def rect(self, x0, y0, x1, y1, fill=None, outline=TINTA, w=1.6, r=0):
        caja = [x0 * S, y0 * S, x1 * S, y1 * S]
        if r:
            self.d.rounded_rectangle(caja, r * S, fill=fill, outline=outline, width=int(w * S) if outline else 0)
        else:
            self.d.rectangle(caja, fill=fill, outline=outline, width=int(w * S) if outline else 0)

    def poly(self, pts, fill=None, outline=TINTA, w=1.6):
        self.d.polygon(self._p(pts), fill=fill)
        if outline:
            q = self._p(pts)
            self.d.line(q + [q[0]], fill=outline, width=int(w * S), joint="curve")

    def linea(self, pts, fill=TINTA, w=1.6):
        self.d.line(self._p(pts), fill=fill, width=int(w * S), joint="curve")

    def arco(self, cx, cy, r, a0, a1, fill=TINTA, w=1.6, ry=None):
        ry = ry if ry is not None else r
        self.d.arc([(cx - r) * S, (cy - ry) * S, (cx + r) * S, (cy + ry) * S], a0, a1, fill=fill, width=int(w * S))

    def cuerda(self, cx, cy, r, a0, a1, fill, ry=None):
        ry = ry if ry is not None else r
        self.d.chord([(cx - r) * S, (cy - ry) * S, (cx + r) * S, (cy + ry) * S], a0, a1, fill=fill)

    def texto(self, x, y, s, tam, fill=TINTA, centro=True, sombra=None):
        f = self.f(tam)
        w = self.d.textlength(s, font=f) / S
        x0 = x - w / 2 if centro else x
        if sombra:
            self.d.text(((x0 + 0.8) * S, (y + 0.8) * S), s, font=f, fill=sombra)
        self.d.text((x0 * S, y * S), s, font=f, fill=fill)

    def medir(self, s, tam):
        return self.d.textlength(s, font=self.f(tam)) / S

    def esfera(self, cx, cy, r, base, luz=None, outline=TINTA, w=1.6, ry=None):
        """Un circulo (o elipse) con volumen: degradado hacia un brillo arriba a la izquierda."""
        ry = ry if ry is not None else r
        luz = luz or mezcla(base, (255, 255, 255), 0.55)
        sombra = mezcla(base, (0, 0, 0), 0.25)
        self.elipse(cx, cy, r, ry, fill=sombra, outline=None)
        for k in range(14):
            u = k / 13
            rr, rry = r * (1 - 0.75 * u), ry * (1 - 0.75 * u)
            ox, oy = -r * 0.28 * u, -ry * 0.28 * u
            self.elipse(cx + ox, cy + oy, rr, rry, fill=mezcla(base, luz, u ** 1.6) if k else base, outline=None)
        if outline:
            self.elipse(cx, cy, r, ry, outline=outline, w=w)

    def creciente(self, cx, cy, r, dx, dy, base, luz=None):
        """Una luna creciente: la esfera menos un circulo corrido (dx, dy), recortada de verdad."""
        capa = Image.new("RGB", self.img.size, (0, 0, 0))
        mascara = Image.new("L", self.img.size, 0)
        otra = Pincel(capa, self.fuente_ruta)
        otra.esfera(cx, cy, r, base, luz, outline=None)
        dm = ImageDraw.Draw(mascara)
        dm.ellipse([(cx - r) * S, (cy - r) * S, (cx + r) * S, (cy + r) * S], fill=255)
        dm.ellipse([(cx + dx - r * 0.9) * S, (cy + dy - r * 0.9) * S, (cx + dx + r * 0.9) * S, (cy + dy + r * 0.9) * S], fill=0)
        self.img.paste(capa, (0, 0), mascara)

    def brillo(self, cx, cy, r, color=(255, 255, 255)):
        """Un destello de cuatro puntas."""
        self.poly([(cx, cy - r), (cx + r * 0.22, cy - r * 0.22), (cx + r, cy), (cx + r * 0.22, cy + r * 0.22), (cx, cy + r), (cx - r * 0.22, cy + r * 0.22), (cx - r, cy), (cx - r * 0.22, cy - r * 0.22)], fill=color, outline=None)

    def degradado(self, x0, y0, x1, y1, arriba, abajo):
        for y in range(int(y0 * S), int(y1 * S)):
            k = (y - y0 * S) / max(1, (y1 - y0) * S)
            self.d.line([(x0 * S, y), (x1 * S, y)], fill=mezcla(arriba, abajo, k))

    def resplandor(self, cx, cy, r, color, fuerza=0.5):
        """Un halo suave (circulos con poca opacidad)."""
        for k in range(10, 0, -1):
            a = int(255 * fuerza * (1 - k / 10) ** 1.5 * 0.35)
            self.elipse(cx, cy, r * k / 10, r * k / 10, fill=color + (a,), outline=None)


# ---------- el marco de todas las cartas ----------
ANCHO_ARTE = (16, 42, 184, 262)  # donde va la ilustracion (en una carta de 200 x 320)


def fondo(p, arriba, abajo, rayos=None, estrellas=12, semilla=0):
    x0, y0, x1, y1 = ANCHO_ARTE
    p.degradado(x0, y0, x1, y1, arriba, abajo)
    if rayos:
        cx, cy = (x0 + x1) / 2, (y0 + y1) / 2 - 8
        for k in range(18):
            a0, a1 = k / 18 * math.tau, (k + 0.5) / 18 * math.tau
            p.poly([(cx, cy), (cx + 220 * math.cos(a0), cy + 220 * math.sin(a0)), (cx + 220 * math.cos(a1), cy + 220 * math.sin(a1))], fill=rayos, outline=None)
    rnd = random.Random(semilla)
    for _ in range(estrellas):
        x, y = rnd.uniform(x0 + 6, x1 - 6), rnd.uniform(y0 + 6, y0 + (y1 - y0) * 0.55)
        p.brillo(x, y, rnd.uniform(1.5, 3.2), (255, 250, 220, 200))


def piso(p, color, y=232, borde=None):
    x0, _, x1, y1 = ANCHO_ARTE
    p.rect(x0, y, x1, y1, fill=color, outline=None)
    p.linea([(x0, y), (x1, y)], fill=borde or mezcla(color, (0, 0, 0), 0.3), w=1.2)


def sombra_piso(p, cx, cy, rx):
    p.elipse(cx, cy, rx, rx * 0.22, fill=(0, 0, 0, 70), outline=None)


def marco(p, romano, nombre):
    W, H = 200, 320
    x0, y0, x1, y1 = ANCHO_ARTE
    # Lo de afuera del arte, crema; con el doble filo dorado.
    capa = Image.new("L", p.img.size, 0)
    ImageDraw.Draw(capa).rectangle([x0 * S, y0 * S, x1 * S, y1 * S], fill=255)
    crema = Image.new("RGB", p.img.size, CREMA)
    p.img.paste(crema, (0, 0), Image.eval(capa, lambda v: 255 - v))
    p.rect(3, 3, W - 3, H - 3, outline=ORO, w=3.2, r=8)
    p.rect(8, 8, W - 8, H - 8, outline=TINTA, w=1.0, r=5)
    p.rect(x0, y0, x1, y1, outline=ORO, w=2.4)
    p.rect(x0 - 3, y0 - 3, x1 + 3, y1 + 3, outline=TINTA, w=0.8)
    # Adornos en las esquinas: un rombo con puntitos.
    for (cx, cy) in ((14, 14), (W - 14, 14), (14, H - 14), (W - 14, H - 14)):
        p.poly([(cx, cy - 5), (cx + 5, cy), (cx, cy + 5), (cx - 5, cy)], fill=ORO, outline=TINTA, w=0.8)
        p.circulo(cx, cy, 1.2, fill=CREMA, outline=None)
    # La plaquita del numero.
    wr = max(26, p.medir(romano, 15) + 16)
    p.rect(100 - wr / 2, 13, 100 + wr / 2, 35, fill=MORADO, outline=ORO, w=1.6, r=5)
    p.texto(100, 15.5, romano, 15, fill=ORO_CLARO)
    # El liston del nombre, con las puntas dobladas.
    yl = 270
    p.poly([(8, yl + 4), (24, yl + 4), (24, yl + 30), (8, yl + 30), (15, yl + 17)], fill=mezcla(MORADO, (0, 0, 0), 0.35), outline=TINTA, w=1.0)
    p.poly([(192, yl + 4), (176, yl + 4), (176, yl + 30), (192, yl + 30), (185, yl + 17)], fill=mezcla(MORADO, (0, 0, 0), 0.35), outline=TINTA, w=1.0)
    p.rect(20, yl, 180, yl + 28, fill=MORADO, outline=ORO, w=1.6, r=3)
    tam = 15
    while p.medir(nombre, tam) > 150 and tam > 9:
        tam -= 0.5
    p.texto(100, yl + 14 - tam * 0.62, nombre, tam, fill=ORO_CLARO, sombra=(20, 8, 30))


def reverso(p):
    W, H = 200, 320
    p.degradado(0, 0, W, H, (60, 28, 110), (30, 12, 60))
    for k in range(24):
        a0, a1 = k / 24 * math.tau, (k + 0.5) / 24 * math.tau
        p.poly([(100, 160), (100 + 300 * math.cos(a0), 160 + 300 * math.sin(a0)), (100 + 300 * math.cos(a1), 160 + 300 * math.sin(a1))], fill=(255, 255, 255, 10), outline=None)
    rnd = random.Random(5)
    for _ in range(40):
        p.brillo(rnd.uniform(16, W - 16), rnd.uniform(16, H - 16), rnd.uniform(1.2, 3.0), (255, 240, 200, 220))
    p.rect(3, 3, W - 3, H - 3, outline=ORO, w=3.2, r=8)
    p.rect(10, 10, W - 10, H - 10, outline=(150, 110, 210), w=1.2, r=5)
    p.rect(16, 16, W - 16, H - 16, outline=ORO, w=1.2, r=3)
    # Un sol y una luna en las esquinas, y el ojo con el simbolo de Mercurio al centro.
    p.resplandor(100, 160, 70, (180, 140, 255), 0.6)
    p.circulo(100, 160, 46, fill=(40, 18, 80), outline=ORO, w=2.4)
    p.circulo(100, 160, 38, outline=(150, 110, 210), w=1.0)
    p.arco(100, 128, 14, 0, 180, fill=ORO, w=3.2)
    p.circulo(100, 152, 15, outline=ORO, w=3.4)
    p.linea([(100, 167), (100, 196)], fill=ORO, w=3.4)
    p.linea([(88, 182), (112, 182)], fill=ORO, w=3.4)
    p.esfera(100, 152, 7, (90, 220, 210))
    for (cx, cy, luna) in ((34, 40, False), (166, 280, True)):
        if luna:
            p.circulo(cx, cy, 11, fill=ORO, outline=None)
            p.circulo(cx + 5, cy - 3, 9.5, fill=(34, 14, 66), outline=None)
        else:
            for k in range(8):
                a = k / 8 * math.tau
                p.linea([(cx + 9 * math.cos(a), cy + 9 * math.sin(a)), (cx + 15 * math.cos(a), cy + 15 * math.sin(a))], fill=ORO, w=1.6)
            p.circulo(cx, cy, 7, fill=ORO, outline=None)


# ---------- las ilustraciones ----------
def c_wifi(p):
    fondo(p, (24, 26, 80), (100, 60, 160), rayos=(255, 255, 255, 14), semilla=1)
    cx, cy = 100, 150
    p.resplandor(cx, cy + 10, 80, (90, 200, 255), 0.7)
    for k, r in enumerate((62, 46, 30)):
        p.arco(cx, cy + 34, r, 225, 315, fill=(40, 120, 220), w=9.5)
        p.arco(cx, cy + 34, r, 228, 312, fill=(130, 220, 255) if k else (220, 245, 255), w=5.5)
    p.esfera(cx, cy + 30, 7, (120, 210, 255))
    # el router
    piso(p, (60, 40, 100))
    sombra_piso(p, cx, 238, 44)
    p.rect(60, 206, 140, 236, fill=(235, 235, 245), outline=TINTA, w=1.8, r=6)
    p.rect(60, 224, 140, 236, fill=(200, 200, 215), outline=TINTA, w=1.2, r=4)
    for x in (66, 134):
        p.linea([(x, 206), (x - 4 * (1 if x < 100 else -1), 186)], fill=TINTA, w=3.2)
    for k, c in enumerate(((90, 230, 120), (90, 230, 120), (255, 200, 60), (255, 80, 80))):
        p.circulo(80 + k * 13, 215, 2.6, fill=c, outline=None)
    for (x, y) in ((40, 70), (160, 90), (150, 60)):
        p.brillo(x, y, 5, (200, 240, 255))


def c_camioneta(p):
    fondo(p, (120, 190, 240), (250, 210, 160), estrellas=0)
    # los volcanes de atras
    p.poly([(16, 200), (66, 110), (78, 110), (130, 200)], fill=(110, 120, 150), outline=None)
    p.poly([(90, 200), (146, 96), (158, 96), (184, 140), (184, 200)], fill=(90, 100, 135), outline=None)
    p.circulo(150, 92, 10, fill=(230, 230, 235, 200), outline=None)
    piso(p, (90, 90, 100), y=214)
    p.linea([(16, 240), (184, 240)], fill=(240, 220, 120), w=2)
    sombra_piso(p, 100, 232, 70)
    # la camioneta: roja, amarilla y verde, con su parrilla llena de bultos
    p.rect(30, 150, 172, 222, fill=(220, 50, 60), outline=TINTA, w=2, r=10)
    p.rect(30, 186, 172, 196, fill=(255, 210, 60), outline=None)
    p.rect(30, 196, 172, 202, fill=(60, 170, 90), outline=None)
    for k in range(5):
        p.rect(40 + k * 25, 158, 58 + k * 25, 178, fill=(170, 220, 255), outline=TINTA, w=1.2, r=2)
        p.linea([(43 + k * 25, 161), (50 + k * 25, 168)], fill=(255, 255, 255), w=1.2)
    p.rect(150, 158, 168, 212, fill=(140, 200, 240), outline=TINTA, w=1.4, r=2)
    p.rect(36, 136, 156, 150, fill=(110, 80, 50), outline=TINTA, w=1.4)
    for k, c in enumerate(((240, 120, 160), (90, 140, 230), (250, 200, 80), (140, 90, 200))):
        p.rect(40 + k * 28, 120 - (k % 2) * 6, 62 + k * 28, 138, fill=c, outline=TINTA, w=1.2, r=3)
    for x in (58, 144):
        p.circulo(x, 222, 11, fill=(40, 40, 50), outline=TINTA, w=1.4)
        p.circulo(x, 222, 4.5, fill=(190, 190, 200), outline=None)
    p.circulo(172, 206, 3.5, fill=(255, 240, 160), outline=TINTA, w=0.8)
    # el ayudante colgado de la puerta
    p.circulo(176, 160, 6, fill=(200, 150, 110), outline=TINTA, w=1.2)
    p.linea([(176, 166), (180, 190)], fill=(60, 100, 200), w=4)
    p.linea([(176, 170), (168, 162)], fill=(200, 150, 110), w=2.4)
    p.linea([(178, 172), (190, 160)], fill=(200, 150, 110), w=2.4)
    p.texto(100, 98, "¡HAY LUGAR!", 13, fill=(255, 255, 255), sombra=TINTA)


def c_ex(p):
    fondo(p, (20, 14, 50), (80, 30, 90), semilla=3)
    p.circulo(150, 70, 16, fill=(250, 240, 200), outline=None)
    p.circulo(158, 64, 14, fill=(26, 16, 54), outline=None)
    p.resplandor(100, 150, 70, (120, 220, 160), 0.4)
    # el telefono
    p.rect(62, 82, 138, 238, fill=(30, 30, 40), outline=TINTA, w=2, r=12)
    p.rect(68, 96, 132, 226, fill=(235, 240, 255), outline=None, r=4)
    p.rect(88, 87, 112, 91, fill=(70, 70, 80), outline=None, r=2)
    p.texto(100, 100, "2:13 a. m.", 10, fill=(120, 120, 140))
    p.rect(72, 118, 124, 140, fill=(120, 220, 140), outline=None, r=8)
    p.poly([(76, 138), (72, 146), (84, 139)], fill=(120, 220, 140), outline=None)
    p.texto(98, 122, "hola :)", 12)
    p.rect(78, 148, 128, 170, fill=(120, 220, 140), outline=None, r=8)
    p.texto(103, 152, "¿cómo", 11)
    p.texto(100, 176, "escribiendo...", 9, fill=(120, 120, 140))
    # el corazon partido
    for s, dx in ((-1, -2), (1, 2)):
        x = 100 + dx
        p.poly([(x, 196), (x + s * 10, 186), (x + s * 18, 192), (x + s * 18, 202), (x, 222)], fill=(230, 50, 80), outline=TINTA, w=1.2)
    p.linea([(100, 196), (96, 204), (103, 210), (98, 220)], fill=(235, 240, 255), w=1.6)


def c_parcial(p):
    fondo(p, (230, 220, 200), (200, 180, 160), estrellas=0)
    piso(p, (150, 100, 60), y=222)
    sombra_piso(p, 100, 228, 60)
    p.poly([(52, 70), (152, 62), (160, 226), (60, 232)], fill=(255, 255, 255), outline=TINTA, w=1.8)
    for k in range(8):
        y = 120 + k * 12
        p.linea([(66, y + 1 - k * 0.08), (150, y - 4 - k * 0.08)], fill=(160, 175, 220), w=1.0)
    p.linea([(74, 72), (78, 230)], fill=(240, 120, 120), w=1.0)
    p.texto(108, 80, "Teoría de la", 9)
    p.texto(108, 92, "Computación", 9)
    p.elipse(98, 128, 26, 20, outline=(220, 30, 40), w=3.2)
    p.texto(98, 112, "59", 26, fill=(220, 30, 40))
    p.texto(130, 150, "¡¿?!", 12, fill=(220, 30, 40))
    # el lapiz
    p.poly([(118, 200), (176, 168), (182, 178), (124, 210)], fill=(250, 200, 60), outline=TINTA, w=1.4)
    p.poly([(118, 200), (124, 210), (110, 212)], fill=(240, 210, 170), outline=TINTA, w=1.0)
    p.poly([(176, 168), (182, 178), (186, 175), (180, 165)], fill=(240, 130, 150), outline=TINTA, w=1.0)
    p.texto(56, 200, "-1", 12, fill=(220, 30, 40))


def c_torre(p):
    fondo(p, (30, 30, 70), (90, 70, 130), estrellas=0)
    for (x, y, r) in ((50, 70, 26), (90, 60, 30), (140, 72, 28), (170, 80, 20)):
        p.circulo(x, y, r, fill=(70, 70, 100), outline=None)
    p.poly([(16, 238), (60, 200), (120, 194), (184, 230), (184, 262), (16, 262)], fill=(60, 110, 70), outline=None)
    # la torre de senal
    cx = 100
    p.linea([(cx - 26, 230), (cx, 80), (cx + 26, 230)], fill=(170, 175, 190), w=3)
    for k in range(6):
        y0, y1 = 220 - k * 24, 196 - k * 24
        w0, w1 = 26 - k * 4.1, 26 - (k + 1) * 4.1
        p.linea([(cx - w0, y0), (cx + w1, y1)], fill=(170, 175, 190), w=1.6)
        p.linea([(cx + w0, y0), (cx - w1, y1)], fill=(170, 175, 190), w=1.6)
    p.esfera(cx, 78, 5, (255, 60, 60))
    p.resplandor(cx, 78, 24, (255, 80, 80), 0.6)
    for k in range(3):
        p.arco(cx, 78, 16 + k * 10, 200, 340, fill=(140, 220, 255), w=1.8)
    # el rayo
    p.poly([(150, 50), (128, 104), (142, 104), (126, 150), (166, 92), (150, 92), (166, 50)], fill=(255, 220, 60), outline=TINTA, w=1.2)
    p.resplandor(146, 96, 40, (255, 230, 120), 0.5)


def c_sol(p):
    fondo(p, (255, 170, 60), (255, 230, 150), rayos=(255, 255, 255, 40), estrellas=0)
    cx, cy = 100, 140
    p.resplandor(cx, cy, 90, (255, 240, 180), 0.8)
    for k in range(16):
        a = k / 16 * math.tau
        r0, r1 = 50, 70 if k % 2 == 0 else 62
        p.poly([(cx + r0 * math.cos(a - 0.12), cy + r0 * math.sin(a - 0.12)), (cx + r1 * math.cos(a), cy + r1 * math.sin(a)), (cx + r0 * math.cos(a + 0.12), cy + r0 * math.sin(a + 0.12))], fill=(255, 140, 30), outline=TINTA, w=1.0)
    p.esfera(cx, cy, 46, (255, 200, 50), (255, 245, 170))
    # los lentes oscuros
    p.rect(66, 124, 96, 142, fill=(30, 30, 50), outline=TINTA, w=1.4, r=6)
    p.rect(104, 124, 134, 142, fill=(30, 30, 50), outline=TINTA, w=1.4, r=6)
    p.linea([(96, 129), (104, 129)], fill=TINTA, w=2.4)
    p.linea([(70, 128), (78, 128)], fill=(140, 140, 200), w=1.6)
    p.linea([(108, 128), (116, 128)], fill=(140, 140, 200), w=1.6)
    p.arco(cx, 148, 16, 20, 160, fill=TINTA, w=2.6)
    p.circulo(72, 154, 5, fill=(255, 140, 120, 160), outline=None)
    p.circulo(128, 154, 5, fill=(255, 140, 120, 160), outline=None)
    piso(p, (240, 200, 120), y=236)
    p.texto(100, 240, "SPF 1000", 11, fill=(180, 90, 30))


def c_luna(p):
    fondo(p, (14, 14, 46), (50, 40, 110), semilla=7, estrellas=22)
    cx, cy = 112, 120
    p.resplandor(cx, cy, 80, (255, 230, 150), 0.5)
    p.creciente(cx, cy, 52, 28, -16, (255, 215, 110), (255, 245, 200))
    for (x, y, r) in ((78, 112, 5), (80, 140, 6), (98, 160, 4)):
        p.circulo(x, y, r, fill=(230, 180, 90), outline=None)
    # el columpio colgado de la punta de abajo, con alguien de coletas cian
    p.linea([(78, 150), (74, 210)], fill=(240, 230, 200), w=1.2)
    p.linea([(104, 166), (106, 210)], fill=(240, 230, 200), w=1.2)
    p.rect(68, 208, 112, 214, fill=(140, 90, 50), outline=TINTA, w=1.0)
    p.circulo(90, 190, 8, fill=(250, 220, 190), outline=TINTA, w=1.0)
    p.poly([(82, 186), (70, 214), (78, 214), (86, 192)], fill=(60, 210, 200), outline=TINTA, w=0.8)
    p.poly([(98, 186), (110, 214), (102, 214), (94, 192)], fill=(60, 210, 200), outline=TINTA, w=0.8)
    p.rect(84, 197, 96, 208, fill=(80, 80, 100), outline=TINTA, w=0.8)
    for k in range(5):
        x, y = 36 + k * 30, 238 - (k % 2) * 10
        p.elipse(x, y, 4, 3, fill=(150, 255, 240), outline=None)
        p.linea([(x + 3.5, y), (x + 3.5, y - 13)], fill=(150, 255, 240), w=1.6)
        p.linea([(x + 3.5, y - 13), (x + 8, y - 8)], fill=(150, 255, 240), w=1.6)


def c_cangrejo(p):
    fondo(p, (120, 200, 240), (200, 235, 250), estrellas=0)
    p.poly([(16, 190), (60, 182), (100, 192), (140, 180), (184, 188), (184, 210), (16, 210)], fill=(80, 160, 220), outline=None)
    piso(p, (240, 215, 160), y=206)
    sombra_piso(p, 100, 228, 56)
    cx, cy = 100, 182
    for s in (-1, 1):
        for k in range(3):
            p.linea([(cx + s * 30, cy + 8 + k * 6), (cx + s * 50, cy + 18 + k * 8), (cx + s * 56, cy + 34 + k * 6)], fill=(200, 70, 30), w=3.2)
        p.linea([(cx + s * 34, cy - 6), (cx + s * 52, cy - 34)], fill=(200, 70, 30), w=4)
        p.esfera(cx + s * 56, cy - 48, 15, (245, 110, 50))
        p.poly([(cx + s * 50, cy - 58), (cx + s * 62, cy - 66), (cx + s * 58, cy - 50)], fill=(240, 215, 160), outline=TINTA, w=1.0)
    p.esfera(cx, cy, 40, (245, 110, 50), ry=26)
    for s in (-1, 1):
        p.linea([(cx + s * 12, cy - 22), (cx + s * 14, cy - 38)], fill=TINTA, w=2)
        p.circulo(cx + s * 14, cy - 42, 6.5, fill=(255, 255, 255), outline=TINTA, w=1.2)
        p.circulo(cx + s * 15, cy - 41, 3, fill=TINTA, outline=None)
    p.arco(cx, cy + 2, 10, 20, 160, fill=TINTA, w=2.2)
    p.texto(100, 76, "Ferris", 14, fill=(200, 70, 30), sombra=(255, 255, 255))
    p.texto(100, 94, "0 segfaults", 10, fill=(40, 90, 140))


def c_chucho(p):
    fondo(p, (250, 200, 150), (240, 160, 130), estrellas=0, rayos=(255, 255, 255, 24))
    piso(p, (170, 170, 180), y=214)
    p.linea([(16, 222), (184, 222)], fill=(130, 130, 140), w=1.2)
    sombra_piso(p, 96, 236, 52)
    cx, cy = 96, 176
    # la cola, moviendose
    for k, a in enumerate((-0.9, -0.6)):
        p.linea([(cx - 34, cy + 6), (cx - 52, cy - 16 + k * 10)], fill=(150, 100, 60) if k else (200, 150, 90), w=6)
    p.esfera(cx - 4, cy + 10, 36, (205, 155, 95), ry=24)
    p.elipse(cx - 18, cy + 4, 12, 9, fill=(120, 80, 50), outline=None)
    for x in (-28, -12, 12, 26):
        p.rect(cx + x - 4, cy + 24, cx + x + 4, cy + 54, fill=(205, 155, 95), outline=TINTA, w=1.2, r=3)
        p.elipse(cx + x, cy + 54, 6, 3, fill=(190, 140, 80), outline=TINTA, w=1.0)
    p.esfera(cx + 28, cy - 30, 24, (205, 155, 95))
    p.poly([(cx + 12, cy - 46), (cx + 6, cy - 74), (cx + 26, cy - 52)], fill=(150, 100, 60), outline=TINTA, w=1.2)
    p.poly([(cx + 38, cy - 50), (cx + 54, cy - 72), (cx + 50, cy - 44)], fill=(150, 100, 60), outline=TINTA, w=1.2)
    p.elipse(cx + 44, cy - 20, 13, 9, fill=(230, 195, 150), outline=TINTA, w=1.2)
    p.circulo(cx + 52, cy - 24, 3.6, fill=TINTA, outline=None)
    p.circulo(cx + 22, cy - 34, 3.4, fill=TINTA, outline=None)
    p.circulo(cx + 36, cy - 36, 3.4, fill=TINTA, outline=None)
    p.circulo(cx + 23, cy - 35.5, 1.0, fill=(255, 255, 255), outline=None)
    p.circulo(cx + 37, cy - 37.5, 1.0, fill=(255, 255, 255), outline=None)
    p.poly([(cx + 44, cy - 12), (cx + 48, cy - 2), (cx + 40, cy - 4)], fill=(240, 100, 120), outline=TINTA, w=0.8)
    p.rect(cx + 8, cy - 12, cx + 30, cy - 6, fill=(220, 40, 60), outline=TINTA, w=1.0, r=2)
    p.circulo(cx + 19, cy - 3, 3, fill=(250, 200, 60), outline=TINTA, w=0.8)


def c_tortilla(p):
    fondo(p, (150, 70, 40), (230, 140, 70), estrellas=0, rayos=(255, 230, 180, 22))
    piso(p, (110, 60, 40), y=210)
    # el comal
    p.elipse(100, 214, 76, 18, fill=(50, 45, 50), outline=TINTA, w=1.6)
    p.elipse(100, 210, 70, 14, fill=(80, 75, 80), outline=None)
    # el canasto con su servilleta tipica
    p.poly([(36, 196), (164, 196), (150, 236), (50, 236)], fill=(190, 140, 70), outline=TINTA, w=1.6)
    for k in range(6):
        p.linea([(44 + k * 22, 198), (54 + k * 18, 234)], fill=(150, 100, 40), w=1.2)
    for k, c in enumerate(((220, 40, 70), (40, 120, 200), (250, 190, 40), (60, 170, 90))):
        p.rect(40, 196 + k * 2.5, 160, 198.5 + k * 2.5, fill=c, outline=None)
    for k in range(5):
        y = 188 - k * 9
        p.elipse(100, y, 56, 12, fill=(245, 220, 150), outline=(190, 150, 80), w=1.4)
        for m in range(4):
            p.circulo(76 + m * 16 + (k % 2) * 6, y - 2 + (m % 2) * 3, 1.6, fill=(200, 150, 80), outline=None)
    for k in range(3):
        x = 76 + k * 24
        p.arco(x, 112, 8, 90, 270, fill=(255, 255, 255, 170), w=2.4)
        p.arco(x, 128, 8, 270, 90, fill=(255, 255, 255, 170), w=2.4)
        p.arco(x, 96, 8, 270, 90, fill=(255, 255, 255, 120), w=2.4)


def c_volcan(p):
    fondo(p, (30, 14, 40), (130, 50, 50), semilla=10, estrellas=10)
    # el Agua de atras y el Fuego echando lava
    p.poly([(16, 230), (56, 150), (70, 146), (110, 230)], fill=(60, 60, 90), outline=None)
    p.poly([(40, 262), (92, 110), (112, 110), (176, 262)], fill=(70, 55, 70), outline=TINTA, w=1.4)
    p.poly([(92, 110), (112, 110), (106, 130), (100, 124), (94, 134)], fill=(255, 120, 30), outline=None)
    for (x0, x1) in ((96, 76), (106, 132)):
        p.linea([(x0, 124), ((x0 + x1) / 2, 170), (x1, 220)], fill=(255, 110, 30), w=3.6)
        p.linea([(x0, 124), ((x0 + x1) / 2, 170), (x1, 220)], fill=(255, 220, 80), w=1.4)
    p.resplandor(102, 104, 60, (255, 120, 40), 0.9)
    rnd = random.Random(4)
    for k in range(10):
        a = -math.pi / 2 + rnd.uniform(-0.8, 0.8)
        r = rnd.uniform(16, 50)
        p.esfera(102 + r * math.cos(a), 100 + r * math.sin(a), rnd.uniform(2.5, 5), (255, 140 + k * 10, 40))
    for (x, y, r) in ((102, 62, 20), (86, 52, 14), (120, 48, 16), (104, 40, 12)):
        p.circulo(x, y, r, fill=(140, 130, 140, 220), outline=None)


def c_puerro(p):
    fondo(p, (90, 200, 190), (190, 240, 220), rayos=(255, 255, 255, 40), estrellas=0)
    p.resplandor(100, 120, 80, (255, 255, 220), 0.6)
    # las hojas
    for k, (a, c) in enumerate(((-0.45, (50, 150, 70)), (0.05, (70, 180, 90)), (0.5, (60, 165, 80)))):
        x1, y1 = 100 + 80 * math.sin(a), 56
        p.poly([(92, 150), (108, 150), (x1 + 9, y1), (x1 - 9, y1 - 2)], fill=c, outline=TINTA, w=1.4)
        p.linea([(100, 150), (x1, y1 + 4)], fill=mezcla(c, (255, 255, 255), 0.3), w=1.2)
    p.poly([(90, 236), (110, 236), (112, 146), (88, 146)], fill=(245, 250, 235), outline=TINTA, w=1.6)
    p.poly([(92, 236), (98, 236), (98, 150), (92, 150)], fill=(255, 255, 255), outline=None)
    for k in range(6):
        p.linea([(94 + k * 2.4, 236), (88 + k * 5, 252)], fill=(220, 210, 180), w=1.2)
    # la mano que lo sostiene en alto
    p.rect(84, 196, 116, 220, fill=(250, 215, 185), outline=TINTA, w=1.4, r=8)
    for k in range(3):
        p.linea([(86, 202 + k * 6), (114, 202 + k * 6)], fill=(220, 170, 140), w=1.0)
    p.rect(88, 220, 112, 260, fill=(60, 200, 190), outline=TINTA, w=1.4)
    for (x, y) in ((50, 80), (152, 100), (60, 150), (146, 60)):
        p.brillo(x, y, 6, (255, 255, 220))


def c_zbuffer(p):
    x0, y0, x1, y1 = ANCHO_ARTE
    for k in range(22):
        g = 20 + k * 10
        p.rect(x0, y0 + k * 10.1, x1, y0 + (k + 1) * 10.1, fill=(g, g, g + 8), outline=None)

    def cubo(cx, cy, s, cara):
        p.poly([(cx - s, cy - s * 0.5), (cx, cy - s), (cx + s, cy - s * 0.5), (cx, cy)], fill=mezcla(cara, (255, 255, 255), 0.35), outline=TINTA, w=1.4)
        p.poly([(cx - s, cy - s * 0.5), (cx, cy), (cx, cy + s), (cx - s, cy + s * 0.5)], fill=cara, outline=TINTA, w=1.4)
        p.poly([(cx, cy), (cx + s, cy - s * 0.5), (cx + s, cy + s * 0.5), (cx, cy + s)], fill=mezcla(cara, (0, 0, 0), 0.35), outline=TINTA, w=1.4)

    cubo(140, 96, 18, (120, 120, 140))
    cubo(66, 130, 24, (170, 170, 190))
    cubo(108, 182, 34, (230, 230, 240))
    p.texto(100, 236, "z = 0.9999", 11, fill=(255, 90, 140))
    p.texto(48, 64, "lejos", 10, fill=(150, 150, 170))
    p.texto(160, 226, "cerca", 10, fill=(40, 40, 50))


def c_bateria(p):
    fondo(p, (40, 140, 90), (150, 230, 140), rayos=(255, 255, 255, 30), estrellas=0)
    p.resplandor(100, 150, 80, (220, 255, 160), 0.7)
    p.rect(64, 76, 136, 234, fill=(40, 45, 60), outline=TINTA, w=2.4, r=10)
    p.rect(84, 64, 116, 78, fill=(40, 45, 60), outline=TINTA, w=2, r=3)
    for k in range(5):
        y = 216 - k * 28
        p.rect(72, y - 22, 128, y + 2, fill=mezcla((90, 220, 110), (200, 255, 150), k / 4), outline=None, r=4)
    p.rect(70, 84, 82, 226, fill=(255, 255, 255, 50), outline=None, r=4)
    p.poly([(106, 96), (84, 158), (100, 158), (92, 212), (122, 136), (104, 136), (116, 96)], fill=(255, 230, 60), outline=TINTA, w=1.6)
    p.texto(100, 238, "100%", 13, fill=(255, 255, 255), sombra=TINTA)


def c_muerte(p):
    fondo(p, (30, 20, 50), (90, 60, 110), semilla=14, estrellas=14)
    piso(p, (50, 40, 60), y=228)
    # flores de muerto y velitas
    for (x, y) in ((32, 232), (54, 238), (150, 236), (170, 230)):
        for k in range(7):
            a = k / 7 * math.tau
            p.circulo(x + 4 * math.cos(a), y + 4 * math.sin(a), 3.4, fill=(250, 150, 30), outline=None)
        p.circulo(x, y, 2.6, fill=(230, 90, 20), outline=None)
    for x in (72, 128):
        p.rect(x - 4, 222, x + 4, 244, fill=(250, 245, 230), outline=TINTA, w=1.0)
        p.elipse(x, 216, 3, 6, fill=(255, 200, 60), outline=None)
        p.resplandor(x, 216, 16, (255, 200, 80), 0.6)
    # la guadana
    p.linea([(146, 70), (140, 226)], fill=(120, 80, 50), w=4)
    p.poly([(146, 72), (96, 52), (70, 72), (104, 64)], fill=(210, 215, 230), outline=TINTA, w=1.4)
    # la calavera con su capa, sonriendo con el telefono
    p.poly([(60, 228), (78, 120), (122, 120), (140, 228)], fill=(40, 30, 60), outline=TINTA, w=1.6)
    p.esfera(100, 112, 28, (240, 236, 225), (255, 255, 250))
    p.circulo(89, 110, 7.5, fill=TINTA, outline=None)
    p.circulo(111, 110, 7.5, fill=TINTA, outline=None)
    p.circulo(91, 108, 2, fill=(120, 220, 255), outline=None)
    p.circulo(113, 108, 2, fill=(120, 220, 255), outline=None)
    p.poly([(100, 118), (96, 126), (104, 126)], fill=TINTA, outline=None)
    p.rect(88, 130, 112, 138, fill=(255, 255, 250), outline=TINTA, w=1.0, r=2)
    for k in range(5):
        p.linea([(90 + k * 5, 130), (90 + k * 5, 138)], fill=TINTA, w=0.8)
    p.rect(110, 160, 128, 190, fill=(30, 30, 40), outline=TINTA, w=1.2, r=3)
    p.rect(113, 164, 125, 184, fill=(255, 80, 80), outline=None, r=1)
    p.texto(119, 168, "1%", 7, fill=(255, 255, 255))
    p.circulo(108, 172, 6, fill=(240, 236, 225), outline=TINTA, w=1.0)


def c_trafico(p):
    fondo(p, (250, 140, 90), (250, 200, 140), estrellas=0)
    for k, h in enumerate((60, 90, 50, 110, 70, 84, 56)):
        x = 16 + k * 25
        p.rect(x, 200 - h, x + 22, 200, fill=(120, 90, 130), outline=None)
        for m in range(int(h / 14)):
            p.rect(x + 5, 206 - h + m * 14, x + 9, 210 - h + m * 14, fill=(255, 230, 140), outline=None)
    p.poly([(16, 262), (84, 196), (116, 196), (184, 262)], fill=(70, 70, 80), outline=None)
    p.rect(16, 196, 184, 200, fill=(70, 70, 80), outline=None)
    for k in range(4):
        y = 206 + k * 14
        p.rect(98, y, 102, y + 6 + k * 2, fill=(240, 220, 120), outline=None)
    # los carros, de atras para adelante (mas grandes)
    for (cx, cy, s, c) in ((92, 204, 0.45, (60, 120, 220)), (114, 214, 0.6, (240, 200, 60)), (76, 230, 0.8, (90, 180, 110)), (122, 248, 1.0, (220, 60, 60))):
        w, h = 34 * s, 20 * s
        p.rect(cx - w, cy - h, cx + w, cy + h * 0.6, fill=c, outline=TINTA, w=1.2, r=4 * s)
        p.rect(cx - w * 0.6, cy - h * 1.6, cx + w * 0.6, cy - h * 0.8, fill=mezcla(c, (0, 0, 0), 0.2), outline=TINTA, w=1.0, r=3 * s)
        for sx in (-1, 1):
            p.circulo(cx + sx * w * 0.75, cy - h * 0.2, 4.5 * s, fill=(255, 40, 40), outline=None)
            p.resplandor(cx + sx * w * 0.75, cy - h * 0.2, 14 * s, (255, 60, 60), 0.6)
    # el semaforo en rojo
    p.rect(152, 80, 170, 130, fill=(40, 40, 50), outline=TINTA, w=1.4, r=4)
    p.linea([(161, 130), (161, 196)], fill=(40, 40, 50), w=3)
    for k, c in enumerate(((255, 60, 60), (90, 70, 40), (40, 80, 50))):
        p.circulo(161, 90 + k * 15, 5.5, fill=c, outline=None)
    p.resplandor(161, 90, 20, (255, 60, 60), 0.7)
    p.texto(70, 70, "2 h 47 min", 13, fill=(200, 30, 50), sombra=(255, 255, 255))


def c_cafe(p):
    fondo(p, (110, 60, 40), (220, 160, 110), estrellas=0, rayos=(255, 230, 200, 24))
    p.poly([(16, 150), (60, 110), (100, 140), (150, 100), (184, 130), (184, 200), (16, 200)], fill=(70, 120, 70), outline=None)
    for k in range(10):
        x = 24 + k * 16
        p.circulo(x, 150 + (k % 3) * 10, 3.5, fill=(200, 30, 40), outline=None)
    piso(p, (120, 80, 60), y=210)
    sombra_piso(p, 100, 236, 56)
    p.elipse(100, 230, 60, 12, fill=(235, 235, 240), outline=TINTA, w=1.6)
    p.poly([(56, 150), (144, 150), (134, 226), (66, 226)], fill=(250, 250, 245), outline=TINTA, w=1.8)
    p.poly([(60, 154), (70, 154), (76, 222), (70, 222)], fill=(255, 255, 255), outline=None)
    p.arco(150, 182, 18, 280, 80, fill=TINTA, w=6)
    p.arco(150, 182, 18, 282, 78, fill=(250, 250, 245), w=3)
    p.elipse(100, 152, 44, 9, fill=(110, 60, 30), outline=TINTA, w=1.4)
    p.elipse(92, 150, 12, 3, fill=(170, 110, 70), outline=None)
    p.texto(100, 176, "GT", 18, fill=(110, 60, 30))
    for k in range(3):
        x = 80 + k * 20
        p.arco(x, 126, 7, 90, 270, fill=(255, 255, 255, 190), w=2.6)
        p.arco(x, 112, 7, 270, 90, fill=(255, 255, 255, 150), w=2.6)
        p.arco(x, 98, 7, 90, 270, fill=(255, 255, 255, 100), w=2.6)


def c_lluvia(p):
    fondo(p, (90, 100, 130), (150, 160, 180), estrellas=0)
    for (x, y, r) in ((60, 86, 28), (96, 72, 34), (136, 86, 28), (80, 100, 24), (118, 102, 26)):
        p.esfera(x, y, r, (170, 175, 195), (220, 225, 240), outline=None)
    p.rect(40, 100, 160, 118, fill=(170, 175, 195), outline=None)
    rnd = random.Random(8)
    for _ in range(26):
        x, y = rnd.uniform(30, 176), rnd.uniform(124, 220)
        p.linea([(x, y), (x - 4, y + 12)], fill=(90, 160, 240), w=2)
    piso(p, (80, 80, 95), y=232)
    p.elipse(100, 246, 50, 8, fill=(110, 150, 200), outline=None)
    for k in range(6):
        a = math.pi + k / 5 * math.pi
        p.linea([(100 + 40 * math.cos(a), 244 + 6 * math.sin(a)), (100 + 52 * math.cos(a), 236 + 10 * math.sin(a))], fill=(160, 200, 240), w=1.6)
    # el paraguas volteado por el viento
    p.poly([(76, 180), (124, 166), (130, 196), (100, 186), (84, 200)], fill=(220, 60, 120), outline=TINTA, w=1.6)
    p.linea([(103, 182), (110, 234)], fill=TINTA, w=2.4)
    p.arco(116, 234, 6, 0, 180, fill=TINTA, w=2.4)
    p.texto(152, 136, "3:00 p. m.", 9, fill=(255, 255, 255), sombra=TINTA)


def c_quetzal(p):
    fondo(p, (40, 110, 80), (140, 200, 140), estrellas=0, rayos=(255, 255, 255, 22))
    for (x, y, r) in ((30, 60, 30), (170, 80, 34), (40, 200, 26)):
        p.circulo(x, y, r, fill=(30, 100, 60, 180), outline=None)
    # la rama
    p.linea([(16, 150), (110, 140), (184, 156)], fill=(110, 70, 40), w=6)
    for (x, y) in ((140, 146), (60, 146)):
        p.elipse(x, y - 8, 12, 5, fill=(60, 150, 70), outline=TINTA, w=1.0)
    # la cola larguisima
    for k, c in enumerate(((30, 140, 80), (40, 170, 100))):
        p.poly([(96 + k * 6, 150), (120 + k * 10, 252), (132 + k * 10, 250), (106 + k * 6, 148)], fill=c, outline=TINTA, w=1.2)
    cx, cy = 96, 116
    p.esfera(cx, cy, 24, (40, 175, 100), ry=30)
    p.cuerda(cx, cy + 6, 20, 0, 180, (220, 40, 50), ry=20)
    p.poly([(cx - 4, cy - 4), (cx + 26, cy + 10), (cx + 10, cy + 30)], fill=(30, 140, 80), outline=TINTA, w=1.2)
    p.esfera(cx - 4, cy - 34, 16, (40, 180, 105))
    p.poly([(cx - 10, cy - 48), (cx - 2, cy - 62), (cx + 6, cy - 46)], fill=(40, 180, 105), outline=TINTA, w=1.2)
    p.poly([(cx - 20, cy - 34), (cx - 32, cy - 30), (cx - 20, cy - 26)], fill=(250, 200, 60), outline=TINTA, w=1.0)
    p.circulo(cx - 10, cy - 38, 3.4, fill=TINTA, outline=None)
    p.circulo(cx - 11, cy - 39, 1, fill=(255, 255, 255), outline=None)


def c_compilador(p):
    fondo(p, (20, 20, 30), (50, 40, 70), semilla=19, estrellas=6)
    p.rect(26, 70, 174, 226, fill=(25, 25, 35), outline=(120, 120, 150), w=2, r=8)
    p.rect(26, 70, 174, 86, fill=(70, 70, 90), outline=None, r=6)
    for k, c in enumerate(((255, 90, 90), (255, 200, 60), (120, 220, 140))):
        p.circulo(36 + k * 11, 78, 3.2, fill=c, outline=None)
    p.texto(34, 94, "$ cargo build", 9.5, fill=(200, 200, 220), centro=False)
    p.texto(34, 110, "error[E0382]:", 10, fill=(255, 90, 100), centro=False)
    p.texto(34, 124, "borrow of moved", 9.5, fill=(230, 230, 240), centro=False)
    p.texto(34, 137, "value: `vida`", 9.5, fill=(230, 230, 240), centro=False)
    p.texto(34, 154, "warning: 847", 9.5, fill=(255, 200, 60), centro=False)
    p.texto(34, 168, "unused `cafe`", 9.5, fill=(255, 200, 60), centro=False)
    p.rect(34, 186, 42, 198, fill=(120, 220, 140), outline=None)
    # el cangrejo asomandose, preocupado
    p.esfera(146, 222, 22, (245, 110, 50), ry=14)
    for s in (-1, 1):
        p.circulo(146 + s * 8, 206, 4.5, fill=(255, 255, 255), outline=TINTA, w=1.0)
        p.circulo(146 + s * 8, 207, 2, fill=TINTA, outline=None)
        p.esfera(146 + s * 30, 210, 8, (245, 110, 50))
    p.arco(146, 226, 6, 200, 340, fill=TINTA, w=1.6)
    p.elipse(160, 196, 2, 3.5, fill=(120, 200, 255), outline=None)


def c_chisme(p):
    fondo(p, (250, 170, 200), (250, 220, 230), estrellas=0, rayos=(255, 255, 255, 40))
    # dos vecinas detras de la pared, una con el telefono
    piso(p, (200, 130, 110), y=200)
    for k in range(5):
        p.linea([(16, 210 + k * 12), (184, 210 + k * 12)], fill=(170, 100, 90), w=1.0)
    for (cx, pelo, ropa) in ((62, (60, 40, 30), (90, 150, 220)), (138, (180, 180, 190), (220, 90, 140))):
        p.esfera(cx, 178, 22, (240, 200, 170))
        p.cuerda(cx, 172, 24, 180, 360, pelo, ry=20)
        p.circulo(cx - 7, 180, 2.6, fill=TINTA, outline=None)
        p.circulo(cx + 7, 180, 2.6, fill=TINTA, outline=None)
        p.elipse(cx, 190, 5, 3.5, fill=(200, 60, 80), outline=None)
        p.rect(cx - 26, 198, cx + 26, 206, fill=ropa, outline=TINTA, w=1.2, r=3)
    p.rect(150, 172, 160, 190, fill=(30, 30, 40), outline=TINTA, w=1.0, r=2)
    p.rect(26, 62, 108, 108, fill=(255, 255, 255), outline=TINTA, w=1.8, r=12)
    p.poly([(52, 106), (46, 126), (68, 106)], fill=(255, 255, 255), outline=TINTA, w=1.6)
    p.texto(67, 72, "¿supo?", 17)
    p.rect(96, 110, 176, 152, fill=(255, 220, 235), outline=TINTA, w=1.8, r=12)
    p.poly([(146, 150), (152, 168), (130, 150)], fill=(255, 220, 235), outline=TINTA, w=1.6)
    p.texto(136, 118, "¡NO!", 20, fill=(200, 40, 90))
    for (x, y) in ((40, 140), (180, 70), (90, 48)):
        p.texto(x, y, "!", 14, fill=(200, 40, 90))


def c_pisto(p):
    fondo(p, (40, 120, 70), (160, 220, 140), rayos=(255, 255, 220, 40), estrellas=0)
    p.resplandor(100, 160, 90, (255, 240, 160), 0.7)
    # la bolsa de pisto
    p.esfera(100, 186, 46, (190, 150, 90), (240, 210, 150))
    p.poly([(84, 142), (116, 142), (124, 124), (76, 124)], fill=(190, 150, 90), outline=TINTA, w=1.6)
    p.linea([(82, 144), (118, 144)], fill=(150, 40, 40), w=4)
    p.circulo(100, 190, 22, fill=(250, 220, 90), outline=TINTA, w=1.6)
    p.texto(100, 177, "Q", 24, fill=(120, 80, 20))
    # los billetes saliendo
    for k, a in enumerate((-0.5, 0.0, 0.5)):
        cx, cy = 100 + 46 * math.sin(a), 104 - 10 * math.cos(a)
        ca, sa = math.cos(a), math.sin(a)
        pts = [(-26, -13), (26, -13), (26, 13), (-26, 13)]
        rot = [(cx + x * ca - y * sa, cy + x * sa + y * ca) for x, y in pts]
        p.poly(rot, fill=(130, 200, 130), outline=TINTA, w=1.4)
        p.circulo(cx, cy, 7, outline=(40, 110, 60), w=1.6)
    for (x, y) in ((40, 236), (62, 244), (150, 240), (168, 230)):
        p.esfera(x, y, 9, (240, 200, 70), (255, 240, 170))
        p.texto(x, y - 6, "Q", 9, fill=(150, 100, 20))


ILUSTRACIONES = [c_wifi, c_camioneta, c_ex, c_parcial, c_torre, c_sol, c_luna, c_cangrejo, c_chucho, c_tortilla, c_volcan,
                 c_puerro, c_zbuffer, c_bateria, c_muerte, c_trafico, c_cafe, c_lluvia, c_quetzal, c_compilador, c_chisme, c_pisto]


def pintar_carta(n, nombre, romano, fuente, ancho, alto):
    """La carta n ya reducida a ancho x alto (con n = None, el reverso)."""
    # En RGB: los colores con alfa se mezclan con lo de abajo (en RGBA, Pillow tambien pisaria el alfa).
    img = Image.new("RGB", (200 * S, 320 * S), CREMA)
    p = Pincel(img, fuente)
    if n is None:
        reverso(p)
    else:
        ILUSTRACIONES[n](p)
        marco(p, romano, nombre)
    return img.resize((ancho, alto), Image.LANCZOS)
