//! Saturno: ¡ANILLAZO! Venus le devolvio a Saturno sus anillos despues del divorcio, y se los manda
//! de uno en uno, a toda velocidad. La nave va por adentro de ellos y tiene que meterse por el
//! hueco de cada uno. Si choca, otra vez, en menos de medio segundo.
//!
//! Se hizo pensando primero en quien juega:
//! - **Sin menu ni tutorial.** Se empieza jugando. El primer anillo trae un hueco enorme que obliga
//!   a moverse poquito; una flecha y la linea de mira (roja o verde) ensenan lo demas.
//! - **Solo izquierda y derecha.** La nave siempre va abajo y el mundo gira alrededor: izquierda
//!   siempre es izquierda.
//! - **Chocar no castiga.** La musica no para (se traba como un disco y sigue), se ve donde estaba
//!   el hueco, y cualquier boton reinicia al instante. No hay pantalla de "game over" que esperar.
//! - **Siempre se puede.** Los anillos salen de manera que el siguiente hueco este al alcance, y la
//!   nave choca con menos de lo que se ve (nunca al reves). Una prueba lo juega a todas las velocidades.
//! - **Si cuesta, ayuda (y lo dice).** Tras varias muertes rapidas seguidas, los huecos se abren un
//!   poco y GLADAS avisa. El record no se toca.
//! - **Al volver de la pausa** el tiempo arranca despacio, para no morir por despausar.
//!
//! Y para que sea vicioso: los anillos llegan en la corchea de la cancion, y la cancion se acelera
//! con cada nivel; pasar rozando da puntos y encadena; cada diez segundos cambia algo, y el record
//! esta marcado en la barra para verlo venir.

pub use crate::cancion::CORCHEA_ANILLOS as CORCHEA;
use crate::entrada::Entrada;
use crate::matematica::{Mat4, Vec3};
use crate::obj::Modelo;
use crate::obra::Obra;
use crate::sonido::Efecto;
use crate::ui;
use raylib::prelude::*;
use std::cell::Cell;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub const FOV: f32 = 66.0;
pub const OJO: Vec3 = Vec3::new(0.0, -1.55, 5.1);
pub const MIRA: Vec3 = Vec3::new(0.0, -1.0, -12.0);
/// El radio por donde va la nave, y los radios de los anillos.
pub const RADIO: f32 = 2.75;
const R_ADENTRO: f32 = 2.0;
const R_AFUERA: f32 = 3.6;
/// Radianes por segundo que gira la nave a fondo.
const GIRO: f32 = 5.4;
/// De donde salen los anillos.
const LEJOS: f32 = 95.0;
const SECTORES: usize = 32;
/// La nave choca con menos de lo que se ve: el hueco cuenta este poquito mas ancho.
const PERDON: f32 = 0.07;
/// Pasar a menos de esto de la orilla es un roce.
const ROCE: f32 = 0.2;
/// De todo lo que la nave alcanza a girar entre dos anillos, cuanto se pide como maximo.
const ALCANCE: f32 = 0.72;
const DURA_NIVEL: f32 = 10.0;
/// Cuanto hay que esperar para que un boton reinicie (si no, el boton con el que se venia jugando
/// salta la pantalla sin ver nada).
const ESPERA_REINICIO: f32 = 0.28;
const MUERTE_RAPIDA: f32 = 8.0;

/// Un nivel: la historia de Saturno y Venus.
struct Nivel {
    nombre: &'static str,
    /// Corcheas entre un anillo y otro.
    paso: u32,
    velocidad: f32,
    /// Medio ancho del hueco, en radianes.
    hueco: f32,
    /// A cuanto va la cancion.
    rapidez: f32,
}

const NIVELES: [Nivel; 8] = [
    Nivel { nombre: "NOVIOS", paso: 6, velocidad: 20.0, hueco: 1.15, rapidez: 1.0 },
    Nivel { nombre: "COMPROMETIDOS", paso: 5, velocidad: 23.0, hueco: 1.0, rapidez: 1.04 },
    Nivel { nombre: "CASADOS", paso: 4, velocidad: 26.0, hueco: 0.85, rapidez: 1.08 },
    Nivel { nombre: "PROBLEMAS", paso: 4, velocidad: 29.0, hueco: 0.72, rapidez: 1.12 },
    Nivel { nombre: "SEPARADOS", paso: 3, velocidad: 32.0, hueco: 0.62, rapidez: 1.17 },
    Nivel { nombre: "ABOGADOS", paso: 3, velocidad: 36.0, hueco: 0.55, rapidez: 1.22 },
    Nivel { nombre: "DIVORCIADOS", paso: 2, velocidad: 40.0, hueco: 0.5, rapidez: 1.27 },
    Nivel { nombre: "ANILLOS LIBRES", paso: 2, velocidad: 45.0, hueco: 0.44, rapidez: 1.33 },
];

/// Lo que se anuncia al subir de nivel: la historia, en una linea.
const LINEAS: [&str; 8] = ["Todo es bonito", "Ya hay anillo", "Son dos anillos", "Duerme en el sofá", "Cada quien su órbita", "Todo por escrito", "¡Se los devuelve!", "Ya nadie lo para"];

/// Lo que se grita por cada roce seguido.
const GRITOS: [&str; 8] = ["¡ROCE!", "¡AL RAS!", "¡DE PELOS!", "¡UY UY UY!", "¡QUÉ NERVIOS!", "¡IMPARABLE!", "¡NO SEA ASÍ!", "¡ESTÁ LOCO!"];

/// Y cada tantos anillos.
const ANIMOS: [&str; 6] = ["¡ESO!", "¡VAYA!", "¡SIGA!", "¡QUÉ VIAJE!", "¡ÉPICO!", "¡A TODO!"];

/// Lo que dicen al chocar: cortito, para leerlo de pasada.
const FRASES: [&str; 12] = [
    "Venus: ¿Ya? Sabía que no duraba.",
    "Saturno: Ese anillo era de mi abuela.",
    "Venus: Te lo dije en el juicio.",
    "Saturno: No me toques los anillos.",
    "GLADAS: El hueco no se mueve. Usted sí.",
    "Miku: ¡Otra, otra, otra!",
    "Venus: Firma aquí. Y aquí. Y en ese anillo.",
    "Saturno: Mi abogado va a oír de esto.",
    "GLADAS: Registrado para el acta del divorcio.",
    "Venus: Y eso que era el fácil.",
    "Miku: Ni el coro se dio cuenta. Bueno, sí.",
    "Saturno: Flotaría en una tina. Usted no.",
];

/// Una forma de sacar anillos.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Patron {
    Libre,
    /// De un lado a otro.
    Zigzag(f32),
    /// Dando la vuelta, siempre para el mismo lado.
    Espiral(f32),
    /// Dos huecos: uno de ellos queda cerca.
    Doble,
    /// El mismo hueco, angostito: un respiro para acomodarse.
    Corredor,
    /// El anillo viene girando (al llegar, el hueco queda donde debe).
    Giro(f32),
}

#[derive(Clone, Copy, Debug)]
struct Hueco {
    centro: f32,
    semi: f32,
}

#[derive(Clone, Debug)]
pub struct Anillo {
    /// En que momento del reloj cruza por la nave.
    llegada: f32,
    huecos: [Option<Hueco>; 2],
    giro: f32,
    nivel: usize,
    pasado: bool,
}

impl Anillo {
    fn hueco_cerca(&self, a: f32) -> Option<(Hueco, f32)> {
        self.huecos.iter().flatten().map(|h| (*h, envolver(h.centro - a))).min_by(|x, y| x.1.abs().total_cmp(&y.1.abs()))
    }

    /// Si la nave en el angulo `a` cabe en algun hueco.
    fn cabe(&self, a: f32) -> bool {
        self.hueco_cerca(a).is_some_and(|(h, d)| d.abs() <= h.semi + PERDON)
    }
}

