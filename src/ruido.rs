//! Ruido procedural, como se vio en clase.
//!
//! - **Perlin mejorado (Ken Perlin, 2002), en 3D**: una reticula de enteros donde cada esquina
//!   tiene un gradiente pseudoaleatorio (sacado de una tabla de permutacion barajada con la
//!   semilla). El valor en un punto es la interpolacion suave (`6t^5 - 15t^4 + 10t^3`) de los
//!   productos punto entre cada gradiente y el vector de la esquina al punto. Da colinas suaves,
//!   sin la cuadricula que se nota en el ruido de valor.
//! - **fBm** (*fractional Brownian motion*): varias octavas de Perlin, cada una al doble de
//!   frecuencia (lacunaridad) y a la mitad de amplitud (ganancia). Montanas con detalle chiquito
//!   encima del grande.
//! - **Crestas** (*ridged multifractal*): `1 - |ruido|`, elevado, para cordilleras con filo.
//! - **Ruido de valor** sin tabla (`valor3`): un hash de las coordenadas enteras interpolado; es
//!   barato y no necesita estado, por eso lo usan los shaders (la lava y el plasma del Sol).
//!
//! La misma semilla da siempre el mismo ruido: el Planeta Chapin sale igual en cada corrida.

use crate::matematica::Vec3;

/// Un generador de Perlin con su tabla de permutacion (de 256, repetida para no hacer modulo).
pub struct Perlin {
    perm: [u8; 512],
}

impl Perlin {
    pub fn new(semilla: u32) -> Self {
        // Fisher-Yates con un generador congruencial lineal (como en C: a = 1664525, c = 1013904223).
        let mut p: [u8; 256] = std::array::from_fn(|i| i as u8);
        let mut s = semilla.wrapping_mul(2_654_435_761) | 1;
        for i in (1..256).rev() {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            p.swap(i, (s >> 8) as usize % (i + 1));
        }
        Perlin { perm: std::array::from_fn(|i| p[i & 255]) }
    }

    /// Perlin 3D, mas o menos en [-1, 1].
    pub fn ruido(&self, q: Vec3) -> f32 {
        let (xi, yi, zi) = (q.x.floor(), q.y.floor(), q.z.floor());
        let (x, y, z) = (q.x - xi, q.y - yi, q.z - zi);
        let (xi, yi, zi) = ((xi as i32 & 255) as usize, (yi as i32 & 255) as usize, (zi as i32 & 255) as usize);
        let (u, v, w) = (suavizar(x), suavizar(y), suavizar(z));
        let p = &self.perm;
        let a = p[xi] as usize + yi;
        let (aa, ab) = (p[a] as usize + zi, p[a + 1] as usize + zi);
        let b = p[xi + 1] as usize + yi;
        let (ba, bb) = (p[b] as usize + zi, p[b + 1] as usize + zi);
        let l = |t: f32, a: f32, b: f32| a + t * (b - a);
        l(
            w,
            l(v, l(u, grad(p[aa], x, y, z), grad(p[ba], x - 1.0, y, z)), l(u, grad(p[ab], x, y - 1.0, z), grad(p[bb], x - 1.0, y - 1.0, z))),
            l(
                v,
                l(u, grad(p[aa + 1], x, y, z - 1.0), grad(p[ba + 1], x - 1.0, y, z - 1.0)),
                l(u, grad(p[ab + 1], x, y - 1.0, z - 1.0), grad(p[bb + 1], x - 1.0, y - 1.0, z - 1.0)),
            ),
        )
    }

    /// fBm: `octavas` capas de Perlin; cada una al doble de frecuencia y la mitad de fuerza.
    /// Normalizado para quedar mas o menos en [-1, 1].
    pub fn fbm(&self, q: Vec3, octavas: u32) -> f32 {
        let (mut suma, mut amplitud, mut frecuencia, mut total) = (0.0, 1.0, 1.0, 0.0);
        for _ in 0..octavas {
            suma += amplitud * self.ruido(q * frecuencia);
            total += amplitud;
            amplitud *= 0.5;
            frecuencia *= 2.0;
        }
        suma / total
    }

    /// Crestas: montanas con filo (en [0, 1]).
    pub fn crestas(&self, q: Vec3, octavas: u32) -> f32 {
        let (mut suma, mut amplitud, mut frecuencia, mut total) = (0.0, 1.0, 1.0, 0.0);
        for _ in 0..octavas {
            let c = 1.0 - self.ruido(q * frecuencia).abs();
            suma += amplitud * c * c;
            total += amplitud;
            amplitud *= 0.5;
            frecuencia *= 2.0;
        }
        suma / total
    }
}

