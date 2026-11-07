//! Aterrizaje en Neptuno: noche de cartas. Se juega ¡NEPTUNO!, que es como
//! UNO pero en el planeta con los vientos mas rapidos del sistema solar.
//!
//! Juegan cuatro: Miku (la piloto, con su mano de cartas enfrente de la
//! camara), el Doom Slayer, el Master Chief y Cloud Strife (fan art voxel).
//! Neptuno es el anfitrion: flota gigante detras de la mesa, comenta todo y
//! controla el viento.
//!
//! Reglas: las del UNO de mesa. Color con color o simbolo con simbolo; el Comodin va sobre lo
//! que sea. El +4 solo es legal sin cartas del color que manda: el siguiente puede aceptarlo
//! (roba 4 y pierde el turno) o RETARLO (si era ilegal, el que lo tiro roba 4; si era legal, el
//! que reto roba 6 y pierde el turno). La carta que se voltea al repartir tambien cuenta. Si un
//! jugador se queda sin cartas con un +2 o un +4, el siguiente igual roba. Al quedarse con una
//! hay que gritar "¡NEPTUNO!" o se roban dos.
//!
//! Dos cosas de la casa: sin jugada se roba HASTA que salga una que se pueda jugar (el usuario lo
//! pidio asi; el reglamento de Mattel dice robar una sola), y las rafagas de 2,000 km/h ya solo
//! sacuden la mesa: no le quitan cartas a nadie.
//!
//! Y a veces llegan visitas que interrumpen la partida: Canela (de Animal
//! Crossing) llega corriendo a decirle al Doom Slayer que Tom Nook lo busca
//! por la hipoteca; el Slayer se va y Canela juega sus cartas hasta que
//! regresa. O Plutón llega rodando a pedir que lo dejen jugar.

use crate::vida::{Tam, Tono, Vida};
use crate::entrada::Entrada;
use crate::matematica::{Mat4, Vec3};
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;
use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tinte {
    Rojo,
    Amarillo,
    Verde,
    Azul,
}

const TINTES: [Tinte; 4] = [Tinte::Rojo, Tinte::Amarillo, Tinte::Verde, Tinte::Azul];

impl Tinte {
    fn nombre(self) -> &'static str {
        match self {
            Tinte::Rojo => "ROJO",
            Tinte::Amarillo => "AMARILLO",
            Tinte::Verde => "VERDE",
            Tinte::Azul => "AZUL",
        }
    }

    fn parte(self) -> &'static str {
        match self {
            Tinte::Rojo => "color_rojo",
            Tinte::Amarillo => "color_amarillo",
            Tinte::Verde => "color_verde",
            Tinte::Azul => "color_azul",
        }
    }

    fn rgb(self) -> Color {
        match self {
            Tinte::Rojo => Color::new(235, 50, 70, 255),
            Tinte::Amarillo => Color::new(250, 200, 40, 255),
            Tinte::Verde => Color::new(60, 180, 90, 255),
            Tinte::Azul => Color::new(50, 110, 245, 255),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Valor {
    Numero(u8),
    Salta,
    Reversa,
    Mas2,
    Comodin,
    Mas4,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Carta {
    /// `None`: carta negra (comodin o +4).
    pub tinte: Option<Tinte>,
    pub valor: Valor,
}

const COLORES_CARTA: [&str; 5] = ["color_rojo", "color_amarillo", "color_verde", "color_azul", "color_negro"];
const SIMBOLOS: [&str; 15] = ["s_0", "s_1", "s_2", "s_3", "s_4", "s_5", "s_6", "s_7", "s_8", "s_9", "s_salta", "s_reversa", "s_mas2", "s_mas4", "s_comodin"];

impl Carta {
    fn simbolo(&self) -> &'static str {
        match self.valor {
            Valor::Numero(n) => SIMBOLOS[n as usize],
            Valor::Salta => "s_salta",
            Valor::Reversa => "s_reversa",
            Valor::Mas2 => "s_mas2",
            Valor::Mas4 => "s_mas4",
            Valor::Comodin => "s_comodin",
        }
    }

    /// Las partes de `carta.obj` que hay que ocultar para que se vea esta carta.
    pub fn ocultar(&self) -> Vec<&'static str> {
        let color = self.tinte.map_or("color_negro", Tinte::parte);
        let simbolo = self.simbolo();
        COLORES_CARTA.iter().chain(SIMBOLOS.iter()).copied().filter(|&p| p != color && p != simbolo).collect()
    }

    fn comodin(&self) -> bool {
        self.tinte.is_none()
    }
}

/// Lo que hay que ocultar para una carta boca abajo (solo se ve el reverso).
pub const BOCA_ABAJO: [&str; 3] = ["color_", "s_", "rombo"];

/// Si `c` se puede jugar sobre `arriba`, con `color` como el color que manda.
pub fn se_puede(c: Carta, arriba: Carta, color: Tinte) -> bool {
    c.comodin() || c.tinte == Some(color) || (!arriba.comodin() && c.valor == arriba.valor)
}

/// Las 108 cartas: por color un 0, dos de cada 1 a 9, dos Salta, dos Reversa
/// y dos +2; y cuatro Comodines y cuatro +4.
pub fn mazo_completo() -> Vec<Carta> {
    let mut v = Vec::new();
    for t in TINTES {
        let c = |valor| Carta { tinte: Some(t), valor };
        v.push(c(Valor::Numero(0)));
        for _ in 0..2 {
            for n in 1..=9 {
                v.push(c(Valor::Numero(n)));
            }
            v.extend([c(Valor::Salta), c(Valor::Reversa), c(Valor::Mas2)]);
        }
    }
    for _ in 0..4 {
        v.push(Carta { tinte: None, valor: Valor::Comodin });
        v.push(Carta { tinte: None, valor: Valor::Mas4 });
    }
    v
}

pub const MIKU: usize = 0;
const SLAYER: usize = 1;
const CHIEF: usize = 2;
const CLOUD: usize = 3;
pub const JUGADORES: usize = 4;
const NOMBRES: [&str; JUGADORES] = ["MIKU", "DOOM SLAYER", "MASTER CHIEF", "CLOUD"];
const NEPTUNO: &str = "NEPTUNO";

/// Donde se para cada uno alrededor de la mesa (Miku de espaldas a la camara: se ve sobre su hombro).
const ASIENTOS: [Vec3; JUGADORES] = [Vec3::new(-1.5, 0.0, -1.9), Vec3::new(-2.0, 0.0, 0.35), Vec3::new(0.0, 0.0, 2.05), Vec3::new(2.0, 0.0, 0.35)];
/// La altura de la mesa (la de la junta de Pluton, escalada 1.6).
pub const ESCALA_MESA: f32 = 1.6;
const ALTO_MESA: f32 = 0.45 * ESCALA_MESA;
const PILA: Vec3 = Vec3::new(0.25, ALTO_MESA + 0.01, -0.1);
/// La carta en juego (la de arriba de la pila) flota sobre la pila, grande e
/// inclinada hacia la camara, para que siempre se vea cual es.
const EN_JUEGO: Vec3 = Vec3::new(0.2, ALTO_MESA + 0.42, -0.35);
const INCLINACION_EN_JUEGO: f32 = 0.55;
const ESCALA_EN_JUEGO: f32 = 1.3;
const MAZO: Vec3 = Vec3::new(-0.45, ALTO_MESA + 0.01, 0.05);
/// Segundos que piensa cada bot antes de jugar.
const PIENSA: f32 = 1.1;
/// Segundos entre carta y carta al robar varias.
const ROBO_PASO: f32 = 0.2;
/// Segundos para gritar ¡NEPTUNO! despues de quedarse con una.
const PARA_GRITAR: f32 = 1.8;

const INTRO: [(&str, &str, Voz); 7] = [
    (NEPTUNO, "¡Bienvenida a Neptuno! Vientos de 2,000 km/h, lluvia de diamantes... y noche de cartas. Hoy se juega ¡NEPTUNO!", Voz::Neptuno),
    (NEPTUNO, "Es como UNO, pero en mi planeta y con viento. Color con color, número con número. Si te queda una carta, gritas mi nombre.", Voz::Neptuno),
    (NEPTUNO, "Los invitados de hoy: el Doom Slayer, el Master Chief y Cloud Strife. No pregunten cómo llegaron. Yo tampoco sé.", Voz::Neptuno),
    ("DOOM SLAYER", "...", Voz::Grunido),
    (NEPTUNO, "Dice que viene a romper y desgarrar. Las cartas no, por favor, que son del mazo.", Voz::Neptuno),
    ("MASTER CHIEF", "Señorita. Terminando esta partida. Cloud, deje de mirar al horizonte.", Voz::Radio),
    ("CLOUD", "No me interesa. ...Bueno, reparte.", Voz::Planeta),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fase {
    Intro(usize),
    Juego,
    Fin,
}

/// Quien interrumpe la partida.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Visita {
    /// Canela llega por el Slayer.
    LlegaCanela,
    /// El Slayer regresa y Canela se despide.
    RegresaSlayer,
    /// Plutón llega rodando a pedir jugar.
    Pluton,
}

impl Visita {
    fn guion(self) -> &'static [(&'static str, &'static str, Voz)] {
        match self {
            Visita::LlegaCanela => &[
                (NEPTUNO, "¡Un momento! Alguien viene corriendo hacia la mesa...", Voz::Neptuno),
                ("CANELA", "¡Buenos días! ¡Perdón, perdón! Soy Canela, de la isla. ¿Está por aquí el señor Slayer?", Voz::Animalito),
                ("DOOM SLAYER", "...", Voz::Grunido),
                ("CANELA", "Señor Slayer, el señor Nook dice que la hipoteca de su casa vence hoy. ¡Y hay demonios en el museo otra vez! Se tiene que ir. ¡Ya!", Voz::Animalito),
                ("DOOM SLAYER", "*se levanta despacio, le da una palmadita en la cabeza a Canela y se va sin decir nada*", Voz::Grunido),
                ("CANELA", "¡Yo le cuido las cartas! No sé jugar, pero aprendo rapidito. ¡Qué emoción!", Voz::Animalito),
                (NEPTUNO, "Esto es oficialmente lo más tierno que ha pasado en mi planeta. Sigan jugando.", Voz::Neptuno),
            ],
            Visita::RegresaSlayer => &[
                ("DOOM SLAYER", "*regresa cubierto de demonio. Trae la hipoteca pagada. Y un fósil para el museo.*", Voz::Grunido),
                ("CANELA", "¡Bienvenido, señor Slayer! Le cuidé las cartas. ¡Que tengan un día maravilloso! ¡Adiós a todos!", Voz::Animalito),
                (NEPTUNO, "Se fue Canela. El Slayer sonrió. Creo. Con el casco no se sabe.", Voz::Neptuno),
            ],
            Visita::Pluton => &[
                (NEPTUNO, "¿Y eso? Algo viene rodando...", Voz::Neptuno),
                ("PLUTÓN", "Hola... ¿puedo jugar? Traje mis propias cartas. Son cinco. Como mis lunas.", Voz::Pluton),
                (NEPTUNO, "Plutón, aquí solo juegan planetas. Bueno, y gente de videojuegos. Y a veces una perrita.", Voz::Neptuno),
                ("PLUTÓN", "...Ok. Me voy a mi cuarto. No me sigan. Bueno, síganme un poquito.", Voz::Pluton),
                ("MIKU", "¡Pobre Plutón! ¡En la próxima partida juegas con nosotros, te lo prometo!", Voz::Miku),
            ],
        }
    }
}

