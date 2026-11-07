//! Aterrizaje en Saturno: Saturno Ritmo en vivo.
//!
//! Miku canta sentada en una luna que se mece sobre el escenario hexagonal
//! del polo norte. Seis reflectores barren el escenario con conos de luz
//! transparentes, el ecualizador late, los anillos giran como un disco y
//! caen lucecitas. Todo va a 120 BPM. La camara cambia de toma cada pocos
//! segundos, como en un concierto televisado.
//!
//! Todo esta en las coordenadas de Saturno (radio 2, escenario en y = 2.1):
//! la escena se dibuja con Saturno sin escalar ni inclinar.

use crate::entrada::Entrada;
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;
use crate::ui;
use raylib::prelude::*;
use std::f32::consts::{PI, TAU};

const BPM: f32 = 120.0;
/// De donde cuelga la luna (su pivote) y donde va Miku sobre ella.
const PIVOTE_LUNA: Vec3 = Vec3::new(0.0, 4.25, 0.0);
const ESCALA_MIKU: f32 = 0.42;
/// Donde apuntan los reflectores: a Miku.
const CENTRO: Vec3 = Vec3::new(0.0, 2.62, 0.0);
const SEGUNDOS_POR_TOMA: f32 = 6.0;

const COLORES: [&str; 4] = ["cian", "rosa", "amarillo", "lima"];
/// Para cada color, las otras tres partes de `haz_luz.obj`, que se ocultan.
pub const OCULTAR_HAZ: [[&str; 3]; 4] = [
    ["haz_rosa", "haz_amarillo", "haz_lima"],
    ["haz_cian", "haz_amarillo", "haz_lima"],
    ["haz_cian", "haz_rosa", "haz_lima"],
    ["haz_cian", "haz_rosa", "haz_amarillo"],
];

const COMENTARIOS: [&str; 5] = [
    "Bienvenidos a Saturno Ritmo. Miku en vivo. Sin playback, supuestamente. Yo no metería las manos al fuego.",
    "Los anillos son un vinil de 280,000 km. El DJ cobra por hora de Saturno: diez horas y media. Ni modo.",
    "Saturno flotaría en una piscina. El público también, si hubiera. Son doce alienígenas y vinieron por la refa.",
    "El ecualizador sigue el ritmo. Venus le manda los anillos de vuelta. {aceptar}: esquivarlos.",
    "La tormenta del polo es hexagonal de verdad. Producción barata, idea cara. Como todo proyecto final.",
];

/// Una toma: la camara va de `a` a `b` mientras dura (ojo, punto al que mira).
const TOMAS: [((Vec3, Vec3), (Vec3, Vec3)); 4] = [
    // De frente, acercandose.
    ((Vec3::new(0.0, 3.45, -3.0), Vec3::new(0.0, 2.6, 0.0)), (Vec3::new(0.0, 3.15, -2.3), Vec3::new(0.0, 2.62, 0.0))),
    // De lado, entre los reflectores.
    ((Vec3::new(2.0, 2.95, -1.6), Vec3::new(0.0, 2.7, 0.0)), (Vec3::new(1.4, 2.85, -2.0), Vec3::new(0.0, 2.7, 0.0))),
    // Primer plano de Miku.
    ((Vec3::new(0.35, 2.78, -1.05), Vec3::new(0.0, 2.62, 0.0)), (Vec3::new(-0.3, 2.75, -1.0), Vec3::new(0.0, 2.62, 0.0))),
    // Lejos: el planeta, los anillos y el escenario arriba.
    ((Vec3::new(-6.5, 4.5, -5.0), Vec3::new(0.0, 2.0, 0.0)), (Vec3::new(-4.0, 5.0, -7.0), Vec3::new(0.0, 2.1, 0.0))),
];

pub enum AccionConcierto {
    Despegar,
    Jugar,
}

pub struct Concierto {
    t: f32,
    /// Lucecitas: posicion inicial y color.
    chispas: Vec<(Vec3, [f32; 3])>,
}

impl Concierto {
    pub fn new() -> Self {
        let mut s: u32 = 0xC0FF_EE11;
        let mut azar = move || {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (s >> 8) as f32 / (1u32 << 24) as f32
        };
        let paleta = [[0.3, 0.95, 1.0], [1.0, 0.45, 0.8], [1.0, 0.85, 0.35], [0.7, 1.0, 0.45]];
        let chispas = (0..170)
            .map(|_| {
                let (a, r) = (azar() * TAU, 0.2 + azar() * 2.2);
                (Vec3::new(r * a.cos(), azar() * 3.0, r * a.sin()), paleta[(azar() * 4.0) as usize % 4])
            })
            .collect();
        Concierto { t: 0.0, chispas }
    }

    /// Avanza. Devuelve que se pidio: despegar o jugar a ¡Anillazo!.
    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> Option<AccionConcierto> {
        self.t += dt;
        if e.aterrizar {
            Some(AccionConcierto::Despegar)
        } else if e.aceptar {
            Some(AccionConcierto::Jugar)
        } else {
            None
        }
    }

