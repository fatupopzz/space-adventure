//! Aterrizaje en el Planeta Hada: la Gran Fuente de las Hadas, el Proyecto 2
//! (que era un raytracer) rehecho en voxels y rasterizado. Homenaje a The
//! Legend of Zelda: Ocarina of Time.
//!
//! Como Link en el Proyecto 2, Miku se para en la Trifuerza del piso y toca
//! una cancion con la ocarina (los cinco botones: A, arriba, abajo, izquierda
//! y derecha). Cada dos notas bien tocadas se enciende un triangulo de la
//! Trifuerza. Con la cancion completa, en vez de la Gran Hada sale del cuenco
//! Megurine Luka cantando, como en un spin-off (con su tarjeta de titulo): la
//! Gran Hada se jubilo y le dejo la fuente. Bendice a Miku con polvo de hada y
//! Miku levanta un Contenedor de Corazon, con la pose de "obtuviste un objeto".
//! Navi anda volando alrededor.
//!
//! Todo esta en las coordenadas de la fuente: el piso de la plaza en y = 0.

use crate::vida::{Tam, Tono, Vida};
use crate::entrada::Entrada;
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;
use std::f32::consts::PI;

/// Los cinco botones de la ocarina.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Nota {
    A,
    Arriba,
    Abajo,
    Izquierda,
    Derecha,
}

impl Nota {
    fn efecto(self) -> Efecto {
        match self {
            Nota::A => Efecto::OcarinaA,
            Nota::Abajo => Efecto::OcarinaAbajo,
            Nota::Derecha => Efecto::OcarinaDerecha,
            Nota::Izquierda => Efecto::OcarinaIzquierda,
            Nota::Arriba => Efecto::OcarinaArriba,
        }
    }
}

/// La Cancion de Miku: la que llama a la Gran Hada.
pub const CANCION: [Nota; 6] = [Nota::Arriba, Nota::Izquierda, Nota::Derecha, Nota::Arriba, Nota::Izquierda, Nota::Derecha];

const GLADAS: &str = ui::NOMBRE_IA;
const NAVI: &str = "NAVI";
const LUKA: &str = "LUKA";

const NAVI_DICE: [&str; 2] = [
    "¡Hey! ¡Escucha! ¡Esta es la Gran Fuente de las Hadas!",
    "Párate en la Trifuerza y toca la Canción de Miku. ¡La Gran Hada sale con música, igualita que tú!",
];
const LUKA_DICE: [&str; 3] = [
    "¡LA, LA, LAAAAA!",
    "¿Miku? ¿En MI spin-off? ...Bueno, tocaste bien la canción. La Gran Hada se jubiló y me dejó la fuente. Ahora es mi estudio de grabación.",
    "Por tu canción te doy mi bendición... y un corazón. Úsalo bien. No lo gastes chocando con el Sol.",
];

/// Donde esta cada cosa en la fuente.
const TRIFUERZA: Vec3 = Vec3::new(0.0, 0.2, -2.55);
const CUENCO: Vec3 = Vec3::new(0.0, 0.62, 1.2);
const ANTORCHAS: [Vec3; 2] = [Vec3::new(-3.3, 1.5, -2.0), Vec3::new(3.3, 1.5, -2.0)];
/// Segundos que tarda la Gran Hada en salir del cuenco.
const SALIR: f32 = 2.8;
const BENDICION: f32 = 3.4;
const OBJETO: f32 = 4.2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fase {
    Llegada,
    Navi(usize),
    Tocar,
    /// Se equivoco de nota: Navi regana y se vuelve a empezar.
    Error,
    Hada(usize),
    Bendicion,
    Objeto,
    Libre,
}

pub enum AccionFuente {
    Nada,
    Despegar,
}

pub struct Fuente {
    fase: Fase,
    t: f32,
    pub total: f32,
    /// Cuantas notas de la cancion van bien.
    progreso: usize,
    /// Las ultimas notas tocadas (para dibujarlas) y cuando sono la ultima.
    tocadas: Vec<Nota>,
    desde_nota: f32,
    /// Segundos desde que salio la Gran Hada (para su baile).
    desde_hada: f32,
    /// Los letreros gigantes, el confeti y las sacudidas (ver vida.rs).
    pub vida: Vida,
    pub eventos: Vec<Efecto>,
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

impl Fuente {
    pub fn new() -> Self {
        Fuente { vida: Vida::sin_cartel(), fase: Fase::Llegada, t: 0.0, total: 0.0, progreso: 0, tocadas: Vec::new(), desde_nota: 9.0, desde_hada: 0.0, eventos: Vec::new() }
    }

