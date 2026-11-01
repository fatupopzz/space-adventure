# Genera los modelos del show y de la intro:
#   luna_columpio.obj: la luna creciente donde Miku canta sentada en Saturno, colgada de dos cuerdas.
#   haz_luz.obj:       el cono de luz de un reflector (cuatro colores, una parte por color).
#   plataforma.obj:    la plataforma de despegue de la intro.
# Uso: python3 herramientas/generar_show.py

import math
from modelo3d import *

def mtl(*materiales):
    out = ["# Materiales"]
    for nombre, kd, ke, d in materiales:
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), "Ks 0.3 0.3 0.3", "Ns 24"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
        if d < 1:
            out.append(f"d {d:.2f}")
    return "\n".join(out) + "\n"

# ---------- luna columpio ----------
# Origen en el punto de donde cuelgan las cuerdas (el pivote del columpio).
# La luna esta 1.6 abajo, mirando a -Z, con la panza abajo como hamaca.
U = 0.025
vx = Voxels(U)
CY = -1.6
for i in range(-16, 16):
    for j in range(-16, 16):
        x, y, _ = vx.centro(i, j, 0)
        afuera = x * x + y * y <= 0.37 ** 2
        adentro = x * x + (y - 0.13) ** 2 <= 0.33 ** 2
        if afuera and not adentro:
            for k in range(-2, 2):
                vx.put(i, j + round(CY / U), k, "luna", "luna")
# cuerdas desde las puntas hasta el pivote, con estrellitas
for lado in (-1, 1):
    x = lado * 0.30
    i = round(x / U - 0.5)
    for j in range(round((CY + 0.18) / U), 0):
        vx.put(i, j, 0, "cuerda", "cuerda")
    for j0 in (round((CY + 0.6) / U), round((CY + 1.1) / U)):
        for di, dj in ((0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)):
            vx.put(i + di, j0 + dj, -1, "estrellitas", "estrellita")
m = Modelo()
vx.mesh(m)
asiento = CY - 0.33 + 0.13 + 0.01
m.write("luna_columpio", f"""# Luna columpio (voxel de 0.025). Origen en el pivote de las cuerdas; la luna esta abajo, mirando a -Z.
# asiento: 0.000 {asiento:.3f} 0.000   (ahi va el origen de miku_piloto, escala 0.42)
""", mtl(("luna", (1.0, 0.92, 0.55), (0.45, 0.38, 0.12), 1), ("cuerda", (1.0, 0.55, 0.85), (0.5, 0.2, 0.4), 1),
         ("estrellita", (1.0, 0.95, 0.5), (0.9, 0.8, 0.3), 1)))

# ---------- haz de luz ----------
# Cono con la punta en el origen, abierto hacia +Y, largo 1 y radio 0.28. Sin tapa:
# desde afuera se ve una sola capa transparente.
m = Modelo()
LADOS = 16
for color in ("cian", "rosa", "amarillo", "lima"):
    p = m.part(f"haz_{color}", f"haz_{color}")
    punta = p.vert((0.0, 0.0, 0.0))
    aro = [p.vert((0.28 * math.cos(2 * math.pi * k / LADOS), 1.0, 0.28 * math.sin(2 * math.pi * k / LADOS))) for k in range(LADOS)]
    for k in range(LADOS):
        p.tri(punta, aro[k], aro[(k + 1) % LADOS], (0.0, 0.6, 0.0))
COLORES = {"cian": (0.3, 0.95, 1.0), "rosa": (1.0, 0.4, 0.8), "amarillo": (1.0, 0.85, 0.3), "lima": (0.6, 1.0, 0.35)}
m.write("haz_luz", """# Haz de luz de un reflector: cono con la punta en el origen, abierto hacia +Y, largo 1.
# Una parte por color (haz_cian, haz_rosa, haz_amarillo, haz_lima): se dibuja una y se ocultan las otras.
""", mtl(*[(f"haz_{c}", v, v, 0.16) for c, v in COLORES.items()]))

