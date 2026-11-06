//! La cancion de Saturno Ritmo: partitura y sintetizador.
//!
//! Es una cancion original, hecha aqui con osciladores (nada de archivos de
//! audio ni musica con derechos). De la MISMA partitura salen el audio y las
//! notas del minijuego: cada `P` del mapa es un golpe de bajo y una nota en
//! el carril del piso, cada `A` es una nota de la melodia y una nota en el
//! carril del aire. Por eso caen exacto en el ritmo.
//!
//! 120 BPM, 16 compases de 4 tiempos: 32 segundos. Cada compas se divide en
//! 8 corcheas de 0.25 s.

use std::f32::consts::TAU;

pub const BPM: f32 = 120.0;
pub const FRECUENCIA: u32 = 44_100;
pub const COMPASES: usize = 16;
/// Segundos por corchea.
pub const CORCHEA: f32 = 60.0 / BPM / 2.0;
pub const DURACION: f32 = COMPASES as f32 * 8.0 * CORCHEA;

/// Un acorde por compas: la menor, fa, do, sol (vi-IV-I-V), en notas MIDI.
const ACORDES: [(i32, [i32; 3]); 4] = [(45, [0, 3, 7]), (41, [0, 4, 7]), (48, [0, 4, 7]), (43, [0, 4, 7])];

/// El mapa de cada compas: 8 corcheas, `P` piso, `A` aire, `.` nada.
/// Compases 0 y 1: intro (solo bateria y bajo, para la cuenta regresiva).
pub fn mapa(compas: usize) -> &'static str {
    const ESTROFA: [&str; 4] = ["P...P...", "P...P.A.", "P.A.P...", "P...P.AA"];
    const CORO: [&str; 4] = ["P.A.P.A.", "PA.APA.A", "P.AAP.A.", "PAPAPAA."];
    match compas {
        0 | 1 => "........",
        2..=7 => ESTROFA[(compas - 2) % 4],
        8..=13 => CORO[(compas - 8) % 4],
        14 => "P...A...",
        _ => "P.......",
    }
}

/// Las notas del minijuego: (segundo, es del aire).
pub fn notas() -> Vec<(f32, bool)> {
    let mut v = Vec::new();
    for compas in 0..COMPASES {
        for (s, c) in mapa(compas).chars().enumerate() {
            let t = (compas * 8 + s) as f32 * CORCHEA;
            match c {
                'P' => v.push((t, false)),
                'A' => v.push((t, true)),
                _ => {}
            }
        }
    }
    v
}

fn midi(n: i32) -> f32 {
    440.0 * 2f32.powf((n - 69) as f32 / 12.0)
}

/// Ruido blanco determinista.
struct Ruido(u32);
impl Ruido {
    fn sig(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 8) as f32 / (1u32 << 23) as f32 - 1.0
    }
}

/// Suma un sonido en `pista` a partir del segundo `desde`. `f(t)` da la
/// muestra a los `t` segundos de empezar y dura `largo` segundos.
fn sumar(pista: &mut [f32], desde: f32, largo: f32, mut f: impl FnMut(f32) -> f32) {
    let i0 = (desde * FRECUENCIA as f32) as usize;
    let n = (largo * FRECUENCIA as f32) as usize;
    for k in 0..n {
        if let Some(m) = pista.get_mut(i0 + k) {
            *m += f(k as f32 / FRECUENCIA as f32);
        }
    }
}

fn cuadrada(fase: f32, ciclo: f32) -> f32 {
    if fase.fract() < ciclo { 1.0 } else { -1.0 }
}

fn triangular(fase: f32) -> f32 {
    4.0 * (fase.fract() - 0.5).abs() - 1.0
}