    fn cambiar(&mut self, fase: Fase) {
        self.fase = fase;
        self.t = 0.0;
    }

    /// Cuantos triangulos de la Trifuerza estan encendidos (uno por cada dos notas).
    pub fn triangulos_encendidos(&self) -> usize {
        match self.fase {
            Fase::Hada(_) | Fase::Bendicion | Fase::Objeto | Fase::Libre => 3,
            _ => (self.progreso / 2).min(3),
        }
    }

    /// Si Navi esta reganando por una nota mala.
    pub fn reganando(&self) -> bool {
        self.fase == Fase::Error
    }

    /// Si la Gran Hada ya salio.
    pub fn hada_afuera(&self) -> bool {
        matches!(self.fase, Fase::Hada(_) | Fase::Bendicion | Fase::Objeto | Fase::Libre)
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionFuente {
        self.t += dt;
        self.total += dt;
        self.desde_nota += dt;
        if self.hada_afuera() {
            self.desde_hada += dt;
        }
        if e.aterrizar {
            return AccionFuente::Despegar;
        }
        let largo = self.hablante().map_or(0, |h| h.1.chars().count()) as f32;
        let siguiente = (e.aceptar && self.t > 0.4) || self.t > 2.5 + largo / 22.0;
        match self.fase {
            Fase::Llegada if siguiente || self.t > 6.0 => self.cambiar(Fase::Navi(0)),
            Fase::Navi(i) if siguiente => self.cambiar(if i + 1 < NAVI_DICE.len() { Fase::Navi(i + 1) } else { Fase::Tocar }),
            Fase::Tocar => {
                let nota = if e.aceptar && self.t > 0.3 {
                    Some(Nota::A)
                } else if e.arriba {
                    Some(Nota::Arriba)
                } else if e.abajo {
                    Some(Nota::Abajo)
                } else if e.izquierda {
                    Some(Nota::Izquierda)
                } else if e.derecha {
                    Some(Nota::Derecha)
                } else {
                    None
                };
                if let Some(n) = nota {
                    self.tocar(n);
                }
            }
            Fase::Error if self.t > 2.6 || (e.aceptar && self.t > 0.5) => {
                self.progreso = 0;
                self.tocadas.clear();
                self.cambiar(Fase::Tocar);
            }
            Fase::Hada(0) if self.t > SALIR => self.cambiar(Fase::Hada(1)),
            Fase::Hada(i) if i > 0 && siguiente => self.cambiar(if i + 1 < LUKA_DICE.len() { Fase::Hada(i + 1) } else { Fase::Bendicion }),
            Fase::Bendicion if self.t > BENDICION => {
                self.vida.exito("¡BENDICIÓN!", "Un corazón más", Tono::Rosa);
                self.eventos.push(Efecto::Perfecto);
                self.cambiar(Fase::Objeto);
            }
            Fase::Objeto if self.t > OBJETO => self.cambiar(Fase::Libre),
            Fase::Libre if e.aceptar && self.t > 0.6 => {
                // Otra vez: la Gran Hada se esconde y se vuelve a tocar.
                self.progreso = 0;
                self.tocadas.clear();
                self.desde_hada = 0.0;
                self.cambiar(Fase::Tocar);
            }
            _ => {}
        }
        AccionFuente::Nada
    }

    /// Toca una nota: si es la que sigue en la cancion, avanza; si no, Navi regana.
    fn tocar(&mut self, n: Nota) {
        self.eventos.push(n.efecto());
        self.tocadas.push(n);
        self.desde_nota = 0.0;
        if CANCION[self.progreso] == n {
            self.progreso += 1;
            if self.progreso % 2 == 0 {
                let palabra = ["¡ESO!", "¡AFINADA!", "¡QUÉ VOZ!", "¡SIGA!"][self.progreso / 2 % 4];
                self.vida.grito(palabra, Tono::Rosa, Tam::Medio);
                self.eventos.push(Efecto::Acierto);
            }
            if self.progreso == CANCION.len() {
                self.vida.exito("¡CANCIÓN COMPLETA!", "La Gran Hada sale", Tono::Rosa);
                self.eventos.push(Efecto::Fuego);
                self.desde_hada = 0.0;
                self.cambiar(Fase::Hada(0));
            }
        } else {
            self.vida.grito("¡DESAFINÓ!", Tono::Rojo, Tam::Grande);
            self.vida.golpe(0.45, Tono::Rojo);
            self.eventos.push(Efecto::Fallo);
            self.cambiar(Fase::Error);
        }
    }

    pub fn hablante(&self) -> Option<(&'static str, &'static str, Voz)> {
        Some(match self.fase {
            Fase::Llegada => (GLADAS, "Planeta Hada. Esto era el Proyecto 2: un raytracer. Aquí lo rehicimos con triángulos. Que no se entere el raytracer.", Voz::Ia),
            Fase::Navi(i) => (NAVI, NAVI_DICE[i], Voz::Robotina),
            Fase::Tocar => return None,
            Fase::Error => (NAVI, "¡Hey! ¡Escucha! Esa no era. Otra vez, desde el principio.", Voz::Robotina),
            Fase::Hada(i) => (LUKA, LUKA_DICE[i], Voz::Hada),
            Fase::Bendicion => (GLADAS, "Detecto polvo de hada. Es brillantina. No sé cómo voy a limpiar la nave.", Voz::Ia),
            Fase::Objeto => ("MIKU", "¡Un corazón! ¡Lo voy a poner en mi próximo concierto!", Voz::Miku),
            Fase::Libre => (LUKA, "Cuando te canses de pelear con planetas, vuelve. Grabamos un dueto. Yo pongo la voz grave. Y el título.", Voz::Hada),
        })
    }

    pub fn expresion_miku(&self) -> &'static str {
        match self.fase {
            Fase::Navi(_) => "guino",
            Fase::Tocar if self.desde_nota < 0.4 => "feliz",
            Fase::Tocar => "serio",
            Fase::Error => "asustada",
            Fase::Hada(0) => "asustada",
            Fase::Hada(_) | Fase::Bendicion => "enamorada",
            Fase::Objeto => "guino",
            _ => "feliz",
        }
    }

