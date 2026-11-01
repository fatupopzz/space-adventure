# Genera los modelos que se ponen sobre el PLANETA CHAPIN (Guatemala hecha planeta). El planeta
# en si no es un .obj: lo arma el juego (voxels con relieve de ruido, ver src/chapin.rs). Estos
# son los objetos de su superficie, todos voxel y propios:
#   modelos/chapin_ceiba.obj:  la ceiba, el arbol nacional: tronco gris y derecho con raices de
#                              contrafuerte y la copa ancha y plana, como sombrilla.
#   modelos/chapin_roca.obj:   una roca volcanica (basalto oscuro con vetas de piedra pomez).
#   modelos/chapin_casita.obj: una casita de Antigua: paredes de color, puerta de madera, ventana
#                              con reja, techo de teja y un balconcito. El juego le cambia el color
#                              a la pared ("pared") para que la calle salga de colores.
#   modelos/chapin_tikal.obj:  el Templo I de Tikal (el del Gran Jaguar): nueve cuerpos
#                              escalonados, la escalinata al frente y el templo con su cresteria.
#   modelos/chapin_arco.obj:   el Arco de Santa Catalina de Antigua, amarillo, con su reloj.
# Origen: al centro de la base, en el suelo (y = 0); arriba es +Y y el frente mira a +Z. El juego
# los para sobre la superficie curva, con su "arriba" hacia afuera del planeta.
# Uso: python3 herramientas/generar_chapin.py

import math
from modelo3d import *


def hash3(i, j, k, s):
    n = (i * 73856093) ^ (j * 19349663) ^ (k * 83492791) ^ (s * 2654435761)
    n = (n ^ (n >> 13)) * 1274126177
    return ((n ^ (n >> 16)) & 0xFFFF) / 65535.0


def mtl(*materiales):
    out = ["# Materiales"]
    for nombre, kd, *extra in materiales:
        ke = extra[0] if extra else None
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), "Ks 0.06 0.06 0.06", "Ns 8"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
    return "\n".join(out) + "\n"


def escribir(vx, nombre, comentario, materiales):
    m = Modelo()
    vx.mesh(m)
    m.write(nombre, f"# {comentario}\n# Origen al centro de la base; arriba +Y, frente +Z.\n", mtl(*materiales))


