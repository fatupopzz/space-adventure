//! Iconos del pack Controllers and Keyboard de Vryell, pasados a los lilas del juego con
//! `herramientas/recolorear_controles.py` (las letras y figuras de color de los botones no cambian).
use crate::entrada::Control;
use raylib::{ffi, prelude::*};
use std::{cell::{Cell, RefCell}, ffi::CString};

const HOJAS: [&str; 7] = ["kb_light_all", "controller_ps", "controller_xbox", "controller_switch", "sticks", "mouse", "controller_minimal"];

thread_local! {
    static TEXTURAS: RefCell<Vec<ffi::Texture2D>> = const { RefCell::new(Vec::new()) };
    static CONTROL: Cell<Control> = const { Cell::new(Control::Teclado) };
}

pub fn usar(control: Control) { CONTROL.set(control); }

/// El control que se esta usando (para nombrar sus botones en los textos).
pub fn control() -> Control { CONTROL.get() }

pub fn cargar(_hilo: &RaylibThread) {
    descargar();
    TEXTURAS.with(|ts| {
        let mut ts = ts.borrow_mut();
        for hoja in HOJAS {
            let ruta = CString::new(format!("assets/ui/controles/{hoja}.png")).unwrap();
            let t = unsafe { ffi::LoadTexture(ruta.as_ptr()) };
            if t.id != 0 { unsafe { ffi::SetTextureFilter(t, TextureFilter::TEXTURE_FILTER_POINT as i32) }; }
            ts.push(t);
        }
    });
}

