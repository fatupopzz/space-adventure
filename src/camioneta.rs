//! Aterrizaje en el PLANETA CHAPIN: "¡HAY CAMPO!", la camioneta extraurbana. Se maneja una
//! camioneta (el bus escolar gringo pintado de colores) por la carretera a la capital, entre
//! ceibas, casitas de Antigua y los volcanes al fondo, con marimba.
//!
//! - Tres carriles: A / D (o el stick) cambian de carril; W le pisa, S frena.
//! - En las PARADAS (a la derecha) se levanta gente: hay que ir en el carril de la derecha y
//!   pasar despacio. Cada pasajero paga su pasaje al llegar a la terminal.
//! - Los TUC-TUCS van mas despacio, los PERROS se cruzan y los BACHES sacuden: chocar abolla la
//!   camioneta (con tres abolladuras llega la grua) y se baja gente asustada.
//! - Los TUMULOS (los tumulos de toda la vida) atraviesan la carretera: si se pasan rapido, la
//!   gente vuela y alguien se baja quejandose. Hay que frenar.
//! - Siempre hay campo: el bus dice 40, pero caben 60.
//!
//! El mundo no avanza: la camioneta se queda en z = 0 mirando hacia -Z y todo lo demas viene
//! hacia ella a su velocidad (la carretera se dibuja en tramos que se repiten).

use crate::entrada::Entrada;
use crate::matematica::{Mat4, Vec3};
use crate::obj::Modelo;
use crate::obra::Obra;
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;

/// La x del centro de cada carril y lo ancho de la carretera.
pub const CARRILES: [f32; 3] = [-2.4, 0.0, 2.4];
pub const ANCHO_CARRETERA: f32 = 7.6;
/// Lo largo de un tramo de carretera (se repite).
pub const TRAMO: f32 = 10.0;
/// Hasta donde aparecen las cosas, y de que tamano es la camioneta (medio ancho y medio largo).
const LEJOS: f32 = 150.0;
const MEDIO_ANCHO: f32 = 1.1;
const MEDIO_LARGO: f32 = 3.6;
/// Velocidades (metros por segundo): la de crucero, pisandole, frenando y la maxima en una parada.
const CRUCERO: f32 = 24.0;
const PISANDOLE: f32 = 33.0;
const MINIMA: f32 = 5.0;
const DESPACIO: f32 = 14.0;
/// Lo que mide la carretera hasta la terminal, y cuanto dura cada cosa.
pub const RUTA: f32 = 1200.0;
const ABOLLADURAS: u32 = 3;
/// Los que caben (segun el letrero) y los que de verdad caben.
const CAPACIDAD: u32 = 40;
const MAXIMO: u32 = 60;
/// El pasaje (quetzales por pasajero).
pub const PASAJE: u32 = 5;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tipo {
    TucTuc,
    Perro,
    Bache,
    Tumulo,
    /// Una parada con gente esperando.
    Parada(u32),
}

#[derive(Clone, Copy, Debug)]
pub struct Cosa {
    pub tipo: Tipo,
    pub x: f32,
    pub z: f32,
    /// El perro: hacia donde camina (en x). El tuc-tuc: su velocidad.
    pub mov: f32,
    /// Ya paso (o ya se choco, o ya se levanto a la gente).
    hecha: bool,
    /// En que carril aparecio (el perro no se sale de el).
    pub carril: usize,
}

/// Lo de la orilla (no choca): ceibas y casitas, a cada lado.
#[derive(Clone, Copy, Debug)]
pub struct Adorno {
    pub casita: bool,
    pub x: f32,
    pub z: f32,
    pub color: usize,
    pub giro: f32,
}

