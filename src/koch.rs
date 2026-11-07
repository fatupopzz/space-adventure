//! Aterrizaje en el PLANETA KOCH: WACK A KOCH. Idea original del profe Koch (que conste en acta).
//! Una maquina de arcade con nueve hoyos: el profe sale, usted le pega con el mazo. Tres parciales
//! y un final, cada uno mas rapido y con algo nuevo.
//!
//! Como funciona (lo que hace buenas a estas maquinas, mas lo de los juegos con buen "tacto"):
//! - Cada cuanto sale uno y cuanto dura afuera son dos cosas aparte: al avanzar el parcial, cada
//!   Koch dura menos y salen mas a la vez (asi lo hace la maquina original).
//! - Todo cae en la corchea de la musica (150 BPM): el juego se siente como tocar un tambor.
//! - El golpe se resuelve al instante, sin esperar la animacion. Encima: congelada de 40 a 160 ms
//!   segun que tan fuerte fue, sacudida, chispas de tiza, destello y un tono que sube con el combo.
//! - Mezcla con pesos: el normal, el del birrete (dos golpes), el enojado (si se escapa, reglazo),
//!   el dorado (rapido y vale mucho) y su companero, al que NO hay que pegarle.
//! - Pegarle al aire corta el combo a la mitad y traba el mazo un momento: aporrear no sirve.
//! - La RECURSION: al llenar el medidor, seis segundos en que salen de todos lados por el triple.
//! - El final es el KOCH FRACTAL: el grande se parte en tres medianos, y cada uno en tres chicos,
//!   como el copo de nieve del otro Koch. Se gana cuando se llega al caso base.
//!
//! Tres reglazos y reprueba (igual que en el examen de la Tierra).

use crate::baile::Paso;
use crate::entrada::Entrada;
use crate::matematica::{Mat4, Vec3};
use crate::obj::Modelo;
use crate::obra::Obra;
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;
use std::cell::Cell;
use std::f32::consts::PI;

pub const FOV: f32 = 50.0;
pub const OJO: Vec3 = Vec3::new(0.0, 6.7, 7.9);
pub const MIRA: Vec3 = Vec3::new(0.0, 0.85, -0.7);
pub const HOYOS: usize = 9;
pub const BPM: f32 = 150.0;
/// Segundos por corchea: el reloj de todo el juego.
pub const CORCHEA: f32 = 60.0 / BPM / 2.0;
const REGLAZOS: u32 = 3;
/// Lo que mide el profe, y cuanto de el se asoma.
const ALTO: f32 = 1.12;
const ASOMA: f32 = 0.66;
const ESCALA: f32 = 2.05;
const SUBE: f32 = 0.11;
const BAJA: f32 = 0.13;
const CAIDA: f32 = 0.30;
/// El mazo: que tan grande va y a que altura esta la muneca.
const MAZO: f32 = 0.74;
/// El giro del mazo: desde donde arranca (casi vertical, hacia atras) y lo que tarda en caer. El
/// acierto se decide al apretar (lo que se agarro ya no se escapa); el golpe se siente al tocar.
const GIRO_ALTO: f32 = 2.05;
const GIRO: f32 = 0.085;
/// Desde cuando el stick cuenta como una direccion (la curva del eje vertical da lo mismo en 0.19
/// que el horizontal en 0.35), y cuanto se recuerda el hoyo al soltarlo.
const UMBRAL_X: f32 = 0.35;
const UMBRAL_Y: f32 = 0.19;
const MEMORIA_STICK: f32 = 0.09;
const MAZO_ALTO: f32 = 1.78;
const DURA_RECURSION: f32 = 6.0;
/// Con estos puntos se gana la parte de la nota que da el puntaje.
const META_PUNTOS: u32 = 26000;

/// Donde esta cada hoyo: de atras hacia el frente y de izquierda a derecha (como Q W E / A S D / Z X C).
pub fn hoyo(k: usize) -> Vec3 {
    Vec3::new((k % 3) as f32 * 2.15 - 2.15, 0.0, (k / 3) as f32 * 1.9 - 1.75)
}

/// Un parcial. Los pares van del principio al final del parcial.
struct Parcial {
    titulo: &'static str,
    tema: &'static str,
    novedad: &'static str,
    /// Cuanto dura, en segundos.
    dura: f32,
    /// Corcheas entre uno y otro.
    paso: (f32, f32),
    /// Segundos afuera.
    arriba: (f32, f32),
    /// Cuantos a la vez.
    cupo: (f32, f32),
    /// Pesos de normal, birrete, enojado, companero y dorado.
    mezcla: [u32; 5],
    /// Cada cuantas corcheas sale una rafaga de tres en linea (0: nunca).
    rafaga: u32,
}

const PARCIALES: [Parcial; 3] = [
    Parcial {
        titulo: "PARCIAL 1",
        tema: "Autómatas finitos",
        novedad: "Péguele antes de que se esconda. Si lo agarra saliendo: ¡PERFECTO!",
        dura: 20.0,
        paso: (5.0, 3.5),
        arriba: (1.45, 1.05),
        cupo: (1.0, 2.0),
        mezcla: [86, 0, 14, 0, 0],
        rafaga: 0,
    },
    Parcial {
        titulo: "PARCIAL 2",
        tema: "Autómatas de pila",
        novedad: "El del birrete aguanta dos golpes. Su compañero viene en aro verde: A ÉL NO.",
        dura: 22.0,
        paso: (3.5, 2.5),
        arriba: (1.1, 0.85),
        cupo: (2.0, 3.0),
        mezcla: [54, 20, 11, 13, 2],
        rafaga: 0,
    },
    Parcial {
        titulo: "PARCIAL 3",
        tema: "Máquinas de Turing",
        novedad: "No determinismo: salen en ráfagas de tres. El dorado vale por diez y se va rapidito.",
        dura: 24.0,
        paso: (2.5, 1.7),
        arriba: (0.88, 0.68),
        cupo: (3.0, 5.0),
        mezcla: [40, 16, 14, 22, 8],
        rafaga: 16,
    },
];
const FINAL: usize = 3;
/// Al terminar se viene aporreando botones: se ignoran hasta que la nota termine de contar (si no,
/// el ultimo golpe con cuadrado despegaria la nave).
const ESPERA_RESULTADO: f32 = 1.5;
const DURA_FINAL: f32 = 42.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tipo {
    Normal,
    Birrete,
    Enojado,
    Dorado,
    /// Un companero del salon (cual de los tres): a el NO.
    Companero(usize),
    /// El Koch fractal: 2 el grande, 1 los medianos, 0 los chicos (el caso base).
    Fractal(u8),
}

impl Tipo {
    /// El color de su aro (indice en `Modelos::aro`).
    pub fn aro(self) -> usize {
        match self {
            Tipo::Normal => 1,
            Tipo::Birrete => 5,
            Tipo::Enojado => 2,
            Tipo::Dorado => 3,
            Tipo::Companero(_) => 4,
            Tipo::Fractal(_) => 6,
        }
    }