# ---------- la ceiba ----------
def ceiba():
    U = 1.0 / 18
    vx = Voxels(U, (-0.0, 0.0, -0.0))
    # El tronco: 2x2, derecho y alto; abajo, las raices de contrafuerte en cruz.
    vx.box("tronco", "tronco", -1, 0, 0, 12, -1, 0)
    for (dx, dz) in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        for paso in range(1, 4):
            alto = 4 - paso
            x = (0 if dx > 0 else -1) + dx * paso
            z = (0 if dz > 0 else -1) + dz * paso
            for j in range(0, alto):
                vx.put(x if dx else -1, j, z if dz else -1, "tronco", "tronco")
                if not dx:
                    vx.put(0, j, z, "tronco", "tronco")
                if not dz:
                    vx.put(x, j, 0, "tronco", "tronco")
    # Unas ramas que abren hacia la copa.
    for (dx, dz) in ((1, 1), (-1, 1), (1, -1), (-1, -1)):
        for paso in range(1, 4):
            vx.put((0 if dx > 0 else -1) + dx * paso, 11 + paso // 2, (0 if dz > 0 else -1) + dz * paso, "tronco", "tronco")
    # La copa: una sombrilla ancha y plana, en tres capas, con huecos de ruido en la orilla.
    for j, radio in ((12, 6.5), (13, 8.0), (14, 7.0), (15, 4.5)):
        R = int(radio) + 1
        for i in range(-R - 1, R + 1):
            for k in range(-R - 1, R + 1):
                d = math.hypot(i + 0.5, k + 0.5)
                # Solo unos cuantos huecos en la orilla (cada hueco son caras de mas).
                if d > radio or (d > radio - 1.0 and hash3(i // 2, j, k // 2, 3) < 0.25):
                    continue
                if (i, j, k) in vx.v:
                    continue
                # El verde claro, solo arriba (donde le pega el sol): menos caras sueltas.
                clara = j == 15
                vx.put(i, j, k, "copa_clara" if clara else "copa", "hoja_clara" if clara else "hoja")
    escribir(vx, "chapin_ceiba", "La ceiba, arbol nacional de Guatemala.",
             [("tronco", (0.68, 0.64, 0.58)), ("hoja", (0.20, 0.58, 0.26)), ("hoja_clara", (0.40, 0.78, 0.32))])


# ---------- la roca volcanica ----------
def roca():
    U = 1.0 / 14
    vx = Voxels(U)
    for i in range(-6, 6):
        for j in range(0, 7):
            for k in range(-5, 5):
                x, y, z = (i + 0.5) / 5.5, (j + 0.5) / 5.0, (k + 0.5) / 4.6
                d = math.sqrt(x * x + y * y * 1.2 + z * z)
                if d > 0.82 + 0.35 * hash3(i // 2, j // 2, k // 2, 5):
                    continue
                pomez = hash3(i, j, k, 11) > 0.86
                vx.put(i, j, k, "pomez" if pomez else "basalto", "pomez" if pomez else "basalto")
    escribir(vx, "chapin_roca", "Roca volcanica: basalto oscuro con vetas de piedra pomez.",
             [("basalto", (0.36, 0.32, 0.42)), ("pomez", (0.72, 0.68, 0.64))])


# ---------- la casita de Antigua ----------
def casita():
    U = 1.0 / 20
    vx = Voxels(U)
    # Paredes gruesas (de adobe), de 10 de ancho, 8 de fondo y 6 de alto.
    vx.box("pared", "pared", -5, 4, 0, 5, -4, 3)
    vx.box("zocalo", "zocalo", -5, 4, 0, 0, -4, 3)
    # La puerta de madera, al frente (+Z), y la ventana con reja.
    vx.box("puerta", "madera", -3, -2, 1, 3, 3, 3)
    vx.box("ventana", "vidrio", 1, 3, 2, 3, 3, 3)
    for i in (1, 3):
        vx.box("reja", "hierro", i, i, 2, 3, 4, 4)
    vx.box("reja", "hierro", 1, 3, 4, 4, 4, 4)
    # El techo de teja: a dos aguas, que sobresale un poco.
    for capa in range(0, 4):
        vx.box("techo", "teja", -6, 5, 6 + capa, 6 + capa, -5 + capa, 4 - capa)
    # Un balconcito de madera arriba de la puerta.
    vx.box("balcon", "madera", -4, -1, 4, 4, 4, 4)
    escribir(vx, "chapin_casita", "Casita de Antigua Guatemala (el juego le cambia el color a la pared).",
             [("pared", (0.95, 0.75, 0.25)), ("zocalo", (0.55, 0.20, 0.15)), ("madera", (0.40, 0.22, 0.10)),
              ("vidrio", (0.20, 0.30, 0.38)), ("hierro", (0.10, 0.10, 0.12)), ("teja", (0.72, 0.30, 0.16))])


# ---------- el Templo I de Tikal ----------
def tikal():
    U = 1.0 / 22
    vx = Voxels(U)
    # Nueve cuerpos escalonados, cada uno mas chico y con su moldura.
    ancho, fondo = 11, 9
    y = 0
    for cuerpo in range(9):
        a, f = ancho - cuerpo, fondo - cuerpo
        alto = 2 if cuerpo < 8 else 1
        mat = "piedra" if cuerpo % 2 == 0 else "piedra_oscura"
        vx.box(f"cuerpo_{mat}", mat, -a, a - 1, y, y + alto - 1, -f, f - 1)
        y += alto
    tope = y
    # La escalinata al frente (+Z): una rampa de escalones angosta que sube hasta arriba.
    for j in range(0, tope):
        z = fondo - 1 - (j * (fondo - 1)) // tope + 1
        vx.box("escalinata", "escalon", -2, 1, j, j, -fondo, z)
    # El templo de arriba: tres cuartos con su puerta oscura al frente.
    vx.box("templo", "piedra", -3, 2, tope, tope + 4, -3, 2)
    vx.box("puerta_templo", "sombra", -1, 0, tope, tope + 2, 2, 2)
    # La cresteria (el peine de piedra del techo), alta y calada.
    for j in range(tope + 5, tope + 10):
        for i in range(-3, 3):
            if j > tope + 6 and (i + j) % 3 == 0:
                continue
            vx.put(i, j, -1, "cresteria", "piedra_clara")
    # Musgo de la selva en algunas esquinas.
    for (i, j, k) in list(vx.v):
        if vx.v[(i, j, k)][1] in ("piedra", "piedra_oscura") and hash3(i, j, k, 21) < 0.08:
            vx.v[(i, j, k)] = ("musgo", "musgo")
    escribir(vx, "chapin_tikal", "Templo I de Tikal (el del Gran Jaguar), con su escalinata y su cresteria.",
             [("piedra", (0.76, 0.72, 0.62)), ("piedra_oscura", (0.64, 0.60, 0.52)), ("piedra_clara", (0.85, 0.82, 0.74)),
              ("escalon", (0.82, 0.78, 0.68)), ("sombra", (0.06, 0.05, 0.05)), ("musgo", (0.36, 0.70, 0.30))])


# ---------- el Arco de Santa Catalina ----------
def arco():
    U = 1.0 / 20
    vx = Voxels(U)
    # Dos pilares y el arco de medio punto entre ellos; encima, el pasillo y la torrecita del reloj.
    for i in range(-8, 8):
        for j in range(0, 12):
            for k in range(-2, 2):
                hueco = abs(i + 0.5) < 4.5 and (j < 6 or math.hypot(i + 0.5, j - 6) < 4.5)
                if not hueco:
                    vx.put(i, j, k, "arco", "amarillo")
    vx.box("cornisa", "blanco", -9, 8, 12, 12, -3, 2)
    vx.box("torre", "amarillo", -2, 1, 13, 17, -1, 0)
    vx.box("reloj", "reloj", -1, 0, 15, 16, 1, 1)
    vx.box("remate", "blanco", -2, 1, 18, 18, -1, 0)
    # Las ventanitas del pasillo de arriba (por donde cruzaban las monjas sin salir a la calle).
    for i in (-6, -3, 2, 5):
        vx.box("ventanita", "sombra", i, i, 9, 10, 2, 2)
    escribir(vx, "chapin_arco", "Arco de Santa Catalina, Antigua Guatemala.",
             [("amarillo", (0.98, 0.78, 0.22)), ("blanco", (0.95, 0.93, 0.88)), ("reloj", (0.95, 0.95, 0.90), (0.25, 0.25, 0.22)),
              ("sombra", (0.10, 0.08, 0.08))])


# ---------- las versiones de lejos (nivel de detalle): pocas cajas, la misma silueta ----------
def ceiba_lejos():
    U = 1.0 / 18
    vx = Voxels(U)
    vx.box("tronco", "tronco", -1, 0, 0, 11, -1, 0)
    vx.box("copa", "hoja", -7, 6, 12, 13, -7, 6)
    vx.box("copa", "hoja", -5, 4, 14, 14, -5, 4)
    vx.box("copa_clara", "hoja_clara", -3, 2, 15, 15, -3, 2)
    escribir(vx, "chapin_ceiba_lejos", "La ceiba vista de lejos (nivel de detalle bajo).",
             [("tronco", (0.68, 0.64, 0.58)), ("hoja", (0.20, 0.58, 0.26)), ("hoja_clara", (0.40, 0.78, 0.32))])


def roca_lejos():
    U = 1.0 / 14
    vx = Voxels(U)
    vx.box("basalto", "basalto", -5, 4, 0, 3, -4, 3)
    vx.box("basalto", "basalto", -3, 2, 4, 5, -2, 2)
    escribir(vx, "chapin_roca_lejos", "La roca volcanica vista de lejos (nivel de detalle bajo).",
             [("basalto", (0.36, 0.32, 0.42))])


ceiba()
ceiba_lejos()
roca()
roca_lejos()
casita()
tikal()
arco()
