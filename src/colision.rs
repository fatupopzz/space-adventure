//! Colisiones de la nave con los cuerpos del sistema solar.
//!
//! La nave es una esfera de radio `RADIO_NAVE`. Cada planeta es una esfera
//! solida, y los anillos de Saturno y de Urano son discos delgados: para
//! revisarlos, la posicion de la nave se lleva al espacio del planeta (se le
//! quita la inclinacion de su eje), que es justo la inversa de su matriz de
//! modelo. Si la nave quedo adentro, se empuja hacia afuera por el camino mas
//! corto.

use crate::escena::{Cuerpo, CUERPOS};
use crate::matematica::{Mat4, Vec3};

pub const RADIO_NAVE: f32 = 0.7;

impl Cuerpo {
    /// El radio del cuerpo solido (sin anillos), ya escalado.
    pub fn radio_cuerpo(&self) -> f32 {
        // El modelo de Saturno es de radio 2; los demas, de radio 1.
        self.escala * if self.archivo == "saturno_ritmo" { 2.0 } else { 1.0 }
    }

    /// Los anillos, de radio interior a exterior, en unidades del modelo.
    fn anillos(&self) -> Option<(f32, f32)> {
        match self.archivo {
            "saturno_ritmo" => Some((2.5, 4.0)),
            "urano" => Some((1.45, 1.7)),
            _ => None,
        }
    }
}

/// Empuja la nave fuera de lo que este tocando. Devuelve contra que cuerpo
/// choco (indice en `CUERPOS`), si choco.
pub fn resolver(pos: &mut Vec3, t: f32) -> Option<usize> {
    let mut choque = None;
    for (i, c) in CUERPOS.iter().enumerate() {
        let centro = c.posicion(t);
        // El cuerpo, con su terreno de verdad (las montanas, los crateres, los volcanes del
        // relieve con ruido): el radio de la superficie en la direccion de la nave.
        let d = *pos - centro;
        let radio = if c.archivo == "chapin" {
            c.escala * crate::chapin::radio_superficie(c.a_local(t, *pos).normalize())
        } else {
            c.radio_cuerpo() * crate::planetas::radio_superficie(c.archivo, c.a_local(t, *pos).normalize()).unwrap_or(1.0)
        };
        let minimo = radio + RADIO_NAVE;
        if d.length() < minimo {
            *pos = centro + d.normalize() * minimo;
            choque = Some(i);
        }
        // Los anillos: un disco delgado en el plano XZ del planeta.
        if let Some((r0, r1)) = c.anillos() {
            let inclinacion = c.inclinacion.to_radians();
            let local = Mat4::rotation_z(-inclinacion).transform_point(*pos - centro) * (1.0 / c.escala);
            let radio = (local.x * local.x + local.z * local.z).sqrt();
            let grosor = RADIO_NAVE / c.escala + 0.05;
            if (r0..=r1).contains(&radio) && local.y.abs() < grosor {
                let empujado = Vec3::new(local.x, grosor.copysign(local.y), local.z) * c.escala;
                *pos = centro + Mat4::rotation_z(inclinacion).transform_point(empujado);
                choque = Some(i);
            }
        }
    }
    choque
}
