# Vistas previas de los modelos con un rasterizador por software chiquito (culling + z-buffer + blending).
# Uso: python3 herramientas/preview.py [miku|nave|saturno|planetas|todo]  ->  previews/<modelo>.png
import math
import os
import sys
import numpy as np
from PIL import Image, ImageDraw

AQUI = os.path.dirname(os.path.abspath(__file__))
MODELOS, PREVIEWS = os.path.join(AQUI, "..", "modelos"), os.path.join(AQUI, "..", "previews")
EMOCIONES = ["feliz", "asustada", "espiral", "enamorada", "dormida", "serio"]


def load(nombre):
    mats, cur = {}, None
    for ln in open(os.path.join(MODELOS, nombre + ".mtl")):
        t = ln.split()
        if not t: continue
        if t[0] == "newmtl": cur = t[1]; mats[cur] = {"Kd": (1, 1, 1), "Ke": (0, 0, 0), "d": 1.0}
        elif t[0] in ("Kd", "Ke"): mats[cur][t[0]] = tuple(map(float, t[1:4]))
        elif t[0] == "d": mats[cur]["d"] = float(t[1])
    V, N, parts = [], [], []
    for ln in open(os.path.join(MODELOS, nombre + ".obj")):
        t = ln.split()
        if not t: continue
        if t[0] == "v": V.append(tuple(map(float, t[1:4])))
        elif t[0] == "vn": N.append(tuple(map(float, t[1:4])))
        elif t[0] == "o": parts.append({"name": t[1], "f": []})
        elif t[0] == "usemtl": parts[-1]["mat"] = t[1]
        elif t[0] == "f": parts[-1]["f"].append([tuple(int(x) - 1 for x in p.split("//")) for p in t[1:]])
    return {"V": np.array(V), "N": np.array(N), "parts": parts, "mats": mats}


def place(model, pos=(0, 0, 0), scale=1.0):
    return dict(model, V=model["V"] * scale + np.array(pos))


def render(scene, eye, target, size, emotion="feliz", fov=30, bg=(28, 24, 40)):
    eye, target = np.array(eye, float), np.array(target, float)
    f = target - eye; f /= np.linalg.norm(f)
    r = np.cross(f, [0, 1, 0]); r /= np.linalg.norm(r)
    R = np.array([r, np.cross(r, f), -f])
    img = np.zeros((size, size, 3)); img[:] = np.array(bg) / 255
    zbuf = np.full((size, size), np.inf)
    light = np.array([-0.4, 0.7, -0.6]); light /= np.linalg.norm(light)
    fpx = size / 2 / math.tan(math.radians(fov) / 2)
    ys, xs = np.mgrid[0:size, 0:size] + 0.5
    jobs = []
    for model in scene:
        cam = (model["V"] - eye) @ R.T
        proj = (size / 2 + fpx * cam[:, 0] / -cam[:, 2], size / 2 - fpx * cam[:, 1] / -cam[:, 2], -cam[:, 2])
        for p in model["parts"]:
            if p["name"].startswith("cara_") and not p["name"].startswith(f"cara_{emotion}_"):
                continue
            jobs.append((model, proj, p))

    def draw(model, proj, part, blend):
        sx, sy, depth = proj
        N = model["N"]
        m = model["mats"][part["mat"]]
        kd, ke, alpha = np.array(m["Kd"]), np.array(m["Ke"]), m["d"]
        for f in part["f"]:
            (a, na), (b, nb), (c, nc) = f
            if min(depth[a], depth[b], depth[c]) <= 0.01: continue
            x0, y0, x1, y1, x2, y2 = sx[a], sy[a], sx[b], sy[b], sx[c], sy[c]
            area = (x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0)
            if area >= 0: continue  # culling: CCW en el mundo = area negativa con Y de pantalla hacia abajo
            minx, maxx = int(max(min(x0, x1, x2), 0)), int(min(max(x0, x1, x2) + 1, size))
            miny, maxy = int(max(min(y0, y1, y2), 0)), int(min(max(y0, y1, y2) + 1, size))
            if minx >= maxx or miny >= maxy: continue
            px, py = xs[miny:maxy, minx:maxx], ys[miny:maxy, minx:maxx]
            w0 = ((x1 - px) * (y2 - py) - (x2 - px) * (y1 - py)) / area
            w1 = ((x2 - px) * (y0 - py) - (x0 - px) * (y2 - py)) / area
            w2 = 1 - w0 - w1
            inside = (w0 >= 0) & (w1 >= 0) & (w2 >= 0)
            if not inside.any(): continue
            z = w0 * depth[a] + w1 * depth[b] + w2 * depth[c]
            zb = zbuf[miny:maxy, minx:maxx]
            ok = inside & (z < zb - 1e-5)
            if not ok.any(): continue
            n = w0[..., None] * N[na] + w1[..., None] * N[nb] + w2[..., None] * N[nc]
            n /= np.linalg.norm(n, axis=-1, keepdims=True)
            diff = np.clip(n @ light, 0, 1)[..., None]
            col = np.clip(kd * (0.38 + 0.62 * diff) + ke, 0, 1)
            region = img[miny:maxy, minx:maxx]
            if blend:
                region[ok] = region[ok] * (1 - alpha) + col[ok] * alpha
            else:
                region[ok] = col[ok]; zb[ok] = z[ok]

    for job in jobs:
        if job[0]["mats"][job[2]["mat"]]["d"] >= 1: draw(*job, False)
    for job in jobs:
        if job[0]["mats"][job[2]["mat"]]["d"] < 1: draw(*job, True)
    return Image.fromarray((img * 255).astype(np.uint8))


