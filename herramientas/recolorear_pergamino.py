# Arma assets/ui/pergamino.png con piezas del pack "Humble Gift" (HumblePixel) en los colores del
# juego: el pergamino y el globo de dialogo en dos versiones (papel lila y noche morada, cambiando
# la paleta pixel por pixel) y el marco con diamantes y el divisor en blanco (se tiñen al dibujar).
# Uso: python3 herramientas/recolorear_pergamino.py "<ruta al SpriteSheet.png del pack>"
import sys, os
from PIL import Image

hoja = Image.open(sys.argv[1]).convert("RGBA")
PAPEL = {
    (238, 189, 138): (232, 224, 252), (255, 206, 150): (248, 244, 255), (254, 222, 163): (252, 249, 255),
    (218, 164, 117): (206, 192, 240), (201, 139, 92): (180, 160, 228), (117, 36, 56): (124, 78, 210),
    (65, 29, 49): (68, 36, 124), (27, 34, 54): (24, 16, 48),
}
NOCHE = {
    (238, 189, 138): (34, 22, 66), (255, 206, 150): (64, 46, 112), (254, 222, 163): (72, 52, 124),
    (218, 164, 117): (26, 16, 52), (201, 139, 92): (20, 12, 42), (117, 36, 56): (185, 140, 255),
    (65, 29, 49): (110, 70, 190), (27, 34, 54): (8, 4, 20),
}

def pieza(x, y, w, h, paleta=None, blanco=False):
    p = hoja.crop((x, y, x + w, y + h))
    px = p.load()
    for j in range(h):
        for i in range(w):
            r, g, b, a = px[i, j]
            if a == 0:
                continue
            if blanco:
                px[i, j] = (255, 255, 255, a)
            elif paleta and (r, g, b) in paleta:
                px[i, j] = paleta[(r, g, b)] + (a,)
    return p

atlas = Image.new("RGBA", (530, 330), (0, 0, 0, 0))
atlas.paste(pieza(32, 1120, 183, 280, PAPEL), (0, 0))      # pergamino lila
atlas.paste(pieza(32, 1120, 183, 280, NOCHE), (184, 0))    # pergamino noche
atlas.paste(pieza(38, 983, 109, 34, PAPEL), (368, 0))      # globo lila
atlas.paste(pieza(38, 983, 109, 34, NOCHE), (368, 40))     # globo noche
atlas.paste(pieza(268, 1135, 159, 249, blanco=True), (368, 80))  # marco con diamantes
atlas.paste(pieza(500, 290, 56, 12, blanco=True), (0, 290))      # divisor
destino = os.path.join(os.path.dirname(__file__), "..", "assets", "ui", "pergamino.png")
atlas.save(destino)
print("pergamino.png:", atlas.size)