/// Lleva un angulo a (-PI, PI].
pub fn envolver(a: f32) -> f32 {
    let x = (a + PI).rem_euclid(TAU) - PI;
    if x <= -PI { x + TAU } else { x }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fase {
    Jugando,
    Caido,
}

pub enum AccionAnillos {
    Nada,
    Despegar,
}

#[derive(Clone, Copy, Debug)]
pub struct Chispa {
    pub pos: Vec3,
    pub vel: Vec3,
    pub vida: f32,
    pub tam: f32,
    pub color: usize,
}

/// Una palabra que revienta en pantalla, con letras de bloques.
struct Letrero {
    texto: String,
    sub: &'static str,
    /// Tonos de arriba y de abajo.
    tonos: (Color, Color),
    vida: f32,
    dura: f32,
    /// Donde (de 0 a 1) y que tan grande.
    x: f32,
    y: f32,
    u: f32,
}

pub struct Anillazo {
    pub fase: Fase,
    /// Segundos en esta fase.
    pub t: f32,
    pub total: f32,
    /// El reloj de la musica: nunca se reinicia, para que todo siga cayendo en la corchea.
    reloj: f32,
    corchea_vista: i64,
    /// El angulo de la nave, y como se inclina al girar.
    pub angulo: f32,
    pub alabeo: f32,
    velocidad: f32,
    estela: f32,
    pub anillos: Vec<Anillo>,
    proxima_llegada: f32,
    guia: f32,
    patron: (Patron, u32),
    /// Lo que lleva vivo esta partida, en segundos de verdad.
    pub vivo: f32,
    pub nivel: usize,
    pub puntos: u32,
    pub pasados: u32,
    pub roces: u32,
    pub racha: u32,
    pub mejor_racha: u32,
    /// El record guardado y la mejor marca de esta visita, en centesimas de segundo.
    pub record: u32,
    pub rompio_record: bool,
    pub partidas: u32,
    pub mejor_vivo: f32,
    /// Por cuanto se quedo corto y cual lo tumbo.
    pub culpable: Option<Anillo>,
    pub frase: &'static str,
    /// Muertes rapidas seguidas (para la ayuda) y cuanto se abren los huecos.
    muertes_rapidas: u32,
    pub ayuda: u32,
    /// Ya se movio alguna vez: se pueden quitar las instrucciones.
    pub ya_se_movio: bool,
    pub sacudida: f32,
    pub destello: f32,
    pub pulso: f32,
    /// Lo que queda del arranque lento despues de la pausa.
    tras_pausa: f32,
    /// Cuanto queda del disco trabado.
    traba: f32,
    pub chispas: Vec<Chispa>,
    letreros: Vec<Letrero>,
    pub eventos: Vec<Efecto>,
    /// Hay que arrancar la cancion (al entrar: despues ya no se detiene, ni al chocar).
    pub musica_de_nuevo: bool,
    semilla: u32,
    raton_antes: Option<(f32, f32)>,
    raton_x: Option<f32>,
    imagen: Cell<(f32, f32, f32, f32)>,
}

impl Anillazo {
    pub fn new(semilla: u32, record: u32) -> Self {
        let mut a = Anillazo {
            fase: Fase::Jugando,
            t: 0.0,
            total: 0.0,
            reloj: 0.0,
            corchea_vista: -1,
            angulo: 0.0,
            alabeo: 0.0,
            velocidad: NIVELES[0].velocidad,
            estela: 0.0,
            anillos: Vec::new(),
            proxima_llegada: 0.0,
            guia: 0.0,
            patron: (Patron::Libre, 0),
            vivo: 0.0,
            nivel: 0,
            puntos: 0,
            pasados: 0,
            roces: 0,
            racha: 0,
            mejor_racha: 0,
            record,
            rompio_record: false,
            partidas: 0,
            mejor_vivo: 0.0,
            culpable: None,
            frase: FRASES[0],
            muertes_rapidas: 0,
            ayuda: 0,
            ya_se_movio: false,
            sacudida: 0.0,
            destello: 0.0,
            pulso: 0.0,
            tras_pausa: 0.0,
            traba: 0.0,
            chispas: Vec::new(),
            letreros: Vec::new(),
            eventos: Vec::new(),
            musica_de_nuevo: true,
            semilla: semilla | 1,
            raton_antes: None,
            raton_x: None,
            imagen: Cell::new((0.0, 0.0, 0.0, 0.0)),
        };
        a.empezar_partida();
        a
    }

    fn azar(&mut self) -> f32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) as f32 / 16_777_216.0
    }

    /// Otra partida, sin parar la musica: el reloj sigue, y el primer anillo cae en una corchea.
    fn empezar_partida(&mut self) {
        self.fase = Fase::Jugando;
        self.t = 0.0;
        self.vivo = 0.0;
        self.nivel = 0;
        self.velocidad = NIVELES[0].velocidad;
        self.anillos.clear();
        self.culpable = None;
        self.puntos = 0;
        self.pasados = 0;
        self.roces = 0;
        self.racha = 0;
        self.rompio_record = false;
        self.patron = (Patron::Libre, 0);
        self.guia = self.angulo;
        self.letreros.clear();
        // Dos segundos para acomodarse, y que caiga en la corchea.
        self.proxima_llegada = ((self.reloj + 2.1) / CORCHEA).ceil() * CORCHEA;
        self.primer_anillo();
    }

    /// El primero ensena: tiene un hueco enorme, pero hay que moverse tantito para entrar.
    fn primer_anillo(&mut self) {
        let lado = if self.azar() < 0.5 { 1.0 } else { -1.0 };
        let semi = 1.15 * self.apertura();
        // La orilla del hueco queda un poco mas alla de la nave.
        let centro = self.angulo + lado * (semi + PERDON + 0.3);
        self.guia = centro;
        self.anillos.push(Anillo { llegada: self.proxima_llegada, huecos: [Some(Hueco { centro, semi }), None], giro: 0.0, nivel: 0, pasado: false });
        self.proxima_llegada += NIVELES[0].paso as f32 * CORCHEA;
    }

    fn apertura(&self) -> f32 {
        1.0 + self.ayuda as f32 * 0.18
    }

    /// A cuanto va la musica, y el juego con ella.
    pub fn rapidez(&self) -> f32 {
        // Al chocar el disco se traba; al volver de la pausa arranca despacio.
        let base = if self.fase == Fase::Caido { 1.0 - 0.45 * self.traba } else { NIVELES[self.nivel].rapidez };
        base * (1.0 - 0.7 * self.tras_pausa.clamp(0.0, 1.0))
    }

    pub fn jugando(&self) -> bool {
        self.fase == Fase::Jugando
    }

    /// El cielo de cada nivel: se va poniendo mas intenso. Horizonte, medio y arriba.
    pub fn cielo(&self) -> [[f32; 3]; 3] {
        const CIELOS: [[[f32; 3]; 3]; 8] = [
            [[0.44, 0.19, 0.66], [0.2, 0.09, 0.42], [0.05, 0.03, 0.17]],
            [[0.66, 0.16, 0.55], [0.3, 0.08, 0.42], [0.08, 0.03, 0.2]],
            [[0.16, 0.34, 0.72], [0.1, 0.13, 0.46], [0.04, 0.04, 0.2]],
            [[0.75, 0.3, 0.16], [0.36, 0.1, 0.34], [0.08, 0.03, 0.2]],
            [[0.72, 0.13, 0.32], [0.3, 0.06, 0.32], [0.08, 0.02, 0.17]],
            [[0.28, 0.1, 0.66], [0.14, 0.05, 0.42], [0.03, 0.02, 0.17]],
            [[0.8, 0.1, 0.58], [0.34, 0.05, 0.42], [0.07, 0.02, 0.2]],
            [[0.86, 0.5, 0.14], [0.5, 0.12, 0.4], [0.1, 0.03, 0.24]],
        ];
        let c = CIELOS[self.nivel];
        // Late con la musica.
        let k = 1.0 + self.pulso * self.pulso * 0.16;
        c.map(|f| f.map(|x| (x * k).min(1.0)))
    }

    /// El letrero gigante: cada letra brinca y se enciende de una en una, con la musica.
    pub fn titulo(&self) -> Vec<(Mat4, usize)> {
        let reloj = self.reloj / CORCHEA;
        let ola = reloj as usize % (TITULO.len() + 2);
        let cadena = self.racha >= 3;
        TITULO
            .iter()
            .enumerate()
            .map(|(k, _)| {
                let brinca = if ola == k { 1.1 * (1.0 - reloj.fract()) } else { 0.0 };
                let color = if cadena {
                    (reloj as usize + k) % 3
                } else if ola == k {
                    3
                } else if k % 2 == 0 {
                    0
                } else {
                    1
                };
                let crece = 1.0 + self.pulso * self.pulso * 0.03;
                (Mat4::translation(Vec3::new(titulo_x(k), Y_TITULO + brinca, Z_TITULO)) * Mat4::scale(crece), color)
            })
            .collect()
    }

    /// Las estrellitas que se prenden sobre las letras.
    pub fn brillos(&self) -> Vec<(Vec3, f32)> {
        (0..14)
            .filter_map(|k| {
                let s = k as f32 * 2.399;
                let t = self.total * (1.0 + (s * 1.3).sin().abs() * 0.6) + s;
                let vuelta = (t / PI).floor();
                let tam = t.sin().abs().powi(6);
                if tam < 0.05 {
                    return None;
                }
                let letra = ((s * 3.7 + vuelta * 5.0).sin().abs() * 7.99) as usize;
                let (dx, dy) = ((s * 5.1 + vuelta * 3.3).sin(), (s * 7.7 + vuelta * 2.1).cos().abs());
                Some((Vec3::new(titulo_x(letra) + dx * U_TITULO * 2.5, Y_TITULO + dy * U_TITULO * 7.0, Z_TITULO + 2.0), tam * 3.2))
            })
            .collect()
    }

    /// Los aros del tunel giran con el mundo.
    pub fn rieles(&self) -> Mat4 {
        Mat4::rotation_z(self.en_pantalla(0.0))
    }

    pub fn aros_tunel(&self) -> Vec<(Mat4, usize)> {
        (0..9)
            .map(|k| {
                let paso = 12.0;
                let z = ((self.reloj * self.velocidad + k as f32 * paso) % (paso * 9.0)) - paso * 8.0 + 5.0;
                let crece = 1.0 + self.pulso * self.pulso * 0.05;
                (Mat4::translation(Vec3::new(0.0, 0.0, z)) * self.rieles() * Mat4::scale_xyz(Vec3::new(crece, crece, 1.0)), k % 2)
            })
            .collect()
    }

    pub fn nombre_nivel(&self) -> &'static str {
        NIVELES[self.nivel].nombre
    }

    /// La nota de la mejor marca de esta visita.
    pub fn nota(&self) -> &'static str {
        self.nota_de(self.mejor_vivo)
    }

    fn nota_de(&self, segundos: f32) -> &'static str {
        match segundos {
            v if v >= 60.0 => "S",
            v if v >= 40.0 => "A",
            v if v >= 20.0 => "B",
            _ => "C",
        }
    }

    pub fn despedida(&self) -> String {
        if self.partidas == 0 {
            "Despegue de Saturno. Nos fuimos sin esquivar ni un anillo. Venus los sigue mandando.".into()
        } else {
            format!("Despegue de Saturno. Aguantamos {:.1} segundos. Venus dice que su abogado no tanto.", self.mejor_vivo)
        }
    }

    fn proximo(&self) -> Option<&Anillo> {
        self.anillos.iter().filter(|r| !r.pasado).min_by(|x, y| x.llegada.total_cmp(&y.llegada))
    }

    /// Para las capturas, los videos y las pruebas: juega solo, siempre hacia el hueco que toca.
    pub fn chofer(&self) -> Entrada {
        if self.fase == Fase::Caido {
            return Entrada { aceptar: self.t > 0.5, ..Default::default() };
        }
        let Some((_, d)) = self.proximo().and_then(|r| r.hueco_cerca(self.angulo)) else { return Entrada::default() };
        Entrada { horizontal: (d * 7.0).clamp(-1.0, 1.0), ..Default::default() }
    }

    // -----------------------------------------------------------------------------------------
    // El juego
    // -----------------------------------------------------------------------------------------

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionAnillos {
        // En la pausa el juego llega con dt 0: al volver, arranca despacio.
        if dt <= 0.0 {
            self.tras_pausa = 1.0;
            return AccionAnillos::Nada;
        }
        self.total += dt;
        self.t += dt;
        self.tras_pausa = (self.tras_pausa - dt * 1.15).max(0.0);
        self.sacudida = (self.sacudida - dt * 2.4).max(0.0);
        self.destello = (self.destello - dt * 5.0).max(0.0);
        self.pulso = (self.pulso - dt * 3.6).max(0.0);
        self.traba = (self.traba - dt * 1.9).max(0.0);
        self.mover_efectos(dt);
        let rapidez = self.rapidez();
        self.reloj += dt * rapidez;
        let corchea = (self.reloj / CORCHEA) as i64;
        if corchea != self.corchea_vista {
            self.corchea_vista = corchea;
            if corchea % 2 == 0 {
                self.pulso = 1.0;
            }
        }
        let de_verdad = dt * if self.tras_pausa > 0.0 { rapidez / NIVELES[self.nivel].rapidez } else { 1.0 };
        match self.fase {
            Fase::Jugando => self.jugar(e, de_verdad),
            Fase::Caido => {
                self.alabeo *= 1.0 - (dt * 6.0).min(1.0);
                // Cualquier boton reinicia; despegar, aparte y no de una vez.
                let otra = e.aceptar || e.clic || (e.hay_control && (e.barrel || e.orbitar || e.pestana_izq || e.pestana_der));
                if self.t > ESPERA_REINICIO && otra {
                    self.eventos.push(Efecto::KochSale);
                    self.empezar_partida();
                } else if self.t > ESPERA_REINICIO * 1.5 && e.aterrizar {
                    return AccionAnillos::Despegar;
                }
            }
        }
        AccionAnillos::Nada
    }

    /// Cuanto se quiere girar: el raton manda si se mueve (segun que tan lejos del centro esta).
    fn volante(&mut self, e: &Entrada) -> f32 {
        if let Some(p) = e.puntero {
            if self.raton_antes.is_some_and(|a| (a.0 - p.0).abs() + (a.1 - p.1).abs() > 1.0) {
                let (x, _, w, _) = self.imagen.get();
                if w > 0.0 {
                    self.raton_x = Some(((p.0 - (x + w * 0.5)) / (w * 0.14)).clamp(-1.0, 1.0));
                }
            }
            self.raton_antes = Some(p);
        }
        if e.horizontal.abs() > 0.05 {
            self.raton_x = None;
            return e.horizontal;
        }
        self.raton_x.unwrap_or(0.0)
    }

    fn jugar(&mut self, e: &Entrada, dt: f32) {
        self.vivo += dt;
        let volante = self.volante(e);
        if volante.abs() > 0.2 {
            self.ya_se_movio = true;
        }
        self.angulo = envolver(self.angulo + volante * GIRO * dt);
        self.alabeo += (volante * 0.62 - self.alabeo) * (dt * 14.0).min(1.0);
        // El nivel sube por tiempo.
        let nivel = ((self.vivo / DURA_NIVEL) as usize).min(NIVELES.len() - 1);
        if nivel != self.nivel {
            self.nivel = nivel;
            self.destello = self.destello.max(0.3);
            self.pulso = 1.0;
            self.eventos.push(Efecto::Sello);
            self.letreros.retain(|l| l.u < 0.012);
            let texto = format!("¡{}!", NIVELES[nivel].nombre);
            self.letreros.push(Letrero { texto, sub: LINEAS[nivel], tonos: (Color::new(255, 245, 140, 255), Color::new(255, 130, 60, 255)), vida: 1.5, dura: 1.5, x: 0.5, y: 0.42, u: 0.0125 });
        }
        self.velocidad += (NIVELES[self.nivel].velocidad - self.velocidad) * (dt * 1.5).min(1.0);
        // La estela de los motores: con cadena de roces, arcoiris.
        self.estela += dt;
        while self.estela > 0.025 && self.chispas.len() < 110 {
            self.estela -= 0.025;
            let (a, b) = (self.azar() - 0.5, self.azar());
            let color = if self.racha >= 3 { 1 + (self.total * 14.0) as usize % 6 } else { 5 };
            self.chispas.push(Chispa { pos: Vec3::new(a * 0.28, -RADIO + 0.04, 0.5), vel: Vec3::new(a * 1.2, b * 0.4, 7.0 + b * 4.0), vida: 0.22 + b * 0.2, tam: 0.05 + b * 0.05, color });
        }
        // El record se ve venir y se celebra sin detener nada.
        let centesimas = (self.vivo * 100.0) as u32;
        if self.record > 0 && !self.rompio_record && centesimas > self.record {
            self.rompio_record = true;
            self.destello = self.destello.max(0.45);
            self.eventos.push(Efecto::Perfecto);
            self.eventos.push(Efecto::Sello);
            self.letreros.retain(|l| l.u < 0.012);
            self.letreros.push(Letrero { texto: "¡NUEVO RÉCORD!".into(), sub: "y sigue contando", tonos: (Color::new(255, 190, 240, 255), Color::new(255, 70, 190, 255)), vida: 1.8, dura: 1.8, x: 0.5, y: 0.42, u: 0.0135 });
        }
        // Salen anillos hasta lo lejos.
        while (self.proxima_llegada - self.reloj) * self.velocidad < LEJOS {
            self.sacar_anillo();
        }
        // Los que cruzan por la nave.
        let (reloj, angulo) = (self.reloj, self.angulo);
        let mut choca = None;
        let mut pasan = Vec::new();
        for (k, r) in self.anillos.iter_mut().enumerate() {
            if !r.pasado && r.llegada <= reloj {
                r.pasado = true;
                if r.cabe(angulo) {
                    pasan.push(k);
                } else {
                    choca = Some(k);
                    break;
                }
            }
        }
        for k in pasan {
            self.paso_anillo(k);
        }
        if let Some(k) = choca {
            self.chocar(k);
            return;
        }
        self.anillos.retain(|r| r.llegada > reloj - 7.0 / r_vel(r.nivel));
    }

    fn sacar_anillo(&mut self) {
        let nivel = NIVELES.get(self.nivel_en_llegada()).unwrap_or(&NIVELES[NIVELES.len() - 1]);
        let (paso, rapidez, semi_base) = (nivel.paso as f32, nivel.rapidez, nivel.hueco);
        let k = self.nivel_en_llegada();
        // Lo que la nave alcanza a girar hasta el siguiente, con margen.
        let alcance = GIRO * paso * CORCHEA / rapidez * ALCANCE;
        let semi = (semi_base * self.apertura()).min(1.5);
        if self.patron.1 == 0 {
            self.patron = self.nuevo_patron(k, alcance);
        }
        self.patron.1 -= 1;
        let (patron, _) = self.patron;
        let lado = if self.azar() < 0.5 { 1.0 } else { -1.0 };
        let (delta, semi, giro, segundo) = match patron {
            Patron::Libre => {
                let d = (self.azar() * 0.7 + 0.3) * alcance * lado;
                (d, semi, 0.0, None)
            }
            Patron::Zigzag(signo) => {
                self.patron.0 = Patron::Zigzag(-signo);
                (signo * alcance * 0.92, semi * 0.92, 0.0, None)
            }
            Patron::Espiral(signo) => (signo * alcance * 0.72, semi, 0.0, None),
            Patron::Doble => {
                let otro = self.azar() * 0.5 + PI * 0.75;
                ((self.azar() - 0.5) * alcance, semi * 0.82, 0.0, Some(otro))
            }
            Patron::Corredor => (0.0, (semi * 0.68).max(0.34), 0.0, None),
            Patron::Giro(g) => ((self.azar() - 0.5) * alcance, semi, g, None),
        };
        self.guia = envolver(self.guia + delta);
        let huecos = [Some(Hueco { centro: self.guia, semi }), segundo.map(|o| Hueco { centro: envolver(self.guia + o), semi })];
        self.anillos.push(Anillo { llegada: self.proxima_llegada, huecos, giro, nivel: k, pasado: false });
        self.proxima_llegada += paso * CORCHEA;
    }

    /// El nivel que va a haber cuando llegue el anillo que se esta sacando.
    fn nivel_en_llegada(&self) -> usize {
        let falta = (self.proxima_llegada - self.reloj).max(0.0) / NIVELES[self.nivel].rapidez;
        (((self.vivo + falta) / DURA_NIVEL) as usize).min(NIVELES.len() - 1)
    }

    /// Que forma sigue. Las nuevas se van abriendo con los niveles: cada una se aprende sola.
    fn nuevo_patron(&mut self, nivel: usize, _alcance: f32) -> (Patron, u32) {
        let signo = if self.azar() < 0.5 { 1.0 } else { -1.0 };
        let r = self.azar();
        let hasta = (nivel + 1).min(6) as f32;
        match (r * hasta) as usize {
            0 => (Patron::Libre, 4),
            1 => (Patron::Zigzag(signo), 4),
            2 => (Patron::Doble, 3),
            3 => (Patron::Espiral(signo), 6),
            4 => (Patron::Corredor, 3),
            _ => (Patron::Giro(signo * (0.8 + self.azar() * 0.5)), 3),
        }
    }

    fn paso_anillo(&mut self, k: usize) {
        let r = &self.anillos[k];
        let roce = r.hueco_cerca(self.angulo).is_some_and(|(h, d)| d.abs() > h.semi - ROCE);
        self.pasados += 1;
        self.puntos += 10;
        self.eventos.push(Efecto::Pase);
        if roce {
            self.roces += 1;
            self.racha += 1;
            self.mejor_racha = self.mejor_racha.max(self.racha);
            let ganados = 25 * self.racha.min(10);
            self.puntos += ganados;
            self.eventos.push(Efecto::Roce);
            self.sacudida = self.sacudida.max(0.16);
            self.destello = self.destello.max(0.05);
            // La palabra revienta al lado de la nave, y crece con la cadena.
            let grito = GRITOS[(self.racha as usize - 1).min(GRITOS.len() - 1)];
            let lado = if self.azar() < 0.5 { 0.27 } else { 0.73 };
            let u = 0.0058 + 0.0009 * self.racha.min(8) as f32;
            self.letreros.push(Letrero { texto: grito.into(), sub: "", tonos: (Color::new(190, 250, 255, 255), Color::new(70, 180, 255, 255)), vida: 0.65, dura: 0.65, x: lado, y: 0.66, u });
            self.letreros.push(Letrero { texto: format!("+{ganados}"), sub: "", tonos: (Color::WHITE, Color::new(255, 230, 130, 255)), vida: 0.65, dura: 0.65, x: lado, y: 0.75, u: 0.0042 });
            let pos = Vec3::new(0.0, -RADIO, 0.0);
            self.polvo(pos, 8, 5, 0.7);
        } else {
            // Pasar por el centro corta la cadena: lo vicioso es arriesgar.
            self.racha = 0;
        }
        // Y cada tantos, un empujon.
        if self.pasados % 10 == 0 {
            let k = (self.pasados / 10 - 1) as usize % ANIMOS.len();
            let texto = if self.pasados % 50 == 0 { format!("¡{} ANILLOS!", self.pasados) } else { ANIMOS[k].to_string() };
            self.letreros.push(Letrero { texto, sub: "", tonos: (Color::new(255, 250, 180, 255), Color::new(255, 190, 60, 255)), vida: 0.8, dura: 0.8, x: 0.5, y: 0.3, u: 0.0078 });
        }
    }

    fn chocar(&mut self, k: usize) {
        let r = self.anillos[k].clone();
        self.culpable = Some(r);
        self.fase = Fase::Caido;
        self.t = 0.0;
        self.traba = 1.0;
        self.sacudida = 1.0;
        self.destello = 0.28;
        self.partidas += 1;
        self.mejor_vivo = self.mejor_vivo.max(self.vivo);
        self.record = self.record.max((self.vivo * 100.0) as u32);
        self.eventos.push(Efecto::GritoAnillazo);
        self.eventos.push(Efecto::Scratch);
        self.eventos.push(Efecto::Choque);
        self.polvo(Vec3::new(0.0, -RADIO, 0.0), 34, 2, 1.6);
        let f = (self.azar() * FRASES.len() as f32) as usize % FRASES.len();
        self.frase = FRASES[f];
        // Si le esta costando, los huecos se abren un poco (y se le dice).
        if self.vivo < MUERTE_RAPIDA {
            self.muertes_rapidas += 1;
        } else {
            self.muertes_rapidas = 0;
            self.ayuda = 0;
        }
        self.ayuda = match self.muertes_rapidas {
            0..=1 => 0,
            2..=3 => 1,
            _ => 2,
        };
    }

    fn polvo(&mut self, pos: Vec3, n: usize, color: usize, fuerza: f32) {
        for _ in 0..n {
            if self.chispas.len() >= 120 {
                break;
            }
            let (a, b, c) = (self.azar() * TAU, self.azar(), self.azar());
            let dir = Vec3::new(a.cos(), a.sin(), (b - 0.5) * 2.0 + 1.0);
            self.chispas.push(Chispa { pos, vel: dir * ((2.0 + c * 6.0) * fuerza), vida: 0.3 + b * 0.5, tam: 0.05 + c * 0.1, color });
        }
    }

    fn mover_efectos(&mut self, dt: f32) {
        for c in &mut self.chispas {
            c.vida -= dt;
            c.pos = c.pos + c.vel * dt;
            c.vel = c.vel * (1.0 - (dt * 2.0).min(1.0));
        }
        self.chispas.retain(|c| c.vida > 0.0);
        for l in &mut self.letreros {
            l.vida -= dt;
        }
        self.letreros.retain(|l| l.vida > 0.0);
    }

    // -----------------------------------------------------------------------------------------
    // Para dibujar
    // -----------------------------------------------------------------------------------------

    pub fn camara(&self) -> (Vec3, Vec3, Vec3) {
        let s = self.sacudida * self.sacudida * 0.22;
        let (a, b) = ((self.total * 73.0).sin(), (self.total * 91.0).cos());
        let empuje = Vec3::new(self.alabeo * 0.22, 0.0, -self.pulso * self.pulso * 0.12);
        let arriba = Vec3::new(self.alabeo * -0.05 + a * s * 0.08, 1.0, 0.0);
        (OJO + empuje + Vec3::new(a * s, b * s, 0.0), MIRA + Vec3::new(self.alabeo * 0.5, 0.0, 0.0), arriba)
    }

    pub fn fov(&self) -> f32 {
        // Abre con la velocidad, para sentirla.
        FOV + (self.velocidad - NIVELES[0].velocidad) * 0.28 + self.pulso * self.pulso * 0.8
    }

    /// El mundo gira alrededor de la nave: un angulo del anillo, en la pantalla.
    fn en_pantalla(&self, a: f32) -> f32 {
        a - self.angulo - FRAC_PI_2
    }

    pub fn nave(&self) -> Mat4 {
        let late = self.pulso * self.pulso * 0.02;
        Mat4::translation(Vec3::new(0.0, -RADIO + 0.05 + late, 0.0)) * Mat4::rotation_z(-self.alabeo) * Mat4::rotation_x(0.1) * Mat4::scale(0.42)
    }

    /// Cada bloque de cada anillo: su matriz y su color (el del nivel; rojo el que lo tumbo).
    pub fn bloques(&self) -> Vec<(Mat4, usize, bool)> {
        let mut v = Vec::with_capacity(self.anillos.len() * SECTORES);
        let vivos = self.anillos.iter().chain(self.culpable.iter());
        for r in vivos {
            let z = -(r.llegada - self.reloj) * self.velocidad;
            if !(-LEJOS * 1.05..=7.5).contains(&z) {
                continue;
            }
            let culpa = self.culpable.as_ref().is_some_and(|c| c.llegada == r.llegada);
            // Crece al salir de lo lejos, y late con la musica.
            let crece = ((LEJOS + z) / 8.0).clamp(0.0, 1.0) * (1.0 + self.pulso * self.pulso * 0.018);
            let vuelta = r.giro * (r.llegada - self.reloj);
            for s in 0..SECTORES {
                let a = (s as f32 + 0.5) / SECTORES as f32 * TAU;
                let abierto = r.huecos.iter().flatten().any(|h| envolver(a - h.centro).abs() < h.semi);
                if abierto {
                    continue;
                }
                let m = Mat4::translation(Vec3::new(0.0, 0.0, z)) * Mat4::rotation_z(self.en_pantalla(a + vuelta)) * Mat4::scale_xyz(Vec3::new(crece, crece, 1.0));
                v.push((m, r.nivel * 2 + s % 2, culpa));
            }
        }
        v
    }

    /// La luz del hueco: una cortina que brilla, para ver de un vistazo a donde hay que ir.
    pub fn puertas(&self) -> Vec<Mat4> {
        let mut v = Vec::new();
        for r in &self.anillos {
            let z = -(r.llegada - self.reloj) * self.velocidad;
            if r.pasado || !(-LEJOS..=2.0).contains(&z) {
                continue;
            }
            let vuelta = r.giro * (r.llegada - self.reloj);
            for s in 0..SECTORES {
                let a = (s as f32 + 0.5) / SECTORES as f32 * TAU;
                if r.huecos.iter().flatten().any(|h| envolver(a - h.centro).abs() < h.semi) {
                    v.push(Mat4::translation(Vec3::new(0.0, 0.0, z)) * Mat4::rotation_z(self.en_pantalla(a + vuelta)));
                }
            }
        }
        v
    }

    /// Las orillas de cada hueco, que brillan: se mira hacia donde hay que ir, no a la pared.
    pub fn orillas(&self) -> Vec<Mat4> {
        let mut v = Vec::new();
        for r in &self.anillos {
            let z = -(r.llegada - self.reloj) * self.velocidad;
            if !(-LEJOS..=7.5).contains(&z) {
                continue;
            }
            let vuelta = r.giro * (r.llegada - self.reloj);
            for h in r.huecos.iter().flatten() {
                for lado in [-1.0, 1.0] {
                    v.push(Mat4::translation(Vec3::new(0.0, 0.0, z)) * Mat4::rotation_z(self.en_pantalla(h.centro + lado * h.semi + vuelta)));
                }
            }
        }
        v
    }

    /// Si la nave esta alineada con el hueco del anillo que sigue (para la linea de mira).
    pub fn alineada(&self) -> bool {
        self.proximo().is_none_or(|r| r.cabe(self.angulo))
    }

    /// El aro que marca por donde va la nave gira con el mundo.
    pub fn giro_pista(&self) -> Mat4 {
        Mat4::rotation_z(self.en_pantalla(0.0))
    }

    /// Las rayitas de velocidad que pasan.
    pub fn rayas(&self) -> Vec<Mat4> {
        (0..34)
            .map(|k| {
                let s = k as f32 * 2.399;
                let z = (((self.reloj * self.velocidad * 0.9 + s * 37.0) % 90.0) - 84.0).min(6.0);
                let a = self.en_pantalla(s * 1.7);
                let r = 3.9 + (s * 3.1).sin().abs() * 3.5;
                Mat4::translation(Vec3::new(a.cos() * r, a.sin() * r, z)) * Mat4::scale_xyz(Vec3::new(0.03, 0.03, 0.8 + self.velocidad * 0.05))
            })
            .collect()
    }

    // -----------------------------------------------------------------------------------------
    // La pantalla
    // -----------------------------------------------------------------------------------------

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, imagen: Rectangle, ancho: i32, alto: i32) {
        self.imagen.set((imagen.x, imagen.y, imagen.width, imagen.height));
        match self.fase {
            Fase::Jugando => self.ui_jugando(d, e, imagen, ancho, alto),
            Fase::Caido => self.ui_caido(d, e, imagen, ancho, alto),
        }
    }

    fn ui_jugando(&self, d: &mut RaylibDrawHandle, e: &Entrada, imagen: Rectangle, ancho: i32, alto: i32) {
        let h = imagen.height;
        let (blanco, oro) = (Color::WHITE, Color::new(255, 225, 130, 255));
        // El tiempo: a la izquierda, enorme, y late con la musica. El letrero queda al centro.
        let u = h * (0.0118 + self.pulso * self.pulso * 0.0009);
        let t = format!("{:.1}", self.vivo);
        let m = imagen.x + h * 0.03;
        ui::bloques(d, &t, m + ui::medir_bloques(&t, u) * 0.5, imagen.y + h * 0.075, u, blanco, oro, 255);
        ui::subtitulo(d, "SEGUNDOS", m as i32, (imagen.y + h * 0.135) as i32, 16, ui::TEXTO);
        // Los puntos abajo del tiempo; la cadena, que crece.
        let p = format!("{}", self.puntos);
        let up = h * 0.0052;
        ui::bloques(d, &p, m + ui::medir_bloques(&p, up) * 0.5, imagen.y + h * 0.2, up, Color::new(190, 240, 255, 255), Color::new(110, 190, 255, 255), 255);
        if self.racha >= 2 {
            let c = format!("× {}", self.racha);
            let uc = h * (0.008 + 0.0005 * self.racha.min(10) as f32);
            ui::bloques(d, &c, m + ui::medir_bloques(&c, uc) * 0.5, imagen.y + h * 0.28, uc, Color::new(255, 190, 240, 255), Color::new(255, 80, 190, 255), 255);
        }
        // El record a la derecha.
        if self.record > 0 {
            let rec = self.record as f32 / 100.0;
            let r = format!("{rec:.1}");
            let ur = h * 0.0062;
            let color = if self.rompio_record { (Color::new(255, 190, 240, 255), Color::new(255, 80, 190, 255)) } else { (blanco, Color::new(210, 190, 255, 255)) };
            let x = imagen.x + imagen.width - h * 0.03 - ui::medir_bloques(&r, ur);
            ui::bloques(d, &r, x + ui::medir_bloques(&r, ur) * 0.5, imagen.y + h * 0.075, ur, color.0, color.1, 255);
            let rotulo = "RÉCORD";
            ui::subtitulo(d, rotulo, (imagen.x + imagen.width - h * 0.03) as i32 - ui::medir(rotulo, 16), (imagen.y + h * 0.112) as i32, 16, ui::TEXTO);
        }
        // Abajo: el nivel, con su barra, y la marca del record.
        let w = (imagen.width * 0.34) as i32;
        let (x, y) = ((ancho - w) / 2, (imagen.y + h * 0.955) as i32);
        let ultimo = self.nivel == NIVELES.len() - 1;
        let k = if ultimo { 1.0 } else { (self.vivo / DURA_NIVEL).fract() };
        d.draw_rectangle(x - 3, y - 3, w + 6, 16, Color::new(30, 10, 54, 230));
        d.draw_rectangle(x, y, (w as f32 * k) as i32, 10, Color::new(255, 120, 220, 255));
        let rec = self.record as f32 / 100.0;
        if self.record > 0 && !self.rompio_record && !ultimo && (rec / DURA_NIVEL) as usize == self.nivel {
            let mx = x + (w as f32 * (rec / DURA_NIVEL).fract()) as i32;
            d.draw_rectangle(mx - 2, y - 7, 4, 24, Color::new(255, 235, 110, 255));
        }
        let nombre = self.nombre_nivel();
        let un = h * 0.0048;
        ui::bloques(d, nombre, ancho as f32 * 0.5, y as f32 - un * 5.0, un, Color::new(255, 240, 170, 255), Color::new(255, 170, 70, 255), 235);
        // La primera vez: tres palabras, sobre el juego ya corriendo.
        if self.partidas == 0 && self.vivo < 2.2 {
            let a = ((2.2 - self.vivo) / 0.4).clamp(0.0, 1.0);
            ui::bloques(d, "¡AL HUECO!", ancho as f32 * 0.5, imagen.y + h * 0.42, h * 0.0125, Color::new(190, 255, 200, 255), Color::new(70, 220, 120, 255), (a * 255.0) as u8);
        }
        self.ui_letreros(d, imagen);
        self.ui_guia(d, e, imagen, ancho, alto);
    }

    /// Las instrucciones son pocas y se van solas: la flecha hacia el hueco y los botones.
    fn ui_guia(&self, d: &mut RaylibDrawHandle, e: &Entrada, imagen: Rectangle, ancho: i32, alto: i32) {
        let novato = self.partidas <= 1 && self.vivo < 9.0;
        if novato && !self.alineada() {
            if let Some((_, delta)) = self.proximo().and_then(|r| r.hueco_cerca(self.angulo)) {
                let cx = imagen.x + imagen.width * 0.5;
                let cy = imagen.y + imagen.height * 0.8;
                let lado = delta.signum();
                let brinco = (self.total * 8.0).sin().abs() * 16.0;
                let px = cx + lado * (imagen.width * 0.13 + brinco);
                let t = imagen.height * 0.05;
                let color = Color::new(255, 235, 110, 255);
                let (a, b, c) = (Vector2::new(px + lado * t, cy), Vector2::new(px, cy - t), Vector2::new(px, cy + t));
                if lado > 0.0 {
                    d.draw_triangle(b, c, a, color);
                } else {
                    d.draw_triangle(a, c, b, color);
                }
            }
        }
        if !self.ya_se_movio || (self.partidas == 0 && self.vivo < 3.0) {
            let apuntar = if e.hay_control { e.nombre("apuntar") } else { "A D" };
            ui::ayudas(d, &[(apuntar, "Girar")], ancho, alto);
        }
    }

    fn ui_letreros(&self, d: &mut RaylibDrawHandle, imagen: Rectangle) {
        for l in &self.letreros {
            let edad = l.dura - l.vida;
            // Revienta: entra grande y se asienta; luego se desvanece subiendo.
            let brote = 1.0 + (1.0 - (edad / 0.12).min(1.0)).powi(2) * 0.8;
            let a = ((l.vida / 0.22).min(1.0) * 255.0) as u8;
            // Que nunca se salga de la pantalla, por larga que sea la palabra.
            let cabe = (imagen.width - 24.0) / (l.texto.chars().count() as f32 * 6.0);
            let u = (imagen.height * l.u * brote).min(cabe);
            let (cx, cy) = (imagen.x + imagen.width * l.x, imagen.y + imagen.height * l.y - edad * imagen.height * 0.035);
            let w = ui::medir_bloques(&l.texto, u);
            let cx = cx.max(imagen.x + w * 0.5 + 8.0).min(imagen.x + imagen.width - w * 0.5 - 8.0);
            ui::bloques(d, &l.texto, cx, cy, u, l.tonos.0, l.tonos.1, a);
            if !l.sub.is_empty() {
                let tam = (imagen.height * 0.037) as i32;
                ui::subtitulo(d, l.sub, cx as i32 - ui::medir(l.sub, tam) / 2, (cy + u * 5.0) as i32, tam, Color::new(255, 255, 255, a));
            }
        }
    }

    fn ui_caido(&self, d: &mut RaylibDrawHandle, e: &Entrada, imagen: Rectangle, ancho: i32, alto: i32) {
        let h = imagen.height;
        // Sin velo encima de lo que paso: se oscurecen solo los bordes de arriba y abajo.
        d.draw_rectangle_gradient_v(imagen.x as i32, imagen.y as i32, imagen.width as i32, (h * 0.6) as i32, Color::new(20, 4, 44, 190), Color::new(20, 4, 44, 0));
        let cx = ancho as f32 * 0.5;
        // ¡ANILLAZO!: entra como un sello, grande y de golpe.
        let k = (self.t / 0.13).min(1.0);
        let sello = 1.0 + (1.0 - k).powi(2) * 1.6;
        let baile = (self.total * 8.0).sin() * h * 0.004;
        ui::bloques(d, "¡ANILLAZO!", cx, imagen.y + h * 0.1 + baile, h * 0.0165 * sello, Color::new(255, 240, 110, 255), Color::new(255, 70, 70, 255), 255);
        // El tiempo, y que tan cerca quedo: es lo que hace querer otra.
        let t = format!("{:.1}", self.vivo);
        ui::bloques(d, &t, cx, imagen.y + h * 0.265, h * 0.0165, Color::WHITE, Color::new(200, 215, 255, 255), 255);
        let y2 = imagen.y + h * 0.385;
        if self.rompio_record {
            let brilla = if (self.total * 7.0) as i32 % 2 == 0 { Color::new(255, 190, 240, 255) } else { Color::new(255, 245, 150, 255) };
            ui::bloques(d, "¡NUEVO RÉCORD!", cx, y2, h * 0.0088, brilla, Color::new(255, 70, 190, 255), 255);
        } else if self.record > 0 {
            let falta = self.record as f32 / 100.0 - self.vivo;
            let (texto, oro) = if falta < 0.05 { ("¡EMPATÓ SU RÉCORD!".to_string(), true) } else if falta < 3.0 { (format!("¡A {falta:.1} DEL RÉCORD!"), true) } else { (format!("RÉCORD: {:.1}", self.record as f32 / 100.0), false) };
            let tonos = if oro { (Color::new(255, 245, 150, 255), Color::new(255, 150, 60, 255)) } else { (Color::new(225, 215, 255, 255), Color::new(170, 150, 235, 255)) };
            ui::bloques(d, &texto, cx, y2, h * 0.0072, tonos.0, tonos.1, 255);
        }
        // La nota, como en la maquina.
        let nota = self.nota_de(self.vivo);
        let color_nota = match nota {
            "S" => Color::new(255, 225, 90, 255),
            "A" => Color::new(120, 240, 160, 255),
            "B" => Color::new(120, 200, 255, 255),
            _ => Color::new(210, 190, 255, 255),
        };
        let xn = imagen.x + imagen.width * 0.88;
        ui::bloques(d, nota, xn, imagen.y + h * 0.27, h * 0.021, color_nota, Color::new(color_nota.r / 2 + 40, color_nota.g / 2 + 20, color_nota.b / 2 + 60, 255), 255);
        ui::subtitulo(d, "NOTA", xn as i32 - ui::medir("NOTA", 18) / 2, (imagen.y + h * 0.355) as i32, 18, ui::TEXTO);
        let datos = format!("{} PTS", self.puntos);
        let xp = imagen.x + imagen.width * 0.12;
        ui::bloques(d, &datos, xp.max(imagen.x + ui::medir_bloques(&datos, h * 0.0052) * 0.5 + 12.0), imagen.y + h * 0.25, h * 0.0052, Color::new(190, 240, 255, 255), Color::new(110, 190, 255, 255), 255);
        let dos = format!("{} ANILLOS", self.pasados);
        ui::bloques(d, &dos, xp.max(imagen.x + ui::medir_bloques(&dos, h * 0.0034) * 0.5 + 12.0), imagen.y + h * 0.3, h * 0.0034, Color::new(255, 240, 200, 255), Color::new(255, 200, 120, 255), 255);
        let tres = format!("{} ROCES", self.roces);
        ui::bloques(d, &tres, xp.max(imagen.x + ui::medir_bloques(&tres, h * 0.0034) * 0.5 + 12.0), imagen.y + h * 0.34, h * 0.0034, Color::new(255, 240, 200, 255), Color::new(255, 200, 120, 255), 255);
        // Quien lo dijo, y la ayuda si hizo falta.
        let tam = (h * 0.042) as i32;
        ui::subtitulo(d, self.frase, (ancho - ui::medir(self.frase, tam)).max(10) / 2, (imagen.y + h * 0.5) as i32, tam, Color::new(255, 240, 190, 255));
        if self.ayuda > 0 && self.ayuda_nueva() {
            let a = "GLADAS: Le abrí un poco los huecos. No le diga a Venus.";
            let tam2 = (h * 0.032) as i32;
            ui::subtitulo(d, a, (ancho - ui::medir(a, tam2)).max(10) / 2, (imagen.y + h * 0.56) as i32, tam2, Color::new(170, 235, 255, 255));
        }
        // Donde estaba el hueco: una flecha verde desde la nave.
        if let Some((_, delta)) = self.culpable.as_ref().and_then(|r| r.hueco_cerca(self.angulo)) {
            let lado = delta.signum();
            let u = h * 0.0048;
            let t = "¡ERA AQUÍ!";
            let w = ui::medir_bloques(t, u);
            let cx = imagen.x + imagen.width * (0.5 + lado * 0.24);
            let cy = imagen.y + h * 0.83;
            ui::bloques(d, t, cx, cy, u, Color::new(210, 255, 215, 255), Color::new(90, 235, 130, 255), 255);
            let (ax, t2) = (cx + lado * (w * 0.5 + 14.0), h * 0.028);
            let (a, b, c) = (Vector2::new(ax + lado * t2 * 1.5, cy), Vector2::new(ax, cy - t2), Vector2::new(ax, cy + t2));
            let verde = Color::new(120, 245, 150, 255);
            if lado > 0.0 {
                d.draw_triangle(b, c, a, verde);
            } else {
                d.draw_triangle(a, c, b, verde);
            }
        }
        ui::ayudas(d, &[(e.nombre("aceptar"), "¡Otra!"), (e.nombre("aterrizar"), "Despegar")], ancho, alto);
    }

    fn ayuda_nueva(&self) -> bool {
        self.muertes_rapidas == 2 || self.muertes_rapidas == 4
    }

    /// Para las capturas: `anillos` (el arranque), `anillos_juego`, `anillos_rapido` (ya en los
    /// ultimos niveles) y `anillos_caido`.
    pub fn preparar_captura(&mut self, modo: &str) {
        match modo {
            "anillos_rapido" => {
                self.vivo = 48.0;
                self.partidas = 3;
                self.ya_se_movio = true;
                self.record = 5230;
            }
            "anillos_juego" => {
                self.partidas = 1;
                self.ya_se_movio = true;
                self.vivo = 14.0;
                self.record = 3120;
            }
            _ => {}
        }
    }
}

