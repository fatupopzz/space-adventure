# Genera modelos/miku_piloto.obj + .mtl: Miku chibi voxel sentada manejando un volantito.
# Uso: python3 herramientas/generar_miku.py
# Y arriba, mira hacia -Z, origen en el centro del asiento. izq/der desde su punto de vista: izq = -X.
# Voxel de 0.04; abajo, i = columna (X), j = fila (Y, 0 = asiento), k = profundidad (Z, negativo = frente).

from modelo3d import *

MTL = """# Materiales de Miku piloto (voxel)

newmtl piel
Kd 1.00 0.86 0.76
Ks 0.1 0.1 0.1
Ns 8

newmtl pelo
Kd 0.25 0.80 0.76
Ks 0.4 0.4 0.4
Ns 32

newmtl pelo_puntas
Kd 0.12 0.58 0.60
Ks 0.4 0.4 0.4
Ns 32

newmtl gris
Kd 0.78 0.80 0.83
Ks 0.2 0.2 0.2
Ns 16

newmtl negro
Kd 0.16 0.16 0.20
Ks 0.3 0.3 0.3
Ns 24

newmtl acento
Kd 0.20 0.85 0.80
Ks 0.4 0.4 0.4
Ns 32
Ke 0.04 0.17 0.16

newmtl audifono
Kd 0.30 0.30 0.35
Ks 0.6 0.6 0.6
Ns 64

newmtl luz
Kd 0.30 0.95 0.90
Ks 0 0 0
Ns 1
Ke 0.30 0.95 0.90

newmtl liga_rosa
Kd 1.00 0.35 0.60
Ks 0.4 0.4 0.4
Ns 32

newmtl volante
Kd 0.72 0.58 0.92
Ks 0.5 0.5 0.5
Ns 48

newmtl asiento
Kd 0.45 0.33 0.62
Ks 0.2 0.2 0.2
Ns 8
""" + MTL_CARA

U = 0.04
vx = Voxels(U)

# ---------- asiento ----------
vx.box("asiento", "asiento", -5, 4, -2, -1, -4, 3)
vx.box("asiento", "asiento", -4, 3, 0, 8, 3, 4)

# ---------- cuerpo chiquito ----------
vx.box("falda_borde", "acento", -5, 4, 0, 0, -5, 2)
vx.box("falda", "negro", -4, 3, 1, 2, -5, 2)
vx.box("falda", "negro", -3, 2, 3, 3, -4, 1)
vx.box("torso", "gris", -3, 2, 4, 8, -2, 1)
vx.box("corbata", "acento", -1, 0, 6, 8, -3, -3)
vx.box("cuello", "piel", -1, 0, 9, 9, -1, 0)

# brazos hacia el volante (lado der; el izq sale en espejo)
vx.box("brazo_{lado}", "piel", 3, 3, 7, 8, -1, 0, espejo=True)
vx.box("manga_{lado}", "negro", 3, 3, 6, 7, -5, -2, espejo=True)
vx.box("manga_{lado}_borde", "acento", 3, 3, 6, 7, -6, -6, espejo=True)
vx.box("mano_{lado}", "piel", 2, 2, 6, 7, -8, -7, espejo=True)

# piernas: muslo bajo la falda, bota larga y zapato con suela turquesa
vx.box("muslo_{lado}", "piel", 1, 2, 0, 1, -6, -3, espejo=True)
vx.box("bota_{lado}_borde", "acento", 1, 2, 1, 1, -8, -7, espejo=True)
vx.box("bota_{lado}", "negro", 1, 2, -3, 0, -8, -7, espejo=True)
vx.box("bota_{lado}", "negro", 1, 2, -4, -4, -9, -7, espejo=True)
vx.box("suela_{lado}", "acento", 1, 2, -5, -5, -9, -7, espejo=True)

# ---------- volante (aro de 4 x 4 con hueco) ----------
vx.box("volante", "volante", -2, 1, 4, 7, -9, -9, si=lambda i, j, k: i in (-2, 1) or j in (4, 7))
vx.box("volante_columna", "audifono", -1, 0, 3, 4, -11, -10)
vx.box("volante_columna", "audifono", -1, 0, 1, 2, -12, -12)

