//! Los asteroides que pasan cerca de la nave mientras se vuela.
//!
//! Siempre hay unas cuantas rocas alrededor (las que se quedan muy atras se
//! vuelven a poner por delante, como el polvo). Hay de tres clases:
//! - grises: solo estorban; al chocar, la nave rebota.
//! - rojas (calientes): al chocar se caen notas. Quedan regadas un ratito y se
//!   pueden volver a recoger, como los anillos de Sonic.
//! - doradas: traen un dato del universo. No hay que chocarlas: basta con pasar
//!   cerca y Bill Nye, el chico de la ciencia, lo cuenta.

use crate::colision::RADIO_NAVE;
use crate::matematica::{Mat4, Vec3};
use crate::sonido::Efecto;
use std::f32::consts::TAU;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clase {
    Gris,
    Roja,
    Dorada,
}

pub struct Roca {
    pub pos: Vec3,
    vel: Vec3,
    eje: Vec3,
    giro: f32,
    pub escala: f32,
    /// Cual de los tres modelos de roca.
    pub modelo: usize,
    pub clase: Clase,
    /// La dorada ya conto su dato (se apaga) / la roja ya pego (se enfria un rato).
    usada: bool,
}

/// Cuantas rocas hay alrededor y entre que distancias aparecen.
const ROCAS: usize = 22;
const CERCA: f32 = 45.0;
const LEJOS: f32 = 150.0;
/// Que tan cerca hay que pasar de una dorada para que cuente su dato.
const RADIO_DATO: f32 = 12.0;
/// Cada cuantos segundos llega una dorada.
const CADA_DORADA: f32 = 35.0;

pub struct Campo {
    pub rocas: Vec<Roca>,
    semilla: u32,
    hasta_dorada: f32,
    /// Lo que paso este cuadro.
    pub choque: Option<Clase>,
    /// Si se paso junto a una dorada (hay que contar un dato).
    pub dato: bool,
    pub eventos: Vec<Efecto>,
    /// Donde fue el ultimo choque con una roja (ahi se riegan las notas).
    pub donde: Vec3,
}

impl Campo {
    pub fn new() -> Self {
        Campo { rocas: Vec::new(), semilla: 777, hasta_dorada: CADA_DORADA * 0.6, choque: None, dato: false, eventos: Vec::new(), donde: Vec3::zero() }
    }

    fn azar(&mut self) -> f32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Una roca nueva cerca de la nave, mas bien por delante (de donde va a venir).
    fn nueva(&mut self, nave: Vec3, adelante: Vec3, clase: Clase) -> Roca {
        let a = self.azar() * TAU;
        let b = (self.azar() - 0.5) * 1.2;
        let dir = Vec3::new(a.cos() * b.cos(), b.sin() * 0.5, a.sin() * b.cos());
        // Dos de cada tres, adelante de la nave.
        let dir = if self.azar() < 0.66 { (dir + adelante * 1.5).normalize() } else { dir };
        let dist = CERCA + (LEJOS - CERCA) * self.azar();
        let deriva = Vec3::new(self.azar() - 0.5, (self.azar() - 0.5) * 0.3, self.azar() - 0.5) * 6.0;
        let (escala, vel) = match clase {
            Clase::Dorada => (1.4, deriva * 0.3),
            Clase::Roja => (1.2 + 1.3 * self.azar(), deriva),
            Clase::Gris => (1.0 + 2.6 * self.azar(), deriva),
        };
        Roca {
            pos: nave + dir * dist,
            vel,
            eje: Vec3::new(self.azar() - 0.5, self.azar() - 0.5, self.azar() - 0.5).normalize(),
            giro: self.azar() * TAU,
            escala,
            modelo: (self.azar() * 3.0) as usize % 3,
            clase,
            usada: false,
        }
    }

    fn clase_al_azar(&mut self) -> Clase {
        if self.azar() < 0.22 { Clase::Roja } else { Clase::Gris }
    }

