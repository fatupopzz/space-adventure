# Genera los modelos de las REFERENCIAS del sistema solar (homenajes; todo voxel y propio):
#   modelos/rocky.obj:      Rocky, el ingeniero eridiano de Project Hail Mary: un caparazon de roca
#                           en forma de pentagono, sin ojos, cinco patas con su rodilla y manos de
#                           tres dedos. Habla con acordes.
#   modelos/blip_a.obj:     la Blip-A, su nave: casco de xenonita, el motor de astrofago (brilla en
#                           rojo infrarrojo) y la burbuja transparente donde vive Rocky.
#   modelos/endurance.obj:  la Endurance de Interstellar: un anillo de doce modulos alrededor del
#                           modulo de mando, con sus rayos. Gira alrededor de Y.
#   modelos/monolito.obj:   el monolito de 2001: Odisea del espacio. Negro brillante, de 1 x 4 x 9
#                           (los cuadrados de 1, 2 y 3).
# Uso: python3 herramientas/generar_referencias.py

import math
from modelo3d import *


def mtl(*materiales):
    out = ["# Materiales"]
    for nombre, kd, *extra in materiales:
        ke = extra[0] if extra else None
        ks, ns = (extra[1], extra[2]) if len(extra) > 2 else (0.15, 16)
        d = extra[3] if len(extra) > 3 else None
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), f"Ks {ks:.2f} {ks:.2f} {ks:.2f}", f"Ns {ns}"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
        if d is not None:
            out.append(f"d {d:.2f}")
    return "\n".join(out) + "\n"


def hash3(i, j, k, s):
    n = (i * 73856093) ^ (j * 19349663) ^ (k * 83492791) ^ (s * 2654435761)
    n = (n ^ (n >> 13)) * 1274126177
    return ((n ^ (n >> 16)) & 0xFFFF) / 65535.0


def escribir(m, nombre, comentario, materiales):
    m.write(nombre, f"# {comentario}\n", mtl(*materiales))