    fn escala(self) -> f32 {
        match self {
            Tipo::Fractal(2) => ESCALA * 1.55,
            Tipo::Fractal(1) => ESCALA * 1.0,
            Tipo::Fractal(_) => ESCALA * 0.74,
            _ => ESCALA,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Estado {
    Sube,
    Arriba,
    /// Le pegaron: se hunde mareado.
    Golpeado,
    Baja,
    /// El enojado se acabo el tiempo y ahi va la regla (todavia se le puede ganar).
    Regla,
}

#[derive(Clone, Copy, Debug)]
pub struct Asomado {
    pub tipo: Tipo,
    estado: Estado,
    /// Segundos en este estado, y desde que salio.
    t: f32,
    edad: f32,
    /// Cuanto puede estar afuera.
    dura: f32,
    vidas: u32,
    /// 1 justo al recibir un golpe; se apaga solo (el aplastado y el destello).
    pub impacto: f32,
    pub con_birrete: bool,
    /// Ya viene el mazo: ve para arriba, asustado, y no se puede ir.
    atrapado: bool,
}

impl Asomado {
    fn nuevo(tipo: Tipo, dura: f32) -> Self {
        let vidas = match tipo {
            Tipo::Birrete | Tipo::Fractal(1) => 2,
            _ => 1,
        };
        Asomado { tipo, estado: Estado::Sube, t: 0.0, edad: 0.0, dura, vidas, impacto: 0.0, con_birrete: tipo == Tipo::Birrete, atrapado: false }
    }

    fn golpeable(&self) -> bool {
        // Un poquito de gracia al esconderse: pegarle justo cuando se va se siente como acierto.
        matches!(self.estado, Estado::Sube | Estado::Arriba | Estado::Regla) || (self.estado == Estado::Baja && self.t < 0.07)
    }

    /// De 0 (escondido) a 1 (todo afuera), con el rebote de cuando sale.
    fn altura(&self) -> f32 {
        match self.estado {
            Estado::Sube => {
                let x = (self.t / SUBE).min(1.0) - 1.0;
                1.0 + x * x * (2.7 * x + 1.7)
            }
            Estado::Arriba | Estado::Regla => 1.0,
            Estado::Baja => 1.0 - (self.t / BAJA).min(1.0).powi(2),
            Estado::Golpeado => (1.0 - ((self.t - 0.06).max(0.0) / CAIDA).min(1.0).powi(2)).max(0.0),
        }
    }

    /// De 1 (recien salido) a 0 (ya se va).
    pub fn queda(&self) -> f32 {
        if matches!(self.estado, Estado::Sube | Estado::Arriba) {
            (1.0 - self.edad / self.dura.max(0.01)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn mareado(&self) -> bool {
        self.estado == Estado::Golpeado
    }

    /// Como baila: el paso y que tan arriba trae los brazos (0 a 1). Saca los brazos del susto al
    /// salir, saluda burlon, el enojado apunta y levanta la regla, el dorado hace corazon.
    fn paso(&self, susto: f32) -> (Paso, f32) {
        if self.atrapado {
            return (Paso::Quieta, 1.0);
        }
        match (self.estado, self.tipo) {
            (Estado::Regla, _) => (Paso::Senalar, 1.0 - (self.t / 0.2).min(1.0).powi(3)),
            (Estado::Golpeado, _) => (Paso::Quieta, 1.0),
            (Estado::Sube, _) => (Paso::Saludo, 1.0 - (self.t / SUBE).min(1.0)),
            _ if susto > 0.0 => (Paso::Quieta, susto),
            (_, Tipo::Companero(_)) => (Paso::Saludo, 0.0),
            (_, Tipo::Enojado) => (Paso::Senalar, (1.0 - self.queda()) * 0.9),
            (_, Tipo::Fractal(2)) => (Paso::Senalar, 0.55),
            (_, Tipo::Dorado) => (Paso::Corazon, 0.0),
            (_, Tipo::Birrete) => (Paso::Quieta, 0.0),
            _ => (Paso::Baile, 0.0),
        }
    }

    fn expresion(&self, susto: f32) -> &'static str {
        if self.atrapado && !matches!(self.tipo, Tipo::Companero(_)) {
            return "asustada";
        }
        match (self.estado, self.tipo) {
            (Estado::Sube | Estado::Arriba, Tipo::Normal | Tipo::Birrete) if susto > 0.0 => "asustada",
            (Estado::Golpeado, Tipo::Companero(_)) => "asustada",
            (Estado::Golpeado, _) => "espiral",
            (_, Tipo::Companero(_)) => "feliz",
            (_, Tipo::Dorado) => "enamorada",
            (_, Tipo::Enojado | Tipo::Fractal(_)) => "enojado",
            (Estado::Sube, _) => "feliz",
            _ => "serio",
        }
    }
}

/// Lo que hace falta para dibujar a uno de los que estan afuera.
pub struct Pose {
    pub tipo: Tipo,
    pub modelo: Mat4,
    pub expresion: &'static str,
    /// El paso de baile y que tan arriba van los brazos.
    pub paso: Paso,
    pub brazos: f32,
    /// El reloj de su baile.
    pub t: f32,
    pub birrete: Option<Mat4>,
    /// El destello blanco del cuadro del golpe.
    pub destello: bool,
    pub sin_lentes: bool,
    /// Las estrellitas de mareado: donde esta la cabeza.
    pub mareo: Option<Vec3>,
}

/// Una chispa de tiza (un cubito con gravedad).
#[derive(Clone, Copy, Debug)]
pub struct Chispa {
    pub pos: Vec3,
    pub vel: Vec3,
    pub vida: f32,
    pub tam: f32,
    pub color: usize,
    pub gira: f32,
}

/// Lo que sale volando: 0 los lentes, 1 el birrete.
#[derive(Clone, Copy, Debug)]
pub struct Vuela {
    pub cual: usize,
    pub pos: Vec3,
    pub vel: Vec3,
    pub gira: f32,
    pub vida: f32,
}

/// Un letrero que sube desde donde paso algo.
struct Letrero {
    pos: Vec3,
    texto: String,
    color: Color,
    vida: f32,
    grande: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fase {
    Llegada,
    /// El letrero de antes de cada parcial.
    Cartel(usize),
    Jugando,
    Resultado,
}

pub enum AccionKoch {
    Nada,
    Despegar,
}

/// El final: lo que falta por salir.
struct Jefe {
    vidas_grande: u32,
    /// Cuanto falta para que el grande vuelva a salir.
    descansa: f32,
    pendientes: Vec<u8>,
}

const VIDAS_GRANDE: u32 = 12;

pub struct Koch {
    pub fase: Fase,
    pub t: f32,
    pub total: f32,
    pub hoyos: [Option<Asomado>; HOYOS],
    pub parcial: usize,
    /// El reloj del parcial, en segundos.
    reloj: f32,
    corchea: i64,
    proxima: f32,
    jefe: Option<Jefe>,
    pub cursor: usize,
    /// El mazo: donde va (se acerca solo), desde cuando pego y si le dio a algo.
    pub mazo_pos: Vec3,
    pub desde_golpe: f32,
    pub dio: bool,
    /// El mazo es un resorte: el angulo, el ladeo y la cabeza (que se atrasa con respecto al mango)
    /// rebotan solos despues de cada golpe.
    mazo_ang: f32,
    mazo_vel: f32,
    /// Girando hacia un hoyo: cual, y hace cuanto arranco.
    giro: Option<(usize, f32)>,
    ladeo: f32,
    ladeo_vel: f32,
    cabeza_ang: f32,
    cabeza_vel: f32,
    aplasta: f32,
    /// El susto de los demas cuando le pegan a uno, y las ondas que salen del hoyo (hoyo y edad).
    pub susto: f32,
    pub ondas: Vec<(usize, f32)>,
    /// El brinco de toda la maquina con el golpe.
    pub brinco: f32,
    /// El ¡WACK! escrito encima del golpe.
    wack_visible: f32,
    wack_donde: usize,
    enfriar: f32,
    pendiente: Option<usize>,
    raton_antes: Option<(f32, f32)>,
    /// Hace cuanto se solto el stick, y si el gatillo ya venia apretado.
    stick_quieto: f32,
    gatillo_antes: bool,
    con_raton: bool,
    /// La congelada de cuando se pega.
    pausa: f32,
    pub sacudida: f32,
    /// Medio segundo de pantalla roja por un reglazo, y el destello.
    pub rojo: f32,
    pub destello: f32,
    /// Late con la musica: 1 en cada tiempo.
    pub pulso: f32,
    pub puntos: u32,
    pub combo: u32,
    pub mejor_combo: u32,
    pub reglazos: u32,
    pub aciertos: u32,
    pub perfectos: u32,
    pub escapes: u32,
    pub al_aire: u32,
    pub a_companeros: u32,
    /// El medidor (0 a 1) y lo que queda de recursion.
    pub recursion: f32,
    pub fiebre: f32,
    pub chispas: Vec<Chispa>,
    pub vuelan: Vec<Vuela>,
    letreros: Vec<Letrero>,
    /// Lo que dice el profe (texto y segundos que le quedan).
    mensaje: Option<(String, f32)>,
    espera_mensaje: f32,
    pub eventos: Vec<Efecto>,
    /// Hay que arrancar la musica desde el principio (para que caiga en el reloj).
    pub musica_de_nuevo: bool,
    semilla: u32,
    /// Lo de la partida que quedo, para el pasaporte y los premios.
    pub partidas: u32,
    pub mejor_nota: u32,
    pub mejor_puntos: u32,
    pub nota: u32,
    pub caso_base: bool,
    pub aprobo_final: bool,
    pub recursiones: u32,
    pub limpia: bool,
    /// El record guardado (viene del progreso).
    pub record: u32,
    /// Donde quedo la imagen en la ventana (para el raton; se usa el cuadro siguiente).
    imagen: Cell<(f32, f32, f32, f32)>,
}

impl Koch {
    pub fn new(semilla: u32, record: u32) -> Self {
        Koch {
            fase: Fase::Llegada,
            t: 0.0,
            total: 0.0,
            hoyos: [None; HOYOS],
            parcial: 0,
            reloj: 0.0,
            corchea: -1,
            proxima: 0.0,
            jefe: None,
            cursor: 4,
            mazo_pos: hoyo(4),
            desde_golpe: 9.0,
            dio: false,
            mazo_ang: 0.6,
            mazo_vel: 0.0,
            giro: None,
            ladeo: 0.0,
            ladeo_vel: 0.0,
            cabeza_ang: 0.0,
            cabeza_vel: 0.0,
            aplasta: 0.0,
            susto: 0.0,
            ondas: Vec::new(),
            brinco: 0.0,
            wack_visible: 0.0,
            wack_donde: 4,
            enfriar: 0.0,
            pendiente: None,
            raton_antes: None,
            stick_quieto: 9.0,
            gatillo_antes: false,
            con_raton: false,
            pausa: 0.0,
            sacudida: 0.0,
            rojo: 0.0,
            destello: 0.0,
            pulso: 0.0,
            puntos: 0,
            combo: 0,
            mejor_combo: 0,
            reglazos: 0,
            aciertos: 0,
            perfectos: 0,
            escapes: 0,
            al_aire: 0,
            a_companeros: 0,
            recursion: 0.0,
            fiebre: 0.0,
            chispas: Vec::new(),
            vuelan: Vec::new(),
            letreros: Vec::new(),
            mensaje: None,
            espera_mensaje: 0.0,
            eventos: Vec::new(),
            musica_de_nuevo: false,
            semilla: semilla | 1,
            partidas: 0,
            mejor_nota: 0,
            mejor_puntos: 0,
            nota: 0,
            caso_base: false,
            aprobo_final: false,
            recursiones: 0,
            limpia: true,
            record,
            imagen: Cell::new((0.0, 0.0, 0.0, 0.0)),
        }
    }

    fn azar(&mut self) -> f32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) as f32 / 16_777_216.0
    }

    fn reiniciar_partida(&mut self) {
        let (s, partidas, nota, pts, caso, rec, record) = (self.semilla, self.partidas, self.mejor_nota, self.mejor_puntos, self.caso_base, self.recursiones, self.record);
        let combo = self.mejor_combo;
        *self = Koch::new(s, record);
        (self.partidas, self.mejor_nota, self.mejor_puntos, self.caso_base, self.recursiones, self.mejor_combo) = (partidas, nota, pts, caso, rec, combo);
        self.fase = Fase::Cartel(0);
    }

    pub fn jugando(&self) -> bool {
        self.fase == Fase::Jugando
    }

    /// A cuanto va la musica (y el juego).
    pub fn rapidez(&self) -> f32 {
        if self.fiebre > 0.0 { 1.12 } else { 1.0 }
    }

    pub fn en_recursion(&self) -> bool {
        self.fiebre > 0.0
    }

    /// Donde va cada letra del titulo y de que color: brincan y se encienden de una en una con la
    /// musica; en la recursion corren en arcoiris; al pegar, todo el titulo da un salto.
    pub fn titulo(&self) -> Vec<(Mat4, usize)> {
        let reloj = self.total / CORCHEA;
        let ola = reloj as usize % (TITULO.len() + 3);
        let salto = if self.dio && self.desde_golpe < 0.14 { (1.0 - self.desde_golpe / 0.14) * 0.09 } else { 0.0 };
        let grande = 1.0 + self.pulso * self.pulso * 0.035 + salto;
        TITULO
            .iter()
            .enumerate()
            .map(|(k, &(_, x, y, u))| {
                let brinca = if ola == k { 0.14 * (1.0 - reloj.fract()) } else { 0.0 };
                let color = if self.fiebre > 0.0 {
                    (reloj as usize + k) % 3
                } else if ola == k {
                    3
                } else if k < 5 {
                    1
                } else {
                    0
                };
                let m = Mat4::translation(Vec3::new(x + u * 2.5, y + brinca, Z_TITULO)) * Mat4::scale(grande);
                (m, color)
            })
            .collect()
    }

    /// Los brillos: estrellas que se prenden y apagan sobre las letras (donde y de que tamano).
    pub fn brillos(&self) -> Vec<(Vec3, f32)> {
        (0..16)
            .filter_map(|k| {
                let s = k as f32 * 2.399;
                let t = self.total * (0.9 + (s * 1.3).sin().abs() * 0.5) + s;
                // Cada brillo salta a otro lugar cada vez que se apaga.
                let vuelta = (t / PI).floor();
                let tam = t.sin().abs().powi(6);
                if tam < 0.04 {
                    return None;
                }
                let (_, x, y, u) = TITULO[((s * 3.7 + vuelta * 5.0).sin().abs() * 8.99) as usize];
                let (dx, dy) = ((s * 5.1 + vuelta * 3.3).sin().abs(), (s * 7.7 + vuelta * 2.1).cos().abs());
                let fuerza = if self.fiebre > 0.0 { 0.75 } else { 0.5 };
                Some((Vec3::new(x + dx * u * 5.0, y + dy * u * 7.0, Z_TITULO + 0.5), tam * fuerza * (0.7 + u)))
            })
            .collect()
    }

    pub fn multiplicador(&self) -> u32 {
        let base = match self.combo {
            0..=7 => 1,
            8..=19 => 2,
            20..=39 => 3,
            _ => 4,
        };
        if self.fiebre > 0.0 { base * 3 } else { base }
    }

    fn progreso(&self) -> f32 {
        PARCIALES.get(self.parcial).map_or(0.0, |p| (self.reloj / p.dura).clamp(0.0, 1.0))
    }

    /// Lo que falta del parcial (de 1 a 0).
    pub fn tiempo(&self) -> f32 {
        let dura = PARCIALES.get(self.parcial).map_or(DURA_FINAL, |p| p.dura);
        (1.0 - self.reloj / dura).clamp(0.0, 1.0)
    }

    fn cuantos_afuera(&self) -> usize {
        self.hoyos.iter().flatten().filter(|a| matches!(a.estado, Estado::Sube | Estado::Arriba)).count()
    }

    fn decir(&mut self, texto: &str, segundos: f32) {
        self.mensaje = Some((texto.to_string(), segundos));
        self.espera_mensaje = segundos + 5.0;
    }

    /// Una linea del profe, si no acaba de hablar.
    fn comentar(&mut self, lineas: &[&str]) {
        if self.espera_mensaje <= 0.0 {
            let k = (self.azar() * lineas.len() as f32) as usize % lineas.len();
            self.decir(lineas[k], 2.2);
        }
    }

    pub fn hablante(&self) -> Option<(&'static str, String, Voz)> {
        self.mensaje.as_ref().map(|(t, _)| ("PROFE KOCH", t.clone(), Voz::Profe))
    }

    pub fn despedida(&self) -> String {
        match (self.partidas, self.caso_base) {
            (0, _) => "Despegue del Planeta Koch. Nos fuimos sin pegarle a nadie. El profe anotó nuestros nombres. Con lapicero.".into(),
            (_, false) => format!("Despegue del Planeta Koch. Mejor nota: {}. El profe dice que hay recuperación. También dice que no hay.", self.mejor_nota),
            _ => format!("Despegue del Planeta Koch. Mejor nota: {}. Llegamos al caso base. El profe pidió que conste que la idea fue suya.", self.mejor_nota),
        }
    }

    // -----------------------------------------------------------------------------------------
    // La entrada
    // -----------------------------------------------------------------------------------------

    /// El hoyo que apunta el raton: el mas cercano en pantalla.
    fn hoyo_del_raton(&self, (mx, my): (f32, f32)) -> Option<usize> {
        let (x, y, w, h) = self.imagen.get();
        if w <= 0.0 {
            return None;
        }
        (0..HOYOS)
            .filter_map(|k| {
                let (px, py) = proyectar(hoyo(k) + Vec3::new(0.0, 0.6, 0.0), (x, y, w, h))?;
                Some((k, (px - mx).powi(2) + (py - my).powi(2)))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(k, _)| k)
    }

    /// Mueve el cursor y dice si se quiere pegar (y donde).
    fn leer_golpe(&mut self, e: &Entrada) -> Option<usize> {
        // El stick, la cruceta y las flechas apuntan directo: cada direccion es un hoyo y soltar
        // es el del centro (asi se apunta de un manotazo, sin ir de hoyo en hoyo). Los umbrales
        // quedan parejos en los dos ejes: `cabecear` ya viene con la curva de los sticks.
        let (x, y) = (e.horizontal, e.cabecear);
        let (lado, altura) = (x.abs() > UMBRAL_X, y.abs() > UMBRAL_Y);
        let mueve = lado || altura;
        if let Some(p) = e.puntero {
            if self.raton_antes.is_some_and(|a| (a.0 - p.0).abs() + (a.1 - p.1).abs() > 1.0) {
                self.con_raton = true;
            }
            self.raton_antes = Some(p);
            if self.con_raton {
                if let Some(k) = self.hoyo_del_raton(p) {
                    self.cursor = k;
                }
            }
        }
        if mueve {
            self.con_raton = false;
            self.stick_quieto = 0.0;
        }
        // Al soltar el stick, el hoyo se recuerda un momentito: empujar y apretar casi juntos
        // cuenta para donde se empujo, no para el centro. Con el teclado, el mazo se queda donde pego.
        if !self.con_raton && (mueve || (e.hay_control && self.stick_quieto > MEMORIA_STICK)) {
            let col = if !lado { 1 } else if x < 0.0 { 0 } else { 2 };
            let fila = if !altura { 1 } else if y > 0.0 { 0 } else { 2 };
            self.cursor = fila * 3 + col;
        }
        // El gatillo se mantiene: cuenta solo al apretarlo, no cada cuadro.
        let gatillo = e.gatillo && !self.gatillo_antes;
        self.gatillo_antes = e.gatillo;
        if let Some(k) = e.celda {
            self.con_raton = false;
            return Some(k);
        }
        // En el control pega cualquier boton (cruz, circulo, triangulo, cuadrado, L1, R1 y R2): asi
        // se pueden alternar dedos como en un tambor, cuando vienen seguidos.
        let boton = e.hay_control && (e.pestana_izq || e.pestana_der || e.barrel || e.orbitar || e.aterrizar || gatillo);
        (e.aceptar || e.clic || boton).then_some(self.cursor)
    }

    /// Para las capturas, los videos y las pruebas: juega solo (no le pega a los companeros).
    pub fn chofer(&self) -> Entrada {
        match self.fase {
            Fase::Llegada => Entrada { aceptar: self.t > 0.6, ..Default::default() },
            Fase::Cartel(_) => Entrada { aceptar: self.t > 0.9, ..Default::default() },
            Fase::Resultado => Entrada::default(),
            Fase::Jugando => {
                if self.pendiente.is_some() || self.enfriar > 0.0 || self.giro.is_some() {
                    return Entrada::default();
                }
                let mejor = (0..HOYOS)
                    .filter(|&k| self.hoyos[k].is_some_and(|a| a.golpeable() && !matches!(a.tipo, Tipo::Companero(_)) && a.edad > 0.09))
                    .min_by(|&a, &b| {
                        let q = |k: usize| self.hoyos[k].map_or(1.0, |x| x.queda());
                        q(a).total_cmp(&q(b))
                    });
                Entrada { celda: mejor, ..Default::default() }
            }
        }
    }

    // -----------------------------------------------------------------------------------------
    // El juego
    // -----------------------------------------------------------------------------------------

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionKoch {
        self.total += dt;
        self.stick_quieto += dt;
        let golpe = self.leer_golpe(e);
        self.sacudida = (self.sacudida - dt * 2.6).max(0.0);
        self.rojo = (self.rojo - dt * 2.4).max(0.0);
        self.destello = (self.destello - dt * 7.0).max(0.0);
        self.pulso = (self.pulso - dt * 5.0).max(0.0);
        if self.fase == Fase::Jugando {
            if let Some(k) = golpe {
                self.pendiente = Some(k);
            }
        }
        if self.pausa > 0.0 {
            // La congelada: todo se queda quieto en el cuadro del golpe.
            self.pausa -= dt;
            return AccionKoch::Nada;
        }
        self.t += dt;
        self.desde_golpe += dt;
        self.enfriar = (self.enfriar - dt).max(0.0);
        self.espera_mensaje = (self.espera_mensaje - dt).max(0.0);
        if let Some((_, q)) = &mut self.mensaje {
            *q -= dt;
            if *q <= 0.0 {
                self.mensaje = None;
            }
        }
        self.mover_mazo(dt);
        self.mover_efectos(dt);

        match self.fase {
            Fase::Llegada => {
                if self.t < dt * 1.5 {
                    self.decir("¿Me va a pegar? Bien. Fue idea mía. Pero se califica.", 4.5);
                }
                if e.aterrizar {
                    return AccionKoch::Despegar;
                }
                if (e.aceptar || e.clic) && self.t > 0.3 {
                    self.eventos.push(Efecto::Acierto);
                    self.mensaje = None;
                    self.fase = Fase::Cartel(0);
                    self.t = 0.0;
                }
            }
            Fase::Cartel(n) => {
                if self.t < dt * 1.5 {
                    self.eventos.push(Efecto::Sello);
                }
                if (e.aceptar || e.clic) && self.t > 0.7 || self.t > 3.4 {
                    self.arrancar(n);
                }
            }
            Fase::Jugando => self.jugar(dt),
            Fase::Resultado => {
                // La nota cuenta para arriba, y suena.
                let antes = ((self.t - dt - 0.3) / 0.06).floor() as i32;
                if self.t > 0.3 && self.t < 1.2 && ((self.t - 0.3) / 0.06).floor() as i32 != antes {
                    self.eventos.push(Efecto::Teclas);
                }
                if self.t > ESPERA_RESULTADO && (e.aceptar || e.clic) {
                    self.eventos.push(Efecto::Acierto);
                    self.reiniciar_partida();
                } else if self.t > ESPERA_RESULTADO && e.aterrizar {
                    return AccionKoch::Despegar;
                }
            }
        }
        AccionKoch::Nada
    }

    fn arrancar(&mut self, n: usize) {
        self.parcial = n;
        self.reloj = 0.0;
        self.corchea = -1;
        self.proxima = 2.0;
        self.t = 0.0;
        self.fase = Fase::Jugando;
        self.hoyos = [None; HOYOS];
        self.musica_de_nuevo = true;
        self.pendiente = None;
        self.eventos.push(Efecto::Claqueta);
        if n == FINAL {
            self.jefe = Some(Jefe { vidas_grande: VIDAS_GRANDE, descansa: 0.8, pendientes: Vec::new() });
            self.decir("Este es el final. Y soy recursivo.", 2.4);
        }
    }

    fn jugar(&mut self, dt: f32) {
        // En la recursion la musica va mas rapido, y el reloj con ella: todo sigue en la corchea.
        self.reloj += dt * self.rapidez();
        if self.fiebre > 0.0 {
            self.fiebre -= dt;
            if self.fiebre <= 0.0 {
                self.fiebre = 0.0;
                self.recursion = 0.0;
            }
        }
        // El golpe de esta vuelta (o el que se apreto durante la congelada).
        if self.enfriar <= 0.0 && self.giro.is_none() {
            if let Some(k) = self.pendiente.take() {
                self.arrancar_giro(k);
            }
        }
        // El mazo llego: ahora si el golpe.
        if let Some((k, t)) = self.giro {
            if t >= GIRO {
                self.golpear(k);
                if self.pausa > 0.0 {
                    return;
                }
            }
        }
        let corchea = (self.reloj / CORCHEA) as i64;
        if corchea != self.corchea {
            self.corchea = corchea;
            if corchea % 2 == 0 {
                self.pulso = 1.0;
            }
            self.en_corchea(corchea as u32);
        }
        self.mover_asomados(dt);
        if let Some(j) = &mut self.jefe {
            j.descansa = (j.descansa - dt).max(0.0);
        }
        if self.parcial == FINAL {
            if self.jefe_vencido() {
                self.ganar_final();
            } else if self.reloj >= DURA_FINAL {
                self.terminar(false);
            }
        } else if self.reloj >= PARCIALES[self.parcial].dura {
            // Se acaba el parcial: los que queden se van y sigue el siguiente.
            for a in self.hoyos.iter_mut().flatten() {
                if a.estado != Estado::Golpeado && a.estado != Estado::Baja {
                    a.estado = Estado::Baja;
                    a.t = 0.0;
                }
            }
            if self.hoyos.iter().all(|h| h.is_none()) || self.reloj >= PARCIALES[self.parcial].dura + 0.7 {
                self.fiebre = 0.0;
                self.fase = Fase::Cartel(self.parcial + 1);
                self.t = 0.0;
                self.mensaje = None;
            }
        }
    }

    fn jefe_vencido(&self) -> bool {
        self.jefe.as_ref().is_some_and(|j| j.vidas_grande == 0 && j.pendientes.is_empty()) && !self.hoyos.iter().flatten().any(|a| matches!(a.tipo, Tipo::Fractal(_)))
    }

    fn en_corchea(&mut self, n: u32) {
        if self.parcial == FINAL {
            self.corchea_del_final(n);
            return;
        }
        let Some(p) = PARCIALES.get(self.parcial) else { return };
        if self.reloj > p.dura - 0.3 {
            return;
        }
        let k = self.progreso();
        let cruza = |(a, b): (f32, f32)| a + (b - a) * k;
        // En la recursion salen en cada corchea, de todos lados, y sin trampas.
        if self.fiebre > 0.0 {
            if self.cuantos_afuera() < 6 {
                let tipo = if self.azar() < 0.08 { Tipo::Dorado } else { Tipo::Normal };
                self.sacar(tipo, 0.95);
            }
            return;
        }
        let arriba = cruza(p.arriba);
        if p.rafaga > 0 && n > 0 && n % p.rafaga == 0 {
            // Tres en linea, los tres de un solo.
            const LINEAS: [[usize; 3]; 8] = [[0, 1, 2], [3, 4, 5], [6, 7, 8], [0, 3, 6], [1, 4, 7], [2, 5, 8], [0, 4, 8], [2, 4, 6]];
            let linea = LINEAS[(self.azar() * 8.0) as usize % 8];
            for h in linea {
                if self.hoyos[h].is_none() {
                    self.hoyos[h] = Some(Asomado::nuevo(Tipo::Normal, arriba + 0.2));
                }
            }
            self.eventos.push(Efecto::KochSale);
            self.proxima = n as f32 + cruza(p.paso).round().max(1.0) * 2.0;
            return;
        }
        if (n as f32) < self.proxima || self.cuantos_afuera() as f32 >= cruza(p.cupo).round() {
            return;
        }
        // Normalmente en su paso; a veces se adelanta una corchea, para que no sea predecible.
        let paso = cruza(p.paso);
        let adelanto = if self.azar() < 0.2 { 1.0 } else { 0.0 };
        self.proxima = n as f32 + (paso - adelanto).round().max(1.0);
        let tipo = self.escoger(p.mezcla);
        let dura = match tipo {
            Tipo::Dorado => arriba * 0.62,
            Tipo::Birrete => arriba + 0.35,
            Tipo::Enojado => arriba + 0.15,
            _ => arriba,
        };
        self.sacar(tipo, dura);
    }

    fn escoger(&mut self, mezcla: [u32; 5]) -> Tipo {
        let total: u32 = mezcla.iter().sum();
        let mut r = (self.azar() * total as f32) as u32;
        for (i, &m) in mezcla.iter().enumerate() {
            if r < m {
                return match i {
                    0 => Tipo::Normal,
                    1 => Tipo::Birrete,
                    2 => Tipo::Enojado,
                    3 => Tipo::Companero((self.azar() * 3.0) as usize % 3),
                    _ => Tipo::Dorado,
                };
            }
            r -= m;
        }
        Tipo::Normal
    }

    /// Saca uno en un hoyo libre (que no sea el que se acaba de pegar, si se puede).
    fn sacar(&mut self, tipo: Tipo, dura: f32) -> bool {
        let libres: Vec<usize> = (0..HOYOS).filter(|&k| self.hoyos[k].is_none()).collect();
        if libres.is_empty() {
            return false;
        }
        let k = libres[(self.azar() * libres.len() as f32) as usize % libres.len()];
        self.hoyos[k] = Some(Asomado::nuevo(tipo, dura));
        self.eventos.push(Efecto::KochSale);
        true
    }

    fn corchea_del_final(&mut self, n: u32) {
        let Some(j) = &self.jefe else { return };
        let (descansa, vidas, hay_pendiente) = (j.descansa, j.vidas_grande, !j.pendientes.is_empty());
        // El grande sale del centro; si esta abajo, salen otros a molestar.
        if vidas > 0 {
            if descansa <= 0.0 && self.hoyos[4].is_none() {
                self.hoyos[4] = Some(Asomado { vidas, ..Asomado::nuevo(Tipo::Fractal(2), 2.4) });
                self.eventos.push(Efecto::Latido);
                return;
            }
            if self.hoyos[4].is_none() && n % 2 == 0 {
                let cual = (self.azar() * 3.0) as usize % 3;
                let tipo = if self.azar() < 0.28 { Tipo::Companero(cual) } else { Tipo::Normal };
                self.sacar(tipo, 0.85);
            }
            return;
        }
        if n % 2 == 0 && hay_pendiente {
            let nivel = self.jefe.as_mut().and_then(|j| j.pendientes.pop());
            if let Some(nivel) = nivel {
                let dura = if nivel == 1 { 2.0 } else { 1.05 };
                if !self.sacar(Tipo::Fractal(nivel), dura) {
                    if let Some(j) = &mut self.jefe {
                        j.pendientes.push(nivel);
                    }
                }
            }
        } else if n % 8 == 5 && self.azar() < 0.55 {
            let cual = (self.azar() * 3.0) as usize % 3;
            self.sacar(Tipo::Companero(cual), 0.9);
        }
    }

    fn mover_asomados(&mut self, dt: f32) {
        let mut escapan: Vec<(usize, Tipo)> = Vec::new();
        let mut reglazo: Option<usize> = None;
        let hacia = self.giro.map(|g| g.0);
        for (k, h) in self.hoyos.iter_mut().enumerate() {
            let Some(a) = h else { continue };
            a.impacto = (a.impacto - dt * 7.0).max(0.0);
            // Solo lo detiene el mazo que viene en camino hacia el: si ya no viene, sigue su vida.
            if hacia != Some(k) {
                a.atrapado = false;
            }
            // Mientras viene el mazo se queda tal cual (si no, el que se iba escondiendo terminaba
            // de esconderse, ya no se le podia pegar y se quedaba ahi para siempre, tapando el hoyo).
            if a.atrapado && a.estado != Estado::Sube {
                continue;
            }
            a.t += dt;
            a.edad += dt;
            match a.estado {
                Estado::Sube if a.t >= SUBE => {
                    a.estado = Estado::Arriba;
                    a.t = 0.0;
                }
                Estado::Sube | Estado::Arriba if a.edad >= a.dura => {
                    if a.tipo == Tipo::Enojado {
                        a.estado = Estado::Regla;
                    } else {
                        a.estado = Estado::Baja;
                        escapan.push((k, a.tipo));
                    }
                    a.t = 0.0;
                }
                Estado::Regla if a.t >= 0.24 => {
                    a.estado = Estado::Baja;
                    a.t = 0.0;
                    reglazo = Some(k);
                }
                Estado::Baja if a.t >= BAJA => *h = None,
                Estado::Golpeado if a.t >= CAIDA + 0.06 => *h = None,
                _ => {}
            }
        }
        for (k, tipo) in escapan {
            self.escapo(k, tipo);
        }
        if let Some(k) = reglazo {
            self.dar_reglazo(k);
        }
    }

    fn escapo(&mut self, k: usize, tipo: Tipo) {
        match tipo {
            // Dejar pasar a un companero esta bien: es lo correcto.
            Tipo::Companero(_) => {}
            Tipo::Fractal(2) => {
                if let Some(j) = &mut self.jefe {
                    j.descansa = 1.6;
                }
            }
            Tipo::Fractal(nivel) => {
                // Se fue sin resolver: la llamada recursiva vuelve a salir.
                if let Some(j) = &mut self.jefe {
                    j.pendientes.insert(0, nivel);
                }
                self.combo = 0;
                self.letrero(k, "¡OTRA VEZ!", ui::ROJO, false);
            }
            _ => {
                self.escapes += 1;
                if self.combo >= 8 {
                    self.eventos.push(Efecto::Fallo);
                    self.letrero(k, "COMBO PERDIDO", ui::ROJO, false);
                }
                self.eventos.push(Efecto::KochRisa);
                self.combo = 0;
                self.comentar(&["Muy lento. Anótelo.", "Eso viene en el examen.", "¿Estudió o solo vino?", "Tarde. Como su entrega."]);
            }
        }
    }

    fn dar_reglazo(&mut self, k: usize) {
        if self.fiebre > 0.0 {
            return;
        }
        self.reglazos += 1;
        self.combo = 0;
        self.recursion *= 0.6;
        self.rojo = 1.0;
        self.sacudida = 1.0;
        self.pausa = 0.14;
        self.eventos.push(Efecto::Reglazo);
        self.letrero(k, "¡REGLAZO!", ui::ROJO, true);
        self.limpia = false;
        if self.reglazos >= REGLAZOS {
            self.terminar(false);
        } else {
            self.comentar(&["Eso duele más a mí que a usted. Mentira.", "Le dije que se calificaba.", "Uno menos. Quedan pocos."]);
        }
    }

    /// Se apreto: el mazo se lanza desde atras, con su soplido, y lo que estaba ahi ya no se escapa.
    fn arrancar_giro(&mut self, k: usize) {
        self.cursor = k;
        self.mazo_pos = hoyo(k);
        self.giro = Some((k, 0.0));
        self.mazo_vel = 0.0;
        self.eventos.push(Efecto::Aire);
        self.dio = false;
        if let Some(a) = &mut self.hoyos[k] {
            if a.golpeable() {
                a.atrapado = true;
                self.dio = true;
            }
        }
    }

    fn golpear(&mut self, k: usize) {
        self.cursor = k;
        self.mazo_pos = hoyo(k);
        self.desde_golpe = 0.0;
        self.giro = None;
        self.aplasta = 1.0;
        let Some(mut a) = self.hoyos[k].filter(|a| a.golpeable()) else {
            self.eventos.push(Efecto::Tablero);
            // Al aire: corta el combo y traba el mazo.
            self.dio = false;
            self.mazo_ang = -0.5;
            self.mazo_vel = 5.0;
            self.cabeza_vel = 9.0;
            self.brinco = self.brinco.max(0.4);
            self.al_aire += 1;
            self.enfriar = 0.2;
            self.combo /= 2;
            self.polvo(hoyo(k) + Vec3::new(0.0, 0.1, 0.0), 6, 5, 0.7);
            return;
        };
        self.dio = true;
        // Pega y rebota; la cabeza sigue de largo un poquito.
        self.mazo_ang = 0.0;
        self.mazo_vel = 9.5;
        self.cabeza_vel = 15.0;
        self.susto = 1.0;
        self.brinco = 1.0;
        self.ondas.push((k, 0.0));
        self.enfriar = 0.06;
        a.impacto = 1.0;
        a.atrapado = false;
        let arriba = hoyo(k) + Vec3::new(0.0, ALTO * a.tipo.escala() * ASOMA, 0.0);
        // A su companero no.
        if let Tipo::Companero(_) = a.tipo {
            self.a_companeros += 1;
            a.estado = Estado::Golpeado;
            a.t = 0.0;
            self.hoyos[k] = Some(a);
            self.polvo(arriba, 10, 4, 1.0);
            self.letrero(k, "¡A ÉL NO!", ui::ROJO, true);
            self.dar_reglazo(k);
            self.comentar(&["¡Ese es su compañero!", "Ya salió en el grupo de WhatsApp.", "Eso se llama falta de ética."]);
            return;
        }
        a.vidas = a.vidas.saturating_sub(if self.fiebre > 0.0 { 2 } else { 1 });
        if a.vidas > 0 {
            self.golpe_parcial(k, a, arriba);
            return;
        }
        // Acierto.
        let perfecto = a.edad < (a.dura * 0.34).max(0.26) && !matches!(a.tipo, Tipo::Fractal(2));
        a.estado = Estado::Golpeado;
        a.t = 0.0;
        self.hoyos[k] = Some(a);
        self.combo += 1;
        self.mejor_combo = self.mejor_combo.max(self.combo);
        self.aciertos += 1;
        self.perfectos += perfecto as u32;
        let base = match a.tipo {
            Tipo::Normal => 100,
            Tipo::Birrete => 200,
            Tipo::Enojado => 250,
            Tipo::Dorado => 1000,
            Tipo::Fractal(1) => 300,
            Tipo::Fractal(0) => 200,
            Tipo::Fractal(_) => 1500,
            Tipo::Companero(_) => 0,
        };
        let ganados = (if perfecto { base * 3 / 2 } else { base }) * self.multiplicador();
        self.puntos += ganados;
        // La congelada y la sacudida segun que tan fuerte fue.
        let (pausa, sacude) = match a.tipo {
            Tipo::Dorado | Tipo::Enojado => (0.085, 0.42),
            Tipo::Fractal(1) => (0.10, 0.5),
            _ if perfecto => (0.065, 0.3),
            _ => (0.045, 0.2),
        };
        self.pausa = pausa;
        self.sacudida = self.sacudida.max(sacude);
        self.destello = self.destello.max(if a.tipo == Tipo::Dorado { 0.32 } else { 0.05 });
        // El sonido: el mazazo y una nota que sube con el combo (de nota en nota, como en Peggle).
        let wack = self.wack();
        self.eventos.push(wack);
        self.eventos.push(nota_del_combo(self.combo));
        if perfecto {
            self.eventos.push(Efecto::Perfecto);
        }
        // Y el pobre: ay, ay.
        if self.azar() < 0.55 || a.tipo != Tipo::Normal {
            self.eventos.push(Efecto::KochAy);
        }
        // El mazazo al final: en el control, la ultima vibracion es la que se siente.
        self.eventos.push(Efecto::Mazazo);
        let color_chispa = if a.tipo == Tipo::Dorado { 3 } else { 0 };
        self.polvo(arriba, if perfecto { 18 } else { 12 }, color_chispa, 1.0);
        let lado = self.azar() * 3.0 - 1.5;
        self.vuelan.push(Vuela { cual: 0, pos: arriba - Vec3::new(0.0, 0.2, -0.2), vel: Vec3::new(lado, 5.0, 1.8), gira: 0.0, vida: 0.9 });
        let texto = if perfecto { format!("¡PERFECTO! +{ganados}") } else { format!("+{ganados}") };
        let color = if perfecto { Color::new(255, 230, 110, 255) } else { Color::WHITE };
        self.letrero(k, &texto, color, perfecto);
        // Como en las caricaturas: la palabra del golpe, enorme.
        self.wack_visible = 1.0;
        self.wack_donde = k;
        self.sumar_recursion(if perfecto { 0.075 } else { 0.05 } + if a.tipo == Tipo::Dorado { 0.4 } else { 0.0 });
        if let Tipo::Fractal(nivel) = a.tipo {
            self.romper_fractal(k, nivel);
        }
    }

    /// Un golpe que no lo tumba: el birrete, un mediano o el grande.
    fn golpe_parcial(&mut self, k: usize, mut a: Asomado, arriba: Vec3) {
        let mut hito = false;
        if a.tipo == Tipo::Birrete && a.con_birrete {
            a.con_birrete = false;
            a.dura += 0.35;
            let lado = self.azar() * 4.0 - 2.0;
            self.vuelan.push(Vuela { cual: 1, pos: arriba + Vec3::new(0.0, 0.1, 0.0), vel: Vec3::new(lado, 7.0, 1.0), gira: 0.0, vida: 1.2 });
            self.eventos.push(Efecto::Pop);
            self.letrero(k, "¡OTRA!", Color::new(120, 235, 255, 255), false);
        }
        if a.tipo == Tipo::Fractal(2) {
            if let Some(j) = &mut self.jefe {
                j.vidas_grande = a.vidas;
            }
            hito = true;
            self.letrero(k, &format!("{}", a.vidas), Color::new(255, 150, 230, 255), true);
        }
        self.hoyos[k] = Some(a);
        self.combo += 1;
        self.mejor_combo = self.mejor_combo.max(self.combo);
        self.puntos += 50 * self.multiplicador();
        self.pausa = if hito { 0.05 } else { 0.04 };
        self.sacudida = self.sacudida.max(if hito { 0.4 } else { 0.2 });
        let wack = self.wack();
        self.eventos.push(wack);
        self.eventos.push(nota_del_combo(self.combo));
        self.eventos.push(Efecto::Mazazo);
        self.polvo(arriba, 8, if hito { 6 } else { 0 }, 0.9);
        self.sumar_recursion(0.025);
    }

    fn romper_fractal(&mut self, k: usize, nivel: u8) {
        if nivel == 2 {
            if let Some(j) = &mut self.jefe {
                j.vidas_grande = 0;
                j.pendientes = vec![1, 1, 1];
            }
            self.pausa = 0.16;
            self.sacudida = 1.0;
            self.destello = 0.55;
            self.eventos.push(Efecto::Recursion);
            self.polvo(hoyo(k) + Vec3::new(0.0, 1.8, 0.0), 40, 6, 2.0);
            self.decir("¡Se parte! ¿Cuál es su caso base?", 2.6);
        } else if nivel == 1 {
            if let Some(j) = &mut self.jefe {
                j.pendientes.extend([0u8, 0, 0]);
            }
        }
    }

    /// ¡WACK!: tres tonos, para que no suene siempre igual.
    fn wack(&mut self) -> Efecto {
        [Efecto::Wack, Efecto::Wack2, Efecto::Wack3][(self.azar() * 3.0) as usize % 3]
    }

    fn sumar_recursion(&mut self, cuanto: f32) {
        if self.fiebre > 0.0 || self.parcial == FINAL {
            return;
        }
        self.recursion = (self.recursion + cuanto).min(1.0);
        if self.recursion >= 1.0 {
            self.fiebre = DURA_RECURSION;
            self.recursiones += 1;
            self.destello = 0.6;
            self.sacudida = 1.0;
            self.eventos.push(Efecto::Recursion);
            self.decir("¡RECURSIÓN! Eso no era el caso base...", 2.2);
        }
    }

    fn ganar_final(&mut self) {
        // Lo que sobro de tiempo, en puntos.
        self.puntos += ((DURA_FINAL - self.reloj).max(0.0) * 80.0) as u32;
        self.caso_base = true;
        self.aprobo_final = true;
        self.terminar(true);
    }

    /// Termina la partida y saca la nota.
    fn terminar(&mut self, gano_final: bool) {
        self.nota = calcular_nota(self.aciertos, self.escapes + self.al_aire / 3 + self.a_companeros * 2, self.puntos, gano_final, self.reglazos);
        self.partidas += 1;
        self.mejor_nota = self.mejor_nota.max(self.nota);
        self.mejor_puntos = self.mejor_puntos.max(self.puntos);
        self.record = self.record.max(self.puntos);
        self.fase = Fase::Resultado;
        self.t = 0.0;
        self.fiebre = 0.0;
        self.pausa = 0.0;
        self.mensaje = None;
        for a in self.hoyos.iter_mut().flatten() {
            if a.estado != Estado::Golpeado {
                a.estado = Estado::Baja;
                a.t = 0.0;
            }
        }
        self.eventos.push(if self.nota >= 61 { Efecto::Sello } else { Efecto::Fallo });
        let (texto, dura) = match (gano_final, self.nota) {
            (true, 100) => ("Cien. Revisé dos veces. Tres. No se acostumbre.", 5.0),
            (true, _) => ("Llegó al caso base. Eso sí no me lo esperaba.", 5.0),
            (false, n) if n >= 61 => ("Aprobado. Le falta el final, pero aprobado.", 5.0),
            _ => ("Reprobado. Pero el planeta sigue aquí. Hay recuperación.", 5.0),
        };
        self.decir(texto, dura);
    }

    fn polvo(&mut self, pos: Vec3, n: usize, color: usize, fuerza: f32) {
        for _ in 0..n {
            if self.chispas.len() >= 140 {
                break;
            }
            let (a, b, c, d) = (self.azar(), self.azar(), self.azar(), self.azar());
            let dir = Vec3::new((a - 0.5) * 2.0, 0.3 + b, (c - 0.5) * 2.0);
            let color = if color == 0 && d < 0.3 { 1 } else { color };
            self.chispas.push(Chispa { pos, vel: dir * ((3.0 + d * 4.0) * fuerza), vida: 0.35 + b * 0.45, tam: 0.05 + c * 0.09, color, gira: a * 9.0 });
        }
    }

    fn letrero(&mut self, k: usize, texto: &str, color: Color, grande: bool) {
        self.letreros.push(Letrero { pos: hoyo(k) + Vec3::new(0.0, 2.55, 0.0), texto: texto.to_string(), color, vida: 0.75, grande });
        if self.letreros.len() > 8 {
            self.letreros.remove(0);
        }
    }

    /// El mazo: resortes con poco freno, para que rebote despues de pegar y se quede vibrando.
    fn mover_mazo(&mut self, dt: f32) {
        let meta = hoyo(self.cursor);
        let antes = self.mazo_pos.x;
        if let Some((_, t)) = &mut self.giro {
            // Cae acelerando: despacito arriba, de golpe abajo.
            *t += dt;
            let k = (*t / GIRO).min(1.0);
            let hasta = if self.dio { 0.0 } else { -0.5 };
            self.mazo_ang = hasta + (GIRO_ALTO - hasta) * (1.0 - k * k);
            self.mazo_vel = 0.0;
            self.cabeza_ang = -0.55 * (1.0 - k);
            self.susto = self.susto.max(0.6 * k);
            return;
        }
        self.mazo_pos = self.mazo_pos.lerp(meta, (dt * 24.0).min(1.0));
        let empuje = (self.mazo_pos.x - antes) / dt.max(1e-4);
        // Se ladea hacia donde va y se pasa un poquito al llegar.
        self.ladeo_vel += (-empuje * 0.09 - self.ladeo * 150.0 - self.ladeo_vel * 13.0) * dt;
        self.ladeo += self.ladeo_vel * dt;
        // En reposo respira con la musica, listo para caer.
        let reposo = 0.2 + self.pulso * 0.06 + (self.total * 3.1).sin() * 0.018;
        self.mazo_vel += ((reposo - self.mazo_ang) * 190.0 - self.mazo_vel * 12.0) * dt;
        self.mazo_ang += self.mazo_vel * dt;
        self.cabeza_vel += (-self.cabeza_ang * 420.0 - self.cabeza_vel * 11.0) * dt;
        self.cabeza_ang += self.cabeza_vel * dt;
        self.aplasta = (self.aplasta - dt * 6.5).max(0.0);
        self.susto = (self.susto - dt * 4.5).max(0.0);
        self.brinco = (self.brinco - dt * 7.0).max(0.0);
        self.wack_visible = (self.wack_visible - dt * 3.6).max(0.0);
        for o in &mut self.ondas {
            o.1 += dt;
        }
        self.ondas.retain(|o| o.1 < 0.17);
    }

    fn mover_efectos(&mut self, dt: f32) {
        for c in &mut self.chispas {
            c.vida -= dt;
            c.vel.y -= 16.0 * dt;
            c.pos = c.pos + c.vel * dt;
            c.gira += dt * 8.0;
            if c.pos.y < 0.03 {
                c.pos.y = 0.03;
                c.vel.y *= -0.4;
            }
        }
        self.chispas.retain(|c| c.vida > 0.0);
        for v in &mut self.vuelan {
            v.vida -= dt;
            v.vel.y -= 18.0 * dt;
            v.pos = v.pos + v.vel * dt;
            v.gira += dt * 14.0;
        }
        self.vuelan.retain(|v| v.vida > 0.0);
        for l in &mut self.letreros {
            l.vida -= dt;
            l.pos.y += dt * 1.1;
        }
        self.letreros.retain(|l| l.vida > 0.0);
    }

    // -----------------------------------------------------------------------------------------
    // Para dibujar
    // -----------------------------------------------------------------------------------------

    pub fn camara(&self) -> (Vec3, Vec3, Vec3) {
        let s = self.sacudida * self.sacudida * 0.34;
        let (a, b) = ((self.total * 71.0).sin(), (self.total * 93.0).cos());
        let empuje = if self.fiebre > 0.0 { Vec3::new((self.total * 0.9).sin() * 0.35, -0.45, -0.7) } else { Vec3::zero() };
        // Un golpecito de zoom con cada tiempo: la camara tambien va con la musica.
        let late = (MIRA - OJO).normalize() * (self.pulso * self.pulso * 0.09);
        let arriba = if self.fiebre > 0.0 { Vec3::new((self.total * 1.3).sin() * 0.05, 1.0, 0.0) } else { Vec3::new(a * s * 0.05, 1.0, 0.0) };
        (OJO + empuje + late + Vec3::new(a * s, b * s, 0.0), MIRA + Vec3::new(b * s * 0.5, a * s * 0.5, 0.0), arriba)
    }

    /// El mazo: el mango y la cabeza (la cabeza se aplasta con el golpe y se atrasa con los giros).
    pub fn mazo(&self) -> (Mat4, Mat4) {
        let (mango, cabeza) = self.mazo_en(self.mazo_ang);
        (mango, cabeza * Mat4::scale_xyz(Vec3::new(1.0 + 0.2 * self.aplasta * self.aplasta, 1.0 - 0.34 * self.aplasta * self.aplasta, 1.0 + 0.2 * self.aplasta * self.aplasta)))
    }

    fn mazo_en(&self, angulo: f32) -> (Mat4, Mat4) {
        // Descansa hacia la camara, para no tapar los hoyos de atras; al girar llega justo encima.
        // Viene un poco de lado (como un brazo de verdad), y si pega al aire, la muneca baja al tablero.
        let (hacia, baja, lado) = match self.giro {
            Some((_, t)) => {
                let k = (t / GIRO).min(1.0);
                (0.75 * (1.0 - k), if self.dio { 0.0 } else { 1.05 * k * k }, 0.6 * (1.0 - k * k))
            }
            None => (
                0.75 * ((self.desde_golpe - 0.03) / 0.16).clamp(0.0, 1.0),
                if self.dio { 0.0 } else { 1.05 * (1.0 - (self.desde_golpe / 0.16).min(1.0)).powi(2) },
                0.0,
            ),
        };
        let mano = Mat4::translation(self.mazo_pos + Vec3::new(0.0, MAZO_ALTO - baja, 1.75 * MAZO + hacia)) * Mat4::rotation_y(-lado) * Mat4::rotation_z(self.ladeo + lado * 0.5) * Mat4::rotation_x(angulo) * Mat4::scale(MAZO);
        let cabeza = mano * Mat4::translation(Vec3::new(0.0, 0.0, -1.75)) * Mat4::rotation_x(self.cabeza_ang * 0.35) * Mat4::rotation_z(self.ladeo * 1.6);
        (mano, cabeza)
    }

    /// La estela del giro: copias del mazo en el arco que acaba de recorrer (como en las caricaturas).
    pub fn estela(&self) -> Vec<Mat4> {
        if self.giro.is_none() && self.desde_golpe > 0.06 {
            return Vec::new();
        }
        [0.32f32, 0.68, 1.05, 1.45].iter().filter(|d| self.mazo_ang + **d <= GIRO_ALTO).map(|d| self.mazo_en(self.mazo_ang + d).1).collect()
    }

    pub fn poses(&self) -> Vec<Pose> {
        self.hoyos
            .iter()
            .enumerate()
            .filter_map(|(k, h)| {
                let a = h.as_ref()?;
                let esc = a.tipo.escala();
                // Todos brincan con la musica, y se agachan del susto cuando le pegan a otro.
                let late = if a.golpeable() { self.pulso * self.pulso * 0.05 } else { 0.0 };
                let agacha = if a.golpeable() && a.impacto <= 0.0 { self.susto * 0.14 } else { 0.0 };
                let y = -ALTO * esc + ALTO * esc * ASOMA * a.altura() + late - agacha;
                let s = a.impacto;
                // Aplastado contra el piso del hoyo: ancho y chaparro. Y el rebote de cuando sale.
                let sale = if a.estado == Estado::Sube { (1.0 - a.t / SUBE).max(0.0) } else { 0.0 };
                let aplasta = Mat4::scale_xyz(Vec3::new(1.0 + 0.38 * s - 0.12 * sale, 1.0 - 0.5 * s + 0.2 * sale, 1.0 + 0.38 * s - 0.12 * sale));
                let ladeo = if a.mareado() { (a.t * 22.0).sin() * 0.2 } else { (k as f32 + self.total * BPM / 60.0 * PI).sin() * 0.045 + s * 0.16 };
                // Un poco para atras: asi mira a la camara y se le ve la cara.
                let cuerpo = Mat4::translation(hoyo(k)) * aplasta * Mat4::translation(Vec3::new(0.0, y, 0.0)) * Mat4::rotation_z(ladeo) * Mat4::rotation_y(PI) * Mat4::rotation_x(0.42) * Mat4::scale(esc);
                let susto = if a.impacto > 0.0 { 0.0 } else { self.susto };
                let (paso, brazos) = a.paso(susto);
                let birrete = a.con_birrete.then_some(cuerpo);
                let cabeza = hoyo(k) + Vec3::new(0.0, y + ALTO * esc + 0.12, 0.0);
                Some(Pose {
                    tipo: a.tipo,
                    modelo: cuerpo,
                    expresion: a.expresion(susto),
                    paso,
                    brazos,
                    t: self.total * BPM / 120.0 + k as f32 * 0.37,
                    birrete,
                    destello: s > 0.72,
                    sin_lentes: a.mareado(),
                    mareo: (a.mareado() && !matches!(a.tipo, Tipo::Companero(_))).then_some(cabeza),
                })
            })
            .collect()
    }

    /// Los aros: de que color esta cada hoyo y cuanto le queda al que esta adentro.
    pub fn aros(&self) -> [(usize, f32); HOYOS] {
        let mut v = [(0usize, 0.0f32); HOYOS];
        for k in 0..HOYOS {
            if self.fiebre > 0.0 {
                // En la recursion, los aros corren en arcoiris con la musica.
                v[k].0 = 1 + ((self.total / CORCHEA) as usize + k) % 6;
            }
            if let Some(a) = &self.hoyos[k] {
                if a.golpeable() {
                    v[k] = (a.tipo.aro(), a.queda());
                }
            }
        }
        v
    }

    pub fn vida_jefe(&self) -> Option<f32> {
        let j = self.jefe.as_ref()?;
        let chicos = j.pendientes.iter().map(|&n| if n == 1 { 4.0 } else { 1.0 }).sum::<f32>()
            + self.hoyos.iter().flatten().filter(|a| a.golpeable()).map(|a| match a.tipo {
                Tipo::Fractal(1) => 4.0,
                Tipo::Fractal(0) => 1.0,
                _ => 0.0,
            }).sum::<f32>();
        // El grande vale la mitad de la barra; la otra mitad, los doce que salen de el.
        Some(if j.vidas_grande > 0 { 0.5 + 0.5 * j.vidas_grande as f32 / VIDAS_GRANDE as f32 } else { 0.5 * (chicos / 12.0).min(1.0) })
    }

    // -----------------------------------------------------------------------------------------
    // La pantalla
    // -----------------------------------------------------------------------------------------

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, imagen: Rectangle, ancho: i32, alto: i32) {
        let rect = (imagen.x, imagen.y, imagen.width, imagen.height);
        self.imagen.set(rect);
        match self.fase {
            Fase::Llegada => self.ui_llegada(d, e, texto, ancho, alto),
            Fase::Cartel(n) => {
                self.ui_marcador(d, ancho);
                self.ui_cartel(d, e, n, ancho, alto);
            }
            Fase::Jugando => {
                self.ui_guia(d, e, rect, ancho, alto);
                self.ui_marcador(d, ancho);
                self.ui_letreros(d, rect);
                self.ui_comentario(d, texto, ancho, alto);
                self.ui_recursion(d, e, ancho, alto);
            }
            Fase::Resultado => {
                self.ui_letreros(d, rect);
                self.ui_resultado(d, e, texto, ancho, alto);
            }
        }
    }

    fn ui_comentario(&self, d: &mut RaylibDrawHandle, texto: &str, ancho: i32, alto: i32) {
        if texto.is_empty() {
            return;
        }
        // Abajo a la izquierda, chiquito: que no tape los hoyos.
        let w = 440.min(ancho - 40);
        let h = ui::alto_dialogo("PROFE KOCH", texto, w);
        ui::caja_dialogo(d, "PROFE KOCH", Color::new(255, 150, 200, 255), texto, 20, alto - h - 64, w);
    }

    fn ui_llegada(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, ancho: i32, alto: i32) {
        let h = alto as f32;
        // El titulo es el letrero gigante de atras: aqui no se repite. Las reglas van debajo de el.
        let t2 = "idea original del profe Koch  ·  que conste en acta";
        let y_credito = (h * 0.455) as i32;
        d.draw_rectangle(0, y_credito - 4, ancho, 32, Color::new(14, 6, 34, 170));
        ui::subtitulo(d, t2, (ancho - ui::medir(t2, 22)) / 2, y_credito, 22, Color::WHITE);
        let (pw, ph) = (620.min(ancho - 40), 168);
        let (px, py) = ((ancho - pw) / 2, y_credito + 34);
        ui::panel(d, px, py, pw, ph);
        let filas: [(Color, &str); 5] = [
            (Color::new(190, 120, 255, 255), "Aro lila: un Koch. Péguele al salir: ¡PERFECTO!"),
            (Color::new(120, 235, 255, 255), "Aro celeste: con birrete. Aguanta dos golpes."),
            (Color::new(255, 90, 100, 255), "Aro rojo: enojado. Si se le escapa: reglazo."),
            (Color::new(120, 240, 150, 255), "Aro verde: su compañero. A ÉL NO. También es reglazo."),
            (Color::new(255, 215, 80, 255), "Aro dorado: rapidísimo. Vale por diez."),
        ];
        for (k, (c, s)) in filas.iter().enumerate() {
            let fy = py + 12 + k as i32 * 30;
            d.draw_rectangle(px + 22, fy + 3, 18, 18, *c);
            ui::texto(d, s, px + 54, fy, 20, ui::TEXTO);
        }
        if !texto.is_empty() {
            let w = 620.min(ancho - 40);
            ui::caja_dialogo(d, "PROFE KOCH", Color::new(255, 150, 200, 255), texto, (ancho - w) / 2, py + ph + 40, w);
        }
        if self.record > 0 {
            let r = format!("Récord: {}", self.record);
            ui::subtitulo(d, &r, ancho - ui::medir(&r, 20) - 24, y_credito + 2, 20, Color::new(255, 225, 120, 255));
        }
        // Como se juega, segun lo que este conectado.
        let (como, y_como) = (
            if e.hay_control { "Stick o cruceta: apuntar  ·  Cualquier botón: pegar" } else { "Q W E · A S D · Z X C: pegar directo  ·  Ratón: apuntar y clic" },
            py + ph + 8,
        );
        ui::subtitulo(d, como, (ancho - ui::medir(como, 20)).max(20) / 2, y_como, 20, Color::new(170, 235, 255, 255));
        ui::ayudas(d, &[(e.nombre("aceptar"), "Empezar"), (e.nombre("aterrizar"), "Despegar")], ancho, alto);
    }

    fn ui_cartel(&self, d: &mut RaylibDrawHandle, e: &Entrada, n: usize, ancho: i32, alto: i32) {
        let h = alto as f32;
        let entra = (self.t / 0.22).min(1.0);
        d.draw_rectangle(0, 0, ancho, alto, Color::new(14, 6, 34, (150.0 * entra) as u8));
        let (titulo, tema, novedad) = match PARCIALES.get(n) {
            Some(p) => (p.titulo, p.tema, p.novedad),
            None => ("EXAMEN FINAL", "El Koch fractal", "El grande se parte en tres, y esos en tres. Llegue al caso base antes de que se acabe el tiempo."),
        };
        let caida = (1.0 - entra).powi(2) * -80.0;
        let mut tam = (h * 0.13) as i32;
        while tam > 20 && ui::medir_titulo(titulo, tam) > ancho - 50 {
            tam -= 1;
        }
        let y = (h * 0.24 + caida) as i32;
        ui::titulo_de(d, titulo, (ancho - ui::medir_titulo(titulo, tam)) / 2, y, tam, if n == FINAL { Color::new(255, 120, 210, 255) } else { Color::new(255, 225, 90, 255) });
        ui::subtitulo(d, tema, (ancho - ui::medir(tema, 28)) / 2, y + tam + 12, 28, Color::WHITE);
        let w = 680.min(ancho - 60);
        ui::parrafo(d, novedad, (ancho - w) / 2, y + tam + 62, 22, w, Color::new(190, 235, 255, 255));
        // Un dato del copo de nieve de Koch (es otro Koch; lo anotamos para que no se confunda).
        let dato = match n {
            0 => "Dato: el copo de nieve de Koch es de 1904. De Helge von Koch. No es pariente. Hasta donde sabemos.",
            1 => "Dato: cada iteración del copo multiplica el perímetro por 4/3. Crece sin parar.",
            2 => "Dato: el área del copo no pasa de 8/5 del triángulo. Perímetro infinito, área finita. Como la entrega.",
            _ => "Dato: toda recursión necesita un caso base. Si no, no termina. Como la entrega.",
        };
        let ancho_dato = ui::medir(dato, 18);
        ui::subtitulo(d, dato, ((ancho - ancho_dato) / 2).max(10), (h * 0.74) as i32, 18, ui::TENUE);
        ui::ayudas(d, &[(e.nombre("aceptar"), "¡Ya!")], ancho, alto);
    }

    /// El marcador: arriba, y que no tape el tablero.
    fn ui_marcador(&self, d: &mut RaylibDrawHandle, ancho: i32) {
        // Los puntos, a la izquierda.
        ui::panel(d, 16, 14, 230, 74);
        ui::texto(d, "PUNTOS", 30, 22, 15, ui::TENUE);
        ui::texto(d, &format!("{}", self.puntos), 30, 40, 34, Color::WHITE);
        // El parcial y su tiempo, al centro.
        let (nombre, tema) = match PARCIALES.get(self.parcial) {
            Some(p) => (p.titulo, p.tema),
            None => ("EXAMEN FINAL", "Koch fractal"),
        };
        let w = 400.min(ancho - 560).max(180);
        let x = (ancho - w) / 2;
        let linea = format!("{nombre}  ·  {tema}");
        ui::subtitulo(d, &ui::recortado(&linea, 18, w), x + (w - ui::medir(&ui::recortado(&linea, 18, w), 18)) / 2, 14, 18, Color::WHITE);
        d.draw_rectangle(x, 42, w, 12, Color::new(30, 16, 60, 230));
        let k = self.tiempo();
        let color = if k < 0.2 { Color::new(255, 90, 110, 255) } else { Color::new(150, 220, 255, 255) };
        d.draw_rectangle(x, 42, (w as f32 * k) as i32, 12, color);
        // La vida del jefe, abajo del tiempo.
        if let Some(v) = self.vida_jefe() {
            ui::subtitulo(d, "KOCH FRACTAL", x, 58, 14, Color::new(255, 150, 230, 255));
            d.draw_rectangle(x, 76, w, 14, Color::new(30, 16, 60, 230));
            d.draw_rectangle(x, 76, (w as f32 * v) as i32, 14, Color::new(255, 110, 220, 255));
        }
        // Los reglazos que quedan, a la derecha: reglitas.
        let px = ancho - 216;
        ui::panel(d, px, 14, 200, 74);
        ui::texto(d, "REGLAZOS", px + 14, 22, 15, ui::TENUE);
        for r in 0..REGLAZOS {
            let usado = r < self.reglazos;
            let c = if usado { Color::new(70, 50, 90, 255) } else { Color::new(255, 205, 120, 255) };
            let rx = px + 14 + r as i32 * 58;
            d.draw_rectangle(rx, 50, 48, 14, c);
            for m in 0..5 {
                d.draw_rectangle(rx + 4 + m * 9, 50, 2, 6, Color::new(90, 60, 40, 255));
            }
        }
        // El combo, grande, bajo los puntos.
        if self.combo >= 2 {
            let mult = self.multiplicador();
            let golpe = (1.0 - (self.desde_golpe / 0.16).min(1.0)) * 7.0;
            let tam = 34 + golpe as i32;
            let s = format!("{} COMBO", self.combo);
            let color = match mult {
                1 => Color::WHITE,
                2 => Color::new(150, 235, 255, 255),
                3 => Color::new(255, 225, 100, 255),
                _ => Color::new(255, 130, 220, 255),
            };
            ui::titulo_de(d, &s, 20, 98, (tam as f32 * 0.72) as i32, color);
            if mult > 1 {
                ui::texto(d, &format!("× {mult}"), 22, 98 + tam, 30, color);
            }
        }
    }

    /// Las letras de cada hoyo en el primer parcial (con teclado), y como apuntar con el control.
    fn ui_guia(&self, d: &mut RaylibDrawHandle, e: &Entrada, rect: (f32, f32, f32, f32), ancho: i32, alto: i32) {
        const LETRAS: [&str; 9] = ["Q", "W", "E", "A", "S", "D", "Z", "X", "C"];
        if !e.hay_control && (self.parcial == 0 || self.reloj < 4.0) && !self.con_raton {
            for (k, l) in LETRAS.iter().enumerate() {
                if let Some((x, y)) = proyectar(hoyo(k) + Vec3::new(0.0, 0.0, 0.92), rect) {
                    ui::tecla(d, l, x as i32 - 10, y as i32 - 12, 18, self.cursor == k && self.desde_golpe < 0.1);
                }
            }
        }
        if self.parcial == 0 && self.reloj < 7.0 {
            let acciones: Vec<(&str, &str)> = if e.hay_control {
                vec![(e.nombre("apuntar"), "Apuntar"), (e.nombre("aceptar"), "Pegar (o cualquier botón)")]
            } else {
                vec![("Q W E · A S D · Z X C", "Pegar en ese hoyo"), ("ratón", "Apuntar y clic")]
            };
            ui::ayudas(d, &acciones, ancho, alto);
        }
    }

    fn ui_letreros(&self, d: &mut RaylibDrawHandle, rect: (f32, f32, f32, f32)) {
        if self.wack_visible > 0.0 {
            if let Some((x, y)) = proyectar(hoyo(self.wack_donde) + Vec3::new(0.0, 1.35, 0.0), rect) {
                // Revienta grande y se encoge poquito, con la misma fuente del letrero.
                let k = self.wack_visible;
                let u = rect.3 * (0.0085 + 0.0065 * k.powi(3));
                let a = (k.min(0.4) / 0.4 * 255.0) as u8;
                ui::bloques(d, "WACK!", x, y, u, Color::new(255, 240, 120, 255), Color::new(255, 150, 40, 255), a);
            }
        }
        for l in &self.letreros {
            let Some((x, y)) = proyectar(l.pos, rect) else { continue };
            let a = ((l.vida / 0.25).min(1.0) * 255.0) as u8;
            let tam = if l.grande { 30 } else { 24 };
            let brote = 1.0 + ((l.vida - 0.6).max(0.0) / 0.15) * 0.4;
            let tam = (tam as f32 * brote) as i32;
            let w = ui::medir(&l.texto, tam);
            ui::texto(d, &l.texto, x as i32 - w / 2 + 2, y as i32 + 2, tam, Color::new(30, 10, 50, a));
            ui::texto(d, &l.texto, x as i32 - w / 2, y as i32, tam, Color::new(l.color.r, l.color.g, l.color.b, a));
        }
    }

    /// El medidor de recursion, abajo al centro.
    fn ui_recursion(&self, d: &mut RaylibDrawHandle, e: &Entrada, ancho: i32, alto: i32) {
        if self.parcial == FINAL {
            return;
        }
        let w = 460.min(ancho - 80);
        let (x, y) = ((ancho - w) / 2, alto - 34 - if self.parcial == 0 && self.reloj < 7.0 { ui::alto_ayudas(&[(e.nombre("aceptar"), "Pegar")], ancho) } else { 0 });
        d.draw_rectangle(x - 3, y - 3, w + 6, 22, Color::new(20, 10, 44, 225));
        if self.fiebre > 0.0 {
            let k = self.fiebre / DURA_RECURSION;
            for b in 0..24 {
                let ww = w / 24;
                if (b as f32 / 24.0) < k {
                    let c = [Color::new(255, 90, 180, 255), Color::new(255, 220, 90, 255), Color::new(120, 235, 255, 255), Color::new(190, 120, 255, 255)][(b + (self.total * 12.0) as i32) as usize % 4];
                    d.draw_rectangle(x + b * ww, y, ww - 1, 16, c);
                }
            }
            let s = "¡¡RECURSIÓN!!  × 3";
            let tam = (alto as f32 * 0.075) as i32;
            let brinco = ((self.total * BPM / 30.0 * PI).sin().abs() * 6.0) as i32;
            ui::titulo_de(d, s, (ancho - ui::medir_titulo(s, tam)) / 2, y - tam - 14 - brinco, tam, Color::new(255, 120, 220, 255));
        } else {
            d.draw_rectangle(x, y, (w as f32 * self.recursion) as i32, 16, Color::new(190, 120, 255, 255));
            let s = "RECURSIÓN";
            ui::subtitulo(d, s, x + (w - ui::medir(s, 14)) / 2, y, 14, Color::WHITE);
        }
    }

    fn ui_resultado(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, ancho: i32, alto: i32) {
        let h = alto as f32;
        let entra = (self.t / 0.4).min(1.0);
        d.draw_rectangle(0, 0, ancho, alto, Color::new(14, 6, 34, (200.0 * entra) as u8));
        let aprobo = self.nota >= 61;
        let cabeza = if self.aprobo_final {
            "CASO BASE"
        } else if self.reglazos >= REGLAZOS {
            "¡TRES REGLAZOS!"
        } else if self.parcial == FINAL {
            "NO SE DETUVO"
        } else {
            "SE ACABÓ"
        };
        let tam = (h * 0.075) as i32;
        ui::titulo_de(d, cabeza, (ancho - ui::medir_titulo(cabeza, tam)) / 2, (h * 0.07) as i32, tam, if aprobo { Color::new(255, 225, 90, 255) } else { Color::new(255, 110, 130, 255) });
        // La nota va contando, como en una maquina de verdad.
        let cuenta = ((self.t - 0.3) / 0.9).clamp(0.0, 1.0);
        let n = (self.nota as f32 * cuenta) as u32;
        let tam_n = (h * 0.24) as i32;
        let s = format!("{n}");
        let color = if aprobo { Color::new(120, 240, 160, 255) } else { Color::new(255, 110, 130, 255) };
        ui::titulo_de(d, &s, (ancho - ui::medir_titulo(&s, tam_n)) / 2, (h * 0.18) as i32, tam_n, color);
        if cuenta >= 1.0 {
            let veredicto = match self.nota {
                100 => "¡CIEN! Enmárquelo.",
                90..=99 => "Excelente. El profe casi sonríe.",
                75..=89 => "Muy bien. Pasa sin zona.",
                61..=74 => "Aprobado. Con 61, pero aprobado.",
                _ => "Reprobado. Nos vemos en la recuperación.",
            };
            ui::subtitulo(d, veredicto, (ancho - ui::medir(veredicto, 26)) / 2, (h * 0.46) as i32, 26, Color::WHITE);
        }
        let filas = [
            format!("Puntos: {}{}", self.puntos, if self.puntos >= self.record && self.puntos > 0 { "  ¡RÉCORD!" } else { "" }),
            format!("Aciertos: {}   ·   ¡Perfectos!: {}   ·   Mejor combo: {}", self.aciertos, self.perfectos, self.mejor_combo),
            format!("Se fueron: {}   ·   Al aire: {}   ·   A sus compañeros: {}", self.escapes, self.al_aire, self.a_companeros),
        ];
        for (k, f) in filas.iter().enumerate() {
            let tam = if k == 0 { 24 } else { 19 };
            ui::subtitulo(d, f, (ancho - ui::medir(f, tam)).max(20) / 2, (h * 0.55) as i32 + k as i32 * 32, tam, if k == 0 { Color::new(255, 225, 120, 255) } else { ui::TEXTO });
        }
        if !texto.is_empty() {
            let w = 640.min(ancho - 40);
            ui::caja_dialogo(d, "PROFE KOCH", Color::new(255, 150, 200, 255), texto, (ancho - w) / 2, (h * 0.74) as i32, w);
        }
        ui::ayudas(d, &[(e.nombre("aceptar"), "Otra vez"), (e.nombre("aterrizar"), "Despegar")], ancho, alto);
    }

    /// Para las capturas: `koch` (la llegada), `koch_cartel`, `koch_juego`, `koch_recursion`,
    /// `koch_jefe`, `koch_partido`, `koch_final`.
    pub fn preparar_captura(&mut self, modo: &str) {
        match modo {
            "koch_cartel" => {
                self.fase = Fase::Cartel(1);
                self.parcial = 1;
            }
            "koch_juego" | "koch_primero" => {
                self.arrancar(if modo == "koch_primero" { 0 } else { 2 });
                self.puntos = 2350;
                self.combo = 12;
            }
            "koch_recursion" => {
                self.arrancar(2);
                self.fiebre = DURA_RECURSION;
                self.puntos = 5400;
                self.combo = 24;
            }
            "koch_jefe" | "koch_partido" => {
                self.arrancar(FINAL);
                self.puntos = 7800;
                if modo == "koch_partido" {
                    if let Some(j) = &mut self.jefe {
                        j.vidas_grande = 0;
                        j.pendientes = vec![0, 0, 1, 0, 1, 1];
                    }
                }
            }
            // El mazo a media caida (`koch_giro`), a punto de tocar (`koch_giro2`) y tocando (`koch_toque`).
            "koch_giro" | "koch_giro2" | "koch_toque" => {
                self.arrancar(0);
                self.puntos = 900;
                self.hoyos[4] = Some(Asomado { estado: Estado::Arriba, ..Asomado::nuevo(Tipo::Normal, 1.2) });
                self.hoyos[0] = Some(Asomado { estado: Estado::Arriba, ..Asomado::nuevo(Tipo::Birrete, 1.2) });
                self.hoyos[8] = Some(Asomado { estado: Estado::Arriba, ..Asomado::nuevo(Tipo::Companero(0), 1.2) });
                self.arrancar_giro(4);
                self.total = 1.0;
                let t = match modo {
                    "koch_giro" => 0.02,
                    "koch_giro2" => 0.06,
                    _ => GIRO + 0.001,
                };
                self.giro = Some((4, t));
                self.mover_mazo(0.0);
                if modo == "koch_toque" {
                    self.golpear(4);
                }
            }
            "koch_final" => {
                self.parcial = FINAL;
                self.aciertos = 118;
                self.perfectos = 41;
                self.escapes = 9;
                self.puntos = 11840;
                self.mejor_combo = 47;
                self.aprobo_final = true;
                self.terminar(true);
            }
            _ => {}
        }
    }
}

fn nota_del_combo(combo: u32) -> Efecto {
    const NOTAS: [Efecto; 8] = [Efecto::Nota0, Efecto::Nota1, Efecto::Nota2, Efecto::Nota3, Efecto::Nota4, Efecto::Nota5, Efecto::Nota6, Efecto::Nota7];
    // Sube con el combo; arriba se queda alternando entre las dos mas altas.
    let k = combo.saturating_sub(1) as usize;
    NOTAS[if k < 8 { k } else { 6 + k % 2 }]
}

/// La nota: lo que mas pesa es la puntería; luego los puntos, y el final.
pub fn calcular_nota(aciertos: u32, errores: u32, puntos: u32, gano_final: bool, reglazos: u32) -> u32 {
    let total = (aciertos + errores).max(1) as f32;
    let punteria = aciertos as f32 / total;
    let n = punteria * 60.0 + (puntos as f32 / META_PUNTOS as f32).min(1.0) * 25.0 + if gano_final { 15.0 } else { 0.0 } - reglazos as f32 * 4.0;
    n.round().clamp(0.0, 100.0) as u32
}

/// Donde cae un punto del mundo en la ventana (con la camara quieta).
fn proyectar(p: Vec3, (x, y, w, h): (f32, f32, f32, f32)) -> Option<(f32, f32)> {
    let vista = Mat4::look_at(OJO, MIRA, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(FOV.to_radians(), 16.0 / 9.0, 0.1, 120.0);
    let c = (proyeccion * vista).mul_vec4(crate::matematica::Vec4::new(p.x, p.y, p.z, 1.0));
    (c.w > 0.01).then(|| (x + (c.x / c.w + 1.0) * 0.5 * w, y + (1.0 - c.y / c.w) * 0.5 * h))
}

// ---------------------------------------------------------------------------------------------
// El copo de nieve de Koch
// ---------------------------------------------------------------------------------------------

/// Los vertices del copo de nieve de Koch en la iteracion `n`, con las puntas a radio 1: el
/// triangulo inicial, y cada segmento se cambia por cuatro de un tercio del largo. Por eso el
/// perimetro crece 4/3 en cada iteracion.
pub fn copo_de_koch(n: usize) -> Vec<[f32; 2]> {
    let mut puntos: Vec<[f32; 2]> = (0..3)
        .map(|k| {
            let a = PI / 2.0 - k as f32 * 2.0 * PI / 3.0;
            [a.cos(), a.sin()]
        })
        .collect();
    for _ in 0..n {
        let mut nuevos = Vec::with_capacity(puntos.len() * 4);
        for i in 0..puntos.len() {
            let (a, b) = (puntos[i], puntos[(i + 1) % puntos.len()]);
            let d = [(b[0] - a[0]) / 3.0, (b[1] - a[1]) / 3.0];
            let p1 = [a[0] + d[0], a[1] + d[1]];
            let p3 = [a[0] + 2.0 * d[0], a[1] + 2.0 * d[1]];
            // El pico: el tercio de en medio, girado 60 grados hacia afuera (el contorno va en el
            // sentido del reloj, asi que afuera queda a la izquierda).
            let (seno, coseno) = (PI / 3.0).sin_cos();
            let p2 = [p1[0] + d[0] * coseno - d[1] * seno, p1[1] + d[0] * seno + d[1] * coseno];
            nuevos.extend([a, p1, p2, p3]);
        }
        puntos = nuevos;
    }
    puntos
}

// ---------------------------------------------------------------------------------------------
// Los modelos
// ---------------------------------------------------------------------------------------------

pub struct Modelos {
    pub mueble: Modelo,
    /// El titulo gigante: cada letra en dorado, rosa, celeste y blanco.
    pub letras: Vec<[Modelo; 4]>,
    pub letrero_fondo: Modelo,
    pub foquitos: [Modelo; 2],
    pub brillo: Modelo,
    /// Los aros de los hoyos: apagado, lila, rojo, dorado, verde, celeste y rosa.
    pub aros: [Modelo; 7],
    pub aros_finos: [Modelo; 7],
    pub mazo_mango: Modelo,
    pub mazo_cabeza: Modelo,
    pub mazo_estela: Modelo,
    pub regla: Modelo,
    pub cubos: [Modelo; 7],
    pub birrete: Modelo,
    pub lentes: Modelo,
    pub copo_suelto: Modelo,
    pub torres: Modelo,
    pub suelo: Modelo,
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

const NEON: [(&str, [f32; 3]); 7] = [
    ("n0", [0.26, 0.17, 0.42]),
    ("n1", [0.72, 0.44, 1.0]),
    ("n2", [1.0, 0.27, 0.33]),
    ("n3", [1.0, 0.80, 0.25]),
    ("n4", [0.35, 1.0, 0.52]),
    ("n5", [0.35, 0.90, 1.0]),
    ("n6", [1.0, 0.38, 0.86]),
];

pub fn paleta(o: &mut Obra) {
    const COLORES: [(&str, [f32; 3], f32); 14] = [
        ("tablero", [0.25, 0.14, 0.44], 0.05),
        ("loseta", [0.33, 0.19, 0.55], 0.06),
        ("lado", [0.15, 0.09, 0.30], 0.03),
        ("hoyo", [0.10, 0.05, 0.20], 0.25),
        ("hoyo_b", [0.20, 0.11, 0.34], 0.1),
        ("madera", [0.50, 0.31, 0.18], 0.0),
        ("madera_b", [0.37, 0.22, 0.13], 0.0),
        ("tiza", [0.92, 0.94, 0.90], 0.25),
        ("pizarra", [0.07, 0.20, 0.17], 0.04),
        ("pizarra_b", [0.09, 0.24, 0.20], 0.04),
        ("negro", [0.06, 0.05, 0.10], 0.0),
        ("mazo", [0.92, 0.26, 0.56], 0.08),
        ("mazo_b", [1.0, 0.82, 0.30], 0.18),
        ("birrete", [0.09, 0.08, 0.16], 0.02),
    ];
    for (nombre, kd, brillo) in COLORES {
        o.material(nombre, kd, kd.map(|c| c * brillo), 0.0, 1.0);
    }
    for (nombre, kd) in NEON {
        // El color va en lo que brilla, no en lo que refleja: si no, con la luz se lava a blanco.
        let (refleja, brillo) = if nombre == "n0" { (0.8, 0.2) } else { (0.22, 0.62) };
        o.material(nombre, kd.map(|c| c * refleja), kd.map(|c| c * brillo), 0.0, 1.0);
    }
    // El titulo: brilla mas que todo lo demas, y el centro de cada bloque, casi blanco.
    for (nombre, c) in [("oro_a", [1.0f32, 0.86, 0.22]), ("oro_b", [1.0, 0.46, 0.08]), ("rosa_a", [1.0, 0.5, 0.82]), ("rosa_b", [0.92, 0.12, 0.6]), ("cian_a", [0.55, 0.95, 1.0]), ("cian_b", [0.1, 0.55, 1.0]), ("blanco_a", [1.0, 0.97, 0.9])] {
        o.material(nombre, c.map(|x| x * 0.25), c.map(|x| x * 0.92), 0.0, 1.0);
    }
    o.material("contorno", [0.03, 0.01, 0.08], [0.0, 0.0, 0.0], 0.0, 1.0);
    o.material("tablero_letrero", [0.10, 0.05, 0.22], [0.035, 0.012, 0.075], 0.0, 1.0);
    o.material("nucleo", [0.6, 0.6, 0.6], [1.0, 0.97, 0.9], 0.0, 1.0);
    o.material("foco", [0.6, 0.5, 0.3], [1.0, 0.82, 0.45], 0.0, 1.0);
    o.material("suelo", [0.20, 0.13, 0.38], [0.012, 0.006, 0.03], 0.0, 1.0);
    o.material("suelo_b", [0.27, 0.17, 0.48], [0.016, 0.008, 0.04], 0.0, 1.0);
    o.material("cristal", [0.55, 0.45, 0.85], [0.08, 0.05, 0.16], 0.0, 1.0);
    o.material("vidrio", [0.75, 0.9, 1.0], [0.3, 0.42, 0.5], 0.0, 1.0);
}

fn aro(mat: &str, grosor: f32, alto: f32) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let (a, b) = (0.74, 0.74 - grosor);
    o.caja(mat, v(-a, 0.0, -a), v(a, alto, -b));
    o.caja(mat, v(-a, 0.0, b), v(a, alto, a));
    o.caja(mat, v(-a, 0.0, -b), v(-b, alto, b));
    o.caja(mat, v(b, 0.0, -b), v(a, alto, b));
    o.modelo()
}

fn copo_suelto() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let p = copo_de_koch(2);
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        // Un abanico desde el centro, por los dos lados.
        o.quad("vidrio", [v(0.0, 0.0, 0.0), v(a[0], a[1], 0.0), v(b[0], b[1], 0.0), v(0.0, 0.0, 0.0)], v(0.0, 0.0, 1.0));
        o.quad("vidrio", [v(0.0, 0.0, 0.0), v(b[0], b[1], 0.0), v(a[0], a[1], 0.0), v(0.0, 0.0, 0.0)], v(0.0, 0.0, -1.0));
    }
    o.modelo()
}

/// El gabinete: la tapa con sus losetas y sus nueve hoyos, los lados y las tiras de neon.
fn mueble() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let (x0, x1, z0, z1) = (-3.35, 3.35, -2.95, 2.75);
    o.caja("tablero", v(x0, -3.2, z0), v(x1, 0.0, z1));
    // El borde levantado.
    o.caja("lado", v(x0 - 0.3, -3.2, z0 - 0.3), v(x0, 0.24, z1 + 0.3));
    o.caja("lado", v(x1, -3.2, z0 - 0.3), v(x1 + 0.3, 0.24, z1 + 0.3));
    o.caja("lado", v(x0, -3.2, z0 - 0.3), v(x1, 0.24, z0));
    o.caja("lado", v(x0, -3.2, z1), v(x1, 0.24, z1 + 0.3));
    // Las tiras de neon: por encima del borde y bajando por el frente.
    o.caja("n6", v(x0 - 0.3, 0.24, z1 + 0.08), v(x1 + 0.3, 0.29, z1 + 0.22));
    o.caja("n5", v(x0 - 0.22, 0.24, z0 - 0.3), v(x0 - 0.08, 0.29, z1 + 0.3));
    o.caja("n5", v(x1 + 0.08, 0.24, z0 - 0.3), v(x1 + 0.22, 0.29, z1 + 0.3));
    for k in 0..4 {
        let y = -0.45 - k as f32 * 0.55;
        o.caja(if k % 2 == 0 { "n1" } else { "n6" }, v(x0 - 0.3, y - 0.05, z1 + 0.3), v(x1 + 0.3, y + 0.05, z1 + 0.33));
    }
    // Un tablero de ajedrez con losetas, debajo de los hoyos.
    for i in 0..15 {
        for j in 0..13 {
            if (i + j) % 2 == 0 {
                let (dx, dz) = ((x1 - x0) / 15.0, (z1 - z0) / 13.0);
                let x = x0 + i as f32 * dx;
                let z = z0 + j as f32 * dz;
                o.caja("loseta", v(x, 0.0, z), v(x + dx, 0.012, z + dz));
            }
        }
    }
    for k in 0..HOYOS {
        let c = hoyo(k);
        // El fondo oscuro: por aqui sale. Tapa el cuerpo que esta "adentro".
        o.caja("hoyo_b", v(c.x - 0.74, 0.0, c.z - 0.74), v(c.x + 0.74, 0.02, c.z + 0.74));
        o.caja("hoyo", v(c.x - 0.6, 0.0, c.z - 0.6), v(c.x + 0.6, 0.03, c.z + 0.6));
    }
    o.modelo()
}

/// Las letras del titulo: (letra, x, y de abajo, lado de cada bloque). Cada una se arma por aparte
/// para que pueda brincar y cambiar de color sola.
const TITULO: [(char, f32, f32, f32); 9] = {
    const ARRIBA: f32 = 2.56;
    const U1: f32 = 0.2;
    const X1: f32 = -3.5;
    const ABAJO: f32 = 0.26;
    const U2: f32 = 0.31;
    const X2: f32 = -3.56;
    [
        ('W', X1, ARRIBA, U1),
        ('A', X1 + 6.0 * U1, ARRIBA, U1),
        ('C', X1 + 12.0 * U1, ARRIBA, U1),
        ('K', X1 + 18.0 * U1, ARRIBA, U1),
        ('A', X1 + 30.0 * U1, ARRIBA, U1),
        ('K', X2, ABAJO, U2),
        ('O', X2 + 6.0 * U2, ABAJO, U2),
        ('C', X2 + 12.0 * U2, ABAJO, U2),
        ('H', X2 + 18.0 * U2, ABAJO, U2),
    ]
};
const Z_TITULO: f32 = -4.0;
/// Lo que engordan los bloques (las letras de arcade son gorditas).
const GORDO: f32 = 0.2;

/// Una letra del titulo: dos tonos, como los logos de arcade, y un filo blanco arriba. El origen
/// queda abajo y al centro de la letra.
pub fn letra(c: char, u: f32, tonos: (&str, &str)) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let g = ui::glifo_bloques(c);
    let medio = u * 2.5;
    let gordo = u * GORDO;
    for (fila, bits) in g.iter().enumerate() {
        for col in 0..5 {
            if bits & (16 >> col) != 0 {
                let x = col as f32 * u - medio;
                let y = (6 - fila) as f32 * u;
                let mat = if fila < 4 { tonos.0 } else { tonos.1 };
                o.caja(mat, v(x - gordo, y - gordo, -u * 0.36), v(x + u + gordo, y + u + gordo, u * 0.36));
            }
        }
    }
    o.modelo()
}

fn letrero_fondo() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    for &(c, x, y, u) in &TITULO {
        let g = ui::glifo_bloques(c);
        for (fila, bits) in g.iter().enumerate() {
            for col in 0..5 {
                if bits & (16 >> col) != 0 {
                    let (cx, cy) = (x + col as f32 * u, y + (6 - fila) as f32 * u);
                    // El contorno oscuro: despega la letra del fondo, como en los logos de arcade.
                    let m = u * 0.3;
                    o.caja("contorno", v(cx - m, cy - m * 1.5, Z_TITULO - u * 0.9), v(cx + u + m, cy + u + m, Z_TITULO - u * 0.3));
                }
            }
        }
    }
    // La estructura de atras, de metal: dos postes y sus travesanos.
    let z = Z_TITULO - 0.75;
    for x in [-4.1f32, 4.1] {
        o.caja("negro", v(x - 0.1, -3.2, z - 0.1), v(x + 0.1, 3.95, z + 0.1));
    }
    // El tablero oscuro de atras, escalonado en las esquinas: hace que las letras revienten de color.
    o.caja("tablero_letrero", v(-4.05, 0.05, z - 0.14), v(4.05, 3.9, z));
    o.caja("tablero_letrero", v(-3.85, 3.9, z - 0.14), v(3.85, 4.03, z));
    o.modelo()
}