/// La curva de Perlin mejorado: 6t^5 - 15t^4 + 10t^3 (derivadas primera y segunda en 0 en los
/// bordes de cada celda: no se notan las costuras).
fn suavizar(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// El producto punto con uno de los 12 gradientes (las aristas de un cubo), elegido por el hash.
fn grad(hash: u8, x: f32, y: f32, z: f32) -> f32 {
    match hash & 15 {
        0 | 12 => x + y,
        1 | 14 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x + z,
        5 => -x + z,
        6 => x - z,
        7 => -x - z,
        8 => y + z,
        9 | 13 => -y + z,
        10 => y - z,
        _ => -y - z,
    }
}

/// Un hash de tres enteros a [0, 1) (para el ruido de valor).
pub fn hash3(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841) ^ (z as u32).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 16_777_216.0
}

/// Ruido de valor 3D en [0, 1]: sin tabla ni estado (para los shaders).
pub fn valor3(q: Vec3) -> f32 {
    let (xi, yi, zi) = (q.x.floor(), q.y.floor(), q.z.floor());
    let (u, v, w) = (suavizar(q.x - xi), suavizar(q.y - yi), suavizar(q.z - zi));
    let (x, y, z) = (xi as i32, yi as i32, zi as i32);
    let l = |t: f32, a: f32, b: f32| a + t * (b - a);
    l(
        w,
        l(v, l(u, hash3(x, y, z), hash3(x + 1, y, z)), l(u, hash3(x, y + 1, z), hash3(x + 1, y + 1, z))),
        l(v, l(u, hash3(x, y, z + 1), hash3(x + 1, y, z + 1)), l(u, hash3(x, y + 1, z + 1), hash3(x + 1, y + 1, z + 1))),
    )
}

/// Ruido de valor 2D que se repite cada `periodo` (la reticula se envuelve): para texturas sin
/// costuras, como el mapa de normales.
pub fn valor2_periodico(x: f32, y: f32, periodo: i32) -> f32 {
    let (xi, yi) = (x.floor(), y.floor());
    let (u, v) = (suavizar(x - xi), suavizar(y - yi));
    let (x0, y0) = ((xi as i32).rem_euclid(periodo), (yi as i32).rem_euclid(periodo));
    let (x1, y1) = ((x0 + 1) % periodo, (y0 + 1) % periodo);
    let l = |t: f32, a: f32, b: f32| a + t * (b - a);
    l(v, l(u, hash3(x0, y0, 0), hash3(x1, y0, 0)), l(u, hash3(x0, y1, 0), hash3(x1, y1, 0)))
}

/// fBm de ruido de valor, en [0, 1].
pub fn valor_fbm(q: Vec3, octavas: u32) -> f32 {
    let (mut suma, mut amplitud, mut frecuencia, mut total) = (0.0, 1.0, 1.0, 0.0);
    for _ in 0..octavas {
        suma += amplitud * valor3(q * frecuencia);
        total += amplitud;
        amplitud *= 0.5;
        frecuencia *= 2.03;
    }
    suma / total
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn la_misma_semilla_da_el_mismo_ruido_y_otra_semilla_otro() {
        let (a, b, c) = (Perlin::new(7), Perlin::new(7), Perlin::new(8));
        let p = Vec3::new(1.37, -2.11, 0.53);
        assert_eq!(a.ruido(p), b.ruido(p));
        assert_ne!(a.fbm(p, 4), c.fbm(p, 4));
    }

    #[test]
    fn el_ruido_queda_en_su_rango_y_es_cero_en_la_reticula() {
        let r = Perlin::new(2026);
        for i in 0..2000 {
            let p = Vec3::new(i as f32 * 0.137, i as f32 * -0.071, i as f32 * 0.029);
            let v = r.ruido(p);
            assert!((-1.05..=1.05).contains(&v), "{v}");
            assert!((0.0..=1.0).contains(&r.crestas(p, 3)));
            assert!((0.0..=1.0).contains(&valor3(p)));
        }
        // En las esquinas de la reticula el Perlin vale cero (el gradiente por un vector nulo).
        assert_eq!(r.ruido(Vec3::new(3.0, 5.0, -2.0)), 0.0);
    }
}
