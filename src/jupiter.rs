//! Aterrizaje en Jupiter: JUPITER ASTEROIDS, en primera persona (como Star Fox).
//!
//! Se ve desde la cabina de la nave, que avanza sola hacia Jupiter (enorme, al
//! fondo). Los asteroides vienen de frente: se apunta moviendo la nave (la mira
//! va al centro) y se dispara con los dos canones. Las rocas grandes se parten;
//! las rojas explotan y se llevan a las de al lado; las doradas, al reventarlas,
//! sueltan un dato de Bill Nye (y muchos puntos). Si una roca le pega a la nave,
//! baja el escudo; sin escudo, se acaba la partida.
//!
//! Los comodines son las cuatro lunas de Galileo: vienen flotando y se atrapan
//! volando hacia ellas.
//! - Io (volcanes): disparo de fuego, rapidisimo.
//! - Europa (hielo): repara el escudo y aguanta golpes un rato.
//! - Ganimedes (la mas grande): rayo gordo que atraviesa todo.
//! - Calisto (crateres): camara lenta para las rocas.
//!
//! Cada tres oleadas, el jefe: LA GRAN MANCHA ROJA, una tormenta enorme al fondo
//! que escupe rocas; se le dispara al ojo cuando se abre. Bill Nye presenta y
//! cuenta datos de Jupiter entre oleadas. Las lunas que se atrapan se guardan en
//! la mochila (en el espacio dan turbo, escudo, iman y cometa).

use crate::vida::{Tam, Tono, Vida};
use crate::entrada::Entrada;
use crate::matematica::{Mat4, Vec3};
use crate::sonido::Efecto;
use crate::ui;
use raylib::prelude::*;
use std::f32::consts::TAU;

/// Hasta donde se puede mover la nave (de lado y de arriba a abajo).
pub const LADO: f32 = 9.0;
pub const ALTO: f32 = 5.5;
/// Donde aparecen las rocas (delante, lejos) y la distancia de la mira.
const LEJOS: f32 = 170.0;
pub const MIRA: f32 = 45.0;
/// Radio de las rocas (grande, mediana, chica), sus puntos.
const RADIO: [f32; 3] = [2.6, 1.5, 0.85];
const PUNTOS: [u32; 3] = [20, 50, 100];
/// Cuanto duran los comodines (segundos).
pub const DURA: [f32; 4] = [10.0, 12.0, 8.0, 8.0];
/// Cuanto dura cada oleada (segundos) y cuanto quita cada golpe del escudo.
const OLEADA: f32 = 32.0;
const GOLPE: f32 = 0.25;

pub const LUNAS: [&str; 4] = ["Io", "Europa", "Ganímedes", "Calisto"];
pub const LUNAS_QUE_HACEN: [&str; 4] = ["disparo de fuego", "escudo de hielo", "rayo que atraviesa", "cámara lenta"];
pub const COLOR_LUNA: [[f32; 3]; 4] = [[1.0, 0.75, 0.2], [0.75, 0.9, 1.0], [0.75, 0.65, 0.55], [0.55, 0.5, 0.75]];

const INTRO: [&str; 3] = [
    "¡Bill Nye, el chico de la ciencia! Bienvenidos a Júpiter, el planeta más grande del sistema solar. ¡Cabrían más de mil Tierras adentro!",
    "Su gravedad jala asteroides como un imán, y vienen directo hacia nosotros. Apunte moviendo la nave: la mira está al centro. ¡Y dispare!",
    "Si ve una luna de Galileo, vuele hacia ella: cada una da un poder. Y si ve una roca dorada, ¡reviéntela! Trae un dato científico. ¡La ciencia es poder!",
];

const DATOS_JUPITER: [&str; 10] = [
    "¡Oleada superada! Dato: un día en Júpiter dura menos de 10 horas. Es el planeta que gira más rápido. Por eso se ve aplastadito.",
    "¡Muy bien! Júpiter no tiene suelo: es casi todo hidrógeno y helio. Si intenta aterrizar, se hunde para siempre. ¡No lo intente!",
    "¡Ciencia! Júpiter tiene al menos 95 lunas. Las cuatro grandes son las de Galileo. Las demás son como estas rocas: chiquitas y mal portadas.",
    "Dato: Júpiter protege a la Tierra. Su gravedad desvía muchos cometas y asteroides que si no, vendrían para allá. ¡Gracias, Júpiter!",
    "¡Considere esto! Júpiter tiene anillos, pero son tan oscuros y delgados que no se descubrieron hasta 1979, con la Voyager 1.",
    "Dato: las auroras de Júpiter son cientos de veces más potentes que las de la Tierra. Y brillan en luz ultravioleta. ¡Fiesta cósmica!",
    "¡Bill! ¡Bill! ¡Bill! En 1994 el cometa Shoemaker-Levy 9 se partió y chocó contra Júpiter. Dejó manchas más grandes que la Tierra.",
    "Dato: el campo magnético de Júpiter es tan grande que si lo pudiéramos ver, en el cielo se vería más grande que la Luna llena.",
    "¡Ciencia! Júpiter emite más calor del que recibe del Sol. Se sigue encogiendo poquito a poco, y eso lo calienta por dentro.",
    "Dato: la sonda Juno lleva años dándole vueltas a Júpiter y mandando fotos de sus tormentas. Son tan bonitas que parecen pinturas.",
];

