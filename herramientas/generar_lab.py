# Genera modelos/lab_cit.obj: el laboratorio de computo del CIT (UVG) a las 11 de la noche, la
# noche antes de la entrega de los tres proyectos del profe Koch (la Tierra).
# Tres filas de escritorios con su computadora (cada pantalla brilla con algo distinto: codigo,
# un juego, una pantalla azul...), sillas, el pizarron con "ENTREGA 23:59" en rojo, el proyector,
# el reloj de pared (las manecillas van aparte: reloj_horario y reloj_minutero, que el juego gira),
# las ventanas con la noche y la lluvia, la maquina de cafe con su luz, el letrero de la UVG y
# las lamparas del techo.
# Origen: el centro del piso. La pared del frente (pizarron y reloj) esta hacia -Z.
# Uso: python3 herramientas/generar_lab.py

import math, random
from modelo3d import *

random.seed(1159)


def mtl(*materiales):
    out = ["# Materiales del lab"]
    for nombre, kd, ke, *extra in materiales:
        ks, ns = (extra + [0.2, 16])[:2] if extra else (0.2, 16)
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), f"Ks {ks} {ks} {ks}", f"Ns {ns}"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
    return "\n".join(out) + "\n"


def caja_m(vx, parte, mat, x0, x1, y0, y1, z0, z1, si=None):
    u = vx.u
    a = lambda v: int(math.floor(v / u + 1e-6))
    b = lambda v: int(math.ceil(v / u - 1e-6)) - 1
    vx.box(parte, mat, a(x0), b(x1), a(y0), b(y1), a(z0), b(z1), si=si)


def por_material(vx):
    for clave, (p, mat) in list(vx.v.items()):
        vx.v[clave] = (p if p == mat else f"{p}_{mat}", mat)


ANCHO, FONDO, ALTO = 6.5, 5.5, 3.2
g = Voxels(0.1)
# El piso de cuadros y el techo.
for i in range(int(-ANCHO), int(ANCHO)):
    for k in range(int(-FONDO), int(FONDO)):
        caja_m(g, "piso", "piso_a" if (i + k) % 2 == 0 else "piso_b", i, i + 1, -0.1, 0.0, k, k + 1)
caja_m(g, "techo", "techo", -ANCHO, ANCHO, ALTO, ALTO + 0.1, -FONDO, FONDO)
# Las paredes: verde abajo (como la UVG), una franja amarilla y crema arriba.
for (x0, x1, z0, z1) in ((-ANCHO - 0.1, ANCHO + 0.1, -FONDO - 0.1, -FONDO), (-ANCHO - 0.1, ANCHO + 0.1, FONDO, FONDO + 0.1),
                         (-ANCHO - 0.1, -ANCHO, -FONDO, FONDO), (ANCHO, ANCHO + 0.1, -FONDO, FONDO)):
    caja_m(g, "pared", "pared", x0, x1, 1.2, ALTO, z0, z1)
    caja_m(g, "pared_baja", "pared_baja", x0, x1, 0.0, 1.1, z0, z1)
    caja_m(g, "pared_franja", "franja", x0, x1, 1.1, 1.2, z0, z1)
# Las lamparas del techo (en filas). Es de noche: la fila de atras ya la apagaron (ahorro de
# energia), solo quedan prendidas las de adelante y las del medio.
for i in (-4, 0, 4):
    for k in (-3.5, 0.0, 3.5):
        prendida = k < 3.0
        caja_m(g, "lamparas" if prendida else "techo_apagadas", "lampara" if prendida else "lampara_apagada", i - 0.6, i + 0.6, ALTO - 0.1, ALTO, k - 0.2, k + 0.2)

