//! Teclado y control juntos: cada cuadro se lee todo una vez y el resto del
//! juego solo pregunta "¿se quiere girar?", sin saber de donde vino.
//!
//! Control (Xbox / PlayStation / genericos, el primero que este conectado):
//!
//! | | teclado | control |
//! |---|---|---|
//! | girar y cabecear | A D, flechas | stick izquierdo |
//! | acelerar / frenar | W / S | RT / LT |
//! | turbo | shift | RB |
//! | barrel roll, gritar ¡NEPTUNO! y ¡PROTESTO! | Q | Y (triangulo) |
//! | elegir carta | flechas | cruceta |
//! | hiperespacio, aceptar | espacio, enter | A (cruz) |
//! | aterrizar / despegar | E | X (cuadrado) |
//! | elegir respuesta | 1 a 4, flechas | cruceta |
//! | modo de dibujo y vistas | 1 a 7 | select |
//! | ocultar HUD | H | (en la pausa) |
//! | pausa y ajustes | ESC | start |
//! | escoger destino del hiperespacio | T | LB |
//! | info del pipeline y controles | TAB | apretar el stick izquierdo |
//! | pasaporte (sellos, finales y logros) | P | apretar el stick derecho |
//! | pantalla completa | F11 o Alt+Enter | |
//! | ritmo: piso / aire | F / J | A o LB / B o RB |

use raylib::prelude::*;

const ZONA_MUERTA: f32 = 0.15;
const CONTROL: i32 = 0;

#[derive(Default)]
pub struct Entrada {
    /// De -1 (derecha) a 1 (izquierda).
    pub girar: f32,
    /// De -1 (nariz abajo) a 1 (nariz arriba).
    pub cabecear: f32,
    /// De -1 (frenar) a 1 (acelerar).
    pub empuje: f32,
    /// Izquierda/derecha MANTENIDO (-1 izquierda, 1 derecha): flechas, A/D, stick y cruceta.
    pub horizontal: f32,
    /// Donde esta el raton en la ventana (si esta suelto) y si se hizo clic este cuadro.
    pub puntero: Option<(f32, f32)>,
    pub clic: bool,
    /// Si el boton izquierdo del raton se esta manteniendo (para echar agua en rafaga en Urano).
    pub raton_apretado: bool,
    /// Lo que se movio el raton atrapado este cuadro, ya en radianes (girar, cabecear): se suma
    /// directo al angulo (no es velocidad, asi no depende de los cuadros por segundo).
    pub raton: (f32, f32),
    pub turbo: bool,
    pub barrel: bool,
    pub aceptar: bool,
    pub aterrizar: bool,
    pub arriba: bool,
    pub abajo: bool,
    pub izquierda: bool,
    pub derecha: bool,
    /// Respuesta elegida con el teclado (1 a 4), contando desde 0.
    pub respuesta: Option<usize>,
    /// Modo de dibujo o vista pedido con el teclado (1 a 7), contando desde 0.
    pub modo: Option<usize>,
    pub siguiente_modo: bool,
    pub culling: bool,
    pub hud: bool,
    /// TAB: la info del pipeline y los controles.
    pub info: bool,
    /// F11 o Alt+Enter: pantalla completa.
    pub pantalla_completa: bool,
    /// P: el pasaporte (sellos, finales y logros).
    pub pasaporte: bool,
    /// ESC o start: el menu de pausa.
    pub pausa: bool,
    /// T o LB: escoger a que planeta va el hiperespacio.
    pub objetivo: bool,
    /// En Saturno Ritmo: golpe al piso y brinco al aire.
    pub piso: bool,
    pub aire: bool,
    /// Q / E o L1 / R1: la pestana anterior o la siguiente del menu del juego.
    /// Si se mantiene apretado el boton de aceptar (para disparar en rafaga en Jupiter).
    pub disparando: bool,
    /// El gatillo derecho (RT) apretado: en la plaga de Urano echa agua, como en un shooter.
    pub gatillo: bool,
    pub pestana_izq: bool,
    pub pestana_der: bool,
    /// Picar (mantener): F o el clic en el teclado, el gatillo derecho en el control (la mineria
    /// de Planeta Vegetta).
    pub picar: bool,
    /// R (o B en el control): prende o apaga el modo orbita junto a un planeta.
    pub orbitar: bool,
    /// V: prende o apaga el descarte de objetos fuera de la vista (para comparar).
    pub descarte: bool,
    /// Los parametros en vivo, - y +: F5 y F6 la semilla, F7 y F8 el relieve, F9 y F10 los shaders,
    /// F1 y F2 el radio del planeta, F3 y F4 la velocidad de las orbitas.
    pub ajuste: Option<usize>,
    pub hay_control: bool,
    /// Que control es (para nombrar sus botones como vienen pintados en el).
    pub control: Control,
}

