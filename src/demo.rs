//! `--demo`: el recorrido que pide la entrega, con piloto automatico (para grabar el video sin
//! manos): vuelo espacial hacia el Planeta Chapin, acercamiento, una vuelta completa en modo orbita
//! (subiendo y bajando un poco a media vuelta) y alejamiento. Cada parte sale rotulada arriba.
//! El piloto automatico solo aprieta "botones" (llena una `Entrada`), como si jugara alguien: la
//! nave, la gravedad y el modo orbita son los mismos del juego. Con cualquier tecla se toma el
//! control.

use crate::entrada::Entrada;
use crate::escena::{CHAPIN, CUERPOS};
use crate::matematica::Vec3;
use crate::nave::Nave;
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fase {
    Vuelo,
    Acercamiento,
    Vuelta,
    Alejamiento,
    Fin,
}

pub struct Demo {
    pub fase: Fase,
    /// Segundos en la fase.
    t: f32,
    /// Cuanto se ha dado la vuelta (radianes alrededor del planeta) y por donde iba.
    vuelta: f32,
    antes: Option<Vec3>,
}

/// Desde que tan lejos de la superficie arranca (y desde donde empieza a frenar).
const DISTANCIA_INICIAL: f32 = 300.0;
const EMPIEZA_A_FRENAR: f32 = 70.0;

/// Donde arranca la nave: lejos del Chapin, en el tiempo `t`, mirando hacia el. Llega por el
/// lado de dia (de tres cuartos, con el Sol a un lado), para que se vea con luz.
pub fn inicio(t: f32) -> (Vec3, f32) {
    let c = &CUERPOS[CHAPIN];
    let centro = c.posicion(t);
    let pos = centro + lado_de_dia(t) * (c.radio_cuerpo() + DISTANCIA_INICIAL);
    (pos, crate::salto::rumbo_hacia(centro - pos).0)
}

/// Desde el Chapin, hacia donde hay que ponerse para verlo de dia y de tres cuartos.
pub fn lado_de_dia(t: f32) -> Vec3 {
    let centro = CUERPOS[CHAPIN].posicion(t);
    let hacia_sol = (CUERPOS[0].posicion(t) - centro).normalize();
    let lado = hacia_sol.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
    (hacia_sol * 0.8 + lado * 0.55 + Vec3::new(0.0, 0.2, 0.0)).normalize()
}

/// El angulo que hay que girar para ir de `a` a `b`, por el camino corto.
fn diferencia(a: f32, b: f32) -> f32 {
    (b - a + PI).rem_euclid(TAU) - PI
}

/// Acelerar o frenar para ir a `objetivo` unidades por segundo.
fn velocidad_hacia(nave: &Nave, objetivo: f32) -> f32 {
    ((objetivo - nave.velocidad) * 0.6).clamp(-1.0, 1.0)
}

/// Los "botones" para apuntar la nariz hacia `direccion` (como lo haria alguien con el stick).
fn apuntar(nave: &Nave, direccion: Vec3) -> (f32, f32) {
    let (yaw, pitch) = crate::salto::rumbo_hacia(direccion);
    ((diferencia(nave.yaw, yaw) * 2.5).clamp(-1.0, 1.0), ((pitch - nave.pitch) * 2.5).clamp(-1.0, 1.0))
}

impl Default for Demo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo {
    pub fn new() -> Self {
        Demo { fase: Fase::Vuelo, t: 0.0, vuelta: 0.0, antes: None }
    }

    fn cambiar(&mut self, fase: Fase) {
        self.fase = fase;
        self.t = 0.0;
    }

    /// Lo que aprieta el piloto automatico este cuadro. `orbita`: si la nave va en modo orbita.
    pub fn entrada(&mut self, nave: &Nave, en_orbita: bool, t_mundo: f32, dt: f32) -> Entrada {
        self.t += dt;
        let c = &CUERPOS[CHAPIN];
        let centro = c.posicion(t_mundo);
        let hacia = centro - nave.pos;
        let altura = hacia.length() - c.radio_cuerpo();
        let mut e = Entrada::default();
        match self.fase {
            Fase::Vuelo => {
                (e.girar, e.cabecear) = apuntar(nave, hacia);
                // A velocidad de crucero (con turbo llegaria en un parpadeo y no se veria el viaje).
                e.empuje = velocidad_hacia(nave, 38.0);
                if altura < EMPIEZA_A_FRENAR {
                    self.cambiar(Fase::Acercamiento);
                }
            }
            Fase::Acercamiento => {
                // Apunta un poco al lado del centro: llega rozando, listo para dar la vuelta.
                let lado = hacia.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
                (e.girar, e.cabecear) = apuntar(nave, hacia.normalize() + lado * 0.45);
                // Frena poco a poco: mas despacio entre mas cerca (para que el acercamiento se vea y
                // la vuelta no se acabe en un suspiro).
                e.empuje = velocidad_hacia(nave, (altura * 0.35).clamp(4.0, 30.0));
                if altura < 4.5 && nave.velocidad < 6.0 {
                    e.orbitar = true;
                    self.cambiar(Fase::Vuelta);
                }
            }
            Fase::Vuelta => {
                if !en_orbita && self.t > 0.5 {
                    // Por si no alcanzo a entrar (otra vez).
                    e.orbitar = true;
                }
                e.empuje = velocidad_hacia(nave, 5.0);
                // Cuanto se ha avanzado alrededor del planeta.
                let arriba = (nave.pos - centro).normalize();
                if let Some(a) = self.antes {
                    self.vuelta += a.dot(arriba).clamp(-1.0, 1.0).acos();
                }
                self.antes = Some(arriba);
                let k = self.vuelta / TAU;
                // A media vuelta sube un poco y luego baja (el control de la altura en orbita).
                e.cabecear = if (0.3..0.42).contains(&k) { 0.7 } else if (0.55..0.67).contains(&k) { -0.7 } else { 0.0 };
                if k >= 1.0 {
                    e.orbitar = true;
                    self.cambiar(Fase::Alejamiento);
                }
            }
            Fase::Alejamiento => {
                let afuera = (nave.pos - centro).normalize();
                (e.girar, e.cabecear) = apuntar(nave, afuera + Vec3::new(0.0, 0.25, 0.0));
                // A velocidad de crucero, sin turbo: con turbo se saldria del sistema solar (y el
                // rescate la regresaria de golpe, en pleno video).
                e.empuje = velocidad_hacia(nave, 35.0);
                if self.t > 9.0 {
                    self.cambiar(Fase::Fin);
                }
            }
            Fase::Fin => {}
        }
        e
    }

