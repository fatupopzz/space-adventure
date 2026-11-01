//! Saturno Ritmo, el minijuego.
//!
//! La nave se para en la puerta de juicio, sobre el anillo de Saturno. Los
//! bichitos-nota llegan por el anillo en dos carriles: el piso (rosa) y el
//! aire (turquesa). Cada nota sale de la partitura de `cancion.rs`, asi que
//! llega a la puerta justo cuando suena. Se golpea el piso o se brinca al
//! aire cuando pasan por la puerta.
//!
//! Lo que luce del pipeline: la posicion de cada nota es solo su matriz M
//! (un angulo sobre el anillo que baja con el tiempo), el FOV late con el
//! ritmo cuando hay combo (proyeccion), y los aciertos estallan en luz
//! (blending aditivo).
//!
//! Se juega como los juegos de ritmo de verdad (Project DIVA, Taiko no
//! Tatsujin, osu!): primero se elige la dificultad; cada golpe suena aunque
//! no haya nota (el "don" del piso y el "ka" del aire); sobre la puerta hay
//! anillos que se encogen hasta el momento justo; y cada acierto dice si fue
//! temprano o tarde.
//!
//! Todo esta en las coordenadas de Saturno, como el concierto.

use crate::cancion::{self, CORCHEA, DURACION};
use crate::entrada::Entrada;
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;
use crate::sonido::Efecto;
use crate::ui;
use raylib::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

/// Radianes por segundo que avanzan las notas por el anillo.
const VELOCIDAD: f32 = 0.55;
const RADIO_CARRIL: f32 = 3.3;
const Y_PISO: f32 = 0.05;
const Y_AIRE: f32 = 0.58;
/// Cuantos segundos antes de su momento aparece el anillo de una nota.
const ANILLO: f32 = 1.1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dificultad {
    Facil,
    Normal,
    Dificil,
}

pub const DIFICULTADES: [Dificultad; 3] = [Dificultad::Facil, Dificultad::Normal, Dificultad::Dificil];

impl Dificultad {
    /// Que tan lejos (en segundos) del momento justo cuenta como perfecto o bien.
    fn ventanas(self) -> (f32, f32) {
        match self {
            Dificultad::Facil => (0.11, 0.26),
            Dificultad::Normal => (0.09, 0.22),
            Dificultad::Dificil => (0.06, 0.15),
        }
    }

    /// Lo minimo entre una nota y la siguiente (en segundos): las de la
    /// partitura que caen mas pegadas no se piden (la cancion igual las toca).
    fn separacion(self) -> f32 {
        match self {
            Dificultad::Facil => 0.95,
            Dificultad::Normal => 0.45,
            Dificultad::Dificil => 0.0,
        }
    }

    /// En facil cualquier boton sirve para cualquier carril.
    fn un_boton(self) -> bool {
        self == Dificultad::Facil
    }

    fn nombre(self) -> &'static str {
        match self {
            Dificultad::Facil => "FÁCIL",
            Dificultad::Normal => "NORMAL",
            Dificultad::Dificil => "DIFÍCIL",
        }
    }

    fn descripcion(self) -> &'static str {
        match self {
            Dificultad::Facil => "Un solo botón para los dos carriles y solo las notas fuertes. Para disfrutar la canción.",
            Dificultad::Normal => "Piso y aire, sin ráfagas de notas pegadas. El ritmo de verdad.",
            Dificultad::Dificil => "La partitura completa, con corcheas seguidas y ventanas cortas. Usted se lo buscó.",
        }
    }

    fn color(self) -> Color {
        match self {
            Dificultad::Facil => Color::new(110, 220, 140, 255),
            Dificultad::Normal => Color::new(120, 190, 255, 255),
            Dificultad::Dificil => Color::new(255, 110, 150, 255),
        }
    }

    fn frase(self) -> &'static str {
        match self {
            Dificultad::Facil => "Modo fácil. Un botón para todo. Si falla aquí, el problema no es el juego.",
            Dificultad::Normal => "Modo normal. Piso y aire. Como la vida: a veces se cae, a veces se brinca.",
            Dificultad::Dificil => "Modo difícil. La partitura completa. Yo no me hago responsable de sus dedos.",
        }
    }
}
/// La nave va un poquito despues de la puerta.
const ANGULO_NAVE: f32 = -0.14;
const ESCALA_NAVE: f32 = 0.16;

