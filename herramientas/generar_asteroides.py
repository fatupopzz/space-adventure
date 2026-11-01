# Genera los asteroides y las cuatro lunas de Galileo (para Jupiter Asteroids y el espacio):
#   modelos/asteroide_1.obj, asteroide_2.obj, asteroide_3.obj: rocas voxel con crateres
#     (el juego las pinta de rojo para las calientes y de oro para las que traen un dato).
#   modelos/io.obj, europa.obj, ganimedes.obj, calisto.obj: las lunas de Jupiter (los comodines).
# Uso: python3 herramientas/generar_asteroides.py
# Todas centradas en el origen, de radio ~1 (el juego las escala).

import math
from modelo3d import *


def ruido(i, j, k, semilla):
    """Un ruido de valor 3D suave (interpolado entre una rejilla de 4 voxels)."""
    def h(a, b, c):
        n = (a * 73856093) ^ (b * 19349663) ^ (c * 83492791) ^ (semilla * 2654435761)
        n = (n ^ (n >> 13)) * 1274126177
        return ((n ^ (n >> 16)) & 0xFFFF) / 65535.0
    x, y, z = i / 4.0, j / 4.0, k / 4.0
    x0, y0, z0 = math.floor(x), math.floor(y), math.floor(z)
    fx, fy, fz = x - x0, y - y0, z - z0
    s = lambda t: t * t * (3 - 2 * t)
    fx, fy, fz = s(fx), s(fy), s(fz)
    total = 0.0
    for dx in (0, 1):
        for dy in (0, 1):
            for dz in (0, 1):
                w = (fx if dx else 1 - fx) * (fy if dy else 1 - fy) * (fz if dz else 1 - fz)
                total += w * h(x0 + dx, y0 + dy, z0 + dz)
    return total


def roca(nombre, semilla, r=7.5, aplastada=(1.0, 0.8, 1.1)):
    u = 1.0 / r
    vx = Voxels(u)
    crateres = []
    for c in range(4):
        a = semilla * 1.7 + c * 2.1
        b = semilla * 0.9 + c * 1.3
        crateres.append((math.cos(a) * math.cos(b), math.sin(b), math.sin(a) * math.cos(b), 0.35 + 0.1 * (c % 2)))
    R = int(r) + 3
    for i in range(-R, R + 1):
        for j in range(-R, R + 1):
            for k in range(-R, R + 1):
                x, y, z = (i + 0.5) / aplastada[0], (j + 0.5) / aplastada[1], (k + 0.5) / aplastada[2]
                d = math.sqrt(x * x + y * y + z * z)
                borde = r * (0.78 + 0.45 * ruido(i, j, k, semilla))
                if d > borde:
                    continue
                n = (x / max(d, 0.01), y / max(d, 0.01), z / max(d, 0.01))
                mat = "roca"
                for (cx, cy, cz, tam) in crateres:
                    cerca = math.acos(max(-1.0, min(1.0, n[0] * cx + n[1] * cy + n[2] * cz)))
                    if cerca < tam and d > borde - 1.6:
                        mat = "crater"
                if mat == "roca" and ruido(i * 3, j * 3, k * 3, semilla + 5) > 0.72:
                    mat = "roca_clara"
                vx.put(i, j, k, "roca" if mat == "roca" else mat, mat)
    m = Modelo()
    vx.mesh(m)
    mtl = """# Materiales del asteroide

newmtl roca
Kd 0.46 0.42 0.40
Ks 0.08 0.08 0.08
Ns 6

newmtl roca_clara
Kd 0.62 0.58 0.54
Ks 0.08 0.08 0.08
Ns 6

newmtl crater
Kd 0.28 0.25 0.24
Ks 0.05 0.05 0.05
Ns 4
"""
    m.write(nombre, f"# {nombre}: una roca voxel de radio ~1 con crateres (el juego la recolorea).\n", mtl)


def luna(nombre, colores, pintar, r=7.0):
    u = 1.0 / r
    vx = Voxels(u)
    R = int(r) + 1
    for i in range(-R, R + 1):
        for j in range(-R, R + 1):
            for k in range(-R, R + 1):
                d = math.sqrt((i + 0.5) ** 2 + (j + 0.5) ** 2 + (k + 0.5) ** 2)
                if d > r:
                    continue
                mat = pintar(i, j, k, d)
                vx.put(i, j, k, mat, mat)
    m = Modelo()
    vx.mesh(m)
    mtl = "# Materiales de la luna\n\n" + "".join(
        f"newmtl {n}\nKd {c[0]:.2f} {c[1]:.2f} {c[2]:.2f}\nKs 0.1 0.1 0.1\nNs 8\n" + (f"Ke {e[0]:.2f} {e[1]:.2f} {e[2]:.2f}\n" if e else "") + "\n"
        for n, (c, e) in colores.items())
    m.write(nombre, f"# {nombre}: luna de Galileo, voxel de radio ~1.\n", mtl)


for n, semilla, apl in (("asteroide_1", 3, (1.0, 0.8, 1.15)), ("asteroide_2", 11, (1.2, 0.9, 0.85)), ("asteroide_3", 27, (0.9, 1.0, 1.0))):
    roca(n, semilla, aplastada=apl)

# Io: amarilla y naranja, con volcanes (puntos rojos que brillan).
luna("io", {"azufre": ((0.95, 0.85, 0.30), None), "naranja": ((0.90, 0.55, 0.18), None), "volcan": ((1.0, 0.25, 0.10), (0.9, 0.2, 0.05))},
     lambda i, j, k, d: "volcan" if ruido(i * 2, j * 2, k * 2, 9) > 0.80 else ("naranja" if ruido(i, j, k, 4) > 0.55 else "azufre"))
# Europa: hielo blanco con rayas cafes cruzadas.
luna("europa", {"hielo": ((0.92, 0.92, 0.96), None), "grieta": ((0.62, 0.38, 0.24), None)},
     lambda i, j, k, d: "grieta" if abs(math.sin(i * 0.9 + j * 0.5) + math.sin(k * 0.8 - j * 0.6)) < 0.25 else "hielo")
# Ganimedes: la mas grande, gris y cafe a manchones.
luna("ganimedes", {"gris": ((0.58, 0.56, 0.54), None), "cafe": ((0.40, 0.33, 0.28), None), "brillo": ((0.80, 0.80, 0.78), None)},
     lambda i, j, k, d: "cafe" if ruido(i, j, k, 21) > 0.55 else ("brillo" if ruido(i * 3, j * 3, k * 3, 2) > 0.8 else "gris"))
# Calisto: oscura y llena de crateres claros.
luna("calisto", {"oscuro": ((0.26, 0.23, 0.21), None), "crater_claro": ((0.72, 0.70, 0.66), None)},
     lambda i, j, k, d: "crater_claro" if ruido(i * 3, j * 3, k * 3, 33) > 0.76 else "oscuro")
