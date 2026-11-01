//! Lo que hay que recoger volando: notas musicales flotando entre las orbitas
//! (en arcos, para que pasar por ellos sea un caminito) y, cada tanto, un
//! cometa que cruza el sistema solar.
//!
//! Recoger notas seguidas hace COMBO: cada nota suena un tono mas arriba en la
//! escala, como en los juegos de plataformas. Las notas renacen despues de un
//! rato. El cometa trae la cola del lado contrario al Sol (como los de verdad)
//! y vale muchas notas si se alcanza.

use crate::escena::CUERPOS;
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;
use crate::sonido::Efecto;
use std::f32::consts::TAU;

/// Que tan cerca hay que pasar para recoger una nota o alcanzar el cometa.
pub const RADIO_NOTA: f32 = 3.2;
const RADIO_COMETA: f32 = 8.0;
/// Segundos que tarda una nota en volver a aparecer.
const RENACE: f32 = 60.0;
/// Segundos sin recoger antes de que se rompa el combo.
const COMBO: f32 = 2.5;
/// Cada cuanto llega un cometa, cuanto dura y que tan rapido va.
const CADA_COMETA: f32 = 80.0;
const VIDA_COMETA: f32 = 26.0;
const RAPIDEZ_COMETA: f32 = 32.0;
pub const NOTAS_COMETA: u32 = 25;
/// Desde que distancia se dibujan las notas.
pub const LEJOS_NOTAS: f32 = 140.0;

pub struct Nota {
    pub pos: Vec3,
    pub viva: bool,
    renace: f32,
    pub rosa: bool,
    /// Las que se cayeron al chocar con un asteroide rojo: viven unos segundos (y no renacen).
    pub fugaz: Option<f32>,
}

pub struct Cometa {
    pub pos: Vec3,
    vel: Vec3,
    vida: f32,
}

pub struct Coleccionables {
    pub notas: Vec<Nota>,
    pub combo: u32,
    desde_ultima: f32,
    pub cometa: Option<Cometa>,
    hasta_cometa: f32,
    semilla: u32,
    /// Lo que paso este cuadro: notas recogidas, si se atrapo un cometa, avisos para la IA.
    pub recogidas: u32,
    /// Donde se recogio cada nota este cuadro, si era rosa y en que numero del combo iba (para las
    /// chispas y el "+1"), y donde se atrapo el cometa.
    pub recogidas_en: Vec<(Vec3, bool, u32)>,
    pub cometa_en: Option<Vec3>,
    pub cometa_atrapado: bool,
    pub aviso: Option<&'static str>,
    pub eventos: Vec<Efecto>,
}

impl Coleccionables {
    pub fn new() -> Self {
        let mut notas = Vec::new();
        // Un anillo de arcos a la mitad entre cada par de orbitas vecinas (sin la Luna).
        let orbitas: Vec<f32> = CUERPOS.iter().filter(|c| c.padre.is_none() && c.orbita > 0.0).map(|c| c.orbita).collect();
        for (n, par) in orbitas.windows(2).enumerate() {
            let r = (par[0] + par[1]) * 0.5;
            let arcos = 8;
            for a in 0..arcos {
                let a0 = a as f32 / arcos as f32 * TAU + n as f32 * 0.7;
                for k in 0..6 {
                    // Las notas de un arco van separadas unos 4.5 de distancia, ondulando de arriba a abajo.
                    let ang = a0 + k as f32 * 4.5 / r;
                    let y = 3.0 * (k as f32 * 0.9 + a as f32).sin();
                    notas.push(Nota { pos: Vec3::new(r * ang.cos(), y, -r * ang.sin()), viva: true, renace: 0.0, rosa: (a + n) % 2 == 0, fugaz: None });
                }
            }
        }
        Coleccionables {
            notas,
            combo: 0,
            desde_ultima: 99.0,
            cometa: None,
            hasta_cometa: CADA_COMETA * 0.5,
            semilla: 12345,
            recogidas: 0,
            recogidas_en: Vec::new(),
            cometa_en: None,
            cometa_atrapado: false,
            aviso: None,
            eventos: Vec::new(),
        }
    }