    pub fn expresion_hada(&self) -> &'static str {
        match self.fase {
            Fase::Hada(0) => "guino",
            Fase::Bendicion => "enamorada",
            Fase::Libre if (self.total * 0.6) as i32 % 3 == 0 => "guino",
            _ => "feliz",
        }
    }

    /// Que paso de baile hace Miku: saluda tocando, levanta el objeto, baila al final.
    pub fn paso_miku(&self) -> crate::baile::Paso {
        use crate::baile::Paso;
        match self.fase {
            Fase::Objeto => Paso::Corazon,
            Fase::Libre => Paso::Baile,
            _ => Paso::Saludo,
        }
    }

    /// Miku parada en la Trifuerza, mirando a la fuente (+Z).
    pub fn miku(&self) -> Mat4 {
        let brinco = if self.fase == Fase::Objeto { 0.1 * (self.t * 3.0).sin().abs() * (self.t < 0.6) as i32 as f32 } else { 0.0 };
        Mat4::translation(TRIFUERZA + Vec3::new(0.0, brinco, 0.0)) * Mat4::rotation_y(PI) * Mat4::scale(0.8)
    }

    /// La Gran Hada: sale del cuenco dando vueltas, y despues flota meciendose.
    pub fn hada(&self) -> Option<Mat4> {
        if !self.hada_afuera() {
            return None;
        }
        let k = suave(self.desde_hada / SALIR);
        let giro = (1.0 - k) * PI * 4.0;
        let flota = 0.12 * (self.desde_hada * 1.4).sin() * k;
        let pos = CUENCO + Vec3::new(0.0, -2.4 * (1.0 - k) + flota + 0.15 * k, 0.0);
        let baile = 0.12 * (self.desde_hada * 1.1).sin() * k;
        Some(Mat4::translation(pos) * Mat4::rotation_y(giro) * Mat4::rotation_z(baile) * Mat4::scale(1.0))
    }

