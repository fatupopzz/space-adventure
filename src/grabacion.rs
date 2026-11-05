//! `--grabar video.mp4`: el recorrido por TODO el juego, grabado por el mismo juego.
//!
//! El guion son tomas: cada una arma una escena (con los mismos modos que `--foto-ui`), la deja
//! correr unos segundos con un piloto automatico (la demo de vuelo, el chofer de la camioneta,
//! el parkour de Willy...) y le pone un rotulo arriba. El reloj va a paso fijo (30 cuadros por
//! segundo), asi que no se pierde ningun cuadro aunque cada uno tarde en dibujarse: cada cuadro
//! de la ventana (con la interfaz) se le pasa a ffmpeg por una tuberia, y el sonido se mezcla al
//! mismo paso (`sonido::Mezcla`). Al terminar se juntan los dos en el `.mp4`.

use std::io::Write;
use std::process::{Child, Command, Stdio};

/// Cuadros por segundo del video.
pub const CUADROS: u32 = 30;
pub const DT: f32 = 1.0 / CUADROS as f32;
/// Lo que tarda el fundido a negro de entrada y de salida de cada toma.
const FUNDIDO: f32 = 0.35;
/// Cuanto dura el rotulo de cada toma en pantalla.
const ROTULO: f32 = 3.4;

/// Quien maneja durante la toma.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Auto {
    /// Nadie: la escena corre sola.
    Nada,
    /// El recorrido de la demo (vuelo, acercamiento, vuelta y alejamiento).
    Demo,
    /// Volando de nota en nota.
    Notas,
    /// En modo orbita, con un poco de empuje.
    Orbita,
    /// Aceptar de vez en cuando (para que los dialogos avancen).
    Dialogo,
    /// El chofer automatico de la camioneta.
    Camioneta,
    /// El parkour de Planeta Vegetta.
    Parkour,
    /// TNT Run: corriendo en circulos.
    Tnt,
    /// La mineria: picando.
    Mina,
    /// Jupiter Asteroids: girando y disparando.
    Asteroides,
    /// Saturno Ritmo: cada nota a tiempo.
    Ritmo,
    /// En un menu: bajando por la lista.
    Abajo,
    /// En un menu: pasando a la derecha.
    Derecha,
}

pub struct Toma {
    /// Los modos de `--foto-ui` (la escena, y lo que se abre encima).
    pub modos: &'static [&'static str],
    /// Los segundos que se deja correr la escena antes de empezar a grabar.
    pub desde: f32,
    pub segundos: f32,
    pub auto: Auto,
    /// Titulo y subtitulo arriba al centro (los primeros segundos).
    pub rotulo: Option<(&'static str, &'static str)>,
}


const fn toma(modos: &'static [&'static str], desde: f32, segundos: f32, auto: Auto, rotulo: Option<(&'static str, &'static str)>) -> Toma {
    Toma { modos, desde, segundos, auto, rotulo }
}

