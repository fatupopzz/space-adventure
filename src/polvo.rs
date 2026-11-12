//! Polvo espacial: puntitos de luz flotando cerca de la camara.
//!
//! Sin nada cerca, en el espacio no se nota que uno se mueve: los planetas
//! estan lejos y las estrellas no se mueven. El polvo si pasa de largo, y
//! con la velocidad se estira en una estela. Hay pocas motas y viven en un
//! cubo alrededor de la camara: la que se sale por un lado vuelve a entrar
//! por el otro, asi que parece que el espacio entero esta lleno.

use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;

/// Lado del cubo de polvo alrededor de la camara.
const LADO: f32 = 36.0;

pub struct Polvo {
    motas: Vec<Vec3>,
}

impl Polvo {
    pub fn new(cuantas: usize) -> Self {
        let mut semilla: u32 = 0xD057_0042;
        let mut azar = move || {
            semilla = semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (semilla >> 8) as f32 / (1u32 << 24) as f32 * LADO
        };
        Polvo { motas: (0..cuantas).map(|_| Vec3::new(azar(), azar(), azar())).collect() }
    }

    /// `vel` es la velocidad de la nave: la estela va hacia atras de ella. Con `turbo` (0 a 1) las
    /// motas se vuelven lineas de velocidad: mas largas, mas claras y tirando a celeste.
    pub fn dibujar(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4, camara: Vec3, vel: Vec3, turbo: f32) {
        let envolver = |x: f32, c: f32| c + (x - c).rem_euclid(LADO) - LADO * 0.5;
        // La estela: lo que la mota "recorre" en un instante, con tope.
        let rapidez = vel.length();
        let largo = (rapidez * 0.04).min(6.0) * (1.0 + 0.7 * turbo);
        let estela = if rapidez > 0.1 { vel * (-largo / rapidez) } else { Vec3::zero() };
        // Quieta, la nave flota entre motas tenues; con prisa se encienden (se nota que uno avanza).
        let prisa = (rapidez / 60.0).min(1.0);
        let fuerza = 0.5 + 0.3 * prisa + 0.4 * turbo;
        for (i, &m) in self.motas.iter().enumerate() {
            let p = Vec3::new(envolver(m.x, camara.x), envolver(m.y, camara.y), envolver(m.z, camara.z));
            let distancia = (p - camara).length();
            // Se apagan al acercarse al borde del cubo para que no aparezcan de golpe.
            let k = (1.0 - distancia / (LADO * 0.5)).clamp(0.0, 1.0);
            if k <= 0.0 {
                continue;
            }
            let (Some(a), Some(b)) = (proyectar(p, vista_proyeccion, fb), proyectar(p + estela, vista_proyeccion, fb)) else {
                continue;
            };
            // (Solo en la orilla: a medio cubo ya van con toda su luz.)
            let brillo = fuerza * (k * 2.5).min(1.0);
            // De tres colores, como el resto del juego: blanco azulado, lila y celeste.
            let tinte = mezclar(TINTES[i % TINTES.len()], [0.55, 0.9, 1.0], turbo * 0.6);
            let cabeza = tinte.map(|c| c * brillo);
            fb.linea_de_luz(a, b, cabeza, tinte.map(|c| c * brillo * 0.08));
            // La cabeza de la mota: un punto mas (si no, quieta es un solo pixel y no se ve).
            fb.linea_de_luz((a.0 + 1.0, a.1, a.2), (a.0 + 1.0, a.1 + 1.0, a.2), cabeza.map(|c| c * 0.6), cabeza.map(|c| c * 0.6));
        }
    }
}

const TINTES: [[f32; 3]; 3] = [[0.8, 0.8, 1.0], [0.85, 0.6, 1.0], [0.5, 0.85, 1.0]];

fn mezclar(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|k| a[k] + (b[k] - a[k]) * t)
}
