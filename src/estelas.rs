//! Las estelas de la nave: cada una con su propia forma, no solo otro color.
//!
//! El rastro son las ultimas posiciones de la nave en el mundo (de la mas vieja a la
//! mas nueva). Cada punto del rastro se queda quieto en el mundo mientras envejece,
//! asi que su posicion sirve de semilla: las particulas que salen de el (puerros,
//! notas, burbujas, confeti...) son siempre las mismas y no brincan de un cuadro a
//! otro. Todo se pinta en el framebuffer con luz aditiva (menos el humo, que oscurece)
//! y respeta el z-buffer: lo que la nave tapa, no se ve.

use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;
use crate::post;

/// Un numero de 0 a 1 que depende solo de `semilla` (siempre el mismo).
fn azar(semilla: u32) -> f32 {
    let mut h = semilla.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65535.0
}

/// La semilla de un punto del rastro: su posicion (que no cambia mientras envejece).
fn semilla(p: Vec3) -> u32 {
    let q = |v: f32| (v * 37.0).round() as i32 as u32;
    q(p.x).wrapping_mul(73_856_093) ^ q(p.y).wrapping_mul(19_349_663) ^ q(p.z).wrapping_mul(83_492_791)
}

/// Un pixel de luz (aditivo), si no lo tapa algo mas cercano.
fn luz(fb: &mut Framebuffer, x: f32, y: f32, z: f32, rgb: [f32; 3]) {
    if x < 0.0 || y < 0.0 || x >= fb.ancho as f32 || y >= fb.alto as f32 {
        return;
    }
    let i = y as usize * fb.ancho + x as usize;
    if fb.pasa_profundidad(i, z) {
        fb.sumar(i, rgb);
    }
}

/// Recorre los pixeles de un rectangulo (recortado a la pantalla).
fn cada_pixel(fb: &Framebuffer, x0: f32, y0: f32, x1: f32, y1: f32, mut f: impl FnMut(usize, usize)) {
    let (xa, xb) = (x0.floor().max(0.0) as usize, (x1.ceil() as isize).min(fb.ancho as isize - 1));
    let (ya, yb) = (y0.floor().max(0.0) as usize, (y1.ceil() as isize).min(fb.alto as isize - 1));
    if xb < 0 || yb < 0 {
        return;
    }
    for y in ya..=yb as usize {
        for x in xa..=xb as usize {
            f(x, y);
        }
    }
}

/// Un cuadrito relleno (pixel art), con luz.
fn cuadro(fb: &mut Framebuffer, cx: f32, cy: f32, z: f32, lado: f32, rgb: [f32; 3]) {
    let m = lado * 0.5;
    let mut pixeles = Vec::new();
    cada_pixel(fb, cx - m, cy - m, cx + m - 1.0, cy + m - 1.0, |x, y| pixeles.push((x, y)));
    for (x, y) in pixeles {
        luz(fb, x as f32, y as f32, z, rgb);
    }
}

/// Un aro (burbuja): borde brillante y adentro casi nada.
fn aro(fb: &mut Framebuffer, cx: f32, cy: f32, z: f32, r: f32, rgb: [f32; 3]) {
    let mut pixeles = Vec::new();
    cada_pixel(fb, cx - r - 1.0, cy - r - 1.0, cx + r + 1.0, cy + r + 1.0, |x, y| pixeles.push((x, y)));
    for (x, y) in pixeles {
        let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
        let borde = (1.0 - (d - r).abs() / 1.2).max(0.0);
        // Un brillito arriba a la izquierda, como las burbujas de verdad.
        let brillo = if (x as f32) < cx - r * 0.3 && (y as f32) < cy - r * 0.3 && d < r * 0.6 && d > r * 0.35 { 0.6 } else { 0.0 };
        let k = borde + brillo + if d < r { 0.06 } else { 0.0 };
        if k > 0.0 {
            luz(fb, x as f32, y as f32, z, rgb.map(|c| c * k));
        }
    }
}

