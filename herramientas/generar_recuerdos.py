# Genera los recuerdos de la MOCHILA (uno por aventura), en voxel, para verlos en 3D:
#   modelos/recuerdo_00.obj .. recuerdo_14.obj, en el orden de `RECUERDOS` (src/mochila.rs).
# Todos centrados en el origen, de ~1 de alto, con el frente mirando a -Z (hacia la camara de la
# vitrina). Los que ya existian (la carta del tarot, la carta +4 y el Contenedor de Corazon) se
# copian de su modelo, dejando solo la cara que toca.
# Uso: python3 herramientas/generar_recuerdos.py

import math, os
from modelo3d import *


def mtl(*materiales):
    """Cada material: (nombre, Kd, Ke) o (nombre, Kd, Ke, Ns)."""
    out = ["# Materiales del recuerdo"]
    for nombre, kd, ke, *ns in materiales:
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), "Ks 0.4 0.4 0.4", f"Ns {ns[0] if ns else 32}"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
    return "\n".join(out) + "\n"


def guardar(n, vx, comentario, materiales):
    # Cada parte lleva un solo material: si una parte usa varios, se parte en una por material.
    for clave, (p, mat) in list(vx.v.items()):
        vx.v[clave] = (p if p == mat else f"{p}_{mat}", mat)
    m = Modelo()
    vx.mesh(m)
    m.write(f"recuerdo_{n:02d}", f"# Recuerdo {n} de la mochila: {comentario}\n# Voxel de {vx.u}, centrado en el origen, el frente mira a -Z.\n", mtl(*materiales))


def copiar(n, origen, quedan, comentario):
    """Copia un .obj dejando solo las caras de las partes que `quedan(nombre)`."""
    lineas, parte = [], None
    for l in open(os.path.join(MODELOS, origen + ".obj")):
        if l.startswith("o "):
            parte = l[2:].strip()
        if l.startswith("f ") and not quedan(parte):
            continue
        if l.startswith("#"):
            continue
        lineas.append(l.rstrip("\n"))
    open(os.path.join(MODELOS, f"recuerdo_{n:02d}.obj"), "w").write(f"# Recuerdo {n} de la mochila: {comentario}\n" + "\n".join(lineas) + "\n")
    print(f"recuerdo_{n:02d}: copia de {origen}")


def esfera(vx, parte, mat, cx, cy, cz, r, u):
    R = int(r / u) + 1
    for i in range(-R, R + 1):
        for j in range(-R, R + 1):
            for k in range(-R, R + 1):
                if (i * u) ** 2 + (j * u) ** 2 + (k * u) ** 2 <= r * r:
                    vx.put(int(round(cx / u)) + i, int(round(cy / u)) + j, int(round(cz / u)) + k, parte, mat)


# 0. La carta del tarot: EL CHUCHO (frente_08), con su reverso y su canto dorado.
copiar(0, "tarot", lambda p: not p.startswith("frente_") or p == "frente_08", "la carta EL CHUCHO del tarot de Madame Mercurio.")

# 1. El anillo de bodas de Saturno (del juicio en Venus): oro, y de piedra un Saturno chiquito.
u = 0.04
vx = Voxels(u)
for i in range(-14, 15):
    for j in range(-14, 15):
        for k in range(-3, 4):
            x, y, z = (i + 0.5) * u, (j + 0.5) * u, (k + 0.5) * u
            if (math.hypot(x, y) - 0.36) ** 2 + z * z <= 0.075 ** 2:
                vx.put(i, j - 4, k, "anillo", "oro_brillo" if (i + j) % 7 == 0 and y > 0 else "oro")
for i in range(-3, 3):
    vx.box("engaste", "oro", i, i, 5, 6, -1, 0)
esfera(vx, "saturno", "saturno", 0.0, 0.42, 0.0, 0.15, u)
for i in range(-9, 10):
    for k in range(-9, 10):
        d = math.hypot(i * u, k * u)
        if 0.19 <= d <= 0.33:
            vx.put(i, 11 + int(round(i * 0.25)), k, "saturno_anillo", "hielo")
