//! Aterrizaje en la Luna: el "set de filmacion" de la llegada a la Luna. Es
//! una broma con la teoria de que fue falsa: un director con boina y megafono
//! esta filmando "la llegada", y Miku le ayuda a revisar la toma. Por cada cosa
//! sospechosa hay que decidir si es DEL SET (se quita: el ventilador que mueve
//! la bandera, el cafe, el reflector, el microfono, el letrero de SALIDA, el
//! profe de extra) o si ASI ES LA LUNA de verdad (el cielo sin estrellas, la
//! Tierra enorme, las huellas perfectas, los saltos en camara lenta: justo
//! los "argumentos" de la conspiracion). GLADAS explica cada cosa con el dato
//! real. Al final la camara se aleja: el set esta EN la Luna de verdad (salia
//! mas barato), y GLADAS aclara que la llegada de 1969 si paso.
//!
//! Cada toma escoge seis sospechas al azar. Todo esta en las coordenadas del
//! set: el centro, en el piso.

use crate::baile::Paso;
use crate::entrada::Entrada;
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Quien {
    Gladas,
    Miku,
    Director,
    Astronauta,
}

pub struct Sospecha {
    pub nombre: &'static str,
    pregunta: &'static str,
    /// Si es cosa del estudio (hay que quitarla) o si asi es la Luna de verdad.
    pub es_set: bool,
    acierta: &'static str,
    falla: &'static str,
    dato: &'static str,
    /// La parte de `set_luna.obj` que se quita (si es del set y tiene).
    pub parte: Option<&'static str>,
    /// A donde ve la camara (y donde brilla) cuando se pregunta.
    foco: Vec3,
}

pub const PROFE: usize = 5;
const ESTRELLAS: usize = 6;
pub const HUELLAS: usize = 8;
const SALTOS: usize = 9;

