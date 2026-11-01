# Genera modelos/nave_lila.obj + .mtl: la Nave Lila voxel, una idol alienigena que es su propia cabeza gigante.
# Uso: python3 herramientas/generar_nave.py
# Y arriba, mira hacia -Z, origen en el centro de la cabeza. izq/der desde su punto de vista: izq = -X.
# Voxel de 0.1; abajo, i = columna (X), j = fila (Y), k = profundidad (Z, negativo = frente).

from modelo3d import *

MTL = """# Materiales de la Nave Lila (voxel)

newmtl piel_lila
Kd 0.70 0.60 0.88
Ks 0.2 0.2 0.2
Ns 16

newmtl pelo_violeta
Kd 0.52 0.28 0.80
Ks 0.4 0.4 0.4
Ns 32

newmtl rojo
Kd 0.92 0.16 0.26
Ks 0.6 0.6 0.6
Ns 48

newmtl rojo_oscuro
Kd 0.55 0.08 0.16
Ks 0.4 0.4 0.4
Ns 32

newmtl rosa
Kd 1.00 0.45 0.75
Ks 0.4 0.4 0.4
Ns 32
Ke 0.20 0.06 0.14

newmtl metal
Kd 0.66 0.66 0.74
Ks 0.9 0.9 0.9
Ns 96

newmtl oscuro
Kd 0.10 0.08 0.14
Ks 0.3 0.3 0.3
Ns 16

newmtl estrella
Kd 1.00 0.86 0.30
Ks 0 0 0
Ns 1
Ke 0.60 0.50 0.10

newmtl laser
Kd 1.00 0.70 0.95
Ks 0 0 0
Ns 1
Ke 1.00 0.55 0.90

newmtl tobera_luz
Kd 1.00 0.50 0.80
Ks 0 0 0
Ns 1
Ke 1.00 0.40 0.75

newmtl llama
Kd 1.00 0.45 0.80
Ks 0 0 0
Ns 1
Ke 1.00 0.35 0.70
d 0.65

newmtl vidrio
Kd 0.75 0.90 1.00
Ks 1.0 1.0 1.0
Ns 128
d 0.25
""" + MTL_CARA

U = 0.1
vx = Voxels(U)
c = vx.centro

# ---------- cabeza: cubo redondeado (superelipsoide) con corte de idol ----------
def en_cabeza(i, j, k):
    x, y, z = c(i, j, k)
    return x**4 + (y/0.9)**4 + z**4 <= 1.0
def color_cabeza(i, j, k):
    x, y, z = c(i, j, k)
    if y > 0.45 or z > 0.25 or abs(x) > 0.75:
        return "pelo", "pelo_violeta"
    if z < -0.6:
        if y > 0.3 or (y > 0.2 and i in (-7, -4, -1, 0, 3, 6)):  # flequillo con puntas
            return "pelo", "pelo_violeta"
        if abs(x) > 0.55 and y > -0.3:                            # mechones a los lados de la cara
            return "pelo", "pelo_violeta"
    return "cabeza", "piel_lila"
vx.box("cabeza", "piel_lila", -10, 9, -9, 8, -10, 9, si=en_cabeza)
for key in list(vx.v):
    vx.v[key] = color_cabeza(*key)

# ---------- diadema y audifonos turbina ----------
def en_diadema(i, j, k):
    x, y, _ = c(i, j, k)
    f = ((abs(x)/1.1)**4 + (y/1.02)**4) ** 0.25
    return 0.93 < f < 1.07 and y > 0.3
vx.box("diadema", "rojo_oscuro", -12, 11, 3, 11, -3, -2, si=en_diadema)

esquina = lambda i, j, k: not (j in (-4, 3) and k in (-4, 3))
vx.box("audifono_{lado}", "rojo", 10, 10, -4, 3, -4, 3, espejo=True, si=lambda i, j, k: esquina(i, j, k) and (j in (-4, 3) or k in (-4, 3)))
vx.box("audifono_{lado}_fondo", "oscuro", 10, 10, -3, 2, -3, 2, espejo=True)
vx.box("audifono_{lado}_aro", "rosa", 11, 11, -4, 3, -4, 3, espejo=True, si=lambda i, j, k: esquina(i, j, k) and (j in (-4, 3) or k in (-4, 3)))
vx.box("audifono_{lado}_aspas", "metal", 11, 11, -3, 2, -3, 2, espejo=True, si=lambda i, j, k: j in (-1, 0) or k in (-1, 0))

# toberas atras de los audifonos y sus llamas
vx.box("tobera_{lado}", "metal", 10, 11, -2, 1, 4, 6, espejo=True)
vx.box("tobera_{lado}", "metal", 10, 11, -2, 1, 7, 7, espejo=True, si=lambda i, j, k: j in (-2, 1))
vx.box("tobera_{lado}_luz", "tobera_luz", 10, 11, -1, 0, 7, 7, espejo=True)
vx.box("llama_{lado}", "llama", 10, 11, -2, 1, 8, 9, espejo=True)
vx.box("llama_{lado}", "llama", 10, 11, -1, 0, 10, 11, espejo=True)
vx.box("llama_{lado}", "llama", 10, 11, -1, -1, 12, 13, espejo=True)

# ---------- microfono canon laser (sale del audifono der) ----------
vx.box("microfono_brazo", "rojo_oscuro", 10, 10, -5, -5, -9, -5)
vx.box("microfono_brazo", "rojo_oscuro", 8, 10, -6, -6, -10, -10)
vx.box("microfono_brazo", "rojo_oscuro", 7, 7, -6, -6, -11, -11)
vx.box("microfono_aro", "rosa", 5, 7, -7, -5, -12, -12)
vx.box("microfono", "metal", 5, 7, -7, -5, -14, -13)
vx.box("microfono_punta", "laser", 6, 6, -6, -6, -15, -15)

