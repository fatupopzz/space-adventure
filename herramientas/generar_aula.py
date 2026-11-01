# Genera el salon de clases de la UVG (modelos/aula_uvg.obj) y el profe robot (modelos/profe_robot.obj).
# Uso: python3 herramientas/generar_aula.py
# Medidas en metros. Y arriba, el pizarron esta al frente (-Z) y los pupitres miran hacia el.
# El salon va de x -4 a 4, de z -3.5 a 3.5 y de y 0 a 3.

from modelo3d import *

# ---------- letras de 3x5 para el pizarron y los letreros ----------
LETRAS = {
    "A": [".#.", "#.#", "###", "#.#", "#.#"], "B": ["##.", "#.#", "##.", "#.#", "##."],
    "C": [".##", "#..", "#..", "#..", ".##"], "D": ["##.", "#.#", "#.#", "#.#", "##."],
    "E": ["###", "#..", "##.", "#..", "###"], "G": [".##", "#..", "#.#", "#.#", ".##"],
    "I": ["###", ".#.", ".#.", ".#.", "###"], "L": ["#..", "#..", "#..", "#..", "###"],
    "M": ["#.#", "###", "###", "#.#", "#.#"], "N": ["##.", "#.#", "#.#", "#.#", "#.#"],
    "O": ["###", "#.#", "#.#", "#.#", "###"], "P": ["##.", "#.#", "##.", "#..", "#.."],
    "Q": ["###", "#.#", "#.#", "###", "..#"], "R": ["##.", "#.#", "##.", "#.#", "#.#"],
    "S": [".##", "#..", ".#.", "..#", "##."], "T": ["###", ".#.", ".#.", ".#.", ".#."],
    "U": ["#.#", "#.#", "#.#", "#.#", "###"], "V": ["#.#", "#.#", "#.#", "#.#", ".#."],
    "0": ["###", "#.#", "#.#", "#.#", "###"], "1": [".#.", "##.", ".#.", ".#.", "###"],
    "2": ["##.", "..#", ".#.", "#..", "###"], "3": ["##.", "..#", ".#.", "..#", "##."],
    "(": [".#", "#.", "#.", "#.", ".#"], ")": ["#.", ".#", ".#", ".#", "#."],
    "|": ["#", "#", "#", "#", "#"], "*": ["...", "#.#", ".#.", "#.#", "..."],
    "=": ["...", "###", "...", "###", "..."], "-": ["...", "...", "###", "...", "..."],
    " ": ["..", "..", "..", "..", ".."],
}

def ancho_texto(s, escala=1):
    return sum((len(LETRAS[c][0]) + 1) * escala for c in s) - escala

def texto(s, poner, escala=1):
    """Llama poner(columna, fila) por cada pixel encendido; fila 0 arriba, crece hacia abajo."""
    x = 0
    for c in s:
        g = LETRAS[c]
        for r, fila in enumerate(g):
            for col, ch in enumerate(fila):
                if ch == "#":
                    for a in range(escala):
                        for b in range(escala):
                            poner(x + col * escala + a, r * escala + b)
        x += (len(g[0]) + 1) * escala

class Grilla:
    """Voxels en metros: box_m llena una caja dada en metros, redondeando a la grilla."""
    def __init__(self, u, origen=(0.0, 0.0, 0.0)):
        self.vx = Voxels(u, origen)
        self.u, self.o = u, origen
    def idx(self, v, eje):
        return round((v - self.o[eje]) / self.u)
    def box_m(self, parte, mat, x0, x1, y0, y1, z0, z1, si=None):
        self.vx.box(parte, mat, self.idx(x0, 0), self.idx(x1, 0) - 1, self.idx(y0, 1), self.idx(y1, 1) - 1,
                    self.idx(z0, 2), self.idx(z1, 2) - 1, si=si)

