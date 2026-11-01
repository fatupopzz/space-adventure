//! La Nave Lila: como se mueve con el teclado y donde la pone su matriz de
//! modelo.
//!
//! El modelo mira hacia -Z. El giro a los lados (yaw) es alrededor de Y, el
//! cabeceo (pitch) alrededor de X, y al dar vuelta se ladea un poco (roll)
//! para que se sienta el giro.
//!
//! El manejo toma ideas de juegos de naves:
//! - **Crucero y mass lock** (No Man's Sky, Elite): la velocidad maxima
//!   depende de que tan lejos esta la superficie mas cercana. En el vacio la
//!   nave puede ir muy rapido; cerca de un planeta, su "gravedad" la frena
//!   sola para que se pueda maniobrar y aterrizar.
//! - **Turbo con barra** (Star Fox): el turbo gasta energia y se recarga.
//! - **Barrel roll** (Star Fox): una vuelta completa sobre su eje con un
//!   brinquito de lado.
//! - **Sincronizar orbita** (Outer Wilds): cerca de un planeta, la nave se
//!   mueve con el en su orbita (eso lo hace `Juego`, que sabe de planetas).
//!
//! Para que no se sienta tosca (ver "Lerp smoothing is broken" de Freya Holmér y las recetas de
//! naves arcade de Godot):
//! - El giro no arranca ni para de golpe: la velocidad de giro persigue a la que pide el control
//!   con un suavizado exponencial (`1 - e^(-k dt)`, igual a cualquier framerate). Responde en una
//!   decima de segundo, pero sin tirones con el teclado (que solo sabe 0 o 1).
//! - El raton no pide velocidad de giro: mueve la nariz directo (como en un shooter), asi no
//!   depende de cuantos cuadros por segundo haya.
//! - El ladeo sale de que tan rapido esta girando de verdad (incluido el raton), y la camara se
//!   ladea un poquito con la nave y se adelanta hacia donde se esta girando.

use crate::entrada::Entrada;
use crate::matematica::{Mat4, Vec3};

/// La nave mide unos 2.4 de ancho en su `.obj`: con 0.5 queda de 1.2, chiquita
/// junto a los planetas (el sistema esta agrandado, ver `ESCALA_SISTEMA`).
const ESCALA: f32 = 0.5;
const ACELERACION: f32 = 14.0;
/// La velocidad maxima en crucero, lejos de todo.
pub const VELOCIDAD_CRUCERO: f32 = 150.0;
/// La velocidad maxima pegada a un planeta (mass lock).
const VELOCIDAD_CERCA: f32 = 9.0;
const TURBO: f32 = 1.8;
/// Energia del turbo que se gasta y se recarga por segundo (la barra va de 0 a 1).
const GASTO_TURBO: f32 = 0.35;
const RECARGA_TURBO: f32 = 0.2;
/// Segundos que dura un barrel roll.
const BARREL: f32 = 0.6;
/// Que tanto frena sola cuando no se acelera (por segundo).
const FRENADO: f32 = 0.8;
/// Radianes por segundo.
const GIRO: f32 = 1.75;
const CABECEO_MAXIMO: f32 = 1.1;
/// Que tan rapido llega la velocidad de giro a la pedida (por segundo): ~0.1 s.
const RESPUESTA_GIRO: f32 = 10.0;
/// Cuanto se ladea (radianes) girando a fondo.
pub const LADEO: f32 = 0.55;

pub struct Nave {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub velocidad: f32,
    /// Velocidad como vector (para el polvo y la camara).
    pub vel: Vec3,
    /// La barra del turbo, de 0 a 1.
    pub energia: f32,
    /// Segundos que le quedan al barrel roll (0: no esta girando) y hacia que lado.
    barrel: f32,
    lado_barrel: f32,
    /// Las velocidades de giro (radianes por segundo) que lleva ahorita.
    vel_yaw: f32,
    vel_pitch: f32,
    /// Que tan rapido giro de verdad el ultimo cuadro (de -1 a 1, con el raton incluido), suavizado.
    pub viraje: f32,
}

