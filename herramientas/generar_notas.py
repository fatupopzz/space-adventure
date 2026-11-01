# Genera la nota musical que flota en el espacio (las que recoge la nave): una corchea de voxels
# brillantes, en dos colores (partes nota_cian y nota_rosa: se oculta la que no toca).
# Uso: python3 herramientas/generar_notas.py

from modelo3d import *

# La corchea (♪), de 7 x 11 pixeles, de arriba hacia abajo.
FORMA = ["...##..",
         "...###.",
         "...#.##",
         "...#..#",
         "...#...",
         "...#...",
         "...#...",
         ".###...",
         "####...",
         "####...",
         ".##...."]
m = Modelo()
for color in ("cian", "rosa"):
    vx = Voxels(0.18, origen=(-0.63, -1.0, -0.18))
    for r, fila in enumerate(FORMA):
        for c, ch in enumerate(fila):
            if ch == "#":
                for k in (0, 1):
                    vx.put(c, 10 - r, k, f"nota_{color}", f"nota_{color}")
    vx.mesh(m)
m.write("nota_espacio", """# La nota musical que flota en el espacio (voxel de 0.18): una corchea de ~1.3 x 2. Dos colores: partes
# nota_cian y nota_rosa (se dibuja una). Centrada mas o menos en el origen, de frente a -Z.
""", """# Materiales

newmtl nota_cian
Kd 0.40 0.95 1.00
Ks 0.3 0.3 0.3
Ns 32
Ke 0.25 0.70 0.80

newmtl nota_rosa
Kd 1.00 0.50 0.85
Ks 0.3 0.3 0.3
Ns 32
Ke 0.80 0.30 0.60
""")
