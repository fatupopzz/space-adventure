//! Aterrizaje en el PLANETA CHAPIN: "¡HAY CAMPO!", la camioneta extraurbana. Se maneja una
//! camioneta (el bus escolar gringo pintado de colores) rumbo a la capital, con marimba y los
//! volcanes al fondo. La ruta cruza tres zonas mientras la tarde se vuelve noche:
//! - EL CAMPO: milpa, ceibas y los barriletes gigantes de Sumpango.
//! - ANTIGUA: calle de casas de colores, papel picado, iglesias y el Arco de Santa Catalina.
//! - LA CAPITAL: edificios con la luz prendida, vallas, la Torre del Reformador y la terminal.
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
pub const DESPACIO: f32 = 14.0;
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

/// Por donde va la ruta.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Zona {
    Campo,
    Antigua,
    Capital,
}

/// La zona a esa distancia de la salida.
pub fn zona_de(metros: f32) -> Zona {
    if metros < INICIO_ANTIGUA {
        Zona::Campo
    } else if metros < INICIO_CAPITAL {
        Zona::Antigua
    } else {
        Zona::Capital
    }
}

/// Donde empieza cada zona, y los puntos fijos de la ruta (se dibujan segun lo recorrido).
pub const INICIO_ANTIGUA: f32 = 430.0;
pub const INICIO_CAPITAL: f32 = 840.0;
pub const ARCO: f32 = 620.0;
pub const TORRE: f32 = 1010.0;

/// Lo de la orilla (no choca).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Deco {
    Ceiba,
    Milpa,
    Casita,
    Barrilete,
    Poste,
    Puesto,
    Casa,
    Iglesia,
    PapelPicado,
    Edificio,
    Valla,
    Farol,
}

#[derive(Clone, Copy, Debug)]
pub struct Adorno {
    pub tipo: Deco,
    pub x: f32,
    pub z: f32,
    pub color: usize,
}

/// Un cubito suelto: humo del escape, confeti de los pasajeros, latas de un choque.
#[derive(Clone, Copy, Debug)]
pub struct Chispa {
    pub pos: Vec3,
    pub vel: Vec3,
    pub vida: f32,
    pub tam: f32,
    pub color: usize,
    /// Si la jala la gravedad (el humo no: sube).
    pub cae: bool,
}

