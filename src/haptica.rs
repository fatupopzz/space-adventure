//! Los hapticos de Apple para el control (solo en macOS; ver `nativo/haptica_mac.m` y `build.rs`).
//!
//! El framework GameController encuentra el control y Core Haptics lo hace vibrar con un evento
//! continuo de cierta INTENSIDAD (que tan fuerte) y NITIDEZ (grave y redondo, o agudo y seco). Es
//! lo que Apple soporta para el DualSense, el DualShock 4 y los de Xbox: por SDL (HID directo) el
//! DualSense acepta la orden en macOS y no vibra.

use std::ffi::{c_char, c_float, c_int, CStr};

extern "C" {
    fn haptica_iniciar() -> c_int;
    fn haptica_cuantos() -> c_int;
    fn haptica_hay_control() -> c_int;
    fn haptica_nombre() -> *const c_char;
    fn haptica_error() -> *const c_char;
    fn haptica_vibrar(intensidad: c_float, nitidez: c_float, segundos: c_float) -> c_int;
    fn haptica_leer(ejes: *mut c_float, botones: *mut std::ffi::c_uint) -> c_int;
}

/// Lo que dice el framework de Apple del control ahorita, en la convencion de raylib.
#[derive(Clone, Debug)]
pub struct Lectura {
    pub ejes: [f32; 6],
    pub botones: u32,
    pub nombre: String,
}

/// Lee el control por GameController (empieza a escucharlos la primera vez).
pub fn leer() -> Option<Lectura> {
    unsafe {
        haptica_iniciar();
        let (mut ejes, mut botones) = ([0.0f32; 6], 0u32);
        (haptica_leer(ejes.as_mut_ptr(), &mut botones) == 1).then(|| Lectura { ejes, botones, nombre: texto(haptica_nombre()) })
    }
}

fn texto(p: *const c_char) -> String {
    if p.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
    }
}

pub struct Haptica;

impl Haptica {
    /// Empieza a escuchar controles (los encuentra unos cuadros despues: llegan por la cola de
    /// eventos de la ventana).
    pub fn abrir() -> Option<Haptica> {
        (unsafe { haptica_iniciar() } == 1).then_some(Haptica)
    }

    pub fn hay_control(&self) -> bool {
        unsafe { haptica_hay_control() == 1 }
    }

    /// Vibra con los dos "motores" del juego (`grave` y `agudo`, de 0 a 1) por `ms` milisegundos:
    /// la intensidad es el mas fuerte, y la nitidez sale de cuanto pesa el agudo.
    pub fn vibrar(&self, grave: f32, agudo: f32, ms: u32) -> bool {
        let intensidad = grave.max(agudo).clamp(0.0, 1.0);
        if intensidad <= 0.0 {
            return true;
        }
        let nitidez = (agudo / (grave + agudo).max(1e-3)).clamp(0.0, 1.0);
        unsafe { haptica_vibrar(intensidad, nitidez, ms as f32 / 1000.0) == 0 }
    }

    pub fn error(&self) -> String {
        texto(unsafe { haptica_error() })
    }

    /// Para `--probar-vibracion`: busca el control unos segundos y lo hace vibrar tres veces
    /// (grave, agudo y las dos cosas). `bombear` mueve los eventos de la ventana.
    pub fn diagnostico(bombear: &dyn Fn()) {
        println!("PRUEBA de los hapticos de Apple (GameController + Core Haptics)");
        let Some(h) = Haptica::abrir() else {
            println!("  No arranco.");
            return;
        };
        for _ in 0..90 {
            bombear();
            std::thread::sleep(std::time::Duration::from_millis(33));
            if h.hay_control() {
                break;
            }
        }
        println!("  Controles que ve: {} ({:?}). Con hapticos: {}", unsafe { haptica_cuantos() }, texto(unsafe { haptica_nombre() }), h.hay_control());
        for (k, (grave, agudo)) in [(1.0, 0.0), (0.0, 1.0), (1.0, 1.0)].into_iter().enumerate() {
            let ok = h.vibrar(grave, agudo, 1500);
            println!("  >>> VIBRANDO {} (grave {grave}, agudo {agudo}): {}", k + 1, if ok { "ok".to_string() } else { format!("fallo: {}", h.error()) });
            for _ in 0..70 {
                bombear();
                std::thread::sleep(std::time::Duration::from_millis(33));
            }
        }
    }
}