const JEFE: [&str; 2] = [
    "¡CUIDADO! ¡LA GRAN MANCHA ROJA! Una tormenta más grande que la Tierra, con vientos de más de 400 km/h. ¡Dispárele al ojo cuando se abra!",
    "¡Lo logró! La tormenta se calmó... por ahora. La Gran Mancha Roja lleva siglos girando. Volverá. La ciencia lo garantiza.",
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Fase {
    Intro(usize),
    Jugando,
    /// Se rompio la nave: segundos que lleva.
    Choque(f32),
    Fin,
}

pub enum AccionJupiter {
    Nada,
    Despegar,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clase {
    Gris,
    Roja,
    Dorada,
}

pub struct Roca {
    pub pos: Vec3,
    vel: Vec3,
    pub tam: usize,
    pub clase: Clase,
    pub giro: f32,
    eje: Vec3,
    pub modelo: usize,
}

pub struct Bala {
    pub pos: Vec3,
    pub vel: Vec3,
    vida: f32,
    pub gorda: bool,
    pub fuego: bool,
}

pub struct LunaSuelta {
    pub pos: Vec3,
    pub cual: usize,
}

pub struct Chispa {
    pub pos: Vec3,
    vel: Vec3,
    pub vida: f32,
    pub color: [f32; 3],
}

pub struct Letrero {
    pub pos: Vec3,
    pub texto: String,
    pub vida: f32,
}

pub struct Mancha {
    pub pos: Vec3,
    pub vida: u32,
    pub vida_max: u32,
    pub t: f32,
    escupe: f32,
    pub golpe: f32,
}

impl Mancha {
    /// El ojo de la tormenta se abre un rato cada pocos segundos (ahi es cuando le duele).
    pub fn abierta(&self) -> bool {
        (self.t * 0.45).fract() > 0.55
    }
}

pub struct Jupiter {
    fase: Fase,
    t: f32,
    pub total: f32,
    semilla: u32,
    /// Donde va la nave (x, y; z siempre 0: lo que avanza es todo lo demas).
    pub nave: Vec3,
    vel: Vec3,
    /// La inclinacion al moverse (para que se sienta el viraje).
    pub ladeo: f32,
    pub escudo: f32,
    recarga: f32,
    canon: bool,
    pub rocas: Vec<Roca>,
    pub balas: Vec<Bala>,
    pub lunas: Vec<LunaSuelta>,
    pub chispas: Vec<Chispa>,
    pub letreros: Vec<Letrero>,
    pub mancha: Option<Mancha>,
    pub oleada: u32,
    tiempo_oleada: f32,
    proxima: f32,
    pub anuncio: f32,
    pub puntos: u32,
    pub record: u32,
    pub combo: u32,
    desde_golpe: f32,
    pub poder: [f32; 4],
    pub recogidas: [u32; 4],
    pub jefes: u32,
    pub dorados: u32,
    bill: Option<(String, f32)>,
    dato: usize,
    /// Un dato que hay que pedirle al juego (las doradas sueltan los de Bill del espacio).
    pub pide_dato: bool,
    pub sacudida: f32,
    /// El muñequito del tablero: angulo y velocidad de lado, y angulo y velocidad adelante.
    cabezon: [f32; 4],
    pub destello: f32,
    pub rojo: f32,
    pub cobrado: bool,
    /// Los letreros gigantes, el confeti y las sacudidas (ver vida.rs).
    pub vida: Vida,
    pub eventos: Vec<Efecto>,
}

impl Jupiter {
    pub fn new(semilla: u32, record: u32) -> Self {
        Jupiter {
            fase: Fase::Intro(0),
            t: 0.0,
            total: 0.0,
            semilla: semilla | 1,
            nave: Vec3::zero(),
            vel: Vec3::zero(),
            ladeo: 0.0,
            escudo: 1.0,
            recarga: 0.0,
            canon: false,
            rocas: Vec::new(),
            balas: Vec::new(),
            lunas: Vec::new(),
            chispas: Vec::new(),
            letreros: Vec::new(),
            mancha: None,
            oleada: 0,
            tiempo_oleada: 0.0,
            proxima: 1.0,
            anuncio: 0.0,
            puntos: 0,
            record,
            combo: 0,
            desde_golpe: 9.0,
            poder: [0.0; 4],
            recogidas: [0; 4],
            jefes: 0,
            dorados: 0,
            bill: None,
            dato: semilla as usize % DATOS_JUPITER.len(),
            pide_dato: false,
            sacudida: 0.0,
            cabezon: [0.0; 4],
            destello: 0.0,
            rojo: 0.0,
            cobrado: false,
            vida: Vida::new("ASTEROIDES", "Júpiter. Dispare a todo menos a la ciencia.", Tono::Oro),
            eventos: Vec::new(),
        }
    }

    fn azar(&mut self) -> f32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Bill dice algo (sin pausar el juego).
    pub fn decir(&mut self, s: impl Into<String>) {
        let s = s.into();
        let dura = 3.5 + s.chars().count() as f32 / 20.0;
        self.bill = Some((s, dura));
    }

    pub fn bill(&self) -> Option<&str> {
        match self.fase {
            Fase::Intro(i) => Some(INTRO[i]),
            _ => self.bill.as_ref().map(|(s, _)| s.as_str()),
        }
    }

    pub fn terminado(&self) -> bool {
        self.fase == Fase::Fin
    }

    pub fn jugando(&self) -> bool {
        matches!(self.fase, Fase::Jugando | Fase::Choque(_))
    }


    /// Que tan rapido avanza todo hacia la nave (sube con las oleadas; Calisto lo frena).
    fn rapidez(&self) -> f32 {
        let base = 24.0 + self.oleada as f32 * 2.5;
        base * if self.poder[3] > 0.0 { 0.35 } else { 1.0 }
    }

    /// Para el fondo: cuanto se ha avanzado (las estrellas y las rayitas de velocidad).
    pub fn avance(&self) -> f32 {
        self.total * self.rapidez()
    }

    fn roca(&mut self, pos: Vec3, tam: usize, clase: Clase) -> Roca {
        let deriva = Vec3::new(self.azar() - 0.5, self.azar() - 0.5, 0.0) * (3.0 + tam as f32 * 2.0);
        let eje = Vec3::new(self.azar() - 0.5, self.azar() - 0.5, self.azar() - 0.5).normalize();
        Roca { pos, vel: deriva, tam, clase, giro: self.azar() * TAU, eje, modelo: (self.azar() * 3.0) as usize % 3 }
    }

    /// Una roca nueva al fondo, mas o menos hacia donde esta la nave (para que haya que esquivar).
    fn soltar_roca(&mut self) {
        let hacia_nave = self.azar() < 0.45;
        let (x, y) = if hacia_nave {
            (self.nave.x + (self.azar() - 0.5) * 6.0, self.nave.y + (self.azar() - 0.5) * 4.0)
        } else {
            ((self.azar() - 0.5) * 2.6 * LADO, (self.azar() - 0.5) * 2.6 * ALTO)
        };
        let suerte = self.azar();
        let clase = if suerte < 0.06 { Clase::Dorada } else if suerte < 0.18 + self.oleada as f32 * 0.01 { Clase::Roja } else { Clase::Gris };
        let tam = if clase == Clase::Dorada { 1 } else if self.azar() < 0.55 { 0 } else { 1 };
        let lejos = LEJOS + self.azar() * 30.0;
        let r = self.roca(Vec3::new(x, y, -lejos), tam, clase);
        self.rocas.push(r);
    }

    fn siguiente_oleada(&mut self) {
        self.oleada += 1;
        self.anuncio = 2.5;
        self.tiempo_oleada = 0.0;
        if self.oleada % 3 == 0 {
            let vida = 30 + self.oleada * 5;
            self.mancha = Some(Mancha { pos: Vec3::new(0.0, 2.0, -95.0), vida, vida_max: vida, t: 0.0, escupe: 2.0, golpe: 0.0 });
            self.decir(JEFE[0]);
            self.vida.desastre("¡LA MANCHA ROJA!", "Se despertó, y viene enojada");
            self.eventos.push(Efecto::Alarma);
        } else if self.oleada > 1 {
            let texto = format!("OLEADA {}", self.oleada);
            self.vida.grito(&texto, Tono::Cian, Tam::Grande);
            let d = DATOS_JUPITER[self.dato % DATOS_JUPITER.len()];
            self.dato += 1;
            self.decir(d);
        }
    }

    fn explotar(&mut self, pos: Vec3, n: usize, color: [f32; 3], fuerza: f32) {
        for _ in 0..n {
            let dir = Vec3::new(self.azar() - 0.5, self.azar() - 0.5, self.azar() - 0.5).normalize();
            let v = fuerza * (0.4 + self.azar());
            let c = [color[0] * (0.7 + 0.3 * self.azar()), color[1] * (0.7 + 0.3 * self.azar()), color[2]];
            let vida = 0.5 + self.azar() * 0.7;
            self.chispas.push(Chispa { pos, vel: dir * v, vida, color: c });
        }
    }

    /// Le pega a la roca `i`: se parte, da puntos, y las rojas explotan.
    fn romper(&mut self, i: usize) {
        let r = self.rocas.remove(i);
        let multi = 1 + (self.combo / 5).min(4);
        let pts = if r.clase == Clase::Dorada { 500 } else { PUNTOS[r.tam] } * multi;
        self.puntos += pts;
        self.combo += 1;
        self.desde_golpe = 0.0;
        if self.combo % 5 == 0 && multi >= 2 {
            let texto = format!("¡COMBO × {multi}!");
            self.vida.grito(&texto, Tono::Oro, Tam::Medio);
        }
        self.letreros.push(Letrero { pos: r.pos, texto: if multi > 1 { format!("+{pts} x{multi}") } else { format!("+{pts}") }, vida: 0.9 });
        let color = match r.clase {
            Clase::Roja => [1.0, 0.45, 0.1],
            Clase::Dorada => [1.0, 0.85, 0.3],
            Clase::Gris => [0.85, 0.8, 0.75],
        };
        self.explotar(r.pos, 14 + 10 * (2 - r.tam), color, 9.0);
        self.sacudida = self.sacudida.max(0.08 + 0.08 * (2 - r.tam) as f32);
        self.eventos.push(if r.tam == 0 { Efecto::Choque } else { Efecto::Don });
        if r.clase == Clase::Dorada {
            self.dorados += 1;
            self.pide_dato = true;
            self.vida.grito("¡ROCA DORADA!", Tono::Oro, Tam::Medio);
            self.vida.confeti(25, Tono::Oro);
            self.eventos.push(Efecto::Sello);
        }
        if r.tam < 2 && r.clase != Clase::Dorada {
            for lado in [-1.0, 1.0] {
                let clase = if r.clase == Clase::Roja && self.azar() < 0.5 { Clase::Roja } else { Clase::Gris };
                let mut hija = self.roca(r.pos, r.tam + 1, clase);
                let arriba = (self.azar() - 0.5) * 4.0;
                hija.vel = Vec3::new(lado * 6.0, arriba, 0.0);
                self.rocas.push(hija);
            }
        }
        if r.clase == Clase::Roja {
            self.explotar(r.pos, 40, [1.0, 0.5, 0.1], 16.0);
            self.destello = self.destello.max(0.3);
            self.vida.grito("¡BOOM!", Tono::Rojo, Tam::Grande);
            self.vida.golpe(0.45, Tono::Rojo);
            self.eventos.push(Efecto::Fuego);
            let mut k = 0;
            while k < self.rocas.len() {
                if (self.rocas[k].pos - r.pos).length() < 9.0 && self.rocas[k].clase != Clase::Roja {
                    self.romper(k);
                } else {
                    k += 1;
                }
            }
        }
        // De las grandes, a veces sale una luna de Galileo (viene flotando hacia la nave).
        if r.tam == 0 && self.azar() < 0.22 {
            let cual = (self.azar() * 4.0) as usize % 4;
            self.lunas.push(LunaSuelta { pos: Vec3::new(r.pos.x * 0.6, r.pos.y * 0.6, r.pos.z), cual });
        }
    }

    fn disparar(&mut self) {
        // Dos canones (a los lados de la cabina) que se turnan; con Io, los dos a la vez.
        let gorda = self.poder[2] > 0.0;
        let fuego = self.poder[0] > 0.0;
        let destino = self.nave + Vec3::new(0.0, 0.0, -MIRA);
        let lados: &[f32] = if fuego { &[-1.0, 1.0] } else if self.canon { &[1.0] } else { &[-1.0] };
        for &l in lados {
            let origen = self.nave + Vec3::new(l * 1.1, -0.7, -1.0);
            let dir = (destino - origen).normalize();
            self.balas.push(Bala { pos: origen, vel: dir * 140.0, vida: 1.2, gorda, fuego });
        }
        self.canon = !self.canon;
        self.recarga = if fuego { 0.07 } else if gorda { 0.18 } else { 0.11 };
        self.eventos.push(Efecto::Ka);
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionJupiter {
        self.t += dt;
        self.total += dt;
        self.pide_dato = false;
        if e.aterrizar {
            return AccionJupiter::Despegar;
        }
        self.sacudida *= (-6.0 * dt).exp();
        // El muñequito: un resorte que se queda atras cuando la nave vira y brinca con los golpes.
        let golpe = self.sacudida * 0.5 * (self.total * 23.0).sin();
        let objetivo = [self.vel.x * 0.018 + golpe, -self.vel.y * 0.012 + golpe * 0.5];
        for k in 0..2 {
            let (a, v) = (self.cabezon[2 * k], self.cabezon[2 * k + 1]);
            let v = v + ((objetivo[k] - a) * 90.0 - v * 2.2) * dt;
            self.cabezon[2 * k] = (a + v * dt).clamp(-0.7, 0.7);
            self.cabezon[2 * k + 1] = v;
        }
        self.destello = (self.destello - dt * 2.0).max(0.0);
        self.rojo = (self.rojo - dt * 2.5).max(0.0);
        if let Some((_, quedan)) = &mut self.bill {
            *quedan -= dt;
            if *quedan <= 0.0 {
                self.bill = None;
            }
        }
        match self.fase {
            Fase::Intro(i) => {
                let largo = INTRO[i].chars().count() as f32;
                if (e.aceptar && self.t > 0.4) || self.t > 3.0 + largo / 20.0 {
                    self.t = 0.0;
                    if i + 1 < INTRO.len() {
                        self.fase = Fase::Intro(i + 1);
                    } else {
                        self.fase = Fase::Jugando;
                        self.siguiente_oleada();
                    }
                }
                self.mover_cosas(dt);
                return AccionJupiter::Nada;
            }
            Fase::Fin => {
                if e.aceptar && self.t > 1.0 {
                    let record = self.record.max(self.puntos);
                    *self = Jupiter::new(self.semilla.wrapping_mul(31).wrapping_add(7), record);
                    self.fase = Fase::Jugando;
                    self.siguiente_oleada();
                }
                self.mover_cosas(dt);
                return AccionJupiter::Nada;
            }
            Fase::Choque(s) => {
                let s = s + dt;
                self.fase = Fase::Choque(s);
                if s > 2.2 {
                    self.fase = Fase::Fin;
                    self.t = 0.0;
                    self.record = self.record.max(self.puntos);
                    self.decir(if self.puntos >= 5000 {
                        "¡Increíble! Con esa puntería podría trabajar en la NASA. O en un arcade. La ciencia necesita gente como usted."
                    } else {
                        "¡Se acabó! Pero recuerde: en ciencia, cada error es un dato. Y usted recolectó MUCHOS datos."
                    });
                }
                self.mover_cosas(dt);
                return AccionJupiter::Nada;
            }
            Fase::Jugando => {}
        }
        // La nave: se mueve en X y Y (apuntar es moverse), con algo de inercia y ladeo.
        let quiere = Vec3::new(-e.girar, e.cabecear, 0.0);
        self.vel = self.vel + (quiere * 26.0 - self.vel) * (1.0 - (-8.0 * dt).exp());
        self.nave = self.nave + self.vel * dt;
        self.nave.x = self.nave.x.clamp(-LADO, LADO);
        self.nave.y = self.nave.y.clamp(-ALTO, ALTO);
        self.ladeo = self.ladeo + (-self.vel.x * 0.035 - self.ladeo) * (1.0 - (-6.0 * dt).exp());
        self.recarga -= dt;
        if (e.disparando || e.aceptar) && self.recarga <= 0.0 {
            self.disparar();
        }
        for p in &mut self.poder {
            *p = (*p - dt).max(0.0);
        }
        self.desde_golpe += dt;
        if self.desde_golpe > 1.5 {
            self.combo = 0;
        }
        // Salen rocas: cada vez mas seguido.
        self.tiempo_oleada += dt;
        self.proxima -= dt * if self.poder[3] > 0.0 { 0.4 } else { 1.0 };
        let toca = self.mancha.is_none() || self.azar() < 0.3;
        if self.proxima <= 0.0 && toca {
            let espera = (1.1 - self.oleada as f32 * 0.08).max(0.35) * (0.6 + 0.8 * self.azar());
            self.proxima = espera;
            self.soltar_roca();
        }
        if self.mancha.is_none() && self.tiempo_oleada > OLEADA {
            self.puntos += 100 * self.oleada;
            self.siguiente_oleada();
        }
        self.mover_cosas(dt);
        // Las balas contra las rocas y contra el ojo de la tormenta.
        let mut b = 0;
        while b < self.balas.len() {
            let (pos, gorda) = (self.balas[b].pos, self.balas[b].gorda);
            let radio_bala = if gorda { 1.4 } else { 0.5 };
            let mut gasto = false;
            if let Some(i) = self.rocas.iter().position(|r| (r.pos - pos).length() < RADIO[r.tam] + radio_bala) {
                self.romper(i);
                gasto = !gorda;
            }
            let mut pega = false;
            if let Some(m) = &mut self.mancha {
                if (m.pos - pos).length() < 9.0 {
                    gasto = true;
                    if m.abierta() {
                        m.vida = m.vida.saturating_sub(if gorda { 3 } else { 1 });
                        m.golpe = 0.15;
                        pega = true;
                    }
                }
            }
            if pega {
                self.puntos += 15;
                self.eventos.push(Efecto::Don);
            }
            if gasto {
                self.explotar(pos, 4, [1.0, 0.9, 0.6], 5.0);
                self.balas.remove(b);
            } else {
                b += 1;
            }
        }
        // La tormenta: escupe rocas hacia la nave; si se le acaba la vida, revienta.
        let mut nuevas = Vec::new();
        let lento = if self.poder[3] > 0.0 { 0.4 } else { 1.0 };
        let oleada = self.oleada;
        if let Some(m) = &mut self.mancha {
            m.t += dt;
            m.golpe = (m.golpe - dt).max(0.0);
            m.pos = Vec3::new((m.t * 0.4).sin() * 8.0, 2.0 + (m.t * 0.6).sin() * 3.0, -95.0);
            m.escupe -= dt * lento;
            if m.escupe <= 0.0 {
                m.escupe = (1.6 - oleada as f32 * 0.04).max(0.7);
                nuevas.push(m.pos);
            }
        }
        for pos in nuevas {
            let clase = if self.azar() < 0.25 { Clase::Roja } else { Clase::Gris };
            let mut r = self.roca(pos, 1, clase);
            r.vel = Vec3::new((self.nave.x - pos.x) * 0.25, (self.nave.y - pos.y) * 0.25, 0.0);
            self.rocas.push(r);
            self.eventos.push(Efecto::Fuego);
        }
        if self.mancha.as_ref().is_some_and(|m| m.vida == 0) {
            if let Some(m) = self.mancha.take() {
                self.explotar(m.pos, 140, [1.0, 0.35, 0.15], 30.0);
                self.puntos += 2500;
                self.jefes += 1;
                self.letreros.push(Letrero { pos: m.pos, texto: "+2500 ¡TORMENTA CALMADA!".into(), vida: 2.0 });
                self.destello = 1.0;
                self.sacudida = 1.0;
                self.decir(JEFE[1]);
                self.vida.exito("¡TORMENTA CALMADA!", "+2500", Tono::Oro);
                self.eventos.push(Efecto::Perfecto);
                self.tiempo_oleada = OLEADA;
            }
        }
        // Las lunas: se atrapan volando hacia ellas.
        let mut l = 0;
        while l < self.lunas.len() {
            let d = self.lunas[l].pos - self.nave;
            if d.z > -2.0 && Vec3::new(d.x, d.y, 0.0).length() < 2.6 {
                let cual = self.lunas.remove(l).cual;
                self.poder[cual] = DURA[cual];
                if cual == 1 {
                    self.escudo = (self.escudo + 0.5).min(1.0);
                }
                self.recogidas[cual] += 1;
                let pos = self.nave + Vec3::new(0.0, 1.5, -12.0);
                self.letreros.push(Letrero { pos, texto: format!("¡{}! {}", LUNAS[cual].to_uppercase(), LUNAS_QUE_HACEN[cual]), vida: 1.8 });
                self.destello = self.destello.max(0.25);
                let luna = format!("¡{}!", LUNAS[cual].to_uppercase());
                self.vida.grito(&luna, Tono::Cian, Tam::Grande);
                self.eventos.push(Efecto::Sello);
            } else if self.lunas[l].pos.z > 5.0 {
                self.lunas.remove(l);
            } else {
                l += 1;
            }
        }
        // Las rocas contra la nave (cuando ya la alcanzaron).
        let mut i = 0;
        while i < self.rocas.len() {
            let r = &self.rocas[i];
            let d = r.pos - self.nave;
            if d.z > -1.5 && d.z < 1.5 && Vec3::new(d.x, d.y, 0.0).length() < RADIO[r.tam] * 0.8 + 0.9 {
                let r = self.rocas.remove(i);
                let pos = r.pos;
                self.explotar(pos, 30, [1.0, 0.6, 0.3], 12.0);
                self.combo = 0;
                if self.poder[1] > 0.0 {
                    // El hielo de Europa aguanta.
                    let pos = self.nave + Vec3::new(0.0, 0.0, -10.0);
                    self.letreros.push(Letrero { pos, texto: "¡EL HIELO AGUANTÓ!".into(), vida: 1.2 });
                    self.eventos.push(Efecto::Protesto);
                } else {
                    self.escudo -= GOLPE * if r.clase == Clase::Roja { 1.6 } else { 1.0 } * if r.tam == 2 { 0.6 } else { 1.0 };
                    self.sacudida = 1.0;
                    self.rojo = 1.0;
                    self.vida.grito("¡AUCH!", Tono::Rojo, Tam::Medio);
                    self.eventos.push(Efecto::Choque);
                    if self.escudo <= 0.0 {
                        self.escudo = 0.0;
                        self.vida.desastre("¡NAVE DESTRUIDA!", "La ciencia no perdona");
                        self.fase = Fase::Choque(0.0);
                        let pos = self.nave + Vec3::new(0.0, 0.0, -4.0);
                        self.explotar(pos, 90, [1.0, 0.5, 0.8], 20.0);
                        self.poder = [0.0; 4];
                    }
                }
            } else {
                i += 1;
            }
        }
        AccionJupiter::Nada
    }

    fn mover_cosas(&mut self, dt: f32) {
        let v = self.rapidez();
        let vel_fondo = Vec3::new(0.0, 0.0, v);
        for r in &mut self.rocas {
            r.pos = r.pos + (vel_fondo + r.vel) * dt;
            r.giro += dt * (1.2 + r.tam as f32 * 0.5);
        }
        self.rocas.retain(|r| r.pos.z < 12.0);
        for b in &mut self.balas {
            b.pos = b.pos + b.vel * dt;
            b.vida -= dt;
        }
        self.balas.retain(|b| b.vida > 0.0);
        for l in &mut self.lunas {
            l.pos = l.pos + vel_fondo * 0.6 * dt;
        }
        for c in &mut self.chispas {
            c.pos = c.pos + (c.vel + vel_fondo * 0.5) * dt;
            c.vel = c.vel * (1.0 - 2.0 * dt);
            c.vida -= dt;
        }
        self.chispas.retain(|c| c.vida > 0.0);
        if self.chispas.len() > 1200 {
            let sobran = self.chispas.len() - 1200;
            self.chispas.drain(0..sobran);
        }
        for l in &mut self.letreros {
            l.pos = l.pos + Vec3::new(0.0, 1.5 * dt, 0.0);
            l.vida -= dt;
        }
        self.letreros.retain(|l| l.vida > 0.0);
        self.anuncio = (self.anuncio - dt).max(0.0);
    }

    pub fn matriz_roca(r: &Roca) -> Mat4 {
        Mat4::translation(r.pos) * Mat4::rotation_y(r.giro * r.eje.y * 1.5 + r.eje.x) * Mat4::rotation_x(r.giro * r.eje.x) * Mat4::scale(RADIO[r.tam])
    }

    /// La camara: en la cabina, mirando al frente, ladeada al virar y temblando con los golpes.
    pub fn camara(&self) -> (Vec3, Vec3, Vec3) {
        let s = self.sacudida * 0.35;
        let temblor = Vec3::new((self.total * 53.0).sin() * s, (self.total * 47.0).cos() * s, 0.0);
        let ojo = self.nave + temblor;
        let mira = ojo + Vec3::new(self.vel.x * 0.02, self.vel.y * 0.02, -10.0);
        let arriba = Vec3::new(self.ladeo.sin(), self.ladeo.cos(), 0.0);
        (ojo, mira, arriba)
    }

    /// De la cabina al mundo: la cabina va pegada a la camara (sus coordenadas son las de la
    /// camara: el origen en el ojo, mirando a -Z).
    pub fn cabina(&self) -> Mat4 {
        let (ojo, mira, arriba) = self.camara();
        let adelante = (mira - ojo).normalize();
        let derecha = adelante.cross(arriba).normalize();
        let arriba = derecha.cross(adelante);
        Mat4::translation(ojo) * Mat4::from_basis(derecha, arriba, adelante * -1.0)
    }

    /// La palanca de mando: (de lado, hacia el piloto), con el viraje.
    pub fn palanca(&self) -> (f32, f32) {
        (-self.vel.x / 26.0 * 0.35, self.vel.y / 26.0 * 0.3)
    }

    /// El bamboleo de la cabeza del muñequito del tablero: (de lado, adelante).
    pub fn cabezon(&self) -> (f32, f32) {
        (self.cabezon[0], self.cabezon[2])
    }

    /// Para las capturas: se salta la intro y juega solo `t` segundos (moviendose y disparando).
    pub fn jugar_solo(&mut self, t: f32, jefe: bool) {
        self.fase = Fase::Jugando;
        if jefe {
            self.oleada = 2;
        }
        self.siguiente_oleada();
        self.poder[0] = 30.0;
        self.poder[1] = 30.0;
        let mut tiempo = 0.0;
        while tiempo < t {
            let e = Entrada { girar: (tiempo * 0.7).sin(), cabecear: (tiempo * 0.5).cos() * 0.6, disparando: true, ..Default::default() };
            self.actualizar(&e, 1.0 / 60.0);
            tiempo += 1.0 / 60.0;
        }
    }

    /// Las notas que da la partida (una por cada 50 puntos).
    pub fn notas_ganadas(&self) -> u32 {
        self.puntos / 50
    }

    /// La pantalla izquierda: el escudo en diez segmentos (parpadea cuando ya casi no queda) y
    /// abajo los cuatro poderes de las lunas, con su anillo que se va vaciando.
    fn pantalla_escudo(&self, d: &mut RaylibDrawHandle, p: &Pantalla) {
        let llenos = (self.escudo * 10.0).ceil() as usize;
        let c = if self.escudo > 0.5 { (120, 255, 220) } else if self.escudo > 0.25 { (255, 200, 80) } else { (255, 80, 90) };
        let parpadeo = self.escudo > 0.25 || (self.total * 8.0).sin() > 0.0;
        for k in 0..10 {
            let u0 = 0.07 + k as f32 * 0.086;
            let a = if k < llenos && parpadeo { 235 } else { 35 };
            p.cuadro(d, u0, 0.16, u0 + 0.07, 0.46, Color::new(c.0, c.1, c.2, a));
        }
        let r = p.alto() * 0.12;
        for k in 0..4 {
            let centro = p.en(0.17 + k as f32 * 0.22, 0.73);
            let cc = COLOR_LUNA[k];
            let color = |a: u8| Color::new((cc[0] * 255.0) as u8, (cc[1] * 255.0) as u8, (cc[2] * 255.0) as u8, a);
            if self.poder[k] > 0.0 {
                d.draw_circle_v(centro, r * 0.75, color(240));
                d.draw_ring(centro, r * 0.95, r * 1.2, -90.0, -90.0 + 360.0 * (self.poder[k] / DURA[k]).min(1.0), 32, color(255));
            } else {
                d.draw_ring(centro, r * 0.95, r * 1.08, 0.0, 360.0, 32, color(45));
            }
        }
    }

    /// El radar del centro, visto desde arriba: lo de lado va de lado y lo lejos va arriba. Las
    /// rocas en gris, rojo u oro (las grandes mas grandes), las lunas de su color, la tormenta como
    /// una espiral roja; y una barrita arriba con la vida de la tormenta.
    fn pantalla_radar(&self, d: &mut RaylibDrawHandle, p: &Pantalla) {
        let rejilla = Color::new(90, 220, 230, 40);
        for v in [0.25, 0.5, 0.75] {
            p.linea(d, (0.03, v), (0.97, v), 1.0, rejilla);
        }
        p.linea(d, (0.5, 0.05), (0.5, 0.95), 1.0, rejilla);
        // el barrido, de lado a lado
        let barrido = 0.5 + 0.47 * (self.total * 1.6).sin();
        for k in 0..6u8 {
            let u = barrido - (self.total * 1.6).cos().signum() * k as f32 * 0.008;
            p.linea(d, (u, 0.04), (u, 0.96), 2.0, Color::new(120, 255, 240, 70 - k * 11));
        }
        let lugar = |pos: Vec3| -> Option<Vector2> {
            let u = 0.5 + (pos.x - self.nave.x) / (LADO * 4.4);
            let v = 0.94 + pos.z / LEJOS * 0.88;
            ((0.02..=0.98).contains(&u) && (0.04..=0.98).contains(&v)).then(|| p.en(u, v))
        };
        let r = p.alto() * 0.035;
        for roca in &self.rocas {
            let c = match roca.clase {
                Clase::Gris => Color::new(200, 210, 220, 220),
                Clase::Roja => Color::new(255, 90, 60, 255),
                Clase::Dorada => Color::new(255, 210, 80, 255),
            };
            if let Some(q) = lugar(roca.pos) {
                d.draw_circle_v(q, r * (1.0 + (2 - roca.tam.min(2)) as f32 * 0.5), c);
            }
        }
        for l in &self.lunas {
            let cc = COLOR_LUNA[l.cual];
            if let Some(q) = lugar(l.pos) {
                let rr = r * (1.6 + 0.4 * (self.total * 8.0).sin());
                d.draw_poly(q, 4, rr, 45.0, Color::new((cc[0] * 255.0) as u8, (cc[1] * 255.0) as u8, (cc[2] * 255.0) as u8, 255));
            }
        }
        if let Some(m) = &self.mancha {
            if let Some(q) = lugar(m.pos) {
                let rr = r * 4.0;
                d.draw_ring(q, rr * 0.7, rr, m.t * 200.0, m.t * 200.0 + 270.0, 24, Color::new(255, 80, 50, 220));
                if m.abierta() {
                    d.draw_circle_v(q, rr * 0.45, Color::new(255, 220, 90, 255));
                }
            }
            let vida = m.vida as f32 / m.vida_max as f32;
            p.cuadro(d, 0.20, 0.03, 0.80, 0.09, Color::new(120, 30, 20, 160));
            p.cuadro(d, 0.20, 0.03, 0.20 + 0.60 * vida, 0.09, Color::new(255, 90, 50, 255));
        }
        // la nave: un triangulito abajo al centro
        let (a, b, c) = (p.en(0.5, 0.86), p.en(0.475, 0.97), p.en(0.525, 0.97));
        triangulo(d, a, b, c, Color::new(200, 160, 255, 255));
    }

    /// La pantalla derecha: los puntos grandotes y, si va en racha, el multiplicador (si no, una
    /// rayita de que tanto falta para el record).
    fn pantalla_puntos(&self, d: &mut RaylibDrawHandle, p: &Pantalla) {
        let tam = (p.alto() * 0.36) as i32;
        let puntos = format!("{:06}", self.puntos);
        let centro = p.en(0.5, 0.40);
        let w = ui::medir(&puntos, tam);
        ui::texto(d, &puntos, centro.x as i32 - w / 2, centro.y as i32 - tam / 2, tam, Color::new(255, 225, 245, 240));
        if self.combo >= 5 {
            let s = format!("x{}", 1 + (self.combo / 5).min(4));
            let t2 = tam * 3 / 4;
            let c = p.en(0.5, 0.78);
            let w = ui::medir(&s, t2);
            ui::texto(d, &s, c.x as i32 - w / 2, c.y as i32 - t2 / 2, t2, Color::new(255, 140, 210, 255));
        } else if self.record > 0 {
            let k = (self.puntos as f32 / self.record as f32).min(1.0);
            p.cuadro(d, 0.20, 0.74, 0.80, 0.80, Color::new(255, 200, 240, 40));
            p.cuadro(d, 0.20, 0.74, 0.20 + 0.60 * k, 0.80, if k >= 1.0 { Color::new(255, 220, 90, 255) } else { Color::new(255, 150, 220, 200) });
        }
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, proyectar: &dyn Fn(Vec3) -> Option<(f32, f32)>, pantallas: &[[Vec3; 4]; 3], ancho: i32, alto: i32) {
        let (w, h) = (ancho as f32, alto as f32);
        if self.rojo > 0.0 {
            d.draw_rectangle(0, 0, ancho, alto, Color::new(255, 40, 40, (self.rojo * 70.0) as u8));
        }
        // La mira: al centro, donde van las balas (a MIRA de distancia).
        if self.jugando() {
            if let Some((cx, cy)) = proyectar(self.nave + Vec3::new(0.0, 0.0, -MIRA)) {
                let r = 16.0 + 3.0 * (self.total * 6.0).sin().abs();
                let c = Color::new(120, 255, 230, 230);
                d.draw_circle_lines(cx as i32, cy as i32, r, c);
                for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
                    d.draw_line_ex(Vector2::new(cx + dx * r * 0.5, cy + dy * r * 0.5), Vector2::new(cx + dx * r * 1.5, cy + dy * r * 1.5), 2.0, c);
                }
                d.draw_circle(cx as i32, cy as i32, 2.0, c);
            }
        }
        // El HUD va adentro de las pantallas del tablero (se proyectan sus esquinas, asi queda
        // pegado al vidrio): el escudo y los poderes a la izquierda, el radar al centro y los
        // puntos a la derecha. Sin letreros: todo se lee con colores y figuras.
        let cab = self.cabina();
        let vidrio = |k: usize| -> Option<Pantalla> {
            let mut q = [Vector2::zero(); 4];
            for (n, p) in pantallas[k].iter().enumerate() {
                let (x, y) = proyectar(cab.transform_point(*p))?;
                q[n] = Vector2::new(x, y);
            }
            Some(Pantalla(q))
        };
        if let Some(p) = vidrio(0) {
            self.pantalla_escudo(d, &p);
        }
        if let Some(p) = vidrio(1) {
            self.pantalla_radar(d, &p);
        }
        if let Some(p) = vidrio(2) {
            self.pantalla_puntos(d, &p);
        }
        // Los letreros de puntos, donde paso.
        for l in &self.letreros {
            if let Some((x, y)) = proyectar(l.pos) {
                let a = (l.vida.min(0.4) / 0.4 * 255.0) as u8;
                let tam = if l.texto.len() > 10 { 22 } else { 20 };
                let wt = ui::medir_titulo(&l.texto, tam);
                ui::titulo_de(d, &l.texto, x as i32 - wt / 2, y as i32, tam, Color::new(255, 240, 160, a));
            }
        }
        if self.anuncio > 0.0 && self.oleada > 0 {
            let a = (self.anuncio.min(0.5) / 0.5 * 255.0) as u8;
            let s = if self.oleada % 3 == 0 { format!("OLEADA {} · ¡JEFE!", self.oleada) } else { format!("OLEADA {}", self.oleada) };
            let tam = (h * 0.09) as i32;
            let wt = ui::medir_titulo(&s, tam);
            ui::titulo_de(d, &s, (ancho - wt) / 2, (h * 0.3) as i32, tam, Color::new(255, 210, 140, a));
        }
        if self.fase == Fase::Fin {
            let pw = 560.min(ancho - 40);
            let (px, py) = ((ancho - pw) / 2, (h * 0.22) as i32);
            ui::panel(d, px, py, pw, 260);
            ui::texto(d, "FIN DE LA PARTIDA", px + 24, py + 18, 32, Color::new(255, 200, 120, 255));
            ui::texto(d, &format!("Puntos: {}", self.puntos), px + 24, py + 66, 22, Color::WHITE);
            ui::texto(d, &format!("Oleada: {}  ·  Tormentas: {}  ·  Doradas: {}", self.oleada, self.jefes, self.dorados), px + 24, py + 96, 18, ui::TENUE);
            ui::texto(d, &format!("Notas ganadas: {}", self.notas_ganadas()), px + 24, py + 124, 18, Color::new(255, 150, 220, 255));
            let lunas: Vec<String> = (0..4).filter(|&k| self.recogidas[k] > 0).map(|k| format!("{} x{}", LUNAS[k], self.recogidas[k])).collect();
            let lunas = if lunas.is_empty() { "ninguna (otra vez será)".to_string() } else { lunas.join(", ") };
            ui::parrafo(d, &format!("Lunas para la mochila: {lunas}"), px + 24, py + 152, 17, pw - 48, ui::TEXTO);
            let otra = format!("{}: otra partida      {}: despegar", ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("aterrizar")));
            ui::texto(d, &otra, px + 24, py + 222, 18, Color::new(255, 230, 160, 255));
        } else if self.jugando() && self.oleada <= 1 && self.tiempo_oleada < 10.0 {
            let ayuda = format!("{}: moverse y apuntar   {}: disparar   {}: salir", if e.hay_control { ui::boton("stick") } else { format!("{} y {}", ui::boton("A / D"), ui::boton("flechas")) }, ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("aterrizar")));
            ui::texto(d, &ayuda, (w - ui::medir(&ayuda, 14) as f32) as i32 / 2, alto - 22, 14, Color::new(200, 190, 230, 200));
        }
    }
}

