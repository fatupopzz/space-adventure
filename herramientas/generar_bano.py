# Genera EL BANO DE DONA CHEPA (Urano): el set y los personajes del juego de administrar el bano
# publico de Urano (estilo Overcooked). Todo voxel y propio.
#   modelos/bano_chepa.obj:   el bano visto desde arriba: piso de azulejos, cuatro cubiculos con
#                             su inodoro y su puerta (puerta_0 a puerta_3, con la bisagra en su
#                             comentario), la caja de Dona Chepa con el letrero de Q1, el estante
#                             (papel, destapacanos y trapeador), los lavamanos con espejo, la
#                             ventana con Urano y la entrada con su letrero.
#   modelos/alien_urano.obj:  un cliente: alien gordito de antenas (el juego le cambia el color a
#                             la "piel"), con caras feliz, urgido y enojado (cara_<emocion>_*).
#   modelos/chepa.obj:        Dona Chepa sentada en su banco: delantal, huipil, mono gris y lentes.
#   modelos/item_papel.obj, item_destapacanos.obj, item_trapeador.obj: lo que se carga en la mano.
# Medidas en metros; Y arriba; el frente del bano (donde esta la camara) es +Z.
# Uso: python3 herramientas/generar_bano.py

import math
from modelo3d import *


def mtl(*materiales):
    out = ["# Materiales"]
    for nombre, kd, *extra in materiales:
        ke = extra[0] if extra else None
        ks = extra[1] if len(extra) > 1 else 0.15
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), f"Ks {ks} {ks} {ks}", "Ns 24"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
    return "\n".join(out) + "\n"


def caja_m(vx, parte, mat, x0, x1, y0, y1, z0, z1):
    """Una caja en metros (rangos), en voxels de vx.u."""
    u = vx.u
    a = lambda v: int(math.floor(v / u + 1e-6))
    b = lambda v: int(math.ceil(v / u - 1e-6)) - 1
    vx.box(parte, mat, a(x0), b(x1), a(y0), b(y1), a(z0), b(z1))


def escribir(vx, nombre, comentario, materiales, comentarios=None):
    m = Modelo()
    vx.mesh(m, comentarios=comentarios)
    m.write(nombre, f"# {comentario}\n", mtl(*materiales))


# =====================================================================
# El bano
# =====================================================================
X0, X1, Z0, Z1 = -5.0, 5.0, -4.0, 4.0
ALTO = 2.6
CUBICULOS = 4
X_CUBICULO, ANCHO_CUBICULO, FONDO_CUBICULO = -4.4, 1.5, 1.6

v = Voxels(0.1)
# El piso: azulejos de 50 cm, blanco y celeste.
for i in range(int(X0 * 2), int(X1 * 2)):
    for k in range(int(Z0 * 2), int(Z1 * 2) + 3):
        mat = "azulejo" if (i + k) % 2 == 0 else "azulejo_celeste"
        caja_m(v, f"piso_{mat}", mat, i * 0.5, i * 0.5 + 0.5, -0.1, 0.0, k * 0.5, k * 0.5 + 0.5)
# Las paredes: el fondo y los lados, con su franja; el frente es bajito (para ver adentro), con la
# entrada a la izquierda.
for (x0, x1, z0, z1) in ((X0 - 0.2, X1 + 0.2, Z0 - 0.2, Z0), (X0 - 0.2, X0, Z0, Z1), (X1, X1 + 0.2, Z0, Z1)):
    caja_m(v, "pared", "pared", x0, x1, 0.0, ALTO, z0, z1)
    caja_m(v, "pared_franja", "franja", x0, x1, 1.1, 1.3, z0, z1)
    caja_m(v, "pared_zocalo", "zocalo", x0, x1, 0.0, 0.2, z0, z1)
caja_m(v, "muro_frente", "pared", -3.0, X1 + 0.2, 0.0, 0.5, Z1, Z1 + 0.2)
caja_m(v, "muro_frente_franja", "franja", -3.0, X1 + 0.2, 0.5, 0.6, Z1, Z1 + 0.2)
# La entrada: dos postes y el letrero encima.
for x in (X0 - 0.2, -3.2):
    caja_m(v, "entrada", "zocalo", x, x + 0.2, 0.0, 2.4, Z1, Z1 + 0.2)
