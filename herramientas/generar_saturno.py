# Genera modelos/saturno_escenario.obj + .mtl en estilo voxel: el escenario de Saturno Ritmo (los
# anillos de vinil, la puerta de juicio, las notas, el escenario, las bocinas y el ecualizador),
# con las mismas partes, posiciones y materiales del original. El planeta en si (las franjas) lo
# arma el juego (src/planetas.rs) y lo junta con este escenario.
# Uso: python3 herramientas/generar_saturno.py
# Centrado en el origen, radio 2, Y arriba, anillos en el plano XZ. La puerta de juicio esta en -Z (angulo 0).
# Angulo de un punto: atan2(x, -z).

import math
from modelo3d import *

MTL = """# Materiales de Saturno Ritmo

newmtl sat_a
Kd 0.98 0.80 0.55
Ks 0.3 0.3 0.3
Ns 16

newmtl sat_b
Kd 0.90 0.60 0.55
Ks 0.3 0.3 0.3
Ns 16

newmtl sat_c
Kd 0.72 0.58 0.92
Ks 0.3 0.3 0.3
Ns 16

newmtl vinil
Kd 0.07 0.06 0.10
Ks 0.9 0.9 0.9
Ns 128

newmtl surco
Kd 0.55 0.25 0.95
Ks 0.6 0.6 0.6
Ns 64
Ke 0.30 0.12 0.55

newmtl metal
Kd 0.62 0.64 0.72
Ks 1.0 1.0 1.0
Ns 96

newmtl oscuro
Kd 0.10 0.09 0.15
Ks 0.5 0.5 0.5
Ns 32

newmtl escenario
Kd 1.00 0.45 0.75
Ks 0.6 0.6 0.6
Ns 48

newmtl brillo
Kd 0.35 0.95 1.00
Ks 0 0 0
Ns 1
Ke 0.35 0.95 1.00

newmtl eq_cian
Kd 0.30 0.95 1.00
Ks 0 0 0
Ns 1
Ke 0.30 0.95 1.00

newmtl eq_rosa
Kd 1.00 0.40 0.80
Ks 0 0 0
Ns 1
Ke 1.00 0.40 0.80

newmtl eq_amarillo
Kd 1.00 0.85 0.25
Ks 0 0 0
Ns 1
Ke 1.00 0.85 0.25

newmtl eq_lima
Kd 0.60 1.00 0.35
Ks 0 0 0
Ns 1
Ke 0.60 1.00 0.35
"""

COLORES = ["eq_cian", "eq_rosa", "eq_amarillo", "eq_lima"]
m = Modelo()

def en_angulo(theta, r, y):
    """Punto a radio r y altura y, con angulo theta medido desde -Z hacia +X."""
    return (r*math.sin(theta), y, -r*math.cos(theta))

# ---------- el planeta (franja_0 a franja_9) lo arma el juego (src/planetas.rs) ----------

# ---------- anillos: disco de vinil de 0.1, de r = 2.5 a 4.0, alto de y = -0.05 a 0.05 ----------
anillos = Voxels(0.1, origen=(0.0, -0.05, 0.0))
BANDAS = [(2.5, 3.0, "vinil"), (3.0, 3.2, "surco"), (3.2, 3.5, "vinil"), (3.5, 3.7, "surco"), (3.7, 4.0, "vinil")]
for i in range(-40, 40):
    for k in range(-40, 40):
        x, _, z = anillos.centro(i, 0, k)
        r = math.hypot(x, z)
        for n, (r0, r1, mat) in enumerate(BANDAS):
            if r0 <= r < r1:
                anillos.put(i, 0, k, f"vinil_{n}", mat)
anillos.mesh(m)

# ---------- puerta de juicio y linea (en -Z, angulo 0) ----------
puerta = Voxels(0.1, origen=(-0.05, 0.05, 0.0))
puerta.box("puerta_poste_int", "eq_cian", 0, 0, 0, 10, -25, -25)
puerta.box("puerta_poste_ext", "eq_cian", 0, 0, 0, 10, -41, -41)
puerta.box("puerta_arco", "eq_cian", 0, 0, 11, 11, -41, -25)
puerta.mesh(m)
linea = Voxels(0.05, origen=(-0.025, 0.05, 0.0))
linea.box("linea_juicio", "eq_rosa", 0, 0, 0, 0, -79, -52)
linea.mesh(m)

# ---------- notas: bichitos de voxel que viajan por el anillo hacia la puerta ----------
# Viajan bajando el angulo; la cara (los ojos) mira hacia donde van.
NOTAS = [(0, "piso", -35, "eq_rosa"), (1, "aire", -10, "eq_cian"), (2, "piso", 10, "eq_amarillo"),
         (3, "piso", 35, "eq_lima"), (4, "aire", 55, "eq_rosa"), (5, "piso", 80, "eq_cian"),
         (6, "aire", 105, "eq_amarillo"), (7, "piso", 135, "eq_rosa")]