    /// Navi da vueltas alrededor de la cabeza de Miku.
    pub fn navi(&self) -> Vec3 {
        let a = self.total * 2.2;
        TRIFUERZA + Vec3::new(0.5 * a.cos(), 1.15 + 0.1 * (self.total * 3.1).sin(), 0.45 * a.sin())
    }

    /// El Contenedor de Corazon arriba de la cabeza de Miku, girando.
    pub fn contenedor(&self) -> Option<Mat4> {
        (self.fase == Fase::Objeto || (self.fase == Fase::Libre && self.t < 1.0)).then(|| {
            let sube = suave(self.t / 0.6);
            Mat4::translation(TRIFUERZA + Vec3::new(0.0, 0.7 + 0.55 * sube, 0.0)) * Mat4::rotation_y(self.total * 2.0) * Mat4::scale(1.2)
        })
    }

    /// Las partes de la fuente a ocultar: los triangulos de la Trifuerza en su
    /// version apagada o encendida.
    pub fn ocultar(&self) -> Vec<&'static str> {
        const APAGADO: [&str; 3] = ["trifuerza_0", "trifuerza_1", "trifuerza_2"];
        const ENCENDIDO: [&str; 3] = ["trifuerza_luz_0", "trifuerza_luz_1", "trifuerza_luz_2"];
        let n = self.triangulos_encendidos();
        (0..3).map(|k| if k < n { APAGADO[k] } else { ENCENDIDO[k] }).collect()
    }

    /// La camara: una toma por momento, como en el Proyecto 2 (que tambien iba
    /// cambiando de plano con la cancion).
    pub fn camara(&self) -> (Vec3, Vec3) {
        let vaiven = 0.15 * (self.total * 0.3).sin();
        match self.fase {
            // Plano general dando la vuelta a la isla, como la pantalla de titulo.
            Fase::Llegada => {
                let a = -1.9 + self.t * 0.12;
                let r = 16.0 - 4.0 * suave(self.t / 6.0);
                (Vec3::new(r * a.cos(), 5.5 - 1.5 * suave(self.t / 6.0), r * a.sin()), Vec3::new(0.0, 0.3, 0.3))
            }
            // Por detras de Miku, viendo la fuente.
            Fase::Navi(_) | Fase::Tocar | Fase::Error => (Vec3::new(1.5 + vaiven, 1.8, -5.4), Vec3::new(0.0, 0.7, 0.4)),
            // Desde abajo, de lado: la Gran Hada sale del cuenco.
            Fase::Hada(0) => (Vec3::new(-2.6, 0.55, -1.4), Vec3::new(0.0, 1.0 + 1.2 * suave(self.t / SALIR), 1.2)),
            Fase::Hada(_) => (Vec3::new(-1.8 + vaiven, 1.2, -1.8), Vec3::new(0.0, 1.9, 1.2)),
            Fase::Bendicion => (Vec3::new(2.4, 1.7, -4.4), Vec3::new(0.0, 1.0, -0.8)),
            // De frente a Miku, con la fuente atras de la camara.
            Fase::Objeto => (Vec3::new(0.6, 1.25, -1.0), Vec3::new(0.0, 1.0, -2.55)),
            // Detras de Miku, un poco arriba, viendo a las dos y la fuente; se mece de lado a lado.
            Fase::Libre => {
                let lado = 1.6 * (self.total * 0.15).sin();
                (Vec3::new(lado, 2.4, -6.2), Vec3::new(0.0, 0.9, 0.2))
            }
        }
    }

    /// La aurora, los brillos que caen, el fuego de las antorchas, el polvo de
    /// la bendicion y la luz de Navi (todo con luz aditiva).
    pub fn dibujar_efectos(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4) {
        // La lluvia de brillos alrededor de la fuente.
        for k in 0..70 {
            let a = k as f32 * 2.399;
            let r = 1.5 + (k as f32 * 0.618).fract() * 4.0;
            let caida = (self.total * (0.12 + 0.05 * (k % 4) as f32) + k as f32 * 0.137).fract();
            let p = Vec3::new(r * a.cos(), 4.5 - caida * 5.0, r * a.sin() + 0.5);
            let brillo = (0.5 + 0.5 * (self.total * 5.0 + k as f32).sin()) * (1.0 - caida);
            let c = if k % 3 == 0 { [1.0, 0.7, 0.9] } else { [0.8, 0.9, 1.0] };
            if let Some((x, y, z)) = proyectar(p, vista_proyeccion, fb) {
                let t = 2.5 * brillo;
                fb.linea_de_luz((x - t, y, z), (x + t, y, z), [0.0; 3], c.map(|v| v * brillo));
                fb.linea_de_luz((x, y - t, z), (x, y + t, z), [0.0; 3], c.map(|v| v * brillo));
            }
        }
        // El polvo de hada dorado: baja de las manos de la Gran Hada hasta Miku.
        if self.fase == Fase::Bendicion {
            for k in 0..60 {
                let u = (self.t * 0.6 + k as f32 / 60.0).fract();
                let lado = if k % 2 == 0 { 0.6 } else { -0.6 };
                let desde = CUENCO + Vec3::new(lado, 1.6, -0.2);
                let hasta = TRIFUERZA + Vec3::new(0.0, 0.9, 0.0);
                let arco = Vec3::new(0.25 * (k as f32 * 1.7 + u * 6.0).sin(), (u * PI).sin() * 0.8, 0.0);
                let p = desde.lerp(hasta, u) + arco;
                if let Some((x, y, z)) = proyectar(p, vista_proyeccion, fb) {
                    let b = (u * PI).sin();
                    fb.linea_de_luz((x - 2.0, y, z), (x + 2.0, y, z), [0.0; 3], [b, 0.8 * b, 0.3 * b]);
                    fb.linea_de_luz((x, y - 2.0, z), (x, y + 2.0, z), [0.0; 3], [b, 0.8 * b, 0.3 * b]);
                }
            }
        }
    }

    /// Los resplandores: Navi, las antorchas, la Trifuerza encendida, el corazon.
    pub fn dibujar_luces(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4) {
        if let Some((x, y, _)) = proyectar(self.navi(), vista_proyeccion, fb) {
            crate::post::halo(fb, (x, y), 34.0, [0.45, 0.65, 0.9], 1.6);
        }
        for (k, a) in ANTORCHAS.iter().enumerate() {
            let parpadeo = 0.8 + 0.2 * (self.total * 13.0 + k as f32 * 2.0).sin() * (self.total * 7.3).sin();
            if let Some((x, y, _)) = proyectar(*a, vista_proyeccion, fb) {
                crate::post::halo(fb, (x, y), 60.0 * parpadeo, [0.6 * parpadeo, 0.3 * parpadeo, 0.08], 1.6);
            }
        }
        let n = self.triangulos_encendidos();
        if n > 0 {
            if let Some((x, y, _)) = proyectar(TRIFUERZA + Vec3::new(0.0, 0.1, 0.0), vista_proyeccion, fb) {
                let k = n as f32 / 3.0;
                crate::post::halo(fb, (x, y), 70.0 + 40.0 * k, [0.5 * k, 0.4 * k, 0.1 * k], 1.4);
            }
        }
        if let Some(m) = self.contenedor() {
            if let Some((x, y, _)) = proyectar(m.transform_point(Vec3::zero()), vista_proyeccion, fb) {
                crate::post::halo(fb, (x, y), 90.0, [0.6, 0.15, 0.25], 1.5);
            }
        }
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, ancho: i32, alto: i32) {
        let (aceptar, salir) = (&ui::boton(e.nombre("aceptar")), &ui::boton(e.nombre("aterrizar")));
        let centrado = |d: &mut RaylibDrawHandle, s: &str, y: i32, tam: i32, color: Color| ui::centrado(d, s, ancho, y, tam, color);
        // La tarjeta del titulo al llegar.
        if self.fase == Fase::Llegada {
            let alfa = (suave(self.t / 0.8) * (1.0 - suave((self.t - 4.5) / 1.0)) * 255.0) as u8;
            let tam = 60;
            let s = "LA GRAN FUENTE DE LAS HADAS";
            let w = ui::medir_titulo(s, tam);
            ui::titulo_de(d, s, (ancho - w) / 2, 70, tam, Color::new(255, 220, 255, alfa));
            let s2 = "Proyecto 2, ahora con triángulos";
            ui::texto(d, s2, (ancho - ui::medir(s2, 24)) / 2, 145, 24, Color::new(220, 200, 255, alfa));
        }
        // Cuando sale Luka: la tarjeta de su spin-off, como titulo de juego.
        if let Fase::Hada(i) = self.fase {
            let k = if i == 0 { suave((self.t - 1.0) / 0.8) } else if i == 1 { 1.0 - suave((self.t - 2.5) / 0.8) } else { 0.0 };
            if k > 0.0 {
                let a = (k * 255.0) as u8;
                let pre = "MEGURINE LUKA";
                ui::texto(d, pre, (ancho - ui::medir(pre, 26)) / 2, 40, 26, Color::new(255, 210, 120, a));
                let titulo = "LA FUENTE DE LUKA";
                let w = ui::medir(titulo, 66);
                ui::texto(d, titulo, (ancho - w) / 2 + 4, 74, 66, Color::new(40, 5, 30, a));
                ui::texto(d, titulo, (ancho - w) / 2, 70, 66, Color::new(255, 150, 200, a));
                let sub = "un spin-off de Space Adventure";
                ui::subtitulo(d, sub, (ancho - ui::medir(sub, 22)) / 2, 146, 22, Color::new(230, 215, 255, a));
            }
        }
        // Lo que se dice, abajo.
        if let (Some((nombre, _, _)), false) = (self.hablante(), texto.is_empty()) {
            let w = 920.min(ancho - 40);
            let h = ui::alto_dialogo(nombre, texto, w);
            let color = match nombre {
                NAVI => Color::new(150, 220, 255, 255),
                LUKA => Color::new(255, 140, 190, 255),
                _ => ui::LILA,
            };
            ui::caja_dialogo(d, nombre, color, texto, (ancho - w) / 2, alto - h - 20, w);
        }
        // Tocando: la cancion con sus botones, y los que ya van bien encendidos.
        if matches!(self.fase, Fase::Tocar | Fase::Error) {
            let (w, h) = (560, 150);
            let (x, y) = ((ancho - w) / 2, alto - h - 30);
            ui::panel(d, x, y, w, h);
            ui::texto(d, "CANCIÓN DE MIKU", x + 24, y + 14, 22, Color::new(255, 220, 140, 255));
            for (k, n) in CANCION.iter().enumerate() {
                let bien = k < self.progreso;
                boton_ocarina(d, *n, (x + 60 + k as i32 * 88) as f32, (y + 84) as f32, 28.0, bien);
            }
            let ayuda = format!("{} y {aceptar}: tocar la ocarina      {salir}: despegar", ui::boton("flechas"));
            ui::texto(d, &ayuda, (ancho - ui::medir(&ayuda, 18)) / 2, y - 30, 18, ui::TEXTO);
        }
        // La nota que acaba de sonar, flotando arriba.
        if let (Some(&n), true) = (self.tocadas.last(), self.desde_nota < 0.6) {
            let sube = (self.desde_nota * 80.0) as i32;
            boton_ocarina(d, n, (ancho / 2) as f32, (alto / 2 - 60 - sube) as f32, 34.0, true);
        }
        if self.fase == Fase::Objeto && self.t > 0.5 {
            let s = "¡OBTUVISTE UN CONTENEDOR DE CORAZÓN!";
            centrado(d, s, 60, 46, Color::new(255, 120, 150, 255));
            centrado(d, "Tu nave aguanta un choque más. Bueno, no. Pero se siente bonito.", 120, 22, ui::TEXTO);
        }
        if self.fase == Fase::Libre {
            let s = format!("{aceptar}: tocar otra vez      {salir}: despegar");
            centrado(d, &s, 30, 24, Color::new(255, 220, 255, 255));
        }
    }
}