/// Sintetiza la cancion entera: bateria, bajo, colchon de acordes y melodia.
pub fn sintetizar() -> Vec<f32> {
    let mut pista = vec![0.0f32; (DURACION * FRECUENCIA as f32) as usize + FRECUENCIA as usize];
    let mut ruido = Ruido(0xBEA7_0001);
    for compas in 0..COMPASES {
        let (raiz, acorde) = ACORDES[compas % 4];
        let inicio = compas as f32 * 8.0 * CORCHEA;
        let mapa = mapa(compas).as_bytes();
        // Colchon: el acorde suave todo el compas.
        for &intervalo in &acorde {
            let f = midi(raiz + 12 + intervalo);
            sumar(&mut pista, inicio, 8.0 * CORCHEA, |t| {
                let env = (t / 0.3).min(1.0) * (1.0 - t / (8.0 * CORCHEA)).max(0.0);
                0.035 * env * (triangular(t * f) + 0.3 * (TAU * t * f * 2.0).sin())
            });
        }
        for s in 0..8 {
            let t0 = inicio + s as f32 * CORCHEA;
            let marca = mapa[s];
            // Bombo en cada tiempo: un seno que cae de 150 a 50 Hz.
            if s % 2 == 0 {
                sumar(&mut pista, t0, 0.18, |t| {
                    let f = 50.0 + 100.0 * (-t * 30.0).exp();
                    0.55 * (-t * 18.0).exp() * (TAU * f * t).sin()
                });
            }
            // Caja en el 2 y el 4: ruido con un golpe de tono.
            if s == 2 || s == 6 {
                let mut r = Ruido(ruido.0 ^ (s as u32 * 977));
                sumar(&mut pista, t0, 0.16, |t| 0.25 * (-t * 22.0).exp() * (r.sig() * 0.8 + 0.4 * (TAU * 190.0 * t).sin()));
            }
            // Hi-hat en las corcheas de en medio: ruido cortito y agudo.
            if s % 2 == 1 {
                let mut previo = 0.0;
                sumar(&mut pista, t0, 0.05, |t| {
                    let x = ruido.sig();
                    let agudo = x - previo;
                    previo = x;
                    0.08 * (-t * 70.0).exp() * agudo
                });
            }
            // Bajo en corcheas, mas fuerte donde hay nota del piso.
            if compas >= 1 {
                let f = midi(raiz - 12 + if s % 4 == 3 { 7 } else { 0 });
                let fuerza = if marca == b'P' { 0.32 } else { 0.18 };
                sumar(&mut pista, t0, CORCHEA * 0.9, |t| fuerza * (-t * 6.0).exp() * (0.7 * triangular(t * f) + 0.3 * cuadrada(t * f, 0.5)));
            }
            // Melodia: cada nota del aire es una nota del arpegio, una octava arriba.
            if marca == b'A' {
                let grado = acorde[(compas + s) % 3] + if s >= 4 { 12 } else { 0 };
                let f = midi(raiz + 24 + grado);
                sumar(&mut pista, t0, 0.3, |t| {
                    let vibrato = 1.0 + 0.004 * (TAU * 6.0 * t).sin();
                    0.16 * (-t * 7.0).exp() * cuadrada(t * f * vibrato, 0.25)
                });
            } else if compas >= 8 && compas <= 13 && s % 2 == 1 {
                // En el coro, notitas suaves de relleno entre las del aire.
                let f = midi(raiz + 24 + acorde[s % 3]);
                sumar(&mut pista, t0, 0.15, |t| 0.05 * (-t * 14.0).exp() * cuadrada(t * f, 0.5));
            }
        }
    }
    // Saturacion suave para que nada truene.
    for m in pista.iter_mut() {
        *m = (*m * 1.1).tanh() * 0.8;
    }
    pista
}

/// La musica de fondo para volar: tranquila y en bucle. Colchon de acordes
/// soñadores (fa, mi menor, re menor y do con septimas), un arpegio suave y
/// campanitas de vez en cuando. 8 compases de 4 segundos que se repiten sin
/// corte: cada acorde se apaga justo cuando entra el siguiente.
pub fn sintetizar_espacio() -> Vec<f32> {
    const ACORDES_ESPACIO: [(i32, [i32; 4]); 4] = [(53, [0, 4, 7, 11]), (52, [0, 3, 7, 10]), (50, [0, 3, 7, 10]), (48, [0, 4, 7, 11])];
    let compas = 4.0;
    let total = 8.0 * compas;
    let mut pista = vec![0.0f32; (total * FRECUENCIA as f32) as usize];
    let largo = pista.len();
    // Para que el bucle no tenga corte, lo que se pasa del final se suma al principio.
    let mut sumar_ciclico = |desde: f32, dura: f32, f: &dyn Fn(f32) -> f32| {
        let i0 = (desde * FRECUENCIA as f32) as usize;
        for k in 0..(dura * FRECUENCIA as f32) as usize {
            pista[(i0 + k) % largo] += f(k as f32 / FRECUENCIA as f32);
        }
    };
    for c in 0..8 {
        let (raiz, acorde) = ACORDES_ESPACIO[c % 4];
        let inicio = c as f32 * compas;
        // Colchon: cada voz un poco desafinada, sube despacio y se cruza con el siguiente acorde.
        for (v, &intervalo) in acorde.iter().enumerate() {
            let f = midi(raiz + intervalo) * (1.0 + 0.002 * (v as f32 - 1.5));
            sumar_ciclico(inicio, compas + 1.0, &|t: f32| {
                let env = (t / 1.2).min(1.0) * (1.0 - ((t - compas) / 1.0).clamp(0.0, 1.0));
                0.045 * env * (triangular(t * f) * 0.7 + 0.3 * (TAU * t * f * 0.5).sin())
            });
        }
        // Bajo: la raiz grave, larga y redonda.
        let fb = midi(raiz - 12);
        sumar_ciclico(inicio, compas, &|t: f32| 0.09 * (t / 0.3).min(1.0) * (1.0 - t / compas) * (TAU * fb * t).sin());
        // Arpegio en corcheas, como gotitas.
        for s in 0..8 {
            let nota = raiz + 12 + acorde[(s * 3 + c) % 4] + if s % 4 == 3 { 12 } else { 0 };
            let f = midi(nota);
            sumar_ciclico(inicio + s as f32 * 0.5, 0.45, &|t: f32| 0.05 * (-t * 7.0).exp() * (TAU * f * t).sin());
        }
        // Una campanita cada dos compases.
        if c % 2 == 1 {
            let f = midi(raiz + 24 + acorde[2]);
            sumar_ciclico(inicio + 2.0, 2.0, &|t: f32| 0.05 * (-t * 2.5).exp() * ((TAU * f * t).sin() + 0.4 * (TAU * f * 2.76 * t).sin()));
        }
    }
    for m in pista.iter_mut() {
        *m = (*m * 1.2).tanh() * 0.8;
    }
    pista
}

