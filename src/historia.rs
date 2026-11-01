//! La historia del universo, en cinematica (se ve desde el Mirador o el menu
//! principal). Diecisiete capitulos narrados por GLADAS, del Big Bang al
//! final del universo (entre ellos, los que hacen falta para llegar aqui):
//!
//! 1. El Big Bang: un punto, un destello y una esfera de particulas que se
//!    expande y se enfria (blanco, naranja, rojo).
//! 2. Las primeras estrellas: gas oscuro, y las estrellas prendiendose una
//!    por una.
//! 3. Las galaxias: una espiral de dos brazos girando.
//! 4. El Sol: un disco de gas que cae en espiral al centro y se prende.
//! 5. Los planetas: el polvo del disco se junta y van apareciendo.
//! 6. La Luna: Theia (del tamano de Marte) choca con la Tierra; los pedazos
//!    forman un anillo y de ahi sale la Luna.
//! 7. La vida, y el asteroide de los dinosaurios.
//! 8. 1969: la llegada a la Luna (la de verdad).
//! 9. 2007: nace Hatsune Miku.
//! 10. Hoy: la Nave Lila.
//!
//! Y los de en medio y del futuro: la primera luz (el fondo cosmico de
//! microondas), las supernovas, los agujeros negros, el cielo que se vuelve
//! azul (las cianobacterias), Andromeda chocando con la Via Lactea, el Sol
//! hecho gigante roja y el final (las estrellas apagandose una por una).
//!
//! Todo con particulas (lineas de luz con z-buffer) y los modelos del juego.
//! ESPACIO pasa al siguiente capitulo; E se sale.

use crate::entrada::Entrada;
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;
use std::f32::consts::{PI, TAU};

/// Que pasa en un capitulo (cada uno tiene su camara, sus modelos y sus particulas).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tipo {
    BigBang,
    Luz,
    Estrellas,
    Galaxias,
    Supernova,
    AgujeroNegro,
    Sol,
    Planetas,
    Luna,
    CieloAzul,
    Vida,
    Apolo,
    Miku,
    Hoy,
    Andromeda,
    GiganteRoja,
    Final,
}

pub struct Capitulo {
    pub tipo: Tipo,
    pub titulo: &'static str,
    pub fecha: &'static str,
    pub texto: &'static str,
    pub dura: f32,
}

const fn cap(tipo: Tipo, titulo: &'static str, fecha: &'static str, texto: &'static str, dura: f32) -> Capitulo {
    Capitulo { tipo, titulo, fecha, texto, dura }
}

pub const CAPITULOS: [Capitulo; 17] = [
    cap(Tipo::BigBang, "EL BIG BANG", "hace 13,800 millones de años", "Al principio no había nada. Luego, todo, de golpe. Nadie sabe quién apretó el botón. Yo sospecho de Plutón.", 11.0),
    cap(
        Tipo::Luz,
        "LA PRIMERA LUZ",
        "380 mil años después",
        "Al principio todo era una sopa tan caliente que la luz no podía pasar. Cuando se enfrió, la luz salió por fin. Todavía nos llega: es el fondo cósmico de microondas. Un poquito de la estática de las teles viejas era eso.",
        13.0,
    ),
    cap(
        Tipo::Estrellas,
        "LAS PRIMERAS ESTRELLAS",
        "hace 13,600 millones de años",
        "Durante millones de años todo estuvo oscuro. Luego se encendieron las primeras estrellas, una por una. Como las luces de las casas a las seis de la tarde.",
        11.0,
    ),
    cap(
        Tipo::Galaxias,
        "LAS GALAXIAS",
        "hace 13,000 millones de años",
        "Las estrellas se juntaron en galaxias. La nuestra, la Vía Láctea, tiene unos 200 mil millones. Y ninguna contesta los mensajes.",
        12.0,
    ),
    cap(
        Tipo::Supernova,
        "LAS SUPERNOVAS",
        "desde entonces, todo el tiempo",
        "Las estrellas grandes viven rápido y mueren explotando: supernovas. Ahí se hicieron el hierro, el calcio, el oro. El hierro de su sangre viene de una estrella que explotó. Usted es reciclaje cósmico.",
        12.0,
    ),
    cap(
        Tipo::AgujeroNegro,
        "LOS AGUJEROS NEGROS",
        "en el centro de casi todas las galaxias",
        "Algunas estrellas, al morir, se encogen tanto que ni la luz se escapa: agujeros negros. En el centro de la Vía Láctea hay uno de cuatro millones de soles: Sagitario A*. Tampoco contesta mensajes.",
        12.0,
    ),
    cap(
        Tipo::Sol,
        "NACE EL SOL",
        "hace 4,600 millones de años",
        "En una orilla de la galaxia, una nube de gas se encogió, se calentó y se prendió: el Sol. Desde entonces no se ha apagado. Ni para dormir.",
        11.0,
    ),
    cap(
        Tipo::Planetas,
        "LOS PLANETAS",
        "hace 4,500 millones de años",
        "Con el polvo que sobró se hicieron los planetas. A Saturno le tocaron los anillos. A Urano, el ángulo. A Plutón, el drama.",
        12.0,
    ),
    cap(
        Tipo::Luna,
        "LA LUNA",
        "hace 4,500 millones de años",
        "Un planeta del tamaño de Marte, Theia, chocó con la Tierra. De los pedazos salió la Luna. El primer choque registrado. Sin seguro.",
        11.0,
    ),
    cap(
        Tipo::CieloAzul,
        "EL CIELO SE VUELVE AZUL",
        "hace 2,400 millones de años",
        "Unas bacterias, las cianobacterias, aprendieron a comer luz y a soltar oxígeno. Llenaron el aire y el cielo se volvió azul. Fue la primera contaminación de la historia. Y gracias a ella respiramos.",
        12.0,
    ),
    cap(
        Tipo::Vida,
        "LA VIDA",
        "hace 540 millones de años",
        "Luego vinieron los peces, las plantas, los dinosaurios... y hace 66 millones de años, un asteroide. Los dinosaurios no tenían programa espacial.",
        12.0,
    ),
    cap(Tipo::Apolo, "LA LUNA, DE VERDAD", "20 de julio de 1969", "Los humanos llegaron a la Luna. De verdad. Ya lo hablamos con el director.", 9.0),
    cap(
        Tipo::Miku,
        "NACE MIKU",
        "31 de agosto de 2007",
        "Y en 2007, en Japón, nació una voz: Hatsune Miku. El universo tardó 13,800 millones de años en llegar a esta parte. Valió la pena.",
        11.0,
    ),
    cap(Tipo::Hoy, "HOY", "", "Y hoy usted maneja una nave lila por un sistema solar de juguete, recogiendo notas. ¿Y después? Ahí le va el futuro.", 10.0),
    cap(
        Tipo::Andromeda,
        "ANDRÓMEDA",
        "dentro de 4,500 millones de años",
        "La galaxia de Andrómeda viene hacia nosotros a 110 kilómetros por segundo. Las dos galaxias se van a fundir en una. Las estrellas casi ni van a chocar: hay muchísimo espacio entre ellas. Como el parqueo de la U un domingo.",
        13.0,
    ),
    cap(
        Tipo::GiganteRoja,
        "EL SOL SE HINCHA",
        "dentro de 5,000 millones de años",
        "Al Sol se le va a acabar el hidrógeno y se va a hinchar: una gigante roja. Se va a tragar a Mercurio y a Venus, y tal vez a la Tierra. Recomendación: para entonces, ya haber despegado.",
        12.0,
    ),
    cap(
        Tipo::Final,
        "EL FINAL",
        "dentro de muchísimo tiempo",
        "Dentro de billones de billones de años, las estrellas se van a apagar una por una, y hasta los agujeros negros se van a evaporar. Pero falta tanto que le da tiempo de recoger todas las notas. Y de volver a empezar.",
        13.0,
    ),
];