/// Los colores de los cubitos (en `Modelos::cubos`).
const HUMO: usize = 0;
const HOLLIN: usize = 1;
const CONFETI: [usize; 5] = [2, 3, 4, 5, 6];

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
    pub chispas: Vec<Chispa>,
    /// Si va frenando (las calaveras se prenden).
    pub frenando: bool,
    /// El letrero de lo que acaba de pasar (+Q25...): texto, color y segundos.
    aviso: Option<(String, Color, f32)>,
    /// El nombre de la zona a la que se acaba de entrar, y hace cuanto.
    zona: Zona,
    desde_zona: f32,
    hasta_humo: f32,
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
            chispas: Vec::new(),
            frenando: false,
            aviso: None,
            zona: Zona::Campo,
            desde_zona: 10.0,
            hasta_humo: 0.0,
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
        c.rellenar_orilla();
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
        self.chispas.clear();
        self.aviso = None;
        self.zona = Zona::Campo;
        self.desde_zona = 10.0;
        self.rellenar_orilla();
        self.hasta_cosa = 30.0;
        self.hasta_parada = 20.0;
        self.tumulos_bien = 0;
        self.tumulos_mal = 0;
        self.mensaje = None;
        self.cambiar(Fase::Jugando);
    }

    /// La orilla llena de aqui hasta donde se ve (al empezar, para que no salga vacia).
    fn rellenar_orilla(&mut self) {
        self.adornos.clear();
        let mut z = -4.0;
        while z > -LEJOS - 10.0 {
            z -= self.poner_adorno(z);
        }
        self.hasta_adorno = 0.0;
    }

    /// Lo que va en la orilla en `z`, segun la zona. Devuelve cuanto falta para el siguiente.
    fn poner_adorno(&mut self, z: f32) -> f32 {
        let zona = zona_de(self.recorrido - z);
        match zona {
            Zona::Campo => {
                let lado = if self.azar() < 0.5 { -1.0 } else { 1.0 };
                let (r, cerca, lejos, color) = (self.azar(), self.azar(), self.azar(), (self.azar() * 6.0) as usize);
                let (tipo, x) = if r < 0.28 {
                    (Deco::Ceiba, 8.0 + lejos * 12.0)
                } else if r < 0.58 {
                    (Deco::Milpa, 8.5 + lejos * 10.0)
                } else if r < 0.72 {
                    (Deco::Casita, 8.5 + cerca * 7.0)
                } else if r < 0.82 {
                    (Deco::Barrilete, 11.0 + lejos * 8.0)
                } else if r < 0.93 {
                    (Deco::Poste, 6.0)
                } else {
                    (Deco::Puesto, 7.0)
                };
                self.adornos.push(Adorno { tipo, x: lado * x, z, color });
                2.0 + self.azar() * 3.0
            }
            Zona::Antigua => {
                // La calle: casas pegadas de los dos lados, a veces una iglesia atras.
                for lado in [-1.0, 1.0] {
                    let color = (self.azar() * 6.0) as usize;
                    self.adornos.push(Adorno { tipo: Deco::Casa, x: lado * 9.2, z, color });
                    if self.azar() < 0.1 {
                        self.adornos.push(Adorno { tipo: Deco::Iglesia, x: lado * 22.0, z, color });
                    }
                }
                if self.azar() < 0.42 {
                    self.adornos.push(Adorno { tipo: Deco::PapelPicado, x: 0.0, z, color: 0 });
                }
                7.2
            }
            Zona::Capital => {
                for lado in [-1.0, 1.0] {
                    let color = (self.azar() * 4.0) as usize;
                    self.adornos.push(Adorno { tipo: Deco::Edificio, x: lado * 12.5, z, color });
                    let r = self.azar();
                    if r < 0.25 {
                        let chiste = (self.azar() * 3.0) as usize;
                        self.adornos.push(Adorno { tipo: Deco::Valla, x: lado * 7.2, z, color: chiste });
                    } else if r < 0.7 {
                        self.adornos.push(Adorno { tipo: Deco::Farol, x: lado * 5.8, z, color: 0 });
                    }
                }
                9.0
            }
        }
    }

    /// Suelta `n` cubitos de confeti (o lo que sea) desde `pos`.
    fn reventar(&mut self, pos: Vec3, n: usize, colores: &[usize], fuerza: f32) {
        for _ in 0..n {
            let vel = Vec3::new((self.azar() - 0.5) * fuerza, 2.0 + self.azar() * fuerza * 0.8, (self.azar() - 0.5) * fuerza);
            let color = colores[(self.azar() * colores.len() as f32) as usize % colores.len()];
            let tam = 0.09 + self.azar() * 0.1;
            let vida = 0.7 + self.azar() * 0.7;
            self.chispas.push(Chispa { pos, vel, vida, tam, color, cae: true });
        }
    }

    /// Los cubitos se mueven: el humo sube y crece, lo demas cae; y todo se queda atras.
    fn mover_chispas(&mut self, vel: f32, dt: f32) {
        for c in &mut self.chispas {
            c.pos = c.pos + c.vel * dt + Vec3::new(0.0, 0.0, vel * dt * 0.8);
            c.vida -= dt;
            if c.cae {
                c.vel.y -= 11.0 * dt;
                if c.pos.y < 0.05 {
                    c.pos.y = 0.05;
                    c.vel.y *= -0.45;
                }
            } else {
                c.tam += dt * 0.35;
            }
        }
        self.chispas.retain(|c| c.vida > 0.0 && c.pos.z < 13.0);
        self.chispas.truncate(160);
    }

    /// El humo negro del escape: mas si le pisa.
    fn echar_humo(&mut self, dt: f32) {
        self.hasta_humo -= dt;
        if self.hasta_humo > 0.0 {
            return;
        }
        let pisando = self.vel > CRUCERO + 2.0;
        self.hasta_humo = if pisando { 0.05 } else { 0.13 };
        let pos = Vec3::new(self.x + 0.72, 0.42, MEDIO_LARGO + 0.1);
        let vel = Vec3::new((self.azar() - 0.5) * 0.7, 0.7 + self.azar(), 1.0);
        let color = if pisando { HOLLIN } else { HUMO };
        let tam = 0.14 + self.azar() * 0.1;
        self.chispas.push(Chispa { pos, vel, vida: 0.75, tam, color, cae: false });
    }

    fn avisar(&mut self, texto: String, color: Color) {
        self.aviso = Some((texto, color, 0.0));
    }

    /// Para que tan de noche va: 0 en la salida, 1 en la terminal.
    pub fn atardecer(&self) -> f32 {
        (self.recorrido / RUTA).clamp(0.0, 1.0)
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
        self.desde_zona += dt;
        if let Some(a) = &mut self.aviso {
            a.2 += dt;
            if a.2 > 1.4 {
                self.aviso = None;
            }
        }
        self.mover_chispas(self.vel, dt);
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
            self.hasta_adorno += self.poner_adorno(-LEJOS - 10.0);
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
        self.frenando = e.empuje < -0.3;
        self.echar_humo(dt);
        let zona = zona_de(self.recorrido);
        if zona != self.zona {
            self.zona = zona;
            self.desde_zona = 0.0;
            self.eventos.push(Efecto::Perfecto);
        }
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
                        self.reventar(Vec3::new(self.x, 0.3, -MEDIO_LARGO), 8, &[3, HOLLIN], 3.5);
                        self.avisar(format!("-{bajan} POR EL TÚMULO"), Color::new(255, 130, 130, 255));
                        self.eventos.push(Efecto::Don);
                        let frase = TUMULOS[(self.azar() * 3.0) as usize % 3];
                        self.decir("PASAJERO", frase, Voz::Grunido);
                    } else {
                        self.tumulos_bien += 1;
                        self.avisar("¡SUAVECITO!".into(), Color::new(120, 235, 160, 255));
                        self.mejor_tumulos = self.mejor_tumulos.max(self.tumulos_bien);
                        self.eventos.push(Efecto::Ka);
                    }
                }
                Tipo::Parada(gente) if c.z > -2.0 => {
                    self.cosas[i].hecha = true;
                    if self.carril == 2 && self.vel < DESPACIO {
                        let antes = self.pasajeros;
                        self.pasajeros = (self.pasajeros + gente).min(MAXIMO);
                        let suben = self.pasajeros - antes;
                        self.reventar(Vec3::new(4.6, 1.2, 0.0), 26, &CONFETI, 6.0);
                        self.avisar(format!("+{suben} PASAJEROS · +Q{}", suben * PASAJE), Color::new(255, 220, 90, 255));
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
                        self.reventar(Vec3::new(c.x, 1.0, c.z), 18, &[2, 3, HOLLIN], 7.0);
                        self.avisar("¡ABOLLADURA!".into(), Color::new(255, 110, 120, 255));
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
            let x = self.x;
            self.reventar(Vec3::new(x, 3.0, -2.0), 70, &CONFETI, 11.0);
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
        // Mas abollada, mas chueca: cada golpe la deja un poquito ladeada.
        let chueca = self.abolladuras as f32 * 0.012;
        Mat4::translation(Vec3::new(self.x, salto, 0.0)) * Mat4::rotation_z(chueca - self.ladeo.clamp(-0.12, 0.12)) * Mat4::rotation_x(-self.brinco * 0.08) * Mat4::scale(escala)
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
            // A la derecha del panel de pasajeros, aunque la ventana sea angosta.
            let bx = (ancho / 2 - 220).max(370);
            let (by, bw) = (36, 440.min(ancho - bx - 70));
            d.draw_rectangle(bx, by + 6, bw, 6, Color::new(40, 30, 60, 220));
            let k = (self.recorrido / RUTA).clamp(0.0, 1.0);
            d.draw_rectangle(bx, by + 6, (bw as f32 * k) as i32, 6, Color::new(255, 200, 60, 255));
            d.draw_circle(bx + (bw as f32 * k) as i32, by + 9, 9.0, Color::new(80, 160, 255, 255));
            ui::subtitulo(d, "TERMINAL", bx + bw - ui::medir("TERMINAL", 14) / 2, by + 20, 14, ui::TENUE);
        }
        // El nombre de la zona, al entrar (como los cambios de mundo de los corredores infinitos).
        if self.fase == Fase::Jugando && self.desde_zona < 2.6 {
            let (nombre, sub) = match self.zona {
                Zona::Campo => ("EL CAMPO", "milpa y barriletes"),
                Zona::Antigua => ("ANTIGUA", "adoquín, papel picado y el Arco"),
                Zona::Capital => ("LA CAPITAL", "ya casi la terminal"),
            };
            let a = ((2.6 - self.desde_zona) / 0.5).clamp(0.0, 1.0) * (self.desde_zona / 0.25).min(1.0);
            let alfa = (a * 255.0) as u8;
            let tam = (h * 0.085) as i32;
            ui::titulo_de(d, nombre, (ancho - ui::medir_titulo(nombre, tam)) / 2, (h * 0.17) as i32, tam, Color::new(255, 225, 110, alfa));
            ui::subtitulo(d, sub, (ancho - ui::medir(sub, 20)) / 2, (h * 0.17) as i32 + tam + 8, 20, Color::new(255, 255, 255, alfa));
        }
        // Lo que acaba de pasar (+Q25, ¡ABOLLADURA!...): sube y se apaga.
        if let Some((s, c, t)) = &self.aviso {
            let alfa = ((1.4 - *t) / 0.4).clamp(0.0, 1.0);
            let tam = 34;
            let y = (h * 0.42) as i32 - (*t * 40.0) as i32;
            ui::titulo_de(d, s, (ancho - ui::medir_titulo(s, tam)) / 2, y, tam, Color::new(c.r, c.g, c.b, (alfa * 255.0) as u8));
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
        // `camioneta_juego` (el campo), `camioneta_antigua`, `camioneta_arco`, `camioneta_capital` y
        // `camioneta_llegada` (la terminal).
        let (desde, segundos) = match modo {
            "camioneta_juego" => (0.0, 9.0),
            "camioneta_antigua" => (450.0, 3.0),
            "camioneta_arco" => (570.0, 1.6),
            "camioneta_capital" => (880.0, 3.0),
            "camioneta_llegada" => (1110.0, 2.5),
            _ => return,
        };
        self.empezar();
        self.recorrido = desde;
        self.rellenar_orilla();
        self.pasajeros = 23;
        self.abolladuras = 1;
        for _ in 0..(segundos * 60.0) as usize {
            self.piloto_automatico(1.0 / 60.0);
        }
        self.mensaje = None;
    }

    /// Un chofer automatico (para las pruebas y las capturas): esquiva lo que viene en su carril,
    /// frena en los tumulos y se pega a la derecha y frena cuando viene una parada.
    pub fn piloto_automatico(&mut self, dt: f32) {
        let e = self.chofer();
        self.actualizar(&e, dt);
    }

    /// Lo que apretaria el chofer automatico en este cuadro (la grabacion lo usa en vivo).
    pub fn chofer(&self) -> Entrada {
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
        e
    }
}

// ---------------------------------------------------------------------------
// Los modelos (hechos aqui, con la obra: cajas y cuadrilateros)
// ---------------------------------------------------------------------------

