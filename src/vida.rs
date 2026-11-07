//! LA VIDA de los minijuegos: lo que hace que algo se sienta vivo, en un solo lugar y sin tocar sus
//! reglas. Letreros gigantes de bloques que revientan, un cartel de entrada con destellos, confeti,
//! sacudida de pantalla y destello. Cada minijuego lleva una `Vida` y le grita lo que pasa.
//! Es la misma estetica de WACK A KOCH y ¡Anillazo!.

use crate::ui;
use raylib::prelude::*;
use std::f32::consts::TAU;

/// Los tonos de un grito.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tono {
    Oro,
    Rosa,
    Cian,
    Verde,
    Rojo,
    Lila,
    Blanco,
}

impl Tono {
    /// El tono que mas se parece a un color.
    pub fn de_color(c: Color) -> Tono {
        let (r, g, b) = (c.r as i32, c.g as i32, c.b as i32);
        if r > 220 && g > 190 && b > 190 {
            Tono::Blanco
        } else if r > 200 && g > 170 {
            Tono::Oro
        } else if g > r && g > b {
            Tono::Verde
        } else if b > r && g > 150 {
            Tono::Cian
        } else if b > r {
            Tono::Lila
        } else if b > 150 {
            Tono::Rosa
        } else {
            Tono::Rojo
        }
    }

    fn colores(self) -> (Color, Color) {
        match self {
            Tono::Oro => (Color::new(255, 240, 130, 255), Color::new(255, 130, 50, 255)),
            Tono::Rosa => (Color::new(255, 190, 240, 255), Color::new(255, 70, 190, 255)),
            Tono::Cian => (Color::new(190, 250, 255, 255), Color::new(70, 180, 255, 255)),
            Tono::Verde => (Color::new(205, 255, 205, 255), Color::new(70, 225, 120, 255)),
            Tono::Rojo => (Color::new(255, 200, 120, 255), Color::new(255, 55, 70, 255)),
            Tono::Lila => (Color::new(225, 200, 255, 255), Color::new(160, 90, 255, 255)),
            Tono::Blanco => (Color::WHITE, Color::new(200, 215, 255, 255)),
        }
    }

    fn color(self) -> Color {
        self.colores().1
    }
}

/// Cuantas veces cabe la altura de la pantalla: lo grande que va un grito.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tam {
    Chico,
    Medio,
    Grande,
    Enorme,
}

impl Tam {
    fn lado(self) -> f32 {
        match self {
            Tam::Chico => 0.0058,
            Tam::Medio => 0.0085,
            Tam::Grande => 0.0125,
            Tam::Enorme => 0.0175,
        }
    }

    fn dura(self) -> f32 {
        match self {
            Tam::Chico => 0.8,
            Tam::Medio => 1.1,
            Tam::Grande => 1.5,
            Tam::Enorme => 1.9,
        }
    }
}

#[derive(Clone)]
struct Grito {
    texto: String,
    sub: String,
    tono: Tono,
    tam: Tam,
    vida: f32,
    dura: f32,
    x: f32,
    y: f32,
}

#[derive(Clone, Copy)]
struct Papelito {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    vida: f32,
    lado: f32,
    color: Color,
}

#[derive(Clone)]
pub struct Vida {
    /// El cartel de entrada: nombre, frase y color.
    cartel: Option<(String, String, Tono)>,
    t: f32,
    gritos: Vec<Grito>,
    papelitos: Vec<Papelito>,
    semilla: u32,
    /// Sacudida (de 0 a 1) y destello de toda la pantalla.
    pub sacudida: f32,
    destello: f32,
    tinte: Color,
}

/// Lo que dura el cartel de entrada, en segundos.
const CARTEL: f32 = 2.6;

impl Vida {
    /// Para las escenas que ya traen su propio titulo.
    pub fn sin_cartel() -> Self {
        let mut v = Vida::new("", "", Tono::Oro);
        v.cartel = None;
        v
    }

    pub fn new(cartel: &str, frase: &str, tono: Tono) -> Self {
        Vida {
            cartel: Some((cartel.to_string(), frase.to_string(), tono)),
            t: 0.0,
            gritos: Vec::new(),
            papelitos: Vec::new(),
            semilla: cartel.len() as u32 * 7919 + 13,
            sacudida: 0.0,
            destello: 0.0,
            tinte: Color::WHITE,
        }
    }