/// Lo que se dibuja alrededor de la mesa: los rivales y las visitas.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Figura {
    /// Uno de los rivales (1 Slayer, 2 Chief, 3 Cloud).
    Rival(usize),
    Canela,
    Pluton,
}

/// Donde se para Canela mientras le habla al Slayer, y por donde se van y llegan.
const JUNTO_AL_SLAYER: Vec3 = Vec3::new(-1.55, 0.0, -0.55);
const PUERTA_IZQUIERDA: Vec3 = Vec3::new(-6.5, 0.0, 0.5);
const PUERTA_DERECHA: Vec3 = Vec3::new(6.5, 0.6, -1.0);
const LUGAR_PLUTON: Vec3 = Vec3::new(2.4, 0.9, -1.25);

pub enum AccionNeptuno {
    Nada,
    Despegar,
}

/// Una carta volando de un lugar a otro (al jugarla, al robarla, con el viento).
struct Vuelo {
    carta: Option<Carta>,
    desde: Vec3,
    hasta: Vec3,
    t: f32,
    dura: f32,
    vueltas: f32,
    a_la_pila: bool,
}

/// Un +4 recien jugado: quien lo tiro, a quien le toca y si era ilegal (tenia del color).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Reto {
    de: usize,
    a: usize,
    ilegal: bool,
    color_anterior: Tinte,
}

pub struct Neptuno {
    fase: Fase,
    t: f32,
    pub total: f32,
    pub manos: [Vec<Carta>; JUGADORES],
    mazo: Vec<Carta>,
    pila: Vec<Carta>,
    pub color: Tinte,
    pub turno: usize,
    /// 1 en el sentido normal, -1 despues de una Reversa.
    pub sentido: i32,
    /// Segundos en el turno actual.
    t_turno: f32,
    cursor: usize,
    /// La carta que Miku robo en este turno (despues de robar, solo esa se puede jugar).
    robo: Option<Carta>,
    /// Miku no tiene jugada: va robando sola hasta que salga una.
    robando: bool,
    /// Un +4 esperando respuesta de Miku (aceptar o retar), y lo que tiene escogido.
    reto: Option<Reto>,
    cursor_reto: usize,
    /// El Comodin que se volteo al repartir: el primero en jugar escoge el color.
    color_libre: bool,
    /// El letrero grande de la jugada (+4, SALTA, etc.): texto, color y segundos que lleva.
    /// Apagarlo en pruebas: los bots nunca retan.
    pub bots_retan: bool,
    /// La carta comodin que Miku esta por jugar, mientras elige el color.
    eligiendo: Option<usize>,
    cursor_color: usize,
    /// Si Miku ya grito ¡NEPTUNO! en este turno, y cuanto le queda para gritar.
    grito: bool,
    pendiente: Option<f32>,
    /// Segundos desde el ultimo grito (para el letrero).
    desde_grito: f32,
    jugadas: u32,
    proxima_rafaga: u32,
    desde_rafaga: f32,
    pub ganador: Option<usize>,
    /// Quien jugo la carta en juego (None: la que se volteo al repartir).
    ultimo: Option<usize>,
    /// La visita en curso, en que linea de su guion va, y cuanto lleva en ella.
    visita: Option<(Visita, usize, f32)>,
    /// Si Canela esta sentada jugando por el Slayer, y cuantas jugadas lleva.
    canela_en_mesa: bool,
    jugadas_canela: u32,
    /// Cuando viene la siguiente visita (en jugadas), y cuales ya vinieron en esta partida.
    proxima_visita: u32,
    vino_canela: bool,
    vino_pluton: bool,
    /// Cuantas partidas van (con las revanchas): las visitas se turnan para no repetirse seguido.
    partidas: u32,
    /// Si Canela y Pluton ya habian venido en otra visita a Neptuno (lo pone el juego, del
    /// pasaporte): entonces ya no llegan siempre, solo a veces.
    pub ya_vinieron: (bool, bool),
    /// Quien viene en esta partida (si alguien): se decide al repartir.
    visita_de_la_partida: Option<Visita>,
    /// Segundos desde que termino la ultima visita (para que las que se van, se vayan caminando).
    desde_visita: f32,
    ultima_visita: Option<Visita>,
    pub victorias: [u32; JUGADORES],
    comentario: Option<(&'static str, String, Voz)>,
    vuelos: Vec<Vuelo>,
    semilla: u32,
    /// Para las pruebas y las capturas: Miku juega sola, como un bot.
    pub miku_automatica: bool,
    /// Los letreros gigantes, el confeti y las sacudidas (ver vida.rs).
    pub vida: Vida,
    pub eventos: Vec<Efecto>,
}

impl Neptuno {
    pub fn new(semilla: u32) -> Self {
        let mut n = Neptuno {
            fase: Fase::Intro(0),
            t: 0.0,
            total: 0.0,
            manos: Default::default(),
            mazo: Vec::new(),
            pila: Vec::new(),
            color: Tinte::Rojo,
            turno: MIKU,
            sentido: 1,
            t_turno: 0.0,
            cursor: 0,
            robo: None,
            robando: false,
            reto: None,
            cursor_reto: 0,
            color_libre: false,
            bots_retan: true,
            eligiendo: None,
            cursor_color: 0,
            grito: false,
            pendiente: None,
            desde_grito: 10.0,
            jugadas: 0,
            proxima_rafaga: 6,
            desde_rafaga: 10.0,
            ganador: None,
            ultimo: None,
            visita: None,
            canela_en_mesa: false,
            jugadas_canela: 0,
            proxima_visita: 7,
            vino_canela: false,
            partidas: 0,
            ya_vinieron: (false, false),
            visita_de_la_partida: None,
            vino_pluton: false,
            desde_visita: 10.0,
            ultima_visita: None,
            victorias: [0; JUGADORES],
            comentario: None,
            vuelos: Vec::new(),
            semilla: semilla | 1,
            miku_automatica: false,
            vida: Vida::sin_cartel(),
            eventos: Vec::new(),
        };
        n.repartir();
        n
    }