pub fn descargar() {
    TEXTURAS.with(|ts| {
        for t in ts.borrow_mut().drain(..) {
            if t.id != 0 { unsafe { ffi::UnloadTexture(t) }; }
        }
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Icono { hoja: usize, columna: i32, fila: i32, resaltada: i32 }

fn tecla(columna: i32, fila: i32) -> Icono { Icono { hoja: 0, columna, fila, resaltada: 1 } }
fn mando(hoja: usize, fila: i32) -> Icono { Icono { hoja, columna: 0, fila, resaltada: 3 } }

fn icono(s: &str, control: Control) -> Option<Icono> {
    if control != Control::Teclado {
        let direccion = match s { "arriba" | "\u{E008}" => Some(1), "abajo" | "\u{E009}" => Some(3), "izquierda" | "<-" | "\u{E00A}" => Some(2), "derecha" | "->" | "\u{E00B}" => Some(4), _ => None };
        if let Some(columna) = direccion {
            let hoja = match control { Control::PlayStation => 1, Control::Switch => 3, _ => 2 };
            return Some(Icono { hoja, columna, fila: 9, resaltada: 0 });
        }
    }
    let i = match s {
        "\u{E000}" => mando(1, 4), "\u{E001}" => mando(1, 6),
        "\u{E002}" => mando(1, 2), "\u{E003}" => mando(1, 0),
        "\u{E004}" => mando(2, 4), "\u{E005}" => mando(2, 6),
        "\u{E006}" => mando(2, 2), "\u{E007}" => mando(2, 0),
        "\u{E00C}" => mando(3, 4), "\u{E00D}" => mando(3, 6),
        "\u{E00E}" => mando(3, 2), "\u{E00F}" => mando(3, 0),
        "ESPACIO" | "espacio" => tecla(8, 8), "Enter" | "ENTER" => tecla(8, 25),
        "ESC" => tecla(8, 0), "TAB" => tecla(8, 2), "shift" | "Shift" => tecla(8, 5),
        "Ctrl" => tecla(8, 6), "Alt" => tecla(8, 7),
        "\u{E008}" | "arriba" => tecla(12, 12), "\u{E009}" | "abajo" => tecla(12, 14),
        "\u{E00A}" | "<-" | "izquierda" => tecla(12, 13), "\u{E00B}" | "->" | "derecha" => tecla(12, 15),
        "L1" => mando(1, 24), "L2" => mando(1, 27), "R1" => mando(1, 30), "R2" => mando(1, 33),
        "LB" => mando(2, 16), "LT" => mando(2, 19), "RB" => mando(2, 22), "RT" => mando(2, 25),
        "L" if control == Control::Switch => mando(3, 26), "ZL" => mando(3, 29),
        "R" if control == Control::Switch => mando(3, 32), "ZR" => mando(3, 35),
        "Menu" => mando(2, 11), "View" => mando(2, 13),
        "Options" => mando(1, 19), "Create" => mando(1, 19),
        "+" if control == Control::Switch => mando(3, 19), "-" if control == Control::Switch => mando(3, 21),
        "stick" | "stick izq." | "L3" => Icono { hoja: 4, columna: 0, fila: 3, resaltada: 0 },
        "R3" => Icono { hoja: 4, columna: 0, fila: 7, resaltada: 0 },
        "cruceta" => match control {
            Control::PlayStation => mando(1, 8), Control::Switch => mando(3, 8), _ => mando(2, 8),
        },
        "ratón" | "raton" => Icono { hoja: 5, columna: 0, fila: 1, resaltada: 0 },
        "clic" | "clic izq." => Icono { hoja: 5, columna: 1, fila: 1, resaltada: 0 },
        _ if s.len() == 1 && s.as_bytes()[0].is_ascii_uppercase() => tecla(0, (s.as_bytes()[0] - b'A') as i32),
        _ if s.len() == 1 && s.as_bytes()[0].is_ascii_digit() => {
            let n = (s.as_bytes()[0] - b'0') as i32;
            tecla(4, if n == 0 { 21 } else { 11 + n })
        }
        _ if s.starts_with('F') && s[1..].parse::<i32>().is_ok_and(|n| (1..=12).contains(&n)) => tecla(4, s[1..].parse::<i32>().ok()? - 1),
        _ => return None,
    };
    Some(i)
}

fn piezas(s: &str, control: Control) -> Option<Vec<Icono>> {
    let s = match s {
        "flechas" | "flechas: escoger y cambiar" if control != Control::Teclado => "cruceta",
        "flechas" => "arriba / abajo / izquierda / derecha",
        "1 a 7" => "1 / 7",
        "1 a 4" => "1 / 4",
        "1 a 3" => "1 / 3",
        "<- ->" => "izquierda / derecha",
        "\u{E008} \u{E009}" => "arriba / abajo",
        "\u{E00A} \u{E00B}" => "izquierda / derecha",
        _ => s,
    };
    s.split(" / ").map(|p| icono(p, control)).collect()
}

/// El mismo tamaño para medir y dibujar; dos pixeles de pantalla por pixel del atlas.
pub fn lado(tam: i32) -> i32 { 16 * ((tam + 15) / 16).clamp(2, 4) }

fn disponibles(iconos: &[Icono]) -> bool {
    TEXTURAS.with(|ts| iconos.iter().all(|i| ts.borrow().get(i.hoja).is_some_and(|t| t.id != 0)))
}

fn hueco(s: &str) -> i32 { if matches!(s, "1 a 7" | "1 a 4" | "1 a 3") { 12 } else { 3 } }

pub fn ancho(s: &str, tam: i32) -> Option<i32> {
    let iconos = piezas(s, CONTROL.get())?;
    disponibles(&iconos).then(|| iconos.len() as i32 * lado(tam) + (iconos.len() as i32 - 1) * hueco(s))
}

fn pintar(i: Icono, x: i32, y: i32, lado: i32, resaltada: bool, alfa: u8) {
    TEXTURAS.with(|ts| {
        let ts = ts.borrow();
        let Some(t) = ts.get(i.hoja).filter(|t| t.id != 0) else { return };
        let columna = i.columna + if resaltada { i.resaltada } else { 0 };
        unsafe {
            ffi::DrawTexturePro(*t,
                ffi::Rectangle { x: (columna * 16) as f32, y: (i.fila * 16) as f32, width: 16.0, height: 16.0 },
                ffi::Rectangle { x: x as f32, y: y as f32, width: lado as f32, height: lado as f32 },
                ffi::Vector2 { x: 0.0, y: 0.0 }, 0.0, Color::new(255, 255, 255, alfa).into());
        }
    });
}

pub fn dibujar(s: &str, x: i32, y: i32, tam: i32, resaltada: bool) -> Option<i32> {
    dibujar_de(s, x, y, lado(tam), resaltada, 255)
}

fn dibujar_de(s: &str, x: i32, y: i32, l: i32, resaltada: bool, alfa: u8) -> Option<i32> {
    let iconos = piezas(s, CONTROL.get())?;
    if !disponibles(&iconos) { return None; }
    let sep = hueco(s);
    for (k, &i) in iconos.iter().enumerate() {
        pintar(i, x + k as i32 * (l + sep), y, l, resaltada, alfa);
        if sep > 3 && k + 1 < iconos.len() {
            unsafe { ffi::DrawRectangle(x + k as i32 * (l + sep) + l + 2, y + l / 2, sep - 4, 2, Color::new(185, 195, 220, 255).into()); }
        }
    }
    Some(iconos.len() as i32 * (l + sep) - sep)
}

/// El lado de un boton metido en una frase de letra `tam` (en multiplos de 16: pixeles parejos).
pub fn lado_en_linea(tam: i32) -> i32 {
    16 * if tam >= 30 { 3 } else if tam >= 13 { 2 } else { 1 }
}

/// Lo ancho de un boton metido en una frase (ver `ui::boton`), si hay icono para el.
pub fn ancho_en_linea(s: &str, tam: i32) -> Option<i32> {
    let iconos = piezas(s, CONTROL.get())?;
    let l = lado_en_linea(tam);
    disponibles(&iconos).then(|| iconos.len() as i32 * l + (iconos.len() as i32 - 1) * hueco(s))
}

/// Dibuja un boton metido en una frase, centrado en el renglon que empieza en `y`. Las teclas van
/// tal cual (lilas: se leen sobre lo oscuro); los botones del control, con su orilla clara.
pub fn dibujar_en_linea(s: &str, x: i32, y: i32, tam: i32, alfa: u8) -> Option<i32> {
    let l = lado_en_linea(tam);
    dibujar_de(s, x, y + (tam - l) / 2, l, CONTROL.get() != Control::Teclado, alfa)
}

/// Botones insertados dentro del texto: misma linea base y ancho que los simbolos anteriores.
pub fn en_linea(s: char, x: f32, y: f32, tam: f32, alfa: u8) -> bool {
    // Las flechas de texto siguen siendo flechas; no son siempre teclas (p. ej. paginacion).
    if ('\u{E008}'..='\u{E00B}').contains(&s) { return false; }
    let Some(i) = icono(&s.to_string(), CONTROL.get()) else { return false };
    if !disponibles(&[i]) { return false; }
    let l = 16 * ((tam / 16.0).round() as i32).max(1);
    pintar(i, (x + (tam - l as f32) * 0.5).round() as i32, (y + (tam - l as f32) * 0.5).round() as i32, l, true, alfa);
    true
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn cada_accion_tiene_un_recorte_valido_en_su_hoja() {
        let imagenes: Vec<_> = HOJAS.iter().map(|n| Image::load_image(&format!("assets/ui/controles/{n}.png")).unwrap()).collect();
        for control in [Control::Teclado, Control::PlayStation, Control::Xbox, Control::Switch, Control::Otro] {
            let e = crate::entrada::Entrada { control, ..Default::default() };
            for accion in ["aceptar", "aterrizar", "barrel", "objetivo", "pestana_izq", "pestana_der", "turbo", "acelerar", "vista", "info", "pausa", "pasaporte", "piso", "aire", "apuntar", "gatillo"] {
                let recortes = piezas(e.nombre(accion), control).unwrap_or_else(|| panic!("sin icono: {control:?} {accion}"));
                for i in recortes {
                    let img = &imagenes[i.hoja];
                    let datos = img.get_image_data();
                    for col in [i.columna, i.columna + i.resaltada] {
                        assert!((col + 1) * 16 <= img.width() && (i.fila + 1) * 16 <= img.height());
                        assert!((i.fila * 16..(i.fila + 1) * 16).any(|y| (col * 16..(col + 1) * 16).any(|x| datos[(y * img.width() + x) as usize].a > 0)), "recorte vacio: {control:?} {accion}");
                    }
                }
            }
        }
    }
}