    fn azar(&mut self) -> f32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Un cuadro: recoge lo que la nave toque (si `puede_recoger`), renacen notas y llega o se va el cometa.
    pub fn actualizar(&mut self, nave: Vec3, dt: f32, puede_recoger: bool) {
        self.recogidas = 0;
        self.recogidas_en.clear();
        self.cometa_en = None;
        self.cometa_atrapado = false;
        self.aviso = None;
        self.desde_ultima += dt;
        if self.desde_ultima > COMBO {
            self.combo = 0;
        }
        // Las que se cayeron se acaban.
        for n in &mut self.notas {
            if let Some(t) = &mut n.fugaz {
                *t -= dt;
            }
        }
        self.notas.retain(|n| n.fugaz.is_none_or(|t| t > 0.0 && n.viva));
        for i in 0..self.notas.len() {
            let n = &mut self.notas[i];
            if !n.viva {
                n.renace -= dt;
                n.viva = n.renace <= 0.0;
                continue;
            }
            if puede_recoger && (n.pos - nave).length() < RADIO_NOTA {
                n.viva = false;
                n.renace = RENACE;
                self.recogidas += 1;
                self.combo += 1;
                self.desde_ultima = 0.0;
                self.recogidas_en.push((n.pos, n.rosa, self.combo));
                // Cada nota del combo suena un tono mas arriba (y vuelve a empezar despues de la octava).
                self.eventos.push(NOTAS[(self.combo as usize - 1) % NOTAS.len()]);
            }
        }
        // El cometa.
        if let Some(c) = &mut self.cometa {
            c.pos = c.pos + c.vel * dt;
            c.vida -= dt;
            if puede_recoger && (c.pos - nave).length() < RADIO_COMETA {
                self.cometa_en = Some(c.pos);
                self.cometa = None;
                self.cometa_atrapado = true;
                self.recogidas += NOTAS_COMETA;
                self.eventos.push(Efecto::Perfecto);
                self.aviso = Some("¡Cometa atrapado! Veinticinco notas. Y un poco de hielo en el parabrisas.");
            } else if c.vida <= 0.0 {
                self.cometa = None;
                self.aviso = Some("El cometa se fue. Vuelve en un rato. Los cometas son puntuales, no como usted.");
            }
        } else {
            self.hasta_cometa -= dt;
            if self.hasta_cometa <= 0.0 && puede_recoger {
                self.hasta_cometa = CADA_COMETA;
                // Pasa a unos 50 de la nave, cruzando de lado a lado.
                let a = self.azar() * TAU;
                let dir = Vec3::new(a.cos(), 0.1 * (self.azar() - 0.5), -a.sin()).normalize();
                let lado = Vec3::new(-dir.z, 0.0, dir.x);
                let cerca = nave + lado * (35.0 + 30.0 * self.azar()) + Vec3::new(0.0, 6.0 * (self.azar() - 0.5), 0.0);
                self.cometa = Some(Cometa { pos: cerca - dir * (RAPIDEZ_COMETA * VIDA_COMETA * 0.4), vel: dir * RAPIDEZ_COMETA, vida: VIDA_COMETA });
                self.eventos.push(Efecto::Cometa);
                self.aviso = Some("¡Un cometa! Pasa cerca. Alcáncelo antes de que se vaya: vale veinticinco notas.");
            }
        }
    }

    /// Que el cometa llegue ya (lo llama Calisto, desde la mochila).
    pub fn llamar_cometa(&mut self) {
        if self.cometa.is_none() {
            self.hasta_cometa = 0.0;
        }
    }

    /// Riega `n` notas alrededor de `donde` (se le cayeron a la nave): duran unos segundos y se
    /// pueden volver a recoger.
    pub fn soltar(&mut self, donde: Vec3, n: u32) {
        for k in 0..n {
            let a = k as f32 / n.max(1) as f32 * TAU + self.azar();
            let r = 5.0 + 6.0 * self.azar();
            let pos = donde + Vec3::new(a.cos() * r, (self.azar() - 0.5) * 4.0, a.sin() * r);
            self.notas.push(Nota { pos, viva: true, renace: 0.0, rosa: k % 2 == 0, fugaz: Some(4.5) });
        }
    }

    /// Las notas vivas cerca de `ojo`, con su matriz (girando y flotando) y su color.
    pub fn visibles(&self, ojo: Vec3, t: f32) -> Vec<(Mat4, bool)> {
        self.notas
            .iter()
            .enumerate()
            // Las que se cayeron parpadean cuando ya se van a acabar.
            .filter(|(_, n)| n.viva && (n.pos - ojo).length() < LEJOS_NOTAS && n.fugaz.is_none_or(|f| f > 1.5 || (t * 10.0) as i32 % 2 == 0))
            .map(|(i, n)| {
                let flota = 0.4 * (t * 1.7 + i as f32).sin();
                (Mat4::translation(n.pos + Vec3::new(0.0, flota, 0.0)) * Mat4::rotation_y(t * 1.5 + i as f32) * Mat4::scale(1.2), n.rosa)
            })
            .collect()
    }

