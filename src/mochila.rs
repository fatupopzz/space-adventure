//! La MOCHILA (una pestana del menu del juego): lo que se trae de los planetas.
//!
//! Arriba, las lunas de Galileo que se recogieron en Jupiter Asteroids: se usan
//! volando (en la pausa, o desde aqui) y cada una hace algo:
//! - Io: turbo infinito un rato.
//! - Europa: escudo de hielo (los asteroides rojos no tumban notas).
//! - Ganimedes: iman de notas (las de cerca vienen solas).
//! - Calisto: llama al cometa de una vez.
//!
//! Abajo, los recuerdos: uno por cada aventura terminada (la carta del tarot,
//! la regla del profe Koch, la claqueta de la Luna...), cada uno con su modelo 3D
//! en voxels (se ven en foto en la cuadricula y girando en la vitrina).

use crate::entrada::Entrada;
use crate::jupiter::{COLOR_LUNA, LUNAS};
use crate::progreso::Progreso;
use crate::sonido::Efecto;
use crate::tienda::Muestra;
use crate::ui;
use raylib::prelude::*;

/// Lo que hace cada luna en el espacio y cuanto dura (segundos; Calisto es al instante).
pub const USO_LUNA: [&str; 4] = [
    "Turbo infinito por 30 segundos. Io tiene más de 400 volcanes: energía le sobra.",
    "Escudo de hielo por 60 segundos: los asteroides rojos no le tumban notas.",
    "Imán de notas por 45 segundos: Ganímedes es la luna más grande y jala todo.",
    "Llama al cometa ahorita. Calisto es la más vieja: sabe dónde andan todos.",
];
pub const DURA_USO: [f32; 4] = [30.0, 60.0, 45.0, 0.0];

/// Un recuerdo: nombre, el logro que lo da, de donde es y que dice GLADAS. Su modelo 3D es
/// `modelos/recuerdo_NN.obj` (en el mismo orden).
struct Recuerdo {
    nombre: &'static str,
    logros: &'static [&'static str],
    lugar: &'static str,
    texto: &'static str,
}

/// Cuantos recuerdos hay (cada uno con su modelo 3D).
pub const CUANTOS: usize = RECUERDOS.len();

const RECUERDOS: [Recuerdo; 15] = [
    Recuerdo { nombre: "El Chucho (carta de tarot)", logros: &["tarot_lectura"], lugar: "Mercurio", texto: "Madame Mercurio se la regaló. O se le cayó. Con el retrógrado, quién sabe. Dice que el chucho trae lealtad. Y pulgas." },
    Recuerdo { nombre: "Anillo de Saturno", logros: &["divorcio_3", "divorcio_0", "divorcio_ganso"], lugar: "Venus", texto: "Un pedacito de anillo del juicio. Es hielo de verdad: no lo deje al sol." },
    Recuerdo { nombre: "La regla del profe Koch", logros: &["examen_5", "examen_aprobada", "examen_reglazo", "koch_examen", "koch_5", "koch_reglazo"], lugar: "Tierra · UVG", texto: "Treinta centímetros de autoridad académica. Koch la dejó olvidada (en el CIT o en el salón de Teoría, depende de a quién le pregunte). Tiene un autómata grabado y marcas de reglazos." },
    Recuerdo { nombre: "Claqueta del set lunar", logros: &["luna_oscar", "luna_razzie", "luna_estrellas", "luna_cables"], lugar: "Luna", texto: "«Toma 1: un pequeño paso». El director dice que es la toma 47." },
    Recuerdo { nombre: "Foto de la cita en Marte", logros: &["cita_boda", "cita_exito", "cita_amigos", "cita_solo", "cita_drama"], lugar: "Marte", texto: "Curiosity sale borroso. Se movió. A 0.14 km/h, pero se movió." },
    Recuerdo { nombre: "La Gran Mancha Roja (en frasco)", logros: &["jupiter_mancha"], lugar: "Júpiter", texto: "Una tormenta más grande que la Tierra, en un frasco de mermelada. No lo abra." },
    Recuerdo { nombre: "Vinil de Saturno Ritmo", logros: &["ritmo_termino"], lugar: "Saturno", texto: "Los anillos de Saturno tocan música. Este es el single." },
    Recuerdo { nombre: "Carta +4 de Neptuno", logros: &["uno_gana", "uno_pierde"], lugar: "Neptuno", texto: "Firmada por el Doom Slayer. No dijo nada al firmarla. Solo asintió." },
    Recuerdo { nombre: "Credencial de planeta", logros: &["junta_si", "junta_no"], lugar: "Plutón", texto: "Dice «PLANETA» en grande y «enano» en letra chiquita." },
    Recuerdo { nombre: "Contenedor de Corazón", logros: &["hada_luka"], lugar: "Planeta Hada", texto: "Le da un corazón más. A la nave no le sirve, pero se ve bonito." },
    Recuerdo { nombre: "Hielo de cometa", logros: &["cometa"], lugar: "El espacio", texto: "Hielo de 4,600 millones de años. Perfecto para una limonada histórica." },
    Recuerdo { nombre: "Autógrafo de Bill Nye", logros: &["ciencia_5"], lugar: "Cinturón de asteroides", texto: "«¡La ciencia manda!» - Bill. Con corbatín dibujado." },
    Recuerdo { nombre: "Cinturón de campeón", logros: &["urano_campeon"], lugar: "Urano", texto: "El gafete de empleado del mes de Doña Chepa, hecho con un cinturón viejo. Sobrevivió al domingo de feria y a la excursión de Plutón." },
    Recuerdo { nombre: "Diamante de Planeta Vegetta", logros: &["vegetta_carrera", "vegetta_juega"], lugar: "Planeta Vegetta", texto: "Un diamante de bloque. Vegetta dice que es el número 777." },
    Recuerdo { nombre: "Patito de hule de Doña Chepa", logros: &["urano_juega"], lugar: "Urano", texto: "Vivía en el lavamanos del baño de Urano. Se lo llevó sin querer en el bolsillo del delantal. Hace cuac." },
];

