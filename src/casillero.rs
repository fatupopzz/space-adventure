//! Dos pantallas del menu principal, como en Fortnite:
//! - el CASILLERO, donde se escoge el personaje (los que faltan dicen en que nivel salen), y
//! - el PASE ESPACIAL, la fila de premios por nivel con la barra de XP.
//! Las dos ensenan a la derecha la vitrina 3D (el personaje bailando, o la nave).

use crate::entrada::Entrada;
use crate::personajes::PERSONAJES;
use crate::progreso::{nombre_del_premio, Premio, Progreso, ESTELAS, PASE};
use crate::sonido::Efecto;
use crate::tienda::{indice_skin, Muestra};
use crate::ui;
use raylib::prelude::*;

/// Las imagenes de las tarjetas (pintadas por el rasterizador al arrancar): el retrato de cada
/// personaje, cada skin de nave y la nave con cada estela.
pub struct Imagenes<'a> {
    pub retratos: &'a [Texture2D],
    pub naves: &'a [Texture2D],
    pub estelas: &'a [Texture2D],
    /// Las fotos de la mochila: las cuatro lunas y luego los recuerdos.
    pub recuerdos: &'a [Texture2D],
}

/// Pega una imagen cuadrada en `(x, y)` de `lado` x `lado`, con ese tinte.
fn imagen(d: &mut RaylibDrawHandle, t: &Texture2D, x: f32, y: f32, lado: f32, tinte: Color) {
    let fuente = Rectangle::new(0.0, 0.0, t.width as f32, t.height as f32);
    d.draw_texture_pro(t, fuente, Rectangle::new(x, y, lado, lado), Vector2::zero(), 0.0, tinte);
}

/// El icono de las notas: una corchea rosa con su brillo (dibujada, no es una textura).
fn icono_de_notas(d: &mut RaylibDrawHandle, cx: f32, cy: f32, lado: f32, alfa: u8) {
    let rosa = Color::new(255, 130, 210, alfa);
    d.draw_circle_v(Vector2::new(cx, cy), lado * 0.42, Color::new(255, 200, 90, alfa / 4));
    let r = lado * 0.13;
    let (x1, y1) = (cx - lado * 0.16, cy + lado * 0.18);
    let (x2, y2) = (cx + lado * 0.20, cy + lado * 0.10);
    d.draw_circle_v(Vector2::new(x1, y1), r, rosa);
    d.draw_circle_v(Vector2::new(x2, y2), r, rosa);
    let grueso = lado * 0.06;
    d.draw_rectangle_v(Vector2::new(x1 + r - grueso, y1 - lado * 0.42), Vector2::new(grueso, lado * 0.42), rosa);
    d.draw_rectangle_v(Vector2::new(x2 + r - grueso, y2 - lado * 0.42), Vector2::new(grueso, lado * 0.42), rosa);
    d.draw_line_ex(Vector2::new(x1 + r - grueso, y1 - lado * 0.42), Vector2::new(x2 + r, y2 - lado * 0.42), grueso * 1.6, rosa);
}

const ROSA: Color = Color::new(255, 130, 210, 235);
const CIAN: Color = Color::new(110, 235, 225, 255);
const ORO: Color = Color::new(255, 230, 150, 255);

/// El titulo inclinado arriba a la izquierda y, a la derecha, el nivel del pase.
fn encabezado(d: &mut RaylibDrawHandle, titulo: &str, p: &Progreso, ancho: i32, alto: i32) {
    let (w, h) = (ancho as f32, alto as f32);
    d.draw_rectangle(0, 0, ancho, alto, Color::new(6, 4, 22, 235));
    ui::paralelogramo(d, -40.0, h * 0.085, w * 0.46, h * 0.072, 26.0, Color::new(18, 40, 120, 235));
    let tam = (h * 0.055).clamp(26.0, 52.0);
    ui::titulo_menu(d, titulo, w * 0.04, h * 0.098, tam, -3.0);
    let t = (h * 0.032).clamp(18.0, 30.0) as i32;
    let (nivel, _, _) = p.nivel_del_pase();
    let s = format!("Nivel {nivel} del pase  ·  {} notas", p.notas);
    ui::texto(d, &s, ancho - ui::medir(&s, t) - (w * 0.04) as i32, (h * 0.10) as i32, t, ORO);
}

