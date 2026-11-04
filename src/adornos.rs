//! Accesorios de las skins. Se agregan al final para conservar los indices de animacion.
use crate::matematica::Vec3;
use crate::obj::{Material, Modelo, Parte, Vertice};

struct Taller<'a>(&'a mut Modelo);

impl Taller<'_> {
    fn color(&mut self, nombre: &str, rgb: [f32; 3], brillo: f32) -> usize {
        let kd = Vec3::new(rgb[0], rgb[1], rgb[2]);
        let material = self.0.materiales.len();
        self.0.materiales.push(Material {
            nombre: format!("adorno_{nombre}"), kd, ks: Vec3::new(0.3, 0.3, 0.3),
            ns: 24.0, ke: kd * brillo, d: 1.0, textura: None,
        });
        let i = self.0.partes.len();
        self.0.partes.push(Parte { nombre: format!("adorno_{nombre}"), material, triangulos: Vec::new(), comentario: String::new() });
        i
    }

    /// Un material translucido (vidrio): `d` es la opacidad.
    fn vidrio(&mut self, nombre: &str, rgb: [f32; 3], d: f32) -> usize {
        let i = self.color(nombre, rgb, 0.05);
        let m = self.0.partes[i].material;
        self.0.materiales[m].d = d;
        self.0.materiales[m].ks = Vec3::new(0.9, 0.9, 0.9);
        self.0.materiales[m].ns = 80.0;
        i
    }

    fn caja(&mut self, parte: usize, centro: [f32; 3], tam: [f32; 3]) {
        let c = Vec3::new(centro[0], centro[1], centro[2]);
        let v: Vec<Vec3> = (0..8).map(|i| c + Vec3::new(
            if i & 1 == 0 { -tam[0] } else { tam[0] } * 0.5,
            if i & 2 == 0 { -tam[1] } else { tam[1] } * 0.5,
            if i & 4 == 0 { -tam[2] } else { tam[2] } * 0.5,
        )).collect();
        for [a, b, cc, d] in [[0, 2, 3, 1], [4, 5, 7, 6], [0, 4, 6, 2], [1, 3, 7, 5], [0, 1, 5, 4], [2, 6, 7, 3]] {
            let n = (v[b] - v[a]).cross(v[cc] - v[a]).normalize();
            for ids in [[a, b, cc], [a, cc, d]] {
                self.0.partes[parte].triangulos.push(ids.map(|i| Vertice { pos: v[i], normal: n, uv: [0.0; 2] }));
            }
        }
    }

    /// Un triangulo con la cara hacia afuera de `adentro` (un punto del lado de adentro).
    fn tri(&mut self, parte: usize, a: Vec3, b: Vec3, c: Vec3, adentro: Vec3) {
        let mut n = (b - a).cross(c - a);
        if n.length() < 1e-9 {
            return;
        }
        let (b, c) = if n.dot((a + b + c) * (1.0 / 3.0) - adentro) < 0.0 {
            n = -n;
            (c, b)
        } else {
            (b, c)
        };
        let n = n.normalize();
        self.0.partes[parte].triangulos.push([a, b, c].map(|pos| Vertice { pos, normal: n, uv: [0.0; 2] }));
    }

    /// Un tronco de cono (o cilindro, o cono si `r1` es 0) del centro `c` a lo largo de `eje`.
    fn cono(&mut self, parte: usize, c: Vec3, eje: Vec3, largo: f32, r0: f32, r1: f32, lados: usize) {
        let eje = eje.normalize();
        let u = if eje.y.abs() < 0.9 { eje.cross(Vec3::new(0.0, 1.0, 0.0)).normalize() } else { eje.cross(Vec3::new(1.0, 0.0, 0.0)).normalize() };
        let v = eje.cross(u);
        let fin = c + eje * largo;
        let adentro = c + eje * (largo * 0.5);
        let aro = |centro: Vec3, r: f32, k: usize| {
            let a = k as f32 / lados as f32 * std::f32::consts::TAU;
            centro + u * (a.cos() * r) + v * (a.sin() * r)
        };
        for k in 0..lados {
            let (a0, a1, b0, b1) = (aro(c, r0, k), aro(c, r0, k + 1), aro(fin, r1, k), aro(fin, r1, k + 1));
            self.tri(parte, a0, a1, b1, adentro);
            if r1 > 1e-4 {
                self.tri(parte, a0, b1, b0, adentro);
                self.tri(parte, fin, b1, b0, fin - eje);
            }
            self.tri(parte, c, a0, a1, c + eje);
        }
    }

    /// Un anillo plano alrededor del eje Y (como el del platillo): de radio `r0` a `r1`.
    fn anillo(&mut self, parte: usize, c: Vec3, r0: f32, r1: f32, alto: f32, lados: usize) {
        let p = |r: f32, k: usize, y: f32| {
            let a = k as f32 / lados as f32 * std::f32::consts::TAU;
            Vec3::new(c.x + a.cos() * r, c.y + y, c.z + a.sin() * r)
        };
        let h = alto * 0.5;
        for k in 0..lados {
            let medio = p((r0 + r1) * 0.5, k, 0.0);
            let medio = Vec3::new(medio.x, c.y, medio.z);
            let quad = |t: &mut Self, a: Vec3, b: Vec3, cc: Vec3, d: Vec3| {
                t.tri(parte, a, b, cc, medio);
                t.tri(parte, a, cc, d, medio);
            };
            quad(self, p(r0, k, h), p(r1, k, h), p(r1, k + 1, h), p(r0, k + 1, h));
            quad(self, p(r0, k, -h), p(r1, k, -h), p(r1, k + 1, -h), p(r0, k + 1, -h));
            quad(self, p(r1, k, -h), p(r1, k + 1, -h), p(r1, k + 1, h), p(r1, k, h));
            quad(self, p(r0, k, -h), p(r0, k + 1, -h), p(r0, k + 1, h), p(r0, k, h));
        }
    }

    /// Una esfera de radio `r` (aplastada con `escala`).
    fn esfera(&mut self, parte: usize, c: Vec3, r: f32, escala: Vec3) {
        let (pisos, lados) = (8, 12);
        let p = |i: usize, k: usize| {
            let (t, f) = (i as f32 / pisos as f32 * std::f32::consts::PI, k as f32 / lados as f32 * std::f32::consts::TAU);
            c + Vec3::new(t.sin() * f.cos() * r * escala.x, t.cos() * r * escala.y, t.sin() * f.sin() * r * escala.z)
        };
        for i in 0..pisos {
            for k in 0..lados {
                self.tri(parte, p(i, k), p(i + 1, k), p(i + 1, k + 1), c);
                self.tri(parte, p(i, k), p(i + 1, k + 1), p(i, k + 1), c);
            }
        }
    }

    /// Una aleta: un poligono (convexo, plano) con grosor hacia `grosor`.
    fn aleta(&mut self, parte: usize, puntos: &[Vec3], grosor: Vec3) {
        let n = puntos.len();
        let centro = puntos.iter().fold(Vec3::zero(), |a, &b| a + b) * (1.0 / n as f32) + grosor * 0.5;
        let otro: Vec<Vec3> = puntos.iter().map(|&p| p + grosor).collect();
        for k in 1..n - 1 {
            self.tri(parte, puntos[0], puntos[k], puntos[k + 1], centro);
            self.tri(parte, otro[0], otro[k], otro[k + 1], centro);
        }
        for k in 0..n {
            let j = (k + 1) % n;
            self.tri(parte, puntos[k], puntos[j], otro[j], centro);
            self.tri(parte, puntos[k], otro[j], otro[k], centro);
        }
    }

    fn letras(&mut self, parte: usize, texto: &str, y: f32, z: f32, pixel: f32) {
        let ancho = (texto.len() * 4 - 1) as f32 * pixel;
        for (i, letra) in texto.chars().enumerate() {
            for (fila, bits) in glifo(letra).iter().enumerate() {
                for col in 0..3 {
                    if bits & (1 << (2 - col)) != 0 {
                        // Se lee desde atras (+Z), la vista habitual durante el vuelo.
                        let x = ancho * 0.5 - (i * 4 + col) as f32 * pixel - pixel * 0.5;
                        self.caja(parte, [x, y + (2.0 - fila as f32) * pixel, z], [pixel * 0.92, pixel * 0.92, 0.025]);
                    }
                }
            }
        }
    }

    fn lentes(&mut self, oscuro: usize, blanco: usize) {
        for x in [-0.47, 0.47] {
            self.caja(oscuro, [x, 0.13, -0.96], [0.78, 0.32, 0.18]);
            self.caja(blanco, [x - 0.19, 0.21, -1.06], [0.15, 0.045, 0.02]);
        }
        self.caja(oscuro, [0.0, 0.16, -1.0], [0.22, 0.08, 0.12]);
    }
}

