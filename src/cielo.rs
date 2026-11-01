//! El cielo: una nebulosa de colores, la franja de la galaxia y las
//! estrellas.
//!
//! LA NEBULOSA se calcula UNA vez al arrancar y se guarda en un cubemap:
//! seis imagenes, una por cara de un cubo alrededor de la camara. Calcular
//! el ruido por pixel en cada cuadro costaria decenas de milisegundos; leer
//! el cubemap en la direccion de cada pixel cuesta casi nada. Es lo mismo
//! que hace un skybox en la GPU.
//!
//! LAS ESTRELLAS son solo direcciones: estan infinitamente lejos, asi que al
//! avanzar no se mueven, solo al girar. Por eso se les aplica la rotacion de
//! la vista pero no su traslacion. En el salto al hiperespacio se estiran
//! en rayas desde el centro de la pantalla.
//!
//! Para que no sea un cielo cualquiera: galaxias espirales a lo lejos,
//! constelaciones con forma de corazon, nota musical, estrella y gatito, y
//! estrellas fugaces.

use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3, Vec4};

/// Lado de cada cara del cubemap, en texeles: con 512 los filamentos de la nebulosa salen
/// nitidos aunque la camara este quieta mirandolos.
const TAM: usize = 512;

/// El plano de la galaxia: las estrellas y el brillo se juntan cerca de el.
const NORMAL_GALAXIA: Vec3 = Vec3::new(0.32, 0.89, 0.33);

/// Galaxias espirales lejanas: direccion, tamano (radianes), inclinacion,
/// color y sentido de los brazos.
const GALAXIAS: [(Vec3, f32, f32, Vec3, f32); 6] = [
    (Vec3::new(0.62, 0.52, 0.58), 0.16, 0.45, Vec3::new(1.4, 1.05, 1.5), 1.0),
    (Vec3::new(-0.70, 0.30, -0.65), 0.12, 0.8, Vec3::new(0.95, 1.25, 1.5), -1.0),
    (Vec3::new(-0.20, -0.55, 0.81), 0.11, 0.3, Vec3::new(1.5, 1.2, 1.0), 1.0),
    (Vec3::new(0.85, -0.25, -0.46), 0.09, 1.0, Vec3::new(1.5, 1.0, 1.35), -1.0),
    (Vec3::new(-0.35, 0.82, -0.45), 0.10, 0.6, Vec3::new(1.1, 1.5, 1.35), 1.0),
    (Vec3::new(0.10, -0.20, -0.97), 0.08, 0.2, Vec3::new(1.35, 1.25, 1.5), -1.0),
];

/// Constelaciones: direccion del centro, tamano (radianes) y la figura
/// (puntos en 2D, unidos en orden; `None` corta la linea).
type Figura = &'static [Option<(f32, f32)>];
const CORAZON: Figura = &[
    Some((0.0, -1.0)), Some((-0.55, -0.45)), Some((-0.95, 0.1)), Some((-0.85, 0.6)), Some((-0.45, 0.8)),
    Some((0.0, 0.45)), Some((0.45, 0.8)), Some((0.85, 0.6)), Some((0.95, 0.1)), Some((0.55, -0.45)), Some((0.0, -1.0)),
];
const NOTA: Figura = &[
    Some((-0.55, -0.85)), Some((-0.2, -0.7)), Some((-0.25, -0.45)), Some((-0.6, -0.55)), Some((-0.55, -0.85)), None,
    Some((-0.2, -0.65)), Some((-0.2, 0.9)), Some((0.35, 0.55)), Some((0.6, 0.0)),
];
const ESTRELLA: Figura = &[
    Some((0.0, 1.0)), Some((0.24, 0.32)), Some((0.95, 0.31)), Some((0.38, -0.12)), Some((0.59, -0.81)),
    Some((0.0, -0.4)), Some((-0.59, -0.81)), Some((-0.38, -0.12)), Some((-0.95, 0.31)), Some((-0.24, 0.32)), Some((0.0, 1.0)),
];
const GATITO: Figura = &[
    Some((-0.8, -0.7)), Some((-1.0, 0.2)), Some((-0.8, 1.0)), Some((-0.35, 0.6)), Some((0.35, 0.6)), Some((0.8, 1.0)),
    Some((1.0, 0.2)), Some((0.8, -0.7)), Some((-0.8, -0.7)), None,
    Some((-0.4, 0.1)), Some((-0.3, 0.1)), None, Some((0.3, 0.1)), Some((0.4, 0.1)), None,
    Some((-0.15, -0.25)), Some((0.0, -0.35)), Some((0.15, -0.25)),
];
const CONSTELACIONES: [(Vec3, f32, Figura); 4] = [
    (Vec3::new(0.25, 0.42, 0.87), 0.16, CORAZON),
    (Vec3::new(-0.62, 0.38, -0.69), 0.15, NOTA),
    (Vec3::new(0.78, 0.30, -0.55), 0.13, ESTRELLA),
    (Vec3::new(-0.86, 0.22, 0.46), 0.14, GATITO),
];