f = Voxels(0.05)
# --- La pared del frente: pizarron, proyector y reloj ---
caja_m(f, "pizarron_marco", "aluminio", -5.6, -1.4, 0.85, 2.35, -FONDO, -FONDO + 0.05)
caja_m(f, "pizarron", "pizarron", -5.5, -1.5, 0.95, 2.25, -FONDO + 0.05, -FONDO + 0.08)
# "ENTREGA 23:59" en rojo, con letras de pixeles.
GLIFOS = {"E": ["###", "#..", "##.", "#..", "###"], "N": ["#.#", "###", "###", "###", "#.#"], "T": ["###", ".#.", ".#.", ".#.", ".#."],
          "R": ["##.", "#.#", "##.", "#.#", "#.#"], "G": ["###", "#..", "#.#", "#.#", "###"], "A": [".#.", "#.#", "###", "#.#", "#.#"],
          "2": ["###", "..#", "###", "#..", "###"], "3": ["###", "..#", "###", "..#", "###"], "5": ["###", "#..", "###", "..#", "###"],
          "9": ["###", "#.#", "###", "..#", "###"], ":": ["...", ".#.", "...", ".#.", "..."], " ": ["...", "...", "...", "...", "..."], "!": [".#.", ".#.", ".#.", "...", ".#."],
          "O": ["###", "#.#", "#.#", "#.#", "###"], "C": ["###", "#..", "#..", "#..", "###"], "M": ["#.#", "###", "###", "#.#", "#.#"],
          "S": ["###", "#..", "###", "..#", "###"], "L": ["#..", "#..", "#..", "#..", "###"], "I": ["###", ".#.", ".#.", ".#.", "###"],
          "D": ["##.", "#.#", "#.#", "#.#", "##."], "B": ["##.", "#.#", "##.", "#.#", "##."]}


def escribir(texto, x0, y0, z, pixel, parte, mat):
    for n, ch in enumerate(texto):
        for fila, bits in enumerate(GLIFOS[ch]):
            for col, c in enumerate(bits):
                if c == "#":
                    x = x0 + (n * 4 + col) * pixel
                    y = y0 - fila * pixel
                    caja_m(f, parte, mat, x, x + pixel, y - pixel, y, z, z + 0.02)


escribir("ENTREGA", -5.2, 2.05, -FONDO + 0.08, 0.1, "pizarron_letras", "marcador_rojo")
escribir("23:59 !", -5.2, 1.45, -FONDO + 0.08, 0.1, "pizarron_letras", "marcador_rojo")
# La pantalla del proyector (donde sale Koch por Teams).
caja_m(f, "proyector_tela", "tela", -1.0, 2.6, 0.9, 2.5, -FONDO + 0.02, -FONDO + 0.05)
caja_m(f, "proyector_tubo", "aluminio", -1.1, 2.7, 2.5, 2.6, -FONDO + 0.02, -FONDO + 0.12)
caja_m(f, "proyector", "plastico_negro", 0.6, 1.0, ALTO - 0.4, ALTO - 0.2, -1.5, -1.1)
caja_m(f, "proyector", "plastico_negro", 0.75, 0.85, ALTO - 0.2, ALTO, -1.35, -1.25)
caja_m(f, "proyector_lente", "lente", 0.72, 0.88, ALTO - 0.35, ALTO - 0.25, -1.55, -1.5)
# El reloj de pared: la cara, las rayitas de las horas, y las manecillas aparte (con su eje).
RELOJ = (3.9, 2.45, -FONDO + 0.06)
for i in range(-9, 10):
    for j in range(-9, 10):
        r = math.hypot(i + 0.5, j + 0.5)
        if r <= 9.5:
            mat = "reloj_borde" if r > 8.2 else "reloj_cara"
            f.put(int(RELOJ[0] / 0.05) + i, int(RELOJ[1] / 0.05) + j, int(RELOJ[2] / 0.05), "reloj", mat)
for h in range(12):
    a = h / 12 * math.tau
    x, y = RELOJ[0] + math.sin(a) * 0.34, RELOJ[1] + math.cos(a) * 0.34
    caja_m(f, "reloj_horas", "negro", x - 0.025, x + 0.025, y - 0.025, y + 0.025, RELOJ[2] + 0.05, RELOJ[2] + 0.07)
