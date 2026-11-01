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

    /// `vel` es la velocidad de la nave: la estela va hacia atras de ella.
    pub fn dibujar(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4, camara: Vec3, vel: Vec3) {
        let envolver = |x: f32, c: f32| c + (x - c).rem_euclid(LADO) - LADO * 0.5;
        // La estela: lo que la mota "recorre" en un instante, con tope.
        let rapidez = vel.length();
        let estela = if rapidez > 0.1 { vel * (-(rapidez * 0.04).min(6.0) / rapidez) } else { Vec3::zero() };
        for &m in &self.motas {
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
            let brillo = 0.55 * k * k;
            fb.linea_de_luz(a, b, [brillo * 0.8, brillo * 0.8, brillo], [0.0, 0.0, brillo * 0.3]);
        }
    }
}