/// Una pantalla del tablero ya proyectada: sus cuatro esquinas en la ventana (arriba izquierda,
/// arriba derecha, abajo derecha, abajo izquierda). Lo de adentro se ubica con (u, v) en [0, 1].
struct Pantalla([Vector2; 4]);

impl Pantalla {
    fn en(&self, u: f32, v: f32) -> Vector2 {
        let [a, b, c, dd] = self.0;
        let mezcla = |p: Vector2, q: Vector2, k: f32| Vector2::new(p.x + (q.x - p.x) * k, p.y + (q.y - p.y) * k);
        mezcla(mezcla(a, b, u), mezcla(dd, c, u), v)
    }

    /// Que tan alta se ve (en pixeles), para escalar lo de adentro.
    fn alto(&self) -> f32 {
        let (a, b) = (self.en(0.5, 0.0), self.en(0.5, 1.0));
        ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
    }

    /// Un rectangulo de la pantalla (sigue la perspectiva del vidrio).
    fn cuadro(&self, d: &mut RaylibDrawHandle, u0: f32, v0: f32, u1: f32, v1: f32, color: Color) {
        let (a, b, c, dd) = (self.en(u0, v0), self.en(u1, v0), self.en(u1, v1), self.en(u0, v1));
        triangulo(d, a, b, c, color);
        triangulo(d, a, c, dd, color);
    }