caja_m(f, "reloj_horario", "negro", RELOJ[0] - 0.025, RELOJ[0] + 0.025, RELOJ[1], RELOJ[1] + 0.22, RELOJ[2] + 0.07, RELOJ[2] + 0.09)
caja_m(f, "reloj_minutero", "rojo_reloj", RELOJ[0] - 0.015, RELOJ[0] + 0.015, RELOJ[1], RELOJ[1] + 0.36, RELOJ[2] + 0.09, RELOJ[2] + 0.11)
# El letrero de la UVG (verde, con las letras).
caja_m(f, "letrero_uvg", "verde_uvg", -ANCHO + 0.02, -ANCHO + 0.07, 1.6, 2.6, -1.5, 1.5)
GLIFOS.update({"U": ["#.#", "#.#", "#.#", "#.#", "###"], "V": ["#.#", "#.#", "#.#", "#.#", ".#."]})
for n, ch in enumerate("UVG"):
    for fila, bits in enumerate(GLIFOS[ch]):
        for col, c in enumerate(bits):
            if c == "#":
                # se lee desde adentro (mirando hacia -X): la derecha es -Z
                z = 1.1 - (n * 4 + col) * 0.1 - 0.1
                y = 2.35 - fila * 0.1
                caja_m(f, "letrero_letras", "blanco", -ANCHO + 0.07, -ANCHO + 0.1, y - 0.1, y, z, z + 0.1)

# --- La pared de la derecha: las ventanas con la noche (y la lluvia) ---
for k in (-3.6, -0.6, 2.4):
    caja_m(f, "ventanas_marco", "aluminio", ANCHO - 0.06, ANCHO, 1.0, 2.6, k - 1.2, k + 1.2)
    caja_m(f, "ventanas", "noche", ANCHO - 0.08, ANCHO - 0.06, 1.05, 2.55, k - 1.15, k + 1.15)
    caja_m(f, "ventanas_marco", "aluminio", ANCHO - 0.1, ANCHO - 0.06, 1.77, 1.83, k - 1.15, k + 1.15)
    # las luces del campus a lo lejos y unas gotas
    for n in range(6):
        y = 1.15 + random.random() * 0.5
        z = k - 1.0 + random.random() * 2.0
        caja_m(f, "ventanas_luces", "luz_lejos", ANCHO - 0.09, ANCHO - 0.08, y, y + 0.04, z, z + 0.04)
    for n in range(14):
        y = 1.1 + random.random() * 1.4
        z = k - 1.1 + random.random() * 2.2
        caja_m(f, "ventanas_gotas", "gota", ANCHO - 0.095, ANCHO - 0.085, y, y + 0.12, z, z + 0.02)

# --- La pared de atras: la puerta y la maquina de cafe ---
caja_m(f, "puerta", "puerta", -4.5, -3.3, 0.0, 2.2, FONDO - 0.05, FONDO)
caja_m(f, "puerta_manija", "aluminio", -3.5, -3.4, 1.0, 1.1, FONDO - 0.1, FONDO - 0.05)
caja_m(f, "cafetera", "cafetera", 3.2, 4.3, 0.0, 1.9, FONDO - 0.8, FONDO)
caja_m(f, "cafetera_panel", "cafetera_luz", 3.35, 4.15, 1.05, 1.75, FONDO - 0.82, FONDO - 0.8)
caja_m(f, "cafetera_boton", "boton_verde", 3.6, 3.75, 0.8, 0.9, FONDO - 0.83, FONDO - 0.8)
caja_m(f, "cafetera_boton", "boton_rojo", 3.85, 4.0, 0.8, 0.9, FONDO - 0.83, FONDO - 0.8)
caja_m(f, "cafetera_hueco", "negro", 3.5, 4.0, 0.3, 0.65, FONDO - 0.82, FONDO - 0.75)
caja_m(f, "basurero", "basurero", 2.5, 2.9, 0.0, 0.7, FONDO - 0.6, FONDO - 0.2)
# El letrero verde de SALIDA sobre la puerta (es lo unico que se ve cuando se va la luz).
caja_m(f, "salida", "salida_caja", -4.3, -3.5, 2.35, 2.62, FONDO - 0.12, FONDO)