pub const SOSPECHAS: [Sospecha; 10] = [
    Sospecha {
        nombre: "EL VENTILADOR",
        pregunta: "Ese ventilador le sopla a la bandera para que ondee bonito. ¿Es del set, o así es en la Luna?",
        es_set: true,
        acierta: "¡Fuera ese ventilador! En la Luna no hay viento. Ni aire. Ni aire acondicionado.",
        falla: "¡CORTE! ¿Viento en la Luna? Eso solo pasa en mi película... y en internet.",
        dato: "En la misión real, la bandera tenía una varilla arriba para que se viera extendida. Se movió solo al clavarla, y como no hay aire que la frene, tardó en dejar de vibrar.",
        parte: Some("ventilador"),
        foco: Vec3::new(-2.3, 1.1, 2.1),
    },
    Sospecha {
        nombre: "EL CAFÉ",
        pregunta: "Hay una mesita con café en plena superficie lunar. ¿Es del set?",
        es_set: true,
        acierta: "Esa taza es mía. ...Bueno, fuera. Me lo tomo en el camerino.",
        falla: "¡CORTE! ¿Café en la Luna? Sin aire, se hierve y se congela al mismo tiempo. Lo dice el químico del set.",
        dato: "En el Apolo 11 sí llevaron café: en bolsitas, adentro del módulo. Afuera, en el vacío, un café hervería y se congelaría a la vez.",
        parte: Some("cafe"),
        foco: Vec3::new(1.0, 0.7, 0.7),
    },
    Sospecha {
        nombre: "EL REFLECTOR",
        pregunta: "Ese reflector gigante ilumina todo el set. ¿Lo dejamos?",
        es_set: true,
        acierta: "¡Apaguen el reflector! En la Luna la única lámpara es el Sol.",
        falla: "¡CORTE! ¿Un reflector en la Luna? ¿Y dónde lo conectan? ¿Dónde está el enchufe?",
        dato: "En la Luna la única luz es la del Sol... y la que rebota del suelo lunar, que es clarito. Por eso en las fotos reales las sombras no son negrísimas.",
        parte: Some("reflector"),
        foco: Vec3::new(3.8, 1.75, -0.8),
    },
    Sospecha {
        nombre: "EL MICRÓFONO",
        pregunta: "El micrófono de jirafa se ve en cuadro, arriba del astronauta. ¿Es del set?",
        es_set: true,
        acierta: "¡Suban ese micrófono! Además, en la Luna no se oye nada.",
        falla: "¡CORTE! En el vacío no viaja el sonido. Ese micrófono no graba ni mis gritos. Y grito mucho.",
        dato: "En el vacío no hay aire que lleve el sonido. Los astronautas se hablaban por radio, adentro de sus cascos.",
        parte: Some("microfono"),
        foco: Vec3::new(0.1, 2.2, 1.3),
    },
    Sospecha {
        nombre: "EL LETRERO DE SALIDA",
        pregunta: "Allá atrás hay un letrero verde que dice SALIDA. ¿Del set, o es de la Luna?",
        es_set: true,
        acierta: "Quiten el letrero. En la Luna no hay salida de emergencia. Bueno, sí: el módulo lunar.",
        falla: "¡CORTE! ¿Salida de emergencia en la Luna? ¿A dónde, a Marte?",
        dato: "La salida de verdad era el módulo lunar: la parte de arriba despegaba usando la de abajo como plataforma. La de abajo se quedó allá, y sigue ahí.",
        parte: Some("letrero"),
        foco: Vec3::new(-3.6, 1.85, 3.8),
    },
    Sospecha {
        nombre: "EL PROFE KOCH DE EXTRA",
        pregunta: "Ese señor es el profe Koch, el de autómatas de la UVG, de extra. ¿Es del set?",
        es_set: true,
        acierta: "¡Profe Koch, fuera de cuadro! Vaya a calificar exámenes.",
        falla: "¡CORTE! ¿El profe Koch en la Luna? ...Bueno, eso explicaría el examen.",
        dato: "Doce personas han caminado en la Luna, todas entre 1969 y 1972. Ningún profe Koch. Que se sepa.",
        parte: None,
        foco: Vec3::new(-3.0, 1.3, 0.6),
    },
    Sospecha {
        nombre: "EL CIELO SIN ESTRELLAS",
        pregunta: "El cielo está negro, sin una sola estrella. ¿Es un error del set? ¿Pintamos estrellas?",
        es_set: false,
        acierta: "¿Así es? ¿Sin estrellas? ...Bueno, me ahorro la pintura.",
        falla: "¡Pinten estrellas! ...¿Cómo que en las fotos de verdad no salen? ¡CORTE! ¡Despinten!",
        dato: "En las fotos reales casi no salen estrellas: el suelo y los trajes brillan tanto con el Sol que la cámara se ajustaba para eso, y las estrellas quedaban muy tenues. Como una foto de noche con flash.",
        parte: None,
        foco: Vec3::new(0.0, 9.0, 14.0),
    },
    Sospecha {
        nombre: "LA TIERRA ENORME",
        pregunta: "Esa Tierra en el cielo se ve enorme. ¿Es una maqueta del set?",
        es_set: false,
        acierta: "¿No es maqueta? ...Está muy bien hecha, la verdad.",
        falla: "¡Quiten la maqueta! ...¿Cómo que no se puede quitar? ¿Cómo que es la de verdad? ¡CORTE!",
        dato: "Desde la Luna, la Tierra se ve unas cuatro veces más ancha que la Luna llena desde la Tierra. Y casi no se mueve del cielo: siempre está en el mismo lugar.",
        parte: None,
        foco: TIERRA_EN,
    },
    Sospecha {
        nombre: "LAS HUELLAS",
        pregunta: "Las huellas quedaron perfectas, como en cemento. ¿Es truco del set?",
        es_set: false,
        acierta: "¿Así quedan? Bueno, que se queden. Son muy fotogénicas.",
        falla: "¡Barran esas huellas! ...¿Cómo que así quedan en la Luna? ¡CORTE! ¡Vuelvan a pisar!",
        dato: "El polvo lunar tiene granitos con filo que se enganchan y aguantan la forma. Y sin viento ni lluvia, las huellas del Apolo siguen allá desde 1969.",
        parte: None,
        foco: Vec3::new(1.2, 0.0, 2.0),
    },
    Sospecha {
        nombre: "LOS SALTOS LENTOS",
        pregunta: "El astronauta brinca como en cámara lenta. ¿Es efecto especial? ¿Le ponemos cables?",
        es_set: false,
        acierta: "¿Así se brinca allá? Bueno. Ahorramos en cables.",
        falla: "¡Pónganle cables! ...¿Cómo que en la Luna así se brinca? ¡CORTE! ¡Quítenle los cables!",
        dato: "En la Luna la gravedad es la sexta parte de la de la Tierra: todo cae más despacio. Por eso los astronautas avanzaban dando saltitos de canguro.",
        parte: None,
        foco: Vec3::new(0.2, 0.9, 1.4),
    },
];

/// Cuantas sospechas hay en cada toma.
pub const POR_TOMA: usize = 6;
const INTRO: [(Quien, &str); 4] = [
    (Quien::Director, "¡CORTE! ¿Quién dejó pasar esa nave? ¡Estamos filmando la llegada a la Luna! Bueno... la \"llegada\"."),
    (Quien::Miku, "¿Cómo que la \"llegada\"? ¿Esto es un set?"),
    (Quien::Director, "Shhh. Usted tiene buen ojo. Revise la toma: lo que sea del estudio, se quita. Lo que así se ve en la Luna de verdad, se queda. Que no nos cachen."),
    (Quien::Astronauta, "Don Director, ¿ya vamos a almorzar? El casco no tiene hoyo para la boca."),
];