    fn linea(&self, d: &mut RaylibDrawHandle, a: (f32, f32), b: (f32, f32), grueso: f32, color: Color) {
        d.draw_line_ex(self.en(a.0, a.1), self.en(b.0, b.1), grueso, color);
    }
}

/// Un triangulo relleno en cualquier orden (raylib los quiere contra las manecillas).
fn triangulo(d: &mut RaylibDrawHandle, a: Vector2, b: Vector2, c: Vector2, color: Color) {
    if (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) > 0.0 {
        d.draw_triangle(a, c, b, color);
    } else {
        d.draw_triangle(a, b, c, color);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn las_rocas_se_parten_y_dan_puntos() {
        let mut j = Jupiter::new(9, 0);
        j.fase = Fase::Jugando;
        let r = j.roca(Vec3::new(0.0, 0.0, -50.0), 0, Clase::Gris);
        j.rocas.push(r);
        j.romper(0);
        assert_eq!(j.rocas.len(), 2, "la grande se parte en dos medianas");
        assert!(j.rocas.iter().all(|r| r.tam == 1));
        assert_eq!(j.puntos, PUNTOS[0]);
    }

    #[test]
    fn una_roca_que_llega_baja_el_escudo() {
        let mut j = Jupiter::new(3, 0);
        j.fase = Fase::Jugando;
        j.proxima = 99.0;
        let r = j.roca(Vec3::new(0.0, 0.0, -0.5), 1, Clase::Gris);
        j.rocas.push(Roca { vel: Vec3::zero(), ..r });
        j.actualizar(&Entrada::default(), 0.016);
        assert!(j.escudo < 1.0);
    }

    #[test]
    fn las_doradas_piden_un_dato_de_bill() {
        let mut j = Jupiter::new(5, 0);
        j.fase = Fase::Jugando;
        let r = j.roca(Vec3::new(0.0, 0.0, -50.0), 1, Clase::Dorada);
        j.rocas.push(r);
        j.romper(0);
        assert!(j.pide_dato && j.dorados == 1 && j.puntos >= 500);
    }
}