/// La vitrina (la textura de la nave o el personaje) en su panel, y debajo el titulo y el texto.
/// Devuelve donde termino el texto.
fn vitrina(d: &mut RaylibDrawHandle, textura: &Texture2D, x: f32, y: f32, w: f32, h: f32, titulo: &str, texto: &str, crudo: bool) -> i32 {
    ui::panel(d, x as i32, y as i32, w as i32, h as i32);
    let fuente = Rectangle::new(0.0, 0.0, textura.width as f32, textura.height as f32);
    let escala = ((w - 24.0) / textura.width as f32).min(h * 0.62 / textura.height as f32);
    let (tw, th) = (textura.width as f32 * escala, textura.height as f32 * escala);
    d.draw_texture_pro(textura, fuente, Rectangle::new(x + (w - tw) / 2.0, y + 12.0, tw, th), Vector2::zero(), 0.0, Color::WHITE);
    let alto = d.get_screen_height() as f32;
    let tam_t = (alto * 0.04).clamp(20.0, 34.0) as i32;
    let y_t = (y + th + 24.0) as i32;
    if crudo {
        ui::texto_crudo(d, titulo, x as i32 + 24, y_t, tam_t, CIAN);
    } else {
        ui::texto(d, titulo, x as i32 + 24, y_t, tam_t, CIAN);
    }
    let tam_p = (alto * 0.024).clamp(15.0, 21.0) as i32;
    ui::parrafo(d, texto, x as i32 + 24, y_t + tam_t + 10, tam_p, w as i32 - 48, ui::TEXTO)
}

pub struct Casillero {
    cursor: usize,
    desde: f32,
    abierta: f32,
    pub reloj: f32,
    aviso: Option<String>,
    pub eventos: Vec<Efecto>,
}

const COLUMNAS: usize = 5;

/// Los personajes que salen en el casillero: todos menos los secretos que todavia no se ganan
/// (esos no se ven hasta desbloquearlos, para que sean sorpresa).
fn en_casillero(p: &Progreso) -> Vec<usize> {
    (0..PERSONAJES.len()).filter(|&k| crate::personajes::por_logro(PERSONAJES[k].id).is_none() || p.tiene_personaje(PERSONAJES[k].id)).collect()
}

impl Casillero {
    pub fn new(p: &Progreso) -> Self {
        Casillero { cursor: crate::personajes::indice(&p.personaje), desde: 1.0, abierta: 0.0, reloj: 0.0, aviso: None, eventos: Vec::new() }
    }

    pub fn muestra(&self) -> Muestra {
        Muestra::Personaje(self.cursor)
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
        // Se mueve entre los que se ven (los secretos no salen hasta ganarlos).
        let visibles = en_casillero(p);
        let n = visibles.len();
        let antes = self.cursor;
        let mut j = visibles.iter().position(|&k| k == self.cursor).unwrap_or(0);
        if e.derecha {
            j = (j + 1) % n;
        }
        if e.izquierda {
            j = (j + n - 1) % n;
        }
        if e.abajo {
            j = (j + COLUMNAS).min(n - 1);
        }
        if e.arriba {
            j = j.saturating_sub(COLUMNAS);
        }
        self.cursor = visibles[j];
        if self.cursor != antes {
            self.desde = 0.0;
            self.aviso = None;
            self.eventos.push(Efecto::Carta);
        }
        if e.aceptar {
            let f = &PERSONAJES[self.cursor];
            if p.personaje == f.id {
            } else if p.tiene_personaje(f.id) {
                p.personaje = f.id.to_string();
                p.guardar();
                self.eventos.push(Efecto::Sello);
                self.aviso = Some(format!("{} es tu piloto.", f.nombre));
            } else {
                self.eventos.push(Efecto::Fallo);
                self.aviso = Some(match crate::personajes::como_se_gana(f.id) {
                    Some(como) => format!("Todavía no. {como}"),
                    None => format!("Todavía no: sale en el nivel {} del pase.", Progreso::nivel_del_personaje(f.id)),
                });
            }
        }
        false
    }