/// Un boton de la ocarina como en el juego: la A azul y las flechas amarillas.
fn boton_ocarina(d: &mut RaylibDrawHandle, n: Nota, x: f32, y: f32, r: f32, encendido: bool) {
    let (fondo, borde) = match (n, encendido) {
        (Nota::A, true) => (Color::new(90, 150, 255, 255), Color::new(200, 230, 255, 255)),
        (Nota::A, false) => (Color::new(40, 60, 120, 255), Color::new(90, 110, 170, 255)),
        (_, true) => (Color::new(255, 210, 60, 255), Color::new(255, 245, 190, 255)),
        (_, false) => (Color::new(110, 90, 40, 255), Color::new(160, 140, 90, 255)),
    };
    d.draw_circle(x as i32, y as i32, r + 3.0, borde);
    d.draw_circle(x as i32, y as i32, r, fondo);
    let s = r * 0.55;
    let blanco = if encendido { Color::WHITE } else { Color::new(200, 190, 170, 255) };
    // Los triangulos van en sentido antihorario para que raylib los pinte.
    let (a, b, c) = match n {
        Nota::A => {
            ui::texto(d, "A", (x - r * 0.3) as i32, (y - r * 0.5) as i32, r as i32, blanco);
            return;
        }
        Nota::Arriba => (Vector2::new(x, y - s), Vector2::new(x - s, y + s * 0.7), Vector2::new(x + s, y + s * 0.7)),
        Nota::Abajo => (Vector2::new(x, y + s), Vector2::new(x + s, y - s * 0.7), Vector2::new(x - s, y - s * 0.7)),
        Nota::Izquierda => (Vector2::new(x - s, y), Vector2::new(x + s * 0.7, y + s), Vector2::new(x + s * 0.7, y - s)),
        Nota::Derecha => (Vector2::new(x + s, y), Vector2::new(x - s * 0.7, y - s), Vector2::new(x - s * 0.7, y + s)),
    };
    d.draw_triangle(a, b, c, blanco);
    d.draw_triangle(a, c, b, blanco);
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn tocar(f: &mut Fuente, n: Nota) {
        let e = match n {
            Nota::A => Entrada { aceptar: true, ..Default::default() },
            Nota::Arriba => Entrada { arriba: true, ..Default::default() },
            Nota::Abajo => Entrada { abajo: true, ..Default::default() },
            Nota::Izquierda => Entrada { izquierda: true, ..Default::default() },
            Nota::Derecha => Entrada { derecha: true, ..Default::default() },
        };
        f.actualizar(&e, 0.5);
    }

    fn hasta_tocar() -> Fuente {
        let mut f = Fuente::new();
        let aceptar = Entrada { aceptar: true, ..Default::default() };
        for _ in 0..20 {
            if f.fase == Fase::Tocar {
                break;
            }
            f.actualizar(&aceptar, 0.5);
        }
        assert_eq!(f.fase, Fase::Tocar);
        f
    }

    /// Con la cancion completa sale la Gran Hada, con la Trifuerza encendida, y al final se obtiene el corazon.
    #[test]
    fn la_cancion_llama_a_la_gran_hada() {
        let mut f = hasta_tocar();
        for (k, n) in CANCION.iter().enumerate() {
            assert_eq!(f.triangulos_encendidos(), k / 2);
            tocar(&mut f, *n);
        }
        assert!(f.hada_afuera());
        assert_eq!(f.triangulos_encendidos(), 3);
        let mut vio_corazon = false;
        for _ in 0..400 {
            f.actualizar(&Entrada::default(), 0.1);
            vio_corazon |= f.contenedor().is_some();
        }
        assert!(vio_corazon, "Miku tiene que levantar el Contenedor de Corazon");
        assert_eq!(f.fase, Fase::Libre);
    }

    /// Una nota equivocada: Navi regana y se empieza de nuevo.
    #[test]
    fn una_nota_mala_empieza_de_nuevo() {
        let mut f = hasta_tocar();
        tocar(&mut f, CANCION[0]);
        tocar(&mut f, CANCION[1]);
        tocar(&mut f, Nota::Abajo);
        assert_eq!(f.fase, Fase::Error);
        assert!(f.hablante().is_some_and(|h| h.0 == NAVI));
        f.actualizar(&Entrada::default(), 3.0);
        assert_eq!(f.fase, Fase::Tocar);
        assert_eq!(f.progreso, 0);
    }
}
