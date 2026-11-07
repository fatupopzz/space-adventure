//! La intro dibujada a mano (los dibujos de Fatima y la voz de Pluton) como video de inicio.
//!
//! El juego no trae un reproductor de video: le pide a ffmpeg que decodifique
//! `assets/intro/intro.mp4` y le mande los cuadros en crudo por una tuberia; un hilo los va
//! leyendo y el juego los pinta en el framebuffer al ritmo del reloj (30 por segundo). El sonido va
//! aparte (`assets/intro/intro.mp3`, ver `sonido.rs`). Si no hay ffmpeg o falta el video, el juego
//! usa la cinematica de siempre, en vivo.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{sync_channel, Receiver};

pub const VIDEO: &str = "assets/intro/intro.mp4";
/// En alta: se dibuja directo en la pantalla, sin pasar por la imagen interna del juego.
pub const ANCHO: usize = 1920;
pub const ALTO: usize = 1080;
const CUADROS_POR_SEGUNDO: f32 = 30.0;
/// Lo que dura la intro, en segundos.
pub const DURACION: f32 = 32.4;

pub struct IntroVideo {
    proceso: Child,
    cuadros: Receiver<Vec<u8>>,
    /// El cuadro que se esta mostrando, y cuantos van.
    pub actual: Vec<u8>,
    mostrados: usize,
    pub t: f32,
    se_acabo: bool,
}

impl IntroVideo {
    pub fn abrir() -> Option<Self> {
        if !std::path::Path::new(VIDEO).exists() {
            return None;
        }
        let ffmpeg = ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg", "ffmpeg"]
            .into_iter()
            .find(|f| !f.starts_with('/') || std::path::Path::new(f).exists())?;
        let mut proceso = Command::new(ffmpeg)
            .args(["-v", "error", "-i", VIDEO, "-f", "rawvideo", "-pix_fmt", "rgba", "-s", &format!("{ANCHO}x{ALTO}"), "-"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let mut salida = proceso.stdout.take()?;
        // Unos cuantos cuadros de adelanto, para que nunca se trabe.
        let (manda, cuadros) = sync_channel::<Vec<u8>>(12);
        std::thread::spawn(move || loop {
            let mut cuadro = vec![0u8; ANCHO * ALTO * 4];
            if salida.read_exact(&mut cuadro).is_err() || manda.send(cuadro).is_err() {
                break;
            }
        });
        Some(IntroVideo { proceso, cuadros, actual: vec![0; ANCHO * ALTO * 4], mostrados: 0, t: 0.0, se_acabo: false })
    }

    /// Avanza el reloj y trae los cuadros que tocan. Devuelve si ya se acabo.
    pub fn actualizar(&mut self, dt: f32) -> bool {
        self.t += dt;
        let toca = (self.t * CUADROS_POR_SEGUNDO) as usize + 1;
        while self.mostrados < toca && !self.se_acabo {
            match self.cuadros.recv() {
                Ok(c) => {
                    self.actual = c;
                    self.mostrados += 1;
                }
                Err(_) => self.se_acabo = true,
            }
        }
        self.se_acabo || self.t >= DURACION
    }
}

impl Drop for IntroVideo {
    fn drop(&mut self) {
        let _ = self.proceso.kill();
        let _ = self.proceso.wait();
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// El video se abre, sus cuadros llegan al ritmo del reloj y no salen en negro.
    #[test]
    fn la_intro_se_decodifica_y_avanza() {
        let Some(mut v) = IntroVideo::abrir() else {
            eprintln!("sin ffmpeg o sin video: el juego usa la cinematica en vivo");
            return;
        };
        let mut acabo = false;
        for _ in 0..150 {
            acabo = v.actualizar(1.0 / 30.0);
        }
        assert!(!acabo, "a los 5 s todavia no termina");
        assert!(v.mostrados >= 150);
        assert_eq!(v.actual.len(), ANCHO * ALTO * 4);
        let brillo: u32 = v.actual.iter().step_by(97).map(|&b| b as u32).sum();
        assert!(brillo > 0, "el cuadro salio negro");
        while !v.actualizar(1.0 / 30.0) {}
        assert!(v.t >= DURACION - 0.5, "se acabo antes de tiempo: {}", v.t);
    }
}
