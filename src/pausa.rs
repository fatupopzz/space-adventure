//! El menu de pausa (ESC o start) y la pantalla de AJUSTES (que tambien se
//! abre desde el menu principal). Los ajustes se guardan con la partida (ver
//! `progreso.rs`).
//!
//! - Volumen general, de la musica y de los efectos (cinco niveles cada uno).
//! - Calidad: el tamano del framebuffer. En compus lentas, "rapida" dibuja
//!   menos pixeles (el rasterizador es por software: cada pixel cuesta).
//! - Pantalla completa y el HUD.
//! - Cabeceo invertido (como en los simuladores: abajo sube la nariz), volar
//!   con el raton y su sensibilidad.
//! - Que tan rapido salen los dialogos.
//! - La vibracion del control (si hay SDL2, ver `vibracion.rs`).
//! - El estilo: "dibujado" (luz en bandas, contorno de tinta y sombras de contacto) o clasico.

use crate::entrada::Entrada;
use crate::ui;
use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ajustes {
    /// De 0 (mudo) a 4 (todo): general, musica y efectos (y voces).
    pub volumen: u8,
    pub musica: u8,
    pub efectos: u8,
    pub calidad_rapida: bool,
    pub invertir: bool,
    pub raton: bool,
    /// Sensibilidad del raton, de 1 a 5.
    pub sensibilidad: u8,
    /// Que tan rapido salen los dialogos: 0 lento, 1 normal, 2 rapido, 3 de golpe.
    pub texto: u8,
    pub vibracion: bool,
    /// El estilo dibujado (como caricatura) o el clasico (degradados, sin contorno).
    pub dibujado: bool,
}

impl Default for Ajustes {
    fn default() -> Self {
        Ajustes { volumen: 3, musica: 4, efectos: 4, calidad_rapida: false, invertir: false, raton: false, sensibilidad: 3, texto: 1, vibracion: true, dibujado: true }
    }
}

impl Ajustes {
    /// Letras por segundo de los dialogos.
    pub fn letras_por_segundo(&self) -> f32 {
        [22.0, 38.0, 70.0, 10_000.0][self.texto.min(3) as usize]
    }
}

/// Lo que pide el menu.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccionPausa {
    Nada,
    /// Cerrar el menu (seguir jugando, o volver al menu principal si se abrio desde ahi).
    Continuar,
    /// Algo de los ajustes cambio (hay que aplicarlo y guardarlo).
    Ajustes,
    PantallaCompleta,
    Hud,
    /// Al menu principal del juego.
    Menu,
    Salir,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pantalla {
    Pausa,
    Ajustes,
}

/// Las filas de los ajustes, en orden.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fila {
    Volumen,
    Musica,
    Efectos,
    Calidad,
    PantallaCompleta,
    Hud,
    Invertir,
    Raton,
    Sensibilidad,
    Texto,
    Vibracion,
    Estilo,
    Volver,
}

const FILAS: [Fila; 13] = [
    Fila::Volumen,
    Fila::Musica,
    Fila::Efectos,
    Fila::Calidad,
    Fila::PantallaCompleta,
    Fila::Hud,
    Fila::Invertir,
    Fila::Raton,
    Fila::Sensibilidad,
    Fila::Texto,
    Fila::Vibracion,
    Fila::Estilo,
    Fila::Volver,
];
/// La pestana JUGAR del menu del juego (personajes, pase, tienda y ajustes son las otras pestanas).
const PAUSA: [&str; 3] = ["Continuar", "Menú principal", "Salir del juego"];

pub struct Pausa {
    pantalla: Pantalla,
    cursor: usize,
    /// Segundos desde que se movio el cursor (para la banda que entra deslizandose).
    desde: f32,
    reloj: f32,
}

impl Pausa {
    /// La pausa del juego.
    pub fn new() -> Self {
        Pausa { pantalla: Pantalla::Pausa, cursor: 0, desde: 0.0, reloj: 0.0 }
    }

    /// La pestana de ajustes.
    pub fn ajustes() -> Self {
        Pausa { pantalla: Pantalla::Ajustes, cursor: 0, desde: 0.0, reloj: 0.0 }
    }

