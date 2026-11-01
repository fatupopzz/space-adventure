//! La intro: en una estacion que orbita junto a la Tierra, Miku esta en el
//! centro del escenario frente a la nave dormida. Saluda a la camara con un
//! guino, baila mientras los fans brincan con sus barras de luz, hace un
//! corazon con los brazos, gira y brinca a la burbuja (fuegos artificiales).
//! La nave se despierta y se levanta: queda flotando en el centro del muelle,
//! con los motores prendidos; la camara se pone detras de la nave y empieza el
//! juego. Con aceptar se salta.
//!
//! Es una secuencia por tiempo: cada cosa es una funcion de los segundos
//! desde que empezo. Todo se mide desde la plataforma, que se mueve con la
//! Tierra: por eso las posiciones son relativas a `plataforma(t)`.

use crate::baile::Paso;
use crate::escena::{CUERPOS, TIERRA};
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::nave::{Camara, Nave};
use crate::pipeline::proyectar;
use crate::sonido::Efecto;
use std::f32::consts::{PI, TAU};

pub const ESCALA_PLATAFORMA: f32 = 0.5;
/// La plataforma es una estacion en orbita con la Tierra: va siempre a este
/// desplazamiento de ella, con la Tierra detras de la nave.
const DESDE_TIERRA: Vec3 = Vec3::new(0.0, -0.6, -9.0);
/// Donde baila Miku: al centro del escenario, enfrente de la nave, sobre el
/// circulo pintado (relativo a la plataforma, que mide 1.6 de radio).
const MIKU: Vec3 = Vec3::new(0.0, 0.0, -0.95);
const MIKU_ESCALA: f32 = 0.5;
const BPM: f32 = 120.0;

/// La coreografia: de que segundo a que segundo va cada paso.
const T_SALUDO: f32 = 3.2;
const T_BAILE: f32 = 4.9;
const T_CORAZON: f32 = 7.0;
const T_GIRO: f32 = 8.1;
const T_SALTO: f32 = 8.6;
const T_ABORDA: f32 = 9.8;
const T_DESPEGUE: f32 = 10.6;
pub const DURACION: f32 = 14.5;
/// Hasta cuando se ve el titulo.
pub const T_TITULO: f32 = 3.0;

/// Lo que dice la IA y desde que segundo.
const GUION: [(f32, &str); 6] = [
    (0.3, "Secuencia de inicio. Piloto: no está. Nave: dormida. Presupuesto: tampoco está."),
    (T_SALUDO, "Piloto localizada. Saludando al público: doce alienígenas y una estación. Igual se emociona. Chilero."),
    (T_CORAZON, "Eso es un corazón. Registrado como afecto no autorizado. Se descuenta de su sueldo, que es cero."),
    (T_SALTO, "Piloto abordando. Por fin. Nueve segundos tarde: récord de puntualidad chapina."),
    (T_DESPEGUE, "Despegue. Destino: todo el sistema solar. Gasolina: la que haya."),
    (13.0, "{aceptar}: hiperespacio.  {aterrizar}: aterrizar.  Yo: juzgar."),
];

/// Los fuegos artificiales: cuando estallan y donde (relativo a la plataforma).
const FUEGOS: [(f32, Vec3, [f32; 3]); 6] = [
    (T_ABORDA, Vec3::new(-1.6, 3.0, 0.8), [1.0, 0.45, 0.8]),
    (T_ABORDA + 0.25, Vec3::new(1.8, 3.4, 1.2), [0.35, 0.95, 1.0]),
    (T_ABORDA + 0.5, Vec3::new(0.0, 4.0, 2.0), [1.0, 0.85, 0.35]),
    (T_DESPEGUE + 0.4, Vec3::new(-2.2, 2.6, -1.0), [0.7, 1.0, 0.45]),
    (T_DESPEGUE + 0.6, Vec3::new(2.0, 3.2, -0.5), [1.0, 0.45, 0.8]),
    (T_DESPEGUE + 0.8, Vec3::new(0.0, 3.8, 0.5), [0.35, 0.95, 1.0]),
];