pub struct Cielo {
    caras: Vec<Vec<[f32; 3]>>,
    /// Direccion, color y una fase propia para titilar.
    estrellas: Vec<(Vec3, [f32; 3], f32)>,
    /// Los segmentos de las constelaciones, como pares de direcciones.
    lineas: Vec<(Vec3, Vec3)>,
}

/// Dos ejes perpendiculares a `d`, para poner cosas "pegadas" al cielo en esa direccion.
fn tangentes(d: Vec3) -> (Vec3, Vec3) {
    let u = d.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
    let u = if u.length() < 0.5 { Vec3::new(1.0, 0.0, 0.0) } else { u };
    (u, u.cross(d).normalize())
}

impl Cielo {
    pub fn new() -> Self {
        // Las seis caras se calculan en paralelo: son independientes.
        let caras = std::thread::scope(|s| {
            let hilos: Vec<_> = (0..6).map(|cara| s.spawn(move || generar_cara(cara))).collect();
            hilos.into_iter().map(|h| h.join().expect("se cayo un hilo del cielo")).collect()
        });
        // Las constelaciones: segmentos entre puntos de la figura, y una
        // estrella brillante en cada punto.
        let mut lineas = Vec::new();
        let mut estrellas = generar_estrellas(5200);
        for &(centro, tam, figura) in &CONSTELACIONES {
            let centro = centro.normalize();
            let (u, v) = tangentes(centro);
            let a_cielo = |(x, y): (f32, f32)| (centro + u * (x * tam) + v * (y * tam)).normalize();
            for par in figura.windows(2) {
                if let [Some(a), Some(b)] = par {
                    lineas.push((a_cielo(*a), a_cielo(*b)));
                }
            }
            for p in figura.iter().flatten() {
                estrellas.push((a_cielo(*p), [1.0, 0.92, 1.0], 0.5));
            }
        }
        Cielo { caras, estrellas, lineas }
    }