for n, carril, grados, mat in NOTAS:
    nota = Voxels(0.055)
    nombre = f"nota_{carril}_{n}"
    nota.box(f"{nombre}_cuerpo", mat, -3, 2, 0, 5, -3, 2,
             si=lambda i, j, k: (i in (-3, 2)) + (j in (0, 5)) + (k in (-3, 2)) < 2)
    nota.box(f"{nombre}_cuerpo", mat, -2, -2, 6, 6, -1, 0)   # antenitas
    nota.box(f"{nombre}_cuerpo", mat, 1, 1, 6, 6, -1, 0)
    nota.put(-2, 3, -4, f"{nombre}_ojo_izq", "oscuro")
    nota.put(1, 3, -4, f"{nombre}_ojo_der", "oscuro")
    theta = math.radians(grados)
    base = en_angulo(theta, 3.30, 0.05 if carril == "piso" else 0.58)
    nota.mesh(m, transform=rot_y(math.pi/2 - theta, base))

# ---------- escenario hexagonal en el polo norte ----------
def en_hexagono(x, z, R):
    q, w = abs(x), abs(z)
    return w <= R*math.sqrt(3)/2 and math.sqrt(3)*q + w <= math.sqrt(3)*R
esc = Voxels(0.1, origen=(0.0, 1.8, 0.0))
for i in range(-9, 9):
    for k in range(-9, 9):
        x, _, z = esc.centro(i, 0, k)
        if en_hexagono(x, z, 0.82):
            for j in range(3):
                borde = j == 2 and not en_hexagono(x, z, 0.70)
                esc.put(i, j, k, "escenario_borde" if borde else "escenario_hexagono",
                        "eq_cian" if borde else "escenario")
esc.mesh(m)

mic = Voxels(0.05, origen=(-0.025, 2.1, -0.225))
mic.box("microfono_pie", "metal", 0, 0, 0, 10, 0, 0)
mic.box("microfono", "oscuro", -1, 1, 11, 13, -1, 1)
mic.mesh(m)

for n in range(6):
    a = math.radians(60*n)
    pos = (0.72*math.cos(a) - 0.025, 2.1, 0.72*math.sin(a) - 0.025)
    ref = Voxels(0.05, origen=pos)
    ref.box(f"reflector_poste_{n}", "metal", 0, 0, 0, 7, 0, 0)
    ref.box(f"reflector_{n}", COLORES[n % 4], -1, 1, 8, 10, -1, 1)
    ref.mesh(m)

# ---------- bocinas en los costados del planeta (miran hacia afuera) ----------
for n, (grados, y) in enumerate([(130, 0.8), (-50, 0.8), (-140, -0.8), (40, -0.8)]):
    boc = Voxels(0.07, origen=(-0.28, -0.28, 0.0))
    boc.box(f"bocina_aro_{n}", "metal", 0, 7, 0, 7, -1, 1, si=lambda i, j, k: i in (0, 7) or j in (0, 7) or k == 1)
    boc.box(f"bocina_cono_{n}", "oscuro", 1, 6, 1, 6, 0, 0, si=lambda i, j, k: not (i in (3, 4) and j in (3, 4)))
    boc.box(f"bocina_centro_{n}", "brillo", 3, 4, 3, 4, -1, -1)
    theta = math.radians(grados)
    r = math.sqrt(4 - y*y) + 0.12
    boc.mesh(m, transform=rot_y(-theta, en_angulo(theta, r, y)))

# ---------- ecualizador: barras de pie alrededor de los polos ----------
ALTOS = [5, 6, 4, 3, 4, 4, 4, 3, 3, 4, 5, 5, 4, 4, 5, 6, 4, 3]
comentarios = {}
for polo, signo, y_base in (("norte", 1, 1.2), ("sur", -1, -1.2)):
    for n in range(18):
        nombre = f"eq_{polo}_{n:02d}"
        theta = math.radians(90 + 20*n)
        barra = Voxels(0.1, origen=(-0.1, 0.0, -0.1))
        alto = ALTOS[n] if polo == "norte" else ALTOS[(n + 7) % 18]
        if signo > 0:
            barra.box(nombre, COLORES[n % 4], 0, 1, 0, alto - 1, 0, 1)
        else:
            barra.box(nombre, COLORES[n % 4], 0, 1, -alto, -1, 0, 1)
        barra.mesh(m, comentarios={nombre: f"crece hacia {'+' if signo > 0 else '-'}Y desde y = {y_base:.2f}: escala Y alrededor de esa base"},
                   transform=rot_y(-theta, en_angulo(theta, 1.55, y_base)))

m.write("saturno_escenario", """# El escenario de Saturno Ritmo (voxel). Centrado en el origen, el planeta (que arma el juego) es de radio 2, Y arriba, anillos en el plano XZ. Inclinalo 26.7 grados con su matriz de modelo.
# vinil_0..vinil_4 son los anillos (giran juntos en Y). La puerta de juicio esta en -Z; las notas viajan por el anillo bajando su angulo atan2(x, -z).
# Las barras eq_norte_* crecen hacia +Y y eq_sur_* hacia -Y: escalalas con el audio alrededor de su base (comentario de cada una).
""", MTL)
