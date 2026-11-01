//! Post-procesado: lo que se le hace a la imagen terminada.
//!
//! - **Bloom**: lo muy brillante (el Sol, las llamas, los brillos emisivos)
//!   se desborda en un resplandor suave. Se toma lo que pasa de un umbral,
//!   se achica la imagen a un cuarto, se difumina y se suma de vuelta.
//! - **Corona y destellos de lente**: el Sol deja un halo y una fila de
//!   circulitos de colores que cruzan la pantalla por el centro, como cuando
//!   una camara de verdad mira a una luz fuerte. Se apagan si algo lo tapa.
//! - **Vineta, tinte y destello**: oscurece las esquinas, tine de azul en el
//!   hiperespacio y pone el blanco de la llegada.

use crate::framebuffer::Framebuffer;

/// Cuanto se achica la imagen para el bloom.
const REDUCCION: usize = 4;
/// Desde que brillo (0 a 1) empieza a desbordarse la luz.
const UMBRAL: f32 = 0.78;

pub struct Post {
    ancho: usize,
    alto: usize,
    a: Vec<[f32; 3]>,
    b: Vec<[f32; 3]>,
}

impl Post {
    pub fn new(fb: &Framebuffer) -> Self {
        let (ancho, alto) = (fb.ancho.div_ceil(REDUCCION), fb.alto.div_ceil(REDUCCION));
        Post { ancho, alto, a: vec![[0.0; 3]; ancho * alto], b: vec![[0.0; 3]; ancho * alto] }
    }

    pub fn bloom(&mut self, fb: &mut Framebuffer, fuerza: f32) {
        let (w, h) = (self.ancho, self.alto);
        // 1. Solo lo brillante, promediado en bloques.
        for by in 0..h {
            for bx in 0..w {
                let mut suma = [0.0f32; 3];
                let mut n = 0.0f32;
                for y in by * REDUCCION..((by + 1) * REDUCCION).min(fb.alto) {
                    for x in bx * REDUCCION..((bx + 1) * REDUCCION).min(fb.ancho) {
                        let c = fb.leer(y * fb.ancho + x);
                        let luz = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
                        let k = ((luz - UMBRAL) / (1.0 - UMBRAL)).clamp(0.0, 1.0);
                        for q in 0..3 {
                            suma[q] += c[q] * k * k;
                        }
                        n += 1.0;
                    }
                }
                self.a[by * w + bx] = suma.map(|s| s / n.max(1.0));
            }
        }
        // 2. Difuminar: gaussiano separable (horizontal y vertical), dos veces
        //    para que el halo llegue lejos.
        const PESOS: [f32; 5] = [0.227, 0.194, 0.121, 0.054, 0.016];
        for _ in 0..2 {
            for (pasada, (dx, dy)) in [(1isize, 0isize), (0, 1)].into_iter().enumerate() {
                let (origen, destino) = if pasada == 0 { (&self.a, &mut self.b) } else { (&self.b, &mut self.a) };
                for y in 0..h as isize {
                    for x in 0..w as isize {
                        let mut s = [0.0f32; 3];
                        let mut sumar = |k: isize, peso: f32| {
                            let px = (x + dx * k).clamp(0, w as isize - 1) as usize;
                            let py = (y + dy * k).clamp(0, h as isize - 1) as usize;
                            let c = origen[py * w + px];
                            for q in 0..3 {
                                s[q] += c[q] * peso;
                            }
                        };
                        sumar(0, PESOS[0]);
                        for (k, &peso) in PESOS.iter().enumerate().skip(1) {
                            sumar(k as isize, peso);
                            sumar(-(k as isize), peso);
                        }
                        destino[y as usize * w + x as usize] = s;
                    }
                }
            }
        }
        // 3. Agrandar con filtro bilineal y sumar.
        for y in 0..fb.alto {
            let fy = ((y as f32 + 0.5) / REDUCCION as f32 - 0.5).clamp(0.0, h as f32 - 1.0);
            let (y0, ty) = (fy as usize, fy.fract());
            let y1 = (y0 + 1).min(h - 1);
            for x in 0..fb.ancho {
                let fx = ((x as f32 + 0.5) / REDUCCION as f32 - 0.5).clamp(0.0, w as f32 - 1.0);
                let (x0, tx) = (fx as usize, fx.fract());
                let x1 = (x0 + 1).min(w - 1);
                let p = |xx: usize, yy: usize| self.a[yy * w + xx];
                let (c00, c10, c01, c11) = (p(x0, y0), p(x1, y0), p(x0, y1), p(x1, y1));
                let c = [0, 1, 2].map(|q| {
                    ((c00[q] * (1.0 - tx) + c10[q] * tx) * (1.0 - ty) + (c01[q] * (1.0 - tx) + c11[q] * tx) * ty) * fuerza
                });
                fb.sumar(y * fb.ancho + x, c);
            }
        }
    }
}