/// Donde esta cada quien en el set.
pub const BANDERA: Vec3 = Vec3::new(-1.2, 1.57, 2.2);
const DIRECTOR_EN: Vec3 = Vec3::new(-1.6, 0.0, -3.0);
const MIKU_EN: Vec3 = Vec3::new(1.5, 0.0, -3.0);
const ASTRONAUTA_EN: Vec3 = Vec3::new(0.2, 0.0, 1.4);
const PROFE_EN: Vec3 = Vec3::new(-3.0, 0.0, 0.6);
pub const TIERRA_EN: Vec3 = Vec3::new(-25.0, 28.0, 160.0);
const REFLECTOR_LUZ: Vec3 = Vec3::new(3.62, 1.75, -0.64);

/// Segundos de las animaciones.
const LLEGADA: f32 = 5.0;
const ACCION: f32 = 4.0;
const REVELAR: f32 = 5.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fase {
    Llegada,
    Intro(usize),
    /// El director pregunta por la sospecha `i` de la toma; se escoge.
    Pregunta(usize),
    /// 0: el director reacciona. 1: GLADAS da el dato.
    Respuesta(usize, usize),
    /// ¡Y... ACCION!: el astronauta da el paso.
    Accion,
    /// La camara se aleja: el set esta en la Luna de verdad. 0: el director. 1: GLADAS.
    Revelacion(usize),
    Calificacion,
    Libre,
}

pub enum AccionLuna {
    Nada,
    Despegar,
}

pub struct Luna {
    fase: Fase,
    t: f32,
    pub total: f32,
    semilla: u32,
    /// Las sospechas de esta toma (indices en `SOSPECHAS`).
    seleccion: [usize; POR_TOMA],
    /// Que se escogio en la pregunta: false = DEL SET, true = ASI ES LA LUNA.
    eleccion: bool,
    /// Si cada respuesta fue buena.
    respuestas: Vec<bool>,
    /// Las cosas del set que ya se quitaron.
    quitadas: Vec<usize>,
    /// Lo que se hizo mal con las cosas de verdad.
    pub estrellas_pintadas: bool,
    pub huellas_barridas: bool,
    pub cables: bool,
    pub eventos: Vec<Efecto>,
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

impl Luna {
    pub fn new(semilla: u32) -> Self {
        let mut l = Luna {
            fase: Fase::Llegada,
            t: 0.0,
            total: 0.0,
            semilla: semilla | 1,
            seleccion: [0; POR_TOMA],
            eleccion: false,
            respuestas: Vec::new(),
            quitadas: Vec::new(),
            estrellas_pintadas: false,
            huellas_barridas: false,
            cables: false,
            eventos: Vec::new(),
        };
        l.escoger();
        l
    }

    fn azar(&mut self, n: usize) -> usize {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((self.semilla >> 8) as usize) % n
    }

    /// Seis sospechas al azar: tres o cuatro del set y el resto de la Luna de verdad, revueltas.
    fn escoger(&mut self) {
        let mut set: Vec<usize> = (0..SOSPECHAS.len()).filter(|&i| SOSPECHAS[i].es_set).collect();
        let mut luna: Vec<usize> = (0..SOSPECHAS.len()).filter(|&i| !SOSPECHAS[i].es_set).collect();
        for v in [&mut set, &mut luna] {
            for i in (1..v.len()).rev() {
                let j = self.azar(i + 1);
                v.swap(i, j);
            }
        }
        let de_set = 3 + self.azar(2);
        let mut todas: Vec<usize> = set[..de_set].iter().chain(&luna[..POR_TOMA - de_set]).copied().collect();
        for i in (1..todas.len()).rev() {
            let j = self.azar(i + 1);
            todas.swap(i, j);
        }
        self.seleccion.copy_from_slice(&todas);
        self.respuestas.clear();
        self.quitadas.clear();
        self.estrellas_pintadas = false;
        self.huellas_barridas = false;
        self.cables = false;
    }

    fn cambiar(&mut self, fase: Fase) {
        self.fase = fase;
        self.t = 0.0;
    }

    pub fn aciertos(&self) -> usize {
        self.respuestas.iter().filter(|r| **r).count()
    }

    /// Si ya se dio la calificacion de la toma.
    pub fn calificada(&self) -> bool {
        matches!(self.fase, Fase::Calificacion | Fase::Libre)
    }

    pub fn terminada(&self) -> bool {
        matches!(self.fase, Fase::Accion | Fase::Revelacion(_) | Fase::Calificacion | Fase::Libre)
    }

    /// Si la cosa `i` de `SOSPECHAS` esta en el set ahora: las del estudio solo si tocan en esta toma y no se han quitado.
    pub fn presente(&self, i: usize) -> bool {
        !SOSPECHAS[i].es_set || (self.seleccion.contains(&i) && !self.quitadas.contains(&i))
    }