    /// Las lineas de las constelaciones y las estrellas fugaces.
    pub fn dibujar_adornos(&self, fb: &mut Framebuffer, vista: &Mat4, proyeccion: &Mat4, t: f32, estiramiento: f32) {
        if estiramiento > 0.05 {
            return;
        }
        let (ancho, alto) = (fb.ancho as f32, fb.alto as f32);
        let a_pantalla = |dir: Vec3| {
            let v = vista.transform_dir(dir);
            let c = proyeccion.mul_vec4(Vec4::new(v.x, v.y, v.z, 1.0));
            (c.w > 0.0).then(|| ((c.x / c.w + 1.0) * 0.5 * ancho, (1.0 - c.y / c.w) * 0.5 * alto, 1.0))
        };
        let brillo = 0.22 + 0.06 * (t * 1.3).sin();
        for &(a, b) in &self.lineas {
            if let (Some(pa), Some(pb)) = (a_pantalla(a), a_pantalla(b)) {
                if (pa.0 - pb.0).abs() < ancho && (pa.1 - pb.1).abs() < alto {
                    fb.linea_de_luz(pa, pb, [brillo * 0.9, brillo * 0.7, brillo * 1.2], [brillo * 0.9, brillo * 0.7, brillo * 1.2]);
                }
            }
        }
        // Una estrella fugaz cada tanto: cada turno de 3.5 s tira una moneda.
        let turno = (t / 3.5).floor();
        let azar = |k: f32| (turno * 12.9898 + k * 78.233).sin() * 43758.545 % 1.0;
        let u = (t / 3.5).fract() / 0.3;
        if azar(1.0).abs() > 0.25 && u < 1.0 {
            let inicio = Vec3::new(azar(2.0), 0.35 + azar(3.0).abs() * 0.5, azar(4.0)).normalize();
            let (tu, tv) = tangentes(inicio);
            let rumbo = (tu * azar(5.0) + tv * -(0.5 + azar(6.0).abs())).normalize();
            let cabeza = (inicio + rumbo * (0.45 * u)).normalize();
            let cola = (inicio + rumbo * (0.45 * u - 0.12).max(0.0)).normalize();
            if let (Some(a), Some(b)) = (a_pantalla(cola), a_pantalla(cabeza)) {
                let k = (1.0 - u) * 1.4;
                fb.linea_de_luz(a, b, [0.0; 3], [0.9 * k, 0.95 * k, 1.0 * k]);
            }
        }
    }

    /// Pinta el cielo en todo el framebuffer. La direccion de cada pixel sale
    /// de la base de la camara (las filas de la matriz de vista) y del FOV.
    /// Se calcula un color cada 2x2 pixeles: la nebulosa no tiene detalle
    /// mas fino que eso y asi cuesta la cuarta parte.
    pub fn dibujar_fondo(&self, fb: &mut Framebuffer, vista: &Mat4, fov_y: f32) {
        let (r, u, f) = base_camara(vista);
        let ty = (fov_y * 0.5).tan();
        let tx = ty * fb.ancho as f32 / fb.alto as f32;
        // Pixel por pixel (con filtro bilineal en el cubemap): asi la nebulosa no sale en cuadritos.
        let paso_x = r * (2.0 / fb.ancho as f32 * tx);
        for y in 0..fb.alto {
            let ny = 1.0 - (y as f32 + 0.5) / fb.alto as f32 * 2.0;
            let mut d = r * (((0.5) / fb.ancho as f32 * 2.0 - 1.0) * tx) + u * (ny * ty) + f;
            for x in 0..fb.ancho {
                fb.pintar(y * fb.ancho + x, self.muestrear(d), 1.0);
                d += paso_x;
            }
        }
    }

