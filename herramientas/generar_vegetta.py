# Genera PLANETA VEGETTA (homenaje a la serie de Minecraft de Vegetta777 y a Willyrex; los
# dibujos son propios):
#   modelos/texturas/bloques.png: el atlas de bloques en pixel art (16 x 16 cada uno, agrandado
#       x4 para que el filtro bilineal no los emborrone): pasto, tierra, piedra, tablas, tronco,
#       hojas, mena de diamante, slime, lava, oro, diamante, el bloque morado con la V, obsidiana,
#       la bandera de la meta y nube. El juego arma las pistas de parkour con el.
#   (el planeta en el sistema solar, un cubo de bloques con su casita, lo arma el juego:
#       src/planetas.rs)
#   modelos/diamante_mc.obj: el diamante que flota en la pista (se recoge).
# Uso: python3 herramientas/generar_vegetta.py

import os, random
from PIL import Image
from modelo3d import *

TEXTURAS = os.path.join(MODELOS, "texturas")
os.makedirs(TEXTURAS, exist_ok=True)

# Los bloques del atlas, en orden (el juego usa los mismos numeros).
NOMBRES = ["pasto_arriba", "pasto_lado", "tierra", "piedra", "tablas", "tronco", "hojas", "mena_diamante",
           "slime", "lava", "oro", "diamante", "vegetta", "obsidiana", "meta", "nube",
           # Los del servidor de minijuegos (TNT Run, la mineria y el lobby):
           "arena", "tnt_lado", "tnt_arriba", "portal", "vidrio", "roca_madre", "mena_oro", "mena_carbon",
           "ladrillo", "lana_morada", "lana_verde", "piedra_lisa"]
T, X = 16, 4  # pixeles de cada bloque y cuanto se agranda


def ruido(rnd, base, var):
    return tuple(max(0, min(255, c + rnd.randint(-var, var))) for c in base)


