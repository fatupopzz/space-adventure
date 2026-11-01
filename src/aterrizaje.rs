//! La cutscene de aterrizaje, una por planeta. Toma ideas de los juegos de
//! naves (No Man's Sky, Outer Wilds, Star Citizen) y de las cinematicas de
//! llegada de los juegos de aventura: mientras dura, las orbitas se congelan
//! (el mundo se queda quieto y la que se mueve es la nave), la camara corta
//! entre tomas FIJAS (la nave cruza el cuadro en vez de quedarse pegada al
//! centro), salen franjas de cine y la tarjeta con el nombre del planeta.
//! Cada planeta tiene su guion: titulo, subtitulo, color de la entrada a la
//! atmosfera y el angulo de sus tomas. Se salta con ESPACIO.
//!
//! 1. **Aproximacion**: la nave baja en curva hasta quedar arriba del punto
//!    de aterrizaje, con la nariz un poco hacia abajo, entrando a la
//!    atmosfera (resplandor y temblor). La camara la persigue.
//! 2. **Descenso**: baja en vertical, rapido arriba y frenando cerca del
//!    suelo, con los propulsores a fondo y bamboleandose un poquito. Cerca
//!    del piso los motores levantan polvo (del color del planeta) que crece.
//!    La camara pasa a ras de suelo, mirandola bajar.
//! 3. **Contacto**: la nave toca, se aplasta un poco y rebota en su
//!    suspension; sale un golpe de polvo y la camara se sacude.
//! 4. **Posado**: queda parada sobre el planeta (lo sigue en su orbita), y en
//!    los planetas sin escena Miku se baja a saludar.
//! 5. **Arranque y despegue**: los motores se encienden temblando y levantando
//!    polvo, y la nave sube acelerando cada vez mas.
//!
//! Los planetas son de juguete: varios son mas chicos que la nave, y Miku
//! queda parada sobre ellos como El Principito.

use crate::escena::CUERPOS;
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::nave::{Camara, Nave};
use crate::pipeline::proyectar;
use crate::salto::rumbo_hacia;
use std::f32::consts::PI;

const APROXIMAR: f32 = 2.4;
const DESCENDER: f32 = 2.6;
const CONTACTO: f32 = 0.45;
const ARRANCAR: f32 = 0.8;
const SUBIR: f32 = 2.2;
/// Cuanto tarda Miku en bajarse (o subirse) de la nave.
const BRINCO_MIKU: f32 = 0.8;
/// Que tan arriba del suelo queda el centro de la nave (sus botas tocan el suelo).
const ALTURA_NAVE: f32 = 0.66;
const ESCALA_MIKU: f32 = 0.32;
/// Desde que altura (sobre el suelo) empiezan los motores a levantar polvo.
const ALTURA_POLVO: f32 = 3.5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fase {
    Aproximacion,
    Descenso,
    Contacto,
    Posado,
    Arranque,
    Subiendo,
}

pub enum Evento {
    Nada,
    /// Toco el suelo (para el golpe y la sacudida).
    Toque,
    /// Ya se asento: se puede entrar a la escena del planeta.
    Posada,
    /// Termino de despegar: se le devuelve el control a la piloto.
    Despego,
}