/// Lo que recorre un anillo por segundo de reloj en su nivel.
fn r_vel(nivel: usize) -> f32 {
    NIVELES[nivel.min(NIVELES.len() - 1)].velocidad
}

// ---------------------------------------------------------------------------------------------
// Los modelos
// ---------------------------------------------------------------------------------------------

pub struct Modelos {
    /// Un bloque de anillo por nivel, y el rojo del que tumbo a la nave.
    pub bloques: Vec<Modelo>,
    pub bloque_rojo: Modelo,
    pub orilla: Modelo,
    pub puerta: Modelo,
    /// El aro por donde va la nave, y la linea de mira: verde si va a caber, roja si no.
    pub pista: Modelo,
    pub mira_bien: Modelo,
    pub mira_mal: Modelo,
    pub raya: Modelo,
    /// El letrero: cada letra en dorado, rosa, celeste y blanco; su fondo, sus foquitos y sus brillos.
    pub letras: Vec<[Modelo; 4]>,
    pub letrero_fondo: Modelo,
    pub foquitos: [Modelo; 2],
    pub brillo: Modelo,
    pub aros_tunel: [Modelo; 2],
    pub cubos: [Modelo; 7],
}

const COLORES: [[f32; 3]; 8] = [
    [0.65, 0.35, 1.0],
    [1.0, 0.32, 0.72],
    [0.2, 0.72, 1.0],
    [1.0, 0.72, 0.15],
    [1.0, 0.42, 0.12],
    [1.0, 0.22, 0.42],
    [0.92, 0.2, 1.0],
    [1.0, 0.88, 0.45],
];

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn paleta(o: &mut Obra) {
    for (k, c) in COLORES.iter().enumerate() {
        o.material(&format!("c{k}"), c.map(|x| x * 0.32), c.map(|x| x * 0.55), 0.0, 1.0);
        o.material(&format!("b{k}"), c.map(|x| x * 0.55), c.map(|x| x * 0.88), 0.0, 1.0);
        o.material(&format!("d{k}"), c.map(|x| x * 0.4), c.map(|x| x * 0.62), 0.0, 1.0);
    }
    o.material("filo", [0.7, 0.7, 0.7], [1.0, 0.96, 0.9], 0.0, 1.0);
    o.material("rojo", [0.5, 0.08, 0.1], [1.0, 0.1, 0.12], 0.0, 1.0);
    o.material("rojo_b", [0.6, 0.15, 0.15], [1.0, 0.3, 0.25], 0.0, 1.0);
    o.material("orilla", [0.4, 0.7, 0.45], [0.4, 1.0, 0.55], 0.0, 1.0);
    o.material("pista", [0.3, 0.3, 0.4], [0.3, 0.3, 0.5], 0.0, 1.0);
    o.material("verde", [0.2, 0.5, 0.25], [0.3, 1.0, 0.45], 0.0, 1.0);
    o.material("luz_hueco", [0.3, 0.9, 0.5], [0.3, 1.0, 0.5], 0.0, 1.0);
    o.transparente("luz_hueco", 0.2);
    o.material("mal", [0.5, 0.15, 0.2], [1.0, 0.24, 0.3], 0.0, 1.0);
    o.material("blanco", [0.6, 0.6, 0.7], [0.75, 0.8, 0.95], 0.0, 1.0);
    for (k, c) in [[1.0f32, 1.0, 1.0], [0.7, 0.45, 1.0], [1.0, 0.4, 0.3], [1.0, 0.8, 0.3], [0.4, 1.0, 0.55], [0.5, 0.92, 1.0], [1.0, 0.45, 0.85]].iter().enumerate() {
        o.material(&format!("q{k}"), c.map(|x| x * 0.4), c.map(|x| x * 0.8), 0.0, 1.0);
    }
}