pub struct Modelos {
    pub camioneta: Modelo,
    /// Las calaveras prendidas (se dibujan encima al frenar).
    pub frenos: Modelo,
    pub ayudante: Modelo,
    pub brazo: Modelo,
    pub gallina: Modelo,
    /// El suelo de cada zona, en dos versiones que se turnan (las franjas dan la velocidad).
    pub suelos: [[Modelo; 2]; 3],
    /// La alfombra de aserrin de Antigua, pintada sobre la calle.
    pub alfombra: Modelo,
    pub llano: Modelo,
    pub tuctuc: Modelo,
    pub perro: Modelo,
    pub bache: Modelo,
    pub tumulo: Modelo,
    pub senal_tumulo: Modelo,
    pub parada: Modelo,
    /// El carril de la parada pintado: verde si se va despacio, naranja si no.
    pub zona_lenta: Modelo,
    pub zona_rapida: Modelo,
    /// La gente (tres camisas distintas).
    pub personas: [Modelo; 3],
    pub volcanes: Modelo,
    pub lava: Modelo,
    pub nube: Modelo,
    pub sol: Modelo,
    pub cubos: [Modelo; 7],
    pub milpa: Modelo,
    pub casitas: [Modelo; 6],
    pub casas: [Modelo; 6],
    pub barriletes: [Modelo; 3],
    pub poste: Modelo,
    pub puesto: Modelo,
    pub iglesia: Modelo,
    pub papel: Modelo,
    pub edificios: [Modelo; 4],
    pub vallas: [Modelo; 3],
    pub farol: Modelo,
    pub arco: Modelo,
    pub torre: Modelo,
    pub portada_antigua: Modelo,
    pub portada_capital: Modelo,
    pub terminal: Modelo,
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// Toda la pintura del planeta: colores planos y vivos, con la paleta del juego. Lo que no se usa
/// en un modelo se descarta al armarlo.
fn paleta(o: &mut Obra) {
    const MATE: [(&str, [f32; 3]); 42] = [
        ("azul", [0.08, 0.32, 0.92]), ("azul_cielo", [0.22, 0.68, 0.98]), ("rojo", [0.92, 0.12, 0.20]), ("amarillo", [1.0, 0.76, 0.06]),
        ("naranja", [1.0, 0.46, 0.08]), ("verde", [0.10, 0.70, 0.30]), ("rosa", [1.0, 0.26, 0.60]), ("lila", [0.62, 0.34, 0.96]),
        ("turquesa", [0.08, 0.78, 0.72]), ("blanco", [0.95, 0.93, 0.88]), ("negro", [0.05, 0.05, 0.07]), ("llanta", [0.05, 0.05, 0.06]),
        ("costal", [0.75, 0.60, 0.36]), ("costal_verde", [0.28, 0.58, 0.28]), ("madera", [0.45, 0.26, 0.13]), ("madera_clara", [0.70, 0.46, 0.24]),
        ("colchon", [0.92, 0.55, 0.66]), ("tierra", [0.56, 0.37, 0.22]), ("tierra_oscura", [0.40, 0.26, 0.16]), ("asfalto", [0.21, 0.20, 0.27]),
        ("asfalto_b", [0.25, 0.24, 0.31]), ("adoquin", [0.39, 0.34, 0.42]), ("adoquin_b", [0.33, 0.29, 0.37]), ("concreto", [0.30, 0.28, 0.40]),
        ("concreto_b", [0.26, 0.24, 0.35]), ("pasto", [0.25, 0.60, 0.25]), ("pasto_b", [0.20, 0.52, 0.24]), ("pasto_lejos", [0.22, 0.50, 0.30]),
        ("flor", [1.0, 0.32, 0.62]), ("flor_b", [1.0, 0.84, 0.18]), ("milpa", [0.40, 0.70, 0.12]), ("elote", [1.0, 0.80, 0.20]),
        ("teja", [0.78, 0.30, 0.14]), ("teja_b", [0.62, 0.22, 0.12]), ("piel", [0.74, 0.50, 0.34]), ("pantalon", [0.16, 0.20, 0.42]),
        ("volcan_verde", [0.16, 0.44, 0.30]), ("volcan_azul", [0.30, 0.28, 0.52]), ("ceniza", [0.34, 0.30, 0.38]), ("hierro", [0.60, 0.30, 0.20]),
        ("gris", [0.52, 0.50, 0.58]), ("hueso", [0.60, 0.50, 0.38]),
    ];
    for (nombre, kd) in MATE {
        o.material(nombre, kd, [0.0; 3], 0.0, 1.0);
    }
    o.material("lamina_azul", [0.08, 0.30, 0.95], [0.0; 3], 0.55, 40.0);
    o.material("lamina_roja", [0.95, 0.10, 0.18], [0.0; 3], 0.55, 40.0);
    o.material("lamina_amarilla", [1.0, 0.74, 0.04], [0.0; 3], 0.55, 40.0);
    o.material("lamina_verde", [0.06, 0.66, 0.34], [0.0; 3], 0.55, 40.0);
    o.material("lamina_rosa", [1.0, 0.20, 0.55], [0.0; 3], 0.55, 40.0);
    o.material("vidrio", [0.10, 0.14, 0.24], [0.02, 0.03, 0.05], 0.9, 80.0);
    o.material("cromo", [0.85, 0.85, 0.92], [0.0; 3], 0.9, 90.0);
    o.material("faro", [1.0, 0.95, 0.75], [1.0, 0.9, 0.6], 0.0, 1.0);
    o.material("calavera", [0.55, 0.06, 0.08], [0.22, 0.02, 0.03], 0.3, 20.0);
    o.material("freno", [1.0, 0.12, 0.12], [1.0, 0.08, 0.08], 0.0, 1.0);
    o.material("ambar", [1.0, 0.65, 0.12], [1.0, 0.5, 0.05], 0.0, 1.0);
    o.material("luz_calida", [0.85, 0.68, 0.38], [0.62, 0.44, 0.18], 0.0, 1.0);
    o.material("luz_cian", [0.38, 0.80, 0.88], [0.16, 0.50, 0.60], 0.0, 1.0);
    o.material("luz_rosa", [1.0, 0.35, 0.72], [0.95, 0.22, 0.60], 0.0, 1.0);
    o.material("luz_lila", [0.72, 0.48, 1.0], [0.52, 0.28, 0.95], 0.0, 1.0);
    o.material("apagada", [0.10, 0.09, 0.20], [0.0; 3], 0.6, 40.0);
    o.material("lava", [1.0, 0.4, 0.1], [1.0, 0.35, 0.05], 0.0, 1.0);
    o.material("sol", [1.0, 0.92, 0.62], [1.0, 0.82, 0.45], 0.0, 1.0);
    o.material("nube", [1.0, 0.86, 0.90], [0.28, 0.20, 0.26], 0.0, 1.0);
    o.material("verde_vivo", [0.20, 0.95, 0.42], [0.10, 0.55, 0.20], 0.0, 1.0);
    o.material("naranja_vivo", [1.0, 0.45, 0.10], [0.70, 0.25, 0.03], 0.0, 1.0);
}

/// Letras de cubitos, centradas en `c`, para leerse desde +Z (desde donde viene el chofer).
fn letrero(o: &mut Obra, mat: &str, texto: &str, c: Vec3, pixel: f32) {
    let n = texto.chars().count();
    let ancho = (n * 4).saturating_sub(1) as f32 * pixel;
    for (i, letra) in texto.chars().enumerate() {
        for (fila, bits) in crate::adornos::glifo(letra).iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    let x = c.x - ancho * 0.5 + (i * 4 + col) as f32 * pixel;
                    let y = c.y + (2.0 - fila as f32) * pixel;
                    o.caja(mat, v(x, y, c.z), v(x + pixel * 0.94, y + pixel * 0.94, c.z + 0.04));
                }
            }
        }
    }
}

/// Un rectangulo pegado a una pared que mira hacia -X (ventanas y puertas).
fn en_pared_x(o: &mut Obra, mat: &str, x: f32, y0: f32, y1: f32, z0: f32, z1: f32) {
    o.quad(mat, [v(x, y0, z0), v(x, y0, z1), v(x, y1, z1), v(x, y1, z0)], v(-1.0, 0.0, 0.0));
}

/// Un rectangulo pegado a una pared que mira hacia +Z (hacia el chofer).
fn en_pared_z(o: &mut Obra, mat: &str, z: f32, x0: f32, x1: f32, y0: f32, y1: f32) {
    o.quad(mat, [v(x0, y0, z), v(x1, y0, z), v(x1, y1, z), v(x0, y1, z)], v(0.0, 0.0, 1.0));
}

/// Un volcan de bloques: capas que se van cerrando, cada una hecha de tres cajas para que de lejos
/// se vea redondo y de cerca, cuadrado.
fn volcan(o: &mut Obra, materiales: [&str; 3], c: Vec3, r: f32, alto: f32, capas: usize) {
    let paso = alto / capas as f32;
    for i in 0..capas {
        let k = i as f32 / capas as f32;
        let radio = r * (0.13 + 0.87 * (1.0 - k).powf(1.35));
        let mat = if k < 0.45 { materiales[0] } else if k < 0.78 { materiales[1] } else { materiales[2] };
        let (dx, dz) = (((i * 7) % 5) as f32 - 2.0, ((i * 3) % 5) as f32 - 2.0);
        let (cx, cz) = (c.x + dx * r * 0.008, c.z + dz * r * 0.008);
        let (y0, y1) = (c.y + i as f32 * paso, c.y + (i + 1) as f32 * paso);
        for (a, b) in [(1.0, 0.62), (0.62, 1.0), (0.86, 0.86)] {
            o.caja(mat, v(cx - radio * a, y0, cz - radio * b), v(cx + radio * a, y1, cz + radio * b));
        }
    }
}