    pub fn es_ajustes(&self) -> bool {
        self.pantalla == Pantalla::Ajustes
    }

    fn filas(&self) -> usize {
        match self.pantalla {
            Pantalla::Pausa => PAUSA.len(),
            Pantalla::Ajustes => FILAS.len(),
        }
    }

    /// Volver: cierra el menu (de vuelta al juego, o a la lista del menu principal).
    pub fn atras(&mut self) -> AccionPausa {
        AccionPausa::Continuar
    }

    pub fn actualizar(&mut self, e: &Entrada, a: &mut Ajustes, dt: f32) -> AccionPausa {
        self.desde += dt;
        self.reloj += dt;
        let n = self.filas();
        if e.abajo {
            self.cursor = (self.cursor + 1) % n;
            self.desde = 0.0;
        }
        if e.arriba {
            self.cursor = (self.cursor + n - 1) % n;
            self.desde = 0.0;
        }
        if self.pantalla == Pantalla::Pausa {
            if !e.aceptar {
                return AccionPausa::Nada;
            }
            return match self.cursor {
                0 => AccionPausa::Continuar,
                1 => AccionPausa::Menu,
                _ => AccionPausa::Salir,
            };
        }
        let mas = e.derecha || e.aceptar;
        let menos = e.izquierda;
        if !(mas || menos) {
            return AccionPausa::Nada;
        }
        // Un nivel de `minimo` a `tope`: con flechas sube y baja; con aceptar da la vuelta.
        let nivel = |v: &mut u8, tope: u8, minimo: u8| {
            *v = if e.aceptar && *v >= tope { minimo } else if mas { (*v + 1).min(tope) } else { v.saturating_sub(1).max(minimo) };
        };
        match FILAS[self.cursor] {
            Fila::Volumen => nivel(&mut a.volumen, 4, 0),
            Fila::Musica => nivel(&mut a.musica, 4, 0),
            Fila::Efectos => nivel(&mut a.efectos, 4, 0),
            Fila::Calidad => a.calidad_rapida = !a.calidad_rapida,
            Fila::PantallaCompleta => return AccionPausa::PantallaCompleta,
            Fila::Hud => return AccionPausa::Hud,
            Fila::Invertir => a.invertir = !a.invertir,
            Fila::Raton => a.raton = !a.raton,
            Fila::Sensibilidad => nivel(&mut a.sensibilidad, 5, 1),
            Fila::Texto => nivel(&mut a.texto, 3, 0),
            Fila::Vibracion => a.vibracion = !a.vibracion,
            Fila::Estilo => a.dibujado = !a.dibujado,
            Fila::Volver if e.aceptar => return self.atras(),
            Fila::Volver => return AccionPausa::Nada,
        }
        AccionPausa::Ajustes
    }