# ---------- plataforma de despegue ----------
m = Modelo()
vp = Voxels(0.1)
for i in range(-33, 33):
    for k in range(-33, 33):
        x, _, z = vp.centro(i, 0, k)
        r = math.hypot(x, z)
        if r > 3.2:
            continue
        for j in (-3, -2):
            vp.put(i, j, k, "plataforma_base", "plataforma_lado")
        if r > 3.0:
            mat = "luces"
        elif 1.7 < r < 1.95:
            mat = "marca"
        elif abs(x) < 0.12 or abs(z) < 0.12:
            mat = "marca" if r < 1.7 else "plataforma"
        else:
            mat = "plataforma"
        vp.put(i, -1, k, f"plataforma_{mat}", mat)
        # columna de abajo
        if r < 0.9:
            for j in range(-12, -3):
                vp.put(i, j, k, "plataforma_base", "plataforma_lado")
# postes con faros en el borde
for n in range(6):
    # Ninguno queda justo al frente (-Z), donde baila Miku.
    a = n * math.pi / 3
    i, k = round(2.85 * math.cos(a) / 0.1), round(2.85 * math.sin(a) / 0.1)
    for j in range(0, 4):
        vp.put(i, j, k, "postes", "plataforma_lado")
    vp.put(i, 4, k, "faros", "luces")
vp.mesh(m)
m.write("plataforma", """# Plataforma de despegue (voxel de 0.1). Origen en el centro, la cara de arriba en y = 0. Radio 3.2.
""", mtl(("plataforma", (0.55, 0.50, 0.68), None, 1), ("plataforma_lado", (0.30, 0.26, 0.40), None, 1),
         ("marca", (0.25, 0.85, 0.80), (0.10, 0.40, 0.38), 1), ("luces", (1.0, 0.45, 0.80), (1.0, 0.35, 0.70), 1)))

# ---------- fans de la intro ----------
# Alienigenas redonditos con su barra de luz. Tres colores en el mismo modelo
# (fan_lila_*, fan_rosa_*, fan_cian_*): se dibuja uno y se ocultan los otros.
# Miran hacia -Z, origen en el piso. La barra gira sobre el pivote de la mano.
m = Modelo()
U = 0.05
COLORES_FAN = {"lila": (0.72, 0.55, 0.98), "rosa": (1.0, 0.55, 0.80), "cian": (0.40, 0.90, 0.95)}
BARRA = {"lila": (1.0, 0.45, 0.80), "rosa": (0.35, 0.95, 1.0), "cian": (1.0, 0.85, 0.30)}
for nombre in COLORES_FAN:
    vf = Voxels(U)
    p = f"fan_{nombre}"
    for i in range(-4, 4):
        for j in range(0, 8):
            for k in range(-4, 4):
                x, y, z = vf.centro(i, j, k)
                if (x / 0.2) ** 2 + ((y - 0.2) / 0.2) ** 2 + (z / 0.19) ** 2 <= 1.0:
                    vf.put(i, j, k, f"{p}_cuerpo", f"{p}_cuerpo")
    for i0 in (-3, 1):                       # ojos grandotes con brillo
        vf.box(f"{p}_ojos", "fan_ojo", i0, i0 + 1, 3, 4, -5, -5)
        vf.put(i0, 4, -6, f"{p}_brillo", "fan_brillo")
    vf.box(f"{p}_antena", "fan_ojo", 0, 0, 8, 9, 0, 0)
    vf.box(f"{p}_antena", "fan_ojo", -1, 1, 10, 10, -1, 1)
    vf.box(f"{p}_mano", f"{p}_cuerpo_mano", 4, 4, 3, 4, -1, 0)
    vf.box(f"{p}_barra", f"{p}_barra", 4, 4, 5, 10, -1, -1)
    vf.mesh(m, {f"{p}_barra"})
COLORES_FAN_MTL = []
for nombre, c in COLORES_FAN.items():
    COLORES_FAN_MTL += [(f"fan_{nombre}_cuerpo", c, None, 1), (f"fan_{nombre}_cuerpo_mano", c, None, 1),
                        (f"fan_{nombre}_barra", BARRA[nombre], BARRA[nombre], 1)]
m.write("fans", f"""# Fans de la intro (voxel de 0.05). Miran hacia -Z, origen en el piso, miden ~0.55.
# Tres colores: partes fan_lila_*, fan_rosa_*, fan_cian_*. La barra (fan_*_barra) gira sobre la mano.
# pivote mano: {4.5*U:.3f} {4.5*U:.3f} {-0.5*U:.3f}
""", mtl(*COLORES_FAN_MTL, ("fan_ojo", (0.06, 0.05, 0.08), None, 1), ("fan_brillo", (1, 1, 1), (0.9, 0.9, 0.9), 1)))