    pub fn dibujar(&self, d: &mut RaylibDrawHandle, e: &Entrada, p: &Progreso, textura: &Texture2D, retratos: &[Texture2D], ancho: i32, alto: i32) {
        let (w, h) = (ancho as f32, alto as f32);
        encabezado(d, "PERSONAJES", p, ancho, alto);
        // Las tarjetas: 5 por fila, a la izquierda.
        let x0 = w * 0.05;
        let ancho_grilla = (w * 0.48).min(720.0);
        let hueco = 10.0;
        let lado_w = (ancho_grilla - hueco * (COLUMNAS as f32 - 1.0)) / COLUMNAS as f32;
        let visibles = en_casillero(p);
        let filas = visibles.len().div_ceil(COLUMNAS);
        let lado_h = ((h * 0.66) / filas as f32 - hueco).min(lado_w * 1.25);
        let y0 = h * 0.19;
        let tam = (lado_h * 0.16).clamp(12.0, 20.0) as i32;
        for (j, &k) in visibles.iter().enumerate() {
            let f = &PERSONAJES[k];
            let (c, r) = (j % COLUMNAS, j / COLUMNAS);
            let entra = ((self.abierta - j as f32 * 0.025) / 0.2).clamp(0.0, 1.0);
            let a = (entra * 255.0) as u8;
            let x = x0 + c as f32 * (lado_w + hueco);
            let y = y0 + r as f32 * (lado_h + hueco) + (1.0 - entra) * 20.0;
            let tiene = p.tiene_personaje(f.id);
            let puesto = p.personaje == f.id;
            let escogida = k == self.cursor;
            let caja = Rectangle::new(x, y, lado_w, lado_h);
            // Tarjeta: degradado de color si se tiene, gris si no.
            let (arriba, abajo) = if tiene { (Color::new(70, 50, 150, a), Color::new(30, 120, 160, a)) } else { (Color::new(40, 36, 60, a), Color::new(24, 22, 40, a)) };
            d.draw_rectangle_gradient_v(x as i32, y as i32, lado_w as i32, lado_h as i32, arriba, abajo);
            if escogida {
                d.draw_rectangle_lines_ex(Rectangle::new(x - 1.0, y - 1.0, lado_w + 2.0, lado_h + 2.0), 2.0, ROSA);
                ui::seleccion(d, x, y, lado_w, lado_h, self.reloj, Color::new(255, 220, 245, 255));
            } else {
                d.draw_rectangle_lines_ex(caja, 1.0, Color::new(185, 140, 255, a / 2));
            }
            // El retrato (oscuro, como silueta, si todavia no se tiene) y el nombre abajo.
            if let Some(r) = retratos.get(k) {
                let lado = (lado_h * 0.68).min(lado_w);
                let tinte = if tiene { Color::new(255, 255, 255, a) } else { Color::new(40, 34, 70, a) };
                let fuente = Rectangle::new(0.0, 0.0, r.width as f32, r.height as f32);
                d.draw_texture_pro(r, fuente, Rectangle::new(x + (lado_w - lado) / 2.0, y + 2.0, lado, lado), Vector2::zero(), 0.0, tinte);
            }
            let nombre_w = ui::medir_crudo(f.nombre, tam);
            let tn = if nombre_w > lado_w as i32 - 8 { tam * (lado_w as i32 - 8) / nombre_w.max(1) } else { tam }.max(9);
            let cn = if tiene { Color::new(240, 232, 255, a) } else { Color::new(150, 140, 180, a) };
            d.draw_rectangle((x + 1.0) as i32, (y + lado_h * 0.70) as i32, lado_w as i32 - 2, (lado_h * 0.30) as i32 - 1, Color::new(10, 8, 30, a / 2));
            ui::texto_crudo(d, f.nombre, (x + lado_w / 2.0) as i32 - ui::medir_crudo(f.nombre, tn) / 2, (y + lado_h * 0.78) as i32, tn, cn);
            if puesto {
                ui::texto(d, "PUESTO", x as i32 + 6, y as i32 + 4, tam * 3 / 4, Color::new(120, 235, 160, a));
            } else if !tiene {
                let s = if crate::personajes::por_logro(f.id).is_some() { "SECRETO".to_string() } else { format!("Nv. {}", Progreso::nivel_del_personaje(f.id)) };
                ui::texto(d, &s, x as i32 + 6, y as i32 + 4, tam * 3 / 4, Color::new(230, 140, 160, a));
            }
        }
        // La vitrina: el escogido bailando.
        let xv = x0 + ancho_grilla + w * 0.04;
        let f = &PERSONAJES[self.cursor];
        let y_fin = vitrina(d, textura, xv, h * 0.17, w - xv - w * 0.04, h * 0.76, f.nombre, f.texto, true);
        let tp = (h * 0.024).clamp(15.0, 21.0) as i32;
        let estado = if p.personaje == f.id {
            ("Es tu piloto ahorita.".to_string(), ui::VERDE)
        } else if p.tiene_personaje(f.id) {
            (format!("{}: escoger", ui::boton(e.nombre("aceptar"))), ORO)
        } else {
            let (nivel, xp, _) = p.nivel_del_pase();
            match crate::personajes::como_se_gana(f.id) {
                Some(como) => (format!("Secreto. {como}"), Color::new(230, 140, 160, 255)),
                None => {
                    let falta: u32 = (nivel + 1..=Progreso::nivel_del_personaje(f.id)).map(crate::progreso::xp_del_nivel).sum::<u32>().saturating_sub(xp);
                    (format!("Se desbloquea en el nivel {} del pase (le faltan {falta} XP).", Progreso::nivel_del_personaje(f.id)), Color::new(230, 140, 160, 255))
                }
            }
        };
        let texto = self.aviso.clone().unwrap_or(estado.0);
        ui::parrafo(d, &texto, xv as i32 + 24, y_fin + 12, tp, (w - xv - w * 0.04) as i32 - 48, estado.1);
        let ayuda = format!("{}: escoger     {}: ponerse     {}: volver", ui::boton("flechas"), ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("pausa")));
        ui::ayuda(d, &ayuda, ancho, alto);
    }
}