/// La marca del control conectado: cambia como se llaman los botones en pantalla.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Control {
    #[default]
    Teclado,
    PlayStation,
    Xbox,
    Switch,
    /// Cualquier otro: se nombra como el de Xbox (es el que imita SDL).
    Otro,
}

impl Control {
    /// Por el nombre que da el sistema ("DualSense Wireless Controller", "Xbox Wireless Controller"...).
    pub fn por_nombre(nombre: &str) -> Self {
        let n = nombre.to_lowercase();
        if ["dualsense", "dualshock", "playstation", "ps5", "ps4", "ps3", "sony"].iter().any(|x| n.contains(x)) || n == "wireless controller" {
            Control::PlayStation
        } else if ["xbox", "x-box", "xinput"].iter().any(|x| n.contains(x)) {
            Control::Xbox
        } else if ["nintendo", "switch", "joy-con", "joycon"].iter().any(|x| n.contains(x)) || n == "pro controller" {
            Control::Switch
        } else {
            Control::Otro
        }
    }
}

fn zona_muerta(v: f32) -> f32 {
    if v.abs() < ZONA_MUERTA { 0.0 } else { (v - ZONA_MUERTA * v.signum()) / (1.0 - ZONA_MUERTA) }
}

/// La curva de los sticks: poquito movimiento da poquito giro (para apuntar fino) y a fondo da
/// todo. Es lo que hacen casi todos los juegos de naves y de disparos.
fn curva(v: f32) -> f32 {
    v.signum() * v.abs().powf(1.6)
}