# ---------- cabeza grandota (14 x 12 x 14 voxels, aristas redondeadas) ----------
# Va en una funcion porque la usan las dos Mikus: la piloto sentada y la que
# baila de pie (con la cabeza `dj` filas mas arriba).
HI0, HI1, HJ0, HJ1, HK0, HK1 = -7, 6, 10, 21, -7, 6
def cabeza_y_pelo(vx, dj):
    def en_cabeza(i, j, k):
        return (i in (HI0, HI1)) + (j in (HJ0, HJ1)) + (k in (HK0, HK1)) < 2
    def color_cabeza(i, j, k):
        if j >= 19 or k >= 3 or i in (HI0, HI1):
            return "pelo", "pelo"
        if k == HK0:
            if i in (-6, -5, 4, 5):                           # mechones de los lados
                return "pelo", "pelo"
            if j >= 17 or (j == 16 and i in (-4, -1, 0, 3)):  # flequillo con puntas
                return "pelo", "pelo"
        if k == HK0 + 1 and j >= 17:
            return "pelo", "pelo"
        return "cabeza", "piel"
    for i in range(HI0, HI1 + 1):
        for j in range(HJ0, HJ1 + 1):
            for k in range(HK0, HK1 + 1):
                if en_cabeza(i, j, k):
                    vx.put(i, j + dj, k, *color_cabeza(i, j, k))

    # mechones que caen enfrente de los hombros
    vx.box("pelo", "pelo", 4, 5, 6 + dj, 9 + dj, -7, -6, espejo=True)
    vx.box("mechones_puntas", "pelo_puntas", 4, 5, 5 + dj, 5 + dj, -7, -6, espejo=True)
    # audifonos a los lados con su lucecita
    vx.box("audifono_{lado}", "audifono", 7, 7, 13 + dj, 15 + dj, -2, 0, espejo=True)
    vx.box("audifono_{lado}_luz", "luz", 8, 8, 14 + dj, 14 + dj, -1, -1, espejo=True)
    # microfono del lado izq
    vx.box("microfono", "audifono", -8, -8, 12 + dj, 12 + dj, -6, -3)
    vx.box("microfono", "audifono", -7, -6, 11 + dj, 11 + dj, -8, -7)

    # coletas (cada una es una parte que se anima)
    vx.box("liga_{lado}", "liga_rosa", 7, 7, 19 + dj, 20 + dj, 0, 1, espejo=True)
    vx.box("coleta_{lado}", "pelo", 8, 9, 18 + dj, 21 + dj, -1, 2, espejo=True)
    vx.box("coleta_{lado}", "pelo", 9, 10, 4 + dj, 17 + dj, 0, 2, espejo=True)
    vx.box("coleta_{lado}", "pelo", 9, 10, 1 + dj, 3 + dj, 0, 1, espejo=True)
    vx.box("coleta_{lado}_puntas", "pelo_puntas", 9, 10, -1 + dj, 0 + dj, 0, 1, espejo=True)

cabeza_y_pelo(vx, 0)

# ---------- escribir ----------
m = Modelo()
animadas = {"coleta_der", "coleta_izq", "coleta_der_puntas", "coleta_izq_puntas", "volante"}
piv = vx.centro_de("liga_der")
vol = vx.centro_de("volante")
vx.mesh(m, animadas, {
    "coleta_der": f"pivote: {piv[0]:.3f} {piv[1]:.3f} {piv[2]:.3f} (la liga); coleta_der_puntas va con la misma matriz",
    "coleta_izq": f"pivote: {-piv[0]:.3f} {piv[1]:.3f} {piv[2]:.3f} (la liga); coleta_izq_puntas va con la misma matriz",
    "volante": f"gira alrededor del eje Z que pasa por {vol[0]:.3f} {vol[1]:.3f} {vol[2]:.3f}",
})
expresiones(m, z_frente=HK0*U, px=U/2, x_centro=0.0, y_base=(HJ0 + 1)*U)

