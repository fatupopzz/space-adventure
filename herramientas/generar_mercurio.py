# Genera los modelos de Mercurio: el consultorio de tarot de Madame Mercurio.
#   texturas/tarot.png: las 22 cartas del tarot (pintadas por arte_tarot.py) y el reverso, en un atlas de 8 x 3.
#   tarot.obj:          una carta con coordenadas UV: frente_N (una por carta, todas sobre el atlas) y el
#                       reverso. Se dibuja ocultando los frentes que no son.
#   carpa_tarot.obj:    la carpa de la adivina sobre el suelo gris de Mercurio: alfombra, mesa baja con mantel,
#                       bola de cristal, velas, cojines, estrellitas colgando y una ventana al Sol (enorme).
#   madame_mercurio.obj: Madame Mercurio, el planeta mismo de adivina: cabeza de Mercurio con crateres,
#                       turbante con gema y las alitas del dios Mercurio, aretes de argolla y chal. Sentada.
# Uso: python3 herramientas/generar_mercurio.py   (necesita Pillow)

import math, os, random
from PIL import Image, ImageDraw, ImageFont
from modelo3d import *
from arte_tarot import pintar_carta

random.seed(11)
AQUI = os.path.dirname(os.path.abspath(__file__))
TEXTURAS = os.path.join(MODELOS, "texturas")
FUENTE = os.path.join(AQUI, "..", "assets", "fuentes", "PixelifySans.ttf")
os.makedirs(TEXTURAS, exist_ok=True)

def mtl(*materiales):
    """Cada material: (nombre, Kd, Ke) o (nombre, Kd, Ke, d)."""
    out = ["# Materiales"]
    for nombre, kd, ke, *d in materiales:
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), "Ks 0.2 0.2 0.2", "Ns 24"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
        if d:
            out.append(f"d {d[0]:.2f}")
    return "\n".join(out) + "\n"

def caja_redonda(vx, parte, mat, i0, i1, j0, j1, k0, k1, r=1.0, espejo=False):
    """Una caja (rangos inclusivos de voxels) con las esquinas y aristas redondeadas con radio r."""
    for i in range(i0, i1 + 1):
        for j in range(j0, j1 + 1):
            for k in range(k0, k1 + 1):
                d = 0.0
                for c, a, b in ((i + 0.5, i0, i1 + 1), (j + 0.5, j0, j1 + 1), (k + 0.5, k0, k1 + 1)):
                    fuera = max(a + r - c, 0.0, c - (b - r))
                    d += fuera * fuera
                if d <= r * r:
                    vx.put(i, j, k, parte, mat, espejo)

def elipsoide(vx, parte, mat, c, r, si=None):
    """Llena un elipsoide de centro c y radios r (en voxels). si(i, j, k) da el material (o None)."""
    for i in range(math.floor(c[0] - r[0]) - 1, math.ceil(c[0] + r[0]) + 1):
        for j in range(math.floor(c[1] - r[1]) - 1, math.ceil(c[1] + r[1]) + 1):
            for k in range(math.floor(c[2] - r[2]) - 1, math.ceil(c[2] + r[2]) + 1):
                if sum(((p + 0.5 - q) / s) ** 2 for p, q, s in zip((i, j, k), c, r)) <= 1:
                    m = si(i, j, k) if si else mat
                    if m:
                        vx.put(i, j, k, parte if m == mat else f"{parte}_{m}", m)

# =====================================================================
# Las cartas: el atlas de texturas
# =====================================================================
CW, CH = 200, 320          # pixeles de cada carta en el atlas
COLS, FILAS = 8, 3
CARTAS = ["EL WIFI", "LA CAMIONETA", "EL EX", "EL PARCIAL", "LA TORRE", "EL SOL", "LA LUNA",
          "EL CANGREJO", "EL CHUCHO", "LA TORTILLA", "EL VOLCÁN", "EL PUERRO", "EL Z-BUFFER", "LA BATERÍA",
          "LA MUERTE", "EL TRÁFICO", "EL CAFÉ", "LA LLUVIA", "EL QUETZAL", "EL COMPILADOR", "EL CHISME", "EL PISTO"]
ROMANOS = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII", "XIII", "XIV",
           "XV", "XVI", "XVII", "XVIII", "XIX", "XX", "XXI", "XXII"]