    /// `vibracion` dice si hay como hacer vibrar (SDL2 instalado) y si hay control conectado.
    #[allow(clippy::too_many_arguments)]
    pub fn dibujar(&self, d: &mut RaylibDrawHandle, a: &Ajustes, hud: bool, vibracion: (bool, bool), e: &Entrada, ancho: i32, alto: i32) {
        let barras = |n: u8, tope: u8| format!("[{}]", (0..tope).map(|k| if k < n { '#' } else { '-' }).collect::<String>());
        let fila = |nombre: &str, valor: String| ui::Fila { nombre: nombre.into(), valor, apagada: false };
        let (titulo, sub, filas, ayuda): (&str, &str, Vec<ui::Fila>, String) = match self.pantalla {
            Pantalla::Pausa => (
                "PAUSA",
                "la nave flota quietecita",
                PAUSA.iter().map(|p| fila(p, String::new())).collect(),
                format!("{}: escoger     {}: aceptar     {}: volver al juego", ui::boton("flechas"), ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("pausa"))),
            ),
            Pantalla::Ajustes => (
                "AJUSTES",
                "como le guste a usted",
                FILAS
                    .iter()
                    .map(|f| match f {
                        Fila::Volumen => fila("Volumen general", if a.volumen == 0 { "mudo".into() } else { barras(a.volumen, 4) }),
                        Fila::Musica => fila("Música", if a.musica == 0 { "sin música".into() } else { barras(a.musica, 4) }),
                        Fila::Efectos => fila("Efectos y voces", if a.efectos == 0 { "sin efectos".into() } else { barras(a.efectos, 4) }),
                        Fila::Calidad => fila("Calidad", if a.calidad_rapida { "rápida".into() } else { "alta".into() }),
                        Fila::PantallaCompleta => fila("Pantalla completa", "F11".into()),
                        Fila::Hud => fila("HUD", if hud { "visible".into() } else { "oculto".into() }),
                        Fila::Invertir => fila("Invertir arriba / abajo de la nave", if a.invertir { "sí: abajo sube".into() } else { "no".into() }),
                        Fila::Raton => fila("Volar con el ratón", if a.raton { "sí".into() } else { "no".into() }),
                        Fila::Sensibilidad => fila("Sensibilidad del ratón", barras(a.sensibilidad, 5)),
                        Fila::Texto => fila("Velocidad del texto", ["lenta", "normal", "rápida", "de golpe"][a.texto.min(3) as usize].into()),
                        Fila::Vibracion => fila(
                            "Vibración del control",
                            match (a.vibracion, vibracion) {
                                (_, (false, _)) => "falta SDL2".into(),
                                (false, _) => "no".into(),
                                (true, (true, false)) => "sí (sin control)".into(),
                                (true, (true, true)) => "sí".into(),
                            },
                        ),
                        Fila::Estilo => fila("Estilo de dibujo", if a.dibujado { "dibujado (caricatura)".into() } else { "clásico".into() }),
                        Fila::Volver => fila("Volver", String::new()),
                    })
                    .collect(),
                format!("{}: escoger y cambiar     {}: aceptar     {}: volver", ui::boton("flechas"), ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("pausa"))),
            ),
        };
        ui::menu_estilo(d, titulo, sub, &filas, self.cursor, self.desde, self.reloj, &ayuda, ancho, alto);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn tecla(abajo: bool, derecha: bool, izquierda: bool, aceptar: bool) -> Entrada {
        Entrada { abajo, derecha, izquierda, aceptar, ..Default::default() }
    }

    #[test]
    fn la_pausa_va_al_menu_y_los_ajustes_cambian() {
        let mut p = Pausa::new();
        let mut a = Ajustes::default();
        p.actualizar(&tecla(true, false, false, false), &mut a, 0.1);
        assert_eq!(p.actualizar(&tecla(false, false, false, true), &mut a, 0.1), AccionPausa::Menu);
        let mut p = Pausa::ajustes();
        // Volumen: sube, baja.
        assert_eq!(p.actualizar(&tecla(false, true, false, false), &mut a, 0.1), AccionPausa::Ajustes);
        assert_eq!(a.volumen, 4);
        p.actualizar(&tecla(false, false, true, false), &mut a, 0.1);
        p.actualizar(&tecla(false, false, true, false), &mut a, 0.1);
        assert_eq!(a.volumen, 2);
        // La vibracion se apaga.
        for _ in 0..10 {
            p.actualizar(&tecla(true, false, false, false), &mut a, 0.1);
        }
        p.actualizar(&tecla(false, true, false, false), &mut a, 0.1);
        assert!(!a.vibracion);
        // El estilo: de dibujado a clasico.
        p.actualizar(&tecla(true, false, false, false), &mut a, 0.1);
        p.actualizar(&tecla(false, true, false, false), &mut a, 0.1);
        assert!(!a.dibujado);
        // Volver: cierra.
        p.actualizar(&tecla(true, false, false, false), &mut a, 0.1);
        assert_eq!(p.actualizar(&tecla(false, false, false, true), &mut a, 0.1), AccionPausa::Continuar);
    }

    #[test]
    fn desde_el_menu_solo_son_ajustes() {
        let mut p = Pausa::ajustes();
        let mut a = Ajustes::default();
        assert_eq!(p.pantalla, Pantalla::Ajustes);
        for _ in 0..9 {
            p.actualizar(&tecla(true, false, false, false), &mut a, 0.1);
        }
        p.actualizar(&tecla(false, true, false, false), &mut a, 0.1);
        assert_eq!(a.texto, 2);
        assert!(a.letras_por_segundo() > 38.0);
        assert_eq!(p.atras(), AccionPausa::Continuar);
    }
}