pub struct Pase {
    cursor: usize,
    desde: f32,
    abierta: f32,
    pub reloj: f32,
    pub eventos: Vec<Efecto>,
    aviso: Option<String>,
}

impl Pase {
    /// Abre en el siguiente premio que falta.
    pub fn new(p: &Progreso) -> Self {
        let nivel = p.nivel_del_pase().0 as usize;
        Pase { cursor: nivel.min(PASE.len() - 1), desde: 1.0, abierta: 0.0, reloj: 0.0, eventos: Vec::new(), aviso: None }
    }

    pub fn muestra(&self, p: &Progreso) -> Muestra {
        match PASE[self.cursor] {
            Premio::Personaje(id) => Muestra::Personaje(crate::personajes::indice(id)),
            Premio::Skin(id) => Muestra::Nave { skin: indice_skin(id), estela: None },
            Premio::Estela(id) => Muestra::Nave { skin: indice_skin(&p.skin), estela: Some(ESTELAS.iter().find(|e| e.0 == id).map_or("", |e| e.0)) },
            Premio::Notas(_) => Muestra::Nave { skin: indice_skin(&p.skin), estela: None },
        }
    }

    pub fn actualizar(&mut self, e: &Entrada, p: &mut Progreso, dt: f32) -> bool {
        self.desde += dt;
        self.abierta += dt;
        self.reloj += dt;
        if e.pausa {
            self.eventos.push(Efecto::Carta);
            return true;
        }
        let antes = self.cursor;
        if e.derecha || e.abajo {
            self.cursor = (self.cursor + 1).min(PASE.len() - 1);
        }
        if e.izquierda || e.arriba {
            self.cursor = self.cursor.saturating_sub(1);
        }
        if self.cursor != antes {
            self.aviso = None;
            self.desde = 0.0;
            self.eventos.push(Efecto::Carta);
        }
        if e.aceptar {
            let premio = PASE[self.cursor];
            // Tambien conserva los cosmeticos comprados en partidas anteriores.
            let disponible = match premio {
                Premio::Personaje(id) => p.tiene_personaje(id),
                Premio::Skin(id) => p.tiene_skin(id),
                Premio::Estela(id) => p.estelas.contains(id),
                Premio::Notas(_) => self.cursor < p.pase_cobrado as usize,
            };
            if disponible {
                match premio {
                    Premio::Personaje(id) => { p.personaje = id.to_string(); p.guardar(); }
                    Premio::Skin(id) => { p.skin_escogida(id); }
                    Premio::Estela(id) if p.estela.as_deref() != Some(id) => { p.estela_escogida(id); }
                    _ => {}
                }
                self.aviso = Some(if matches!(premio, Premio::Notas(_)) { "Estas notas ya se entregaron.".into() } else { format!("Equipado: {}", nombre_del_premio(premio)) });
                self.eventos.push(Efecto::Acierto);
            } else {
                self.aviso = Some(format!("Se desbloquea en el nivel {} del pase.", self.cursor + 1));
                self.eventos.push(Efecto::Fallo);
            }
        }
        false
    }

