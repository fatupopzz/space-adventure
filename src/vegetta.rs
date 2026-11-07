//! Aterrizaje en PLANETA VEGETTA (homenaje a la serie de Minecraft de Vegetta777 y a
//! Willyrex): el SERVIDOR DE MINIJUEGOS de Vegetta. Se llega al lobby (una isla flotante con tres
//! portales de obsidiana, Vegetta y Willy, y el marcador de la tarde) y cada portal lleva a un
//! minijuego contra Willyrex:
//!
//! - **PARKOUR**: la carrera sobre islas flotantes (se arma distinta cada vez): huecos, pilares,
//!   slime que avienta, lava, arbolitos y puntos de control (el bloque morado de Vegetta).
//! - **TNT RUN**: tres pisos de arena sobre TNT. Lo que se pisa se cae (con su TNT) un instante
//!   despues: hay que moverse siempre y no caer hasta abajo. Gana el ultimo que siga arriba. Willy
//!   juega con la misma fisica (y a veces se equivoca).
//! - **MINERIA**: dos pozos, uno por jugador, con el frente de vidrio (se ve todo, como granja de
//!   hormigas). Se pica hacia abajo hasta la roca madre: tierra, piedra, menas de carbon, oro y
//!   diamante, bolsas de lava (queman) y obsidiana al fondo. El primero en llegar gana; los
//!   diamantes que se saquen se cambian por notas. Abajo no hay sol: alumbra la antorcha.
//!
//! Tercera persona. Los bloques miden 1: el bloque (x, y, z) ocupa de (x, y, z) a (x+1, y+1, z+1).

use crate::vida::{Tam, Tono, Vida};
use crate::entrada::Entrada;
use crate::matematica::{Mat4, Vec3};
use crate::obj::{Modelo, Textura};
use crate::obra::Obra;
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};
use std::sync::Arc;

const GRAVEDAD: f32 = 24.0;
const SALTO: f32 = 8.2;
const REBOTE: f32 = 15.0;
const VEL: f32 = 5.2;
const VEL_CORRER: f32 = 6.8;
const GIRO: f32 = 2.6;
const RADIO: f32 = 0.28;
const ALTO: f32 = 1.1;
const TRAMOS: usize = 24;
const CADA_CONTROL: usize = 6;
/// La velocidad de Willyrex en el parkour (metros por segundo).
const VEL_WILLY: f32 = 4.4;
const CAIDA: f32 = -14.0;
const CASTIGO: f32 = 3.0;
/// TNT Run: cuanto tarda en caerse lo que se pisa, los pisos y el tamano de la arena.
const TNT_ESPERA: f32 = 0.32;
const TNT_PISOS: [i32; 3] = [0, -6, -12];
const TNT_MEDIO: i32 = 6;
/// Mineria: hasta donde se pica (la roca madre), donde estan los pozos (x de su columna central)
/// y que tan lento pica Willy (comparado con el protagonista).
const FONDO: i32 = -24;
const POZO_PROTA: i32 = -3;
const POZO_WILLY: i32 = 3;
const LENTITUD_WILLY: f32 = 1.3;
/// El campo de vision de Planeta Vegetta (grados).
pub const FOV: f32 = 70.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Bloque {
    Pasto,
    Tierra,
    Piedra,
    Tablas,
    Tronco,
    Hojas,
    MenaDiamante,
    Slime,
    Lava,
    Oro,
    Diamante,
    /// El bloque morado de Vegetta: punto de control.
    Vegetta,
    Obsidiana,
    Meta,
    Arena,
    Tnt,
    Portal,
    Vidrio,
    RocaMadre,
    MenaOro,
    MenaCarbon,
    Ladrillo,
    LanaMorada,
    LanaVerde,
    PiedraLisa,
}

impl Bloque {
    pub const TODOS: [Bloque; 25] = [
        Bloque::Pasto, Bloque::Tierra, Bloque::Piedra, Bloque::Tablas, Bloque::Tronco, Bloque::Hojas, Bloque::MenaDiamante, Bloque::Slime, Bloque::Lava,
        Bloque::Oro, Bloque::Diamante, Bloque::Vegetta, Bloque::Obsidiana, Bloque::Meta, Bloque::Arena, Bloque::Tnt, Bloque::Portal, Bloque::Vidrio,
        Bloque::RocaMadre, Bloque::MenaOro, Bloque::MenaCarbon, Bloque::Ladrillo, Bloque::LanaMorada, Bloque::LanaVerde, Bloque::PiedraLisa,
    ];

    /// Que dibujo del atlas lleva en cada cara: (arriba, lados, abajo).
    fn caras(self) -> (usize, usize, usize) {
        match self {
            Bloque::Pasto => (0, 1, 2),
            Bloque::Tierra => (2, 2, 2),
            Bloque::Piedra => (3, 3, 3),
            Bloque::Tablas => (4, 4, 4),
            Bloque::Tronco => (5, 5, 5),
            Bloque::Hojas => (6, 6, 6),
            Bloque::MenaDiamante => (7, 7, 7),
            Bloque::Slime => (8, 8, 8),
            Bloque::Lava => (9, 9, 9),
            Bloque::Oro => (10, 10, 10),
            Bloque::Diamante => (11, 11, 11),
            Bloque::Vegetta => (12, 12, 12),
            Bloque::Obsidiana => (13, 13, 13),
            Bloque::Meta => (14, 3, 3),
            Bloque::Arena => (16, 16, 16),
            Bloque::Tnt => (18, 17, 18),
            Bloque::Portal => (19, 19, 19),
            Bloque::Vidrio => (20, 20, 20),
            Bloque::RocaMadre => (21, 21, 21),
            Bloque::MenaOro => (22, 22, 22),
            Bloque::MenaCarbon => (23, 23, 23),
            Bloque::Ladrillo => (24, 24, 24),
            Bloque::LanaMorada => (25, 25, 25),
            Bloque::LanaVerde => (26, 26, 26),
            Bloque::PiedraLisa => (27, 27, 27),
        }
    }

    /// Se atraviesa (el portal) o no.
    fn solido(self) -> bool {
        self != Bloque::Portal
    }

    /// Cuanto tarda en picarse (segundos); la roca madre, el vidrio y la lava no se pican.
    fn dureza(self) -> f32 {
        match self {
            Bloque::Tierra | Bloque::Pasto | Bloque::Arena => 0.3,
            Bloque::Piedra => 0.55,
            Bloque::MenaCarbon => 0.6,
            Bloque::MenaOro => 0.75,
            Bloque::MenaDiamante => 0.9,
            Bloque::Obsidiana => 1.6,
            Bloque::RocaMadre | Bloque::Vidrio | Bloque::Lava => f32::INFINITY,
            _ => 0.5,
        }
    }
}

struct Azar(u32);

impl Azar {
    fn n(&mut self, k: i32) -> i32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((self.0 >> 8) % k.max(1) as u32) as i32
    }

    fn f(&mut self) -> f32 {
        self.n(10_000) as f32 / 10_000.0
    }
}

/// Una plataforma del parkour: de (x0, z0) a (x1, z1) (inclusivos), con el piso arriba de y.
#[derive(Clone, Copy, Debug)]
struct Plataforma {
    x0: i32,
    x1: i32,
    z0: i32,
    z1: i32,
    y: i32,
}

impl Plataforma {
    fn centro(&self) -> Vec3 {
        Vec3::new((self.x0 + self.x1 + 1) as f32 * 0.5, self.y as f32 + 1.0, (self.z0 + self.z1 + 1) as f32 * 0.5)
    }
}

/// El mundo de bloques del minijuego (o del lobby).
pub struct Pista {
    bloques: HashMap<(i32, i32, i32), Bloque>,
    plataformas: Vec<Plataforma>,
    pub diamantes: Vec<Vec3>,
    /// El camino de Willyrex en el parkour: puntos y si para llegar al siguiente hay que saltar.
    camino: Vec<(Vec3, bool)>,
    largo: f32,
    nubes: Vec<(Vec3, Vec3)>,
}

impl Pista {
    fn vacia() -> Pista {
        Pista { bloques: HashMap::new(), plataformas: Vec::new(), diamantes: Vec::new(), camino: Vec::new(), largo: 0.0, nubes: Vec::new() }
    }

    fn poner(&mut self, x: i32, y: i32, z: i32, b: Bloque) {
        self.bloques.insert((x, y, z), b);
    }

    fn caja(&mut self, (x0, x1): (i32, i32), (y0, y1): (i32, i32), (z0, z1): (i32, i32), b: Bloque) {
        for x in x0..=x1 {
            for y in y0..=y1 {
                for z in z0..=z1 {
                    self.poner(x, y, z, b);
                }
            }
        }
    }

    fn poner_plataforma(&mut self, p: Plataforma, arriba: Bloque, az: &mut Azar) {
        for x in p.x0..=p.x1 {
            for z in p.z0..=p.z1 {
                self.poner(x, p.y, z, arriba);
                // Abajo, tierra y piedra; a veces cuelga mas (islas flotantes).
                let fondo = 1 + az.n(3);
                for k in 1..=fondo {
                    let b = if k == 1 && arriba == Bloque::Pasto { Bloque::Tierra } else if az.n(9) == 0 { Bloque::MenaDiamante } else { Bloque::Piedra };
                    if k < 3 || (x - p.x0 + z - p.z0) % 2 == 0 {
                        self.poner(x, p.y - k, z, b);
                    }
                }
            }
        }
        self.plataformas.push(p);
    }

    fn nubes(&mut self, az: &mut Azar, z_max: f32) {
        for _ in 0..14 {
            let c = Vec3::new(az.f() * 120.0 - 60.0, 22.0 + az.f() * 10.0, az.f() * (z_max + 60.0) - 30.0);
            let tam = Vec3::new(6.0 + az.f() * 10.0, 1.0, 4.0 + az.f() * 6.0);
            self.nubes.push((c, tam));
        }
    }

