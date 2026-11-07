//! La vibracion del control (DualShock 4, DualSense, Xbox...).
//!
//! raylib, en escritorio, no sabe hacer vibrar los controles (su
//! `SetGamepadVibration` solo avisa "no disponible"). SDL2 si, incluso los de
//! PlayStation por Bluetooth. Pero para no obligar a nadie a instalar SDL2
//! para poder jugar, la libreria se abre al arrancar (como un plugin, con
//! `dlopen`): si esta, el control vibra; si no, el juego sigue igual, sin
//! vibracion. Solo se usan unas pocas funciones de SDL: abrir el control y
//! pedirle una vibracion (motor grave y motor agudo, por unos milisegundos).

use std::cell::Cell;
use std::ffi::{c_char, c_int, c_void, CString};

type FnInit = unsafe extern "C" fn(u32) -> c_int;
type FnHint = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type FnNum = unsafe extern "C" fn() -> c_int;
type FnEsControl = unsafe extern "C" fn(c_int) -> c_int;
type FnAbrir = unsafe extern "C" fn(c_int) -> *mut c_void;
type FnVibrar = unsafe extern "C" fn(*mut c_void, u16, u16, u32) -> c_int;
type FnActualizar = unsafe extern "C" fn();
type FnConectado = unsafe extern "C" fn(*mut c_void) -> c_int;

const SDL_INIT_JOYSTICK: u32 = 0x0000_0200;
const SDL_INIT_GAMECONTROLLER: u32 = 0x0000_2000;

#[cfg(unix)]
mod sistema {
    use std::ffi::{c_char, c_int, c_void};
    extern "C" {
        pub fn dlopen(nombre: *const c_char, banderas: c_int) -> *mut c_void;
        pub fn dlsym(libreria: *mut c_void, simbolo: *const c_char) -> *mut c_void;
    }
    pub const RTLD_NOW: c_int = 2;
    pub unsafe fn abrir(nombre: *const c_char) -> *mut c_void {
        dlopen(nombre, RTLD_NOW)
    }
    pub unsafe fn simbolo(l: *mut c_void, s: *const c_char) -> *mut c_void {
        dlsym(l, s)
    }
}

#[cfg(windows)]
mod sistema {
    use std::ffi::{c_char, c_void};
    extern "system" {
        fn LoadLibraryA(nombre: *const c_char) -> *mut c_void;
        fn GetProcAddress(libreria: *mut c_void, simbolo: *const c_char) -> *mut c_void;
    }
    pub unsafe fn abrir(nombre: *const c_char) -> *mut c_void {
        LoadLibraryA(nombre)
    }
    pub unsafe fn simbolo(l: *mut c_void, s: *const c_char) -> *mut c_void {
        GetProcAddress(l, s)
    }
}

/// Donde se busca SDL2 en cada sistema.
#[cfg(target_os = "macos")]
const LUGARES: [&str; 4] = ["/opt/homebrew/lib/libSDL2-2.0.0.dylib", "/usr/local/lib/libSDL2-2.0.0.dylib", "libSDL2-2.0.0.dylib", "libSDL2.dylib"];
#[cfg(all(unix, not(target_os = "macos")))]
const LUGARES: [&str; 2] = ["libSDL2-2.0.so.0", "libSDL2.so"];
#[cfg(windows)]
const LUGARES: [&str; 1] = ["SDL2.dll"];

/// Por donde le habla SDL al control. En macOS hay dos caminos y depende del control y de la
/// version del sistema cual vibra de verdad (el otro acepta la orden y no hace nada).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Modo {
    /// HIDAPI: SDL le habla directo al control (sus propios reportes HID).
    Directo,
    /// El framework GameController de Apple (MFi), con su motor de hapticos (Core Haptics).
    Apple,
    /// Los dos, como viene SDL de fabrica (HIDAPI primero; lo que no agarre, Apple).
    Ambos,
}

impl Modo {
    pub const TODOS: [Modo; 3] = [Modo::Apple, Modo::Directo, Modo::Ambos];