    fn azar(&mut self) -> f32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) as f32 / (1u32 << 24) as f32
    }

    fn barajar(&mut self, v: &mut [Carta]) {
        for i in (1..v.len()).rev() {
            let j = ((self.azar() * (i + 1) as f32) as usize).min(i);
            v.swap(i, j);
        }
    }

    /// Baraja todo y da siete a cada uno. La primera carta de la pila vale como en el juego de
    /// mesa: un +4 regresa al mazo; las demas hacen efecto sobre el primero en jugar.
    fn repartir(&mut self) {
        let mut mazo = mazo_completo();
        self.barajar(&mut mazo);
        self.mazo = mazo;
        self.manos = Default::default();
        for _ in 0..7 {
            for j in 0..JUGADORES {
                let c = self.mazo.pop().unwrap();
                self.manos[j].push(c);
            }
        }
        while self.mazo.last().map(|c| c.valor) == Some(Valor::Mas4) {
            let c = self.mazo.pop().unwrap();
            self.mazo.insert(0, c);
        }
        let primera = self.mazo.pop().unwrap();
        self.color = primera.tinte.unwrap_or(Tinte::Rojo);
        self.pila = vec![primera];
        self.turno = MIKU;
        self.sentido = 1;
        self.t_turno = 0.0;
        self.cursor = 0;
        self.robo = None;
        self.robando = false;
        self.reto = None;
        self.color_libre = false;
        self.eligiendo = None;
        self.grito = false;
        self.pendiente = None;
        self.jugadas = 0;
        self.proxima_rafaga = 6;
        self.ganador = None;
        self.ultimo = None;
        self.vuelos.clear();
        self.visita = None;
        self.canela_en_mesa = false;
        self.vino_canela = false;
        self.vino_pluton = false;
        self.partidas += 1;
        // A media partida, no al principio (antes llegaba en la primera vuelta, cada vez).
        self.proxima_visita = 12 + (self.azar() * 10.0) as u32;
        self.visita_de_la_partida = self.escoger_visita();
        self.ordenar_mano();
        self.carta_inicial(primera);
    }

    /// Lo que hace la carta que se volteo. Reparte Cloud, asi que Miku juega primero y, con una
    /// Reversa, empieza el que reparte.
    fn carta_inicial(&mut self, primera: Carta) {
        match primera.valor {
            Valor::Salta => {
                self.turno = self.siguiente(MIKU, 0);
                self.anunciar("SALTA A MIKU", Color::new(255, 220, 120, 255));
            }
            Valor::Reversa => {
                self.sentido = -1;
                self.turno = CLOUD;
                self.anunciar("EMPIEZA CLOUD", Color::new(255, 220, 120, 255));
            }
            Valor::Mas2 => {
                self.robar(MIKU, 2);
                self.turno = self.siguiente(MIKU, 0);
                self.anunciar("+2 PARA MIKU", Color::new(255, 110, 130, 255));
            }
            Valor::Comodin => self.color_libre = true,
            _ => {}
        }
    }

    fn anunciar(&mut self, texto: impl Into<String>, color: Color) {
        let texto = texto.into();
        let tono = Tono::de_color(color);
        self.vida.grito(&texto, tono, Tam::Grande);
        self.vida.golpe(0.25, tono);
    }

    /// La mano de Miku ordenada por color y numero, para que sea facil de leer.
    fn ordenar_mano(&mut self) {
        let clave = |c: &Carta| {
            let color = c.tinte.map_or(4, |t| t as usize);
            let valor = match c.valor {
                Valor::Numero(n) => n as usize,
                Valor::Salta => 10,
                Valor::Reversa => 11,
                Valor::Mas2 => 12,
                Valor::Comodin => 13,
                Valor::Mas4 => 14,
            };
            color * 20 + valor
        };
        self.manos[MIKU].sort_by_key(clave);
        self.cursor = self.cursor.min(self.manos[MIKU].len().saturating_sub(1));
    }

    pub fn arriba(&self) -> Carta {
        *self.pila.last().unwrap()
    }

    /// Cuantas cartas hay en total (siempre deberian ser 108).
    #[cfg(test)]
    fn total_cartas(&self) -> usize {
        self.manos.iter().map(Vec::len).sum::<usize>() + self.mazo.len() + self.pila.len()
    }

    fn decir(&mut self, quien: &'static str, que: impl Into<String>, voz: Voz) {
        self.comentario = Some((quien, que.into(), voz));
    }

    fn siguiente(&self, j: usize, saltar: usize) -> usize {
        (j as i32 + self.sentido * (1 + saltar as i32)).rem_euclid(JUGADORES as i32) as usize
    }

    fn robar(&mut self, j: usize, n: usize) {
        for k in 0..n {
            if self.mazo.is_empty() {
                // Se rebaraja la pila (menos la de arriba).
                let arriba = self.pila.pop().unwrap();
                let mut resto = std::mem::take(&mut self.pila);
                self.barajar(&mut resto);
                self.mazo = resto;
                self.pila = vec![arriba];
            }
            if let Some(c) = self.mazo.pop() {
                self.manos[j].push(c);
                let hasta = self.lugar_mano(j);
                // Una por una, para que se vean y se puedan contar (cada una suena al salir).
                self.vuelos.push(Vuelo { carta: None, desde: MAZO, hasta, t: -(k as f32) * ROBO_PASO, dura: 0.35, vueltas: 0.0, a_la_pila: false });
            }
        }
        self.eventos.push(Efecto::Carta);
        if j == MIKU {
            self.ordenar_mano();
        }
    }

    fn pasar_turno(&mut self, saltar: usize) {
        self.turno = self.siguiente(self.turno, saltar);
        self.t_turno = 0.0;
        self.robo = None;
        self.robando = false;
        if self.turno == MIKU {
            self.grito = false;
        }
    }

    /// El jugador `j` juega la carta `i` de su mano (con el color elegido, si es comodin).
    fn jugar(&mut self, j: usize, i: usize, color: Option<Tinte>) {
        let c = self.manos[j].remove(i);
        let color_anterior = self.color;
        // El +4 es ilegal si en la mano queda alguna carta del color que mandaba.
        let ilegal = self.manos[j].iter().any(|k| k.tinte == Some(color_anterior));
        let desde = self.lugar_mano(j);
        self.pila.push(c);
        self.ultimo = Some(j);
        self.color = c.tinte.or(color).unwrap_or(self.color);
        self.vuelos.push(Vuelo { carta: Some(c), desde, hasta: EN_JUEGO, t: 0.0, dura: 0.35, vueltas: 0.0, a_la_pila: true });
        self.eventos.push(Efecto::Carta);
        if j == MIKU {
            self.cursor = self.cursor.min(self.manos[MIKU].len().saturating_sub(1));
        }
        if self.manos[j].is_empty() {
            // Se acabo la partida, pero el +2 o +4 de la ultima carta igual se roba.
            let victima = self.siguiente(j, 0);
            match c.valor {
                Valor::Mas2 => self.robar(victima, 2),
                Valor::Mas4 => self.robar(victima, 4),
                _ => {}
            }
            self.terminar(j);
            return;
        }
        if self.manos[j].len() == 1 {
            self.queda_una(j);
        }
        let victima = self.siguiente(j, 0);
        let mut saltar = 0;
        match c.valor {
            Valor::Salta => {
                saltar = 1;
                self.anunciar(format!("¡SALTA A {}!", self.nombre(victima)), Color::new(255, 220, 120, 255));
                if victima == MIKU {
                    self.decir(NEPTUNO, "Salta. Usted descanse, Miku. Se ve cansada de tanto ganar. Bueno, de tanto jugar.", Voz::Neptuno);
                }
            }
            Valor::Reversa => {
                self.sentido = -self.sentido;
                self.anunciar("¡REVERSA!", Color::new(120, 230, 255, 255));
                if j == CHIEF {
                    self.decir("MASTER CHIEF", "Reversa. Cambiando el flanco. Mi IA lo recomendó.", Voz::Radio);
                } else {
                    self.decir(NEPTUNO, "¡Reversa! Ahora giramos al revés. Como mi luna Tritón. Que no está invitada.", Voz::Neptuno);
                }
            }
            Valor::Mas2 => {
                self.robar(victima, 2);
                saltar = 1;
                self.comentar_ataque(j, victima, 2);
                self.anunciar(format!("+2 PARA {}", self.nombre(victima)), Color::new(255, 110, 130, 255));
            }
            Valor::Mas4 => {
                // No se roba todavia: primero se puede retar.
                let reto = Reto { de: j, a: victima, ilegal, color_anterior };
                self.anunciar("+4", Color::new(255, 110, 130, 255));
                self.jugadas += 1;
                if victima == MIKU && !self.miku_automatica {
                    self.reto = Some(reto);
                    self.cursor_reto = 0;
                    self.decir(NEPTUNO, format!("¡+4 de {}! Pide {}. Puede aceptarlo o retarlo, Miku.", nombre_corto(j), self.color.nombre()), Voz::Neptuno);
                } else {
                    let retar = self.bots_retan && self.bot_reta(victima);
                    self.resolver_reto(reto, retar);
                }
                return;
            }
            _ => {}
        }
        if c.comodin() {
            let color = self.color.nombre();
            if !matches!(self.comentario, Some((q, _, _)) if q == self.nombre(j)) && j != MIKU {
                let quien = if self.es_canela(j) { "Canela" } else { ["Miku", "El Slayer", "El Chief", "Cloud"][j] };
                self.decir(NEPTUNO, format!("Comodín. {quien} pide {color}."), Voz::Neptuno);
            }
        }
        self.jugadas += 1;
        self.tras_la_jugada();
        // El que roba no juega hasta que le lleguen todas.
        let robadas = if c.valor == Valor::Mas2 { 2 } else { 0 };
        self.pasar_turno(saltar);
        self.t_turno = -(robadas as f32) * ROBO_PASO;
    }

    fn tras_la_jugada(&mut self) {
        self.quizas_visita();
        if self.jugadas >= self.proxima_rafaga {
            self.rafaga();
            self.proxima_rafaga = self.jugadas + 6 + (self.azar() * 4.0) as u32;
        }
    }

    /// Si un bot reta el +4 que le tiraron (Canela no, porque le da pena; el Slayer, seguido).
    fn bot_reta(&mut self, j: usize) -> bool {
        let ganas = match j {
            _ if self.es_canela(j) => 0.0,
            SLAYER => 0.4,
            CHIEF => 0.2,
            _ => 0.25,
        };
        self.azar() < ganas
    }

    /// El final del +4: se acepta, o se reta y alguien paga.
    fn resolver_reto(&mut self, r: Reto, retar: bool) {
        self.reto = None;
        self.turno = r.de;
        let robadas = if !retar {
            self.robar(r.a, 4);
            self.comentar_ataque(r.de, r.a, 4);
            self.anunciar(format!("+4 PARA {}", self.nombre(r.a)), Color::new(255, 110, 130, 255));
            self.pasar_turno(1);
            4
        } else if r.ilegal {
            self.robar(r.de, 4);
            self.eventos.push(Efecto::Perfecto);
            self.decir(NEPTUNO, format!("¡RETO GANADO! {} tenía {} en la mano. Roba 4 por tramposo.", nombre_corto(r.de), r.color_anterior.nombre()), Voz::Neptuno);
            self.anunciar(format!("¡{} ROBA 4!", self.nombre(r.de)), Color::new(120, 235, 160, 255));
            self.pasar_turno(0);
            4
        } else {
            self.robar(r.a, 6);
            self.eventos.push(Efecto::Fallo);
            self.decir(NEPTUNO, format!("Reto perdido: {} no tenía {}. {} se lleva seis.", nombre_corto(r.de), r.color_anterior.nombre(), nombre_corto(r.a)), Voz::Neptuno);
            self.anunciar(format!("+6 PARA {}", self.nombre(r.a)), Color::new(255, 110, 130, 255));
            self.pasar_turno(1);
            6
        };
        self.t_turno = -(robadas as f32) * ROBO_PASO;
        self.tras_la_jugada();
    }

    fn comentar_ataque(&mut self, j: usize, victima: usize, n: usize) {
        let (quien, que, voz): (&'static str, String, Voz) = match (j, n) {
            (SLAYER, _) if self.canela_en_mesa => ("CANELA", "¡Perdón, perdón! ¡Era la única carta que tenía! ¡No se enoje!".into(), Voz::Animalito),
            (SLAYER, _) => ("DOOM SLAYER", "*truena los nudillos*".into(), Voz::Grunido),
            (CHIEF, 4) => ("MASTER CHIEF", format!("+4 para {}. Lo siento. Es la guerra.", nombre_corto(victima)), Voz::Radio),
            (CHIEF, _) => ("MASTER CHIEF", "+2. Granada.".into(), Voz::Radio),
            (CLOUD, _) => ("CLOUD", "No me interesa tu sufrimiento.".into(), Voz::Planeta),
            (_, _) if victima == SLAYER => (NEPTUNO, format!("¡Miku le tiró un +{n} al Doom Slayer! Valiente. O inconsciente.") , Voz::Neptuno),
            _ => (NEPTUNO, format!("+{n} para {}. Así se juega en mi planeta.", nombre_corto(victima)), Voz::Neptuno),
        };
        self.decir(quien, que, voz);
    }

    /// Alguien se quedo con una carta: hay que gritar.
    fn queda_una(&mut self, j: usize) {
        if j == MIKU {
            if self.grito || self.miku_automatica {
                self.gritar(MIKU);
            } else {
                self.pendiente = Some(PARA_GRITAR);
            }
        } else if j == CLOUD && self.azar() < 0.35 {
            self.robar(CLOUD, 2);
            self.decir(NEPTUNO, "Cloud no gritó. Dos cartas. Estaba mirando al horizonte, pensativo.", Voz::Neptuno);
        } else {
            self.gritar(j);
        }
    }

    fn gritar(&mut self, j: usize) {
        self.desde_grito = 0.0;
        self.vida.grito("¡NEPTUNO!", Tono::Cian, if j == MIKU { Tam::Enorme } else { Tam::Grande });
        self.vida.golpe(if j == MIKU { 0.55 } else { 0.3 }, Tono::Cian);
        self.eventos.push(Efecto::Perfecto);
        let (que, voz) = match j {
            MIKU => ("¡NEPTUNO!", Voz::Miku),
            SLAYER if self.canela_en_mesa => ("¡NEPTUNO! ...¿Así se dice? ¡Qué emoción!", Voz::Animalito),
            SLAYER => ("...¡NEPTUNO!", Voz::Grunido),
            CHIEF => ("¡NEPTUNO! ...Así se dice, ¿verdad?", Voz::Radio),
            _ => ("Neptuno. ...No me interesa, pero neptuno.", Voz::Planeta),
        };
        self.decir(self.nombre(j), que, voz);
    }

    /// La rafaga: sacude la mesa y unas cartas dan la vuelta sobre el mazo, pero nadie pierde
    /// ninguna (si no, un +4 parecia dar tres).
    fn rafaga(&mut self) {
        for k in 0..5 {
            let hasta = MAZO + Vec3::new(0.3 * k as f32 - 0.6, 0.5, 0.2);
            self.vuelos.push(Vuelo { carta: None, desde: MAZO, hasta, t: -(k as f32) * 0.08, dura: 0.9, vueltas: 4.0, a_la_pila: false });
        }
        self.desde_rafaga = 0.0;
        self.eventos.push(Efecto::Salto);
        self.decir(NEPTUNO, "¡RÁFAGA DE 2,000 KM/H! Agarren sus cartas. Yo no fui. Bueno, sí fui.", Voz::Neptuno);
    }

    fn terminar(&mut self, j: usize) {
        self.ganador = Some(j);
        self.victorias[j] += 1;
        self.pendiente = None;
        self.fase = Fase::Fin;
        self.t = 0.0;
        if j == MIKU {
            self.vida.exito("¡GANÓ MIKU!", "Tres héroes en la lona", Tono::Oro);
        } else {
            let texto = format!("¡GANÓ {}!", self.nombre(j).to_uppercase());
            self.vida.desastre(&texto, "Hay revancha");
        }
        self.eventos.push(if j == MIKU { Efecto::Fuego } else { Efecto::Fallo });
        let (quien, que, voz) = match j {
            MIKU => (NEPTUNO, "¡GANÓ MIKU! Le ganó a tres héroes de videojuego con cartas de colores. Esto va directo a mi noticiero.", Voz::Neptuno),
            SLAYER if self.canela_en_mesa => (NEPTUNO, "¡Ganó Canela! Con las cartas del Slayer. Él no lo va a creer. Bueno, él no dice nada nunca.", Voz::Neptuno),
            SLAYER => (NEPTUNO, "Ganó el Doom Slayer. No dijo nada. Solo sopló el cañón de la escopeta. Me dio miedo.", Voz::Neptuno),
            CHIEF => ("MASTER CHIEF", "Partida terminada. Pediría una celebración, pero no me quito el casco.", Voz::Radio),
            _ => ("CLOUD", "Gané. ...No me interesa. Bueno, un poquito sí.", Voz::Planeta),
        };
        self.decir(quien, que, voz);
    }

    /// Un bot (o Miku en automatico) decide que jugar.
    fn decidir(&mut self, j: usize) {
        let arriba = self.arriba();
        if self.color_libre {
            self.color_libre = false;
            self.color = self.color_favorito(j);
            self.decir(NEPTUNO, format!("Salió un Comodín al repartir. {} pide {}.", nombre_corto(j), self.color.nombre()), Voz::Neptuno);
        }
        let color = self.color;
        let tiene_color = self.manos[j].iter().any(|k| k.tinte == Some(color));
        // El +4 con cartas del color es trampa: el Chief nunca, Canela tampoco; los otros, a veces.
        let farolea = tiene_color && !self.es_canela(j) && j != CHIEF && self.azar() < 0.12;
        let jugables: Vec<usize> = (0..self.manos[j].len())
            .filter(|&i| se_puede(self.manos[j][i], arriba, color))
            .filter(|&i| self.manos[j][i].valor != Valor::Mas4 || !tiene_color || farolea)
            .collect();
        if jugables.is_empty() {
            // Sin jugada: roba una por una hasta que salga algo.
            let antes = self.manos[j].len();
            self.robar(j, 1);
            if self.manos[j].len() == antes {
                // No queda ni una carta en el mazo ni en la pila: se pasa.
                self.pasar_turno(0);
            } else {
                self.t_turno = PIENSA - 0.32;
            }
            return;
        }
        let peligro = self.manos[self.siguiente(j, 0)].len() <= 2;
        let mut mejor = (f32::MIN, jugables[0]);
        for &i in &jugables {
            let mut puntos = match self.manos[j][i].valor {
                Valor::Numero(n) => 1.0 + n as f32 * 0.02,
                Valor::Salta | Valor::Reversa => if peligro || j == SLAYER { 3.0 } else { 1.5 },
                Valor::Mas2 => if peligro || j == SLAYER { 4.0 } else { 1.2 },
                Valor::Comodin => 0.5,
                Valor::Mas4 => if peligro { 5.0 } else { 0.3 },
            };
            if j == CLOUD {
                puntos += self.azar() * 2.0;
            }
            // Canela juega bonito: casi nunca tira cartas que hagan robar.
            if self.es_canela(j) && matches!(self.manos[j][i].valor, Valor::Mas2 | Valor::Mas4) {
                puntos = 0.05;
            }
            if puntos > mejor.0 {
                mejor = (puntos, i);
            }
        }
        let elegido = self.color_favorito(j);
        self.jugar(j, mejor.1, Some(elegido));
    }

    /// El color que mas tiene en la mano (para los comodines de los bots).
    fn color_favorito(&mut self, j: usize) -> Tinte {
        let mut cuenta = [0; 4];
        for c in &self.manos[j] {
            if let Some(t) = c.tinte {
                cuenta[t as usize] += 1;
            }
        }
        let max = *cuenta.iter().max().unwrap();
        if max == 0 {
            return TINTES[((self.azar() * 4.0) as usize).min(3)];
        }
        TINTES[cuenta.iter().position(|&c| c == max).unwrap()]
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionNeptuno {
        self.t += dt;
        self.total += dt;
        self.t_turno += dt;
        self.desde_grito += dt;
        self.desde_rafaga += dt;
        self.desde_visita += dt;
        let mut salieron = 0;
        for v in &mut self.vuelos {
            if v.t < 0.0 && v.t + dt >= 0.0 {
                salieron += 1;
            }
            v.t += dt;
        }
        for _ in 0..salieron.min(1) {
            self.eventos.push(Efecto::Carta);
        }
        self.vuelos.retain(|v| v.t < v.dura);
        // Si ya no le queda una sola (le tiraron un +2), ya no hay nada que gritar.
        if self.manos[MIKU].len() != 1 {
            self.pendiente = None;
        }
        if e.aterrizar {
            return AccionNeptuno::Despegar;
        }
        match self.fase {
            Fase::Intro(i) => {
                let largo = INTRO[i].1.chars().count() as f32;
                if (e.aceptar && self.t > 0.4) || self.t > 2.5 + largo / 22.0 {
                    self.t = 0.0;
                    if i + 1 < INTRO.len() {
                        self.fase = Fase::Intro(i + 1);
                    } else {
                        self.fase = Fase::Juego;
                        self.t_turno = 0.0;
                        self.decir("MIKU", "¡Yo voy a ganar! Tengo 39 razones. Mi-ku, tres-nueve. ¿Entienden? ...¿No?", Voz::Miku);
                        // Lo que hizo la carta volteada se dijo durante la intro: se repite ahora.
                        match self.arriba().valor {
                            Valor::Salta => self.anunciar("SALTA A MIKU", Color::new(255, 220, 120, 255)),
                            Valor::Reversa => self.anunciar("EMPIEZA CLOUD", Color::new(255, 220, 120, 255)),
                            Valor::Mas2 => self.anunciar("+2 PARA MIKU", Color::new(255, 110, 130, 255)),
                            Valor::Comodin => self.anunciar("COMODÍN: ESCOJA COLOR", Color::new(190, 140, 255, 255)),
                            _ => {}
                        }
                    }
                }
            }
            Fase::Juego if self.visita.is_some() => {
                // Mientras alguien interrumpe, la partida espera.
                self.avanzar_visita(e, dt);
            }
            Fase::Juego => {
                // El tiempo para gritar corre aunque ya no sea su turno.
                if let Some(quedan) = &mut self.pendiente {
                    *quedan -= dt;
                    if e.barrel {
                        self.pendiente = None;
                        self.gritar(MIKU);
                    } else if *quedan <= 0.0 {
                        self.pendiente = None;
                        self.robar(MIKU, 2);
                        self.eventos.push(Efecto::Fallo);
                        self.decir(NEPTUNO, "¡No gritó NEPTUNO! Dos cartas. Las reglas son las reglas. Las inventé yo hace cinco minutos.", Voz::Neptuno);
                    }
                }
                if let Some(r) = self.reto {
                    self.responder_reto(r, e);
                } else if self.turno == MIKU && !self.miku_automatica {
                    self.turno_miku(e);
                } else if self.t_turno > PIENSA {
                    let j = self.turno;
                    self.decidir(j);
                }
            }
            Fase::Fin => {
                if e.aceptar && self.t > 0.8 {
                    self.repartir();
                    self.fase = Fase::Juego;
                    self.decir(NEPTUNO, "¡Otra partida! Barajo yo. No hago trampa. Casi nunca.", Voz::Neptuno);
                }
            }
        }
        AccionNeptuno::Nada
    }

    /// Le tiraron un +4 a Miku: aceptar (roba 4) o retar.
    fn responder_reto(&mut self, r: Reto, e: &Entrada) {
        if e.izquierda || e.derecha {
            self.cursor_reto = 1 - self.cursor_reto;
            self.eventos.push(Efecto::Carta);
        }
        if e.aceptar && self.t_turno > 0.3 {
            self.resolver_reto(r, self.cursor_reto == 1);
        }
    }

    fn turno_miku(&mut self, e: &Entrada) {
        let n = self.manos[MIKU].len();
        // El Comodin que se volteo al repartir: Miku escoge el color antes de jugar.
        if self.color_libre {
            if e.izquierda {
                self.cursor_color = (self.cursor_color + 3) % 4;
            }
            if e.derecha {
                self.cursor_color = (self.cursor_color + 1) % 4;
            }
            let elegido = e.respuesta.filter(|&r| r < 4).or(if e.aceptar { Some(self.cursor_color) } else { None });
            if let Some(k) = elegido {
                self.color = TINTES[k];
                self.color_libre = false;
                self.t_turno = 0.0;
            }
            return;
        }
        // Robando sola hasta que salga algo: una carta cada tantito.
        if self.robando {
            if self.t_turno > 0.32 {
                self.t_turno = 0.0;
                self.robar_una_miku();
            }
            return;
        }
        if let Some(i) = self.eligiendo {
            if e.izquierda {
                self.cursor_color = (self.cursor_color + 3) % 4;
            }
            if e.derecha {
                self.cursor_color = (self.cursor_color + 1) % 4;
            }
            let elegido = e.respuesta.filter(|&r| r < 4).or(if e.aceptar { Some(self.cursor_color) } else { None });
            if let Some(k) = elegido {
                self.eligiendo = None;
                self.jugar(MIKU, i, Some(TINTES[k]));
            }
            return;
        }
        if e.izquierda && n > 0 {
            self.cursor = (self.cursor + n - 1) % n;
        }
        if e.derecha && n > 0 {
            self.cursor = (self.cursor + 1) % n;
        }
        if e.barrel && self.pendiente.is_none() {
            if n <= 2 {
                self.grito = true;
                self.desde_grito = 0.0;
                self.decir("MIKU", "¡NEPTUNO! (por si acaso)", Voz::Miku);
            } else {
                self.decir(NEPTUNO, "Todavía no, Miku. Cuente sus cartas. Son más de una. Bastantes más.", Voz::Neptuno);
            }
        }
        if e.aceptar && self.t_turno > 0.2 && n > 0 {
            let c = self.manos[MIKU][self.cursor];
            if self.robo.is_some_and(|r| r != c) {
                self.eventos.push(Efecto::Fallo);
                self.decir(NEPTUNO, "Después de robar solo se puede jugar la que robó. O pasar, abajo.", Voz::Neptuno);
            } else if se_puede(c, self.arriba(), self.color) {
                if c.comodin() {
                    self.eligiendo = Some(self.cursor);
                    self.cursor_color = 0;
                } else {
                    self.jugar(MIKU, self.cursor, None);
                }
            } else {
                self.eventos.push(Efecto::Fallo);
                self.decir(NEPTUNO, "Esa no se puede, Miku. Color con color, número con número. O un comodín, que para eso están.", Voz::Neptuno);
            }
        }
        // Robar es a proposito: con la cruceta, no con un rozon del stick.
        if e.abajo && !e.por_stick && self.t_turno > 0.2 {
            if self.robo.is_some() {
                self.pasar_turno(0);
            } else if self.manos[MIKU].iter().any(|&c| se_puede(c, self.arriba(), self.color)) {
                // Tiene jugada y quiere robar igual: es valido, pero roba una sola.
                self.robar_una_miku();
            } else {
                self.robando = true;
                self.t_turno = 0.0;
                self.robar_una_miku();
            }
        }
    }

    /// Miku roba una y el cursor se va a esa. Si se puede jugar, deja de robar y escoge.
    fn robar_una_miku(&mut self) {
        let antes = self.manos[MIKU].clone();
        self.robar(MIKU, 1);
        if self.manos[MIKU].len() == antes.len() {
            // No queda de donde robar.
            self.pasar_turno(0);
            return;
        }
        let nueva = self.manos[MIKU].iter().enumerate().position(|(k, c)| antes.get(k) != Some(c)).unwrap_or(0);
        self.cursor = nueva;
        let c = self.manos[MIKU][nueva];
        if se_puede(c, self.arriba(), self.color) {
            self.robando = false;
            self.robo = Some(c);
            self.t_turno = 0.0;
        } else if !self.robando {
            // Tenia jugada, robo una que no sirve: se le acaba el turno.
            self.pasar_turno(0);
        }
    }

    /// Para las capturas: `neptuno_reto` deja a Miku respondiendo un +4 de Cloud.
    pub fn preparar_reto(&mut self) {
        self.fase = Fase::Juego;
        self.visita = None;
        self.turno = CLOUD;
        self.manos[CLOUD].insert(0, Carta { tinte: None, valor: Valor::Mas4 });
        self.jugar(CLOUD, 0, Some(Tinte::Azul));
    }

    /// Si en esta partida ya vinieron Canela y Pluton.
    pub fn visitas(&self) -> (bool, bool) {
        (self.vino_canela, self.vino_pluton)
    }

    pub fn hablante(&self) -> Option<(&str, &str, Voz)> {
        match self.fase {
            Fase::Intro(i) => Some(INTRO[i]),
            _ => match self.visita {
                Some((v, i, _)) => Some(v.guion()[i]),
                None => self.comentario.as_ref().map(|(q, t, v)| (*q, t.as_str(), *v)),
            },
        }
    }

    pub fn expresion_neptuno(&self) -> &'static str {
        match self.fase {
            Fase::Intro(_) => "feliz",
            Fase::Fin if self.ganador == Some(MIKU) => "enamorada",
            Fase::Fin => "guino",
            _ if self.desde_rafaga < 2.0 => "guino",
            _ if self.pendiente.is_some() => "asustada",
            _ if self.manos.iter().any(|m| m.len() == 1) => "asustada",
            _ => "feliz",
        }
    }

    pub fn expresion_miku(&self) -> &'static str {
        match self.fase {
            Fase::Fin if self.ganador == Some(MIKU) => "enamorada",
            Fase::Fin => "espiral",
            _ if self.pendiente.is_some() => "asustada",
            _ if self.manos[MIKU].len() > 12 => "espiral",
            _ if self.turno == MIKU => "guino",
            _ => "feliz",
        }
    }

    /// La cara de un rival, si tiene (solo Cloud: los otros dos traen casco).
    pub fn expresion_rival(&self, j: usize) -> Option<&'static str> {
        (j == CLOUD).then(|| match self.fase {
            Fase::Fin if self.ganador == Some(CLOUD) => "guino",
            Fase::Fin => "dormida",
            _ if self.desde_rafaga < 2.0 => "espiral",
            _ if self.pendiente.is_some() => "guino",
            _ => "serio",
        })
    }

    pub fn miku_baila(&self) -> bool {
        self.fase == Fase::Fin && self.ganador == Some(MIKU)
    }

    // ---------- donde va todo en la escena ----------

    pub fn camara(&self) -> (Vec3, Vec3) {
        let sacudida = if self.desde_rafaga < 1.0 { 0.05 * (1.0 - self.desde_rafaga) * (self.total * 45.0).sin() } else { 0.0 };
        (Vec3::new(sacudida, 2.45, -3.7), Vec3::new(0.0, 0.75, 0.55))
    }

    /// La base de la camara: derecha, arriba y adelante.
    fn ejes(&self) -> (Vec3, Vec3, Vec3) {
        let (ojo, mira) = self.camara();
        let f = (mira - ojo).normalize();
        let derecha = f.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
        (derecha, derecha.cross(f), f)
    }

    /// Donde esta la mano de cada jugador (la de Miku, enfrente de la camara).
    fn lugar_mano(&self, j: usize) -> Vec3 {
        if j == MIKU {
            let (ojo, _) = self.camara();
            let (_, arriba, f) = self.ejes();
            return ojo + f * 1.45 - arriba * 0.5;
        }
        let a = ASIENTOS[j];
        let al_centro = Vec3::new(-a.x, 0.0, -a.z).normalize();
        a + al_centro * 0.75 + Vec3::new(0.0, 0.86, 0.0)
    }

    /// Una carta parada mirando a la camara: de frente o de espaldas.
    fn de_frente(&self, pos: Vec3, escala: f32, frente: bool, giro: f32) -> Mat4 {
        let (derecha, arriba, f) = self.ejes();
        let base = if frente { Mat4::from_basis(derecha * -1.0, arriba, f) } else { Mat4::from_basis(derecha, arriba, f * -1.0) };
        Mat4::translation(pos) * base * Mat4::rotation_z(giro) * Mat4::scale(escala)
    }

    /// Todas las cartas a dibujar: (matriz, partes a ocultar).
    pub fn cartas(&self) -> Vec<(Mat4, Vec<&'static str>)> {
        let mut v: Vec<(Mat4, Vec<&'static str>)> = Vec::new();
        let abajo = || BOCA_ABAJO.to_vec();
        // El mazo: unas cuantas boca abajo, apiladas.
        for k in 0..4.min(self.mazo.len()) {
            let m = Mat4::translation(MAZO + Vec3::new(0.0, k as f32 * 0.012, 0.0)) * Mat4::rotation_y(0.15) * Mat4::rotation_x(-PI / 2.0) * Mat4::scale(0.95);
            v.push((m, abajo()));
        }
        // La pila: las de abajo acostadas, cada una un poco girada; la de arriba
        // (la carta en juego) flota grande sobre ellas, mirando a la camara.
        let volando = self.vuelos.iter().any(|w| w.a_la_pila);
        let hasta = self.pila.len() - volando as usize;
        if hasta > 0 {
            let acostadas = hasta - 1;
            for (k, c) in self.pila[..acostadas].iter().enumerate().skip(acostadas.saturating_sub(4)) {
                let giro = ((k * 2654435761) % 1000) as f32 / 1000.0 - 0.5;
                let m = Mat4::translation(PILA + Vec3::new(0.0, k as f32 * 0.002 % 0.01, 0.0)) * Mat4::rotation_y(giro * 0.8) * Mat4::rotation_x(PI / 2.0) * Mat4::scale(0.95);
                v.push((m, c.ocultar()));
            }
            let arriba = if volando { self.pila.get(hasta.wrapping_sub(1)) } else { self.pila.last() };
            if let Some(c) = arriba {
                if volando {
                    // Mientras vuela la nueva, la anterior queda acostada en la pila.
                    let m = Mat4::translation(PILA + Vec3::new(0.0, 0.012, 0.0)) * Mat4::rotation_x(PI / 2.0) * Mat4::scale(0.95);
                    v.push((m, c.ocultar()));
                } else {
                    v.push((self.carta_en_juego(), c.ocultar()));
                }
            }
        }
        // Las manos de los bots, boca abajo y en abanico.
        let (derecha, arriba, _) = self.ejes();
        for j in 1..JUGADORES {
            let n = self.manos[j].len();
            let paso = (1.0 / n.max(1) as f32).min(0.11);
            let centro = self.lugar_mano(j) + Vec3::new(0.0, self.brinco(j), 0.0);
            for k in 0..n {
                let u = k as f32 - (n as f32 - 1.0) * 0.5;
                let pos = centro + derecha * (u * paso) + arriba * (-(u * paso).powi(2) * 0.6) - (centro - self.camara().0).normalize() * (k as f32 * 0.002);
                v.push((self.de_frente(pos, 0.42, false, -u * paso * 0.9), abajo()));
            }
        }
        // La mano de Miku, de frente, en abanico; la elegida sube.
        let n = self.manos[MIKU].len();
        let paso = (2.6 / n.max(1) as f32).min(0.27);
        let centro = self.lugar_mano(MIKU);
        let su_turno = self.turno == MIKU && self.fase == Fase::Juego;
        for (k, c) in self.manos[MIKU].iter().enumerate() {
            let u = k as f32 - (n as f32 - 1.0) * 0.5;
            let elegida = su_turno && k == self.cursor;
            let jugable = su_turno && self.robo.is_none_or(|r| r == *c) && se_puede(*c, self.arriba(), self.color);
            let alto = if elegida { 0.13 } else if jugable { 0.04 } else { 0.0 };
            let pos = centro + derecha * (u * paso) + arriba * (alto - (u * paso).powi(2) * 0.12) - (centro - self.camara().0).normalize() * (k as f32 * 0.004);
            v.push((self.de_frente(pos, 0.5, true, -u * paso * 0.25), c.ocultar()));
        }
        // Las que van volando.
        for w in self.vuelos.iter().filter(|w| w.t >= 0.0) {
            let k = w.t / w.dura;
            let s = k * k * (3.0 - 2.0 * k);
            let pos = w.desde.lerp(w.hasta, s) + Vec3::new(0.0, (k * PI).sin() * 0.4, 0.0);
            let m = match w.carta {
                // Llega justo a la pose de la carta en juego: girando cada vez menos y creciendo.
                Some(c) => (
                    Mat4::translation(pos) * Mat4::rotation_y((1.0 - s) * 2.5) * Mat4::rotation_x(INCLINACION_EN_JUEGO * s) * Mat4::scale(0.7 + (ESCALA_EN_JUEGO - 0.7) * s),
                    c.ocultar(),
                ),
                None => (self.de_frente(pos, 0.5, false, k * w.vueltas * PI * 2.0), abajo()),
            };
            v.push(m);
        }
        v
    }

    /// La carta en juego: flotando sobre la pila, inclinada hacia la camara y meciendose.
    fn carta_en_juego(&self) -> Mat4 {
        let flota = Vec3::new(0.0, 0.03 * (self.total * 2.0).sin(), 0.0);
        Mat4::translation(EN_JUEGO + flota) * Mat4::rotation_z(0.05 * (self.total * 1.3).sin()) * Mat4::rotation_x(INCLINACION_EN_JUEGO) * Mat4::scale(ESCALA_EN_JUEGO)
    }

    /// Un brinquito: el que tiene el turno, y el que gano (mucho).
    fn brinco(&self, j: usize) -> f32 {
        if self.ganador == Some(j) {
            0.25 * (self.total * 7.0).sin().abs()
        } else if self.turno == j && self.fase == Fase::Juego {
            0.05 * (self.total * 6.0).sin().abs()
        } else {
            0.0
        }
    }

    /// Alguien parado en `pos` mirando hacia `objetivo`.
    fn parado(pos: Vec3, objetivo: Vec3, escala: f32) -> Mat4 {
        let d = objetivo - pos;
        Mat4::translation(pos) * Mat4::rotation_y((-d.x).atan2(-d.z)) * Mat4::scale(escala)
    }

    /// Todo lo que se para alrededor de la mesa: los rivales (el Slayer puede
    /// estar afuera) y las visitas, caminando segun su guion. Con su expresion, si tienen cara.
    pub fn figuras(&self) -> Vec<(Figura, Mat4, Option<&'static str>)> {
        let mut v = Vec::new();
        let centro = Vec3::new(0.0, 0.0, 0.0);
        let camina = |desde: Vec3, hasta: Vec3, k: f32| desde.lerp(hasta, suave(k)) + Vec3::new(0.0, 0.05 * (self.total * 12.0).sin().abs() * (k > 0.0 && k < 1.0) as i32 as f32, 0.0);
        let asiento = ASIENTOS[SLAYER] + Vec3::new(0.0, self.brinco(SLAYER), 0.0);
        // El Slayer y Canela, segun la visita.
        let (slayer, canela): (Option<Vec3>, Option<Vec3>) = match (self.visita, self.ultima_visita) {
            (Some((Visita::LlegaCanela, i, t)), _) => {
                let llega = if i == 0 { (t / 1.5).min(1.0) } else { 1.0 };
                let canela = if i >= 5 { camina(JUNTO_AL_SLAYER, ASIENTOS[SLAYER], t / 1.2) } else { camina(PUERTA_IZQUIERDA, JUNTO_AL_SLAYER, llega) };
                let slayer = if i >= 4 { camina(ASIENTOS[SLAYER], PUERTA_IZQUIERDA, t / 2.0) } else { asiento };
                (Some(slayer), Some(canela))
            }
            (Some((Visita::RegresaSlayer, i, t)), _) => {
                let slayer = if i == 0 { camina(PUERTA_IZQUIERDA, JUNTO_AL_SLAYER, t / 1.8) } else { camina(JUNTO_AL_SLAYER, ASIENTOS[SLAYER], (t / 1.0).min(1.0)) };
                let canela = if i >= 1 { camina(ASIENTOS[SLAYER], PUERTA_IZQUIERDA, t / 2.2) } else { asiento };
                (Some(slayer), Some(canela))
            }
            (None, Some(Visita::RegresaSlayer)) if self.desde_visita < 2.5 => (Some(asiento), Some(camina(ASIENTOS[SLAYER], PUERTA_IZQUIERDA, 1.0))),
            _ if self.canela_en_mesa => (None, Some(asiento)),
            _ => (Some(asiento), None),
        };
        if let Some(p) = slayer {
            v.push((Figura::Rival(SLAYER), Self::parado(p, if p == asiento { centro } else { JUNTO_AL_SLAYER.lerp(centro, 0.5) }, 1.05), None));
        }
        if let Some(p) = canela {
            let cara = match self.visita {
                Some((Visita::LlegaCanela, 1, _)) => "feliz",
                Some((Visita::LlegaCanela, 3, _)) => "asustada",
                Some((Visita::LlegaCanela, 4, _)) => "enamorada",
                Some((Visita::RegresaSlayer, _, _)) => "guino",
                _ if self.ganador == Some(SLAYER) => "enamorada",
                _ if self.turno == SLAYER => "guino",
                _ => "feliz",
            };
            let mira = if self.canela_en_mesa && self.visita.is_none() { centro } else { ASIENTOS[SLAYER] };
            v.push((Figura::Canela, Self::parado(p, if (p - mira).length() < 0.1 { centro } else { mira }, 1.0), Some(cara)));
        }
        for j in [CHIEF, CLOUD] {
            let a = ASIENTOS[j] + Vec3::new(0.0, self.brinco(j), 0.0);
            v.push((Figura::Rival(j), Self::parado(a, centro, 1.05), self.expresion_rival(j)));
        }
        // Pluton llega rodando por la derecha, pide jugar y se va.
        if let Some((Visita::Pluton, i, t)) = self.visita {
            let ultima = Visita::Pluton.guion().len() - 1;
            let p = if i == 0 { camina(PUERTA_DERECHA, LUGAR_PLUTON, t / 1.5) } else if i >= ultima { camina(LUGAR_PLUTON, PUERTA_DERECHA, t / 2.5) } else { LUGAR_PLUTON };
            let cara = match i {
                1 => "feliz",
                2 => "asustada",
                3 => "espiral",
                _ => "serio",
            };
            let rueda = Mat4::rotation_z(-p.x * 1.2);
            v.push((Figura::Pluton, Mat4::translation(p) * Mat4::rotation_y(0.6) * rueda * Mat4::scale(0.55), Some(cara)));
        }
        v
    }

    pub fn miku(&self) -> Mat4 {
        let a = ASIENTOS[MIKU] + Vec3::new(0.0, self.brinco(MIKU), 0.0);
        // Mira hacia la mesa: de espaldas a la camara (se le ven las coletas).
        Mat4::translation(a) * Mat4::rotation_y(a.x.atan2(a.z)) * Mat4::scale(0.8)
    }

    /// Neptuno, el anfitrion, flotando gigante detras de la mesa.
    pub fn neptuno(&self) -> Mat4 {
        let rafaga = if self.desde_rafaga < 1.5 { 0.15 * (self.desde_rafaga * 20.0).sin() * (1.5 - self.desde_rafaga) } else { 0.0 };
        Mat4::translation(Vec3::new(0.0, 2.9 + 0.15 * (self.total * 0.8).sin(), 9.0)) * Mat4::rotation_z(0.05 * (self.total * 0.6).sin() + rafaga) * Mat4::scale(3.2)
    }

    /// La cabeza de quien tiene el turno (para el reflector).
    pub fn foco(&self) -> Option<Vec3> {
        (self.fase == Fase::Juego && self.turno != MIKU).then(|| ASIENTOS[self.turno] + Vec3::new(0.0, 1.1, 0.0))
    }

    /// Que tan fuerte sopla (1 normal, mas justo despues de una rafaga).
    pub fn viento(&self) -> f32 {
        1.0 + 3.0 * (1.0 - self.desde_rafaga / 1.5).max(0.0)
    }

    /// Donde va el nombre de cada rival (arriba de su cabeza), para la interfaz.
    pub fn cabezas(&self) -> [Vec3; JUGADORES] {
        let mut v = [Vec3::zero(); JUGADORES];
        for j in 0..JUGADORES {
            v[j] = ASIENTOS[j] + Vec3::new(0.0, 1.65, 0.0);
        }
        v
    }

    /// `cabezas`: los nombres en pantalla (de `cabezas()` ya proyectadas).
    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, cabezas: &[Option<(f32, f32)>], ancho: i32, alto: i32) {
        let (aceptar, gritar) = (&ui::boton(e.nombre("aceptar")), &ui::boton(e.nombre("barrel")));
        let centrado = |d: &mut RaylibDrawHandle, s: &str, y: i32, tam: i32, color: Color| ui::centrado(d, s, ancho, y, tam, color);
        // Quien habla, arriba.
        if let (Some((nombre, _, _)), false) = (self.hablante(), texto.is_empty()) {
            let w = 860.min(ancho - 40);
            let lineas = ui::envolver(texto, 22, w - 48).len() as i32;
            let h = 54 + lineas * 28;
            let x = (ancho - w) / 2;
            ui::panel(d, x, 14, w, h);
            ui::texto(d, nombre, x + 24, 24, 18, Color::new(120, 190, 255, 255));
            ui::parrafo(d, texto, x + 24, 48, 22, w - 48, ui::TEXTO);
        }
        if matches!(self.fase, Fase::Intro(_)) {
            if matches!(self.fase, Fase::Intro(0) | Fase::Intro(1)) {
                centrado(d, "¡NEPTUNO!", alto / 2 - 60, 96, Color::new(110, 190, 255, 255));
            }
            return;
        }
        // Los nombres y cuantas cartas tiene cada rival, sobre su cabeza.
        for j in 1..JUGADORES {
            if let Some(Some((x, y))) = cabezas.get(j) {
                let s = format!("{}  {}", self.nombre(j), self.manos[j].len());
                let w = ui::medir(&s, 20);
                let turno = self.turno == j && self.fase == Fase::Juego;
                let color = if turno { Color::new(255, 220, 120, 255) } else { ui::TEXTO };
                d.draw_rectangle(*x as i32 - w / 2 - 8, *y as i32 - 4, w + 16, 28, Color::new(10, 15, 50, 190));
                ui::texto(d, &s, *x as i32 - w / 2, *y as i32, 20, color);
            }
        }
        // El color que manda y el sentido, a la izquierda.
        let (x, y) = (20, alto / 2 - 70);
        ui::panel(d, x, y, 230, 136);
        ui::texto(d, "COLOR", x + 16, y + 12, 16, ui::TENUE);
        d.draw_rectangle_rounded(Rectangle::new((x + 16) as f32, (y + 34) as f32, 28.0, 28.0), 0.3, 6, self.color.rgb());
        ui::texto(d, self.color.nombre(), x + 54, y + 36, 24, self.color.rgb());
        let jugo = self.ultimo.map_or("LA BARAJA".to_string(), |j| self.nombre(j).to_string());
        ui::texto(d, "JUGÓ", x + 16, y + 74, 16, ui::TENUE);
        ui::texto(d, &jugo, x + 60, y + 72, 20, ui::TEXTO);
        let sentido = if self.sentido > 0 { "SENTIDO: NORMAL" } else { "SENTIDO: AL REVÉS" };
        ui::texto(d, sentido, x + 16, y + 104, 16, ui::TENUE);
        // El marcador, a la derecha.
        let (x, y) = (ancho - 230, alto / 2 - 80);
        ui::panel(d, x, y, 210, 150);
        ui::texto(d, "VICTORIAS", x + 16, y + 12, 18, ui::TENUE);
        for j in 0..JUGADORES {
            ui::texto(d, &format!("{}: {}", NOMBRES[j], self.victorias[j]), x + 16, y + 38 + j as i32 * 26, 20, ui::TEXTO);
        }
        // El grito de ¡NEPTUNO!, grande, y el aviso de que hay que gritar.
        if self.desde_grito < 1.0 {
            let tam = 70 + (20.0 * (1.0 - self.desde_grito)) as i32;
            centrado(d, "¡NEPTUNO!", alto / 2 - 120, tam, Color::new(110, 200, 255, 255));
        }
        if let Some(quedan) = self.pendiente {
            if (quedan * 6.0) as i32 % 2 == 0 {
                centrado(d, "¡GRITA NEPTUNO!", alto / 2 - 110, 48, Color::new(255, 90, 110, 255));
                centrado(d, &format!("¡apriete {gritar}!"), alto / 2 - 52, 24, Color::new(255, 200, 210, 255));
            }
        }
        if self.desde_rafaga < 1.6 {
            centrado(d, "¡RÁFAGA!", alto / 2 - 40, 64, Color::new(200, 235, 255, 255));
        }
        // El mazo y la mano de Miku, para contar.
        let cuenta = format!("MAZO {}   ·   TU MANO {}", self.mazo.len(), self.manos[MIKU].len());
        ui::subtitulo(d, &cuenta, 20, alto / 2 + 92, 16, ui::TENUE);
        // Le tiraron un +4: aceptar o retar.
        if let (Some(r), Fase::Juego, None) = (self.reto, self.fase, self.visita) {
            let (w, h) = (640.min(ancho - 40), 190);
            let (x, y) = ((ancho - w) / 2, alto / 2 - 110);
            ui::panel(d, x, y, w, h);
            let titulo = format!("+4 DE {}", self.nombre(r.de));
            ui::texto(d, &titulo, x + 24, y + 14, 26, Color::new(255, 130, 150, 255));
            let opciones = [("ACEPTAR", "Robas 4 y pierdes el turno.".to_string()), ("RETAR", format!("Si tenía {}, roba 4 él. Si no, tú robas 6.", r.color_anterior.nombre()))];
            for (k, (n, ayuda)) in opciones.iter().enumerate() {
                let cx = x + 24 + k as i32 * (w - 48) / 2;
                let cw = (w - 48) / 2 - 10;
                let fondo = if k == self.cursor_reto { Color::new(190, 90, 255, 255) } else { Color::new(50, 36, 90, 255) };
                d.draw_rectangle_rounded(Rectangle::new(cx as f32, (y + 54) as f32, cw as f32, 118.0), 0.12, 6, fondo);
                ui::texto(d, n, cx + 14, y + 64, 24, Color::WHITE);
                ui::parrafo(d, ayuda, cx + 14, y + 98, 18, cw - 28, ui::TEXTO);
            }
        }
        // El turno de Miku: ayuda abajo, y el selector de color si jugo un comodin.
        if self.fase == Fase::Juego && self.turno == MIKU && !self.miku_automatica && self.visita.is_none() {
            if self.eligiendo.is_some() || self.color_libre {
                let (w, h) = (520, 130);
                let (x, y) = ((ancho - w) / 2, alto / 2 - 100);
                ui::panel(d, x, y, w, h);
                ui::texto(d, "ELIGE EL COLOR", x + 24, y + 14, 22, ui::LILA);
                for (k, t) in TINTES.iter().enumerate() {
                    let cx = x + 30 + k as i32 * 120;
                    if k == self.cursor_color {
                        d.draw_rectangle(cx - 6, y + 48, 102, 64, Color::WHITE);
                    }
                    d.draw_rectangle(cx, y + 54, 90, 52, t.rgb());
                    ui::texto(d, &format!("{}", k + 1), cx + 38, y + 66, 26, Color::WHITE);
                }
            } else {
                let llamado = if self.robando { "ROBANDO HASTA QUE SALGA ALGO..." } else { "TU TURNO" };
                centrado(d, llamado, alto - 330, 30, Color::new(255, 220, 120, 255));
                let ayuda = if self.robo.is_some() {
                    format!("{}: elegir   {aceptar}: jugar   {}: pasar   {gritar}: ¡NEPTUNO!", ui::boton("\u{E00A} \u{E00B}"), ui::boton("abajo"))
                } else {
                    format!("{}: elegir   {aceptar}: jugar   {}: robar   {gritar}: ¡NEPTUNO!", ui::boton("\u{E00A} \u{E00B}"), ui::boton("abajo"))
                };
                d.draw_rectangle(0, alto - 34, ancho, 34, Color::new(10, 15, 50, 200));
                centrado(d, &ayuda, alto - 29, 20, ui::TEXTO);
            }
        }
        if self.fase == Fase::Fin {
            let g = self.ganador.unwrap_or(MIKU);
            let titulo = if g == MIKU { "¡GANASTE!".to_string() } else { format!("GANÓ {}", self.nombre(g)) };
            centrado(d, &titulo, alto / 2 - 90, 72, if g == MIKU { Color::new(255, 130, 200, 255) } else { Color::new(255, 200, 120, 255) });
            centrado(d, &format!("{aceptar}: otra partida      {}: despegar", ui::boton(e.nombre("aterrizar"))), alto / 2, 24, ui::TEXTO);
        }
    }
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn nombre_corto(j: usize) -> &'static str {
    ["Miku", "el Slayer", "el Chief", "Cloud"][j]
}