/// Las notas que se piden en una dificultad: las de la partitura, sin las que
/// caen demasiado pegadas a la anterior.
pub fn notas_del_juego(d: Dificultad) -> Vec<(f32, bool)> {
    let mut v: Vec<(f32, bool)> = Vec::new();
    for (t, aire) in cancion::notas() {
        if v.last().is_none_or(|&(antes, _)| t - antes >= d.separacion() - 1e-3) {
            v.push((t, aire));
        }
    }
    v
}

#[derive(Clone, Copy, PartialEq)]
enum Estado {
    Pendiente,
    /// Acertada en ese segundo.
    Acertada(f32),
    Fallada,
}

struct Nota {
    t: f32,
    aire: bool,
    estado: Estado,
}

#[derive(Clone, Copy, PartialEq)]
enum Juicio {
    Perfecto,
    Bien,
    Fallo,
}

pub enum Accion {
    Nada,
    OtraVez,
    Despegar,
}

pub struct Ritmo {
    /// Si ya se esta jugando (antes se elige la dificultad).
    jugando: bool,
    cursor: usize,
    pub dificultad: Dificultad,
    /// El mejor puntaje de cada dificultad en esta visita a Saturno.
    records: [u32; 3],
    /// Se pone en `true` el cuadro en que arranca la partida: ahi la cancion tiene que empezar.
    pub empezar_cancion: bool,
    /// Que tan temprano (negativo) o tarde fue el ultimo acierto.
    desvio: f32,
    t: f32,
    notas: Vec<Nota>,
    puntaje: u32,
    combo: u32,
    max_combo: u32,
    perfectos: u32,
    buenos: u32,
    fallos: u32,
    juicio: Option<(Juicio, f32)>,
    ultimo_piso: f32,
    ultimo_aire: f32,
    ultimo_fallo: f32,
    ia: &'static str,
    /// Los sonidos que hay que tocar este cuadro.
    pub eventos: Vec<Efecto>,
}

/// Donde queda un punto del carril, a cierto angulo de la puerta.
fn en_anillo(angulo: f32, radio: f32, y: f32) -> Vec3 {
    Vec3::new(radio * angulo.sin(), y, -radio * angulo.cos())
}

impl Ritmo {
    /// Empieza en la pantalla de dificultad, con NORMAL elegida.
    pub fn new() -> Self {
        Ritmo::eligiendo(Dificultad::Normal, [0; 3])
    }

    fn eligiendo(d: Dificultad, records: [u32; 3]) -> Self {
        Ritmo {
            jugando: false,
            cursor: d as usize,
            dificultad: d,
            records,
            empezar_cancion: false,
            desvio: 0.0,
            t: 0.0,
            notas: Vec::new(),
            puntaje: 0,
            combo: 0,
            max_combo: 0,
            perfectos: 0,
            buenos: 0,
            fallos: 0,
            juicio: None,
            ultimo_piso: -9.0,
            ultimo_aire: -9.0,
            ultimo_fallo: -9.0,
            ia: "Saturno Ritmo. Elija dificultad. Le recomiendo la que crea que puede, menos una.",
            eventos: Vec::new(),
        }
    }

    /// Otra partida: de vuelta a elegir, con la misma dificultad marcada y los records.
    pub fn otra_vez(&self) -> Self {
        Ritmo::eligiendo(self.dificultad, self.records)
    }

    /// Arranca la partida en la dificultad `d` (la cancion empieza desde cero).
    pub fn empezar(&mut self, d: Dificultad) {
        *self = Ritmo { jugando: true, empezar_cancion: true, ..Ritmo::eligiendo(d, self.records) };
        self.notas = notas_del_juego(d).into_iter().map(|(t, aire)| Nota { t, aire, estado: Estado::Pendiente }).collect();
        self.ia = d.frase();
    }

    pub fn jugando(&self) -> bool {
        self.jugando
    }

    /// Como termino la cancion, si ya termino: la dificultad, la nota (S, A, B, C) y los fallos.
    pub fn resultado(&self) -> Option<(Dificultad, &'static str, u32)> {
        self.termino().then(|| (self.dificultad, self.nota_final(), self.fallos))
    }

