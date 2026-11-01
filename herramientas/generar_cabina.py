# Genera la cabina de la Nave Lila por dentro (Jupiter Asteroids, en primera persona):
#   modelos/cabina.obj: el tablero, las pantallas y los pilares de la burbuja.
#   modelos/cabina_palanca.obj: la palanca de mando (el juego la inclina con el viraje).
# Geometria lisa, no voxel: un tablero de tres paneles que abrazan al piloto (el del centro
# derecho, los de los lados girados hacia el), con la visera arriba y un filo de luz lila; tres
# pantallas hundidas en sus biseles (el juego dibuja el HUD adentro: escudo, radar y puntos),
# lucecitas y switches abajo, y los dos pilares de la burbuja que se curvan hacia el techo.
# Coordenadas de la camara: el ojo en el origen mirando a -Z, Y arriba.
# Uso: python3 herramientas/generar_cabina.py

import math
from modelo3d import *

m = Modelo()
partes = {}


def parte(nombre, mat):
    if (nombre, mat) not in partes:
        partes[(nombre, mat)] = m.part(nombre, mat)
    return partes[(nombre, mat)]


# ---------- el tablero ----------
# El perfil (y, z) del tablero visto de lado, de atras (contra el vidrio) hacia el piloto; cada
# tramo con su material. Lo de afuera queda a la izquierda del recorrido (girando en el plano z-y).
PERFIL = [
    ((-0.300, -1.660), "casco"),   # la visera, contra el vidrio
    ((-0.268, -1.460), "casco"),   # la visera por arriba
    ((-0.284, -1.416), "casco"),   # el filo redondeado
    ((-0.298, -1.412), "filo"),    # la linea de luz
    ((-0.312, -1.409), "casco"),
    ((-0.330, -1.405), "oscuro"),  # la ceja (por debajo)
    ((-0.380, -1.470), "panel"),   # el panel de las pantallas
    ((-0.800, -1.250), "casco"),   # la repisa de los switches
    ((-0.900, -1.120), "panel"),
    ((-1.250, -0.950), None),      # hacia las rodillas (fuera de cuadro)
]
CARA = 6  # el tramo del panel de las pantallas
CORTES = [-2.6, -1.5, -0.55, 0.55, 1.5, 2.6]


def zoff(x):
    """Cuanto se acerca el tablero al piloto en x: los paneles de los lados lo abrazan."""
    a = abs(x)
    if a <= 0.55:
        return 0.0
    if a <= 1.5:
        return 0.22 * (a - 0.55)
    return 0.22 * 0.95 + 0.6 * (a - 1.5)


for c in range(len(CORTES) - 1):
    xa, xb = CORTES[c], CORTES[c + 1]
    for i in range(len(PERFIL) - 1):
        (p0, mat), (p1, _) = PERFIL[i], PERFIL[i + 1]
        dz, dy = p1[1] - p0[1], p1[0] - p0[0]
        fuera = norm((0.0, dz, -dy))  # (x, y, z): girar el recorrido en el plano z-y
        medio = ((xa + xb) / 2, (p0[0] + p1[0]) / 2, (p0[1] + p1[1]) / 2 + zoff((xa + xb) / 2))
        adentro = sub(medio, mul(fuera, 0.05))
        pts = [(xa, p0[0], p0[1] + zoff(xa)), (xb, p0[0], p0[1] + zoff(xb)), (xb, p1[0], p1[1] + zoff(xb)), (xa, p1[0], p1[1] + zoff(xa))]
        parte("tablero_" + mat, mat).quad(pts, adentro)


def en_cara(x, s):
    """Un punto del panel de las pantallas: x real y s de 0 (arriba) a 1 (abajo)."""
    (y0, z0), (y1, z1) = PERFIL[CARA][0], PERFIL[CARA + 1][0]
    return (x, y0 + (y1 - y0) * s, z0 + (z1 - z0) * s + zoff(x))


def normal_cara(x):
    a, b, c = en_cara(x - 0.01, 0.5), en_cara(x + 0.01, 0.5), en_cara(x, 0.6)
    n = norm(cross(sub(b, a), sub(c, a)))
    return n if dot(n, a) < 0 else mul(n, -1)  # hacia el piloto