pub fn glifo(c: char) -> [u8; 5] {
    match c {
        'A' => [2, 5, 7, 5, 5], 'C' => [7, 4, 4, 4, 7], 'D' => [6, 5, 5, 5, 6],
        'E' => [7, 4, 6, 4, 7], 'F' => [7, 4, 6, 4, 4], 'G' => [7, 4, 5, 5, 7],
        'H' => [5, 5, 7, 5, 5], 'I' => [7, 2, 2, 2, 7], 'L' => [4, 4, 4, 4, 7],
        'M' => [5, 7, 7, 5, 5], 'N' => [5, 7, 7, 7, 5], 'O' => [7, 5, 5, 5, 7],
        'P' => [6, 5, 6, 4, 4], 'R' => [6, 5, 6, 5, 5], 'S' => [7, 4, 7, 1, 7],
        'T' => [7, 2, 2, 2, 2], 'U' => [5, 5, 5, 5, 7], 'W' => [5, 5, 7, 7, 5],
        'Q' => [7, 5, 5, 7, 1], 'Y' => [5, 5, 2, 2, 2], '6' => [7, 4, 7, 5, 7], '7' => [7, 1, 2, 2, 2],
        '9' => [7, 5, 7, 1, 7], '+' => [0, 2, 7, 2, 0],
        'B' => [6, 5, 6, 5, 6], 'K' => [5, 5, 6, 5, 5], 'V' => [5, 5, 5, 5, 2], 'X' => [5, 5, 2, 5, 5],
        'Z' => [7, 1, 2, 4, 7], 'J' => [1, 1, 1, 5, 7], '0' => [7, 5, 5, 5, 7], '1' => [2, 6, 2, 2, 7],
        '2' => [7, 1, 7, 4, 7], '3' => [7, 1, 3, 1, 7], '4' => [5, 5, 7, 1, 1], '5' => [7, 4, 7, 1, 7], '8' => [7, 5, 7, 5, 7],
        _ => [0; 5],
    }
}