/// Una cuerda pulsada (algoritmo de Karplus-Strong): un poco de ruido que
/// da vueltas en una linea de retardo del largo de un periodo, promediandose
/// consigo mismo. Suena a arpa.
fn cuerda(f: f32, dura: f32, semilla: u32) -> Vec<f32> {
    let n = (FRECUENCIA as f32 / f).max(2.0) as usize;
    let mut ruido = Ruido(semilla | 1);
    let mut linea: Vec<f32> = (0..n).map(|_| ruido.sig() * 0.5).collect();
    let total = (dura * FRECUENCIA as f32) as usize;
    let mut v = Vec::with_capacity(total);
    for i in 0..total {
        let a = linea[i % n];
        let b = linea[(i + 1) % n];
        let nuevo = 0.5 * (a + b) * 0.996;
        linea[i % n] = nuevo;
        v.push(a);
    }
    v
}

/// La musica de la Gran Fuente de las Hadas: original, de arpa, con ese aire
/// de cuento (arpegios que suben y bajan sobre acordes de septima mayor, un
/// colchon suave y campanitas). 8 compases que se repiten sin corte.
pub fn sintetizar_fuente() -> Vec<f32> {
    // fa, mi menor, re menor, do, si bemol, la menor, sol menor, do: con septimas.
    const ACORDES: [(i32, [i32; 4]); 8] =
        [(53, [0, 4, 7, 11]), (52, [0, 3, 7, 10]), (50, [0, 3, 7, 10]), (48, [0, 4, 7, 11]), (46, [0, 4, 7, 11]), (45, [0, 3, 7, 10]), (43, [0, 3, 7, 10]), (48, [0, 4, 7, 10])];
    let semicorchea = 60.0 / 92.0 / 4.0;
    let compas = semicorchea * 16.0;
    let total = compas * ACORDES.len() as f32;
    let largo = (total * FRECUENCIA as f32) as usize;
    let mut pista = vec![0.0f32; largo];
    let mut sumar = |desde: f32, muestras: &[f32], volumen: f32| {
        let i0 = (desde * FRECUENCIA as f32) as usize;
        for (k, m) in muestras.iter().enumerate() {
            pista[(i0 + k) % largo] += m * volumen;
        }
    };
    for (c, &(raiz, acorde)) in ACORDES.iter().enumerate() {
        let inicio = c as f32 * compas;
        // El arpegio: sube ocho notas por dos octavas y baja otras ocho.
        for s in 0..16 {
            let paso = if s < 8 { s } else { 15 - s };
            let nota = raiz + 12 + acorde[paso % 4] + 12 * (paso / 4) as i32;
            let f = midi(nota);
            let pulso = cuerda(f, 1.4, (c * 16 + s) as u32 * 7919 + 13);
            sumar(inicio + s as f32 * semicorchea, &pulso, 0.32);
        }
        // El colchon: el acorde suave, que entra y sale despacito.
        for (v, &intervalo) in acorde.iter().enumerate() {
            let f = midi(raiz + intervalo) * (1.0 + 0.002 * (v as f32 - 1.5));
            let colchon: Vec<f32> = (0..((compas + 0.8) * FRECUENCIA as f32) as usize)
                .map(|k| {
                    let t = k as f32 / FRECUENCIA as f32;
                    let env = (t / 0.8).min(1.0) * (1.0 - ((t - compas) / 0.8).clamp(0.0, 1.0));
                    env * (TAU * f * t).sin()
                })
                .collect();
            sumar(inicio, &colchon, 0.03);
        }
        // Una campanita arriba en el tiempo 3 de cada compas.
        let f = midi(raiz + 36 + acorde[(c % 2) * 2]);
        let campana: Vec<f32> = (0..(1.6 * FRECUENCIA as f32) as usize)
            .map(|k| {
                let t = k as f32 / FRECUENCIA as f32;
                (-t * 3.0).exp() * ((TAU * f * t).sin() + 0.35 * (TAU * f * 2.76 * t).sin())
            })
            .collect();
        sumar(inicio + 8.0 * semicorchea, &campana, 0.045);
    }
    for m in pista.iter_mut() {
        *m = (*m * 1.3).tanh() * 0.8;
    }
    pista
}