    /// El resplandor de las notas (para verlas de lejos) y el cometa con su cola.
    pub fn dibujar_luces(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4, ojo: Vec3, t: f32) {
        for n in self.notas.iter().filter(|n| n.viva) {
            let d = (n.pos - ojo).length();
            if d > LEJOS_NOTAS * 1.6 {
                continue;
            }
            if let Some((x, y, _)) = proyectar(n.pos, vista_proyeccion, fb) {
                let k = (1.0 - d / (LEJOS_NOTAS * 1.6)).clamp(0.0, 1.0) * (0.75 + 0.25 * (t * 3.0 + n.pos.x).sin());
                let c = if n.rosa { [0.5, 0.15, 0.35] } else { [0.12, 0.4, 0.5] };
                crate::post::halo(fb, (x, y), 10.0 + 900.0 / d.max(8.0), c.map(|v| v * k), 1.8);
            }
        }
        if let Some(c) = &self.cometa {
            // La cola apunta lejos del Sol (el viento solar la empuja), no hacia atras del cometa.
            let afuera = (c.pos - CUERPOS[0].posicion(0.0)).normalize();
            for k in 0..30 {
                let u = k as f32 / 30.0;
                let abre = 2.5 * u * (t * 3.0 + k as f32).sin();
                let p0 = c.pos + afuera * (u * 30.0) + Vec3::new(0.0, abre, 0.0);
                let p1 = c.pos + afuera * ((u + 0.04) * 30.0) + Vec3::new(0.0, abre, 0.0);
                if let (Some(a), Some(b)) = (proyectar(p0, vista_proyeccion, fb), proyectar(p1, vista_proyeccion, fb)) {
                    let b_ = 0.9 * (1.0 - u);
                    // La cola se abre: tres hilos de luz que se separan hacia la punta.
                    for (dy, k) in [(0.0, 1.0), (2.0 + 6.0 * u, 0.6), (-2.0 - 6.0 * u, 0.6)] {
                        fb.linea_de_luz((a.0, a.1 + dy, a.2), (b.0, b.1 + dy, b.2), [0.5 * b_ * k, 0.85 * b_ * k, b_ * k], [0.4 * b_ * k, 0.7 * b_ * k, b_ * k]);
                    }
                }
            }
            if let Some((x, y, _)) = proyectar(c.pos, vista_proyeccion, fb) {
                let d = (c.pos - ojo).length();
                crate::post::halo(fb, (x, y), 30.0 + 3600.0 / d.max(10.0), [0.7, 0.9, 1.0], 1.4);
                crate::post::halo(fb, (x, y), 10.0 + 600.0 / d.max(10.0), [1.0, 1.0, 1.0], 1.0);
            }
        }
    }
}

/// La escala del combo: pentatonica de do, subiendo.
const NOTAS: [Efecto; 8] = [Efecto::Nota0, Efecto::Nota1, Efecto::Nota2, Efecto::Nota3, Efecto::Nota4, Efecto::Nota5, Efecto::Nota6, Efecto::Nota7];

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Pasar por una nota la recoge y empieza el combo; la nota renace despues.
    #[test]
    fn pasar_por_una_nota_la_recoge() {
        let mut c = Coleccionables::new();
        let total = c.notas.len();
        assert!(total > 100);
        let p = c.notas[0].pos;
        c.actualizar(p, 0.1, true);
        assert_eq!(c.recogidas, 1);
        assert_eq!(c.combo, 1);
        assert!(!c.notas[0].viva);
        c.actualizar(p, RENACE + 1.0, true);
        assert!(c.notas[0].viva, "renace");
    }

    /// Recoger un arco de seguido sube el combo; si se tarda, se rompe.
    #[test]
    fn el_combo_sube_y_se_rompe() {
        let mut c = Coleccionables::new();
        for k in 0..6 {
            let p = c.notas[k].pos;
            c.actualizar(p, 0.3, true);
        }
        assert_eq!(c.combo, 6);
        c.actualizar(Vec3::new(0.0, 500.0, 0.0), COMBO + 0.1, true);
        assert_eq!(c.combo, 0);
    }

    /// Llega un cometa, y alcanzarlo da notas.
    #[test]
    fn el_cometa_llega_y_se_atrapa() {
        let mut c = Coleccionables::new();
        let lejos = Vec3::new(0.0, 900.0, 0.0);
        c.actualizar(lejos, CADA_COMETA, true);
        assert!(c.cometa.is_some());
        let pos = c.cometa.as_ref().unwrap().pos;
        c.actualizar(pos, 0.01, true);
        assert!(c.cometa_atrapado);
        assert_eq!(c.recogidas, NOTAS_COMETA);
    }
}