MTL_AULA = """# Materiales del salon

newmtl pizarra_dibujo
Kd 1.0 1.0 1.0
Ke 0.25 0.25 0.25
map_Kd texturas/pizarra.png

newmtl piso_a
Kd 0.82 0.80 0.76
Ks 0.3 0.3 0.3
Ns 32

newmtl piso_b
Kd 0.60 0.58 0.56
Ks 0.3 0.3 0.3
Ns 32

newmtl pared
Kd 0.82 0.80 0.72
Ks 0.05 0.05 0.05
Ns 4

newmtl zocalo
Kd 0.22 0.55 0.42
Ks 0.2 0.2 0.2
Ns 16

newmtl techo
Kd 0.80 0.80 0.80
Ks 0 0 0
Ns 1

newmtl lampara
Kd 1.00 1.00 0.95
Ks 0 0 0
Ns 1
Ke 1.00 0.98 0.90

newmtl pizarra
Kd 0.97 0.98 1.00
Ks 0.6 0.6 0.6
Ns 64

newmtl marco
Kd 0.70 0.72 0.76
Ks 0.8 0.8 0.8
Ns 64

newmtl marcador_azul
Kd 0.10 0.22 0.70
Ks 0 0 0
Ns 1

newmtl marcador_rojo
Kd 0.80 0.12 0.18
Ks 0 0 0
Ns 1

newmtl marcador_verde
Kd 0.10 0.55 0.25
Ks 0 0 0
Ns 1

newmtl letrero
Kd 0.12 0.40 0.30
Ks 0.2 0.2 0.2
Ns 16

newmtl madera
Kd 0.70 0.50 0.32
Ks 0.2 0.2 0.2
Ns 16

newmtl metal
Kd 0.40 0.42 0.46
Ks 0.8 0.8 0.8
Ns 64

newmtl silla
Kd 0.25 0.62 0.55
Ks 0.3 0.3 0.3
Ns 24

newmtl puerta
Kd 0.50 0.32 0.20
Ks 0.2 0.2 0.2
Ns 16

newmtl laptop
Kd 0.25 0.25 0.28
Ks 0.6 0.6 0.6
Ns 48

newmtl laptop_pantalla
Kd 0.40 0.70 1.00
Ks 0 0 0
Ns 1
Ke 0.25 0.45 0.70

newmtl cartel
Kd 1.00 1.00 0.96
Ks 0 0 0
Ns 1

newmtl tinta
Kd 0.08 0.08 0.10
Ks 0 0 0
Ns 1

newmtl cuaderno_rosa
Kd 0.95 0.40 0.62
Ks 0.1 0.1 0.1
Ns 8

newmtl cuaderno_azul
Kd 0.30 0.50 0.95
Ks 0.1 0.1 0.1
Ns 8

newmtl cuaderno_amarillo
Kd 0.98 0.82 0.25
Ks 0.1 0.1 0.1
Ns 8

newmtl cielo_gt
Kd 0.55 0.78 1.00
Ks 0 0 0
Ns 1
Ke 0.40 0.62 0.90

newmtl nube_gt
Kd 1.00 1.00 1.00
Ks 0 0 0
Ns 1
Ke 0.85 0.88 0.92

newmtl volcan_gt
Kd 0.35 0.45 0.55
Ks 0 0 0
Ns 1
Ke 0.25 0.32 0.42

newmtl cerro_gt
Kd 0.30 0.62 0.30
Ks 0 0 0
Ns 1
Ke 0.18 0.40 0.18
"""

m = Modelo()