# ---------- antena con estrella (radar) ----------
vx.box("antena", "metal", 5, 5, 8, 11, -4, -4)
vx.box("antena", "metal", 6, 6, 12, 13, -5, -5)
vx.pattern("antena_estrella", "estrella", ["..#..", ".###.", "#####", ".###.", ".#.#."], 4, 18, -6)

# ---------- bracitos aleta ----------
vx.box("aleta_{lado}", "piel_lila", 9, 10, -7, -5, -2, 1, espejo=True)
vx.box("aleta_{lado}", "piel_lila", 11, 11, -8, -6, -1, 0, espejo=True)

# ---------- patitas con botas cohete ----------
vx.box("pierna_{lado}", "piel_lila", 3, 4, -12, -9, -1, 0, espejo=True)
vx.box("bota_{lado}", "rojo", 2, 5, -15, -14, -2, 1, espejo=True)
vx.box("bota_{lado}", "rojo", 2, 5, -15, -14, -4, -3, espejo=True)
vx.box("bota_{lado}_aro", "rosa", 2, 5, -13, -13, -2, 1, espejo=True)
vx.box("bota_{lado}_tobera", "metal", 3, 4, -16, -16, -1, 0, espejo=True)
vx.box("bota_{lado}_llama", "llama", 3, 4, -18, -17, -1, 0, espejo=True)
vx.box("bota_{lado}_llama", "llama", 3, 4, -20, -19, -1, -1, espejo=True)

# ---------- colita propulsora ----------
vx.box("colita", "piel_lila", -1, 0, -6, -5, 9, 11)
vx.box("colita", "piel_lila", -1, 0, -5, -3, 12, 12)
vx.box("colita_centro", "rosa", -1, 0, -3, -2, 13, 13)
vx.box("colita_helice", "metal", -3, 2, -3, -2, 14, 14)
vx.box("colita_helice", "metal", -1, 0, -5, 0, 14, 14)

# ---------- burbuja de la piloto ----------
def dist_burbuja(i, k):
    x, _, z = c(i, 0, k)
    return (x*x + (z - 0.3)**2) ** 0.5
vx.box("burbuja_base", "metal", -5, 4, 9, 9, -2, 8, si=lambda i, j, k: dist_burbuja(i, k) <= 0.4)
vx.box("burbuja_aro", "rosa", -5, 4, 10, 10, -2, 8, si=lambda i, j, k: 0.3 < dist_burbuja(i, k) <= 0.4)
def en_vidrio(i, j, k):
    x, y, z = c(i, j, k)
    return x*x + (y - 1.45)**2 + (z - 0.3)**2 <= 0.5**2 and (i, j, k) not in vx.v
vx.box("burbuja", "vidrio", -6, 5, 10, 19, -3, 8, si=en_vidrio)
piloto = (0.0, 1.0 + 0.20*0.5, 0.3)

# ---------- escribir ----------
m = Modelo()
animadas = {"audifono_der_aspas", "audifono_izq_aspas", "antena_estrella", "colita_helice",
            "llama_der", "llama_izq", "bota_der_llama", "bota_izq_llama"}
asp, est, hel = vx.centro_de("audifono_der_aspas"), vx.centro_de("antena_estrella"), vx.centro_de("colita_helice")
vx.mesh(m, animadas, {
    "audifono_der_aspas": f"turbina: gira alrededor del eje X que pasa por {asp[0]:.2f} {asp[1]:.2f} {asp[2]:.2f}",
    "audifono_izq_aspas": f"turbina: gira alrededor del eje X que pasa por {-asp[0]:.2f} {asp[1]:.2f} {asp[2]:.2f}",
    "antena_estrella": f"radar: gira en Y alrededor de {est[0]:.2f} {est[1]:.2f} {est[2]:.2f}",
    "colita_helice": f"helice: gira alrededor del eje Z que pasa por {hel[0]:.2f} {hel[1]:.2f} {hel[2]:.2f}",
    "llama_der": "escala en Z con el tiempo desde z = 0.80",
    "llama_izq": "escala en Z con el tiempo desde z = 0.80",
    "bota_der_llama": "escala en -Y con el tiempo desde y = -1.60",
    "bota_izq_llama": "escala en -Y con el tiempo desde y = -1.60",
    "microfono_punta": "canon: dispara hacia -Z desde aqui",
    "burbuja": "vidrio: dibujalo al final con blending, despues de la piloto",
})
expresiones(m, z_frente=-1.0, px=0.07, x_centro=0.0, y_base=-0.54, separacion=0.004)

m.write("nave_lila", f"""# Nave Lila (voxel de 0.1): la idol alienigena es la nave, su cabeza gigante es el cuerpo.
# Y arriba, mira hacia -Z, origen en el centro de la cabeza. izq = -X, der = +X.
# piloto: {piloto[0]:.3f} {piloto[1]:.3f} {piloto[2]:.3f} escala 0.5   (miku_piloto.obj se dibuja con esta traslacion y escala)
# Expresiones: dibuja solo las partes cara_<emocion>_* (feliz, asustada, espiral, enamorada, dormida, serio), en orden.
# Animables (pivote y eje en el comentario de cada una): audifono_*_aspas, antena_estrella, colita_helice,
#   llama_* y bota_*_llama. llamas, burbuja, rubor y sudor van con blending al final.
""", MTL)
