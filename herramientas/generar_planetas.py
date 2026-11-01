# Los planetas ya NO salen de aqui: el juego los arma al arrancar (src/planetas.rs: voxels,
# deformacion con ruido Perlin y niveles de detalle, como el Planeta Chapin).
# Este script hace solo lo que no es el planeta: las CARAS de Neptuno y Pluton, calcomanias con
# sus expresiones (cara_<expresion>_*) pegadas al frente (z = -1, hacia -Z) de una esfera de
# radio 1. El juego las junta con el planeta que arma.
# Uso: python3 herramientas/generar_planetas.py

from modelo3d import *

for nombre, texto in (("neptuno", "Neptuno"), ("pluton", "Pluton")):
    m = Modelo()
    expresiones(m, z_frente=-1.0, px=0.035, x_centro=0.0, y_base=-0.2, separacion=0.004)
    m.write(nombre + "_cara", f"# La cara de {texto}: sus expresiones, al frente de la esfera (el planeta lo arma el juego).\n", MTL_CARA)