/// Que fraccion del Sol se ve: se miran puntos sobre su disco y se cuenta
/// en cuantos lo que hay en el z-buffer esta detras de su cara delantera.
pub fn visibilidad_sol(fb: &Framebuffer, centro: (f32, f32), radio: f32, z_frente: f32) -> f32 {
    let mut vistos = 0;
    let mut total = 0;
    for k in 0..13 {
        let (dx, dy) = if k == 0 {
            (0.0, 0.0)
        } else {
            let a = k as f32 * std::f32::consts::TAU / 12.0;
            (a.cos() * radio * 0.6, a.sin() * radio * 0.6)
        };
        let (x, y) = (centro.0 + dx, centro.1 + dy);
        if x < 0.0 || y < 0.0 || x >= fb.ancho as f32 || y >= fb.alto as f32 {
            continue;
        }
        total += 1;
        if fb.profundidad(y as usize * fb.ancho + x as usize) >= z_frente {
            vistos += 1;
        }
    }
    if total == 0 { 0.0 } else { vistos as f32 / total as f32 }
}

/// Un circulo de luz suave sumado en `centro`.
pub fn halo(fb: &mut Framebuffer, centro: (f32, f32), radio: f32, color: [f32; 3], forma: f32) {
    if radio < 0.5 {
        return;
    }
    let x0 = (centro.0 - radio).floor().max(0.0) as usize;
    let x1 = ((centro.0 + radio).ceil() as isize).min(fb.ancho as isize - 1);
    let y0 = (centro.1 - radio).floor().max(0.0) as usize;
    let y1 = ((centro.1 + radio).ceil() as isize).min(fb.alto as isize - 1);
    if x1 < 0 || y1 < 0 {
        return;
    }
    for y in y0..=y1 as usize {
        for x in x0..=x1 as usize {
            let d = ((x as f32 + 0.5 - centro.0).powi(2) + (y as f32 + 0.5 - centro.1).powi(2)).sqrt() / radio;
            if d < 1.0 {
                let k = (1.0 - d).powf(forma);
                fb.sumar(y * fb.ancho + x, color.map(|c| c * k));
            }
        }
    }
}

/// Como `halo`, pero solo detras de lo que tenga profundidad `z` (lo de enfrente se queda como
/// esta): asi algo oscuro se recorta contra el brillo (el monolito, a contraluz).
pub fn halo_detras(fb: &mut Framebuffer, centro: (f32, f32), radio: f32, color: [f32; 3], forma: f32, z: f32) {
    if radio < 0.5 {
        return;
    }
    let x0 = (centro.0 - radio).floor().max(0.0) as usize;
    let x1 = ((centro.0 + radio).ceil() as isize).min(fb.ancho as isize - 1);
    let y0 = (centro.1 - radio).floor().max(0.0) as usize;
    let y1 = ((centro.1 + radio).ceil() as isize).min(fb.alto as isize - 1);
    if x1 < 0 || y1 < 0 {
        return;
    }
    for y in y0..=y1 as usize {
        for x in x0..=x1 as usize {
            let i = y * fb.ancho + x;
            if fb.profundidad(i) < z {
                continue;
            }
            let d = ((x as f32 + 0.5 - centro.0).powi(2) + (y as f32 + 0.5 - centro.1).powi(2)).sqrt() / radio;
            if d < 1.0 {
                fb.sumar(i, color.map(|c| c * (1.0 - d).powf(forma)));
            }
        }
    }
}