/// El letrero gigante: en lo alto, sin girar con el mundo.
const TITULO: [char; 8] = ['A', 'N', 'I', 'L', 'L', 'A', 'Z', 'O'];
const U_TITULO: f32 = 1.5;
const Y_TITULO: f32 = 20.0;
const Z_TITULO: f32 = -48.0;

fn titulo_x(k: usize) -> f32 {
    (k as f32 - 3.5) * 6.0 * U_TITULO
}

/// Lo de atras del letrero: el contorno de cada letra, el tablero y la estructura.
fn letrero_fondo() -> Modelo {
    let mut o = Obra::new();
    crate::koch::paleta(&mut o);
    let u = U_TITULO;
    for (k, &c) in TITULO.iter().enumerate() {
        let x0 = titulo_x(k) - u * 2.5;
        for (fila, bits) in ui::glifo_bloques(c).iter().enumerate() {
            for col in 0..5 {
                if bits & (16 >> col) != 0 {
                    let (cx, cy) = (x0 + col as f32 * u, Y_TITULO + (6 - fila) as f32 * u);
                    let m = u * 0.3;
                    o.caja("contorno", v(cx - m, cy - m * 1.5, Z_TITULO - u * 0.9), v(cx + u + m, cy + u + m, Z_TITULO - u * 0.3));
                }
            }
        }
    }
    let ancho = titulo_x(7) + u * 4.0;
    o.caja("tablero_letrero", v(-ancho, Y_TITULO - 2.2, Z_TITULO - 2.0), v(ancho, Y_TITULO + u * 7.0 + 2.2, Z_TITULO - 1.4));
    o.modelo()
}