    fn termino(&self) -> bool {
        self.jugando && self.t > DURACION + 0.5
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> Accion {
        self.t += dt;
        if e.aterrizar {
            return Accion::Despegar;
        }
        if !self.jugando {
            if e.arriba {
                self.cursor = (self.cursor + 2) % 3;
            }
            if e.abajo {
                self.cursor = (self.cursor + 1) % 3;
            }
            let elegida = e.respuesta.filter(|&r| r < 3).or(if e.aceptar && self.t > 0.3 { Some(self.cursor) } else { None });
            if let Some(k) = elegida {
                self.empezar(DIFICULTADES[k]);
                self.eventos.push(Efecto::Perfecto);
            }
            return Accion::Nada;
        }
        if self.termino() {
            if e.aceptar {
                return Accion::OtraVez;
            }
            return Accion::Nada;
        }
        // Cada golpe suena, haya nota o no (asi se siente el control).
        if self.dificultad.un_boton() {
            if e.piso || e.aire {
                // Se le acredita al carril de la nota mas cercana.
                let aire = self.mas_cercana().is_some_and(|i| self.notas[i].aire);
                if aire { self.ultimo_aire = self.t } else { self.ultimo_piso = self.t }
                self.eventos.push(if aire { Efecto::Ka } else { Efecto::Don });
                self.golpear(None);
            }
        } else {
            if e.piso {
                self.ultimo_piso = self.t;
                self.eventos.push(Efecto::Don);
                self.golpear(Some(false));
            }
            if e.aire {
                self.ultimo_aire = self.t;
                self.eventos.push(Efecto::Ka);
                self.golpear(Some(true));
            }
        }
        // Las que pasaron de largo sin golpe.
        let t = self.t;
        let bien = self.dificultad.ventanas().1;
        let tarde: Vec<usize> = self.notas.iter().enumerate().filter(|(_, n)| n.estado == Estado::Pendiente && t - n.t > bien).map(|(i, _)| i).collect();
        for i in tarde {
            {
                self.notas[i].estado = Estado::Fallada;
                self.fallos += 1;
                self.combo = 0;
                self.juicio = Some((Juicio::Fallo, self.t));
                self.ultimo_fallo = self.t;
                self.eventos.push(Efecto::Fallo);
                if self.fallos == 1 {
                    self.ia = "Primera nota perdida. La música desafina, los bichitos lloran y yo tomo nota.";
                }
            }
        }
        if self.t > DURACION && self.ia_final().is_some() {
            self.ia = self.ia_final().unwrap();
            let k = self.dificultad as usize;
            self.records[k] = self.records[k].max(self.puntaje);
        }
        Accion::Nada
    }

    /// La nota pendiente mas cercana a ahora (de cualquier carril) que este en la ventana.
    fn mas_cercana(&self) -> Option<usize> {
        self.cercana_en(None)
    }

    fn cercana_en(&self, carril: Option<bool>) -> Option<usize> {
        let (t, bien) = (self.t, self.dificultad.ventanas().1);
        self.notas
            .iter()
            .enumerate()
            .filter(|(_, n)| carril.is_none_or(|a| n.aire == a) && n.estado == Estado::Pendiente && (n.t - t).abs() <= bien)
            .min_by(|a, b| (a.1.t - t).abs().total_cmp(&(b.1.t - t).abs()))
            .map(|(i, _)| i)
    }

    /// Golpe en un carril (o en cualquiera, en facil): acierta la nota
    /// pendiente mas cercana si esta en la ventana.
    fn golpear(&mut self, carril: Option<bool>) {
        let t = self.t;
        let Some(i) = self.cercana_en(carril) else { return };
        self.desvio = t - self.notas[i].t;
        let perfecto = self.desvio.abs() <= self.dificultad.ventanas().0;
        self.notas[i].estado = Estado::Acertada(t);
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        let base = if perfecto { 300 } else { 100 };
        self.puntaje += base * (1 + self.combo / 10);
        if perfecto {
            self.perfectos += 1;
        } else {
            self.buenos += 1;
        }
        self.juicio = Some((if perfecto { Juicio::Perfecto } else { Juicio::Bien }, t));
        if self.combo % 10 == 0 {
            self.eventos.push(Efecto::Perfecto);
        }
        self.ia = match self.combo {
            10 => "Combo de 10. La nave se emociona. Lo disimula, pero se le nota en las turbinas.",
            25 => "Combo de 25. Ya no parece accidente. Eso me preocupa.",
            50 => "Combo de 50. Anotado en la bitácora como milagro, junto a pasar Física 2.",
            _ => self.ia,
        };
    }

    fn precision(&self) -> f32 {
        let total = self.notas.len().max(1) as f32;
        (self.perfectos as f32 + 0.8 * self.buenos as f32) / total
    }

    fn nota_final(&self) -> &'static str {
        match self.precision() {
            p if p >= 0.9 => "S",
            p if p >= 0.75 => "A",
            p if p >= 0.55 => "B",
            _ => "C",
        }
    }