# ---------- notas del minijuego de ritmo ----------
# Bichitos redondos: rosa con patitas (carril del piso) y turquesa con alitas
# (carril del aire). Miran hacia -Z, origen abajo al centro.
m = Modelo()
U = 0.05
for carril, color in (("piso", "nota_rosa"), ("aire", "nota_cian")):
    vn = Voxels(U)
    p = f"nota_{carril}"
    for i in range(-4, 4):
        for j in range(0, 7):
            for k in range(-4, 4):
                x, y, z = vn.centro(i, j, k)
                if (x / 0.19) ** 2 + ((y - 0.18) / 0.17) ** 2 + (z / 0.19) ** 2 <= 1.0:
                    vn.put(i, j, k, f"{p}_cuerpo", color)
    for i0 in (-3, 1):
        vn.box(f"{p}_ojos", "fan_ojo", i0, i0 + 1, 3, 4, -5, -5)
        vn.put(i0, 4, -6, f"{p}_brillo", "fan_brillo")
    if carril == "piso":
        for i0 in (-3, 2):
            vn.box(f"{p}_patas", "fan_ojo", i0, i0, -1, 0, -1, 0)
    else:
        for lado in (-1, 1):
            i0 = 4 if lado > 0 else -5
            vn.box(f"{p}_alas", "nota_ala", i0, i0, 3, 6, -1, 2)
            vn.box(f"{p}_alas", "nota_ala", i0 + lado, i0 + lado, 4, 7, 0, 3)
    vn.mesh(m)
m.write("notas_ritmo", """# Notas del minijuego de ritmo (voxel de 0.05). Miran hacia -Z, origen abajo al centro.
# nota_piso_*: carril del piso (rosa, con patitas). nota_aire_*: carril del aire (turquesa, con alitas).
""", mtl(("nota_rosa", (1.0, 0.45, 0.78), (0.35, 0.10, 0.25), 1), ("nota_cian", (0.35, 0.92, 1.0), (0.10, 0.30, 0.35), 1),
         ("nota_ala", (0.92, 0.98, 1.0), (0.4, 0.45, 0.5), 1), ("fan_ojo", (0.06, 0.05, 0.08), None, 1), ("fan_brillo", (1, 1, 1), (0.9, 0.9, 0.9), 1)))

# ---------- tripulantes de la junta de emergencia (homenaje a los juegos de impostores) ----------
# Frijolitos con visor y mochila, cada uno con el traje del planeta que representa.
# Ocho variantes en el mismo modelo (t0_* ... t7_*): se dibuja una y se ocultan las otras.
# Miran hacia -Z, origen en el piso, miden ~0.7.
TRAJES = [  # (planeta, color, gorro)
    ("mercurio", (0.62, 0.62, 0.66), None),
    ("venus", (0.96, 0.80, 0.35), None),
    ("tierra", (0.22, 0.46, 0.95), "brote"),
    ("marte", (0.90, 0.32, 0.20), None),
    ("jupiter", (0.93, 0.62, 0.32), "mancha"),
    ("saturno", (0.96, 0.86, 0.60), "anillo"),
    ("urano", (0.45, 0.88, 0.90), None),
    ("neptuno", (0.18, 0.28, 0.80), None),
]
m = Modelo()
U = 0.04
materiales_trip = [("visor", (0.55, 0.85, 0.96), (0.15, 0.25, 0.3), 1), ("visor_brillo", (1, 1, 1), (0.9, 0.9, 0.9), 1),
                   ("brote", (0.35, 0.85, 0.35), None, 1), ("anillo_gorro", (1.0, 0.5, 0.8), (0.3, 0.1, 0.2), 1),
                   ("mancha", (0.85, 0.22, 0.18), None, 1)]