    pub fn dibujar(&self, d: &mut RaylibDrawHandle, e: &Entrada, p: &Progreso, textura: &Texture2D, img: &Imagenes, ancho: i32, alto: i32) {
        let (w, h) = (ancho as f32, alto as f32);
        encabezado(d, "PASE ESPACIAL", p, ancho, alto);
        let (nivel, xp, siguiente) = p.nivel_del_pase();
        // A la izquierda: el nivel en grande y la barra de XP.
        let x0 = w * 0.05;
        let ancho_izq = (w * 0.42).min(620.0);
        let y0 = h * 0.18;
        let tam_n = (h * 0.15).clamp(60.0, 140.0);
        ui::texto(d, "NIVEL", x0 as i32, y0 as i32, (tam_n * 0.25) as i32, ui::TENUE);
        let s = nivel.to_string();
        ui::titulo_menu(d, &s, x0, y0 + tam_n * 0.25, tam_n, -4.0);
        let yb = y0 + tam_n * 1.35;
        let alto_b = (h * 0.03).max(14.0);
        d.draw_rectangle_rounded(Rectangle::new(x0, yb, ancho_izq, alto_b), 0.5, 8, Color::new(40, 30, 80, 255));
        let lleno = if nivel as usize >= PASE.len() { 1.0 } else { xp as f32 / siguiente as f32 };
        if lleno > 0.0 {
            d.draw_rectangle_rounded(Rectangle::new(x0, yb, (ancho_izq * lleno).max(alto_b), alto_b), 0.5, 8, CIAN);
        }
        let tb = (h * 0.026).clamp(15.0, 22.0) as i32;
        let barra = if nivel as usize >= PASE.len() { "¡Pase completo!".to_string() } else { format!("{xp} / {siguiente} XP para el nivel {}", nivel + 1) };
        ui::texto(d, &barra, x0 as i32, (yb + alto_b + 8.0) as i32, tb, ui::TEXTO);
        let como = "XP: 5 por nota, 40 por logro, 120 por sello, 60 por encargo y 80 por cometa.";
        ui::parrafo(d, como, x0 as i32, (yb + alto_b + 8.0) as i32 + tb + 10, tb * 4 / 5, ancho_izq as i32, ui::TENUE);

        // La vitrina del premio escogido, a la derecha.
        let premio = PASE[self.cursor];
        let (tipo, texto) = match premio {
            Premio::Personaje(id) => ("PERSONAJE", PERSONAJES[crate::personajes::indice(id)].texto.to_string()),
            Premio::Skin(id) => ("SKIN DE NAVE", crate::tienda::SKINS[indice_skin(id)].texto.to_string()),
            Premio::Estela(id) => ("ESTELA", crate::tienda::texto_estela(id).to_string()),
            Premio::Notas(n) => ("NOTAS", format!("{n} notas. Fondo de emergencia para caprichos: la tienda acepta música, el casero todavía no.")),
        };
        let xv = x0 + ancho_izq + w * 0.04;
        let wv = w - xv - w * 0.04;
        let titulo = format!("{} · {}", tipo, nombre_del_premio(premio));
        vitrina(d, textura, xv, h * 0.17, wv, h * 0.48, &titulo, &texto, true);

        // Abajo: la fila de premios, nivel por nivel, centrada en el escogido.
        let yc = h * 0.675;
        let alto_c = h * 0.255;
        let ancho_c = (w * 0.11).clamp(110.0, 180.0);
        let paso = ancho_c + 12.0;
        let visibles = ((w - 2.0 * x0) / paso) as usize;
        let primero = self.cursor.saturating_sub(visibles / 2).min(PASE.len().saturating_sub(visibles));
        let tc = (alto_c * 0.11).clamp(12.0, 18.0) as i32;
        for (j, k) in (primero..PASE.len()).take(visibles).enumerate() {
            let x = x0 + j as f32 * paso;
            let entra = ((self.abierta - j as f32 * 0.03) / 0.2).clamp(0.0, 1.0);
            let a = (entra * 255.0) as u8;
            let y = yc + (1.0 - entra) * 20.0;
            let obtenido = (k as u32) < nivel;
            let escogida = k == self.cursor;
            let p_k = PASE[k];
            let color = match p_k {
                Premio::Personaje(_) => Color::new(150, 70, 200, a),
                Premio::Skin(_) => Color::new(40, 110, 190, a),
                Premio::Estela(_) => Color::new(200, 80, 150, a),
                Premio::Notas(_) => Color::new(170, 130, 40, a),
            };
            let fondo = if obtenido { color } else { Color::new(color.r / 3, color.g / 3, color.b / 3 + 20, a) };
            d.draw_rectangle_gradient_v(x as i32, y as i32, ancho_c as i32, alto_c as i32, fondo, Color::new(10, 8, 30, a));
            if escogida {
                d.draw_rectangle_lines_ex(Rectangle::new(x - 1.0, y - 1.0, ancho_c + 2.0, alto_c + 2.0), 2.0, ROSA);
                ui::seleccion(d, x, y, ancho_c, alto_c, self.reloj, Color::new(255, 220, 245, 255));
            }
            // La imagen del premio, arriba al centro; los bloqueados, un poco apagados.
            let lado = (alto_c * 0.52).min(ancho_c - 10.0);
            let (xi, yi) = (x + (ancho_c - lado) / 2.0, y + alto_c * 0.06);
            let tinte = if obtenido { Color::new(255, 255, 255, a) } else { Color::new(175, 170, 200, a) };
            match p_k {
                Premio::Personaje(id) => {
                    if let Some(t) = img.retratos.get(crate::personajes::indice(id)) {
                        imagen(d, t, xi, yi, lado, tinte);
                    }
                }
                Premio::Skin(id) => {
                    if let Some(t) = img.naves.get(indice_skin(id)) {
                        imagen(d, t, xi, yi, lado, tinte);
                    }
                }
                Premio::Estela(id) => {
                    if let Some(t) = ESTELAS.iter().position(|e| e.0 == id).and_then(|i| img.estelas.get(i)) {
                        imagen(d, t, xi, yi, lado, tinte);
                    }
                }
                Premio::Notas(_) => icono_de_notas(d, xi + lado / 2.0, yi + lado / 2.0, lado, if obtenido { a } else { (a as f32 * 0.7) as u8 }),
            }
            // El numero del nivel, en una esquina, encima de la imagen.
            d.draw_rectangle(x as i32, y as i32, tc * 2 + 6, tc + 10, Color::new(10, 8, 30, a / 2 + 60));
            ui::texto(d, &format!("{}", k + 1), x as i32 + 5, y as i32 + 5, tc, Color::new(255, 255, 255, a));
            let tipo = match p_k {
                Premio::Personaje(_) => "personaje",
                Premio::Skin(_) => "skin",
                Premio::Estela(_) => "estela",
                Premio::Notas(_) => "notas",
            };
            let y_texto = y + alto_c * 0.60;
            ui::texto(d, tipo, x as i32 + 8, y_texto as i32, tc * 3 / 4, Color::new(220, 210, 255, a));
            let nombre = nombre_del_premio(p_k);
            let lineas = ui::envolver(&nombre, tc, ancho_c as i32 - 14);
            for (l, linea) in lineas.iter().take(2).enumerate() {
                ui::texto_crudo(d, linea, x as i32 + 8, y_texto as i32 + tc + l as i32 * (tc + 2), tc, Color::new(255, 255, 255, a));
            }
            // Arriba a la derecha: una palomita verde si ya se gano, un candadito si no.
            let (cx, cy, r) = (x + ancho_c - tc as f32 * 0.9, y + tc as f32 * 0.9, tc as f32 * 0.62);
            if obtenido {
                d.draw_circle_v(Vector2::new(cx, cy), r, Color::new(40, 160, 90, a));
                d.draw_line_ex(Vector2::new(cx - r * 0.45, cy), Vector2::new(cx - r * 0.1, cy + r * 0.38), r * 0.28, Color::WHITE);
                d.draw_line_ex(Vector2::new(cx - r * 0.1, cy + r * 0.38), Vector2::new(cx + r * 0.5, cy - r * 0.35), r * 0.28, Color::WHITE);
            } else {
                d.draw_circle_v(Vector2::new(cx, cy), r, Color::new(10, 8, 30, a / 2 + 80));
                let gris = Color::new(210, 190, 230, a);
                d.draw_ring(Vector2::new(cx, cy - r * 0.18), r * 0.22, r * 0.36, 180.0, 360.0, 12, gris);
                d.draw_rectangle_v(Vector2::new(cx - r * 0.45, cy - r * 0.15), Vector2::new(r * 0.9, r * 0.6), gris);
            }
        }
        if let Some(aviso) = &self.aviso {
            ui::texto(d, aviso, x0 as i32, (h * 0.62) as i32, 18, ui::VERDE);
        }
        let ayuda = format!("{}: ver premios     {}: equipar premio obtenido     {}: volver", ui::boton("flechas"), ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("pausa")));
        ui::ayuda(d, &ayuda, ancho, alto);
    }
}