/// Lo ancho (la mitad) de cada cosa que choca, para los choques: el tamano de su modelo.
fn medio_ancho(t: Tipo) -> f32 {
    match t {
        Tipo::TucTuc => 0.7,
        Tipo::Perro => 0.3,
        Tipo::Bache => 0.7,
        _ => 0.0,
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Fase {
    Llegada(usize),
    Jugando,
    /// Se acabo: si llego a la terminal.
    Fin(bool),
}

pub enum AccionCamioneta {
    Nada,
    Despegar,
}

pub const AYUDANTE: &str = "AYUDANTE";

const LLEGADA: [(&str, &str, Voz); 3] = [
    (AYUDANTE, "¡Capital, capital, capital! ¡Hay campo, hay campo! ...Ah, usted es el chofer nuevo. Súbase, que ya vamos tarde.", Voz::Radio),
    (AYUDANTE, "A y D para cambiar de carril, W le pisa, S frena. La gente espera a la derecha: carril derecho y despacito, que si no, se pasa la parada.", Voz::Radio),
    (ui::NOMBRE_IA, "Ojo con los túmulos: si no frena, los pasajeros salen volando. Y con los tuc-tucs. Y con los perros. Y con los baches. Bienvenido a la carretera.", Voz::Ia),
];
const SUBEN: [&str; 4] = ["¡Súbase, súbase! ¡Atrás hay lugar!", "¡Córranse para atrás, que vienen más!", "¡Hay campo, hay campo!", "¡Pasaje, pasaje! Cinco quetzales, mija."];
const LLENOS: [&str; 3] = ["¡Ya no hay campo! ...Mentira, siempre hay campo. Apriétense.", "¡Tres por asiento, tres por asiento!", "¡El de la gallina, súbala a la parrilla!"];
const CHOQUES: [&str; 4] = ["¡El tuc-tuc! ¡Ese no estaba ahí!", "¡Cuidado, chofer! ¡Se bajaron tres del susto!", "¡Perro, perro, perro! ...No le pasó nada al perro. A la camioneta sí.", "¡Bache! Ahí se fue un amortiguador."];
const TUMULOS: [&str; 3] = ["¡TÚMULO, CHOFER! ¡Mi cabeza!", "¡Ay! ¡Se me cayó el tamal!", "¡Bájeme aquí, que yo mejor me voy a pie!"];

pub struct Camioneta {
    fase: Fase,
    t: f32,
    pub total: f32,
    carril: usize,
    /// La x de la camioneta (va suave hacia su carril) y lo que se ladea al cambiar.
    pub x: f32,
    pub ladeo: f32,
    pub vel: f32,
    /// Metros recorridos (y lo que avanza la carretera para dibujarla).
    pub recorrido: f32,
    pub pasajeros: u32,
    pub abolladuras: u32,
    invulnerable: f32,
    /// El brinco de la camioneta al pasar un tumulo o un bache (se apaga solo).
    pub brinco: f32,
    pub cosas: Vec<Cosa>,
    pub adornos: Vec<Adorno>,
    hasta_cosa: f32,
    hasta_parada: f32,
    hasta_adorno: f32,
    semilla: u32,
    tumulos_bien: u32,
    tumulos_mal: u32,
    mensaje: Option<(&'static str, String, Voz, f32)>,
    pub sacudida: f32,
    pub destello: f32,
    pub eventos: Vec<Efecto>,
    // Lo de toda la visita (para el pago y los logros).
    pub partidas: u32,
    pub llegadas: u32,
    pub mejor_pasajeros: u32,
    pub mejor_tumulos: u32,
    girar_antes: f32,
}

impl Camioneta {
    pub fn new(semilla: u32) -> Self {
        let mut c = Camioneta {
            fase: Fase::Llegada(0),
            t: 0.0,
            total: 0.0,
            carril: 1,
            x: CARRILES[1],
            ladeo: 0.0,
            vel: 0.0,
            recorrido: 0.0,
            pasajeros: 0,
            abolladuras: 0,
            invulnerable: 0.0,
            brinco: 0.0,
            cosas: Vec::new(),
            adornos: Vec::new(),
            hasta_cosa: 30.0,
            hasta_parada: 25.0,
            hasta_adorno: 0.0,
            semilla: semilla | 1,
            tumulos_bien: 0,
            tumulos_mal: 0,
            mensaje: None,
            sacudida: 0.0,
            destello: 0.0,
            eventos: Vec::new(),
            partidas: 0,
            llegadas: 0,
            mejor_pasajeros: 0,
            mejor_tumulos: 0,
            girar_antes: 0.0,
        };
        // La orilla ya llena desde el principio.
        while c.adornos.len() < 40 {
            c.poner_adorno(-(c.adornos.len() as f32) * 5.0 - 5.0);
        }
        let (q, t, v) = LLEGADA[0];
        c.decir(q, t, v);
        c
    }

    fn azar(&mut self) -> f32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) as f32 / 16_777_216.0
    }

    fn decir(&mut self, quien: &'static str, texto: &str, voz: Voz) {
        let largo = texto.chars().count() as f32;
        self.mensaje = Some((quien, texto.to_string(), voz, 2.2 + largo / 20.0));
    }

    fn cambiar(&mut self, f: Fase) {
        self.fase = f;
        self.t = 0.0;
    }

    /// Una ronda nueva (la misma visita).
    fn empezar(&mut self) {
        self.carril = 1;
        self.x = CARRILES[1];
        self.vel = CRUCERO * 0.5;
        self.recorrido = 0.0;
        self.pasajeros = 0;
        self.abolladuras = 0;
        self.invulnerable = 0.0;
        self.cosas.clear();
        self.hasta_cosa = 30.0;
        self.hasta_parada = 20.0;
        self.tumulos_bien = 0;
        self.tumulos_mal = 0;
        self.mensaje = None;
        self.cambiar(Fase::Jugando);
    }

    fn poner_adorno(&mut self, z: f32) {
        let lado = if self.azar() < 0.5 { -1.0 } else { 1.0 };
        let casita = self.azar() < 0.45;
        let x = lado * (6.2 + self.azar() * 7.0 + if casita { 0.5 } else { 0.0 });
        let color = (self.azar() * 5.0) as usize;
        let giro = if casita { if lado > 0.0 { -std::f32::consts::FRAC_PI_2 } else { std::f32::consts::FRAC_PI_2 } } else { self.azar() * 6.28 };
        self.adornos.push(Adorno { casita, x, z, color, giro });
    }

    /// Algo nuevo en la carretera, alla adelante. Nunca tapa los tres carriles con algo que choca.
    fn poner_cosa(&mut self) {
        let lejos = -LEJOS;
        let r = self.azar();
        let dificultad = (self.recorrido / RUTA).clamp(0.0, 1.0);
        if r < 0.14 {
            self.cosas.push(Cosa { tipo: Tipo::Tumulo, x: 0.0, z: lejos, mov: 0.0, hecha: false, carril: 1 });
            return;
        }
        // Uno o dos carriles con algo (con la dificultad, mas seguido dos). El que queda libre es
        // uno sin nada cerca de donde aparece (los tuc-tucs van despacio y se juntan con la fila
        // que sigue): asi siempre hay por donde pasar.
        let ocupado = |cosas: &[Cosa], c: usize| cosas.iter().any(|k| matches!(k.tipo, Tipo::TucTuc | Tipo::Perro | Tipo::Bache) && (k.x - CARRILES[c]).abs() < 1.6 && k.z < lejos + 45.0);
        let libres: Vec<usize> = (0..3).filter(|&c| !ocupado(&self.cosas, c)).collect();
        if libres.is_empty() {
            return;
        }
        let libre = libres[(self.azar() * libres.len() as f32) as usize % libres.len()];
        // Y no se tapan los otros dos si alguno ya traia algo cerca (quedaria una pared).
        let ya_hay = (0..3).any(|c| c != libre && ocupado(&self.cosas, c));
        let dos = !ya_hay && self.azar() < 0.25 + 0.35 * dificultad;
        for c in 0..3 {
            if c == libre || (!dos && c != (libre + 1) % 3) {
                continue;
            }
            let q = self.azar();
            let (tipo, mov) = if q < 0.45 {
                (Tipo::TucTuc, 9.0 + self.azar() * 5.0)
            } else if q < 0.7 {
                (Tipo::Bache, 0.0)
            } else {
                (Tipo::Perro, if self.azar() < 0.5 { 1.2 } else { -1.2 })
            };
            let z = lejos - self.azar() * 6.0;
            self.cosas.push(Cosa { tipo, x: CARRILES[c], z, mov, hecha: false, carril: c });
        }
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionCamioneta {
        self.t += dt;
        self.total += dt;
        self.sacudida = (self.sacudida - dt * 2.5).max(0.0);
        self.destello = (self.destello - dt * 3.0).max(0.0);
        self.brinco = (self.brinco - dt * 3.0).max(0.0);
        if let Some(m) = &mut self.mensaje {
            m.3 -= dt;
            if m.3 <= 0.0 {
                self.mensaje = None;
            }
        }
        if e.aterrizar {
            return AccionCamioneta::Despegar;
        }
        match self.fase {
            Fase::Llegada(i) => {
                // La camioneta avanza despacito mientras el ayudante explica.
                self.mover_mundo(6.0, dt);
                if (e.aceptar && self.t > 0.3) || self.mensaje.is_none() {
                    if i + 1 < LLEGADA.len() {
                        let (q, t, v) = LLEGADA[i + 1];
                        self.decir(q, t, v);
                        self.cambiar(Fase::Llegada(i + 1));
                    } else {
                        self.empezar();
                        self.eventos.push(Efecto::Salto);
                    }
                }
            }
            Fase::Jugando => self.jugar(e, dt),
            Fase::Fin(_) => {
                self.mover_mundo(self.vel, dt);
                self.vel = (self.vel - 12.0 * dt).max(0.0);
                if e.aceptar && self.t > 1.2 {
                    self.empezar();
                    self.eventos.push(Efecto::Salto);
                }
            }
        }
        AccionCamioneta::Nada
    }

    /// Lo que se mueve aunque no se este jugando: la carretera y la orilla.
    fn mover_mundo(&mut self, vel: f32, dt: f32) {
        self.recorrido += vel * dt;
        for a in &mut self.adornos {
            a.z += vel * dt;
        }
        self.adornos.retain(|a| a.z < 12.0);
        self.hasta_adorno -= vel * dt;
        while self.hasta_adorno <= 0.0 {
            self.hasta_adorno += 4.0 + self.azar() * 4.0;
            self.poner_adorno(-LEJOS - 10.0);
        }
    }

    fn jugar(&mut self, e: &Entrada, dt: f32) {
        // Cambiar de carril: al apretar (no se queda cambiando mientras se mantiene).
        let girar = e.girar;
        let a_la_izquierda = e.izquierda || (girar > 0.5 && self.girar_antes <= 0.5);
        let a_la_derecha = e.derecha || (girar < -0.5 && self.girar_antes >= -0.5);
        self.girar_antes = girar;
        if a_la_izquierda && self.carril > 0 {
            self.carril -= 1;
            self.eventos.push(Efecto::Carta);
        }
        if a_la_derecha && self.carril < 2 {
            self.carril += 1;
            self.eventos.push(Efecto::Carta);
        }
        let objetivo = CARRILES[self.carril];
        let antes = self.x;
        self.x += (objetivo - self.x) * (1.0 - (-9.0 * dt).exp());
        self.ladeo += (((self.x - antes) / dt.max(1e-4)) * 0.05 - self.ladeo) * (1.0 - (-8.0 * dt).exp());
        // La velocidad: sola va a la de crucero; W le pisa, S frena.
        let deseada = if e.empuje > 0.3 { PISANDOLE } else if e.empuje < -0.3 { MINIMA } else { CRUCERO };
        let k = if deseada < self.vel { 3.2 } else { 0.9 };
        self.vel += (deseada - self.vel) * (1.0 - (-k * dt).exp());
        self.invulnerable = (self.invulnerable - dt).max(0.0);
        self.mover_mundo(self.vel, dt);
        // Lo que viene en la carretera.
        self.hasta_cosa -= self.vel * dt;
        if self.hasta_cosa <= 0.0 && self.recorrido < RUTA - LEJOS {
            let dificultad = (self.recorrido / RUTA).clamp(0.0, 1.0);
            self.hasta_cosa = 26.0 - 10.0 * dificultad + self.azar() * 12.0;
            self.poner_cosa();
        }
        self.hasta_parada -= self.vel * dt;
        if self.hasta_parada <= 0.0 && self.recorrido < RUTA - LEJOS {
            self.hasta_parada = 120.0 + self.azar() * 60.0;
            let gente = 2 + (self.azar() * 6.0) as u32;
            self.cosas.push(Cosa { tipo: Tipo::Parada(gente), x: CARRILES[2], z: -LEJOS, mov: 0.0, hecha: false, carril: 2 });
        }
        let vel = self.vel;
        for c in &mut self.cosas {
            let propia = if c.tipo == Tipo::TucTuc { c.mov } else { 0.0 };
            c.z += (vel - propia) * dt;
            // El perro va y viene, pero sin salirse de su carril (no tapa el libre).
            if c.tipo == Tipo::Perro {
                c.x += c.mov * dt;
                if (c.x - CARRILES[c.carril]).abs() > 0.6 {
                    c.mov = -c.mov;
                    c.x = CARRILES[c.carril] + 0.6 * (c.x - CARRILES[c.carril]).signum();
                }
            }
        }
        // Los choques y las paradas.
        for i in 0..self.cosas.len() {
            let c = self.cosas[i];
            if c.hecha {
                continue;
            }
            let en_largo = c.z > -MEDIO_LARGO - 0.6 && c.z < MEDIO_LARGO;
            match c.tipo {
                Tipo::Tumulo if c.z > -MEDIO_LARGO => {
                    self.cosas[i].hecha = true;
                    self.brinco = 1.0;
                    if self.vel > 15.0 {
                        self.tumulos_mal += 1;
                        let bajan = self.pasajeros.min(1 + (self.vel > 25.0) as u32);
                        self.pasajeros -= bajan;
                        self.sacudida = 0.7;
                        self.eventos.push(Efecto::Don);
                        let frase = TUMULOS[(self.azar() * 3.0) as usize % 3];
                        self.decir("PASAJERO", frase, Voz::Grunido);
                    } else {
                        self.tumulos_bien += 1;
                        self.mejor_tumulos = self.mejor_tumulos.max(self.tumulos_bien);
                        self.eventos.push(Efecto::Ka);
                    }
                }
                Tipo::Parada(gente) if c.z > -2.0 => {
                    self.cosas[i].hecha = true;
                    if self.carril == 2 && self.vel < DESPACIO {
                        let antes = self.pasajeros;
                        self.pasajeros = (self.pasajeros + gente).min(MAXIMO);
                        self.cosas[i].tipo = Tipo::Parada(0);
                        self.eventos.push(Efecto::Nota3);
                        let frase = if antes < CAPACIDAD && self.pasajeros >= CAPACIDAD { LLENOS[0] } else if self.pasajeros > CAPACIDAD { LLENOS[(self.azar() * 2.0) as usize % 2 + 1] } else { SUBEN[(self.azar() * 4.0) as usize % 4] };
                        self.decir(AYUDANTE, frase, Voz::Radio);
                    } else if self.carril == 2 {
                        self.eventos.push(Efecto::Fallo);
                        self.decir(AYUDANTE, "¡Se pasó la parada, chofer! ¡Frene, frene! ...Bueno, ya ni modo.", Voz::Radio);
                    }
                }
                Tipo::TucTuc | Tipo::Perro | Tipo::Bache if en_largo && (c.x - self.x).abs() < MEDIO_ANCHO + medio_ancho(c.tipo) => {
                    self.cosas[i].hecha = true;
                    if c.tipo == Tipo::Bache {
                        // El bache no abolla: sacude, y alguien se baja enojado.
                        self.brinco = 0.8;
                        self.sacudida = 0.5;
                        self.pasajeros = self.pasajeros.saturating_sub(1);
                        self.eventos.push(Efecto::Don);
                        self.decir(AYUDANTE, CHOQUES[3], Voz::Radio);
                    } else if self.invulnerable <= 0.0 {
                        self.abolladuras += 1;
                        self.invulnerable = 1.5;
                        self.pasajeros = self.pasajeros.saturating_sub(3);
                        self.vel = self.vel.min(9.0);
                        self.sacudida = 1.0;
                        self.destello = 0.6;
                        self.eventos.push(Efecto::Choque);
                        let frase = if c.tipo == Tipo::Perro { CHOQUES[2] } else { CHOQUES[(self.azar() * 2.0) as usize % 2] };
                        self.decir(AYUDANTE, frase, Voz::Radio);
                        if self.abolladuras >= ABOLLADURAS {
                            self.terminar(false);
                            return;
                        }
                    }
                }
                _ => {}
            }
        }
        self.cosas.retain(|c| c.z < 14.0);
        if self.recorrido >= RUTA {
            self.terminar(true);
        }
    }

    fn terminar(&mut self, llego: bool) {
        self.partidas += 1;
        if llego {
            self.llegadas += 1;
            self.mejor_pasajeros = self.mejor_pasajeros.max(self.pasajeros);
            self.eventos.push(Efecto::Perfecto);
            let frase = if self.pasajeros >= CAPACIDAD {
                format!("¡Terminal! ¡Bajan, bajan! {} pasajeros, Q{}. Usted nació para esto, chofer.", self.pasajeros, self.pasajeros * PASAJE)
            } else {
                format!("¡Terminal de la capital! Bajan {} pasajeros: Q{}. Para la otra, más paradas.", self.pasajeros, self.pasajeros * PASAJE)
            };
            self.decir(AYUDANTE, &frase, Voz::Radio);
        } else {
            self.eventos.push(Efecto::Fallo);
            self.decir(AYUDANTE, "Tres abolladuras. Ya viene la grúa. Los pasajeros se fueron en otra camioneta. Me voy con ellos.", Voz::Radio);
        }
        self.cambiar(Fase::Fin(llego));
    }

    /// Los pasajeros que llegaron a la terminal en la ultima vuelta (si llego).
    #[cfg(test)]
    pub fn entregados(&self) -> Option<u32> {
        matches!(self.fase, Fase::Fin(true)).then_some(self.pasajeros)
    }

    #[cfg(test)]
    pub fn jugando(&self) -> bool {
        self.fase == Fase::Jugando
    }

    pub fn hablante(&self) -> Option<(&'static str, String, Voz)> {
        self.mensaje.as_ref().map(|(q, t, v, _)| (*q, t.clone(), *v))
    }

    pub fn despedida(&self) -> String {
        match (self.llegadas, self.partidas) {
            (0, 0) => "Despegue del Planeta Chapín. Nos fuimos sin manejar. El ayudante sigue gritando «¡hay campo!». Lo va a seguir gritando siempre.".into(),
            (0, _) => "Despegue del Planeta Chapín. La camioneta se quedó en el taller. El ayudante dice que la culpa fue del perro.".into(),
            _ => format!("Despegue del Planeta Chapín. Mejor viaje: {} pasajeros a la capital. El ayudante le guardó un shuco.", self.mejor_pasajeros),
        }
    }

    /// La camara: atras y arriba de la camioneta, siguiendola de lado (un poquito), con la sacudida.
    pub fn camara(&self) -> (Vec3, Vec3) {
        let temblor = Vec3::new((self.total * 47.0).sin(), (self.total * 53.0).cos(), 0.0) * (self.sacudida * 0.12);
        (Vec3::new(self.x * 0.55, 5.2, 10.5) + temblor, Vec3::new(self.x * 0.7, 1.4, -14.0))
    }

    /// La matriz de la camioneta: ladeada al cambiar de carril y brincando en los tumulos.
    pub fn matriz(&self) -> Mat4 {
        let salto = (self.brinco * std::f32::consts::PI).sin().abs() * 0.35 * self.brinco;
        let parpadeo = self.invulnerable > 0.0 && (self.total * 16.0) as i32 % 2 == 0;
        let escala = if parpadeo { 0.98 } else { 1.0 };
        Mat4::translation(Vec3::new(self.x, salto, 0.0)) * Mat4::rotation_z(-self.ladeo.clamp(-0.12, 0.12)) * Mat4::rotation_x(-self.brinco * 0.08) * Mat4::scale(escala)
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, ancho: i32, alto: i32) {
        let h = alto as f32;
        if matches!(self.fase, Fase::Llegada(0)) {
            let a = ((self.t / 0.8).min(1.0) * 255.0) as u8;
            let t1 = "¡HAY CAMPO!";
            let tam = (h * 0.09) as i32;
            ui::titulo_de(d, t1, (ancho - ui::medir_titulo(t1, tam)) / 2, (h * 0.1) as i32, tam, Color::new(255, 210, 70, a));
            let t2 = "la camioneta extraurbana · rumbo a la capital";
            ui::subtitulo(d, t2, (ancho - ui::medir(t2, 22)) / 2, (h * 0.1) as i32 + tam + 12, 22, Color::new(255, 255, 255, a));
        }
        if !matches!(self.fase, Fase::Llegada(_)) {
            ui::panel(d, 20, 20, 320, 104);
            let lleno = if self.pasajeros > CAPACIDAD { Color::new(255, 120, 120, 255) } else { ui::TEXTO };
            ui::texto(d, &format!("PASAJEROS {}/{}", self.pasajeros, CAPACIDAD), 36, 32, 22, lleno);
            ui::texto(d, &format!("Q{}  ·  {:.0} km/h", self.pasajeros * PASAJE, self.vel * 3.6), 36, 64, 20, Color::new(255, 220, 120, 255));
            // Las abolladuras que quedan (corazones de lamina).
            for k in 0..ABOLLADURAS {
                let x = 36 + k as i32 * 26;
                let color = if k < ABOLLADURAS - self.abolladuras { Color::new(255, 90, 120, 255) } else { Color::new(70, 50, 80, 255) };
                d.draw_circle(x + 5, 102, 6.0, color);
                d.draw_circle(x + 13, 102, 6.0, color);
                d.draw_triangle(Vector2::new(x as f32 - 1.0, 104.0), Vector2::new(x as f32 + 9.0, 114.0), Vector2::new(x as f32 + 19.0, 104.0), color);
            }
            // La ruta hasta la terminal.
            let (bx, by, bw) = (ancho / 2 - 220, 36, 440);
            d.draw_rectangle(bx, by + 6, bw, 6, Color::new(40, 30, 60, 220));
            let k = (self.recorrido / RUTA).clamp(0.0, 1.0);
            d.draw_rectangle(bx, by + 6, (bw as f32 * k) as i32, 6, Color::new(255, 200, 60, 255));
            d.draw_circle(bx + (bw as f32 * k) as i32, by + 9, 9.0, Color::new(80, 160, 255, 255));
            ui::subtitulo(d, "TERMINAL", bx + bw - ui::medir("TERMINAL", 14) / 2, by + 20, 14, ui::TENUE);
        }
        if let Fase::Fin(llego) = self.fase {
            let (w, hp) = (520.min(ancho - 40), 150);
            let (x, y) = ((ancho - w) / 2, (h * 0.28) as i32);
            ui::panel(d, x, y, w, hp);
            let titulo = if llego { "¡TERMINAL!" } else { "LLEGÓ LA GRÚA" };
            ui::titulo_de(d, titulo, x + (w - ui::medir_titulo(titulo, 40)) / 2, y + 16, 40, if llego { Color::new(255, 220, 90, 255) } else { Color::new(255, 120, 120, 255) });
            let linea = if llego { format!("{} pasajeros · Q{} · túmulos bien pasados: {}", self.pasajeros, self.pasajeros * PASAJE, self.tumulos_bien) } else { "Tres abolladuras. El perro está bien.".to_string() };
            ui::subtitulo(d, &linea, x + (w - ui::medir(&linea, 18)) / 2, y + 74, 18, ui::TEXTO);
            let mejor = format!("Mejor viaje: {} pasajeros", self.mejor_pasajeros);
            ui::subtitulo(d, &mejor, x + (w - ui::medir(&mejor, 16)) / 2, y + 106, 16, ui::TENUE);
        }
        let (aceptar, salir) = (e.nombre("aceptar"), e.nombre("aterrizar"));
        let mover = if e.hay_control { "stick" } else { "A / D" };
        let acciones: Vec<(&str, &str)> = match self.fase {
            Fase::Llegada(_) => vec![(aceptar, "Seguir"), (salir, "Despegar")],
            Fase::Fin(_) => vec![(aceptar, "Otra vuelta"), (salir, "Despegar")],
            Fase::Jugando => vec![(mover, "Carril"), ("W", "Pisarle"), ("S", "Frenar"), (salir, "Despegar")],
        };
        let pie = ui::alto_ayudas(&acciones, ancho);
        if let (Some((nombre, completo, _)), false) = (self.hablante(), texto.is_empty()) {
            let wd = 880.min(ancho - 40);
            let hd = ui::alto_dialogo(nombre, &completo, wd);
            let color = match nombre {
                AYUDANTE => Color::new(255, 200, 70, 255),
                "PASAJERO" => Color::new(255, 140, 140, 255),
                _ => ui::LILA,
            };
            ui::caja_dialogo(d, nombre, color, texto, (ancho - wd) / 2, alto - pie - hd - 12, wd);
        }
        ui::ayudas(d, &acciones, ancho, alto);
    }

    /// Para las capturas: `camioneta` (la llegada) y `camioneta_juego` (a media ruta, con gente).
    pub fn preparar_captura(&mut self, modo: &str) {
        if modo == "camioneta_juego" {
            self.empezar();
            self.pasajeros = 23;
            self.abolladuras = 1;
            for _ in 0..(60 * 9) {
                self.piloto_automatico(1.0 / 60.0);
            }
            self.mensaje = None;
        }
    }

    /// Un chofer automatico (para las pruebas y las capturas): esquiva lo que viene en su carril,
    /// frena en los tumulos y se pega a la derecha y frena cuando viene una parada.
    pub fn piloto_automatico(&mut self, dt: f32) {
        // Que tan peligroso es un carril (lo que choca, de un poco atras a bastante adelante).
        let peligro = |carril: usize, desde: f32, cosas: &[Cosa]| cosas.iter().any(|c| matches!(c.tipo, Tipo::TucTuc | Tipo::Perro | Tipo::Bache) && !c.hecha && c.z > desde && c.z < 6.0 && (c.x - CARRILES[carril]).abs() < 2.0);
        let parada_cerca = self.cosas.iter().any(|c| matches!(c.tipo, Tipo::Parada(n) if n > 0) && !c.hecha && c.z > -45.0);
        let parada_encima = self.cosas.iter().any(|c| matches!(c.tipo, Tipo::Parada(n) if n > 0) && !c.hecha && c.z > -22.0);
        let tumulo_cerca = self.cosas.iter().any(|c| c.tipo == Tipo::Tumulo && !c.hecha && c.z > -30.0);
        let mut e = Entrada::default();
        // Solo cambia cuando ya llego a su carril, y de uno en uno: al carril de al lado si esta
        // libre (para esquivar, o para irse a la derecha si viene una parada).
        let asentado = (self.x - CARRILES[self.carril]).abs() < 0.2;
        if asentado {
            let vecinos: Vec<usize> = [self.carril.wrapping_sub(1), self.carril + 1].into_iter().filter(|&c| c < 3 && !peligro(c, -26.0, &self.cosas)).collect();
            let destino = if peligro(self.carril, -34.0, &self.cosas) {
                vecinos.iter().copied().min_by_key(|&c| if peligro(c, -60.0, &self.cosas) { 1 } else { 0 })
            } else if parada_cerca && self.carril < 2 {
                vecinos.iter().copied().find(|&c| c > self.carril)
            } else {
                None
            };
            match destino {
                Some(c) if c < self.carril => e.izquierda = true,
                Some(c) if c > self.carril => e.derecha = true,
                _ => {}
            }
        }
        if tumulo_cerca || (parada_encima && self.carril == 2) {
            e.empuje = -1.0;
        }
        self.actualizar(&e, dt);
    }
}

// ---------------------------------------------------------------------------
// Los modelos (hechos aqui, con la obra: cajas y cuadrilateros)
// ---------------------------------------------------------------------------

pub struct Modelos {
    pub camioneta: Modelo,
    pub carretera: Modelo,
    pub tuctuc: Modelo,
    pub perro: Modelo,
    pub bache: Modelo,
    pub tumulo: Modelo,
    pub parada: Modelo,
    /// La gente (tres camisas distintas).
    pub personas: [Modelo; 3],
    pub volcanes: Modelo,
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// Un cono (de cuadrilateros) de radio `r0` abajo a `r1` arriba: los volcanes.
fn cono(o: &mut Obra, mat: &str, c: Vec3, r0: f32, r1: f32, alto: f32, lados: usize) {
    for k in 0..lados {
        let (a0, a1) = (k as f32 / lados as f32 * std::f32::consts::TAU, (k + 1) as f32 / lados as f32 * std::f32::consts::TAU);
        let p = |a: f32, r: f32, y: f32| c + v(a.cos() * r, y, a.sin() * r);
        let medio = (a0 + a1) * 0.5;
        let normal = v(medio.cos() * alto, r0 - r1, medio.sin() * alto).normalize();
        o.quad(mat, [p(a0, r0, 0.0), p(a1, r0, 0.0), p(a1, r1, alto), p(a0, r1, alto)], normal);
    }
}

pub fn modelos() -> Modelos {
    // La camioneta: el bus escolar con la trompa adelante (-Z), pintado de azul, amarillo y rojo,
    // con su parrilla llena de costales arriba.
    let mut o = Obra::new();
    o.material("lamina_azul", [0.12, 0.38, 0.95], [0.0; 3], 0.5, 40.0);
    o.material("lamina_roja", [0.92, 0.15, 0.18], [0.0; 3], 0.5, 40.0);
    o.material("lamina_amarilla", [1.0, 0.80, 0.10], [0.0; 3], 0.5, 40.0);
    o.material("vidrio", [0.10, 0.14, 0.20], [0.0; 3], 0.9, 80.0);
    o.material("llanta", [0.05, 0.05, 0.06], [0.0; 3], 0.1, 8.0);
    o.material("cromo", [0.85, 0.85, 0.9], [0.0; 3], 0.9, 90.0);
    o.material("faro", [1.0, 0.95, 0.75], [1.0, 0.9, 0.6], 0.0, 1.0);
    o.material("calavera", [0.9, 0.1, 0.1], [0.8, 0.05, 0.05], 0.0, 1.0);
    o.material("costal", [0.75, 0.62, 0.40], [0.0; 3], 0.0, 1.0);
    o.material("costal_verde", [0.30, 0.60, 0.30], [0.0; 3], 0.0, 1.0);
    o.caja("llanta", v(-1.05, 0.3, -3.5), v(1.05, 0.6, 3.5));
    o.caja("lamina_azul", v(-1.1, 0.6, -3.4), v(1.1, 1.4, 3.4));
    o.caja("lamina_amarilla", v(-1.12, 1.4, -3.4), v(1.12, 1.55, 3.4));
    o.caja("vidrio", v(-1.11, 1.55, -3.3), v(1.11, 2.2, 3.3));
    for k in 0..8 {
        let z = -3.3 + k as f32 * 0.94;
        o.caja("lamina_roja", v(-1.13, 1.55, z), v(1.13, 2.2, z + 0.12));
    }
    o.caja("lamina_roja", v(-1.1, 2.2, -3.4), v(1.1, 2.4, 3.4));
    o.caja("lamina_amarilla", v(-1.12, 0.9, -3.42), v(1.12, 1.0, 3.42));
    // La trompa, el parabrisas, la parrilla del frente y los faros.
    o.caja("lamina_amarilla", v(-1.0, 0.55, -4.6), v(1.0, 1.45, -3.4));
    o.caja("vidrio", v(-1.0, 1.55, -3.43), v(1.0, 2.15, -3.41));
    o.caja("cromo", v(-0.7, 0.65, -4.63), v(0.7, 1.2, -4.6));
    o.caja("cromo", v(-1.05, 0.45, -4.75), v(1.05, 0.6, -4.55));
    o.caja("faro", v(-0.95, 1.0, -4.64), v(-0.75, 1.2, -4.61));
    o.caja("faro", v(0.75, 1.0, -4.64), v(0.95, 1.2, -4.61));
    o.caja("calavera", v(-1.0, 0.8, 3.41), v(-0.7, 1.0, 3.44));
    o.caja("calavera", v(0.7, 0.8, 3.41), v(1.0, 1.0, 3.44));
    // Las llantas (dobles atras).
    for (z, ancho) in [(-3.4, 0.25), (1.5, 0.3), (2.5, 0.3)] {
        for lado in [-1.0, 1.0] {
            o.caja("llanta", v(lado * 1.05 - ancho * 0.5, 0.0, z - 0.4), v(lado * 1.05 + ancho * 0.5, 0.75, z + 0.4));
            o.caja("cromo", v(lado * (1.05 + ancho * 0.5), 0.25, z - 0.15), v(lado * (1.07 + ancho * 0.5), 0.5, z + 0.15));
        }
    }
    // La parrilla del techo con sus costales.
    for lado in [-1.0, 1.0] {
        o.caja("cromo", v(lado * 1.0 - 0.04, 2.4, -3.0), v(lado * 1.0 + 0.04, 2.7, 3.0));
    }
    for (k, (x, z)) in [(-0.5, -2.2), (0.3, -1.0), (-0.3, 0.4), (0.4, 1.6), (-0.4, 2.4)].iter().enumerate() {
        let mat = if k % 2 == 0 { "costal" } else { "costal_verde" };
        o.caja(mat, v(x - 0.4, 2.4, z - 0.5), v(x + 0.4, 2.85, z + 0.5));
    }
    let camioneta = o.modelo();

    // Un tramo de carretera: el asfalto, las rayas, el bordillo y el pasto.
    let mut o = Obra::new();
    o.material("asfalto", [0.20, 0.20, 0.23], [0.0; 3], 0.05, 4.0);
    o.material("raya", [1.0, 0.95, 0.85], [0.08, 0.08, 0.07], 0.0, 1.0);
    o.material("raya_amarilla", [1.0, 0.8, 0.1], [0.1, 0.08, 0.0], 0.0, 1.0);
    o.material("bordillo", [0.75, 0.72, 0.68], [0.0; 3], 0.0, 1.0);
    o.material("pasto", [0.30, 0.66, 0.28], [0.0; 3], 0.0, 1.0);
    o.material("tierra", [0.55, 0.38, 0.24], [0.0; 3], 0.0, 1.0);
    let h = ANCHO_CARRETERA * 0.5;
    let arriba = v(0.0, 1.0, 0.0);
    o.quad("asfalto", [v(-h, 0.0, 0.0), v(h, 0.0, 0.0), v(h, 0.0, -TRAMO), v(-h, 0.0, -TRAMO)], arriba);
    for x in [-1.2, 1.2] {
        o.quad("raya", [v(x - 0.08, 0.01, -1.0), v(x + 0.08, 0.01, -1.0), v(x + 0.08, 0.01, -4.5), v(x - 0.08, 0.01, -4.5)], arriba);
    }
    for x in [-h + 0.25, h - 0.25] {
        o.quad("raya_amarilla", [v(x - 0.07, 0.01, 0.0), v(x + 0.07, 0.01, 0.0), v(x + 0.07, 0.01, -TRAMO), v(x - 0.07, 0.01, -TRAMO)], arriba);
    }
    for lado in [-1.0, 1.0] {
        o.caja("bordillo", v(lado * h - 0.2, 0.0, -TRAMO), v(lado * h + 0.2, 0.18, 0.0));
        let (x0, x1) = if lado < 0.0 { (-40.0, -h - 0.2) } else { (h + 0.2, 40.0) };
        o.quad("pasto", [v(x0, -0.02, 0.0), v(x1, -0.02, 0.0), v(x1, -0.02, -TRAMO), v(x0, -0.02, -TRAMO)], arriba);
        o.quad("tierra", [v(lado * (h + 0.2), -0.01, 0.0), v(lado * (h + 1.2), -0.01, 0.0), v(lado * (h + 1.2), -0.01, -TRAMO), v(lado * (h + 0.2), -0.01, -TRAMO)], arriba);
    }
    let carretera = o.modelo();

    // El tuc-tuc (de espaldas: va para el mismo lado).
    let mut o = Obra::new();
    o.material("tuctuc", [0.95, 0.20, 0.20], [0.0; 3], 0.5, 40.0);
    o.material("techo_tuctuc", [0.08, 0.08, 0.09], [0.0; 3], 0.2, 8.0);
    o.material("llanta", [0.05, 0.05, 0.06], [0.0; 3], 0.1, 8.0);
    o.material("calavera", [0.9, 0.1, 0.1], [0.9, 0.08, 0.08], 0.0, 1.0);
    o.caja("tuctuc", v(-0.65, 0.3, -0.9), v(0.65, 1.15, 0.9));
    o.caja("techo_tuctuc", v(-0.7, 1.15, -0.95), v(0.7, 1.75, 0.95));
    o.caja("techo_tuctuc", v(-0.72, 1.75, -1.0), v(0.72, 1.85, 1.0));
    for (x, z) in [(-0.6, 0.6), (0.6, 0.6), (0.0, -0.8)] {
        o.caja("llanta", v(x - 0.12, 0.0, z - 0.25), v(x + 0.12, 0.45, z + 0.25));
    }
    o.caja("calavera", v(-0.55, 0.6, 0.91), v(-0.35, 0.75, 0.93));
    o.caja("calavera", v(0.35, 0.6, 0.91), v(0.55, 0.75, 0.93));
    let tuctuc = o.modelo();

    // El perro (callejero, cafe con manchas).
    let mut o = Obra::new();
    o.material("perro", [0.62, 0.42, 0.24], [0.0; 3], 0.0, 1.0);
    o.material("mancha", [0.30, 0.20, 0.12], [0.0; 3], 0.0, 1.0);
    o.caja("perro", v(-0.25, 0.35, -0.5), v(0.25, 0.75, 0.5));
    o.caja("perro", v(-0.2, 0.6, -0.85), v(0.2, 1.0, -0.45));
    o.caja("mancha", v(-0.1, 0.95, -0.75), v(0.1, 1.15, -0.55));
    for (x, z) in [(-0.17, -0.4), (0.17, -0.4), (-0.17, 0.4), (0.17, 0.4)] {
        o.caja("perro", v(x - 0.07, 0.0, z - 0.07), v(x + 0.07, 0.4, z + 0.07));
    }
    o.caja("mancha", v(-0.05, 0.7, 0.5), v(0.05, 1.0, 0.6));
    let perro = o.modelo();

    // El bache: un hoyo oscuro con la orilla rota.
    let mut o = Obra::new();
    o.material("hoyo", [0.05, 0.05, 0.06], [0.0; 3], 0.0, 1.0);
    o.material("orilla", [0.35, 0.33, 0.32], [0.0; 3], 0.0, 1.0);
    o.caja("orilla", v(-0.9, 0.0, -0.75), v(0.9, 0.03, 0.75));
    o.caja("hoyo", v(-0.7, 0.0, -0.55), v(0.7, 0.04, 0.55));
    let bache = o.modelo();

    // El tumulo: atraviesa la carretera, a rayas amarillas y negras.
    let mut o = Obra::new();
    o.material("tumulo_amarillo", [1.0, 0.8, 0.1], [0.12, 0.09, 0.0], 0.0, 1.0);
    o.material("tumulo_negro", [0.06, 0.06, 0.07], [0.0; 3], 0.0, 1.0);
    for k in 0..10 {
        let x0 = -ANCHO_CARRETERA * 0.5 + k as f32 * ANCHO_CARRETERA / 10.0;
        o.caja(if k % 2 == 0 { "tumulo_amarillo" } else { "tumulo_negro" }, v(x0, 0.0, -0.35), v(x0 + ANCHO_CARRETERA / 10.0, 0.2, 0.35));
    }
    let tumulo = o.modelo();

    // La parada: el poste con su letrero y una banquita, en la orilla derecha.
    let mut o = Obra::new();
    o.material("poste", [0.6, 0.6, 0.65], [0.0; 3], 0.6, 40.0);
    o.material("letrero", [0.15, 0.45, 0.95], [0.05, 0.12, 0.3], 0.3, 20.0);
    o.material("banca", [0.55, 0.35, 0.20], [0.0; 3], 0.0, 1.0);
    o.caja("poste", v(4.3, 0.0, -0.05), v(4.4, 2.6, 0.05));
    o.caja("letrero", v(4.0, 2.2, -0.03), v(4.7, 2.9, 0.03));
    o.caja("banca", v(4.7, 0.4, -1.2), v(5.3, 0.5, 1.2));
    o.caja("banca", v(5.2, 0.5, -1.2), v(5.3, 1.0, 1.2));
    let parada = o.modelo();

    // La gente esperando: tres camisas.
    let persona = |camisa: [f32; 3]| {
        let mut o = Obra::new();
        o.material("camisa", camisa, [0.0; 3], 0.0, 1.0);
        o.material("piel", [0.78, 0.56, 0.40], [0.0; 3], 0.0, 1.0);
        o.material("pantalon", [0.2, 0.25, 0.45], [0.0; 3], 0.0, 1.0);
        o.caja("pantalon", v(-0.18, 0.0, -0.12), v(0.18, 0.75, 0.12));
        o.caja("camisa", v(-0.24, 0.75, -0.15), v(0.24, 1.35, 0.15));
        o.caja("piel", v(-0.15, 1.35, -0.15), v(0.15, 1.65, 0.15));
        o.modelo()
    };
    let personas = [persona([0.95, 0.3, 0.4]), persona([0.3, 0.75, 0.95]), persona([0.95, 0.85, 0.3])];

    // Los volcanes al fondo (Agua, Fuego y Acatenango), con lava en el de Fuego.
    let mut o = Obra::new();
    o.material("volcan", [0.32, 0.36, 0.48], [0.0; 3], 0.0, 1.0);
    o.material("volcan_verde", [0.26, 0.48, 0.32], [0.0; 3], 0.0, 1.0);
    o.material("lava", [1.0, 0.4, 0.1], [1.0, 0.35, 0.05], 0.0, 1.0);
    for (x, z, r, alto, mat) in [(-70.0, -230.0, 70.0, 85.0, "volcan_verde"), (40.0, -260.0, 80.0, 110.0, "volcan"), (110.0, -240.0, 60.0, 80.0, "volcan")] {
        cono(&mut o, mat, v(x, -1.0, z), r, r * 0.12, alto, 14);
    }
    cono(&mut o, "lava", v(40.0, 108.0, -260.0), 9.0, 5.0, 4.0, 10);
    let volcanes = o.modelo();

    Modelos { camioneta, carretera, tuctuc, perro, bache, tumulo, parada, personas, volcanes }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// El chofer automatico llega a la terminal con gente (las paradas, los carriles y los frenos
    /// funcionan) en un tiempo razonable.
    #[test]
    fn el_chofer_llega_a_la_terminal_con_gente() {
        // Con varias semillas (que no sea suerte): siempre hay por donde pasar, y las paradas
        // levantan gente (en promedio; a veces la parada viene con trafico en la derecha).
        let mut total = 0;
        for semilla in [7, 13, 21, 42, 99, 2026] {
            let mut c = Camioneta::new(semilla);
            c.empezar();
            for _ in 0..(60 * 120) {
                c.piloto_automatico(1.0 / 60.0);
                if !c.jugando() {
                    break;
                }
            }
            assert!(c.entregados().is_some(), "semilla {semilla}: no llego: {} abolladuras a {} m", c.abolladuras, c.recorrido);
            total += c.pasajeros;
        }
        assert!(total >= 60, "en promedio llego casi vacio: {total} en seis viajes");
    }

    /// Ir rapido sobre un tumulo baja gente; frenando, no.
    #[test]
    fn el_tumulo_se_pasa_frenando() {
        for (frena, baja) in [(true, false), (false, true)] {
            let mut c = Camioneta::new(3);
            c.empezar();
            c.pasajeros = 10;
            c.vel = if frena { 8.0 } else { 30.0 };
            c.hasta_cosa = 1e9;
            c.hasta_parada = 1e9;
            c.cosas.push(Cosa { tipo: Tipo::Tumulo, x: 0.0, z: -4.0, mov: 0.0, hecha: false, carril: 1 });
            let e = Entrada { empuje: if frena { -1.0 } else { 1.0 }, ..Default::default() };
            for _ in 0..30 {
                c.actualizar(&e, 1.0 / 60.0);
            }
            assert_eq!(c.pasajeros < 10, baja, "frenando: {frena}");
        }
    }

    /// Chocar con un tuc-tuc abolla; con tres, se acaba (y no se llega).
    #[test]
    fn tres_choques_y_llega_la_grua() {
        let mut c = Camioneta::new(5);
        c.empezar();
        c.hasta_cosa = 1e9;
        c.hasta_parada = 1e9;
        for _ in 0..3 {
            c.invulnerable = 0.0;
            c.cosas.push(Cosa { tipo: Tipo::TucTuc, x: c.x, z: -2.0, mov: 0.0, hecha: false, carril: 1 });
            c.actualizar(&Entrada::default(), 1.0 / 60.0);
        }
        assert_eq!(c.abolladuras, 3);
        assert!(!c.jugando() && c.entregados().is_none());
    }
}