    fn pistas(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Modo::Directo => &[("SDL_JOYSTICK_HIDAPI", "1"), ("SDL_JOYSTICK_HIDAPI_PS4_RUMBLE", "1"), ("SDL_JOYSTICK_HIDAPI_PS5_RUMBLE", "1"), ("SDL_JOYSTICK_MFI", "0")],
            Modo::Apple => &[("SDL_JOYSTICK_HIDAPI", "0"), ("SDL_JOYSTICK_MFI", "1")],
            Modo::Ambos => &[("SDL_JOYSTICK_HIDAPI", "1"), ("SDL_JOYSTICK_HIDAPI_PS4_RUMBLE", "1"), ("SDL_JOYSTICK_HIDAPI_PS5_RUMBLE", "1"), ("SDL_JOYSTICK_MFI", "1")],
        }
    }

    pub fn nombre(self) -> &'static str {
        match self {
            Modo::Directo => "directo (HIDAPI)",
            Modo::Apple => "Apple (GameController + Core Haptics)",
            Modo::Ambos => "los dos (de fabrica)",
        }
    }

    /// El que usa el juego: el de la variable `SPACE_VIBRACION` (directo, apple o ambos) si esta,
    /// si no el que funciona en macOS.
    pub fn del_juego() -> Modo {
        match std::env::var("SPACE_VIBRACION").ok().as_deref() {
            Some("directo") => Modo::Directo,
            Some("apple") => Modo::Apple,
            Some("ambos") => Modo::Ambos,
            _ => MODO_DE_FABRICA,
        }
    }
}

/// El camino que vibra de verdad en macOS (ver `--probar-vibracion`).
#[cfg(target_os = "macos")]
const MODO_DE_FABRICA: Modo = Modo::Directo;
#[cfg(not(target_os = "macos"))]
const MODO_DE_FABRICA: Modo = Modo::Directo;

/// Quien hace vibrar el control: en macOS, los hapticos de Apple (GameController + Core Haptics,
/// ver `haptica.rs`); en los demas sistemas (o con `SPACE_VIBRACION=directo`/`ambos`), SDL2.
pub enum Vibrador {
    #[cfg(haptica_apple)]
    Apple(crate::haptica::Haptica),
    Sdl(Vibracion),
}

impl Vibrador {
    pub fn abrir() -> Option<Self> {
        #[cfg(haptica_apple)]
        if !matches!(std::env::var("SPACE_VIBRACION").as_deref(), Ok("directo" | "ambos")) {
            if let Some(h) = crate::haptica::Haptica::abrir() {
                return Some(Vibrador::Apple(h));
            }
        }
        Vibracion::abrir().map(Vibrador::Sdl)
    }

    /// Cada cuadro (SDL necesita procesar el control; Apple lo hace solo).
    pub fn actualizar(&self, dt: f32) {
        if let Vibrador::Sdl(v) = self {
            v.actualizar(dt);
        }
    }

    pub fn hay_control(&self) -> bool {
        match self {
            #[cfg(haptica_apple)]
            Vibrador::Apple(h) => h.hay_control(),
            Vibrador::Sdl(v) => v.hay_control(),
        }
    }

    /// Vibra: `grave` y `agudo` de 0 a 1, por `ms` milisegundos.
    pub fn vibrar(&self, grave: f32, agudo: f32, ms: u32) {
        match self {
            #[cfg(haptica_apple)]
            Vibrador::Apple(h) => {
                h.vibrar(grave, agudo, ms);
            }
            Vibrador::Sdl(v) => v.vibrar(grave, agudo, ms),
        }
    }
}

pub struct Vibracion {
    num: FnNum,
    es_control: FnEsControl,
    abrir: FnAbrir,
    vibrar: FnVibrar,
    actualizar: FnActualizar,
    conectado: FnConectado,
    /// El control abierto (o nulo si no hay).
    control: Cell<*mut c_void>,
    /// Segundos hasta volver a buscar un control (por si se conecta despues).
    buscar_en: Cell<f32>,
}

impl Vibracion {
    /// Abre SDL2 si esta instalado. Sin SDL2 (o si no arranca), `None`: el juego sigue sin vibracion.
    pub fn abrir() -> Option<Self> {
        Self::abrir_con(Modo::del_juego())
    }

