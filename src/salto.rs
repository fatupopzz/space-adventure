//! El salto al hiperespacio: con espacio, la nave apunta al siguiente
//! planeta, las estrellas se estiran en rayas, el FOV se abre y aparece a un
//! lado del planeta con un destello.
//!
//! Tres fases:
//! 1. **Carga**: la nave gira hacia el destino y el FOV se cierra un poco,
//!    como tomando aire.
//! 2. **Viaje**: recorre el camino de golpe (suavizado al principio y al
//!    final), con el FOV muy abierto: es la etapa de proyeccion luciendose.
//! 3. **Salida**: destello blanco y las rayas se encogen hasta volver a ser
//!    estrellas.

use crate::escena::CUERPOS;
use crate::matematica::Vec3;
use crate::nave::Nave;

const CARGA: f32 = 0.7;
const VIAJE: f32 = 1.1;
const SALIDA: f32 = 0.8;
/// Cuantos grados se abre el FOV en el viaje.
const FOV_VIAJE: f32 = 32.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fase {
    Quieto,
    Carga,
    Viaje,
    Salida,
}

pub struct Salto {
    fase: Fase,
    /// Segundos dentro de la fase actual.
    t: f32,
    origen: Vec3,
    yaw0: f32,
    pitch0: f32,
    pub destino: usize,
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Yaw y pitch para que la nariz de la nave apunte en la direccion `d`.
pub fn rumbo_hacia(d: Vec3) -> (f32, f32) {
    let d = d.normalize();
    (f32::atan2(-d.x, -d.z), d.y.clamp(-1.0, 1.0).asin().clamp(-1.0, 1.0))
}

/// Interpola angulos por el camino corto.
fn hacia_angulo(a: f32, b: f32, k: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let d = (b - a).rem_euclid(tau);
    let d = if d > tau * 0.5 { d - tau } else { d };
    a + d * k
}

impl Salto {
    pub fn new() -> Self {
        Salto { fase: Fase::Quieto, t: 0.0, origen: Vec3::zero(), yaw0: 0.0, pitch0: 0.0, destino: 1 }
    }

    pub fn activo(&self) -> bool {
        self.fase != Fase::Quieto
    }

    /// Si ya se puede manejar la nave: quieto o en el destello de llegada.
    pub fn puede_maniobrar(&self) -> bool {
        matches!(self.fase, Fase::Quieto | Fase::Salida)
    }

    pub fn iniciar(&mut self, nave: &Nave, destino: usize) {
        if self.activo() {
            return;
        }
        *self = Salto { fase: Fase::Carga, t: 0.0, origen: nave.pos, yaw0: nave.yaw, pitch0: nave.pitch, destino };
    }

    /// Donde termina el salto: afuera del planeta, del lado por donde llega
    /// la nave y un poco arriba, para verlo de frente. Se recalcula cada
    /// cuadro porque el planeta sigue orbitando.
    fn llegada(&self, t_mundo: f32) -> Vec3 {
        let c = &CUERPOS[self.destino];
        let p = c.posicion(t_mundo);
        let desde = self.origen - p;
        let lado = Vec3::new(desde.x, 0.0, desde.z).normalize();
        let r = c.radio_visual();
        p + lado * (r * 2.2 + 5.0) + Vec3::new(0.0, r * 0.5 + 1.5, 0.0)
    }

    pub fn actualizar(&mut self, nave: &mut Nave, dt: f32, t_mundo: f32) {
        self.t += dt;
        match self.fase {
            Fase::Quieto => {}
            Fase::Carga => {
                let (yaw, pitch) = rumbo_hacia(self.llegada(t_mundo) - nave.pos);
                let k = suave(self.t / CARGA);
                nave.yaw = hacia_angulo(self.yaw0, yaw, k);
                nave.pitch = self.pitch0 + (pitch - self.pitch0) * k;
                nave.roll *= (-4.0 * dt).exp();
                nave.velocidad *= (-3.0 * dt).exp();
                nave.vel = nave.adelante() * nave.velocidad;
                nave.pos += nave.vel * dt;
                if self.t >= CARGA {
                    self.fase = Fase::Viaje;
                    self.t = 0.0;
                    self.origen = nave.pos;
                }
            }
            Fase::Viaje => {
                let llegada = self.llegada(t_mundo);
                let u = (self.t / VIAJE).min(1.0);
                // Arranca y frena suave: cubica de entrada y salida.
                let k = if u < 0.5 { 4.0 * u * u * u } else { 1.0 - (-2.0 * u + 2.0).powi(3) / 2.0 };
                let antes = nave.pos;
                nave.pos = self.origen.lerp(llegada, k);
                nave.vel = (nave.pos - antes) * (1.0 / dt.max(1e-4));
                (nave.yaw, nave.pitch) = rumbo_hacia(llegada - self.origen);
                if u >= 1.0 {
                    // Llega mirando al planeta.
                    (nave.yaw, nave.pitch) = rumbo_hacia(CUERPOS[self.destino].posicion(t_mundo) - nave.pos);
                    nave.velocidad = 3.0;
                    self.fase = Fase::Salida;
                    self.t = 0.0;
                }
            }
            Fase::Salida => {
                if self.t >= SALIDA {
                    self.fase = Fase::Quieto;
                }
            }
        }
    }

    /// Que tan estiradas van las estrellas, de 0 a 1.
    pub fn estiramiento(&self) -> f32 {
        match self.fase {
            Fase::Quieto => 0.0,
            Fase::Carga => 0.3 * suave(self.t / CARGA),
            Fase::Viaje => 0.3 + 0.7 * suave(self.t / 0.25),
            Fase::Salida => 1.0 - suave(self.t / SALIDA),
        }
    }

    /// Grados que se le suman al FOV.
    pub fn fov_extra(&self) -> f32 {
        match self.fase {
            Fase::Quieto => 0.0,
            Fase::Carga => -6.0 * suave(self.t / CARGA),
            Fase::Viaje => -6.0 + (FOV_VIAJE + 6.0) * suave(self.t / 0.3),
            Fase::Salida => FOV_VIAJE * (1.0 - suave(self.t / SALIDA)),
        }
    }

    /// El destello blanco: al arrancar el viaje y al llegar.
    pub fn destello(&self) -> f32 {
        match self.fase {
            Fase::Viaje => 0.5 * (1.0 - self.t / 0.2).max(0.0),
            Fase::Salida => 0.85 * (1.0 - self.t / 0.45).max(0.0),
            _ => 0.0,
        }
    }

    /// Que tanto empujan los motores por el salto (para las llamas).
    pub fn empuje(&self) -> f32 {
        match self.fase {
            Fase::Quieto => 0.0,
            Fase::Carga => suave(self.t / CARGA),
            Fase::Viaje => 1.4,
            Fase::Salida => 1.0 - suave(self.t / SALIDA),
        }
    }
}