pub struct Mochila {
    /// 0..4: las lunas; 4..: los recuerdos.
    cursor: usize,
    desde: f32,
    abierta: f32,
    pub reloj: f32,
    /// Si se esta volando (las lunas solo se usan en el espacio).
    volando: bool,
    /// La luna que se acaba de usar (la recoge el juego y aplica el efecto).
    pub usar: Option<usize>,
    aviso: Option<String>,
    pub eventos: Vec<Efecto>,
}

impl Mochila {
    pub fn new(volando: bool) -> Self {
        Mochila { cursor: 0, desde: 1.0, abierta: 0.0, reloj: 0.0, volando, usar: None, aviso: None, eventos: Vec::new() }
    }

    fn total() -> usize {
        4 + RECUERDOS.len()
    }

    /// Si se tiene el recuerdo `k` (0..4 las lunas: esas siempre se ven, aunque no queden).
    fn tiene(p: &Progreso, k: usize) -> bool {
        k < 4 || RECUERDOS[k - 4].logros.iter().any(|l| p.tiene(l))
    }

    /// Lo que gira en la vitrina: lo escogido.
    pub fn muestra(&self, p: &Progreso) -> Muestra {
        Muestra::Recuerdo { cual: self.cursor, tiene: Self::tiene(p, self.cursor) }
    }

    /// Devuelve si hay que cerrar.
    pub fn actualizar(&mut self, e: &Entrada, p: &mut Progreso, dt: f32) -> bool {
        self.desde += dt;
        self.abierta += dt;
        self.reloj += dt;
        if e.pausa {
            self.eventos.push(Efecto::Carta);
            return true;
        }
        let n = Self::total();
        let antes = self.cursor;
        // Las lunas van en una fila de 4; los recuerdos, en filas de 5.
        if e.derecha {
            self.cursor = (self.cursor + 1) % n;
        }
        if e.izquierda {
            self.cursor = (self.cursor + n - 1) % n;
        }
        if e.abajo {
            self.cursor = if self.cursor < 4 { 4 + self.cursor.min(4) } else { (self.cursor + 5).min(n - 1) };
        }
        if e.arriba {
            self.cursor = if self.cursor < 4 { self.cursor } else if self.cursor < 9 { (self.cursor - 4).min(3) } else { self.cursor - 5 };
        }
        if self.cursor != antes {
            self.desde = 0.0;
            self.aviso = None;
            self.eventos.push(Efecto::Carta);
        }
        if e.aceptar && self.cursor < 4 {
            let k = self.cursor;
            if p.mochila[k] == 0 {
                self.eventos.push(Efecto::Fallo);
                self.aviso = Some("No trae ninguna. Las lunas se consiguen jugando asteroides en Júpiter.".into());
            } else if !self.volando {
                self.eventos.push(Efecto::Fallo);
                self.aviso = Some("Las lunas se usan volando. Salga al espacio y ábrala desde la pausa.".into());
            } else {
                p.mochila[k] -= 1;
                p.guardar();
                self.usar = Some(k);
                self.eventos.push(Efecto::Sello);
                return true;
            }
        }
        false
    }