fn foquitos(par: bool) -> Modelo {
    let mut o = Obra::new();
    crate::koch::paleta(&mut o);
    let ancho = titulo_x(7) + U_TITULO * 4.0;
    let (y0, y1, z) = (Y_TITULO - 2.2, Y_TITULO + U_TITULO * 7.0 + 2.2, Z_TITULO - 1.3);
    let t = 0.55;
    let mut k = 0;
    let mut x = -ancho;
    while x < ancho {
        if (k % 2 == 0) == par {
            o.caja("foco", v(x, y1 - t * 1.4, z), v(x + t, y1 - t * 0.4, z + t));
            o.caja("foco", v(x, y0 + t * 0.4, z), v(x + t, y0 + t * 1.4, z + t));
        }
        k += 1;
        x += 1.8;
    }
    o.modelo()
}

fn aro_tunel(mat: &str) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    for k in 0..32 {
        let (a, b) = (k as f32 / 32.0 * TAU, (k as f32 + 0.8) / 32.0 * TAU);
        let (r0, r1) = (8.4, 8.62);
        o.quad(mat, [v(a.cos() * r0, a.sin() * r0, 0.0), v(b.cos() * r0, b.sin() * r0, 0.0), v(b.cos() * r1, b.sin() * r1, 0.0), v(a.cos() * r1, a.sin() * r1, 0.0)], v(0.0, 0.0, 1.0));
    }
    o.modelo()
}

