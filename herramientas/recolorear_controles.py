# Pasa las hojas del pack "Controllers and Keyboard" de Vryell (teclas, botones de PlayStation, Xbox
# y Switch, sticks y raton) a los colores del juego y las deja en assets/ui/controles/.
# Solo cambian los grises azulados del pack (el cuerpo de las teclas y los botones, sus sombras y
# sus orillas), que pasan a los lilas y morados de la interfaz con la misma luz: los colores de
# las letras de Xbox (A verde, B roja...) y de las figuras de PlayStation se quedan como estan.
# Para las teclas se usa la hoja clara (tecla lila, letra oscura), como las teclas que dibujaba el
# juego antes.
# Uso: python3 herramientas/recolorear_controles.py "<carpeta Controllers and Keyboard>"
import os, sys
from PIL import Image

base = sys.argv[1]
aqui = os.path.dirname(os.path.abspath(__file__))
destino = os.path.join(aqui, "..", "assets", "ui", "controles")

# Gris del pack -> lila del juego (de lo mas oscuro a lo mas claro).
CAMBIOS = {
    (20, 24, 46): (26, 14, 50),
    (43, 43, 69): (54, 36, 96),
    (58, 63, 94): (74, 54, 124),
    (64, 73, 115): (92, 66, 156),
    (76, 104, 133): (124, 98, 196),
    (104, 111, 153): (150, 130, 205),
    (163, 167, 194): (200, 188, 238),
    (223, 224, 232): (232, 224, 252),
    (245, 255, 232): (252, 246, 255),
}

HOJAS = ["kb_light_all", "controller_ps", "controller_xbox", "controller_switch", "sticks", "mouse", "controller_minimal"]

os.makedirs(destino, exist_ok=True)
for nombre in HOJAS:
    hoja = Image.open(os.path.join(base, nombre + ".png")).convert("RGBA")
    px = hoja.load()
    for y in range(hoja.height):
        for x in range(hoja.width):
            r, g, b, a = px[x, y]
            if a and (r, g, b) in CAMBIOS:
                px[x, y] = CAMBIOS[(r, g, b)] + (a,)
            # El cuadrado de PlayStation (fila 2), amarillo: rosa, se confundia con el circulo rojo.
            if nombre == "controller_ps" and 32 <= y < 48 and (r, g, b) == (255, 82, 119):
                px[x, y] = (255, 209, 64, a)
    hoja.save(os.path.join(destino, nombre + ".png"))
    print(f"{nombre}.png: {hoja.width}x{hoja.height}")