for n, (planeta, color, gorro) in enumerate(TRAJES):
    vt = Voxels(U)
    p = f"t{n}"
    traje = f"traje_{planeta}"
    materiales_trip.append((traje, color, None, 1))
    for i in range(-6, 6):
        for j in range(0, 18):
            for k in range(-5, 7):
                x, y, z = vt.centro(i, j, k)
                cuerpo = (x / 0.22) ** 2 + (z / 0.18) ** 2 <= 1.0 and 0.18 <= y <= 0.5
                cupula = (x / 0.22) ** 2 + ((y - 0.5) / 0.2) ** 2 + (z / 0.18) ** 2 <= 1.0 and y > 0.5
                pierna = 0.0 <= y < 0.2 and 0.05 <= abs(x) <= 0.2 and abs(z) <= 0.13
                mochila = 0.24 <= y <= 0.52 and abs(x) <= 0.16 and 0.16 <= z <= 0.27
                if cuerpo or cupula or pierna or mochila:
                    vt.put(i, j, k, f"{p}_traje", traje)
    # El visor: ovalado, corrido a la derecha, saliendo del frente.
    for i in range(-2, 5):
        for j in range(10, 14):
            if (i, j) in ((-2, 10), (-2, 13), (4, 10), (4, 13)):
                continue
            vt.put(i, j, -6, f"{p}_visor", "visor")
    vt.put(2, 13, -7, f"{p}_visor_brillo", "visor_brillo")
    vt.put(3, 12, -7, f"{p}_visor_brillo", "visor_brillo")
    if gorro == "brote":
        vt.box(f"{p}_gorro", "brote", 0, 0, 18, 20, 0, 0)
        vt.box(f"{p}_gorro", "brote", -2, -1, 20, 20, 0, 0)
        vt.box(f"{p}_gorro", "brote", 1, 2, 21, 21, 0, 0)
    elif gorro == "anillo":
        for i in range(-8, 8):
            for k in range(-7, 9):
                x, _, z = vt.centro(i, 0, k)
                if 0.24 <= (x * x + (z - 0.02) ** 2) ** 0.5 <= 0.32:
                    vt.put(i, 16, k, f"{p}_gorro", "anillo_gorro")
    elif gorro == "mancha":
        vt.box(f"{p}_gorro", "mancha", 1, 3, 6, 7, -5, -5)
    vt.mesh(m)
m.write("tripulantes", """# Tripulantes de la junta (voxel de 0.04): frijolitos con visor y mochila, uno por planeta.
# Variantes t0..t7 (mercurio, venus, tierra, marte, jupiter, saturno, urano, neptuno). Miran hacia -Z, origen en el piso.
""", mtl(*materiales_trip))

# ---------- la mesa de la junta con el boton de emergencia ----------
m = Modelo()
vm = Voxels(0.05)
for i in range(-20, 20):
    for k in range(-20, 20):
        x, _, z = vm.centro(i, 0, k)
        r = (x * x + z * z) ** 0.5
        if r <= 0.95:
            vm.put(i, 8, k, "mesa", "mesa")
            if r > 0.88:
                vm.put(i, 9, k, "mesa_borde", "mesa_borde")
        if r <= 0.25:
            for j in range(0, 8):
                vm.put(i, j, k, "mesa_pata", "mesa_pata")
        if r <= 0.22:
            vm.put(i, 9, k, "boton_base", "boton_base")
        if r <= 0.15:
            vm.put(i, 10, k, "boton", "boton")
            if r <= 0.1:
                vm.put(i, 11, k, "boton", "boton")
# El piso de la sala: losetas a cuadros (de 0.3) en un circulo de radio 2.6, con orilla.
for i in range(-52, 52):
    for k in range(-52, 52):
        x, _, z = vm.centro(i, 0, k)
        r = (x * x + z * z) ** 0.5
        if r <= 2.6:
            if r > 2.45:
                vm.put(i, -1, k, "piso_orilla", "piso_orilla")
                vm.put(i, 0, k, "piso_orilla", "piso_orilla")
            else:
                m_piso = "piso_claro" if (i // 6 + k // 6) % 2 == 0 else "piso_oscuro"
                vm.put(i, -1, k, m_piso, m_piso)
vm.mesh(m)
m.write("mesa_junta", """# La sala de la junta (voxel de 0.05): mesa con el boton rojo de emergencia al centro y piso a cuadros
# de radio 2.6. Origen en el piso, la mesa mide 0.95 de radio.
""", mtl(("piso_claro", (0.50, 0.53, 0.62), None, 1), ("piso_oscuro", (0.34, 0.36, 0.45), None, 1),
         ("piso_orilla", (0.80, 0.45, 0.95), (0.25, 0.08, 0.35), 1), ("mesa", (0.80, 0.84, 0.92), None, 1), ("mesa_borde", (0.55, 0.60, 0.72), None, 1), ("mesa_pata", (0.45, 0.48, 0.58), None, 1),
         ("boton_base", (0.35, 0.36, 0.42), None, 1), ("boton", (0.95, 0.12, 0.15), (0.6, 0.05, 0.05), 1)))