/// Segundos de la tarjeta de FIN.
const FIN: f32 = 4.0;

/// Lo que se dibuja con un modelo del juego.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cosa {
    /// Un cuerpo de `CUERPOS` (con su modelo ligero).
    Planeta(usize),
    Miku,
    Nave,
    Bandera,
    Astronauta,
}

pub enum AccionHistoria {
    Nada,
    Terminar,
}

pub struct Historia {
    /// Si se abrio desde el menu principal (al terminar regresa ahi, no al Mirador).
    pub desde_menu: bool,
    pub capitulo: usize,
    /// Segundos dentro del capitulo (o de la tarjeta de FIN).
    pub tc: f32,
    pub total: f32,
    pub terminada: bool,
    /// Si se vio completa (hasta el FIN, sin salirse).
    pub completa: bool,
    pub eventos: Vec<Efecto>,
}

/// Un numero "al azar" en [0, 1) que siempre sale igual para el mismo `i`.
fn h(i: u32) -> f32 {
    let x = i.wrapping_mul(2_654_435_761) ^ 0x9E37_79B9;
    let x = x.wrapping_mul(1_597_334_677) ^ (x >> 15);
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// Una direccion al azar (uniforme en la esfera) para la particula `i`.
fn direccion(i: u32) -> Vec3 {
    let (a, z) = (h(i * 3) * TAU, h(i * 3 + 1) * 2.0 - 1.0);
    let r = (1.0 - z * z).max(0.0).sqrt();
    Vec3::new(r * a.cos(), z, r * a.sin())
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn mezcla(a: [f32; 3], b: [f32; 3], k: f32) -> [f32; 3] {
    [0, 1, 2].map(|q| a[q] + (b[q] - a[q]) * k.clamp(0.0, 1.0))
}

/// Un puntito de luz (una crucecita de `t` pixeles) en `p`.
fn punto(fb: &mut Framebuffer, vp: &Mat4, p: Vec3, t: f32, c: [f32; 3]) {
    if let Some((x, y, z)) = proyectar(p, vp, fb) {
        fb.linea_de_luz((x - t, y, z), (x + t, y, z), c, c);
        if t > 0.6 {
            fb.linea_de_luz((x, y - t, z), (x, y + t, z), c, c);
        }
    }
}

/// Los radios de las orbitas (de juguete) y los tamanos de los planetas en el capitulo 5.
const PLANETAS: [(usize, f32, f32); 9] =
    [(1, 3.0, 0.25), (2, 4.2, 0.38), (3, 5.6, 0.42), (5, 7.0, 0.32), (6, 9.5, 0.95), (7, 12.0, 0.75), (8, 14.0, 0.6), (9, 15.8, 0.58), (10, 17.4, 0.2)];

impl Historia {
    pub fn new() -> Self {
        Historia { desde_menu: false, capitulo: 0, tc: 0.0, total: 0.0, terminada: false, completa: false, eventos: vec![Efecto::Fuego] }
    }

    fn en_fin(&self) -> bool {
        self.capitulo >= CAPITULOS.len()
    }

    /// Que capitulo es (en la tarjeta de FIN, el ultimo).
    pub fn tipo(&self) -> Tipo {
        CAPITULOS[self.capitulo.min(CAPITULOS.len() - 1)].tipo
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionHistoria {
        self.tc += dt;
        self.total += dt;
        if e.aterrizar {
            self.terminada = true;
            return AccionHistoria::Terminar;
        }
        if self.en_fin() {
            if self.tc > FIN || (e.aceptar && self.tc > 0.5) {
                self.terminada = true;
                self.completa = true;
                return AccionHistoria::Terminar;
            }
            return AccionHistoria::Nada;
        }
        // <- ->: capitulo anterior o siguiente.
        if e.izquierda && self.capitulo > 0 {
            self.capitulo -= 1;
            self.tc = 0.0;
            return AccionHistoria::Nada;
        }
        if e.derecha {
            self.capitulo += 1;
            self.tc = 0.0;
            return AccionHistoria::Nada;
        }
        let antes = self.tc - dt;
        // Los golpes: el Big Bang, el choque de Theia, el asteroide y Miku.
        let golpe = |t0: f32| antes < t0 && self.tc >= t0;
        match self.tipo() {
            Tipo::Luna if golpe(4.0) => self.eventos.push(Efecto::Choque),
            Tipo::Supernova if golpe(4.0) => self.eventos.push(Efecto::Fuego),
            Tipo::Andromeda if golpe(8.0) => self.eventos.push(Efecto::Aterrizaje),
            Tipo::Vida if golpe(8.3) => self.eventos.push(Efecto::Fuego),
            Tipo::Miku if golpe(1.0) => self.eventos.push(Efecto::Perfecto),
            _ => {}
        }
        if self.tc > CAPITULOS[self.capitulo].dura + if self.intervencion().is_some() { 4.5 } else { 0.0 } || (e.aceptar && self.tc > 0.4) {
            self.capitulo += 1;
            self.tc = 0.0;
        }
        AccionHistoria::Nada
    }

    /// Lo que narra GLADAS ahorita.
    pub fn texto(&self) -> &'static str {
        let Some(c) = CAPITULOS.get(self.capitulo) else { return "" };
        if c.tipo == Tipo::Miku && ui::protagonista().id != "miku" {
            ui::protagonista().texto
        } else { c.texto }
    }

    /// Tres intervenciones en momentos clave, sin repetir la misma replica en cada capitulo.
    fn intervencion(&self) -> Option<crate::personajes::Replica> {
        use crate::personajes::Replica;
        match self.tipo() {
            Tipo::BigBang => Some(Replica::Cosmos),
            Tipo::Vida => Some(Replica::Vida),
            Tipo::Final => Some(Replica::Futuro),
            _ => None,
        }
    }

    pub fn dialogo(&self) -> (&'static str, &'static str, Voz) {
        let Some(c) = CAPITULOS.get(self.capitulo) else { return (ui::NOMBRE_IA, "", Voz::Ia) };
        if let Some(situacion) = self.intervencion().filter(|_| self.tc >= c.dura) {
            let p = ui::protagonista();
            (p.nombre, crate::personajes::replica(p.id, situacion), Voz::Miku)
        } else {
            (ui::NOMBRE_IA, self.texto(), Voz::Ia)
        }
    }

    /// Que tan visibles son las estrellas del fondo (antes de las primeras estrellas, nada).
    pub fn estrellas(&self) -> f32 {
        match self.tipo() {
            Tipo::BigBang | Tipo::Luz | Tipo::Estrellas => 0.0,
            Tipo::Galaxias | Tipo::Andromeda => 0.4,
            Tipo::Supernova | Tipo::AgujeroNegro => 0.6,
            Tipo::Final => (1.0 - self.tc / 6.0).max(0.0),
            Tipo::Miku => 0.2,
            _ => 1.0,
        }
    }

    /// El destello blanco (el Big Bang y el choque de Theia).
    pub fn destello(&self) -> f32 {
        let pico = |t0: f32, largo: f32, fuerza: f32| if self.tc >= t0 { (1.0 - (self.tc - t0) / largo).max(0.0) * fuerza } else { 0.0 };
        match self.tipo() {
            Tipo::BigBang => pico(0.6, 0.9, 1.0),
            Tipo::Supernova => pico(4.0, 1.0, 0.9),
            Tipo::Luna => pico(4.0, 0.8, 0.8),
            Tipo::Vida => pico(8.3, 0.6, 0.4),
            Tipo::Miku => pico(1.0, 0.6, 0.5),
            _ => 0.0,
        }
    }

    /// La camara de cada capitulo. (ojo, mira)
    pub fn camara(&self) -> (Vec3, Vec3) {
        let t = self.tc;
        let u = if self.en_fin() { 0.0 } else { t / CAPITULOS[self.capitulo].dura };
        match self.tipo() {
            Tipo::BigBang => (Vec3::new(0.0, 0.0, -10.0 - 18.0 * suave(u)), Vec3::zero()),
            Tipo::Luz => {
                let a = -PI / 2.0 + 0.6 * u;
                (Vec3::new(20.0 * a.cos(), 3.0, 20.0 * a.sin()), Vec3::zero())
            }
            Tipo::Supernova => (Vec3::new(1.5 * u, 0.5, -16.0 - 6.0 * suave((t - 4.0) / 6.0)), Vec3::zero()),
            Tipo::AgujeroNegro => {
                let a = -PI / 2.0 + 0.25 * u;
                (Vec3::new(9.5 * a.cos(), 1.6, 9.5 * a.sin()), Vec3::zero())
            }
            Tipo::CieloAzul => (Vec3::new(-1.5 + 2.0 * u, 1.0, -8.5), Vec3::zero()),
            Tipo::Andromeda => {
                let a = -PI / 2.0 + 0.3 * u;
                (Vec3::new(26.0 * a.cos(), 10.0, 26.0 * a.sin()), Vec3::zero())
            }
            Tipo::GiganteRoja => (Vec3::new(0.0, 10.0 - 2.0 * u, -24.0 + 2.0 * u), Vec3::zero()),
            Tipo::Final => (Vec3::new(0.0, 0.0, -20.0 - 6.0 * u), Vec3::zero()),
            Tipo::Estrellas => (Vec3::new(0.0, 2.0 - u, -25.0 + 7.0 * suave(u)), Vec3::zero()),
            Tipo::Galaxias => {
                let a = 0.3 + t * 0.06;
                (Vec3::new(22.0 * a.cos(), 9.0 - 2.0 * u, 22.0 * a.sin()), Vec3::zero())
            }
            Tipo::Sol => (Vec3::new(0.0, 6.0 - 3.0 * suave(u), -14.0 + 4.0 * suave(u)), Vec3::zero()),
            Tipo::Planetas => {
                let a = -PI / 2.0 + 0.4 * u;
                (Vec3::new(24.0 * a.cos(), 13.0 - 3.0 * u, 24.0 * a.sin()), Vec3::zero())
            }
            Tipo::Luna => (Vec3::new(-2.0 * u, 3.0, -13.0 + 2.0 * u), Vec3::new(-1.0, 0.0, 0.0)),
            Tipo::Vida => (Vec3::new(2.0 * u, 1.0 + 0.5 * u, -8.0 + u), Vec3::zero()),
            Tipo::Apolo => {
                let a = -PI / 2.0 - 0.4 + 0.5 * u;
                (Vec3::new(4.2 * a.cos(), 4.1, 4.2 * a.sin()), Vec3::new(0.0, 3.2, 0.0))
            }
            Tipo::Miku => {
                let a = -PI / 2.0 + 0.3 * (t * 0.3).sin();
                (Vec3::new(3.0 * a.cos(), 1.0, 3.0 * a.sin()), Vec3::new(0.0, 0.75, 0.0))
            }
            Tipo::Hoy => {
                let x = self.nave_x();
                (Vec3::new(x - 2.0, 1.4, -5.5), Vec3::new(x + 1.0, 0.4, 0.0))
            }
        }
    }

    fn nave_x(&self) -> f32 {
        -6.0 + self.tc * 1.3
    }

    /// Lo que se dibuja con modelos: (que, matriz, expresion).
    pub fn cosas(&self) -> Vec<(Cosa, Mat4, Option<&'static str>)> {
        let t = self.tc;
        let mut v = Vec::new();
        let crece = |t0: f32, dura: f32| suave((t - t0) / dura);
        match self.tipo() {
            Tipo::CieloAzul => {
                v.push((Cosa::Planeta(3), Mat4::rotation_z(0.41) * Mat4::rotation_y(t * 0.2) * Mat4::scale(2.6), None));
            }
            Tipo::GiganteRoja => {
                // El Sol crece y se traga a Mercurio y a Venus (la Tierra se salva... por poquito).
                let r_sol = 1.4 + 6.5 * suave((t - 1.5) / 8.0);
                v.push((Cosa::Planeta(0), Mat4::rotation_y(t * 0.1) * Mat4::scale(r_sol), None));
                for (n, &(i, r)) in [(1usize, 4.0f32), (2, 6.5), (3, 9.5)].iter().enumerate() {
                    if r > r_sol + 0.2 {
                        let a = n as f32 * 2.3 + t * 1.2 / r;
                        v.push((Cosa::Planeta(i), Mat4::translation(Vec3::new(r * a.cos(), 0.0, r * a.sin())) * Mat4::scale(0.4), None));
                    }
                }
            }
            Tipo::Sol => {
                let k = crece(6.0, 1.5);
                if k > 0.0 {
                    v.push((Cosa::Planeta(0), Mat4::rotation_y(t * 0.2) * Mat4::scale(1.8 * k), None));
                }
            }
            Tipo::Planetas => {
                v.push((Cosa::Planeta(0), Mat4::rotation_y(t * 0.2) * Mat4::scale(1.6), None));
                for (n, &(i, r, tam)) in PLANETAS.iter().enumerate() {
                    let k = crece(1.0 + n as f32 * 0.9, 1.2);
                    if k > 0.0 {
                        let a = n as f32 * 2.1 + t * 1.8 / r;
                        let pos = Vec3::new(r * a.cos(), 0.0, r * a.sin());
                        v.push((Cosa::Planeta(i), Mat4::translation(pos) * Mat4::rotation_y(t) * Mat4::scale(tam * k), None));
                    }
                }
            }
            Tipo::Luna => {
                v.push((Cosa::Planeta(3), Mat4::rotation_z(0.41) * Mat4::rotation_y(t * 0.3) * Mat4::scale(2.0), None));
                if t < 4.0 {
                    let k = suave(t / 4.0);
                    let pos = Vec3::new(-14.0, 1.0, 6.0).lerp(Vec3::new(-2.4, 0.3, 0.5), k * k);
                    v.push((Cosa::Planeta(5), Mat4::translation(pos) * Mat4::scale(1.0), None));
                }
                let k = crece(7.0, 2.0);
                if k > 0.0 {
                    let a = 2.0 + t * 0.4;
                    v.push((Cosa::Planeta(4), Mat4::translation(Vec3::new(5.0 * a.cos(), 0.3, 5.0 * a.sin())) * Mat4::scale(0.55 * k), None));
                }
            }
            Tipo::Vida => {
                v.push((Cosa::Planeta(3), Mat4::rotation_z(0.41) * Mat4::rotation_y(t * 0.25) * Mat4::scale(3.0), None));
            }
            Tipo::Apolo => {
                v.push((Cosa::Planeta(4), Mat4::rotation_y(-0.6) * Mat4::scale(3.0), None));
                v.push((Cosa::Astronauta, Mat4::translation(Vec3::new(0.0, 2.97 + 0.05 * (t * 1.3).sin().abs(), 0.0)) * Mat4::scale(0.35), None));
                v.push((Cosa::Bandera, Mat4::translation(Vec3::new(-0.6, 3.5, 0.2)) * Mat4::scale(0.35), None));
                v.push((Cosa::Planeta(3), Mat4::translation(Vec3::new(-6.0, 7.5, 22.0)) * Mat4::rotation_y(t * 0.2) * Mat4::scale(1.6), None));
            }
            Tipo::Miku => {
                let k = crece(1.0, 0.5);
                if k > 0.0 {
                    v.push((Cosa::Miku, Mat4::translation(Vec3::new(0.0, -0.3 * (1.0 - k), 0.0)) * Mat4::scale(k), Some("guino")));
                }
            }
            Tipo::Hoy => {
                let x = self.nave_x();
                v.push((Cosa::Nave, Mat4::translation(Vec3::new(x, 0.3 * (t * 1.5).sin(), 0.0)) * Mat4::rotation_y(-PI / 2.0) * Mat4::rotation_z(0.15 * (t * 1.5).cos()) * Mat4::scale(0.5), Some("feliz")));
                v.push((Cosa::Planeta(7), Mat4::translation(Vec3::new(13.0, -2.5, 28.0)) * Mat4::rotation_z(0.45) * Mat4::rotation_y(t * 0.3) * Mat4::scale(1.6), None));
                v.push((Cosa::Planeta(3), Mat4::translation(Vec3::new(-6.0, 2.5, 22.0)) * Mat4::rotation_y(t * 0.2) * Mat4::scale(1.4), None));
                v.push((Cosa::Planeta(4), Mat4::translation(Vec3::new(-3.0, 4.0, 20.0)) * Mat4::scale(0.4), None));
            }
            _ => {}
        }
        v
    }

    /// Las particulas y los resplandores de cada capitulo.
    pub fn dibujar_efectos(&self, fb: &mut Framebuffer, vp: &Mat4) {
        let t = self.tc;
        match self.tipo() {
            // La primera luz: la niebla caliente se aclara y queda la "foto" del fondo cosmico de
            // microondas: una esfera de manchitas mas calientes y mas frias.
            Tipo::Luz => {
                let aclara = suave((t - 1.0) / 4.0);
                for i in 0..2400u32 {
                    let d = direccion(i + 3000);
                    let ruido = (d.x * 5.0 + 1.3).sin() * (d.y * 6.0 + 0.4).cos() + 0.6 * (d.z * 9.0 + d.x * 4.0).sin() + 0.4 * ((d.y + d.z) * 13.0).sin();
                    let k = (ruido * 0.7 + 0.5).clamp(0.0, 1.0);
                    let c = if k < 0.5 { mezcla([0.15, 0.3, 1.0], [0.9, 0.9, 0.95], k * 2.0) } else { mezcla([0.9, 0.9, 0.95], [1.0, 0.35, 0.1], (k - 0.5) * 2.0) };
                    punto(fb, vp, d * 8.0, 1.0, c.map(|v| v * aclara * 0.9));
                }
                for i in 0..500u32 {
                    let p = direccion(i + 7000) * (2.0 + 16.0 * h(i * 3));
                    let k = (1.0 - aclara) * 0.5;
                    punto(fb, vp, p, 1.5, [k, 0.55 * k, 0.2 * k]);
                }
            }
            // Una supernova: la estrella se infla, se encoge de golpe y explota en colores.
            Tipo::Supernova => {
                if t < 4.0 {
                    let crece = 1.0 + t * 0.4 + 0.15 * (t * 6.0).sin();
                    let encoge = if t > 3.6 { (4.0 - t) / 0.4 } else { 1.0 };
                    if let Some((x, y, _)) = proyectar(Vec3::zero(), vp, fb) {
                        crate::post::halo(fb, (x, y), 50.0 * crece * encoge, [1.0, 0.6, 0.3], 1.3);
                    }
                } else {
                    let te = t - 4.0;
                    const COLORES: [[f32; 3]; 5] = [[1.0, 0.5, 0.2], [0.4, 0.8, 1.0], [1.0, 0.9, 0.4], [0.9, 0.3, 0.6], [0.5, 1.0, 0.6]];
                    for i in 0..1400u32 {
                        let d = direccion(i + 11000);
                        let r = (te * 2.5).powf(0.75) * (2.0 + 1.5 * h(i * 7));
                        let c = COLORES[(i % 5) as usize];
                        let k = (1.0 - te / 9.0).max(0.15);
                        punto(fb, vp, d * r, 1.0, c.map(|v| v * k));
                    }
                    // Lo que queda: una estrella de neutrones, chiquita y brillante.
                    if let Some((x, y, _)) = proyectar(Vec3::zero(), vp, fb) {
                        crate::post::halo(fb, (x, y), 18.0, [0.6, 0.8, 1.0], 1.2);
                    }
                }
            }
            // Un agujero negro: el disco que gira rapidisimo (mas brillante del lado que viene hacia
            // aca), el anillo de luz y, encima, la parte de atras del disco doblada por la gravedad.
            Tipo::AgujeroNegro => {
                let centro = proyectar(Vec3::zero(), vp, fb);
                let sombra = |p: (f32, f32, f32), rh: f32| centro.is_some_and(|c| ((p.0 - c.0).powi(2) + (p.1 - c.1).powi(2)).sqrt() < rh && p.2 > c.2);
                let rh = centro.and_then(|c| proyectar(Vec3::new(1.4, 0.0, 0.0), vp, fb).map(|q| ((q.0 - c.0).powi(2) + (q.1 - c.1).powi(2)).sqrt())).unwrap_or(30.0);
                for i in 0..1800u32 {
                    let r = 2.0 + 4.5 * h(i * 3).powf(1.5);
                    let a = h(i * 3 + 1) * TAU + t * (6.0 / r);
                    let p = Vec3::new(r * a.cos(), (h(i * 7) - 0.5) * 0.08, r * a.sin());
                    let doppler = 0.55 + 0.45 * a.sin();
                    let c = mezcla([1.0, 0.95, 0.8], [0.9, 0.35, 0.05], (r - 2.0) / 4.5).map(|v| v * doppler);
                    if let Some(q) = proyectar(p, vp, fb) {
                        if !sombra(q, rh) {
                            fb.linea_de_luz((q.0 - 1.0, q.1, q.2), (q.0 + 1.0, q.1, q.2), c, c);
                        }
                    }
                }
                if let Some(c) = centro {
                    // La parte de atras del disco, doblada: un arco arriba y uno abajo del agujero.
                    for k in 0..240 {
                        let a = k as f32 / 240.0 * TAU;
                        let (rx, ry) = (rh * 1.9, rh * 1.25);
                        let p = (c.0 + rx * a.cos(), c.1 + ry * a.sin(), 0.0);
                        let b = 0.35 + 0.25 * a.cos();
                        fb.linea_de_luz(p, (p.0 + 1.0, p.1, 0.0), [b, 0.55 * b, 0.2 * b], [b, 0.55 * b, 0.2 * b]);
                    }
                    // El anillo de fotones, delgadito, justo afuera de la sombra.
                    for k in 0..200 {
                        let a = k as f32 / 200.0 * TAU;
                        let p = (c.0 + rh * 1.05 * a.cos(), c.1 + rh * 1.05 * a.sin(), 0.0);
                        fb.linea_de_luz(p, (p.0 + 1.0, p.1, 0.0), [1.0, 0.8, 0.5], [1.0, 0.8, 0.5]);
                    }
                }
            }
            // El cielo se vuelve azul: el resplandor de la atmosfera pasa de naranja a azul, y las
            // cianobacterias (puntitos verdes) se van regando por el mar.
            Tipo::CieloAzul => {
                let k = suave((t - 2.0) / 7.0);
                if let Some((x, y, _)) = proyectar(Vec3::zero(), vp, fb) {
                    let r = fb.alto as f32 * 0.42;
                    let c = mezcla([0.6, 0.3, 0.05], [0.15, 0.35, 0.8], k);
                    crate::post::halo(fb, (x, y), r, c, 0.8);
                }
                for i in 0..((60.0 + 500.0 * k) as u32) {
                    let d = direccion(i + 15000);
                    if d.z > -0.2 {
                        continue;
                    }
                    let b = 0.5 + 0.5 * (t * 3.0 + i as f32).sin();
                    punto(fb, vp, d * 2.65, 0.8, [0.1 * b, 0.7 * b, 0.3 * b]);
                }
            }
            // Andromeda y la Via Lactea: dos espirales que se acercan, giran una alrededor de la otra y se funden.
            Tipo::Andromeda => {
                let u = suave(t / 11.0);
                let sep = 16.0 * (1.0 - u);
                let giro = u * 2.5;
                for (n, (signo, inclina)) in [(1.0f32, 0.3f32), (-1.0, -0.6)].iter().enumerate() {
                    let centro = Vec3::new(sep * signo * giro.cos(), 0.0, sep * signo * giro.sin());
                    for i in 0..900u32 {
                        let brazo = (i % 2) as f32;
                        let r = 0.5 + 6.5 * h(i * 3 + n as u32 * 9001).powf(0.8);
                        let a = r.ln() * 2.3 + brazo * PI + t * 0.2 * signo + (h(i * 3 + 2) - 0.5) * 0.9;
                        let local = Vec3::new(r * a.cos(), r * a.sin() * inclina.sin(), r * a.sin() * inclina.cos());
                        // Al fundirse, las estrellas se revuelven en una nube redonda.
                        let revuelto = direccion(i + 20000 + n as u32 * 900) * r;
                        let p = centro + local.lerp(revuelto, suave((u - 0.7) / 0.3));
                        let c = if n == 0 { mezcla([1.0, 0.85, 0.6], [0.6, 0.7, 1.0], r / 5.0) } else { mezcla([1.0, 0.8, 0.7], [1.0, 0.6, 0.8], r / 5.0) };
                        punto(fb, vp, p, 0.8, c.map(|v| v * 0.8));
                    }
                }
            }
            // La gigante roja: un resplandor rojo cada vez mas grande alrededor del Sol.
            Tipo::GiganteRoja => {
                let k = suave((t - 1.5) / 8.0);
                if let Some((x, y, _)) = proyectar(Vec3::zero(), vp, fb) {
                    crate::post::halo(fb, (x, y), 120.0 + 380.0 * k, [0.9, 0.3 * (1.0 - k) + 0.12, 0.05], 1.2);
                }
            }
            // El final: las estrellas se van apagando una por una. Al ultimo, una notita cian.
            Tipo::Final => {
                for i in 0..300u32 {
                    let apaga = 1.0 + 9.0 * h(i * 13);
                    if t > apaga {
                        continue;
                    }
                    let b = (1.0 - t / apaga).clamp(0.0, 1.0) * (0.6 + 0.4 * h(i * 3));
                    let p = direccion(i + 30000) * (6.0 + 12.0 * h(i * 5));
                    let c = if i % 4 == 0 { [1.0, 0.5, 0.3] } else { [0.8, 0.85, 1.0] };
                    punto(fb, vp, p, 1.5, c.map(|v| v * b));
                }
                if t > 10.0 {
                    let b = suave((t - 10.0) / 1.5) * (0.7 + 0.3 * (t * 3.0).sin());
                    if let Some((x, y, _)) = proyectar(Vec3::zero(), vp, fb) {
                        crate::post::halo(fb, (x, y), 22.0, [0.2 * b, 0.9 * b, 0.9 * b], 1.3);
                    }
                }
            }
            // El Big Bang: un puntito, y luego la esfera que se expande y se enfria.
            Tipo::BigBang => {
                if t < 0.6 {
                    if let Some((x, y, _)) = proyectar(Vec3::zero(), vp, fb) {
                        crate::post::halo(fb, (x, y), 6.0 + 30.0 * t, [1.0, 1.0, 1.0], 1.0);
                    }
                    return;
                }
                let te = t - 0.6;
                for i in 0..900u32 {
                    let d = direccion(i);
                    let rapidez = 0.4 + h(i * 7 + 5) * 0.8;
                    let r = rapidez * (te * 3.0).powf(0.7) * 2.0;
                    let enfria = (te / 7.0).min(1.0);
                    let c = if enfria < 0.4 { mezcla([1.0, 1.0, 0.95], [1.0, 0.6, 0.2], enfria / 0.4) } else { mezcla([1.0, 0.6, 0.2], [0.5, 0.15, 0.4], (enfria - 0.4) / 0.6) };
                    let b = (1.2 - enfria * 0.7) * (0.5 + 0.5 * h(i * 11));
                    let (a, z) = (d * r, d * (r - 0.25 * rapidez * (1.0 - enfria)));
                    if let (Some(p), Some(q)) = (proyectar(a, vp, fb), proyectar(z, vp, fb)) {
                        fb.linea_de_luz(p, q, c.map(|v| v * b), c.map(|v| v * b * 0.3));
                    }
                }
                if let Some((x, y, _)) = proyectar(Vec3::zero(), vp, fb) {
                    let k = (1.0 - te / 4.0).max(0.0);
                    crate::post::halo(fb, (x, y), 260.0 * k + 20.0, [k, 0.8 * k, 0.5 * k], 1.4);
                }
            }
            // Las primeras estrellas: gas oscuro y estrellas que se prenden.
            Tipo::Estrellas => {
                for i in 0..500u32 {
                    let p = direccion(i) * (4.0 + 11.0 * h(i * 5 + 2)) + Vec3::new(0.0, 0.0, t * 0.05);
                    punto(fb, vp, p, 1.0, [0.12, 0.06, 0.16]);
                }
                for k in 0..60u32 {
                    let prende = 0.6 + k as f32 * 0.15;
                    if t < prende {
                        continue;
                    }
                    let p = direccion(k + 900) * (3.0 + 10.0 * h(k * 13 + 1));
                    let brillo = suave((t - prende) / 0.6) * (0.7 + 0.3 * (t * 3.0 + k as f32).sin());
                    let c = if k % 3 == 0 { [0.6, 0.8, 1.0] } else if k % 3 == 1 { [1.0, 0.9, 0.7] } else { [1.0, 1.0, 1.0] };
                    if let Some((x, y, _)) = proyectar(p, vp, fb) {
                        crate::post::halo(fb, (x, y), 14.0, c.map(|v| v * brillo), 1.6);
                    }
                    punto(fb, vp, p, 2.0, c.map(|v| v * brillo));
                }
            }
            // Una galaxia espiral de dos brazos.
            Tipo::Galaxias => {
                for i in 0..1700u32 {
                    let brazo = (i % 2) as f32;
                    let r = 0.6 + 13.0 * h(i * 3 + 1).powf(0.8);
                    let a = r.ln() * 2.3 + brazo * PI + t * 0.12 + (h(i * 3 + 2) - 0.5) * 0.9;
                    let alto = (h(i * 7) - 0.5) * 0.8 * (1.0 - r / 14.0);
                    let p = Vec3::new(r * a.cos(), alto, r * a.sin());
                    let c = mezcla([1.0, 0.85, 0.6], [0.6, 0.7, 1.0], r / 9.0);
                    let b = 0.55 + 0.45 * h(i * 17);
                    punto(fb, vp, p, 0.8, c.map(|v| v * b));
                }
                if let Some((x, y, _)) = proyectar(Vec3::zero(), vp, fb) {
                    crate::post::halo(fb, (x, y), 90.0, [0.7, 0.55, 0.35], 1.5);
                }
            }
            // El disco de gas que cae en espiral y se prende en el centro.
            Tipo::Sol => {
                let u = (t / 6.5).min(1.0);
                for i in 0..800u32 {
                    let r0 = 2.0 + 9.0 * h(i * 3);
                    let r = r0 * (1.0 - 0.85 * suave(u) * h(i * 5 + 1).max(0.3));
                    let a = h(i * 3 + 1) * TAU + t * (3.0 / r.max(0.5));
                    let p = Vec3::new(r * a.cos(), (h(i * 7) - 0.5) * 0.3, r * a.sin());
                    let c = mezcla([0.5, 0.3, 0.6], [1.0, 0.7, 0.3], u);
                    punto(fb, vp, p, 1.0, c.map(|v| v * 0.8));
                }
                if let Some((x, y, _)) = proyectar(Vec3::zero(), vp, fb) {
                    let k = suave(t / 6.0);
                    crate::post::halo(fb, (x, y), 40.0 + 220.0 * k, [k, 0.65 * k, 0.25 * k], 1.6);
                }
            }
            // El polvo del disco, que se va acabando mientras salen los planetas.
            Tipo::Planetas => {
                let queda = 1.0 - suave(t / 10.0);
                for i in 0..700u32 {
                    let r = 2.3 + 16.0 * h(i * 3);
                    let a = h(i * 3 + 1) * TAU + t * (1.8 / r);
                    let p = Vec3::new(r * a.cos(), (h(i * 7) - 0.5) * 0.4, r * a.sin());
                    punto(fb, vp, p, 0.8, [0.5 * queda, 0.4 * queda, 0.3 * queda]);
                }
                if let Some((x, y, _)) = proyectar(Vec3::zero(), vp, fb) {
                    crate::post::halo(fb, (x, y), 160.0, [0.8, 0.5, 0.15], 1.5);
                }
            }
            // El choque de Theia: pedazos que salen y quedan en anillo.
            Tipo::Luna => {
                if t >= 4.0 {
                    let te = t - 4.0;
                    let choque = Vec3::new(-2.0, 0.3, 0.4);
                    for i in 0..500u32 {
                        let a = h(i * 3) * TAU;
                        let anillo = Vec3::new((3.5 + 2.0 * h(i * 5)) * a.cos(), (h(i * 7) - 0.5) * 0.3, (3.5 + 2.0 * h(i * 5)) * a.sin());
                        let sale = choque + direccion(i) * (te * 3.0 * h(i * 11 + 1));
                        let p = sale.lerp(anillo, suave(te / 3.0));
                        let k = 1.0 - suave((te - 3.5) / 3.0) * 0.7;
                        punto(fb, vp, p, 1.0, [0.9 * k, 0.6 * k, 0.35 * k]);
                    }
                    if let Some((x, y, _)) = proyectar(choque, vp, fb) {
                        let k = (1.0 - te / 1.5).max(0.0);
                        crate::post::halo(fb, (x, y), 200.0 * k, [k, 0.7 * k, 0.3 * k], 1.4);
                    }
                }
            }
            // El asteroide de los dinosaurios.
            Tipo::Vida => {
                let destino = Vec3::new(-1.6, 1.8, -1.9);
                if (7.3..8.3).contains(&t) {
                    let k = (t - 7.3) / 1.0;
                    let desde = Vec3::new(9.0, 7.0, -6.0);
                    let p = desde.lerp(destino, k);
                    let cola = desde.lerp(destino, (k - 0.15).max(0.0));
                    if let (Some(a), Some(b)) = (proyectar(p, vp, fb), proyectar(cola, vp, fb)) {
                        for dy in [-1.0, 0.0, 1.0] {
                            fb.linea_de_luz((a.0, a.1 + dy, a.2), (b.0, b.1 + dy, b.2), [1.0, 0.7, 0.3], [0.3, 0.1, 0.0]);
                        }
                    }
                } else if t >= 8.3 {
                    if let Some((x, y, _)) = proyectar(destino, vp, fb) {
                        let k = (1.0 - (t - 8.3) / 3.0).max(0.0);
                        crate::post::halo(fb, (x, y), 120.0 * k + 10.0, [k, 0.45 * k, 0.1 * k], 1.4);
                    }
                }
            }
            // Miku: un aro de luz cian y notitas que suben.
            Tipo::Miku => {
                if t >= 1.0 {
                    let te = t - 1.0;
                    for i in 0..240u32 {
                        let a = i as f32 / 240.0 * TAU;
                        let r = 0.3 + te * 2.5;
                        let k = (1.0 - te / 2.5).max(0.0);
                        punto(fb, vp, Vec3::new(r * a.cos(), 0.1, r * a.sin()), 1.0, [0.3 * k, 1.0 * k, 0.9 * k]);
                    }
                    for i in 0..40u32 {
                        let u = (te * 0.25 + h(i)).fract();
                        let a = h(i * 3) * TAU + te * 0.5;
                        let p = Vec3::new(1.2 * a.cos(), u * 2.6, 1.2 * a.sin());
                        let c = if i % 2 == 0 { [0.3, 0.9, 1.0] } else { [1.0, 0.4, 0.8] };
                        punto(fb, vp, p, 2.0, c.map(|v| v * (1.0 - u)));
                    }
                    if let Some((x, y, _)) = proyectar(Vec3::new(0.0, 0.8, 0.4), vp, fb) {
                        crate::post::halo(fb, (x, y), 150.0, [0.04, 0.16, 0.16], 1.6);
                    }
                }
            }
            // La estela de la nave, de colores.
            Tipo::Hoy => {
                let x = self.nave_x();
                for k in 0..40 {
                    let (x0, x1) = (x - 0.6 - k as f32 * 0.25, x - 0.6 - (k + 1) as f32 * 0.25);
                    let y0 = 0.3 * ((self.tc - k as f32 * 0.19) * 1.5).sin();
                    let y1 = 0.3 * ((self.tc - (k + 1) as f32 * 0.19) * 1.5).sin();
                    let a = k as f32 / 40.0;
                    let c = [0.5 + 0.5 * (a * 6.0).sin(), 0.5 + 0.5 * (a * 6.0 + 2.1).sin(), 0.5 + 0.5 * (a * 6.0 + 4.2).sin()].map(|v| v * (1.0 - a));
                    if let (Some(p), Some(q)) = (proyectar(Vec3::new(x0, y0, 0.0), vp, fb), proyectar(Vec3::new(x1, y1, 0.0), vp, fb)) {
                        fb.linea_de_luz(p, q, c, c);
                    }
                }
                if let Some((x, y, _)) = proyectar(Vec3::new(-30.0, 8.0, 120.0), vp, fb) {
                    crate::post::halo(fb, (x, y), 120.0, [0.8, 0.5, 0.2], 1.6);
                }
            }
            _ => {}
        }
    }

    /// La tarjeta del capitulo (una banda inclinada arriba a la izquierda), el FIN y como saltar.
    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, ancho: i32, alto: i32) {
        let ayuda = format!("{}: capítulo     {}: siguiente     {}: salir", ui::boton("\u{E00A} \u{E00B}"), ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("aterrizar")));
        ui::texto(d, &ayuda, ancho - ui::medir(&ayuda, 16) - 20, 20, 16, Color::new(200, 210, 240, 200));
        if self.en_fin() {
            let a = (suave(self.tc / 0.8) * 255.0) as u8;
            let s = "FIN";
            ui::texto(d, s, (ancho - ui::medir(s, 90)) / 2, alto / 2 - 80, 90, Color::new(255, 255, 255, a));
            let s2 = "(por ahora)";
            ui::texto(d, s2, (ancho - ui::medir(s2, 28)) / 2, alto / 2 + 20, 28, Color::new(180, 220, 255, a));
            return;
        }
        let c = &CAPITULOS[self.capitulo];
        let entra = suave(self.tc / 0.5);
        let x = -400.0 + 420.0 * entra;
        let a = (entra * 255.0) as u8;
        ui::paralelogramo(d, x - 40.0, 40.0, 560.0, 94.0, 26.0, Color::new(18, 40, 120, (a as f32 * 0.85) as u8));
        ui::paralelogramo(d, x - 40.0, 134.0, 520.0, 6.0, 26.0, Color::new(80, 230, 230, a));
        let n = format!("CAPÍTULO {}", self.capitulo + 1);
        ui::texto(d, &n, x as i32 + 20, 48, 18, Color::new(150, 230, 240, a));
        let otro_piloto = c.tipo == Tipo::Miku && ui::protagonista().id != "miku";
        let titulo = if otro_piloto { "SU PILOTO" } else { c.titulo };
        let fecha = if otro_piloto { "Antes del primer despegue" } else { c.fecha };
        ui::texto(d, titulo, x as i32 + 20, 70, 36, Color::new(255, 255, 255, a));
        if !fecha.is_empty() {
            ui::texto(d, fecha, x as i32 + 24, 148, 20, Color::new(210, 220, 255, a));
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn la_historia_presenta_al_piloto_y_le_da_su_turno() {
        let mut h = Historia::new();
        h.capitulo = CAPITULOS.iter().position(|c| c.tipo == Tipo::Miku).unwrap();
        for p in &crate::personajes::PERSONAJES {
            ui::poner_protagonista(p.id, p.nombre, p.ella);
            h.capitulo = CAPITULOS.iter().position(|c| c.tipo == Tipo::Miku).unwrap();
            h.tc = 0.0;
            let (nombre, texto, voz) = h.dialogo();
            assert_eq!(nombre, ui::NOMBRE_IA);
            assert!(voz == Voz::Ia);
            if p.id == "miku" { assert!(texto.contains("Hatsune Miku")); }
            else { assert_eq!(texto, p.texto); assert!(!texto.contains("Hatsune Miku")); }
            h.capitulo = CAPITULOS.iter().position(|c| c.tipo == Tipo::Vida).unwrap();
            h.tc = CAPITULOS[h.capitulo].dura + 0.1;
            let (nombre, texto, voz) = h.dialogo();
            assert_eq!(nombre, p.nombre);
            assert!(voz == Voz::Miku && !texto.is_empty());
            let mut dialogo = crate::sonido::Dialogo::new();
            assert_eq!(dialogo.mostrar_literal(texto, voz, 100.0, None), texto);
        }
        ui::poner_protagonista("miku", "Miku", true);
    }

    /// Sin tocar nada, pasa por los diez capitulos y termina completa.
    #[test]
    fn la_historia_llega_al_fin() {
        let mut hst = Historia::new();
        let nada = Entrada::default();
        let mut vistos = vec![0];
        for _ in 0..10_000 {
            if let AccionHistoria::Terminar = hst.actualizar(&nada, 0.05) {
                break;
            }
            if vistos.last() != Some(&hst.capitulo) {
                vistos.push(hst.capitulo);
            }
            // Cada capitulo se puede dibujar (camara y cosas sin problemas).
            let _ = (hst.camara(), hst.cosas());
        }
        assert!(hst.terminada && hst.completa);
        assert_eq!(vistos, (0..=CAPITULOS.len()).collect::<Vec<_>>());
    }

    /// E se sale a media historia, y no cuenta como completa.
    #[test]
    fn salirse_no_cuenta() {
        let mut hst = Historia::new();
        hst.actualizar(&Entrada::default(), 1.0);
        assert!(matches!(hst.actualizar(&Entrada { aterrizar: true, ..Default::default() }, 0.1), AccionHistoria::Terminar));
        assert!(!hst.completa);
    }
}