    /// Las partes del set que no se dibujan.
    pub fn ocultar(&self) -> Vec<&'static str> {
        let mut v: Vec<&'static str> = (0..SOSPECHAS.len()).filter(|&i| !self.presente(i)).filter_map(|i| SOSPECHAS[i].parte).collect();
        if self.huellas_barridas {
            v.push("huellas");
        }
        v
    }

    /// Si el ventilador esta prendido (y la bandera ondea).
    pub fn ventilador(&self) -> bool {
        self.presente(0)
    }

    pub fn reflector(&self) -> bool {
        self.presente(2)
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionLuna {
        self.t += dt;
        self.total += dt;
        if e.aterrizar {
            return AccionLuna::Despegar;
        }
        let largo = self.hablante().map_or(0, |h| h.1.chars().count()) as f32;
        let siguiente = (e.aceptar && self.t > 0.4) || self.t > 2.8 + largo / 20.0;
        match self.fase {
            Fase::Llegada if self.t > LLEGADA || (e.aceptar && self.t > 0.5) => {
                self.eventos.push(Efecto::Claqueta);
                self.cambiar(Fase::Intro(0));
            }
            Fase::Intro(i) if siguiente => self.cambiar(if i + 1 < INTRO.len() { Fase::Intro(i + 1) } else { Fase::Pregunta(0) }),
            Fase::Pregunta(i) if self.t > 0.4 => {
                if e.izquierda || e.respuesta == Some(0) {
                    self.eleccion = false;
                } else if e.derecha || e.respuesta == Some(1) {
                    self.eleccion = true;
                }
                if e.aceptar || e.respuesta.is_some_and(|r| r < 2) {
                    self.responder(i);
                }
            }
            Fase::Respuesta(i, 0) if siguiente => self.cambiar(Fase::Respuesta(i, 1)),
            Fase::Respuesta(i, _) if siguiente => {
                if i + 1 < POR_TOMA {
                    self.eleccion = false;
                    self.cambiar(Fase::Pregunta(i + 1));
                } else {
                    self.eventos.push(Efecto::Claqueta);
                    self.cambiar(Fase::Accion);
                }
            }
            Fase::Accion if self.t > ACCION => self.cambiar(Fase::Revelacion(0)),
            Fase::Revelacion(0) if self.t > REVELAR && siguiente => self.cambiar(Fase::Revelacion(1)),
            Fase::Revelacion(1) if siguiente => {
                self.eventos.push(if self.aciertos() >= 4 { Efecto::Perfecto } else { Efecto::Fallo });
                self.cambiar(Fase::Calificacion);
            }
            Fase::Calificacion if siguiente => self.cambiar(Fase::Libre),
            Fase::Libre if e.aceptar && self.t > 0.6 => {
                // Otra toma: otras sospechas, el set como nuevo.
                self.escoger();
                self.eventos.push(Efecto::Claqueta);
                self.cambiar(Fase::Pregunta(0));
                self.eleccion = false;
            }
            _ => {}
        }
        AccionLuna::Nada
    }

    /// Contesta la sospecha `i` de la toma con lo escogido.
    fn responder(&mut self, i: usize) {
        let s = self.seleccion[i];
        let quitar = !self.eleccion;
        let bien = quitar == SOSPECHAS[s].es_set;
        self.respuestas.push(bien);
        if quitar && SOSPECHAS[s].es_set {
            self.quitadas.push(s);
        }
        // Lo que se "arregla" mal: estrellas pintadas, huellas barridas, cables.
        if quitar && !SOSPECHAS[s].es_set {
            match s {
                ESTRELLAS => self.estrellas_pintadas = true,
                HUELLAS => self.huellas_barridas = true,
                SALTOS => self.cables = true,
                _ => {}
            }
        }
        self.eventos.push(if bien { Efecto::Acierto } else { Efecto::Fallo });
        self.cambiar(Fase::Respuesta(i, 0));
    }

    /// La sospecha que se esta viendo, si hay.
    fn sospecha(&self) -> Option<&'static Sospecha> {
        match self.fase {
            Fase::Pregunta(i) | Fase::Respuesta(i, _) => Some(&SOSPECHAS[self.seleccion[i]]),
            _ => None,
        }
    }