    /// La pista del parkour (distinta con cada semilla).
    pub fn generar(semilla: u32) -> Pista {
        let mut az = Azar(semilla | 1);
        let mut pista = Pista::vacia();
        // La salida: una isla de 5 x 5 con el bloque de Vegetta al centro.
        pista.poner_plataforma(Plataforma { x0: -2, x1: 2, z0: -2, z1: 2, y: 0 }, Bloque::Pasto, &mut az);
        pista.poner(0, 0, 0, Bloque::Vegetta);
        let mut anterior = pista.plataformas[0];
        let mut x = 0i32;
        for t in 0..TRAMOS {
            let control = (t + 1) % CADA_CONTROL == 0;
            let tipo = if control { 0 } else { az.n(10) };
            let sube = if tipo == 9 { 0 } else { az.n(3) - 1 };
            // La altura (sin irse ni muy abajo ni muy arriba), y con lo que de verdad subio, el hueco:
            // de uno a tres bloques al mismo nivel; subiendo, de uno a dos; bajando, uno.
            let y = (anterior.y + sube).clamp(-3, 14);
            let hueco = match y - anterior.y {
                0 => 1 + az.n(3),
                1 => 1 + az.n(2),
                _ => 1,
            };
            x = (x + az.n(5) - 2).clamp(-6, 6);
            let z0 = anterior.z1 + hueco + 1;
            let (ancho, largo) = match tipo {
                0 => (3, 3),
                1 | 2 => (1, 1),
                3 | 4 => (2, 2),
                _ => (2 + az.n(2), 2 + az.n(3)),
            };
            let x0 = x - ancho / 2;
            let mut p = Plataforma { x0, x1: x0 + ancho - 1, z0, z1: z0 + largo - 1, y };
            let arriba = match tipo {
                0 => Bloque::Vegetta,
                1 => Bloque::Oro,
                2 => Bloque::Tablas,
                3 => Bloque::Piedra,
                _ => [Bloque::Pasto, Bloque::Pasto, Bloque::Tablas, Bloque::Piedra, Bloque::Obsidiana][az.n(5) as usize],
            };
            if tipo == 9 && !control {
                // El slime: un bloque que avienta para arriba, y la siguiente isla bien alta.
                let s = Plataforma { x0: x, x1: x, z0, z1: z0, y };
                pista.poner_plataforma(s, Bloque::Slime, &mut az);
                pista.camino.push((s.centro(), true));
                p = Plataforma { x0: x - 1, x1: x + 1, z0: z0 + 2, z1: z0 + 6, y: y + 3 };
                pista.poner_plataforma(p, Bloque::Pasto, &mut az);
                pista.diamantes.push(s.centro() + Vec3::new(0.0, 3.5, 1.0));
            } else if tipo == 8 && ancho >= 2 {
                // La isla con una franja de lava en medio (hay que brincarla).
                p.z1 = p.z0 + 6;
                pista.poner_plataforma(p, arriba, &mut az);
                for xx in p.x0..=p.x1 {
                    pista.poner(xx, p.y, p.z0 + 2, Bloque::Lava);
                }
            } else {
                pista.poner_plataforma(p, arriba, &mut az);
                // Un arbolito en las islas grandes (de obstaculo).
                if ancho >= 3 && largo >= 3 && !control && az.n(2) == 0 {
                    let (tx, tz) = (p.x0, p.z1);
                    for k in 1..=3 {
                        pista.poner(tx, p.y + k, tz, Bloque::Tronco);
                    }
                    pista.caja((tx - 1, tx + 1), (p.y + 4, p.y + 4), (tz - 1, tz + 1), Bloque::Hojas);
                }
            }
            // Willy pasa por el centro de cada isla (saltando de una a otra).
            pista.camino.push((Plataforma { x0: p.x0, x1: p.x1, z0: p.z0, z1: p.z0, y: p.y }.centro(), false));
            pista.camino.push((p.centro(), true));
            // Los diamantes: sobre la isla o en el aire del hueco (hay que saltar por ellos).
            if az.n(3) > 0 {
                let en_hueco = az.n(2) == 0;
                let c = if en_hueco { Vec3::new(x as f32 + 0.5, y as f32 + 2.2, (z0 - hueco) as f32 + hueco as f32 * 0.5) } else { p.centro() + Vec3::new(0.0, 0.8, 0.0) };
                pista.diamantes.push(c);
            }
            anterior = p;
        }
        // La meta: una isla grande de cuadros, con una torre de diamante y oro.
        let z0 = anterior.z1 + 3;
        let meta = Plataforma { x0: x - 3, x1: x + 3, z0, z1: z0 + 6, y: anterior.y };
        pista.poner_plataforma(meta, Bloque::Meta, &mut az);
        for k in 1..=5 {
            pista.poner(x, meta.y + k, meta.z1 - 1, if k % 2 == 0 { Bloque::Oro } else { Bloque::Diamante });
        }
        pista.camino.push((Plataforma { z1: z0, ..meta }.centro(), false));
        pista.camino.push((meta.centro(), false));
        // Willy arranca junto al protagonista.
        pista.camino.insert(0, (Vec3::new(1.2, 1.0, 0.5), false));
        pista.largo = pista.camino.windows(2).map(|w| (w[1].0 - w[0].0).length()).sum();
        pista.nubes(&mut az, z0 as f32);
        pista
    }

    /// El lobby: una isla flotante con los tres portales (parkour, TNT Run, mineria), las
    /// banderas de Vegetta y de Willy y un arbolito.
    pub fn lobby() -> Pista {
        let mut az = Azar(77);
        let mut p = Pista::vacia();
        // La isla: pasto alrededor, piedra lisa al centro, tierra y piedra colgando.
        for x in -8i32..=8 {
            for z in -8i32..=8 {
                if x * x + z * z > 72 {
                    continue;
                }
                let centro = x.abs() <= 3 && z.abs() <= 3;
                p.poner(x, 0, z, if centro { Bloque::PiedraLisa } else { Bloque::Pasto });
                let hondo = 1 + az.n(2) + (8 - (x.abs().max(z.abs()))) / 3;
                for k in 1..=hondo {
                    p.poner(x, -k, z, if k == 1 { Bloque::Tierra } else { Bloque::Piedra });
                }
            }
        }
        // El bloque morado de Vegetta al centro.
        p.poner(0, 0, 0, Bloque::Vegetta);
        // Los tres portales al fondo (z = 6): marco de obsidiana, por dentro el portal, y una placa
        // de color enfrente (morada el parkour, de TNT el TNT Run, de mena la mineria).
        for (modo, cx) in PORTALES {
            p.caja((cx - 2, cx + 1), (1, 4), (6, 6), Bloque::Obsidiana);
            p.caja((cx - 1, cx), (1, 3), (6, 6), Bloque::Portal);
            let placa = match modo {
                Modo::Parkour => Bloque::LanaMorada,
                Modo::Tnt => Bloque::Tnt,
                _ => Bloque::MenaDiamante,
            };
            p.caja((cx - 1, cx), (0, 0), (5, 5), placa);
        }
        // Las banderas: Vegetta (morada) a la izquierda, Willy (verde) a la derecha.
        for (x, lana) in [(-7, Bloque::LanaMorada), (7, Bloque::LanaVerde)] {
            p.caja((x, x), (1, 4), (-1, -1), Bloque::Tablas);
            p.caja((x, x), (3, 4), (0, 1), lana);
        }
        // Un arbolito y la banca de ladrillo.
        for k in 1..=3 {
            p.poner(-5, k, -5, Bloque::Tronco);
        }
        p.caja((-6, -4), (4, 5), (-6, -4), Bloque::Hojas);
        p.caja((3, 5), (1, 1), (-5, -5), Bloque::Ladrillo);
        p.nubes(&mut az, 20.0);
        p
    }

    /// TNT Run: tres pisos de arena sobre TNT, separados, sobre el vacio.
    pub fn tnt() -> Pista {
        let mut p = Pista::vacia();
        for &y in &TNT_PISOS {
            p.caja((-TNT_MEDIO, TNT_MEDIO), (y, y), (-TNT_MEDIO, TNT_MEDIO), Bloque::Arena);
            p.caja((-TNT_MEDIO, TNT_MEDIO), (y - 1, y - 1), (-TNT_MEDIO, TNT_MEDIO), Bloque::Tnt);
        }
        // El balcon de obsidiana de Vegetta (para ver la partida desde arriba).
        p.caja((-1, 1), (3, 3), (-TNT_MEDIO - 4, -TNT_MEDIO - 2), Bloque::Obsidiana);
        let mut az = Azar(5);
        p.nubes(&mut az, 30.0);
        p
    }

    /// La mineria: dos pozos de tres bloques de ancho y uno de fondo (como granja de hormigas:
    /// vidrio al frente y entre los dos), llenos de tierra, piedra, menas y bolsas de lava, con
    /// obsidiana y roca madre al fondo. Los dos pozos traen lo mismo (es justo).
    pub fn mina(semilla: u32) -> Pista {
        let mut az = Azar(semilla | 1);
        let mut p = Pista::vacia();
        // Atras y a los lados, roca madre (oscura: asi resalta lo de dentro de los pozos y lo
        // ya picado se ve hueco), con pasto encima al ras de los pozos.
        p.caja((-7, 7), (FONDO, -2), (-3, -1), Bloque::RocaMadre);
        p.caja((-7, 7), (-1, -1), (-3, -1), Bloque::Pasto);
        for x0 in [-7, 5] {
            p.caja((x0, x0 + 2), (FONDO, -2), (0, 0), Bloque::RocaMadre);
            p.caja((x0, x0 + 2), (-1, -1), (0, 0), Bloque::Pasto);
        }
        // El vidrio: al frente (con un barandal arriba), entre los pozos y en las orillas.
        p.caja((-7, 7), (FONDO, 0), (1, 1), Bloque::Vidrio);
        p.caja((-1, 1), (FONDO, 0), (0, 0), Bloque::Vidrio);
        p.caja((-5, -5), (0, 0), (0, 0), Bloque::Vidrio);
        p.caja((5, 5), (0, 0), (0, 0), Bloque::Vidrio);
        p.caja((-7, 7), (FONDO - 1, FONDO - 1), (-3, 1), Bloque::RocaMadre);
        for y in FONDO..0 {
            for dx in -1..=1 {
                let b = if y == FONDO {
                    Bloque::RocaMadre
                } else if y == FONDO + 1 {
                    Bloque::Obsidiana
                } else if y >= -3 {
                    Bloque::Tierra
                } else {
                    // Las menas buenas, mas abajo; la lava, solo en las columnas de los lados
                    // (por la de en medio siempre se puede bajar derecho).
                    match az.n(100) {
                        0..=5 if y < -7 => Bloque::MenaDiamante,
                        6..=13 => Bloque::MenaOro,
                        14..=24 => Bloque::MenaCarbon,
                        25..=31 if y < -6 && dx != 0 => Bloque::Lava,
                        _ => Bloque::Piedra,
                    }
                };
                p.poner(POZO_PROTA + dx, y, 0, b);
                p.poner(POZO_WILLY + dx, y, 0, b);
            }
        }
        p
    }

    pub fn bloque(&self, x: i32, y: i32, z: i32) -> Option<Bloque> {
        self.bloques.get(&(x, y, z)).copied()
    }

    fn solido(&self, x: i32, y: i32, z: i32) -> bool {
        self.bloque(x, y, z).is_some_and(Bloque::solido)
    }

    /// Si una caja de radio `RADIO` y alto `ALTO` con los pies en `p` toca algun bloque.
    fn choca(&self, p: Vec3) -> bool {
        let e = 1e-3;
        let (x0, x1) = ((p.x - RADIO + e).floor() as i32, (p.x + RADIO - e).floor() as i32);
        let (y0, y1) = ((p.y + e).floor() as i32, (p.y + ALTO - e).floor() as i32);
        let (z0, z1) = ((p.z - RADIO + e).floor() as i32, (p.z + RADIO - e).floor() as i32);
        (x0..=x1).any(|x| (y0..=y1).any(|y| (z0..=z1).any(|z| self.solido(x, y, z))))
    }

    /// El bloque que se tiene bajo los pies (y donde esta).
    fn bajo(&self, p: Vec3) -> Option<((i32, i32, i32), Bloque)> {
        let y = (p.y - 0.05).floor() as i32;
        for (dx, dz) in [(0.0, 0.0), (-RADIO, -RADIO), (RADIO, -RADIO), (-RADIO, RADIO), (RADIO, RADIO)] {
            let (x, z) = ((p.x + dx).floor() as i32, (p.z + dz).floor() as i32);
            if let Some(b) = self.bloque(x, y, z).filter(|b| b.solido()) {
                return Some(((x, y, z), b));
            }
        }
        None
    }

    /// Donde va Willy despues de recorrer `s` metros de su camino (saltando entre islas).
    fn en_camino(&self, s: f32) -> (Vec3, Vec3) {
        let mut queda = s.max(0.0);
        for w in self.camino.windows(2) {
            let (a, salta) = w[0];
            let b = w[1].0;
            let largo = (b - a).length();
            if queda <= largo {
                let u = if largo > 0.0 { queda / largo } else { 1.0 };
                let mut p = a.lerp(b, u);
                if salta {
                    let alto = 1.2 + (b.y - a.y).max(0.0);
                    p.y += alto * (u * PI).sin();
                }
                return (p, (b - a).normalize());
            }
            queda -= largo;
        }
        let ultimo = self.camino.last().map_or(Vec3::zero(), |c| c.0);
        (ultimo, Vec3::new(0.0, 0.0, 1.0))
    }