    /// El comentario de la IA que toca ahora, si toca.
    pub fn comentario(&self) -> &'static str {
        let i = (self.t / 9.0) as usize % COMENTARIOS.len();
        if (self.t % 9.0) < 6.0 { COMENTARIOS[i] } else { "" }
    }

    /// El golpe del ritmo: 1 justo en el beat y se apaga hasta el siguiente.
    pub fn ritmo(&self) -> f32 {
        (-(self.t * BPM / 60.0).fract() * 5.0).exp()
    }

    pub fn t(&self) -> f32 {
        self.t
    }

    pub fn camara(&self) -> (Vec3, Vec3) {
        let n = (self.t / SEGUNDOS_POR_TOMA) as usize;
        let (a, b) = TOMAS[n % TOMAS.len()];
        let k = (self.t / SEGUNDOS_POR_TOMA).fract();
        (a.0.lerp(b.0, k), a.1.lerp(b.1, k))
    }

    /// El vaiven de la luna (radianes), al compas.
    pub fn vaiven(&self) -> f32 {
        vaiven_en(self.t)
    }

    pub fn columpio(&self) -> Mat4 {
        columpio_en(self.t)
    }

    /// Miku sobre la luna. `asiento` sale de `luna_columpio.obj`.
    pub fn miku(&self, asiento: Vec3) -> Mat4 {
        miku_en(self.t, asiento)
    }

    pub fn expresion_miku(&self) -> &'static str {
        // Canta feliz y en cada cuarto compas se enamora del publico.
        if ((self.t * BPM / 60.0) / 4.0) as i32 % 4 == 3 { "enamorada" } else { "feliz" }
    }

    pub fn haces(&self) -> Vec<(Mat4, usize)> {
        haces_en(self.t)
    }

    /// Las lucecitas caen despacio dando vueltas; al llegar abajo vuelven a salir arriba.
    pub fn dibujar_chispas(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4) {
        for (k, &(p, color)) in self.chispas.iter().enumerate() {
            let caida = (p.y - self.t * 0.35).rem_euclid(3.0);
            let giro = self.t * 0.2;
            let (s, c) = giro.sin_cos();
            let pos = Vec3::new(p.x * c - p.z * s, 2.1 + caida, p.x * s + p.z * c);
            let brillo = 0.5 + 0.5 * (self.t * 4.0 + k as f32).sin();
            let col = color.map(|v| v * brillo * (0.6 + 0.6 * self.ritmo()));
            if let (Some(a), Some(b)) = (proyectar(pos, vista_proyeccion, fb), proyectar(pos + Vec3::new(0.0, 0.03, 0.0), vista_proyeccion, fb)) {
                fb.linea_de_luz(a, b, col, col);
                fb.linea_de_luz((a.0 - 1.0, a.1, a.2), (a.0 + 1.0, a.1, a.2), col, col);
            }
        }
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, ia: &str, ancho: i32, alto: i32) {
        ui::panel(d, 20, 20, 560, 74);
        ui::texto(d, "SATURNO RITMO", 36, 30, 28, ui::TEXTO);
        let vivo = if (self.t * 2.0) as i32 % 2 == 0 { ui::ROJO } else { ui::TENUE };
        ui::texto(d, "EN VIVO", 36, 64, 18, vivo);
        ui::texto(d, &format!("{}: jugar     {}: despegar", ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("aterrizar"))), 260, 64, 18, ui::LILA);
        if !ia.is_empty() {
            ui::mensaje_ia(d, ia, ancho, alto);
        }
    }
}

/// El vaiven de la luna (radianes) en el segundo `t`, al compas.
pub fn vaiven_en(t: f32) -> f32 {
    0.12 * (t * BPM / 60.0 * PI * 0.5).sin()
}

/// La matriz de la luna: cuelga del pivote y se mece hacia adelante y atras.
pub fn columpio_en(t: f32) -> Mat4 {
    Mat4::translation(PIVOTE_LUNA) * Mat4::rotation_x(vaiven_en(t)) * Mat4::rotation_z(0.04 * (t * 0.9).sin())
}

/// Miku sentada en la luna.
pub fn miku_en(t: f32, asiento: Vec3) -> Mat4 {
    columpio_en(t) * Mat4::translation(asiento) * Mat4::scale(ESCALA_MIKU)
}

/// Los seis haces de los reflectores: su matriz (el cono apunta a donde barre) y su color.
pub fn haces_en(t: f32) -> Vec<(Mat4, usize)> {
    (0..6)
        .map(|n| {
            let a = n as f32 * PI / 3.0;
            let lampara = Vec3::new(0.72 * a.cos(), 2.575, 0.72 * a.sin());
            // Barren de un lado a otro alrededor de Miku, cada uno a su paso.
            let barrido = Vec3::new(0.5 * (t * 0.9 + n as f32).sin(), 0.35 * (t * 1.3 + n as f32 * 2.0).sin(), 0.5 * (t * 0.7 + n as f32 * 1.7).cos());
            let y = (CENTRO + barrido + Vec3::new(0.0, 0.6, 0.0) - lampara).normalize();
            let x = y.cross(Vec3::new(0.0, 0.0, 1.0)).normalize();
            let z = x.cross(y);
            let m = Mat4::translation(lampara) * Mat4::from_basis(x, y, z) * Mat4::scale_xyz(Vec3::new(1.3, 2.6, 1.3));
            (m, (n + ((t * BPM / 60.0 / 4.0) as usize)) % COLORES.len())
        })
        .collect()
}