/// Un bloque: un trapecio de anillo que apunta hacia +X (a la derecha), con la cara de enfrente
/// mas clara: asi, de frente, se ve como una pared con filo.
fn bloque(cuerpo: &str, filo: &str) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let medio = PI / SECTORES as f32 * 0.93;
    let (c, s) = (medio.cos(), medio.sin());
    let p = |r: f32, lado: f32, z: f32| v(r * c, r * s * lado, z);
    let (z0, z1) = (-0.34, 0.34);
    let (a, b) = (R_ADENTRO, R_AFUERA);
    // Enfrente, hacia la camara.
    o.quad(filo, [p(a, -1.0, z1), p(b, -1.0, z1), p(b, 1.0, z1), p(a, 1.0, z1)], v(0.0, 0.0, 1.0));
    o.quad(cuerpo, [p(a, -1.0, z0), p(b, -1.0, z0), p(b, 1.0, z0), p(a, 1.0, z0)], v(0.0, 0.0, -1.0));
    o.quad(cuerpo, [p(a, -1.0, z0), p(a, 1.0, z0), p(a, 1.0, z1), p(a, -1.0, z1)], v(-1.0, 0.0, 0.0));
    o.quad(cuerpo, [p(b, -1.0, z0), p(b, 1.0, z0), p(b, 1.0, z1), p(b, -1.0, z1)], v(1.0, 0.0, 0.0));
    o.quad(cuerpo, [p(a, 1.0, z0), p(b, 1.0, z0), p(b, 1.0, z1), p(a, 1.0, z1)], v(-s, c, 0.0));
    o.quad(cuerpo, [p(a, -1.0, z0), p(b, -1.0, z0), p(b, -1.0, z1), p(a, -1.0, z1)], v(-s, -c, 0.0));
    // Sus filos de neon, adentro y afuera: de lejos dibujan el aro.
    let zf = z1 + 0.012;
    let (a2, b2) = (a + 0.15, b - 0.15);
    o.quad("filo", [p(a, -1.0, zf), p(a2, -1.0, zf), p(a2, 1.0, zf), p(a, 1.0, zf)], v(0.0, 0.0, 1.0));
    o.quad("filo", [p(b2, -1.0, zf), p(b, -1.0, zf), p(b, 1.0, zf), p(b2, 1.0, zf)], v(0.0, 0.0, 1.0));
    o.modelo()
}

