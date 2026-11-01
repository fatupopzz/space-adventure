# Genera gladas.obj: GLADAS, la IA de la nave. Mezcla de dos robots famosos (fan art propio):
#   - de Astro Bot: la cabeza blanca redondita, el visor negro con ojos LED que cambian de
#     expresion, y las orejeras con su luz azul;
#   - de GLaDOS: la capsula alargada hacia atras y el brazo mecanico articulado del que cuelga,
#     y los ojos en amarillo.
# El origen esta en el centro de la cabeza (para girarla facil); mira hacia -Z; el brazo sube.
# Uso: python3 herramientas/generar_gladas.py

import math
from modelo3d import *

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
# La capsula de atras (GLaDOS), con una junta oscura donde se une a la cabeza
elipsoide(vx, "capsula", "gris", (0.0, 0.5, 11.0), (5.5, 5.0, 8.0))
for i in range(-6, 6):
    for j in range(-5, 7):
        if (i + 0.5) ** 2 / 30 + (j - 0.5) ** 2 / 25 <= 1:
            vx.put(i, j, 8, "junta", "negro")
# La cabeza (Astro Bot): blanca y redonda
caja_redonda(vx, "cabeza", "blanco", -8, 7, -6, 6, -7, 7, 4.0)
# El visor negro, al frente, con esquinas redondas
for i in range(-6, 6):
    for j in range(-4, 5):
        esquina = (i in (-6, 5)) and (j in (-4, 4))
        if not esquina:
            vx.put(i, j, -8, "visor", "visor")
# Las orejeras con su aro de luz
for lado in (1, -1):
    i = 8 if lado > 0 else -9
    caja_redonda(vx, "orejeras", "gris", i, i, -3, 3, -3, 3, 1.2)
    for j, k in ((-2, -2), (-2, 1), (1, -2), (1, 1), (-2, -1), (-2, 0), (1, -1), (1, 0), (-1, -2), (0, -2), (-1, 1), (0, 1)):
        vx.put(i + lado, j, k, "luz_orejas", "led_azul")
# El brazo mecanico (GLaDOS): sube desde la capsula, con articulaciones
for j in range(5, 40):
    articulacion = j % 7 == 0
    r = 2 if articulacion else 1
    for i in range(-r, r):
        for k in range(12 - r, 12 + r):
            vx.put(i, j, k, "articulaciones" if articulacion else "brazo", "gris" if articulacion else "negro")
m = Modelo()
vx.mesh(m)
# Los ojos LED: las mismas expresiones de todos, pero en luz amarilla sobre el visor
CARAS_LED = {emo: [(pieza, "led" if mat in ("ojo", "boca") else mat, rects) for pieza, mat, rects in piezas] for emo, piezas in CARAS.items()}
expresiones(m, z_frente=-8 * U, px=0.019, x_centro=0.0, y_base=-0.15, caras=CARAS_LED)
m.write("gladas", """# GLADAS, la IA de la nave (voxel de 0.04): cabeza de Astro Bot con visor y ojos LED (las caras de
# siempre, en luz amarilla), capsula y brazo mecanico de GLaDOS. Fan art propio. Origen en el centro
# de la cabeza, mira hacia -Z, el brazo sube por +Y.
""", """# Materiales

newmtl blanco
Kd 0.95 0.95 0.97
Ks 0.5 0.5 0.5
Ns 40

newmtl gris
Kd 0.55 0.57 0.62
Ks 0.3 0.3 0.3
Ns 24

newmtl negro
Kd 0.10 0.10 0.12
Ks 0.2 0.2 0.2
Ns 16

newmtl visor
Kd 0.04 0.04 0.07
Ks 0.9 0.9 0.9
Ns 80

newmtl led
Kd 1.0 0.80 0.25
Ke 1.0 0.70 0.15

newmtl led_azul
Kd 0.3 0.75 1.0
Ke 0.25 0.65 1.0
""" + MTL_CARA)