/// Un corazon relleno: (x² + y² − 1)³ − x² y³ ≤ 0.
fn corazon(fb: &mut Framebuffer, cx: f32, cy: f32, z: f32, s: f32, rgb: [f32; 3]) {
    let mut pixeles = Vec::new();
    cada_pixel(fb, cx - s * 1.3, cy - s * 1.3, cx + s * 1.3, cy + s * 1.3, |x, y| pixeles.push((x, y)));
    for (x, y) in pixeles {
        let (u, v) = ((x as f32 + 0.5 - cx) / s, -(y as f32 + 0.5 - cy) / s);
        let f = (u * u + v * v - 1.0).powi(3) - u * u * v * v * v;
        if f <= 0.0 {
            luz(fb, x as f32, y as f32, z, rgb);
        }
    }
}

/// Una estrellita de cuatro picos (dos rayitas cruzadas que se adelgazan).
fn estrella(fb: &mut Framebuffer, cx: f32, cy: f32, z: f32, s: f32, rgb: [f32; 3]) {
    let mut pixeles = Vec::new();
    cada_pixel(fb, cx - s, cy - s, cx + s, cy + s, |x, y| pixeles.push((x, y)));
    for (x, y) in pixeles {
        let (dx, dy) = ((x as f32 + 0.5 - cx).abs() / s, (y as f32 + 0.5 - cy).abs() / s);
        let k = ((1.0 - dx) * (1.0 - dy * 6.0).max(0.0)).max((1.0 - dy) * (1.0 - dx * 6.0).max(0.0));
        if k > 0.0 {
            luz(fb, x as f32, y as f32, z, rgb.map(|c| c * k));
        }
    }
}

/// Una corchea: la cabeza (elipse), el palito y la banderita.
fn nota(fb: &mut Framebuffer, cx: f32, cy: f32, z: f32, s: f32, rgb: [f32; 3]) {
    let mut pixeles = Vec::new();
    cada_pixel(fb, cx - s, cy - s * 0.6, cx + s, cy + s * 0.6, |x, y| pixeles.push((x, y)));
    for (x, y) in pixeles {
        let (u, v) = ((x as f32 + 0.5 - cx) / s, (y as f32 + 0.5 - cy) / (s * 0.65));
        if u * u + v * v <= 1.0 {
            luz(fb, x as f32, y as f32, z, rgb);
        }
    }
    let (px, top) = (cx + s * 0.85, cy - s * 2.6);
    fb.linea_de_luz((px, cy, z), (px, top, z), rgb, rgb);
    fb.linea_de_luz((px + 1.0, cy, z), (px + 1.0, top, z), rgb, rgb);
    fb.linea_de_luz((px, top, z), (px + s * 1.1, top + s * 1.0, z), rgb, rgb);
}

/// Una nube de humo: oscurece (no suma luz). La unica estela que no brilla.
fn humo(fb: &mut Framebuffer, cx: f32, cy: f32, z: f32, r: f32, fuerza: f32) {
    let mut pixeles = Vec::new();
    cada_pixel(fb, cx - r, cy - r, cx + r, cy + r, |x, y| pixeles.push((x, y)));
    for (x, y) in pixeles {
        let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt() / r;
        if d >= 1.0 {
            continue;
        }
        let i = y * fb.ancho + x;
        if !fb.pasa_profundidad(i, z) {
            continue;
        }
        let a = (1.0 - d * d) * fuerza;
        let c = fb.leer(i);
        fb.pintar(i, c.map(|v| v * (1.0 - a) + 0.07 * a), 1.0);
    }
}

/// La direccion de la pantalla perpendicular al rastro en el punto `a -> b`.
fn normal(a: (f32, f32, f32), b: (f32, f32, f32)) -> (f32, f32) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l = (dx * dx + dy * dy).sqrt().max(0.001);
    (-dy / l, dx / l)
}