pub struct Aterrizaje {
    pub planeta: usize,
    fase: Fase,
    /// Segundos en la fase.
    t: f32,
    desde: Vec3,
    yaw0: f32,
    /// Si en este planeta Miku se baja a saludar.
    pub miku_baja: bool,
    /// Si ya se pidio despegar (se cumple en cuanto Miku termine de bajarse).
    quiere_despegar: bool,
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Bezier cuadratica de `a` a `c` pasando cerca de `b`.
fn curva(a: Vec3, b: Vec3, c: Vec3, k: f32) -> Vec3 {
    a.lerp(b, k).lerp(b.lerp(c, k), k)
}

/// El color del polvo que levantan los motores, segun el planeta.
fn color_polvo(planeta: usize) -> [f32; 3] {
    match CUERPOS[planeta].archivo {
        "marte" => [0.95, 0.5, 0.3],
        "mercurio" | "luna" => [0.8, 0.8, 0.82],
        "venus" | "jupiter" | "saturno_ritmo" => [1.0, 0.85, 0.55],
        "tierra" | "hada" => [0.7, 0.95, 0.7],
        "urano" | "neptuno" => [0.6, 0.85, 1.0],
        "pluton" => [0.95, 0.85, 0.75],
        _ => [0.9, 0.8, 0.95],
    }
}

/// El guion de la cutscene de cada planeta.
struct Guion {
    titulo: &'static str,
    subtitulo: &'static str,
    /// El color del resplandor de entrada a la atmosfera.
    entrada: [f32; 3],
    /// Hacia que lado (en radianes, desde el Sol) se pone la toma general, y que tan alta.
    angulo: f32,
    altura: f32,
}

fn guion(planeta: usize) -> Guion {
    let g = |titulo, subtitulo, entrada, angulo, altura| Guion { titulo, subtitulo, entrada, angulo, altura };
    match CUERPOS[planeta].archivo {
        "mercurio" => g("MERCURIO", "Consultorio de Madame Mercurio. Hoy: retrógrado.", [1.0, 0.75, 0.45], 0.9, 0.5),
        "venus" => g("VENUS", "Planeta del amor. Juzgado de divorcios.", [1.0, 0.85, 0.4], -0.8, 0.8),
        "tierra" => g("LA TIERRA", "Hogar, dulce hogar. Y tres entregas a las 23:59.", [0.55, 0.75, 1.0], 0.5, 0.6),
        "planeta_vegetta" => g("PLANETA VEGETTA", "¡Muy buenas a todos, guapísimos!", [0.75, 0.5, 1.0], -0.6, 0.6),
        "luna" => g("LA LUNA", "¡Silencio en el set!", [0.85, 0.85, 0.95], -0.6, 0.4),
        "marte" => g("MARTE", "El planeta rojo. Un rover soltero.", [1.0, 0.5, 0.25], 0.8, 0.5),
        "jupiter" => g("JÚPITER", "Sin piso. Con tormentas. Con todo.", [1.0, 0.7, 0.4], -1.0, 1.0),
        "saturno_ritmo" => g("SATURNO RITMO", "Esta noche: Miku en vivo.", [1.0, 0.45, 0.8], 1.1, 0.9),
        "urano" => g("URANO", "Acostado desde hace cuatro mil millones de años.", [0.5, 0.95, 1.0], -0.7, 0.7),
        "neptuno" => g("NEPTUNO", "Noche de cartas. Vientos de 2,000 km/h.", [0.4, 0.6, 1.0], 0.7, 0.6),
        "pluton" => g("PLUTÓN", "Planeta. (En disputa.)", [0.95, 0.8, 0.65], -0.5, 0.4),
        "hada" => g("PLANETA HADA", "Proyecto 2: la Gran Fuente de las Hadas.", [1.0, 0.55, 0.9], 0.6, 0.7),
        "chapin" => g("PLANETA CHAPÍN", "Terminal de buses. Destino: la capital. ¡Hay campo!", [0.55, 0.85, 1.0], 0.4, 0.6),
        _ => g("", "", [1.0, 0.6, 0.7], 0.5, 0.6),
    }
}

/// Lo que cuenta la IA al posarse en cada planeta.
pub fn dato(planeta: usize) -> &'static str {
    match CUERPOS[planeta].archivo {
        "mercurio" => "Mercurio. Hay una carpa de tarot y la adivina es el planeta. Dice que está en retrógrado. Dice que siempre está en retrógrado.",
        "chapin" => "Planeta Chapín. Arena negra, ceibas, volcanes que echan lava y Tikal entre la selva. Huele a café y a shuco. El Volcán de Fuego saluda: lo hace cada quince minutos.",
        "venus" => "Venus, el planeta del amor. Por eso aquí está el juzgado de divorcios. Gira al revés, como los matrimonios. Saturno nos está esperando.",
        "tierra" => "La Tierra. Detectamos la UVG. Las luces parpadean. Se oye una regla. Nadie estudió.",
        "planeta_vegetta" => "Planeta Vegetta. Un cubo. Literal: el planeta es un cubo. Detecto a dos youtubers y una pista de parkour.",
        "luna" => "La Luna siempre le enseña la misma cara a la Tierra. Hoy detecto... ¿un set de filmación? Y una silla que dice DIRECTOR.",
        "marte" => "Marte. Aquí vive un rover que en 2013 se cantó feliz cumpleaños solito. Hoy le conseguimos cita.",
        "jupiter" => "Júpiter no tiene piso. Estamos parados sobre gas. No pregunte de quién.",
        "saturno_ritmo" => "Saturno flotaría en agua. Y tiene escenario propio. Mejor vida social que nosotros.",
        "urano" => "Urano rueda acostado a 98 grados. Nadie explica nada. Igual que el reglamento de la U.",
        "neptuno" => "Neptuno: vientos de 2,000 km/h. Sujete las coletas. Y la dignidad.",
        "pluton" => "Plutón dejó de ser planeta en 2006. Desde entonces no sale de su cuarto. No lo mire feo.",
        "hada" => "Planeta Hada. La cueva tiene un portal al Proyecto 2. Detecto hadas. Y una canción pegajosa.",
        _ => "Aterrizaje completado.",
    }
}