def bloque(n):
    rnd = random.Random(100 + n)
    px = [[(0, 0, 0)] * T for _ in range(T)]
    for y in range(T):
        for x in range(T):
            if n == 0:  # pasto de arriba
                px[y][x] = ruido(rnd, (95, 160, 60), 22)
            elif n == 1:  # pasto de lado: tierra con el pasto que gotea
                gotea = 3 + (1 if rnd.random() < 0.4 else 0) + (1 if (x * 7) % 5 == 0 else 0)
                px[y][x] = ruido(rnd, (95, 160, 60), 18) if y < gotea else ruido(rnd, (135, 96, 64), 18)
            elif n == 2:  # tierra
                px[y][x] = ruido(rnd, (135, 96, 64), 18) if rnd.random() > 0.08 else (90, 62, 40)
            elif n == 3:  # piedra con grietas
                px[y][x] = ruido(rnd, (125, 125, 125), 14)
            elif n == 4:  # tablas: cuatro tablas con sus junturas
                c = (170, 130, 80) if (y // 4) % 2 == 0 else (160, 120, 72)
                px[y][x] = (110, 80, 45) if y % 4 == 3 or (x == (y // 4 * 5 + 3) % T) else ruido(rnd, c, 8)
            elif n == 5:  # tronco: corteza en rayas verticales
                px[y][x] = ruido(rnd, (100, 76, 46) if x % 4 in (0, 1) else (80, 60, 36), 8)
            elif n == 6:  # hojas, con hoyitos oscuros
                px[y][x] = ruido(rnd, (60, 130, 45), 25) if rnd.random() > 0.15 else (30, 70, 25)
            elif n == 7:  # mena de diamante: piedra con piedritas celestes
                px[y][x] = ruido(rnd, (125, 125, 125), 14)
            elif n == 8:  # slime: verde claro con un cuadro adentro
                borde = x in (0, T - 1) or y in (0, T - 1)
                adentro = 4 <= x <= 11 and 4 <= y <= 11
                px[y][x] = (90, 190, 80) if borde else ((80, 170, 70) if adentro else ruido(rnd, (120, 215, 100), 10))
            elif n == 9:  # lava: naranja con remolinos amarillos
                k = (x * 3 + y * 5 + (x * y) % 7) % 9
                px[y][x] = (255, 200, 60) if k < 2 else ruido(rnd, (230, 100, 20), 20)
            elif n == 10:  # oro
                borde = x in (0, T - 1) or y in (0, T - 1)
                px[y][x] = (200, 150, 30) if borde else ruido(rnd, (250, 215, 70), 12)
            elif n == 11:  # bloque de diamante
                borde = x in (0, T - 1) or y in (0, T - 1)
                brillo = (x + y) % 7 == 0
                px[y][x] = (40, 160, 160) if borde else ((210, 255, 255) if brillo else ruido(rnd, (100, 230, 225), 10))
            elif n == 12:  # el bloque de Vegetta: morado con la V blanca
                px[y][x] = ruido(rnd, (120, 50, 170), 10)
            elif n == 13:  # obsidiana
                px[y][x] = ruido(rnd, (30, 18, 45), 10) if rnd.random() > 0.1 else (70, 40, 100)
            elif n == 14:  # la meta: cuadros blanco y negro
                px[y][x] = (240, 240, 240) if ((x // 4) + (y // 4)) % 2 == 0 else (25, 25, 25)
            elif n == 15:  # nube
                px[y][x] = ruido(rnd, (245, 245, 250), 6)
            elif n == 16:  # arena, con granitos
                px[y][x] = ruido(rnd, (222, 206, 150), 12) if rnd.random() > 0.06 else (190, 170, 115)
            elif n == 17:  # TNT de lado: rojo, con la franja blanca y las letras
                franja = 6 <= y <= 9
                px[y][x] = ((240, 240, 235) if franja else ruido(rnd, (200, 45, 40), 10)) if x % 4 != 3 or franja else (150, 30, 28)
            elif n == 18:  # TNT de arriba: rojo con la mecha al centro
                px[y][x] = ruido(rnd, (200, 45, 40), 10)
            elif n == 19:  # portal: remolino morado
                k = (x - 7.5) ** 2 + (y - 7.5) ** 2
                giro = int((k ** 0.5) * 1.3 + (x + y) * 0.4) % 4
                px[y][x] = [(150, 60, 230), (190, 110, 255), (120, 40, 200), (230, 170, 255)][giro]
            elif n == 20:  # vidrio: casi blanco con marco y dos rayitas de brillo
                borde = x in (0, T - 1) or y in (0, T - 1)
                brillo = (x - y) in (4, 5) or (x - y) in (-6,)
                px[y][x] = (200, 225, 235) if borde else ((250, 255, 255) if brillo else (160, 200, 220))
            elif n == 21:  # roca madre: gris y negro revuelto
                px[y][x] = ruido(rnd, (70, 70, 70), 20) if rnd.random() > 0.35 else ruido(rnd, (30, 30, 30), 10)
            elif n == 22:  # mena de oro: piedra con piedritas amarillas
                px[y][x] = ruido(rnd, (125, 125, 125), 14)
            elif n == 23:  # mena de carbon: piedra con piedritas negras
                px[y][x] = ruido(rnd, (125, 125, 125), 14)
            elif n == 24:  # ladrillo
                fila = y // 4
                junta = y % 4 == 3 or (x + (4 if fila % 2 else 0)) % 8 == 7
                px[y][x] = (200, 190, 175) if junta else ruido(rnd, (160, 70, 55), 12)
            elif n == 25:  # lana morada (la de Vegetta)
                px[y][x] = ruido(rnd, (140, 70, 200), 10)
            elif n == 26:  # lana verde (la de Willy)
                px[y][x] = ruido(rnd, (70, 170, 70), 10)
            else:  # piedra lisa (el piso del lobby)
                borde = x in (0, T - 1) or y in (0, T - 1)
                px[y][x] = (120, 120, 125) if borde else ruido(rnd, (165, 165, 170), 6)
    if n == 3:
        for _ in range(3):
            x, y = rnd.randrange(T), rnd.randrange(T)
            for k in range(rnd.randint(3, 6)):
                if 0 <= x < T and 0 <= y < T:
                    px[y][x] = (95, 95, 95)
                x += rnd.choice((-1, 0, 1))
                y += 1
    if n == 7:
        for cx, cy in ((4, 4), (11, 6), (6, 11), (12, 12)):
            for dx, dy in ((0, 0), (1, 0), (0, 1), (1, 1), (-1, 0)):
                x, y = cx + dx, cy + dy
                if 0 <= x < T and 0 <= y < T:
                    px[y][x] = (90, 240, 230) if (dx + dy) % 2 == 0 else (200, 255, 255)
    if n in (22, 23):
        color = (250, 215, 70) if n == 22 else (25, 25, 28)
        for cx, cy in ((4, 4), (11, 6), (6, 11), (12, 12)):
            for dx, dy in ((0, 0), (1, 0), (0, 1)):
                px[cy + dy][cx + dx] = color
    if n == 17:
        # Las letras TNT en la franja (de 3 x 4 pixeles, en negro).
        for i, letra in enumerate(["###/.#./.#./.#.", "#.#/###/#.#/#.#", "###/.#./.#./.#."]):
            for fy, fila in enumerate(letra.split("/")):
                for fx, c in enumerate(fila):
                    if c == "#":
                        px[6 + fy][2 + i * 4 + fx] = (30, 30, 30)
    if n == 18:
        for y in range(6, 10):
            for x in range(6, 10):
                px[y][x] = (90, 80, 70)
        px[5][8] = (240, 240, 240)
    if n == 12:
        for y in range(3, 13):
            k = (y - 3) // 2
            for x in (3 + k, 12 - k, 4 + k, 11 - k):
                px[y][x] = (250, 250, 250)
    return px


atlas = Image.new("RGB", (8 * T * X, 4 * T * X))
for n in range(32):
    px = bloque(n)
    img = Image.new("RGB", (T, T))
    for y in range(T):
        for x in range(T):
            img.putpixel((x, y), px[y][x])
    img = img.resize((T * X, T * X), Image.NEAREST)
    atlas.paste(img, ((n % 8) * T * X, (n // 8) * T * X))
atlas.save(os.path.join(TEXTURAS, "bloques.png"))
print("bloques.png:", atlas.size)


def mtl(*materiales):
    out = ["# Materiales"]
    for nombre, kd, ke in materiales:
        out += ["", f"newmtl {nombre}", "Kd " + " ".join(f"{c:.2f}" for c in kd), "Ks 0.15 0.15 0.15", "Ns 12"]
        if ke:
            out.append("Ke " + " ".join(f"{c:.2f}" for c in ke))
    return "\n".join(out) + "\n"


# --- El planeta (el cubo de bloques) lo arma el juego: src/planetas.rs ---


# --- El diamante que flota: el dibujito de una gema, con grosor ---
GEMA = ["....####....",
        "...#aaab#...",
        "..#aabbbb#..",
        ".#aabbbbcc#.",
        "#aabbbbbccc#",
        "#abbbbbbbcc#",
        ".#bbbbbbcc#.",
        "..#bbbbcc#..",
        "...#bbcc#...",
        "....#cc#....",
        ".....##....."]
vx = Voxels(0.04)
colores = {"#": "orilla", "a": "brillo", "b": "gema", "c": "sombra"}
for r, fila in enumerate(GEMA):
    for c, ch in enumerate(fila):
        if ch in colores:
            for k in (-1, 0):
                vx.put(c - 6, len(GEMA) - r, k, colores[ch], colores[ch])
m = Modelo()
vx.mesh(m)
m.write("diamante_mc", "# El diamante de Planeta Vegetta: una gema de pixeles con grosor. Base en y = 0, de frente a -Z.\n",
        mtl(("orilla", (0.05, 0.30, 0.30), None), ("brillo", (0.85, 1.0, 1.0), (0.4, 0.5, 0.5)), ("gema", (0.35, 0.92, 0.88), (0.12, 0.35, 0.33)),
            ("sombra", (0.15, 0.62, 0.62), (0.04, 0.15, 0.15))))