/// La orilla del hueco: un poste que brilla, un poco mas largo que el anillo.
fn orilla() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("orilla", v(R_ADENTRO - 0.26, -0.09, -0.45), v(R_AFUERA + 0.26, 0.09, 0.45));
    o.modelo()
}

/// Un pedazo de la cortina de luz del hueco, por los dos lados.
fn puerta() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let medio = PI / SECTORES as f32;
    let (c, s) = (medio.cos(), medio.sin());
    let p = |r: f32, lado: f32| v(r * c, r * s * lado, 0.0);
    let (a, b) = (R_ADENTRO, R_AFUERA);
    o.quad("luz_hueco", [p(a, -1.0), p(b, -1.0), p(b, 1.0), p(a, 1.0)], v(0.0, 0.0, 1.0));
    o.modelo()
}

fn pista() -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    // Marcas cada tanto, asi tambien se nota que el mundo gira.
    for k in 0..48 {
        let (a, b) = (k as f32 / 48.0 * TAU, (k as f32 + 0.55) / 48.0 * TAU);
        let (r0, r1) = (RADIO - 0.035, RADIO + 0.035);
        o.quad("pista", [v(a.cos() * r0, a.sin() * r0, 0.0), v(b.cos() * r0, b.sin() * r0, 0.0), v(b.cos() * r1, b.sin() * r1, 0.0), v(a.cos() * r1, a.sin() * r1, 0.0)], v(0.0, 0.0, 1.0));
    }
    o.modelo()
}