impl Aterrizaje {
    pub fn new(planeta: usize, nave: &Nave, miku_baja: bool) -> Self {
        Aterrizaje { planeta, fase: Fase::Aproximacion, t: 0.0, desde: nave.pos, yaw0: nave.yaw, miku_baja, quiere_despegar: false }
    }

    /// Empieza ya posado y despegando (al volver de una escena).
    pub fn despegando(planeta: usize, nave: &Nave) -> Self {
        Aterrizaje { planeta, fase: Fase::Arranque, t: 0.0, desde: nave.pos, yaw0: nave.yaw, miku_baja: false, quiere_despegar: false }
    }

    pub fn posado(&self) -> bool {
        self.fase == Fase::Posado
    }

    /// Salta la cutscene: la nave queda tocando el suelo.
    pub fn saltar(&mut self, _t_mundo: f32) {
        if matches!(self.fase, Fase::Aproximacion | Fase::Descenso) {
            self.cambiar(Fase::Descenso);
            self.t = DESCENDER;
        }
    }

    /// Si se esta en la parte de cine (franjas, titulo, sin HUD).
    pub fn en_cine(&self) -> bool {
        matches!(self.fase, Fase::Aproximacion | Fase::Descenso | Fase::Contacto) || (self.fase == Fase::Posado && self.t < 1.5)
    }

    /// Que tanto se ven las franjas de cine (0 a 1).
    pub fn franjas(&self) -> f32 {
        match self.fase {
            Fase::Aproximacion => suave(self.t / 0.6),
            Fase::Descenso | Fase::Contacto => 1.0,
            Fase::Posado => 1.0 - suave(self.t / 1.5),
            _ => 0.0,
        }
    }

    /// La tarjeta con el nombre del planeta: (titulo, subtitulo, que tanto se ve).
    pub fn tarjeta(&self) -> (&'static str, &'static str, f32) {
        let g = guion(self.planeta);
        let alfa = match self.fase {
            Fase::Descenso => suave(self.t / 0.8),
            Fase::Contacto => 1.0,
            Fase::Posado => 1.0 - suave(self.t / 1.2),
            _ => 0.0,
        };
        (g.titulo, g.subtitulo, alfa)
    }

    /// El color de la entrada a la atmosfera de este planeta.
    pub fn color_entrada(&self) -> [f32; 3] {
        guion(self.planeta).entrada
    }

    fn arriba_vec() -> Vec3 {
        Vec3::new(0.0, 1.0, 0.0)
    }

    /// Donde queda el centro de la nave parada sobre el planeta.
    fn arriba(&self, t_mundo: f32) -> Vec3 {
        let c = &CUERPOS[self.planeta];
        c.posicion(t_mundo) + Self::arriba_vec() * (c.radio_cuerpo() + ALTURA_NAVE)
    }

    /// Que tan alto empieza el descenso vertical (sobre el punto de aterrizaje).
    fn altura_descenso(&self) -> f32 {
        5.0 + CUERPOS[self.planeta].radio_cuerpo() * 0.4
    }

    /// Altura sobre el suelo de la nave en `pos` (0: tocando).
    pub fn altitud(&self, pos: Vec3, t_mundo: f32) -> f32 {
        (pos - self.arriba(t_mundo)).dot(Self::arriba_vec()).max(0.0)
    }