caja_m(v, "letrero", "letrero", X0 - 0.2, -3.0, 2.4, 2.9, Z1, Z1 + 0.2)
# Los cubiculos, contra el fondo: divisiones, el frente con su hueco, el inodoro adentro.
for n in range(CUBICULOS + 1):
    x = X_CUBICULO + n * ANCHO_CUBICULO
    caja_m(v, "cubiculos", "cubiculo", x - 0.05, x + 0.05, 0.1, 2.1, Z0, Z0 + FONDO_CUBICULO)
comentarios = {}
for n in range(CUBICULOS):
    xc = X_CUBICULO + (n + 0.5) * ANCHO_CUBICULO
    zf = Z0 + FONDO_CUBICULO
    # El frente del cubiculo a los lados de la puerta.
    caja_m(v, "cubiculos", "cubiculo", xc - ANCHO_CUBICULO / 2, xc - 0.45, 0.1, 2.1, zf - 0.05, zf + 0.05)
    caja_m(v, "cubiculos", "cubiculo", xc + 0.45, xc + ANCHO_CUBICULO / 2, 0.1, 2.1, zf - 0.05, zf + 0.05)
    caja_m(v, "cubiculos", "cubiculo", xc - 0.45, xc + 0.45, 1.9, 2.1, zf - 0.05, zf + 0.05)
    # La puerta (su propia parte, para que se abra: gira en la bisagra izquierda).
    caja_m(v, f"puerta_{n}", "puerta", xc - 0.45, xc + 0.45, 0.15, 1.9, zf - 0.03, zf + 0.03)
    comentarios[f"puerta_{n}"] = f"bisagra {xc - 0.45:.2f} 0 {zf:.2f}"
    # El inodoro: la taza, el asiento y el tanque.
    caja_m(v, "inodoros", "loza", xc - 0.25, xc + 0.25, 0.0, 0.45, Z0 + 0.35, Z0 + 0.95)
    caja_m(v, "inodoros_asiento", "asiento", xc - 0.25, xc + 0.25, 0.45, 0.5, Z0 + 0.35, Z0 + 0.95)
    caja_m(v, "inodoros", "loza", xc - 0.3, xc + 0.3, 0.45, 1.0, Z0, Z0 + 0.3)
    # El numero arriba de la puerta.
    caja_m(v, f"numero_{n}", "numero", xc - 0.15, xc + 0.15, 2.15, 2.45, zf - 0.05, zf + 0.05)
# La ventana con Urano (acostado, con sus anillos parados), en el fondo a la derecha.
caja_m(v, "ventana_marco", "marco", 2.0, 4.6, 1.0, 2.4, Z0 - 0.05, Z0 + 0.05)
caja_m(v, "ventana_cielo", "cielo", 2.1, 4.5, 1.1, 2.3, Z0 + 0.05, Z0 + 0.1)
for i in range(-5, 6):
    for j in range(-5, 6):
        if i * i + j * j <= 25:
            caja_m(v, "ventana_urano", "urano", 3.3 + i * 0.05, 3.35 + i * 0.05, 1.7 + j * 0.05, 1.75 + j * 0.05, Z0 + 0.1, Z0 + 0.15)
for j in range(-9, 10):
    caja_m(v, "ventana_anillo", "anillo", 3.3 + 0.02 * j, 3.35 + 0.02 * j, 1.7 + j * 0.05, 1.75 + j * 0.05, Z0 + 0.15, Z0 + 0.2)
