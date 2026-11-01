//! Aterrizaje en Venus, el planeta del amor... y por eso, del juzgado de
//! divorcios. Hoy se divorcia Saturno: tiene tres anillos principales (A, B y
//! C, los de verdad) y cada uno es de una boda distinta.
//! Los tres ex lo demandan para quedarse con el anillo de su boda: Sonic (por
//! los anillos, obvio), Gollum (su tesssoro) y el Sinluz de Elden Ring (que no
//! habla: deja mensajes en el piso). Los representa Phoenix Wright, y el juez
//! es el ganso de Untitled Goose Game, con peluca.
//!
//! Se juega como Ace Attorney: cada ex testifica cuatro declaraciones y una
//! es mentira. Miku (la abogada de Saturno) las recorre con las flechas, las
//! compara con el expediente (datos reales de cada anillo), puede presionar
//! para que le cuenten mas y, cuando encuentra la mentira, grita ¡PROTESTO!.
//! Si protesta mal, al juez se le acaba la paciencia; si se le acaba del
//! todo, el anillo se lo lleva el ex... o se lo roba el ganso. El orden de
//! los ex, sus testimonios y varias lineas cambian en cada juicio; al final
//! se puede abrir otro juicio.
//!
//! Todo esta en las coordenadas de la sala: el centro, en el piso.

use crate::baile::Paso;
use crate::entrada::Entrada;
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// Quien habla (para el nombre, la voz y la toma de la camara).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Quien {
    Gladas,
    Miku,
    Saturno,
    Phoenix,
    Juez,
    Ex(usize),
}

struct Declaracion {
    dice: &'static str,
    /// Lo que pregunta Miku al presionar, y lo que le contestan.
    pregunta: &'static str,
    respuesta: &'static str,
}

const fn dec(dice: &'static str, pregunta: &'static str, respuesta: &'static str) -> Declaracion {
    Declaracion { dice, pregunta, respuesta }
}