def prisma(nombre, mat, base, n, h0, h1):
    """Una caja sobre el panel: la base (4 esquinas en el panel) levantada de h0 a h1 por n."""
    abajo = [add(p, mul(n, h0)) for p in base]
    arriba = [add(p, mul(n, h1)) for p in base]
    centro = mul(add(add(arriba[0], arriba[2]), add(abajo[0], abajo[2])), 0.25)
    P = parte(nombre, mat)
    P.quad(arriba, centro)
    for e in range(4):
        P.quad([abajo[e], abajo[(e + 1) % 4], arriba[(e + 1) % 4], arriba[e]], centro)


def rect(x0, x1, s0, s1):
    return [en_cara(x0, s0), en_cara(x1, s0), en_cara(x1, s1), en_cara(x0, s1)]


# Las tres pantallas: el vidrio oscuro hundido en un bisel con botoncitos abajo (como las MFD de
# un avion). Sus esquinas van en la cabecera para que el juego dibuje el HUD justo adentro.
DS = 0.074  # 3.5 cm del panel, en s
PANTALLAS = {"pantalla_izq": (-1.30, -0.68, 0.10, 0.60), "pantalla_centro": (-0.40, 0.40, 0.08, 0.62), "pantalla_der": (0.68, 1.30, 0.10, 0.60)}
esquinas = {}
for nombre, (x0, x1, s0, s1) in PANTALLAS.items():
    n = normal_cara((x0 + x1) / 2)
    b = 0.035
    for (a0, a1, c0, c1) in ((x0 - b, x1 + b, s0 - DS, s0), (x0 - b, x1 + b, s1, s1 + DS), (x0 - b, x0, s0, s1), (x1, x1 + b, s0, s1)):
        prisma("bisel", "bisel", rect(a0, a1, c0, c1), n, -0.01, 0.018)
    vidrio = [add(p, mul(n, 0.005)) for p in rect(x0, x1, s0, s1)]
    parte(nombre, "vidrio").quad(vidrio, sub(mul(add(vidrio[0], vidrio[2]), 0.5), mul(n, 0.05)))
    esquinas[nombre] = vidrio
    # los botoncitos del bisel de abajo
    for k in range(6):
        x = x0 + (x1 - x0) * (k + 0.5) / 6
        prisma("bisel_botones", "boton", rect(x - 0.022, x + 0.022, s1 + DS * 0.25, s1 + DS * 0.75), n, 0.018, 0.026)
    # tornillitos dorados en las esquinas del bisel
    for (x, s) in ((x0 - b / 2, s0 - DS / 2), (x1 + b / 2, s0 - DS / 2), (x0 - b / 2, s1 + DS / 2), (x1 + b / 2, s1 + DS / 2)):
        prisma("tornillos", "dorado", rect(x - 0.006, x + 0.006, s - 0.012, s + 0.012), n, 0.018, 0.024)

# Abajo de las pantallas: una fila de lucecitas y unos switches con su palanquita.
LEDS = ["led_verde", "led_verde", "led_cian", "led_ambar", "led_verde", "led_rojo", "led_cian", "led_verde"]
for lado in (-1, 1):
    for k in range(8):
        x = lado * (0.66 + k * 0.08)
        n = normal_cara(x)
        prisma("leds", LEDS[(k * 3 + (lado > 0)) % 8], rect(x - 0.012, x + 0.012, 0.80, 0.84), n, 0.0, 0.008)