/// Los colores de los fans, en el orden de `fans.obj`.
pub const OCULTAR_FAN: [[&str; 2]; 3] = [["fan_rosa", "fan_cian"], ["fan_lila", "fan_cian"], ["fan_lila", "fan_rosa"]];

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Donde esta la plataforma (su cara de arriba) en el tiempo `t` del mundo.
pub fn plataforma(t: f32) -> Vec3 {
    CUERPOS[TIERRA].posicion(t) + DESDE_TIERRA
}

/// Donde espera la nave, parada sobre la plataforma.
pub fn nave_parada(t: f32) -> Vec3 {
    plataforma(t) + Vec3::new(0.0, 1.0, 0.0)
}

/// Que tanto se levanta la nave al despertar.
const ALTO_FLOTANDO: f32 = 0.45;

/// Donde empieza a volar: flotando sobre el centro del muelle, ya despierta.
pub fn nave_en_el_muelle(t: f32) -> Vec3 {
    nave_parada(t) + Vec3::new(0.0, ALTO_FLOTANDO, 0.0)
}

pub struct Intro {
    pub t: f32,
    confeti: Vec<(Vec3, [f32; 3])>,
}

impl Intro {
    pub fn new() -> Self {
        let mut s: u32 = 0x1A7E_0F11;
        let mut azar = move || {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (s >> 8) as f32 / (1u32 << 24) as f32
        };
        let paleta = [[1.0, 0.45, 0.8], [0.35, 0.95, 1.0], [1.0, 0.85, 0.35], [0.7, 1.0, 0.45]];
        let confeti = (0..160)
            .map(|_| {
                let (a, r) = (azar() * TAU, 0.3 + azar() * 2.6);
                (Vec3::new(r * a.cos(), azar() * 3.5, r * a.sin()), paleta[(azar() * 4.0) as usize % 4])
            })
            .collect();
        Intro { t: 0.0, confeti }
    }

    pub fn termino(&self) -> bool {
        self.t >= DURACION
    }

    fn golpe(&self) -> f32 {
        (-(self.t * BPM / 60.0).fract() * 5.0).exp()
    }

    /// Avanza la intro y mueve la nave. `saltar` la manda al final.
    pub fn actualizar(&mut self, nave: &mut Nave, saltar: bool, dt: f32, t_mundo: f32) {
        self.t = if saltar { DURACION } else { self.t + dt };
        nave.yaw = 0.0;
        nave.pitch = 0.0;
        if self.t < T_DESPEGUE {
            nave.pos = nave_parada(t_mundo);
            nave.velocidad = 0.0;
            nave.vel = Vec3::zero();
        } else {
            // Se despierta y se levanta: queda flotando en el centro del muelle, con los motores
            // prendidos, lista para que la piloto arranque.
            let k = self.t - T_DESPEGUE;
            nave.velocidad = 0.0;
            nave.vel = Vec3::zero();
            nave.pos = nave_en_el_muelle(t_mundo) - Vec3::new(0.0, ALTO_FLOTANDO * (1.0 - suave(k / 2.0)), 0.0);
        }
    }