# La caja de Dona Chepa, junto a la entrada: el mostrador, la alcancia y el letrero de Q1.
caja_m(v, "mostrador", "madera", X0, -3.5, 0.0, 1.0, 1.2, 2.2)
caja_m(v, "mostrador_tapa", "madera_clara", X0, -3.4, 1.0, 1.1, 1.1, 2.3)
caja_m(v, "alcancia", "alcancia", -3.9, -3.6, 1.1, 1.35, 1.5, 1.8)
caja_m(v, "letrero_q1", "letrero_q1", -3.5, -3.45, 0.3, 0.8, 1.35, 2.05)
# El banco de Chepa.
caja_m(v, "banco", "madera", -4.55, -4.15, 0.0, 0.55, 0.35, 0.85)
# El estante, en la pared derecha: papel, destapacanos y trapeador, cada uno en su lugar.
# (abierto hacia el bano: fondo, costados, techo, repisa de abajo y divisiones; lo de adentro se ve)
caja_m(v, "estante", "madera", 4.85, 5.0, 0.0, 1.9, -1.55, 1.55)
caja_m(v, "estante", "madera", 4.45, 5.0, 1.8, 1.9, -1.55, 1.55)
caja_m(v, "estante", "madera", 4.45, 5.0, 0.0, 0.15, -1.55, 1.55)
for z in (-1.55, -0.5, 0.5, 1.5):
    caja_m(v, "estante", "madera", 4.45, 5.0, 0.0, 1.9, z - 0.05, z + 0.05)
caja_m(v, "estante_hueco", "sombra", 4.82, 4.85, 0.15, 1.8, -1.5, 1.5)
for (z, y) in ((-1.15, 0.15), (-0.85, 0.15), (-1.0, 0.4)):
    caja_m(v, "estante_papel", "papel", 4.55, 4.85, y, y + 0.25, z - 0.13, z + 0.13)
caja_m(v, "estante_destapacanos", "madera_clara", 4.65, 4.7, 0.3, 1.4, -0.03, 0.03)
caja_m(v, "estante_destapacanos_copa", "hule", 4.55, 4.8, 0.2, 0.4, -0.13, 0.13)
caja_m(v, "estante_trapeador", "madera_clara", 4.65, 4.7, 0.35, 1.6, 0.97, 1.03)
caja_m(v, "estante_trapeador_mecha", "mecha", 4.55, 4.8, 0.2, 0.4, 0.85, 1.15)
caja_m(v, "estante_cubeta", "cubeta", 4.5, 4.85, 0.0, 0.3, 1.15, 1.45)
# Los lavamanos con el espejo, en la pared derecha hacia el frente.
for z in (2.2, 3.3):
    caja_m(v, "lavamanos", "loza", 4.4, 5.0, 0.75, 0.95, z - 0.35, z + 0.35)
    caja_m(v, "lavamanos", "loza", 4.6, 5.0, 0.0, 0.75, z - 0.15, z + 0.15)
    caja_m(v, "llave", "metal", 4.75, 4.85, 0.95, 1.15, z - 0.05, z + 0.05)
caja_m(v, "espejo", "espejo", 4.95, 5.0, 1.25, 2.2, 1.7, 3.8)
# La lampara del techo (no hay techo: se ve desde arriba), una barra que brilla.
caja_m(v, "lampara", "lampara", -0.8, 0.8, 2.45, 2.55, -3.95, -3.85)

escribir(v, "bano_chepa", "El bano publico de Urano de Dona Chepa (visto desde arriba, el frente en +Z).",
         [("azulejo", (0.88, 0.90, 0.93), None, 0.08), ("azulejo_celeste", (0.52, 0.78, 0.84), None, 0.08),
          ("pared", (0.85, 0.88, 0.92)), ("franja", (0.40, 0.80, 0.85)), ("zocalo", (0.25, 0.35, 0.45)),
          ("cubiculo", (0.45, 0.62, 0.80), None, 0.3), ("puerta", (0.55, 0.72, 0.90), None, 0.3),
          ("loza", (0.98, 0.98, 1.0), None, 0.8), ("asiento", (0.30, 0.30, 0.36)),
          ("numero", (1.0, 0.95, 0.6), (0.25, 0.22, 0.08)),
          ("marco", (0.30, 0.30, 0.36)), ("cielo", (0.02, 0.02, 0.07), (0.01, 0.01, 0.04)),
          ("urano", (0.55, 0.88, 0.92), (0.30, 0.55, 0.60)), ("anillo", (0.75, 0.85, 0.95), (0.35, 0.40, 0.45)),
          ("madera", (0.55, 0.33, 0.18)), ("madera_clara", (0.78, 0.58, 0.36)),
          ("alcancia", (0.95, 0.45, 0.65)), ("letrero_q1", (1.0, 0.85, 0.25), (0.35, 0.28, 0.05)),
          ("letrero", (0.40, 0.90, 0.95), (0.30, 0.70, 0.75)),
          ("sombra", (0.12, 0.10, 0.12)), ("papel", (0.98, 0.98, 0.94)), ("hule", (0.85, 0.15, 0.15)),
          ("mecha", (0.75, 0.75, 0.70)), ("cubeta", (0.25, 0.45, 0.95)), ("metal", (0.70, 0.72, 0.76), None, 0.8),
          ("espejo", (0.70, 0.82, 0.90), (0.12, 0.15, 0.18), 1.0), ("lampara", (1.0, 1.0, 0.95), (1.0, 0.98, 0.92))],
         comentarios)