/// La cinematica de inicio: un piano triste y lento, original, en la menor, con lluvia. 12 compases que
/// se repiten sin corte.
pub fn sintetizar_prologo() -> Vec<f32> {
    // la menor, fa, do, sol (con la raiz abajo y el acorde arriba, separados).
    const ACORDES: [(i32, [i32; 3]); 4] = [(45, [0, 3, 7]), (41, [0, 4, 7]), (48, [0, 4, 7]), (43, [0, 4, 7])];
    const MELODIA: [i32; 12] = [76, 72, 71, 72, 76, 74, 72, 67, 69, 72, 71, 67];
    let compas = 60.0 / 54.0 * 4.0;
    let total = compas * 12.0;
    let largo = (total * FRECUENCIA as f32) as usize;
    let mut pista = vec![0.0f32; largo];
    let piano = |f: f32, dura: f32| -> Vec<f32> {
        (0..(dura * FRECUENCIA as f32) as usize)
            .map(|k| {
                let t = k as f32 / FRECUENCIA as f32;
                let cuerpo = (TAU * f * t).sin() + 0.45 * (TAU * f * 2.0 * t).sin() * (-t * 1.8).exp() + 0.18 * (TAU * f * 3.0 * t).sin() * (-t * 3.2).exp();
                cuerpo * (t / 0.006).min(1.0) * (-t * 1.1).exp()
            })
            .collect()
    };
    let mut sumar = |desde: f32, muestras: &[f32], volumen: f32| {
        let i0 = (desde * FRECUENCIA as f32) as usize;
        for (k, m) in muestras.iter().enumerate() {
            pista[(i0 + k) % largo] += m * volumen;
        }
    };
    for c in 0..12 {
        let (raiz, acorde) = ACORDES[c % 4];
        let inicio = c as f32 * compas;
        sumar(inicio, &piano(midi(raiz - 12), 5.0), 0.28);
        for (k, &i) in acorde.iter().enumerate() {
            sumar(inicio + compas * 0.25 * (k + 1) as f32, &piano(midi(raiz + 12 + i), 3.2), 0.13);
        }
        // La melodia entra poco a poco y deja huecos.
        if c >= 2 {
            sumar(inicio + compas * 0.5, &piano(midi(MELODIA[c]), 4.0), 0.2);
        }
    }
    // Eco de cuarto vacio.
    let eco = (0.31 * FRECUENCIA as f32) as usize;
    for i in 0..largo {
        pista[i] += pista[(i + largo - eco) % largo] * 0.3;
    }
    // La lluvia, muy bajita.
    let mut azar = 7u32;
    let mut pasado = 0.0f32;
    for m in pista.iter_mut() {
        azar = azar.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let r = (azar >> 8) as f32 / 8_388_608.0 - 1.0;
        pasado += 0.3 * (r - pasado);
        *m = (*m + (r - pasado) * 0.018).tanh() * 0.8;
    }
    pista
}

/// La musica del consultorio de Madame Mercurio: original, misteriosa, en mi
/// frigio dominante (el sonido "de adivina"): un bordon grave de mi y si,
/// arpegios de cuerda pulsada y una melodia lenta de theremin con vibrato.
/// 8 compases que se repiten sin corte (y al reves, para el retrogrado).
pub fn sintetizar_tarot() -> Vec<f32> {
    // mi, fa, mi, re menor, do, fa, sol, mi.
    const ACORDES: [(i32, [i32; 3]); 8] = [(52, [0, 4, 7]), (53, [0, 4, 7]), (52, [0, 4, 7]), (50, [0, 3, 7]), (48, [0, 4, 7]), (53, [0, 4, 7]), (55, [0, 4, 7]), (52, [0, 4, 7])];
    const MELODIA: [i32; 16] = [76, 77, 80, 77, 76, 74, 77, 76, 72, 74, 77, 80, 83, 81, 80, 76];
    let corchea = 60.0 / 80.0 / 2.0;
    let compas = corchea * 8.0;
    let total = compas * ACORDES.len() as f32;
    let largo = (total * FRECUENCIA as f32) as usize;
    let mut pista = vec![0.0f32; largo];
    let mut sumar = |desde: f32, muestras: &[f32], volumen: f32| {
        let i0 = (desde * FRECUENCIA as f32) as usize;
        for (k, m) in muestras.iter().enumerate() {
            pista[(i0 + k) % largo] += m * volumen;
        }
    };
    // El bordon: mi y si graves todo el tiempo, que respiran despacio.
    let bordon: Vec<f32> = (0..largo)
        .map(|k| {
            let t = k as f32 / FRECUENCIA as f32;
            let respira = 0.75 + 0.25 * (TAU * t / compas).sin();
            respira * ((TAU * midi(40) * t).sin() + 0.6 * (TAU * midi(47) * t).sin() + 0.25 * (TAU * midi(52) * t * 1.002).sin())
        })
        .collect();
    sumar(0.0, &bordon, 0.05);
    for (c, &(raiz, acorde)) in ACORDES.iter().enumerate() {
        let inicio = c as f32 * compas;
        // El arpegio de cuerda: raiz, quinta, octava, tercera arriba... y de regreso.
        for s in 0..8 {
            let paso = [0, 2, 3, 1, 4, 2, 3, 1][s];
            let nota = raiz + 12 + [acorde[0], acorde[1], acorde[2], 12, 12 + acorde[1]][paso];
            let pulso = cuerda(midi(nota), 1.2, (c * 8 + s) as u32 * 6151 + 29);
            sumar(inicio + s as f32 * corchea, &pulso, if s == 0 { 0.36 } else { 0.24 });
        }
        // La melodia de theremin: dos notas por compas, que se deslizan.
        for h in 0..2 {
            let nota = MELODIA[c * 2 + h];
            let previa = MELODIA[(c * 2 + h + 15) % 16];
            let dura = compas / 2.0;
            let tono: Vec<f32> = (0..((dura + 0.3) * FRECUENCIA as f32) as usize)
                .scan(0.0f32, |fase, k| {
                    let t = k as f32 / FRECUENCIA as f32;
                    let desliza = ((t / 0.12).min(1.0) - 1.0) * (nota - previa) as f32;
                    let f = midi(nota) * 2f32.powf(desliza / 12.0) * (1.0 + 0.012 * (TAU * 5.0 * t).sin() * (t / 0.4).min(1.0));
                    *fase += f / FRECUENCIA as f32;
                    let env = (t / 0.08).min(1.0) * (1.0 - ((t - dura) / 0.3).clamp(0.0, 1.0));
                    Some(env * ((TAU * *fase).sin() + 0.12 * (TAU * 2.0 * *fase).sin()))
                })
                .collect();
            sumar(inicio + h as f32 * dura, &tono, 0.06);
        }
        // Un platillo de dedo (crotalo) al empezar cada dos compases.
        if c % 2 == 0 {
            let f = midi(88);
            let crotalo: Vec<f32> = (0..(2.0 * FRECUENCIA as f32) as usize)
                .map(|k| {
                    let t = k as f32 / FRECUENCIA as f32;
                    (-t * 2.2).exp() * ((TAU * f * t).sin() + 0.5 * (TAU * f * 2.41 * t).sin() + 0.3 * (TAU * f * 3.93 * t).sin())
                })
                .collect();
            sumar(inicio, &crotalo, 0.03);
        }
    }
    for m in pista.iter_mut() {
        *m = (*m * 1.3).tanh() * 0.8;
    }
    pista
}