fn foquitos(par: bool) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let z = Z_TITULO - 0.6;
    let mut k = 0;
    let mut x = -4.0;
    while x < 4.0 {
        if (k % 2 == 0) == par {
            o.caja("foco", v(x, 3.97, z), v(x + 0.13, 4.1, z + 0.13));
            o.caja("foco", v(x, 0.02, z + 0.3), v(x + 0.13, 0.15, z + 0.43));
        }
        k += 1;
        x += 0.32;
    }
    let mut y = 0.2;
    while y < 3.9 {
        if (k % 2 == 0) == par {
            o.caja("foco", v(-4.1, y, z), v(-3.97, y + 0.13, z + 0.13));
            o.caja("foco", v(3.97, y, z), v(4.1, y + 0.13, z + 0.13));
        }
        k += 1;
        y += 0.32;
    }
    o.modelo()
}

/// Un brillo: una estrella de cuatro puntas que aparece y se apaga sobre las letras.
pub fn brillo() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let (l, a) = (1.0, 0.09);
    o.quad("nucleo", [v(0.0, l, 0.0), v(a, 0.0, 0.0), v(0.0, -l, 0.0), v(-a, 0.0, 0.0)], v(0.0, 0.0, 1.0));
    o.quad("nucleo", [v(-l, 0.0, 0.0), v(0.0, -a, 0.0), v(l, 0.0, 0.0), v(0.0, a, 0.0)], v(0.0, 0.0, 1.0));
    o.modelo()
}