    /// La camara de cine de la vuelta: afuera de la orbita y un poco atras, viendo hacia un punto
    /// entre la nave y el planeta (la camara de siempre va detras de la nave, y en orbita el
    /// planeta queda abajo, fuera de cuadro). `None` fuera de la vuelta: la de siempre.
    pub fn camara(&self, nave: &Nave, t_mundo: f32) -> Option<(Vec3, Vec3)> {
        if self.fase != Fase::Vuelta {
            return None;
        }
        let centro = CUERPOS[CHAPIN].posicion(t_mundo);
        let afuera = (nave.pos - centro).normalize();
        let ojo = nave.pos + afuera * 6.0 - nave.adelante() * 6.0 + Vec3::new(0.0, 1.5, 0.0);
        Some((ojo, nave.pos.lerp(centro, 0.4)))
    }

    /// Lleva la camara del juego hacia la de cine (suave, para entrar y salir sin cortes).
    /// `antes` es como estaba antes de que el juego la moviera este cuadro.
    pub fn mover_camara(&self, camara: &mut crate::nave::Camara, antes: (Vec3, Vec3), nave: &Nave, t_mundo: f32, dt: f32) {
        if let Some((ojo, mira)) = self.camara(nave, t_mundo) {
            let k = 1.0 - (-2.0 * dt).exp();
            camara.ojo = antes.0.lerp(ojo, k);
            camara.mira = antes.1.lerp(mira, k);
        }
    }

    /// El rotulo de la parte del recorrido que va.
    pub fn rotulo(&self) -> Option<(&'static str, &'static str)> {
        match self.fase {
            Fase::Vuelo => Some(("1 · VUELO ESPACIAL", "rumbo al Planeta Chapín")),
            Fase::Acercamiento => Some(("2 · ACERCAMIENTO", "sin pantallas de carga: el mismo mundo")),
            Fase::Vuelta => Some(("3 · VUELTA AL PLANETA", "modo órbita: la misma altura sobre la curva")),
            Fase::Alejamiento => Some(("4 · ALEJAMIENTO", "y de vuelta al sistema solar")),
            Fase::Fin => None,
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// El recorrido completo se hace solo: llega, da la vuelta entera y se aleja.
    #[test]
    fn el_recorrido_de_la_demo_se_completa() {
        let mut demo = Demo::new();
        let (pos, yaw) = inicio(0.0);
        let mut nave = Nave::new(pos);
        nave.yaw = yaw;
        // Una simulacion chiquita: el piloto, la nave y el modo orbita (como en `Juego`).
        let c = &CUERPOS[CHAPIN];
        let mut orbita: Option<f32> = None;
        let mut minimo = f32::MAX;
        let mut fases = vec![demo.fase];
        let dt = 1.0 / 60.0;
        for k in 0..(60 * 180) {
            let t = k as f32 * dt;
            let e = demo.entrada(&nave, orbita.is_some(), t, dt);
            let centro = c.posicion(t);
            let distancia = (nave.pos - centro).length() - c.radio_cuerpo();
            nave.actualizar(&e, dt, crate::nave::limite_de_velocidad(distancia));
            // El planeta se mueve en su orbita: la nave lo acompana (como la sincronia de `Juego`).
            nave.pos += centro - c.posicion(t - dt);
            if e.orbitar {
                orbita = match orbita {
                    Some(_) => None,
                    None => Some(distancia.max(1.5)),
                };
            }
            if let Some(altura) = orbita {
                let arriba = (nave.pos - centro).normalize();
                nave.pos = centro + arriba * (c.radio_cuerpo() + altura);
                let adelante = nave.adelante();
                (nave.yaw, nave.pitch) = crate::salto::rumbo_hacia(adelante - arriba * adelante.dot(arriba));
            }
            minimo = minimo.min((nave.pos - centro).length() - c.radio_cuerpo());
            if fases.last() != Some(&demo.fase) {
                fases.push(demo.fase);
            }
            if demo.fase == Fase::Fin {
                break;
            }
        }
        assert_eq!(fases, vec![Fase::Vuelo, Fase::Acercamiento, Fase::Vuelta, Fase::Alejamiento, Fase::Fin], "se quedo en {:?}", demo.fase);
        assert!(minimo < 6.0, "nunca se acerco: {minimo}");
    }
}