    /// Las estrellas, sumadas como luz. `estiramiento` en (0, 1] es el
    /// hiperespacio: cada estrella se vuelve una raya que sale del centro.
    pub fn dibujar_estrellas(&self, fb: &mut Framebuffer, vista: &Mat4, proyeccion: &Mat4, t: f32, estiramiento: f32) {
        let (cx, cy) = (fb.ancho as f32 * 0.5, fb.alto as f32 * 0.5);
        for &(dir, color, fase) in &self.estrellas {
            let v = vista.transform_dir(dir);
            let c = proyeccion.mul_vec4(Vec4::new(v.x, v.y, v.z, 1.0));
            if c.w <= 0.0 {
                continue;
            }
            let x = (c.x / c.w + 1.0) * 0.5 * fb.ancho as f32;
            let y = (1.0 - c.y / c.w) * 0.5 * fb.alto as f32;
            let titilar = 0.75 + 0.25 * (t * (1.5 + fase * 3.0) + fase * 40.0).sin();
            if estiramiento > 0.01 {
                // La raya va de un poco mas cerca del centro hasta la estrella,
                // mas larga mientras mas rapido y mas lejos del centro.
                let k = 1.0 - 0.75 * estiramiento;
                let azul = [color[0] * 0.6, color[1] * 0.8, color[2] * 1.3].map(|c| c * (1.0 + estiramiento));
                let inicio = (cx + (x - cx) * k, cy + (y - cy) * k, 1.0);
                fb.linea_de_luz(inicio, (x, y, 1.0), [0.0; 3], azul);
                continue;
            }
            if x < 0.0 || y < 0.0 || x >= fb.ancho as f32 || y >= fb.alto as f32 {
                continue;
            }
            let (xi, yi) = (x as usize, y as usize);
            let brillo = color.map(|c| c * titilar);
            fb.sumar(yi * fb.ancho + xi, brillo);
            let total = color[0] + color[1] + color[2];
            // Las brillantes llevan un halito alrededor...
            if total > 1.5 {
                let k = ((total - 1.5) / 1.5).min(1.0) * 0.4;
                for (dx, dy) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1), (-1, -1), (1, 1), (-1, 1), (1, -1)] {
                    let (px, py) = (xi as isize + dx, yi as isize + dy);
                    let peso = if dx != 0 && dy != 0 { 0.5 } else { 1.0 };
                    if px >= 0 && py >= 0 && (px as usize) < fb.ancho && (py as usize) < fb.alto {
                        fb.sumar(py as usize * fb.ancho + px as usize, brillo.map(|c| c * k * peso));
                    }
                }
            }
            // ...y las mas brillantes, los destellos en cruz (los de los telescopios), que titilan.
            if total > 2.3 {
                let largo = (3.0 + 7.0 * ((total - 2.3) / 2.0).min(1.0)) * (0.7 + 0.3 * titilar);
                let c = brillo.map(|c| c * 0.45);
                for (dx, dy) in [(1.0f32, 0.0f32), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
                    let fin = (x + dx * largo, y + dy * largo * 0.8, 1.0);
                    fb.linea_de_luz((x + dx, y + dy, 1.0), fin, c, [0.0; 3]);
                }
            }
        }
    }

    /// El color del cielo en una direccion: elige la cara del cubo segun el
    /// eje mas largo y lee con filtro bilineal.
    fn muestrear(&self, d: Vec3) -> [f32; 3] {
        let (ax, ay, az) = (d.x.abs(), d.y.abs(), d.z.abs());
        let (cara, u, v) = if ax >= ay && ax >= az {
            if d.x > 0.0 { (0, -d.z / ax, -d.y / ax) } else { (1, d.z / ax, -d.y / ax) }
        } else if ay >= az {
            if d.y > 0.0 { (2, d.x / ay, d.z / ay) } else { (3, d.x / ay, -d.z / ay) }
        } else if d.z > 0.0 {
            (4, d.x / az, -d.y / az)
        } else {
            (5, -d.x / az, -d.y / az)
        };
        let img = &self.caras[cara];
        let fx = ((u + 1.0) * 0.5 * TAM as f32 - 0.5).clamp(0.0, TAM as f32 - 1.001);
        let fy = ((v + 1.0) * 0.5 * TAM as f32 - 0.5).clamp(0.0, TAM as f32 - 1.001);
        let (x0, y0) = (fx as usize, fy as usize);
        let (sx, sy) = (fx - x0 as f32, fy - y0 as f32);
        let p = |x: usize, y: usize| img[y.min(TAM - 1) * TAM + x.min(TAM - 1)];
        let (a, b, c, e) = (p(x0, y0), p(x0 + 1, y0), p(x0, y0 + 1), p(x0 + 1, y0 + 1));
        [0, 1, 2].map(|k| (a[k] * (1.0 - sx) + b[k] * sx) * (1.0 - sy) + (c[k] * (1.0 - sx) + e[k] * sx) * sy)
    }
}

/// Derecha, arriba y adelante de la camara en el mundo: las filas de la
/// matriz de vista (la tercera es "atras").
fn base_camara(vista: &Mat4) -> (Vec3, Vec3, Vec3) {
    let m = &vista.m;
    (
        Vec3::new(m[0][0], m[0][1], m[0][2]),
        Vec3::new(m[1][0], m[1][1], m[1][2]),
        Vec3::new(-m[2][0], -m[2][1], -m[2][2]),
    )
}

