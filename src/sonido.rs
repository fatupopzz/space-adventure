//! Todo el sonido del juego, sintetizado al arrancar: no hay archivos de
//! audio. Cada sonido se genera como muestras, se empaqueta como un `.wav`
//! en memoria y raylib lo carga como `Sound`.
//!
//! - **Voces de blips**: cada personaje "habla" con sonidito por letra, cada
//!   uno con su timbre (GLADAS calmada, Miku aguda, el profe grave). El
//!   texto aparece letra por letra al mismo paso (ver `Dialogo`).
//! - **Efectos**: acierto, perfecto, fallo, salto, choque, aterrizaje,
//!   fuegos artificiales.
//! - **Musica**: la cancion de `cancion.rs`.
//!
//! Si no hay dispositivo de audio, todo sigue funcionando en silencio.

use crate::cancion::{self, FRECUENCIA};
use raylib::prelude::*;
use std::cell::Cell;
use std::f32::consts::TAU;

/// Segundos que tarda una pista en subir o bajar del todo.
const FUNDIDO_MUSICA: f32 = 1.8;

thread_local! {
    static TONO_PROTAGONISTA: Cell<f32> = const { Cell::new(1.0) };
    static VOZ_PROTAGONISTA: std::cell::Cell<Voz> = const { std::cell::Cell::new(Voz::Miku) };
}

/// La voz del personaje escogido (la usan sus lineas, que se escribieron para Miku).
pub fn poner_voz_del_protagonista(v: Voz, tono: f32) {
    TONO_PROTAGONISTA.with(|c| c.set(tono));
    VOZ_PROTAGONISTA.with(|c| c.set(v));
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Voz {
    Ia,
    Miku,
    Profe,
    /// Grave, temblorosa y dramatica.
    Pluton,
    /// La de los demas planetas del consejo.
    Planeta,
    /// Curiosity: un "bwip" de robotito que baja.
    Rover,
    /// Las candidatas de la cita en Marte: agudita y con un piquito que sube.
    Robotina,
    /// Neptuno, anfitrion de la noche de cartas: grave, alegre y con eco de estadio.
    Neptuno,
    /// El Doom Slayer no habla: grune (ruido grave).
    Grunido,
    /// El Master Chief, por radio: cuadrada filtrada con estatica.
    Radio,
    /// Canela: "animales" como en Animal Crossing, agudito, cortito y saltarin.
    Animalito,
    /// Luka (la que sale de la fuente): mas grave que Miku, dulce y con coro.
    Hada,
    /// Madame Mercurio: grave, con mucho vibrato, como un theremin.
    Vidente,
    /// Sonic: agudo, cortito y rapidisimo.
    Erizo,
    /// Gollum: grave, rasposo, con gargara.
    Gollum,
    /// Los mensajes en el piso del Sinluz: una campanita suave.
    Mensaje,
    /// Phoenix Wright: el blip grave de los juzgados.
    Abogado,
    /// El juez: un ganso. Honk.
    Ganso,
    /// El director de la Luna, gritando por su megafono: zumbido rasposo.
    Megafono,
}

/// Las pistas de musica.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pista {
    /// La cancion de Saturno Ritmo (intro, concierto y minijuego).
    Cancion,
    /// El fondo tranquilo para volar.
    Espacio,
    /// El arpa de la Gran Fuente de las Hadas.
    Fuente,
    /// El consultorio de tarot de Mercurio, y la misma pista al reves (para el retrogrado).
    Tarot,
    TarotReves,
    /// El juicio de divorcio de Saturno: tensa, como interrogatorio.
    Juicio,
    /// Jupiter Asteroids: chiptune rapido de arcade.
    Arcade,
    /// La noche de la entrega en el CIT: lo-fi para estudiar, con lluvia.
    Lofi,
    /// El Planeta Chapin: un son de marimba en 6/8.
    Marimba,
    /// La cinematica de inicio: la cancion local si esta, o el piano triste original.
    Prologo,
    /// Whack-a-Koch: un chiptune de feria a 150 BPM.
    Koch,
    /// ¡Anillazo!: dance a 140 BPM, que se acelera con cada nivel.
    Anillos,
}

/// El sonido de la intro dibujada a mano (la voz de Pluton y el piano del juego; ver
/// `intro_video.rs`). Si no esta, suena el piano original de `cancion::sintetizar_prologo`.
const CANCION_DEL_PROLOGO: &str = "assets/intro/intro.mp3";