pub fn adornar(modelo: &mut Modelo, id: &str) {
    // Le cambian las partes: la esfera que lo envuelve se vuelve a calcular.
    modelo.esfera_guardada = Default::default();
    let mut t = Taller(modelo);
    let negro = t.color("grafito", [0.025, 0.035, 0.055], 0.0);
    let blanco = t.color("crema", [1.0, 0.95, 0.83], 0.12);
    let oro = t.color("mostaza", [1.0, 0.68, 0.05], 0.12);
    let rosa = t.color("rosa", [1.0, 0.13, 0.44], 0.16);
    let verde = t.color("lima", [0.40, 1.0, 0.12], 0.2);
    let azul = t.color("azul", [0.10, 0.65, 1.0], 0.15);
    let pan = t.color("pan", [0.78, 0.41, 0.14], 0.0);
    let placa = match id {
        "miku" => "MI WIFI", "chapina" => "GT", "camioneta" => "HAY CAMPO", "luka" => "DELULU",
        "shuco" => "CON TODO", "quetzal" => "AURA", "sombra" => "NO FUI", "oro" => "+9999",
        "aurora" => "AURA +67", "cobre" => "CAFECITO", "hielo" => "SIN WIFI", "uva" => "67",
        "platillo" => "OVNI", "cohete" => "NASA", "tiburon" => "MUERDE", "dragon" => "FUEGO", "tuktuk" => "TUK TUK", "patito" => "CUAC",
        "endurance" => "ENDURANCE", "blip" => "ASTROFAGO",
        "barrilete" => "SUMPANGO", "pinata" => "PINATA", "disco" => "DISCO", "tars" => "TARS",
        "tralalero" => "TRALALA", "tungtung" => "TUNG TUNG", "bombardiro" => "BOMBARDIRO", "ballerina" => "CAPPUCCINA", "gamer" => "GG EZ", _ => "NPC",
    };
    t.caja(negro, [0.0, 0.06, 1.03], [1.82, 0.42, 0.10]);
    t.letras(blanco, placa, 0.06, 1.095, 0.046);
    // Lo que distingue a una skin mitica de cualquiera de la tienda: el halo del pase.
    if crate::tienda::es_del_pase(id) {
        let halo = t.color("halo_pase", [1.0, 0.82, 0.28], 1.3);
        let alto = if matches!(id, "chapina" | "oro") { 2.45 } else { 2.0 };
        t.anillo(halo, Vec3::new(0.0, alto, -0.1), 0.62, 0.8, 0.07, 18);
    }
    match id {
        "miku" => {
            t.caja(blanco, [-0.85, 1.14, -0.20], [0.19, 0.72, 0.19]);
            for (x, h) in [(-1.0, 0.45), (-0.8, 0.60), (-0.65, 0.4)] {
                t.caja(verde, [x, 1.5 + h * 0.5, -0.20], [0.12, h, 0.15]);
            }
        }
        "chapina" => {
            // Alas de bandera (azul, blanco, azul) y un volcan de sombrero, con su lava.
            let lava = t.color("lava", [1.0, 0.42, 0.08], 1.2);
            let cerro = t.color("cerro", [0.20, 0.52, 0.34], 0.0);
            for lado in [-1.0f32, 1.0] {
                for (k, m) in [azul, blanco, azul].iter().enumerate() {
                    let (x0, x1) = (lado * (1.0 + k as f32 * 0.46), lado * (1.46 + k as f32 * 0.46));
                    let (y0, y1) = (0.72 + k as f32 * 0.13, 0.85 + k as f32 * 0.13);
                    t.aleta(*m, &[Vec3::new(x0, y0, 0.1), Vec3::new(x1, y1, 0.1), Vec3::new(x1, y1 - 0.95 + k as f32 * 0.14, 0.45), Vec3::new(x0, y0 - 1.05 + k as f32 * 0.14, 0.45)], Vec3::new(0.0, 0.0, 0.05));
                }
            }
            t.cono(cerro, Vec3::new(0.15, 0.9, -0.2), Vec3::new(0.0, 1.0, 0.0), 0.72, 0.62, 0.17, 10);
            t.cono(lava, Vec3::new(0.15, 1.6, -0.2), Vec3::new(0.0, 1.0, 0.0), 0.09, 0.17, 0.12, 8);
            for (x, y, tam) in [(0.2, 1.84, 0.13), (0.08, 2.02, 0.1), (0.26, 2.16, 0.07)] {
                t.caja(lava, [x, y, -0.2], [tam, tam, tam]);
            }
        }
        "camioneta" => {
            // La carroceria entera: cofre, ventanas, llantas, parrilla cargada y escalera.
            let vidrio = t.color("vidrio_bus", [0.08, 0.12, 0.22], 0.05);
            let llanta = t.color("llanta", [0.04, 0.04, 0.05], 0.0);
            let cromo = t.color("cromo", [0.85, 0.85, 0.92], 0.08);
            let costal = t.color("costal", [0.75, 0.60, 0.36], 0.0);
            t.caja(oro, [0.0, 1.06, -0.05], [2.32, 0.2, 2.5]);
            t.caja(rosa, [0.0, 1.18, -0.05], [2.36, 0.05, 2.54]);
            t.letras(negro, "HAY CAMPO", 1.04, 1.21, 0.045);
            for lado in [-1.0f32, 1.0] {
                t.caja(azul, [lado * 1.14, 0.3, -0.05], [0.14, 1.3, 2.4]);
                t.caja(oro, [lado * 1.21, -0.1, -0.05], [0.03, 0.14, 2.4]);
                for k in 0..4 {
                    t.caja(vidrio, [lado * 1.22, 0.55, -0.85 + k as f32 * 0.55], [0.03, 0.42, 0.4]);
                }
                for z in [-0.72, 0.68] {
                    t.cono(llanta, Vec3::new(lado * 1.1, -0.48, z), Vec3::new(lado, 0.0, 0.0), 0.3, 0.33, 0.33, 12);
                    t.cono(cromo, Vec3::new(lado * 1.4, -0.48, z), Vec3::new(lado, 0.0, 0.0), 0.02, 0.14, 0.14, 8);
                }
                t.caja(cromo, [lado * 1.02, 1.32, -0.05], [0.05, 0.24, 2.4]);
            }
            t.caja(cromo, [0.0, -0.58, -1.32], [2.1, 0.12, 0.14]);
            t.caja(oro, [0.0, -0.28, -1.42], [1.6, 0.52, 0.45]);
            for x in [-0.56, 0.56] { t.caja(blanco, [x, -0.2, -1.66], [0.28, 0.2, 0.04]); }
            t.caja(costal, [-0.45, 1.34, -0.55], [0.72, 0.34, 0.86]);
            t.caja(rosa, [0.45, 1.32, -0.3], [0.7, 0.3, 0.7]);
            t.caja(verde, [-0.32, 1.34, 0.55], [0.72, 0.34, 0.8]);
            for k in 0..6 { t.caja(cromo, [0.76, 0.0 + k as f32 * 0.22, 1.3], [0.34, 0.03, 0.04]); }
        }
        "luka" => {
            for (fila, bits) in [0b01010, 0b11111, 0b11111, 0b01110, 0b00100].iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << col) != 0 { t.caja(rosa, [-0.84 + col as f32 * 0.13, 1.60 - fila as f32 * 0.13, -0.5], [0.13, 0.13, 0.16]); }
                }
            }
        }
        "shuco" => {
            // La nave va adentro: dos panes a los lados, salchicha encima, mostaza y guacamol.
            let salchicha = t.color("salchicha", [0.80, 0.22, 0.20], 0.03);
            let tostado = t.color("tostado", [0.56, 0.26, 0.08], 0.0);
            for lado in [-1.0f32, 1.0] {
                t.esfera(pan, Vec3::new(lado * 1.28, 0.25, 0.0), 1.0, Vec3::new(0.48, 0.85, 1.6));
                for z in [-0.8, 0.0, 0.8] { t.caja(tostado, [lado * 1.72, 0.45, z], [0.05, 0.16, 0.42]); }
            }
            t.cono(salchicha, Vec3::new(0.0, 1.22, -1.9), Vec3::new(0.0, 0.0, 1.0), 3.8, 0.34, 0.34, 10);
            for k in 0..15 {
                let z = -1.72 + k as f32 * 0.24;
                t.caja(oro, [if k % 2 == 0 { 0.12 } else { -0.12 }, 1.57, z], [0.38, 0.05, 0.11]);
            }
            for (x, z) in [(-0.3, -1.3), (0.3, -0.5), (-0.28, 0.4), (0.3, 1.3)] { t.caja(verde, [x, 1.5, z], [0.24, 0.13, 0.26]); }
        }
        "quetzal" => {
            // Pico, cresta, pecho rojo, alas y la cola de plumas que tiene fama.
            let esmeralda = t.color("esmeralda", [0.05, 0.84, 0.46], 0.28);
            let esmeralda_b = t.color("esmeralda_b", [0.04, 0.58, 0.60], 0.2);
            let pecho = t.color("pecho", [0.95, 0.08, 0.16], 0.1);
            t.caja(pecho, [0.0, -0.42, -0.88], [1.36, 0.62, 0.3]);
            t.aleta(oro, &[Vec3::new(-0.2, 0.05, -1.02), Vec3::new(0.2, 0.05, -1.02), Vec3::new(0.0, -0.15, -1.48)], Vec3::new(0.0, -0.06, 0.0));
            for k in 0..7 {
                let x = -0.42 + k as f32 * 0.14;
                let h = 0.52 - (k as f32 - 3.0).abs() * 0.1;
                t.caja(esmeralda, [x, 1.04 + h * 0.5, -0.4], [0.11, h, 0.34]);
            }
            for lado in [-1.0f32, 1.0] {
                t.aleta(esmeralda_b, &[Vec3::new(lado * 1.0, 0.65, -0.3), Vec3::new(lado * 2.25, 0.36, 0.15), Vec3::new(lado * 1.85, -0.25, 0.42), Vec3::new(lado * 1.0, -0.2, 0.25)], Vec3::new(0.0, 0.05, 0.0));
                t.aleta(esmeralda, &[Vec3::new(lado * 1.0, 0.42, -0.05), Vec3::new(lado * 1.7, 0.22, 0.2), Vec3::new(lado * 1.42, -0.2, 0.42), Vec3::new(lado * 1.0, -0.25, 0.3)], Vec3::new(0.0, 0.08, 0.0));
            }
            // Las dos plumas largas, mas largas que toda la nave, y dos cortas.
            for (x, largo, m) in [(-0.3, 4.1, esmeralda), (0.3, 3.4, esmeralda), (-0.14, 2.2, esmeralda_b), (0.14, 2.6, esmeralda_b)] {
                t.aleta(m, &[Vec3::new(x - 0.13, -0.28, 0.95), Vec3::new(x + 0.13, -0.28, 0.95), Vec3::new(x + 0.2, -0.65, 0.95 + largo * 0.7), Vec3::new(x, -0.72, 0.95 + largo), Vec3::new(x - 0.2, -0.65, 0.95 + largo * 0.7)], Vec3::new(0.0, 0.04, 0.0));
            }
        }
        "sombra" => {
            // Capucha, ojos de neon, katana a la espalda, cortes rojos y cuchillas alrededor.
            let neon = t.color("neon_rojo", [1.0, 0.06, 0.16], 1.5);
            let capucha = t.color("capucha", [0.03, 0.03, 0.07], 0.0);
            let acero = t.color("acero", [0.8, 0.85, 0.95], 0.2);
            t.caja(capucha, [0.0, 1.02, -0.05], [2.28, 0.3, 2.2]);
            t.caja(capucha, [0.0, 1.32, 0.1], [1.5, 0.3, 1.6]);
            t.caja(capucha, [0.0, 1.58, 0.28], [0.72, 0.26, 0.92]);
            for lado in [-1.0f32, 1.0] {
                t.caja(capucha, [lado * 1.1, 0.5, 0.1], [0.2, 1.05, 1.9]);
                t.caja(neon, [lado * 1.21, 0.5, 0.1], [0.02, 0.04, 1.7]);
                t.caja(neon, [lado * 0.47, 0.13, -1.1], [0.62, 0.07, 0.02]);
            }
            t.caja(neon, [0.0, 1.18, 1.05], [1.8, 0.04, 0.03]);
            // La katana cruzada.
            t.aleta(acero, &[Vec3::new(-1.42, 1.72, 1.15), Vec3::new(-1.3, 1.82, 1.15), Vec3::new(1.05, -0.22, 1.15), Vec3::new(0.93, -0.32, 1.15)], Vec3::new(0.0, 0.0, 0.05));
            t.caja(neon, [1.1, -0.35, 1.17], [0.42, 0.42, 0.06]);
            t.caja(capucha, [1.28, -0.54, 1.17], [0.34, 0.34, 0.07]);
            // Las cuchillas que la siguen, en circulo.
            for k in 0..8 {
                let a = k as f32 / 8.0 * std::f32::consts::TAU;
                let (cx, cz) = (a.cos() * 2.0, a.sin() * 2.0);
                let (ux, uz) = (a.cos() * 0.34, a.sin() * 0.34);
                t.aleta(if k % 2 == 0 { neon } else { acero }, &[Vec3::new(cx - uz, 0.1, cz + ux), Vec3::new(cx + ux, 0.1, cz + uz), Vec3::new(cx + uz, 0.1, cz - ux), Vec3::new(cx - ux, 0.1, cz - uz)], Vec3::new(0.0, 0.05, 0.0));
            }
        }
        "oro" => {
            // Corona grande con joyas, alas de oro y su capa de tarjeta sin saldo.
            let dorado = t.color("dorado", [1.0, 0.78, 0.18], 0.45);
            let capa = t.color("capa", [0.72, 0.05, 0.22], 0.05);
            t.caja(dorado, [0.0, 0.98, -0.1], [1.72, 0.2, 1.6]);
            for (x, z) in [(-0.74, -0.8), (0.0, -0.8), (0.74, -0.8), (-0.74, 0.6), (0.0, 0.6), (0.74, 0.6), (-0.74, -0.1), (0.74, -0.1)] {
                let h = if x == 0.0 { 0.72 } else { 0.48 };
                t.caja(dorado, [x, 1.08 + h * 0.5, z], [0.26, h, 0.26]);
                t.caja(if x == 0.0 { rosa } else { azul }, [x, 1.08 + h + 0.08, z], [0.16, 0.16, 0.16]);
            }
            for lado in [-1.0f32, 1.0] {
                for k in 0..5 {
                    let e = k as f32 * 0.42;
                    let (y0, y1) = (0.62 + e * 0.32, 0.84 + e * 0.42);
                    t.aleta(dorado, &[Vec3::new(lado * (1.0 + e), y0, 0.1), Vec3::new(lado * (1.42 + e), y1, 0.1), Vec3::new(lado * (1.42 + e), y1 - 1.0 + e * 0.16, 0.5), Vec3::new(lado * (1.0 + e), y0 - 1.0 + e * 0.16, 0.5)], Vec3::new(0.0, 0.0, 0.05));
                }
            }
            t.aleta(capa, &[Vec3::new(-0.92, 0.75, 1.02), Vec3::new(0.92, 0.75, 1.02), Vec3::new(1.15, -1.15, 1.42), Vec3::new(-1.15, -1.15, 1.42)], Vec3::new(0.0, 0.0, 0.05));
            t.caja(dorado, [0.0, -0.1, 1.28], [0.5, 0.5, 0.03]);
        }
        "aurora" => {
            // Lentes, y arriba las cintas de la aurora, ondulando.
            t.lentes(negro, blanco);
            let cinta = [t.color("aurora_a", [0.25, 1.0, 0.70], 0.4), t.color("aurora_b", [0.60, 0.42, 1.0], 0.4)];
            for k in 0..14 {
                let x = -0.84 + k as f32 * 0.13;
                let y = 1.2 + (k as f32 * 0.9).sin() * 0.14;
                t.caja(cinta[k % 2], [x, y, -0.2], [0.13, 0.42, 0.05]);
            }
        }
        "cobre" => {
            // Taza abierta: cuatro paredes, cafe y un asa rectangular con hueco.
            t.caja(blanco, [-0.78, 1.0, -0.35], [0.55, 0.12, 0.52]);
            for x in [-1.02, -0.54] { t.caja(blanco, [x, 1.3, -0.35], [0.07, 0.55, 0.52]); }
            for z in [-0.59, -0.11] { t.caja(blanco, [-0.78, 1.3, z], [0.55, 0.55, 0.07]); }
            t.caja(pan, [-0.78, 1.49, -0.35], [0.42, 0.02, 0.40]);
            for y in [1.10, 1.46] { t.caja(blanco, [-1.18, y, -0.35], [0.34, 0.07, 0.11]); }
            t.caja(blanco, [-1.32, 1.28, -0.35], [0.07, 0.40, 0.11]);
        }
        "hielo" => {
            t.caja(azul, [-0.65, 1.15, -0.2], [0.06, 0.5, 0.06]);
            t.caja(blanco, [-0.65, 1.45, -0.3], [0.18, 0.15, 0.95]);
            t.caja(blanco, [-0.65, 1.45, -0.25], [1.05, 0.08, 0.24]);
            t.caja(azul, [-0.65, 1.59, 0.08], [0.08, 0.3, 0.20]);
        }
        // Las carrocerias: cambian la silueta de la nave entera.
        "platillo" => {
            // Un platillo volador: el anillo grandote alrededor, con su borde y sus luces.
            let plata = t.color("plata", [0.80, 0.82, 0.88], 0.04);
            t.anillo(plata, Vec3::new(0.0, -0.15, 0.0), 1.05, 2.55, 0.16, 32);
            t.anillo(negro, Vec3::new(0.0, -0.06, 0.0), 1.05, 1.9, 0.05, 32);
            for k in 0..14 {
                let a = k as f32 / 14.0 * std::f32::consts::TAU;
                let luz = [verde, rosa, azul][k % 3];
                t.caja(luz, [a.cos() * 2.45, -0.15, a.sin() * 2.45], [0.16, 0.12, 0.16]);
            }
            t.cono(plata, Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, -1.0, 0.0), 0.35, 0.7, 0.25, 16);
            t.caja(verde, [0.0, -1.4, 0.0], [0.4, 0.05, 0.4]);
        }
        "cohete" => {
            // Un cohete de los de antes: la punta adelante (a franjas) y tres aletas atras.
            let blanco_brillo = t.color("cohete_blanco", [0.97, 0.97, 0.97], 0.05);
            let rojo = t.color("cohete_rojo", [0.95, 0.12, 0.12], 0.08);
            t.cono(blanco_brillo, Vec3::new(0.0, 0.0, -0.95), Vec3::new(0.0, 0.0, -1.0), 0.55, 0.95, 0.7, 20);
            t.cono(rojo, Vec3::new(0.0, 0.0, -1.5), Vec3::new(0.0, 0.0, -1.0), 0.35, 0.7, 0.5, 20);
            t.cono(blanco_brillo, Vec3::new(0.0, 0.0, -1.85), Vec3::new(0.0, 0.0, -1.0), 0.75, 0.5, 0.0, 20);
            t.aleta(rojo, &[Vec3::new(0.0, 0.9, 0.4), Vec3::new(0.0, 2.1, 1.7), Vec3::new(0.0, 0.9, 1.4)], Vec3::new(0.08, 0.0, 0.0));
            for lado in [-1.0, 1.0] {
                t.aleta(rojo, &[Vec3::new(lado * 1.0, -0.6, 0.4), Vec3::new(lado * 2.1, -1.5, 1.7), Vec3::new(lado * 1.0, -0.6, 1.4)], Vec3::new(0.0, 0.08, 0.0));
            }
        }
        "tiburon" => {
            // Un tiburon: la aleta de arriba, la cola, las aletas de lado y los dientes.
            let gris = t.color("tiburon", [0.42, 0.52, 0.62], 0.0);
            t.aleta(gris, &[Vec3::new(-0.05, 0.95, -0.35), Vec3::new(-0.05, 2.45, 0.75), Vec3::new(-0.05, 0.95, 0.85)], Vec3::new(0.1, 0.0, 0.0));
            t.aleta(gris, &[Vec3::new(-0.05, 0.0, 1.05), Vec3::new(-0.05, 1.4, 2.25), Vec3::new(-0.05, 0.1, 1.65)], Vec3::new(0.1, 0.0, 0.0));
            t.aleta(gris, &[Vec3::new(-0.05, -0.1, 1.05), Vec3::new(-0.05, -1.2, 2.0), Vec3::new(-0.05, -0.2, 1.6)], Vec3::new(0.1, 0.0, 0.0));
            for lado in [-1.0, 1.0] {
                t.aleta(gris, &[Vec3::new(lado * 1.0, -0.55, -0.4), Vec3::new(lado * 2.2, -1.25, 0.35), Vec3::new(lado * 1.0, -0.55, 0.3)], Vec3::new(0.0, 0.07, 0.0));
                for k in 0..3 {
                    t.caja(negro, [lado * 1.02, 0.1 - k as f32 * 0.18, 0.15], [0.04, 0.05, 0.32]);
                }
            }
            for k in 0..9 {
                let x = -0.64 + k as f32 * 0.16;
                t.cono(blanco, Vec3::new(x, -0.42, -1.04), Vec3::new(0.0, if k % 2 == 0 { -1.0 } else { 1.0 }, 0.0), 0.16, 0.07, 0.0, 6);
            }
        }
        "dragon" => {
            // Un dragon: alas de murcielago con sus varillas, cuernos y la cola con su punta.
            let ala = t.color("ala_dragon", [0.75, 0.15, 0.20], 0.05);
            let hueso = t.color("hueso", [0.95, 0.90, 0.75], 0.0);
            for lado in [-1.0, 1.0] {
                let raiz = Vec3::new(lado * 1.05, 0.55, 0.1);
                let puntas = [Vec3::new(lado * 2.6, 1.9, -0.2), Vec3::new(lado * 3.4, 0.9, 0.3), Vec3::new(lado * 3.0, -0.1, 0.9), Vec3::new(lado * 1.9, 0.0, 0.9)];
                for w in puntas.windows(2) {
                    t.aleta(ala, &[raiz, w[0], w[1]], Vec3::new(0.0, 0.05, 0.0));
                }
                for p in puntas.iter().take(3) {
                    let d = *p - raiz;
                    t.cono(hueso, raiz, d, d.length(), 0.06, 0.03, 6);
                }
                t.cono(hueso, Vec3::new(lado * 0.55, 0.95, -0.45), Vec3::new(lado * 0.3, 1.0, 0.5), 0.7, 0.14, 0.0, 8);
            }
            let mut c = Vec3::new(0.0, -0.2, 1.0);
            for k in 0..6 {
                let r = 0.32 - k as f32 * 0.045;
                let siguiente = c + Vec3::new(0.0, -0.08 * k as f32, 0.32);
                t.cono(ala, c, siguiente - c, (siguiente - c).length(), r, r - 0.04, 10);
                c = siguiente;
            }
            t.aleta(hueso, &[c + Vec3::new(0.0, 0.2, 0.0), c + Vec3::new(0.0, -0.1, 0.6), c + Vec3::new(0.0, -0.35, 0.0)], Vec3::new(0.06, 0.0, 0.0));
        }
        "tuktuk" => {
            // Un tuk-tuk: el techo con sus postes, el parabrisas, tres llantas y calcomanias.
            let techo = t.color("techo", [0.95, 0.20, 0.25], 0.05);
            let llanta = t.color("llanta", [0.06, 0.06, 0.07], 0.0);
            t.caja(techo, [0.0, 2.25, 0.15], [2.3, 0.14, 2.6]);
            t.caja(oro, [0.0, 2.36, 0.15], [2.0, 0.06, 2.3]);
            for (x, z) in [(-1.05, -1.05), (1.05, -1.05), (-1.05, 1.3), (1.05, 1.3)] {
                t.caja(negro, [x, 1.2, z], [0.08, 2.1, 0.08]);
            }
            t.caja(azul, [0.0, 1.7, -1.08], [1.9, 0.06, 0.06]);
            for (x, z) in [(0.0, -1.0), (-0.95, 0.9), (0.95, 0.9)] {
                t.cono(llanta, Vec3::new(x - 0.12, -1.25, z), Vec3::new(1.0, 0.0, 0.0), 0.24, 0.32, 0.32, 14);
                t.caja(oro, [x, -1.25, z], [0.27, 0.12, 0.12]);
            }
            for (k, m) in [rosa, verde, azul, oro].iter().enumerate() {
                t.caja(*m, [-0.9 + k as f32 * 0.6, 2.18, -1.17], [0.4, 0.06, 0.03]);
            }
        }
        "patito" => {
            // Un patito de hule gigante: la cabeza arriba con su pico, los ojos y la colita.
            let amarillo = t.color("patito", [1.0, 0.86, 0.12], 0.08);
            let pico = t.color("pico", [1.0, 0.45, 0.08], 0.05);
            t.esfera(amarillo, Vec3::new(0.0, 1.75, -0.55), 0.62, Vec3::new(1.0, 1.0, 1.0));
            t.cono(pico, Vec3::new(0.0, 1.62, -1.08), Vec3::new(0.0, -0.1, -1.0), 0.42, 0.26, 0.08, 10);
            for x in [-0.25, 0.25] {
                t.esfera(negro, Vec3::new(x, 1.88, -1.08), 0.09, Vec3::new(1.0, 1.0, 0.5));
            }
            t.cono(amarillo, Vec3::new(0.0, 0.6, 1.05), Vec3::new(0.0, 1.0, 0.7), 0.9, 0.4, 0.05, 10);
            for lado in [-1.0, 1.0] {
                t.esfera(amarillo, Vec3::new(lado * 1.2, 0.2, 0.25), 0.45, Vec3::new(0.35, 0.8, 1.2));
            }
        }
        "endurance" => {
            // El anillo de la Endurance alrededor de la nave (en el plano de la cabina, girando
            // sobre el eje de vuelo: las partes `_gira` las mueve la animacion), con sus rayos.
            let blanco_e = t.color("anillo_gira", [0.92, 0.92, 0.90], 0.03);
            let gris_e = t.color("anillo_gira_gris", [0.46, 0.47, 0.52], 0.0);
            let panel = t.color("anillo_gira_panel", [0.12, 0.17, 0.42], 0.08);
            let luz = t.color("anillo_gira_luz", [1.0, 0.88, 0.6], 0.9);
            let (radio, z) = (2.35, 0.25);
            for n in 0..12 {
                let a = n as f32 / 12.0 * std::f32::consts::TAU;
                let (c, s) = (a.cos(), a.sin());
                let tangente = Vec3::new(-s, c, 0.0);
                let centro = Vec3::new(c * radio, s * radio, z);
                t.cono(if n % 2 == 0 { blanco_e } else { gris_e }, centro - tangente * 0.46, tangente, 0.92, 0.25, 0.25, 4);
                if n % 3 == 0 {
                    // Paneles solares hacia afuera.
                    let afuera = Vec3::new(c, s, 0.0);
                    let base = centro + afuera * 0.25;
                    t.aleta(panel, &[base - tangente * 0.3, base - tangente * 0.3 + afuera * 0.55, base + tangente * 0.3 + afuera * 0.55, base + tangente * 0.3], Vec3::new(0.0, 0.0, 0.03));
                } else {
                    let ventana = centro + Vec3::new(c, s, 0.0) * 0.25;
                    t.caja(luz, [ventana.x, ventana.y, ventana.z], [0.1, 0.1, 0.1]);
                }
            }
            // Los cuatro rayos, del centro al anillo.
            for n in 0..4 {
                let a = n as f32 / 4.0 * std::f32::consts::TAU + std::f32::consts::FRAC_PI_4;
                let d = Vec3::new(a.cos(), a.sin(), 0.0);
                t.cono(gris_e, Vec3::new(0.0, 0.0, z) + d * 1.0, d, radio - 1.15, 0.06, 0.06, 6);
            }
        }
        "blip" => {
            // La Blip-A: la nariz de xenonita, el motor de astrofago (rojo infrarrojo) atras y la
            // burbuja translucida al costado, con Rocky adentro.
            let xenonita = t.color("xenonita", [0.48, 0.52, 0.47], 0.0);
            let astrofago = t.color("astrofago", [0.9, 0.08, 0.12], 1.1);
            let roca = t.color("rocky", [0.50, 0.44, 0.40], 0.0);
            t.cono(xenonita, Vec3::new(0.0, 0.0, -0.95), Vec3::new(0.0, 0.0, -1.0), 0.75, 0.45, 0.0, 8);
            t.cono(astrofago, Vec3::new(0.0, -0.05, 1.0), Vec3::new(0.0, 0.0, 1.0), 0.18, 0.78, 0.66, 18);
            t.cono(xenonita, Vec3::new(0.0, -0.05, 1.0), Vec3::new(0.0, 0.0, 1.0), 0.12, 0.92, 0.82, 18);
            // El tunel y la burbuja.
            let c = Vec3::new(2.15, 0.05, 0.15);
            t.cono(xenonita, Vec3::new(0.95, 0.05, 0.15), Vec3::new(1.0, 0.0, 0.0), 0.5, 0.2, 0.2, 8);
            // Rocky: el caparazon (aplastado) y sus cinco patas hasta el piso de la burbuja.
            t.esfera(roca, c + Vec3::new(0.0, -0.05, 0.0), 0.3, Vec3::new(1.0, 0.6, 1.0));
            for n in 0..5 {
                let a = n as f32 / 5.0 * std::f32::consts::TAU + 0.3;
                let afuera = Vec3::new(a.cos(), 0.0, a.sin());
                let rodilla = c + afuera * 0.42 + Vec3::new(0.0, 0.12, 0.0);
                t.cono(roca, c + afuera * 0.2, rodilla - (c + afuera * 0.2), (rodilla - (c + afuera * 0.2)).length(), 0.05, 0.04, 5);
                let pie = c + afuera * 0.55 + Vec3::new(0.0, -0.45, 0.0);
                t.cono(roca, rodilla, pie - rodilla, (pie - rodilla).length(), 0.04, 0.03, 5);
            }
            let burbuja = t.vidrio("burbuja", [0.55, 0.85, 0.75], 0.28);
            t.esfera(burbuja, c, 0.72, Vec3::new(1.0, 1.0, 1.0));
        }
        "barrilete" => {
            // El barrilete: un disco de papel de china atras de la nave (anillos de cunas de colores
            // y una estrella al centro), con los flecos de la orilla y la cola.
            let colores = [
                t.color("papel_rojo", [0.92, 0.15, 0.25], 0.08),
                t.color("papel_amarillo", [1.0, 0.82, 0.15], 0.08),
                t.color("papel_azul", [0.15, 0.50, 0.95], 0.08),
                t.color("papel_verde", [0.20, 0.80, 0.40], 0.08),
                t.color("papel_morado", [0.65, 0.25, 0.85], 0.08),
            ];
            let z = 1.25;
            let anillos = [0.35, 0.95, 1.55, 2.15, 2.6];
            for r in 0..anillos.len() - 1 {
                let lados = 8 + r * 4;
                for n in 0..lados {
                    let (a0, a1) = (n as f32 / lados as f32 * std::f32::consts::TAU, (n + 1) as f32 / lados as f32 * std::f32::consts::TAU);
                    let p = |a: f32, rr: f32| Vec3::new(a.cos() * rr, a.sin() * rr, z);
                    let c = colores[(n + r * 2) % colores.len()];
                    t.aleta(c, &[p(a0, anillos[r]), p(a1, anillos[r]), p(a1, anillos[r + 1]), p(a0, anillos[r + 1])], Vec3::new(0.0, 0.0, 0.04));
                }
            }
            t.esfera(colores[1], Vec3::new(0.0, 0.0, z), 0.36, Vec3::new(1.0, 1.0, 0.3));
            // Los flecos: tiritas que cuelgan de la orilla.
            for n in 0..36 {
                let a = n as f32 / 36.0 * std::f32::consts::TAU;
                let p = Vec3::new(a.cos() * 2.72, a.sin() * 2.72, z);
                t.caja(colores[n % colores.len()], [p.x, p.y, p.z], [0.12, 0.22, 0.03]);
            }
            // La cola, ondulando hacia abajo y atras.
            for k in 0..9 {
                let y = -2.6 - k as f32 * 0.32;
                t.caja(colores[k % colores.len()], [0.25 * (k as f32 * 0.9).sin(), y, z + 0.2 + k as f32 * 0.12], [0.3, 0.12, 0.05]);
            }
        }
        "pinata" => {
            // Siete picos de papel que salen de la nave, a franjas, con flecos en la punta.
            let colores = [t.color("pico_rosa", [1.0, 0.35, 0.70], 0.1), t.color("pico_amarillo", [1.0, 0.85, 0.15], 0.1), t.color("pico_verde", [0.25, 0.90, 0.55], 0.1), t.color("pico_azul", [0.25, 0.65, 1.0], 0.1)];
            let direcciones = [Vec3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 0.3, 0.2), Vec3::new(-1.0, 0.3, 0.2), Vec3::new(0.7, -0.6, 0.3), Vec3::new(-0.7, -0.6, 0.3), Vec3::new(0.4, 0.5, 0.8), Vec3::new(-0.4, 0.5, 0.8)];
            for (k, d) in direcciones.iter().enumerate() {
                let d = d.normalize();
                let base = d * 0.85;
                // Tres tramos de cono de colores (las franjas) y el fleco.
                let tramos = [(0.0, 0.42, 0.30), (0.42, 0.30, 0.17), (0.84, 0.17, 0.0)];
                for (q, &(desde, r0, r1)) in tramos.iter().enumerate() {
                    t.cono(colores[(k + q) % colores.len()], base + d * (desde * 1.0), d, 0.42, r0, r1, 10);
                }
                let punta = base + d * 1.3;
                for f in 0..3 {
                    t.caja(colores[(k + f + 1) % colores.len()], [punta.x + (f as f32 - 1.0) * 0.08, punta.y - 0.15, punta.z], [0.05, 0.3, 0.05]);
                }
            }
        }
        "disco" => {
            // La bola disco (gira: `_giray`) con sus espejitos, las luces de colores alrededor y
            // dos bocinas.
            let espejo = t.color("bola_giray", [0.85, 0.88, 0.95], 0.15);
            let brillos = t.color("brillos_giray", [1.0, 1.0, 1.0], 1.0);
            let c = Vec3::new(0.0, 1.75, -0.1);
            t.cono(negro, Vec3::new(0.0, 1.0, -0.1), Vec3::new(0.0, 1.0, 0.0), 0.3, 0.03, 0.03, 6);
            t.esfera(espejo, c, 0.48, Vec3::new(1.0, 1.0, 1.0));
            for n in 0..14 {
                let a = n as f32 * 2.399;
                let y = 0.4 * ((n as f32 / 14.0) * 2.0 - 1.0);
                let r = (0.48f32 * 0.48 - y * y).max(0.0).sqrt();
                let p = c + Vec3::new(a.cos() * r, y, a.sin() * r);
                t.caja(brillos, [p.x, p.y, p.z], [0.07, 0.07, 0.07]);
            }
            for n in 0..12 {
                let a = n as f32 / 12.0 * std::f32::consts::TAU;
                t.caja([rosa, verde, azul, oro][n % 4], [a.cos() * 1.15, 0.95, a.sin() * 1.15], [0.14, 0.1, 0.14]);
            }
            for lado in [-1.0, 1.0] {
                t.caja(negro, [lado * 1.25, -0.2, 0.3], [0.42, 0.7, 0.5]);
                t.cono(blanco, Vec3::new(lado * 1.47, -0.05, 0.3), Vec3::new(lado, 0.0, 0.0), 0.04, 0.16, 0.16, 10);
                t.cono(blanco, Vec3::new(lado * 1.47, -0.4, 0.3), Vec3::new(lado, 0.0, 0.0), 0.04, 0.1, 0.1, 10);
            }
        }
        "tars" => {
            // Los paneles de TARS: cuatro losas a los lados (dos por lado), con la pantallita.
            let losa = t.color("losa", [0.30, 0.31, 0.34], 0.0);
            let pantalla = t.color("pantalla_tars", [0.75, 0.9, 1.0], 0.6);
            for lado in [-1.0, 1.0] {
                for (k, x) in [1.15, 1.55].iter().enumerate() {
                    let y = if k == 0 { 0.1 } else { -0.15 };
                    t.caja(losa, [lado * x, y, 0.15], [0.32, 2.3, 0.75]);
                    t.caja(negro, [lado * x, y + 0.75, -0.24], [0.26, 0.04, 0.02]);
                }
                t.caja(pantalla, [lado * 1.15, 0.75, -0.24], [0.22, 0.26, 0.03]);
                t.caja(negro, [lado * 1.15, 0.75, -0.25], [0.04, 0.2, 0.03]);
            }
        }
        "gamer" => {
            // Audifonos con su diadema y el microfono, y tiras RGB (los siete colores) alrededor.
            let negro_mate = t.color("negro_mate", [0.06, 0.06, 0.07], 0.0);
            let rgb = [[1.0, 0.1, 0.2], [1.0, 0.55, 0.0], [1.0, 0.95, 0.1], [0.1, 1.0, 0.3], [0.1, 0.8, 1.0], [0.3, 0.3, 1.0], [0.85, 0.2, 1.0]].map(|c| c);
            let tiras: Vec<usize> = rgb.iter().enumerate().map(|(k, c)| t.color(&format!("rgb_{k}"), *c, 1.0)).collect();
            for lado in [-1.0f32, 1.0] {
                t.cono(negro_mate, Vec3::new(lado * 1.02, 0.25, -0.1), Vec3::new(lado, 0.0, 0.0), 0.28, 0.5, 0.5, 14);
                t.cono(tiras[if lado < 0.0 { 2 } else { 5 }], Vec3::new(lado * 1.3, 0.25, -0.1), Vec3::new(lado, 0.0, 0.0), 0.03, 0.42, 0.42, 14);
            }
            // La diadema: un arco por arriba.
            for k in 0..10 {
                let (a0, a1) = (k as f32 / 10.0 * std::f32::consts::PI, (k + 1) as f32 / 10.0 * std::f32::consts::PI);
                let p0 = Vec3::new(a0.cos() * 1.1, 0.5 + a0.sin() * 0.75, -0.1);
                let p1 = Vec3::new(a1.cos() * 1.1, 0.5 + a1.sin() * 0.75, -0.1);
                t.cono(negro_mate, p0, p1 - p0, (p1 - p0).length(), 0.07, 0.07, 6);
            }
            t.cono(negro_mate, Vec3::new(-1.2, 0.05, -0.15), Vec3::new(0.35, -0.35, -1.0), 0.75, 0.04, 0.04, 6);
            t.esfera(tiras[3], Vec3::new(-0.9, -0.2, -0.85), 0.09, Vec3::new(1.0, 1.0, 1.0));
            // La tira RGB alrededor de la cintura.
            for k in 0..21 {
                let a = k as f32 / 21.0 * std::f32::consts::TAU;
                t.caja(tiras[k % 7], [a.cos() * 1.05, -0.45, a.sin() * 1.05], [0.2, 0.06, 0.2]);
            }
        }
        "tralalero" => {
            // La aleta de arriba, la cola y tres tenis azules colgando (con suela y rayita blancas).
            let tiburon = t.color("tiburon_azul", [0.32, 0.55, 0.85], 0.0);
            let tenis = t.color("tenis", [0.12, 0.35, 0.95], 0.12);
            t.aleta(tiburon, &[Vec3::new(-0.05, 0.95, -0.25), Vec3::new(-0.05, 2.1, 0.55), Vec3::new(-0.05, 0.95, 0.75)], Vec3::new(0.1, 0.0, 0.0));
            t.aleta(tiburon, &[Vec3::new(-0.05, 0.0, 1.05), Vec3::new(-0.05, 1.2, 2.05), Vec3::new(-0.05, 0.1, 1.55)], Vec3::new(0.1, 0.0, 0.0));
            t.aleta(tiburon, &[Vec3::new(-0.05, -0.1, 1.05), Vec3::new(-0.05, -1.0, 1.85), Vec3::new(-0.05, -0.2, 1.5)], Vec3::new(0.1, 0.0, 0.0));
            for (x, z) in [(-0.55, -0.4), (0.55, -0.4), (0.0, 0.6)] {
                t.cono(tiburon, Vec3::new(x, -0.9, z), Vec3::new(0.0, -1.0, 0.0), 0.45, 0.1, 0.1, 6);
                t.caja(tenis, [x, -1.45, z - 0.08], [0.32, 0.2, 0.5]);
                t.caja(blanco, [x, -1.58, z - 0.08], [0.34, 0.06, 0.52]);
                t.caja(blanco, [x + 0.17, -1.43, z - 0.08], [0.02, 0.06, 0.28]);
            }
        }
        "tungtung" => {
            // Los anillos del tronco arriba, dos bracitos de palo y el bate al costado derecho.
            let claro = t.color("madera_clara", [0.88, 0.70, 0.45], 0.0);
            let oscuro = t.color("madera_oscura", [0.48, 0.32, 0.18], 0.0);
            for (k, r) in [0.95, 0.75, 0.55, 0.35, 0.15].iter().enumerate() {
                t.anillo(if k % 2 == 0 { claro } else { oscuro }, Vec3::new(0.0, 1.0 + k as f32 * 0.005, -0.05), (r - 0.2f32).max(0.0), *r, 0.06, 20);
            }
            for lado in [-1.0, 1.0] {
                t.cono(oscuro, Vec3::new(lado * 1.0, 0.0, -0.1), Vec3::new(lado, 0.15, -0.2), 0.75, 0.06, 0.05, 6);
            }
            t.cono(claro, Vec3::new(1.65, -0.6, -0.35), Vec3::new(0.15, 1.0, -0.25), 2.2, 0.07, 0.16, 8);
            t.esfera(claro, Vec3::new(1.65, -0.62, -0.35), 0.09, Vec3::new(1.0, 1.0, 1.0));
        }
        "bombardiro" => {
            // El hocico de cocodrilo adelante (con dientes y ojos amarillos), alas, cola y dos
            // motores con su helice (giran: `_helicez`, con su pivote en el comentario).
            let croco = t.color("cocodrilo", [0.25, 0.62, 0.22], 0.0);
            let ala = t.color("ala_bombardero", [0.55, 0.60, 0.32], 0.0);
            let ojo = t.color("ojo_croco", [1.0, 0.85, 0.1], 0.5);
            t.caja(croco, [0.0, -0.25, -1.55], [1.2, 0.5, 1.3]);
            for k in 0..5 {
                for lado in [-1.0, 1.0] {
                    t.cono(blanco, Vec3::new(lado * 0.5, -0.02, -1.0 - k as f32 * 0.24), Vec3::new(0.0, -1.0, 0.0), 0.16, 0.06, 0.0, 5);
                }
            }
            for lado in [-1.0, 1.0] {
                t.caja(ojo, [lado * 0.45, 0.82, -0.85], [0.26, 0.2, 0.2]);
                t.caja(negro, [lado * 0.45, 0.84, -0.96], [0.08, 0.16, 0.02]);
            }
            t.aleta(ala, &[Vec3::new(-3.0, -0.1, 0.2), Vec3::new(3.0, -0.1, 0.2), Vec3::new(3.0, -0.1, 0.8), Vec3::new(-3.0, -0.1, 0.8)], Vec3::new(0.0, 0.08, 0.0));
            t.aleta(ala, &[Vec3::new(-0.05, 0.6, 1.0), Vec3::new(-0.05, 1.7, 1.9), Vec3::new(-0.05, 0.6, 1.9)], Vec3::new(0.1, 0.0, 0.0));
            for lado in [-1.0, 1.0] {
                let c = Vec3::new(lado * 1.9, -0.05, -0.2);
                t.cono(negro, c, Vec3::new(0.0, 0.0, 1.0), 0.8, 0.22, 0.18, 10);
                let helice = t.color(if lado < 0.0 { "izq_helicez" } else { "der_helicez" }, [0.8, 0.8, 0.82], 0.05);
                t.0.partes[helice].comentario = format!("pivote: {:.3} {:.3} {:.3}", c.x, c.y, c.z - 0.05);
                t.caja(helice, [c.x, c.y, c.z - 0.05], [0.9, 0.1, 0.04]);
                t.caja(helice, [c.x, c.y, c.z - 0.05], [0.1, 0.9, 0.04]);
            }
        }
        "ballerina" => {
            // La taza de capuchino en el techo (con el asa y el corazon de espuma), el tutu en la
            // cintura y las zapatillas de punta abajo.
            let taza = t.color("taza", [0.97, 0.96, 0.93], 0.05);
            let cafe = t.color("capuchino", [0.55, 0.33, 0.18], 0.0);
            let espuma = t.color("espuma", [1.0, 0.95, 0.86], 0.15);
            let tutu = t.color("tutu", [1.0, 0.70, 0.85], 0.1);
            let c = Vec3::new(0.0, 1.0, -0.1);
            t.cono(taza, c, Vec3::new(0.0, 1.0, 0.0), 0.7, 0.55, 0.68, 18);
            t.cono(cafe, c + Vec3::new(0.0, 0.66, 0.0), Vec3::new(0.0, 1.0, 0.0), 0.05, 0.6, 0.6, 18);
            for (x, z) in [(-0.1, 0.0), (0.1, 0.0), (0.0, 0.12), (-0.18, -0.08), (0.18, -0.08)] {
                t.caja(espuma, [x, c.y + 0.73, c.z + z], [0.12, 0.03, 0.12]);
            }
            t.anillo(taza, Vec3::new(0.78, c.y + 0.35, c.z), 0.1, 0.24, 0.08, 10);
            t.anillo(tutu, Vec3::new(0.0, -0.45, 0.0), 0.9, 1.75, 0.1, 28);
            t.anillo(tutu, Vec3::new(0.0, -0.35, 0.0), 0.9, 1.55, 0.08, 28);
            for x in [-0.35, 0.35] {
                t.cono(tutu, Vec3::new(x, -1.0, 0.0), Vec3::new(0.0, -1.0, 0.0), 0.5, 0.12, 0.05, 8);
            }
        }
        "uva" => {
            t.caja(negro, [0.0, 1.2, -0.62], [1.0, 0.72, 0.12]);
            t.letras(verde, "67", 1.2, -0.54, 0.12);
            t.caja(rosa, [-0.65, 1.25, -0.62], [0.14, 0.14, 0.14]);
            t.caja(azul, [0.63, 1.42, -0.62], [0.12, 0.12, 0.12]);
        }
        _ => {
            for (i, h) in [0.18, 0.32, 0.46].iter().enumerate() {
                t.caja(azul, [-0.95 + i as f32 * 0.19, 0.92 + h * 0.5, -0.35], [0.13, *h, 0.12]);
            }
        }
    }
}