    fn ia_final(&self) -> Option<&'static str> {
        Some(match self.nota_final() {
            "S" => "Nota S. Saturno pide otra. Yo pido el replay para revisar si hubo trampa.",
            "A" => "Nota A. Muy bien. El vinil queda satisfecho. Yo, moderadamente.",
            "B" => "Nota B. Aceptable. Los bichitos sobrevivieron, con traumas.",
            _ => "Nota C. El anillo no juzga. Yo sí. Bastante.",
        })
    }

    pub fn ia(&self) -> &str {
        self.ia
    }

    /// El golpe del ritmo: 1 en el beat y se apaga hasta el siguiente.
    pub fn golpe(&self) -> f32 {
        (-(self.t / (2.0 * CORCHEA)).fract() * 5.0).exp()
    }

    pub fn t(&self) -> f32 {
        self.t
    }

    /// Grados que el FOV late con el ritmo, mas mientras mas combo.
    pub fn fov_extra(&self) -> f32 {
        5.0 * self.golpe() * (self.combo as f32 / 10.0).min(1.0)
    }

    pub fn expresion_miku(&self) -> &'static str {
        if self.t - self.ultimo_fallo < 0.6 {
            "asustada"
        } else if self.combo >= 20 {
            "enamorada"
        } else {
            "feliz"
        }
    }

    /// Las notas que se ven: su matriz y si son del aire.
    pub fn notas_visibles(&self) -> Vec<(Mat4, bool)> {
        self.notas
            .iter()
            .enumerate()
            .filter_map(|(k, n)| {
                let angulo = (n.t - self.t) * VELOCIDAD;
                let crecer = match n.estado {
                    Estado::Pendiente if (-0.4..2.6).contains(&angulo) => 1.0,
                    // Al acertarla se infla y desaparece.
                    Estado::Acertada(t) if self.t - t < 0.18 => 1.0 + 3.0 * (self.t - t),
                    _ => return None,
                };
                let angulo = if let Estado::Acertada(_) = n.estado { 0.0 } else { angulo };
                let y = if n.aire { Y_AIRE + 0.05 * (self.t * 6.0 + k as f32).sin() } else { Y_PISO };
                let brinco = if n.aire { 0.0 } else { 0.04 * (self.t * 10.0 + k as f32).sin().abs() };
                let m = Mat4::translation(en_anillo(angulo, RADIO_CARRIL, y + brinco))
                    * Mat4::rotation_y(FRAC_PI_2 - angulo)
                    * Mat4::scale(0.85 * crecer);
                Some((m, n.aire))
            })
            .collect()
    }

    /// La nave en la puerta, mirando hacia donde vienen las notas. Brinca con
    /// el aire y se agacha con el piso.
    pub fn nave(&self) -> Mat4 {
        let brinco = {
            let k = (self.t - self.ultimo_aire) / 0.35;
            if (0.0..1.0).contains(&k) { 0.55 * (k * PI).sin() } else { 0.0 }
        };
        let agacho = if self.t - self.ultimo_piso < 0.12 { -0.04 } else { 0.0 };
        let pos = en_anillo(ANGULO_NAVE, RADIO_CARRIL, Y_PISO + 0.33 + brinco + agacho);
        let (s, c) = ANGULO_NAVE.sin_cos();
        let yaw = f32::atan2(-c, -s);
        Mat4::translation(pos) * Mat4::rotation_y(yaw) * Mat4::scale(ESCALA_NAVE)
    }

    pub fn camara(&self) -> (Vec3, Vec3) {
        let vaiven = 0.03 * (self.t * 0.7).sin();
        // Un poco afuera del anillo y arriba, viendo el carril de frente (la nave no tapa la puerta): el planeta queda a un lado.
        (en_anillo(-0.55 + vaiven, RADIO_CARRIL + 1.0, 1.85), en_anillo(0.42, RADIO_CARRIL + 0.1, 0.2))
    }

    /// Los aciertos estallan en lucecitas en la puerta. Y sobre la puerta, en
    /// cada carril, el blanco (un circulo fijo que se ilumina al golpear) y
    /// los anillos de las notas que vienen, que se encogen hasta tocar el
    /// blanco justo en su momento (como en osu!).
    pub fn dibujar_efectos(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4) {
        if self.jugando && !self.termino() {
            let circulo = |fb: &mut Framebuffer, centro: Vec3, radio: f32, color: [f32; 3]| {
                for i in 0..32 {
                    let (a0, a1) = (i as f32 * PI / 16.0, (i + 1) as f32 * PI / 16.0);
                    let punto = |a: f32| centro + Vec3::new(0.0, a.cos() * radio, a.sin() * radio);
                    if let (Some(p0), Some(p1)) = (proyectar(punto(a0), vista_proyeccion, fb), proyectar(punto(a1), vista_proyeccion, fb)) {
                        fb.linea_de_luz(p0, p1, color, color);
                    }
                }
            };
            for aire in [false, true] {
                let centro = en_anillo(0.0, RADIO_CARRIL, if aire { Y_AIRE + 0.15 } else { Y_PISO + 0.15 });
                let color = if aire { [0.35, 0.95, 1.0] } else { [1.0, 0.45, 0.8] };
                let golpe = self.t - if aire { self.ultimo_aire } else { self.ultimo_piso };
                let brillo = if golpe < 0.12 { 1.6 } else { 0.6 };
                circulo(fb, centro, 0.2, color.map(|c| c * brillo));
                circulo(fb, centro, 0.215, color.map(|c| c * brillo));
                for n in self.notas.iter().filter(|n| n.aire == aire && n.estado == Estado::Pendiente) {
                    let falta = n.t - self.t;
                    if (0.0..ANILLO).contains(&falta) {
                        let k = falta / ANILLO;
                        let luz = (1.0 - k) * 1.4;
                        circulo(fb, centro, 0.2 + 0.7 * k, color.map(|c| c * luz));
                        circulo(fb, centro, 0.215 + 0.7 * k, color.map(|c| c * luz * 0.6));
                    }
                }
            }
        }
        for n in &self.notas {
            let Estado::Acertada(t) = n.estado else { continue };
            let edad = self.t - t;
            if edad > 0.5 {
                continue;
            }
            let centro = en_anillo(0.0, RADIO_CARRIL, if n.aire { Y_AIRE + 0.15 } else { Y_PISO + 0.15 });
            let color = if n.aire { [0.35, 0.95, 1.0] } else { [1.0, 0.45, 0.8] };
            let luz = 1.0 - edad / 0.5;
            let radio = 0.15 + 0.6 * edad;
            for i in 0..20 {
                let a = i as f32 * 0.314;
                let dir = Vec3::new(a.cos(), a.sin(), 0.4 * (a * 3.0).sin()).normalize();
                if let (Some(p0), Some(p1)) = (proyectar(centro + dir * (radio * 0.6), vista_proyeccion, fb), proyectar(centro + dir * radio, vista_proyeccion, fb)) {
                    fb.linea_de_luz(p0, p1, color.map(|c| c * luz * 0.3), color.map(|c| c * luz * 1.3));
                }
            }
        }
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, ia: &str, ancho: i32, alto: i32) {
        let (piso, aire, aceptar, salir) = (&ui::boton(e.nombre("piso")), &ui::boton(e.nombre("aire")), &ui::boton(e.nombre("aceptar")), &ui::boton(e.nombre("aterrizar")));
        if !self.jugando {
            self.dibujar_dificultades(d, aceptar, salir, ancho, alto);
            if !ia.is_empty() {
                ui::mensaje_ia(d, ia, ancho, alto);
            }
            return;
        }
        // Puntaje y combo.
        ui::panel(d, ancho - 290, 20, 270, 92);
        ui::texto(d, &format!("{:07}", self.puntaje), ancho - 272, 30, 40, ui::TEXTO);
        ui::texto(d, &format!("combo {}", self.combo), ancho - 272, 76, 24, if self.combo >= 10 { ui::LILA } else { ui::TENUE });
        ui::panel(d, 20, 20, 520, 60);
        // Con control, los gatillos de arriba: R1 (o RB) arriba y L1 (o LB) abajo, como se ven
        // las notas en la pista. En el teclado, las flechas (o J y F).
        let (arriba, abajo) = if e.hay_control {
            (ui::boton(e.nombre("pestana_der")), ui::boton(e.nombre("pestana_izq")))
        } else {
            (format!("{} o {aire}", ui::boton(ui::ARRIBA)), format!("{} o {piso}", ui::boton(ui::ABAJO)))
        };
        let controles = if self.dificultad.un_boton() {
            format!("{arriba} o {abajo}: cualquier nota      {salir}: salir")
        } else {
            format!("{arriba}: arriba      {abajo}: abajo      {salir}: salir")
        };
        ui::texto(d, &controles, 36, 36, 24, ui::TEXTO);
        ui::texto(d, self.dificultad.nombre(), ancho - 272, 120, 22, self.dificultad.color());

        // Cuenta regresiva en el segundo compas (la intro de la cancion).
        let tiempo = (self.t / (2.0 * CORCHEA)) as i32;
        let cuenta = match tiempo {
            4 => Some("3"),
            5 => Some("2"),
            6 => Some("1"),
            7 => Some("¡YA!"),
            _ => None,
        };
        if let Some(c) = cuenta {
            let tam = 120;
            let w = ui::medir(c, tam);
            ui::texto(d, c, (ancho - w) / 2 + 4, alto / 2 - 96, tam, Color::new(40, 10, 60, 255));
            ui::texto(d, c, (ancho - w) / 2, alto / 2 - 100, tam, Color::new(255, 140, 210, 255));
        }

        // El juicio del ultimo golpe, que sube y se desvanece.
        if let Some((j, t0)) = self.juicio {
            let edad = self.t - t0;
            if edad < 0.6 {
                let (texto, color) = match j {
                    Juicio::Perfecto => ("¡PERFECTO!", Color::new(255, 220, 90, 255)),
                    Juicio::Bien => ("BIEN", ui::VERDE),
                    Juicio::Fallo => ("FALLO", ui::ROJO),
                };
                let tam = 56 + (16.0 * (1.0 - edad / 0.6)) as i32;
                let a = ((1.0 - edad / 0.6) * 255.0) as u8;
                let w = ui::medir_titulo(texto, tam);
                let y = alto / 2 - 40 - (edad * 60.0) as i32;
                ui::titulo_de(d, texto, (ancho - w) / 2, y, tam, Color::new(color.r, color.g, color.b, a));
                // Temprano o tarde, para corregir (solo si no fue perfecto).
                if j == Juicio::Bien {
                    let lado = if self.desvio < 0.0 { "TEMPRANO" } else { "TARDE" };
                    let w = ui::medir(lado, 26);
                    ui::texto(d, lado, (ancho - w) / 2, y + tam + 4, 26, Color::new(200, 200, 255, a));
                }
            }
        }

        if self.termino() {
            let (w, h) = (680, 370);
            let (x, y) = ((ancho - w) / 2, (alto - h) / 2 - 30);
            ui::panel(d, x, y, w, h);
            ui::titulo_de(d, "SATURNO RITMO", x + 28, y + 24, 36, Color::new(255, 190, 235, 255));
            // La nota, sobre su insignia (S: con alas; A: azul; B: verde; C: naranja).
            let nota = self.nota_final();
            let insignia = match nota {
                "S" => ui::Insignia::AlasAzul,
                "A" => ui::Insignia::GemaAzul,
                "B" => ui::Insignia::GemaVerde,
                _ => ui::Insignia::GemaNaranja,
            };
            let (cx, cy) = ((x + w - 92) as f32, (y + 74) as f32);
            ui::insignia(d, insignia, cx, cy, if nota == "S" { 2.4 } else { 3.2 }, Color::WHITE);
            ui::titulo_de(d, nota, cx as i32 - ui::medir_titulo(nota, 64) / 2, cy as i32 - 34, 64, Color::new(255, 245, 250, 255));
            let lineas = [
                format!("puntaje: {}", self.puntaje),
                format!("combo maximo: {}", self.max_combo),
                format!("perfectos: {}   bien: {}   fallos: {}", self.perfectos, self.buenos, self.fallos),
                format!("precision: {:.0}%   ({})   récord: {}", self.precision() * 100.0, self.dificultad.nombre(), self.records[self.dificultad as usize]),
            ];
            for (k, l) in lineas.iter().enumerate() {
                ui::texto(d, l, x + 28, y + 90 + k as i32 * 40, 28, ui::TEXTO);
            }
            ui::texto(d, &format!("{aceptar}: otra vez (o cambiar dificultad)"), x + 28, y + h - 78, 24, ui::LILA);
            ui::texto(d, &format!("{salir}: despegar"), x + 28, y + h - 46, 24, ui::LILA);
        }
        if !ia.is_empty() {
            ui::mensaje_ia(d, ia, ancho, alto);
        }
    }
}