    pub fn actualizar(&mut self, nave: &mut Nave, despegar: bool, dt: f32, t_mundo: f32) -> Evento {
        self.t += dt;
        let antes = nave.pos;
        let mut evento = Evento::Nada;
        let arriba = self.arriba(t_mundo);
        let alto = arriba + Self::arriba_vec() * self.altura_descenso();
        match self.fase {
            Fase::Aproximacion => {
                let k = (self.t / APROXIMAR).min(1.0);
                // En curva: primero avanza y despues cae sobre el punto.
                let control = self.desde.lerp(alto, 0.6) + Self::arriba_vec() * (self.altura_descenso() * 0.6);
                nave.pos = curva(self.desde, control, alto, suave(k));
                // Gira para quedar mirando hacia el Sol (le da la luz en la cara).
                let (yaw, _) = rumbo_hacia(CUERPOS[self.planeta].posicion(t_mundo) * -1.0 + Vec3::new(0.0, 0.0, 0.001));
                let tau = 2.0 * PI;
                let giro = (yaw - self.yaw0).rem_euclid(tau);
                let giro = if giro > PI { giro - tau } else { giro };
                nave.yaw = self.yaw0 + giro * suave(k);
                // La nariz baja al entrar y se nivela al final; se ladea hacia el giro.
                nave.pitch = -0.3 * (k * PI).sin();
                nave.roll = 0.35 * giro.signum() * (k * PI).sin();
                if k >= 1.0 {
                    self.cambiar(Fase::Descenso);
                }
            }
            Fase::Descenso => {
                let k = (self.t / DESCENDER).min(1.0);
                // Rapido arriba y frenando cerca del suelo.
                let h = self.altura_descenso() * (1.0 - k) * (1.0 - k);
                let bamboleo = Vec3::new(0.12 * (self.t * 1.9).sin(), 0.0, 0.08 * (self.t * 1.3).cos()) * (1.0 - k);
                nave.pos = arriba + Self::arriba_vec() * h + bamboleo;
                nave.pitch *= 1.0 - (6.0 * dt).min(1.0);
                nave.roll = 0.06 * (self.t * 2.3).sin() * (1.0 - k);
                if k >= 1.0 {
                    self.cambiar(Fase::Contacto);
                    evento = Evento::Toque;
                }
            }
            Fase::Contacto => {
                // La suspension: se aplasta y rebota, cada vez menos.
                let k = self.t / CONTACTO;
                let rebote = -0.09 * (k * PI * 2.0).sin() * (1.0 - k).max(0.0);
                nave.pos = arriba + Self::arriba_vec() * rebote;
                nave.roll = 0.0;
                if k >= 1.0 {
                    self.cambiar(Fase::Posado);
                    evento = Evento::Posada;
                }
            }
            Fase::Posado => {
                nave.pos = arriba;
                self.quiere_despegar |= despegar;
                if self.quiere_despegar && self.t > BRINCO_MIKU {
                    self.cambiar(Fase::Arranque);
                }
            }
            Fase::Arranque => {
                // Miku vuelve primero a la burbuja; los motores se encienden temblando.
                let espera = if self.miku_baja { BRINCO_MIKU } else { 0.0 };
                let k = ((self.t - espera) / ARRANCAR).clamp(0.0, 1.0);
                nave.pos = arriba + Self::arriba_vec() * (0.15 * k * k) + Vec3::new(0.02 * (self.t * 47.0).sin(), 0.0, 0.0) * k;
                if self.t >= espera + ARRANCAR {
                    self.cambiar(Fase::Subiendo);
                    self.desde = nave.pos;
                }
            }
            Fase::Subiendo => {
                // Acelera cada vez mas: despacio al principio, de golpe al final.
                let k = (self.t / SUBIR).min(1.0);
                let afuera = nave.adelante() * 4.0;
                let destino = alto + afuera + Self::arriba_vec() * 2.0;
                nave.pos = self.desde.lerp(destino, k * k * k);
                nave.pitch = 0.25 * k;
                if k >= 1.0 {
                    nave.velocidad = 6.0;
                    evento = Evento::Despego;
                }
            }
        }
        nave.velocidad = if matches!(evento, Evento::Despego) { nave.velocidad } else { 0.0 };
        nave.vel = (nave.pos - antes) * (1.0 / dt.max(1e-4));
        evento
    }

    fn cambiar(&mut self, fase: Fase) {
        self.fase = fase;
        self.t = 0.0;
    }

