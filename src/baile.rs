//! El baile de Miku en la intro.
//!
//! `miku_baile.obj` trae a Miku de pie con los pivotes de hombros, caderas,
//! cuello y coletas en su cabecera. Cada parte pertenece a un grupo segun su
//! nombre, y cada grupo gira sobre su pivote: es una jerarquia chiquita (el
//! cuerpo mueve a la cabeza, la cabeza a las coletas) armada con matrices.
//!
//! La coreografia la decide la intro con `Paso`: saludar a la camara,
//! bailar (va cambiando de movimiento cada compas a 120 BPM), hacer un
//! corazon con los brazos, y abrir los brazos para girar.

use crate::matematica::{Mat4, Vec3};
use crate::obj::Modelo;
use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Paso {
    Saludo,
    Baile,
    Corazon,
    Giro,
    /// Parada, con los brazos abajo y respirando (en el juzgado).
    Quieta,
    /// Señalando al frente con el brazo derecho: ¡PROTESTO!
    Senalar,
}

#[derive(Clone, Copy)]
enum Grupo {
    Cuerpo,
    Cabeza,
    /// El lado: 1 derecho (+X), -1 izquierdo.
    Brazo(f32),
    Pierna(f32),
    Coleta(f32),
}

/// Los dos angulos de un brazo: hacia el frente (X) y hacia el lado (Z).
type Brazo = (f32, f32);

/// Una pose completa: brazo derecho, brazo izquierdo, inclinacion de la
/// cabeza y que tanto rebota.
#[derive(Clone, Copy)]
struct Pose {
    der: Brazo,
    izq: Brazo,
    cabeza: f32,
    rebote: f32,
}

impl Pose {
    fn mezclar(a: Pose, b: Pose, k: f32) -> Pose {
        let l = |x: f32, y: f32| x + (y - x) * k;
        let lb = |x: Brazo, y: Brazo| (l(x.0, y.0), l(x.1, y.1));
        Pose { der: lb(a.der, b.der), izq: lb(a.izq, b.izq), cabeza: l(a.cabeza, b.cabeza), rebote: l(a.rebote, b.rebote) }
    }
}

pub struct Baile {
    grupos: Vec<Grupo>,
    hombro: [Vec3; 2],
    cadera: [Vec3; 2],
    cuello: Vec3,
    coleta: [Vec3; 2],
}

fn pivote(m: &Modelo, nombre: &str) -> Vec3 {
    let n: Vec<f32> = m
        .comentario(&format!("pivote {nombre}"))
        .unwrap_or("")
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    match n[..] {
        [x, y, z, ..] => Vec3::new(x, y, z),
        _ => Vec3::zero(),
    }
}

impl Baile {
    pub fn de(m: &Modelo) -> Self {
        let grupos = m
            .partes
            .iter()
            .map(|p| {
                let n = p.nombre.as_str();
                let lado = if n.contains("_der") { 1.0 } else { -1.0 };
                let empieza = |pre: &[&str]| pre.iter().any(|x| n.starts_with(x));
                if empieza(&["brazo_", "manga_", "mano_"]) {
                    Grupo::Brazo(lado)
                } else if empieza(&["muslo_", "bota_", "suela_"]) {
                    Grupo::Pierna(lado)
                } else if empieza(&["torso", "corbata", "falda", "cuello"]) {
                    Grupo::Cuerpo
                } else if n.starts_with("coleta_") {
                    Grupo::Coleta(lado)
                } else {
                    Grupo::Cabeza
                }
            })
            .collect();
        Baile {
            grupos,
            hombro: [pivote(m, "hombro_der"), pivote(m, "hombro_izq")],
            cadera: [pivote(m, "cadera_der"), pivote(m, "cadera_izq")],
            cuello: pivote(m, "cuello"),
            coleta: [pivote(m, "coleta_der"), pivote(m, "coleta_izq")],
        }
    }

    /// La pose de cada paso en el tiempo `t`.
    fn pose(paso: Paso, t: f32) -> Pose {
        let fase = t * 2.0 * PI; // un ciclo por beat (120 BPM = 2 beats por segundo)
        match paso {
            // La mano derecha arriba saludando rapido; la otra en la cintura.
            Paso::Saludo => Pose { der: (0.15, 2.6 + 0.4 * (t * 9.0).sin()), izq: (0.0, -0.45), cabeza: 0.18, rebote: 0.5 },
            Paso::Baile => {
                // Cambia de movimiento cada compas de 4 tiempos.
                match ((t * 2.0 / 4.0) as i32).rem_euclid(3) {
                    0 => {
                        let a = 2.5 + 0.35 * (fase).sin();
                        Pose { der: (0.0, a), izq: (0.0, a), cabeza: 0.15 * (fase * 0.5).sin(), rebote: 1.0 }
                    }
                    1 => {
                        let s = (fase * 0.5).sin();
                        Pose { der: (1.3 + 0.9 * s, 0.2), izq: (1.3 - 0.9 * s, 0.2), cabeza: 0.12 * s, rebote: 1.0 }
                    }
                    _ => {
                        let c = 0.5 + 0.25 * fase.sin();
                        Pose { der: (1.3, -c), izq: (1.3, -c), cabeza: 0.1 * (fase * 0.5).sin(), rebote: 1.0 }
                    }
                }
            }
            // Los dos brazos arriba y un poco al frente: las manos se juntan sobre la cabeza.
            Paso::Corazon => Pose { der: (0.35, 2.75), izq: (0.35, 2.75), cabeza: 0.12 * (t * 3.0).sin(), rebote: 0.3 },
            // Brazos abiertos para girar.
            Paso::Giro => Pose { der: (0.0, 1.5), izq: (0.0, 1.5), cabeza: 0.0, rebote: 0.0 },
            Paso::Quieta => Pose { der: (0.05 * (t * 1.5).sin(), 0.12), izq: (0.05 * (t * 1.5).sin(), 0.12), cabeza: 0.04 * (t * 0.7).sin(), rebote: 0.1 },
            // El brazo derecho estirado al frente y la otra mano en la cintura, como en Ace Attorney.
            Paso::Senalar => Pose { der: (1.55, 0.1), izq: (0.0, -0.45), cabeza: -0.08, rebote: 0.0 },
        }
    }