    /// `vitrina`: lo escogido girando en 3D; `fotos`: las fotos de las lunas y los recuerdos.
    #[allow(clippy::too_many_arguments)]
    pub fn dibujar(&self, d: &mut RaylibDrawHandle, e: &Entrada, p: &Progreso, vitrina: &Texture2D, fotos: &[Texture2D], ancho: i32, alto: i32) {
        let (w, h) = (ancho as f32, alto as f32);
        d.draw_rectangle(0, 0, ancho, alto, Color::new(6, 4, 22, 235));
        ui::paralelogramo(d, -40.0, h * 0.085, w * 0.42, h * 0.072, 26.0, Color::new(18, 40, 120, 235));
        let tam_titulo = (h * 0.055).clamp(26.0, 52.0);
        ui::titulo_menu(d, "MOCHILA", w * 0.04, h * 0.098, tam_titulo, -3.0);
        let x0 = w * 0.05;
        let ancho_izq = (w * 0.56).min(820.0);
        let foto = |d: &mut RaylibDrawHandle, k: usize, cx: f32, cy: f32, lado: f32, tinte: Color| {
            if let Some(t) = fotos.get(k) {
                let fuente = Rectangle::new(0.0, 0.0, t.width as f32, t.height as f32);
                d.draw_texture_pro(t, fuente, Rectangle::new(cx - lado / 2.0, cy - lado / 2.0, lado, lado), Vector2::zero(), 0.0, tinte);
            }
        };
        // Las lunas: cuatro tarjetas grandes con su foto y cuantas quedan.
        let tam = (h * 0.026).clamp(15.0, 22.0) as i32;
        ui::texto(d, "LUNAS DE GALILEO · se usan volando", x0 as i32, (h * 0.19) as i32, tam, Color::new(255, 210, 120, 255));
        let lado = ((ancho_izq - 3.0 * 14.0) / 4.0).min(h * 0.17);
        for k in 0..4 {
            let x = x0 + k as f32 * (lado + 14.0);
            let y = h * 0.23;
            let entra = ((self.abierta - k as f32 * 0.04) / 0.2).clamp(0.0, 1.0);
            let a = (entra * 255.0) as u8;
            let escogida = self.cursor == k;
            let n = p.mochila[k];
            let c = COLOR_LUNA[k];
            let color = Color::new((c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, a);
            d.draw_rectangle_gradient_v(x as i32, y as i32, lado as i32, lado as i32, Color::new(40, 30, 70, a), Color::new(14, 10, 32, a));
            if escogida {
                d.draw_rectangle_lines_ex(Rectangle::new(x - 1.0, y - 1.0, lado + 2.0, lado + 2.0), 2.0, Color::new(255, 130, 210, 255));
                ui::seleccion(d, x, y, lado, lado, self.reloj, Color::new(255, 220, 245, 255));
            }
            let (cx, cy) = (x + lado / 2.0, y + lado * 0.40);
            if n > 0 {
                d.draw_circle_v(Vector2::new(cx, cy), lado * 0.30, Color::new(color.r, color.g, color.b, a / 6));
            }
            let crece = if escogida { 1.0 + 0.04 * (self.reloj * 4.0).sin() } else { 1.0 };
            foto(d, k, cx, cy, lado * 0.72 * crece, if n > 0 { Color::new(255, 255, 255, a) } else { Color::new(90, 80, 120, a) });
            let nombre = LUNAS[k];
            ui::texto(d, nombre, (cx as i32) - ui::medir(nombre, tam) / 2, (y + lado * 0.76) as i32, tam, Color::new(255, 255, 255, a));
            let cuantas = format!("x{n}");
            ui::texto(d, &cuantas, (x + lado) as i32 - ui::medir(&cuantas, tam) - 8, y as i32 + 6, tam, if n > 0 { Color::new(255, 230, 150, a) } else { ui::TENUE });
        }
        // Los recuerdos: una cuadricula con su foto en 3D (en sombra si todavia no se tiene).
        let y_rec = h * 0.23 + lado + 34.0;
        let tenidos = (4..Self::total()).filter(|&k| Self::tiene(p, k)).count();
        let encabezado = format!("RECUERDOS · {tenidos} de {}", RECUERDOS.len());
        ui::texto(d, &encabezado, x0 as i32, (y_rec - 26.0) as i32, tam, Color::new(255, 210, 120, 255));
        let cols = 5;
        let lado_r = ((ancho_izq - (cols - 1) as f32 * 12.0) / cols as f32).min((h * 0.93 - y_rec) / 3.0 - 12.0);
        for i in 0..RECUERDOS.len() {
            let (c, f) = (i % cols, i / cols);
            let x = x0 + c as f32 * (lado_r + 12.0);
            let y = y_rec + f as f32 * (lado_r + 12.0);
            let tiene = Self::tiene(p, 4 + i);
            let escogida = self.cursor == 4 + i;
            let fondo = if tiene { Color::new(36, 28, 66, 255) } else { Color::new(22, 18, 40, 255) };
            d.draw_rectangle_rounded(Rectangle::new(x, y, lado_r, lado_r), 0.12, 6, fondo);
            if tiene {
                d.draw_circle_gradient((x + lado_r / 2.0) as i32, (y + lado_r * 0.55) as i32, lado_r * 0.42, Color::new(110, 70, 190, 90), Color::new(36, 28, 66, 0));
            }
            if escogida {
                d.draw_rectangle_rounded_lines(Rectangle::new(x - 1.0, y - 1.0, lado_r + 2.0, lado_r + 2.0), 0.12, 6, Color::new(255, 130, 210, 255));
                ui::seleccion(d, x, y, lado_r, lado_r, self.reloj, Color::new(255, 220, 245, 255));
            }
            let crece = if escogida { 1.0 + 0.05 * (self.reloj * 4.0).sin() } else { 1.0 };
            foto(d, 4 + i, x + lado_r / 2.0, y + lado_r / 2.0, lado_r * 0.92 * crece, if tiene { Color::WHITE } else { Color::new(16, 12, 34, 255) });
            if !tiene {
                let t = (lado_r * 0.26) as i32;
                ui::texto(d, "?", (x + lado_r / 2.0) as i32 - ui::medir("?", t) / 2, (y + lado_r / 2.0) as i32 - t / 2, t, Color::new(150, 140, 190, 200));
            }
        }
        // A la derecha: lo escogido, en grande y girando (la vitrina).
        let xd = x0 + ancho_izq + w * 0.03;
        let wd = w - xd - w * 0.04;
        ui::panel(d, xd as i32, (h * 0.19) as i32, wd as i32, (h * 0.72) as i32);
        let tt = (h * 0.034).clamp(18.0, 30.0) as i32;
        let tp = (h * 0.024).clamp(15.0, 20.0) as i32;
        let (titulo, sub, texto, extra) = if self.cursor < 4 {
            let k = self.cursor;
            let extra = if self.volando { format!("{}: usar ahorita", ui::boton(e.nombre("aceptar"))) } else { "Se usa volando (desde la pausa).".to_string() };
            (format!("Luna {}", LUNAS[k]), format!("tiene {}", p.mochila[k]), USO_LUNA[k].to_string(), extra)
        } else {
            let r = &RECUERDOS[self.cursor - 4];
            if Self::tiene(p, self.cursor) {
                (r.nombre.to_string(), r.lugar.to_string(), r.texto.to_string(), String::new())
            } else {
                ("???".to_string(), r.lugar.to_string(), format!("Termine la aventura de {} para traer este recuerdo.", r.lugar), String::new())
            }
        };
        let xi = xd as i32 + 22;
        let mut y = (h * 0.19) as i32 + 14;
        let wv = wd - 28.0;
        let hv = (wv * vitrina.height as f32 / vitrina.width as f32).min(h * 0.32);
        let wv = hv * vitrina.width as f32 / vitrina.height as f32;
        let fuente = Rectangle::new(0.0, 0.0, vitrina.width as f32, vitrina.height as f32);
        d.draw_texture_pro(vitrina, fuente, Rectangle::new(xd + (wd - wv) / 2.0, y as f32, wv, hv), Vector2::zero(), 0.0, Color::WHITE);
        y += hv as i32 + 14;
        ui::texto(d, &titulo, xi, y, tt, Color::new(110, 235, 225, 255));
        ui::texto(d, &sub, xi, y + tt + 6, tp, ui::TENUE);
        let y_fin = ui::parrafo(d, &texto, xi, y + tt + tp + 18, tp, wd as i32 - 44, ui::TEXTO);
        if !extra.is_empty() {
            ui::texto(d, &extra, xi, y_fin + 12, tp, Color::new(255, 230, 150, 255));
        }
        if let Some(a) = &self.aviso {
            ui::parrafo(d, a, xi, y_fin + 44, tp, wd as i32 - 44, Color::new(255, 150, 160, 255));
        }
        let ayuda = format!("{}: escoger     {}: usar     {}: volver", ui::boton("flechas"), ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("pausa")));
        ui::ayuda(d, &ayuda, ancho, alto);
    }
}