/// El mango: desde la muneca (el origen) hacia -Z.
fn mazo_mango() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("madera", v(-0.075, -0.075, -1.45), v(0.075, 0.075, 0.0));
    o.caja("madera_b", v(-0.095, -0.095, -0.34), v(0.095, 0.095, 0.0));
    o.caja("mazo_b", v(-0.1, -0.1, -1.42), v(0.1, 0.1, -1.32));
    o.modelo()
}

fn cabeza_mazo(cuerpo: &str, bandas: &str) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja(cuerpo, v(-0.62, -0.34, -0.36), v(0.62, 0.34, 0.36));
    o.caja(bandas, v(-0.68, -0.38, -0.4), v(-0.47, 0.38, 0.4));
    o.caja(bandas, v(0.47, -0.38, -0.4), v(0.68, 0.38, 0.4));
    o.caja("nucleo", v(-0.38, 0.34, -0.2), v(0.38, 0.365, -0.08));
    o.modelo()
}

fn estela_mazo() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    o.material("humo_mazo", [0.9, 0.5, 0.8], [0.8, 0.4, 0.7], 0.0, 1.0);
    o.transparente("humo_mazo", 0.3);
    o.caja("humo_mazo", v(-0.62, -0.34, -0.36), v(0.62, 0.34, 0.36));
    o.modelo()
}