struct Testimonio {
    titulo: &'static str,
    declaraciones: [Declaracion; 4],
    /// Cual de las cuatro contradice al expediente.
    mentira: usize,
    /// Lo que explica Miku al protestar bien.
    protesto: &'static str,
    /// Como se derrumba el ex.
    derrota: [&'static str; 2],
}

struct DeclaracionDelCaso {
    dice: String,
    pregunta: String,
    respuesta: String,
}

struct Caso {
    titulo: String,
    declaraciones: [DeclaracionDelCaso; 4],
    mentira: usize,
    protesto: String,
    derrota: [&'static str; 2],
    pruebas: [(String, String); 3],
    prueba: usize,
    consecuencia: String,
}

/// Un numero al azar en `0..n` con la semilla del juicio (la misma cuenta que `Divorcio::azar`).
fn azar_con(semilla: &mut u32, n: usize) -> usize {
    *semilla = semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    ((*semilla >> 8) as usize) % n
}

/// Los datos de las bodas de mentira (las actas son ficticias; los anillos, no).
const LUGARES: [&str; 6] = ["la Luna", "el cinturón de asteroides", "una estación espacial", "Titán", "la cola de un cometa", "el polo norte de Marte"];
const FECHAS: [&str; 6] = ["un martes", "el día del eclipse", "en Navidad", "un 15 de septiembre", "un viernes 13", "en pleno año bisiesto"];
const TESTIGOS: [&str; 6] = ["Plutón", "Neptuno", "el Sol", "Júpiter", "un cometa con prisa", "la Estación Espacial"];
const PASTELES: [&str; 6] = ["de tres pisos", "de chocolate", "en forma de anillo", "de queso", "de hielo de cometa", "de tres leches"];

/// Como dice cada ex cada dato de su boda (0 lugar, 1 fecha, 2 testigo, 3 pastel), y que le
/// contestan al preguntarle. `{}` es el dato.
const BODA_DICE: [[&str; 4]; 3] = [
    ["¡Nos casamos en {}! Llegué primero, obvio.", "Fue {}. Me acuerdo porque ese día corrí. Bueno, corro todos los días.", "El testigo fue {}. Le di la mano rapidísimo.", "El pastel era {}. Me lo comí en 0.3 segundos."],
    ["Nos casamosss en {}, sí, preciossso, en {}.", "Fue {}, sí. Gollum se acuerda. Gollum nunca olvida.", "El testigo fue {}. No nos caía bien. ¡Nunca!", "Había un pastel {}. Gollum prefiere pescado crudo, pero lo comimos."],
    ["«boda, por lo tanto, {}»", "«{}, y luego, boda»", "«testigo por delante: {}»", "«si tan solo tuviera pastel {}...»"],
];
const BODA_PREGUNTA: [&str; 4] = [
    "¿Seguro que fue ahí? ¿No se confunde de lugar?",
    "¿Y cómo sabe que fue ese día? ¿Lo apuntó?",
    "¿El testigo? ¿Está seguro de quién era?",
    "¿Y el pastel? ¿Cómo se acuerda tan bien?",
];
const BODA_RESPUESTA: [[&str; 4]; 3] = [
    ["¡Seguro! Tengo buena memoria para los mapas. Los recorro muy rápido, pero los recorro.", "Lo apunté en mi tenis. Bueno, se borró. Pero me acuerdo.", "¡Clarísimo! Salió en la foto... creo. Yo salí borroso, como siempre.", "Porque fue lo único que me comí despacio. Bueno, no."],
    ["¡Lo vimosss! Gollum estaba ahí, sí, con su mejor taparrabo.", "Gollum lo anotó en una piedra. La piedra se perdió. Pero Gollum se acuerda.", "¡Sí, sí! Estaba ahí, mirando feo. Todos miran feo a Gollum.", "Porque Gollum se escondió debajo de la mesa con su pedazo. Sí."],
    ["Mi cliente asiente con la cabeza. Creo. Con el casco no se sabe.", "Mi cliente escribe en el piso: «fecha, por lo tanto, cierta». Eso es un sí.", "Mi cliente señala la foto. Bueno, señala la pared. Pero con convicción.", "Mi cliente deja otro mensaje: «pastel, delicioso». No ayuda, pero es tierno."],
];
const BODA_TITULO: [&str; 3] = ["La boda más rápida del sistema solar", "La boda del tesssoro", "Mensajes de boda en el piso"];
const BODA_DERROTA: [[&str; 2]; 3] = [
    ["¿El acta? ¿Hay acta? ...Yo pensé que nadie leía los papeles.", "Ya no quiero el anillo. Me voy. Rápido, como siempre."],
    ["¡Papeles mentirosos! ¡Malos! ¡Nos odian, preciossso!", "...Está bien. Gollum se va a su cueva. Sin anillo. Otra vez."],
    ["*el Sinluz escribe en el piso: «derrota, por lo tanto, vergüenza»*", "*hace una reverencia y se va rodando hacia atrás, muy digno*"],
];

/// Los dos documentos de relleno de los casos del anillo: no contradicen nada (la que
/// sirve es la ficha astronomica).
fn relleno(nombre: &str) -> [(String, String); 2] {
    [
        ("Invitación a la boda".into(), format!("Saturno y {nombre} los invitan a celebrar su unión. Habrá pastel y música. Se ruega no traer más anillos.")),
        ("Recibo del salón de Titán".into(), "Renta del salón de fiestas: pagada por Saturno. Incluye mesas, sillas y una pista de baile con gravedad reducida.".into()),
    ]
}

impl Caso {
    /// El caso del ex `ex` (indice en `EXES`): las variantes 0 y 1 son sus testimonios de
    /// siempre sobre el anillo (la prueba buena es la ficha astronomica, con datos de verdad);
    /// de la 2 en adelante, una boda inventada al azar, que se desmiente con sus papeles.
    /// Las tres pruebas salen en desorden: hay que escoger la que demuestra la mentira.
    fn nuevo(ex: usize, variante: usize, semilla: &mut u32) -> Caso {
        let e = &EXES[ex];
        let nombre = titulo(e.nombre);
        let (titulo_caso, declaraciones, mentira, protesto, derrota, documentos, buena, consecuencia) = if variante < 2 {
            let t = &e.testimonios[variante];
            let ficha = (format!("Ficha astronómica del anillo {}", ANILLOS[e.anillo]), e.evidencia.to_string());
            let [a, b] = relleno(&nombre);
            let declaraciones = t.declaraciones.each_ref().map(|d| DeclaracionDelCaso { dice: d.dice.into(), pregunta: d.pregunta.into(), respuesta: d.respuesta.into() });
            let consecuencia = "La ficha astronómica no se equivoca: los datos del anillo ganan.".to_string();
            (t.titulo.to_string(), declaraciones, t.mentira, t.protesto.to_string(), t.derrota, [ficha, a, b], 0, consecuencia)
        } else {
            let elegir = |semilla: &mut u32, lista: &[&'static str; 6]| lista[azar_con(semilla, lista.len())];
            let verdad = [elegir(semilla, &LUGARES), elegir(semilla, &FECHAS), elegir(semilla, &TESTIGOS), elegir(semilla, &PASTELES)];
            let listas = [&LUGARES, &FECHAS, &TESTIGOS, &PASTELES];
            let mentira = azar_con(semilla, 4);
            // El dato falso: cualquiera de su lista menos el verdadero.
            let opciones: Vec<&str> = listas[mentira].iter().copied().filter(|x| *x != verdad[mentira]).collect();
            let falso = opciones[azar_con(semilla, opciones.len())];
            let dicho = |k: usize| if k == mentira { falso } else { verdad[k] };
            let declaraciones = std::array::from_fn(|k| DeclaracionDelCaso {
                dice: BODA_DICE[ex][k].replace("{}", dicho(k)),
                pregunta: BODA_PREGUNTA[k].into(),
                respuesta: BODA_RESPUESTA[ex][k].into(),
            });
            let documentos = [
                ("Acta de matrimonio (Registro Civil de Titán)".to_string(), format!("Saturno y {nombre} se casaron en {}, {}. Firma: un notario muy cansado.", verdad[0], verdad[1])),
                ("Foto de la boda".to_string(), format!("Salen los novios y, de testigo, {}, sonriendo a medias. Atrás, el salón decorado con estrellitas.", verdad[2])),
                ("Recibo de la pastelería La Vía Láctea".to_string(), format!("Un pastel {}, entregado a domicilio. Pagado en efectivo (y en anillos).", verdad[3])),
            ];
            // Que papel tiene cada dato: el acta, el lugar y la fecha; la foto, el testigo; el recibo, el pastel.
            let buena = [0, 0, 1, 2][mentira];
            let (protesto, consecuencia) = match mentira {
                0 => (
                    format!("¡PROTESTO! Dice que se casaron en {falso}, ¡pero el acta de matrimonio dice que fue en {}!", verdad[0]),
                    format!("El Registro Civil de Titán le abre un expediente a {nombre} por inventar lugares."),
                ),
                1 => (
                    format!("¡PROTESTO! Dice que fue {falso}, ¡pero el acta de matrimonio dice que fue {}!", verdad[1]),
                    format!("El notario cansado le manda a {nombre} un calendario. De regalo. Con indirecta."),
                ),
                2 => (
                    format!("¡PROTESTO! Dice que el testigo fue {falso}, ¡pero en la foto de la boda el testigo es {}!", verdad[2]),
                    format!("{} confirma la foto y pide salir más guapo la próxima vez.", titulo_mayuscula(verdad[2])),
                ),
                _ => (
                    format!("¡PROTESTO! Dice que el pastel era {falso}, ¡pero el recibo de la pastelería dice que era {}!", verdad[3]),
                    format!("La pastelería La Vía Láctea le cobra a {nombre} el pastel {falso} que nunca existió."),
                ),
            };
            (BODA_TITULO[ex].to_string(), declaraciones, mentira, protesto, BODA_DERROTA[ex], documentos, buena, consecuencia)
        };
        // Las tres pruebas en desorden (y se recuerda en que lugar quedo la buena).
        let mut orden = [0, 1, 2];
        for i in (1..3).rev() {
            orden.swap(i, azar_con(semilla, i + 1));
        }
        let mut documentos = documentos.map(Some);
        let pruebas = orden.map(|k| documentos[k].take().unwrap_or_default());
        let prueba = orden.iter().position(|&k| k == buena).unwrap_or(0);
        Caso { titulo: titulo_caso, declaraciones, mentira, protesto, derrota, pruebas, prueba, consecuencia }
    }
}

/// "el Sol" -> "El Sol" (para empezar una oracion).
fn titulo_mayuscula(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |p| p.to_uppercase().collect::<String>() + c.as_str())
}

pub struct Ex {
    pub nombre: &'static str,
    /// El anillo que pide: 0 = A, 1 = B, 2 = C.
    pub anillo: usize,
    presentacion: &'static str,
    evidencia: &'static str,
    testimonios: [Testimonio; 2],
    voz: Voz,
    /// Quien contesta cuando se le presiona (el Sinluz no habla: contesta Phoenix).
    contesta: Quien,
}

pub const ANILLOS: [&str; 3] = ["A", "B", "C"];
/// Las partes de cada anillo en `saturno_cliente.obj`.
pub const PARTES_ANILLO: [&str; 3] = ["anillo_a", "anillo_b", "anillo_c"];

pub const EXES: [Ex; 3] = [
    Ex {
        nombre: "SONIC",
        anillo: 1,
        presentacion: "¡Gotta go fast! Pero este divorcio va lentísimo. Quiero el anillo B: es el más grande y el más brillante. Lo vi primero.",
        evidencia: "ANILLO B. El más grande y brillante de los tres. Es casi pura agua congelada: más del 90% es hielo. Llega a 117,580 km del centro de Saturno.",
        testimonios: [
            Testimonio {
                titulo: "Lo que Sonic sabe de anillos",
                declaraciones: [
                    dec("Me casé con Saturno por amor. Bueno, por amor a sus anillos.", "¿Por amor a los anillos?", "Amor del bueno. Los anillos me dan vidas extra. Saturno no, pero igual."),
                    dec("Llevo desde 1991 juntando anillos. Sé exactamente de qué están hechos.", "¿Y de qué están hechos?", "Hacen \"cling\". Todo lo que hace \"cling\" es valioso. Es ciencia."),
                    dec("El anillo B es de oro puro. Por eso brilla tanto.", "¿Oro puro? ¿Seguro?", "¡Segurísimo! Lo vi de lejos. Muy de lejos. A la velocidad del sonido todo se ve dorado."),
                    dec("Si me lo dan, lo cuido. Nunca lo suelto. Bueno, solo cuando me pegan.", "¿Y cuando le pegan?", "Se me caen todos y salen volando. Los recojo... a veces. Unos tres."),
                ],
                mentira: 2,
                protesto: "¡PROTESTO! El expediente dice que el anillo B es más del 90% hielo de agua. ¡No es oro! ¡Usted se casó por interés... y ni siquiera era de oro!",
                derrota: ["¿Hielo? ¿HIELO? ...Con razón en Ice Cap Zone me sentía en casa.", "Ya no lo quiero. Me voy. Rápido, como siempre. Es lo único que sé hacer."],
            },
            Testimonio {
                titulo: "La vuelta al anillo",
                declaraciones: [
                    dec("Saturno y yo éramos inseparables. Salíamos a correr todas las mañanas.", "¿Saturno corre?", "Saturno gira. Para mí es lo mismo. Da una vuelta cada diez horas y media. Lento, pero cuenta."),
                    dec("Una vez le di la vuelta completa al anillo B en un segundo. Es pequeñito.", "¿En un segundo?", "¡Un segundo! Soy el más rápido. Lo dice el juego. En la caja. Con letras grandes."),
                    dec("Ese anillo lo encontré yo, adentro de una caja con una pantallita.", "¿Una caja con pantallita?", "Uno la rompe y sale el anillo. Así se compromete uno en mi mundo."),
                    dec("Además ya le grabé mis iniciales: S.T.H.", "¿S.T.H.?", "Sonic The Hedgehog. O \"Saturno Te Hereda\". Las dos aplican."),
                ],
                mentira: 1,
                protesto: "¡PROTESTO! El anillo B llega a 117,580 km del centro: la vuelta mide más de 700 mil kilómetros. ¡A la velocidad del sonido tardaría como 25 días! ¡Y en el espacio ni hay sonido!",
                derrota: ["¿25 días? ¿Y SIN sonido? ...Entonces allá arriba ni soy rápido.", "Me rindo. Quédese con el anillo. Yo me voy a buscar esmeraldas. Esas no se divorcian."],
            },
        ],
        voz: Voz::Erizo,
        contesta: Quien::Ex(0),
    },
    Ex {
        nombre: "GOLLUM",
        anillo: 0,
        presentacion: "Mi tesssoro... el anillo A. Saturno nos lo quitó. Nosotros lo queremos de vuelta, sí, sí. ¡Gollum! ¡Gollum!",
        evidencia: "ANILLO A. El de afuera de los tres. Mide unos 15,000 km de ancho y no es uno solo: está hecho de miles de anillitos delgados, como los surcos de un disco.",
        testimonios: [
            Testimonio {
                titulo: "El Anillo Único",
                declaraciones: [
                    dec("Nosotros conocimos a Saturno en una cueva. Muy romántico. Muy oscuro.", "¿Una cueva? ¿En el espacio?", "Una cueva en las nubes. Oscura, húmeda. Como nos gustan. Saturno andaba perdido. Nosotros también."),
                    dec("El tesoro es el Anillo Único. Uno solo. No hay otro igual en el universo.", "¿Uno solo?", "¡Uno! Lo contamos con los dedos. Tenemos pocos dedos, pero contamos bien."),
                    dec("Se lo dimos en la boda, envuelto en pescado. Fresquito.", "¿Envuelto en pescado?", "Crudo. Saturno no se lo comió. Grosero. Malagradecido."),
                    dec("Si no nos lo dan, nos ponemos tristes. Y luego nos ponemos Gollum.", "¿Cómo es ponerse Gollum?", "*glup* *glup* ...Así. Así es."),
                ],
                mentira: 1,
                protesto: "¡PROTESTO! El anillo A no es único: está hecho de MILES de anillitos delgados, como los surcos de un disco. ¡No es un tesoro! ¡Son miles!",
                derrota: ["¿Miles? ¿MILES DE TESOROS? ...Y ninguno es nuestro. ¡Saturno tramposo!", "Sméagol odia los juzgados. Sméagol se va a su cueva. Con su pescado."],
            },
            Testimonio {
                titulo: "El tesoro del río",
                declaraciones: [
                    dec("Nosotros lo pescamos del río. Estaba en el fondo, brillando, brillando.", "¿Qué río?", "El río de Saturno. Bueno, no tiene. Pero tiene nubes, y las nubes son ríos flojos."),
                    dec("Es pequeñito, redondito. Cabe en nuestro dedo. Lo usábamos todos los días.", "¿Le cabía en el dedo?", "En este dedo. El de en medio. No, ese no. Este."),
                    dec("Saturno nos lo quitó mientras dormíamos. Ladrón, ladrón.", "¿Se lo quitó dormido?", "Nosotros no dormimos. Fingimos. Pero igual nos lo quitó."),
                    dec("Lo queremos porque hoy es nuestro cumpleaños. Y nadie nos regaló nada.", "¿Hoy es su cumpleaños?", "Todos los días es nuestro cumpleaños. Así pedimos regalos más seguido."),
                ],
                mentira: 1,
                protesto: "¡PROTESTO! El anillo A mide unos 15,000 kilómetros de ancho. ¡No le cabe en el dedo ni a Júpiter!",
                derrota: ["¿Quince mil kilómetros? ...Con razón se nos resbalaba.", "No es nuestro tamaño. Que se lo quede Saturno. Nosotros queremos un tesoro chiquito. Y un pescado."],
            },
        ],
        voz: Voz::Gollum,
        contesta: Quien::Ex(1),
    },
    Ex {
        nombre: "EL SINLUZ",
        anillo: 2,
        presentacion: "*El Sinluz hace una reverencia y deja un mensaje en el piso:* «anillo C adelante»",
        evidencia: "ANILLO C. El de adentro de los tres. El más oscuro y transparente: tiene más polvo y roca. Los anillos se le van cayendo a Saturno como lluvia: en unos cientos de millones de años ya no habrá.",
        testimonios: [
            Testimonio {
                titulo: "Mensajes en el piso",
                declaraciones: [
                    dec("«matrimonio adelante»", "¿Matrimonio adelante?", "Mi cliente dice que se casó después de morir 400 veces en la misma puerta. Saturno fue lo primero que vio al pasar."),
                    dec("«pruebe saltar, pero divorcio»", "¿Pruebe saltar?", "Es un consejo. Lo calificaron 3,000 personas. Todas se cayeron."),
                    dec("«anillo C, por lo tanto, el más brillante»", "¿El más brillante?", "Mi cliente dice que brilla como el Árbol Áureo. *el Sinluz hace el gesto de \"brillo\"*"),
                    dec("«visiones de anillo...»", "¿Visiones de anillo?", "Así termina todos sus mensajes. Es como su firma. Nadie sabe por qué."),
                ],
                mentira: 2,
                protesto: "¡PROTESTO! El anillo C es el MÁS OSCURO de los tres: tiene más polvo y roca, casi ni se ve. ¡Brillante, nada!",
                derrota: ["*el Sinluz hace el gesto de \"desesperación\" y deja otro mensaje:* «engaño adelante»", "*el Sinluz se va rodando. Rueda hasta la puerta. Rueda por la puerta. Sigue rodando.*"],
            },
            Testimonio {
                titulo: "Más mensajes en el piso",
                declaraciones: [
                    dec("«anillo, por lo tanto, eterno»", "¿Eterno?", "Mi cliente dice que el anillo va a durar para siempre. Como su búsqueda del Anillo de Elden."),
                    dec("«si tan solo tuviera un anillo...»", "¿Si tan solo tuviera un anillo?", "Lo escribió el día de la boda. Y hoy. Lo reutiliza."),
                    dec("«¡cuidado: abogado!»", "¿Cuidado, abogado?", "...Eso es para mí. Lo dejó en mi escritorio. Tres veces."),
                    dec("«Saturno adelante, por lo tanto, ex»", "¿Por lo tanto, ex?", "Es lo más romántico que ha escrito. Lloramos todos."),
                ],
                mentira: 0,
                protesto: "¡PROTESTO! Los anillos NO son eternos: se le están cayendo a Saturno como lluvia. ¡En unos cientos de millones de años ya no va a haber anillo!",
                derrota: ["*el Sinluz se queda quieto y deja un mensaje:* «tristeza adelante»", "*el Sinluz se sienta en la gracia más cercana a descansar. Afuera reaparecen todos los enemigos.*"],
            },
        ],
        voz: Voz::Mensaje,
        contesta: Quien::Phoenix,
    },
];

const APERTURA: [(Quien, &str); 4] = [
    (Quien::Juez, "¡HONK! (Traducción: se abre la sesión. Caso: Saturno contra sus tres ex.)"),
    (Quien::Phoenix, "Phoenix Wright, abogado de la parte demandante. Mis clientes se casaron con Saturno y cada uno quiere el anillo de su boda. No me pregunte cómo terminé aquí. Yo tampoco sé."),
    (Quien::Saturno, "Abogada Miku, se lo ruego. Sin mis anillos soy una bola. Como Júpiter. ...Bueno, Júpiter tiene anillos. Como Mercurio."),
    (Quien::Miku, "¡Tranquilo, Saturno! Si alguien miente, le grito ¡PROTESTO! Lo vi en un juego."),
];
const AVISO: &str = "Antes del interrogatorio, así se juega. Lea la tarjeta: es corta, lo prometo. (Las bodas son inventadas; los datos de los anillos, no.)";
const MAL: [&str; 3] = [
    "¡PROTESTO! La defensa protesta por protestar. Eso no contradice nada.",
    "¡PROTESTO! Su Señoría, eso es verdad. Hasta yo me sorprendí.",
    "¡PROTESTO! Esa es la única cosa cierta que ha dicho mi cliente en todo el día.",
];
const PHOENIX_SUDA: [&str; 3] = [
    "Mi cliente... mi cliente mintió. Qué raro. Nunca me pasa. Bueno, siempre me pasa.",
    "Esto ya lo viví. Muchas veces. Demasiadas. Así es mi vida.",
    "¡Su Señoría, pido un receso! ...¿No? ¿Ni uno chiquito?",
];
const PHOENIX_PIERDE: [&str; 3] = [
    "Perdí. Otra vez. Mi carrera de abogado va igual que la de cartas de mis clientes.",
    "Bueno, por lo menos el juez no se comió mis papeles. Esta vez.",
    "Voy a pedir que me paguen en anillos. Ah, no. Ya vi cómo termina eso.",
];
const TESTIGO_NUMERO: [&str; 3] = ["primer", "segundo", "último"];
/// Plumas de paciencia del juez en cada caso.
const PACIENCIA: u32 = 3;

/// Donde esta cada quien en la sala.
// La defensa va en +X: con la camara viendo hacia +Z, queda a la izquierda de la pantalla (como en el juego).
/// Miku es chiquita: se para en un escaloncito para que la mesa no la tape.
const MIKU_EN: Vec3 = Vec3::new(3.1, 0.2, 0.25);
const SATURNO_EN: Vec3 = Vec3::new(3.0, 1.1, 1.2);
const PHOENIX_EN: Vec3 = Vec3::new(-3.1, 0.0, 0.25);
const TESTIGO_EN: Vec3 = Vec3::new(0.0, 0.0, 1.3);
pub const JUEZ_EN: Vec3 = Vec3::new(0.0, 1.3, 3.55);
const PUBLICO: [Vec3; 3] = [Vec3::new(-2.2, 0.0, -2.15), Vec3::new(2.2, 0.0, -2.15), Vec3::new(-3.4, 0.0, -2.15)];

/// Segundos de las animaciones.
const PROTESTO: f32 = 1.3;
const MAZO: f32 = 0.9;

/// A donde fue a dar cada anillo.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Destino {
    Saturno,
    Ex,
    Ganso,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fase {
    Llegada,
    Apertura(usize),
    /// 0: Phoenix llama al testigo. 1: el testigo se presenta.
    Entra(usize),
    /// El testimonio de corrido, declaracion por declaracion.
    Testimonio(usize),
    /// Solo en el primer caso: GLADAS explica como se juega.
    Aviso,
    Interrogatorio,
    /// 0: Miku pregunta. 1: le contestan.
    Presion(usize),
    /// El grito, la mano y el golpe en la mesa.
    Protesto,
    /// 0: Miku explica. 1 y 2: el ex se derrumba. 3: Phoenix suda. 4: el juez le da el anillo a Saturno.
    Acierto(usize),
    /// 0: Phoenix protesta la protesta. 1: el juez pierde paciencia.
    Error(usize),
    /// Se le acabo la paciencia al juez. 0: decide. 1: Saturno reacciona.
    Perdido(usize),
    /// 0: el juez. 1: Saturno. 2: Phoenix. 3: la fiesta.
    Veredicto(usize),
    Libre,
}

pub enum AccionDivorcio {
    Nada,
    Despegar,
    /// Otro juicio desde el principio (otro orden, otros testimonios).
    OtroJuicio,
}

pub struct Divorcio {
    fase: Fase,
    t: f32,
    pub total: f32,
    semilla: u32,
    /// En que orden testifican los ex (indices en `EXES`) y que testimonio le toca a cada uno.
    orden: [usize; 3],
    variante: [usize; 3],
    casos: [Caso; 3],
    prueba: usize,
    /// Que caso va (0 a 2).
    caso: usize,
    /// La declaracion que se ve en el interrogatorio.
    cursor: usize,
    /// Las declaraciones que ya se presionaron (se marcan en la lista).
    presionadas: [bool; 4],
    paciencia: u32,
    /// A donde fue cada anillo (A, B, C), cuando ya se decidio.
    pub anillos: [Option<Destino>; 3],
    /// Si la protesta que esta en el aire es la buena.
    protesta_buena: bool,
    /// Lineas que cambian en cada juicio.
    mal: usize,
    suda: usize,
    pierde: usize,
    pub eventos: Vec<Efecto>,
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

impl Divorcio {
    pub fn new(semilla: u32) -> Self {
        let mut d = Divorcio {
            fase: Fase::Llegada,
            t: 0.0,
            total: 0.0,
            semilla,
            orden: [0, 1, 2],
            variante: [0; 3],
            casos: std::array::from_fn(|i| Caso::nuevo(i, 0, &mut 7)),
            prueba: 0,
            caso: 0,
            cursor: 0,
            presionadas: [false; 4],
            paciencia: PACIENCIA,
            anillos: [None; 3],
            protesta_buena: false,
            mal: 0,
            suda: 0,
            pierde: 0,
            eventos: Vec::new(),
        };
        for i in (1..3).rev() {
            let j = d.azar(i + 1);
            d.orden.swap(i, j);
        }
        for k in 0..3 {
            d.variante[k] = d.azar(5);
            d.casos[k] = Caso::nuevo(k, d.variante[k], &mut d.semilla);
        }
        d.mal = d.azar(MAL.len());
        d.suda = d.azar(PHOENIX_SUDA.len());
        d.pierde = d.azar(PHOENIX_PIERDE.len());
        d
    }