REVERSO = len(CARTAS)       # el lugar del reverso en el atlas

# Cada carta la pinta arte_tarot.py (con supermuestreo, sombras y su marco dorado).
img = Image.new("RGB", (CW * COLS, CH * FILAS), (20, 10, 30))
for n in range(len(CARTAS) + 1):
    x0, y0 = (n % COLS) * CW, (n // COLS) * CH
    carta = pintar_carta(None if n == REVERSO else n, CARTAS[n] if n < REVERSO else "", ROMANOS[n] if n < REVERSO else "", FUENTE, CW, CH)
    img.paste(carta, (x0, y0))
img.save(os.path.join(TEXTURAS, "tarot.png"))

# ---------- el modelo de la carta ----------
# 0.36 x 0.6, centrada, el frente mira a -Z. Visto de frente, la derecha es -X: por eso la u crece hacia -X.
W2, H2, GROSOR = 0.18, 0.30, 0.004
def uv_de(n):
    c, f = n % COLS, n // COLS
    u0, u1 = c / COLS, (c + 1) / COLS
    v_arriba, v_abajo = 1 - f / FILAS, 1 - (f + 1) / FILAS
    # Medio texel hacia adentro, para que el filtro bilineal no agarre la carta de al lado.
    eu, ev = 0.5 / (CW * COLS), 0.5 / (CH * FILAS)
    return u0 + eu, u1 - eu, v_arriba - ev, v_abajo + ev
m = Modelo()
for n in range(len(CARTAS)):
    u0, u1, va, vb = uv_de(n)
    m.part(f"frente_{n:02d}", "tarot").quad(
        [(W2, -H2, -GROSOR), (-W2, -H2, -GROSOR), (-W2, H2, -GROSOR), (W2, H2, -GROSOR)], (0, 0, 1),
        uvs=[(u0, vb), (u1, vb), (u1, va), (u0, va)])
u0, u1, va, vb = uv_de(REVERSO)
m.part("reverso", "tarot").quad(
    [(-W2, -H2, GROSOR), (W2, -H2, GROSOR), (W2, H2, GROSOR), (-W2, H2, GROSOR)], (0, 0, -1),
    uvs=[(u0, vb), (u1, vb), (u1, va), (u0, va)])
canto = m.part("canto", "canto_oro")
for a, b in (((W2, -H2), (W2, H2)), ((-W2, -H2), (-W2, H2)), ((-W2, H2), (W2, H2)), ((-W2, -H2), (W2, -H2))):
    canto.quad([(a[0], a[1], -GROSOR), (b[0], b[1], -GROSOR), (b[0], b[1], GROSOR), (a[0], a[1], GROSOR)], (0, 0, 0))
m.write("tarot", f"""# Carta del tarot de Madame Mercurio (0.36 x 0.6, centrada, el frente mira a -Z), con UV sobre el atlas
# texturas/tarot.png. Partes frente_00 .. frente_{len(CARTAS) - 1} (una por carta, con dos digitos para que ningun
# nombre sea prefijo de otro: se ocultan las que no son),
# el reverso (+Z) y el canto dorado. Cartas: {", ".join(CARTAS)}.
""", """# Materiales

newmtl tarot
Kd 1.0 1.0 1.0
Ks 0.0 0.0 0.0
Ns 1
Ke 0.12 0.10 0.08
map_Kd texturas/tarot.png

newmtl canto_oro
Kd 1.0 0.78 0.30
Ks 0.4 0.4 0.4
Ns 32
Ke 0.10 0.07 0.0
""")

# =====================================================================
# La carpa
# =====================================================================
# Origen en el centro de la mesa, en el piso. Madame se sienta en +Z (mira a -Z), Miku en -Z.
# La mesa baja tiene la tabla en y = 0.55. La ventana al Sol esta en +Z, la puerta en -Z.
R_CARPA, ALTO_PARED = 4.5, 3.0
MESA_Y, R_MESA = 0.55, 0.95
m = Modelo()

# El suelo de Mercurio afuera: gris, con crateres (voxel grande)
suelo = Voxels(0.5, origen=(0.0, -0.5, 0.0))
crateres = [(random.uniform(-26, 26), random.uniform(-26, 26), random.uniform(1.5, 4.5)) for _ in range(16)]
for i in range(-56, 56):
    for k in range(-56, 56):
        x, _, z = suelo.centro(i, 0, k)
        r = math.hypot(x, z)
        if r > 28 or r < R_CARPA - 0.6:
            continue
        mat = "roca_mercurio"
        hueco = False
        for (cx, cz, cr) in crateres:
            dc = math.hypot(x - cx, z - cz)
            if math.hypot(cx, cz) > R_CARPA + cr + 1 and dc < cr:
                hueco = dc < cr * 0.75
                mat = "roca_borde" if not hueco else "roca_oscura"
        if (i * 7 + k * 13) % 11 == 0 and mat == "roca_mercurio":
            mat = "roca_clara"
        suelo.put(i, 0 if not hueco else -1, k, f"suelo_{mat}", mat)
        if mat == "roca_borde":
            suelo.put(i, 1, k, "suelo_roca_borde", mat)
suelo.mesh(m)

v = Voxels(0.1)
# La alfombra redonda adentro, con anillos dorados
for i in range(-46, 46):
    for k in range(-46, 46):
        x, _, z = v.centro(i, 0, k)
        r = math.hypot(x, z)
        if r <= R_CARPA:
            if r < 2.6:
                mat = "alfombra_oro" if abs(r - 2.3) < 0.07 or abs(r - 1.5) < 0.06 else "alfombra"
            else:
                mat = "piso_carpa"
            v.put(i, 0, k, f"piso_{mat}", mat)

# Las paredes a rayas, con la ventana (+Z) y la puerta (-Z)
def en_hueco(a, y):
    # a: angulo desde +X; la ventana mira a +Z (a = pi/2), la puerta a -Z (a = -pi/2)
    ventana = abs(a - math.pi / 2) < 0.30 and 0.9 <= y <= 2.5
    puerta = abs(a + math.pi / 2) < 0.24 and y <= 2.6
    return ventana or puerta
for i in range(-47, 47):
    for k in range(-47, 47):
        x, _, z = v.centro(i, 0, k)
        r = math.hypot(x, z)
        if R_CARPA - 0.05 <= r < R_CARPA + 0.1:
            a = math.atan2(z, x)
            raya = int((a + math.pi) / (2 * math.pi) * 24) % 2
            for j in range(1, int(ALTO_PARED / 0.1)):
                y = j * 0.1
                if en_hueco(a, y):
                    # El marco dorado alrededor de los huecos
                    continue
                cerca = any(en_hueco(a + da, y + dy) for da in (-0.03, 0.03) for dy in (-0.1, 0.1))
                mat = "oro_carpa" if cerca else ("tela_morada" if raya else "tela_azul")
                v.put(i, j, k, f"pared_{mat}", mat)
            # La cenefa de arriba, con picos
            v.put(i, int(ALTO_PARED / 0.1), k, "cenefa", "oro_carpa")
# El techo: un cono que sube hasta el centro
for i in range(-47, 47):
    for k in range(-47, 47):
        x, _, z = v.centro(i, 0, k)
        r = math.hypot(x, z)
        if r < R_CARPA + 0.15:
            y = ALTO_PARED + (R_CARPA + 0.15 - r) * 0.55
            a = math.atan2(z, x)
            raya = int((a + math.pi) / (2 * math.pi) * 12) % 2
            mat = "tela_morada" if raya else "tela_azul"
            v.put(i, int(y / 0.1), k, f"techo_{mat}", mat)
# La mesa baja: pata gruesa, tabla y mantel con fleco dorado
for i in range(-11, 11):
    for k in range(-11, 11):
        x, _, z = v.centro(i, 0, k)
        r = math.hypot(x, z)
        if r <= R_MESA:
            v.put(i, int(MESA_Y / 0.1) - 1, k, "mantel", "mantel")
            if r > R_MESA - 0.12:
                for j in range(1, int(MESA_Y / 0.1) - 1):
                    v.put(i, j, k, "mantel_caida" if j > 1 else "fleco", "mantel" if j > 1 else "oro_carpa")
# Los cojines: el de Miku (-Z) y el de Madame (+Z)
for (cz, parte, mat) in ((-1.25, "cojin_miku", "cojin_turquesa"), (1.3, "cojin_madame", "cojin_rosa")):
    for i in range(-5, 5):
        for k in range(-5, 5):
            x, _, z = v.centro(i, 0, k)
            if math.hypot(x, z) <= 0.45:
                for j in range(1, 3):
                    v.put(i, j, k + round(cz / 0.1), parte, mat)
# Los candelabros altos en el piso, a los lados de Madame
VELAS = []
for lado in (-1, 1):
    x0, z0 = lado * 1.9, 1.6
    for j in range(1, 16):
        r = 0.18 if j < 3 or j > 13 else 0.06
        for i in range(-2, 2):
            for k in range(-2, 2):
                x, _, z = v.centro(i, 0, k)
                if math.hypot(x, z) <= r + 0.02:
                    v.put(i + round(x0 / 0.1), j, k + round(z0 / 0.1), "candelabros", "oro_carpa")
    VELAS.append((x0, 1.75, z0))
# Las estrellas y lunitas que cuelgan del techo
for n in range(10):
    a = n / 10 * 2 * math.pi + 0.3
    r = 2.0 + (n % 3) * 0.7
    x0, z0 = r * math.cos(a), r * math.sin(a)
    largo = 3 + (n * 7) % 5
    i0, k0 = round(x0 / 0.1), round(z0 / 0.1)
    techo = ALTO_PARED + (R_CARPA + 0.15 - r) * 0.55
    j_techo = int(techo / 0.1)
    for j in range(j_techo - largo * 2, j_techo):
        v.put(i0, j, k0, "hilos", "hilo")
    jb = j_techo - largo * 2 - 1
    if n % 2 == 0:
        for (di, dj) in ((0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)):
            v.put(i0 + di, jb + dj, k0, "estrellas", "estrella")
    else:
        for (di, dj) in ((0, 1), (-1, 0), (-1, -1), (0, -2)):
            v.put(i0 + di, jb + dj, k0, "estrellas", "estrella")
v.mesh(m)

# Las cosas chiquitas (voxel de 0.03): la bola de cristal con su base, las velitas de la mesa y el fuego
w = Voxels(0.03, origen=(0.0, MESA_Y, 0.0))
BOLA = (0.0, 0.0 + 0.30, 0.42)  # centro de la bola en el mundo
for i in range(-6, 6):
    for k in range(-6, 6):
        x, _, z = w.centro(i, 0, k)
        for j in range(0, 3):
            if math.hypot(x, z) <= 0.16 - j * 0.02:
                w.put(i, j, k + 14, "base_bola", "oro_carpa")
elipsoide(w, "bola", "cristal", (-0.0, 7.5 + 0.0, 14.0), (5.6, 5.6, 5.6))
# Velitas sobre la mesa (tres, en el borde de Madame)
for (x0, z0) in ((-0.62, 0.55), (0.62, 0.55), (-0.75, 0.05)):
    i0, k0 = round(x0 / 0.03), round(z0 / 0.03)
    for j in range(0, 7):
        w.box("velitas", "cera", i0 - 1, i0, j, j, k0 - 1, k0)
    w.box("mechas", "hilo", i0, i0, 7, 7, k0, k0)
    VELAS.append((x0, MESA_Y + 0.27, z0))
w.mesh(m)
fuego = Voxels(0.03, origen=(0.0, 0.0, 0.0))
for (x0, y0, z0) in VELAS:
    i0, j0, k0 = round(x0 / 0.03), round(y0 / 0.03), round(z0 / 0.03)
    grande = y0 > 1.0
    for dj, r in enumerate((1, 1, 1, 0) if grande else (1, 0, 0)):
        for di in range(-r, 1):
            for dk in range(-r, 1):
                fuego.put(i0 + di, j0 + dj, k0 + dk, "llamas", "llama")
fuego.mesh(m)
m.write("carpa_tarot", f"""# El consultorio de Madame Mercurio: carpa a rayas (radio {R_CARPA}) sobre el suelo gris y con crateres de
# Mercurio. Origen en el centro de la mesa, en el piso. Mesa baja con mantel (tabla en y = {MESA_Y}, radio {R_MESA}),
# bola de cristal en {BOLA}, cojin de Miku en z = -1.25 y de Madame en z = 1.3, ventana al Sol en +Z y puerta en -Z.
# velas: {" ".join(f"{x:.2f} {y:.2f} {z:.2f}" for x, y, z in VELAS)}
""", mtl(("roca_mercurio", (0.50, 0.47, 0.45), None), ("roca_clara", (0.62, 0.58, 0.55), None),
         ("roca_borde", (0.58, 0.55, 0.52), None), ("roca_oscura", (0.30, 0.28, 0.28), None),
         ("alfombra", (0.55, 0.12, 0.30), None), ("alfombra_oro", (1.0, 0.75, 0.30), None), ("piso_carpa", (0.30, 0.18, 0.30), None),
         ("tela_morada", (0.42, 0.18, 0.62), None), ("tela_azul", (0.16, 0.14, 0.42), None), ("oro_carpa", (1.0, 0.76, 0.30), (0.10, 0.07, 0.0)),
         ("mantel", (0.30, 0.10, 0.45), None), ("cojin_turquesa", (0.20, 0.75, 0.72), None), ("cojin_rosa", (0.95, 0.40, 0.60), None),
         ("hilo", (0.9, 0.85, 0.7), None), ("estrella", (1.0, 0.85, 0.40), (0.45, 0.35, 0.1)),
         ("cristal", (0.70, 0.55, 1.0), (0.35, 0.22, 0.55), 0.62), ("cera", (0.98, 0.94, 0.85), (0.1, 0.08, 0.05)),
         ("llama", (1.0, 0.75, 0.3), (1.0, 0.6, 0.2))))

# =====================================================================
# Madame Mercurio
# =====================================================================
# Sentada (el origen es el piso bajo el cojin), mira hacia -Z. Voxel de 0.04.
U = 0.04
vx = Voxels(U)
# La falda/tunica sentada: un monticulo ancho, y el torso
caja_redonda(vx, "tunica", "tunica", 0, 8, 0, 8, -6, 5, 3.0, espejo=True)
caja_redonda(vx, "torso", "tunica", 0, 5, 9, 16, -3, 3, 1.5, espejo=True)
for (i, j, k) in ((2, 4, -7), (6, 6, -6), (4, 13, -4), (1, 2, -7), (7, 2, -5)):
    vx.put(i, j, k, "estrellitas_tunica", "estrella_tunica", espejo=True)
# El chal fucsia sobre los hombros, cayendo adelante
caja_redonda(vx, "chal", "chal", 0, 7, 14, 17, -4, 4, 1.2, espejo=True)
for j in range(9, 14):
    vx.box("chal", "chal", 4, 6, j, j, -5, -4, espejo=True)
for i in range(4, 7):
    vx.put(i, 8, -5, "fleco_chal", "oro_madame", espejo=True)
# Los brazos estirados hacia adelante, con las manos sobre la bola (abiertas, misteriosas)
caja_redonda(vx, "brazos", "tunica", 5, 7, 11, 14, -9, -3, 0.8, espejo=True)
caja_redonda(vx, "manos", "piel_mercurio", 4, 6, 11, 13, -12, -10, 0.7, espejo=True)
vx.box("anillos", "oro_madame", 5, 5, 11, 11, -11, -11, espejo=True)
# La cabeza: Mercurio, gris tirando a cafe, con crateres
CABEZA = (-0.0, 25.0, 0.0)
RC = 8.0
CRATERES = [(0.55, 0.65, 0.5), (-0.6, 0.35, -0.7), (0.8, -0.2, 0.55), (-0.3, 0.8, 0.5), (0.2, -0.5, 0.85), (-0.85, -0.1, 0.5),
            (0.3, 0.3, 0.9), (-0.5, -0.6, 0.6)]
CRATERES = [tuple(q / math.sqrt(sum(p * p for p in cr)) for q in cr) for cr in CRATERES]
def material_cabeza(i, j, k):
    p = (i + 0.5 - CABEZA[0], j + 0.5 - CABEZA[1], k + 0.5 - CABEZA[2])
    l = math.sqrt(sum(q * q for q in p)) or 1
    n = tuple(q / l for q in p)
    if n[2] < -0.55 and abs(n[0]) < 0.6 and -0.55 < n[1] < 0.35:
        return "piel_mercurio"   # la cara, lisita
    for c in CRATERES:
        if sum(a * b for a, b in zip(n, c)) > 0.95:
            return "crater"
        if sum(a * b for a, b in zip(n, c)) > 0.91:
            return "crater_borde"
    return "piel_mercurio"
elipsoide(vx, "cabeza", "piel_mercurio", CABEZA, (RC, RC, RC), si=material_cabeza)
# La cara se aplana un poco al frente, para que los ojos queden pegados
for i in range(-7, 7):
    for j in range(17, 33):
        if (i + 0.5) ** 2 + (j + 0.5 - CABEZA[1]) ** 2 <= 6.0 ** 2:
            for k in (-8, -7):
                vx.put(i, j, k, "cabeza", "piel_mercurio")
# El turbante: morado, enrollado, con la gema de Mercurio al frente
for j in range(29, 36):
    rr = 8.4 - max(0, j - 31) * 1.2
    for i in range(-10, 10):
        for k in range(-10, 10):
            if (i + 0.5) ** 2 + (k + 0.5) ** 2 <= rr * rr:
                vuelta = (j + (i + k) // 3) % 3 == 0
                vx.put(i, j, k, "turbante_vuelta" if vuelta else "turbante", "turbante_claro" if vuelta else "turbante")
vx.box("gema_engaste", "oro_madame", -2, 1, 31, 34, -9, -9)
vx.box("gema", "gema", -1, 0, 32, 33, -10, -10)
vx.box("pluma", "oro_madame", 0, 0, 35, 39, -7, -7)
vx.box("pluma_punta", "chal", -1, 1, 40, 41, -7, -7)
# Las alitas del dios Mercurio a los lados del turbante
for t in range(6):
    for h in range(4 - t // 2):
        vx.put(9 + t, 31 + h + t // 2, 1 - t // 3, "alas", "ala", espejo=True)
        vx.put(9 + t, 31 + h + t // 2, 2 - t // 3, "alas", "ala", espejo=True)
# Aretes de argolla
for (dj, dk) in ((0, 0), (-1, -1), (-2, -1), (-3, 0), (-2, 1), (-1, 1)):
    vx.put(8, 22 + dj, dk, "aretes", "oro_madame", espejo=True)
m = Modelo()
vx.mesh(m)
# La cara: las de siempre y la "mistica" (ojos entrecerrados) con labios morados
CARAS_MADAME = dict(CARAS)
CARAS_MADAME["mistica"] = [
    ("ojos", "ojo", espejo(pixeles(["#####", ".###."], 2, 8))),
    ("sombra", "sombra", espejo(pixeles(["#####"], 2, 10))),
    ("boca", "labios", pixeles(["####"], -2, 3)),
]
expresiones(m, z_frente=-RC * U, px=0.021, x_centro=0.0, y_base=(CABEZA[1] - 6.0) * U, caras=CARAS_MADAME)
m.write("madame_mercurio", """# Madame Mercurio (voxel de 0.04): el planeta Mercurio de adivina. Cabeza gris con crateres, turbante morado
# con gema y pluma, las alitas del dios Mercurio, aretes de argolla, chal fucsia y las manos sobre la bola de
# cristal. Sentada: origen en el piso bajo el cojin, mira hacia -Z. Caras de siempre y "mistica".
""", mtl(("tunica", (0.22, 0.12, 0.40), None), ("estrella_tunica", (1.0, 0.85, 0.4), (0.3, 0.22, 0.05)),
         ("chal", (0.95, 0.25, 0.60), None), ("oro_madame", (1.0, 0.78, 0.30), (0.12, 0.08, 0.0)),
         ("piel_mercurio", (0.66, 0.62, 0.60), None), ("crater", (0.40, 0.37, 0.37), None), ("crater_borde", (0.78, 0.74, 0.70), None),
         ("turbante", (0.45, 0.20, 0.70), None), ("turbante_claro", (0.60, 0.35, 0.85), None),
         ("gema", (0.40, 1.0, 0.90), (0.25, 0.7, 0.6)), ("ala", (0.98, 0.98, 1.0), (0.15, 0.15, 0.2)),
         ("sombra", (0.55, 0.25, 0.75), None), ("labios", (0.55, 0.15, 0.45), None)) + MTL_CARA)