/// La reglita de los mini Koch.
fn regla() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("mazo_b", v(-0.025, 0.0, -0.05), v(0.025, 0.46, 0.05));
    for k in 0..5 {
        o.caja("madera_b", v(-0.03, 0.05 + k as f32 * 0.08, -0.05), v(0.03, 0.065 + k as f32 * 0.08, 0.0));
    }
    o.modelo()
}

fn birrete() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    // A la medida de la cabezota de los mini Koch.
    o.caja("birrete", v(-0.3, -0.1, -0.3), v(0.3, 0.05, 0.3));
    o.caja("birrete", v(-0.46, 0.05, -0.46), v(0.46, 0.09, 0.46));
    o.caja("n3", v(-0.04, 0.09, -0.04), v(0.04, 0.115, 0.04));
    o.caja("n3", v(0.0, 0.09, -0.012), v(0.42, 0.105, 0.012));
    o.caja("n3", v(0.4, -0.16, -0.025), v(0.45, 0.09, 0.025));
    o.modelo()
}

fn lentes() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("negro", v(-0.26, -0.08, -0.02), v(-0.03, 0.08, 0.02));
    o.caja("negro", v(0.03, -0.08, -0.02), v(0.26, 0.08, 0.02));
    o.caja("negro", v(-0.03, 0.0, -0.015), v(0.03, 0.03, 0.015));
    o.caja("vidrio", v(-0.22, -0.05, 0.02), v(-0.07, 0.05, 0.03));
    o.caja("vidrio", v(0.07, -0.05, 0.02), v(0.22, 0.05, 0.03));
    o.modelo()
}