# =====================================================================
# El alien (cliente)
# =====================================================================
U = 0.05
a = Voxels(U)
# El cuerpo: un frijol gordito (elipsoide), con patitas.
for i in range(-8, 8):
    for j in range(2, 18):
        for k in range(-7, 7):
            x, y, z = (i + 0.5) / 7.5, (j + 0.5 - 10) / 8.5, (k + 0.5) / 6.5
            if x * x + y * y + z * z <= 1.0:
                a.put(i, j, k, "piel", "piel")
for i in (-4, 2):
    a.box("patas", "pata", i, i + 1, 0, 2, -1, 1)
# Las antenas con su bolita que brilla.
for s in (-1, 1):
    for j in range(17, 22):
        a.put(s * 3 + (0 if s > 0 else -1), j, 0, "antenas", "pata")
    a.box("antenas_bolita", "bolita", s * 3 - 1 + (0 if s > 0 else -1), s * 3 + 1 + (0 if s > 0 else -1), 22, 23, -1, 0)
# Las caras, pegadas al frente (+Z): feliz, urgido (ojos grandes y sudor) y enojado (cejas).
def cara(emo, ojos, boca, extra=()):
    for (i, j) in ojos:
        a.put(i, j, 7, f"cara_{emo}_ojos", "ojo")
    for (i, j) in boca:
        a.put(i, j, 7, f"cara_{emo}_boca", "boca")
    for (i, j, mat) in extra:
        a.put(i, j, 7, f"cara_{emo}_{mat}", mat)
cara("feliz", [(-3, 11), (-3, 12), (2, 11), (2, 12)], [(-2, 8), (-1, 7), (0, 7), (1, 8)])
cara("urgido", [(-4, 11), (-3, 11), (-4, 12), (-3, 12), (2, 11), (3, 11), (2, 12), (3, 12)], [(-1, 7), (0, 7), (-1, 8), (0, 8)],
     [(5, 13, "sudor"), (5, 12, "sudor")])
cara("enojado", [(-3, 11), (2, 11)], [(-2, 7), (-1, 8), (0, 8), (1, 7)], [(-4, 13, "ceja"), (-3, 12, "ceja"), (2, 12, "ceja"), (3, 13, "ceja")])
escribir(a, "alien_urano", "Un cliente del bano de Urano (el juego le cambia el color a la piel). Origen en los pies, mira a +Z.",
         [("piel", (0.45, 0.85, 0.55)), ("pata", (0.20, 0.30, 0.25)), ("bolita", (1.0, 0.85, 0.3), (0.6, 0.45, 0.1)),
          ("ojo", (0.05, 0.04, 0.07)), ("boca", (0.55, 0.12, 0.25)), ("sudor", (0.4, 0.78, 1.0), (0.1, 0.2, 0.3)),
          ("ceja", (0.1, 0.06, 0.06))])