    /// Un cuadro. `activo`: si se vuela libre (no en el hiperespacio ni aterrizando).
    /// `escudo`: si la nave trae el escudo de Europa (las rojas no le hacen nada).
    pub fn actualizar(&mut self, nave: &mut Vec3, adelante: Vec3, dt: f32, activo: bool, escudo: bool) {
        self.choque = None;
        self.dato = false;
        if !activo {
            return;
        }
        while self.rocas.len() < ROCAS {
            let c = self.clase_al_azar();
            let r = self.nueva(*nave, adelante, c);
            self.rocas.push(r);
        }
        self.hasta_dorada -= dt;
        if self.hasta_dorada <= 0.0 {
            self.hasta_dorada = CADA_DORADA;
            // La dorada pasa cerquita por delante, para que se note.
            let mut r = self.nueva(*nave, adelante, Clase::Dorada);
            r.pos = *nave + adelante * 70.0 + Vec3::new(self.azar() - 0.5, 0.0, self.azar() - 0.5) * 20.0;
            r.vel = (*nave - r.pos).normalize() * 2.0;
            if let Some(i) = self.rocas.iter().position(|r| r.clase == Clase::Gris) {
                self.rocas[i] = r;
            }
        }
        for i in 0..self.rocas.len() {
            let lejos = (self.rocas[i].pos - *nave).length() > LEJOS * 1.25;
            if lejos {
                let c = if self.rocas[i].clase == Clase::Dorada && !self.rocas[i].usada { Clase::Dorada } else { self.clase_al_azar() };
                self.rocas[i] = self.nueva(*nave, adelante, c);
                continue;
            }
            let r = &mut self.rocas[i];
            r.pos = r.pos + r.vel * dt;
            r.giro += dt * 0.6;
            let d = (r.pos - *nave).length();
            match r.clase {
                Clase::Dorada if !r.usada && d < RADIO_DATO => {
                    r.usada = true;
                    self.dato = true;
                    self.eventos.push(Efecto::Sello);
                }
                Clase::Gris | Clase::Roja => {
                    let radio = r.escala * 1.05 + RADIO_NAVE;
                    if d < radio {
                        // La nave se empuja afuera de la roca.
                        let fuera = (*nave - r.pos).normalize();
                        *nave = r.pos + fuera * radio;
                        r.vel = r.vel - fuera * 4.0;
                        let calor = r.clase == Clase::Roja && !r.usada && !escudo;
                        if r.clase == Clase::Roja {
                            // Despues de pegar, se enfria: ya no vuelve a tumbar notas.
                            r.usada = true;
                        }
                        self.donde = *nave;
                        self.choque = Some(if calor { Clase::Roja } else { Clase::Gris });
                        self.eventos.push(if calor { Efecto::Fuego } else { Efecto::Choque });
                    }
                }
                _ => {}
            }
        }
    }

    /// Las rocas cerca de `ojo` para dibujar: (matriz, modelo, clase, encendida).
    pub fn visibles(&self, ojo: Vec3) -> Vec<(Mat4, usize, Clase, bool)> {
        self.rocas
            .iter()
            .filter(|r| (r.pos - ojo).length() < LEJOS * 1.2)
            .map(|r| {
                let m = Mat4::translation(r.pos) * Mat4::rotation_y(r.giro * r.eje.y * 2.0 + r.eje.x) * Mat4::rotation_x(r.giro * r.eje.x * 1.5) * Mat4::scale(r.escala);
                (m, r.modelo, r.clase, !r.usada)
            })
            .collect()
    }
}