/// Dibuja la estela `estela` a lo largo del rastro.
pub fn dibujar(fb: &mut Framebuffer, vp: &Mat4, rastro: &[Vec3], estela: &str, reloj: f32) {
    let n = rastro.len();
    if n < 2 {
        return;
    }
    let esc = (fb.alto as f32 / 360.0).max(1.0);
    let pantalla: Vec<Option<(f32, f32, f32)>> = rastro.iter().map(|&p| proyectar(p, vp, fb)).collect();
    // El hilo de luz de siempre, abajo de todo (menos en las que lo tapan).
    let hilo = match estela {
        "humo" | "pixel" | "confeti" => None,
        "fuego" => Some([1.0, 0.45, 0.1]),
        "ion" => Some([0.35, 0.6, 1.0]),
        "aurora" => Some([0.3, 1.0, 0.6]),
        "estelar" => Some([1.0, 0.8, 0.3]),
        "corazones" => Some([1.0, 0.35, 0.65]),
        "burbujas" => Some([0.5, 0.8, 1.0]),
        "adn" => Some([0.5, 0.5, 0.9]),
        "astrofago" => Some([1.0, 0.08, 0.16]),
        "teseracto" => Some([1.0, 0.75, 0.35]),
        "seisiete" => Some([0.9, 0.7, 1.0]),
        _ => Some([0.2, 0.9, 0.85]),
    };
    if let Some(c) = hilo {
        for k in 1..n {
            let vida = k as f32 / n as f32;
            if let (Some(a), Some(b)) = (pantalla[k - 1], pantalla[k]) {
                let c = c.map(|v| v * vida * 0.45);
                fb.linea_de_luz(a, b, c, c);
            }
        }
    }
    for k in 1..n {
        let (Some(a), Some(b)) = (pantalla[k - 1], pantalla[k]) else { continue };
        // vida: 1 recien salido de la nave, 0 a punto de apagarse. edad: al reves.
        let vida = k as f32 / n as f32;
        let edad = 1.0 - vida;
        let s = semilla(rastro[k]);
        let (nx, ny) = normal(a, b);
        let z = b.2;
        match estela {
            // Puerros que salen girando: el bulbo blanco y las hojas verdes.
            "miku" if k % 4 == 0 => {
                let p = rastro[k] + Vec3::new(azar(s) - 0.5, azar(s + 1) - 0.3, azar(s + 2) - 0.5) * (edad * 1.6);
                if let Some(c) = proyectar(p, vp, fb) {
                    let ang = azar(s + 3) * 6.28 + edad * 9.0;
                    let (dx, dy) = (ang.cos() * 7.0 * esc, ang.sin() * 7.0 * esc);
                    let blanco = [0.95, 0.95, 0.9].map(|v| v * vida);
                    let verde = [0.25, 0.85, 0.35].map(|v| v * vida);
                    fb.linea_de_luz((c.0 - dx, c.1 - dy, c.2), (c.0, c.1, c.2), blanco, blanco);
                    fb.linea_de_luz((c.0, c.1, c.2), (c.0 + dx * 1.4, c.1 + dy * 1.4, c.2), verde, verde);
                    fb.linea_de_luz((c.0 + 1.0, c.1, c.2), (c.0 + dx * 1.2 + 2.0, c.1 + dy * 1.2, c.2), verde, verde);
                }
            }
            // Llamas que suben y se apagan: amarillo, naranja, rojo... y una chispa de vez en cuando.
            "fuego" if k % 2 == 0 => {
                let sube = Vec3::new((azar(s) - 0.5) * 0.4, edad * 1.4, (azar(s + 1) - 0.5) * 0.4);
                if let Some(c) = proyectar(rastro[k] + sube, vp, fb) {
                    let parpadeo = 0.7 + 0.3 * (reloj * 20.0 + azar(s) * 9.0).sin();
                    let color = [1.0, 0.85 - 0.6 * edad, 0.35 - 0.35 * edad].map(|v| v * vida * parpadeo);
                    post::halo(fb, (c.0, c.1), (3.0 + 6.0 * vida) * esc, color, 1.4);
                }
                if azar(s + 9) > 0.85 {
                    if let Some(c) = proyectar(rastro[k] + Vec3::new(0.0, edad * 2.5, 0.0), vp, fb) {
                        luz(fb, c.0, c.1, c.2, [1.0, 0.9, 0.5].map(|v| v * vida));
                    }
                }
            }
            // Arcoiris estilo gato de internet: seis franjas paralelas y estrellitas.
            "arcoiris" => {
                const FRANJAS: [[f32; 3]; 6] = [[1.0, 0.15, 0.2], [1.0, 0.55, 0.1], [1.0, 0.95, 0.2], [0.25, 0.95, 0.3], [0.2, 0.55, 1.0], [0.6, 0.25, 0.95]];
                let ancho = 1.6 * esc;
                let ola = (k as f32 * 0.5 - reloj * 8.0).sin() * 1.5 * esc;
                for (f, c) in FRANJAS.iter().enumerate() {
                    let o = (f as f32 - 2.5) * ancho + ola;
                    let c = c.map(|v| v * vida * 0.8);
                    fb.linea_de_luz((a.0 + nx * o, a.1 + ny * o, a.2), (b.0 + nx * o, b.1 + ny * o, z), c, c);
                }
                if k % 9 == 0 {
                    let o = (azar(s) - 0.5) * 30.0 * esc;
                    estrella(fb, b.0 + nx * o, b.1 + ny * o, z, 4.0 * esc * (0.6 + 0.4 * (reloj * 6.0 + k as f32).sin().abs()), [1.0, 1.0, 1.0].map(|v| v * vida));
                }
            }
            // Corcheas rosas y cian que se van flotando.
            "notas" if k % 4 == 0 => {
                let p = rastro[k] + Vec3::new((reloj * 2.0 + azar(s) * 6.0).sin() * 0.3 * edad, edad * 1.2, 0.0);
                if let Some(c) = proyectar(p, vp, fb) {
                    let color = if azar(s) < 0.5 { [1.0, 0.4, 0.8] } else { [0.3, 0.9, 1.0] };
                    nota(fb, c.0, c.1, c.2, 4.0 * esc * (0.6 + 0.4 * vida), color.map(|v| v * vida));
                }
            }
            // Corazones que laten y suben.
            "corazones" if k % 4 == 0 => {
                let p = rastro[k] + Vec3::new((azar(s) - 0.5) * 0.6 * edad, edad * 0.9, 0.0);
                if let Some(c) = proyectar(p, vp, fb) {
                    let late = 1.0 + 0.2 * (reloj * 8.0 + azar(s) * 6.0).sin();
                    corazon(fb, c.0, c.1, c.2, 5.0 * esc * late * (0.5 + 0.5 * vida), [1.0, 0.25, 0.55].map(|v| v * vida));
                }
            }
            // La aurora: una cortina que cuelga del rastro y ondula.
            "aurora" => {
                let largo = (16.0 + 10.0 * (k as f32 * 0.3 - reloj * 2.0).sin()) * esc * vida;
                let onda = 0.5 + 0.5 * (k as f32 * 0.25 - reloj * 1.5).sin();
                let arriba = [0.2 + 0.5 * onda, 1.0 - 0.5 * onda, 0.6 + 0.4 * onda].map(|v| v * vida * 1.3);
                fb.linea_de_luz((b.0, b.1, z), (b.0, b.1 + largo, z), arriba, [0.0, 0.0, 0.0]);
            }
            // Un rayo: zigzag que cambia de forma doce veces por segundo, con destellos.
            "ion" => {
                let cuadro = (reloj * 12.0) as u32;
                let sacudir = |q: usize| (azar(cuadro.wrapping_mul(31) + q as u32 * 7) - 0.5) * 9.0 * esc;
                let (oa, ob) = (sacudir(k - 1), sacudir(k));
                let c = [0.7, 0.85, 1.0].map(|v| v * vida);
                fb.linea_de_luz((a.0 + nx * oa, a.1 + ny * oa, a.2), (b.0 + nx * ob, b.1 + ny * ob, z), c, c);
                if k % 12 == (cuadro % 12) as usize {
                    post::halo(fb, (b.0 + nx * ob, b.1 + ny * ob), 8.0 * esc * vida, [0.3, 0.45, 1.0], 1.6);
                }
            }
            // Glitter: estrellitas de cuatro picos regadas alrededor, que titilan.
            "estelar" if k % 2 == 0 => {
                let o = (azar(s) - 0.5) * 26.0 * esc * (0.3 + edad);
                let brillo = (reloj * 5.0 + azar(s + 4) * 6.28).sin().max(0.0);
                if brillo > 0.1 {
                    estrella(fb, b.0 + nx * o, b.1 + ny * o, z, (3.0 + 6.0 * brillo) * esc, [1.0, 0.85, 0.4].map(|v| v * (0.4 + vida) * brillo));
                }
            }
            // 8 bits: cuadritos en la cuadricula de pantalla, con la paleta de una consola vieja.
            "pixel" if k % 2 == 0 => {
                const PALETA: [[f32; 3]; 4] = [[0.55, 0.8, 0.2], [0.2, 0.55, 0.25], [0.95, 0.4, 0.3], [0.3, 0.5, 0.95]];
                let rejilla = 4.0 * esc;
                let lado = (rejilla * (0.5 + 1.5 * vida)).round().max(1.0);
                let (x, y) = ((b.0 / rejilla).round() * rejilla, (b.1 / rejilla).round() * rejilla);
                cuadro(fb, x, y, z, lado, PALETA[(k / 2 + (reloj * 4.0) as usize) % 4].map(|v| v * vida));
            }
            // Burbujas que suben, crecen y revientan.
            "burbujas" if k % 7 == 0 => {
                let p = rastro[k] + Vec3::new((azar(s) - 0.5) * 0.8, edad * 1.6, (azar(s + 1) - 0.5) * 0.8);
                if let Some(c) = proyectar(p, vp, fb) {
                    if edad < 0.85 {
                        aro(fb, c.0, c.1, c.2, (2.0 + 6.0 * edad + 2.0 * azar(s)) * esc, [0.6, 0.85, 1.0].map(|v| v * (0.4 + 0.6 * vida)));
                    } else {
                        // ¡Pop!
                        estrella(fb, c.0, c.1, c.2, 5.0 * esc, [0.7, 0.9, 1.0].map(|v| v * 0.6));
                    }
                }
            }
            // ADN: dos hebras que giran una alrededor de la otra, con sus escalones.
            "adn" => {
                let fase = |q: usize| q as f32 * 0.45 - reloj * 5.0;
                let amp = 7.0 * esc * (0.4 + 0.6 * vida);
                for (hebra, color) in [(0.0, [1.0, 0.35, 0.6]), (std::f32::consts::PI, [0.35, 0.7, 1.0])] {
                    let (oa, ob) = ((fase(k - 1) + hebra).sin() * amp, (fase(k) + hebra).sin() * amp);
                    let c = color.map(|v| v * vida);
                    fb.linea_de_luz((a.0 + nx * oa, a.1 + ny * oa, a.2), (b.0 + nx * ob, b.1 + ny * ob, z), c, c);
                }
                if k % 3 == 0 {
                    let o = fase(k).sin() * amp;
                    let c = [0.8, 0.8, 0.9].map(|v| v * vida * 0.5);
                    fb.linea_de_luz((b.0 + nx * o, b.1 + ny * o, z), (b.0 - nx * o, b.1 - ny * o, z), c, c);
                }
            }
            // Astrofago (Project Hail Mary): bichitos de luz infrarroja que se quedan flotando y
            // laten (se alimentan de la luz: los de cerca de la nave brillan mas).
            "astrofago" if k % 2 == 0 => {
                let deriva = Vec3::new((azar(s) - 0.5) * 1.4, (azar(s + 1) - 0.5) * 1.4, (azar(s + 2) - 0.5) * 1.4) * edad;
                if let Some(c) = proyectar(rastro[k] + deriva, vp, fb) {
                    let late = 0.55 + 0.45 * (reloj * 7.0 + azar(s + 3) * 6.28).sin();
                    let color = [1.0, 0.08, 0.16].map(|v| v * (0.35 + 0.65 * vida) * late);
                    post::halo(fb, (c.0, c.1), (3.0 + 5.0 * vida) * esc, color.map(|v| v * 1.3), 1.0);
                    cuadro(fb, c.0, c.1, c.2, (1.0 + vida) * esc, [1.0, 0.45, 0.5].map(|v| v * (0.4 + 0.6 * vida) * late));
                }
            }
            // El teseracto (Interstellar): marcos dorados que se abren desde el rastro y giran,
            // con los lomos de los libros adentro.
            "teseracto" if k % 6 == 0 => {
                let lado = (3.0 + 16.0 * edad) * esc;
                let giro = edad * 1.6 + azar(s) * 0.5;
                let (c, sn) = (giro.cos() * lado, giro.sin() * lado);
                let esquinas = [(b.0 + c, b.1 + sn), (b.0 - sn, b.1 + c), (b.0 - c, b.1 - sn), (b.0 + sn, b.1 - c)];
                let color = [1.0, 0.78, 0.38].map(|v| v * vida * 0.9);
                for q in 0..4 {
                    let (p, r) = (esquinas[q], esquinas[(q + 1) % 4]);
                    fb.linea_de_luz((p.0, p.1, z), (r.0, r.1, z), color, color.map(|v| v * 0.4));
                }
                // Los lomos: rayitas paralelas a un lado del marco.
                for l in 1..4 {
                    let u = l as f32 / 4.0;
                    let (p, r) = (esquinas[0], esquinas[1]);
                    let (q0, q1) = ((p.0 + (esquinas[3].0 - p.0) * u, p.1 + (esquinas[3].1 - p.1) * u), (r.0 + (esquinas[2].0 - r.0) * u, r.1 + (esquinas[2].1 - r.1) * u));
                    fb.linea_de_luz((q0.0, q0.1, z), ((q0.0 + q1.0) * 0.5, (q0.1 + q1.1) * 0.5, z), color.map(|v| v * 0.5), [0.0; 3]);
                }
            }
            // Six seven: un 6 y un 7 de pixeles que se van turnando y suben (el meme de 2025).
            "seisiete" if k % 5 == 0 => {
                const SEIS: [u8; 5] = [7, 4, 7, 5, 7];
                const SIETE: [u8; 5] = [7, 1, 2, 2, 2];
                let digito = if (k / 5) % 2 == 0 { SEIS } else { SIETE };
                let p = rastro[k] + Vec3::new((azar(s) - 0.5) * 0.6, edad * 1.5, 0.0);
                if let Some(c) = proyectar(p, vp, fb) {
                    let px = (1.5 + 1.5 * vida) * esc;
                    let color = if (k / 5) % 2 == 0 { [1.0, 0.5, 0.9] } else { [0.5, 0.85, 1.0] };
                    for (fila, bits) in digito.iter().enumerate() {
                        for col in 0..3 {
                            if bits & (1 << (2 - col)) != 0 {
                                cuadro(fb, c.0 + (col as f32 - 1.0) * px, c.1 + (fila as f32 - 2.0) * px, c.2, px, color.map(|v| v * vida));
                            }
                        }
                    }
                }
            }
            // Humo negro de camioneta extraurbana: bocanadas que crecen y oscurecen.
            "humo" if k % 3 == 0 => {
                let p = rastro[k] + Vec3::new((azar(s) - 0.5) * 0.5, edad * 0.7, (azar(s + 1) - 0.5) * 0.5);
                if let Some(c) = proyectar(p, vp, fb) {
                    humo(fb, c.0, c.1, c.2, (3.0 + 11.0 * edad) * esc, 0.55 * vida.powf(0.6));
                }
            }
            // Confeti de colores que cae meciendose.
            "confeti" if k % 2 == 0 => {
                const COLORES: [[f32; 3]; 5] = [[1.0, 0.3, 0.4], [1.0, 0.85, 0.2], [0.3, 0.9, 0.5], [0.3, 0.6, 1.0], [0.9, 0.4, 1.0]];
                let p = rastro[k] + Vec3::new((reloj * 3.0 + azar(s) * 6.0).sin() * 0.4 * edad, -edad * 1.4, (azar(s + 1) - 0.5) * 0.8);
                if let Some(c) = proyectar(p, vp, fb) {
                    let gira = (reloj * 6.0 + azar(s + 2) * 6.0).sin();
                    let (w, h) = ((1.0 + 2.0 * gira.abs()) * esc, 3.0 * esc);
                    let color = COLORES[(azar(s + 3) * 5.0) as usize % 5].map(|v| v * (0.3 + 0.7 * vida));
                    let mut pixeles = Vec::new();
                    cada_pixel(fb, c.0 - w, c.1 - h * 0.5, c.0 + w, c.1 + h * 0.5, |x, y| pixeles.push((x, y)));
                    for (x, y) in pixeles {
                        luz(fb, x as f32, y as f32, c.2, color);
                    }
                }
            }
            _ => {}
        }
    }
}