/// La corona del Sol y los destellos de lente. `centro` y `radio` son los
/// del Sol en pantalla; `visible` sale de `visibilidad_sol`.
pub fn sol(fb: &mut Framebuffer, centro: (f32, f32), radio: f32, visible: f32) {
    if visible <= 0.0 {
        return;
    }
    halo(fb, centro, radio * 3.2, [1.0 * visible * 0.55, 0.62 * visible * 0.55, 0.25 * visible * 0.55], 2.2);
    halo(fb, centro, radio * 1.6, [visible * 0.35, visible * 0.30, visible * 0.18], 1.5);

    // Los destellos van sobre la recta Sol -> centro de la pantalla, y se
    // ven mas cuando el Sol esta cerca del centro.
    let medio = (fb.ancho as f32 * 0.5, fb.alto as f32 * 0.5);
    let lejos = (((centro.0 - medio.0) / medio.0).powi(2) + ((centro.1 - medio.1) / medio.1).powi(2)).sqrt();
    let fuerza = visible * (1.0 - lejos * 0.5).clamp(0.15, 1.0) * 0.22;
    let tam = fb.alto as f32;
    for (k, r, color) in [
        (0.45, 0.035, [0.85, 0.55, 1.0]),
        (0.75, 0.020, [0.4, 1.0, 0.9]),
        (1.15, 0.060, [1.0, 0.45, 0.75]),
        (1.45, 0.028, [1.0, 0.85, 0.4]),
        (1.9, 0.090, [0.55, 0.5, 1.0]),
    ] {
        let p = (centro.0 + (medio.0 - centro.0) * k, centro.1 + (medio.1 - centro.1) * k);
        halo(fb, p, r * tam, color.map(|c| c * fuerza), 0.6);
    }
}

/// Exposicion automatica, como el ojo o una camara: se mide el brillo
/// promedio de la imagen (mirando uno de cada 7 pixeles) y, si ya es muy
/// brillante, se devuelve un factor menor que 1 para bajar el bloom.
pub fn exposicion(fb: &Framebuffer) -> f32 {
    let (mut suma, mut n) = (0.0f32, 0.0f32);
    for i in (0..fb.ancho * fb.alto).step_by(7) {
        let c = fb.leer(i);
        suma += 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
        n += 1.0;
    }
    let promedio = suma / n.max(1.0);
    (1.0 - (promedio - 0.25) * 2.2).clamp(0.15, 1.0)
}

/// Vineta en las esquinas, tinte (hiperespacio) y destello blanco (llegada).
pub fn terminar(fb: &mut Framebuffer, tinte: [f32; 3], tinte_fuerza: f32, blanco: f32) {
    let (cx, cy) = (fb.ancho as f32 * 0.5, fb.alto as f32 * 0.5);
    for y in 0..fb.alto {
        let dy = (y as f32 + 0.5 - cy) / cy;
        for x in 0..fb.ancho {
            let dx = (x as f32 + 0.5 - cx) / cx;
            let r2 = dx * dx * 0.6 + dy * dy;
            let vineta = 1.0 - 0.35 * r2 * r2;
            let i = y * fb.ancho + x;
            let c = fb.leer(i);
            let c = [0, 1, 2].map(|q| {
                let v = c[q] * vineta;
                // El tinte es luz que se suma hacia los bordes, como un tunel.
                let v = v + tinte[q] * r2 * tinte_fuerza * 0.3;
                v + (1.0 - v) * blanco
            });
            fb.pintar(i, c, 1.0);
        }
    }
}