    fn azar(&mut self, n: usize) -> usize {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((self.semilla >> 8) as usize) % n
    }

    fn cambiar(&mut self, fase: Fase) {
        self.fase = fase;
        self.t = 0.0;
    }

    /// El ex del caso actual (indice en `EXES`).
    fn ex(&self) -> usize {
        self.orden[self.caso.min(2)]
    }

    fn testimonio(&self) -> &Caso {
        &self.casos[self.ex()]
    }

    pub fn otra_vez(&self, semilla: u32) -> Self {
        let mut nuevo = Self::new(semilla);
        for i in 0..3 {
            // Un nuevo juicio trae otro tipo de expediente para cada demandante.
            if nuevo.variante[i] == self.variante[i] {
                nuevo.variante[i] = (self.variante[i] + 1 + nuevo.azar(4)) % 5;
                nuevo.casos[i] = Caso::nuevo(i, nuevo.variante[i], &mut nuevo.semilla);
            }
        }
        if nuevo.orden == self.orden { nuevo.orden.rotate_left(1); }
        nuevo
    }

    /// Cuantos anillos se quedo Saturno, y cuantos se robo el ganso.
    pub fn cuantos(&self, destino: Destino) -> usize {
        self.anillos.iter().filter(|a| **a == Some(destino)).count()
    }