/// La pantalla que esta abierta encima del menu principal (la tienda, el casillero o el pase).
pub enum Submenu {
    Tienda(crate::tienda::Tienda),
    Casillero(Casillero),
    Pase(Pase),
    Mochila(crate::mochila::Mochila),
}

impl Submenu {
    /// Devuelve si hay que cerrarla.
    pub fn actualizar(&mut self, e: &Entrada, p: &mut Progreso, dt: f32) -> bool {
        match self {
            Submenu::Tienda(t) => matches!(t.actualizar(e, p, dt), crate::tienda::AccionTienda::Cerrar),
            Submenu::Casillero(c) => c.actualizar(e, p, dt),
            Submenu::Pase(s) => s.actualizar(e, p, dt),
            Submenu::Mochila(m) => m.actualizar(e, p, dt),
        }
    }

    pub fn eventos(&mut self) -> Vec<Efecto> {
        match self {
            Submenu::Tienda(t) => std::mem::take(&mut t.eventos),
            Submenu::Casillero(c) => std::mem::take(&mut c.eventos),
            Submenu::Pase(s) => std::mem::take(&mut s.eventos),
            Submenu::Mochila(m) => std::mem::take(&mut m.eventos),
        }
    }

    /// Lo que ensena la vitrina y su reloj (para que gire).
    pub fn muestra(&self, p: &Progreso) -> (Muestra, f32) {
        match self {
            Submenu::Tienda(t) => (t.vitrina(p), t.reloj),
            Submenu::Casillero(c) => (c.muestra(), c.reloj),
            Submenu::Pase(s) => (s.muestra(p), s.reloj),
            Submenu::Mochila(m) => (m.muestra(p), m.reloj),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn dibujar(&self, d: &mut RaylibDrawHandle, e: &Entrada, p: &Progreso, textura: &Texture2D, img: &Imagenes, ancho: i32, alto: i32) {
        match self {
            Submenu::Tienda(t) => t.dibujar(d, e, p, textura, ancho, alto),
            Submenu::Casillero(c) => c.dibujar(d, e, p, textura, img.retratos, ancho, alto),
            Submenu::Pase(s) => s.dibujar(d, e, p, textura, img, ancho, alto),
            Submenu::Mochila(m) => m.dibujar(d, e, p, textura, img.recuerdos, ancho, alto),
        }
    }
}

/// Las pestanas del menu del juego (las mismas en el menu principal y en la pausa).
pub const PESTANAS: [&str; 6] = ["JUGAR", "PERSONAJES", "MOCHILA", "PASE", "TIENDA", "AJUSTES"];

/// La barra de pestanas arriba: la activa rellena de cian; las que tienen algo nuevo, con un
/// puntito rosa; y a los lados, los botones para cambiar (L1 / R1, LB / RB o Q / E).
#[allow(clippy::too_many_arguments)]
pub fn dibujar_pestanas(d: &mut RaylibDrawHandle, e: &Entrada, activa: usize, en_menu: bool, novedades: (bool, bool, bool), reloj: f32, ancho: i32, alto: i32) {
    let (w, h) = (ancho as f32, alto as f32);
    let alto_barra = (h * 0.065).max(40.0);
    d.draw_rectangle_gradient_v(0, 0, ancho, alto_barra as i32, Color::new(10, 6, 34, 245), Color::new(10, 6, 34, 200));
    d.draw_rectangle(0, alto_barra as i32 - 2, ancho, 2, Color::new(110, 235, 225, 120));
    let nombres: Vec<&str> = PESTANAS.iter().enumerate().map(|(k, n)| if k == 0 && en_menu { "INICIO" } else { n }).collect();
    let (izq, der) = (e.nombre("pestana_izq"), e.nombre("pestana_der"));
    // Todas caben siempre (con los botones de los lados): en una ventana angosta, mas chicas.
    let medidas = |tam: i32| {
        let hueco = (w * 0.025).clamp(10.0, 18.0 + tam as f32);
        let anchos: Vec<f32> = nombres.iter().map(|n| ui::medir(n, tam) as f32 + tam as f32 * 1.6).collect();
        let total: f32 = anchos.iter().sum::<f32>() + hueco * (nombres.len() as f32 - 1.0);
        let botones = (ui::medir_tecla(izq, tam) + ui::medir_tecla(der, tam)) as f32 + hueco * 2.0;
        (hueco, anchos, total, total + botones)
    };
    let mut tam = (alto_barra * 0.42) as i32;
    while tam > 11 && medidas(tam).3 > w * 0.96 {
        tam -= 1;
    }
    let (hueco, anchos, total, _) = medidas(tam);
    let mut x = (w - total) / 2.0;
    let y = alto_barra * 0.16;
    let alto_p = alto_barra * 0.68;
    // Los botones para cambiar de pestana, a los lados.
    let wi = ui::medir_tecla(izq, tam);
    ui::tecla(d, izq, (x - wi as f32 - hueco) as i32, (y + alto_p * 0.1) as i32, tam, false);
    for (k, nombre) in nombres.iter().enumerate() {
        let activa_k = k == activa;
        if activa_k {
            ui::paralelogramo(d, x, y, anchos[k], alto_p, 10.0, Color::new(110, 235, 225, 255));
        }
        let color = if activa_k { Color::new(10, 10, 30, 255) } else { Color::new(220, 210, 255, 220) };
        ui::texto(d, nombre, (x + tam as f32 * 0.8) as i32, (y + (alto_p - tam as f32) / 2.0) as i32, tam, color);
        let nuevo = match k {
            1 => novedades.0,
            3 => novedades.1,
            4 => novedades.2,
            _ => false,
        };
        if nuevo && !activa_k {
            let r = 4.0 + 1.0 * (reloj * 5.0).sin().abs();
            d.draw_circle_v(Vector2::new(x + anchos[k] - 8.0, y + 4.0), r, Color::new(255, 110, 200, 255));
        }
        x += anchos[k] + hueco;
    }
    ui::tecla(d, der, x as i32, (y + alto_p * 0.1) as i32, tam, false);
}

#[cfg(test)]
mod pruebas_pase {
    use super::*;

    #[test]
    fn el_pase_equipa_premios_sin_cobrar_ni_cerrar() {
        let mut p = Progreso::en_memoria();
        p.notas_totales = 5000;
        p.cobrar_pase();
        let notas = p.notas;
        let mut pase = Pase::new(&p);
        for (i, premio) in PASE.iter().enumerate() {
            pase.cursor = i;
            assert!(!pase.actualizar(&Entrada { aceptar: true, ..Default::default() }, &mut p, 0.1));
            match premio {
                Premio::Skin(id) => assert_eq!(&p.skin, id),
                Premio::Estela(id) => assert_eq!(p.estela.as_deref(), Some(*id)),
                Premio::Personaje(id) => assert_eq!(&p.personaje, id),
                Premio::Notas(_) => {}
            }
            assert_eq!(p.notas, notas);
        }
        assert!(pase.actualizar(&Entrada { pausa: true, ..Default::default() }, &mut p, 0.1));
    }

    #[test]
    fn un_premio_bloqueado_no_se_equipa_pero_una_compra_antigua_si() {
        let mut p = Progreso::en_memoria();
        p.notas = 1000;
        let mut pase = Pase::new(&p);
        pase.cursor = PASE.iter().position(|x| *x == Premio::Skin("chapina")).unwrap();
        pase.actualizar(&Entrada { aceptar: true, ..Default::default() }, &mut p, 0.1);
        assert_eq!(p.skin, "lila");
        assert_eq!(p.notas, 1000);
        p.skins.insert("chapina".into());
        pase.actualizar(&Entrada { aceptar: true, ..Default::default() }, &mut p, 0.1);
        assert_eq!(p.skin, "chapina");
        assert_eq!(p.notas, 1000);
    }
}