impl Ritmo {
    /// La pantalla de dificultad: tres tarjetas, con sus pelotitas de dificultad y su record.
    fn dibujar_dificultades(&self, d: &mut RaylibDrawHandle, aceptar: &str, salir: &str, ancho: i32, alto: i32) {
        let (w, h) = (640, 420);
        let (x, y) = ((ancho - w) / 2, (alto - h) / 2 - 50);
        ui::panel(d, x, y, w, h);
        ui::texto(d, "SATURNO RITMO", x + 28, y + 18, 44, Color::new(255, 140, 210, 255));
        ui::texto(d, "elige la dificultad", x + 30, y + 66, 22, ui::TENUE);
        for (k, dif) in DIFICULTADES.iter().enumerate() {
            let fila = y + 104 + k as i32 * 62;
            let sel = k == self.cursor;
            if sel {
                d.draw_rectangle(x + 18, fila - 6, w - 36, 54, Color::new(255, 255, 255, 28));
                d.draw_rectangle(x + 18, fila - 6, 6, 54, dif.color());
            }
            ui::texto(d, &format!("{}) {}", k + 1, dif.nombre()), x + 40, fila, 32, if sel { dif.color() } else { ui::TEXTO });
            for p in 0..3i32 {
                let lleno = p <= k as i32;
                let color = if lleno { dif.color() } else { Color::new(80, 70, 100, 255) };
                d.draw_circle(x + 300 + p * 26, fila + 18, 9.0, color);
            }
            let record = self.records[k];
            if record > 0 {
                ui::texto(d, &format!("récord {record}"), x + 420, fila + 6, 22, ui::TENUE);
            }
        }
        let desc = DIFICULTADES[self.cursor].descripcion();
        ui::parrafo(d, desc, x + 30, y + 300, 22, w - 60, ui::TEXTO);
        ui::texto(d, &format!("{} o {}: elegir     {aceptar}: ¡a jugar!     {salir}: salir", ui::boton("\u{E008} \u{E009}"), ui::boton("1 a 3")), x + 30, y + h - 40, 20, ui::LILA);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Mas dificil, mas notas; y en cada dificultad vienen tan separadas como dice.
    #[test]
    fn cada_dificultad_tiene_sus_notas() {
        let n: Vec<usize> = DIFICULTADES.iter().map(|&d| notas_del_juego(d).len()).collect();
        assert!(n[0] < n[1] && n[1] < n[2], "{n:?}");
        assert!(n[0] >= 20, "facil se quedo sin notas: {n:?}");
        assert_eq!(n[2], cancion::notas().len());
        for d in DIFICULTADES {
            for par in notas_del_juego(d).windows(2) {
                assert!(par[1].0 - par[0].0 >= d.separacion() - 1e-3);
            }
        }
    }

    /// En facil, apretar cualquier boton acierta cualquier carril.
    #[test]
    fn en_facil_un_boton_sirve_para_todo() {
        let mut r = Ritmo::new();
        r.empezar(Dificultad::Facil);
        let notas = notas_del_juego(Dificultad::Facil);
        let paso = 1.0 / 240.0;
        let mut ahora = 0.0f32;
        while ahora < DURACION + 1.0 {
            let toca = notas.iter().any(|&(tn, _)| tn > ahora && tn <= ahora + paso);
            r.actualizar(&Entrada { piso: toca, ..Default::default() }, paso);
            ahora += paso;
        }
        assert_eq!(r.fallos, 0);
        assert_eq!(r.nota_final(), "S");
    }

    #[test]
    fn primero_se_elige_la_dificultad() {
        let mut r = Ritmo::new();
        assert!(!r.jugando());
        r.actualizar(&Entrada { abajo: true, ..Default::default() }, 0.1);
        r.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.3);
        assert!(r.jugando());
        assert_eq!(r.dificultad, Dificultad::Dificil);
        assert!(r.empezar_cancion);
    }
}