    pub fn terminado(&self) -> bool {
        matches!(self.fase, Fase::Veredicto(_) | Fase::Libre)
    }

    /// Si suena la fiesta (la cancion) en vez de la musica del juicio.
    pub fn de_fiesta(&self) -> bool {
        matches!(self.fase, Fase::Veredicto(3) | Fase::Libre)
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionDivorcio {
        self.t += dt;
        self.total += dt;
        if e.aterrizar {
            return AccionDivorcio::Despegar;
        }
        let largo = self.hablante().map_or(0, |h| h.1.chars().count()) as f32;
        let siguiente = (e.aceptar && self.t > 0.4) || self.t > 2.8 + largo / 20.0;
        match self.fase {
            Fase::Llegada if siguiente || self.t > 7.0 => {
                self.eventos.push(Efecto::Mazo);
                self.cambiar(Fase::Apertura(0));
            }
            Fase::Apertura(i) if siguiente => self.cambiar(if i + 1 < APERTURA.len() { Fase::Apertura(i + 1) } else { Fase::Entra(0) }),
            Fase::Entra(0) if siguiente => self.cambiar(Fase::Entra(1)),
            Fase::Entra(_) if siguiente => self.cambiar(Fase::Testimonio(0)),
            Fase::Testimonio(i) if siguiente => {
                if i < 3 {
                    self.cambiar(Fase::Testimonio(i + 1));
                } else {
                    self.cursor = 0;
                    self.cambiar(if self.caso == 0 { Fase::Aviso } else { Fase::Interrogatorio });
                }
            }
            Fase::Aviso if siguiente => self.cambiar(Fase::Interrogatorio),
            Fase::Interrogatorio if self.t > 0.3 => {
                if e.derecha {
                    self.cursor = (self.cursor + 1) % 4;
                    self.t = 0.0;
                } else if e.izquierda {
                    self.cursor = (self.cursor + 3) % 4;
                    self.t = 0.0;
                } else if e.arriba || e.abajo {
                    self.prueba = (self.prueba + if e.abajo { 1 } else { 2 }) % 3;
                    self.eventos.push(Efecto::Carta);
                } else if e.barrel {
                    self.protesta_buena = self.cursor == self.testimonio().mentira && self.prueba == self.testimonio().prueba;
                    self.eventos.push(Efecto::Protesto);
                    self.cambiar(Fase::Protesto);
                } else if e.aceptar {
                    self.presionadas[self.cursor] = true;
                    self.cambiar(Fase::Presion(0));
                }
            }
            Fase::Presion(0) if siguiente => self.cambiar(Fase::Presion(1)),
            Fase::Presion(_) if siguiente => self.cambiar(Fase::Interrogatorio),
            Fase::Protesto if self.t > PROTESTO => {
                if self.protesta_buena {
                    self.eventos.push(Efecto::Acierto);
                    self.cambiar(Fase::Acierto(0));
                } else {
                    self.eventos.push(Efecto::Fallo);
                    self.cambiar(Fase::Error(0));
                }
            }
            Fase::Acierto(3) if siguiente => {
                self.eventos.push(Efecto::Mazo);
                self.eventos.push(Efecto::Perfecto);
                self.anillos[EXES[self.ex()].anillo] = Some(Destino::Saturno);
                self.cambiar(Fase::Acierto(4));
            }
            Fase::Acierto(4) if siguiente && self.t > MAZO => self.siguiente_caso(),
            Fase::Acierto(i) if siguiente && i < 3 => self.cambiar(Fase::Acierto(i + 1)),
            Fase::Error(0) if siguiente => {
                self.paciencia = self.paciencia.saturating_sub(1);
                self.eventos.push(Efecto::Mazo);
                self.cambiar(Fase::Error(1));
            }
            Fase::Error(_) if siguiente => {
                if self.paciencia == 0 {
                    // El juez decide: o gana el ex, o el ganso se roba el anillo.
                    let destino = if self.azar(2) == 0 { Destino::Ex } else { Destino::Ganso };
                    self.anillos[EXES[self.ex()].anillo] = Some(destino);
                    self.eventos.push(Efecto::Mazo);
                    self.cambiar(Fase::Perdido(0));
                } else {
                    self.cambiar(Fase::Interrogatorio);
                }
            }
            Fase::Perdido(0) if siguiente => self.cambiar(Fase::Perdido(1)),
            Fase::Perdido(_) if siguiente => self.siguiente_caso(),
            Fase::Veredicto(0) if siguiente => self.cambiar(Fase::Veredicto(1)),
            Fase::Veredicto(i) if siguiente && i < 3 => self.cambiar(Fase::Veredicto(i + 1)),
            Fase::Veredicto(_) if siguiente => self.cambiar(Fase::Libre),
            Fase::Libre if e.aceptar && self.t > 0.6 => return AccionDivorcio::OtroJuicio,
            _ => {}
        }
        AccionDivorcio::Nada
    }

    fn siguiente_caso(&mut self) {
        self.caso += 1;
        self.paciencia = PACIENCIA;
        self.cursor = 0;
        self.prueba = 0;
        self.presionadas = [false; 4];
        if self.caso < 3 {
            self.cambiar(Fase::Entra(0));
        } else {
            self.eventos.push(Efecto::Mazo);
            self.cambiar(Fase::Veredicto(0));
        }
    }

    pub fn hablante(&self) -> Option<(Quien, String, Voz)> {
        let ex = &EXES[self.ex()];
        let anillo = ANILLOS[ex.anillo];
        let t = self.testimonio();
        let decir = |q: Quien, s: String| Some((q, s, voz(q)));
        match self.fase {
            Fase::Llegada => decir(Quien::Gladas, "Venus: 465 grados y lluvia de ácido, como todo buen divorcio. Saturno vino a divorciarse: tiene tres anillos, A, B y C, y cada uno es de una boda. Usted es su abogada. No pregunte; yo tampoco entendí.".into()),
            Fase::Apertura(i) => decir(APERTURA[i].0, APERTURA[i].1.into()),
            Fase::Entra(0) => decir(Quien::Phoenix, format!("La parte demandante llama a su {} testigo: ¡{}! Pide el anillo {}.", TESTIGO_NUMERO[self.caso.min(2)], titulo(ex.nombre), anillo)),
            Fase::Entra(_) => decir(Quien::Ex(self.ex()), ex.presentacion.into()),
            Fase::Testimonio(i) => decir(Quien::Ex(self.ex()), t.declaraciones[i].dice.clone()),
            Fase::Aviso => decir(Quien::Gladas, AVISO.into()),
            Fase::Interrogatorio => decir(Quien::Ex(self.ex()), t.declaraciones[self.cursor].dice.clone()),
            Fase::Presion(0) => decir(Quien::Miku, t.declaraciones[self.cursor].pregunta.clone()),
            Fase::Presion(_) => decir(ex.contesta, t.declaraciones[self.cursor].respuesta.clone()),
            Fase::Protesto => None,
            Fase::Acierto(0) => decir(Quien::Miku, t.protesto.clone()),
            Fase::Acierto(i @ (1 | 2)) => decir(Quien::Ex(self.ex()), t.derrota[i - 1].into()),
            Fase::Acierto(3) => decir(Quien::Phoenix, PHOENIX_SUDA[(self.suda + self.caso) % PHOENIX_SUDA.len()].into()),
            Fase::Acierto(_) => decir(Quien::Juez, format!("¡HONK! El anillo {anillo} se queda con Saturno. {}", t.consecuencia)),
            Fase::Error(0) if self.cursor == t.mentira => decir(Quien::Phoenix, "Esa frase huele a mentira, pero el documento que presentó no la contradice. ¡Hay que probarlo! Revise las otras pruebas con arriba y abajo.".into()),
            Fase::Error(0) => decir(Quien::Phoenix, MAL[(self.mal + self.paciencia as usize) % MAL.len()].into()),
            Fase::Error(_) => decir(
                Quien::Juez,
                match self.paciencia {
                    0 => "¡HOOONK! (Traducción: se me acabó la paciencia.)".into(),
                    1 => "¡HONK! (Traducción: una más y decido yo. Me queda una pluma de paciencia.)".into(),
                    n => format!("¡HONK! (Traducción: ¡orden en la sala! Me quedan {n} plumas de paciencia.)"),
                },
            ),
            Fase::Perdido(0) => match self.anillos[ex.anillo] {
                Some(Destino::Ganso) => decir(Quien::Juez, format!("¡HONK! *el juez se roba el anillo {anillo} y se lo pone de collar* (Traducción: ahora es mío.)")),
                _ => decir(Quien::Juez, format!("¡HONK! (Traducción: el anillo {anillo} es para {}.)", titulo(ex.nombre))),
            },
            Fase::Perdido(_) => decir(
                Quien::Saturno,
                if self.anillos[ex.anillo] == Some(Destino::Ganso) {
                    "¡Eso ni siquiera es legal! ...¿O sí? ¿Quién le reclama a un ganso?".into()
                } else {
                    "¡Mi anillo! ...Bueno. Igual nunca me quedó bien.".into()
                },
            ),
            Fase::Veredicto(0) => decir(Quien::Juez, "¡HONK! ¡HONK! ¡HONK! (Traducción: se declara a Saturno... ¡DIVORCIADO!)".into()),
            Fase::Veredicto(1) => decir(
                Quien::Saturno,
                match self.cuantos(Destino::Saturno) {
                    3 => "¡Mis tres anillos! Soltero, ligero... flotaría en una tina. Bueno, eso ya lo hacía.",
                    2 => "Me quedan dos anillos. Igual sigo siendo el más bonito del sistema solar. Perdón, Urano.",
                    1 => "Un anillo. Bueno, Neptuno tiene anillitos y nadie se burla. ...Bueno, sí se burlan.",
                    _ => "Sin anillos... Soy Júpiter con depresión. Bueno, Júpiter tiene anillos. Soy Mercurio con depresión.",
                }
                .into(),
            ),
            Fase::Veredicto(2) if self.cuantos(Destino::Ganso) > 0 => decir(Quien::Juez, "honk. (Traducción: el juez se queda con su collar nuevo. No acepta apelaciones.)".into()),
            Fase::Veredicto(2) => decir(Quien::Phoenix, PHOENIX_PIERDE[self.pierde].into()),
            Fase::Veredicto(_) => decir(
                Quien::Saturno,
                if self.cuantos(Destino::Saturno) == 0 {
                    "¡Igual hay fiesta de divorcio en Saturno Ritmo! Sin anillos, pero con dignidad. Miku, ¿cantas?".into()
                } else {
                    "¡Esto hay que celebrarlo! ¡Fiesta de divorcio en mi escenario! Miku, te espero en Saturno Ritmo. Pongo los anillos de luces.".into()
                },
            ),
            Fase::Libre => decir(Quien::Juez, "honk. (Traducción: se levanta la sesión.)".into()),
        }
    }

    /// El nombre que sale en la caja de dialogo.
    pub fn nombre(&self, q: Quien) -> &'static str {
        match q {
            Quien::Gladas => ui::NOMBRE_IA,
            Quien::Miku => "MIKU",
            Quien::Saturno => "SATURNO",
            Quien::Phoenix => "PHOENIX WRIGHT",
            Quien::Juez => "EL JUEZ",
            Quien::Ex(i) => EXES[i].nombre,
        }
    }