    /// Que tanto empujan los motores: a fondo bajando, mas al arrancar.
    pub fn empuje(&self) -> f32 {
        match self.fase {
            Fase::Aproximacion => 0.8,
            Fase::Descenso => 1.3,
            Fase::Contacto => 0.5 * (1.0 - self.t / CONTACTO),
            Fase::Posado => 0.0,
            Fase::Arranque => 1.5 * (self.t / ARRANCAR).min(1.0),
            Fase::Subiendo => 1.6,
        }
    }

    /// El resplandor de entrada a la atmosfera (0 a 1).
    pub fn entrada(&self) -> f32 {
        if self.fase != Fase::Aproximacion {
            return 0.0;
        }
        let k = self.t / APROXIMAR;
        if (0.25..0.95).contains(&k) { ((k - 0.25) / 0.7 * PI).sin() } else { 0.0 }
    }

    /// Cuanto tiembla la camara ahorita (la entrada, los motores, el contacto).
    pub fn temblor(&self) -> f32 {
        match self.fase {
            Fase::Aproximacion => 0.35 * self.entrada(),
            Fase::Descenso => 0.08 + 0.12 * (self.t / DESCENDER),
            Fase::Arranque => 0.3 * (self.t / ARRANCAR).min(1.0),
            Fase::Subiendo => 0.25 * (1.0 - self.t / SUBIR).max(0.0),
            _ => 0.0,
        }
    }