pub fn hay_cancion_del_prologo() -> bool {
    std::path::Path::new(CANCION_DEL_PROLOGO).exists()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Efecto {
    Acierto,
    Perfecto,
    Fallo,
    Salto,
    Choque,
    Aterrizaje,
    Fuego,
    Alarma,
    /// Feliz cumpleanos tocado con el taladro de Curiosity.
    Cumpleanos,
    /// Una carta que se tira en la mesa.
    Carta,
    /// Golpe al piso en Saturno Ritmo: un tambor grave (el "don" de Taiko).
    Don,
    /// Brinco al aire en Saturno Ritmo: un golpe de aro agudo (el "ka").
    Ka,
    /// Las cinco notas de la ocarina (re, fa, la, si y re alto), como sus cinco botones.
    OcarinaA,
    OcarinaAbajo,
    OcarinaDerecha,
    OcarinaIzquierda,
    OcarinaArriba,
    /// Una cinta rebobinando: chirridos que suben, una y otra vez.
    Rebobinar,
    /// ¡PROTESTO!: el golpe y el acorde de metales.
    Protesto,
    /// Tres golpes del mazo del juez.
    Mazo,
    /// La claqueta del set de la Luna: ¡clac!
    Claqueta,
    /// Las notas que se recogen volando: la escala pentatonica, subiendo con el combo.
    Nota0,
    Nota1,
    Nota2,
    Nota3,
    Nota4,
    Nota5,
    Nota6,
    Nota7,
    /// Pasa un cometa: un barrido brillante.
    Cometa,
    /// ¡Sello nuevo en el pasaporte!: el golpe del sello y una campanita.
    Sello,
    /// Chasquido seco de madera, al tocar la regla al protagonista.
    Reglazo,
    /// Un latido (lub-dub): cuando Koch anda cerca.
    Latido,
    /// Un trueno que retumba (la tormenta sobre el CIT).
    Trueno,
    /// Una puerta que rechina.
    Puerta,
    /// La notificacion del grupo de WhatsApp (dos notitas).
    Whatsapp,
    /// Teclear rapidito (cuando se resuelve algo en la compu).
    Teclas,
    /// El chorro de la manguera (un siseo cortito de agua).
    Chorro,
    /// Un globo que revienta.
    Pop,
    /// Rocky (Project Hail Mary) hablando: los eridianos hablan con acordes. Cuatro, subiendo
    /// (esta contento).
    Rocky,
    /// El monolito (2001): tres notas de metales que suben (do, sol, do), el acorde que pasa de
    /// mayor a menor y los timbales (el amanecer de Asi hablo Zaratustra, de Strauss, de 1896).
    Monolito,
    /// Adentro del teseracto (Interstellar): un acorde de organo que crece y un retumbo grave.
    Teseracto,
    /// Al entrar al agujero de gusano: un soplido que baja.
    Gusano,
    /// Whack-a-Koch: el mazo pegandole a algo.
    Mazazo,
    /// El mismo mazazo cuando el golpe fue ¡PERFECTO! (o al dorado): suena con mas cuerpo y, en el
    /// control, se siente mas fuerte y mas largo.
    MazazoFuerte,
    /// El mazo pegandole al aire.
    Aire,
    /// Un Koch que sale del hoyo: un toquecito que sube.
    KochSale,
    /// ¡RECURSION!: un arpegio que se dispara.
    Recursion,
    /// ¡WACK!: la palabra, dicha por la maquina, en tres tonos.
    Wack,
    Wack2,
    Wack3,
    /// El mazo contra el tablero, cuando se pega al aire.
    Tablero,
    /// El mini Koch: ¡ay!
    KochAy,
    /// El mini Koch burlandose: je-je.
    KochRisa,
    /// ¡Anillazo!: un anillo que pasa, un roce, y el disco que se raya al chocar.
    Pase,
    Roce,
    Scratch,
    /// ¡A-NI-LLA-ZO!: la palabra, dicha al chocar.
    GritoAnillazo,
}

/// Empaqueta muestras en [-1, 1] como un `.wav` mono de 16 bits.
fn wav(muestras: &[f32]) -> Vec<u8> {
    let datos = muestras.len() as u32 * 2;
    let mut b = Vec::with_capacity(44 + datos as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + datos).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&1u16.to_le_bytes()); // mono
    b.extend_from_slice(&FRECUENCIA.to_le_bytes());
    b.extend_from_slice(&(FRECUENCIA * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&datos.to_le_bytes());
    for m in muestras {
        b.extend_from_slice(&((m.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    b
}

/// Genera `largo` segundos con `f(t)`.
fn generar(largo: f32, mut f: impl FnMut(f32) -> f32) -> Vec<f32> {
    (0..(largo * FRECUENCIA as f32) as usize).map(|k| f(k as f32 / FRECUENCIA as f32)).collect()
}

/// Una voz sintetica de caricatura: una onda de sierra (la cuerda vocal) que pasa por dos filtros
/// resonantes (la boca). `formantes` dice a que frecuencias resuena en cada momento: eso es lo que
/// hace que suene a una vocal. La voz dura `sonando` segundos dentro de un sonido de `largo`.
fn voz_formantes(largo: f32, sonando: f32, tono_base: f32, baja: f32, formantes: impl Fn(f32) -> (f32, f32), volumen: f32) -> Vec<f32> {
    let dt = 1.0 / FRECUENCIA as f32;
    let mut fase = 0.0f32;
    // Dos resonadores de segundo orden, con sus dos muestras pasadas.
    let (mut a1, mut a2, mut b1, mut b2) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let mut salida: Vec<f32> = generar(largo, |t| {
        let f0 = tono_base + baja * t;
        fase = (fase + f0 * dt).fract();
        let fuente = if t < sonando { 2.0 * fase - 1.0 } else { 0.0 };
        let (f1, f2) = formantes(t);
        let resonar = |x: f32, f: f32, y1: &mut f32, y2: &mut f32| {
            let r = 0.965f32;
            let c = 2.0 * r * (TAU * f * dt).cos();
            let y = (1.0 - r) * 2.0 * x + c * *y1 - r * r * *y2;
            *y2 = *y1;
            *y1 = y;
            y
        };
        let v1 = resonar(fuente, f1, &mut a1, &mut a2);
        let v2 = resonar(fuente, f2, &mut b1, &mut b2);
        let sale = ((sonando - t) / 0.012).clamp(0.0, 1.0) * (t / 0.006).min(1.0);
        (v1 + 0.6 * v2) * sale
    });
    let pico = salida.iter().fold(0.0f32, |m, x| m.max(x.abs())).max(1e-6);
    for m in &mut salida {
        *m = *m / pico * volumen;
    }
    salida
}

/// Un blip de voz: una notita cortita con el timbre de cada personaje.
fn blip(voz: Voz, semitonos: f32) -> Vec<f32> {
    let (base, largo) = match voz {
        // GLADAS: grave y larguita, para que suene tranquila (antes daba ansiedad).
        Voz::Ia => (220.0, 0.085),
        Voz::Miku => (700.0, 0.05),
        Voz::Profe => (170.0, 0.07),
        Voz::Pluton => (125.0, 0.09),
        Voz::Planeta => (260.0, 0.06),
        Voz::Rover => (240.0, 0.07),
        Voz::Robotina => (560.0, 0.05),
        Voz::Neptuno => (150.0, 0.08),
        Voz::Grunido => (70.0, 0.1),
        Voz::Radio => (210.0, 0.06),
        Voz::Animalito => (640.0, 0.045),
        Voz::Hada => (280.0, 0.1),
        Voz::Vidente => (185.0, 0.11),
        Voz::Erizo => (620.0, 0.035),
        Voz::Gollum => (130.0, 0.09),
        Voz::Mensaje => (520.0, 0.14),
        Voz::Abogado => (165.0, 0.06),
        Voz::Ganso => (330.0, 0.11),
        Voz::Megafono => (190.0, 0.07),
    };
    let f = base * 2f32.powf(semitonos / 12.0);
    generar(largo, |t| {
        // Entrada y salida suaves (en la de GLADAS, mas suaves todavia: sin "clic").
        let ataque = if voz == Voz::Ia { 0.015 } else { 0.004 };
        let env = (t / ataque).min(1.0) * (1.0 - t / largo).max(0.0).powi(2);
        let fase = t * f;
        let s = match voz {
            // Seno suave con un poquito de su octava y un temblor leve de robot:
            // sigue sonando a maquina, pero calmada.
            Voz::Ia => ((TAU * fase).sin() * 0.8 + 0.2 * (TAU * fase * 2.0).sin()) * (0.9 + 0.1 * (TAU * 30.0 * t).sin()),
            // Triangular con su octava: brillante y dulce.
            Voz::Miku => (4.0 * (fase.fract() - 0.5).abs() - 1.0) * 0.8 + 0.25 * (TAU * fase * 2.0).sin(),
            // Cuadrada grave y un poco nasal.
            Voz::Profe => (if fase.fract() < 0.5 { 1.0 } else { -1.0 }) * 0.7 + 0.3 * (TAU * fase * 3.0).sin(),
            // Triangular con un vibrato hondo: suena a que esta por llorar.
            Voz::Pluton => {
                let f2 = 1.0 + 0.06 * (TAU * 7.0 * t).sin();
                4.0 * ((fase * f2).fract() - 0.5).abs() - 1.0
            }
            // Seno con un poquito de cuadrada.
            Voz::Planeta => 0.8 * (TAU * fase).sin() + 0.2 * (if fase.fract() < 0.5 { 1.0 } else { -1.0 }),
            // Diente de sierra que cae de tono: "bwip".
            Voz::Rover => {
                let fase = t * f * (1.0 - 2.5 * t);
                (2.0 * fase.fract() - 1.0) * 0.7
            }
            // Seno que sube de tono, como piquito.
            Voz::Robotina => (TAU * t * f * (1.0 + 3.0 * t)).sin(),
            // Seno con su quinta y un eco cortito.
            Voz::Neptuno => 0.6 * (TAU * fase).sin() + 0.3 * (TAU * fase * 1.5).sin() + 0.2 * (TAU * (t - 0.02).max(0.0) * f).sin(),
            // Un grunido: seno muy grave con mucho ruido encima.
            Voz::Grunido => {
                let r = ((t * 7919.0 * f).sin() * 43758.5).fract() * 2.0 - 1.0;
                0.5 * (TAU * fase).sin() + 0.5 * r
            }
            // Cuadrada angosta con un zumbido de radio.
            Voz::Radio => (if fase.fract() < 0.25 { 1.0 } else { -1.0 }) * (0.7 + 0.3 * (TAU * 120.0 * t).sin()) * 0.8,
            // Dos senos un poquito desafinados (coro) con vibrato hondo.
            Voz::Hada => {
                let v = 1.0 + 0.012 * (TAU * 6.0 * t).sin();
                0.55 * (TAU * fase * v).sin() + 0.45 * (TAU * fase * v * 1.006).sin()
            }
            // Una "vocal" que sube de golpe: seno con un formante que se mueve.
            Voz::Vidente => {
                let v = 1.0 + 0.03 * (TAU * 5.5 * t).sin();
                0.75 * (TAU * fase * v).sin() + 0.25 * (TAU * fase * v * 2.0).sin() * (0.5 + 0.5 * (TAU * 3.0 * t).sin())
            }
            Voz::Erizo => if (fase * (1.0 + 2.0 * t)).fract() < 0.4 { 0.8 } else { -0.8 },
            Voz::Gollum => {
                let r = ((t * 9137.0 * f).sin() * 43758.5).fract() * 2.0 - 1.0;
                let gargara = 0.6 + 0.4 * (TAU * 28.0 * t).sin();
                (0.6 * (2.0 * fase.fract() - 1.0) + 0.4 * r) * gargara
            }
            Voz::Mensaje => (-t * 30.0).exp() * 3.0 * ((TAU * fase).sin() + 0.4 * (TAU * fase * 2.76).sin()),
            Voz::Abogado => {
                let fase = t * f * (1.0 - 1.5 * t);
                if fase.fract() < 0.5 { 0.7 } else { -0.7 }
            }
            Voz::Ganso => {
                let fase = t * f * (1.0 - 0.8 * t);
                let diente = 2.0 * fase.fract() - 1.0;
                0.6 * diente + 0.4 * (TAU * fase * 2.0).sin() * diente.signum()
            }
            Voz::Megafono => {
                let s = (TAU * fase).sin() + 0.5 * (TAU * fase * 2.0).sin() + 0.3 * (TAU * fase * 3.0).sin();
                (3.0 * s).tanh() * 0.7 * (0.7 + 0.3 * (TAU * 1150.0 * t).sin())
            }
            Voz::Animalito => {
                let f2 = 1.0 + 0.5 * (t / 0.045);
                0.7 * (TAU * fase * f2).sin() + 0.3 * (TAU * fase * 3.0).sin()
            }
        };
        // La de GLADAS va mas bajita: es la que mas habla.
        let volumen = if voz == Voz::Ia { 0.2 } else { 0.32 };
        volumen * env * s
    })
}

fn efecto(e: Efecto) -> Vec<f32> {
    let mut semilla: u32 = 0x5F00_1234u32.wrapping_add(e as u32);
    let mut ruido = move || {
        semilla = semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (semilla >> 8) as f32 / (1u32 << 23) as f32 - 1.0
    };
    let tono = |f: f32, t: f32| (TAU * f * t).sin();
    match e {
        // Dos notas hacia arriba.
        Efecto::Acierto => generar(0.18, |t| 0.4 * (-t * 14.0).exp() * tono(if t < 0.06 { 880.0 } else { 1320.0 }, t)),
        // Tres notas y un brillo.
        Efecto::Perfecto => generar(0.3, |t| {
            let f = if t < 0.06 { 1046.0 } else if t < 0.12 { 1318.0 } else { 1568.0 };
            0.4 * (-t * 9.0).exp() * (tono(f, t) + 0.3 * tono(f * 2.0, t))
        }),
        // Un zumbido que baja.
        Efecto::Fallo => generar(0.25, |t| 0.3 * (-t * 8.0).exp() * if (t * (200.0 - 300.0 * t)).fract() < 0.5 { 1.0 } else { -1.0 }),
        // Un silbido que sube y se va: el hiperespacio.
        Efecto::Salto => generar(1.4, |t| {
            let f = 200.0 + 1400.0 * (t / 1.4).powi(2);
            0.25 * (t / 0.2).min(1.0) * (1.0 - t / 1.4) * (tono(f, t) * 0.5 + ruido() * 0.5)
        }),
        // Un golpe sordo.
        Efecto::Choque => generar(0.35, |t| 0.6 * (-t * 12.0).exp() * (ruido() * 0.6 + tono(70.0 - 40.0 * t, t))),
        Efecto::Reglazo => generar(0.22, |t| {
            0.7 * (-t * 35.0).exp() * (0.65 * ruido() + 0.25 * tono(930.0, t) + 0.1 * tono(1850.0, t))
        }),
        // Dos golpes graves: lub... dub.
        Efecto::Latido => generar(0.55, |t| {
            let golpe = |t0: f32, fuerza: f32| if t >= t0 { fuerza * (-(t - t0) * 22.0).exp() * tono(52.0, t - t0) } else { 0.0 };
            0.9 * (golpe(0.0, 1.0) + golpe(0.17, 0.7))
        }),
        // Un trueno: un chasquido y luego el retumbo grave que va y viene.
        Efecto::Trueno => {
            let mut lp = 0.0f32;
            let mut lp2 = 0.0f32;
            generar(2.6, |t| {
                let r = ruido();
                lp += (r - lp) * 0.05;
                lp2 += (lp - lp2) * 0.05;
                let chasquido = if t < 0.12 { (1.0 - t / 0.12) * r * 0.5 } else { 0.0 };
                let retumbo = (t / 0.15).min(1.0) * (-t * 1.3).exp() * (0.7 + 0.3 * (t * 9.0).sin() * (t * 3.1).sin());
                (chasquido + 9.0 * lp2 * retumbo).clamp(-1.0, 1.0) * 0.9
            })
        }
        // La notificacion: dos notitas redondas (la segunda mas aguda).
        Efecto::Whatsapp => generar(0.32, |t| {
            let nota = |t0: f32, f: f32| if t >= t0 { (-(t - t0) * 18.0).exp() * (tono(f, t - t0) + 0.3 * tono(f * 2.0, t - t0)) } else { 0.0 };
            0.35 * (nota(0.0, 880.0) + nota(0.11, 1318.5))
        }),
        // El chorro: ruido que pasa por un filtro (siseo de agua), cortito y que se apaga.
        Efecto::Chorro => {
            let mut lp = 0.0f32;
            generar(0.09, |t| {
                lp += (ruido() - lp) * 0.35;
                0.22 * (1.0 - t / 0.09) * (lp + 0.15 * tono(900.0 + 400.0 * t, t))
            })
        }
        // ¡WACK!: se arma como una voz de caricatura. Una cuerda que vibra pasa por dos resonancias
        // (como la boca): arranca en "u" y se abre a "a" (eso es la W), y corta seco en una K.
        Efecto::Wack | Efecto::Wack2 | Efecto::Wack3 => {
            let tono_base = match e {
                Efecto::Wack => 235.0,
                Efecto::Wack2 => 200.0,
                _ => 275.0,
            };
            voz_formantes(0.27, 0.205, tono_base, -150.0, |t| {
                let abre = (t / 0.05).min(1.0);
                (320.0 + 480.0 * abre, 750.0 + 620.0 * abre)
            }, 0.8)
                .into_iter()
                .enumerate()
                .map(|(i, m)| {
                    let t = i as f32 / FRECUENCIA as f32;
                    // La K: un chasquido corto al final.
                    let k = if (0.215..0.25).contains(&t) { 0.5 * ruido() * (-(t - 0.215) * 110.0).exp() } else { 0.0 };
                    m + k
                })
                .collect()
        }
        // ¡A-NI-LLA-ZO!: la boca va de vocal en vocal, con la Z de soplido. Tono de locutor de feria.
        Efecto::GritoAnillazo => {
            const VOCALES: [(f32, f32, f32); 6] = [(0.0, 820.0, 1250.0), (0.1, 300.0, 1500.0), (0.15, 320.0, 2300.0), (0.24, 320.0, 2100.0), (0.3, 820.0, 1250.0), (0.46, 470.0, 860.0)];
            let boca = |t: f32| {
                let k = VOCALES.iter().rposition(|v| v.0 <= t).unwrap_or(0);
                let (a, b) = (VOCALES[k], VOCALES[(k + 1).min(5)]);
                let m = if b.0 > a.0 { ((t - a.0) / (b.0 - a.0)).clamp(0.0, 1.0) } else { 1.0 };
                let m = (m * 2.5).min(1.0);
                (a.1 + (b.1 - a.1) * m, a.2 + (b.2 - a.2) * m)
            };
            voz_formantes(0.7, 0.66, 210.0, -70.0, boca, 0.62)
                .into_iter()
                .enumerate()
                .map(|(i, m)| {
                    let t = i as f32 / FRECUENCIA as f32;
                    // La Z: un silbido antes de la O.
                    let z = if (0.4..0.47).contains(&t) { 0.16 * ruido() } else { 0.0 };
                    m + z
                })
                .collect()
        }
        // Un anillo que pasa zumbando: un soplo cortito.
        Efecto::Pase => generar(0.09, |t| 0.13 * (t / 0.09 * std::f32::consts::PI).sin() * (0.6 * ruido() + 0.4 * tono(190.0 + 900.0 * t, t))),
        // Un roce: chispazo agudo que baja, como metal contra metal.
        Efecto::Roce => generar(0.2, |t| (-t * 13.0).exp() * (0.3 * ruido() * (t * 300.0).sin().abs() + 0.26 * tono(2300.0 - 5000.0 * t, t))),
        // El disco que se raya: el chirrido va y viene, como la mano de un DJ.
        Efecto::Scratch => {
            let mut fase = 0.0f32;
            generar(0.42, |t| {
                let mano = 900.0 + 800.0 * (t * 31.0).sin() * if t < 0.2 { 1.0 } else { 0.5 };
                fase = (fase + mano.abs() / FRECUENCIA as f32).fract();
                let sierra = 2.0 * fase - 1.0;
                (1.0 - t / 0.42) * (0.26 * sierra + 0.24 * ruido())
            })
        }
        // El mazo contra la madera: un toc hueco.
        Efecto::Tablero => generar(0.12, |t| 0.5 * (-t * 38.0).exp() * (tono(190.0 - 300.0 * t, t) + 0.5 * tono(410.0, t)) + if t < 0.005 { 0.3 * ruido() } else { 0.0 }),
        // ¡Ay!: de "a" a "i", agudito y subiendo (es chiquito).
        Efecto::KochAy => voz_formantes(0.2, 0.18, 430.0, 380.0, |t| {
            let k = (t / 0.16).min(1.0);
            (820.0 - 450.0 * k, 1250.0 + 1050.0 * k)
        }, 0.34),
        // Je-je: dos golpecitos de "e", burlones.
        Efecto::KochRisa => {
            let mut v = voz_formantes(0.11, 0.09, 380.0, -120.0, |_| (480.0, 1800.0), 0.3);
            v.extend(std::iter::repeat_n(0.0, (0.03 * FRECUENCIA as f32) as usize));
            v.extend(voz_formantes(0.11, 0.09, 430.0, -150.0, |_| (480.0, 1800.0), 0.3));
            v
        }
        // El mazazo: un golpe grave que baja, el chasquido de la goma y un toque de madera.
        Efecto::Mazazo => generar(0.17, |t| {
            let chasquido = if t < 0.008 { ruido() * (1.0 - t / 0.008) } else { 0.0 };
            0.55 * chasquido + 0.75 * (-t * 26.0).exp() * tono(165.0 - 620.0 * t, t) + 0.22 * (-t * 60.0).exp() * tono(520.0, t)
        }),
        // El perfecto: el mismo golpe, con un grave que dura mas y una campanita encima.
        Efecto::MazazoFuerte => generar(0.24, |t| {
            let chasquido = if t < 0.008 { ruido() * (1.0 - t / 0.008) } else { 0.0 };
            0.55 * chasquido + 0.8 * (-t * 19.0).exp() * tono(150.0 - 420.0 * t, t) + 0.2 * (-t * 60.0).exp() * tono(520.0, t) + 0.1 * (-t * 22.0).exp() * tono(1040.0, t)
        }),
        // Al aire: un soplido corto.
        Efecto::Aire => generar(0.13, |t| 0.22 * (t / 0.13 * std::f32::consts::PI).sin() * ruido()),
        Efecto::KochSale => generar(0.07, |t| 0.16 * (1.0 - t / 0.07) * tono(420.0 + 5200.0 * t, t)),
        // ¡RECURSION!: un arpegio que se repite cada vez mas agudo, y un barrido.
        Efecto::Recursion => generar(0.95, |t| {
            let paso = (t / 0.055) as i32;
            let nota = [0, 4, 7, 12][paso.rem_euclid(4) as usize] + paso / 4 * 12;
            let f = 330.0 * 2f32.powf(nota as f32 / 12.0);
            let cuadrada = if tono(f, t) > 0.0 { 1.0 } else { -1.0 };
            (1.0 - t / 0.95) * (0.2 * cuadrada + 0.14 * ruido() * (t / 0.95))
        }),
        // El globo que revienta: un chasquido seco y un "pop" que baja de tono.
        Efecto::Pop => generar(0.16, |t| {
            let chasquido = if t < 0.012 { ruido() * (1.0 - t / 0.012) } else { 0.0 };
            0.55 * chasquido + 0.45 * (-t * 30.0).exp() * tono(520.0 - 1800.0 * t, t)
        }),
        // Teclear: seis tecleos secos, desparejos.
        Efecto::Teclas => generar(0.5, |t| {
            [0.0f32, 0.07, 0.15, 0.2, 0.29, 0.36].iter().map(|&t0| if t >= t0 && t < t0 + 0.03 { 0.4 * (-(t - t0) * 160.0).exp() * (ruido() * 0.7 + 0.3 * tono(2400.0, t)) } else { 0.0 }).sum::<f32>()
        }),
        // Una bisagra oxidada: un tono que sube y baja, rasposo.
        Efecto::Puerta => generar(0.8, |t| {
            let f = 380.0 + 160.0 * (t * 7.0).sin() + 90.0 * (t * 23.0).sin();
            let raspa = if (t * 55.0).fract() < 0.3 { 1.0 } else { 0.4 };
            0.25 * (1.0 - t / 0.8) * raspa * (tono(f, t) + 0.3 * ruido())
        }),
        // Un soplido largo que baja.
        Efecto::Aterrizaje => generar(1.6, |t| {
            let env = (t / 0.3).min(1.0) * (1.0 - t / 1.6);
            0.25 * env * (ruido() * 0.7 + 0.3 * tono(120.0 - 50.0 * t, t))
        }),
        // Un "pum" con chisporroteo.
        Efecto::Fuego => generar(0.6, |t| 0.4 * (-t * 7.0).exp() * (ruido() * (0.5 + 0.5 * (t * 80.0).sin().abs()) + 0.4 * tono(90.0, t))),
        // La sirena de la junta de emergencia: dos tonos que se alternan.
        Efecto::Alarma => generar(1.6, |t| {
            let f = if (t * 4.0) as i32 % 2 == 0 { 880.0 } else { 660.0 };
            0.3 * (1.0 - t / 1.6) * if (t * f).fract() < 0.5 { 1.0 } else { -1.0 }
        }),
        // Un "flap": ruido cortito que baja, como carta contra la mesa.
        Efecto::Carta => generar(0.09, |t| 0.45 * (-t * 45.0).exp() * (ruido() * 0.8 + 0.2 * tono(300.0 - 1500.0 * t, t))),
        Efecto::OcarinaA | Efecto::OcarinaAbajo | Efecto::OcarinaDerecha | Efecto::OcarinaIzquierda | Efecto::OcarinaArriba => {
            // Flauta: seno con un poquito de armonicos, vibrato y el soplido al empezar.
            let nota = match e {
                Efecto::OcarinaA => 62,
                Efecto::OcarinaAbajo => 65,
                Efecto::OcarinaDerecha => 69,
                Efecto::OcarinaIzquierda => 71,
                _ => 74,
            };
            let f = 440.0 * 2f32.powf((nota - 69) as f32 / 12.0);
            generar(0.6, |t| {
                let env = (t / 0.05).min(1.0) * (1.0 - ((t - 0.42) / 0.18).clamp(0.0, 1.0));
                let vib = 1.0 + 0.006 * (TAU * 5.5 * t).sin() * (t / 0.2).min(1.0);
                let fase = f * vib * t;
                let soplido = ruido() * 0.06 * (1.0 - t / 0.12).max(0.0);
                0.3 * env * ((TAU * fase).sin() + 0.15 * (TAU * 2.0 * fase).sin() + 0.05 * (TAU * 3.0 * fase).sin() + soplido)
            })
        }
        Efecto::Rebobinar => generar(2.6, |t| {
            let u = (t * 3.0).fract();
            let f = 300.0 + 2600.0 * u * u;
            let env = (t / 0.1).min(1.0) * (1.0 - ((t - 2.2) / 0.4).clamp(0.0, 1.0));
            0.16 * env * (tono(f, t) * 0.6 + ruido() * 0.4 * u)
        }),
        Efecto::Protesto => generar(1.0, |t| {
            let golpe = (-t * 18.0).exp() * (tono(55.0, t) * 0.8 + ruido() * 0.5);
            let env = ((t - 0.05) / 0.05).clamp(0.0, 1.0) * (1.0 - ((t - 0.6) / 0.4).clamp(0.0, 1.0));
            let diente = |f: f32| 2.0 * (f * t).fract() - 1.0;
            let acorde = (diente(220.0) + diente(277.2) + diente(329.6) + diente(440.0)) * 0.12;
            0.5 * golpe + 0.4 * env * acorde
        }),
        Efecto::Mazo => generar(0.6, |t| {
            let golpe = |t0: f32| if t >= t0 { (-(t - t0) * 40.0).exp() } else { 0.0 };
            let g = golpe(0.0) + golpe(0.18) + golpe(0.36);
            0.5 * g * (tono(180.0, t) * 0.6 + ruido() * 0.4)
        }),
        Efecto::Claqueta => generar(0.12, |t| 0.6 * (-t * 70.0).exp() * (ruido() * 0.6 + tono(2400.0, t) * 0.4)),
        Efecto::Nota0 | Efecto::Nota1 | Efecto::Nota2 | Efecto::Nota3 | Efecto::Nota4 | Efecto::Nota5 | Efecto::Nota6 | Efecto::Nota7 => {
            // Do pentatonica: do, re, mi, sol, la, do, re, mi.
            const ESCALA: [i32; 8] = [72, 74, 76, 79, 81, 84, 86, 88];
            let k = e as usize - Efecto::Nota0 as usize;
            let f = 440.0 * 2f32.powf((ESCALA[k] - 69) as f32 / 12.0);
            generar(0.35, |t| 0.28 * (-t * 9.0).exp() * (tono(f, t) + 0.4 * tono(f * 2.0, t) + 0.15 * tono(f * 3.0, t)))
        }
        Efecto::Rocky => {
            // Cuatro acordes de flauta (seno con un poco de octava), de 0.38 s, subiendo: do, re,
            // mi mayores y el do de arriba.
            let midi = |n: f32| 440.0 * 2f32.powf((n - 69.0) / 12.0);
            let acordes = [[72.0, 76.0, 79.0], [74.0, 78.0, 81.0], [76.0, 80.0, 83.0], [84.0, 88.0, 91.0]];
            generar(1.6, |t| {
                let k = ((t / 0.38) as usize).min(3);
                let local = t - k as f32 * 0.38;
                let env = (local / 0.03).min(1.0) * (-local * 4.0).exp() * if t < 1.52 { 1.0 } else { ((1.6 - t) / 0.08).max(0.0) };
                let vibrato = 1.0 + 0.004 * (t * 33.0).sin();
                0.14 * env * acordes[k].iter().map(|&n| tono(midi(n) * vibrato, t) + 0.25 * tono(midi(n) * 2.0, t)).sum::<f32>()
            })
        }
        Efecto::Monolito => generar(3.4, |t| {
            // Metales: diente de sierra suavizado con su envolvente.
            let metal = |f: f32, t0: f32, dura: f32| {
                if t < t0 {
                    return 0.0;
                }
                let u = t - t0;
                let env = (u / 0.08).min(1.0) * (1.0 - ((u - dura + 0.25) / 0.25).clamp(0.0, 1.0));
                let diente = 2.0 * (f * t).fract() - 1.0;
                env * (0.55 * diente + 0.45 * tono(f, t))
            };
            let notas = metal(130.8, 0.0, 0.7) + metal(196.0, 0.7, 0.7) + metal(261.6, 1.4, 0.75);
            // El acorde: do-mi-sol que se vuelve do-mib-sol (mayor a menor).
            let tercera = if t < 2.4 { 329.6 } else { 311.1 };
            let acorde = metal(261.6, 2.15, 1.2) + metal(tercera, 2.15, 1.2) + metal(392.0, 2.15, 1.2);
            // Los timbales: do y sol graves, alternando, al final.
            let timbal = |t0: f32, f: f32| if t >= t0 { (-(t - t0) * 9.0).exp() * tono(f, t) } else { 0.0 };
            let timbales = timbal(2.2, 65.4) + timbal(2.45, 49.0) + timbal(2.7, 65.4) + timbal(2.95, 49.0) + timbal(3.2, 65.4);
            0.13 * notas + 0.08 * acorde + 0.35 * timbales
        }),
        Efecto::Teseracto => generar(3.8, |t| {
            // Organo: la menor con novena (la, mi, si, do), cada voz con sus armonicos, creciendo.
            let env = (t / 1.6).min(1.0) * ((3.8 - t) / 0.6).clamp(0.0, 1.0);
            let organo: f32 = [110.0, 164.8, 246.9, 261.6].iter().map(|&f| tono(f, t) + 0.5 * tono(f * 2.0, t) + 0.25 * tono(f * 4.0, t)).sum();
            let retumbo = tono(41.2, t) * (0.6 + 0.4 * (t * 2.0).sin()) + 0.3 * ruido();
            0.07 * env * organo + 0.12 * env * retumbo
        }),
        Efecto::Gusano => generar(1.4, |t| {
            let u = t / 1.4;
            let env = (t / 0.15).min(1.0) * (1.0 - u);
            0.2 * env * (ruido() * 0.6 + tono(900.0 * (1.0 - 0.8 * u), t) * 0.4)
        }),
        Efecto::Cometa => generar(1.8, |t| {
            let f = 2400.0 - 1600.0 * (t / 1.8);
            let env = (t / 0.3).min(1.0) * (1.0 - t / 1.8);
            0.12 * env * (tono(f, t) * 0.5 + tono(f * 1.5, t) * 0.3 + ruido() * 0.2)
        }),
        Efecto::Sello => generar(0.9, |t| {
            let golpe = (-t * 30.0).exp() * (tono(90.0, t) * 0.7 + ruido() * 0.3);
            let campana = if t > 0.12 { (-(t - 0.12) * 5.0).exp() * (tono(1568.0, t) + 0.5 * tono(2093.0, t)) } else { 0.0 };
            0.45 * golpe + 0.15 * campana
        }),
        Efecto::Don => generar(0.2, |t| 0.6 * (-t * 16.0).exp() * tono(150.0 * (1.0 - 0.45 * (t / 0.2)), t)),
        Efecto::Ka => generar(0.07, |t| 0.35 * (-t * 60.0).exp() * (tono(1900.0, t) * 0.6 + ruido() * 0.4)),
        // Feliz cumpleanos (dominio publico) con una onda de pulso que tiembla
        // a 30 Hz, como las vibraciones del taladro con que se lo canto Curiosity en 2013.
        Efecto::Cumpleanos => {
            const MELODIA: [(i32, f32); 25] = [
                (67, 0.75), (67, 0.25), (69, 1.0), (67, 1.0), (72, 1.0), (71, 2.0),
                (67, 0.75), (67, 0.25), (69, 1.0), (67, 1.0), (74, 1.0), (72, 2.0),
                (67, 0.75), (67, 0.25), (79, 1.0), (76, 1.0), (72, 1.0), (71, 1.0), (69, 2.0),
                (77, 0.75), (77, 0.25), (76, 1.0), (72, 1.0), (74, 1.0), (72, 3.0),
            ];
            const TIEMPO: f32 = 0.32;
            let mut v = Vec::new();
            for (nota, tiempos) in MELODIA {
                let f = 440.0 * 2f32.powf((nota - 12 - 69) as f32 / 12.0);
                let largo = tiempos * TIEMPO;
                v.extend(generar(largo, |t| {
                    let env = (t / 0.02).min(1.0) * (1.0 - (t - largo + 0.06).max(0.0) / 0.06);
                    let pulso = if (t * f).fract() < 0.3 { 1.0 } else { -1.0 };
                    0.22 * env * pulso * (0.7 + 0.3 * (TAU * 30.0 * t).sin())
                }));
            }
            v
        }
    }
}

/// Un sonido cargado en raylib y su numero en la mezcla de la grabacion.
struct Canal<'a> {
    sonido: Sound<'a>,
    id: usize,
}

/// La mezcla de la grabacion (`--grabar`): en vez de sonar por las bocinas, cada sonido se suma
/// aqui, cuadro por cuadro, al mismo paso fijo que el video. Asi el audio queda clavado con la
/// imagen aunque cada cuadro tarde lo que tarde en dibujarse. Cada canal es una sola voz, como un
/// `Sound` de raylib: si se vuelve a tocar mientras suena, empieza de nuevo.
pub struct Mezcla {
    muestras: Vec<Vec<f32>>,
    voces: Vec<VozMezcla>,
    salida: Vec<f32>,
    /// Lo mismo sin la musica (los canales desde `primera_pista` son las pistas), para editar el
    /// video con otra musica encima.
    efectos: Vec<f32>,
    primera_pista: usize,
    /// Los segundos que ya van (la salida se llena hasta aqui).
    reloj: f64,
}

#[derive(Clone, Copy)]
struct VozMezcla {
    pos: f64,
    paso: f64,
    volumen: f32,
    sonando: bool,
}

impl Mezcla {
    fn tocar(&mut self, id: usize, volumen: f32) {
        let v = &mut self.voces[id];
        (v.pos, v.volumen, v.sonando) = (0.0, volumen, true);
    }

    /// Llena la salida hasta `segundos` mas: lo que suena en ese rato.
    fn avanzar(&mut self, segundos: f32) {
        self.reloj += segundos as f64;
        let hasta = (self.reloj * FRECUENCIA as f64) as usize;
        while self.salida.len() < hasta {
            let (mut m, mut e) = (0.0, 0.0);
            for (id, (v, muestras)) in self.voces.iter_mut().zip(&self.muestras).enumerate() {
                if !v.sonando {
                    continue;
                }
                let k = v.pos as usize;
                if k + 1 >= muestras.len() {
                    v.sonando = false;
                    continue;
                }
                let f = (v.pos - k as f64) as f32;
                let x = (muestras[k] * (1.0 - f) + muestras[k + 1] * f) * v.volumen;
                m += x;
                if id < self.primera_pista {
                    e += x;
                }
                v.pos += v.paso;
            }
            self.salida.push(m);
            self.efectos.push(e);
        }
    }

    /// El `.wav` de todo lo grabado, con un limitador suave (la musica y los efectos juntos
    /// pasan de 1 de vez en cuando).
    pub fn wav(&self) -> Vec<u8> {
        wav(&limitar(&self.salida))
    }

    /// El `.wav` de los efectos y las voces, sin la musica.
    pub fn wav_efectos(&self) -> Vec<u8> {
        wav(&limitar(&self.efectos))
    }
}

/// Un limitador suave: lo que pasa de 0.7 se aplasta hacia 1.
fn limitar(muestras: &[f32]) -> Vec<f32> {
    muestras.iter().map(|&m| if m.abs() < 0.7 { m } else { m.signum() * (0.7 + 0.3 * ((m.abs() - 0.7) / 0.3).tanh()) }).collect()
}

/// Los sonidos ya cargados. Vive mientras viva el dispositivo de audio.
pub struct Sonidos<'a> {
    blips: Vec<(Voz, Vec<Canal<'a>>)>,
    efectos: Vec<Canal<'a>>,
    /// Cada pista con su volumen actual (cambia de a poco: fundidos).
    pistas: Vec<(Pista, Canal<'a>, Cell<f32>)>,
    vol_musica: Cell<f32>,
    vol_efectos: Cell<f32>,
    /// Si se esta grabando: ahi va todo, y por las bocinas no suena nada.
    mezcla: Option<std::cell::RefCell<Mezcla>>,
}

impl<'a> Sonidos<'a> {
    /// Con `grabando`, nada suena: todo se mezcla para el video (ver `Mezcla`).
    pub fn cargar(audio: &'a RaylibAudio, grabando: bool) -> Self {
        let banco = std::cell::RefCell::new(Vec::new());
        let cargar = |muestras: Vec<f32>| {
            let sonido = audio.new_wave_from_memory(".wav", &wav(&muestras)).ok().and_then(|w| audio.new_sound_from_wave(&w).ok())?;
            let mut b = banco.borrow_mut();
            b.push(if grabando { muestras } else { Vec::new() });
            Some(Canal { sonido, id: b.len() - 1 })
        };
        // Ocho alturas por voz, sobre una escala pentatonica: suena a que "habla".
        let escala = [0.0, 2.0, 4.0, 7.0, 9.0, 12.0, 14.0, 16.0];
        // GLADAS brinca menos de nota: casi monotona, como quien habla tranquilo.
        let escala_ia = [0.0, 2.0, 3.0, 5.0, 3.0, 2.0, 0.0, -2.0];
        let blips = [Voz::Ia, Voz::Miku, Voz::Profe, Voz::Pluton, Voz::Planeta, Voz::Rover, Voz::Robotina, Voz::Neptuno, Voz::Grunido, Voz::Radio, Voz::Animalito, Voz::Hada, Voz::Vidente, Voz::Erizo, Voz::Gollum, Voz::Mensaje, Voz::Abogado, Voz::Ganso, Voz::Megafono]
            .into_iter()
            .map(|v| (v, if v == Voz::Ia { &escala_ia } else { &escala }.iter().filter_map(|&s| cargar(blip(v, s))).collect()))
            .collect();
        let efectos = [Efecto::Acierto, Efecto::Perfecto, Efecto::Fallo, Efecto::Salto, Efecto::Choque, Efecto::Aterrizaje, Efecto::Fuego, Efecto::Alarma, Efecto::Cumpleanos, Efecto::Carta, Efecto::Don, Efecto::Ka, Efecto::OcarinaA, Efecto::OcarinaAbajo, Efecto::OcarinaDerecha, Efecto::OcarinaIzquierda, Efecto::OcarinaArriba, Efecto::Rebobinar, Efecto::Protesto, Efecto::Mazo, Efecto::Claqueta, Efecto::Nota0, Efecto::Nota1, Efecto::Nota2, Efecto::Nota3, Efecto::Nota4, Efecto::Nota5, Efecto::Nota6, Efecto::Nota7, Efecto::Cometa, Efecto::Sello, Efecto::Reglazo, Efecto::Latido, Efecto::Trueno, Efecto::Puerta, Efecto::Whatsapp, Efecto::Teclas, Efecto::Chorro, Efecto::Pop, Efecto::Rocky, Efecto::Monolito, Efecto::Teseracto, Efecto::Gusano, Efecto::Mazazo, Efecto::MazazoFuerte, Efecto::Aire, Efecto::KochSale, Efecto::Recursion, Efecto::Wack, Efecto::Wack2, Efecto::Wack3, Efecto::Tablero, Efecto::KochAy, Efecto::KochRisa, Efecto::Pase, Efecto::Roce, Efecto::Scratch, Efecto::GritoAnillazo]
            .into_iter()
            .filter_map(|e| cargar(efecto(e)))
            .collect();
        let primera_pista = banco.borrow().len();
        let mut pistas: Vec<(Pista, Canal<'a>, Cell<f32>)> = [(Pista::Cancion, cancion::sintetizar()), (Pista::Espacio, cancion::sintetizar_espacio()), (Pista::Fuente, cancion::sintetizar_fuente()), (Pista::Tarot, cancion::sintetizar_tarot()), (Pista::TarotReves, cancion::sintetizar_tarot().into_iter().rev().collect()), (Pista::Juicio, cancion::sintetizar_juicio()), (Pista::Arcade, cancion::sintetizar_arcade()), (Pista::Lofi, cancion::sintetizar_lofi()), (Pista::Marimba, cancion::sintetizar_marimba()), (Pista::Koch, cancion::sintetizar_koch()), (Pista::Anillos, cancion::sintetizar_anillos())]
            .into_iter()
            .filter_map(|(p, muestras)| cargar(muestras).map(|s| (p, s, Cell::new(0.0))))
            .collect();
        // La de la cinematica: la cancion local si esta (y no se graba); si no, el piano original.
        let local = if grabando || !hay_cancion_del_prologo() { None } else { audio.new_sound(CANCION_DEL_PROLOGO).ok() };
        match local {
            Some(sonido) => pistas.push((Pista::Prologo, Canal { sonido, id: 0 }, Cell::new(0.0))),
            None => pistas.extend(cargar(cancion::sintetizar_prologo()).map(|s| (Pista::Prologo, s, Cell::new(0.0)))),
        }
        let muestras = banco.into_inner();
        let mezcla = grabando.then(|| {
            let voces = vec![VozMezcla { pos: 0.0, paso: 1.0, volumen: 1.0, sonando: false }; muestras.len()];
            std::cell::RefCell::new(Mezcla { muestras, voces, salida: Vec::new(), efectos: Vec::new(), primera_pista, reloj: 0.0 })
        });
        Sonidos { blips, efectos, pistas, vol_musica: Cell::new(1.0), vol_efectos: Cell::new(1.0), mezcla }
    }

    /// Grabando: suma a la mezcla lo que suena en los siguientes `segundos` (un cuadro del video).
    pub fn grabar(&self, segundos: f32) {
        if let Some(m) = &self.mezcla {
            m.borrow_mut().avanzar(segundos);
        }
    }

    /// La mezcla de la grabacion, si se esta grabando.
    pub fn mezcla(&self) -> Option<std::cell::Ref<'_, Mezcla>> {
        self.mezcla.as_ref().map(|m| m.borrow())
    }

    /// Toca un canal (por las bocinas, o en la mezcla si se graba).
    fn tocar(&self, c: &Canal, volumen: f32) {
        match &self.mezcla {
            Some(m) => m.borrow_mut().tocar(c.id, volumen),
            None => {
                c.sonido.set_volume(volumen);
                c.sonido.play();
            }
        }
    }

    fn parar(&self, c: &Canal) {
        match &self.mezcla {
            Some(m) => m.borrow_mut().voces[c.id].sonando = false,
            None => c.sonido.stop(),
        }
    }

    fn sonando(&self, c: &Canal) -> bool {
        match &self.mezcla {
            Some(m) => m.borrow().voces[c.id].sonando,
            None => c.sonido.is_playing(),
        }
    }

    fn volumen(&self, c: &Canal, volumen: f32) {
        match &self.mezcla {
            Some(m) => m.borrow_mut().voces[c.id].volumen = volumen,
            None => c.sonido.set_volume(volumen),
        }
    }

    fn velocidad(&self, c: &Canal, paso: f32) {
        match &self.mezcla {
            Some(m) => m.borrow_mut().voces[c.id].paso = paso as f64,
            None => c.sonido.set_pitch(paso),
        }
    }

    /// El volumen de la musica y el de los efectos (y voces), de 0 a 1 (aparte del general).
    pub fn volumenes(&self, musica: f32, efectos: f32) {
        self.vol_musica.set(musica.clamp(0.0, 1.0));
        self.vol_efectos.set(efectos.clamp(0.0, 1.0));
    }

    pub fn efecto(&self, e: Efecto) {
        self.efecto_a(e, 1.0);
    }

    /// Un efecto mas bajito (de 0 a 1): lo que suena lejos, como la regla de Koch.
    pub fn efecto_a(&self, e: Efecto, volumen: f32) {
        if let Some(s) = self.efectos.get(e as usize) {
            self.tocar(s, self.vol_efectos.get() * volumen.clamp(0.0, 1.0));
        }
    }

    /// El blip de una letra: la altura sale de la letra, asi cada palabra
    /// suena siempre igual.
    pub fn blip(&self, voz: Voz, letra: char) {
        // La voz "de Miku" es la del protagonista: la del personaje escogido.
        let tono = if voz == Voz::Miku { TONO_PROTAGONISTA.with(|t| t.get()) } else { 1.0 };
        let voz = if voz == Voz::Miku { VOZ_PROTAGONISTA.with(|v| v.get()) } else { voz };
        if let Some((_, sonidos)) = self.blips.iter().find(|(v, _)| *v == voz) {
            if !sonidos.is_empty() {
                let i = (letra.to_ascii_lowercase() as usize * 7) % sonidos.len();
                self.velocidad(&sonidos[i], tono);
                self.tocar(&sonidos[i], self.vol_efectos.get());
            }
        }
    }

    /// Una pista desde el principio y a todo volumen, sin fundido (el
    /// minijuego va sincronizado con ella).
    pub fn reiniciar(&self, pista: Pista) {
        for (p, s, v) in &self.pistas {
            if *p == pista {
                self.parar(s);
                v.set(1.0);
                self.tocar(s, self.vol_musica.get());
            }
        }
    }

    /// Que tan rapido suena una pista (1: normal). En la entrega, el lo-fi se acelera al final.
    pub fn ritmo(&self, pista: Pista, velocidad: f32) {
        for (p, s, _) in &self.pistas {
            if *p == pista {
                self.velocidad(s, velocidad);
            }
        }
    }

    /// Cada cuadro: la pista `objetivo` sube y las demas bajan hasta callarse.
    /// Con `bucle`, la que suena vuelve a empezar al acabarse.
    pub fn musica(&self, objetivo: Option<Pista>, bucle: bool, dt: f32) {
        for (p, s, v) in &self.pistas {
            let sube = Some(*p) == objetivo;
            let paso = dt / FUNDIDO_MUSICA;
            let nuevo = (v.get() + if sube { paso } else { -paso }).clamp(0.0, 1.0);
            v.set(nuevo);
            self.volumen(s, nuevo * self.vol_musica.get());
            if sube && bucle && !self.sonando(s) {
                self.tocar(s, nuevo * self.vol_musica.get());
            }
            if !sube && nuevo <= 0.0 && self.sonando(s) {
                self.parar(s);
            }
        }
    }
}

/// Un texto que aparece letra por letra, con un blip cada dos letras.
pub struct Dialogo {
    texto: String,
    letras: f32,
    sonadas: usize,
}

/// Letras por segundo de los dialogos (se cambia en los ajustes; se guarda como bits de f32).
static VELOCIDAD: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x4218_0000); // 38.0

/// Que tan rapido salen las letras de los dialogos.
pub fn velocidad_del_texto(letras_por_segundo: f32) {
    VELOCIDAD.store(letras_por_segundo.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

impl Dialogo {
    pub fn new() -> Self {
        Dialogo { texto: String::new(), letras: 0.0, sonadas: 0 }
    }

    /// Avanza y devuelve lo que se ve del texto. Si `texto` cambio, empieza de nuevo.
    pub fn mostrar(&mut self, texto: &str, voz: Voz, dt: f32, sonidos: Option<&Sonidos>) -> &str {
        // Con el nombre del personaje ya puesto, para que se escriba letra por letra bien. Si habla
        // el protagonista, lo dice a su manera (con su muletilla, o con su sonido y la traduccion).
        let propio = if voz == Voz::Miku { crate::ui::como_lo_dice_el_protagonista(texto) } else { None };
        self.mostrar_literal(propio.as_deref().unwrap_or(texto), voz, dt, sonidos)
    }

    /// Una replica ya escrita para el piloto: evita volver a añadir traduccion o muletillas.
    pub fn mostrar_literal(&mut self, texto: &str, voz: Voz, dt: f32, sonidos: Option<&Sonidos>) -> &str {
        let texto = &*crate::ui::con_protagonista(texto);
        if self.texto != texto {
            self.texto = texto.to_string();
            self.letras = 0.0;
            self.sonadas = 0;
        }
        let velocidad = f32::from_bits(VELOCIDAD.load(std::sync::atomic::Ordering::Relaxed));
        self.letras += dt * velocidad;
        let visibles = (self.letras as usize).min(self.texto.chars().count());
        // "De golpe": todo el texto sale junto, con un solo blip (no cien a la vez).
        if velocidad > 1000.0 && self.sonadas < visibles {
            if let (Some(s), Some(c)) = (sonidos, self.texto.chars().find(|c| c.is_alphanumeric())) {
                if self.sonadas == 0 {
                    s.blip(voz, c);
                }
            }
            self.sonadas = visibles;
        }
        // Un blip por cada dos letras nuevas (las que no son espacios ni signos);
        // GLADAS, uno cada tres, que habla mucho y asi no satura.
        let cada = if voz == Voz::Ia { 3 } else { 2 };
        for (k, c) in self.texto.chars().enumerate().take(visibles).skip(self.sonadas) {
            if c.is_alphanumeric() && k % cada == 0 {
                if let Some(s) = sonidos {
                    s.blip(voz, c);
                }
            }
        }
        self.sonadas = visibles;
        let fin = self.texto.char_indices().nth(visibles).map_or(self.texto.len(), |(i, _)| i);
        &self.texto[..crate::ui::sin_partir_botones(&self.texto, fin)]
    }
}

#[cfg(test)]
mod pruebas_del_wack {
    use super::*;

    #[test]
    fn el_wack_y_compania_suenan_y_no_se_salen() {
        for e in [Efecto::Wack, Efecto::Wack2, Efecto::Wack3, Efecto::Mazazo, Efecto::MazazoFuerte, Efecto::Aire, Efecto::Tablero, Efecto::KochAy, Efecto::KochRisa, Efecto::KochSale, Efecto::Recursion, Efecto::Pase, Efecto::Roce, Efecto::Scratch, Efecto::GritoAnillazo] {
            let m = efecto(e);
            assert!(!m.is_empty(), "{e:?} esta vacio");
            let pico = m.iter().fold(0.0f32, |p, x| p.max(x.abs()));
            assert!(m.iter().all(|x| x.is_finite()), "{e:?} tiene basura");
            assert!(pico > 0.1, "{e:?} no se oye: pico {pico}");
            assert!(pico < 1.6, "{e:?} satura: pico {pico}");
        }
    }

    /// La K del WACK: la voz se corta y hay un chasquido al final; la W: arranca oscura y se abre.
    #[test]
    fn el_wack_tiene_su_w_y_su_k() {
        let m = efecto(Efecto::Wack);
        let f = FRECUENCIA as f32;
        let energia = |a: f32, b: f32| {
            let t = &m[(a * f) as usize..(b * f) as usize];
            t.iter().map(|x| x * x).sum::<f32>() / t.len() as f32
        };
        assert!(energia(0.05, 0.18) > energia(0.207, 0.214) * 4.0, "antes de la K debe callarse");
        assert!(energia(0.215, 0.225) > energia(0.207, 0.214), "la K debe chasquear");
        // La W: al principio la boca esta cerrada ("u": suena grave) y luego se abre ("a": suben
        // las resonancias). Se mide cuanto suena en los agudos contra los graves.
        let banda = |a: f32, b: f32, hz: f32| {
            let t = &m[(a * f) as usize..(b * f) as usize];
            let (mut re, mut im) = (0.0f32, 0.0f32);
            for (i, x) in t.iter().enumerate() {
                let w = TAU * hz * i as f32 / f;
                re += x * w.cos();
                im += x * w.sin();
            }
            (re * re + im * im).sqrt() / t.len() as f32
        };
        let brillo = |a: f32, b: f32| (banda(a, b, 800.0) + banda(a, b, 1370.0)) / (banda(a, b, 320.0) + banda(a, b, 480.0)).max(1e-5);
        assert!(brillo(0.09, 0.17) > brillo(0.006, 0.03), "la W debe abrirse a la A: {} contra {}", brillo(0.09, 0.17), brillo(0.006, 0.03));
    }
}