/// La musica del juicio de divorcio de Saturno: original, tensa, como de
/// interrogatorio en un juzgado. La menor a 140 BPM: un bajo insistente en
/// corcheas, bombo y tarola, acordes cortos de metales y una melodia que sube.
/// 8 compases que se repiten sin corte.
pub fn sintetizar_juicio() -> Vec<f32> {
    // la menor, fa, re menor, mi; la menor, fa, sol, mi (con septima).
    const ACORDES: [(i32, [i32; 3]); 8] = [(45, [0, 3, 7]), (41, [0, 4, 7]), (38, [0, 3, 7]), (40, [0, 4, 7]), (45, [0, 3, 7]), (41, [0, 4, 7]), (43, [0, 4, 7]), (40, [0, 4, 10])];
    // La melodia: una nota por negra (0 = silencio).
    const MELODIA: [i32; 32] = [
        69, 72, 76, 74, 72, 0, 71, 72, 74, 0, 72, 69, 74, 72, 71, 68, 69, 72, 76, 79, 77, 76, 74, 72, 74, 76, 77, 79, 80, 0, 76, 0,
    ];
    let corchea = 60.0 / 140.0 / 2.0;
    let compas = corchea * 8.0;
    let total = compas * ACORDES.len() as f32;
    let largo = (total * FRECUENCIA as f32) as usize;
    let mut pista = vec![0.0f32; largo];
    let mut ruido = Ruido(0x7A11_5EED);
    for (c, &(raiz, acorde)) in ACORDES.iter().enumerate() {
        let inicio = c as f32 * compas;
        // El bajo: corcheas, raiz y octava.
        for s in 0..8 {
            let nota = raiz + if s % 4 == 3 { 12 } else { 0 };
            let f = midi(nota);
            sumar(&mut pista, inicio + s as f32 * corchea, corchea * 0.9, |t| {
                let env = (1.0 - t / (corchea * 0.9)).max(0.0);
                0.22 * env * ((2.0 * (f * t).fract() - 1.0) * 0.6 + 0.4 * (TAU * f * t).sin())
            });
        }
        // Bombo en 1 y 3, tarola en 2 y 4.
        for b in 0..4 {
            let t0 = inicio + b as f32 * corchea * 2.0;
            if b % 2 == 0 {
                sumar(&mut pista, t0, 0.25, |t| 0.5 * (-t * 22.0).exp() * (TAU * (90.0 - 120.0 * t) * t).sin());
            } else {
                let semilla: Vec<f32> = (0..(0.2 * FRECUENCIA as f32) as usize).map(|_| ruido.sig()).collect();
                sumar(&mut pista, t0, 0.2, |t| 0.18 * (-t * 25.0).exp() * semilla[((t * FRECUENCIA as f32) as usize).min(semilla.len() - 1)]);
            }
        }
        // Los metales: el acorde corto en el 1 y en el "y" del 2.
        for &golpe in &[0.0, 3.0] {
            for &intervalo in &acorde {
                let f = midi(raiz + 24 + intervalo);
                sumar(&mut pista, inicio + golpe * corchea, corchea * 1.2, |t| {
                    let env = (t / 0.01).min(1.0) * (1.0 - t / (corchea * 1.2)).max(0.0);
                    0.05 * env * (2.0 * (f * t).fract() - 1.0)
                });
            }
        }
        // La melodia, con un poco de vibrato.
        for n in 0..4 {
            let nota = MELODIA[c * 4 + n];
            if nota == 0 {
                continue;
            }
            let f = midi(nota);
            let dura = corchea * 1.9;
            sumar(&mut pista, inicio + n as f32 * corchea * 2.0, dura, |t| {
                let env = (t / 0.02).min(1.0) * (1.0 - (t - dura + 0.05).max(0.0) / 0.05);
                let vib = 1.0 + 0.008 * (TAU * 6.0 * t).sin();
                let fase = f * vib * t;
                0.07 * env * ((2.0 * fase.fract() - 1.0) * 0.5 + (TAU * fase).sin() * 0.5)
            });
        }
    }
    for m in pista.iter_mut() {
        *m = (*m * 1.4).tanh() * 0.8;
    }
    pista
}

