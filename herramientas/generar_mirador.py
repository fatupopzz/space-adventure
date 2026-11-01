# Genera el Mirador: el punto de descanso en el punto L2 de la Tierra (donde vive el telescopio James Webb).
# Una plataforma flotante con piso de madera, barandal, una banca para Miku, un telescopio, paneles solares,
# una baliza que parpadea y una fogata con una espada clavada (las hogueras de Dark Souls).
# Origen en el centro del piso (arriba del piso en y = 0). La banca mira hacia -Z (hacia la Tierra).
# Uso: python3 herramientas/generar_mirador.py

import math
from modelo3d import *

def mtl(*materiales):
    out = ["# Materiales"]
    for nombre, kd, ke, *d in materiales:
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), "Ks 0.25 0.25 0.25", "Ns 32"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
        if d:
            out.append(f"d {d[0]:.2f}")
    return "\n".join(out) + "\n"

def palo(vx, parte, mat, a, b, grueso=0):
    u = vx.u
    pasos = int(max(abs(b[q] - a[q]) for q in range(3)) / u * 2) + 1
    for s in range(pasos + 1):
        t = s / pasos
        p = [a[q] + (b[q] - a[q]) * t - vx.o[q] for q in range(3)]
        i, j, k = (math.floor(p[q] / u) for q in range(3))
        for di in range(-grueso, grueso + 1):
            for dj in range(-grueso, grueso + 1):
                for dk in range(-grueso, grueso + 1):
                    vx.put(i + di, j + dj, k + dk, parte, mat)