    /// La frase de la IA que toca ahora.
    pub fn mensaje(&self) -> &'static str {
        GUION.iter().rev().find(|(t, _)| self.t >= *t).map_or("", |(_, m)| m)
    }

    pub fn piloto_a_bordo(&self) -> bool {
        self.t >= T_ABORDA
    }

    /// Que tanto brillan las chispas de cuando Miku entra a la burbuja.
    pub fn chispas(&self) -> f32 {
        let k = self.t - T_ABORDA;
        if (0.0..0.6).contains(&k) { 1.0 - k / 0.6 } else { 0.0 }
    }

    /// La nave duerme mientras no hay piloto.
    pub fn expresion_nave(&self) -> &'static str {
        if self.piloto_a_bordo() { "feliz" } else { "dormida" }
    }

    pub fn expresion_miku(&self) -> &'static str {
        match self.t {
            t if t < T_SALUDO => "feliz",
            t if t < T_BAILE => "guino",
            t if t < T_CORAZON => if (t * 1.0) as i32 % 2 == 0 { "feliz" } else { "guino" },
            t if t < T_GIRO => "enamorada",
            _ => "feliz",
        }
    }

    /// El paso de baile de antes, el de ahora y que tanto va del uno al otro.
    pub fn paso(&self) -> (Paso, Paso, f32) {
        const PASOS: [(f32, Paso); 4] = [(0.0, Paso::Saludo), (T_BAILE, Paso::Baile), (T_CORAZON, Paso::Corazon), (T_GIRO, Paso::Giro)];
        let i = PASOS.iter().rposition(|(t, _)| self.t >= *t).unwrap_or(0);
        let antes = PASOS[i.saturating_sub(1)].1;
        (antes, PASOS[i].1, suave((self.t - PASOS[i].0) / 0.35))
    }

    /// Cuantos corazones y notas flotan alrededor de Miku (0 a 1).
    pub fn corazones(&self) -> f32 {
        if self.t < T_SALUDO || self.t > T_SALTO { 0.0 } else if self.t > T_CORAZON { 1.0 } else { 0.55 }
    }

    /// Donde esta la cabeza de Miku (para los corazones que flotan).
    pub fn cabeza_miku(&self, t_mundo: f32) -> Vec3 {
        plataforma(t_mundo) + MIKU + Vec3::new(0.0, 0.85 * MIKU_ESCALA * 1.12, 0.0)
    }

    /// La matriz de modelo de Miku bailando (o brincando) y que tanto esta
    /// brincando, o `None` si ya esta adentro. `asiento` es donde queda.
    pub fn miku(&self, asiento: Vec3, t_mundo: f32) -> Option<(Mat4, f32)> {
        if self.piloto_a_bordo() {
            return None;
        }
        let piso = plataforma(t_mundo) + MIKU;
        let k = ((self.t - T_SALTO) / (T_ABORDA - T_SALTO)).clamp(0.0, 1.0);
        // Un arco: sube por encima de la burbuja y cae adentro, dando una vuelta.
        let pos = piso.lerp(asiento, suave(k)) + Vec3::new(0.0, 1.3 * (k * PI).sin(), 0.0);
        let escala = MIKU_ESCALA + (0.25 - MIKU_ESCALA) * k;
        let giro = TAU * suave((self.t - T_GIRO) / (T_SALTO - T_GIRO)) + TAU * suave(k);
        let saltando = if self.t > T_SALTO - 0.3 { suave((self.t - T_SALTO + 0.3) / 0.3) } else { 0.0 };
        Some((Mat4::translation(pos) * Mat4::rotation_y(giro) * Mat4::scale(escala), saltando))
    }

    /// Los fans alrededor de la orilla (detras y a los lados, para no tapar
    /// a Miku): su matriz, su color y el angulo de su barra de luz.
    pub fn fans(&self, t_mundo: f32) -> Vec<(Mat4, usize, f32)> {
        let base = plataforma(t_mundo);
        let tiempos = self.t * BPM / 60.0;
        (0..12)
            .map(|n| {
                let a = 0.95 + n as f32 * (TAU - 1.9) / 11.0;
                let p = Vec3::new(1.3 * a.sin(), 0.0, -1.3 * a.cos());
                let brinco = 0.09 * ((tiempos + n as f32 * 0.37) * PI).sin().abs();
                // Miran hacia Miku.
                let d = MIKU - p;
                let yaw = f32::atan2(-d.x, -d.z) + 0.25 * (self.t * 2.0 + n as f32).sin();
                let m = Mat4::translation(base + p + Vec3::new(0.0, brinco, 0.0)) * Mat4::rotation_y(yaw) * Mat4::scale(0.6);
                let barra = 0.7 * ((tiempos + n as f32 * 0.5) * PI).sin();
                (m, n % 3, barra)
            })
            .collect()
    }

    /// Los reflectores de los postes de la orilla: su matriz y su color.
    pub fn haces(&self, t_mundo: f32) -> Vec<(Mat4, usize)> {
        // Se apagan cuando la nave ya se fue (si no, la camara pasa por uno).
        if self.t > T_DESPEGUE + 1.0 {
            return Vec::new();
        }
        let base = plataforma(t_mundo);
        (0..6)
            .map(|n| {
                let a = n as f32 * PI / 3.0;
                let poste = base + Vec3::new(1.425 * a.cos(), 0.25, 1.425 * a.sin());
                let blanco = base + MIKU + Vec3::new(1.2 * (self.t * 0.8 + n as f32).sin(), 2.4 + 0.8 * (self.t * 1.1 + n as f32).cos(), 1.2 * (self.t * 0.6 + n as f32 * 2.0).sin());
                let y = (blanco - poste).normalize();
                let x = y.cross(Vec3::new(0.0, 0.0, 1.0)).normalize();
                let z = x.cross(y);
                let m = Mat4::translation(poste) * Mat4::from_basis(x, y, z) * Mat4::scale_xyz(Vec3::new(0.8, 3.2, 0.8));
                (m, (n + (self.t * BPM / 60.0 / 4.0) as usize) % 4)
            })
            .collect()
    }

    /// El confeti que cae y los fuegos artificiales, sumados como luz.
    pub fn dibujar_efectos(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4, t_mundo: f32) {
        let base = plataforma(t_mundo);
        let golpe = self.golpe();
        for (k, &(p, color)) in self.confeti.iter().enumerate() {
            let caida = (p.y - self.t * 0.5).rem_euclid(3.5);
            let (s, c) = (self.t * 0.25).sin_cos();
            let pos = base + Vec3::new(p.x * c - p.z * s, caida, p.x * s + p.z * c);
            let brillo = (0.5 + 0.5 * (self.t * 5.0 + k as f32).sin()) * (0.6 + 0.6 * golpe);
            let col = color.map(|v| v * brillo);
            if let (Some(a), Some(b)) = (proyectar(pos, vista_proyeccion, fb), proyectar(pos + Vec3::new(0.0, 0.04, 0.0), vista_proyeccion, fb)) {
                fb.linea_de_luz(a, b, col, col);
                fb.linea_de_luz((a.0 - 1.0, a.1, a.2), (a.0 + 1.0, a.1, a.2), col, col);
            }
        }
        for &(t0, centro, color) in &FUEGOS {
            let edad = self.t - t0;
            if !(0.0..2.2).contains(&edad) {
                continue;
            }
            let radio = 1.7 * (1.0 - (-3.0 * edad).exp());
            let luz = (-1.6 * edad).exp();
            let centro = base + centro + Vec3::new(0.0, -0.35 * edad * edad, 0.0);
            // 70 chispas repartidas parejo sobre una esfera (espiral de Fibonacci).
            for i in 0..70 {
                let y = 1.0 - 2.0 * (i as f32 + 0.5) / 70.0;
                let r = (1.0 - y * y).sqrt();
                let a = i as f32 * 2.399_963;
                let dir = Vec3::new(r * a.cos(), y, r * a.sin());
                let (p0, p1) = (centro + dir * (radio * 0.78), centro + dir * radio);
                if let (Some(a), Some(b)) = (proyectar(p0, vista_proyeccion, fb), proyectar(p1, vista_proyeccion, fb)) {
                    fb.linea_de_luz(a, b, color.map(|c| c * luz * 0.3), color.map(|c| c * luz * 1.2));
                }
            }
        }
    }

    /// La camara: tomas que se mueven y, al despegar, se va detras de la nave.
    pub fn camara(&self, nave: &Nave, t_mundo: f32) -> (Vec3, Vec3) {
        let base = plataforma(t_mundo);
        let t = self.t;
        // Un poco de camara en mano.
        let mano = Vec3::new((t * 1.3).sin() * 0.03, (t * 1.7).cos() * 0.02, 0.0);
        let cara = MIKU + Vec3::new(0.0, 0.45, 0.0);
        let entre = |a: f32, b: f32| suave((t - a) / (b - a));
        let fija = if t < T_SALUDO {
            // Plano general dando la vuelta: la estacion, los fans, la Tierra detras.
            let a = -0.75 + 0.5 * entre(0.0, T_SALUDO);
            (Vec3::new(6.0 * a.sin(), 2.6 - 0.4 * entre(0.0, T_SALUDO), -6.0 * a.cos()), Vec3::new(0.0, 0.6, 0.0))
        } else if t < T_BAILE {
            // De frente y cerquita: Miku saluda a la camara.
            (cara + Vec3::new(0.05, 0.02, -1.05).lerp(Vec3::new(0.0, 0.0, -0.85), entre(T_SALUDO, T_BAILE)), cara)
        } else if t < T_CORAZON {
            // Alrededor de Miku mientras baila, con los fans detras.
            let b = -0.8 + 1.6 * entre(T_BAILE, T_CORAZON);
            (MIKU + Vec3::new(1.5 * b.sin(), 0.55 + 0.05 * (t * 1.3).sin(), -1.5 * b.cos()), MIKU + Vec3::new(0.0, 0.32, 0.0))
        } else if t < T_SALTO {
            // De frente, un poco abajo, para el corazon y el giro.
            (cara + Vec3::new(-0.1, -0.12, -1.25), cara + Vec3::new(0.0, 0.08, 0.0))
        } else {
            // Plano bajo para el brinco a la burbuja.
            let k = entre(T_SALTO, T_DESPEGUE);
            (Vec3::new(2.4, 0.8, -3.6).lerp(Vec3::new(2.0, 1.4, -3.3), k), (MIKU + Vec3::new(0.0, 0.6, 0.0)).lerp(Vec3::new(0.0, 1.8, 0.0), k))
        };
        let fija = (base + fija.0 + mano, base + fija.1);
        let k = suave((t - T_DESPEGUE) / 2.8);
        let (ojo, mira) = Camara::ideal(nave);
        (fija.0.lerp(ojo, k), fija.1.lerp(mira, k))
    }

    /// Lo que dice Miku en su globito, si esta diciendo algo.
    pub fn globo(&self) -> Option<&'static str> {
        if (T_SALUDO + 0.3..T_BAILE).contains(&self.t) {
            Some("¡Hola a todos!")
        } else if (T_CORAZON..T_GIRO + 0.4).contains(&self.t) {
            Some("¡Los quiero mucho!")
        } else {
            None
        }
    }

    /// Los sonidos que tocan entre el segundo `antes` y el actual.
    pub fn eventos_desde(&self, antes: f32) -> Vec<Efecto> {
        let cruzo = |t0: f32| antes < t0 && self.t >= t0;
        let mut v: Vec<Efecto> = FUEGOS.iter().filter(|f| cruzo(f.0)).map(|_| Efecto::Fuego).collect();
        if cruzo(T_DESPEGUE) {
            v.push(Efecto::Salto);
        }
        v
    }

    /// Donde salen las "Z" de la nave dormida (arriba de su cabeza).
    pub fn ronquido(&self, t_mundo: f32) -> Option<Vec3> {
        (!self.piloto_a_bordo()).then(|| nave_parada(t_mundo) + Vec3::new(0.35, 1.0, -0.2))
    }
}