/// La camioneta: el bus escolar gringo, ya guatemalteco. Lo que mas se ve es la espalda y el techo:
/// ahi va el rotulo, la escalera, la parrilla cargada y las luces de feria.
fn armar_camioneta() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    // Carroceria: azul abajo, franjas de feria, ventanas y techo rojo.
    o.caja("llanta", v(-1.05, 0.3, -3.5), v(1.05, 0.6, 3.5));
    o.caja("lamina_azul", v(-1.1, 0.6, -3.4), v(1.1, 1.4, 3.4));
    o.caja("lamina_amarilla", v(-1.12, 1.26, -3.42), v(1.12, 1.4, 3.42));
    o.caja("lamina_verde", v(-1.12, 1.12, -3.42), v(1.12, 1.26, 3.42));
    o.caja("lamina_rosa", v(-1.12, 0.98, -3.42), v(1.12, 1.12, 3.42));
    o.caja("vidrio", v(-1.09, 1.4, -3.32), v(1.09, 2.15, 3.32));
    for k in 0..9 {
        let z = -3.34 + k as f32 * 0.82;
        o.caja("lamina_azul", v(-1.12, 1.4, z), v(1.12, 2.15, z + 0.13));
    }
    o.caja("lamina_amarilla", v(-1.12, 2.15, -3.42), v(1.12, 2.26, 3.42));
    o.caja("lamina_roja", v(-1.1, 2.26, -3.4), v(1.1, 2.42, 3.4));
    // Las llamas de los lados, de cubos.
    for lado in [-1.0f32, 1.0] {
        for (k, h) in [0.34f32, 0.5, 0.26, 0.42, 0.2].iter().enumerate() {
            let z = -2.9 + k as f32 * 0.3;
            let x = lado * 1.13;
            o.caja("naranja", v(x - 0.02, 0.6, z), v(x + 0.02, 0.6 + h, z + 0.28));
        }
    }
    // La trompa, el parabrisas, la parrilla y los faros.
    o.caja("lamina_amarilla", v(-1.0, 0.55, -4.6), v(1.0, 1.45, -3.4));
    o.caja("lamina_roja", v(-0.3, 1.45, -4.6), v(0.3, 1.5, -3.4));
    o.caja("vidrio", v(-1.0, 1.45, -3.43), v(1.0, 2.15, -3.41));
    o.caja("cromo", v(-0.7, 0.65, -4.63), v(0.7, 1.2, -4.6));
    o.caja("cromo", v(-1.05, 0.45, -4.75), v(1.05, 0.6, -4.55));
    o.caja("faro", v(-0.95, 1.0, -4.64), v(-0.75, 1.2, -4.61));
    o.caja("faro", v(0.75, 1.0, -4.64), v(0.95, 1.2, -4.61));
    // La espalda: puerta de emergencia, ventanas, rotulo, escalera, defensa y luces.
    o.caja("lamina_azul", v(-1.1, 1.4, 3.32), v(1.1, 2.15, 3.42));
    en_pared_z(&mut o, "vidrio", 3.43, -0.95, -0.4, 1.55, 2.05);
    en_pared_z(&mut o, "vidrio", 3.43, 0.4, 0.95, 1.55, 2.05);
    en_pared_z(&mut o, "vidrio", 3.43, -0.25, 0.25, 1.55, 2.05);
    o.caja("negro", v(-0.95, 0.68, 3.42), v(0.95, 1.36, 3.46));
    letrero(&mut o, "lamina_amarilla", "HAY", v(-0.3, 1.09, 3.46), 0.05);
    letrero(&mut o, "lamina_rosa", "CAMPO", v(-0.3, 0.76, 3.46), 0.05);
    o.caja("cromo", v(-1.12, 0.42, 3.42), v(1.12, 0.6, 3.62));
    o.caja("calavera", v(-1.05, 0.72, 3.41), v(-0.95, 1.3, 3.5));
    o.caja("calavera", v(0.95, 0.72, 3.41), v(1.05, 1.3, 3.5));
    for rung in 0..9 {
        let y = 0.65 + rung as f32 * 0.22;
        o.caja("cromo", v(0.48, y, 3.5), v(0.78, y + 0.04, 3.54));
    }
    for x in [0.48, 0.74] {
        o.caja("cromo", v(x, 0.6, 3.5), v(x + 0.04, 2.7, 3.54));
    }
    // Luces de feria arriba de la espalda y a los lados del techo.
    for k in 0..7 {
        let x = -0.96 + k as f32 * 0.32;
        let mat = ["ambar", "luz_rosa", "luz_cian"][k % 3];
        o.caja(mat, v(x - 0.06, 2.3, 3.4), v(x + 0.06, 2.42, 3.47));
    }
    // Las llantas (dobles atras).
    for (z, ancho) in [(-3.4, 0.25), (1.5, 0.3), (2.5, 0.3)] {
        for lado in [-1.0, 1.0] {
            o.caja("llanta", v(lado * 1.05 - ancho * 0.5, 0.0, z - 0.4), v(lado * 1.05 + ancho * 0.5, 0.75, z + 0.4));
            o.caja("cromo", v(lado * (1.05 + ancho * 0.5), 0.25, z - 0.15), v(lado * (1.07 + ancho * 0.5), 0.5, z + 0.15));
        }
    }
    // La parrilla del techo, cargada hasta donde no se debe.
    for lado in [-1.0, 1.0] {
        o.caja("cromo", v(lado * 1.02 - 0.04, 2.42, -3.1), v(lado * 1.02 + 0.04, 2.72, 3.1));
        o.caja("cromo", v(lado * 1.02 - 0.05, 2.7, -3.1), v(lado * 1.02 + 0.05, 2.76, 3.1));
    }
    o.caja("cromo", v(-1.02, 2.7, 3.06), v(1.02, 2.76, 3.14));
    o.caja("cromo", v(-1.02, 2.7, -3.14), v(1.02, 2.76, -3.06));
    o.caja("colchon", v(-0.9, 2.42, -2.8), v(0.9, 2.62, -1.2));
    o.caja("blanco", v(-0.9, 2.5, -2.8), v(0.9, 2.54, -1.2));
    o.caja("costal", v(-0.85, 2.42, -1.05), v(-0.1, 2.9, 0.0));
    o.caja("costal_verde", v(0.05, 2.42, -1.05), v(0.85, 2.82, -0.2));
    o.caja("madera_clara", v(-0.85, 2.42, 0.2), v(-0.05, 3.0, 1.25));
    for k in 0..4 {
        o.caja("madera", v(-0.88, 2.5 + k as f32 * 0.14, 0.15), v(0.0, 2.54 + k as f32 * 0.14, 1.3));
    }
    o.caja("costal", v(0.1, 2.42, 0.25), v(0.9, 2.8, 1.1));
    o.caja("lamina_rosa", v(0.25, 2.8, 0.4), v(0.75, 3.06, 0.95));
    o.caja("costal_verde", v(-0.85, 2.42, 1.45), v(-0.1, 2.76, 2.5));
    o.caja("azul_cielo", v(0.1, 2.42, 1.45), v(0.82, 2.72, 2.4));
    o.modelo()
}

/// Los suelos: un tramo de carretera y sus orillas segun la zona. `par` turna las franjas.
fn armar_suelo(zona: Zona, par: bool) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let h = ANCHO_CARRETERA * 0.5;
    let arriba = v(0.0, 1.0, 0.0);
    let plano = |o: &mut Obra, mat: &str, x0: f32, x1: f32, z0: f32, z1: f32, y: f32| o.quad(mat, [v(x0, y, z0), v(x1, y, z0), v(x1, y, z1), v(x0, y, z1)], arriba);
    let (calle, orilla_cerca, orilla_lejos, bordo) = match (zona, par) {
        (Zona::Campo, true) => ("asfalto", "tierra", "pasto", "blanco"),
        (Zona::Campo, false) => ("asfalto_b", "tierra_oscura", "pasto_b", "rojo"),
        (Zona::Antigua, true) => ("adoquin", "hueso", "tierra", "gris"),
        (Zona::Antigua, false) => ("adoquin_b", "hueso", "tierra", "teja_b"),
        (Zona::Capital, true) => ("asfalto", "concreto", "concreto", "amarillo"),
        (Zona::Capital, false) => ("asfalto_b", "concreto_b", "concreto_b", "negro"),
    };
    plano(&mut o, calle, -h, h, 0.0, -TRAMO, 0.0);
    // Los adoquines de Antigua: juntas oscuras; el asfalto lleva parches.
    if zona == Zona::Antigua {
        for k in 0..4 {
            let z = -1.25 - k as f32 * 2.5;
            plano(&mut o, "tierra_oscura", -h, h, z - 0.04, z + 0.04, 0.006);
        }
        for x in [-2.5, -1.25, 1.25, 2.5] {
            plano(&mut o, "tierra_oscura", x - 0.03, x + 0.03, 0.0, -TRAMO, 0.006);
        }
    } else if par {
        plano(&mut o, "asfalto_b", -3.1, -1.5, -2.0, -3.8, 0.004);
    } else {
        plano(&mut o, "asfalto", 0.5, 2.3, -6.0, -8.5, 0.004);
    }
    for x in [-1.2, 1.2] {
        plano(&mut o, "blanco", x - 0.09, x + 0.09, -1.0, -5.5, 0.012);
    }
    for x in [-h + 0.25, h - 0.25] {
        plano(&mut o, "amarillo", x - 0.07, x + 0.07, 0.0, -TRAMO, 0.012);
    }
    for lado in [-1.0, 1.0] {
        o.caja(bordo, v(lado * h - 0.2, 0.0, -TRAMO), v(lado * h + 0.2, 0.2, 0.0));
        let (x0, x1, x2) = if lado < 0.0 { (-60.0, -h - 1.6, -h - 0.2) } else { (60.0, h + 1.6, h + 0.2) };
        plano(&mut o, orilla_lejos, x0, x1, 0.0, -TRAMO, -0.02);
        plano(&mut o, orilla_cerca, x1, x2, 0.0, -TRAMO, -0.01);
    }
    // Lo que da textura al suelo: matas y flores de cubitos (o bancas de banqueta).
    if zona == Zona::Campo {
        for k in 0..7u32 {
            let lado = if k % 2 == 0 { -1.0 } else { 1.0 };
            let semilla = k.wrapping_mul(2654435761).wrapping_add(par as u32 * 97);
            let x = lado * (h + 2.4 + (semilla >> 8) as f32 % 24.0);
            let z = -((semilla >> 3) as f32 % TRAMO);
            // Una mata, con dos florcitas si le toca.
            o.caja("pasto_lejos", v(x - 0.34, 0.0, z - 0.3), v(x + 0.34, 0.26, z + 0.3));
            o.caja("pasto_b", v(x - 0.2, 0.26, z - 0.18), v(x + 0.2, 0.42, z + 0.18));
            if k % 3 == 0 {
                let flor = if k % 2 == 0 { "flor" } else { "flor_b" };
                for (dx, dz) in [(-0.12, 0.0), (0.14, 0.1)] {
                    o.caja(flor, v(x + dx - 0.05, 0.42, z + dz - 0.05), v(x + dx + 0.05, 0.52, z + dz + 0.05));
                }
            }
        }
    }
    o.modelo()
}