def escribir_atras(texto, x_der, y0, z, pixel, parte, mat):
    """Letras en la pared de atras (se leen desde adentro: van de derecha a izquierda en X)."""
    for n, ch in enumerate(texto):
        for fila, bits in enumerate(GLIFOS[ch]):
            for col, c in enumerate(bits):
                if c == "#":
                    x = x_der - (n * 4 + col) * pixel
                    y = y0 - fila * pixel
                    caja_m(f, parte, mat, x - pixel, x, y - pixel, y, z - 0.02, z)


escribir_atras("SALIDA", -3.55, 2.56, FONDO - 0.12, 0.033, "salida_letras", "salida_luz")
# Un cartel de "NO COMER" (con una pizza tachada) en la pared de atras. Nadie le hace caso.
caja_m(f, "cartel", "cartel_blanco", -1.6, 0.4, 1.35, 2.35, FONDO - 0.03, FONDO)
caja_m(f, "cartel_borde", "cartel_rojo", -1.6, 0.4, 2.25, 2.35, FONDO - 0.04, FONDO - 0.03)
escribir_atras("NO COMER", 0.25, 2.15, FONDO - 0.03, 0.05, "cartel_letras", "cartel_rojo")
for i in range(-6, 7):
    for j in range(-6, 7):
        r = math.hypot(i, j)
        if r <= 6:
            x, y = -0.6 + i * 0.04, 1.65 + j * 0.04
            mat = "cartel_rojo" if r > 5 or abs(i - j) <= 0 else ("pizza_orilla" if r > 4 else ("pepperoni" if (i * 3 + j * 5) % 7 == 0 else "pizza_queso"))
            caja_m(f, "cartel_pizza", mat, x, x + 0.04, y, y + 0.04, FONDO - 0.035, FONDO - 0.03)
# Un cartel de la Semana de Graficas en la pared de la izquierda (con un cubito en perspectiva).
caja_m(f, "afiche", "afiche", -ANCHO + 0.02, -ANCHO + 0.05, 1.3, 2.4, 2.2, 3.6)
for (y, z0, z1) in ((2.0, 2.6, 3.0), (1.6, 2.6, 3.0), (2.15, 2.8, 3.2), (1.75, 2.8, 3.2)):
    caja_m(f, "afiche_cubo", "afiche_linea", -ANCHO + 0.05, -ANCHO + 0.06, y, y + 0.03, z0, z1)
for (z, y0, y1) in ((2.6, 1.6, 2.03), (3.0, 1.6, 2.03), (2.8, 1.75, 2.18), (3.2, 1.75, 2.18)):
    caja_m(f, "afiche_cubo", "afiche_linea", -ANCHO + 0.05, -ANCHO + 0.06, y0, y1, z, z + 0.03)

# --- Los escritorios: tres filas de cuatro (el del protagonista es el de adelante, segundo) ---
PANTALLAS = ["pantalla_codigo", "pantalla_azul", "pantalla_juego", "pantalla_codigo", "pantalla_negra", "pantalla_codigo", "pantalla_youtube", "pantalla_codigo"]
ESCRITORIOS = []
for fila, z in enumerate((-1.6, 0.6, 2.8)):
    for col, x in enumerate((-4.2, -1.4, 1.4, 4.2)):
        ESCRITORIOS.append((x, z))