def sheet(nombre, scene, views, center, face_eye, face_target, face_fov):
    W = 4 * 360
    out = Image.new("RGB", (W, 370 + 240), (18, 16, 26))
    d = ImageDraw.Draw(out)
    for i, (lbl, eye) in enumerate(views):
        out.paste(render(scene, eye, center, 360), (i * 360, 0)); d.text((i * 360 + 8, 6), lbl, fill=(230, 230, 240))
    for i, emo in enumerate(EMOCIONES):
        out.paste(render(scene, face_eye, face_target, 240, emo, fov=face_fov), (i * 240, 370))
        d.text((i * 240 + 8, 376), emo, fill=(230, 230, 240))
    os.makedirs(PREVIEWS, exist_ok=True)
    out.save(os.path.join(PREVIEWS, nombre + ".png"))
    print("previews/" + nombre + ".png")


def piloto_de_la_nave():
    """Lee de nave_lila.obj donde va la piloto (comentario 'piloto: x y z escala s')."""
    for ln in open(os.path.join(MODELOS, "nave_lila.obj")):
        if ln.startswith("# piloto:"):
            t = ln.split(":")[1].split()
            return (float(t[0]), float(t[1]), float(t[2])), float(t[4])


def sheet_saturno():
    sat = load("saturno_escenario")  # el escenario (el planeta lo arma el juego)
    W = 4 * 360
    out = Image.new("RGB", (W, 370 + 360), (18, 16, 26))
    d = ImageDraw.Draw(out)
    vistas = [("frente", (0, 3.0, -11)), ("3/4", (-7.5, 5.0, -7.5)), ("arriba", (0.01, 12, -1.5)), ("lado", (11, 1.2, 0))]
    for i, (lbl, eye) in enumerate(vistas):
        out.paste(render([sat], eye, (0, 0.2, 0), 360), (i * 360, 0)); d.text((i * 360 + 8, 6), lbl, fill=(230, 230, 240))
    cerca = [("escenario", (1.4, 3.4, -2.2), (0, 2.25, 0)), ("puerta y notas", (2.6, 1.8, -5.6), (0.3, 0.4, -3.0)),
             ("bocina y ecualizador", (4.2, 2.4, 3.6), (1.2, 0.9, 1.0)), ("carril", (-1.2, 1.0, -5.2), (0.0, 0.4, -3.2))]
    for i, (lbl, eye, tgt) in enumerate(cerca):
        out.paste(render([sat], eye, tgt, 360, fov=40), (i * 360, 370)); d.text((i * 360 + 8, 376), lbl, fill=(230, 230, 240))
    os.makedirs(PREVIEWS, exist_ok=True)
    out.save(os.path.join(PREVIEWS, "saturno_escenario.png"))
    print("previews/saturno_escenario.png")


def sheet_planetas():
    # Los planetas ya no son .obj: los arma el juego (src/planetas.rs). Se ven con
    # `cargo run --release -- --captura foto.ppm 0 cerca:<planeta>`.
    print("planetas: los arma el juego; ver `--captura foto.ppm 0 cerca:<planeta>`")


if __name__ == "__main__":
    que = sys.argv[1] if len(sys.argv) > 1 else "todo"
    if que in ("planetas", "todo"):
        sheet_planetas()
    if que in ("saturno", "todo"):
        sheet_saturno()
    if que in ("miku", "todo"):
        miku = load("miku_piloto")
        sheet("miku_piloto", [miku],
              [("frente", (0, 0.6, -3.3)), ("3/4", (-2.2, 1.1, -2.4)), ("lado", (3.3, 0.6, 0)), ("atras", (0.9, 1.3, 3.1))],
              (0, 0.42, 0), (0.25, 0.78, -1.7), (0, 0.68, 0), 22)
    if que in ("nave", "todo"):
        pos, esc = piloto_de_la_nave()
        scene = [load("nave_lila"), place(load("miku_piloto"), pos, esc)]
        sheet("nave_lila", scene,
              [("frente", (0, 0.5, -7.5)), ("3/4", (-5.2, 2.6, -5.2)), ("lado", (7.5, 0.8, 0.0)), ("atras", (2.4, 3.2, 6.8))],
              (0, 0.15, 0.1), (0.5, 0.5, -5.6), (0, 0.15, 0), 30)