/// Que tan rapido se puede ir a `distancia` de la superficie mas cercana:
/// crece con la distancia, de `VELOCIDAD_CERCA` hasta `VELOCIDAD_CRUCERO`.
pub fn limite_de_velocidad(distancia: f32) -> f32 {
    (VELOCIDAD_CERCA + distancia.max(0.0) * 0.9).min(VELOCIDAD_CRUCERO)
}

impl Nave {
    pub fn new(pos: Vec3) -> Self {
        Nave { pos, yaw: 0.0, pitch: 0.0, roll: 0.0, velocidad: 0.0, vel: Vec3::zero(), energia: 1.0, barrel: 0.0, lado_barrel: 1.0, vel_yaw: 0.0, vel_pitch: 0.0, viraje: 0.0 }
    }

    /// Hacia donde apunta la nariz: el -Z del modelo despues de cabecear y girar.
    pub fn adelante(&self) -> Vec3 {
        Vec3::new(-self.pitch.cos() * self.yaw.sin(), self.pitch.sin(), -self.pitch.cos() * self.yaw.cos())
    }

    /// Avanza `dt` segundos. `limite` es la velocidad maxima ahi (ver `limite_de_velocidad`).
    /// Devuelve si empezo un barrel roll.
    pub fn actualizar(&mut self, e: &Entrada, dt: f32, limite: f32) -> bool {
        // El giro: la velocidad persigue a la pedida (suave, pero en una decima de segundo).
        let k = 1.0 - (-RESPUESTA_GIRO * dt).exp();
        self.vel_yaw += (e.girar * GIRO - self.vel_yaw) * k;
        self.vel_pitch += (e.cabecear * GIRO * 0.75 - self.vel_pitch) * k;
        // El raton mueve la nariz directo.
        let (raton_yaw, raton_pitch) = e.raton;
        self.yaw += self.vel_yaw * dt + raton_yaw;
        let pitch_antes = self.pitch;
        self.pitch = (self.pitch + self.vel_pitch * dt + raton_pitch).clamp(-CABECEO_MAXIMO, CABECEO_MAXIMO);
        if self.pitch != pitch_antes + self.vel_pitch * dt + raton_pitch {
            // Topo con el limite del cabeceo: no se queda "cargado".
            self.vel_pitch = 0.0;
        }
        // El ladeo sale de que tan rapido se esta girando de verdad (con el raton incluido).
        let giro_real = if dt > 0.0 { (self.vel_yaw + raton_yaw / dt) / GIRO } else { 0.0 };
        self.viraje += (giro_real.clamp(-1.0, 1.0) - self.viraje) * (1.0 - (-8.0 * dt).exp());
        self.roll += (self.viraje * LADEO - self.roll) * (1.0 - (-7.0 * dt).exp());

        // Turbo: solo si queda energia; si no se usa, se recarga.
        let turbo_activo = e.turbo && self.energia > 0.0 && e.empuje > 0.0;
        self.energia = if turbo_activo { self.energia - GASTO_TURBO * dt } else { self.energia + RECARGA_TURBO * dt }.clamp(0.0, 1.0);
        let turbo = if turbo_activo { TURBO } else { 1.0 };
        let maxima = limite * turbo;
        // Acelera mas fuerte cuanto mas alto es el limite: en crucero se llega rapido a fondo.
        let aceleracion = ACELERACION.max(maxima * 0.8) * turbo;
        if e.empuje != 0.0 {
            self.velocidad += e.empuje * aceleracion * dt;
        } else {
            self.velocidad *= (-FRENADO * dt).exp();
        }
        // Si va mas rapido que el limite (llego a un planeta a toda velocidad),
        // la "gravedad" la frena de a poco en vez de pararla en seco.
        if self.velocidad > maxima {
            self.velocidad += (maxima - self.velocidad) * (1.0 - (-2.5 * dt).exp());
        }
        self.velocidad = self.velocidad.max(-VELOCIDAD_CERCA);
        self.vel = self.adelante() * self.velocidad;
        self.pos += self.vel * dt;

        // Barrel roll: una vuelta completa y un brinquito de lado.
        let empezo = e.barrel && self.barrel <= 0.0;
        if empezo {
            self.barrel = BARREL;
            self.lado_barrel = if e.girar < 0.0 { -1.0 } else { 1.0 };
        }
        if self.barrel > 0.0 {
            self.barrel = (self.barrel - dt).max(0.0);
            let derecha = Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin());
            let k = 1.0 - self.barrel / BARREL;
            self.pos += derecha * (-self.lado_barrel * 5.0 * (k * std::f32::consts::PI).sin() * dt);
        }
        empezo
    }

    /// El angulo extra del barrel roll ahorita (suavizado al principio y al final).
    fn giro_barrel(&self) -> f32 {
        if self.barrel <= 0.0 {
            return 0.0;
        }
        let k = 1.0 - self.barrel / BARREL;
        let k = k * k * (3.0 - 2.0 * k);
        self.lado_barrel * std::f32::consts::TAU * k
    }

    /// La matriz de modelo M de la nave.
    pub fn matriz(&self) -> Mat4 {
        Mat4::translation(self.pos)
            * Mat4::rotation_y(self.yaw)
            * Mat4::rotation_x(self.pitch)
            * Mat4::rotation_z(self.roll + self.giro_barrel())
            * Mat4::scale(ESCALA)
    }
}