/// La direccion de un texel (u, v en [-1, 1]) de una cara del cubo. Es la
/// inversa exacta de la eleccion de cara de `muestrear`.
fn direccion_de_cara(cara: usize, u: f32, v: f32) -> Vec3 {
    match cara {
        0 => Vec3::new(1.0, -v, -u),
        1 => Vec3::new(-1.0, -v, u),
        2 => Vec3::new(u, 1.0, v),
        3 => Vec3::new(u, -1.0, -v),
        4 => Vec3::new(u, -v, 1.0),
        _ => Vec3::new(-u, -v, -1.0),
    }
    .normalize()
}

fn generar_cara(cara: usize) -> Vec<[f32; 3]> {
    let mut img = Vec::with_capacity(TAM * TAM);
    for y in 0..TAM {
        for x in 0..TAM {
            let u = (x as f32 + 0.5) / TAM as f32 * 2.0 - 1.0;
            let v = (y as f32 + 0.5) / TAM as f32 * 2.0 - 1.0;
            img.push(nebulosa(direccion_de_cara(cara, u, v)));
        }
    }
    img
}

/// Que tan cerca de la franja de la galaxia esta una direccion, en [0, 1].
fn banda(d: Vec3) -> f32 {
    let h = d.dot(NORMAL_GALAXIA);
    (-h * h / 0.035).exp()
}

/// Ruido "con crestas": 1 - |2n - 1| (al cuadrado) en varias capas. Hace lineas finas y
/// ramificadas, como los filamentos de gas de las nebulosas de verdad.
fn crestas(p: Vec3, octavas: u32) -> f32 {
    let (mut suma, mut amp, mut total, mut q) = (0.0, 0.5, 0.0, p);
    for _ in 0..octavas {
        let n = 1.0 - (2.0 * ruido(q) - 1.0).abs();
        suma += n * n * amp;
        total += amp;
        amp *= 0.5;
        q = Vec3::new(0.80 * q.y + 0.60 * q.z, -0.80 * q.x + 0.36 * q.y - 0.48 * q.z, -0.60 * q.x - 0.48 * q.y + 0.64 * q.z) * 2.07;
    }
    suma / total
}