    /// Arma el mundo en triangulos (solo las caras que se ven), con el atlas de bloques.
    pub fn modelo(&self, atlas: Option<Arc<Textura>>) -> Modelo {
        let mut o = Obra::new();
        o.material_con_textura("bloques", [1.0, 1.0, 1.0], [0.0; 3], 0.05, 8.0, atlas.clone());
        o.material_con_textura("lava", [1.0, 1.0, 1.0], [0.9, 0.5, 0.2], 0.0, 1.0, atlas.clone());
        o.material_con_textura("brillan", [1.0, 1.0, 1.0], [0.25, 0.25, 0.25], 0.4, 40.0, atlas.clone());
        o.material_con_textura("portal", [1.0, 1.0, 1.0], [0.55, 0.25, 0.85], 0.0, 1.0, atlas.clone());
        o.material_con_textura("vidrio", [1.0, 1.0, 1.0], [0.05, 0.07, 0.08], 0.25, 30.0, atlas);
        o.transparente("portal", 0.78);
        o.transparente("vidrio", 0.2);
        o.material("nube", [0.98, 0.98, 1.0], [0.35, 0.35, 0.38], 0.0, 1.0);
        let uv = uv_atlas;
        for (&(x, y, z), &b) in &self.bloques {
            let (arriba, lado, abajo) = b.caras();
            let mat = match b {
                Bloque::Lava => "lava",
                Bloque::Portal => "portal",
                Bloque::Vidrio => "vidrio",
                Bloque::Diamante | Bloque::Oro | Bloque::Vegetta | Bloque::Slime => "brillan",
                _ => "bloques",
            };
            let p = |dx: f32, dy: f32, dz: f32| Vec3::new(x as f32 + dx, y as f32 + dy, z as f32 + dz);
            // Cada cara solo si del otro lado no hay bloque que la tape (las hojas, la lava, el
            // vidrio y el portal tapan menos: siempre). Entre dos vidrios no hay cara.
            let libre = |dx: i32, dy: i32, dz: i32| match self.bloque(x + dx, y + dy, z + dz) {
                None => true,
                Some(o) if o == b && matches!(b, Bloque::Vidrio | Bloque::Portal) => false,
                Some(Bloque::Hojas | Bloque::Lava | Bloque::Vidrio | Bloque::Portal) => true,
                Some(_) => false,
            };
            if libre(0, 1, 0) {
                o.quad_uv(mat, [p(0.0, 1.0, 0.0), p(1.0, 1.0, 0.0), p(1.0, 1.0, 1.0), p(0.0, 1.0, 1.0)], uv(arriba), Vec3::new(0.0, 1.0, 0.0));
            }
            if libre(0, -1, 0) {
                o.quad_uv(mat, [p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), p(1.0, 0.0, 1.0), p(0.0, 0.0, 1.0)], uv(abajo), Vec3::new(0.0, -1.0, 0.0));
            }
            if libre(0, 0, -1) {
                o.quad_uv(mat, [p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0), p(0.0, 1.0, 0.0)], uv(lado), Vec3::new(0.0, 0.0, -1.0));
            }
            if libre(0, 0, 1) {
                o.quad_uv(mat, [p(1.0, 0.0, 1.0), p(0.0, 0.0, 1.0), p(0.0, 1.0, 1.0), p(1.0, 1.0, 1.0)], uv(lado), Vec3::new(0.0, 0.0, 1.0));
            }
            if libre(-1, 0, 0) {
                o.quad_uv(mat, [p(0.0, 0.0, 1.0), p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0), p(0.0, 1.0, 1.0)], uv(lado), Vec3::new(-1.0, 0.0, 0.0));
            }
            if libre(1, 0, 0) {
                o.quad_uv(mat, [p(1.0, 0.0, 0.0), p(1.0, 0.0, 1.0), p(1.0, 1.0, 1.0), p(1.0, 1.0, 0.0)], uv(lado), Vec3::new(1.0, 0.0, 0.0));
            }
        }
        for (c, t) in &self.nubes {
            o.caja("nube", *c - *t * 0.5, *c + *t * 0.5);
        }
        o.modelo()
    }
}

/// Las UV del dibujo `n` del atlas (8 columnas por 4 filas), un poquito hacia adentro.
fn uv_atlas(n: usize) -> [[f32; 2]; 4] {
    let (eu, ev) = (0.5 / 512.0, 0.5 / 256.0);
    let (c, f) = ((n % 8) as f32, (n / 8) as f32);
    let (u0, u1) = (c / 8.0 + eu, (c + 1.0) / 8.0 - eu);
    let (vt, vb) = (1.0 - f / 4.0 - ev, 1.0 - (f + 1.0) / 4.0 + ev);
    [[u0, vb], [u1, vb], [u1, vt], [u0, vt]]
}

/// Un bloque suelto de lado 1, centrado en el origen (los que se caen y los pedacitos).
pub fn cubo(b: Bloque, atlas: Option<Arc<Textura>>) -> Modelo {
    let mut o = Obra::new();
    let emisivo = if b == Bloque::Lava { [0.9, 0.5, 0.2] } else { [0.0; 3] };
    o.material_con_textura("bloque", [1.0, 1.0, 1.0], emisivo, 0.05, 8.0, atlas);
    let (arriba, lado, abajo) = b.caras();
    let p = |x: f32, y: f32, z: f32| Vec3::new(x - 0.5, y - 0.5, z - 0.5);
    o.quad_uv("bloque", [p(0.0, 1.0, 0.0), p(1.0, 1.0, 0.0), p(1.0, 1.0, 1.0), p(0.0, 1.0, 1.0)], uv_atlas(arriba), Vec3::new(0.0, 1.0, 0.0));
    o.quad_uv("bloque", [p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), p(1.0, 0.0, 1.0), p(0.0, 0.0, 1.0)], uv_atlas(abajo), Vec3::new(0.0, -1.0, 0.0));
    o.quad_uv("bloque", [p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0), p(0.0, 1.0, 0.0)], uv_atlas(lado), Vec3::new(0.0, 0.0, -1.0));
    o.quad_uv("bloque", [p(1.0, 0.0, 1.0), p(0.0, 0.0, 1.0), p(0.0, 1.0, 1.0), p(1.0, 1.0, 1.0)], uv_atlas(lado), Vec3::new(0.0, 0.0, 1.0));
    o.quad_uv("bloque", [p(0.0, 0.0, 1.0), p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0), p(0.0, 1.0, 1.0)], uv_atlas(lado), Vec3::new(-1.0, 0.0, 0.0));
    o.quad_uv("bloque", [p(1.0, 0.0, 0.0), p(1.0, 0.0, 1.0), p(1.0, 1.0, 1.0), p(1.0, 1.0, 0.0)], uv_atlas(lado), Vec3::new(1.0, 0.0, 0.0));
    o.modelo()
}

/// Un cuerpo con fisica de plataformas (el protagonista, y Willy en TNT Run).
#[derive(Clone, Debug)]
pub struct Cuerpo {
    pub pos: Vec3,
    vel: Vec3,
    pub yaw: f32,
    en_suelo: bool,
    coyote: f32,
    /// El salto pedido un poquito antes de caer al piso (se hace al tocarlo) y si sigue subiendo
    /// con el boton apretado (soltarlo antes corta el salto: salto chiquito).
    buffer_salto: f32,
    saltando: bool,
    pub paso: f32,
}

impl Cuerpo {
    fn new(pos: Vec3, yaw: f32) -> Self {
        Cuerpo { pos, vel: Vec3::zero(), yaw, en_suelo: false, coyote: 0.0, buffer_salto: 0.0, saltando: false, paso: 0.0 }
    }

    pub fn adelante(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos())
    }

    /// Corre, gira y salta con la entrada. Devuelve si salto (para el sonido).
    fn mover(&mut self, e: &Entrada, dt: f32, mundo: &Pista) -> bool {
        let avance = (e.empuje + e.cabecear).clamp(-1.0, 1.0);
        self.yaw += e.girar * GIRO * dt + e.raton.0;
        let vel = if e.turbo { VEL_CORRER } else { VEL };
        let deseada = self.adelante() * (avance * vel);
        // En el aire se controla menos.
        let agarre = if self.en_suelo { 14.0 } else { 4.0 };
        let k = 1.0 - (-agarre * dt).exp();
        self.vel.x += (deseada.x - self.vel.x) * k;
        self.vel.z += (deseada.z - self.vel.z) * k;
        self.coyote = if self.en_suelo { 0.12 } else { (self.coyote - dt).max(0.0) };
        self.buffer_salto = if e.aceptar { 0.13 } else { (self.buffer_salto - dt).max(0.0) };
        let mut salto = false;
        if self.buffer_salto > 0.0 && self.coyote > 0.0 {
            self.vel.y = SALTO;
            self.coyote = 0.0;
            self.buffer_salto = 0.0;
            self.en_suelo = false;
            self.saltando = true;
            salto = true;
        }
        // Soltar el boton subiendo corta el salto (como en Mario): se controla que tan alto.
        if self.saltando && !e.disparando && !e.aceptar && self.vel.y > 0.0 {
            self.vel.y *= 0.55;
            self.saltando = false;
        }
        if self.vel.y <= 0.0 {
            self.saltando = false;
        }
        self.vel.y -= GRAVEDAD * dt;
        // Por ejes: primero de lado, luego para adelante, luego vertical.
        let mut p = self.pos;
        let nx = Vec3::new(p.x + self.vel.x * dt, p.y, p.z);
        if !mundo.choca(nx) {
            p = nx;
        } else {
            self.vel.x = 0.0;
        }
        let nz = Vec3::new(p.x, p.y, p.z + self.vel.z * dt);
        if !mundo.choca(nz) {
            p = nz;
        } else {
            self.vel.z = 0.0;
        }
        let ny = Vec3::new(p.x, p.y + self.vel.y * dt, p.z);
        if mundo.choca(ny) {
            if self.vel.y < 0.0 {
                p.y = (ny.y + 1e-3).floor() + 1.0;
                self.en_suelo = true;
            }
            self.vel.y = 0.0;
        } else {
            p = ny;
            self.en_suelo = false;
        }
        self.pos = p;
        if self.en_suelo {
            self.paso += Vec3::new(self.vel.x, 0.0, self.vel.z).length() * dt;
        }
        salto
    }

    /// Los "botones" para ir hacia `destino`: girar hacia el, avanzar y saltar en las orillas.
    fn ir_hacia(&self, destino: Vec3, mundo: &Pista) -> Entrada {
        let a = destino - self.pos;
        let quiere = (-a.x).atan2(-a.z);
        let giro = (quiere - self.yaw + PI).rem_euclid(TAU) - PI;
        let piso = |d: f32| {
            let q = self.pos + self.adelante() * d;
            mundo.bloque(q.x.floor() as i32, (self.pos.y - 0.05).floor() as i32, q.z.floor() as i32)
        };
        let sin_piso = piso(0.5).is_none();
        let lejos = Vec3::new(a.x, 0.0, a.z).length() > 0.6;
        Entrada {
            girar: (giro * 3.0).clamp(-1.0, 1.0),
            empuje: if lejos && giro.abs() < 1.2 { 1.0 } else { 0.0 },
            aceptar: self.en_suelo && sin_piso && lejos && giro.abs() < 0.5,
            disparando: true,
            ..Default::default()
        }
    }
}