    pub fn hablante(&self) -> Option<(Quien, &'static str, Voz)> {
        let director = |s| Some((Quien::Director, s, Voz::Megafono));
        match self.fase {
            Fase::Llegada => None,
            Fase::Intro(i) => Some((INTRO[i].0, INTRO[i].1, voz(INTRO[i].0))),
            Fase::Pregunta(_) => director(self.sospecha()?.pregunta),
            Fase::Respuesta(i, 0) => {
                let s = self.sospecha()?;
                director(if self.respuestas[i] { s.acierta } else { s.falla })
            }
            Fase::Respuesta(_, _) => Some((Quien::Gladas, self.sospecha()?.dato, Voz::Ia)),
            Fase::Accion => Some((Quien::Astronauta, "Un pequeño paso para el actor... un gran salto para mi carrera.", Voz::Radio)),
            Fase::Revelacion(0) => director("...¿Qué? Filmar en la Luna de verdad salía más barato que rentar un estudio. Shhh. No le diga a nadie."),
            Fase::Revelacion(_) => Some((
                Quien::Gladas,
                "Para que conste: la llegada a la Luna de 1969 fue real. Dejaron espejos allá, y todavía les rebotan láseres que les disparan desde la Tierra. Esto es un chiste. El director es el de mentira.",
                Voz::Ia,
            )),
            Fase::Calificacion => director(match self.aciertos() {
                6 => "¡Seis de seis! ¡Toma perfecta! Esto se lleva el Óscar. Bueno, un Óscar de la Luna: es de roca.",
                4 | 5 => "¡Bastante bien! Unos detallitos, pero nadie se va a dar cuenta. Nadie nunca se da cuenta.",
                2 | 3 => "Mmm. Con tantos errores, en cincuenta años alguien va a decir que fue falso. ...Ah, no. Ya lo dicen.",
                _ => "¡Desastre! Nominada a la peor película del año. Y a la peor bandera.",
            }),
            Fase::Libre => director("¡Silencio en el set! ...Bueno, en la Luna siempre hay silencio. No hay aire."),
        }
    }

