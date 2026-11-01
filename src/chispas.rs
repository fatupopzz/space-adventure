//! El JUGO de recoger (lo que hace que se sienta bien): lo que el jugador hace mas seguido, pasar
//! por una nota, tiene que verse y sentirse. Al recoger:
//!
//! - **Chispas** del color de la nota que salen volando y se frenan (pocas: unas quince; con mas
//!   se vuelve ruido), dibujadas como rayitas de luz que se estiran con la velocidad.
//! - Una **onda** de luz que se abre desde donde estaba la nota.
//! - Un **letrero** que sube y se desvanece ("+1"); cada cinco del combo, uno grande que rebota.
//! - Con el cometa, ademas, una **pausa cortita** (hit-stop, menos de una decima de segundo: el
//!   mundo se detiene y las chispas siguen), un **destello** y la camara se sacude.
//!
//! Todo vive en el mundo (se proyecta con la camara), asi que las chispas se quedan donde estaba la
//! nota aunque la nave siga de largo.

use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;

/// Los colores de las notas (rosa y cian) y el dorado de los premios grandes.
pub const ROSA: [f32; 3] = [1.0, 0.45, 0.82];
pub const CIAN: [f32; 3] = [0.35, 0.95, 1.0];
pub const DORADO: [f32; 3] = [1.0, 0.85, 0.35];

struct Chispa {
    pos: Vec3,
    vel: Vec3,
    color: [f32; 3],
    vida: f32,
    dura: f32,
}

struct Onda {
    pos: Vec3,
    color: [f32; 3],
    vida: f32,
    dura: f32,
    tam: f32,
}

/// Un letrero que flota en el mundo: lo dibuja la interfaz (ver `letreros`).
pub struct Letrero {
    pub pos: Vec3,
    pub texto: String,
    pub color: [f32; 3],
    vida: f32,
    dura: f32,
    pub grande: bool,
}

impl Letrero {
    /// De 0 (recien salido) a 1 (ya se fue).
    pub fn avance(&self) -> f32 {
        1.0 - (self.vida / self.dura).clamp(0.0, 1.0)
    }
}

#[derive(Default)]
pub struct Jugo {
    chispas: Vec<Chispa>,
    ondas: Vec<Onda>,
    pub letreros: Vec<Letrero>,
    semilla: u32,
    /// El hit-stop que queda (segundos): mientras dure, el mundo no avanza.
    pub pausa: f32,
    /// El destello blanco (de 0 a 1), que se apaga solo.
    pub destello: f32,
    /// El latido del contador de notas de la interfaz (1 al recoger, se apaga solo).
    pub latido_notas: f32,
}