/// El color de la nebulosa: un fondo azul noche casi negro, nubes de violeta a magenta a rosa
/// (torcidas con "domain warping": el ruido deforma al ruido), filamentos brillantes por donde
/// corre el gas, nudos donde nacen estrellas (rosados y casi blancos), velos turquesa, carriles
/// de polvo oscuro que cortan todo, y la franja lechosa de la galaxia. Los colores son los del
/// proyecto (la nave, Miku, Saturno).
fn nebulosa(d: Vec3) -> [f32; 3] {
    let torcer = Vec3::new(
        fbm(d * 1.5 + Vec3::new(5.2, 1.3, 2.8), 4),
        fbm(d * 1.5 + Vec3::new(1.7, 9.2, 4.1), 4),
        fbm(d * 1.5 + Vec3::new(8.3, 2.8, 6.6), 4),
    );
    let n1 = fbm(d * 2.0 + torcer * 2.2 + Vec3::new(3.1, 7.7, 1.3), 6);
    let n2 = fbm(d * 3.6 + torcer * 0.8 + Vec3::new(9.2, 2.4, 5.5), 4);
    let n3 = fbm(d * 2.8 + Vec3::new(1.7, 4.9, 8.3), 4);
    let fil = crestas(d * 3.2 + torcer * 1.8 + Vec3::new(2.2, 6.1, 3.9), 5);
    let polvo = fbm(d * 5.5 + torcer * 1.2 + Vec3::new(6.6, 1.1, 3.3), 5);

    let violeta = Vec3::new(0.30, 0.09, 0.62);
    let magenta = Vec3::new(0.85, 0.18, 0.70);
    let rosa = Vec3::new(1.0, 0.42, 0.62);
    let turquesa = Vec3::new(0.05, 0.55, 0.70);
    let azul = Vec3::new(0.10, 0.20, 0.70);
    let leche = Vec3::new(0.55, 0.46, 0.78);
    let durazno = Vec3::new(1.0, 0.60, 0.38);

    // El fondo: azul noche, un poquito mas claro hacia la franja de la galaxia.
    let b = banda(d);
    let mut c = Vec3::new(0.006, 0.006, 0.022) + Vec3::new(0.012, 0.010, 0.035) * b;
    // Las nubes: el color va de violeta a magenta a rosa segun n2.
    let nube = suave(0.40, 0.74, n1);
    let tono = if n2 < 0.5 { violeta.lerp(magenta, suave(0.3, 0.5, n2)) } else { magenta.lerp(rosa, suave(0.5, 0.7, n2)) };
    c += tono * (nube * 0.48);
    // Los filamentos: brillan mas dentro de las nubes.
    let filamentos = suave(0.55, 0.92, fil) * (0.25 + 0.75 * nube);
    c += tono.lerp(Vec3::new(1.0, 0.8, 0.95), 0.35) * (filamentos * 0.55);
    // Los nudos donde nacen estrellas: lo mas denso de la nube, casi blanco.
    let nudo = suave(0.70, 0.88, n1);
    c += Vec3::new(1.0, 0.70, 0.90) * (nudo * nudo * 0.55);
    // Velos turquesa y azules donde no hay nube rosa (contraste de colores complementarios).
    c += turquesa * (suave(0.52, 0.78, n3) * (1.0 - nube * 0.7) * 0.32);
    c += azul * (suave(0.45, 0.7, n2) * (1.0 - nube) * 0.18);
    c += durazno * (suave(0.6, 0.8, n3) * nube * 0.16);
    // La franja de la galaxia.
    c += leche * (b * (0.10 + 0.24 * n2));
    // Los carriles de polvo: oscurecen la nube (sin llegar a negro).
    let sombra = suave(0.50, 0.66, polvo) * (0.35 + 0.65 * nube.max(b));
    c = c * (1.0 - 0.72 * sombra);
    for &(centro, tam, inclinacion, color, giro) in &GALAXIAS {
        c += color * galaxia(d, centro.normalize(), tam, inclinacion, giro);
    }
    [c.x, c.y, c.z]
}

/// Una galaxia espiral vista de lejos: un nucleo brillante y dos brazos que
/// se enrollan (espiral logaritmica), aplastada segun su inclinacion.
fn galaxia(d: Vec3, centro: Vec3, tam: f32, inclinacion: f32, giro: f32) -> f32 {
    let cerca = d.dot(centro);
    if cerca < (tam * 1.5).cos() {
        return 0.0;
    }
    let (u, v) = tangentes(centro);
    let (x, y) = (d.dot(u) / tam, d.dot(v) / tam);
    let (s, co) = inclinacion.sin_cos();
    let (gx, gy) = (x * co + y * s, (-x * s + y * co) * 2.4);
    let r = (gx * gx + gy * gy).sqrt();
    if r > 1.4 {
        return 0.0;
    }
    let angulo = gy.atan2(gx);
    let brazos = 0.5 + 0.5 * (2.0 * (angulo - giro * 3.2 * (r + 0.05).ln())).cos();
    let disco = (-r * 3.2).exp();
    let nucleo = (-r * r * 60.0).exp();
    0.55 * disco * (0.35 + 0.65 * brazos) + 0.9 * nucleo
}