/// Un VHS rebobinando (el retrogrado de Mercurio): cada fila se corre de
/// lado, los canales rojo y azul se separan, una barra de "tracking" baja
/// por la pantalla y aparecen rayas de estatica. `fuerza` de 0 a 1.
pub fn vhs(fb: &mut Framebuffer, t: f32, fuerza: f32) {
    if fuerza <= 0.0 {
        return;
    }
    let (w, h) = (fb.ancho, fb.alto);
    let mut fila = vec![[0.0f32; 3]; w];
    // La barra de tracking baja muy rapido (al rebobinar todo va acelerado).
    let barra = (t * 1.7).fract() * h as f32;
    let separa = (3.0 * fuerza) as isize;
    for y in 0..h {
        let yf = y as f32;
        let ruido = ((yf * 12.9898 + (t * 60.0).floor() * 78.233).sin() * 43758.5).fract();
        let en_barra = ((yf - barra) / (h as f32 * 0.06)).abs() < 1.0;
        let corre = fuerza * (4.0 * (yf * 0.05 + t * 20.0).sin() + if en_barra { 26.0 * (ruido - 0.3) } else { 0.0 });
        for (x, c) in fila.iter_mut().enumerate() {
            *c = fb.leer(y * w + x);
        }
        for x in 0..w {
            let tomar = |dx: isize| fila[(x as isize + dx - corre as isize).clamp(0, w as isize - 1) as usize];
            let (r, g, b) = (tomar(separa)[0], tomar(0)[1], tomar(-separa)[2]);
            let estatica = if ruido > 0.985 { 0.6 * fuerza } else { 0.0 };
            let lavado = if en_barra { 0.12 * fuerza } else { 0.0 };
            fb.pintar(y * w + x, [r + estatica + lavado, g + estatica + lavado, b + estatica + lavado], 1.0);
        }
    }
}

/// NIEBLA por distancia: con el z-buffer se saca que tan lejos esta cada pixel (deshaciendo la
/// proyeccion en perspectiva) y se mezcla con `color` de `desde` a `hasta` metros. Lo que no
/// tiene nada dibujado (el cielo) queda del color de la niebla, salvo con `respetar_cielo`.
pub fn niebla(fb: &mut Framebuffer, cerca: f32, lejos: f32, desde: f32, hasta: f32, color: [f32; 3], respetar_cielo: bool) {
    for i in 0..fb.ancho * fb.alto {
        let z = fb.profundidad(i);
        let k = if z.is_finite() {
            let nz = z * 2.0 - 1.0;
            let d = 2.0 * cerca * lejos / (lejos + cerca - nz * (lejos - cerca));
            ((d - desde) / (hasta - desde)).clamp(0.0, 1.0)
        } else if respetar_cielo {
            0.0
        } else {
            1.0
        };
        if k > 0.0 {
            let c = fb.leer(i);
            fb.pintar(i, [c[0] + (color[0] - c[0]) * k, c[1] + (color[1] - c[1]) * k, c[2] + (color[2] - c[2]) * k], 1.0);
        }
    }
}

/// Sombras de contacto (el "unsharp masking" del z-buffer, de Luft, Colditz y Deussen, 2006): se
/// compara la distancia de cada pixel con el promedio de su alrededor; si esta mas lejos que lo que
/// lo rodea (un rincon, el piso junto a un personaje, la pared detras de algo), se oscurece. Da
/// la sensacion de profundidad que da la oclusion ambiental, y con dos pasadas de caja (separables,
/// con sumas corridas) cuesta casi nada. `radio` en pixeles; `fuerza` de 0 a 1.
pub fn sombras_de_contacto(fb: &mut Framebuffer, radio: usize, fuerza: f32) {
    let (w, h) = (fb.ancho, fb.alto);
    if w < 2 * radio + 2 || h < 2 * radio + 2 {
        return;
    }
    // La distancia (salvo una escala): en perspectiva, 1 / (1 - z). El cielo, muy lejos.
    let lejos = 2.0e3f32;
    let dist: Vec<f32> = (0..w * h).map(|i| {
        let z = fb.profundidad(i);
        if z.is_finite() { (1.0 / (1.0 - z).max(1e-6)).min(lejos) } else { lejos }
    }).collect();
    // El promedio de la caja: primero a lo largo de cada fila, luego de cada columna.
    let mut filas = vec![0.0f32; w * h];
    let n = (2 * radio + 1) as f32;
    for y in 0..h {
        let fila = &dist[y * w..(y + 1) * w];
        let mut suma: f32 = (0..=radio).map(|x| fila[x]).sum::<f32>() + fila[0] * radio as f32;
        for x in 0..w {
            filas[y * w + x] = suma / n;
            let entra = fila[(x + radio + 1).min(w - 1)];
            let sale = fila[x.saturating_sub(radio)];
            suma += entra - sale;
        }
    }
    let mut prom = vec![0.0f32; w * h];
    for x in 0..w {
        let mut suma: f32 = (0..=radio).map(|y| filas[y * w + x]).sum::<f32>() + filas[x] * radio as f32;
        for y in 0..h {
            prom[y * w + x] = suma / n;
            let entra = filas[(y + radio + 1).min(h - 1) * w + x];
            let sale = filas[y.saturating_sub(radio) * w + x];
            suma += entra - sale;
        }
    }
    for i in 0..w * h {
        let d = dist[i];
        if d >= lejos {
            continue;
        }
        // Que tan mas lejos esta que su alrededor (relativo a su distancia).
        let atras = (d - prom[i]) / d;
        if atras > 0.0 {
            let k = 1.0 - fuerza * (atras * 3.0).min(1.0);
            let c = fb.leer(i);
            fb.pintar(i, [c[0] * k, c[1] * k, c[2] * k], 1.0);
        }
    }
}

