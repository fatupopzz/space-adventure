# Genera los modelos de la cita en Marte (Doki Doki Marte):
#   curiosity.obj:    el rover solito, con cara en la cabeza de su mastil.
#   perseverance.obj: la rover nueva, mas blanca y con un monito en la cabeza.
#   ingenuity.obj:    la helicoptera, con una aspa quebrada (como la real desde 2024).
#   insight.obj:      la sismologa, con sus dos paneles redondos llenos de polvo.
#   cita_marte.obj:   el piso de Marte con dunas, piedras y el picnic (mantel, pastel, rosa y velitas).
# Todos miran hacia -Z, con el origen en el piso. Los cuatro personajes llevan las caras chibi.
# Uso: python3 herramientas/generar_cita.py

import math
from modelo3d import *

def mtl(*materiales):
    out = ["# Materiales"]
    for nombre, kd, ke in materiales:
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), "Ks 0.3 0.3 0.3", "Ns 24"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
    return "\n".join(out) + "\n"

U = 0.04

def cilindro_x(vx, parte, mat, i0, i1, cj, ck, r):
    """Un cilindro acostado a lo largo de X (una rueda), centro (cj, ck) en voxels."""
    for i in range(i0, i1 + 1):
        for j in range(cj - r - 1, cj + r + 1):
            for k in range(ck - r - 1, ck + r + 1):
                if (j - cj) ** 2 + (k - ck) ** 2 <= r * r + 0.5:
                    vx.put(i, j, k, parte, mat)

# ---------- los rovers (Curiosity y Perseverance comparten el chasis) ----------
def rover(nombre, header, cuerpo, acento, lazo):
    vx = Voxels(U)
    # Chasis
    vx.box("chasis", "chasis", -6, 5, 7, 10, -7, 6)
    vx.box("cubierta", "cubierta", -5, 4, 11, 11, -6, 5)
    # El generador nuclear atras, con aletas
    vx.box("generador", "generador", -2, 1, 8, 12, 7, 10)
    for j in (9, 11):
        vx.box("generador", "generador", -4, 3, j, j, 8, 9)
    # Seis ruedas con su suspension
    for k in (-6, 0, 6):
        cilindro_x(vx, "ruedas", "rueda", 7, 9, 2, k, 2)
        cilindro_x(vx, "ruedas", "rueda", -10, -8, 2, k, 2)
        vx.box("suspension", "suspension", 7, 7, 3, 6, k, k)
        vx.box("suspension", "suspension", -8, -8, 3, 6, k, k)
    vx.box("suspension", "suspension", 7, 7, 6, 6, -6, 6)
    vx.box("suspension", "suspension", -8, -8, 6, 6, -6, 6)
    # Antena plato (la de alta ganancia), atras a la izquierda
    for i in range(-6, 0):
        for k in range(1, 7):
            if (i + 3.5) ** 2 + (k - 3.5) ** 2 <= 9:
                vx.put(i, 13, k, "antena", "acento")
    vx.box("antena", "acento", -4, -3, 12, 12, 3, 4)
    # El brazo robotico al frente, doblado, con su torreta
    vx.box("brazo", "suspension", -5, -4, 9, 9, -11, -8)
    vx.box("brazo", "suspension", -5, -4, 5, 8, -11, -11)
    vx.box("torreta", "acento", -6, -3, 3, 5, -13, -10)
    # El mastil y la cabeza (una cabezota cuadrada, como robot de caricatura)
    vx.box("mastil", "suspension", 1, 2, 12, 18, -4, -3)
    vx.box("cabeza", "cabeza", -2, 6, 19, 25, -7, -2)
    vx.box("cabeza_lente", "lente", 3, 5, 26, 27, -6, -4)
    if lazo:
        # Un monito arriba de la cabeza: dos lazos y el nudo
        for (i0, i1) in ((-3, -1), (3, 5)):
            vx.box("lazo", "lazo", i0, i1, 26, 28, -5, -4)
        vx.box("lazo", "lazo", 0, 2, 26, 27, -5, -4)
    m = Modelo()
    vx.mesh(m)
    # Cara en el frente de la cabeza (z = -7 voxels)
    expresiones(m, z_frente=-7 * U, px=0.017, x_centro=2.5 * U, y_base=19 * U + 0.012)
    m.write(nombre, header, mtl(("chasis", cuerpo, None), ("cubierta", (0.80, 0.62, 0.30), None),
                                ("generador", (0.25, 0.25, 0.30), None), ("rueda", (0.12, 0.12, 0.15), None),
                                ("suspension", (0.55, 0.56, 0.62), None), ("acento", acento, None),
                                ("cabeza", (0.95, 0.94, 0.92), None), ("lente", (0.2, 0.6, 1.0), (0.2, 0.6, 1.0)),
                                ("lazo", (1.0, 0.35, 0.6), (0.25, 0.05, 0.12))) + MTL_CARA)