/// Jupiter Asteroids: chiptune de arcade a 160 BPM. Bajo de onda cuadrada en corcheas,
/// arpegios rapidisimos (como las consolas viejas, que no podian tocar acordes), una
/// melodia de pulso delgado y bateria de ruido. Ocho compases que se repiten.
pub fn sintetizar_arcade() -> Vec<f32> {
    // la menor, fa, do, sol; la menor, fa, sol, mi.
    const ACORDES: [(i32, [i32; 3]); 8] = [(45, [0, 3, 7]), (41, [0, 4, 7]), (48, [0, 4, 7]), (43, [0, 4, 7]), (45, [0, 3, 7]), (41, [0, 4, 7]), (43, [0, 4, 7]), (40, [0, 4, 8])];
    // Dos notas por negra (corcheas); 0 = silencio.
    const MELODIA: [i32; 64] = [
        76, 0, 76, 79, 81, 79, 76, 74, 72, 0, 72, 74, 76, 74, 72, 69, 72, 0, 72, 76, 79, 81, 79, 76, 74, 0, 71, 74, 79, 0, 0, 0,
        76, 0, 76, 79, 81, 79, 76, 74, 77, 0, 77, 76, 74, 72, 74, 76, 79, 0, 79, 81, 83, 81, 79, 77, 76, 0, 80, 0, 83, 0, 88, 0,
    ];
    let corchea = 60.0 / 160.0 / 2.0;
    let compas = corchea * 8.0;
    let total = compas * ACORDES.len() as f32;
    let mut pista = vec![0.0f32; (total * FRECUENCIA as f32) as usize];
    let mut ruido = Ruido(0xA57E_101D);
    for (c, &(raiz, acorde)) in ACORDES.iter().enumerate() {
        let inicio = c as f32 * compas;
        for s in 0..8 {
            let t0 = inicio + s as f32 * corchea;
            // El bajo: cuadrada, raiz y octava saltando.
            let f = midi(raiz - 12 + if s % 2 == 1 { 12 } else { 0 });
            sumar(&mut pista, t0, corchea * 0.85, |t| 0.10 * (1.0 - t / (corchea * 0.85)).max(0.0).sqrt() * cuadrada(f * t, 0.5));
            // El arpegio: cuatro notas por corchea, rapidisimo (el acorde "de mentiras").
            for q in 0..4 {
                let f = midi(raiz + 12 + acorde[(s * 4 + q) % 3]);
                let dq = corchea / 4.0;
                sumar(&mut pista, t0 + q as f32 * dq, dq, |t| 0.035 * cuadrada(f * t, 0.25) * (1.0 - t / dq * 0.5));
            }
            // La bateria: bombo en 1 y 3, tarola en 2 y 4, hi-hat en todas.
            if s % 4 == 0 {
                sumar(&mut pista, t0, 0.18, |t| 0.4 * (-t * 25.0).exp() * (TAU * (110.0 - 160.0 * t) * t).sin());
            }
            let sem: Vec<f32> = (0..(0.12 * FRECUENCIA as f32) as usize).map(|_| ruido.sig()).collect();
            let fuerte = s % 4 == 2;
            sumar(&mut pista, t0, if fuerte { 0.12 } else { 0.03 }, |t| {
                let k = ((t * FRECUENCIA as f32) as usize).min(sem.len() - 1);
                (if fuerte { 0.16 } else { 0.05 }) * (-t * if fuerte { 30.0 } else { 120.0 }).exp() * sem[k]
            });
        }
        // La melodia: pulso de 12.5% (el sonido "nasal" de las consolas), con un vibratito.
        for n in 0..8 {
            let nota = MELODIA[c * 8 + n];
            if nota == 0 {
                continue;
            }
            let f = midi(nota);
            let largo = if MELODIA.get(c * 8 + n + 1) == Some(&0) { corchea * 1.9 } else { corchea * 0.95 };
            sumar(&mut pista, inicio + n as f32 * corchea, largo, |t| {
                let env = (t / 0.005).min(1.0) * (1.0 - (t - largo + 0.03).max(0.0) / 0.03);
                let vib = 1.0 + 0.006 * (TAU * 7.0 * (t - 0.1).max(0.0)).sin();
                0.065 * env * cuadrada(f * vib * t, 0.125)
            });
        }
    }
    for m in pista.iter_mut() {
        *m = (*m * 1.3).tanh() * 0.8;
    }
    pista
}