/// Un cubo con cubos pegados en sus caras, y esos con los suyos: la superficie cuadratica de Koch.
fn cubo_fractal(o: &mut Obra, c: Vec3, r: f32, nivel: u32, venia: Option<Vec3>) {
    let mat = if nivel == 0 { "n6" } else if nivel == 1 { "suelo_b" } else { "cristal" };
    o.caja(mat, c - Vec3::new(r, r, r), c + Vec3::new(r, r, r));
    if nivel == 0 {
        return;
    }
    let dirs = [v(1.0, 0.0, 0.0), v(-1.0, 0.0, 0.0), v(0.0, 1.0, 0.0), v(0.0, 0.0, 1.0), v(0.0, 0.0, -1.0)];
    for d in dirs {
        if venia.is_some_and(|w| (w + d).length() < 0.01) {
            continue;
        }
        let hijo = r * 0.42;
        cubo_fractal(o, c + d * (r + hijo), hijo, nivel - 1, Some(d));
    }
}

fn torres() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    for &(x, y, z, r) in &[(-10.5, -1.0, -6.5, 1.25f32), (10.5, -1.0, -6.5, 1.25), (-14.5, -1.2, -1.0, 1.0), (14.5, -1.2, -1.0, 1.0), (-7.2, -1.9, -10.5, 1.4), (7.2, -1.9, -10.5, 1.4), (0.0, -1.6, -13.0, 1.5)] {
        cubo_fractal(&mut o, v(x, y, z), r, 2, None);
    }
    o.modelo()
}