m.write("miku_piloto", """# Miku piloto (chibi voxel, voxel de 0.04). Y arriba, mira hacia -Z, origen en el centro del asiento.
# izq/der desde su punto de vista: izq = -X, der = +X.
# Expresiones: dibuja solo las partes cara_<emocion>_* de la emocion actual
#   (feliz, asustada, espiral, enamorada, dormida, serio); van pegadas al frente de la cara, en orden.
# Animables: coleta_* (pivote en su comentario), volante (eje en su comentario).
# rubor y sudor usan d < 1: dibujalos con blending al final.
""", MTL)

# ================= Miku de pie, para bailar en la intro =================
# Mismo estilo; origen en el piso entre los pies. Brazos, piernas, cabeza y coletas
# giran sobre los pivotes de la cabecera.
DJ = 6
vb = Voxels(U)
# piernas juntitas y botas gorditas (lo chibi)
vb.box("suela_{lado}", "acento", 0, 1, 0, 0, -3, 1, espejo=True)
vb.box("bota_{lado}", "negro", 0, 1, 1, 4, -2, 1, espejo=True)
vb.box("bota_{lado}", "negro", 0, 1, 1, 2, -3, -3, espejo=True)
vb.box("bota_{lado}_borde", "acento", 0, 1, 5, 5, -2, 1, espejo=True)
vb.box("muslo_{lado}", "piel", 0, 1, 6, 7, -1, 0, espejo=True)
vb.box("falda_borde", "acento", -5, 4, 7, 7, -3, 2)
vb.box("falda", "negro", -4, 3, 8, 8, -3, 2)
vb.box("falda", "negro", -3, 2, 9, 9, -2, 1)
vb.box("torso", "gris", -3, 2, 10, 14, -2, 1)
vb.box("corbata", "acento", -1, 0, 12, 14, -3, -3)
vb.box("cuello", "piel", -1, 0, 15, 15, -1, 0)
vb.box("brazo_{lado}", "piel", 3, 3, 13, 14, -1, 0, espejo=True)
vb.box("manga_{lado}", "negro", 3, 3, 10, 12, -1, 0, espejo=True)
vb.box("manga_{lado}_borde", "acento", 3, 3, 9, 9, -1, 0, espejo=True)
vb.box("mano_{lado}", "piel", 3, 4, 7, 8, -1, 0, espejo=True)
cabeza_y_pelo(vb, DJ)

mb = Modelo()
vb.mesh(mb, {"coleta_der", "coleta_izq", "coleta_der_puntas", "coleta_izq_puntas",
             "brazo_der", "brazo_izq", "manga_der", "manga_izq", "manga_der_borde", "manga_izq_borde", "mano_der", "mano_izq",
             "muslo_der", "muslo_izq", "bota_der", "bota_izq", "bota_der_borde", "bota_izq_borde", "suela_der", "suela_izq"})
expresiones(mb, z_frente=HK0*U, px=U/2, x_centro=0.0, y_base=(HJ0 + DJ + 1)*U)
liga = vb.centro_de("liga_der")
mb.write("miku_baile", f"""# Miku de pie para bailar (chibi voxel, voxel de 0.04). Y arriba, mira hacia -Z, origen en el piso.
# izq/der desde su punto de vista: izq = -X, der = +X. Mide ~1.12.
# Grupos (por nombre): brazo_/manga_/mano_ = brazo; muslo_/bota_/suela_ = pierna; torso/corbata/falda/cuello = cuerpo;
#   coleta_* = coleta; lo demas (cabeza, pelo, audifonos, microfono, ligas, cara_*) = cabeza.
# pivote hombro_der: {3.5*U:.3f} {14.5*U:.3f} 0.000
# pivote hombro_izq: {-3.5*U:.3f} {14.5*U:.3f} 0.000
# pivote cadera_der: {1.0*U:.3f} {8.0*U:.3f} 0.000
# pivote cadera_izq: {-1.0*U:.3f} {8.0*U:.3f} 0.000
# pivote cuello: 0.000 {15.5*U:.3f} 0.000
# pivote coleta_der: {liga[0]:.3f} {liga[1]:.3f} {liga[2]:.3f}
# pivote coleta_izq: {-liga[0]:.3f} {liga[1]:.3f} {liga[2]:.3f}
""", MTL)