guardar(1, vx, "el anillo de bodas de Saturno.", [("oro", (0.95, 0.72, 0.25), (0.12, 0.08, 0.0), 64), ("oro_brillo", (1.0, 0.9, 0.55), (0.4, 0.3, 0.1), 64),
                                                  ("saturno", (0.92, 0.80, 0.55), None), ("hielo", (0.75, 0.88, 1.0), (0.15, 0.2, 0.3))])

# 2. La regla del profe Koch: madera, rayitas, un automata grabado (q0 -> q1) y su manchita roja.
u = 0.04
vx = Voxels(u)
vx.box("regla", "madera", -15, 14, -3, 3, 0, 1)
for i in range(-14, 14):
    largo = 3 if i % 5 == 0 else 1
    vx.box("rayitas", "grabado", i, i, 3 - largo + 1, 3, -1, -1)
for (ci, doble) in ((-6, False), (5, True)):
    for i in range(-3, 4):
        for j in range(-3, 4):
            d = math.hypot(i, j)
            if 1.6 <= d <= 2.6 or (doble and 0.6 <= d <= 1.2):
                vx.put(ci + i, j - 1, -1, "automata", "grabado")
vx.box("automata", "grabado", -3, 2, -1, -1, -1, -1)
vx.put(1, 0, -1, "automata", "grabado")
vx.put(1, -2, -1, "automata", "grabado")
vx.box("flecha_inicio", "grabado", -12, -10, -1, -1, -1, -1)
vx.box("tinta_roja", "rojo", 11, 12, -3, -2, -1, -1)
guardar(2, vx, "la regla del profe Koch.", [("madera", (0.80, 0.58, 0.32), None, 16), ("grabado", (0.30, 0.18, 0.08), None, 8), ("rojo", (0.85, 0.12, 0.12), None)])

# 3. La claqueta del set lunar: pizarra negra con letras de tiza y el palo de rayas abierto.
u = 0.04
vx = Voxels(u)
vx.box("pizarra", "negro", -11, 11, -10, 4, 0, 1)
for (j, i0, i1) in ((2, -9, -2), (2, 1, 8), (-1, -9, 5), (-4, -9, -4), (-4, -1, 3), (-7, -9, 9)):
    vx.box("tiza", "tiza", i0, i1, j, j, -1, -1)