/// El guion del video: todo el juego, en orden.
pub const GUION: &[Toma] = &[
    toma(&["menu_lista"], 0.0, 6.0, Auto::Nada, None),
    toma(&["intro"], 0.0, 9.0, Auto::Nada, None),
    toma(&["vuelo_demo"], 0.0, 33.0, Auto::Demo, None),
    toma(&["recoger"], 0.0, 5.0, Auto::Notas, Some(("NOTAS MUSICALES", "se recogen volando: combo, chispas y la escala pentatónica"))),
    toma(&["maravilla:gargantua"], 0.0, 6.0, Auto::Nada, Some(("GARGANTÚA", "el agujero negro: lente gravitacional como shader de post-proceso"))),
    toma(&["maravilla:teseracto"], 0.0, 5.0, Auto::Nada, Some(("EL TESERACTO", "adentro del agujero negro (Interstellar)"))),
    toma(&["maravilla:gusano"], 0.0, 5.0, Auto::Nada, Some(("EL AGUJERO DE GUSANO", "junto a Saturno"))),
    toma(&["maravilla:blip"], 0.0, 5.0, Auto::Nada, Some(("LA BLIP-A", "Rocky saluda con acordes (Project Hail Mary)"))),
    toma(&["maravilla:monolito"], 0.0, 4.5, Auto::Nada, Some(("EL MONOLITO", "2001: Odisea del espacio"))),
    toma(&["maravilla:huerto"], 0.0, 4.5, Auto::Nada, Some(("EL HUERTO DE PAPAS", "en Marte, como The Martian"))),
    toma(&["cerca:sol"], 0.0, 3.5, Auto::Nada, Some(("PLANETAS PROCEDURALES", "vóxeles deformados con ruido Perlin, en cinco niveles de detalle"))),
    toma(&["cerca:tierra"], 0.0, 3.0, Auto::Nada, None),
    toma(&["cerca:jupiter"], 0.0, 3.0, Auto::Nada, None),
    toma(&["cerca:saturno_ritmo"], 0.0, 3.0, Auto::Nada, None),
    toma(&["cerca:neptuno"], 0.0, 3.0, Auto::Nada, None),
    toma(&["cerca:planeta_vegetta"], 0.0, 3.0, Auto::Nada, None),
    toma(&["cerca:saturno_ritmo", "zbuffer"], 0.0, 2.5, Auto::Nada, Some(("EL PIPELINE", "z-buffer, normales y coordenadas UV"))),
    toma(&["cerca:saturno_ritmo", "normales"], 0.0, 2.5, Auto::Nada, None),
    toma(&["cerca:saturno_ritmo", "uv"], 0.0, 2.5, Auto::Nada, None),
    toma(&["cerca:jupiter", "info"], 0.0, 4.0, Auto::Nada, Some(("TAB", "la info del pipeline y los parámetros en vivo"))),
    toma(&["chapin_orbita"], 0.0, 7.0, Auto::Orbita, Some(("PLANETA CHAPÍN", "volcanes, ceibas y Tikal, todo en vóxeles"))),
    toma(&["camioneta"], 1.0, 5.0, Auto::Nada, None),
    toma(&["camioneta_juego"], 0.0, 11.0, Auto::Camioneta, Some(("LA CAMIONETA EXTRAURBANA", "¡Hay campo! Esquivar tuc-tucs, perros y túmulos"))),
    toma(&["camioneta_antigua"], 0.0, 8.0, Auto::Camioneta, Some(("ANTIGUA", "adoquín, papel picado y el Arco de Santa Catalina"))),
    toma(&["camioneta_capital"], 0.0, 7.0, Auto::Camioneta, Some(("LA CAPITAL", "cae la noche: edificios, vallas y la Torre del Reformador"))),
    toma(&["camioneta_llegada"], 0.0, 6.0, Auto::Camioneta, None),
    toma(&["vegetta"], 0.5, 4.0, Auto::Nada, None),
    toma(&["vegetta_carrera"], 0.5, 9.0, Auto::Parkour, None),
    toma(&["vegetta_tnt"], 0.0, 7.0, Auto::Tnt, Some(("TNT RUN", "el piso se cae atrás de uno"))),
    toma(&["vegetta_mina"], 0.0, 6.0, Auto::Mina, Some(("MINERÍA", "a picar hasta los diamantes"))),
    toma(&["urano"], 0.5, 4.0, Auto::Nada, None),
    toma(&["urano_juego"], 0.0, 9.0, Auto::Nada, Some(("URANO", "el baño de la Chepa: papel, destapacaños y fila"))),
    toma(&["urano_final"], 0.0, 4.0, Auto::Nada, None),
    toma(&["ritmo"], 0.0, 13.0, Auto::Ritmo, Some(("SATURNO RITMO", "un juego de ritmo con su propia canción, sintetizada"))),
    toma(&["concierto"], 2.0, 6.0, Auto::Nada, Some(("EL CONCIERTO", "Miku en los anillos de Saturno"))),
    toma(&["divorcio"], 1.0, 8.0, Auto::Dialogo, Some(("EL DIVORCIO", "Saturno contra sus anillos: ¡PROTESTO!"))),
    toma(&["jupiter"], 2.0, 9.0, Auto::Asteroides, Some(("JUPITER ASTEROIDS", "arcade en la Gran Mancha Roja"))),
    toma(&["jupiter_jefe"], 3.0, 6.0, Auto::Asteroides, None),
    toma(&["mercurio"], 1.0, 7.0, Auto::Dialogo, Some(("MERCURIO RETRÓGRADO", "el tarot"))),
    toma(&["luna"], 1.0, 7.0, Auto::Dialogo, Some(("LA LUNA", "el set donde se grabó la llegada"))),
    toma(&["cita"], 1.0, 7.0, Auto::Dialogo, Some(("LA CITA", "en Marte"))),
    toma(&["neptuno_juego"], 1.0, 7.0, Auto::Nada, Some(("¡NEPTUNO!", "las reglas del juego de mesa: el +4 se puede retar"))),
    toma(&["neptuno_reto"], 0.5, 3.5, Auto::Nada, None),
    toma(&["junta"], 1.0, 6.0, Auto::Dialogo, Some(("PLUTÓN", "la junta para volver a ser planeta"))),
    toma(&["fuente_hada"], 0.0, 7.0, Auto::Nada, Some(("LA GRAN FUENTE DE LAS HADAS", "la ocarina en el Planeta Hada"))),
    toma(&["aula"], 1.0, 6.0, Auto::Dialogo, None),
    toma(&["entrega"], 1.0, 7.0, Auto::Dialogo, None),
    toma(&["entrega_windows"], 0.5, 4.0, Auto::Nada, None),
    toma(&["mirador"], 1.0, 5.0, Auto::Nada, None),
    toma(&["casillero"], 0.5, 6.5, Auto::Derecha, None),
    toma(&["pase"], 0.5, 6.5, Auto::Derecha, None),
    toma(&["tienda"], 0.5, 5.5, Auto::Abajo, None),
    toma(&["tienda_estelas"], 0.5, 4.5, Auto::Abajo, None),
    toma(&["tienda_personajes"], 0.5, 5.5, Auto::Abajo, None),
    toma(&["pasaporte"], 0.5, 4.0, Auto::Nada, None),
    toma(&["mochila"], 0.5, 3.5, Auto::Nada, None),
    toma(&["menu"], 0.0, 6.0, Auto::Nada, Some(("GRACIAS POR VER", "github.com/fatupopzz/space-adventure"))),
];