/// Un pedacito de bloque que sale volando (al picar o al caerse la arena).
#[derive(Clone, Copy, Debug)]
pub struct Particula {
    pub pos: Vec3,
    vel: Vec3,
    pub bloque: Bloque,
    vida: f32,
    pub tam: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Modo {
    Lobby,
    Parkour,
    Tnt,
    Mineria,
}

impl Modo {
    fn nombre(self) -> &'static str {
        match self {
            Modo::Lobby => "LOBBY",
            Modo::Parkour => "PARKOUR",
            Modo::Tnt => "TNT RUN",
            Modo::Mineria => "MINERÍA",
        }
    }

    fn indice(self) -> usize {
        match self {
            Modo::Parkour => 0,
            Modo::Tnt => 1,
            _ => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Fase {
    Llegada(usize),
    /// En el lobby, caminando.
    Libre,
    /// 3, 2, 1...
    Cuenta,
    Jugando,
    /// Se acabo el minijuego: si gano.
    Fin(bool),
}

pub enum AccionVegetta {
    Nada,
    Despegar,
}

pub const VEGETTA: &str = "VEGETTA777";
pub const WILLY: &str = "WILLYREX";

const LLEGADA: [(&str, &str, Voz); 3] = [
    (ui::NOMBRE_IA, "Planeta Vegetta. Todo es de bloques. Hasta las nubes. Hasta yo, si me quedo mucho rato.", Voz::Ia),
    (VEGETTA, "¡Muy buenas a todos, guapísimos! Bienvenidos a mi servidor. Tres portales, tres minijuegos: parkour, TNT Run y minería.", Voz::Neptuno),
    (WILLY, "Y en los tres te voy a ganar, guapi. El que pierda le construye la casa al otro. De tierra. Camina a un portal.", Voz::Erizo),
];
const TROPIEZOS: [&str; 3] = ["¡Ay, que me tropecé con un bloque!", "¡Lag! ¡Fue lag, lo juro!", "¡Vegetta, me empujaron!"];
/// Los portales del lobby: el minijuego, la x de su centro. Se ven desde -z (+x queda a la
/// izquierda de la pantalla): asi se leen parkour, TNT Run y mineria de izquierda a derecha.
const PORTALES: [(Modo, i32); 3] = [(Modo::Parkour, 5), (Modo::Tnt, 0), (Modo::Mineria, -5)];

pub struct Vegetta {
    pub modo: Modo,
    pub pista: Pista,
    pub modelo: Modelo,
    /// El mundo cambio (se cayo o se pico algo): hay que volver a armar su modelo.
    sucio: bool,
    atlas: Option<Arc<Textura>>,
    fase: Fase,
    t: f32,
    pub total: f32,
    semilla: u32,
    pub prota: Cuerpo,
    control: Vec3,
    pub caidas: u32,
    // Willy: en el parkour sigue su camino; en TNT Run y en la mina, su propio cuerpo.
    s_willy: f32,
    pub willy: Cuerpo,
    tropiezo: f32,
    tropiezos: u32,
    pub willy_llego: Option<f32>,
    willy_fuera: bool,
    prota_fuera: bool,
    meta_willy: Option<Vec3>,
    // TNT Run: lo pisado (cuanto le falta para caerse) y lo que va cayendo.
    pisadas: HashMap<(i32, i32, i32), f32>,
    pub cayendo: Vec<(Bloque, Vec3, f32)>,
    // La mina: lo que se esta picando (y cuanto lleva), los diamantes y el humo de la lava.
    pub picando: Option<((i32, i32, i32), f32)>,
    willy_pica: f32,
    pub diamantes_mina: u32,
    pub diamantes_willy: u32,
    quemado: f32,
    pub particulas: Vec<Particula>,
    /// Las chispas de los diamantes, los puntos de control y la meta.
    pub jugo: crate::chispas::Jugo,
    // El minijuego.
    pub tiempo: f32,
    pub tomados: Vec<bool>,
    pub gano: bool,
    /// El marcador de la tarde: (tu, Willy) en parkour, TNT Run y mineria.
    pub marcador: [(u32, u32); 3],
    pub terminadas: u32,
    pub ganadas: u32,
    camara: (Vec3, Vec3),
    mensaje: Option<(&'static str, String, Voz, f32)>,
    /// Los letreros gigantes, el confeti y las sacudidas (ver vida.rs).
    pub vida: Vida,
    pub eventos: Vec<Efecto>,
}

impl Vegetta {
    pub fn new(semilla: u32, atlas: Option<Arc<Textura>>) -> Self {
        let pista = Pista::lobby();
        let modelo = pista.modelo(atlas.clone());
        let pos = Vec3::new(0.5, 1.0, -4.5);
        let mut v = Vegetta {
            modo: Modo::Lobby,
            pista,
            modelo,
            sucio: false,
            atlas,
            fase: Fase::Llegada(0),
            t: 0.0,
            total: 0.0,
            semilla: semilla | 1,
            prota: Cuerpo::new(pos, PI),
            control: pos,
            caidas: 0,
            s_willy: 0.0,
            willy: Cuerpo::new(Vec3::new(3.0, 1.0, -1.0), 0.0),
            tropiezo: 0.0,
            tropiezos: 0,
            willy_llego: None,
            willy_fuera: false,
            prota_fuera: false,
            meta_willy: None,
            pisadas: HashMap::new(),
            cayendo: Vec::new(),
            picando: None,
            willy_pica: 0.0,
            diamantes_mina: 0,
            diamantes_willy: 0,
            quemado: 0.0,
            particulas: Vec::new(),
            jugo: Default::default(),
            tiempo: 0.0,
            tomados: Vec::new(),
            gano: false,
            marcador: [(0, 0); 3],
            terminadas: 0,
            ganadas: 0,
            camara: (pos + Vec3::new(0.0, 2.5, -4.5), pos),
            mensaje: None,
            vida: Vida::sin_cartel(),
            eventos: Vec::new(),
        };
        let (q, t, voz) = LLEGADA[0];
        v.decir(q, t, voz);
        v
    }

    fn azar(&mut self) -> f32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) as f32 / 16_777_216.0
    }

    fn decir(&mut self, quien: &'static str, texto: &str, voz: Voz) {
        let largo = texto.chars().count() as f32;
        self.mensaje = Some((quien, texto.to_string(), voz, 2.5 + largo / 18.0));
    }

    fn anunciar(&mut self, s: &str) {
        self.vida.grito(s, Tono::Oro, Tam::Grande);
        self.vida.golpe(0.2, Tono::Oro);
    }

    fn cambiar(&mut self, f: Fase) {
        self.fase = f;
        self.t = 0.0;
    }

    fn poner_mundo(&mut self, pista: Pista) {
        self.modelo = pista.modelo(self.atlas.clone());
        self.pista = pista;
        self.sucio = false;
        self.pisadas.clear();
        self.cayendo.clear();
        self.particulas.clear();
        self.picando = None;
    }

    /// Vuelve al lobby (frente al portal de donde se vino).
    fn al_lobby(&mut self) {
        let de = self.modo;
        self.modo = Modo::Lobby;
        self.poner_mundo(Pista::lobby());
        let x = PORTALES.iter().find(|p| p.0 == de).map_or(0, |p| p.1) as f32;
        self.prota = Cuerpo::new(Vec3::new(x, 1.0, 3.0), 0.0);
        self.willy = Cuerpo::new(Vec3::new(3.0, 1.0, -1.0), 0.0);
        self.mensaje = None;
        self.cambiar(Fase::Libre);
    }

    /// Entra al minijuego `modo` (al pasar por su portal).
    fn empezar(&mut self, modo: Modo) {
        self.modo = modo;
        self.tiempo = 0.0;
        self.caidas = 0;
        self.gano = false;
        self.willy_llego = None;
        self.willy_fuera = false;
        self.prota_fuera = false;
        self.meta_willy = None;
        self.tropiezo = 0.0;
        self.tropiezos = 0;
        self.s_willy = 0.0;
        self.diamantes_mina = 0;
        self.diamantes_willy = 0;
        self.quemado = 0.0;
        self.semilla = self.semilla.wrapping_mul(2_654_435_761).wrapping_add(13);
        match modo {
            Modo::Parkour => {
                let pista = Pista::generar(self.semilla);
                self.tomados = vec![false; pista.diamantes.len()];
                self.poner_mundo(pista);
                self.prota = Cuerpo::new(Vec3::new(-0.8, 1.0, 0.5), PI);
                self.decir(VEGETTA, "¡Parkour! Saltos, slime, lava y diamantes. Si se cae al vacío, vuelve al último bloque morado.", Voz::Neptuno);
            }
            Modo::Tnt => {
                self.poner_mundo(Pista::tnt());
                // Los dos viendo hacia el centro de la arena.
                self.prota = Cuerpo::new(Vec3::new(-3.5, 1.0, 0.5), -PI * 0.5);
                self.willy = Cuerpo::new(Vec3::new(4.5, 1.0, 0.5), PI * 0.5);
                self.decir(VEGETTA, "¡TNT Run! Lo que pisan se cae. ¡No se queden quietos! Gana el último que siga arriba.", Voz::Neptuno);
            }
            Modo::Mineria => {
                let semilla = self.semilla;
                self.poner_mundo(Pista::mina(semilla));
                self.prota = Cuerpo::new(Vec3::new(POZO_PROTA as f32 + 0.5, 0.0, 0.5), PI);
                self.willy = Cuerpo::new(Vec3::new(POZO_WILLY as f32 + 0.5, 0.0, 0.5), PI);
                self.willy_pica = 0.0;
                self.decir(VEGETTA, "¡Minería! Piquen para abajo hasta la roca madre. Los diamantes son suyos, pero ojo: en los lados hay lava.", Voz::Neptuno);
            }
            Modo::Lobby => {}
        }
        self.control = self.prota.pos;
        self.anunciar(modo.nombre());
        self.eventos.push(Efecto::Salto);
        self.cambiar(Fase::Cuenta);
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionVegetta {
        self.t += dt;
        self.total += dt;
        self.jugo.actualizar(dt);
        if let Some(m) = &mut self.mensaje {
            m.3 -= dt;
            if m.3 <= 0.0 {
                self.mensaje = None;
            }
        }
        if e.aterrizar {
            return AccionVegetta::Despegar;
        }
        match self.fase {
            Fase::Llegada(i) => {
                if (e.aceptar && self.t > 0.4) || self.mensaje.is_none() {
                    if i + 1 < LLEGADA.len() {
                        let (q, t, voz) = LLEGADA[i + 1];
                        self.decir(q, t, voz);
                        self.cambiar(Fase::Llegada(i + 1));
                    } else {
                        self.mensaje = None;
                        self.cambiar(Fase::Libre);
                    }
                }
            }
            Fase::Libre => {
                self.mover_prota(e, dt);
                // Al cruzar un portal: a su minijuego.
                let p = self.prota.pos;
                if let Some(&(modo, _)) = PORTALES.iter().find(|&&(_, cx)| p.z > 5.6 && p.z < 7.0 && p.x >= (cx - 1) as f32 && p.x <= (cx + 1) as f32) {
                    self.empezar(modo);
                }
                if self.prota.pos.y < CAIDA {
                    self.prota = Cuerpo::new(Vec3::new(0.5, 1.0, -4.5), PI);
                    self.anunciar("¡AL VACÍO!");
                }
            }
            Fase::Cuenta => {
                let antes = (self.t - dt).floor() as i32;
                if self.t.floor() as i32 != antes && self.t < 3.5 {
                    self.eventos.push(Efecto::Carta);
                }
                if self.t >= 3.0 {
                    self.anunciar("¡YA!");
                    self.eventos.push(Efecto::Perfecto);
                    self.cambiar(Fase::Jugando);
                }
            }
            Fase::Jugando => {
                self.tiempo += dt;
                match self.modo {
                    Modo::Parkour => {
                        self.mover_prota(e, dt);
                        self.parkour();
                        self.mover_willy_parkour(dt);
                    }
                    Modo::Tnt => self.tnt(e, dt),
                    Modo::Mineria => self.mina(e, dt),
                    Modo::Lobby => {}
                }
            }
            Fase::Fin(_) => {
                if self.modo == Modo::Parkour {
                    self.mover_willy_parkour(dt);
                }
                if e.aceptar && self.t > 1.5 {
                    self.al_lobby();
                }
            }
        }
        self.cayendo_y_particulas(dt);
        if self.sucio {
            self.modelo = self.pista.modelo(self.atlas.clone());
            self.sucio = false;
        }
        self.mover_camara(dt);
        AccionVegetta::Nada
    }

    fn mover_prota(&mut self, e: &Entrada, dt: f32) {
        if self.prota.mover(e, dt, &self.pista) {
            self.eventos.push(Efecto::Ka);
        }
    }

    // -----------------------------------------------------------------------
    // Parkour
    // -----------------------------------------------------------------------

    fn parkour(&mut self) {
        if self.prota.en_suelo {
            match self.pista.bajo(self.prota.pos).map(|b| b.1) {
                Some(Bloque::Slime) => {
                    self.prota.vel.y = REBOTE;
                    self.prota.en_suelo = false;
                    self.vida.grito_en("¡BOING!", "", Tono::Verde, Tam::Chico, 0.5, 0.4);
                    self.eventos.push(Efecto::Don);
                }
                Some(Bloque::Lava) => {
                    self.anunciar("¡LAVA!");
                    self.reaparecer();
                    return;
                }
                Some(Bloque::Vegetta) => {
                    let c = Vec3::new(self.prota.pos.x.floor() + 0.5, self.prota.pos.y, self.prota.pos.z.floor() + 0.5);
                    if (c - self.control).length() > 3.0 {
                        self.control = c;
                        self.anunciar("¡PUNTO DE CONTROL!");
                        self.jugo.estallar(c + Vec3::new(0.0, 0.8, 0.0), [0.75, 0.4, 1.0], 26, 6.0);
                        self.jugo.onda(c + Vec3::new(0.0, 0.2, 0.0), [0.75, 0.4, 1.0], 4.0);
                        self.eventos.push(Efecto::Sello);
                    }
                }
                Some(Bloque::Meta) => self.llegar(),
                _ => {}
            }
        }
        if self.prota.pos.y < CAIDA {
            self.anunciar("¡AL VACÍO!");
            self.reaparecer();
        }
        // Los diamantes.
        for i in 0..self.pista.diamantes.len() {
            if !self.tomados[i] && (self.pista.diamantes[i] - (self.prota.pos + Vec3::new(0.0, 0.6, 0.0))).length() < 0.9 {
                self.tomados[i] = true;
                let p = self.pista.diamantes[i];
                self.jugo.estallar(p, [0.35, 1.0, 0.95], 24, 6.0);
                self.jugo.onda(p, [0.35, 1.0, 0.95], 3.0);
                self.vida.grito("¡DIAMANTE!", Tono::Cian, Tam::Medio);
                self.vida.golpe(0.15, Tono::Cian);
                self.eventos.push(Efecto::Nota3);
            }
        }
    }

    fn reaparecer(&mut self) {
        self.caidas += 1;
        self.tiempo += CASTIGO;
        self.prota.pos = self.control + Vec3::new(0.0, 0.2, 0.0);
        self.prota.vel = Vec3::zero();
        self.vida.grito("¡AL VACÍO!", Tono::Rojo, Tam::Grande);
        self.vida.golpe(0.5, Tono::Rojo);
        self.eventos.push(Efecto::Fallo);
        if self.caidas % 3 == 1 {
            self.decir(WILLY, "¡Jajaja! ¡Al vacío! Así no se juega, guapi.", Voz::Erizo);
        }
    }

    fn llegar(&mut self) {
        if self.fase != Fase::Jugando {
            return;
        }
        let gano = self.willy_llego.is_none();
        self.terminar(gano);
    }

    /// Se acaba el minijuego: el marcador, el letrero y lo que dicen.
    fn terminar(&mut self, gano: bool) {
        self.terminadas += 1;
        self.gano = gano;
        let k = self.modo.indice();
        if gano {
            self.ganadas += 1;
            self.marcador[k].0 += 1;
            self.vida.exito("¡GANASTE!", "Willy te debe una casa", Tono::Oro);
            let p = self.prota.pos + Vec3::new(0.0, 1.0, 0.0);
            self.jugo.estallar(p, [1.0, 0.84, 0.25], 70, 11.0);
            self.jugo.estallar(p, [1.0, 0.36, 0.7], 40, 9.0);
            self.jugo.onda(p, [1.0, 0.84, 0.25], 9.0);
            self.eventos.push(Efecto::Fuego);
            let frase = match self.modo {
                Modo::Parkour => "¡Increíble! ¡Le ganaste a Willy en el parkour! Willy, ya sabes: la casa. De tierra.",
                Modo::Tnt => "¡Willy se cayó primero! ¡Qué piernas, guapísimo! Eso es TNT Run.",
                _ => "¡Llegaste a la roca madre primero! Willy sigue picando tierra. Literal.",
            };
            self.decir(VEGETTA, frase, Voz::Neptuno);
        } else {
            self.marcador[k].1 += 1;
            self.vida.desastre("¡GANÓ WILLY!", "Dice que ya lo sabía");
            self.eventos.push(Efecto::Fallo);
            let frase = match self.modo {
                Modo::Parkour => "¡Te gané! Me debes una casa. Con piscina. De lava. Bueno, de agua.",
                Modo::Tnt => "¡Adiós, guapi! Te vi caer desde el piso de arriba. Fue precioso.",
                _ => "¡Roca madre! ¡Primero! Y con diamantes. Bueno, con carbón, pero cuenta.",
            };
            self.decir(WILLY, frase, Voz::Erizo);
        }
        self.cambiar(Fase::Fin(gano));
    }

    fn mover_willy_parkour(&mut self, dt: f32) {
        if self.willy_llego.is_some() {
            return;
        }
        if self.tropiezo > 0.0 {
            self.tropiezo -= dt;
            return;
        }
        self.s_willy += VEL_WILLY * dt;
        // A veces se tropieza (dos veces por carrera, mas o menos).
        let antes = ((self.s_willy - VEL_WILLY * dt) / 40.0) as i32;
        if (self.s_willy / 40.0) as i32 != antes && self.tropiezos < 3 {
            if self.azar() < 0.66 {
                self.tropiezos += 1;
                self.tropiezo = 1.6;
                let frase = TROPIEZOS[(self.azar() * 3.0) as usize % TROPIEZOS.len()];
                if self.fase == Fase::Jugando {
                    self.decir(WILLY, frase, Voz::Erizo);
                }
            }
        }
        if self.s_willy >= self.pista.largo {
            self.willy_llego = Some(self.tiempo);
            if self.fase == Fase::Jugando {
                self.decir(WILLY, "¡Llegué! ¡Primero! ¿Dónde estás, guapi?", Voz::Erizo);
                self.vida.grito("¡LLEGÓ WILLY!", Tono::Rojo, Tam::Medio);
                self.eventos.push(Efecto::Alarma);
            }
        }
    }

    // -----------------------------------------------------------------------
    // TNT Run
    // -----------------------------------------------------------------------

    fn tnt(&mut self, e: &Entrada, dt: f32) {
        if !self.prota_fuera {
            self.mover_prota(e, dt);
        }
        if !self.willy_fuera {
            let e_willy = self.cerebro_tnt();
            let mut w = self.willy.clone();
            w.mover(&e_willy, dt, &self.pista);
            self.willy = w;
        }
        // Lo que pisa cada quien empieza a caerse (con su TNT de abajo).
        for p in [(!self.prota_fuera).then_some(self.prota.pos), (!self.willy_fuera).then_some(self.willy.pos)].into_iter().flatten() {
            if let Some((c, Bloque::Arena)) = self.pista.bajo(p) {
                self.pisadas.entry(c).or_insert(TNT_ESPERA);
            }
        }
        let mut caen: Vec<(i32, i32, i32)> = Vec::new();
        for (c, t) in self.pisadas.iter_mut() {
            *t -= dt;
            if *t <= 0.0 {
                caen.push(*c);
            }
        }
        for c in caen {
            self.pisadas.remove(&c);
            for (dy, b) in [(0, Bloque::Arena), (-1, Bloque::Tnt)] {
                if self.pista.bloques.remove(&(c.0, c.1 + dy, c.2)).is_some() {
                    self.cayendo.push((b, Vec3::new(c.0 as f32 + 0.5, (c.1 + dy) as f32 + 0.5, c.2 as f32 + 0.5), 0.0));
                }
            }
            self.sucio = true;
            if self.azar() < 0.25 {
                self.eventos.push(Efecto::Pop);
            }
        }
        // Quien cae al vacio queda fuera.
        let abajo = TNT_PISOS[TNT_PISOS.len() - 1] as f32 - 8.0;
        if !self.prota_fuera && self.prota.pos.y < abajo {
            self.prota_fuera = true;
        }
        if !self.willy_fuera && self.willy.pos.y < abajo {
            self.willy_fuera = true;
        }
        match (self.prota_fuera, self.willy_fuera) {
            (true, _) => self.terminar(false),
            (false, true) => self.terminar(true),
            _ => {
                // Que no dure para siempre: a los 90 segundos gana el que este mas arriba.
                if self.tiempo > 90.0 {
                    let gano = self.prota.pos.y >= self.willy.pos.y;
                    self.terminar(gano);
                }
            }
        }
    }

    /// Willy en TNT Run: busca un lugar seguro cerca (con piso alrededor), corre hacia el y
    /// salta los huecos. De vez en cuando escoge mal.
    fn cerebro_tnt(&mut self) -> Entrada {
        let w = self.willy.clone();
        let piso_y = (w.pos.y - 0.05).floor() as i32;
        let llego = self.meta_willy.is_none_or(|m| Vec3::new(m.x - w.pos.x, 0.0, m.z - w.pos.z).length() < 0.7);
        let mal_lugar = self.meta_willy.is_some_and(|m| self.pista.bloque(m.x.floor() as i32, piso_y, m.z.floor() as i32).is_none());
        if llego || mal_lugar {
            let mut mejor: Option<(Vec3, f32)> = None;
            let (cx, cz) = (w.pos.x.floor() as i32, w.pos.z.floor() as i32);
            let error = self.azar() < 0.12;
            for _ in 0..20 {
                let dx = (self.azar() * 9.0) as i32 - 4;
                let dz = (self.azar() * 9.0) as i32 - 4;
                if dx.abs() + dz.abs() < 2 {
                    continue;
                }
                let (x, z) = (cx + dx, cz + dz);
                if self.pista.bloque(x, piso_y, z) != Some(Bloque::Arena) || self.pisadas.contains_key(&(x, piso_y, z)) {
                    continue;
                }
                let vecinos = (-1..=1).flat_map(|a| (-1..=1).map(move |b| (a, b))).filter(|&(a, b)| self.pista.bloque(x + a, piso_y, z + b).is_some()).count() as f32;
                let puntos = if error { self.azar() } else { vecinos + self.azar() };
                if mejor.is_none_or(|m| puntos > m.1) {
                    mejor = Some((Vec3::new(x as f32 + 0.5, w.pos.y, z as f32 + 0.5), puntos));
                }
            }
            self.meta_willy = mejor.map(|m| m.0).or(Some(w.pos + w.adelante() * 2.0));
        }
        let meta = self.meta_willy.unwrap_or(w.pos);
        w.ir_hacia(meta, &self.pista)
    }

    // -----------------------------------------------------------------------
    // Mineria
    // -----------------------------------------------------------------------

    /// El bloque que va a picar el protagonista: hacia el lado al que empuja (el de los pies y
    /// luego el de la cabeza, para poder pasar); si no empuja, el de abajo de sus pies.
    pub fn objetivo_pico(&self, e: &Entrada) -> Option<(i32, i32, i32)> {
        let p = self.prota.pos;
        let picable = |c: (i32, i32, i32)| self.pista.bloque(c.0, c.1, c.2).is_some_and(|b| b.dureza().is_finite());
        let lado = lado_mina(e);
        if lado.abs() > 0.3 {
            let x = (p.x + lado.signum() * 0.7).floor() as i32;
            for dy in [0.1, 1.0] {
                let c = (x, (p.y + dy).floor() as i32, p.z.floor() as i32);
                if picable(c) {
                    return Some(c);
                }
            }
        }
        let c = (p.x.floor() as i32, (p.y - 0.05).floor() as i32, p.z.floor() as i32);
        picable(c).then_some(c)
    }

    /// En la mina la camara ve los pozos de frente: se camina a la izquierda y a la derecha de la
    /// pantalla (y se salta, para subir un escalon).
    fn mover_minero(&mut self, e: &Entrada, dt: f32) {
        let lado = lado_mina(e);
        if lado.abs() > 0.2 {
            self.prota.yaw = -lado.signum() * PI * 0.5;
        }
        let mando = Entrada { empuje: lado.abs(), aceptar: e.aceptar, disparando: e.disparando, ..Default::default() };
        self.mover_prota(&mando, dt);
        self.prota.pos.z = 0.5;
    }

    fn mina(&mut self, e: &Entrada, dt: f32) {
        if self.quemado > 0.0 {
            self.quemado -= dt;
            self.picando = None;
        } else {
            self.mover_minero(e, dt);
            // Picar (mantener): avanza segun la dureza del bloque; al acabar, se rompe.
            let objetivo = if e.picar { self.objetivo_pico(e) } else { None };
            match (objetivo, self.picando) {
                (Some(c), Some((anterior, avance))) if c == anterior => {
                    let b = self.pista.bloque(c.0, c.1, c.2).unwrap_or(Bloque::Piedra);
                    let avance = avance + dt / b.dureza();
                    if (self.total * 7.0).fract() < dt * 7.0 {
                        self.eventos.push(Efecto::Teclas);
                        self.pedacitos(c, b, 2);
                    }
                    if avance >= 1.0 {
                        self.romper(c, true);
                        self.picando = None;
                    } else {
                        self.picando = Some((c, avance));
                    }
                }
                (Some(c), _) => self.picando = Some((c, 0.0)),
                (None, _) => self.picando = None,
            }
            // Caer en lava quema (un rato sin moverse) y la lava se vuelve obsidiana, que tarda
            // mucho en picarse: por eso conviene bajar por la columna de en medio.
            if self.prota.en_suelo {
                if let Some((c, Bloque::Lava)) = self.pista.bajo(self.prota.pos) {
                    self.pista.poner(c.0, c.1, c.2, Bloque::Obsidiana);
                    self.sucio = true;
                    self.quemado = 1.4;
                    self.prota.vel = Vec3::zero();
                    self.pedacitos(c, Bloque::Lava, 12);
                    self.anunciar("¡TE QUEMASTE!");
                    self.eventos.push(Efecto::Fuego);
                    self.decir(WILLY, "¡Jajaja! ¡Lava! Eso pasa por andar de codicioso, guapi.", Voz::Erizo);
                }
            }
        }
        self.willy_mina(dt);
        // Llegar a la roca madre.
        let en_fondo = |p: Vec3| p.y <= FONDO as f32 + 1.05;
        if en_fondo(self.prota.pos) {
            self.terminar(true);
        } else if en_fondo(self.willy.pos) {
            self.terminar(false);
        } else if self.tiempo > 150.0 {
            let gano = self.prota.pos.y <= self.willy.pos.y;
            self.terminar(gano);
        }
    }

    /// Rompe el bloque `c` (de quien pica: el protagonista o Willy): pedacitos y, si era mena,
    /// el premio.
    fn romper(&mut self, c: (i32, i32, i32), prota: bool) {
        let Some(b) = self.pista.bloques.remove(&c) else { return };
        self.sucio = true;
        self.pedacitos(c, b, 10);
        if !prota {
            if b == Bloque::MenaDiamante {
                self.diamantes_willy += 1;
            }
            return;
        }
        match b {
            Bloque::MenaDiamante => {
                self.diamantes_mina += 1;
                self.anunciar("¡DIAMANTE!");
                let p = Vec3::new(c.0 as f32 + 0.5, c.1 as f32 + 0.5, c.2 as f32 + 0.5);
                self.jugo.estallar(p, [0.35, 1.0, 0.95], 34, 7.0);
                self.jugo.onda(p, [0.35, 1.0, 0.95], 3.5);
                self.eventos.push(Efecto::Perfecto);
            }
            Bloque::MenaOro => self.eventos.push(Efecto::Nota3),
            _ => self.eventos.push(Efecto::Carta),
        }
    }

    /// Willy en la mina: pica derecho hacia abajo por la columna de en medio, un poco mas lento,
    /// y a veces se distrae (o se mete a la lava por una mena).
    fn willy_mina(&mut self, dt: f32) {
        let mut w = self.willy.clone();
        w.mover(&Entrada::default(), dt, &self.pista);
        w.pos.z = 0.5;
        self.willy = w;
        if !self.willy.en_suelo {
            return;
        }
        // Aturdido (quemado o distraido): no pica.
        if self.willy_pica < 0.0 {
            self.willy_pica += dt;
            return;
        }
        let p = self.willy.pos;
        let pies = (p.x.floor() as i32, (p.y - 0.05).floor() as i32, 0);
        let Some(b) = self.pista.bloque(pies.0, pies.1, pies.2) else { return };
        if b == Bloque::Lava {
            self.pista.poner(pies.0, pies.1, pies.2, Bloque::Obsidiana);
            self.sucio = true;
            self.pedacitos(pies, Bloque::Lava, 12);
            self.willy_pica = -1.4;
            self.decir(WILLY, "¡AY! ¡Lava! No pasó nada, no pasó nada...", Voz::Erizo);
            return;
        }
        if !b.dureza().is_finite() {
            return;
        }
        self.willy_pica += dt / (b.dureza() * LENTITUD_WILLY);
        if (self.total * 6.0).fract() < dt * 6.0 {
            self.pedacitos(pies, b, 1);
        }
        if self.willy_pica >= 1.0 {
            self.willy_pica = 0.0;
            self.romper(pies, false);
            let r = self.azar();
            if r < 0.07 {
                // Se distrae.
                self.willy_pica = -1.5;
                let frase = ["¡Mira, un murciélago!", "Espérate, que se me cayó la antorcha.", "¿Escuchaste eso? Creo que es un creeper..."][(self.azar() * 3.0) as usize % 3];
                self.decir(WILLY, frase, Voz::Erizo);
            } else if r < 0.14 {
                // Ve una mena en la columna de al lado y se cambia (ahi puede haber lava).
                let otra = if self.azar() < 0.5 { -1 } else { 1 };
                let x = POZO_WILLY + otra;
                if self.pista.bloque(x, pies.1, 0).is_none() && self.pista.bloque(x, pies.1 + 1, 0).is_none() {
                    self.willy.pos.x = x as f32 + 0.5;
                }
            }
        }
    }

    /// Unos pedacitos que saltan del bloque `c`.
    fn pedacitos(&mut self, c: (i32, i32, i32), b: Bloque, cuantos: usize) {
        for _ in 0..cuantos {
            let pos = Vec3::new(c.0 as f32 + 0.2 + self.azar() * 0.6, c.1 as f32 + 0.6 + self.azar() * 0.5, c.2 as f32 + 0.2 + self.azar() * 0.6);
            let vel = Vec3::new(self.azar() * 4.0 - 2.0, 2.0 + self.azar() * 3.0, self.azar() * 4.0 - 2.0);
            let tam = 0.1 + self.azar() * 0.1;
            let vida = 0.7 + self.azar() * 0.4;
            self.particulas.push(Particula { pos, vel, bloque: b, vida, tam });
        }
        if self.particulas.len() > 160 {
            let sobra = self.particulas.len() - 160;
            self.particulas.drain(..sobra);
        }
    }

    fn cayendo_y_particulas(&mut self, dt: f32) {
        for c in &mut self.cayendo {
            c.2 += GRAVEDAD * 0.8 * dt;
            c.1.y -= c.2 * dt;
        }
        self.cayendo.retain(|c| c.1.y > -40.0);
        for p in &mut self.particulas {
            p.vel.y -= GRAVEDAD * dt;
            p.pos += p.vel * dt;
            p.vida -= dt;
        }
        self.particulas.retain(|p| p.vida > 0.0);
    }

    // -----------------------------------------------------------------------
    // La camara
    // -----------------------------------------------------------------------

    fn mover_camara(&mut self, dt: f32) {
        let p = self.prota.pos;
        let (ojo, mira) = match (self.fase, self.modo) {
            (Fase::Llegada(_), _) => {
                // Al llegar: dando la vuelta a la isla del lobby (desde afuera).
                let a = self.total * 0.22 + PI;
                (Vec3::new(a.sin() * 17.0, 8.0, a.cos() * 17.0), Vec3::new(0.0, 1.0, 1.0))
            }
            (_, Modo::Mineria) => {
                // De lado, por el vidrio: los dos pozos a la vista, siguiendo la profundidad.
                let y = p.y * 0.75 + self.willy.pos.y * 0.25;
                (Vec3::new(0.0, y + 2.2, 9.5), Vec3::new(0.0, y - 0.8, 0.0))
            }
            (_, Modo::Tnt) => (p - self.prota.adelante() * 4.6 + Vec3::new(0.0, 3.4, 0.0), p + Vec3::new(0.0, 0.6, 0.0) + self.prota.adelante() * 2.0),
            _ => (p - self.prota.adelante() * 4.2 + Vec3::new(0.0, 2.3, 0.0), p + Vec3::new(0.0, 1.0, 0.0) + self.prota.adelante() * 2.5),
        };
        let k = 1.0 - (-6.0 * dt).exp();
        self.camara = (self.camara.0.lerp(ojo, k), self.camara.1.lerp(mira, k));
    }

    // -----------------------------------------------------------------------
    // Para las pruebas y capturas
    // -----------------------------------------------------------------------

    /// Corre solo el parkour (siguiendo el camino de Willy, a saltos) un rato.
    pub fn correr_solo(&mut self, segundos: f32) {
        if self.modo != Modo::Parkour {
            self.empezar(Modo::Parkour);
        }
        self.mensaje = None;
        self.cambiar(Fase::Jugando);
        let mut t = 0.0;
        while t < segundos && self.fase == Fase::Jugando {
            let e = self.entrada_parkour();
            self.actualizar(&e, 1.0 / 60.0);
            t += 1.0 / 60.0;
        }
        let p = &self.prota;
        self.camara = (p.pos - p.adelante() * 4.2 + Vec3::new(0.0, 2.3, 0.0), p.pos + Vec3::new(0.0, 1.0, 0.0) + p.adelante() * 2.5);
    }

    /// Si se esta corriendo el parkour (para que la grabacion lo juegue sola).
    pub fn en_parkour(&self) -> bool {
        self.modo == Modo::Parkour && self.fase == Fase::Jugando
    }

    /// Si se esta jugando la mineria.
    pub fn en_mina(&self) -> bool {
        self.modo == Modo::Mineria && self.fase == Fase::Jugando
    }

    /// Lo que apretaria quien corre el parkour solo: sigue el camino de Willy, a saltos.
    pub fn entrada_parkour(&self) -> Entrada {
        let p = &self.prota;
        let siguiente = self.pista.camino.iter().filter(|c| c.1).map(|c| c.0).find(|c| c.z > p.pos.z + 0.8).unwrap_or(p.pos + Vec3::new(0.0, 0.0, 1.0));
        let a = siguiente - p.pos;
        let quiere = (-a.x).atan2(-a.z);
        let giro = (quiere - p.yaw + PI).rem_euclid(TAU) - PI;
        let adelante = p.pos + p.adelante() * 0.45;
        let piso = |d: f32| {
            let q = p.pos + p.adelante() * d;
            self.pista.bloque(q.x.floor() as i32, (p.pos.y - 0.05).floor() as i32, q.z.floor() as i32)
        };
        let sin_piso = piso(0.45).is_none_or(|b| b == Bloque::Lava) || piso(0.9) == Some(Bloque::Lava);
        let escalon = self.pista.solido(adelante.x.floor() as i32, (p.pos.y + 0.1).floor() as i32, adelante.z.floor() as i32);
        let derecho = giro.abs() < 0.35;
        let hueco_largo = self.pista.plataformas.iter().find(|q| q.z0 as f32 > p.pos.z + 0.5).is_some_and(|q| {
            let antes = self.pista.plataformas.iter().filter(|r| (r.z1 as f32) < q.z0 as f32).map(|r| r.z1).max().unwrap_or(0);
            q.z0 - antes > 3
        });
        let cerca = !p.en_suelo && Vec3::new(a.x, 0.0, a.z).length() < 0.9;
        let empuje = if cerca || (!derecho && sin_piso) { 0.0 } else { 1.0 };
        Entrada { empuje, turbo: hueco_largo, girar: (giro * 3.0).clamp(-1.0, 1.0), aceptar: p.en_suelo && derecho && (sin_piso || escalon), disparando: true, ..Default::default() }
    }

    /// Para las capturas: `vegetta` (la llegada), `vegetta_lobby`, `vegetta_tnt` (a media
    /// partida) y `vegetta_mina` (ya bajando).
    pub fn preparar_captura(&mut self, modo: &str) {
        self.mensaje = None;
        match modo {
            "vegetta_lobby" => {
                self.cambiar(Fase::Libre);
                self.prota.pos = Vec3::new(0.5, 1.0, -3.0);
                self.prota.yaw = PI;
            }
            "vegetta_tnt" => {
                self.empezar(Modo::Tnt);
                self.mensaje = None;
                self.cambiar(Fase::Jugando);
                // Corriendo en circulos a media arena (y Willy por su cuenta): ya agujereada.
                self.prota = Cuerpo::new(Vec3::new(0.5, 1.0, 3.0), -PI * 0.5);
                for _ in 0..50 {
                    let e = Entrada { empuje: 1.0, girar: 0.55, disparando: true, ..Default::default() };
                    self.actualizar(&e, 1.0 / 60.0);
                }
            }
            "vegetta_mina" => {
                self.empezar(Modo::Mineria);
                self.mensaje = None;
                self.cambiar(Fase::Jugando);
                for _ in 0..(60 * 9) {
                    self.actualizar(&Entrada { picar: true, ..Default::default() }, 1.0 / 60.0);
                }
            }
            _ => {}
        }
        let p = &self.prota;
        if self.modo != Modo::Mineria {
            self.camara = (p.pos - p.adelante() * 4.6 + Vec3::new(0.0, 3.0, 0.0), p.pos + Vec3::new(0.0, 0.8, 0.0) + p.adelante() * 2.0);
        } else {
            for _ in 0..60 {
                self.mover_camara(1.0 / 30.0);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Para dibujar
    // -----------------------------------------------------------------------

    pub fn camara(&self) -> (Vec3, Vec3) {
        self.camara
    }

    pub fn matriz_prota(&self) -> Mat4 {
        let quemado = if self.quemado > 0.0 { Mat4::rotation_y(self.quemado * 12.0) } else { Mat4::identity() };
        Mat4::translation(self.prota.pos) * Mat4::rotation_y(self.prota.yaw) * quemado
    }

    pub fn en_suelo(&self) -> bool {
        self.prota.en_suelo
    }

    pub fn paso(&self) -> f32 {
        self.prota.paso
    }

    /// Willy: en el parkour, por su camino (y tumbado si se tropezo); en los otros, su cuerpo.
    pub fn willy(&self) -> (Mat4, f32) {
        match self.modo {
            Modo::Parkour => {
                let (p, dir) = self.pista.en_camino(self.s_willy);
                let giro = (-dir.x).atan2(-dir.z);
                let tumbado = if self.tropiezo > 0.0 { Mat4::rotation_x(-1.2 * (self.tropiezo / 1.6).min(1.0)) } else { Mat4::identity() };
                (Mat4::translation(p) * Mat4::rotation_y(giro) * tumbado, self.s_willy * 3.0)
            }
            _ => (Mat4::translation(self.willy.pos) * Mat4::rotation_y(self.willy.yaw), self.willy.paso * 3.2 + if self.modo == Modo::Mineria { self.total * 6.0 } else { 0.0 }),
        }
    }

    /// Vegetta: en el lobby recibiendo; en el parkour, en la meta echando porras; en los otros,
    /// mirando desde arriba.
    pub fn vegetta(&self) -> Mat4 {
        let p = match self.modo {
            Modo::Lobby => Vec3::new(-2.5, 1.0, -1.0),
            Modo::Parkour if !matches!(self.fase, Fase::Cuenta) => self.pista.camino.last().map_or(Vec3::zero(), |c| c.0) + Vec3::new(1.5, 0.0, 1.0),
            Modo::Parkour => Vec3::new(0.0, 1.0, 1.5),
            Modo::Tnt => Vec3::new(0.5, 4.0, -TNT_MEDIO as f32 - 2.5),
            Modo::Mineria => Vec3::new(0.5, 1.0, 0.5),
        };
        // En el lobby y el parkour ve hacia donde llega uno; en los otros, hacia la partida.
        let mira = if matches!(self.modo, Modo::Lobby | Modo::Parkour) { 0.0 } else { PI };
        Mat4::translation(p) * Mat4::rotation_y(mira)
    }

    pub fn matrices_diamantes(&self) -> Vec<Mat4> {
        if self.modo != Modo::Parkour {
            return Vec::new();
        }
        self.pista
            .diamantes
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.tomados.get(*i).copied().unwrap_or(true))
            .map(|(i, d)| Mat4::translation(*d + Vec3::new(0.0, 0.1 * (self.total * 2.0 + i as f32).sin() - 0.25, 0.0)) * Mat4::rotation_y(self.total * 1.6 + i as f32) * Mat4::scale(1.1))
            .collect()
    }

    /// Los diamantes que se lleva (del parkour y de la mina): se cambian por notas al despegar.
    pub fn diamantes(&self) -> usize {
        self.tomados.iter().filter(|t| **t).count() + self.diamantes_mina as usize
    }

    /// Que tanto lleva cada quien del parkour (protagonista, Willy), de 0 a 1.
    pub fn avance(&self) -> (f32, f32) {
        let fin = self.pista.camino.last().map_or(1.0, |c| c.0.z).max(1.0);
        ((self.prota.pos.z / fin).clamp(0.0, 1.0), (self.s_willy / self.pista.largo.max(1.0)).clamp(0.0, 1.0))
    }

    pub fn en_meta(&self) -> bool {
        matches!(self.fase, Fase::Fin(true))
    }

    /// Los colores del cielo (abajo, medio, arriba): de dia en el parkour, atardecer en TNT Run,
    /// de noche en la mina; en el lobby, el dia pasa (un ciclo cada dos minutos).
    pub fn cielo(&self) -> [[f32; 3]; 3] {
        let dia = [[0.62, 0.82, 1.0], [0.42, 0.68, 1.0], [0.16, 0.36, 0.92]];
        let tarde = [[1.0, 0.72, 0.45], [0.95, 0.5, 0.45], [0.35, 0.25, 0.55]];
        let noche = [[0.10, 0.10, 0.22], [0.05, 0.05, 0.14], [0.02, 0.02, 0.06]];
        match self.modo {
            Modo::Parkour => dia,
            Modo::Tnt => tarde,
            Modo::Mineria => noche,
            Modo::Lobby => {
                let fase = (self.total / 120.0 * TAU).cos() * 0.5 + 0.5;
                let mezcla = |a: [[f32; 3]; 3], b: [[f32; 3]; 3], k: f32| std::array::from_fn(|i| std::array::from_fn(|j| a[i][j] + (b[i][j] - a[i][j]) * k));
                if fase > 0.5 {
                    mezcla(tarde, dia, (fase - 0.5) * 2.0)
                } else {
                    mezcla(noche, tarde, fase * 2.0)
                }
            }
        }
    }

    /// Bajo tierra (en la mina) solo alumbra la antorcha del protagonista.
    pub fn bajo_tierra(&self) -> bool {
        self.modo == Modo::Mineria
    }

    pub fn hablante(&self) -> Option<(&'static str, String, Voz)> {
        self.mensaje.as_ref().map(|(q, t, v, _)| (*q, t.clone(), *v))
    }

    pub fn despedida(&self) -> String {
        let (tu, willy): (u32, u32) = self.marcador.iter().fold((0, 0), |a, m| (a.0 + m.0, a.1 + m.1));
        if tu > willy {
            format!("Despegue de Planeta Vegetta. Le ganamos a Willyrex {tu} a {willy}. Nos debe una casa de tierra. Con ventanas, espero.")
        } else if self.terminadas > 0 {
            format!("Despegue de Planeta Vegetta. Willyrex {willy}, nosotros {tu}. Ahora hay que construirle una casa. Yo no tengo manos. Le toca a usted.")
        } else {
            "Despegue de Planeta Vegetta. Nos fuimos sin entrar a ningún portal. Willy dice que fue por miedo. Fue por miedo.".into()
        }
    }

    /// Donde va un letrero sobre cada portal (para la interfaz): el punto en el mundo, el nombre
    /// y el marcador de ese minijuego.
    pub fn letreros_portales(&self) -> Vec<(Vec3, &'static str, (u32, u32))> {
        if self.modo != Modo::Lobby {
            return Vec::new();
        }
        PORTALES.iter().map(|&(m, cx)| (Vec3::new(cx as f32, 5.6, 6.5), m.nombre(), self.marcador[m.indice()])).collect()
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, ancho: i32, alto: i32, en_ventana: &dyn Fn(Vec3) -> Option<(f32, f32)>) {
        let h = alto as f32;
        let (aceptar, salir) = (e.nombre("aceptar"), e.nombre("aterrizar"));
        if matches!(self.fase, Fase::Llegada(0)) {
            let a = ((self.t / 0.8).min(1.0) * 255.0) as u8;
            let t1 = "PLANETA VEGETTA";
            let tam = (h * 0.08) as i32;
            ui::titulo_de(d, t1, (ancho - ui::medir_titulo(t1, tam)) / 2, (h * 0.12) as i32, tam, Color::new(190, 120, 255, a));
            let t2 = "el servidor de minijuegos · contra Willyrex";
            ui::subtitulo(d, t2, (ancho - ui::medir(t2, 24)) / 2, (h * 0.12) as i32 + tam + 14, 24, Color::new(255, 255, 255, a));
        }
        // Los letreros de los portales (y el marcador de cada uno).
        for (p, nombre, (tu, willy)) in self.letreros_portales() {
            if let Some((x, y)) = en_ventana(p) {
                ui::titulo_de(d, nombre, x as i32 - ui::medir_titulo(nombre, 20) / 2, y as i32 - 28, 20, Color::new(255, 240, 150, 255));
                let m = format!("tú {tu} · Willy {willy}");
                ui::subtitulo(d, &m, x as i32 - ui::medir(&m, 18) / 2, y as i32 - 2, 18, ui::TEXTO);
            }
        }
        // Arriba: lo de cada minijuego.
        if self.modo != Modo::Lobby && !matches!(self.fase, Fase::Llegada(_)) {
            ui::panel(d, 20, 20, 300, 96);
            let tiempo = format!("{}:{:05.2}", (self.tiempo / 60.0) as i32, self.tiempo % 60.0);
            ui::texto(d, &tiempo, 36, 30, 30, ui::TEXTO);
            let sub = match self.modo {
                Modo::Parkour => format!("DIAMANTES {}/{}", self.tomados.iter().filter(|t| **t).count(), self.pista.diamantes.len()),
                Modo::Tnt => format!("PISO {} · Willy en el {}", piso_tnt(self.prota.pos.y), piso_tnt(self.willy.pos.y)),
                _ => format!("DIAMANTES {} · Willy {}", self.diamantes_mina, self.diamantes_willy),
            };
            ui::texto(d, &sub, 36, 70, 18, Color::new(120, 240, 230, 255));
            let (bx, by, bw) = (ancho / 2 - 220, 34, 440);
            // El avance de los dos: en el parkour, por la pista; en la mina, que tan hondo van.
            let (yo, willy) = match self.modo {
                Modo::Parkour => self.avance(),
                Modo::Mineria => ((-self.prota.pos.y / -(FONDO as f32 + 1.0)).clamp(0.0, 1.0), (-self.willy.pos.y / -(FONDO as f32 + 1.0)).clamp(0.0, 1.0)),
                _ => (0.0, 0.0),
            };
            if self.modo != Modo::Tnt {
                d.draw_rectangle(bx, by + 10, bw, 6, Color::new(40, 30, 60, 220));
                d.draw_rectangle(bx + bw - 8, by, 8, 26, Color::new(240, 240, 240, 255));
                let marca = |d: &mut RaylibDrawHandle, k: f32, c: Color, etiqueta: &str, arriba: bool| {
                    let x = bx + (bw as f32 * k) as i32;
                    d.draw_circle(x, by + 13, 9.0, c);
                    let y = if arriba { by - 14 } else { by + 26 };
                    ui::subtitulo(d, etiqueta, x - ui::medir(etiqueta, 14) / 2, y, 14, c);
                };
                marca(d, willy, Color::new(255, 90, 90, 255), "Willy", true);
                marca(d, yo, Color::new(150, 255, 170, 255), ui::protagonista().nombre, false);
            }
        }
        if self.fase == Fase::Cuenta {
            let n = 3 - self.t.floor() as i32;
            if n > 0 {
                let s = n.to_string();
                let tam = (h * 0.22) as i32;
                ui::titulo_de(d, &s, (ancho - ui::medir_titulo(&s, tam)) / 2, (h * 0.32) as i32, tam, Color::new(255, 230, 90, 255));
            }
        }
        // Las ayudas de abajo, segun donde se este.
        let mover = if e.hay_control { "stick" } else { "W / A / S / D" };
        let acciones: Vec<(&str, &str)> = match (self.fase, self.modo) {
            (Fase::Llegada(_), _) => vec![(aceptar, "Seguir"), (salir, "Despegar")],
            (Fase::Fin(_), _) => vec![(aceptar, "Volver al lobby"), (salir, "Despegar")],
            (_, Modo::Lobby) => vec![(mover, "Caminar a un portal"), (aceptar, "Saltar"), (salir, "Despegar")],
            (_, Modo::Mineria) => vec![(mover, "Moverse"), (e.nombre("picar"), "Picar (mantener)"), (aceptar, "Saltar"), (salir, "Despegar")],
            _ => vec![(mover, "Correr y girar"), (e.nombre("turbo"), "Más rápido"), (aceptar, "Saltar"), (salir, "Despegar")],
        };
        let pie = ui::alto_ayudas(&acciones, ancho);
        if let (Some((nombre, completo, _)), false) = (self.hablante(), texto.is_empty()) {
            let wd = 880.min(ancho - 40);
            let hd = ui::alto_dialogo(nombre, &completo, wd);
            let color = match nombre {
                VEGETTA => Color::new(200, 130, 255, 255),
                WILLY => Color::new(255, 110, 110, 255),
                _ => ui::LILA,
            };
            ui::caja_dialogo(d, nombre, color, texto, (ancho - wd) / 2, alto - pie - hd - 12, wd);
        }
        ui::ayudas(d, &acciones, ancho, alto);
    }
}

/// Hacia donde empuja (en la pantalla) en la mina: 1 a la derecha, -1 a la izquierda.
fn lado_mina(e: &Entrada) -> f32 {
    (-e.girar).clamp(-1.0, 1.0)
}

/// En que piso de TNT Run esta alguien a la altura `y` (1 el de arriba; 0 si ya cayo).
fn piso_tnt(y: f32) -> usize {
    TNT_PISOS.iter().position(|&p| y >= p as f32 + 0.5).map_or(0, |i| i + 1)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn la_pista_tiene_meta_y_diamantes() {
        for semilla in 1..20 {
            let p = Pista::generar(semilla * 31);
            assert!(p.bloques.values().any(|b| *b == Bloque::Meta));
            assert!(!p.diamantes.is_empty());
            assert!(p.largo > 50.0);
            assert!(p.bloques.values().filter(|b| **b == Bloque::Vegetta).count() >= 3, "puntos de control");
        }
    }

    #[test]
    fn se_para_en_el_piso_y_salta() {
        let mut v = Vegetta::new(5, None);
        v.empezar(Modo::Parkour);
        v.cambiar(Fase::Jugando);
        for _ in 0..30 {
            v.actualizar(&Entrada::default(), 1.0 / 60.0);
        }
        assert!(v.prota.en_suelo, "se queda parado en la isla de salida");
        assert!((v.prota.pos.y - 1.0).abs() < 0.01);
        v.actualizar(&Entrada { aceptar: true, ..Default::default() }, 1.0 / 60.0);
        v.actualizar(&Entrada::default(), 0.1);
        assert!(v.prota.pos.y > 1.3, "salto");
    }

    #[test]
    fn caerse_regresa_al_punto_de_control() {
        let mut v = Vegetta::new(7, None);
        v.empezar(Modo::Parkour);
        v.cambiar(Fase::Jugando);
        v.prota.pos = Vec3::new(40.0, 1.0, 40.0);
        for _ in 0..200 {
            v.actualizar(&Entrada::default(), 1.0 / 60.0);
        }
        assert_eq!(v.caidas, 1);
        assert!(v.tiempo >= CASTIGO);
    }

    /// Cada salto de una isla a la siguiente se puede hacer.
    #[test]
    fn todos_los_saltos_se_pueden() {
        for semilla in 1..40 {
            let p = Pista::generar(semilla * 97);
            for w in p.plataformas.windows(2) {
                let (a, b) = (w[0], w[1]);
                let hueco = b.z0 - a.z1 - 1;
                let sube = b.y - a.y;
                let slime = p.bloque(a.x0, a.y, a.z0) == Some(Bloque::Slime);
                assert!(hueco >= 0, "semilla {semilla}: islas encimadas");
                if slime {
                    assert!(sube <= 3 && hueco <= 1, "semilla {semilla}: el slime no alcanza");
                } else {
                    assert!(sube <= 1, "semilla {semilla}: sube {sube}");
                    let maximo = match sube {
                        0 => 3,
                        1 => 2,
                        _ => 1,
                    };
                    assert!(hueco <= maximo, "semilla {semilla}: hueco de {hueco} subiendo {sube}");
                }
            }
        }
    }

    /// En el lobby, caminar al portal del medio lleva a TNT Run.
    #[test]
    fn el_portal_lleva_al_minijuego() {
        let mut v = Vegetta::new(3, None);
        v.cambiar(Fase::Libre);
        v.prota = Cuerpo::new(Vec3::new(0.0, 1.0, 3.0), PI);
        for _ in 0..240 {
            v.actualizar(&Entrada { empuje: 1.0, ..Default::default() }, 1.0 / 60.0);
            if v.modo != Modo::Lobby {
                break;
            }
        }
        assert_eq!(v.modo, Modo::Tnt);
    }

    /// En TNT Run lo pisado se cae un instante despues (con su TNT), y quedarse quieto es caer.
    #[test]
    fn en_tnt_run_lo_pisado_se_cae() {
        let mut v = Vegetta::new(3, None);
        v.empezar(Modo::Tnt);
        v.cambiar(Fase::Jugando);
        let bajo = v.pista.bajo(v.prota.pos).map(|b| b.0);
        assert!(bajo.is_some());
        let c = bajo.unwrap();
        for _ in 0..60 {
            v.actualizar(&Entrada::default(), 1.0 / 60.0);
        }
        assert!(v.pista.bloque(c.0, c.1, c.2).is_none(), "la arena se cayo");
        assert!(v.pista.bloque(c.0, c.1 - 1, c.2).is_none(), "y su TNT tambien");
        // Quieto, va cayendo de piso en piso hasta quedar fuera: pierde.
        for _ in 0..(60 * 20) {
            v.actualizar(&Entrada::default(), 1.0 / 60.0);
            if matches!(v.fase, Fase::Fin(_)) {
                break;
            }
        }
        assert_eq!(v.fase, Fase::Fin(false));
        assert_eq!(v.marcador[1], (0, 1));
    }

    /// En la mina, picar hacia abajo baja de bloque en bloque hasta la roca madre.
    #[test]
    fn picando_se_llega_a_la_roca_madre() {
        let mut v = Vegetta::new(3, None);
        v.empezar(Modo::Mineria);
        v.cambiar(Fase::Jugando);
        // Una columna sin lava (la del centro nunca tiene) y Willy muy lento (para que no gane).
        v.prota.pos = Vec3::new(POZO_PROTA as f32 + 0.5, 0.0, 0.5);
        let mut antes = v.prota.pos.y;
        for _ in 0..(60 * 140) {
            v.willy_pica = 0.0;
            v.actualizar(&Entrada { picar: true, ..Default::default() }, 1.0 / 60.0);
            assert!(v.prota.pos.y <= antes + 3.01, "subio de mas");
            antes = v.prota.pos.y;
            if matches!(v.fase, Fase::Fin(_)) {
                break;
            }
        }
        assert_eq!(v.fase, Fase::Fin(true), "llego al fondo a y = {}", v.prota.pos.y);
    }
}