/// La linea de mira: por donde va a pasar la nave, de ella hacia adelante.
fn mira(mat: &str) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    let mut z = -0.9;
    while z > -50.0 {
        o.caja(mat, v(-0.05, -0.02, z - 0.9), v(0.05, 0.02, z));
        z -= 1.8;
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
        // Dos tonos por nivel, de bloque en bloque: asi se nota que el mundo gira.
        bloques: (0..COLORES.len()).flat_map(|k| [bloque(&format!("c{k}"), &format!("b{k}")), bloque(&format!("c{k}"), &format!("d{k}"))]).collect(),
        bloque_rojo: bloque("rojo", "rojo_b"),
        orilla: orilla(),
        puerta: puerta(),
        pista: pista(),
        mira_bien: mira("verde"),
        mira_mal: mira("mal"),
        raya: cubo("blanco"),
        letras: TITULO.iter().map(|&c| [("oro_a", "oro_b"), ("rosa_a", "rosa_b"), ("cian_a", "cian_b"), ("blanco_a", "blanco_a")].map(|t| crate::koch::letra(c, U_TITULO, t))).collect(),
        letrero_fondo: letrero_fondo(),
        foquitos: [foquitos(false), foquitos(true)],
        brillo: crate::koch::brillo(),
        aros_tunel: [aro_tunel("q6"), aro_tunel("q5")],
        cubos: ["q0", "q1", "q2", "q3", "q4", "q5", "q6"].map(cubo),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    /// Juega solo hasta `segundos` de vida o hasta chocar.
    fn jugar(semilla: u32, segundos: f32) -> Anillazo {
        let mut a = Anillazo::new(semilla, 0);
        while a.fase == Fase::Jugando && a.vivo < segundos {
            let e = a.chofer();
            a.actualizar(&e, DT);
            a.eventos.clear();
        }
        a
    }

    /// Lo mas importante: siempre se puede. Quien va al hueco llega, en todos los niveles, y mas
    /// alla, con muchas semillas.
    #[test]
    fn todos_los_anillos_se_pueden_pasar() {
        for s in [1, 7, 42, 99, 1234, 777, 2026, 31337] {
            let a = jugar(s, 200.0);
            assert_eq!(a.fase, Fase::Jugando, "semilla {s}: choco a los {:.1} s, nivel {}", a.vivo, a.nivel);
            assert_eq!(a.nivel, NIVELES.len() - 1);
            assert!(a.pasados > 300, "semilla {s}: solo {} anillos", a.pasados);
        }
    }

    /// Y con un reflejo humano (reacciona tarde y gira a media maquina) igual se llega lejos: no
    /// hace falta ser una maquina para ver todos los niveles... al menos hasta los ultimos.
    #[test]
    fn con_reflejos_humanos_se_llega_lejos() {
        let mut marcas = Vec::new();
        for s in [3, 11, 23, 58, 91, 140, 512, 900] {
            let mut a = Anillazo::new(s, 0);
            // Ve el hueco que sigue con 230 ms de atraso, pero sabe donde va su nave; gira a 85%.
            let mut vistos: Vec<Option<f32>> = vec![None; 14];
            while a.fase == Fase::Jugando && a.vivo < 120.0 {
                vistos.push(a.proximo().and_then(|r| r.hueco_cerca(a.angulo)).map(|(h, _)| h.centro));
                let meta = vistos.remove(0);
                let volante = meta.map_or(0.0, |c| (envolver(c - a.angulo) * 5.0).clamp(-1.0, 1.0) * 0.85);
                a.actualizar(&Entrada { horizontal: volante, ..Default::default() }, DT);
                a.eventos.clear();
            }
            marcas.push(a.vivo);
        }
        let peor = marcas.iter().fold(f32::MAX, |m, x| m.min(*x));
        let medio = marcas.iter().sum::<f32>() / marcas.len() as f32;
        eprintln!("marcas con reflejos humanos: {marcas:?}");
        assert!(peor > 25.0, "con 230 ms de reaccion se muere muy pronto: {marcas:?}");
        assert!(medio > 40.0, "muy dificil para una persona: {marcas:?}");
    }

    /// Sin tocar nada, el primer anillo lo tumba: obliga a aprender a moverse, y da tiempo de verlo.
    #[test]
    fn el_primer_anillo_ensena_a_moverse() {
        let mut a = Anillazo::new(5, 0);
        let nada = Entrada::default();
        while a.fase == Fase::Jugando && a.vivo < 10.0 {
            a.actualizar(&nada, DT);
        }
        assert_eq!(a.fase, Fase::Caido, "sin moverse no se puede pasar");
        assert!((1.6..3.0).contains(&a.vivo), "le dio tiempo de reaccionar: {}", a.vivo);
        // Y para pasarlo basta un toquecito hacia el lado correcto.
        let mut a = Anillazo::new(5, 0);
        let lado = a.proximo().unwrap().hueco_cerca(a.angulo).unwrap().1.signum();
        for _ in 0..12 {
            a.actualizar(&Entrada { horizontal: lado, ..Default::default() }, DT);
        }
        assert!(a.alineada(), "un cuarto de segundo debe bastar");
    }

    /// Reiniciar tiene que ser casi instantaneo, y un boton aplastado desde antes no salta nada.
    #[test]
    fn otra_vez_en_menos_de_medio_segundo() {
        let mut a = Anillazo::new(5, 0);
        let nada = Entrada::default();
        while a.fase == Fase::Jugando {
            a.actualizar(&nada, DT);
        }
        let boton = Entrada { aceptar: true, ..Default::default() };
        a.actualizar(&boton, DT);
        assert_eq!(a.fase, Fase::Caido, "no se salta la pantalla sin verla");
        let mut t = DT;
        while a.fase == Fase::Caido {
            a.actualizar(&boton, DT);
            t += DT;
        }
        assert!(t < 0.5, "tardo {t} s en reiniciar");
        assert_eq!(a.vivo, 0.0);
        assert!(a.culpable.is_none());
        // La musica no se reinicia: el reloj sigue.
        assert!(a.reloj > 2.0);
        assert_eq!(a.partidas, 1);
    }

    #[test]
    fn cualquier_boton_del_control_reinicia_y_despegar_va_aparte() {
        for e in [Entrada { orbitar: true, hay_control: true, ..Default::default() }, Entrada { barrel: true, hay_control: true, ..Default::default() }, Entrada { pestana_der: true, hay_control: true, ..Default::default() }] {
            let mut a = Anillazo::new(5, 0);
            a.actualizar(&Entrada::default(), DT);
            let k = 0;
            a.chocar(k);
            a.t = 1.0;
            a.actualizar(&e, DT);
            assert_eq!(a.fase, Fase::Jugando);
        }
        let mut a = Anillazo::new(5, 0);
        a.chocar(0);
        let cuadrado = Entrada { aterrizar: true, hay_control: true, ..Default::default() };
        assert!(matches!(a.actualizar(&cuadrado, DT), AccionAnillos::Nada), "recien chocado no despega");
        a.t = 1.0;
        assert!(matches!(a.actualizar(&cuadrado, DT), AccionAnillos::Despegar));
    }

    /// Si cuesta, ayuda: y solo abre los huecos, no toca el record.
    #[test]
    fn tras_varias_muertes_rapidas_los_huecos_se_abren() {
        let mut a = Anillazo::new(5, 0);
        assert_eq!(a.ayuda, 0);
        let nada = Entrada::default();
        let mut semis = Vec::new();
        for _ in 0..5 {
            while a.fase == Fase::Jugando {
                a.actualizar(&nada, DT);
            }
            a.t = 1.0;
            a.actualizar(&Entrada { aceptar: true, ..Default::default() }, DT);
            semis.push(a.anillos[0].huecos[0].unwrap().semi);
        }
        assert_eq!(a.ayuda, 2);
        assert!(semis[4] > semis[0] * 1.2, "{semis:?}");
        // Y una buena partida la quita.
        a.vivo = 30.0;
        a.chocar(0);
        assert_eq!(a.ayuda, 0);
    }

    #[test]
    fn pasar_rozando_da_puntos_y_encadena() {
        let mut a = Anillazo::new(5, 0);
        a.anillos.clear();
        let hueco = Hueco { centro: 1.0, semi: 0.5 };
        a.anillos.push(Anillo { llegada: 0.0, huecos: [Some(hueco), None], giro: 0.0, nivel: 0, pasado: true });
        a.angulo = 1.0;
        a.paso_anillo(0);
        assert_eq!(a.racha, 0, "por el centro no hay roce");
        assert_eq!(a.puntos, 10);
        a.angulo = 1.4;
        a.paso_anillo(0);
        a.paso_anillo(0);
        assert_eq!(a.roces, 2);
        assert_eq!(a.racha, 2);
        assert_eq!(a.puntos, 10 + 10 + 25 + 10 + 50);
        assert!(a.eventos.contains(&Efecto::Roce));
    }

    /// La nave choca con menos de lo que se ve: un roce visible nunca mata.
    #[test]
    fn se_perdona_la_orilla() {
        let r = Anillo { llegada: 0.0, huecos: [Some(Hueco { centro: 0.0, semi: 0.5 }), None], giro: 0.0, nivel: 0, pasado: false };
        assert!(r.cabe(0.5 + PERDON * 0.9));
        assert!(!r.cabe(0.5 + PERDON * 1.5));
        assert!(r.cabe(envolver(TAU + 0.2)), "da la vuelta bien");
    }

    #[test]
    fn la_musica_se_acelera_con_cada_nivel() {
        for par in NIVELES.windows(2) {
            assert!(par[1].rapidez > par[0].rapidez);
            assert!(par[1].velocidad > par[0].velocidad);
            assert!(par[1].hueco < par[0].hueco);
            assert!(par[1].paso <= par[0].paso);
        }
        // Ni el ultimo tiene huecos imposibles para el ancho de la nave.
        assert!(NIVELES[NIVELES.len() - 1].hueco > 0.3);
    }

    #[test]
    fn los_anillos_caen_en_la_corchea() {
        let a = jugar(7, 30.0);
        for r in &a.anillos {
            let c = r.llegada / CORCHEA;
            assert!((c - c.round()).abs() < 1e-3, "llegada {}", r.llegada);
        }
    }

    /// Al volver de la pausa, el tiempo arranca despacio.
    #[test]
    fn despues_de_la_pausa_arranca_despacio() {
        let mut a = Anillazo::new(5, 0);
        for _ in 0..30 {
            a.actualizar(&Entrada::default(), DT);
        }
        let reloj = a.reloj;
        a.actualizar(&Entrada::default(), 0.0);
        assert_eq!(a.reloj, reloj, "en pausa no corre");
        let antes = a.reloj;
        a.actualizar(&Entrada::default(), DT);
        assert!(a.reloj - antes < DT * 0.5, "arranca lento");
        for _ in 0..90 {
            a.actualizar(&Entrada::default(), DT);
        }
        assert!(a.tras_pausa <= 0.0);
    }

    #[test]
    fn el_record_se_ve_y_se_celebra_sin_parar() {
        let mut a = Anillazo::new(7, 500);
        while a.vivo < 5.5 {
            let e = a.chofer();
            a.actualizar(&e, DT);
            if a.eventos.contains(&Efecto::Perfecto) {
                assert_eq!(a.fase, Fase::Jugando);
            }
            a.eventos.clear();
        }
        assert!(a.rompio_record);
    }

    #[test]
    fn el_control_gira_como_el_teclado() {
        let mut a = Anillazo::new(5, 0);
        a.actualizar(&Entrada { horizontal: 1.0, hay_control: true, ..Default::default() }, DT);
        assert!(a.angulo > 0.0, "derecha suma");
        assert!(a.ya_se_movio);
    }

    #[test]
    fn la_nota_sale_de_la_mejor_marca() {
        let mut a = Anillazo::new(5, 0);
        for (t, n) in [(5.0, "C"), (25.0, "B"), (45.0, "A"), (70.0, "S")] {
            a.mejor_vivo = t;
            assert_eq!(a.nota(), n);
        }
    }

    #[test]
    fn envolver_queda_en_la_vuelta() {
        for x in [-10.0f32, -3.3, -PI, 0.0, PI, 3.3, 10.0, 100.0] {
            let y = envolver(x);
            assert!(y > -PI - 1e-4 && y <= PI + 1e-4, "{x} -> {y}");
            assert!((y - x).rem_euclid(TAU).min(TAU - (y - x).rem_euclid(TAU)) < 1e-3);
        }
    }
}