/// Lo-fi para estudiar (la noche de la entrega en el CIT): 78 BPM con swing, piano electrico con
/// acordes de septima (fa maj7, mi m7, re m7, do maj7: la bajada de siempre del lo-fi), bajo
/// redondo, bateria polvosa, una melodia de flauta muy de vez en cuando, crujido de vinil, la
/// cinta que se estira un poquito (wow) y lluvia en la ventana. Todo con un filtro tibio encima.
pub fn sintetizar_lofi() -> Vec<f32> {
    // Raiz y notas del acorde (en semitonos desde la raiz), voceado cerca del do central.
    const ACORDES: [(i32, [i32; 4]); 8] = [
        (53, [4, 7, 11, 14]),
        (52, [3, 7, 10, 14]),
        (50, [3, 7, 10, 12]),
        (48, [4, 7, 11, 14]),
        (53, [4, 7, 11, 14]),
        (52, [3, 7, 10, 14]),
        (50, [3, 7, 10, 17]),
        (55, [4, 5, 10, 14]),
    ];
    // La flauta: (compas, octavo, nota, octavos que dura). Poquitas notas, con aire.
    const FLAUTA: [(usize, usize, i32, usize); 12] = [
        (0, 2, 76, 2),
        (0, 4, 74, 2),
        (0, 6, 72, 3),
        (1, 3, 71, 2),
        (1, 5, 67, 3),
        (3, 0, 72, 2),
        (3, 2, 74, 2),
        (3, 4, 76, 4),
        (5, 2, 79, 2),
        (5, 4, 77, 2),
        (5, 6, 76, 4),
        (7, 4, 74, 4),
    ];
    let negra = 60.0 / 78.0;
    let compas = negra * 4.0;
    let total = compas * ACORDES.len() as f32;
    let mut pista = vec![0.0f32; (total * FRECUENCIA as f32) as usize];
    let mut ruido = Ruido(0x10F1_2359);
    // Swing: el segundo octavo de cada negra llega tarde (62% en vez de 50%).
    let octavo = |n: usize| (n / 2) as f32 * negra + if n % 2 == 1 { negra * 0.62 } else { 0.0 };
    // El piano electrico (tipo Rhodes): seno con dos armonicos, ataque suave, se apaga despacio y
    // con su tremolo.
    let rhodes = |f: f32, t: f32, largo: f32| {
        let env = (t / 0.012).min(1.0) * (-t * 1.6).exp() * (1.0 - ((t - largo + 0.08) / 0.08).clamp(0.0, 1.0));
        let wow = 1.0 + 0.0035 * (TAU * 0.6 * t).sin();
        let fase = TAU * f * wow * t;
        let campana = (fase * 2.0 + 0.6 * (fase * 7.0).sin() * (-t * 9.0).exp()).sin();
        env * (fase.sin() + 0.25 * campana + 0.06 * (fase * 3.0).sin()) * (1.0 + 0.15 * (TAU * 4.5 * t).sin())
    };
    for (c, &(raiz, notas)) in ACORDES.iter().enumerate() {
        let inicio = c as f32 * compas;
        // El acorde: en el uno, y otra vez (mas suavecito) en el "y" del dos, como amague.
        for (golpe, fuerza, largo) in [(0usize, 1.0f32, negra * 2.4), (3, 0.55, negra * 2.0)] {
            for (k, &n) in notas.iter().enumerate() {
                let f = midi(raiz + n);
                // Rasgueado: cada nota un pelito despues de la otra.
                let t0 = inicio + octavo(golpe) + k as f32 * 0.012;
                sumar(&mut pista, t0, largo, |t| 0.045 * fuerza * rhodes(f, t, largo));
            }
        }
        // El bajo: redondo (seno con un poquito de saturacion), en el uno, el "y" del dos y el tres.
        for (o, nota) in [(0usize, raiz - 24), (3, raiz - 24), (4, raiz - 17)] {
            let f = midi(nota);
            let largo = negra * 0.9;
            sumar(&mut pista, inicio + octavo(o), largo, |t| {
                let env = (t / 0.01).min(1.0) * (1.0 - t / largo).max(0.0).powf(0.6);
                0.22 * env * ((TAU * f * t).sin() * 1.6).tanh()
            });
        }
        // La bateria, polvosa: bombo en el 1 y el "y" del 2; tarola (con aire) en 2 y 4; hi-hat
        // suave en los octavos con swing.
        for o in [0usize, 3] {
            sumar(&mut pista, inicio + octavo(o), 0.3, |t| 0.42 * (-t * 14.0).exp() * (TAU * (52.0 + 90.0 * (-t * 30.0).exp()) * t).sin());
        }
        for o in [2usize, 6] {
            let sem: Vec<f32> = (0..(0.25 * FRECUENCIA as f32) as usize).map(|_| ruido.sig()).collect();
            let mut lp = 0.0f32;
            sumar(&mut pista, inicio + octavo(o), 0.25, |t| {
                let k = ((t * FRECUENCIA as f32) as usize).min(sem.len() - 1);
                lp += (sem[k] - lp) * 0.35;
                0.16 * (-t * 16.0).exp() * lp + 0.05 * (-t * 30.0).exp() * (TAU * 185.0 * t).sin()
            });
        }
        for o in 0..8usize {
            let sem: Vec<f32> = (0..(0.05 * FRECUENCIA as f32) as usize).map(|_| ruido.sig()).collect();
            let fuerte = if o % 2 == 0 { 0.035 } else { 0.022 };
            let mut ant = 0.0f32;
            sumar(&mut pista, inicio + octavo(o), 0.05, |t| {
                let k = ((t * FRECUENCIA as f32) as usize).min(sem.len() - 1);
                // Pasa altos (lo agudo del ruido): resta lo que se parece al anterior.
                let v = sem[k] - ant;
                ant = sem[k];
                fuerte * (-t * 90.0).exp() * v
            });
        }
    }
    // La flauta: seno con un poco de soplido y vibrato que entra tarde.
    for &(c, o, nota, dura) in &FLAUTA {
        let f = midi(nota);
        let t0 = c as f32 * compas + octavo(o);
        let largo = dura as f32 * negra * 0.5;
        let sem: Vec<f32> = (0..(largo * FRECUENCIA as f32) as usize + 1).map(|_| ruido.sig()).collect();
        sumar(&mut pista, t0, largo, |t| {
            let env = (t / 0.08).min(1.0) * (1.0 - ((t - largo + 0.15) / 0.15).clamp(0.0, 1.0));
            let vib = 1.0 + 0.008 * (TAU * 5.2 * t).sin() * (t / 0.4).min(1.0);
            let k = ((t * FRECUENCIA as f32) as usize).min(sem.len() - 1);
            0.05 * env * ((TAU * f * vib * t).sin() + 0.12 * (TAU * 2.0 * f * vib * t).sin() + 0.08 * sem[k])
        });
    }
    // El vinil y la lluvia: chasquidos de vez en cuando y un siseo que sube y baja.
    let mut lluvia = 0.0f32;
    let n = pista.len();
    for (i, m) in pista.iter_mut().enumerate() {
        let t = i as f32 / FRECUENCIA as f32;
        let r = ruido.sig();
        lluvia += (r - lluvia) * 0.08;
        *m += lluvia * 0.035 * (0.75 + 0.25 * (TAU * t / 7.0).sin());
        if ruido.sig() > 0.9993 {
            *m += 0.12 * ruido.sig();
        }
        // Que el bucle no truene: entra y sale suavecito en los extremos.
        let borde = ((i as f32 / 400.0).min(1.0)).min(((n - i) as f32 / 400.0).min(1.0));
        *m *= borde;
    }
    // El filtro tibio (pasa bajos de un polo, ~5 kHz) y la saturacion de cinta.
    let mut lp = 0.0f32;
    let a = 1.0 - (-TAU * 5000.0 / FRECUENCIA as f32).exp();
    for m in pista.iter_mut() {
        lp += (*m - lp) * a;
        *m = (lp * 1.4).tanh() * 0.85;
    }
    pista
}