/// Rayos de luz (los "god rays" del Sol, como en GPU Gems 3): lo brillante de la imagen se
/// difumina en lineas hacia el `centro` (donde esta el Sol en pantalla). Lo que tapa al Sol
/// (un planeta, la nave) corta los rayos solo, porque ahi no hay brillo que arrastrar. Se hace a
/// un cuarto de resolucion: son rayos suaves, y asi cuesta poco.
pub fn rayos_de_luz(fb: &mut Framebuffer, centro: (f32, f32), color: [f32; 3], fuerza: f32) {
    const R: usize = 4;
    const PASOS: usize = 28;
    if fuerza <= 0.0 {
        return;
    }
    let (w, h) = (fb.ancho / R, fb.alto / R);
    // Lo muy brillante, a un cuarto de resolucion.
    let mut brillo = vec![0.0f32; w * h];
    for by in 0..h {
        for bx in 0..w {
            let mut s = 0.0;
            for y in by * R..by * R + R {
                for x in bx * R..bx * R + R {
                    let c = fb.leer(y * fb.ancho + x);
                    let luz = 0.3 * c[0] + 0.55 * c[1] + 0.15 * c[2];
                    s += ((luz - 0.8) / 0.2).clamp(0.0, 1.0);
                }
            }
            brillo[by * w + bx] = s / (R * R) as f32;
        }
    }
    let (cx, cy) = (centro.0 / R as f32, centro.1 / R as f32);
    let mut rayos = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let (mut px, mut py) = (x as f32, y as f32);
            // Hasta el 85% del camino hacia el Sol, cada vez pesando menos.
            let (dx, dy) = ((cx - px) * 0.85 / PASOS as f32, (cy - py) * 0.85 / PASOS as f32);
            let (mut suma, mut peso) = (0.0, 1.0);
            for _ in 0..PASOS {
                px += dx;
                py += dy;
                if px < 0.0 || py < 0.0 || px >= w as f32 || py >= h as f32 {
                    break;
                }
                suma += brillo[py as usize * w + px as usize] * peso;
                peso *= 0.95;
            }
            rayos[y * w + x] = suma / PASOS as f32;
        }
    }
    // De vuelta a la resolucion completa (bilineal) y sumado como luz.
    for y in 0..fb.alto {
        let fy = ((y as f32 + 0.5) / R as f32 - 0.5).clamp(0.0, h as f32 - 1.001);
        let (y0, sy) = (fy as usize, fy.fract());
        for x in 0..fb.ancho {
            let fx = ((x as f32 + 0.5) / R as f32 - 0.5).clamp(0.0, w as f32 - 1.001);
            let (x0, sx) = (fx as usize, fx.fract());
            let p = |xx: usize, yy: usize| rayos[yy.min(h - 1) * w + xx.min(w - 1)];
            let v = (p(x0, y0) * (1.0 - sx) + p(x0 + 1, y0) * sx) * (1.0 - sy) + (p(x0, y0 + 1) * (1.0 - sx) + p(x0 + 1, y0 + 1) * sx) * sy;
            if v > 0.001 {
                fb.sumar(y * fb.ancho + x, color.map(|c| c * v * fuerza));
            }
        }
    }
}