    // ---------- donde esta cada quien ----------

    pub fn paso_miku(&self) -> Paso {
        match self.fase {
            Fase::Protesto | Fase::Acierto(0) => Paso::Senalar,
            Fase::Libre | Fase::Veredicto(3) => Paso::Baile,
            Fase::Veredicto(1) if self.cuantos(Destino::Saturno) > 0 => Paso::Corazon,
            _ => Paso::Quieta,
        }
    }

    pub fn expresion_miku(&self) -> &'static str {
        match self.fase {
            Fase::Protesto | Fase::Acierto(0) => "serio",
            Fase::Error(_) | Fase::Perdido(_) => "asustada",
            Fase::Interrogatorio | Fase::Presion(_) => "serio",
            Fase::Veredicto(_) | Fase::Libre => "guino",
            _ => "feliz",
        }
    }

    /// Miku detras de la mesa de la defensa, viendo hacia el centro (-X). Al protestar da un pasito al frente.
    pub fn miku(&self) -> Mat4 {
        let golpe = if self.fase == Fase::Protesto { 0.12 * suave(self.t / 0.15) } else { 0.0 };
        Mat4::translation(MIKU_EN - Vec3::new(golpe, 0.0, 0.0)) * Mat4::rotation_y(FRAC_PI_2) * Mat4::scale(0.88)
    }

    pub fn expresion_saturno(&self) -> &'static str {
        match self.fase {
            Fase::Perdido(_) => "espiral",
            Fase::Acierto(4) | Fase::Veredicto(_) | Fase::Libre if self.cuantos(Destino::Saturno) > 0 => "enamorada",
            Fase::Acierto(_) => "feliz",
            Fase::Veredicto(_) | Fase::Libre => "dormida",
            _ => "asustada",
        }
    }

    /// Saturno flotando junto a Miku, meciendose, con los anillos inclinados.
    pub fn saturno(&self) -> Mat4 {
        let flota = 0.05 * (self.total * 1.3).sin();
        let brinco = if self.fase == Fase::Acierto(4) { 0.15 * (self.t * 8.0).sin().abs() * (1.0 - self.t).max(0.0) } else { 0.0 };
        Mat4::translation(SATURNO_EN + Vec3::new(0.0, flota + brinco, 0.0)) * Mat4::rotation_y(FRAC_PI_2) * Mat4::rotation_x(0.35) * Mat4::rotation_z(0.15) * Mat4::scale(1.1)
    }

    /// Los anillos que ya no tiene Saturno (para ocultarlos).
    pub fn anillos_perdidos(&self) -> Vec<&'static str> {
        (0..3).filter(|&k| matches!(self.anillos[k], Some(Destino::Ex | Destino::Ganso))).map(|k| PARTES_ANILLO[k]).collect()
    }

    /// Los anillos que se robo el ganso, colgados de su cuello: (anillo, matriz).
    pub fn collares(&self) -> Vec<(usize, Mat4)> {
        (0..3)
            .filter(|&k| self.anillos[k] == Some(Destino::Ganso))
            .map(|k| {
                let (base, _) = self.juez();
                let m = base * Mat4::translation(Vec3::new(0.0, 0.62 + 0.04 * k as f32, -0.14)) * Mat4::rotation_x(0.25) * Mat4::scale(0.42 - 0.04 * k as f32);
                (k, m)
            })
            .collect()
    }