for i in range(-11, 12):
    mat = "blanco" if ((i + 11) // 3) % 2 == 0 else "negro"
    vx.box("palo_abajo", mat, i, i, 5, 6, 0, 1)
for i in range(-11, 12):
    sube = int((i + 11) * 0.35)
    mat = "blanco" if ((i + 11) // 3) % 2 == 1 else "negro"
    vx.box("palo_arriba", mat, i, i, 8 + sube, 9 + sube, 0, 1)
vx.box("bisagra", "metal", -12, -11, 5, 9, -1, 2)
guardar(3, vx, "la claqueta del set de la Luna.", [("negro", (0.08, 0.08, 0.10), None), ("blanco", (0.95, 0.95, 0.95), None), ("tiza", (0.85, 0.85, 0.80), (0.1, 0.1, 0.1)), ("metal", (0.6, 0.6, 0.65), None, 64)])

# 4. La foto de la cita en Marte (polaroid): el cielo rojo, dos chibis, Curiosity borroso y un corazon.
u = 0.04
vx = Voxels(u)
vx.box("marco", "papel", -10, 9, -13, 11, 0, 0)
for j in range(-6, 10):
    mat = "cielo_alto" if j > 5 else ("cielo" if j > -1 else "suelo")
    vx.box("foto", mat, -8, 7, j, j, -1, -1)
for (x, ropa) in ((-4, "ropa_a"), (-1, "ropa_b")):
    vx.box("chibis", "piel", x, x + 1, 3, 4, -2, -2)
    vx.box("chibis", ropa, x, x + 1, -1, 2, -2, -2)
vx.box("curiosity", "gris", 3, 6, 0, 1, -2, -2)
vx.box("curiosity", "gris_borroso", 2, 2, 0, 1, -2, -2)
vx.box("curiosity", "gris", 4, 4, 2, 3, -2, -2)
vx.put(5, 7, -2, "sol", "sol")
for (i, j) in ((-2, -10), (-1, -10), (1, -10), (2, -10), (-2, -9), (-1, -9), (0, -9), (1, -9), (2, -9), (-1, -8), (0, -8), (1, -8), (0, -7)):
    vx.put(i, j + 0, -1, "corazon", "rosa")
guardar(4, vx, "la foto de la cita en Marte.", [("papel", (0.96, 0.95, 0.90), None, 8), ("cielo_alto", (0.78, 0.48, 0.32), None), ("cielo", (0.88, 0.58, 0.38), None), ("suelo", (0.62, 0.28, 0.14), None),
                                                ("piel", (0.95, 0.78, 0.62), None), ("ropa_a", (0.20, 0.80, 0.80), None), ("ropa_b", (0.90, 0.30, 0.55), None),
                                                ("gris", (0.70, 0.70, 0.72), None), ("gris_borroso", (0.62, 0.48, 0.42), None), ("sol", (0.9, 0.9, 0.95), (0.6, 0.6, 0.65)), ("rosa", (1.0, 0.35, 0.6), (0.3, 0.05, 0.15))])

# 5. La Gran Mancha Roja en un frasco de mermelada: el vidrio atras (para verla), la tapa de
#    cuadritos y la tormenta girando adentro.
u = 0.04
vx = Voxels(u)
R = 7
for j in range(-10, 7):
    for i in range(-R, R + 1):
        for k in range(-R, R + 1):
            d = math.hypot(i, k)
            if j == -10 and d <= R:
                vx.put(i, j, k, "vidrio", "vidrio")
            elif R - 1 <= d <= R and (k >= 1 or j in (-9, 6)):
                vx.put(i, j, k, "vidrio", "vidrio_borde" if j in (-9, 6) else "vidrio")
for j in range(7, 10):
    for i in range(-R - 1, R + 2):
        for k in range(-R - 1, R + 2):
            if math.hypot(i, k) <= R + 1:
                mat = "tela_roja" if ((i // 2) + (k // 2)) % 2 == 0 else "tela_blanca"
                vx.put(i, j, k, "tapa", mat if j == 9 else "tapa")
for n in range(160):
    a = n * 0.23
    r = 0.4 + n * 0.035
    y = -8 + (n % 40) * 0.35
    i, k = int(round(math.cos(a) * r)), int(round(math.sin(a) * r * 0.8))
    if math.hypot(i, k) < R - 1:
        vx.put(i, int(y), k, "tormenta", "tormenta_ojo" if r < 1.5 else ("tormenta" if n % 3 else "tormenta_clara"))
guardar(5, vx, "la Gran Mancha Roja en un frasco.", [("vidrio", (0.55, 0.70, 0.80), (0.04, 0.06, 0.08), 90), ("vidrio_borde", (0.80, 0.92, 1.0), (0.1, 0.12, 0.15), 90), ("tapa", (0.75, 0.55, 0.30), None),
                                                     ("tela_roja", (0.85, 0.15, 0.15), None), ("tela_blanca", (0.95, 0.95, 0.92), None),
                                                     ("tormenta", (0.95, 0.35, 0.15), (0.6, 0.18, 0.05)), ("tormenta_clara", (1.0, 0.7, 0.4), (0.6, 0.35, 0.15)), ("tormenta_ojo", (1.0, 0.9, 0.5), (0.9, 0.7, 0.3))])

# 6. El vinil de Saturno Ritmo, saliendo de su funda (la funda trae a Saturno).
u = 0.04
vx = Voxels(u)
vx.box("funda", "funda", -14, 6, -10, 10, 1, 1)
for i in range(-14, 7):
    for j in range(-10, 11):
        x, y = (i + 4) * 1.0, j * 1.0
        if math.hypot(x, y) <= 4.5:
            vx.put(i, j, 0, "funda_saturno", "saturno")
        elif abs(y + x * 0.25) <= 0.8 and math.hypot(x, y) <= 8:
            vx.put(i, j, 0, "funda_saturno", "rosa")
for i in range(-11, 12):
    for j in range(-11, 12):
        d = math.hypot(i, j)
        if d <= 11:
            mat = "etiqueta" if d <= 3.5 else ("vinil_brillo" if int(d) % 2 == 0 else "vinil")
            if d <= 0.8:
                continue
            vx.put(i + 5, j, -1, "disco", mat)
guardar(6, vx, "el vinil de Saturno Ritmo.", [("funda", (0.25, 0.12, 0.40), None), ("saturno", (0.95, 0.82, 0.50), (0.15, 0.12, 0.05)), ("rosa", (1.0, 0.45, 0.75), (0.3, 0.1, 0.2)),
                                             ("vinil", (0.06, 0.05, 0.08), None, 80), ("vinil_brillo", (0.16, 0.14, 0.20), None, 80), ("etiqueta", (1.0, 0.45, 0.75), (0.2, 0.05, 0.12))])

# 7. La carta +4 de ¡NEPTUNO! (la negra), firmada por el Doom Slayer.
copiar(7, "carta", lambda p: not (p.startswith("color_") or p.startswith("s_")) or p in ("color_negro", "s_mas4"), "la carta +4 de Neptuno.")

# 8. La credencial de planeta de Pluton: dice PLANETA en grande (la franja roja), la foto con su
#    corazon (Tombaugh Regio) y el cordon morado.
u = 0.035
vx = Voxels(u)
vx.box("credencial", "plastico", -8, 8, -11, 11, 0, 0)
vx.box("franja", "rojo", -8, 8, 7, 11, -1, -1)
for i in range(-6, 7, 2):
    vx.box("letras", "blanco", i, i, 8, 10, -2, -2)
for i in range(-6, 1):
    for j in range(-3, 5):
        vx.put(i, j, -1, "foto", "foto_fondo")
for i in range(-6, 1):
    for j in range(-3, 5):
        if math.hypot(i + 3, j - 0.5) <= 3.2:
            corazon = math.hypot(i + 3.6, j - 0.2) <= 1.3 or math.hypot(i + 2.0, j - 0.2) <= 1.3 or (j <= 0 and abs(i + 2.8) <= 1 + j * 0.5)
            vx.put(i, j, -2, "pluton", "corazon" if corazon else "pluton")
for (j, largo) in ((3, 6), (1, 5), (-1, 6), (-3, 3)):
    vx.box("lineas", "gris", 2, 2 + largo - 1, j, j, -1, -1)
vx.box("enano", "gris", 3, 6, -8, -8, -1, -1)
vx.box("broche", "metal", -2, 1, 11, 13, 0, 0)
for n in range(10):
    vx.put(-1 - n, 14 + n, 0, "cordon", "cordon")
    vx.put(-1 - n, 15 + n, 0, "cordon", "cordon")
    vx.put(n, 14 + n, 0, "cordon", "cordon")
    vx.put(n, 15 + n, 0, "cordon", "cordon")
guardar(8, vx, "la credencial de planeta de Plutón.", [("plastico", (0.96, 0.95, 0.92), None, 48), ("rojo", (0.85, 0.15, 0.20), None), ("blanco", (1.0, 1.0, 1.0), None), ("foto_fondo", (0.08, 0.08, 0.15), None),
                                                       ("pluton", (0.70, 0.58, 0.48), None), ("corazon", (0.96, 0.92, 0.85), (0.1, 0.1, 0.08)), ("gris", (0.5, 0.5, 0.55), None),
                                                       ("metal", (0.7, 0.7, 0.75), None, 80), ("cordon", (0.55, 0.30, 0.85), None)])

# 9. El Contenedor de Corazon (el de la Fuente de las Hadas).
copiar(9, "contenedor", lambda p: True, "el Contenedor de Corazón.")

# 10. Hielo de cometa: cristales de hielo sobre un pedacito de roca oscura.
u = 0.04
vx = Voxels(u)
for i in range(-8, 9):
    for k in range(-6, 7):
        for j in range(-11, -6):
            if (i / 8.0) ** 2 + (k / 6.0) ** 2 + ((j + 9) / 3.0) ** 2 <= 1.0:
                vx.put(i, j, k, "roca", "roca" if (i + j * 3 + k) % 5 else "roca_clara")
for (ci, ck, alto, r) in ((0, 0, 16, 2.2), (-4, 1, 10, 1.6), (4, -1, 12, 1.7), (2, 3, 7, 1.3), (-2, -3, 8, 1.3), (-6, -2, 5, 1.0)):
    for j in range(-7, -7 + alto):
        rr = r if j < -7 + alto - 3 else r * (-7 + alto - j) / 3.0
        for i in range(-3, 4):
            for k in range(-3, 4):
                if abs(i) + abs(k) * 0.6 <= rr + 0.3 and max(abs(i), abs(k)) <= rr:
                    vx.put(ci + i, j, ck + k, "cristales", "hielo_claro" if (i == 0 and k <= 0) else "hielo")
guardar(10, vx, "hielo de cometa.", [("roca", (0.22, 0.20, 0.22), None, 8), ("roca_clara", (0.36, 0.34, 0.36), None, 8), ("hielo", (0.55, 0.85, 1.0), (0.15, 0.35, 0.5), 90), ("hielo_claro", (0.9, 0.98, 1.0), (0.4, 0.55, 0.65), 90)])

# 11. El autografo de Bill Nye: su corbatin (rojo con bolitas azules) sobre una tarjeta firmada.
u = 0.035
vx = Voxels(u)
vx.box("tarjeta", "papel", -13, 13, -12, 6, 2, 2)
for n in range(40):
    x = -10 + n * 0.5
    y = -8 + 2.2 * math.sin(n * 0.7) + (1.5 if 12 < n < 20 else 0)
    vx.put(int(x), int(y), 1, "firma", "tinta")
for i in range(-11, 12):
    a = abs(i)
    if a <= 1:
        vx.box("corbatin_nudo", "rojo_oscuro", i, i, -1, 3, -1, 0)
        continue
    alto = int(1 + a * 0.55)
    for j in range(1 - alto, 2 + alto):
        mat = "azul" if (i % 3 == 0 and j % 3 == 0) else "rojo"
        vx.put(i, j, -1 if a < 6 else 0, "corbatin", mat)
guardar(11, vx, "el autógrafo de Bill Nye (con su corbatín).", [("papel", (0.97, 0.96, 0.92), None, 8), ("tinta", (0.15, 0.25, 0.75), None), ("rojo", (0.85, 0.12, 0.15), None, 24), ("rojo_oscuro", (0.6, 0.06, 0.1), None), ("azul", (0.2, 0.35, 0.9), None)])

# 12. El cinturon de campeon del boxeo de Urano: correa de cuero, placa de oro y de piedra, Urano.
u = 0.035
vx = Voxels(u)
for i in range(-16, 17):
    atras = int((abs(i) / 16.0) ** 2 * 8)
    vx.box("correa", "cuero", i, i, -4, 4, atras, atras + 1)
    if abs(i) > 7:
        vx.box("costura", "costura", i, i, 4, 4, atras - 1, atras - 1)
        vx.box("costura", "costura", i, i, -4, -4, atras - 1, atras - 1)
for (ci, mat) in ((-10, "plata"), (10, "plata")):
    vx.box("placas", mat, ci - 2, ci + 2, -3, 3, 1, 2)
for i in range(-8, 9):
    for j in range(-8, 9):
        d = max(abs(i), abs(j)) * 0.8 + min(abs(i), abs(j)) * 0.5
        if d <= 7.5:
            mat = "oro_claro" if d > 6.3 else "oro"
            vx.put(i, j, -1, "placa", mat)
            vx.put(i, j, 0, "placa", "oro")
esfera(vx, "urano", "urano", 0.0, 0.0, -0.07, 0.11, u)
for i in range(-5, 6):
    vx.put(i, int(round(i * 0.9)), -3, "urano_anillo", "oro_claro")
guardar(12, vx, "el cinturón de campeón de Urano.", [("cuero", (0.55, 0.08, 0.10), None, 20), ("costura", (0.95, 0.85, 0.5), None), ("plata", (0.78, 0.78, 0.82), None, 80),
                                                    ("oro", (0.95, 0.72, 0.25), (0.12, 0.08, 0.0), 64), ("oro_claro", (1.0, 0.88, 0.5), (0.25, 0.18, 0.05), 64), ("urano", (0.55, 0.90, 0.95), (0.15, 0.3, 0.3))])

# 13. El diamante de Planeta Vegetta: un bloque de mena de diamante (piedra con diamantes).
u = 0.07
vx = Voxels(u)
for i in range(-5, 5):
    for j in range(-5, 5):
        for k in range(-5, 5):
            borde = i in (-5, 4) or j in (-5, 4) or k in (-5, 4)
            if not borde:
                continue
            h = (i * 73 + j * 37 + k * 91) % 11
            mat = "diamante" if h in (0, 3) and (i * j) % 2 == 0 else ("piedra_clara" if h in (5, 6) else ("piedra_oscura" if h == 8 else "piedra"))
            vx.put(i, j, k, "bloque", mat)
guardar(13, vx, "el bloque de diamante de Planeta Vegetta.", [("piedra", (0.50, 0.50, 0.50), None, 8), ("piedra_clara", (0.62, 0.62, 0.62), None, 8), ("piedra_oscura", (0.36, 0.36, 0.38), None, 8),
                                                             ("diamante", (0.35, 0.95, 0.90), (0.15, 0.5, 0.5), 90)])

# 14. El patito de hule del referi: amarillo, con su camisa de rayas y su silbato.
u = 0.04
vx = Voxels(u)
for i in range(-12, 13):
    for j in range(-10, 14):
        for k in range(-9, 10):
            x, y, z = i + 0.5, j + 0.5, k + 0.5
            cuerpo = (x / 11.0) ** 2 + ((y + 4) / 6.5) ** 2 + (z / 8.5) ** 2 <= 1.0
            cola = x > 6 and (x - 6) / 5.0 + ((y + 1) / 4.0) ** 2 + (z / 4.0) ** 2 <= 1.0 and y > -3
            cabeza = ((x + 5) / 5.5) ** 2 + ((y - 6) / 5.5) ** 2 + (z / 5.5) ** 2 <= 1.0
            if cuerpo:
                raya = y < -2 and ((i // 2) % 2 == 0)
                vx.put(i, j, k, "cuerpo", ("raya_negra" if raya else "raya_blanca") if y < -2 and abs(z) < 8 else "amarillo")
            elif cola or cabeza:
                vx.put(i, j, k, "cabeza", "amarillo")
for j in range(5, 7):
    for k in range(-2, 3):
        for i in range(-14, -9):
            vx.put(i, j, k, "pico", "naranja")
for k in (-4, 4):
    vx.box("ojos", "negro", -9, -8, 7, 8, k - 1 if k > 0 else k, k if k > 0 else k + 1)
vx.box("silbato", "metal", -8, -6, -1, 0, -9, -8)
for n in range(5):
    vx.put(-6 + n, 1 + (n % 2), -8, "cordon", "negro")
guardar(14, vx, "el patito de hule del réferi.", [("amarillo", (1.0, 0.85, 0.15), (0.12, 0.09, 0.0), 48), ("naranja", (1.0, 0.5, 0.1), None), ("negro", (0.06, 0.06, 0.06), None),
                                                 ("raya_negra", (0.08, 0.08, 0.08), None), ("raya_blanca", (0.95, 0.95, 0.95), None), ("metal", (0.75, 0.75, 0.8), None, 80)])