fn generar_estrellas(cuantas: usize) -> Vec<(Vec3, [f32; 3], f32)> {
    // Generador congruencial: siempre el mismo cielo, sin dependencias.
    let mut semilla: u32 = 0x5EED_1A1A;
    let mut azar = move || {
        semilla = semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (semilla >> 8) as f32 / (1u32 << 24) as f32
    };
    // Colores por temperatura: azuladas, blancas, amarillas y naranjas.
    let colores = [[0.70, 0.80, 1.0], [1.0, 1.0, 1.0], [1.0, 0.93, 0.78], [1.0, 0.78, 0.60]];
    let mut estrellas = Vec::with_capacity(cuantas);
    while estrellas.len() < cuantas {
        // Uniforme sobre la esfera: altura uniforme y angulo uniforme.
        let y = azar() * 2.0 - 1.0;
        let a = azar() * std::f32::consts::TAU;
        let r = (1.0 - y * y).sqrt();
        let dir = Vec3::new(r * a.cos(), y, r * a.sin());
        // Mas estrellas cerca de la franja de la galaxia.
        if azar() > 0.3 + 0.7 * banda(dir) {
            continue;
        }
        let base = colores[(azar() * colores.len() as f32) as usize % colores.len()];
        // Muchas tenues y pocas brillantes.
        let brillo = 0.15 + 0.95 * azar().powi(5);
        estrellas.push((dir, base.map(|c| c * brillo), azar()));
    }
    // Las protagonistas: pocas estrellas muy brillantes (azules, blancas y rosadas), con destellos.
    let heroes = [[0.75, 0.88, 1.0], [1.0, 1.0, 1.0], [1.0, 0.75, 0.92], [1.0, 0.9, 0.7]];
    for k in 0..36 {
        let y = azar() * 2.0 - 1.0;
        let a = azar() * std::f32::consts::TAU;
        let r = (1.0 - y * y).sqrt();
        let dir = Vec3::new(r * a.cos(), y, r * a.sin());
        let brillo = 1.0 + 0.8 * azar();
        estrellas.push((dir, heroes[k % heroes.len()].map(|c| c * brillo), azar()));
    }
    // Cuatro cumulos: montoncitos apretados de estrellas azules y blancas.
    for _ in 0..4 {
        let centro = Vec3::new(azar() * 2.0 - 1.0, azar() * 1.6 - 0.8, azar() * 2.0 - 1.0).normalize();
        let (u, v) = tangentes(centro);
        for _ in 0..140 {
            let (a, r) = (azar() * std::f32::consts::TAU, 0.06 * azar().powi(2));
            let dir = (centro + u * (r * a.cos()) + v * (r * a.sin())).normalize();
            let brillo = 0.2 + 0.8 * azar().powi(4);
            estrellas.push((dir, [0.75 * brillo, 0.85 * brillo, brillo], azar()));
        }
    }
    estrellas
}

// ---------- ruido ----------

fn suave(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Un numero "al azar" en [0, 1] fijo para cada punto de la reja entera.
fn hash(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841) ^ (z as u32).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 0x00ff_ffff as f32
}

/// Ruido de valor: los valores de la reja interpolados suave en 3D.
fn ruido(p: Vec3) -> f32 {
    let (fx, fy, fz) = (p.x.floor(), p.y.floor(), p.z.floor());
    let (x, y, z) = (fx as i32, fy as i32, fz as i32);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (u, v, w) = (s(p.x - fx), s(p.y - fy), s(p.z - fz));
    let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
    l(
        l(l(hash(x, y, z), hash(x + 1, y, z), u), l(hash(x, y + 1, z), hash(x + 1, y + 1, z), u), v),
        l(l(hash(x, y, z + 1), hash(x + 1, y, z + 1), u), l(hash(x, y + 1, z + 1), hash(x + 1, y + 1, z + 1), u), v),
        w,
    )
}

/// Varias capas de ruido, cada una al doble de detalle y la mitad de fuerza.
/// Entre capa y capa se gira el espacio: si no, todas las capas comparten
/// la misma reja y las nubes salen con bordes cuadrados.
fn fbm(p: Vec3, octavas: u32) -> f32 {
    let (mut suma, mut amp, mut total, mut q) = (0.0, 0.5, 0.0, p);
    for _ in 0..octavas {
        suma += ruido(q) * amp;
        total += amp;
        amp *= 0.5;
        q = Vec3::new(
            0.80 * q.y + 0.60 * q.z,
            -0.80 * q.x + 0.36 * q.y - 0.48 * q.z,
            -0.60 * q.x - 0.48 * q.y + 0.64 * q.z,
        ) * 2.03;
    }
    suma / total
}