for n, (x, z) in enumerate(ESCRITORIOS):
    mio = n == 1
    # la mesa, larga, con sus patas
    caja_m(f, "mesas", "mesa", x - 1.1, x + 1.1, 0.72, 0.77, z - 0.4, z + 0.4)
    for dx in (-1.05, 1.0):
        caja_m(f, "mesas_patas", "aluminio", x + dx, x + dx + 0.05, 0.0, 0.72, z - 0.35, z + 0.35)
    # el monitor, el teclado, el raton y el CPU
    caja_m(f, "monitores", "plastico_negro", x - 0.42, x + 0.42, 0.85, 1.35, z - 0.3, z - 0.25)
    caja_m(f, "monitores_pie", "plastico_negro", x - 0.05, x + 0.05, 0.77, 0.85, z - 0.28, z - 0.22)
    caja_m(f, "monitores_pie", "plastico_negro", x - 0.15, x + 0.15, 0.77, 0.79, z - 0.32, z - 0.18)
    pantalla = "pantalla_mia" if mio else PANTALLAS[n % len(PANTALLAS)]
    # el vidrio va del lado de la silla (+Z), que es donde se sienta el que la usa
    caja_m(f, "pantallas_" + pantalla, pantalla, x - 0.38, x + 0.38, 0.89, 1.31, z - 0.25, z - 0.24)
    caja_m(f, "teclados", "plastico_gris", x - 0.25, x + 0.25, 0.77, 0.79, z - 0.1, z + 0.08)
    caja_m(f, "ratones", "plastico_gris", x + 0.35, x + 0.42, 0.77, 0.8, z - 0.05, z + 0.05)
    caja_m(f, "cpus", "plastico_negro", x + 0.6, x + 0.82, 0.0, 0.5, z - 0.3, z + 0.2)
    caja_m(f, "cpus_luz", "luz_cpu", x + 0.6, x + 0.62, 0.35, 0.45, z + 0.2, z + 0.21)
    # la silla
    caja_m(f, "sillas", "silla", x - 0.25, x + 0.25, 0.45, 0.5, z + 0.35, z + 0.85)
    caja_m(f, "sillas", "silla", x - 0.25, x + 0.25, 0.5, 1.05, z + 0.82, z + 0.87)
    caja_m(f, "sillas_pata", "aluminio", x - 0.03, x + 0.03, 0.0, 0.45, z + 0.57, z + 0.63)
    # en algunas mesas: latas de bebida energetica y una taza de cafe
    if n % 3 == 0:
        caja_m(f, "latas", "lata", x - 0.8, x - 0.72, 0.77, 0.92, z - 0.2, z - 0.12)
    if n % 4 == 1:
        caja_m(f, "tazas", "taza", x - 0.7, x - 0.6, 0.77, 0.88, z + 0.05, z + 0.15)
    # mochilas tiradas junto a las sillas
    if n % 3 == 1:
        color = ["mochila_roja", "mochila_azul", "mochila_negra"][n % 3 if n % 2 else 2]
        caja_m(f, "mochilas", color, x - 0.75, x - 0.45, 0.0, 0.42, z + 0.5, z + 0.7)
        caja_m(f, "mochilas", color, x - 0.72, x - 0.48, 0.05, 0.25, z + 0.7, z + 0.76)
# Una caja de pizza abierta en la mesa de atras (la del evento de la pizza ya llego antes).
caja_m(f, "pizza_caja", "carton", 3.6, 4.1, 0.77, 0.8, 2.6, 3.1)
caja_m(f, "pizza_caja", "carton", 3.6, 4.1, 0.8, 1.25, 3.08, 3.12)
for i in range(-5, 6):
    for j in range(-5, 6):
        if math.hypot(i, j) <= 5 and not (i > 0 and j > 0):
            x, z = 3.85 + i * 0.04, 2.85 + j * 0.04
            mat = "pizza_orilla" if math.hypot(i, j) > 4 else ("pepperoni" if (i * 3 + j * 5) % 7 == 0 else "pizza_queso")
            caja_m(f, "pizza", mat, x, x + 0.04, 0.8, 0.82, z, z + 0.04)