# ---------- cuarto (voxel de 0.1) ----------
cuarto = Grilla(0.1)
c = cuarto.vx
for i in range(-41, 41):
    for k in range(-36, 36):
        c.put(i, -1, k, "piso_a" if ((i // 6) + (k // 6)) % 2 == 0 else "piso_b", "piso_a" if ((i // 6) + (k // 6)) % 2 == 0 else "piso_b")
        c.put(i, 30, k, "techo", "techo")
for j in range(0, 30):
    parte = "zocalo" if j < 9 else "pared"
    for k in range(-36, 36):
        # Hueco de la ventana en la pared izquierda.
        if not (-25 <= k <= 4 and 10 <= j <= 21):
            c.put(-41, j, k, parte, parte)
        c.put(40, j, k, parte, parte)
    for i in range(-41, 41):
        c.put(i, j, -36, parte, parte)
        c.put(i, j, 35, parte, parte)
# marco de la ventana con su cruz
for k in range(-26, 6):
    for j in (9, 22, 16):
        c.put(-41, j, k, "marco", "marco")
for j in range(9, 23):
    for k in (-26, 5, -10):
        c.put(-41, j, k, "marco", "marco")
# lamparas en el techo
for x0 in (-2.6, 1.4):
    for z0 in (-2.0, 1.2):
        cuarto.box_m("lampara", "lampara", x0, x0 + 1.2, 2.9, 3.0, z0, z0 + 0.5)
# pizarron con marco
cuarto.box_m("pizarra", "pizarra", -2.5, 2.5, 0.9, 2.3, -3.5, -3.4)
cuarto.box_m("marco", "marco", -2.6, 2.6, 0.8, 0.9, -3.5, -3.3)
cuarto.box_m("marco", "marco", -2.6, 2.6, 2.3, 2.4, -3.5, -3.4)
cuarto.box_m("marco", "marco", -2.6, -2.5, 0.9, 2.3, -3.5, -3.4)
cuarto.box_m("marco", "marco", 2.5, 2.6, 0.9, 2.3, -3.5, -3.4)
# puerta en la pared derecha
cuarto.box_m("puerta", "puerta", 3.9, 4.0, 0.0, 2.1, 1.6, 2.6)
cuarto.box_m("metal", "metal", 3.8, 3.9, 1.0, 1.1, 1.7, 1.8)
c.mesh(m)

# ---------- paisaje por la ventana: el Volcan de Agua ----------
paisaje = Grilla(0.1)
p = paisaje.vx
for k in range(-50, 30):
    pico = 34 - abs(k + 10) * 0.75
    for j in range(-5, 42):
        if j < 13 + 2 * (((k * 7) % 5) == 0):
            mat = "cerro_gt"
        elif j < pico:
            mat = "volcan_gt"
        elif 30 < j < 33 and (k % 11) in range(0, 6):
            mat = "nube_gt"
        else:
            mat = "cielo_gt"
        p.put(-56, j, k, mat, mat)
p.mesh(m)

# ---------- pupitres: tres filas de tres ----------
muebles = Grilla(0.05)
def pupitre(x, z):
    # silla
    muebles.box_m("silla", "silla", x - 0.22, x + 0.22, 0.40, 0.45, z - 0.20, z + 0.22)
    muebles.box_m("silla", "silla", x - 0.22, x + 0.22, 0.45, 0.85, z + 0.17, z + 0.22)
    for dx in (-0.2, 0.15):
        for dz in (-0.18, 0.15):
            muebles.box_m("metal", "metal", x + dx, x + dx + 0.05, 0.0, 0.40, z + dz, z + dz + 0.05)
    # mesa
    muebles.box_m("madera", "madera", x - 0.32, x + 0.32, 0.70, 0.75, z - 0.65, z - 0.22)
    for dx in (-0.3, 0.25):
        muebles.box_m("metal", "metal", x + dx, x + dx + 0.05, 0.0, 0.70, z - 0.60, z - 0.55)
COLORES_CUADERNO = ["cuaderno_rosa", "cuaderno_azul", "cuaderno_amarillo"]
for n, x in enumerate((-2.4, 0.0, 2.4)):
    for m_, z in enumerate((-0.9, 0.8, 2.5)):
        pupitre(x, z)
        c_ = COLORES_CUADERNO[(n + m_) % 3]
        muebles.box_m(c_, c_, x - 0.15, x + 0.10, 0.75, 0.80, z - 0.55, z - 0.35)
# escritorio del profe con su laptop
muebles.box_m("madera", "madera", -2.8, -1.2, 0.72, 0.78, -2.9, -2.2)
muebles.box_m("madera", "madera", -2.8, -1.2, 0.0, 0.72, -2.9, -2.85)
muebles.box_m("madera", "madera", -2.8, -2.75, 0.0, 0.72, -2.9, -2.2)
muebles.box_m("madera", "madera", -1.25, -1.2, 0.0, 0.72, -2.9, -2.2)
muebles.box_m("laptop", "laptop", -2.2, -1.75, 0.78, 0.80, -2.75, -2.45)
muebles.box_m("laptop", "laptop", -2.2, -1.75, 0.80, 1.10, -2.80, -2.75)
muebles.box_m("laptop_pantalla", "laptop_pantalla", -2.15, -1.80, 0.83, 1.07, -2.75, -2.70)
muebles.vx.mesh(m)

# ---------- lo escrito en el pizarron: un rectangulo con TEXTURA ----------
# Un solo quad con coordenadas UV, apenas salido del pizarron (z = -3.4), con la textura
# texturas/pizarra.png (la dibuja generar_texturas.py). El fragment shader la muestrea en la UV
# interpolada de cada pixel. Visto desde el salon (mirando a -Z) la derecha es +X: u crece con x.
X0, X1, Y0, Y1, ZP = -2.45, 2.45, 0.95, 2.25, -3.395
m.part("pizarra_dibujo", "pizarra_dibujo").quad(
    [(X0, Y0, ZP), (X1, Y0, ZP), (X1, Y1, ZP), (X0, Y1, ZP)], (0.0, 1.6, -4.0),
    uvs=[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)])

# ---------- letrero sobre el pizarron y cartel de la maquina de Turing ----------
letrero = Grilla(0.03, origen=(0.0, 0.0, -3.5))
lv = letrero.vx
s = "UVG - TEORIA DE LA COMPUTACION"
i0 = letrero.idx(-ancho_texto(s) * 0.03 / 2, 0)
j0 = letrero.idx(2.66, 1)
texto(s, lambda col, fila: lv.put(i0 + col, j0 - fila, 0, "letrero", "letrero"))
lv.mesh(m)

cartel = Grilla(0.03, origen=(3.97, 0.0, 0.0))
cv = cartel.vx
# Una cinta de 8 celdas en la pared derecha: 0 1 1 0 1 0 0 1, con la cabeza en la cuarta.
# Vista desde adentro del salon (mirando hacia +X), la derecha es +Z: se escribe hacia +Z.
z_izq, y_arr = -2.3, 2.2
for n, digito in enumerate("01101001"):
    k0 = cartel.idx(z_izq + n * 0.27, 2)
    j0 = cartel.idx(y_arr, 1)
    for a in range(9):
        for b in range(9):
            borde = a in (0, 8) or b in (0, 8)
            cv.put(0, j0 - b, k0 + a, "tinta" if borde else "cartel", "tinta" if borde else "cartel")
    texto(digito, lambda col, fila: cv.put(-1, j0 - 2 - fila, k0 + 3 + col, "tinta", "tinta"))
k_cab = cartel.idx(z_izq + 3 * 0.27, 2) + 4
for d in range(4):
    for a in range(-d, d + 1):
        cv.put(0, cartel.idx(y_arr, 1) - 11 - d, k_cab + a, "marcador_rojo", "marcador_rojo")
cv.mesh(m)

m.write("aula_uvg", """# Salon de clases de la UVG (voxel). Metros; Y arriba; el pizarron esta en z = -3.4 mirando hacia +Z.
# Los pupitres estan en x = -2.4, 0, 2.4 y z = -0.9, 0.8, 2.5 (la silla; la mesa queda adelante).
# La luz del salon sale de las lamparas del techo: usa (0, 2.8, 0.5) como posicion de la luz.
""", MTL_AULA)

# ---------- el profe (mira hacia -Z; origen en el piso) ----------
# Un profe nerd: lentes, camisa de manga corta con chaleco de rombos, corbata,
# tres circulos tatuados en el antebrazo izquierdo y su regla.
MTL_PROFE = """# Materiales del profe

newmtl piel_profe
Kd 0.93 0.76 0.62
Ks 0.1 0.1 0.1
Ns 8

newmtl pelo_profe
Kd 0.09 0.07 0.06
Ks 0.3 0.3 0.3
Ns 24

newmtl camisa
Kd 0.10 0.10 0.13
Ks 0.25 0.25 0.25
Ns 20

newmtl chaleco
Kd 0.08 0.08 0.11
Ks 0.1 0.1 0.1
Ns 8

newmtl rombo
Kd 0.08 0.08 0.11
Ks 0.1 0.1 0.1
Ns 8

newmtl corbata
Kd 0.20 0.32 0.24
Ks 0.3 0.3 0.3
Ns 16

newmtl pantalon
Kd 0.14 0.14 0.17
Ks 0.1 0.1 0.1
Ns 8

newmtl zapato
Kd 0.12 0.10 0.10
Ks 0.5 0.5 0.5
Ns 32

newmtl lentes
Kd 0.06 0.06 0.08
Ks 0.6 0.6 0.6
Ns 64

newmtl tatuaje
Kd 0.12 0.12 0.20
Ks 0 0 0
Ns 1

newmtl ceja
Kd 0.20 0.13 0.08
Ks 0 0 0
Ns 1

newmtl cristal
Kd 0.75 0.90 1.00
Ks 1.0 1.0 1.0
Ns 128
d 0.30

newmtl brillo_lente
Kd 1.00 1.00 1.00
Ks 0 0 0
Ns 1
Ke 0.80 0.85 0.90
d 0.80

newmtl regla
Kd 0.85 0.68 0.38
Ks 0.2 0.2 0.2
Ns 16
""" + MTL_CARA

r = Modelo()
pr = Grilla(0.025)
# piernas y zapatos
for x0 in (-0.175, 0.025):
    pr.box_m("zapatos", "zapato", x0, x0 + 0.15, 0.0, 0.05, -0.12, 0.08)
    pr.box_m("pantalon", "pantalon", x0, x0 + 0.15, 0.05, 0.85, -0.075, 0.075)
# torso: camisa, chaleco de rombos, corbata
pr.box_m("camisa", "camisa", -0.225, 0.225, 0.85, 1.40, -0.10, 0.10)
def rombo(i, j, k):
    return (abs(((i + 200) % 6) - 3) + abs(((j + 200) % 6) - 3)) <= 1
pr.box_m("chaleco", "chaleco", -0.225, 0.225, 0.85, 1.30, -0.125, -0.10, si=lambda i, j, k: not rombo(i, j, k))
pr.box_m("rombos", "rombo", -0.225, 0.225, 0.85, 1.30, -0.125, -0.10, si=rombo)
pr.box_m("chaleco", "chaleco", -0.225, 0.225, 0.85, 1.30, 0.10, 0.125)
pr.box_m("corbata", "corbata", -0.025, 0.025, 1.05, 1.40, -0.15, -0.125)
pr.box_m("camisa", "camisa", -0.075, 0.075, 1.30, 1.40, -0.125, -0.10)
# brazos: manga corta y antebrazos con piel
pr.box_m("camisa", "camisa", -0.30, -0.225, 1.20, 1.40, -0.075, 0.075)
pr.box_m("camisa", "camisa", 0.225, 0.30, 1.20, 1.40, -0.075, 0.075)
pr.box_m("camisa", "camisa", -0.30, -0.225, 0.92, 1.20, -0.075, 0.075)
pr.box_m("brazos", "piel_profe", -0.30, -0.225, 0.85, 0.92, -0.075, 0.075)
# el brazo derecho se dobla hacia adelante con la regla
pr.box_m("brazo_regla_manga", "camisa", 0.225, 0.30, 1.05, 1.20, -0.075, 0.075)
pr.box_m("brazo_regla_manga", "camisa", 0.225, 0.30, 1.05, 1.125, -0.25, -0.075)
pr.box_m("brazo_regla", "piel_profe", 0.225, 0.30, 1.05, 1.125, -0.30, -0.25)
pr.box_m("regla", "regla", 0.24, 0.285, 1.10, 1.135, -0.80, -0.30)
# cuello y cabeza
pr.box_m("cuello", "piel_profe", -0.05, 0.05, 1.40, 1.45, -0.05, 0.05)
pr.box_m("cuello_chumpa", "camisa", -0.125, 0.125, 1.40, 1.475, -0.10, 0.10, si=lambda i, j, k: not (-2 <= i <= 1 and k <= -3))
pr.box_m("cabeza", "piel_profe", -0.15, 0.15, 1.45, 1.78, -0.15, 0.15)
pr.box_m("pelo", "pelo_profe", -0.175, 0.175, 1.75, 1.85, -0.175, 0.175)
pr.box_m("pelo", "pelo_profe", -0.175, 0.175, 1.55, 1.75, 0.15, 0.175)
pr.box_m("pelo", "pelo_profe", -0.175, -0.15, 1.58, 1.75, -0.15, 0.15)
pr.box_m("pelo", "pelo_profe", 0.15, 0.175, 1.58, 1.75, -0.15, 0.15)
# nariz y orejas
pr.box_m("cabeza", "piel_profe", -0.175, -0.15, 1.56, 1.66, -0.025, 0.025)
pr.box_m("cabeza", "piel_profe", 0.15, 0.175, 1.56, 1.66, -0.025, 0.025)
# copete despeinado
pr.box_m("pelo", "pelo_profe", -0.15, 0.05, 1.70, 1.75, -0.175, -0.15)
pr.box_m("pelo", "pelo_profe", -0.10, 0.0, 1.85, 1.875, -0.125, 0.05)
pr.vx.mesh(r)

# (Antes tenia tres circulos tatuados en el antebrazo; ahora lleva chumpa de manga larga.)

# La cara del profe no es chibi: ojos chiquitos detras de los lentes, cejas y boca.
# Pixeles de 0.01; x = 0 al centro, el ojo derecho va de x 3 a 5, la boca abajo.
CEJA_RECTA = espejo(pixeles(["####"], 2, 13))
CARAS_PROFE = {
    "feliz": [("ojos", "ojo", espejo(pixeles(["##", "##"], 3, 9))), ("cejas", "ceja", CEJA_RECTA),
              ("boca", "boca", pixeles(["#....#", ".####."], -3, 3))],
    "enamorada": [("ojos", "ojo", espejo(pixeles(["#..#", ".##."], 2, 9))), ("cejas", "ceja", CEJA_RECTA),
                  ("boca", "boca", pixeles(["#......#", ".######.", "..####.."], -4, 4)), ("rubor", "rubor", espejo(pixeles(["##"], 6, 3)))],
    "asustada": [("ojos", "ojo", espejo(pixeles(["##", "##"], 3, 9))), ("cejas", "ceja", espejo(pixeles(["####"], 2, 14))),
                 ("boca", "boca", pixeles(["##", "##"], -1, 3))],
    "espiral": [("ojos", "ojo", espejo(pixeles(["#.#", ".#.", "#.#"], 3, 10))), ("cejas", "ceja", CEJA_RECTA),
                ("boca", "ojo", pixeles(["#.#.#.", ".#.#.#"], -3, 3))],
    "dormida": [("ojos", "ojo", espejo(pixeles(["###"], 3, 8))), ("cejas", "ceja", CEJA_RECTA),
                ("boca", "boca", pixeles(["##"], -1, 3))],
    "enojado": [("ojos", "ojo", espejo(pixeles(["##", "##"], 3, 9))), ("cejas", "ceja", espejo(pixeles(["...###", ".##...", "#....."], 1, 13))),
                ("boca", "boca", pixeles([".####.", "#....#"], -3, 3))],
    "serio": [("ojos", "ojo", espejo(pixeles(["##"], 3, 8))), ("cejas", "ceja", espejo(pixeles(["####"], 2, 12))),
              ("boca", "ojo", pixeles(["######"], -3, 2))],
}

# La cara: las expresiones y, delante de ellas, los lentes.
PX, Y_CARA, Z_CARA = 0.01, 1.53, -0.15
expresiones(r, z_frente=Z_CARA, px=PX, x_centro=0.0, y_base=Y_CARA, caras=CARAS_PROFE)
lentes = r.part("lentes", "lentes")
zl = Z_CARA - 0.012
def tira(x0, x1, y0, y1):
    lentes.quad([(x0, y0, zl), (x1, y0, zl), (x1, y1, zl), (x0, y1, zl)], (0.0, y0, 0.0))
# Cada lente rodea su ojo: el ojo ocupa los pixeles 2 a 6 en x y 4 a 10 en y.
for lado in (-1, 1):
    a, b = sorted((lado * 0.5 * PX, lado * 7.5 * PX))
    y0, y1 = Y_CARA + 2.5 * PX, Y_CARA + 11.5 * PX
    g = 0.8 * PX
    tira(a, b, y1 - g, y1); tira(a, b, y0, y0 + g)      # arriba y abajo del marco
    tira(a, a + g, y0, y1); tira(b - g, b, y0, y1)      # lados
tira(-0.5 * PX, 0.5 * PX, Y_CARA + 9 * PX, Y_CARA + 10 * PX)    # el puente
# Los cristales: transparentes, entre los ojos y el marco, con un brillito.
cristal = r.part("lentes_cristal", "cristal")
zc = Z_CARA - 0.009
for lado in (-1, 1):
    a_, b_ = sorted((lado * 0.5 * PX, lado * 7.5 * PX))
    y0, y1 = Y_CARA + 2.5 * PX, Y_CARA + 11.5 * PX
    cristal.quad([(a_, y0, zc), (b_, y0, zc), (b_, y1, zc), (a_, y1, zc)], (0.0, y0, 0.0))
brillo = r.part("lentes_brillo", "brillo_lente")
zb = Z_CARA - 0.010
for lado in (-1, 1):
    x0 = lado * 5.2 * PX if lado > 0 else lado * 6.2 * PX
    brillo.quad([(x0, Y_CARA + 9 * PX, zb), (x0 + PX, Y_CARA + 9 * PX, zb), (x0 + PX, Y_CARA + 10.5 * PX, zb), (x0, Y_CARA + 10.5 * PX, zb)], (0.0, 0.0, 0.0))
pr2 = Grilla(0.0125)
pr2.box_m("nariz", "piel_profe", -0.0125, 0.0125, 1.5625, 1.6, -0.1625, -0.15)
pr2.box_m("lentes_patas", "lentes", -0.1625, -0.15, 1.625, 1.6375, -0.15, 0.05)
pr2.box_m("lentes_patas", "lentes", 0.15, 0.1625, 1.625, 1.6375, -0.15, 0.05)
pr2.vx.mesh(r)

r.write("profe", """# El profe (voxel). Mira hacia -Z, origen en el piso, mide 1.87. Lleva su regla en la mano derecha.
# Expresiones en su cara: partes cara_<emocion>_* (feliz, asustada, espiral, enamorada, dormida, enojado, serio).
""", MTL_PROFE)