/// La atmosfera de un planeta: un anillo de luz de su color justo en la orilla (por dentro, como
/// el brillo de canto del aire visto de lado, y por fuera, desvaneciendose). Lo que este delante
/// del planeta (mas cerca que `z_frente`) no se pinta.
pub fn atmosfera(fb: &mut Framebuffer, centro: (f32, f32), radio: f32, color: [f32; 3], fuerza: f32, z_frente: f32) {
    if radio < 2.0 || fuerza <= 0.0 {
        return;
    }
    let alcance = radio * 1.3;
    let x0 = (centro.0 - alcance).floor().max(0.0) as usize;
    let x1 = ((centro.0 + alcance).ceil() as isize).min(fb.ancho as isize - 1);
    let y0 = (centro.1 - alcance).floor().max(0.0) as usize;
    let y1 = ((centro.1 + alcance).ceil() as isize).min(fb.alto as isize - 1);
    if x1 < 0 || y1 < 0 {
        return;
    }
    for y in y0..=y1 as usize {
        for x in x0..=x1 as usize {
            let d = ((x as f32 + 0.5 - centro.0).powi(2) + (y as f32 + 0.5 - centro.1).powi(2)).sqrt() / radio;
            let k = if d < 1.0 {
                // Por dentro: solo cerca de la orilla.
                let t = ((d - 0.78) / 0.22).clamp(0.0, 1.0);
                t * t * 0.7
            } else {
                // Por fuera: se apaga rapido.
                (-((d - 1.0) / 0.1).powi(2)).exp()
            };
            if k < 0.01 {
                continue;
            }
            let i = y * fb.ancho + x;
            if fb.profundidad(i) < z_frente {
                continue;
            }
            fb.sumar(i, color.map(|c| c * k * fuerza));
        }
    }
}

/// La gradacion de color (como el "color grading" de las peliculas): mas saturacion y una
/// curva en S de contraste, y las sombras teñidas de `sombras` (y lo claro, de `luces`).
pub fn gradacion(fb: &mut Framebuffer, saturacion: f32, contraste: f32, sombras: [f32; 3], luces: [f32; 3]) {
    for i in 0..fb.ancho * fb.alto {
        let c = fb.leer(i);
        let luma = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
        let s = c.map(|v| (luma + (v - luma) * saturacion).clamp(0.0, 1.0));
        let curva = s.map(|v| v + contraste * (v * v * (3.0 - 2.0 * v) - v));
        let oscuro = (1.0 - luma * 2.0).clamp(0.0, 1.0);
        let claro = ((luma - 0.5) * 2.0).clamp(0.0, 1.0);
        let r = [0, 1, 2].map(|k| curva[k] + sombras[k] * oscuro + luces[k] * claro);
        fb.pintar(i, r, 1.0);
    }
}