for k, x in enumerate((-0.36, -0.22, 0.22, 0.36)):
    n = normal_cara(x)
    prisma("switches", "bisel", rect(x - 0.03, x + 0.03, 0.74, 0.90), n, 0.0, 0.012)
    pie = add(en_cara(x, 0.82), mul(n, 0.012))
    punta = add(pie, add(mul(n, 0.05), (0.0, 0.025 if k % 2 else -0.025, 0.0)))
    d = norm(sub(punta, pie))
    lado = norm(cross(d, n))
    perp = norm(cross(lado, d))
    P = parte("switches_palanca", "metal")
    for (a, bb) in ((pie, punta),):
        sec = lambda c: [add(c, add(mul(lado, u * 0.006), mul(perp, v * 0.006))) for (u, v) in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
        k0, k1 = sec(a), sec(bb)
        centro = mul(add(a, bb), 0.5)
        for e in range(4):
            P.quad([k0[e], k0[(e + 1) % 4], k1[(e + 1) % 4], k1[e]], centro)
        P.quad(k1, centro)
    prisma("switches_punta", "led_ambar" if k == 1 else "boton", [add(p, sub(punta, pie)) for p in rect(x - 0.009, x + 0.009, 0.815, 0.825)], n, 0.0, 0.014)


# ---------- los pilares de la burbuja ----------
def catmull(pts, pasos):
    """Una curva suave que pasa por los puntos."""
    out = []
    for i in range(len(pts) - 1):
        p0, p1, p2, p3 = pts[max(i - 1, 0)], pts[i], pts[i + 1], pts[min(i + 2, len(pts) - 1)]
        for k in range(pasos):
            t = k / pasos
            t2, t3 = t * t, t * t * t
            out.append(tuple(0.5 * (2 * p1[q] + (-p0[q] + p2[q]) * t + (2 * p0[q] - 5 * p1[q] + 4 * p2[q] - p3[q]) * t2 + (-p0[q] + 3 * p1[q] - 3 * p2[q] + p3[q]) * t3) for q in range(3)))
    out.append(pts[-1])
    return out


def tubo(nombre, mat, pts, ancho, fondo, corrido=(0.0, 0.0)):
    """Un tubo de seccion rectangular por la curva: ancho de lado (visto desde el piloto) y fondo
    hacia el. `corrido` lo mueve hacia adentro (hacia el centro de la vista) y hacia el piloto."""
    secciones = []
    for i, p in enumerate(pts):
        t = norm(sub(pts[min(i + 1, len(pts) - 1)], pts[max(i - 1, 0)]))
        hacia = mul(norm(p), -1)
        nn = norm(sub(hacia, mul(t, dot(hacia, t))))
        s = norm(cross(t, nn))
        if dot(s, (-p[0], -p[1], 0.0)) < 0:
            s = mul(s, -1)
        c = add(p, add(mul(s, corrido[0]), mul(nn, corrido[1])))
        secciones.append((c, [add(c, add(mul(s, a * ancho / 2), mul(nn, b * fondo / 2))) for a, b in ((-1, -1), (1, -1), (1, 1), (-1, 1))], t))
    P = parte(nombre, mat)
    for i in range(len(secciones) - 1):
        (c0, k0, _), (c1, k1, _) = secciones[i], secciones[i + 1]
        centro = mul(add(c0, c1), 0.5)
        for e in range(4):
            P.quad([k0[e], k0[(e + 1) % 4], k1[(e + 1) % 4], k1[e]], centro)
    for (c, k, t), signo in ((secciones[0], 1), (secciones[-1], -1)):
        P.quad(k, add(c, mul(t, 0.01 * signo)))


for lado in (-1, 1):
    camino = catmull([(lado * 1.40, -0.31, -1.52), (lado * 1.38, 0.15, -1.53), (lado * 1.29, 0.56, -1.52),
                      (lado * 1.10, 0.90, -1.50), (lado * 0.80, 1.13, -1.48), (lado * 0.40, 1.27, -1.46), (0.0, 1.31, -1.45)], 8)
    tubo("pilar", "pilar", camino, 0.085, 0.10)
    tubo("pilar_filo", "filo", camino, 0.010, 0.010, corrido=(0.030, 0.052))
    # la base del pilar sobre la visera, con sus tornillos
    tubo("pilar_base", "casco", [(lado * 1.40, -0.33, -1.52), (lado * 1.40, -0.22, -1.525)], 0.15, 0.15)
    for dy in (-0.30, -0.25):
        tubo("tornillos", "dorado", [(lado * 1.40, dy, -1.44), (lado * 1.40, dy, -1.43)], 0.018, 0.018, corrido=(0.05, 0.0))

cabecera = """# La cabina de la Nave Lila por dentro (primera persona). Coordenadas de la camara:
# la camara en el origen mirando a -Z, Y arriba. El juego dibuja el HUD en las pantallas: estas son
# las esquinas del vidrio (arriba izq, arriba der, abajo der, abajo izq, vistas desde el piloto).
""" + "".join(f"# {n}: " + " ".join(f"{c:.4f}" for p in esquinas[n] for c in p) + "\n" for n in PANTALLAS)

MTL = """# Materiales de la cabina

newmtl casco
Kd 0.32 0.27 0.44
Ks 0.7 0.7 0.75
Ns 60

newmtl panel
Kd 0.13 0.12 0.17
Ks 0.3 0.3 0.3
Ns 20

newmtl oscuro
Kd 0.05 0.04 0.07
Ks 0.1 0.1 0.1
Ns 8

newmtl bisel
Kd 0.08 0.08 0.10
Ks 0.5 0.5 0.5
Ns 40

newmtl vidrio
Kd 0.01 0.02 0.03
Ks 0.9 0.9 0.9
Ns 90
Ke 0.012 0.03 0.045

newmtl filo
Kd 0.80 0.60 1.00
Ks 0.5 0.5 0.5
Ns 32
Ke 0.75 0.50 1.00

newmtl pilar
Kd 0.27 0.23 0.37
Ks 0.7 0.7 0.75
Ns 60

newmtl dorado
Kd 0.95 0.75 0.30
Ks 0.9 0.8 0.5
Ns 64

newmtl boton
Kd 0.20 0.20 0.24
Ks 0.4 0.4 0.4
Ns 30

newmtl metal
Kd 0.55 0.55 0.60
Ks 0.9 0.9 0.9
Ns 80

newmtl led_verde
Kd 0.2 0.9 0.4
Ke 0.2 0.9 0.4

newmtl led_cian
Kd 0.2 0.8 1.0
Ke 0.2 0.8 1.0

newmtl led_ambar
Kd 1.0 0.6 0.1
Ke 1.0 0.6 0.1

newmtl led_rojo
Kd 1.0 0.15 0.15
Ke 1.0 0.15 0.15
"""
m.write("cabina", cabecera, MTL)


# ---------- la palanca de mando ----------
# Origen en el pivote (abajo), Y arriba, mirando a -Z. Una bota de hule, el eje y el mango con su
# boton rojo arriba y el gatillo al frente.
p = Modelo()
partes = {}
m = p


def caja(nombre, mat, x0, x1, y0, y1, z0, z1, arriba_menos=0.0):
    """Una caja; con arriba_menos la tapa de arriba se angosta (como una bota de hule)."""
    a = arriba_menos
    abajo = [(x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1)]
    tapa = [(x0 + a, y1, z0 + a), (x1 - a, y1, z0 + a), (x1 - a, y1, z1 - a), (x0 + a, y1, z1 - a)]
    centro = ((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2)
    P = parte(nombre, mat)
    P.quad(tapa, centro)
    P.quad(abajo, centro)
    for e in range(4):
        P.quad([abajo[e], abajo[(e + 1) % 4], tapa[(e + 1) % 4], tapa[e]], centro)


caja("bota", "goma", -0.06, 0.06, 0.0, 0.07, -0.06, 0.06, arriba_menos=0.04)
caja("eje", "metal", -0.012, 0.012, 0.07, 0.24, -0.012, 0.012)
caja("mango", "goma", -0.028, 0.028, 0.22, 0.36, -0.032, 0.026, arriba_menos=0.004)
caja("mango_dedos", "goma_clara", -0.030, 0.030, 0.25, 0.33, -0.036, -0.030)
caja("boton_rojo", "boton_rojo", -0.012, 0.012, 0.36, 0.372, -0.012, 0.012)
caja("gatillo", "metal", -0.010, 0.010, 0.27, 0.30, -0.050, -0.036)
p.write("cabina_palanca", "# La palanca de mando de la cabina: origen en el pivote (abajo), Y arriba, mira a -Z.\n", """# Materiales de la palanca

newmtl goma
Kd 0.20 0.17 0.27
Ks 0.3 0.3 0.3
Ns 16

newmtl goma_clara
Kd 0.42 0.34 0.58
Ks 0.3 0.3 0.3
Ns 16

newmtl metal
Kd 0.55 0.55 0.60
Ks 0.9 0.9 0.9
Ns 80

newmtl boton_rojo
Kd 1.0 0.2 0.2
Ks 0.5 0.5 0.5
Ns 32
Ke 0.8 0.1 0.1
""")