    /// Lo que dice el tablero de la nave.
    pub fn etapa(&self) -> &'static str {
        match self.fase {
            Fase::Aproximacion if self.entrada() > 0.2 => "ENTRADA ATMOSFÉRICA",
            Fase::Aproximacion => "APROXIMACIÓN",
            Fase::Descenso => "DESCENSO",
            Fase::Contacto => "¡CONTACTO!",
            Fase::Posado => "EN TIERRA",
            Fase::Arranque => "ENCENDIENDO MOTORES",
            Fase::Subiendo => "DESPEGUE",
        }
    }

    /// La camara, como en una cinematica: tomas FIJAS con cortes.
    /// 1. Plano general: la camara espera adelante, del lado del Sol, y ve llegar
    ///    la nave desde lejos (solo gira para no perderla, como un camarografo).
    /// 2. Contrapicado a ras de suelo para el descenso y el contacto.
    /// 3. Ya posada, una vuelta lenta alrededor.
    /// 4. Al despegar, desde el suelo la ve irse, y al final vuelve a seguirla.
    pub fn camara(&self, nave: &Nave, t_mundo: f32) -> (Vec3, Vec3) {
        let c = &CUERPOS[self.planeta];
        let g = guion(self.planeta);
        let r = c.radio_cuerpo();
        let suelo = self.arriba(t_mundo) - Self::arriba_vec() * ALTURA_NAVE;
        let al_sol = c.posicion(t_mundo) * -1.0;
        let base = al_sol.z.atan2(al_sol.x);
        let lado = |a: f32| Vec3::new(a.cos(), 0.0, a.sin());
        // 1. Plano general.
        let general_ojo = suelo + lado(base + g.angulo) * (7.5 + r * 0.9) + Self::arriba_vec() * (self.altura_descenso() * g.altura + 1.0);
        let general = (general_ojo, nave.pos.lerp(suelo, 0.3));
        // 2. Contrapicado a ras de suelo, del otro lado.
        let ras_ojo = suelo + lado(base - g.angulo * 0.6) * (4.2 + r * 0.45) + Self::arriba_vec() * (0.55 + r * 0.06);
        let a_ras = (ras_ojo, nave.pos.lerp(suelo, 0.2) + Self::arriba_vec() * 0.25);
        // 3. Vuelta lenta (con el reloj de la cutscene, que las orbitas estan congeladas).
        let vuelta = base - g.angulo * 0.6 + 0.35 * suave(self.t / 6.0) + 0.05 * self.t;
        let orbita = (suelo + lado(vuelta) * (4.4 + r * 0.45) + Self::arriba_vec() * (1.5 + r * 0.12), suelo + Self::arriba_vec() * 0.6);
        let seguir = Camara::ideal(nave);
        let mezcla = |a: (Vec3, Vec3), b: (Vec3, Vec3), k: f32| (a.0.lerp(b.0, k), a.1.lerp(b.1, k));
        match self.fase {
            // Al principio un instante siguiendola, y corte al plano general.
            Fase::Aproximacion if self.t < 0.35 => seguir,
            Fase::Aproximacion => general,
            Fase::Descenso | Fase::Contacto => a_ras,
            Fase::Posado => mezcla(a_ras, orbita, suave(self.t / 1.5)),
            Fase::Arranque => mezcla(orbita, a_ras, suave(self.t / 0.6)),
            Fase::Subiendo => mezcla(a_ras, seguir, suave((self.t - SUBIR * 0.55) / (SUBIR * 0.45))),
        }
    }

    /// Miku afuera, parada sobre el planeta (o brincando entre el planeta y
    /// la burbuja): su matriz, o `None` si esta adentro.
    pub fn miku(&self, asiento: Vec3, t_mundo: f32) -> Option<Mat4> {
        if !self.miku_baja {
            return None;
        }
        let k = match self.fase {
            Fase::Posado => (self.t / BRINCO_MIKU).min(1.0),
            Fase::Arranque => 1.0 - (self.t / BRINCO_MIKU).min(1.0),
            _ => return None,
        };
        if k <= 0.0 {
            return None;
        }
        let c = &CUERPOS[self.planeta];
        // Un punto en la superficie, a un lado de la nave, y "arriba" es la normal ahi.
        let normal = Vec3::new(0.55, 1.0, -0.45).normalize();
        let piso = c.posicion(t_mundo) + normal * c.radio_cuerpo();
        let pos = asiento.lerp(piso, suave(k)) + Vec3::new(0.0, 0.8 * (k * PI).sin(), 0.0);
        let x = normal.cross(Vec3::new(0.0, 0.0, 1.0)).normalize();
        let z = x.cross(normal);
        // De pie sobre la normal del planeta y girada para ver a la camara.
        let base = Mat4::from_basis(x, normal, z) * Mat4::rotation_y(0.6);
        Some(Mat4::translation(pos) * base * Mat4::scale(ESCALA_MIKU))
    }

    /// El polvo que levantan los motores: rayas que salen del suelo bajo la
    /// nave. Mas cerca del piso, mas polvo; al tocar, un anillo que se abre.
    pub fn dibujar_polvo(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4, nave: Vec3, t_mundo: f32) {
        let suelo = self.arriba(t_mundo) - Self::arriba_vec() * (ALTURA_NAVE - 0.05);
        let color = color_polvo(self.planeta);
        let altura = self.altitud(nave, t_mundo);
        // Que tanto polvo: por la altura (bajando o subiendo con los motores) o por el golpe.
        let (fuerza, radio) = match self.fase {
            Fase::Descenso | Fase::Subiendo | Fase::Arranque if altura < ALTURA_POLVO => {
                let cerca = 1.0 - altura / ALTURA_POLVO;
                let motor = if self.fase == Fase::Arranque { (self.t / ARRANCAR).min(1.0) } else { 1.0 };
                (cerca * motor, 0.6 + 1.6 * cerca)
            }
            Fase::Contacto => (1.0, 1.4 + 2.2 * self.t / CONTACTO),
            Fase::Posado if self.t < 1.2 => (1.0 - self.t / 1.2, 2.4 + 1.5 * self.t),
            _ => return,
        };
        if fuerza <= 0.0 {
            return;
        }
        for i in 0..40 {
            let a = i as f32 * PI * 2.0 / 40.0 + t_mundo * 0.7 + (i % 3) as f32 * 0.2;
            let largo = 0.5 + 0.4 * ((i * 7) % 5) as f32 / 5.0;
            let dir = Vec3::new(a.cos(), 0.08 + 0.05 * ((i * 3) % 4) as f32, a.sin());
            let r0 = radio * (0.45 + 0.1 * ((i * 11) % 3) as f32);
            if let (Some(p0), Some(p1)) =
                (proyectar(suelo + dir * r0, vista_proyeccion, fb), proyectar(suelo + dir * (r0 + largo * radio * 0.6), vista_proyeccion, fb))
            {
                let c = color.map(|v| v * fuerza * 0.7);
                fb.linea_de_luz(p0, p1, c, [0.0; 3]);
            }
        }
    }
}