/// LENTE GRAVITACIONAL (un shader de pantalla, despues de dibujar todo): un agujero negro dobla
/// la luz que pasa cerca. Con la formula del lente puntual, el pixel a distancia `r` del centro
/// muestra lo que en la imagen estaria a `r - einstein^2 / r`: el fondo se estira alrededor, sale
/// el anillo de Einstein y, cerca de la sombra, lo de atras aparece del otro lado (por eso el disco
/// de acrecion de atras se ve doblado arriba y abajo de la sombra, como Gargantua en Interstellar).
/// Adentro de `sombra`, negro. Lo que esta DELANTE del agujero (su profundidad es menor que `z`:
/// la nave, la mitad de enfrente del disco) no se dobla.
///
/// Con `esfera` (un agujero de gusano): adentro de la esfera se ve todo el cielo de atras, de
/// cabeza y encogido (r -> einstein^2 / r), con el `tinte` de otra galaxia, como una bola de cristal.
pub fn lente(fb: &mut Framebuffer, centro: (f32, f32), z: f32, sombra: f32, einstein: f32, alcance: f32, esfera: Option<(f32, [f32; 3])>) {
    let (ancho, alto) = (fb.ancho as i32, fb.alto as i32);
    let (x0, x1) = (((centro.0 - alcance) as i32).max(0), ((centro.0 + alcance) as i32 + 1).min(ancho));
    let (y0, y1) = (((centro.1 - alcance) as i32).max(0), ((centro.1 + alcance) as i32 + 1).min(alto));
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    // La imagen de antes (para leer de ella mientras se escribe la nueva), con un margen: el lente
    // puede traer luz de un poco mas afuera.
    let margen = (einstein * einstein / sombra.max(1.0)).min(alcance) as i32 + 2;
    let (mx0, mx1, my0, my1) = ((x0 - margen).max(0), (x1 + margen).min(ancho), (y0 - margen).max(0), (y1 + margen).min(alto));
    let w = (mx1 - mx0) as usize;
    let antes: Vec<([f32; 3], f32)> = (my0..my1).flat_map(|y| (mx0..mx1).map(move |x| (y, x))).map(|(y, x)| {
        let i = (y * ancho + x) as usize;
        (fb.leer(i), fb.profundidad(i))
    }).collect();
    // Solo se dobla la luz de lo que esta DETRAS (lo de enfrente, como la nave, no pasa por el
    // lente: de alla se ve el fondo oscuro).
    let leer = |x: f32, y: f32| -> Option<[f32; 3]> {
        let (xi, yi) = (x as i32, y as i32);
        if xi < mx0 || xi >= mx1 || yi < my0 || yi >= my1 {
            return None;
        }
        let (color, profundidad) = antes[(yi - my0) as usize * w + (xi - mx0) as usize];
        Some(if profundidad < z { [0.01, 0.01, 0.03] } else { color })
    };
    let e2 = einstein * einstein;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * ancho + x) as usize;
            // Lo de enfrente del agujero no se toca.
            if fb.profundidad(i) < z {
                continue;
            }
            let (dx, dy) = (x as f32 + 0.5 - centro.0, y as f32 + 0.5 - centro.1);
            let r = (dx * dx + dy * dy).sqrt().max(0.01);
            if r > alcance {
                continue;
            }
            // Se desvanece hacia la orilla del alcance (sin corte).
            let orilla = paso(alcance, alcance * 0.7, r);
            let color = match esfera {
                Some((radio, tinte)) if r < radio => {
                    // La bola de cristal: el cielo de atras, invertido y con el color de alla.
                    let s = e2 / r;
                    let fuente = leer(centro.0 - dx / r * s, centro.1 - dy / r * s).unwrap_or([0.0; 3]);
                    let borde = paso(radio, radio * 0.8, r);
                    let c = [fuente[0] * tinte[0], fuente[1] * tinte[1], fuente[2] * tinte[2]];
                    [c[0] + borde * 0.25 * tinte[0], c[1] + borde * 0.25 * tinte[1], c[2] + borde * 0.25 * tinte[2]]
                }
                _ if r < sombra => [0.0, 0.0, 0.0],
                _ => {
                    let s = r - e2 / r;
                    let Some(fuente) = leer(centro.0 + dx / r * s, centro.1 + dy / r * s) else { continue };
                    // El anillo de fotones: un hilo de luz justo afuera de la sombra.
                    let anillo = if esfera.is_none() { (-(r - sombra * 1.04).powi(2) / (sombra * 0.004 + 0.6)).exp() * 0.9 } else { 0.0 };
                    [fuente[0] + anillo, fuente[1] + anillo * 0.85, fuente[2] + anillo * 0.6]
                }
            };
            let viejo = fb.leer(i);
            fb.pintar(i, [viejo[0] + (color[0] - viejo[0]) * orilla, viejo[1] + (color[1] - viejo[1]) * orilla, viejo[2] + (color[2] - viejo[2]) * orilla], 1.0);
        }
    }
}

/// 0 en `desde`, 1 en `hasta` (suave), para cualquier orden de los dos.
fn paso(desde: f32, hasta: f32, x: f32) -> f32 {
    let t = ((x - desde) / (hasta - desde)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