    pub fn nombre(q: Quien) -> &'static str {
        match q {
            Quien::Gladas => ui::NOMBRE_IA,
            Quien::Miku => "MIKU",
            Quien::Director => "EL DIRECTOR",
            Quien::Astronauta => "EL ASTRONAUTA (ACTOR)",
        }
    }

    // ---------- donde esta cada quien ----------

    pub fn paso_miku(&self) -> Paso {
        match self.fase {
            Fase::Pregunta(_) => Paso::Senalar,
            Fase::Calificacion | Fase::Libre if self.aciertos() >= 4 => Paso::Baile,
            Fase::Intro(1) => Paso::Saludo,
            _ => Paso::Quieta,
        }
    }

    pub fn expresion_miku(&self) -> &'static str {
        match self.fase {
            Fase::Intro(1) | Fase::Revelacion(0) => "asustada",
            Fase::Pregunta(_) => "serio",
            Fase::Respuesta(i, _) if !self.respuestas[i] => "asustada",
            Fase::Calificacion | Fase::Libre if self.aciertos() >= 4 => "guino",
            _ => "feliz",
        }
    }

    /// Miku junto a la camara de cine, viendo al set (+Z).
    pub fn miku(&self) -> Mat4 {
        Mat4::translation(MIKU_EN) * Mat4::rotation_y(PI) * Mat4::scale(0.85)
    }

    pub fn expresion_director(&self) -> &'static str {
        match self.fase {
            Fase::Intro(0) => "enojado",
            Fase::Respuesta(i, 0) if !self.respuestas[i] => "enojado",
            Fase::Respuesta(_, 0) => "feliz",
            Fase::Revelacion(0) => "asustada",
            Fase::Calificacion if self.aciertos() >= 4 => "enamorada",
            Fase::Calificacion => "espiral",
            _ => "serio",
        }
    }

    /// El director junto a su silla. Cuando grita, levanta el megafono (se mece).
    pub fn director(&self) -> Mat4 {
        let grita = self.hablante().is_some_and(|h| h.0 == Quien::Director);
        let mece = if grita { 0.06 * (self.total * 9.0).sin() } else { 0.0 };
        Mat4::translation(DIRECTOR_EN) * Mat4::rotation_y(PI + 0.3 + mece) * Mat4::scale(1.0)
    }

    /// El astronauta: saltitos lentos (gravedad de un sexto). Con cables, cuelga tieso.
    pub fn astronauta(&self) -> Mat4 {
        let alto = if self.fase == Fase::Accion { 0.45 } else { 0.18 };
        let brinco = if self.cables { 0.6 + 0.05 * (self.total * 2.0).sin() } else { alto * (self.total * 1.3).sin().abs() };
        let paso = if self.fase == Fase::Accion { Vec3::new(0.0, 0.0, -0.5 * suave(self.t / ACCION)) } else { Vec3::zero() };
        Mat4::translation(ASTRONAUTA_EN + paso + Vec3::new(0.0, brinco, 0.0)) * Mat4::rotation_x(0.08 * (self.total * 1.3).cos())
    }

    /// Los cables del astronauta (si se los pusieron): de sus hombros hacia arriba.
    pub fn cables_del_astronauta(&self) -> Option<[(Vec3, Vec3); 2]> {
        self.cables.then(|| {
            let p = self.astronauta().transform_point(Vec3::new(0.0, 0.7, 0.0));
            [(p + Vec3::new(0.25, 0.0, 0.0), p + Vec3::new(0.25, 6.0, 0.0)), (p - Vec3::new(0.25, 0.0, 0.0), p + Vec3::new(-0.25, 6.0, 0.0))]
        })
    }

    pub fn profe(&self) -> Option<Mat4> {
        self.presente(PROFE).then(|| Mat4::translation(PROFE_EN) * Mat4::rotation_y(0.5 + 0.1 * (self.total * 0.7).sin()) * Mat4::scale(0.8))
    }

    /// La tela de la bandera: si el ventilador esta prendido, ondea; si no, quieta.
    pub fn bandera(&self) -> Mat4 {
        let ondea = if self.ventilador() { 0.35 * (self.total * 7.0).sin() + 0.15 * (self.total * 13.0).sin() } else { 0.0 };
        Mat4::translation(BANDERA) * Mat4::rotation_y(ondea)
    }

    /// La Tierra en el cielo.
    pub fn tierra(&self) -> Mat4 {
        Mat4::translation(TIERRA_EN) * Mat4::rotation_z(0.41) * Mat4::rotation_y(self.total * 0.05) * Mat4::scale(14.0)
    }

    /// La camara. (ojo, mira)
    pub fn camara(&self) -> (Vec3, Vec3) {
        let vaiven = Vec3::new(0.08 * (self.total * 0.3).sin(), 0.04 * (self.total * 0.45).sin(), 0.0);
        let general = (Vec3::new(0.8, 2.0, -6.3), Vec3::new(0.0, 0.9, 1.5));
        let (ojo, mira) = match self.fase {
            Fase::Llegada => {
                let u = suave(self.t / LLEGADA);
                (Vec3::new(9.0 - 8.2 * u, 7.0 - 5.0 * u, -20.0 + 13.7 * u), Vec3::new(0.0, 0.9, 1.5))
            }
            Fase::Pregunta(_) | Fase::Respuesta(_, _) => {
                let s = self.seleccion[match self.fase {
                    Fase::Pregunta(i) | Fase::Respuesta(i, _) => i,
                    _ => 0,
                }];
                let f = SOSPECHAS[s].foco;
                match s {
                    // Las cosas del cielo se ven desde el piso, hacia arriba.
                    ESTRELLAS => (Vec3::new(0.3, 1.2, -2.5), f),
                    7 => (Vec3::new(0.0, 1.4, -2.5), TIERRA_EN),
                    HUELLAS => (Vec3::new(1.6, 1.4, 0.3), f),
                    _ => (f + Vec3::new(0.7, 0.5, -2.3), f),
                }
            }
            Fase::Accion => {
                let u = suave(self.t / ACCION);
                (Vec3::new(-0.7 - 0.3 * u, 0.25, 0.1 - 0.3 * u), ASTRONAUTA_EN + Vec3::new(0.0, 0.9, 0.0))
            }
            Fase::Revelacion(_) => {
                let u = suave(self.t / REVELAR);
                (Vec3::new(-1.5 * u, 2.4 + 6.0 * u, -6.5 - 12.0 * u), Vec3::new(-3.0 * u, 0.6 + 4.5 * u, 1.5 + 40.0 * u))
            }
            Fase::Calificacion | Fase::Libre => general,
            Fase::Intro(_) => match self.hablante().map(|h| h.0) {
                Some(Quien::Director) => (Vec3::new(-1.0, 1.35, -1.8), DIRECTOR_EN + Vec3::new(0.0, 0.95, 0.0)),
                Some(Quien::Miku) => (Vec3::new(1.0, 1.15, -1.9), MIKU_EN + Vec3::new(0.0, 0.8, 0.0)),
                Some(Quien::Astronauta) => (Vec3::new(0.6, 1.3, -0.5), ASTRONAUTA_EN + Vec3::new(0.0, 1.0, 0.0)),
                _ => general,
            },
        };
        (ojo + vaiven, mira)
    }

    /// Las estrellas pintadas (si se equivocaron), los cables, el reflector y el brillo de la sospecha.
    pub fn dibujar_efectos(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4) {
        if self.estrellas_pintadas {
            // Estrellas de cartulina amarilla, de cinco picos, colgadas del cielo.
            for k in 0..14 {
                let c = Vec3::new(-18.0 + (k as f32 * 7.3) % 36.0, 7.0 + (k as f32 * 3.1) % 10.0, 18.0 + (k as f32 * 5.7) % 20.0);
                let r = 0.8;
                for p in 0..5 {
                    let a0 = p as f32 * TAU * 2.0 / 5.0 - PI / 2.0;
                    let a1 = (p + 1) as f32 * TAU * 2.0 / 5.0 - PI / 2.0;
                    let (q0, q1) = (c + Vec3::new(r * a0.cos(), -r * a0.sin(), 0.0), c + Vec3::new(r * a1.cos(), -r * a1.sin(), 0.0));
                    if let (Some(x), Some(y)) = (proyectar(q0, vista_proyeccion, fb), proyectar(q1, vista_proyeccion, fb)) {
                        fb.linea_de_luz(x, y, [0.8, 0.7, 0.1], [0.8, 0.7, 0.1]);
                    }
                }
            }
        }
        if let Some(cables) = self.cables_del_astronauta() {
            for (a, b) in cables {
                if let (Some(x), Some(y)) = (proyectar(a, vista_proyeccion, fb), proyectar(b, vista_proyeccion, fb)) {
                    fb.linea_de_luz(x, y, [0.25, 0.25, 0.25], [0.25, 0.25, 0.25]);
                }
            }
        }
        if self.reflector() {
            if let Some((x, y, _)) = proyectar(REFLECTOR_LUZ, vista_proyeccion, fb) {
                crate::post::halo(fb, (x, y), 110.0, [0.6, 0.58, 0.5], 1.4);
            }
        }
        if let (Fase::Pregunta(_), Some(s)) = (self.fase, self.sospecha()) {
            if let Some((x, y, _)) = proyectar(s.foco, vista_proyeccion, fb) {
                let b = 0.6 + 0.4 * (self.total * 5.0).sin();
                crate::post::halo(fb, (x, y), 90.0, [0.5 * b, 0.4 * b, 0.1 * b], 1.5);
            }
        }
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, ancho: i32, alto: i32) {
        let (aceptar, salir) = (&ui::boton(e.nombre("aceptar")), &ui::boton(e.nombre("aterrizar")));
        let centrado = |d: &mut RaylibDrawHandle, s: &str, y: i32, tam: i32, color: Color| ui::centrado(d, s, ancho, y, tam, color);
        if self.fase == Fase::Llegada {
            let a = (suave(self.t / 0.8) * (1.0 - suave((self.t - LLEGADA + 1.0) / 1.0)) * 255.0) as u8;
            centrado(d, "LA LUNA", 70, 60, Color::new(240, 240, 250, a));
            centrado(d, "Set 3 · Toma 1 · 1969", 140, 26, Color::new(220, 220, 230, a));
        }
        // La claqueta de cada toma, arriba a la izquierda: escena, toma y aciertos.
        if !matches!(self.fase, Fase::Llegada | Fase::Intro(_)) {
            ui::panel(d, 16, 16, 260, 74);
            ui::texto(d, "LA LLEGADA (TOMA 1)", 30, 24, 16, ui::TENUE);
            let n = match self.fase {
                Fase::Pregunta(i) | Fase::Respuesta(i, _) => i + 1,
                _ => POR_TOMA,
            };
            ui::texto(d, &format!("sospecha {n} de {POR_TOMA}"), 30, 46, 20, ui::TEXTO);
            ui::texto(d, &format!("bien: {}", self.aciertos()), 190, 46, 20, ui::VERDE);
        }
        // La sospecha: su nombre y las dos opciones.
        if let (Fase::Pregunta(_), Some(s)) = (self.fase, self.sospecha()) {
            centrado(d, s.nombre, 100, 40, Color::new(255, 230, 140, 255));
            let (w, h) = (300, 64);
            // Abajo, justo arriba del dialogo: asi no tapan lo que se pregunta.
            let y = alto - 250;
            for (k, (titulo, sub)) in [("DEL SET", "quitarlo"), ("ASÍ ES LA LUNA", "dejarlo")].iter().enumerate() {
                let x = ancho / 2 - w - 20 + k as i32 * (w + 40);
                let activa = (k == 1) == self.eleccion;
                ui::panel(d, x, y, w, h);
                if activa {
                    d.draw_rectangle_lines_ex(Rectangle::new(x as f32, y as f32, w as f32, h as f32), 3.0, Color::new(255, 220, 120, 255));
                }
                let color = if activa { Color::new(255, 230, 140, 255) } else { ui::TENUE };
                ui::texto(d, &format!("{} {titulo}", k + 1), x + 18, y + 10, 24, color);
                ui::texto(d, sub, x + 18, y + 38, 18, ui::TENUE);
            }
            centrado(d, &format!("{} o {}: escoger      {aceptar}: decidir      {salir}: despegar", ui::boton("\u{E00A} \u{E00B}"), ui::boton("1 / 2")), y + h + 12, 18, ui::TEXTO);
        }
        // ¡CORTE! cuando se equivoca.
        if let Fase::Respuesta(i, 0) = self.fase {
            if !self.respuestas[i] && self.t < 1.5 && (self.t * 6.0) as i32 % 2 == 0 {
                centrado(d, "¡CORTE!", alto / 2 - 120, 90, Color::new(240, 40, 50, 255));
            }
        }
        if self.fase == Fase::Accion && self.t < 2.0 {
            centrado(d, "¡Y... ACCIÓN!", alto / 2 - 140, 72, Color::new(255, 240, 200, (255.0 * (1.0 - suave((self.t - 1.4) / 0.6))) as u8));
        }
        if self.fase == Fase::Calificacion {
            let (titulo, color) = match self.aciertos() {
                6 => ("ÓSCAR (DE ROCA)", Color::new(255, 215, 90, 255)),
                4 | 5 => ("TOMA BUENA", ui::VERDE),
                2 | 3 => ("TOMA DUDOSA", Color::new(255, 190, 120, 255)),
                _ => ("PEOR PELÍCULA DEL AÑO", ui::ROJO),
            };
            centrado(d, &format!("{} de {POR_TOMA}", self.aciertos()), 110, 30, ui::TEXTO);
            centrado(d, titulo, 146, 52, color);
        }
        if self.fase == Fase::Libre {
            centrado(d, &format!("{aceptar}: otra toma      {salir}: despegar"), 110, 24, Color::new(240, 240, 250, 255));
        }
        if let (Some((quien, _, _)), false) = (self.hablante(), texto.is_empty()) {
            let nombre = Self::nombre(quien);
            let w = 920.min(ancho - 40);
            let h = ui::alto_dialogo(nombre, texto, w);
            let color = match quien {
                Quien::Director => Color::new(255, 130, 120, 255),
                Quien::Miku => Color::new(120, 230, 220, 255),
                Quien::Astronauta => Color::new(220, 220, 235, 255),
                Quien::Gladas => ui::LILA,
            };
            ui::caja_dialogo(d, nombre, color, texto, (ancho - w) / 2, alto - h - 20, w);
        }
    }

    /// Juega solo hasta la fase que empieza con `objetivo` (con `#n`, la n-esima vez), contestando
    /// bien (o mal, con `_mal`): para las capturas.
    pub fn jugar_hasta(&mut self, objetivo: &str) {
        let (objetivo, veces) = objetivo.split_once('#').map_or((objetivo, 1), |(o, n)| (o, n.parse().unwrap_or(1)));
        let (objetivo, mal) = objetivo.strip_suffix("_mal").map_or((objetivo, false), |o| (o, true));
        let mut vistas = 0;
        let mut antes = false;
        for _ in 0..6000 {
            let ahora = format!("{:?}", self.fase).starts_with(objetivo);
            if ahora && !antes {
                vistas += 1;
                if vistas == veces {
                    return;
                }
            }
            antes = ahora;
            let mut e = Entrada::default();
            if let Fase::Pregunta(i) = self.fase {
                if self.t > 0.5 {
                    let luna = !SOSPECHAS[self.seleccion[i]].es_set;
                    e.respuesta = Some((luna != mal) as usize);
                }
            } else if self.hablante().is_some() && self.t > 0.5 {
                e.aceptar = true;
            }
            self.actualizar(&e, 0.05);
        }
    }
}

