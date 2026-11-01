//! El menu principal del juego (lo primero que se ve): una pantalla de titulo,
//! distinta a la de pausa. Primero solo el logo y "Presiona ESPACIO"; luego
//! las opciones abajo a la izquierda, en una lista limpia con un rombito que
//! marca la escogida. Miku bailando en grande a la derecha y el sistema solar
//! girando atras.

use crate::entrada::Entrada;
use crate::sonido::Efecto;
use crate::ui;
use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Opcion {
    Continuar,
    Nueva,
    Historia,
    Pasaporte,
    Creditos,
    Salir,
}

pub enum AccionMenu {
    Nada,
    Escoger(Opcion),
}

const CREDITOS: [&str; 18] = [
    "SPACE ADVENTURE",
    "Proyecto 3 · Gráficas por Computadora (CC2018) · UVG",
    "",
    "Un rasterizador por software en Rust: cada pixel lo pinta el juego.",
    "raylib solo pone la ventana, el teclado, el control y el sonido.",
    "",
    "Hatsune Miku y Megurine Luka © Crypton Future Media (Piapro).",
    "Fan art, con modelos voxel propios: Among Us, UNO, DOOM, Halo, Final",
    "Fantasy VII, Animal Crossing, Astro Bot, Portal, Zelda, Sonic, Gollum,",
    "Elden Ring, Untitled Goose Game y Ace Attorney.",
    "",
    "Texto: Pixelify Sans (SIL Open Font License). Títulos: 04b_30, de Yuji Oshimoto.",
    "Interfaz: Complete UI Essential Pack de Crusenho (CC BY 4.0), Humble Gift de HumblePixel y Pixel UI pack 3, pasados a los colores del juego.",
    "Iconos de controles: Vryell (paleta Rosy 42 de PineTreePizza), en lila.",
    "Música y sonidos: sintetizados por el juego.",
    "",
    "GLADAS, la IA de la nave, quiere aclarar que no pidió salir en los créditos.",
    "Pero ya que está: gracias por jugar.",
];

pub struct Menu {
    cursor: usize,
    /// Segundos desde que se movio el cursor (la escogida se corre a la derecha).
    desde: f32,
    /// Segundos desde que aparecio la lista (entra una sola vez, no cada que se mueve el cursor).
    lista: f32,
    pub reloj: f32,
    /// Si hay partida guardada (para ofrecer "Continuar").
    hay_partida: bool,
    /// Los creditos abiertos.
    pub creditos: bool,
    /// Si ya se paso la pantalla de "Presiona ESPACIO".
    iniciado: bool,
    pub eventos: Vec<Efecto>,
}

impl Menu {
    pub fn new(hay_partida: bool) -> Self {
        Menu { cursor: 0, desde: 0.0, lista: 0.0, reloj: 0.0, hay_partida, creditos: false, iniciado: false, eventos: Vec::new() }
    }

    fn opciones(&self) -> Vec<Opcion> {
        let mut v = vec![Opcion::Nueva, Opcion::Historia, Opcion::Pasaporte, Opcion::Creditos, Opcion::Salir];
        if self.hay_partida {
            v.insert(0, Opcion::Continuar);
        }
        v
    }

    /// Si ya se paso el "Presiona ESPACIO" (ya se ve la lista).
    pub fn iniciado(&self) -> bool {
        self.iniciado
    }