/// La camara en tercera persona: va atras y arriba de la nave y la sigue con
/// un poco de retraso, que es lo que hace que se sienta la velocidad.
pub struct Camara {
    pub ojo: Vec3,
    pub mira: Vec3,
    /// Cuanto se ladea la camara (radianes): un poquito con la nave, para que se sienta la vuelta.
    pub ladeo: f32,
}

impl Camara {
    pub fn new(ojo: Vec3, mira: Vec3) -> Self {
        Camara { ojo, mira, ladeo: 0.0 }
    }

    pub fn detras_de(nave: &Nave) -> Self {
        let (ojo, mira) = Self::ideal(nave);
        Camara { ojo, mira, ladeo: 0.0 }
    }

    /// El "arriba" de la camara (ladeado con `ladeo`), para la matriz de vista.
    pub fn arriba(&self) -> Vec3 {
        let adelante = (self.mira - self.ojo).normalize();
        let derecha = adelante.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
        let arriba = derecha.cross(adelante);
        (arriba * self.ladeo.cos() - derecha * self.ladeo.sin()).normalize()
    }

    /// Donde quiere estar la camara (ojo y punto al que mira) para seguir a la nave.
    pub fn ideal(nave: &Nave) -> (Vec3, Vec3) {
        let adelante = nave.adelante();
        let arriba = Vec3::new(0.0, 1.0, 0.0);
        // A mas velocidad, la camara se queda un poco mas atras: se siente el jalon.
        let rapido = (nave.velocidad / VELOCIDAD_CRUCERO).clamp(0.0, 1.0);
        // Girando, la camara se adelanta un poco hacia donde va la vuelta (se ve a donde se va).
        let derecha = Vec3::new(nave.yaw.cos(), 0.0, -nave.yaw.sin());
        let hacia = derecha * (-nave.viraje * 2.2);
        (nave.pos - adelante * (5.0 + 3.0 * rapido) + arriba * (2.4 + 0.8 * rapido), nave.pos + adelante * 6.0 + arriba * 1.0 + hacia)
    }

    /// `rigidez` es que tan pegada va (por segundo): en el hiperespacio se
    /// pega mucho mas para que la nave no se le escape.
    pub fn seguir(&mut self, nave: &Nave, dt: f32, rigidez: f32) {
        let (ojo, mira) = Self::ideal(nave);
        let k = 1.0 - (-rigidez * dt).exp();
        self.ojo = self.ojo.lerp(ojo, k);
        self.mira = self.mira.lerp(mira, 1.0 - (-rigidez * 1.6 * dt).exp());
        // Se ladea con la nave, pero menos (si no, marea).
        self.ladeo += (nave.roll * 0.35 - self.ladeo) * (1.0 - (-5.0 * dt).exp());
    }
}