impl Neptuno {
    /// El nombre de quien juega en el lugar `j` (en el del Slayer puede estar Canela).
    fn nombre(&self, j: usize) -> &'static str {
        if j == SLAYER && self.canela_en_mesa { "CANELA" } else { NOMBRES[j] }
    }

    fn es_canela(&self, j: usize) -> bool {
        j == SLAYER && self.canela_en_mesa
    }

    /// Vuelve a escoger la visita de la partida (despues de poner `ya_vinieron`).
    pub fn reescoger_visita(&mut self) {
        self.visita_de_la_partida = self.escoger_visita();
    }

    /// Quien viene en la partida que empieza: la primera vez que se visita Neptuno, los que
    /// nunca han venido (primero Canela); despues, solo a veces (una de cada tres partidas).
    fn escoger_visita(&mut self) -> Option<Visita> {
        let (canela, pluton) = self.ya_vinieron;
        let suerte = self.azar();
        if self.partidas <= 1 && !canela {
            Some(Visita::LlegaCanela)
        } else if self.partidas <= 1 && !pluton {
            Some(Visita::Pluton)
        } else if suerte < 0.17 {
            Some(Visita::LlegaCanela)
        } else if suerte < 0.34 {
            Some(Visita::Pluton)
        } else {
            None
        }
    }

    /// Empieza una visita (la partida se pausa mientras dura su guion).
    fn llega(&mut self, v: Visita) {
        self.visita = Some((v, 0, 0.0));
        self.eventos.push(if v == Visita::Pluton { Efecto::Choque } else { Efecto::Salto });
        match v {
            Visita::LlegaCanela => self.vino_canela = true,
            Visita::Pluton => self.vino_pluton = true,
            Visita::RegresaSlayer => {}
        }
    }

    /// Despues de cada jugada: ¿llega alguien?
    fn quizas_visita(&mut self) {
        if self.visita.is_some() || self.fase != Fase::Juego {
            return;
        }
        if self.canela_en_mesa {
            self.jugadas_canela += 1;
            // Se queda unas cuatro vueltas de la mesa (antes se iba casi luego de llegar y la
            // escena del regreso parecia la misma otra vez).
            if self.jugadas_canela >= 16 {
                self.llega(Visita::RegresaSlayer);
            }
            return;
        }
        if self.jugadas >= self.proxima_visita {
            self.proxima_visita = self.jugadas + 9 + (self.azar() * 6.0) as u32;
            // Una visita por partida como mucho (la que se escogio al repartir), y nunca justo
            // cuando le toca a la jugadora: si no, entra y no puede jugar.
            if self.turno == MIKU {
                self.proxima_visita = self.jugadas + 1;
                return;
            }
            match self.visita_de_la_partida.take() {
                Some(Visita::LlegaCanela) if !self.vino_canela => self.llega(Visita::LlegaCanela),
                Some(Visita::Pluton) if !self.vino_pluton => self.llega(Visita::Pluton),
                _ => {}
            }
        }
    }

    /// Avanza el guion de la visita; al terminar, aplica lo que paso.
    fn avanzar_visita(&mut self, e: &Entrada, dt: f32) {
        let Some((v, i, t)) = self.visita else { return };
        let t = t + dt;
        let largo = v.guion()[i].1.chars().count() as f32;
        if (e.aceptar && t > 0.4) || t > 2.2 + largo / 22.0 {
            if i + 1 < v.guion().len() {
                self.visita = Some((v, i + 1, 0.0));
            } else {
                self.visita = None;
                self.ultima_visita = Some(v);
                self.desde_visita = 0.0;
                self.t_turno = 0.0;
                match v {
                    Visita::LlegaCanela => {
                        self.canela_en_mesa = true;
                        self.jugadas_canela = 0;
                    }
                    Visita::RegresaSlayer => self.canela_en_mesa = false,
                    Visita::Pluton => {}
                }
            }
        } else {
            self.visita = Some((v, i, t));
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_mazo_tiene_108_cartas() {
        assert_eq!(mazo_completo().len(), 108);
        let n = Neptuno::new(7);
        assert_eq!(n.total_cartas(), 108);
        assert_eq!(n.manos.iter().map(Vec::len).sum::<usize>() % 2, 0);
        assert!(n.manos[SLAYER].len() == 7 && n.manos[CHIEF].len() == 7 && n.manos[CLOUD].len() == 7);
    }

    #[test]
    fn que_se_puede_jugar() {
        let c = |t, v| Carta { tinte: Some(t), valor: v };
        let siete_rojo = c(Tinte::Rojo, Valor::Numero(7));
        assert!(se_puede(c(Tinte::Rojo, Valor::Numero(2)), siete_rojo, Tinte::Rojo));
        assert!(se_puede(c(Tinte::Azul, Valor::Numero(7)), siete_rojo, Tinte::Rojo));
        assert!(!se_puede(c(Tinte::Azul, Valor::Numero(3)), siete_rojo, Tinte::Rojo));
        assert!(se_puede(Carta { tinte: None, valor: Valor::Mas4 }, siete_rojo, Tinte::Rojo));
        // Sobre un comodin solo manda el color elegido.
        let comodin = Carta { tinte: None, valor: Valor::Comodin };
        assert!(se_puede(c(Tinte::Verde, Valor::Numero(1)), comodin, Tinte::Verde));
        assert!(!se_puede(c(Tinte::Rojo, Valor::Numero(1)), comodin, Tinte::Verde));
    }

    /// Partidas enteras entre bots: nunca se pierde ni aparece una carta, y alguien gana.
    #[test]
    fn partidas_completas_entre_bots() {
        for semilla in 1..20 {
            let mut n = Neptuno::new(semilla * 7919);
            n.miku_automatica = true;
            n.fase = Fase::Juego;
            for _ in 0..20_000 {
                n.actualizar(&Entrada::default(), 0.6);
                assert_eq!(n.total_cartas(), 108, "semilla {semilla}");
                if n.fase == Fase::Fin {
                    break;
                }
            }
            let g = n.ganador.expect("alguien tiene que ganar");
            assert!(n.manos[g].is_empty());
        }
    }

    /// Canela llega, corre al Slayer y juega por el; el Slayer regresa. Mientras
    /// alguien habla nadie juega, y nunca se pierde una carta.
    #[test]
    fn canela_corre_al_slayer_y_el_regresa() {
        let mut vio_canela_jugando = false;
        let mut regreso = false;
        for semilla in 1..12 {
            let mut n = Neptuno::new(semilla * 104729);
            n.miku_automatica = true;
            n.fase = Fase::Juego;
            for _ in 0..20_000 {
                let (jugadas, en_visita) = (n.jugadas, n.visita.is_some());
                n.actualizar(&Entrada::default(), 0.6);
                assert_eq!(n.total_cartas(), 108);
                if en_visita && n.visita.is_some() {
                    assert_eq!(n.jugadas, jugadas, "durante una visita nadie juega");
                }
                vio_canela_jugando |= n.canela_en_mesa && n.ultimo == Some(SLAYER);
                regreso |= n.ultima_visita == Some(Visita::RegresaSlayer);
                if n.fase == Fase::Fin {
                    break;
                }
            }
        }
        assert!(vio_canela_jugando, "Canela tiene que jugar por el Slayer alguna vez");
        assert!(regreso, "el Slayer tiene que regresar alguna vez");
    }

    fn mesa(semilla: u32) -> Neptuno {
        let mut n = Neptuno::new(semilla);
        n.fase = Fase::Juego;
        n.bots_retan = false;
        n.turno = MIKU;
        n.sentido = 1;
        n.color_libre = false;
        n.proxima_rafaga = u32::MAX;
        n.proxima_visita = u32::MAX;
        n
    }

    fn c(t: Tinte, v: Valor) -> Carta {
        Carta { tinte: Some(t), valor: v }
    }
    const MAS4: Carta = Carta { tinte: None, valor: Valor::Mas4 };

    #[test]
    fn mas4_hace_robar_cuatro_y_saltar() {
        let mut n = mesa(3);
        n.color = Tinte::Rojo;
        n.manos[MIKU] = vec![MAS4, c(Tinte::Azul, Valor::Numero(1)), c(Tinte::Azul, Valor::Numero(2))];
        let antes = n.manos[SLAYER].len();
        n.jugar(MIKU, 0, Some(Tinte::Azul));
        assert_eq!(n.manos[SLAYER].len(), antes + 4);
        assert_eq!(n.color, Tinte::Azul);
        assert_eq!(n.turno, CHIEF, "el Slayer pierde el turno");
    }

    /// El +4 a Miku espera su respuesta; aceptar da exactamente cuatro y le quita el turno.
    #[test]
    fn miku_acepta_el_mas4_y_roba_cuatro() {
        let mut n = mesa(9);
        n.color = Tinte::Rojo;
        n.turno = CLOUD;
        n.manos[CLOUD] = vec![MAS4, c(Tinte::Azul, Valor::Numero(1)), c(Tinte::Azul, Valor::Numero(2))];
        let antes = n.manos[MIKU].len();
        n.jugar(CLOUD, 0, Some(Tinte::Azul));
        assert!(n.reto.is_some());
        assert_eq!(n.manos[MIKU].len(), antes, "todavia no roba");
        n.t_turno = 1.0;
        n.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.01);
        assert_eq!(n.manos[MIKU].len(), antes + 4);
        assert_eq!(n.turno, SLAYER, "Miku pierde el turno");
    }

    /// Reto ganado: el +4 era ilegal y roba el que lo tiro. Reto perdido: roba seis el que reto.
    #[test]
    fn retar_el_mas4() {
        for (tenia, cloud_roba, miku_roba, turno) in [(true, 4, 0, MIKU), (false, 0, 6, SLAYER)] {
            let mut n = mesa(9);
            n.color = Tinte::Rojo;
            n.turno = CLOUD;
            let otra = if tenia { Tinte::Rojo } else { Tinte::Azul };
            n.manos[CLOUD] = vec![MAS4, c(otra, Valor::Numero(1)), c(otra, Valor::Numero(2))];
            let (mano_miku, mano_cloud) = (n.manos[MIKU].len(), 2);
            n.jugar(CLOUD, 0, Some(Tinte::Azul));
            n.t_turno = 1.0;
            n.actualizar(&Entrada { derecha: true, ..Default::default() }, 0.01);
            n.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.01);
            assert_eq!(n.manos[CLOUD].len(), mano_cloud + cloud_roba);
            assert_eq!(n.manos[MIKU].len(), mano_miku + miku_roba);
            assert_eq!(n.turno, turno);
        }
    }

    /// Sin jugada se roba hasta que salga una que se pueda jugar, y entonces se puede jugar.
    #[test]
    fn se_roba_hasta_que_salga_algo() {
        let mut n = mesa(21);
        n.color = Tinte::Rojo;
        n.pila = vec![c(Tinte::Rojo, Valor::Numero(5))];
        n.manos[MIKU] = vec![c(Tinte::Azul, Valor::Numero(1))];
        n.mazo = vec![c(Tinte::Rojo, Valor::Numero(9)), c(Tinte::Verde, Valor::Numero(2)), c(Tinte::Azul, Valor::Numero(3)), c(Tinte::Verde, Valor::Numero(7))];
        n.t_turno = 1.0;
        n.actualizar(&Entrada { abajo: true, ..Default::default() }, 0.01);
        for _ in 0..200 {
            n.actualizar(&Entrada::default(), 0.05);
        }
        assert_eq!(n.manos[MIKU].len(), 5, "robo las cuatro: hasta el 9 rojo");
        assert_eq!(n.robo, Some(c(Tinte::Rojo, Valor::Numero(9))));
        assert_eq!(n.turno, MIKU, "le sigue tocando: puede jugarla");
        n.t_turno = 1.0;
        n.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.01);
        assert_eq!(n.arriba(), c(Tinte::Rojo, Valor::Numero(9)));
    }

    /// Los bots tambien roban hasta que salga algo: nunca pasan si queda con que robar.
    #[test]
    fn los_bots_roban_hasta_que_salga_algo() {
        let mut n = mesa(33);
        n.color = Tinte::Rojo;
        n.pila = vec![c(Tinte::Rojo, Valor::Numero(5))];
        n.turno = CHIEF;
        n.manos[CHIEF] = vec![c(Tinte::Azul, Valor::Numero(1)), c(Tinte::Azul, Valor::Numero(2))];
        n.mazo = vec![c(Tinte::Rojo, Valor::Numero(9)), c(Tinte::Verde, Valor::Numero(2)), c(Tinte::Azul, Valor::Numero(3))];
        for _ in 0..100 {
            n.actualizar(&Entrada::default(), 0.05);
            if n.turno != CHIEF {
                break;
            }
        }
        assert_eq!(n.arriba(), c(Tinte::Rojo, Valor::Numero(9)));
        assert_eq!(n.manos[CHIEF].len(), 4);
    }

    /// La rafaga ya no le quita cartas a nadie.
    #[test]
    fn la_rafaga_no_mueve_cartas() {
        let mut n = mesa(5);
        let antes: Vec<usize> = n.manos.iter().map(Vec::len).collect();
        n.rafaga();
        assert_eq!(n.manos.iter().map(Vec::len).collect::<Vec<_>>(), antes);
    }

    /// La carta volteada al repartir hace efecto, y nunca es un +4.
    #[test]
    fn la_carta_inicial_cuenta() {
        let mut vistos = [false; 4];
        for semilla in 1..400 {
            let n = Neptuno::new(semilla * 31);
            let primera = n.arriba();
            assert_ne!(primera.valor, Valor::Mas4);
            match primera.valor {
                Valor::Salta => { assert_eq!(n.turno, SLAYER); vistos[0] = true; }
                Valor::Reversa => { assert_eq!((n.turno, n.sentido), (CLOUD, -1)); vistos[1] = true; }
                Valor::Mas2 => { assert_eq!((n.turno, n.manos[MIKU].len()), (SLAYER, 9)); vistos[2] = true; }
                Valor::Comodin => { assert!(n.color_libre); vistos[3] = true; }
                _ => assert_eq!(n.turno, MIKU),
            }
        }
        assert_eq!(vistos, [true; 4]);
    }

    /// La ultima carta, si es +2, igual se la come el siguiente.
    #[test]
    fn la_ultima_carta_igual_hace_robar() {
        let mut n = mesa(4);
        n.color = Tinte::Rojo;
        n.manos[MIKU] = vec![c(Tinte::Rojo, Valor::Mas2)];
        let antes = n.manos[SLAYER].len();
        n.jugar(MIKU, 0, None);
        assert_eq!(n.ganador, Some(MIKU));
        assert_eq!(n.manos[SLAYER].len(), antes + 2);
    }

    #[test]
    fn reversa_cambia_el_sentido() {
        let mut n = mesa(5);
        n.color = Tinte::Rojo;
        n.manos[MIKU] = vec![c(Tinte::Rojo, Valor::Reversa), c(Tinte::Rojo, Valor::Numero(1)), c(Tinte::Rojo, Valor::Numero(2))];
        n.jugar(MIKU, 0, None);
        assert_eq!(n.sentido, -1);
        assert_eq!(n.turno, CLOUD);
    }

    #[test]
    fn sin_gritar_se_roban_dos() {
        let mut n = mesa(11);
        let c = n.color;
        n.manos[MIKU] = vec![Carta { tinte: Some(c), valor: Valor::Numero(3) }, Carta { tinte: Some(c), valor: Valor::Numero(4) }];
        n.jugar(MIKU, 0, None);
        assert_eq!(n.manos[MIKU].len(), 1);
        // Se le pasa el tiempo sin gritar (los bots juegan mientras tanto).
        let antes = n.manos[MIKU].len();
        n.actualizar(&Entrada::default(), PARA_GRITAR + 0.1);
        assert!(n.manos[MIKU].len() >= antes + 2, "tenia que robar dos");
        // Y gritando a tiempo no pasa nada.
        let mut n = mesa(11);
        n.manos[MIKU] = vec![Carta { tinte: Some(c), valor: Valor::Numero(3) }, Carta { tinte: Some(c), valor: Valor::Numero(4) }];
        n.jugar(MIKU, 0, None);
        n.actualizar(&Entrada { barrel: true, ..Default::default() }, 0.1);
        assert!(n.pendiente.is_none());
        assert_eq!(n.manos[MIKU].len(), 1);
    }
}