/// El suelo del planeta: losas lilas que se pierden en el horizonte.
fn suelo() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    for i in -8..8 {
        for j in -7..3 {
            let (x, z) = (i as f32 * 5.0, j as f32 * 5.0);
            let mat = if (i + j) % 2 == 0 { "suelo" } else { "suelo_b" };
            let alto = -3.2 + (((i * 7 + j * 13) % 5) as f32).abs() * 0.05;
            o.caja(mat, v(x, -4.0, z), v(x + 5.0, alto, z + 5.0));
        }
    }
    o.modelo()
}

fn cubo(mat: &str) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja(mat, v(-0.5, -0.5, -0.5), v(0.5, 0.5, 0.5));
    o.modelo()
}

pub fn modelos() -> Modelos {
    Modelos {
        mueble: mueble(),
        letras: TITULO.iter().map(|&(c, _, _, u)| [("oro_a", "oro_b"), ("rosa_a", "rosa_b"), ("cian_a", "cian_b"), ("blanco_a", "blanco_a")].map(|t| letra(c, u, t))).collect(),
        letrero_fondo: letrero_fondo(),
        foquitos: [foquitos(false), foquitos(true)],
        brillo: brillo(),
        aros: NEON.map(|(m, _)| aro(m, 0.13, 0.15)),
        aros_finos: NEON.map(|(m, _)| aro(m, 0.035, 0.02)),
        mazo_mango: mazo_mango(),
        mazo_cabeza: cabeza_mazo("mazo", "mazo_b"),
        mazo_estela: estela_mazo(),
        regla: regla(),
        // 0 tiza, 1 lila, 2 rojo, 3 dorado, 4 verde, 5 celeste, 6 rosa.
        cubos: [cubo("tiza"), cubo("n1"), cubo("n2"), cubo("n3"), cubo("n4"), cubo("n5"), cubo("n6")],
        birrete: birrete(),
        lentes: lentes(),
        copo_suelto: copo_suelto(),
        torres: torres(),
        suelo: suelo(),
    }
}