/// Una casa colonial de Antigua: larga como un tramo de calle, mirando a la carretera (-X).
fn armar_casa(color: &str, largo: f32, alto: f32, con_balcon: bool) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let (x0, x1) = (-2.6, 2.6);
    let (z0, z1) = (-largo * 0.5, largo * 0.5);
    o.caja(color, v(x0, 0.0, z0), v(x1, alto, z1));
    o.caja("blanco", v(x0 - 0.05, 0.0, z0 - 0.02), v(x1, 0.42, z1 + 0.02));
    o.caja("blanco", v(x0 - 0.05, alto - 0.28, z0 - 0.02), v(x1, alto - 0.1, z1 + 0.02));
    // El techo de teja, en escalones.
    for k in 0..3 {
        let e = k as f32 * 0.7;
        let (y0, y1) = (alto + k as f32 * 0.32, alto + (k + 1) as f32 * 0.32);
        let mat = if k % 2 == 0 { "teja" } else { "teja_b" };
        o.caja(mat, v(x0 - 0.3 + e, y0, z0 - 0.1), v(x1 + 0.3 - e, y1, z1 + 0.1));
    }
    // Puerta y ventanas con reja de madera, en la cara de la calle.
    let cara = x0 - 0.06;
    en_pared_x(&mut o, "madera", cara, 0.0, 2.1, -0.65, 0.65);
    en_pared_x(&mut o, "amarillo", cara - 0.01, 0.95, 1.08, -0.1, 0.1);
    for z in [z0 + 1.1, z1 - 1.1] {
        en_pared_x(&mut o, "blanco", cara, 0.9, 2.6, z - 0.7, z + 0.7);
        en_pared_x(&mut o, "vidrio", cara - 0.01, 1.05, 2.45, z - 0.55, z + 0.55);
        for k in 0..4 {
            let bz = z - 0.42 + k as f32 * 0.28;
            o.caja("madera", v(cara - 0.1, 1.0, bz - 0.03), v(cara, 2.5, bz + 0.03));
        }
        o.caja("madera", v(cara - 0.14, 0.92, z - 0.65), v(cara, 1.02, z + 0.65));
    }
    if con_balcon {
        o.caja("madera", v(cara - 0.6, 2.75, -1.1), v(cara, 2.86, 1.1));
        for k in 0..6 {
            let z = -1.0 + k as f32 * 0.4;
            o.caja("madera", v(cara - 0.58, 2.85, z - 0.03), v(cara - 0.5, 3.2, z + 0.03));
        }
        o.caja("flor", v(cara - 0.56, 2.86, -0.95), v(cara - 0.3, 3.06, -0.5));
        o.caja("flor_b", v(cara - 0.56, 2.86, 0.5), v(cara - 0.3, 3.06, 0.95));
    }
    o.modelo()
}

/// Un edificio de la capital: ventanas prendidas, unas cuantas apagadas, y un neon arriba.
fn armar_edificio(pared: &str, luz: &str, neon: &str, pisos: usize, semilla: u32) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let (mx, mz, piso) = (3.6, 4.0, 2.3);
    let alto = pisos as f32 * piso + 0.6;
    o.caja(pared, v(-mx, 0.0, -mz), v(mx, alto, mz));
    o.caja("apagada", v(-mx - 0.04, 0.0, -mz - 0.04), v(mx + 0.04, 2.0, mz + 0.04));
    o.caja(neon, v(-mx - 0.06, 2.0, -mz - 0.06), v(mx + 0.06, 2.2, mz + 0.06));
    let mut azar = semilla;
    let mut prendida = move || {
        azar = azar.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (azar >> 16) & 7 > 1
    };
    for p in 1..pisos {
        let y = p as f32 * piso + 0.5;
        for k in 0..4 {
            let z = -mz + 1.05 + k as f32 * 2.0;
            en_pared_x(&mut o, if prendida() { luz } else { "apagada" }, -mx - 0.03, y, y + 1.2, z - 0.6, z + 0.6);
        }
        for k in 0..3 {
            let x = -mx + 1.35 + k as f32 * 2.45;
            en_pared_z(&mut o, if prendida() { luz } else { "apagada" }, mz + 0.03, x - 0.7, x + 0.7, y, y + 1.2);
        }
    }
    // La azotea: el tinaco y la antena de siempre.
    o.caja("gris", v(-1.2, alto, -1.2), v(0.6, alto + 0.9, 0.6));
    o.caja("gris", v(1.5, alto, 1.5), v(1.6, alto + 2.6, 1.6));
    o.caja(neon, v(1.42, alto + 2.6, 1.42), v(1.68, alto + 2.86, 1.68));
    o.modelo()
}

/// Un barrilete gigante de Sumpango: anillos de colores sobre una armazon de cana.
fn armar_barrilete(colores: [&str; 5]) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let centro = 5.2;
    for lado in [-1.0, 1.0] {
        o.caja("madera", v(lado * 1.5 - 0.1, 0.0, -0.1), v(lado * 1.5 + 0.1, centro, 0.1));
    }
    for (k, mat) in colores.iter().enumerate() {
        let r = 4.1 - k as f32 * 0.8;
        let z = k as f32 * 0.04;
        for (a, b) in [(1.0, 0.45), (0.45, 1.0), (0.78, 0.78)] {
            o.caja(mat, v(-r * a, centro - r * b, z), v(r * a, centro + r * b, z + 0.05));
        }
    }
    // Los flecos de la orilla.
    for k in 0..9 {
        let x = -3.2 + k as f32 * 0.8;
        o.caja(colores[k % 5], v(x - 0.12, centro - 5.0, 0.0), v(x + 0.12, centro - 3.6, 0.04));
    }
    o.modelo()
}

/// La valla de la orilla, con su chiste, iluminada.
fn armar_valla(texto: &str, fondo: &str, letra: &str) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    for x in [-2.6, 2.6] {
        o.caja("gris", v(x - 0.12, 0.0, -0.12), v(x + 0.12, 3.2, 0.12));
    }
    o.caja(fondo, v(-4.2, 3.2, -0.12), v(4.2, 6.2, 0.12));
    o.caja("luz_calida", v(-4.2, 6.2, 0.0), v(4.2, 6.3, 0.2));
    let pixel = (7.4 / (texto.len() as f32 * 4.0)).min(0.24);
    letrero(&mut o, letra, texto, v(0.0, 4.4, 0.13), pixel);
    o.modelo()
}