    pub fn abrir_con(modo: Modo) -> Option<Self> {
        unsafe {
            let libreria = LUGARES.iter().find_map(|l| {
                let c = CString::new(*l).ok()?;
                let h = sistema::abrir(c.as_ptr());
                (!h.is_null()).then_some(h)
            })?;
            let simbolo = |nombre: &str| {
                let c = CString::new(nombre).ok()?;
                let p = sistema::simbolo(libreria, c.as_ptr());
                (!p.is_null()).then_some(p)
            };
            let init: FnInit = std::mem::transmute(simbolo("SDL_Init")?);
            let hint: FnHint = std::mem::transmute(simbolo("SDL_SetHint")?);
            let v = Vibracion {
                num: std::mem::transmute(simbolo("SDL_NumJoysticks")?),
                es_control: std::mem::transmute(simbolo("SDL_IsGameController")?),
                abrir: std::mem::transmute(simbolo("SDL_GameControllerOpen")?),
                vibrar: std::mem::transmute(simbolo("SDL_GameControllerRumble")?),
                actualizar: std::mem::transmute(simbolo("SDL_GameControllerUpdate")?),
                conectado: std::mem::transmute(simbolo("SDL_GameControllerGetAttached")?),
                control: Cell::new(std::ptr::null_mut()),
                buscar_en: Cell::new(0.0),
            };
            // Los de PlayStation solo vibran por HIDAPI si se le pide a SDL (por Bluetooth hablan su
            // propio protocolo); por el camino de Apple, vibran con Core Haptics.
            for &(h, valor) in modo.pistas().iter().chain(&[("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1")]) {
                let (h, valor) = (CString::new(h).ok()?, CString::new(valor).ok()?);
                hint(h.as_ptr(), valor.as_ptr());
            }
            if init(SDL_INIT_JOYSTICK | SDL_INIT_GAMECONTROLLER) != 0 {
                return None;
            }
            Some(v)
        }
    }