    pub fn expresion_phoenix(&self) -> &'static str {
        match self.fase {
            Fase::Acierto(1..=3) | Fase::Veredicto(2) => "asustada",
            Fase::Error(0) | Fase::Entra(0) => "serio",
            Fase::Protesto => "asustada",
            _ => "feliz",
        }
    }

    /// Phoenix detras de su mesa, viendo hacia el centro (+X). Golpea la mesa al protestar.
    pub fn phoenix(&self) -> Mat4 {
        let golpe = if self.fase == Fase::Error(0) { 0.1 * suave(self.t / 0.15) * (1.0 - suave((self.t - 1.0) / 0.3)) } else { 0.0 };
        Mat4::translation(PHOENIX_EN + Vec3::new(golpe, 0.0, 0.0)) * Mat4::rotation_y(-FRAC_PI_2) * Mat4::rotation_x(-golpe) * Mat4::scale(1.0)
    }

    /// El ganso juez arriba del estrado (con el mazo). Cuando habla, cabecea; en el veredicto aletea.
    /// Devuelve la matriz y si esta hablando.
    pub fn juez(&self) -> (Mat4, bool) {
        let habla = self.hablante().is_some_and(|h| h.0 == Quien::Juez);
        let cabeceo = if habla { 0.15 * (self.total * 14.0).sin().max(0.0) } else { 0.03 * (self.total * 1.1).sin() };
        let giro = 0.2 * (self.total * 0.4).sin();
        (Mat4::translation(JUEZ_EN) * Mat4::rotation_y(giro) * Mat4::rotation_x(-cabeceo) * Mat4::scale(1.1), habla)
    }

    /// Los tres ex: el del caso en el banquillo, los demas en las bancas del publico. (ex, matriz, expresion).
    pub fn exes(&self) -> Vec<(usize, Mat4, Option<&'static str>)> {
        (0..3)
            .map(|k| {
                let ex = self.orden[k];
                let en_banquillo = k == self.caso && !self.terminado();
                let decidido = self.anillos[EXES[ex].anillo];
                let expresion = if en_banquillo {
                    match self.fase {
                        Fase::Acierto(1) | Fase::Protesto if self.protesta_buena => "asustada",
                        Fase::Acierto(2) => "espiral",
                        Fase::Perdido(_) if decidido == Some(Destino::Ex) => "enamorada",
                        Fase::Perdido(_) => "enojado",
                        _ => "feliz",
                    }
                } else {
                    match decidido {
                        Some(Destino::Saturno) => "espiral",
                        Some(Destino::Ex) => "enamorada",
                        Some(Destino::Ganso) => "enojado",
                        None => "serio",
                    }
                };
                // El Sinluz no tiene cara (casco cerrado).
                let expresion = (ex != 2).then_some(expresion);
                let m = if en_banquillo {
                    // Sonic no se esta quieto: brinca en su lugar.
                    let brinco = if ex == 0 { 0.04 * (self.total * 9.0).sin().abs() } else { 0.0 };
                    let derrumbe = if matches!(self.fase, Fase::Acierto(1 | 2)) { 0.25 * suave(self.t / 0.3) } else { 0.0 };
                    Mat4::translation(TESTIGO_EN + Vec3::new(0.0, brinco, 0.0)) * Mat4::rotation_x(derrumbe)
                } else {
                    Mat4::translation(PUBLICO[k]) * Mat4::rotation_y(PI)
                };
                (ex, m, expresion)
            })
            .collect()
    }

    /// Si el Sinluz esta en el banquillo (para su luz de gracia).
    fn sinluz_testifica(&self) -> bool {
        self.ex() == 2 && !self.terminado() && !matches!(self.fase, Fase::Llegada | Fase::Apertura(_))
    }

    /// La camara: una toma por quien habla, como en Ace Attorney (cortes secos). (ojo, mira, arriba)
    pub fn camara(&self) -> (Vec3, Vec3, Vec3) {
        let arriba = Vec3::new(0.0, 1.0, 0.0);
        let vaiven = Vec3::new(0.06 * (self.total * 0.35).sin(), 0.03 * (self.total * 0.5).sin(), 0.0);
        let general = (Vec3::new(0.0, 2.6, -4.8), Vec3::new(0.0, 1.0, 1.5));
        let (ojo, mira) = match self.fase {
            Fase::Llegada => {
                // Baja de las nubes y entra a la sala.
                let u = suave(self.t / 6.0);
                (Vec3::new(0.0, 9.0 - 6.4 * u, -14.0 + 9.2 * u), Vec3::new(0.0, 2.0 - 1.0 * u, 3.0 - 1.5 * u))
            }
            Fase::Protesto => {
                // Zoom de golpe a Miku, con temblor.
                let z = suave(self.t / 0.12);
                let temblor = Vec3::new(0.0, 0.04 * (self.t * 60.0).sin(), 0.04 * (self.t * 47.0).cos()) * (1.0 - suave((self.t - 0.4) / 0.6));
                (Vec3::new(1.5 + 0.45 * z, 1.05, -0.55 + 0.35 * z) + temblor, MIKU_EN + Vec3::new(0.0, 0.85, 0.15))
            }
            Fase::Veredicto(0) | Fase::Libre => general,
            Fase::Veredicto(3) => (Vec3::new(0.8, 1.8, -2.6), Vec3::new(2.2, 0.9, 0.8)),
            _ => match self.hablante().map(|h| h.0) {
                Some(Quien::Miku) => (Vec3::new(1.45, 1.1, -0.8), MIKU_EN + Vec3::new(0.0, 0.8, 0.3)),
                Some(Quien::Saturno) => (Vec3::new(1.55, 1.35, 2.05), SATURNO_EN + Vec3::new(0.0, -0.05, 0.0)),
                Some(Quien::Phoenix) => (Vec3::new(-1.4, 1.25, -0.8), PHOENIX_EN + Vec3::new(0.0, 0.95, 0.3)),
                Some(Quien::Juez) => (Vec3::new(0.0, 1.5, 1.2), JUEZ_EN + Vec3::new(0.0, 0.75, 0.0)),
                Some(Quien::Ex(_)) => (Vec3::new(0.25, 1.1, -0.9), TESTIGO_EN + Vec3::new(0.0, 0.75, 0.0)),
                _ => general,
            },
        };
        (ojo + vaiven, mira, arriba)
    }

    /// Cuanto destello blanco hay (el golpe del ¡PROTESTO!).
    pub fn destello(&self) -> f32 {
        if self.fase == Fase::Protesto { (1.0 - self.t / 0.25).max(0.0) * 0.8 } else { 0.0 }
    }

    /// La luz de gracia dorada del Sinluz y el brillo de los anillos robados.
    pub fn dibujar_luces(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4) {
        if self.sinluz_testifica() {
            let k = 0.7 + 0.3 * (self.total * 2.0).sin();
            if let Some((x, y, _)) = proyectar(TESTIGO_EN + Vec3::new(0.35, 0.3, -0.3), vista_proyeccion, fb) {
                crate::post::halo(fb, (x, y), 50.0, [0.6 * k, 0.45 * k, 0.1 * k], 1.6);
            }
        }
        for (_, m) in self.collares() {
            if let Some((x, y, _)) = proyectar(m.transform_point(Vec3::zero()), vista_proyeccion, fb) {
                crate::post::halo(fb, (x, y), 40.0, [0.4, 0.35, 0.2], 1.6);
            }
        }
    }

    /// Las nubes de Venus en el cielo: franjas de luz que corren al reves (Venus gira al reves, y sus
    /// nubes le dan la vuelta en solo cuatro dias, mucho mas rapido que el planeta).
    pub fn dibujar_cielo(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4) {
        for k in 0..26 {
            let altura = 6.0 + (k as f32 * 2.3) % 22.0;
            let z = -60.0 + (k as f32 * 37.0) % 160.0;
            let largo = 14.0 + (k % 5) as f32 * 6.0;
            let x = 90.0 - (self.total * (3.0 + (k % 3) as f32) + k as f32 * 29.0) % 180.0;
            let a = proyectar(Vec3::new(x, altura, z), vista_proyeccion, fb);
            let b = proyectar(Vec3::new(x + largo, altura, z), vista_proyeccion, fb);
            if let (Some(a), Some(b)) = (a, b) {
                let c = 0.10 + 0.05 * (k % 3) as f32;
                for dy in 0..3 {
                    let d = dy as f32 * 1.5;
                    fb.linea_de_luz((a.0, a.1 + d, a.2), (b.0, b.1 + d, b.2), [c, c * 0.85, c * 0.5], [c, c * 0.85, c * 0.5]);
                }
            }
        }
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, ancho: i32, alto: i32) {
        let (aceptar, salir, protesto) = (e.nombre("aceptar"), e.nombre("aterrizar"), e.nombre("barrel"));
        // Los mismos botones, para meterlos en una frase (con su icono).
        let (b_aceptar, b_salir, b_protesto) = (ui::boton(aceptar), ui::boton(salir), ui::boton(protesto));
        let centrado = |d: &mut RaylibDrawHandle, s: &str, y: i32, tam: i32, color: Color| ui::centrado(d, s, ancho, y, tam, color);
        if self.fase == Fase::Llegada {
            let a = (suave(self.t / 0.8) * (1.0 - suave((self.t - 5.5) / 1.0)) * 255.0) as u8;
            centrado(d, "JUZGADO DE LO FAMILIAR DE VENUS", 70, 46, Color::new(255, 225, 150, a));
            centrado(d, "Caso: Saturno contra sus tres ex", 132, 26, Color::new(255, 245, 230, a));
        }
        let en_juicio = !matches!(self.fase, Fase::Llegada | Fase::Apertura(_) | Fase::Veredicto(_) | Fase::Libre);
        // Arriba a la izquierda: la paciencia del juez (plumas) y los anillos.
        if !matches!(self.fase, Fase::Llegada) {
            ui::panel(d, 16, 16, 250, 92);
            ui::texto(d, "ANILLOS DE SATURNO", 30, 24, 16, ui::TENUE);
            for k in 0..3 {
                let (color, marca) = match self.anillos[k] {
                    Some(Destino::Saturno) => (ui::VERDE, "sí"),
                    Some(Destino::Ex) => (ui::ROJO, "ex"),
                    Some(Destino::Ganso) => (Color::new(255, 200, 120, 255), "ganso"),
                    None => (ui::TEXTO, "?"),
                };
                let x = 30 + k as i32 * 76;
                ui::texto(d, ANILLOS[k], x, 44, 24, color);
                ui::texto(d, marca, x + 20, 50, 16, color);
            }
            if en_juicio {
                ui::texto(d, "PACIENCIA DEL JUEZ", 30, 76, 14, ui::TENUE);
                for p in 0..PACIENCIA {
                    let x = 180 + p as i32 * 24;
                    let c = if p < self.paciencia { Color::WHITE } else { Color::new(90, 80, 70, 255) };
                    d.draw_ellipse(x, 84, 6.0, 11.0, c);
                    d.draw_line(x, 74, x, 96, Color::new(120, 110, 100, 255));
                }
            }
        }
        // El expediente del anillo en disputa, arriba a la derecha: compacto, para no tapar la sala.
        let con_expediente = matches!(self.fase, Fase::Testimonio(_) | Fase::Aviso | Fase::Interrogatorio | Fase::Presion(_) | Fase::Protesto | Fase::Error(_));
        if con_expediente {
            let w = 360.min(ancho / 3);
            let x = ancho - w - 16;
            let tam = 15;
            let cuerpo = ui::envolver(&self.testimonio().pruebas[self.prueba].1, tam, w - 28);
            let h = 70 + cuerpo.len() as i32 * (tam + tam / 3) + 8;
            ui::panel(d, x, 16, w, h);
            ui::texto(d, &format!("PRUEBA {}/3 · arriba / abajo", self.prueba + 1), x + 14, 24, 15, Color::new(255, 210, 120, 255));
            ui::texto(d, &self.testimonio().pruebas[self.prueba].0, x + 14, 46, 14, ui::TENUE);
            ui::parrafo(d, &self.testimonio().pruebas[self.prueba].1, x + 14, 68, tam, w - 28, ui::TEXTO);
        }
        // Las declaraciones del testigo, en una lista a la izquierda (las que ya dijo), con la
        // escogida marcada: asi se pueden comparar todas con el expediente.
        let dichas = match self.fase {
            Fase::Testimonio(i) => i + 1,
            Fase::Aviso | Fase::Interrogatorio | Fase::Presion(_) | Fase::Protesto | Fase::Error(_) => 4,
            _ => 0,
        };
        let interrogando = matches!(self.fase, Fase::Interrogatorio | Fase::Presion(_) | Fase::Protesto | Fase::Error(_));
        if dichas > 0 {
            let w = 400.min(ancho / 3);
            let (x, y0) = (16, 124);
            let tam = 15;
            let t = self.testimonio();
            let lineas: Vec<Vec<String>> = t.declaraciones[..dichas].iter().map(|dc| ui::envolver(&dc.dice, tam, w - 54)).collect();
            // Despues de protestar mal, una pista al pie de la lista.
            let pista = (interrogando && self.paciencia < PACIENCIA)
                .then_some("Pista: lea las 3 pruebas (arriba / abajo) y busque un dato que no coincida: un lugar, una fecha, un nombre, un número. Protesta con esa prueba en pantalla.");
            let lineas_pista = pista.map_or(Vec::new(), |p| ui::envolver(p, 13, w - 28));
            let h = 64 + lineas.iter().map(|l| l.len() as i32 * (tam + 5) + 10).sum::<i32>() + lineas_pista.len() as i32 * 17 + if pista.is_some() { 10 } else { 0 };
            ui::panel(d, x, y0, w, h);
            let titulo = if interrogando { "¿CUÁL ES LA MENTIRA?" } else { "TESTIMONIO" };
            ui::texto(d, titulo, x + 14, y0 + 10, 18, if interrogando { Color::new(120, 230, 140, 255) } else { Color::new(255, 150, 90, 255) });
            let sub = if interrogando { "una de las 4 choca con una de las pruebas" } else { &t.titulo };
            ui::texto(d, sub, x + 14, y0 + 34, 13, ui::TENUE);
            let mut y = y0 + 58;
            for (k, ls) in lineas.iter().enumerate() {
                let alto_k = ls.len() as i32 * (tam + 5) + 4;
                let escogida = interrogando && k == self.cursor;
                if escogida {
                    d.draw_rectangle_rounded(Rectangle::new((x + 8) as f32, (y - 4) as f32, (w - 16) as f32, (alto_k + 4) as f32), 0.3, 6, Color::new(120, 230, 140, 60));
                    d.draw_rectangle(x + 8, y - 4, 4, alto_k + 4, Color::new(120, 230, 140, 255));
                }
                let color_n = if escogida { Color::new(120, 230, 140, 255) } else { ui::TENUE };
                ui::texto(d, &format!("{}", k + 1), x + 20, y, tam + 2, color_n);
                let color = if escogida { Color::WHITE } else { Color::new(220, 212, 235, 255) };
                for (l, linea) in ls.iter().enumerate() {
                    ui::texto(d, linea, x + 42, y + l as i32 * (tam + 5), tam, color);
                }
                if self.presionadas[k] && interrogando {
                    ui::texto(d, "?", x + w - 24, y, tam, Color::new(255, 210, 120, 255));
                }
                y += alto_k + 6;
            }
            for (l, linea) in lineas_pista.iter().enumerate() {
                ui::texto(d, linea, x + 14, y + 4 + l as i32 * 17, 13, Color::new(255, 210, 120, 255));
            }
        }
        // El titulo del testimonio.
        if let Fase::Testimonio(0) = self.fase {
            let a = (suave(self.t / 0.3) * 255.0) as u8;
            centrado(d, "~ TESTIMONIO ~", 120, 26, Color::new(255, 150, 90, a));
            centrado(d, &self.testimonio().titulo, 152, 28, Color::new(255, 240, 220, a));
        }
        // Interrogatorio: los botones, en su propia barra encima del dialogo (no encima de la sala).
        if matches!(self.fase, Fase::Interrogatorio) {
            let (izq, der) = (ui::IZQUIERDA, ui::DERECHA);
            let acciones = [
                (format!("{izq} {der}"), "otra declaración", false),
                (aceptar.to_string(), "preguntar más", false),
                (protesto.to_string(), "presentar prueba", true),
                (salir.to_string(), "salir", false),
            ];
            let tam = 17;
            let anchos: Vec<i32> = acciones.iter().map(|(b, q, _)| ui::medir_accion(b, q, tam) + 26).collect();
            let total: i32 = anchos.iter().sum();
            let y = alto - 200;
            let mut x = (ancho - total) / 2;
            d.draw_rectangle_rounded(Rectangle::new((x - 14) as f32, (y - 8) as f32, (total + 28) as f32, (tam + tam / 2 + 16) as f32), 0.4, 8, Color::new(15, 8, 30, 220));
            for ((b, q, fuerte), w) in acciones.iter().zip(anchos) {
                ui::accion(d, b, q, x, y, tam, *fuerte);
                x += w;
            }
        }
        // Solo la primera vez: la tarjeta de como se juega, en grande.
        if self.fase == Fase::Aviso {
            let w = 760.min(ancho - 40);
            let pasos = [
                ("1", "El testigo dijo 4 cosas. UNA es mentira.".to_string()),
                ("2", "Hay 3 PRUEBAS (arriba a la derecha): cámbielas con {arriba_abajo}. Una de ellas desmiente la mentira.".to_string()),
                ("3", "Pase de una declaración a otra con {izq_der}.".to_string()),
                ("4", format!("Con la declaración falsa Y la prueba que la desmiente en pantalla: {b_protesto} ¡PROTESTO!")),
                ("5", format!("¿No está segura? {b_aceptar} para preguntar más: a veces sueltan la sopa.")),
                ("!", "Si protesta mal (o con la prueba equivocada), el juez pierde una pluma de paciencia. Sin plumas, el ex se queda con el anillo.".to_string()),
            ];
            let tam = 19;
            let alto_pasos: i32 = pasos.iter().map(|(_, t)| ui::envolver(t, tam, w - 110).len() as i32 * (tam + 7) + 14).sum();
            let h = 90 + alto_pasos + 40;
            let (x, y) = ((ancho - w) / 2, (alto - h) / 2 - 60);
            d.draw_rectangle(0, 0, ancho, alto, Color::new(0, 0, 0, 120));
            ui::panel(d, x, y, w, h);
            ui::texto(d, "CÓMO SE JUEGA", x + 30, y + 22, 30, Color::new(255, 210, 120, 255));
            ui::texto(d, "(como Ace Attorney, pero con anillos de Saturno)", x + 30, y + 58, 15, ui::TENUE);
            let mut yy = y + 96;
            for (n, texto) in &pasos {
                let color = if *n == "!" { Color::new(255, 150, 120, 255) } else { Color::new(120, 230, 140, 255) };
                d.draw_circle(x + 46, yy + 11, 16.0, color);
                ui::texto(d, n, x + 46 - ui::medir(n, 18) / 2, yy + 2, 18, Color::new(15, 10, 30, 255));
                yy = ui::parrafo(d, texto, x + 80, yy, tam, w - 110, ui::TEXTO) + 14;
            }
            let seguir = format!("{b_aceptar}: ¡a protestar!");
            ui::texto(d, &seguir, x + w - ui::medir(&seguir, 18) - 26, y + h - 36, 18, Color::new(255, 230, 160, 255));
        }
        // ¡PROTESTO!: la burbuja con picos, como en Ace Attorney.
        if self.fase == Fase::Protesto {
            let k = suave(self.t / 0.12);
            let (cx, cy) = (ancho as f32 * 0.5, alto as f32 * 0.38);
            let r = 230.0 * k;
            // Lineas de velocidad atras.
            for i in 0..36 {
                let a = i as f32 / 36.0 * TAU;
                let (c, s) = (a.cos(), a.sin());
                d.draw_line_ex(Vector2::new(cx + c * r * 0.9, cy + s * r * 0.6), Vector2::new(cx + c * 1400.0, cy + s * 1400.0), 3.0, Color::new(255, 255, 255, 90));
            }
            let picos = 18;
            let punto = |i: usize| {
                let a = i as f32 / picos as f32 * TAU;
                let rr = if i % 2 == 0 { r } else { r * 0.72 };
                Vector2::new(cx + a.cos() * rr * 1.3, cy + a.sin() * rr * 0.62)
            };
            for (borde, color) in [(1.08, Color::new(200, 20, 30, 255)), (1.0, Color::WHITE)] {
                for i in 0..picos {
                    let (p0, p1) = (punto(i), punto(i + 1));
                    let esc = |p: Vector2| Vector2::new(cx + (p.x - cx) * borde, cy + (p.y - cy) * borde);
                    let (a, b, c) = (Vector2::new(cx, cy), esc(p0), esc(p1));
                    d.draw_triangle(a, b, c, color);
                    d.draw_triangle(a, c, b, color);
                }
            }
            if k > 0.5 {
                let tam = 78;
                let s = "¡PROTESTO!";
                let w = ui::medir_titulo(s, tam);
                let (x, y) = ((cx as i32) - w / 2, cy as i32 - tam / 2);
                ui::titulo_de(d, s, x, y, tam, Color::new(230, 30, 40, 255));
            }
        }
        // El veredicto: el sello de DIVORCIADO.
        if matches!(self.fase, Fase::Veredicto(0)) && self.t > 0.6 {
            let k = suave((self.t - 0.6) / 0.25);
            let tam = (140.0 - 50.0 * k) as i32;
            let s = "DIVORCIADO";
            let w = ui::medir(s, tam);
            let a = (k * 255.0) as u8;
            let (x, y) = ((ancho - w) / 2, alto / 2 - 140);
            d.draw_rectangle_lines_ex(Rectangle::new((x - 24) as f32, (y - 10) as f32, (w + 48) as f32, (tam + 20) as f32), 8.0, Color::new(220, 30, 40, a));
            ui::texto(d, s, x, y, tam, Color::new(220, 30, 40, a));
        }
        if self.fase == Fase::Libre {
            centrado(d, &format!("{b_aceptar}: otro juicio      {b_salir}: despegar"), 130, 24, Color::new(255, 230, 200, 255));
        }
        // Lo que se dice, abajo. En el interrogatorio el texto del testigo va en verde, como en el juego.
        if let (Some((quien, _, _)), false) = (self.hablante(), texto.is_empty()) {
            let nombre = self.nombre(quien);
            let w = 940.min(ancho - 40);
            let h = ui::alto_dialogo(nombre, texto, w);
            let color = match quien {
                Quien::Miku => Color::new(120, 230, 220, 255),
                Quien::Phoenix => Color::new(120, 160, 255, 255),
                Quien::Juez => Color::new(255, 220, 120, 255),
                Quien::Saturno => Color::new(255, 200, 140, 255),
                Quien::Ex(_) if self.fase == Fase::Interrogatorio => Color::new(120, 230, 140, 255),
                Quien::Ex(_) => Color::new(255, 150, 120, 255),
                Quien::Gladas => ui::LILA,
            };
            ui::caja_dialogo(d, nombre, color, texto, (ancho - w) / 2, alto - h - 20, w);
        }
    }

    /// Juega solo hasta llegar a la fase que empieza con `objetivo` (con `#n`, la n-esima vez):
    /// para las capturas. Protesta en la declaracion buena, salvo con `mal` en el objetivo.
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
            if self.fase == Fase::Interrogatorio && self.t > 0.4 {
                let buena = self.testimonio().mentira;
                let prueba = self.testimonio().prueba;
                if mal {
                    if self.cursor == buena {
                        e.derecha = true;
                    } else {
                        e.barrel = true;
                    }
                } else if self.cursor != buena {
                    e.derecha = true;
                } else if self.prueba != prueba {
                    e.abajo = true;
                } else {
                    e.barrel = true;
                }
            } else if self.hablante().is_some() && self.t > 0.5 {
                e.aceptar = true;
            }
            self.actualizar(&e, 0.05);
        }
    }
}