/// La marimba (el Planeta Chapin, la camioneta): un son en 6/8 en sol mayor, como los de la
/// marimba de feria. Cada tecla es un seno que se apaga rapido con su armonico de cuatro (el
/// sonido de la madera con su tubo resonador); las notas largas van en tremolo (redoblando, como
/// se toca la marimba de verdad). Abajo, el bajo en los graves del mismo instrumento.
pub fn sintetizar_marimba() -> Vec<f32> {
    // Los acordes por compas (raiz, tercera, quinta) y la melodia: (compas, corchea 0..5, nota, corcheas).
    const ACORDES: [[i32; 3]; 8] = [[55, 59, 62], [55, 59, 62], [50, 54, 57], [50, 54, 57], [48, 52, 55], [55, 59, 62], [50, 54, 57], [55, 59, 62]];
    const MELODIA: [(usize, usize, i32, usize); 26] = [
        (0, 0, 74, 1), (0, 1, 76, 1), (0, 2, 79, 2), (0, 4, 78, 1), (0, 5, 76, 1),
        (1, 0, 74, 3), (1, 3, 71, 1), (1, 4, 74, 1), (1, 5, 76, 1),
        (2, 0, 78, 1), (2, 1, 76, 1), (2, 2, 74, 2), (2, 4, 72, 1), (2, 5, 69, 1),
        (3, 0, 72, 3), (3, 3, 74, 3),
        (4, 0, 76, 1), (4, 1, 76, 1), (4, 2, 76, 1), (4, 3, 79, 1), (4, 4, 77, 1), (4, 5, 76, 1),
        (5, 0, 74, 3),
        (6, 0, 72, 2), (6, 3, 78, 3),
        (7, 0, 79, 6),
    ];
    let corchea = 60.0 / 168.0 / 2.0 * 1.5;
    let compas = corchea * 6.0;
    let total = compas * ACORDES.len() as f32 * 2.0;
    let mut pista = vec![0.0f32; (total * FRECUENCIA as f32) as usize];
    // Una tecla de marimba: el fundamental y el armonico de cuatro (que se apaga mas rapido).
    let tecla = |f: f32, t: f32| ((TAU * f * t).sin() + 0.35 * (TAU * 4.0 * f * t).sin() * (-t * 18.0).exp()) * (-t * 7.0).exp() * (t / 0.002).min(1.0);
    for vuelta in 0..2 {
        let desde = vuelta as f32 * compas * ACORDES.len() as f32;
        for (c, acorde) in ACORDES.iter().enumerate() {
            let inicio = desde + c as f32 * compas;
            // El bajo: la raiz en el 1 y la quinta en el 4 (el "tun... tun" del 6/8).
            for (o, n) in [(0usize, acorde[0] - 12), (3, acorde[2] - 24)] {
                let f = midi(n);
                sumar(&mut pista, inicio + o as f32 * corchea, 0.6, |t| 0.28 * tecla(f, t));
            }
            // El acompanamiento: el acorde en las corcheas 2, 3, 5 y 6 (el "chun-chun").
            for o in [1usize, 2, 4, 5] {
                for &n in acorde {
                    let f = midi(n);
                    sumar(&mut pista, inicio + o as f32 * corchea, 0.35, |t| 0.06 * tecla(f, t));
                }
            }
        }
        for &(c, o, nota, dura) in &MELODIA {
            let f = midi(nota);
            let t0 = desde + c as f32 * compas + o as f32 * corchea;
            if dura >= 2 {
                // Las largas, en tremolo: golpecitos muy seguidos de la misma tecla.
                let golpes = (dura as f32 * corchea / 0.055) as usize;
                for g in 0..golpes {
                    sumar(&mut pista, t0 + g as f32 * 0.055, 0.2, |t| 0.10 * tecla(f, t) * if g % 2 == 0 { 1.0 } else { 0.8 });
                }
            } else {
                sumar(&mut pista, t0, 0.45, |t| 0.16 * tecla(f, t));
            }
        }
    }
    let pico = pista.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-6);
    for m in &mut pista {
        *m = *m / pico * 0.8;
    }
    pista
}