/// Los datos que cuenta Bill Nye (de verdad: astronomia y ciencia).
pub const DATOS: [&str; 32] = [
    "¡Bill Nye, el chico de la ciencia! Dato: un día en Venus dura más que un año en Venus. Gira tan lento que le da la vuelta al Sol antes de dar una vuelta sobre sí mismo.",
    "¡Ciencia! En Júpiter cabrían más de mil Tierras. Y su Gran Mancha Roja es una tormenta más grande que nuestro planeta, que lleva siglos girando.",
    "Dato: la luz del Sol tarda unos ocho minutos en llegar a la Tierra. Si el Sol se apagara, ¡no nos enteraríamos hasta ocho minutos después!",
    "¡Considera esto! Saturno es tan liviano para su tamaño que flotaría en una tina. Necesitarías una tina más grande que la Tierra, claro.",
    "Dato: en el espacio no hay aire, así que no se oye nada. Las explosiones de las películas son de mentira. Las de aquí también, pero suenan bonito.",
    "¡Ciencia! Hay más estrellas en el universo que granos de arena en todas las playas de la Tierra. Y eso que hay MUCHAS playas.",
    "Dato: la Luna se aleja de la Tierra unos 3.8 centímetros cada año. Más o menos lo que crecen tus uñas. ¡No te vayas, Luna!",
    "¡Bill! ¡Bill! ¡Bill! Marte es rojo porque está oxidado: su suelo está lleno de óxido de hierro. Es un planeta con herrumbre, literalmente.",
    "Dato: Neptuno tiene los vientos más rápidos del sistema solar, de más de 2,000 kilómetros por hora. Ni el cabello de Bill aguantaría eso.",
    "¡Considera esto! Urano gira acostado, inclinado unos 98 grados. Probablemente algo enorme lo chocó hace mucho y lo dejó así.",
    "Dato: los átomos de tu cuerpo se hicieron dentro de estrellas que explotaron hace miles de millones de años. ¡Eres polvo de estrellas! Literalmente.",
    "¡Ciencia! Mercurio es el planeta más cercano al Sol, pero el más caliente es Venus: su atmósfera atrapa el calor como un invernadero gigante.",
    "Dato: Plutón es más pequeño que nuestra Luna. Por eso en 2006 lo pasaron a planeta enano. Él todavía no lo supera.",
    "¡Bill Nye! Un año en Neptuno dura 165 años terrestres. Desde que lo descubrimos en 1846, apenas ha dado una vuelta al Sol.",
    "Dato: en la Luna hay huellas de astronautas de 1969 que siguen ahí, porque no hay viento que las borre. ¡Durarán millones de años!",
    "¡Considera esto! Una cucharadita de una estrella de neutrones pesaría unos mil millones de toneladas. No te la comas.",
    "Dato: la galaxia de Andrómeda viene hacia nosotros. En unos 4,500 millones de años chocará con la Vía Láctea. Tranquilo: hay tiempo.",
    "¡Ciencia! El Sol es tan grande que adentro cabrían más de un millón de Tierras. Y es una estrella mediana. Bien normalita.",
    "Dato: los anillos de Saturno son casi puro hielo, en pedazos desde granitos hasta casas. Y son delgadísimos: en partes miden apenas decenas de metros.",
    "¡Bill! ¡Bill! ¡Bill! Europa, la luna de Júpiter, tiene un océano de agua líquida debajo del hielo. Uno de los mejores lugares para buscar vida.",
    "Dato: Io, otra luna de Júpiter, es el lugar con más volcanes del sistema solar. Tiene cientos, y algunos escupen lava a cientos de kilómetros.",
    "¡Considera esto! Ganímedes es la luna más grande del sistema solar: más grande que Mercurio. Y tiene su propio campo magnético.",
    "Dato: la Voyager 1, lanzada en 1977, ya salió del sistema solar. Lleva un disco de oro con saludos, música y sonidos de la Tierra.",
    "¡Ciencia! Cuando ves una estrella, ves luz que salió de ella hace años, siglos o milenios. Estás viendo el pasado. Literalmente.",
    "Dato: un agujero negro no aspira como una aspiradora. Si el Sol se volviera agujero negro (no puede, es muy chico), la Tierra seguiría en su órbita.",
    "¡Bill Nye! Los astronautas crecen un poquito en el espacio: sin gravedad, su columna se estira hasta unos 5 centímetros. Luego se encogen al volver.",
    "Dato: el cinturón de asteroides no es como en las películas. Las rocas están tan separadas que las naves lo cruzan sin ver ninguna. Aquí hicimos trampa.",
    "¡Considera esto! Hace 66 millones de años, un asteroide de unos 10 kilómetros cayó en lo que hoy es México. Los dinosaurios no tenían programa espacial.",
    "Dato: la Estación Espacial Internacional le da la vuelta a la Tierra cada 90 minutos. Sus astronautas ven 16 amaneceres al día.",
    "¡Ciencia! El universo tiene unos 13,800 millones de años. Si fuera un año de calendario, los humanos llegaríamos el 31 de diciembre a las 11:52 de la noche.",
    "Dato: en Júpiter y Saturno podría llover diamantes. La presión es tan alta que el carbono se aprieta hasta volverse diamante. ¡Ciencia brillante!",
    "¡Bill! ¡Bill! ¡Bill! Las estrellas parpadean por el aire de la Tierra, que se mueve. En el espacio, como aquí, las estrellas no parpadean. ¡La ciencia manda!",
];