/// El modelo del profe con otro tinte (el dorado, el enojado, el fractal). `brillo` lo hace
/// brillar solo; todo blanco es el destello del golpe.
pub fn tenido(base: &Modelo, tinte: [f32; 3], mezcla: f32, brillo: f32) -> Modelo {
    let mut m = base.clone();
    for mat in &mut m.materiales {
        let k = mat.kd;
        let c = Vec3::new(k.x + (tinte[0] - k.x) * mezcla, k.y + (tinte[1] - k.y) * mezcla, k.z + (tinte[2] - k.z) * mezcla);
        mat.kd = c;
        mat.ke = mat.ke + c * brillo;
    }
    m
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Aprieta y deja que el mazo llegue.
    fn pegar(k: &mut Koch, hoyo: usize) {
        k.actualizar(&Entrada { celda: Some(hoyo), ..Default::default() }, 1.0 / 60.0);
        let nada = Entrada::default();
        while k.giro.is_some() {
            k.actualizar(&nada, 1.0 / 60.0);
        }
    }

    fn jugar_solo(semilla: u32, segundos: f32) -> Koch {
        let mut k = Koch::new(semilla, 0);
        let dt = 1.0 / 60.0;
        let mut t = 0.0;
        while t < segundos && k.fase != Fase::Resultado {
            let e = k.chofer();
            k.actualizar(&e, dt);
            k.eventos.clear();
            t += dt;
        }
        k
    }

    /// Como una persona: pega al azar, a veces atina, a veces al aire, a veces se queda quieta. Ni un
    /// parcial ni el final pueden quedarse sin nada que hacer, ni dejar un hoyo tapado para siempre.
    #[test]
    fn nunca_se_traba_jugando_como_una_persona() {
        for semilla in 1..600u32 {
            let parcial = semilla as usize % 4;
            let mut k = Koch::new(semilla, 0);
            k.arrancar(parcial);
            let mut azar = semilla.wrapping_mul(2654435761) | 1;
            let mut dado = move || {
                azar = azar.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (azar >> 8) as f32 / 16_777_216.0
            };
            // De muy mala punteria a muy buena, y de pocos golpes a aporrear.
            let punteria = (semilla % 10) as f32 / 10.0;
            let pega = 0.02 + (semilla % 7) as f32 * 0.03;
            let mut quieto = 0.0f32;
            let mut t = 0.0f32;
            while k.fase == Fase::Jugando && t < 60.0 {
                let mut e = Entrada::default();
                if dado() < pega {
                    let afuera: Vec<usize> = (0..HOYOS).filter(|&h| k.hoyos[h].is_some()).collect();
                    let al_hoyo = if !afuera.is_empty() && dado() < punteria { afuera[(dado() * afuera.len() as f32) as usize % afuera.len()] } else { (dado() * 9.0) as usize % 9 };
                    e.celda = Some(al_hoyo);
                }
                k.actualizar(&e, 1.0 / 60.0);
                k.eventos.clear();
                t += 1.0 / 60.0;
                for (h, a) in k.hoyos.iter().enumerate() {
                    if let Some(a) = a {
                        assert!(!a.atrapado || k.giro.is_some_and(|g| g.0 == h) || k.pausa > 0.0, "semilla {semilla}: el hoyo {h} quedo atrapado sin mazo en camino, a los {t} s");
                    }
                }
                // Si no hay nadie afuera ni hay mazo en camino, tiene que salir alguien pronto.
                let hay_algo = k.hoyos.iter().flatten().any(|a| !a.atrapado) || k.giro.is_some();
                quieto = if hay_algo { 0.0 } else { quieto + 1.0 / 60.0 };
                assert!(quieto < 3.0, "semilla {semilla}: {quieto} s sin nada, a los {t} s");
            }
            assert_ne!(k.fase, Fase::Jugando, "semilla {semilla}, parcial {parcial}: no termino en {t} s (reloj {})", k.reloj);
        }
    }

    /// El caso exacto del trabado: pegarle justo cuando se esta escondiendo.
    #[test]
    fn pegarle_al_que_se_esconde_no_tapa_el_hoyo() {
        let mut k = Koch::new(5, 0);
        k.arrancar(FINAL);
        k.hoyos[4] = Some(Asomado { estado: Estado::Baja, t: 0.03, ..Asomado::nuevo(Tipo::Normal, 1.0) });
        pegar(&mut k, 4);
        assert_eq!(k.aciertos, 1, "todavia se le alcanza a pegar, aunque el mazo tarde en llegar");
        for _ in 0..120 {
            k.actualizar(&Entrada::default(), 1.0 / 60.0);
        }
        assert!(k.hoyos[4].as_ref().is_none_or(|a| a.tipo == Tipo::Fractal(2)), "el hoyo del centro se libera para el Koch grande");
    }

    #[test]
    fn el_copo_multiplica_sus_lados_por_cuatro() {
        for n in 0..4 {
            assert_eq!(copo_de_koch(n).len(), 3 * 4usize.pow(n as u32));
        }
        // El perimetro crece 4/3 en cada iteracion, y los puntos salen hacia afuera del triangulo.
        let perimetro = |n: usize| {
            let p = copo_de_koch(n);
            (0..p.len()).map(|i| {
                let (a, b) = (p[i], p[(i + 1) % p.len()]);
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
            }).sum::<f32>()
        };
        assert!((perimetro(2) / perimetro(1) - 4.0 / 3.0).abs() < 0.001);
        let p1 = copo_de_koch(1);
        let lejos = p1.iter().map(|p| (p[0] * p[0] + p[1] * p[1]).sqrt()).fold(0.0f32, f32::max);
        let cerca = p1.iter().map(|p| (p[0] * p[0] + p[1] * p[1]).sqrt()).fold(9.0f32, f32::min);
        assert!((lejos - 1.0).abs() < 0.001, "las puntas siguen en radio 1: {lejos}");
        assert!(cerca > 0.5, "los picos deben salir hacia afuera, no hacia adentro: {cerca}");
    }

    #[test]
    fn jugando_solo_llega_al_caso_base() {
        for s in [3, 77, 1234] {
            let k = jugar_solo(s, 240.0);
            assert_eq!(k.fase, Fase::Resultado, "semilla {s}: no termino");
            assert!(k.caso_base, "semilla {s}: no gano el final (reglazos {}, parcial {})", k.reglazos, k.parcial);
            assert_eq!(k.a_companeros, 0, "el chofer no le pega a sus companeros");
            assert!(k.nota >= 61 && k.nota <= 100, "nota {}", k.nota);
            assert!(k.recursiones >= 1, "semilla {s}: un buen jugador debe llegar a la recursion");
            assert!(k.puntos > META_PUNTOS / 2, "puntos {} muy pocos para la meta", k.puntos);
        }
    }

    #[test]
    fn sin_jugar_se_escapan_y_los_enojados_reprueban() {
        let mut k = Koch::new(9, 0);
        let nada = Entrada::default();
        k.arrancar(1);
        let mut t = 0.0;
        while t < 90.0 && k.fase == Fase::Jugando {
            k.actualizar(&nada, 1.0 / 60.0);
            t += 1.0 / 60.0;
        }
        // (Solo el enojado da reglazo: al que no hace nada lo reprueba, o se le acaba el parcial.)
        assert!(k.escapes > 0);
        assert_eq!(k.aciertos, 0);
        assert_eq!(k.combo, 0);
    }

    #[test]
    fn los_parciales_se_ponen_mas_dificiles() {
        for par in PARCIALES.windows(2) {
            assert!(par[1].arriba.0 <= par[0].arriba.0 && par[1].arriba.1 <= par[0].arriba.1, "duran menos afuera");
            assert!(par[1].paso.1 <= par[0].paso.1, "salen mas seguido");
            assert!(par[1].cupo.1 >= par[0].cupo.1, "salen mas a la vez");
        }
        // Y ninguno sale tan rapido que no le de tiempo de reaccionar.
        for p in &PARCIALES {
            assert!(p.arriba.1 >= 0.65, "{}: menos de 650 ms no es reaccionar, es adivinar", p.titulo);
        }
    }

    #[test]
    fn pegar_cae_al_instante_y_congela() {
        let mut k = Koch::new(5, 0);
        k.arrancar(0);
        k.hoyos[2] = Some(Asomado::nuevo(Tipo::Normal, 1.0));
        // Lo que se agarro al apretar ya no se escapa, aunque el mazo tarde en llegar.
        k.hoyos[2].as_mut().unwrap().dura = 0.03;
        k.actualizar(&Entrada { celda: Some(2), ..Default::default() }, 1.0 / 60.0);
        assert!(k.hoyos[2].unwrap().atrapado, "se apreto: ya es suyo");
        assert_eq!(k.aciertos, 0, "el golpe se siente hasta que el mazo llega");
        let nada = Entrada::default();
        let mut cuadros = 1;
        while k.giro.is_some() {
            k.actualizar(&nada, 1.0 / 60.0);
            cuadros += 1;
        }
        assert!(cuadros <= 7, "el giro tarda {cuadros} cuadros: se siente lento");
        assert_eq!(k.aciertos, 1);
        assert_eq!(k.perfectos, 1, "recien salido es perfecto");
        assert!(k.pausa > 0.03 && k.pausa <= 0.1, "congelada de {} s", k.pausa);
        assert!(k.eventos.contains(&Efecto::Mazazo));
        assert!(k.eventos.contains(&Efecto::Aire), "el soplido del giro");
        assert!(k.eventos.iter().any(|e| matches!(e, Efecto::Wack | Efecto::Wack2 | Efecto::Wack3)), "¡WACK!");
        assert_eq!(k.puntos, 150);
    }

    #[test]
    fn el_tono_sube_con_el_combo() {
        assert_eq!(nota_del_combo(1) as usize, Efecto::Nota0 as usize);
        assert_eq!(nota_del_combo(4) as usize, Efecto::Nota3 as usize);
        assert_eq!(nota_del_combo(8) as usize, Efecto::Nota7 as usize);
        assert!(nota_del_combo(50) as usize >= Efecto::Nota6 as usize);
    }

    #[test]
    fn al_aire_corta_el_combo_y_traba_el_mazo() {
        let mut k = Koch::new(5, 0);
        k.arrancar(0);
        k.combo = 10;
        pegar(&mut k, 0);
        assert_eq!(k.combo, 5);
        assert_eq!(k.al_aire, 1);
        assert!(k.enfriar > 0.1);
    }

    #[test]
    fn a_su_companero_no() {
        let mut k = Koch::new(5, 0);
        k.arrancar(1);
        k.combo = 10;
        k.hoyos[3] = Some(Asomado::nuevo(Tipo::Companero(1), 1.0));
        pegar(&mut k, 3);
        assert_eq!(k.reglazos, 1);
        assert_eq!(k.combo, 0);
        assert_eq!(k.a_companeros, 1);
        // Dejarlo pasar no cuesta nada.
        let mut k = Koch::new(5, 0);
        k.arrancar(1);
        k.hoyos[3] = Some(Asomado::nuevo(Tipo::Companero(1), 0.1));
        k.escapo(3, Tipo::Companero(1));
        assert_eq!(k.escapes, 0);
    }

    #[test]
    fn el_birrete_aguanta_dos() {
        let mut k = Koch::new(5, 0);
        k.arrancar(1);
        k.hoyos[0] = Some(Asomado::nuevo(Tipo::Birrete, 1.0));
        pegar(&mut k, 0);
        assert_eq!(k.aciertos, 0);
        assert!(!k.hoyos[0].unwrap().con_birrete, "se le cae al primero");
        assert_eq!(k.vuelan.iter().filter(|v| v.cual == 1).count(), 1);
        k.pausa = 0.0;
        k.enfriar = 0.0;
        pegar(&mut k, 0);
        assert_eq!(k.aciertos, 1);
    }

    #[test]
    fn el_enojado_da_reglazo_si_se_escapa() {
        let mut k = Koch::new(5, 0);
        k.arrancar(0);
        k.hoyos[4] = Some(Asomado::nuevo(Tipo::Enojado, 0.3));
        let nada = Entrada::default();
        for _ in 0..60 {
            k.actualizar(&nada, 1.0 / 60.0);
        }
        assert_eq!(k.reglazos, 1);
    }

    #[test]
    fn tres_reglazos_y_reprueba() {
        let mut k = Koch::new(5, 0);
        k.arrancar(1);
        for _ in 0..3 {
            k.dar_reglazo(0);
        }
        assert_eq!(k.fase, Fase::Resultado);
        assert!(!k.caso_base);
        assert_eq!(k.partidas, 1);
    }

    #[test]
    fn en_la_recursion_no_hay_reglazos_y_vale_el_triple() {
        let mut k = Koch::new(5, 0);
        k.arrancar(2);
        k.sumar_recursion(2.0);
        assert!(k.en_recursion());
        assert_eq!(k.multiplicador(), 3);
        k.dar_reglazo(0);
        assert_eq!(k.reglazos, 0);
    }

    #[test]
    fn el_fractal_se_parte_hasta_el_caso_base() {
        let mut k = Koch::new(5, 0);
        k.arrancar(FINAL);
        k.romper_fractal(4, 2);
        assert_eq!(k.jefe.as_ref().unwrap().pendientes, vec![1, 1, 1]);
        k.romper_fractal(0, 1);
        assert_eq!(k.jefe.as_ref().unwrap().pendientes.len(), 6, "de un mediano salen tres chicos");
        k.romper_fractal(0, 0);
        assert_eq!(k.jefe.as_ref().unwrap().pendientes.len(), 6, "el chico es el caso base");
    }

    #[test]
    fn la_nota_premia_la_punteria() {
        assert_eq!(calcular_nota(0, 0, 0, false, 0), 0);
        assert_eq!(calcular_nota(100, 0, META_PUNTOS, true, 0), 100);
        assert!(calcular_nota(100, 0, 1000, false, 0) > calcular_nota(60, 40, 1000, false, 0));
        assert!(calcular_nota(100, 0, 1000, false, 2) < calcular_nota(100, 0, 1000, false, 0));
    }

    #[test]
    fn el_control_apunta_directo() {
        let mut k = Koch::new(5, 0);
        k.arrancar(0);
        k.stick_quieto = 9.0;
        let casos = [((-1.0, 1.0), 0), ((0.0, 1.0), 1), ((1.0, 1.0), 2), ((-1.0, 0.0), 3), ((0.0, 0.0), 4), ((1.0, 0.0), 5), ((-1.0, -1.0), 6), ((0.0, -1.0), 7), ((1.0, -1.0), 8)];
        for ((x, y), hoyo) in casos {
            k.stick_quieto = 9.0;
            k.leer_golpe(&Entrada { horizontal: x, cabecear: y, hay_control: true, ..Default::default() });
            assert_eq!(k.cursor, hoyo, "stick {x},{y}");
        }
    }

    /// Empujar y apretar casi juntos: aunque el stick ya regreso, cuenta para donde se empujo.
    #[test]
    fn el_hoyo_se_recuerda_un_momentito_al_soltar_el_stick() {
        let mut k = Koch::new(5, 0);
        k.arrancar(0);
        let suelto = Entrada { hay_control: true, ..Default::default() };
        k.actualizar(&Entrada { horizontal: 1.0, hay_control: true, ..Default::default() }, 1.0 / 60.0);
        assert_eq!(k.cursor, 5);
        k.actualizar(&suelto, 1.0 / 60.0);
        assert_eq!(k.cursor, 5, "recien soltado sigue ahi");
        for _ in 0..12 {
            k.actualizar(&suelto, 1.0 / 60.0);
        }
        assert_eq!(k.cursor, 4, "ya regreso al centro");
    }

    #[test]
    fn en_el_control_pega_cualquier_boton() {
        let base = || Entrada { hay_control: true, control: crate::entrada::Control::PlayStation, ..Default::default() };
        let botones: [(&str, Entrada); 7] = [
            ("cruz", Entrada { aceptar: true, ..base() }),
            ("circulo", Entrada { orbitar: true, ..base() }),
            ("triangulo", Entrada { barrel: true, ..base() }),
            ("cuadrado", Entrada { aterrizar: true, ..base() }),
            ("L1", Entrada { pestana_izq: true, ..base() }),
            ("R1", Entrada { pestana_der: true, ..base() }),
            ("R2", Entrada { gatillo: true, ..base() }),
        ];
        for (nombre, e) in botones {
            let mut k = Koch::new(5, 0);
            k.arrancar(0);
            k.hoyos[4] = Some(Asomado::nuevo(Tipo::Normal, 1.0));
            assert_eq!(k.leer_golpe(&e), Some(4), "{nombre} no pega");
        }
    }

    #[test]
    fn mantener_el_gatillo_no_pega_cada_cuadro() {
        let mut k = Koch::new(5, 0);
        k.arrancar(0);
        let r2 = Entrada { gatillo: true, hay_control: true, ..Default::default() };
        assert!(k.leer_golpe(&r2).is_some());
        assert!(k.leer_golpe(&r2).is_none(), "mantenerlo no repite");
        k.leer_golpe(&Entrada { hay_control: true, ..Default::default() });
        assert!(k.leer_golpe(&r2).is_some(), "soltarlo y apretar otra vez si");
    }

    /// Con control, el ultimo golpe suele ser con cuadrado: no puede despegar la nave de una vez.
    #[test]
    fn aporrear_al_terminar_no_despega_ni_salta_la_nota() {
        let mut k = Koch::new(5, 0);
        k.arrancar(1);
        for _ in 0..3 {
            k.dar_reglazo(0);
        }
        let aporrea = Entrada { aceptar: true, aterrizar: true, hay_control: true, ..Default::default() };
        for _ in 0..80 {
            assert!(matches!(k.actualizar(&aporrea, 1.0 / 60.0), AccionKoch::Nada), "despego a los {} s", k.t);
            assert_eq!(k.fase, Fase::Resultado);
        }
    }

    /// Lo ultimo que suena es lo que vibra: el mazazo tiene que ser lo ultimo del golpe.
    #[test]
    fn el_golpe_se_siente_en_las_manos() {
        let mut k = Koch::new(5, 0);
        k.arrancar(0);
        k.hoyos[2] = Some(Asomado::nuevo(Tipo::Normal, 1.0));
        pegar(&mut k, 2);
        let ultima = k.eventos.iter().rev().find_map(|&e| crate::vibracion::de_efecto(e));
        assert_eq!(ultima, crate::vibracion::de_efecto(Efecto::Mazazo));
        assert!(crate::vibracion::de_efecto(Efecto::Mazazo).is_some_and(|(grave, _, _)| grave > 0.5));
        assert!(crate::vibracion::de_efecto(Efecto::Reglazo).is_some());
        assert!(crate::vibracion::de_efecto(Efecto::Recursion).is_some());
    }

    /// Todo lo que sale en pantalla para el control tiene nombre de boton de verdad.
    #[test]
    fn cada_control_nombra_sus_botones() {
        use crate::entrada::Control::*;
        for c in [PlayStation, Xbox, Switch, Otro] {
            let e = Entrada { hay_control: true, control: c, ..Default::default() };
            for accion in ["aceptar", "aterrizar", "apuntar"] {
                assert_ne!(e.nombre(accion), "?", "{c:?} no sabe nombrar {accion}");
            }
        }
    }

    #[test]
    fn la_siguiente_partida_empieza_limpia_pero_recuerda_lo_mejor() {
        let mut k = Koch::new(5, 0);
        k.arrancar(1);
        k.puntos = 700;
        k.aciertos = 9;
        for _ in 0..3 {
            k.dar_reglazo(0);
        }
        let nota = k.nota;
        k.reiniciar_partida();
        assert_eq!(k.puntos, 0);
        assert_eq!(k.reglazos, 0);
        assert_eq!(k.partidas, 1);
        assert_eq!(k.mejor_nota, nota);
        assert_eq!(k.fase, Fase::Cartel(0));
    }
}