    /// Cada cuadro: SDL procesa el control (y si no hay, de vez en cuando se busca uno).
    pub fn actualizar(&self, dt: f32) {
        unsafe {
            (self.actualizar)();
            let c = self.control.get();
            if !c.is_null() && (self.conectado)(c) == 0 {
                self.control.set(std::ptr::null_mut());
            }
            if self.control.get().is_null() {
                self.buscar_en.set(self.buscar_en.get() - dt);
                if self.buscar_en.get() <= 0.0 {
                    self.buscar_en.set(1.0);
                    for i in 0..(self.num)() {
                        if (self.es_control)(i) != 0 {
                            let c = (self.abrir)(i);
                            if !c.is_null() {
                                self.control.set(c);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn hay_control(&self) -> bool {
        !self.control.get().is_null()
    }

    /// Vibra: `grave` y `agudo` de 0 a 1 (los dos motores), por `ms` milisegundos.
    pub fn vibrar(&self, grave: f32, agudo: f32, ms: u32) {
        let c = self.control.get();
        if c.is_null() {
            return;
        }
        let a = |v: f32| (v.clamp(0.0, 1.0) * 65535.0) as u16;
        unsafe {
            (self.vibrar)(c, a(grave), a(agudo), ms);
        }
    }
}

/// `--probar-vibracion`: prueba cada camino de SDL (ver `Modo`), uno tras otro, con una
/// vibracion larga y fuerte en cada uno, y dice cual es cual: el que se sienta es el bueno.
/// `bombear` mueve los eventos de la ventana (el camino de Apple avisa de los controles por ahi).
pub fn diagnostico(bombear: &dyn Fn()) {
    let espera = |segundos: f32, v: &Vibracion| {
        for _ in 0..(segundos * 30.0) as i32 {
            bombear();
            unsafe { (v.actualizar)() };
            std::thread::sleep(std::time::Duration::from_millis(33));
        }
    };
    for (k, modo) in Modo::TODOS.into_iter().enumerate() {
        println!();
        println!("PRUEBA {} de {}: camino {}", k + 1, Modo::TODOS.len(), modo.nombre());
        let Some(v) = Vibracion::abrir_con(modo) else {
            println!("  SDL2 no se pudo abrir (o no arranco) con este camino.");
            continue;
        };
        let simbolo = |n: &str| unsafe {
            let libreria = LUGARES.iter().find_map(|l| {
                let c = CString::new(*l).ok()?;
                let h = sistema::abrir(c.as_ptr());
                (!h.is_null()).then_some(h)
            })?;
            let c = CString::new(n).ok()?;
            let p = sistema::simbolo(libreria, c.as_ptr());
            (!p.is_null()).then_some(p)
        };
        unsafe {
            let error: Option<unsafe extern "C" fn() -> *const c_char> = simbolo("SDL_GetError").map(|p| std::mem::transmute(p));
            let nombre: Option<unsafe extern "C" fn(c_int) -> *const c_char> = simbolo("SDL_JoystickNameForIndex").map(|p| std::mem::transmute(p));
            let ruta: Option<unsafe extern "C" fn(c_int) -> *const c_char> = simbolo("SDL_JoystickPathForIndex").map(|p| std::mem::transmute(p));
            let tiene: Option<unsafe extern "C" fn(*mut c_void) -> c_int> = simbolo("SDL_GameControllerHasRumble").map(|p| std::mem::transmute(p));
            let cerrar: Option<unsafe extern "C" fn(*mut c_void)> = simbolo("SDL_GameControllerClose").map(|p| std::mem::transmute(p));
            let salir: Option<unsafe extern "C" fn()> = simbolo("SDL_Quit").map(|p| std::mem::transmute(p));
            let texto = |p: *const c_char| if p.is_null() { String::new() } else { std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() };
            let ultimo_error = || error.map_or(String::new(), |f| texto(f()));
            espera(1.5, &v);
            let n = (v.num)();
            println!("  SDL ve {n} joystick(s). {}", ultimo_error());
            for i in 0..n {
                println!("    #{i}: {:?} ({}), es control: {}", nombre.map_or(String::new(), |f| texto(f(i))), ruta.map_or(String::new(), |f| texto(f(i))), (v.es_control)(i) != 0);
            }
            v.actualizar(1.0);
            let c = v.control.get();
            if c.is_null() {
                println!("  No se pudo abrir ningun control. {}", ultimo_error());
            } else {
                println!("  Control abierto. Tiene vibracion: {:?}", tiene.map(|f| f(c) != 0));
                println!("  >>> VIBRANDO {} (dos segundos)...", k + 1);
                let r = (v.vibrar)(c, 65535, 65535, 2000);
                println!("      SDL dice: {}", if r == 0 { "ok".to_string() } else { format!("fallo: {}", ultimo_error()) });
                espera(2.2, &v);
                if let Some(f) = cerrar {
                    f(c);
                }
            }
            if let Some(f) = salir {
                f();
            }
        }
        // Una pausa entre prueba y prueba, para que se note cual vibro.
        for _ in 0..60 {
            bombear();
            std::thread::sleep(std::time::Duration::from_millis(33));
        }
    }
    println!();
    println!("¿En cual prueba vibro el control? Ese camino es el bueno.");
}

/// Que tanto vibra cada sonido del juego (los dos motores y la duracion), si vibra.
pub fn de_efecto(e: crate::sonido::Efecto) -> Option<(f32, f32, u32)> {
    use crate::sonido::Efecto::*;
    Some(match e {
        Choque => (1.0, 0.8, 320),
        Aterrizaje => (0.7, 0.4, 550),
        Salto => (0.6, 0.9, 400),
        Fuego => (0.8, 0.5, 350),
        Protesto => (1.0, 1.0, 450),
        Mazo => (0.9, 0.3, 300),
        Perfecto => (0.4, 0.9, 200),
        Sello => (0.6, 0.9, 260),
        Reglazo => (0.85, 0.65, 160),
        Trueno => (0.7, 0.2, 700),
        Latido => (0.5, 0.0, 120),
        Cometa => (0.5, 0.8, 700),
        Fallo => (0.7, 0.2, 200),
        Nota0 | Nota1 | Nota2 | Nota3 | Nota4 | Nota5 | Nota6 | Nota7 => (0.3, 0.6, 90),
        Don => (0.8, 0.1, 90),
        Ka => (0.1, 0.8, 80),
        Rebobinar => (0.4, 0.4, 1500),
        // El teseracto retumba largo y grave; el agujero de gusano jala.
        Teseracto => (0.6, 0.1, 1800),
        Gusano => (0.5, 0.7, 900),
        // Los menus: un toquecito al mover el cursor y uno mas firme al escoger.
        Carta => (0.0, 0.45, 45),
        Acierto => (0.45, 0.7, 120),
        // El celular vibrando en la mesa (el grupo de WhatsApp) y el teclear.
        Whatsapp => (0.25, 0.0, 160),
        Teclas => (0.0, 0.25, 60),
        Chorro => (0.05, 0.12, 40),
        Pop => (0.2, 0.5, 70),
        // El mazo se siente en las manos; pegarle al aire, apenas.
        Mazazo => (0.75, 0.55, 70),
        // El soplido del giro, apenitas: avisa en las manos que ya viene el golpe.
        Aire => (0.0, 0.14, 40),
        Tablero => (0.45, 0.2, 60),
        // Cada anillo se siente pasar, apenas; el roce, mas.
        Pase => (0.06, 0.1, 25),
        Roce => (0.2, 0.55, 80),
        Recursion => (1.0, 0.9, 500),
        Puerta => (0.3, 0.1, 250),
        Alarma => (0.5, 0.6, 300),
        _ => return None,
    })
}