    /// Se salta el "Presiona ESPACIO" (para las fotos de la interfaz).
    pub fn iniciar(&mut self) {
        self.iniciado = true;
        self.desde = 1.0;
        self.lista = 1.0;
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionMenu {
        self.reloj += dt;
        self.desde += dt;
        self.lista += dt;
        if self.creditos {
            if e.aceptar || e.aterrizar || e.pausa {
                self.creditos = false;
            }
            return AccionMenu::Nada;
        }
        if !self.iniciado {
            if e.aceptar && self.reloj > 0.3 {
                self.iniciado = true;
                self.desde = 0.0;
                self.lista = 0.0;
                self.eventos.push(Efecto::Perfecto);
            }
            return AccionMenu::Nada;
        }
        let n = self.opciones().len();
        if e.abajo {
            self.cursor = (self.cursor + 1) % n;
            self.desde = 0.0;
            self.eventos.push(Efecto::Carta);
        }
        if e.arriba {
            self.cursor = (self.cursor + n - 1) % n;
            self.desde = 0.0;
            self.eventos.push(Efecto::Carta);
        }
        if e.aceptar && self.reloj > 0.3 {
            let o = self.opciones()[self.cursor];
            self.eventos.push(Efecto::Acierto);
            if o == Opcion::Creditos {
                self.creditos = true;
                self.desde = 0.0;
                return AccionMenu::Nada;
            }
            return AccionMenu::Escoger(o);
        }
        AccionMenu::Nada
    }

    /// (Personajes, pase, tienda y ajustes no van en la lista: son las pestanas de arriba.)
    pub fn dibujar(&self, d: &mut RaylibDrawHandle, e: &Entrada, notas: u32, conteo: (usize, usize), ancho: i32, alto: i32) {
        if self.creditos {
            let (w, h) = (ancho as f32, alto as f32);
            d.draw_rectangle(0, 0, ancho, alto, Color::new(4, 6, 28, 225));
            // El titulo como en las otras pantallas del menu (debajo de la barra de pestanas).
            ui::paralelogramo(d, -40.0, h * 0.13, w * 0.42, h * 0.072, 26.0, Color::new(18, 40, 120, 235));
            ui::titulo_menu(d, "CRÉDITOS", w * 0.04, h * 0.134, (h * 0.055).clamp(26.0, 52.0), -3.0);
            // Las lineas, envueltas al ancho y del tamano mas grande en que caben todas.
            let (x0, y0) = ((w * 0.06) as i32, (h * 0.27) as i32);
            let ancho_texto = ancho - x0 * 2;
            let renglones = |tam: i32| -> Vec<(usize, String, i32)> {
                let mut v = Vec::new();
                for (k, linea) in CREDITOS.iter().enumerate() {
                    let t = if k == 0 { tam * 3 / 2 } else { tam };
                    if linea.is_empty() {
                        v.push((k, String::new(), t));
                    }
                    for r in ui::envolver(linea, t, ancho_texto) {
                        v.push((k, r, t));
                    }
                }
                v
            };
            let alto_de = |v: &[(usize, String, i32)]| v.iter().map(|(_, r, t)| if r.is_empty() { t * 2 / 3 } else { t + t / 2 }).sum::<i32>();
            let mut tam = 20;
            while tam > 12 && alto_de(&renglones(tam)) > alto - 80 - y0 {
                tam -= 1;
            }
            let mut y = y0;
            for (k, renglon, t) in renglones(tam) {
                if renglon.is_empty() {
                    y += t * 2 / 3;
                    continue;
                }
                let entra = ((self.desde - k as f32 * 0.06) / 0.25).clamp(0.0, 1.0);
                let a = (entra * 255.0) as u8;
                let color = if k == 0 { Color::new(150, 230, 240, a) } else { Color::new(225, 232, 255, a) };
                ui::texto(d, &renglon, x0 + ((1.0 - entra) * 40.0) as i32, y, t, color);
                y += t + t / 2;
            }
            ui::ayudas(d, &[(e.nombre("aceptar"), "Volver")], ancho, alto);
            return;
        }
        let (w, h) = (ancho as f32, alto as f32);
        // Un velo suave abajo a la izquierda (para leer) y el resto, el espacio.
        d.draw_rectangle_gradient_h(0, 0, (w * 0.6) as i32, alto, Color::new(4, 4, 22, 200), Color::new(4, 4, 22, 0));
        // El logo: SPACE en blanco y MIKU en cian, inclinados y con sombra, y una raya debajo.
        let flota = 4.0 * (self.reloj * 1.2).sin();
        let mut tam2 = (h * 0.098).clamp(32.0, 84.0);
        while tam2 > 26.0 && ui::medir_titulo("ADVENTURE", tam2 as i32) as f32 > w * 0.86 { tam2 -= 1.0; }
        let tam1 = tam2 * 0.76;
        // Debajo de la barra de pestanas (y de lo que sube la palabra inclinada).
        let (x0, y0) = (w * 0.06, h * 0.135 + flota);
        ui::titulo_girado(d, "SPACE", x0, y0, tam1, -6.0, Color::WHITE, Color::new(150, 90, 255, 255));
        let y2 = y0 + tam1 * 1.12;
        ui::titulo_girado(d, "ADVENTURE", x0, y2, tam2, -6.0, Color::new(110, 235, 225, 255), Color::new(40, 140, 200, 255));
        let ancho_logo = ui::medir_titulo("ADVENTURE", tam2 as i32) as f32;
        let y_raya = y2 + tam2 * 1.08;
        d.draw_rectangle_pro(Rectangle::new(x0 - 6.0, y_raya, ancho_logo + 40.0, 5.0), Vector2::zero(), -6.0, Color::new(255, 130, 210, 230));
        let sub = "Divorcios en Venus. Citas en Marte. ¿Y la entrega?";
        let mut tam_sub = 18;
        while tam_sub > 11 && ui::medir(sub, tam_sub) as f32 > w * 0.88 { tam_sub -= 1; }
        ui::texto_girado(d, sub, x0 + 4.0, y_raya + 6.0, tam_sub as f32, -6.0, Color::new(220, 210, 255, 255));
        let pie = "Proyecto 3 · Gráficas por Computadora · UVG";
        ui::texto(d, pie, ancho - ui::medir(pie, 15) - 20, if self.iniciado { alto - 73 } else { alto - 28 }, 13, Color::new(190, 180, 220, 200));
        if !self.iniciado {
            if (self.reloj * 1.6) as i32 % 2 == 0 {
                ui::accion(d, e.nombre("aceptar"), "Comenzar", (w * 0.08) as i32, (h * 0.72) as i32, 24, true);
            }
            return;
        }
        // Las opciones: una lista limpia abajo a la izquierda. La escogida se corre a la derecha,
        // en cian y con un rombito; las demas, blancas y medio transparentes.
        let opciones = self.opciones();
        // Que quepan todas entre el logo y la ayuda de abajo.
        let y_lista = (h * 0.47) as i32;
        let cabe = (alto - 100 - y_lista) as f32 / opciones.len() as f32;
        let tam = (h * 0.040).min(cabe / 1.45).clamp(15.0, 30.0) as i32;
        let paso = (tam + tam / 2).min(cabe as i32);
        for (k, o) in opciones.iter().enumerate() {
            let (nombre, valor) = match o {
                Opcion::Continuar => ("Continuar", format!("{notas} notas")),
                Opcion::Nueva => ("Nueva aventura", if self.hay_partida { "con la intro".into() } else { String::new() }),
                Opcion::Historia => ("La historia del universo", String::new()),
                Opcion::Pasaporte => ("Pasaporte", format!("{}/{}", conteo.0, conteo.1)),
                Opcion::Creditos => ("Créditos", String::new()),
                Opcion::Salir => ("Salir", String::new()),
            };
            let y = y_lista + k as i32 * paso;
            let escogida = k == self.cursor;
            let entra = ((self.lista - k as f32 * 0.04) / 0.2).clamp(0.0, 1.0);
            let x = (w * 0.07) as i32 + if escogida { (18.0 * (self.desde / 0.12).min(1.0)) as i32 } else { 0 } - ((1.0 - entra) * 30.0) as i32;
            if escogida {
                let (cx, cy) = (x as f32 - 16.0, y as f32 + tam as f32 * 0.55);
                d.draw_poly(Vector2::new(cx, cy), 4, 7.0, 45.0, Color::new(255, 130, 210, 255));
                d.draw_line_ex(Vector2::new(x as f32, (y + tam + 4) as f32), Vector2::new((x + ui::medir(nombre, tam)) as f32, (y + tam + 4) as f32), 2.0, Color::new(110, 235, 225, 200));
            }
            let a = (entra * 255.0) as u8;
            let color = if escogida { Color::new(110, 235, 225, a) } else { Color::new(235, 235, 255, (a as f32 * 0.7) as u8) };
            ui::texto(d, nombre, x + 2, y + 2, tam, Color::new(5, 3, 20, a));
            ui::texto(d, nombre, x, y, tam, color);
            if !valor.is_empty() {
                ui::texto(d, &valor, x + ui::medir(nombre, tam) + 18, y + tam / 4, tam * 2 / 3, Color::new(200, 160, 255, a));
            }
        }
        let pestanas = format!("{} / {}", e.nombre("pestana_izq"), e.nombre("pestana_der"));
        ui::ayudas(d, &[
            ("arriba / abajo", "Escoger"),
            (e.nombre("aceptar"), "Aceptar"),
            (&pestanas, "Pestañas"),
        ], ancho, alto);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn continuar_solo_si_hay_partida() {
        assert_eq!(Menu::new(false).opciones()[0], Opcion::Nueva);
        assert_eq!(Menu::new(true).opciones()[0], Opcion::Continuar);
        let mut m = Menu::new(true);
        m.actualizar(&Entrada::default(), 0.5);
        // Primero "Presiona ESPACIO".
        m.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.1);
        m.actualizar(&Entrada { abajo: true, ..Default::default() }, 0.1);
        assert!(matches!(m.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.1), AccionMenu::Escoger(Opcion::Nueva)));
    }

    #[test]
    fn los_creditos_se_abren_y_se_cierran() {
        let mut m = Menu::new(false);
        m.actualizar(&Entrada::default(), 0.5);
        m.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.1);
        for _ in 0..3 {
            m.actualizar(&Entrada { abajo: true, ..Default::default() }, 0.1);
        }
        m.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.1);
        assert!(m.creditos);
        m.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.1);
        assert!(!m.creditos);
    }
}