/// La grabacion en curso: la toma, su reloj, y ffmpeg recibiendo los cuadros.
pub struct Grabacion {
    pub archivo: String,
    /// El video sin sonido (se junta con el audio al final).
    mudo: String,
    ffmpeg: Option<Child>,
    tamano: (i32, i32),
    pub toma: usize,
    /// La toma en que se para (sin incluirla): para grabar solo un pedazo.
    pub hasta: usize,
    /// Segundos dentro de la toma.
    pub t: f32,
    /// Si la toma acaba de empezar (hay que armar la escena).
    pub nueva: bool,
    pub cuadros: u64,
    /// `--limpio`: sin rotulos ni fundidos (las tomas en crudo, para editarlas despues).
    pub limpio: bool,
    empezo: std::time::Instant,
}

impl Grabacion {
    pub fn new(archivo: &str) -> Self {
        let mudo = format!("{archivo}.mudo.mp4");
        Grabacion { archivo: archivo.to_string(), mudo, ffmpeg: None, tamano: (0, 0), toma: 0, hasta: GUION.len(), t: 0.0, nueva: true, cuadros: 0, limpio: false, empezo: std::time::Instant::now() }
    }

    pub fn actual(&self) -> &'static Toma {
        &GUION[self.toma.min(GUION.len() - 1)]
    }

    /// Si es la ultima toma: su rotulo va abajo (arriba esta el logo del menu) y dura toda la toma.
    pub fn al_final(&self) -> bool {
        self.toma + 1 == GUION.len()
    }

    pub fn termino(&self) -> bool {
        self.toma >= self.hasta.min(GUION.len())
    }

    /// Lo negro del fundido de entrada y salida de la toma (0: nada, 1: todo negro).
    pub fn negro(&self) -> f32 {
        if self.limpio {
            return 0.0;
        }
        let toma = self.actual();
        let entrada = 1.0 - self.t / FUNDIDO;
        let salida = 1.0 - (toma.segundos - self.t) / FUNDIDO;
        entrada.max(salida).clamp(0.0, 1.0)
    }

    /// El rotulo de la toma y que tanto se ve (sube y baja en medio segundo).
    pub fn rotulo(&self) -> Option<((&'static str, &'static str), f32)> {
        let toma = self.actual();
        let r = toma.rotulo.filter(|_| !self.limpio)?;
        let dura = if self.al_final() { toma.segundos } else { ROTULO };
        let alfa = (self.t / 0.5).min((dura - self.t) / 0.5).clamp(0.0, 1.0);
        (alfa > 0.0).then_some((r, alfa))
    }

    /// Si en este cuadro se aprieta el boton de una entrada "a pulsos" (cada `cada` segundos).
    pub fn pulso(&self, cada: f32, desde: f32) -> bool {
        let k = |t: f32| ((t - desde) / cada).floor();
        self.t >= desde && k(self.t) != k(self.t - DT)
    }

    /// Le pasa a ffmpeg el cuadro de la ventana (RGBA, de arriba abajo).
    pub fn cuadro(&mut self, ancho: i32, alto: i32, pixeles: &[u8]) {
        if self.ffmpeg.is_none() {
            self.tamano = (ancho, alto);
            let hijo = Command::new(ffmpeg())
                .args(["-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "rgba", "-s", &format!("{ancho}x{alto}"), "-r", &CUADROS.to_string(), "-i", "-"])
                .args(["-vf", "scale=1920:1080:flags=lanczos,format=yuv420p", "-c:v", "libx264", "-preset", "medium", "-crf", "17", &self.mudo])
                .stdin(Stdio::piped())
                .spawn()
                .expect("no se pudo abrir ffmpeg (brew install ffmpeg)");
            self.ffmpeg = Some(hijo);
        }
        // Si la ventana cambio de tamano a media grabacion, el cuadro no le sirve a ffmpeg.
        if (ancho, alto) != self.tamano {
            eprintln!("grabacion: la ventana cambio de tamano ({ancho}x{alto}); se salta el cuadro");
            return;
        }
        if let Some(entrada) = self.ffmpeg.as_mut().and_then(|h| h.stdin.as_mut()) {
            entrada.write_all(pixeles).expect("ffmpeg dejo de recibir cuadros");
        }
        self.cuadros += 1;
    }

    /// Avanza el reloj un cuadro (y pasa a la toma siguiente cuando se acaba esta).
    pub fn avanzar(&mut self) {
        self.t += DT;
        self.nueva = false;
        if self.t >= self.actual().segundos - DT * 0.5 {
            self.toma += 1;
            self.t = 0.0;
            self.nueva = true;
            if !self.termino() {
                let total: f32 = GUION.iter().map(|t| t.segundos).sum();
                let hecho: f32 = GUION[..self.toma].iter().map(|t| t.segundos).sum();
                eprintln!("grabacion: toma {}/{} {:?} ({:.0}/{:.0} s de video, {:.0} s reales)", self.toma + 1, GUION.len(), GUION[self.toma].modos, hecho, total, self.empezo.elapsed().as_secs_f32());
            }
        }
    }

    /// Cierra el video y lo junta con el sonido (el `.wav` de la mezcla).
    pub fn terminar(&mut self, wav: Option<Vec<u8>>) {
        if let Some(mut h) = self.ffmpeg.take() {
            drop(h.stdin.take());
            let _ = h.wait();
        }
        let Some(wav) = wav else {
            let _ = std::fs::rename(&self.mudo, &self.archivo);
            eprintln!("grabacion: {} (sin sonido)", self.archivo);
            return;
        };
        let audio = format!("{}.wav", self.archivo);
        std::fs::write(&audio, wav).expect("no se pudo guardar el sonido");
        let ok = Command::new(ffmpeg())
            .args(["-y", "-loglevel", "error", "-i", &self.mudo, "-i", &audio, "-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-shortest", "-movflags", "+faststart", &self.archivo])
            .status()
            .is_ok_and(|s| s.success());
        if ok {
            let _ = std::fs::remove_file(&self.mudo);
            let _ = std::fs::remove_file(&audio);
        }
        eprintln!("grabacion: {} ({} cuadros, {:.0} s reales)", self.archivo, self.cuadros, self.empezo.elapsed().as_secs_f32());
    }
}

/// ffmpeg: el de Homebrew si esta, si no el del PATH.
fn ffmpeg() -> &'static str {
    ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg"].into_iter().find(|r| std::path::Path::new(r).exists()).unwrap_or("ffmpeg")
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Cada toma dura lo bastante para su fundido, y los pulsos caen una vez por periodo.
    #[test]
    fn el_guion_tiene_tomas_que_se_alcanzan_a_ver() {
        assert!(GUION.iter().all(|t| t.segundos > FUNDIDO * 2.0 + 1.0 && !t.modos.is_empty()));
        let mut g = Grabacion::new("x.mp4");
        let mut pulsos = 0;
        for _ in 0..(CUADROS * 3 + 2) {
            g.avanzar();
            if g.toma > 0 {
                break;
            }
            pulsos += g.pulso(1.0, 0.0) as u32;
        }
        assert_eq!(pulsos, 3);
    }
}