# ---------- Rocky ----------
def rocky():
    U = 1.0 / 16
    vx = Voxels(U)
    # El caparazon: un pentagono que se va cerrando hacia arriba (una roca), con vetas.
    def en_pentagono(x, z, r):
        a = math.atan2(z, x)
        sector = 2 * math.pi / 5
        b = (a % sector) - sector / 2
        return math.hypot(x, z) * math.cos(b) <= r * math.cos(sector / 2)
    for j in range(4, 9):
        r = 5.2 - (j - 4) * 0.75
        for i in range(-7, 7):
            for k in range(-7, 7):
                if en_pentagono(i + 0.5, k + 0.5, r):
                    veta = hash3(i // 2, j, k // 2, 7) > 0.72
                    m_ = "roca_oscura" if veta else "roca"
                    vx.put(i, j, k, m_, m_)
    # El respiradero de arriba (no tiene ojos: oye con todo el cuerpo).
    vx.put(0, 9, 0, "respiradero", "respiradero")
    vx.put(-1, 9, 0, "respiradero", "respiradero")
    # Cinco patas: salen del caparazon, suben a la rodilla y bajan al piso, con mano de tres dedos.
    for n in range(5):
        a = n * 2 * math.pi / 5 + 0.3
        c, s = math.cos(a), math.sin(a)
        puntos = []
        for t in range(0, 21):
            u = t / 20
            if u < 0.4:
                r, y = 4.5 + u / 0.4 * 3.5, 5.5 + u / 0.4 * 3.0
            else:
                v = (u - 0.4) / 0.6
                r, y = 8.0 + v * 2.5, 8.5 - v * 8.5
            puntos.append((r * c, y, r * s))
        for (x, y, z) in puntos:
            vx.put(math.floor(x), math.floor(y), math.floor(z), "pata", "pata")
            vx.put(math.floor(x), math.floor(y) + 1, math.floor(z), "pata", "pata")
        # La mano: tres dedos cortitos en el piso.
        x, z = 10.5 * c, 10.5 * s
        for dedo in (-1, 0, 1):
            dx, dz = -s * dedo, c * dedo
            vx.put(math.floor(x + dx + c), 0, math.floor(z + dz + s), "mano", "mano")
    m = Modelo()
    vx.mesh(m)
    escribir(m, "rocky", "Rocky (Project Hail Mary): caparazon de roca, cinco patas, sin ojos. Origen en el piso, centro del cuerpo.",
             [("roca", (0.50, 0.44, 0.40)), ("roca_oscura", (0.34, 0.30, 0.28)), ("respiradero", (0.15, 0.12, 0.12)),
              ("pata", (0.42, 0.37, 0.34)), ("mano", (0.30, 0.27, 0.25))])


# ---------- La Blip-A ----------
def blip_a():
    U = 1.0 / 4
    vx = Voxels(U)
    # El casco: largo (hacia -Z va la nariz), de seccion octagonal, que se angosta adelante.
    for k in range(-16, 14):
        largo = (k + 16) / 30
        radio = 3.2 if k > -10 else 3.2 * (0.45 + 0.55 * (k + 16) / 6)
        for i in range(-5, 5):
            for j in range(-5, 5):
                x, y = i + 0.5, j + 0.5
                if abs(x) + abs(y) <= radio * 1.35 and max(abs(x), abs(y)) <= radio:
                    panel = (k % 5 == 0) or hash3(i, j, k // 3, 3) > 0.9
                    m_ = "xenonita_clara" if panel else "xenonita"
                    vx.put(i, j, k, m_, m_)
    # El anillo de los motores de astrofago, atras, que brilla en infrarrojo.
    for i in range(-5, 5):
        for j in range(-5, 5):
            r = math.hypot(i + 0.5, j + 0.5)
            if 1.2 <= r <= 3.6:
                vx.put(i, j, 14, "motor", "astrofago")
                if r <= 2.4:
                    vx.put(i, j, 15, "motor", "astrofago")
    # Las antenas y los tanques de los lados.
    for lado in (-1, 1):
        for k in range(-6, 6):
            vx.put(lado * 6 if lado > 0 else -7, 0, k, "tanques", "tanque")
            vx.put(lado * 6 if lado > 0 else -7, -1, k, "tanques", "tanque")
    for j in range(5, 9):
        vx.put(0, j, -4, "xenonita_clara", "xenonita_clara")
    # El tunel a la burbuja (a la derecha, +X) y la burbuja de xenonita transparente.
    for i in range(5, 9):
        vx.put(i, 2, 2, "xenonita_clara", "xenonita_clara")
        vx.put(i, 2, 3, "xenonita_clara", "xenonita_clara")
    m = Modelo()
    vx.mesh(m)
    # La burbuja: una esfera de voxels hueca (solo la cascara), aparte para que sea transparente.
    burbuja = Voxels(U)
    cx, cy, cz = 11.0, 2.5, 2.5
    for i in range(6, 17):
        for j in range(-3, 9):
            for k in range(-3, 9):
                r = math.sqrt((i + 0.5 - cx) ** 2 + (j + 0.5 - cy) ** 2 + (k + 0.5 - cz) ** 2)
                if 4.3 <= r <= 5.2:
                    burbuja.put(i, j, k, "burbuja", "vidrio_xenonita")
    burbuja.mesh(m)
    escribir(m, "blip_a", f"La Blip-A (Project Hail Mary). La nariz hacia -Z; la burbuja de Rocky a la derecha, con centro en {cx * U:.3f} {cy * U:.3f} {cz * U:.3f}.",
             [("xenonita", (0.30, 0.34, 0.31), None, 0.35, 32), ("xenonita_clara", (0.48, 0.52, 0.47), None, 0.35, 32),
              ("astrofago", (0.55, 0.06, 0.08), (0.95, 0.12, 0.15)), ("tanque", (0.62, 0.60, 0.55)),
              ("vidrio_xenonita", (0.45, 0.75, 0.68), (0.02, 0.05, 0.04), 0.5, 64, 0.18)])


# ---------- La Endurance ----------
def endurance():
    m = Modelo()
    radio = 2.6
    # Los doce modulos del anillo, cada uno girado a su lugar (alrededor de Y).
    for n in range(12):
        a = n * 2 * math.pi / 12
        modulo = Voxels(1.0 / 10)
        modulo.box("blanco", "blanco", -6, 5, -2, 1, -3, 2)
        modulo.box("gris", "gris", -6, 5, -3, -3, -3, 2)
        # Paneles solares en uno de cada tres; ventanitas con luz en los demas.
        if n % 3 == 0:
            modulo.box("paneles", "panel_solar", -5, 4, 2, 2, -2, 1)
        else:
            for i in (-4, -1, 2):
                modulo.put(i, 0, 3, "luces", "luz")
        modulo.mesh(m, transform=rot_y(a + math.pi / 2, (radio * math.sin(a), 0.0, radio * math.cos(a))))
    # Los conectores entre modulos (el anillo cerrado).
    for n in range(12):
        a = (n + 0.5) * 2 * math.pi / 12
        union = Voxels(1.0 / 10)
        union.box("gris", "gris", -1, 0, -1, 0, -1, 0)
        union.mesh(m, transform=rot_y(a + math.pi / 2, (radio * 0.985 * math.sin(a), 0.0, radio * 0.985 * math.cos(a))))
    # El modulo de mando al centro y cuatro rayos.
    centro = Voxels(1.0 / 10)
    centro.box("blanco", "blanco", -4, 3, -3, 2, -4, 3)
    centro.box("gris", "gris", -2, 1, 3, 4, -2, 1)
    for i in range(-2, 2):
        centro.put(i, 0, -5, "luces", "luz")
    for (x0, x1, z0, z1) in ((4, 23, -1, 0), (-24, -5, -1, 0), (-1, 0, 4, 23), (-1, 0, -24, -5)):
        centro.box("gris", "gris", x0, x1, -1, 0, z0, z1)
    centro.mesh(m)
    escribir(m, "endurance", "La Endurance (Interstellar): anillo de doce modulos que gira alrededor de Y, con el mando al centro.",
             [("blanco", (0.88, 0.88, 0.86), None, 0.3, 24), ("gris", (0.46, 0.47, 0.52), None, 0.3, 24),
              ("panel_solar", (0.12, 0.17, 0.42), (0.02, 0.03, 0.08), 0.8, 64), ("luz", (1.0, 0.9, 0.65), (0.9, 0.8, 0.55))])


# ---------- El monolito ----------
def monolito():
    U = 1.0 / 8
    vx = Voxels(U)
    vx.box("monolito", "monolito", -1, 0, 0, 17, -4, 3)
    m = Modelo()
    vx.mesh(m)
    escribir(m, "monolito", "El monolito (2001: Odisea del espacio): 1 x 4 x 9, negro brillante. Origen al centro de la base.",
             [("monolito", (0.02, 0.02, 0.03), None, 0.9, 160)])


rocky()
blip_a()
endurance()
monolito()