impl Entrada {
    pub fn leer(rl: &RaylibHandle) -> Self {
        use GamepadAxis::*;
        use GamepadButton::*;
        use KeyboardKey::*;
        let tecla = |k| rl.is_key_down(k);
        let pulsada = |k| rl.is_key_pressed(k);
        let eje = |pos, neg| (tecla(pos) as i32 - tecla(neg) as i32) as f32;
        let numero = |teclas: &[KeyboardKey]| teclas.iter().position(|&k| pulsada(k));

        let mut e = Entrada {
            girar: eje(KEY_A, KEY_D),
            cabecear: eje(KEY_UP, KEY_DOWN),
            empuje: eje(KEY_W, KEY_S),
            raton: (0.0, 0.0),
            puntero: None,
            clic: false,
            raton_apretado: false,
            horizontal: (eje(KEY_RIGHT, KEY_LEFT) + eje(KEY_D, KEY_A)).clamp(-1.0, 1.0),
            turbo: tecla(KEY_LEFT_SHIFT),
            barrel: pulsada(KEY_Q),
            // Alt+Enter es pantalla completa, no aceptar.
            aceptar: pulsada(KEY_SPACE) || (pulsada(KEY_ENTER) && !tecla(KEY_LEFT_ALT) && !tecla(KEY_RIGHT_ALT)),
            aterrizar: pulsada(KEY_E),
            arriba: pulsada(KEY_UP),
            abajo: pulsada(KEY_DOWN),
            izquierda: pulsada(KEY_LEFT),
            derecha: pulsada(KEY_RIGHT),
            respuesta: numero(&[KEY_ONE, KEY_TWO, KEY_THREE, KEY_FOUR]),
            modo: numero(&[KEY_ONE, KEY_TWO, KEY_THREE, KEY_FOUR, KEY_FIVE, KEY_SIX, KEY_SEVEN]),
            siguiente_modo: false,
            culling: pulsada(KEY_C),
            hud: pulsada(KEY_H),
            info: pulsada(KEY_TAB),
            pasaporte: pulsada(KEY_P),
            pausa: pulsada(KEY_ESCAPE),
            objetivo: pulsada(KEY_T),
            pantalla_completa: pulsada(KEY_F11) || ((tecla(KEY_LEFT_ALT) || tecla(KEY_RIGHT_ALT)) && pulsada(KEY_ENTER)),
            piso: pulsada(KEY_F) || pulsada(KEY_DOWN) || pulsada(KEY_SPACE),
            aire: pulsada(KEY_J) || pulsada(KEY_UP),
            disparando: tecla(KEY_SPACE) || tecla(KEY_ENTER),
            gatillo: false,
            orbitar: pulsada(KEY_R),
            picar: tecla(KEY_F),
            descarte: pulsada(KEY_V),
            ajuste: numero(&[KEY_F5, KEY_F6, KEY_F7, KEY_F8, KEY_F9, KEY_F10, KEY_F1, KEY_F2, KEY_F3, KEY_F4]),
            pestana_izq: pulsada(KEY_Q),
            pestana_der: pulsada(KEY_E),
            hay_control: rl.is_gamepad_available(CONTROL),
            control: Control::Teclado,
        };
        if e.hay_control {
            e.control = rl.get_gamepad_name(CONTROL).map_or(Control::Otro, |n| Control::por_nombre(&n));
        }

        if e.hay_control {
            let eje_c = |a| zona_muerta(rl.get_gamepad_axis_movement(CONTROL, a));
            // Los gatillos van de -1 (suelto) a 1 (a fondo).
            let gatillo = |a| ((rl.get_gamepad_axis_movement(CONTROL, a) + 1.0) * 0.5).clamp(0.0, 1.0);
            let boton = |b| rl.is_gamepad_button_pressed(CONTROL, b);
            let girar = curva(-eje_c(GAMEPAD_AXIS_LEFT_X));
            let cabecear = curva(-eje_c(GAMEPAD_AXIS_LEFT_Y));
            let empuje = zona_muerta(gatillo(GAMEPAD_AXIS_RIGHT_TRIGGER) - gatillo(GAMEPAD_AXIS_LEFT_TRIGGER));
            // Gana el que se este moviendo mas, teclado o control.
            let mayor = |a: f32, b: f32| if b.abs() > a.abs() { b } else { a };
            e.girar = mayor(e.girar, girar);
            e.cabecear = mayor(e.cabecear, cabecear);
            e.empuje = mayor(e.empuje, empuje);
            // La cruceta mantenida tambien gira (y cubre, en el box).
            let cruceta = |b| rl.is_gamepad_button_down(CONTROL, b);
            let cruz_x = cruceta(GAMEPAD_BUTTON_LEFT_FACE_RIGHT) as i32 as f32 - cruceta(GAMEPAD_BUTTON_LEFT_FACE_LEFT) as i32 as f32;
            let cruz_y = cruceta(GAMEPAD_BUTTON_LEFT_FACE_UP) as i32 as f32 - cruceta(GAMEPAD_BUTTON_LEFT_FACE_DOWN) as i32 as f32;
            e.girar = mayor(e.girar, -cruz_x);
            e.cabecear = mayor(e.cabecear, cruz_y);
            e.horizontal = mayor(mayor(e.horizontal, eje_c(GAMEPAD_AXIS_LEFT_X)), cruz_x);
            e.turbo |= rl.is_gamepad_button_down(CONTROL, GAMEPAD_BUTTON_RIGHT_TRIGGER_1);
            e.aceptar |= boton(GAMEPAD_BUTTON_RIGHT_FACE_DOWN);
            e.barrel |= boton(GAMEPAD_BUTTON_RIGHT_FACE_UP);
            e.aterrizar |= boton(GAMEPAD_BUTTON_RIGHT_FACE_LEFT);
            e.arriba |= boton(GAMEPAD_BUTTON_LEFT_FACE_UP);
            e.abajo |= boton(GAMEPAD_BUTTON_LEFT_FACE_DOWN);
            e.izquierda |= boton(GAMEPAD_BUTTON_LEFT_FACE_LEFT);
            e.derecha |= boton(GAMEPAD_BUTTON_LEFT_FACE_RIGHT);
            // Create / View: la info y los controles (lo que en el teclado es TAB); L3: la vista.
            e.siguiente_modo = boton(GAMEPAD_BUTTON_LEFT_THUMB);
            e.piso |= boton(GAMEPAD_BUTTON_RIGHT_FACE_DOWN) || boton(GAMEPAD_BUTTON_LEFT_TRIGGER_1);
            e.aire |= boton(GAMEPAD_BUTTON_RIGHT_FACE_RIGHT) || boton(GAMEPAD_BUTTON_RIGHT_TRIGGER_1);
            e.orbitar |= boton(GAMEPAD_BUTTON_RIGHT_FACE_RIGHT);
            e.pausa |= boton(GAMEPAD_BUTTON_MIDDLE_RIGHT);
            e.objetivo |= boton(GAMEPAD_BUTTON_LEFT_TRIGGER_1);
            e.info |= boton(GAMEPAD_BUTTON_MIDDLE_LEFT);
            e.pasaporte |= boton(GAMEPAD_BUTTON_RIGHT_THUMB);
            e.pestana_izq |= boton(GAMEPAD_BUTTON_LEFT_TRIGGER_1);
            e.disparando |= rl.is_gamepad_button_down(CONTROL, GAMEPAD_BUTTON_RIGHT_FACE_DOWN);
            e.gatillo = gatillo(GAMEPAD_AXIS_RIGHT_TRIGGER) > 0.4;
            e.picar |= e.gatillo;
            e.pestana_der |= boton(GAMEPAD_BUTTON_RIGHT_TRIGGER_1);
        }
        e
    }