R = 2.4
m = Modelo()
v = Voxels(0.1, origen=(0.0, -0.1, 0.0))
# El piso de tablas y la orilla de metal
for i in range(-25, 25):
    for k in range(-25, 25):
        x, _, z = v.centro(i, 0, k)
        r = math.hypot(x, z)
        if r <= R:
            if r > R - 0.15:
                v.put(i, 0, k, "orilla", "metal_mirador")
            else:
                mat = "madera_mirador" if (k // 2) % 2 == 0 else "madera_clara_mirador"
                v.put(i, 0, k, f"piso_{mat}", mat)
# El casco de abajo: un cono que se afila, con un anillo de luz
for j in range(1, 14):
    r_capa = R * (1 - j / 15) ** 0.8
    for i in range(-25, 25):
        for k in range(-25, 25):
            x, _, z = v.centro(i, 0, k)
            r = math.hypot(x, z)
            if r_capa - 0.25 <= r <= r_capa:
                mat = "luz_cian" if j == 3 else "metal_oscuro_mirador"
                v.put(i, -j, k, "casco_luz" if j == 3 else "casco", mat)
# El barandal: postes y pasamanos
for n in range(16):
    a = n / 16 * 2 * math.pi
    x, z = (R - 0.12) * math.cos(a), (R - 0.12) * math.sin(a)
    for j in range(1, 9):
        v.put(math.floor(x / 0.1), j, math.floor(z / 0.1), "postes", "metal_mirador")
for i in range(-25, 25):
    for k in range(-25, 25):
        x, _, z = v.centro(i, 0, k)
        if abs(math.hypot(x, z) - (R - 0.12)) < 0.06:
            v.put(i, 9, k, "pasamanos", "madera_clara_mirador")
v.mesh(m)

w = Voxels(0.05)
# La banca (mira hacia -Z)
w.box("banca", "madera_mirador", -16, 15, 8, 9, 16, 22)
w.box("banca", "madera_mirador", -16, 15, 10, 18, 23, 24)
for x in (-14, 13):
    w.box("banca_patas", "metal_mirador", x, x + 1, 0, 7, 17, 22)
# La fogata: piedras en circulo, lenos, la espada clavada y el fuego
FX, FZ = 0.0, -0.9
for n in range(10):
    a = n / 10 * 2 * math.pi
    i, k = math.floor((FX + 0.32 * math.cos(a)) / 0.05), math.floor((FZ + 0.32 * math.sin(a)) / 0.05)
    w.box("piedras", "piedra", i - 1, i, 0, 1, k - 1, k)
palo(w, "lenos", "leno", (FX - 0.25, 0.04, FZ - 0.1), (FX + 0.25, 0.04, FZ + 0.1), 0)
palo(w, "lenos", "leno", (FX - 0.1, 0.04, FZ + 0.25), (FX + 0.1, 0.04, FZ - 0.25), 0)
palo(w, "espada", "acero_mirador", (FX + 0.02, 0.0, FZ), (FX - 0.06, 0.7, FZ + 0.06), 0)
palo(w, "espada_guarda", "oro_mirador", (FX - 0.15, 0.62, FZ + 0.05), (FX + 0.05, 0.62, FZ + 0.05), 0)
palo(w, "espada_puno", "leno", (FX - 0.06, 0.65, FZ + 0.06), (FX - 0.08, 0.8, FZ + 0.07), 0)
for j in range(1, 8):
    r = 0.18 * (1 - j / 9)
    for i in range(-5, 5):
        for k in range(-5, 5):
            x, z = (i + 0.5) * 0.05, (k + 0.5) * 0.05
            if math.hypot(x, z) <= r + 0.03:
                w.put(i + math.floor(FX / 0.05), j, k + math.floor(FZ / 0.05), "fuego_mirador", "fuego_mirador")
# El telescopio en su tripie, apuntando al cielo hacia -Z
TX, TZ = 1.5, -0.4
for a in (0.3, 2.4, 4.5):
    palo(w, "tripie_telescopio", "metal_oscuro_mirador", (TX, 1.1, TZ), (TX + 0.35 * math.sin(a), 0.0, TZ + 0.35 * math.cos(a)), 0)
palo(w, "telescopio", "blanco_mirador", (TX + 0.1, 1.05, TZ + 0.35), (TX - 0.15, 1.55, TZ - 0.6), 1)
palo(w, "telescopio_lente", "metal_oscuro_mirador", (TX - 0.15, 1.55, TZ - 0.6), (TX - 0.17, 1.59, TZ - 0.68), 1)
# La baliza en su poste
palo(w, "baliza_poste", "metal_mirador", (-1.7, 0.0, 1.3), (-1.7, 2.3, 1.3), 0)
for i in range(-2, 2):
    for j in range(46, 50):
        for k in range(-2, 2):
            w.put(i + math.floor(-1.7 / 0.05), j, k + math.floor(1.3 / 0.05), "baliza", "luz_baliza")
# Los paneles solares a los lados
for lado in (-1, 1):
    palo(w, "brazos_panel", "metal_mirador", (lado * R, -0.3, 0.0), (lado * (R + 0.8), -0.3, 0.0), 0)
    x0 = lado * (R + 0.8)
    for i in range(0, 30):
        for k in range(-12, 12):
            mat = "marco_panel" if i % 6 == 0 or k % 6 == 0 else "panel_solar"
            w.put(math.floor(x0 / 0.05) + lado * i, -6, k, "panel_" + mat, mat)
w.mesh(m)
m.write("mirador", """# El Mirador (punto de descanso en el L2 de la Tierra): plataforma de 2.4 de radio con piso de tablas, barandal,
# casco abajo con un anillo de luz, banca (mira a -Z) en z ~ 1.0, fogata con espada en (0, -0.9), telescopio,
# baliza en (-1.7, 2.4, 1.3) y paneles solares a los lados. Piso en y = 0.
""", mtl(("madera_mirador", (0.55, 0.36, 0.22), None), ("madera_clara_mirador", (0.68, 0.48, 0.30), None),
         ("metal_mirador", (0.72, 0.74, 0.80), None), ("metal_oscuro_mirador", (0.30, 0.31, 0.36), None),
         ("luz_cian", (0.4, 0.95, 1.0), (0.3, 0.8, 0.9)), ("piedra", (0.45, 0.43, 0.42), None), ("leno", (0.35, 0.22, 0.12), None),
         ("acero_mirador", (0.85, 0.87, 0.92), None), ("oro_mirador", (1.0, 0.78, 0.3), (0.1, 0.07, 0.0)),
         ("fuego_mirador", (1.0, 0.6, 0.2), (1.0, 0.55, 0.15)), ("blanco_mirador", (0.92, 0.92, 0.95), None),
         ("luz_baliza", (1.0, 0.4, 0.7), (0.9, 0.3, 0.6)), ("panel_solar", (0.15, 0.25, 0.60), (0.02, 0.04, 0.1)),
         ("marco_panel", (0.80, 0.82, 0.88), None)))