fn voz(q: Quien) -> Voz {
    match q {
        Quien::Gladas => Voz::Ia,
        Quien::Miku => Voz::Miku,
        Quien::Director => Voz::Megafono,
        Quien::Astronauta => Voz::Radio,
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Cada toma tiene seis sospechas distintas, y de los dos tipos.
    #[test]
    fn cada_toma_mezcla_set_y_luna() {
        for semilla in 1..30 {
            let l = Luna::new(semilla);
            let mut v = l.seleccion.to_vec();
            v.sort();
            v.dedup();
            assert_eq!(v.len(), POR_TOMA);
            let de_set = l.seleccion.iter().filter(|&&i| SOSPECHAS[i].es_set).count();
            assert!((3..=4).contains(&de_set));
        }
    }

    /// Contestando todo bien: se quitan todas las cosas del set de la toma, y sale el seis de seis.
    #[test]
    fn todo_bien_limpia_el_set() {
        let mut l = Luna::new(11);
        l.jugar_hasta("Calificacion");
        assert_eq!(l.aciertos(), POR_TOMA);
        assert!((0..SOSPECHAS.len()).filter(|&i| SOSPECHAS[i].es_set).all(|i| !l.presente(i)));
        assert!(!l.estrellas_pintadas && !l.huellas_barridas && !l.cables);
    }

    /// Contestando todo mal: las cosas del set se quedan, y lo de verdad se "arregla" mal.
    #[test]
    fn todo_mal_deja_el_set_y_arruina_la_luna() {
        let mut l = Luna::new(11);
        l.jugar_hasta("Calificacion_mal");
        assert_eq!(l.aciertos(), 0);
        for &i in &l.seleccion {
            if SOSPECHAS[i].es_set {
                assert!(l.presente(i), "{} deberia seguir en el set", SOSPECHAS[i].nombre);
            }
        }
        let arruinadas = [ESTRELLAS, HUELLAS, SALTOS].iter().filter(|i| l.seleccion.contains(i)).count();
        let marcas = [l.estrellas_pintadas, l.huellas_barridas, l.cables].iter().filter(|b| **b).count();
        assert_eq!(arruinadas, marcas);
    }
}