    /// Como se llama el boton de cada accion, segun haya control o no.
    pub fn nombre(&self, accion: &str) -> &'static str {
        use Control::*;
        match (accion, self.control) {
            ("apuntar", Teclado) => "ratón",
            ("apuntar", _) => "stick",
            ("gatillo", Teclado) => "clic",
            ("gatillo", PlayStation) => "R2",
            ("gatillo", Switch) => "ZR",
            ("gatillo", Xbox | Otro) => "RT",
            ("aceptar" | "piso", PlayStation) => crate::ui::CRUZ,
            ("aterrizar", PlayStation) => crate::ui::CUADRADO,
            ("aire", PlayStation) => crate::ui::CIRCULO,
            ("barrel", PlayStation) => crate::ui::TRIANGULO,
            ("objetivo" | "pestana_izq", PlayStation) => "L1",
            ("pestana_der", PlayStation) => "R1",
            ("turbo", PlayStation) => "R1",
            ("acelerar", PlayStation) => "R2 / L2",
            ("vista", PlayStation | Xbox | Switch | Otro) => "L3",
            ("info", PlayStation) => "Create",
            ("info", Xbox | Otro) => "View",
            ("info", Teclado) => "TAB",
            ("pausa", PlayStation) => "Options",
            ("aceptar" | "piso", Xbox | Otro) => crate::ui::XBOX_A,
            ("aterrizar", Xbox | Otro) => crate::ui::XBOX_X,
            ("aire", Xbox | Otro) => crate::ui::XBOX_B,
            ("barrel", Xbox | Otro) => crate::ui::XBOX_Y,
            ("objetivo" | "pestana_izq", Xbox | Otro) => "LB",
            ("pestana_der", Xbox | Otro) => "RB",
            ("turbo", Xbox | Otro) => "RB",
            ("acelerar", Xbox | Otro) => "RT / LT",
            ("pausa", Xbox | Otro) => "Menu",
            // Raylib nombra los botones por su posicion: abajo es B en Nintendo.
            ("aceptar" | "piso", Switch) => crate::ui::SWITCH_B,
            ("aire", Switch) => crate::ui::SWITCH_A,
            ("aterrizar", Switch) => crate::ui::SWITCH_Y,
            ("barrel", Switch) => crate::ui::SWITCH_X,
            ("objetivo" | "pestana_izq", Switch) => "L",
            ("turbo" | "pestana_der", Switch) => "R",
            ("acelerar", Switch) => "ZR / ZL",
            ("info", Switch) => "-",
            ("pausa", Switch) => "+",
            ("pasaporte", PlayStation | Xbox | Switch | Otro) => "R3",
            ("aceptar", Teclado) => "ESPACIO",
            ("aterrizar", Teclado) => "E",
            ("piso", Teclado) => "F",
            ("aire", Teclado) => "J",
            ("barrel", Teclado) => "Q",
            ("pasaporte", Teclado) => "P",
            ("orbitar", Teclado) => "R",
            ("picar", Teclado) => "F",
            ("picar", PlayStation) => "R2",
            ("picar", Switch) => "ZR",
            ("picar", Xbox | Otro) => "RT",
            ("orbitar", PlayStation) => crate::ui::CIRCULO,
            ("orbitar", Switch) => crate::ui::SWITCH_A,
            ("orbitar", Xbox | Otro) => crate::ui::XBOX_B,
            ("objetivo", Teclado) => "T",
            ("pestana_izq", Teclado) => "Q",
            ("pestana_der", Teclado) => "E",
            ("turbo", Teclado) => "shift",
            ("acelerar", Teclado) => "W / S",
            ("vista", Teclado) => "1 a 7",
            ("pausa", Teclado) => "ESC",
            _ => "?",
        }
    }

}

#[cfg(test)]
mod pruebas_controles {
    use super::*;

    #[test]
    fn reconoce_los_mandos_y_respeta_la_posicion_de_sus_botones() {
        for (nombre, tipo) in [
            ("DualSense Wireless Controller", Control::PlayStation),
            ("Wireless Controller", Control::PlayStation),
            ("Xbox Wireless Controller", Control::Xbox),
            ("Nintendo Switch Pro Controller", Control::Switch),
            ("Pro Controller", Control::Switch),
            ("Joy-Con (L/R)", Control::Switch),
            ("USB Gamepad", Control::Otro),
        ] {
            assert_eq!(Control::por_nombre(nombre), tipo);
        }
        let nintendo = Entrada { control: Control::Switch, hay_control: true, ..Default::default() };
        assert_eq!(nintendo.nombre("aceptar"), crate::ui::SWITCH_B);
        assert_eq!(nintendo.nombre("aire"), crate::ui::SWITCH_A);
        assert_eq!(nintendo.nombre("barrel"), crate::ui::SWITCH_X);
        assert_eq!(nintendo.nombre("aterrizar"), crate::ui::SWITCH_Y);
        assert_eq!(nintendo.nombre("pausa"), "+");
    }
}