/// "SONIC" -> "Sonic", "EL SINLUZ" -> "el Sinluz".
fn titulo(nombre: &str) -> String {
    nombre
        .split(' ')
        .map(|p| if p == "EL" { "el".to_string() } else { p[..1].to_string() + &p[1..].to_lowercase() })
        .collect::<Vec<_>>()
        .join(" ")
}

fn voz(q: Quien) -> Voz {
    match q {
        Quien::Gladas => Voz::Ia,
        Quien::Miku => Voz::Miku,
        Quien::Saturno => Voz::Planeta,
        Quien::Phoenix => Voz::Abogado,
        Quien::Juez => Voz::Ganso,
        Quien::Ex(i) => EXES[i].voz,
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Protestando siempre en la mentira, Saturno se queda con sus tres anillos y hay veredicto.
    #[test]
    fn protestar_bien_salva_los_tres_anillos() {
        for semilla in 1..12 {
            let mut d = Divorcio::new(semilla);
            d.jugar_hasta("Veredicto");
            assert!(d.terminado(), "el juicio {semilla} no termino: {:?}", d.fase);
            assert_eq!(d.cuantos(Destino::Saturno), 3);
            assert!(d.anillos_perdidos().is_empty());
        }
    }

    /// Protestando siempre mal, se acaba la paciencia en cada caso y Saturno pierde los tres.
    #[test]
    fn protestar_mal_pierde_los_anillos() {
        let mut d = Divorcio::new(7);
        d.jugar_hasta("Veredicto_mal");
        assert!(d.terminado());
        assert_eq!(d.cuantos(Destino::Saturno), 0);
        assert_eq!(d.cuantos(Destino::Ex) + d.cuantos(Destino::Ganso), 3);
        assert_eq!(d.anillos_perdidos().len(), 3);
        assert_eq!(d.collares().len(), d.cuantos(Destino::Ganso));
    }

    /// Cada ex pide un anillo distinto, y su mentira es una declaracion valida en los dos testimonios.
    #[test]
    fn cada_ex_pide_su_anillo() {
        let mut anillos: Vec<usize> = EXES.iter().map(|e| e.anillo).collect();
        anillos.sort();
        assert_eq!(anillos, vec![0, 1, 2]);
        for ex in &EXES {
            for t in &ex.testimonios {
                assert!(t.mentira < 4);
            }
        }
    }

    /// Presionar no cambia nada del caso: regresa al interrogatorio en la misma declaracion.
    #[test]
    fn presionar_regresa_al_interrogatorio() {
        let mut d = Divorcio::new(3);
        d.jugar_hasta("Interrogatorio");
        d.actualizar(&Entrada::default(), 0.5);
        d.actualizar(&Entrada { derecha: true, ..Default::default() }, 0.1);
        let cursor = d.cursor;
        d.actualizar(&Entrada::default(), 0.5);
        d.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.1);
        assert_eq!(d.fase, Fase::Presion(0));
        for _ in 0..2 {
            d.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.5);
        }
        assert_eq!(d.fase, Fase::Interrogatorio);
        assert_eq!(d.cursor, cursor);
        assert_eq!(d.paciencia, PACIENCIA);
    }

    #[test]
    fn los_nombres_se_ven_bien() {
        assert_eq!(titulo("SONIC"), "Sonic");
        assert_eq!(titulo("EL SINLUZ"), "el Sinluz");
    }
}