/// La portada que atraviesa la calle al entrar a una zona: alta, para que la camara pase abajo.
fn armar_portada(texto: &str, marco: &str, letra: &str) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    for x in [-6.2, 6.2] {
        o.caja(marco, v(x - 0.3, 0.0, -0.3), v(x + 0.3, 7.8, 0.3));
    }
    o.caja(marco, v(-6.5, 7.8, -0.3), v(6.5, 10.4, 0.3));
    letrero(&mut o, letra, texto, v(0.0, 8.5, 0.31), 0.32);
    for k in 0..13 {
        let x = -6.0 + k as f32;
        o.caja(["luz_rosa", "luz_cian", "ambar"][k % 3], v(x - 0.1, 10.4, -0.1), v(x + 0.1, 10.6, 0.1));
    }
    o.modelo()
}

pub fn modelos() -> Modelos {
    let camioneta = armar_camioneta();

    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("freno", v(-1.06, 0.7, 3.46), v(-0.94, 1.32, 3.53));
    o.caja("freno", v(0.94, 0.7, 3.46), v(1.06, 1.32, 3.53));
    let frenos = o.modelo();

    // El ayudante, colgado de la escalera. El brazo va aparte: saluda.
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("pantalon", v(-0.13, 0.62, -0.1), v(0.13, 1.25, 0.1));
    o.caja("rojo", v(-0.18, 1.25, -0.12), v(0.18, 1.78, 0.12));
    o.caja("piel", v(-0.13, 1.78, -0.13), v(0.13, 2.05, 0.13));
    o.caja("negro", v(-0.15, 2.0, -0.15), v(0.15, 2.12, 0.15));
    o.caja("negro", v(-0.15, 2.0, -0.28), v(0.15, 2.05, -0.1));
    o.caja("rojo", v(-0.28, 1.45, -0.05), v(-0.18, 1.75, 0.05));
    let ayudante = o.modelo();
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("rojo", v(-0.05, 0.0, -0.05), v(0.05, 0.3, 0.05));
    o.caja("piel", v(-0.05, 0.3, -0.05), v(0.05, 0.5, 0.05));
    let brazo = o.modelo();

    // La gallina de la parrilla (por algo les dicen chicken bus).
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("blanco", v(-0.12, 0.08, -0.16), v(0.12, 0.3, 0.16));
    o.caja("blanco", v(-0.08, 0.26, -0.26), v(0.08, 0.44, -0.08));
    o.caja("rojo", v(-0.03, 0.44, -0.22), v(0.03, 0.52, -0.1));
    o.caja("amarillo", v(-0.03, 0.32, -0.32), v(0.03, 0.37, -0.26));
    o.caja("amarillo", v(-0.08, 0.0, -0.03), v(-0.04, 0.08, 0.03));
    o.caja("amarillo", v(0.04, 0.0, -0.03), v(0.08, 0.08, 0.03));
    let gallina = o.modelo();

    let suelos = [
        [armar_suelo(Zona::Campo, true), armar_suelo(Zona::Campo, false)],
        [armar_suelo(Zona::Antigua, true), armar_suelo(Zona::Antigua, false)],
        [armar_suelo(Zona::Capital, true), armar_suelo(Zona::Capital, false)],
    ];

    // La alfombra de aserrin de Semana Santa, sobre la calle.
    let mut o = Obra::new();
    paleta(&mut o);
    let h = ANCHO_CARRETERA * 0.5 - 0.5;
    let plano = |o: &mut Obra, mat: &str, x0: f32, x1: f32, z0: f32, z1: f32, y: f32| o.quad(mat, [v(x0, y, z0), v(x1, y, z0), v(x1, y, z1), v(x0, y, z1)], v(0.0, 1.0, 0.0));
    plano(&mut o, "lila", -h, h, -0.5, -9.5, 0.015);
    plano(&mut o, "amarillo", -h + 0.25, h - 0.25, -0.75, -9.25, 0.018);
    plano(&mut o, "rojo", -h + 0.5, h - 0.5, -1.0, -9.0, 0.021);
    for k in 0..4 {
        let z = -2.0 - k as f32 * 2.0;
        for (r, mat) in [(1.0, "verde"), (0.6, "blanco"), (0.25, "amarillo")] {
            for x in [-1.6, 0.0, 1.6] {
                let y = 0.024 + (1.0 - r) * 0.01;
                plano(&mut o, mat, x - r * 0.7, x + r * 0.7, z - r * 0.5, z + r * 0.5, y);
            }
        }
    }
    let alfombra = o.modelo();

    // El valle: un llano enorme abajo de todo, para que nunca se asome el cielo por el piso.
    let mut o = Obra::new();
    paleta(&mut o);
    o.quad("pasto_lejos", [v(-900.0, -0.08, 40.0), v(900.0, -0.08, 40.0), v(900.0, -0.08, -900.0), v(-900.0, -0.08, -900.0)], v(0.0, 1.0, 0.0));
    let llano = o.modelo();

    // El tuc-tuc rojo, de espaldas, con su piloto y su calcomania.
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("lamina_roja", v(-0.65, 0.3, -0.9), v(0.65, 1.15, 0.9));
    o.caja("blanco", v(-0.67, 0.72, -0.92), v(0.67, 0.82, 0.92));
    o.caja("negro", v(-0.7, 1.15, 0.72), v(0.7, 1.75, 0.9));
    o.caja("negro", v(-0.72, 1.75, -1.0), v(0.72, 1.87, 1.0));
    o.caja("amarillo", v(-0.72, 1.87, -1.0), v(0.72, 1.92, 1.0));
    for x in [-0.66, 0.62] {
        o.caja("negro", v(x, 1.15, -0.95), v(x + 0.05, 1.75, -0.88));
    }
    o.caja("piel", v(-0.14, 1.2, -0.45), v(0.14, 1.5, -0.2));
    for (x, z) in [(-0.6, 0.6), (0.6, 0.6), (0.0, -0.8)] {
        o.caja("llanta", v(x - 0.12, 0.0, z - 0.25), v(x + 0.12, 0.45, z + 0.25));
    }
    o.caja("freno", v(-0.55, 0.5, 0.91), v(-0.3, 0.68, 0.94));
    o.caja("freno", v(0.3, 0.5, 0.91), v(0.55, 0.68, 0.94));
    letrero(&mut o, "amarillo", "TUK", v(0.0, 0.92, 0.91), 0.06);
    let tuctuc = o.modelo();

    // El perro (callejero, cafe con manchas).
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("madera_clara", v(-0.25, 0.35, -0.5), v(0.25, 0.75, 0.5));
    o.caja("madera", v(-0.26, 0.5, -0.1), v(0.26, 0.76, 0.3));
    o.caja("madera_clara", v(-0.2, 0.6, -0.85), v(0.2, 1.0, -0.45));
    o.caja("negro", v(-0.08, 0.68, -0.95), v(0.08, 0.82, -0.82));
    for x in [-0.2, 0.12] {
        o.caja("madera", v(x, 1.0, -0.7), v(x + 0.08, 1.18, -0.55));
    }
    for (x, z) in [(-0.17, -0.4), (0.17, -0.4), (-0.17, 0.4), (0.17, 0.4)] {
        o.caja("madera_clara", v(x - 0.07, 0.0, z - 0.07), v(x + 0.07, 0.4, z + 0.07));
    }
    o.caja("madera", v(-0.05, 0.7, 0.5), v(0.05, 1.05, 0.6));
    let perro = o.modelo();

    // El bache: hoyo oscuro, orilla rota y el cono que le puso un vecino.
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("gris", v(-0.9, 0.0, -0.75), v(0.9, 0.03, 0.75));
    o.caja("negro", v(-0.7, 0.0, -0.55), v(0.7, 0.04, 0.55));
    o.caja("naranja_vivo", v(0.72, 0.0, 0.42), v(0.98, 0.1, 0.68));
    o.caja("naranja_vivo", v(0.78, 0.1, 0.48), v(0.92, 0.42, 0.62));
    o.caja("blanco", v(0.77, 0.24, 0.47), v(0.93, 0.3, 0.63));
    let bache = o.modelo();

    // El tumulo: atraviesa la carretera, a rayas amarillas y negras.
    let mut o = Obra::new();
    paleta(&mut o);
    for k in 0..10 {
        let x0 = -ANCHO_CARRETERA * 0.5 + k as f32 * ANCHO_CARRETERA / 10.0;
        o.caja(if k % 2 == 0 { "amarillo" } else { "negro" }, v(x0, 0.0, -0.35), v(x0 + ANCHO_CARRETERA / 10.0, 0.2, 0.35));
    }
    let tumulo = o.modelo();

    // La senal de que viene un tumulo.
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("gris", v(-0.06, 0.0, -0.06), v(0.06, 2.0, 0.06));
    o.caja("naranja_vivo", v(-0.7, 2.0, -0.05), v(0.7, 3.2, 0.05));
    letrero(&mut o, "negro", "TUMULO", v(0.0, 2.4, 0.06), 0.052);
    let senal_tumulo = o.modelo();

    // La parada: techito con su letrero, a la orilla derecha.
    let mut o = Obra::new();
    paleta(&mut o);
    for z in [-1.8, 1.8] {
        o.caja("gris", v(4.35, 0.0, z - 0.06), v(4.47, 2.7, z + 0.06));
        o.caja("gris", v(5.75, 0.0, z - 0.06), v(5.87, 2.7, z + 0.06));
    }
    o.caja("lamina_azul", v(4.1, 2.7, -2.1), v(6.1, 2.85, 2.1));
    o.caja("lamina_amarilla", v(4.08, 2.62, -2.12), v(6.12, 2.7, 2.12));
    o.caja("madera", v(5.3, 0.45, -1.6), v(5.8, 0.55, 1.6));
    o.caja("gris", v(4.25, 0.0, 2.4), v(4.37, 3.4, 2.52));
    o.caja("lamina_azul", v(3.45, 3.4, 2.4), v(5.15, 4.15, 2.5));
    letrero(&mut o, "blanco", "PARADA", v(4.3, 3.62, 2.51), 0.07);
    let parada = o.modelo();

    // El carril de la parada, con flechas: verde si ya va despacio, naranja si no.
    let zona_parada = |mat: &str| {
        let mut o = Obra::new();
        paleta(&mut o);
        for k in 0..6 {
            let z = -18.0 + k as f32 * 3.0;
            let (x0, x1) = (CARRILES[2] - 0.9, CARRILES[2] + 0.9);
            o.quad(mat, [v(x0, 0.02, z), v(x1, 0.02, z), v(x1, 0.02, z - 0.7), v(x0, 0.02, z - 0.7)], v(0.0, 1.0, 0.0));
        }
        o.modelo()
    };
    let zona_lenta = zona_parada("verde_vivo");
    let zona_rapida = zona_parada("naranja_vivo");

    // La gente esperando: tres camisas.
    let persona = |camisa: &str, falda: bool| {
        let mut o = Obra::new();
        paleta(&mut o);
        o.caja(if falda { "rosa" } else { "pantalon" }, v(-0.18, 0.0, -0.12), v(0.18, 0.75, 0.12));
        o.caja(camisa, v(-0.24, 0.75, -0.15), v(0.24, 1.35, 0.15));
        o.caja("piel", v(-0.15, 1.35, -0.15), v(0.15, 1.65, 0.15));
        o.caja("negro", v(-0.16, 1.58, -0.16), v(0.16, 1.7, 0.16));
        o.modelo()
    };
    let personas = [persona("rojo", true), persona("azul_cielo", false), persona("amarillo", false)];

    // Los volcanes de bloques: Agua, Fuego y Acatenango, y los cerros de enfrente.
    let mut o = Obra::new();
    paleta(&mut o);
    volcan(&mut o, ["volcan_verde", "volcan_verde", "volcan_azul"], v(-120.0, -1.0, -340.0), 95.0, 105.0, 16);
    volcan(&mut o, ["volcan_verde", "volcan_azul", "ceniza"], v(38.0, -1.0, -380.0), 105.0, 145.0, 20);
    volcan(&mut o, ["volcan_verde", "volcan_azul", "ceniza"], v(170.0, -1.0, -350.0), 85.0, 100.0, 15);
    for k in 0..14 {
        let x = -320.0 + k as f32 * 50.0;
        let alto = 10.0 + ((k * 37) % 13) as f32 * 1.6;
        let frente = -215.0 - ((k * 23) % 7) as f32 * 3.0;
        o.caja(if k % 2 == 0 { "volcan_verde" } else { "pasto_lejos" }, v(x - 32.0, -1.0, frente - 30.0), v(x + 32.0, alto, frente));
    }
    let volcanes = o.modelo();

    // El crater de Fuego, siempre encendido.
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("lava", v(30.0, 143.0, -388.0), v(46.0, 148.0, -372.0));
    let lava = o.modelo();

    // Una nube de cubos, rosada por la tarde.
    let mut o = Obra::new();
    paleta(&mut o);
    for (x, y, z, a, h, l) in [(0.0, 0.0, 0.0, 18.0, 5.0, 9.0), (-8.0, -1.0, 2.0, 11.0, 3.5, 8.0), (9.0, -1.2, -1.0, 12.0, 3.5, 7.0), (2.0, 3.5, 0.0, 9.0, 3.5, 7.0)] {
        o.caja("nube", v(x - a, y - h * 0.5, z - l), v(x + a, y + h * 0.5, z + l));
    }
    let nube = o.modelo();

    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("sol", v(-22.0, -22.0, -1.0), v(22.0, 22.0, 1.0));
    let sol = o.modelo();

    let cubo = |mat: &str| {
        let mut o = Obra::new();
        paleta(&mut o);
        o.caja(mat, v(-0.5, -0.5, -0.5), v(0.5, 0.5, 0.5));
        o.modelo()
    };
    let cubos = [cubo("gris"), cubo("negro"), cubo("rosa"), cubo("amarillo"), cubo("azul_cielo"), cubo("verde"), cubo("lila")];

    // La milpa: surcos de cana con elote, sobre tierra.
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("tierra_oscura", v(-3.0, 0.0, -4.0), v(3.0, 0.07, 4.0));
    for i in 0..4 {
        for j in 0..5 {
            let (x, z) = (-2.2 + i as f32 * 1.45, -3.3 + j as f32 * 1.65);
            let alto = 1.5 + ((i * 3 + j * 5) % 4) as f32 * 0.22;
            o.caja("pasto_b", v(x - 0.34, 0.0, z - 0.34), v(x + 0.34, alto * 0.55, z + 0.34));
            o.caja("milpa", v(x - 0.24, alto * 0.55, z - 0.24), v(x + 0.24, alto, z + 0.24));
            o.caja("elote", v(x - 0.06, alto, z - 0.06), v(x + 0.06, alto + 0.26, z + 0.06));
        }
    }
    let milpa = o.modelo();

    // Las casas: de colores, como las de Antigua; las del campo, mas chiquitas.
    const COLORES_CASA: [&str; 6] = ["amarillo", "azul_cielo", "rosa", "turquesa", "naranja", "lila"];
    let casitas = COLORES_CASA.map(|c| armar_casa(c, 5.2, 2.6, false));
    let casas = [armar_casa(COLORES_CASA[0], 7.2, 3.8, true), armar_casa(COLORES_CASA[1], 7.2, 3.3, false), armar_casa(COLORES_CASA[2], 7.2, 4.1, true), armar_casa(COLORES_CASA[3], 7.2, 3.5, false), armar_casa(COLORES_CASA[4], 7.2, 3.9, true), armar_casa(COLORES_CASA[5], 7.2, 3.4, false)];

    let barriletes = [
        armar_barrilete(["rosa", "amarillo", "turquesa", "rojo", "blanco"]),
        armar_barrilete(["azul", "blanco", "azul_cielo", "amarillo", "rojo"]),
        armar_barrilete(["lila", "naranja", "verde", "amarillo", "rosa"]),
    ];

    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("madera", v(-0.12, 0.0, -0.12), v(0.12, 7.0, 0.12));
    o.caja("madera", v(-1.2, 6.2, -0.08), v(1.2, 6.36, 0.08));
    o.caja("gris", v(0.2, 5.0, -0.25), v(0.65, 5.8, 0.25));
    let poste = o.modelo();

    // El puesto de shucos: mesa, sombrilla de colores y la seno atendiendo.
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("madera", v(-0.7, 0.0, -1.2), v(0.2, 0.95, 1.2));
    o.caja("blanco", v(-0.75, 0.95, -1.25), v(0.25, 1.0, 1.25));
    o.caja("madera_clara", v(-0.5, 1.0, -0.8), v(-0.1, 1.2, -0.2));
    o.caja("rojo", v(-0.48, 1.2, -0.75), v(-0.12, 1.26, -0.25));
    o.caja("verde", v(-0.5, 1.0, 0.2), v(-0.1, 1.25, 0.8));
    o.caja("gris", v(-0.08, 0.0, -0.08), v(0.08, 3.0, 0.08));
    for (k, mat) in ["rojo", "amarillo", "verde", "azul_cielo"].iter().enumerate() {
        let r = 2.0 - k as f32 * 0.45;
        let y = 2.85 + k as f32 * 0.14;
        o.caja(mat, v(-r, y, -r), v(r, y + 0.14, r));
    }
    o.caja("rosa", v(0.45, 0.0, -0.25), v(0.95, 1.0, 0.25));
    o.caja("azul_cielo", v(0.45, 1.0, -0.28), v(0.95, 1.6, 0.28));
    o.caja("piel", v(0.55, 1.6, -0.18), v(0.85, 1.9, 0.18));
    o.caja("negro", v(0.53, 1.84, -0.2), v(0.87, 2.0, 0.2));
    let puesto = o.modelo();

    // La iglesia amarilla, con torres y remates blancos (mira al chofer).
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("amarillo", v(-7.0, 0.0, -10.0), v(7.0, 9.0, 7.0));
    for x in [-7.0, 4.0] {
        o.caja("amarillo", v(x, 0.0, 4.0), v(x + 3.0, 14.0, 7.2));
        o.caja("blanco", v(x - 0.1, 14.0, 3.9), v(x + 3.1, 14.4, 7.3));
        o.caja("amarillo", v(x + 0.5, 14.4, 4.5), v(x + 2.5, 16.0, 6.7));
        o.caja("blanco", v(x + 1.3, 16.0, 5.3), v(x + 1.7, 17.6, 5.9));
        o.caja("blanco", v(x + 0.9, 16.8, 5.5), v(x + 2.1, 17.1, 5.7));
        en_pared_z(&mut o, "madera", 7.21, x + 1.0, x + 2.0, 10.5, 12.5);
    }
    o.caja("amarillo", v(-4.0, 9.0, 6.0), v(4.0, 12.0, 7.0));
    o.caja("amarillo", v(-2.2, 12.0, 6.0), v(2.2, 13.6, 7.0));
    o.caja("blanco", v(-7.1, 4.4, 7.0), v(7.1, 4.7, 7.25));
    o.caja("blanco", v(-7.1, 8.8, 7.0), v(7.1, 9.1, 7.25));
    en_pared_z(&mut o, "madera", 7.02, -1.6, 1.6, 0.0, 3.9);
    en_pared_z(&mut o, "blanco", 7.01, -2.1, 2.1, 0.0, 4.3);
    en_pared_z(&mut o, "vidrio", 7.02, -0.9, 0.9, 5.4, 7.5);
    o.caja("teja", v(-6.0, 9.0, -10.0), v(6.0, 10.2, 6.0));
    let iglesia = o.modelo();

    // El papel picado: filas de banderitas de lado a lado de la calle, arriba de la camara.
    let mut o = Obra::new();
    paleta(&mut o);
    for x in [-6.3, 6.3] {
        o.caja("madera", v(x - 0.08, 0.0, -0.08), v(x + 0.08, 7.3, 0.08));
    }
    const PICADO: [&str; 6] = ["rosa", "amarillo", "turquesa", "rojo", "lila", "verde"];
    for (fila, y) in [6.9, 6.25].iter().enumerate() {
        o.caja("blanco", v(-6.3, y + 0.28, -0.015), v(6.3, y + 0.31, 0.015));
        for k in 0..18 {
            let x = -5.95 + k as f32 * 0.7;
            let mat = PICADO[(k + fila * 2) % 6];
            o.caja(mat, v(x - 0.27, *y - 0.1, -0.012), v(x + 0.27, y + 0.28, 0.012));
        }
    }
    let papel = o.modelo();

    let edificios = [
        armar_edificio("concreto", "luz_calida", "luz_rosa", 5, 3),
        armar_edificio("volcan_azul", "luz_cian", "luz_lila", 8, 11),
        armar_edificio("lila", "luz_calida", "luz_cian", 6, 29),
        armar_edificio("azul", "luz_calida", "ambar", 10, 47),
    ];
    let vallas = [armar_valla("HAY SHUCOS", "lila", "amarillo"), armar_valla("TE VI EN LINEA", "rosa", "blanco"), armar_valla("PRESTAME Q5", "azul", "amarillo")];

    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("gris", v(-0.08, 0.0, -0.08), v(0.08, 5.6, 0.08));
    o.caja("gris", v(-0.9, 5.5, -0.06), v(0.08, 5.62, 0.06));
    o.caja("luz_calida", v(-1.05, 5.38, -0.14), v(-0.5, 5.52, 0.14));
    let farol = o.modelo();

    // El Arco de Santa Catalina: amarillo, con su reloj, y la calle pasando por abajo.
    let mut o = Obra::new();
    paleta(&mut o);
    let hueco = 5.2;
    for lado in [-1.0f32, 1.0] {
        let (x0, x1) = if lado < 0.0 { (-11.0, -hueco) } else { (hueco, 11.0) };
        o.caja("amarillo", v(x0, 0.0, -2.2), v(x1, 11.0, 2.2));
        o.caja("blanco", v(x0, 0.0, 2.2), v(x1, 0.6, 2.26));
        // El medio arco, en escalones.
        for k in 0..4 {
            let e = k as f32 * 0.55;
            let (a, b) = if lado < 0.0 { (-hueco - 0.05, -hueco + e + 0.55) } else { (hueco - e - 0.55, hueco + 0.05) };
            o.caja("amarillo", v(a, 8.2 + k as f32 * 0.42, -2.2), v(b, 11.0, 2.2));
        }
        // Las ventanitas del paso de las monjas.
        for k in 0..2 {
            let x = lado * (hueco + 1.5 + k as f32 * 2.4);
            en_pared_z(&mut o, "madera", 2.21, x - 0.5, x + 0.5, 5.0, 7.0);
            en_pared_z(&mut o, "blanco", 2.205, x - 0.7, x + 0.7, 4.8, 7.2);
        }
    }
    o.caja("blanco", v(-11.2, 11.0, -2.4), v(11.2, 11.4, 2.4));
    o.caja("teja", v(-11.0, 11.4, -2.2), v(11.0, 12.1, 2.2));
    o.caja("amarillo", v(-2.2, 12.1, -1.4), v(2.2, 16.2, 1.4));
    o.caja("blanco", v(-2.5, 16.2, -1.7), v(2.5, 16.6, 1.7));
    o.caja("teja", v(-1.8, 16.6, -1.2), v(1.8, 17.8, 1.2));
    en_pared_z(&mut o, "blanco", 1.41, -1.2, 1.2, 12.9, 15.3);
    en_pared_z(&mut o, "negro", 1.42, -0.06, 0.06, 14.1, 15.1);
    en_pared_z(&mut o, "negro", 1.42, -0.06, 0.7, 14.05, 14.2);
    let arco = o.modelo();

    // La Torre del Reformador: la torre de hierro de la capital, en cubos.
    let mut o = Obra::new();
    paleta(&mut o);
    let alto_torre = 34.0;
    for k in 0..17 {
        let t = k as f32 / 17.0;
        let abierto = 6.0 * (1.0 - t).powf(1.8) + 0.8;
        let (y0, y1) = (t * alto_torre, (k + 1) as f32 / 17.0 * alto_torre);
        for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            let (x, z) = (sx * abierto, sz * abierto);
            o.caja("hierro", v(x - 0.42, y0, z - 0.42), v(x + 0.42, y1, z + 0.42));
        }
        if k % 3 == 0 {
            for sz in [-1.0, 1.0] {
                o.caja("hierro", v(-abierto, y0, sz * abierto - 0.15), v(abierto, y0 + 0.3, sz * abierto + 0.15));
                o.caja("hierro", v(sz * abierto - 0.15, y0, -abierto), v(sz * abierto + 0.15, y0 + 0.3, abierto));
            }
        }
    }
    o.caja("hierro", v(-1.4, alto_torre, -1.4), v(1.4, alto_torre + 1.4, 1.4));
    o.caja("ambar", v(-0.6, alto_torre + 1.4, -0.6), v(0.6, alto_torre + 2.8, 0.6));
    o.caja("hierro", v(-0.12, alto_torre + 2.8, -0.12), v(0.12, alto_torre + 6.0, 0.12));
    o.caja("luz_rosa", v(-0.25, alto_torre + 6.0, -0.25), v(0.25, alto_torre + 6.5, 0.25));
    let torre = o.modelo();

    let portada_antigua = armar_portada("ANTIGUA", "lila", "amarillo");
    let portada_capital = armar_portada("LA CAPITAL", "azul", "blanco");
    let terminal = armar_portada("TERMINAL", "rosa", "amarillo");

    Modelos {
        camioneta,
        frenos,
        ayudante,
        brazo,
        gallina,
        suelos,
        alfombra,
        llano,
        tuctuc,
        perro,
        bache,
        tumulo,
        senal_tumulo,
        parada,
        zona_lenta,
        zona_rapida,
        personas,
        volcanes,
        lava,
        nube,
        sol,
        cubos,
        milpa,
        casitas,
        casas,
        barriletes,
        poste,
        puesto,
        iglesia,
        papel,
        edificios,
        vallas,
        farol,
        arco,
        torre,
        portada_antigua,
        portada_capital,
        terminal,
    }
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