    fn azar(&mut self) -> f32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) as f32 / 16_777_216.0
    }

    /// Un letrero que revienta en el centro de arriba. Quita al que estaba ahi: no se enciman.
    pub fn grito(&mut self, texto: &str, tono: Tono, tam: Tam) {
        self.grito_en(texto, "", tono, tam, 0.5, 0.26);
    }

    /// Con una frase abajo.
    pub fn grito_con(&mut self, texto: &str, sub: &str, tono: Tono, tam: Tam) {
        self.grito_en(texto, sub, tono, tam, 0.5, 0.26);
    }

    /// Sin cartel de entrada: para las escenas que ya traen el suyo, y para los videos (donde cada
    /// toma lleva su rotulo).
    pub fn quitar_cartel(&mut self) {
        self.cartel = None;
    }

    /// En un lugar de la pantalla (de 0 a 1).
    pub fn grito_en(&mut self, texto: &str, sub: &str, tono: Tono, tam: Tam, x: f32, y: f32) {
        // El cartel de entrada le cede el lugar: se apaga rapidito para que no se encimen.
        if tam != Tam::Chico && y < 0.5 {
            self.t = self.t.max(CARTEL - 0.2);
        }
        self.gritos.retain(|g| (g.x - x).abs() > 0.2 || (g.y - y).abs() > 0.12);
        self.gritos.push(Grito { texto: texto.to_string(), sub: sub.to_string(), tono, tam, vida: tam.dura(), dura: tam.dura(), x, y });
        if self.gritos.len() > 6 {
            self.gritos.remove(0);
        }
    }

    /// Un golpe: sacude y destella. `fuerza` de 0 a 1.
    pub fn golpe(&mut self, fuerza: f32, tono: Tono) {
        self.sacudida = self.sacudida.max(fuerza);
        self.destello = self.destello.max(fuerza * 0.32);
        self.tinte = tono.colores().0;
    }

    /// Una lluvia de papelitos: para ganar algo.
    pub fn confeti(&mut self, n: usize, tono: Tono) {
        const OTROS: [Tono; 5] = [Tono::Oro, Tono::Rosa, Tono::Cian, Tono::Verde, Tono::Lila];
        for k in 0..n {
            if self.papelitos.len() >= 220 {
                break;
            }
            let (a, b, c, d) = (self.azar(), self.azar(), self.azar(), self.azar());
            let color = if k % 2 == 0 { tono.color() } else { OTROS[(c * 5.0) as usize % 5].color() };
            // Sale de la mitad de arriba, hacia todos lados.
            let dir = a * TAU;
            self.papelitos.push(Papelito { x: 0.5 + (b - 0.5) * 0.5, y: 0.3 + (d - 0.5) * 0.2, vx: dir.cos() * (0.15 + c * 0.45), vy: dir.sin() * (0.15 + c * 0.45) - 0.35, vida: 1.2 + d * 1.1, lado: 0.008 + c * 0.012, color });
        }
    }

    /// Que paso algo muy bueno: todo junto.
    pub fn exito(&mut self, texto: &str, sub: &str, tono: Tono) {
        self.grito_con(texto, sub, tono, Tam::Enorme);
        self.golpe(0.55, tono);
        self.confeti(90, tono);
    }

    /// Que paso algo muy malo.
    pub fn desastre(&mut self, texto: &str, sub: &str) {
        self.grito_con(texto, sub, Tono::Rojo, Tam::Enorme);
        self.golpe(0.9, Tono::Rojo);
    }

    pub fn actualizar(&mut self, dt: f32) {
        self.t += dt;
        self.sacudida = (self.sacudida - dt * 2.6).max(0.0);
        self.destello = (self.destello - dt * 3.0).max(0.0);
        for g in &mut self.gritos {
            g.vida -= dt;
        }
        self.gritos.retain(|g| g.vida > 0.0);
        for p in &mut self.papelitos {
            p.vida -= dt;
            p.vy += dt * 0.9;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vx *= 1.0 - (dt * 1.2).min(1.0);
        }
        self.papelitos.retain(|p| p.vida > 0.0 && p.y < 1.1);
    }

    /// Cuanto correr la imagen del juego (en pixeles de ventana): la sacudida de pantalla.
    pub fn corrimiento(&self, alto: f32) -> (f32, f32) {
        let s = self.sacudida * self.sacudida * alto * 0.018;
        ((self.t * 71.0).sin() * s, (self.t * 93.0).cos() * s)
    }

    pub fn dibujar(&self, d: &mut RaylibDrawHandle, imagen: Rectangle) {
        let (ix, iy, w, h) = (imagen.x, imagen.y, imagen.width, imagen.height);
        if self.destello > 0.0 {
            let c = Color::new(self.tinte.r, self.tinte.g, self.tinte.b, (self.destello.min(0.6) * 255.0) as u8);
            d.draw_rectangle(ix as i32, iy as i32, w as i32, h as i32, c);
        }
        for p in &self.papelitos {
            let a = (p.vida / 0.4).min(1.0);
            let l = h * p.lado;
            let c = Color::new(p.color.r, p.color.g, p.color.b, (a * 255.0) as u8);
            d.draw_rectangle_rec(Rectangle::new(ix + p.x * w, iy + p.y * h, l, l), c);
        }
        // El cartel de entrada: revienta, se queda y sube a desaparecer.
        if let Some((nombre, frase, tono)) = &self.cartel {
            if self.t < CARTEL {
                let entra = (self.t / 0.16).min(1.0);
                let brote = 1.0 + (1.0 - entra).powi(2) * 0.9;
                let a = (((CARTEL - self.t) / 0.45).clamp(0.0, 1.0) * 255.0) as u8;
                let cabe = (w - 30.0) / (nombre.chars().count() as f32 * 6.0);
                let u = (h * 0.0165 * brote).min(cabe);
                let (cx, cy) = (ix + w * 0.5, iy + h * 0.2 - (self.t / CARTEL).powi(3) * h * 0.04);
                let (t0, t1) = tono.colores();
                ui::bloques(d, nombre, cx, cy, u, t0, t1, a);
                if !frase.is_empty() {
                    let tam = (h * 0.04) as i32;
                    let ancho = ui::medir(frase, tam);
                    ui::subtitulo(d, frase, cx as i32 - ancho / 2, (cy + u * 5.0) as i32, tam, Color::new(255, 255, 255, a));
                }
                // Sus destellos.
                let largo = ui::medir_bloques(nombre, u);
                for k in 0..7 {
                    let s = k as f32 * 2.399;
                    let brillo = (self.t * 3.4 + s).sin().abs().powi(5);
                    if brillo > 0.05 {
                        let (x, y) = (cx + (s * 4.3).sin() * largo * 0.5, cy + (s * 6.1).cos() * u * 3.5);
                        estrella(d, x, y, h * 0.045 * brillo, Color::new(255, 255, 245, a));
                    }
                }
            }
        }
        for g in &self.gritos {
            let edad = g.dura - g.vida;
            let brote = 1.0 + (1.0 - (edad / 0.12).min(1.0)).powi(2) * 0.8;
            let a = ((g.vida / 0.25).min(1.0) * 255.0) as u8;
            let cabe = (w - 24.0) / (g.texto.chars().count() as f32 * 6.0);
            let u = (h * g.tam.lado() * brote).min(cabe);
            let ancho = ui::medir_bloques(&g.texto, u);
            let cx = (ix + w * g.x).max(ix + ancho * 0.5 + 8.0).min(ix + w - ancho * 0.5 - 8.0);
            let cy = iy + h * g.y - edad * h * 0.012;
            let (t0, t1) = g.tono.colores();
            ui::bloques(d, &g.texto, cx, cy, u, t0, t1, a);
            if !g.sub.is_empty() {
                let tam = (h * 0.038) as i32;
                let ancho = ui::medir(&g.sub, tam);
                ui::subtitulo(d, &g.sub, cx as i32 - ancho / 2, (cy + u * 5.0) as i32, tam, Color::new(255, 255, 255, a));
            }
        }
    }
}