impl Jugo {
    fn azar(&mut self) -> f32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) as f32 / 16_777_216.0
    }

    /// `n` chispas que salen de `pos` en todas direcciones, a mas o menos `rapidez`.
    pub fn estallar(&mut self, pos: Vec3, color: [f32; 3], n: usize, rapidez: f32) {
        for _ in 0..n {
            // Una direccion al azar sobre la esfera.
            let z = self.azar() * 2.0 - 1.0;
            let a = self.azar() * std::f32::consts::TAU;
            let r = (1.0 - z * z).max(0.0).sqrt();
            let dir = Vec3::new(r * a.cos(), z, r * a.sin());
            let vel = dir * (rapidez * (0.55 + 0.6 * self.azar()));
            let dura = 0.35 + 0.3 * self.azar();
            // Unas mas claritas que otras (como brillitos).
            let k = 0.75 + 0.5 * self.azar();
            self.chispas.push(Chispa { pos, vel, color: color.map(|c| (c * k).min(1.2)), vida: dura, dura });
        }
        // Que no se junten de mas (un combo largo con iman podria soltar cientos).
        if self.chispas.len() > 400 {
            let sobran = self.chispas.len() - 400;
            self.chispas.drain(..sobran);
        }
    }

    /// Una onda de luz que se abre desde `pos` (`tam`: que tan grande llega, en el mundo).
    pub fn onda(&mut self, pos: Vec3, color: [f32; 3], tam: f32) {
        self.ondas.push(Onda { pos, color, vida: 0.4, dura: 0.4, tam });
    }

    /// Un letrero que sube desde `pos`.
    pub fn letrero(&mut self, pos: Vec3, texto: impl Into<String>, color: [f32; 3], grande: bool) {
        let dura = if grande { 1.4 } else { 0.8 };
        self.letreros.push(Letrero { pos, texto: texto.into(), color, vida: dura, dura, grande });
        if self.letreros.len() > 24 {
            self.letreros.remove(0);
        }
    }

    pub fn actualizar(&mut self, dt: f32) {
        // Las chispas se frenan rapido (como en el aire, aunque sea el espacio: se lee mejor).
        let freno = (-3.5 * dt).exp();
        for c in &mut self.chispas {
            c.pos += c.vel * dt;
            c.vel = c.vel * freno;
            c.vida -= dt;
        }
        self.chispas.retain(|c| c.vida > 0.0);
        for o in &mut self.ondas {
            o.vida -= dt;
        }
        self.ondas.retain(|o| o.vida > 0.0);
        for l in &mut self.letreros {
            l.vida -= dt;
            // Suben, cada vez mas despacio.
            l.pos += Vec3::new(0.0, (if l.grande { 1.2 } else { 2.2 }) * dt * (l.vida / l.dura), 0.0);
        }
        self.letreros.retain(|l| l.vida > 0.0);
        self.destello = (self.destello - dt * 4.0).max(0.0);
        self.latido_notas = (self.latido_notas - dt * 4.0).max(0.0);
    }

    /// Las chispas y las ondas, sobre lo ya dibujado (se suman como luz).
    pub fn dibujar(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4, ojo: Vec3) {
        for o in &self.ondas {
            if let Some((x, y, z)) = proyectar(o.pos, vista_proyeccion, fb) {
                let u = 1.0 - o.vida / o.dura;
                let d = (o.pos - ojo).length().max(1.0);
                // Un aro nitido que se abre rapido (sale de golpe y se frena) y se apaga.
                let abre = 1.0 - (1.0 - u).powi(3);
                let radio = (6.0 + o.tam * 420.0 / d) * (0.15 + abre);
                let k = (1.0 - u).powi(2);
                let color = o.color.map(|c| c * 1.3 * k);
                let lados = 40;
                for i in 0..lados {
                    let (a0, a1) = (i as f32 / lados as f32 * std::f32::consts::TAU, (i + 1) as f32 / lados as f32 * std::f32::consts::TAU);
                    for grosor in [0.0, 1.0] {
                        let r = radio + grosor;
                        fb.linea_de_luz((x + a0.cos() * r, y + a0.sin() * r, z), (x + a1.cos() * r, y + a1.sin() * r, z), color, color.map(|c| c * 0.3));
                    }
                }
                crate::post::halo(fb, (x, y), radio * 0.8, o.color.map(|c| c * 0.18 * k), 1.2);
            }
        }
        for c in &self.chispas {
            let k = (c.vida / c.dura).clamp(0.0, 1.0);
            // La rayita: de donde esta a donde estaba hace un instante (se estira con la velocidad),
            // de dos pixeles de grueso, y en la punta un brillito (como los "star bits").
            let atras = c.pos - c.vel * 0.045;
            if let (Some(a), Some(b)) = (proyectar(c.pos, vista_proyeccion, fb), proyectar(atras, vista_proyeccion, fb)) {
                let brillo = k * k;
                let (cola, borde) = (c.color.map(|v| v * brillo * 1.5), c.color.map(|v| v * brillo * 0.4));
                fb.linea_de_luz(a, b, cola, borde);
                fb.linea_de_luz((a.0 + 1.0, a.1, a.2), (b.0 + 1.0, b.1, b.2), cola, borde);
                let d = (c.pos - ojo).length().max(1.0);
                crate::post::halo(fb, (a.0, a.1), (2.0 + 60.0 / d).min(9.0), c.color.map(|v| v * brillo * 0.9), 1.0);
            }
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Las chispas salen, se frenan y se acaban; los letreros suben y se van.
    #[test]
    fn todo_se_apaga_solo() {
        let mut j = Jugo::default();
        j.estallar(Vec3::zero(), ROSA, 16, 9.0);
        j.onda(Vec3::zero(), ROSA, 2.0);
        j.letrero(Vec3::zero(), "+1", ROSA, false);
        assert_eq!(j.chispas.len(), 16);
        j.actualizar(0.1);
        let lejos = j.chispas.iter().map(|c| c.pos.length()).fold(0.0f32, f32::max);
        assert!(lejos > 0.3, "las chispas no salieron");
        assert!(j.letreros[0].pos.y > 0.0, "el letrero no subio");
        for _ in 0..100 {
            j.actualizar(0.05);
        }
        assert!(j.chispas.is_empty() && j.ondas.is_empty() && j.letreros.is_empty());
    }
}