    /// El muñequito cabezon del tablero (Jupiter): el cuerpo quieto y la cabeza, mas grande,
    /// bamboleandose sobre el cuello con los angulos `z` (de lado) y `x` (adelante).
    pub fn cabezon(&self, z: f32, x: f32) -> Vec<Mat4> {
        let alrededor = |p: Vec3, r: Mat4| Mat4::translation(p) * r * Mat4::translation(-p);
        let cabeza = alrededor(self.cuello, Mat4::rotation_z(z) * Mat4::rotation_x(x) * Mat4::scale(1.3));
        self.grupos
            .iter()
            .map(|g| match *g {
                Grupo::Cabeza => cabeza,
                Grupo::Coleta(lado) => {
                    let i = if lado > 0.0 { 0 } else { 1 };
                    cabeza * alrededor(self.coleta[i], Mat4::rotation_z(lado * 0.2 - z * 0.8))
                }
                _ => Mat4::identity(),
            })
            .collect()
    }

    /// Caminando: las piernas y los brazos van y vienen (cruzados) con `fase` (radianes), con un
    /// rebotito en cada paso.
    pub fn caminar(&self, fase: f32) -> Vec<Mat4> {
        let alrededor = |p: Vec3, r: Mat4| Mat4::translation(p) * r * Mat4::translation(-p);
        let s = fase.sin();
        let raiz = Mat4::translation(Vec3::new(0.0, 0.02 * s.abs(), 0.0));
        self.grupos
            .iter()
            .map(|g| match *g {
                Grupo::Cuerpo | Grupo::Cabeza => raiz,
                Grupo::Brazo(lado) => {
                    let i = if lado > 0.0 { 0 } else { 1 };
                    raiz * alrededor(self.hombro[i], Mat4::rotation_z(lado * 0.12) * Mat4::rotation_x(0.5 * s * lado))
                }
                Grupo::Pierna(lado) => {
                    let i = if lado > 0.0 { 0 } else { 1 };
                    raiz * alrededor(self.cadera[i], Mat4::rotation_x(-0.5 * s * lado))
                }
                Grupo::Coleta(lado) => {
                    let i = if lado > 0.0 { 0 } else { 1 };
                    raiz * alrededor(self.coleta[i], Mat4::rotation_z(lado * 0.3))
                }
            })
            .collect()
    }

    /// Una matriz por parte. Pasa de `antes` a `ahora` con `mezcla` en [0, 1];
    /// con `saltando` en 1 los brazos quedan arriba (el brinco a la nave).
    pub fn matrices(&self, t: f32, antes: Paso, ahora: Paso, mezcla: f32, saltando: f32) -> Vec<Mat4> {
        let alrededor = |p: Vec3, r: Mat4| Mat4::translation(p) * r * Mat4::translation(-p);
        let fase = t * 2.0 * PI;
        let mut pose = Pose::mezclar(Self::pose(antes, t), Self::pose(ahora, t), mezcla);
        let arriba = Pose { der: (0.0, 2.8), izq: (0.0, 2.8), cabeza: 0.0, rebote: 0.0 };
        pose = Pose::mezclar(pose, arriba, saltando);

        let raiz = Mat4::translation(Vec3::new(0.0, 0.05 * (fase * 0.5).sin().abs() * pose.rebote, 0.0))
            * Mat4::rotation_y(0.3 * (fase * 0.125).sin() * pose.rebote)
            * Mat4::rotation_z(0.07 * (fase * 0.25).sin() * pose.rebote);
        let cabeza = raiz * alrededor(self.cuello, Mat4::rotation_z(pose.cabeza) * Mat4::rotation_x(0.06 * (fase * 0.5).sin()));

        self.grupos
            .iter()
            .map(|g| match *g {
                Grupo::Cuerpo => raiz,
                Grupo::Cabeza => cabeza,
                Grupo::Brazo(lado) => {
                    let (i, (frente, abierto)) = if lado > 0.0 { (0, pose.der) } else { (1, pose.izq) };
                    raiz * alrededor(self.hombro[i], Mat4::rotation_z(lado * abierto) * Mat4::rotation_x(frente))
                }
                Grupo::Pierna(lado) => {
                    let i = if lado > 0.0 { 0 } else { 1 };
                    raiz * alrededor(self.cadera[i], Mat4::rotation_x(lado * 0.3 * (fase * 0.5).sin() * pose.rebote))
                }
                Grupo::Coleta(lado) => {
                    let i = if lado > 0.0 { 0 } else { 1 };
                    let r = Mat4::rotation_z(lado * (0.25 + 0.2 * (fase * 0.5 + lado).sin())) * Mat4::rotation_x(-0.15 - 0.1 * (fase * 0.5).sin());
                    cabeza * alrededor(self.coleta[i], r)
                }
            })
            .collect()
    }
}