/// Una estrellita de cuatro puntas.
fn estrella(d: &mut RaylibDrawHandle, x: f32, y: f32, r: f32, c: Color) {
    let a = r * 0.09;
    d.draw_triangle(Vector2::new(x, y - r), Vector2::new(x - a, y), Vector2::new(x + a, y), c);
    d.draw_triangle(Vector2::new(x, y + r), Vector2::new(x + a, y), Vector2::new(x - a, y), c);
    d.draw_triangle(Vector2::new(x - r, y), Vector2::new(x, y + a), Vector2::new(x, y - a), c);
    d.draw_triangle(Vector2::new(x + r, y), Vector2::new(x, y - a), Vector2::new(x, y + a), c);
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn los_gritos_duran_y_se_van() {
        let mut v = Vida::new("PRUEBA", "", Tono::Oro);
        assert!(v.t < CARTEL);
        v.grito("¡ESO!", Tono::Rosa, Tam::Medio);
        assert!(v.t >= CARTEL - 0.2, "el cartel cede su lugar");
        assert_eq!(v.gritos.len(), 1);
        v.grito("¡OTRO!", Tono::Cian, Tam::Medio);
        assert_eq!(v.gritos.len(), 1, "en el mismo lugar no se enciman");
        for _ in 0..200 {
            v.actualizar(1.0 / 60.0);
        }
        assert!(v.gritos.is_empty());
    }

    #[test]
    fn un_exito_sacude_y_tira_confeti_y_todo_se_apaga() {
        let mut v = Vida::new("PRUEBA", "", Tono::Oro);
        v.exito("¡GANÓ!", "bien", Tono::Oro);
        assert!(v.sacudida > 0.0 && !v.papelitos.is_empty());
        let (x, y) = v.corrimiento(720.0);
        assert!(x.abs() < 20.0 && y.abs() < 20.0, "la sacudida no marea");
        for _ in 0..400 {
            v.actualizar(1.0 / 60.0);
        }
        assert_eq!(v.sacudida, 0.0);
        assert!(v.papelitos.is_empty());
    }
}