# =====================================================================
# Dona Chepa
# =====================================================================
c = Voxels(U)
# Sentada: piernas al frente, el delantal sobre el huipil, los brazos y la cabeza con su mono.
c.box("piernas", "falda", -4, 3, 6, 9, 0, 6)
c.box("pies", "zapato", -4, 3, 0, 1, 5, 7)
c.box("canillas", "piel", -3, 2, 2, 5, 5, 6)
c.box("cuerpo", "huipil", -5, 4, 10, 20, -2, 3)
for j in range(11, 20, 3):
    for i in range(-5, 5):
        if (i + j) % 2 == 0:
            c.put(i, j, 4, "huipil_bordado", "bordado")
c.box("delantal", "delantal", -3, 2, 7, 17, 4, 4)
c.box("brazos", "huipil", -7, -6, 12, 19, -1, 2)
c.box("brazos", "huipil", 5, 6, 12, 19, -1, 2)
c.box("manos", "piel", -7, -6, 10, 11, 1, 3)
c.box("manos", "piel", 5, 6, 10, 11, 1, 3)
c.box("cabeza", "piel", -4, 3, 21, 28, -3, 3)
c.box("pelo", "pelo", -4, 3, 28, 29, -3, 3)
c.box("pelo", "pelo", -4, 3, 22, 28, -4, -4)
c.box("pelo_mono", "pelo", -2, 1, 28, 31, -5, -2)
c.box("lentes", "lentes", -3, 2, 25, 25, 4, 4)
c.put(-3, 24, 4, "ojos", "ojo")
c.put(1, 24, 4, "ojos", "ojo")
c.box("boca", "boca", -1, 0, 22, 22, 4, 4)
c.box("cachetes", "rubor", -4, -3, 23, 23, 4, 4)
c.box("cachetes", "rubor", 2, 3, 23, 23, 4, 4)
# El rollo de papel en la mano (el que cobra a Q1).
c.box("rollo", "papel", 5, 7, 11, 13, 3, 5)
escribir(c, "chepa", "Dona Chepa, sentada en su banco (origen en el piso, mira a +Z).",
         [("falda", (0.20, 0.25, 0.55)), ("zapato", (0.15, 0.10, 0.08)), ("piel", (0.82, 0.62, 0.48)),
          ("huipil", (0.75, 0.18, 0.30)), ("bordado", (1.0, 0.80, 0.25)), ("delantal", (0.95, 0.95, 0.92)),
          ("pelo", (0.70, 0.70, 0.72)), ("lentes", (0.25, 0.70, 0.95), (0.05, 0.12, 0.18)), ("ojo", (0.05, 0.04, 0.07)),
          ("boca", (0.55, 0.12, 0.25)), ("rubor", (1.0, 0.55, 0.60)), ("papel", (0.98, 0.98, 0.94))])


# =====================================================================
# Lo que se carga en la mano
# =====================================================================
p = Voxels(U)
for i in range(-3, 3):
    for j in range(0, 5):
        for k in range(-3, 3):
            d = math.hypot(i + 0.5, k + 0.5)
            if d <= 3.0:
                p.put(i, j, k, "rollo" if d > 1.2 else "carton", "papel" if d > 1.2 else "carton")
escribir(p, "item_papel", "Un rollo de papel higienico (origen abajo).", [("papel", (0.98, 0.98, 0.94)), ("carton", (0.65, 0.45, 0.25))])

dp = Voxels(U)
dp.box("palo", "madera", -1, 0, 4, 20, -1, 0)
for i in range(-3, 3):
    for j in range(0, 4):
        for k in range(-3, 3):
            if math.hypot(i + 0.5, k + 0.5) <= 3.0 - j * 0.3:
                dp.put(i, j, k, "copa", "hule")
escribir(dp, "item_destapacanos", "El destapacanos (origen abajo).", [("madera", (0.78, 0.58, 0.36)), ("hule", (0.85, 0.15, 0.15))])

t = Voxels(U)
t.box("palo", "madera", -1, 0, 4, 24, -1, 0)
for i in range(-4, 4):
    for k in range(-3, 3):
        for j in range(0, 4 - abs(i) // 3):
            t.put(i, j, k, "mecha", "mecha")
escribir(t, "item_trapeador", "El trapeador (origen abajo).", [("madera", (0.78, 0.58, 0.36)), ("mecha", (0.80, 0.80, 0.74))])