por_material(g)
por_material(f)
m = Modelo()
g.mesh(m)
f.mesh(m)
ejes = f"# reloj: {RELOJ[0]:.3f} {RELOJ[1]:.3f} {RELOJ[2]:.3f}   (las manecillas giran en Z alrededor de aqui)\n"
lugares = "".join(f"# escritorio {n}: {x:.2f} {z:.2f}\n" for n, (x, z) in enumerate(ESCRITORIOS))
m.write("lab_cit", f"""# El laboratorio de computo del CIT (UVG), la noche antes de la entrega. Origen al centro del piso,
# el frente (pizarron, proyector y reloj) hacia -Z. El escritorio 1 es el del protagonista.
{ejes}{lugares}""",
        mtl(("piso_a", (0.55, 0.55, 0.55), None, 0.4, 30), ("piso_b", (0.42, 0.43, 0.45), None, 0.4, 30), ("techo", (0.75, 0.75, 0.72), None),
            ("pared", (0.82, 0.80, 0.72), None), ("pared_baja", (0.25, 0.45, 0.30), None), ("franja", (0.90, 0.75, 0.25), None),
            ("lampara", (1.0, 1.0, 0.95), (0.85, 0.88, 0.80)), ("lampara_apagada", (0.45, 0.46, 0.48), None),
            ("salida_caja", (0.05, 0.25, 0.10), (0.02, 0.15, 0.05)), ("salida_luz", (0.3, 1.0, 0.45), (0.25, 0.95, 0.4)),
            ("cartel_blanco", (0.95, 0.95, 0.92), None), ("cartel_rojo", (0.85, 0.12, 0.12), None),
            ("pizza_queso", (0.98, 0.80, 0.30), None), ("pizza_orilla", (0.80, 0.50, 0.20), None), ("pepperoni", (0.70, 0.12, 0.08), None),
            ("afiche", (0.12, 0.10, 0.30), (0.03, 0.02, 0.08)), ("afiche_linea", (0.40, 0.95, 0.95), (0.25, 0.6, 0.6)),
            ("mochila_roja", (0.75, 0.15, 0.18), None), ("mochila_azul", (0.15, 0.30, 0.70), None), ("mochila_negra", (0.12, 0.12, 0.14), None),
            ("carton", (0.72, 0.55, 0.35), None), ("aluminio", (0.70, 0.72, 0.75), None, 0.8, 64), ("pizarron", (0.95, 0.95, 0.93), None, 0.6, 50),
            ("marcador_rojo", (0.85, 0.10, 0.12), (0.15, 0.0, 0.0)), ("tela", (0.92, 0.93, 0.95), (0.05, 0.05, 0.06)), ("plastico_negro", (0.08, 0.08, 0.10), None, 0.5, 40),
            ("lente", (0.3, 0.5, 0.9), (0.2, 0.3, 0.6)), ("reloj_cara", (0.97, 0.97, 0.95), (0.05, 0.05, 0.05)), ("reloj_borde", (0.15, 0.15, 0.18), None),
            ("negro", (0.05, 0.05, 0.06), None), ("rojo_reloj", (0.85, 0.12, 0.12), None), ("verde_uvg", (0.10, 0.50, 0.25), (0.02, 0.08, 0.04)),
            ("blanco", (0.98, 0.98, 0.98), (0.1, 0.1, 0.1)), ("noche", (0.04, 0.05, 0.12), (0.02, 0.03, 0.07)), ("luz_lejos", (1.0, 0.85, 0.5), (0.9, 0.7, 0.35)),
            ("gota", (0.6, 0.7, 0.9), (0.15, 0.18, 0.25)), ("puerta", (0.50, 0.33, 0.18), None), ("cafetera", (0.55, 0.12, 0.10), None, 0.5, 40),
            ("cafetera_luz", (1.0, 0.75, 0.35), (0.9, 0.6, 0.2)), ("boton_verde", (0.2, 0.9, 0.3), (0.2, 0.8, 0.3)), ("boton_rojo", (0.9, 0.2, 0.2), (0.8, 0.15, 0.15)),
            ("basurero", (0.30, 0.32, 0.35), None), ("mesa", (0.85, 0.83, 0.78), None, 0.3, 30), ("plastico_gris", (0.35, 0.36, 0.40), None),
            ("luz_cpu", (0.2, 0.6, 1.0), (0.2, 0.6, 1.0)), ("silla", (0.15, 0.18, 0.30), None), ("lata", (0.2, 0.75, 0.3), None, 0.8, 60),
            ("taza", (0.95, 0.95, 0.95), None),
            ("pantalla_mia", (0.05, 0.06, 0.09), (0.07, 0.08, 0.12)), ("pantalla_codigo", (0.05, 0.06, 0.09), (0.07, 0.09, 0.14)),
            ("pantalla_azul", (0.10, 0.30, 0.85), (0.08, 0.25, 0.75)), ("pantalla_juego", (0.10, 0.25, 0.45), (0.10, 0.22, 0.42)),
            ("pantalla_negra", (0.02, 0.02, 0.02), (0.01, 0.01, 0.01)), ("pantalla_youtube", (0.08, 0.08, 0.08), (0.10, 0.10, 0.10))))