rover("curiosity", """# Curiosity (voxel de 0.04): seis ruedas, generador atras, brazo al frente y la cabeza en el mastil
# con las caras chibi. Mira hacia -Z, origen en el piso. Lleva solito en Marte desde 2012.
""", (0.88, 0.86, 0.80), (0.85, 0.65, 0.25), False)
rover("perseverance", """# Perseverance (voxel de 0.04): el mismo chasis que Curiosity, mas blanca, con acentos rosas
# y un monito en la cabeza. Mira hacia -Z, origen en el piso.
""", (0.97, 0.97, 0.99), (0.95, 0.55, 0.70), True)

# ---------- Ingenuity, la helicoptera ----------
vx = Voxels(U)
vx.box("cuerpo", "cuerpo_ingenuity", -3, 2, 5, 10, -3, 2)
# Cuatro patitas en diagonal, hacia afuera
for si, sk in ((-1, -1), (-1, 1), (1, -1), (1, 1)):
    for s in range(6):
        i = (2 if si > 0 else -3) + si * (s // 2)
        k = (2 if sk > 0 else -3) + sk * (s // 2)
        vx.put(i, 5 - s, k, "patas", "patas")
# Mastil, dos rotores y el panel solar arriba
vx.box("mastil", "patas", -1, 0, 11, 19, -1, 0)
for i in range(-12, 12):
    vx.put(i, 14, -1, "aspas", "aspas")
# La de arriba esta quebrada: le falta media aspa
for k in range(-12, 5):
    vx.put(-1, 17, k, "aspas", "aspas")
vx.box("panel", "panel", -4, 3, 20, 20, -4, 3)
m = Modelo()
vx.mesh(m)
expresiones(m, z_frente=-3 * U, px=0.0115, x_centro=-0.5 * U, y_base=5 * U + 0.03)
m.write("ingenuity", """# Ingenuity (voxel de 0.04): cajita con cuatro patas, dos rotores (uno quebrado) y su panel solar.
# Cara chibi en el frente del cuerpo. Mira hacia -Z, origen en el piso.
""", mtl(("cuerpo_ingenuity", (0.85, 0.70, 0.35), None), ("patas", (0.30, 0.30, 0.34), None),
         ("aspas", (0.92, 0.92, 0.95), None), ("panel", (0.15, 0.25, 0.60), (0.03, 0.05, 0.15))) + MTL_CARA)

# ---------- InSight, la sismologa ----------
vx = Voxels(U)
vx.box("cubierta", "cubierta_insight", -6, 5, 4, 10, -5, 4)
for i, k in ((-5, -4), (4, -4), (-1, 3)):
    vx.box("patas", "patas", i, i, 0, 3, k, k)
# Los dos paneles redondos, uno de cada lado, con manchas de polvo
for lado in (-1, 1):
    cx = lado * 13
    for i in range(cx - 8, cx + 8):
        for k in range(-8, 8):
            d = ((i + 0.5 - cx) ** 2 + (k + 0.5) ** 2) ** 0.5
            if d <= 7.5:
                borde = d > 6.6
                polvo = not borde and math.sin(i * 1.7 + k * 0.9) * math.cos(k * 1.3 - i * 0.4) > 0.45
                mat = "panel_borde" if borde else ("polvo" if polvo else "panel_insight")
                vx.put(i, 9, k, mat, mat)
    vx.box("patas", "patas", lado * 6 if lado > 0 else -7, lado * 6 if lado > 0 else -7, 8, 9, -1, 0)
# El sismometro: un domito en el piso, enfrente
for i in range(-4, 4):
    for j in range(0, 4):
        for k in range(-14, -6):
            if (i + 0.5) ** 2 + (j + 0.5) ** 2 * 1.6 + (k + 9.5) ** 2 <= 14:
                vx.put(i, j, k, "domo", "domo")
# El brazo con la palita
vx.box("brazo", "patas", 4, 4, 10, 13, 2, 2)
vx.box("brazo", "patas", 4, 4, 13, 13, -3, 2)
vx.box("palita", "panel_borde", 3, 5, 11, 12, -4, -3)
m = Modelo()
vx.mesh(m)
expresiones(m, z_frente=-5 * U, px=0.018, x_centro=-0.5 * U, y_base=4 * U + 0.01)
m.write("insight", """# InSight (voxel de 0.04): el modulo con dos paneles solares redondos (con polvo), el domo del
# sismometro enfrente y su brazo con palita. Cara chibi al frente. Mira hacia -Z, origen en el piso.
""", mtl(("cubierta_insight", (0.90, 0.88, 0.84), None), ("patas", (0.35, 0.35, 0.40), None),
         ("panel_insight", (0.20, 0.30, 0.65), (0.03, 0.05, 0.15)), ("panel_borde", (0.85, 0.55, 0.25), None),
         ("polvo", (0.75, 0.45, 0.30), None), ("domo", (0.95, 0.95, 0.97), None)) + MTL_CARA)

# ---------- Sojourner, la abuelita (1997) ----------
# Chiquita como un microondas: seis rueditas, el panel solar plano arriba y la cara al frente.
vx = Voxels(U)
vx.box("cuerpo", "cuerpo_sojourner", -4, 3, 3, 6, -5, 4)
vx.box("panel", "panel_sojourner", -5, 4, 7, 7, -6, 5)
for k in (-4, 0, 4):
    cilindro_x(vx, "ruedas", "rueda", 5, 6, 1, k, 1)
    cilindro_x(vx, "ruedas", "rueda", -7, -6, 1, k, 1)
vx.box("antena", "acento_sojourner", 2, 2, 8, 12, 3, 3)
vx.put(2, 13, 3, "antena_punta", "acento_sojourner")
# Unos lentecitos de abuelita sobre la cara
m = Modelo()
vx.mesh(m)
expresiones(m, z_frente=-5 * U, px=0.0105, x_centro=-0.5 * U, y_base=3 * U + 0.005)
m.write("sojourner", """# Sojourner (voxel de 0.04): el primer rover de Marte (1997), chiquito como un microondas: seis rueditas,
# panel solar plano y antena. Cara chibi al frente. Mira hacia -Z, origen en el piso.
""", mtl(("cuerpo_sojourner", (0.85, 0.80, 0.68), None), ("panel_sojourner", (0.20, 0.30, 0.65), (0.03, 0.05, 0.15)),
         ("rueda", (0.25, 0.25, 0.28), None), ("acento_sojourner", (0.95, 0.75, 0.30), None)) + MTL_CARA)

# ---------- El escenario: Marte al atardecer con un picnic ----------
vx = Voxels(0.12)
for i in range(-75, 75):
    for k in range(-75, 75):
        x, _, z = vx.centro(i, 0, k)
        r = (x * x + z * z) ** 0.5
        if r > 9.0:
            continue
        # Plano en medio y dunas que suben hacia la orilla (tapan el borde del mundo)
        h = 0.0
        if r > 3.5:
            h = ((r - 3.5) / 5.5) ** 1.6 * 2.6 * (0.75 + 0.25 * math.sin(math.atan2(z, x) * 5 + r))
        h += 0.06 * math.sin(x * 2.1 + 0.5) * math.sin(z * 1.7)
        alto = max(0, round(h / 0.12))
        oscuro = math.sin(x * 1.3 + z * 0.7) * math.sin(z * 1.1 - x * 0.4) > 0.35
        mat = "suelo_oscuro" if oscuro else "suelo"
        for j in range(-1, alto):
            vx.put(i, j, k, mat, mat)
# Piedras sueltas
for (x, z, s) in ((-2.6, 1.8, 2), (2.9, 2.4, 3), (-3.2, -1.2, 1), (3.4, -0.6, 2), (0.9, 3.2, 1), (-1.4, 3.0, 2)):
    i0, k0 = round(x / 0.12), round(z / 0.12)
    for i in range(i0 - s, i0 + s + 1):
        for k in range(k0 - s, k0 + s + 1):
            for j in range(0, s + 1):
                if (i - i0) ** 2 + (k - k0) ** 2 + (j * 1.4) ** 2 <= s * s + 1:
                    vx.put(i, j, k, "piedras", "piedras")
m = Modelo()
vx.mesh(m)
# El picnic, con voxels mas finos
vp = Voxels(0.04)
for i in range(-18, 18):
    for k in range(-12, 12):
        mat = "mantel_rojo" if (i // 3 + k // 3) % 2 == 0 else "mantel_blanco"
        vp.put(i, 0, k, mat, mat)
# El pastel de cumpleanos (con una velita) en medio
for i in range(-4, 4):
    for k in range(-4, 4):
        if (i + 0.5) ** 2 + (k + 0.5) ** 2 <= 14:
            for j in range(1, 4):
                vp.put(i, j, k, "pastel", "pastel")
            vp.put(i, 4, k, "betun", "betun")
vp.box("vela", "vela", 0, 0, 5, 7, 0, 0)
vp.put(0, 8, 0, "llama", "llama")
# Una rosa en un vasito
vp.box("vasito", "vasito", 7, 8, 1, 3, -6, -5)
vp.box("tallo", "tallo", 7, 7, 4, 7, -6, -6)
vp.box("rosa", "rosa", 6, 8, 8, 9, -7, -5)
# Dos velitas en las esquinas
for i, k in ((-14, -9), (14, 9)):
    vp.box("vela", "vela", i, i, 1, 3, k, k)
    vp.put(i, 4, k, "llama", "llama")
vp.mesh(m)
m.write("cita_marte", """# La cita en Marte: suelo voxel de 0.12 en un circulo de radio 9 con dunas en la orilla y piedras,
# y en medio el picnic (voxel de 0.04): mantel a cuadros, pastel con velita, una rosa y dos velas.
# Origen en el piso, el mantel en el centro.
""", mtl(("suelo", (0.78, 0.40, 0.24), None), ("suelo_oscuro", (0.62, 0.30, 0.18), None), ("piedras", (0.45, 0.25, 0.18), None),
         ("mantel_rojo", (0.92, 0.25, 0.32), None), ("mantel_blanco", (0.97, 0.94, 0.94), None),
         ("pastel", (0.98, 0.80, 0.86), None), ("betun", (1.0, 0.98, 0.98), None), ("vela", (0.6, 0.85, 1.0), None),
         ("llama", (1.0, 0.75, 0.3), (1.0, 0.6, 0.15)), ("vasito", (0.7, 0.9, 1.0), None), ("tallo", (0.3, 0.7, 0.3), None),
         ("rosa", (0.95, 0.15, 0.3), (0.2, 0.0, 0.05))))
