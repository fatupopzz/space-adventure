//! El sistema solar de juguete: el Sol, los planetas, la Luna y el Planeta
//! Hada, cada uno con su orbita y su rotacion.
//!
//! Tamanos y distancias van comprimidos a proposito: a escala real Neptuno
//! quedaria fuera de cualquier pantalla y la Tierra seria un pixel. Lo que
//! si se respeta es el orden, quien es mas grande, y que los de adentro
//! orbitan mas rapido (periodo proporcional a distancia^1.5, como Kepler).
//!
//! La tabla de abajo esta en "unidades de diseno" y todo se multiplica por
//! `ESCALA_SISTEMA`: asi la nave (que mide lo mismo) queda chiquita junto a
//! los planetas, como en los juegos de naves, sin reescribir cada numero.

use crate::matematica::{Mat4, Vec3};
use std::f32::consts::TAU;

pub struct Cuerpo {
    pub nombre: &'static str,
    /// El `.obj` en `modelos/`, sin extension.
    pub archivo: &'static str,
    /// Escala del modelo.
    pub escala: f32,
    /// Hasta donde llega el modelo antes de escalar: 1 para los planetas
    /// (radio 1), mas para los que tienen anillos.
    pub radio_modelo: f32,
    /// Alrededor de quien orbita (indice en `CUERPOS`); `None` es el Sol.
    pub padre: Option<usize>,
    /// Radio de la orbita alrededor de su padre.
    pub orbita: f32,
    /// Segundos para dar una vuelta.
    pub periodo_orbita: f32,
    /// Segundos para girar sobre si mismo. Negativo: gira al reves.
    pub periodo_rotacion: f32,
    /// Inclinacion del eje, en grados.
    pub inclinacion: f32,
    /// Donde arranca en su orbita, en radianes.
    pub fase: f32,
}

/// Cuanto se agranda todo el sistema (tamanos y orbitas) respecto a la nave.
pub const ESCALA_SISTEMA: f32 = 3.0;
/// Las orbitas van mas lentas que en la tabla: asi los planetas no se le
/// escapan a la nave, y cuando la nave acompana a un planeta en su orbita
/// (ver `Juego::actualizar`) casi no se nota que todo se desliza.
const LENTITUD_ORBITAS: f32 = 4.0;

const fn planeta(nombre: &'static str, archivo: &'static str, escala: f32, orbita: f32, periodo_orbita: f32, periodo_rotacion: f32, inclinacion: f32, fase: f32) -> Cuerpo {
    Cuerpo {
        nombre,
        archivo,
        escala: escala * ESCALA_SISTEMA,
        radio_modelo: 1.0,
        padre: None,
        orbita: orbita * ESCALA_SISTEMA,
        periodo_orbita: periodo_orbita * LENTITUD_ORBITAS,
        periodo_rotacion,
        inclinacion,
        fase,
    }
}

pub const TIERRA: usize = 3;
pub const MARTE: usize = 5;
pub const SATURNO: usize = 7;
pub const NEPTUNO: usize = 9;
pub const PLUTON: usize = 10;
pub const HADA: usize = 11;
pub const MERCURIO: usize = 1;
pub const VENUS: usize = 2;
pub const LUNA: usize = 4;
pub const JUPITER: usize = 6;
pub const URANO: usize = 8;
pub const VEGETTA: usize = 12;
pub const CHAPIN: usize = 13;

pub const CUERPOS: [Cuerpo; 14] = [
    planeta("Sol", "sol", 6.0, 0.0, 1.0, 60.0, 0.0, 0.0),
    // Mercurio: el que orbita mas rapido.
    planeta("Mercurio", "mercurio", 0.7, 16.0, 24.0, 30.0, 0.0, 0.6),
    // Venus gira al reves de todos los demas.
    planeta("Venus", "venus", 1.2, 24.0, 44.0, -40.0, 0.0, 2.4),
    planeta("Tierra", "tierra", 1.3, 34.0, 74.0, 12.0, 23.5, 4.2),
    // La Luna orbita a la Tierra y gira igual de rapido que su orbita: por
    // eso siempre le muestra la misma cara.
    Cuerpo {
        nombre: "Luna",
        archivo: "luna",
        escala: 0.38 * ESCALA_SISTEMA,
        radio_modelo: 1.0,
        padre: Some(TIERRA),
        orbita: 4.2 * ESCALA_SISTEMA,
        periodo_orbita: 9.0 * LENTITUD_ORBITAS,
        periodo_rotacion: 9.0 * LENTITUD_ORBITAS,
        inclinacion: 0.0,
        fase: 0.0,
    },
    planeta("Marte", "marte", 0.9, 46.0, 117.0, 13.0, 25.2, 1.3),
    planeta("Jupiter", "jupiter", 3.2, 66.0, 201.0, 6.0, 3.1, 5.0),
    Cuerpo { radio_modelo: 4.0, ..planeta("Saturno", "saturno_ritmo", 1.6, 90.0, 320.0, 20.0, 26.7, 3.3) },
    // Urano rueda acostado: su eje esta inclinado 98 grados.
    Cuerpo { radio_modelo: 1.7, ..planeta("Urano", "urano", 2.0, 112.0, 444.0, -10.0, 98.0, 0.4) },
    planeta("Neptuno", "neptuno", 1.9, 132.0, 569.0, 9.0, 28.3, 2.0),
    planeta("Pluton", "pluton", 0.5, 150.0, 689.0, -25.0, 120.0, 4.9),
    planeta("Planeta Hada", "hada", 1.4, 170.0, 831.0, 25.0, 10.0, 1.0),
    // Planeta Vegetta: un cubo de bloques (por eso llega mas lejos que su radio: las esquinas).
    Cuerpo { radio_modelo: 1.75, ..planeta("Planeta Vegetta", "planeta_vegetta", 1.25, 190.0, 960.0, 30.0, 15.0, 2.6) },
    // Planeta Chapin: Guatemala hecha planeta, entre Marte y Jupiter. Su malla no es un `.obj`: la
    // arma el juego (voxels con relieve de ruido, ver chapin.rs); con montanas llega mas lejos.
    Cuerpo { radio_modelo: 1.15, ..planeta("Planeta Chapín", "chapin", 1.4, 56.0, 165.0, 45.0, 14.0, 3.6) },
];

impl Cuerpo {
    /// Donde esta en el tiempo `t`. Las orbitas van en el plano XZ, alrededor
    /// de la posicion del padre: la Luna sigue a la Tierra (jerarquia).
    pub fn posicion(&self, t: f32) -> Vec3 {
        let a = self.fase + TAU * t / self.periodo_orbita;
        let centro = self.padre.map_or(Vec3::zero(), |p| CUERPOS[p].posicion(t));
        centro + Vec3::new(self.orbita * a.cos(), 0.0, -self.orbita * a.sin())
    }

    /// Que tan grande se ve: el radio del planeta, o el de los anillos.
    pub fn radio_visual(&self) -> f32 {
        self.escala * self.radio_modelo
    }

    /// Un punto del mundo llevado al espacio del planeta, sin la escala (la inversa de la
    /// rotacion de `matriz`): asi se sabe sobre que parte de su superficie esta.
    pub fn a_local(&self, t: f32, p: Vec3) -> Vec3 {
        let giro = TAU * t / self.periodo_rotacion;
        Mat4::rotation_y(-giro).transform_point(Mat4::rotation_z(-self.inclinacion.to_radians()).transform_point(p - self.posicion(t)))
    }

    /// El eje de giro del planeta en el mundo (su "arriba").
    pub fn eje(&self) -> Vec3 {
        Mat4::rotation_z(self.inclinacion.to_radians()).transform_point(Vec3::new(0.0, 1.0, 0.0))
    }

    /// La matriz de modelo M: escala, gira sobre su eje, inclina el eje y lo
    /// lleva a su lugar en la orbita (en ese orden, de derecha a izquierda).
    pub fn matriz(&self, t: f32) -> Mat4 {
        let giro = TAU * t / self.periodo_rotacion;
        Mat4::translation(self.posicion(t))
            * Mat4::rotation_z(self.inclinacion.to_radians())
            * Mat4::rotation_y(giro)
            * Mat4::scale(self.escala)
    }
}
