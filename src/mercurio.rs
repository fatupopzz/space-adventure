//! Aterrizaje en Mercurio: el consultorio de tarot de Madame Mercurio (el
//! planeta mismo, de adivina, con turbante y las alitas del dios Mercurio).
//!
//! Miku escoge tres cartas del abanico: pasado, presente y futuro. Cada carta
//! vuela, da la vuelta y flota frente a la camara mientras Madame la lee. Pero
//! una vez por lectura Mercurio "entra en retrogrado": todo se rebobina como
//! un VHS (la carta regresa al abanico, el dialogo se lee al reves, la musica
//! suena al reves y el Sol de la ventana hace el rizo de la retrogradacion), y
//! la carta que se vuelva a sacar sale INVERTIDA. Las cartas, el orden y los
//! dialogos cambian en cada lectura.
//!
//! Para que valga la pena volver: Madame amanece cada vez de un humor (de buenas,
//! de malas y diciendo verdades que duelen, dormida, o dramatica de telenovela),
//! la baraja tiene 22 cartas que se van descubriendo, y al final las tres cartas
//! dan un VEREDICTO con consecuencias: con BENDICION regala una luna de Jupiter,
//! con SALAZON le cae a la nave la maldicion del retrogrado (al volar, los
//! controles van al reves un rato)... a menos que se pague una limpia con huevo.
//!
//! Todo esta en las coordenadas de la carpa: el centro de la mesa en el piso.

use crate::vida::{Tam, Tono, Vida};
use crate::entrada::Entrada;
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub struct Carta {
    pub nombre: &'static str,
    /// Lo que significa derecha.
    derecha: &'static str,
    /// Lo que significa invertida (al reves).
    invertida: &'static str,
    /// Cuanta suerte da: (derecha, invertida). Se suman las tres para el veredicto.
    pub suerte: (i32, i32),
}

const fn carta(nombre: &'static str, derecha: &'static str, invertida: &'static str, suerte: (i32, i32)) -> Carta {
    Carta { nombre, derecha, invertida, suerte }
}

/// La baraja de Madame Mercurio, en el orden del atlas `texturas/tarot.png`.
pub const BARAJA: [Carta; 22] = [
    carta(
        "EL WIFI",
        "Una conexión fuerte viene a su vida. Cinco rayitas. No la desperdicie viendo videos de gatos. ...O sí.",
        "Una rayita. Todo va a cargar hasta el 99% y se va a quedar ahí. Para siempre. Eso es Mercurio, mija.",
        (2, -2),
    ),
    carta(
        "LA CAMIONETA",
        "Un viaje largo, con música a todo volumen y un ayudante gritando: ¡hay lugar, hay lugar! No había lugar.",
        "La camioneta pasa llena y no para. Usted corre atrás. El ayudante le dice adiós con la mano.",
        (0, -1),
    ),
    carta(
        "EL EX",
        "Alguien del pasado le va a escribir \"hola, ¿cómo estás?\" a las 2 de la mañana. No conteste. Ni con sticker.",
        "USTED le va a escribir al ex. A las 2 de la mañana. Y va a decir que fue culpa de Mercurio. Y le van a creer.",
        (-1, -2),
    ),
    carta(
        "EL PARCIAL",
        "Un 61. Pasó raspado. Raspadísimo. Bendito sea el redondeo.",
        "Un 59. El ingeniero no redondea. Mercurio tampoco.",
        (1, -2),
    ),
    carta(
        "LA TORRE",
        "Una torre de señal cerca. Le van a entrar todos los audios de WhatsApp de una vez. Los de su tía también.",
        "Se cayó la torre. Sin señal. Sin mensajes. Por fin paz. ...Es lo más bonito que le he leído a alguien.",
        (-1, 2),
    ),
    carta(
        "EL SOL",
        "Días brillantes. 430 grados. Use bloqueador. En serio: el Sol está pegado a la ventana.",
        "El Sol sale por el oeste. No, esa es Venus. Me confundí de planeta. Pasa mucho en retrógrado.",
        (2, -1),
    ),
    carta(
        "LA LUNA",
        "Noches de columpio, canciones y fans con barritas de luz. Su carrera artística va para arriba.",
        "La Luna le enseña su lado oscuro. Ahí no hay nada. Puro polvo. Qué decepción.",
        (2, -1),
    ),
    carta(
        "EL CANGREJO",
        "Un cangrejo naranja le va a cuidar la memoria. Sin fugas, sin segfaults. Hay paz en su código.",
        "El borrow checker no la deja compilar. \"cannot borrow as mutable\". Llore. Es parte del proceso.",
        (2, -2),
    ),
    carta(
        "EL CHUCHO",
        "Un chucho la va a seguir hasta su casa. Ya es suyo. Así funciona. Póngale nombre.",
        "El chucho le ladra. Nada personal: también le ladra a la camioneta, al cartero y a la Luna.",
        (1, -1),
    ),
    carta(
        "LA TORTILLA",
        "Abundancia. Tortillas recién salidas del comal, calientitas, con sal. No hay mejor futuro.",
        "Tortillas de ayer, tiesas. Pero con eso se hacen chilaquilas. Siempre hay esperanza.",
        (2, 1),
    ),
    carta(
        "EL VOLCÁN",
        "Algo en su vida va a hacer erupción. Como el de Fuego: puntual, ruidoso y sale en las noticias.",
        "Sus emociones están como el Pacaya: tranquilas por fuera, lava por dentro. Tome agüita.",
        (0, -1),
    ),
    carta(
        "EL PUERRO",
        "Un puerro: el símbolo de su poder. Agítelo en el aire y todo saldrá bien. Bueno, casi todo.",
        "El puerro al revés significa... sopa. Significa sopa.",
        (1, 0),
    ),
    carta(
        "EL Z-BUFFER",
        "Todo en su vida está en orden: lo de adelante, adelante; lo de atrás, atrás. Qué profundo.",
        "Z-fighting. Dos cosas en su vida pelean por el mismo lugar y parpadean. Decídase.",
        (1, -1),
    ),
    carta(
        "LA BATERÍA",
        "Energía al cien. Va a poder con todo. Hasta con el proyecto de gráficas.",
        "Batería al 1% y sin cargador. ¿Sabe quién tiene cargador? El ex. Mercurio sabe lo que hace.",
        (2, -2),
    ),

    carta(
        "LA MUERTE",
        "No se asuste: en el tarot la Muerte es cambio. Se muere el semestre y nace otro. Igual de difícil, pero otro.",
        "Usted no se va a morir. Su planta sí. Y su racha de Duolingo. Las dos por lo mismo: abandono.",
        (0, -1),
    ),
    carta(
        "EL TRÁFICO",
        "La Roosevelt despejada a las 6 de la tarde. Llega temprano. Nadie le cree. Ni usted.",
        "Tres horas en el Periférico. Le salen canas en el carro. Y una muela del juicio. Y un juicio.",
        (1, -2),
    ),
    carta(
        "EL CAFÉ",
        "Un café de altura le va a salvar la mañana. Y la tarde. Y el parcial. El café todo lo puede.",
        "Café frío, con nata, olvidado en el escritorio desde el martes. Se lo va a tomar igual. La conozco.",
        (1, -1),
    ),
    carta(
        "LA LLUVIA",
        "Va a llover a las 3 en punto, como siempre. Pero usted SÍ trajo paraguas. Milagro. Anótelo.",
        "Sale sin paraguas. Pasa un carro por el charco. Usted ya sabe lo que sigue. Yo también lo vi.",
        (1, -2),
    ),
    carta(
        "EL QUETZAL",
        "Libertad: el quetzal no vive enjaulado y usted tampoco en ese grupo de WhatsApp. Sálgase. Hoy.",
        "El quetzal no le alcanza: le quedan Q3.50 para llegar a fin de mes. Tortillas con sal, mija.",
        (2, -2),
    ),
    carta(
        "EL COMPILADOR",
        "Compila a la primera. Desconfíe. Eso nunca pasa. Algo está tramando.",
        "847 warnings y un error. El error es suyo. Los warnings también. El compilador lo sabe.",
        (0, -2),
    ),
    carta(
        "EL CHISME",
        "Se va a enterar de un chisme buenísimo. Con pruebas. Con capturas. Con audio de la tía.",
        "El chisme es sobre usted. Y ya lo sabe todo el salón. Y la tía. Y la tía de la tía.",
        (1, -2),
    ),
    carta(
        "EL PISTO",
        "Viene pisto. Poquito, pero viene. Le alcanza para unas shucas y un fresco de súchiles.",
        "Viene pisto... pero para el banco. A cobrarle. La tarjeta dice que no. La tarjeta siempre dice que no.",
        (2, -2),
    ),
];

/// Los nombres de los frentes en `tarot.obj` (para ocultar los que no son). Con dos digitos: `ocultar`
/// compara por prefijo, y "frente_1" tambien taparia a "frente_11".
pub const FRENTES: [&str; 22] = [
    "frente_00", "frente_01", "frente_02", "frente_03", "frente_04", "frente_05", "frente_06", "frente_07", "frente_08", "frente_09", "frente_10",
    "frente_11", "frente_12", "frente_13", "frente_14", "frente_15", "frente_16", "frente_17", "frente_18", "frente_19", "frente_20", "frente_21",
];

const POSICIONES: [&str; 3] = ["PASADO", "PRESENTE", "FUTURO"];
/// Cuantas cartas hay en el abanico.
const ABANICO: usize = 7;

const GLADAS: &str = ui::NOMBRE_IA;
pub const MADAME: &str = "MADAME MERCURIO";
const MIKU: &str = "MIKU";

/// Como amanecio Madame (cambia en cada lectura): lo que dice, como se mueve y que tan mal le va a uno.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Humor {
    DeBuenas,
    /// Dice verdades que duelen. Y salen mas cartas invertidas.
    DeMalas,
    /// Bosteza, ronca y se le olvida que carta era.
    Dormida,
    /// Todo es una telenovela: grita, se desmaya, caen rayos.
    Dramatica,
}

const HUMORES: [Humor; 4] = [Humor::DeBuenas, Humor::DeMalas, Humor::Dormida, Humor::Dramatica];

const SALUDOS_BUENAS: [&str; 3] = [
    "Pase adelante, mija, siéntese. Soy Madame Mercurio. Leo las cartas, la mano y los mensajes que le dejaron en visto.",
    "¡Bienvenida! Soy Madame Mercurio, el planeta más cerca del Sol. Por eso lo veo todo clarito. Con lentes oscuros.",
    "Ah, la estaba esperando. Bueno, no. Pero así se dice. Soy Madame Mercurio y hoy le leo la suerte.",
];
const SALUDOS_MALAS: [&str; 3] = [
    "Ay, usted otra vez. Siéntese. Hoy amanecí de malas y las cartas también. No diga que no le avisé.",
    "No me hable bonito que hoy no estoy para nadie. Estamos a 430 grados y yo sin aire acondicionado.",
    "Hoy no leo la suerte: leo la verdad. Y la verdad, mija, a veces cae como agua fría. Siéntese.",
];
const SALUDOS_DORMIDA: [&str; 3] = [
    "Mmm... ¿ah? Sí, sí, estaba meditando. Con los ojos cerrados. Y roncando. Es una técnica milenaria.",
    "(bostezo cósmico) ...Perdón. Leí cartas toda la noche. A mí misma. Me salió EL CAFÉ. Y no había café.",
    "Un día en Mercurio dura 176 días de la Tierra. Y yo no he dormido en todo el día. Siéntese, rapidito.",
];
const SALUDOS_DRAMA: [&str; 3] = [
    "¡AY, DIOS MÍO! ¡Lo sabía! ¡Las cartas me dijeron que vendría alguien hoy! ...Siempre viene alguien, pero igual.",
    "¡Siéntese, por favor, el destino nos espera! ¡Música! ...No hay música. ¡Imagínesela! ¡Con violines!",
    "¡Usted! ¡Sí, USTED! ¡Su aura! ¡Su aura está...! ...Normal. Está normal. Pero la quiero leer igual.",
];
const INSTRUCCIONES: [&str; 3] = [
    "Escoja tres cartas: pasado, presente y futuro. No se preocupe, Mercurio está directo. Por ahora.",
    "Tres cartas: pasado, presente y futuro. Si algo sale mal, no es mi culpa. Es astronómico.",
    "Concéntrese en una pregunta. ...No, en esa no. En otra. Ya. Ahora escoja tres cartas.",
];
/// Lo que agrega Madame despues de cada carta, segun como amanecio (la de buenas no agrega nada).
const PULLAS: [&str; 10] = [
    "Y le digo más: esa ropa no le queda. Las cartas no dicen nada de eso. Eso lo digo yo.",
    "Su crush la dejó en visto. Dos veces. Con doble palomita azul. Lo vi en la bola de cristal.",
    "Va a reprobar algo este semestre. No sé qué. Usted sí sabe qué.",
    "Su pasaporte espacial está casi vacío. Hasta Plutón tiene más sellos que usted. Y no es planeta.",
    "Su nave huele a shuco. Desde aquí se siente. Y estamos en Mercurio.",
    "Tiene cara de que entrega las tareas a las 11:59. Y de que le salen a las 12:01.",
    "Las cartas dicen que usted canta en la ducha. Y que los vecinos ya se quejaron. Por escrito.",
    "Ni el retrógrado le trae tanta mala suerte como usted misma. Se lo digo con cariño. Poquito.",
    "Usted es de las que dicen \"ya voy\" y ni se han bañado.",
    "Su contraseña es su fecha de nacimiento. Cámbiela. Las cartas no se lo van a decir dos veces.",
];
const DRAMAS: [&str; 6] = [
    "¡NOOOO! ¡Esta carta! ¡Esta carta es...! ...Normal. Es normal. Perdón, me emocioné.",
    "¡Siento una presencia! ¡Algo se acerca! ...Ah, no, es el calor. 430 grados. Sigamos.",
    "¡El destino! ¡La pasión! ¡La traición! ...Eso sale en la telenovela de las 8. Pero en su carta también, poquito.",
    "¡Ay, me desmayo! ¡Me desmayo! ...No, ya se me pasó. Sigamos.",
    "¡Que alguien llame a mi madre! ...Es Venus. No contesta: está en un juicio de divorcio.",
    "¡Esto es más fuerte que yo! ¡Más fuerte que la gravedad del Sol! ¡Más fuerte que el WiFi de la U!",
];
const RONQUIDOS: [&str; 4] = [
    "...Zzz... ¿Ah? Sí, sí, muy profundo todo. Muy... zzz.",
    "Esta carta significa... (bostezo)... que usted necesita dormir. Como yo. Ahorita.",
    "Mmm... perdón, ¿qué carta era? Ah, sí. Esa. Muy bonita. Siguiente.",
    "Zzz... cinco minutitos más, mami... ¡AY! Sí, la carta. La carta. Ya la leí. Creo.",
];
const EXCUSAS: [&str; 3] = [
    "¡Ay no! ¡Entré en retrógrado! Todo va para atrás. Mi café se está des-sirviendo. Escoja otra vez.",
    "¡Retrógrado! Por eso no le contestan, por eso se le cae el WiFi, por eso llegó tarde. Todo es por mí. Perdón. Otra carta.",
    "Ya me dio el retrógrado. Ahorita todo sale al revés. Escoja otra carta. Y no le mande mensaje a nadie.",
];
const DATO_GLADAS: &str = "Dato: Mercurio nunca va para atrás. Solo lo parece desde la Tierra, porque la Tierra lo rebasa. Como cuando usted rebasa una camioneta y parece que retrocede.";

/// El veredicto de las tres cartas (su suerte sumada).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Destino {
    Bendicion,
    BuenaSuerte,
    NiFuNiFa,
    Salazon,
    SalazonTotal,
}

impl Destino {
    pub fn de_puntos(puntos: i32) -> Self {
        match puntos {
            p if p >= 4 => Destino::Bendicion,
            2..=3 => Destino::BuenaSuerte,
            -1..=1 => Destino::NiFuNiFa,
            -3..=-2 => Destino::Salazon,
            _ => Destino::SalazonTotal,
        }
    }

    /// El sello grande que cae en la pantalla.
    pub fn sello(self) -> &'static str {
        match self {
            Destino::Bendicion => "¡BENDICIÓN!",
            Destino::BuenaSuerte => "BUENA SUERTE",
            Destino::NiFuNiFa => "NI FU NI FA",
            Destino::Salazon => "SALAZÓN",
            Destino::SalazonTotal => "¡SALAZÓN TOTAL!",
        }
    }

    pub fn es_salazon(self) -> bool {
        matches!(self, Destino::Salazon | Destino::SalazonTotal)
    }

    /// Lo que dice Madame del veredicto (y lo que pasa).
    fn texto(self) -> &'static str {
        match self {
            Destino::Bendicion => "¡BENDICIÓN! Las tres cartas le sonríen. En 4,500 millones de años no había visto algo así. Tome: le regalo una luna de Júpiter para el camino. Ya está en su mochila.",
            Destino::BuenaSuerte => "Buena suerte, mija. No es para tanto, pero alcanza. Le dejo diez notas de propina del universo. No se las gaste en estelas. ...Bueno, sí.",
            Destino::NiFuNiFa => "Ni fu ni fa. Su destino es... tibio. Como el café que dejó en el escritorio. Siga así y nadie se va a acordar de nada.",
            Destino::Salazon => "Uy. Uy, uy, uy. Usted trae una SALAZÓN de aquellas. Le va a caer la maldición del retrógrado: al volar, los controles van al revés un ratito.",
            Destino::SalazonTotal => "¡SALAZÓN TOTAL! Nunca, NUNCA había visto tanta mala suerte junta. Maldición del retrógrado doble: al volar, todo al revés. Le recomiendo una limpia. En serio.",
        }
    }

    /// Lo que opina Miku del veredicto.
    fn reaccion(self) -> &'static str {
        match self {
            Destino::Bendicion => "¿Una luna gratis? ¡Vuelvo mañana! Y pasado. Y el otro. Madame, usted es mi nueva mejor amiga.",
            Destino::BuenaSuerte => "Me gustó la parte de mi futuro. Las otras dos... ¿se pueden devolver?",
            Destino::NiFuNiFa => "¿Tibio? ¿Mi destino es TIBIO? Prefiero que me hubiera salido la Muerte. Por lo menos es emocionante.",
            Destino::Salazon => "¿Salazón? ¿Yo? Si yo soy un amor. ...Bueno, a veces. Los martes.",
            Destino::SalazonTotal => "Quiero mi dinero de vuelta. ...No pagué nada. Quiero mi dinero de vuelta igual.",
        }
    }

    /// Cuantos segundos dura la maldicion del retrogrado al volar.
    fn maldicion(self) -> f32 {
        match self {
            Destino::Salazon => 40.0,
            Destino::SalazonTotal => 80.0,
            _ => 0.0,
        }
    }
}

const COBROS: [&str; 3] = [
    "Son cincuenta quetzales. Ah, pero estoy en retrógrado... entonces yo le pago a usted. Tome: cinco quetzales y un chicle.",
    "La consulta es gratis. Lo que se cobra es salir de la carpa. Bromita. ¿O no? Con el retrógrado, quién sabe.",
    "Acepto efectivo, tarjeta y transferencia. En retrógrado la transferencia llega... ayer.",
];
/// Lo que cuesta la limpia (en notas).
pub const PRECIO_LIMPIA: u32 = 15;
const OFRECE_LIMPIA: &str = "Con esa salazón no la dejo ir así. Le hago una limpia: huevo, ruda, una vela y un poquito de WiFi. Son 15 notas. ¿Se la hago?";
const LIMPIA_HECHA: &str = "Quieta, quieta... le paso el huevo por la cabeza... por los hombros... ¡Mire el huevo! ¡Salió con la cara de Mercurio retrógrado! Ya está. Sin salazón. Vaya con Dios.";
const LIMPIA_DE_POBRE: &str = "No le alcanzan las notas, mija. Pero le hago una limpia de pobre: con tortilla. ...Ya. Le quité la mitad de la salazón. La otra mitad, ahí se la lleva.";
const SIN_LIMPIA: &str = "Allá usted. La salazón se le pega a la nave. Cuando todo le salga al revés, no diga que no le avisé.";
const DESPEDIDA: &str = "Vuelva cuando quiera. Si no me encuentra, ando dándole la vuelta al Sol. En 88 días regreso.";

/// Donde esta cada cosa en la carpa.
pub const MESA_Y: f32 = 0.55;
pub const BOLA: Vec3 = Vec3::new(0.0, MESA_Y + 0.225, 0.42);
pub const VELAS: [Vec3; 5] =
    [Vec3::new(-1.9, 1.75, 1.6), Vec3::new(1.9, 1.75, 1.6), Vec3::new(-0.62, MESA_Y + 0.27, 0.55), Vec3::new(0.62, MESA_Y + 0.27, 0.55), Vec3::new(-0.75, MESA_Y + 0.27, 0.05)];
const MADAME_EN: Vec3 = Vec3::new(0.0, 0.28, 1.2);
const MIKU_EN: Vec3 = Vec3::new(0.0, 0.42, -1.2);
/// Donde queda el Sol, afuera de la ventana.
const SOL: Vec3 = Vec3::new(0.0, 6.0, 80.0);
const ESCALA_CARTA: f32 = 0.62;

/// Segundos de cada animacion.
const BARAJAR: f32 = 2.2;
const REVELAR: f32 = 1.3;
const ASENTAR: f32 = 0.5;
const REBOBINAR: f32 = 2.8;
/// Cuanto habla Madame antes de que le de el retrogrado.
const INTERRUMPIR: f32 = 2.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fase {
    Llegada,
    Saludo(usize),
    Barajar,
    Elegir,
    /// La carta vuela del abanico y queda flotando boca arriba.
    Revelar,
    /// Madame empieza a leer... y le da el retrogrado.
    Interrumpida,
    /// 0: se rebobina todo. 1: la excusa. 2: GLADAS explica.
    Retrogrado(usize),
    Lectura,
    /// Lo que agrega Madame segun como amanecio: una pulla, un drama o un ronquido.
    Comentario,
    /// La carta leida baja a su lugar en la mesa.
    Asentar,
    /// 0: el resumen. 1: el veredicto (cae el sello). 2: Miku opina.
    Profecia(usize),
    Cobro,
    /// Con salazon: 0 Madame ofrece la limpia (se escoge si o no), 1 la limpia, 2 no quiso.
    Limpia(usize),
    Libre,
}

pub enum AccionMercurio {
    Nada,
    Despegar,
}

/// Una carta sacada del abanico.
#[derive(Clone, Copy, Debug)]
struct Sacada {
    /// En que lugar del abanico estaba.
    lugar: usize,
    invertida: bool,
}

pub struct Mercurio {
    fase: Fase,
    t: f32,
    pub total: f32,
    semilla: u32,
    /// Las cartas del abanico (indices en `BARAJA`), boca abajo.
    abanico: [usize; ABANICO],
    sacadas: Vec<Sacada>,
    cursor: usize,
    /// Despues de revelar que carta (0, 1 o 2) le da el retrogrado, y si ya le dio.
    retro_en: usize,
    retro_hecho: bool,
    /// La carta que se rebobino (para dibujarla volviendo) mientras dura el rebobinado.
    rebobinada: Option<Sacada>,
    saludo: usize,
    instruccion: usize,
    excusa: usize,
    cobro: usize,
    /// Como amanecio Madame hoy, y que comentario le toca a cada carta (sin repetir).
    pub humor: Humor,
    comentarios: [usize; 3],
    /// Un relampago (de 1 a 0): cuando sale una carta mala, en los dramas y con la salazon.
    pub relampago: f32,
    /// Lo que paso al final, para el juego: la luna que se regalo, los segundos de maldicion
    /// (al volar) y las notas que hay que cobrar por la limpia.
    pub regalo: Option<usize>,
    pub propina: u32,
    pub maldicion: f32,
    pub cobrar: u32,
    pub limpia_hecha: bool,
    /// Si en la oferta de la limpia esta escogido el "si".
    limpia_si: bool,
    /// Lo que el juego le cuenta: las notas que se tienen y las cartas ya descubiertas (bits).
    pub notas: u32,
    pub descubiertas: u32,
    /// Si la carta que se esta leyendo es la primera vez que sale (para el letrero de "nueva").
    nueva: bool,
    /// Los letreros gigantes, el confeti y las sacudidas (ver vida.rs).
    pub vida: Vida,
    pub eventos: Vec<Efecto>,
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// "EL WIFI" -> "el wifi".
fn en_minusculas(s: &str) -> String {
    s.to_lowercase()
}

impl Mercurio {
    pub fn new(semilla: u32) -> Self {
        let mut m = Mercurio {
            fase: Fase::Llegada,
            t: 0.0,
            total: 0.0,
            semilla: semilla | 1,
            abanico: [0; ABANICO],
            sacadas: Vec::new(),
            cursor: ABANICO / 2,
            retro_en: 1,
            retro_hecho: false,
            rebobinada: None,
            saludo: 0,
            instruccion: 0,
            excusa: 0,
            cobro: 0,
            humor: Humor::DeBuenas,
            comentarios: [0; 3],
            relampago: 0.0,
            regalo: None,
            propina: 0,
            maldicion: 0.0,
            cobrar: 0,
            limpia_hecha: false,
            limpia_si: true,
            notas: 0,
            descubiertas: 0,
            nueva: false,
            vida: Vida::sin_cartel(),
            eventos: Vec::new(),
        };
        m.barajar();
        m
    }

    fn azar(&mut self, n: usize) -> usize {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((self.semilla >> 8) as usize) % n
    }

    /// Una lectura nueva: baraja, escoge siete cartas, cuando le da el retrogrado y que dice cada quien.
    fn barajar(&mut self) {
        let mut mazo: Vec<usize> = (0..BARAJA.len()).collect();
        for i in (1..mazo.len()).rev() {
            let j = self.azar(i + 1);
            mazo.swap(i, j);
        }
        self.abanico.copy_from_slice(&mazo[..ABANICO]);
        self.sacadas.clear();
        self.cursor = ABANICO / 2;
        self.retro_en = 1 + self.azar(2);
        self.retro_hecho = false;
        self.rebobinada = None;
        self.humor = HUMORES[self.azar(HUMORES.len())];
        self.saludo = self.azar(3);
        self.instruccion = self.azar(INSTRUCCIONES.len());
        self.excusa = self.azar(EXCUSAS.len());
        self.cobro = self.azar(COBROS.len());
        // Tres comentarios distintos (los primeros tres de la lista revuelta).
        let mut lista: Vec<usize> = (0..self.lista_de_comentarios().len()).collect();
        for i in 0..lista.len().min(3) {
            let j = i + self.azar(lista.len() - i);
            lista.swap(i, j);
        }
        for (k, c) in self.comentarios.iter_mut().enumerate() {
            *c = lista.get(k).copied().unwrap_or(0);
        }
        self.limpia_si = true;
    }

    /// Los comentarios del humor de hoy (la de buenas no comenta).
    fn lista_de_comentarios(&self) -> &'static [&'static str] {
        match self.humor {
            Humor::DeBuenas => &[],
            Humor::DeMalas => &PULLAS,
            Humor::Dormida => &RONQUIDOS,
            Humor::Dramatica => &DRAMAS,
        }
    }

    /// Si Madame comenta despues de la carta `i` (la dormida no siempre se acuerda).
    fn comenta(&self, i: usize) -> bool {
        match self.humor {
            Humor::DeBuenas => false,
            Humor::Dormida => i != 1,
            _ => true,
        }
    }

    /// La suerte sumada de las cartas sacadas.
    pub fn puntos(&self) -> i32 {
        self.sacadas
            .iter()
            .map(|s| {
                let c = &BARAJA[self.abanico[s.lugar]];
                if s.invertida { c.suerte.1 } else { c.suerte.0 }
            })
            .sum()
    }

    /// El veredicto, cuando ya salieron las tres.
    pub fn destino(&self) -> Option<Destino> {
        (self.sacadas.len() == POSICIONES.len() && self.terminada()).then(|| Destino::de_puntos(self.puntos()))
    }

    /// Las cartas que ya se vieron boca arriba en esta visita (bits de `BARAJA`).
    pub fn vistas(&self) -> u32 {
        let leidas = self.sacadas.len() - matches!(self.fase, Fase::Revelar) as usize;
        self.sacadas[..leidas].iter().fold(0, |b, s| b | 1 << self.abanico[s.lugar])
    }

    /// El veredicto ya cae: sus consecuencias (una sola vez).
    fn sentenciar(&mut self) {
        let Some(d) = Some(Destino::de_puntos(self.puntos())) else { return };
        match d {
            Destino::Bendicion => {
                self.regalo = Some(self.azar(4));
                self.vida.exito("¡BENDICIÓN!", "Las cartas están de su lado", Tono::Oro);
                self.eventos.push(Efecto::Perfecto);
            }
            Destino::BuenaSuerte => {
                self.propina = 10;
                self.vida.grito_con("¡BUENA SUERTE!", "Hoy sí le sale algo", Tono::Verde, Tam::Grande);
                self.vida.golpe(0.3, Tono::Verde);
                self.vida.confeti(35, Tono::Verde);
                self.eventos.push(Efecto::Acierto);
            }
            Destino::NiFuNiFa => {
                self.vida.grito_con("NI FU NI FA", "Ni bien ni mal. Como siempre", Tono::Lila, Tam::Grande);
                self.eventos.push(Efecto::Carta);
            }
            Destino::Salazon | Destino::SalazonTotal => {
                self.maldicion = d.maldicion();
                self.relampago = 1.0;
                self.vida.desastre(if d == Destino::SalazonTotal { "¡SALAZÓN TOTAL!" } else { "¡SALAZÓN!" }, "Mejor no salga de su casa");
                self.eventos.push(Efecto::Alarma);
            }
        }
        self.eventos.push(Efecto::Sello);
    }

    fn cambiar(&mut self, fase: Fase) {
        self.fase = fase;
        self.t = 0.0;
    }

    fn libre(&self, lugar: usize) -> bool {
        !self.sacadas.iter().any(|s| s.lugar == lugar)
    }

    /// El lugar libre mas cercano a `desde`, buscando hacia `paso`.
    fn mover_cursor(&mut self, paso: i32) {
        let mut k = self.cursor as i32;
        for _ in 0..ABANICO {
            k += paso;
            if k < 0 || k >= ABANICO as i32 {
                return;
            }
            if self.libre(k as usize) {
                self.cursor = k as usize;
                return;
            }
        }
    }

    fn cursor_valido(&mut self) {
        if !self.libre(self.cursor) {
            if let Some(k) = (0..ABANICO).min_by_key(|&k| if self.libre(k) { (k as i32 - self.cursor as i32).abs() } else { 99 }) {
                self.cursor = k;
            }
        }
    }

    /// Si ya terminaron las tres cartas.
    pub fn terminada(&self) -> bool {
        matches!(self.fase, Fase::Profecia(_) | Fase::Cobro | Fase::Limpia(_) | Fase::Libre)
    }

    /// Cuantas de las cartas sacadas salieron invertidas.
    pub fn invertidas(&self) -> usize {
        self.sacadas.iter().filter(|s| s.invertida).count()
    }

    /// Si esta rebobinando (para el efecto VHS y la musica al reves): de 0 a 1.
    pub fn rebobinando(&self) -> f32 {
        match self.fase {
            Fase::Retrogrado(0) => suave(self.t / 0.3) * (1.0 - suave((self.t - REBOBINAR + 0.4) / 0.4)),
            _ => 0.0,
        }
    }

    pub fn en_retrogrado(&self) -> bool {
        matches!(self.fase, Fase::Retrogrado(_))
    }

    /// Cuantas cartas se leyeron (para el mensaje al despegar).
    pub fn leidas(&self) -> usize {
        self.sacadas.len() - matches!(self.fase, Fase::Revelar | Fase::Interrumpida | Fase::Lectura | Fase::Comentario) as usize
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionMercurio {
        self.t += dt;
        self.total += dt;
        if e.aterrizar {
            return AccionMercurio::Despegar;
        }
        self.relampago = (self.relampago - dt * 2.2).max(0.0);
        let largo = self.hablante().map_or(0, |h| h.1.chars().count()) as f32;
        let siguiente = (e.aceptar && self.t > 0.4) || self.t > 2.5 + largo / 22.0;
        match self.fase {
            Fase::Llegada if siguiente || self.t > 6.0 => self.cambiar(Fase::Saludo(0)),
            Fase::Saludo(0) if siguiente => self.cambiar(Fase::Saludo(1)),
            Fase::Saludo(_) if siguiente => {
                self.eventos.push(Efecto::Carta);
                self.cambiar(Fase::Barajar);
            }
            Fase::Barajar => {
                // El ruidito de las cartas al barajar.
                if ((self.t - dt) * 7.0) as i32 != (self.t * 7.0) as i32 && self.t < BARAJAR - 0.3 {
                    self.eventos.push(Efecto::Carta);
                }
                if self.t > BARAJAR {
                    self.cambiar(Fase::Elegir);
                }
            }
            Fase::Elegir => {
                if e.izquierda {
                    self.mover_cursor(-1);
                    self.eventos.push(Efecto::Carta);
                } else if e.derecha {
                    self.mover_cursor(1);
                    self.eventos.push(Efecto::Carta);
                } else if e.aceptar && self.t > 0.3 {
                    self.sacar();
                }
            }
            Fase::Revelar if self.t > REVELAR => {
                let i = self.sacadas.len() - 1;
                if i == self.retro_en && !self.retro_hecho {
                    self.cambiar(Fase::Interrumpida);
                } else {
                    self.empezar_lectura();
                }
            }
            Fase::Interrumpida if self.t > INTERRUMPIR => {
                self.vida.desastre("¡RETRÓGRADO!", "Mercurio se lleva su carta");
                self.eventos.push(Efecto::Alarma);
                self.eventos.push(Efecto::Rebobinar);
                self.rebobinada = self.sacadas.pop();
                self.retro_hecho = true;
                self.cambiar(Fase::Retrogrado(0));
            }
            Fase::Retrogrado(0) if self.t > REBOBINAR => {
                self.rebobinada = None;
                self.cursor_valido();
                self.cambiar(Fase::Retrogrado(1));
            }
            Fase::Retrogrado(1) if siguiente => self.cambiar(Fase::Retrogrado(2)),
            Fase::Retrogrado(2) if siguiente => self.cambiar(Fase::Elegir),
            Fase::Lectura if siguiente => {
                if self.comenta(self.sacadas.len() - 1) {
                    if self.humor == Humor::Dramatica {
                        self.relampago = 0.8;
                        self.vida.grito("¡UUUUH!", Tono::Lila, Tam::Grande);
                        self.vida.golpe(0.45, Tono::Lila);
                        self.eventos.push(Efecto::Protesto);
                    }
                    self.cambiar(Fase::Comentario);
                } else {
                    self.eventos.push(Efecto::Carta);
                    self.cambiar(Fase::Asentar);
                }
            }
            Fase::Comentario if siguiente => {
                self.eventos.push(Efecto::Carta);
                self.cambiar(Fase::Asentar);
            }
            Fase::Asentar if self.t > ASENTAR => {
                if self.sacadas.len() == POSICIONES.len() {
                    self.eventos.push(Efecto::Perfecto);
                    self.cambiar(Fase::Profecia(0));
                } else {
                    self.cursor_valido();
                    self.cambiar(Fase::Elegir);
                }
            }
            Fase::Profecia(i) if siguiente => {
                if i == 0 {
                    self.sentenciar();
                }
                self.cambiar(if i < 2 { Fase::Profecia(i + 1) } else { Fase::Cobro });
            }
            Fase::Cobro if siguiente => {
                let salado = Destino::de_puntos(self.puntos()).es_salazon();
                self.cambiar(if salado { Fase::Limpia(0) } else { Fase::Libre });
            }
            Fase::Limpia(0) => {
                if e.izquierda || e.derecha || e.arriba || e.abajo {
                    self.limpia_si = !self.limpia_si;
                    self.eventos.push(Efecto::Carta);
                } else if e.aceptar && self.t > 0.5 {
                    if !self.limpia_si {
                        self.eventos.push(Efecto::Fallo);
                        self.cambiar(Fase::Limpia(2));
                    } else if self.notas >= PRECIO_LIMPIA {
                        self.cobrar = PRECIO_LIMPIA;
                        self.maldicion = 0.0;
                        self.limpia_hecha = true;
                        self.eventos.push(Efecto::Perfecto);
                        self.cambiar(Fase::Limpia(1));
                    } else {
                        self.maldicion *= 0.5;
                        self.limpia_hecha = true;
                        self.eventos.push(Efecto::Acierto);
                        self.cambiar(Fase::Limpia(1));
                    }
                }
            }
            Fase::Limpia(_) if siguiente => self.cambiar(Fase::Libre),
            Fase::Libre if e.aceptar && self.t > 0.6 => {
                // Otra lectura: cartas, orden, humor y dialogos nuevos.
                self.barajar();
                self.eventos.push(Efecto::Carta);
                self.cambiar(Fase::Barajar);
            }
            _ => {}
        }
        AccionMercurio::Nada
    }

    /// Juega solo (aceptando los dialogos y sacando cartas) hasta llegar a la fase que empieza con
    /// `objetivo` (como `Lectura`, `Retrogrado(0)` o `Libre`): para las capturas. Con `#n` (como
    /// `Lectura#3`), hasta la n-esima vez que entra a esa fase.
    pub fn jugar_hasta(&mut self, objetivo: &str) {
        let aceptar = Entrada { aceptar: true, ..Default::default() };
        let (objetivo, veces) = objetivo.split_once('#').map_or((objetivo, 1), |(o, n)| (o, n.parse().unwrap_or(1)));
        let mut vistas = 0;
        let mut antes = false;
        for _ in 0..3000 {
            let ahora = format!("{:?}", self.fase).starts_with(objetivo);
            if ahora && !antes {
                vistas += 1;
                if vistas == veces {
                    return;
                }
            }
            antes = ahora;
            let e = if (self.hablante().is_some() || self.fase == Fase::Elegir) && self.t > 0.5 { &aceptar } else { &Entrada::default() };
            self.actualizar(e, 0.05);
        }
    }

    /// Saca la carta del cursor. Despues del retrogrado sale invertida, siempre.
    fn sacar(&mut self) {
        if !self.libre(self.cursor) {
            return;
        }
        // De malas salen mas invertidas (la mitad); dramatica, un tercio; si no, una de cada cuatro.
        let cada = match self.humor {
            Humor::DeMalas => 2,
            Humor::Dramatica => 3,
            _ => 4,
        };
        let invertida = (self.retro_hecho && self.sacadas.len() == self.retro_en) || self.azar(cada) == 0;
        self.sacadas.push(Sacada { lugar: self.cursor, invertida });
        self.eventos.push(Efecto::Carta);
        self.eventos.push(Efecto::Fuego);
        self.cambiar(Fase::Revelar);
    }

    /// La carta ya quedo de frente: a leerla. Si es mala, cae un rayo (y Madame tiembla).
    fn empezar_lectura(&mut self) {
        if let Some((_, c, invertida)) = self.actual() {
            let k = BARAJA.iter().position(|b| std::ptr::eq(b, c)).unwrap_or(0);
            self.nueva = self.descubiertas & (1 << k) == 0;
            let valor = if invertida { c.suerte.1 } else { c.suerte.0 };
            if valor < 0 || self.humor == Humor::Dramatica {
                self.relampago = if valor < 0 { 1.0 } else { 0.6 };
                self.eventos.push(Efecto::Alarma);
            }
        }
        self.cambiar(Fase::Lectura);
    }

    /// La carta que se esta leyendo (la ultima sacada).
    fn actual(&self) -> Option<(usize, &'static Carta, bool)> {
        let i = self.sacadas.len().checked_sub(1)?;
        let s = self.sacadas[i];
        Some((i, &BARAJA[self.abanico[s.lugar]], s.invertida))
    }

    pub fn hablante(&self) -> Option<(&'static str, String, Voz)> {
        let madame = |s: String| Some((MADAME, s, Voz::Vidente));
        match self.fase {
            Fase::Llegada => Some((GLADAS, "Mercurio. 430 grados de día, 180 bajo cero de noche. Y una carpa de tarot. El seguro de la nave no cubre maldiciones.".into(), Voz::Ia)),
            Fase::Saludo(0) => madame(
                match self.humor {
                    Humor::DeBuenas => SALUDOS_BUENAS[self.saludo],
                    Humor::DeMalas => SALUDOS_MALAS[self.saludo],
                    Humor::Dormida => SALUDOS_DORMIDA[self.saludo],
                    Humor::Dramatica => SALUDOS_DRAMA[self.saludo],
                }
                .into(),
            ),
            Fase::Saludo(_) => madame(INSTRUCCIONES[self.instruccion].into()),
            Fase::Barajar | Fase::Elegir | Fase::Revelar | Fase::Asentar => None,
            Fase::Interrumpida => {
                let (_, c, _) = self.actual()?;
                madame(format!("Mmm... {}. Esta carta significa que su...", c.nombre))
            }
            // El rebobinado: lo ultimo que dijo, al reves.
            Fase::Retrogrado(0) => {
                let c = &BARAJA[self.abanico[self.rebobinada?.lugar]];
                madame(format!("Mmm... {}. Esta carta significa que su...", c.nombre).chars().rev().collect())
            }
            Fase::Retrogrado(1) => madame(EXCUSAS[self.excusa].into()),
            Fase::Retrogrado(_) => Some((GLADAS, DATO_GLADAS.into(), Voz::Ia)),
            Fase::Lectura => {
                let (i, c, invertida) = self.actual()?;
                let inicio = ["En su pasado", "En su presente", "Y en su futuro..."][i];
                let texto = if invertida { c.invertida } else { c.derecha };
                let nota = if invertida && self.retro_hecho && i == self.retro_en {
                    " Invertida. Obvio. Gracias, retrógrado."
                } else if invertida {
                    " Invertida."
                } else {
                    ""
                };
                madame(format!("{inicio}: {}.{nota} {texto}", en_minusculas(c.nombre)))
            }
            Fase::Profecia(0) => {
                let n: Vec<String> = self.sacadas.iter().map(|s| en_minusculas(BARAJA[self.abanico[s.lugar]].nombre)).collect();
                madame(format!("Resumiendo: {} en su pasado, {} en su presente... y {} en su futuro.", n[0], n[1], n[2]))
            }
            Fase::Comentario => {
                let lista = self.lista_de_comentarios();
                let i = self.sacadas.len().saturating_sub(1).min(2);
                madame(lista.get(self.comentarios[i] % lista.len().max(1))?.to_string())
            }
            Fase::Profecia(1) => madame(Destino::de_puntos(self.puntos()).texto().into()),
            Fase::Profecia(_) => Some((MIKU, Destino::de_puntos(self.puntos()).reaccion().into(), Voz::Miku)),
            Fase::Cobro => madame(COBROS[self.cobro].into()),
            Fase::Limpia(0) => madame(OFRECE_LIMPIA.into()),
            Fase::Limpia(1) => madame(if self.cobrar > 0 { LIMPIA_HECHA } else { LIMPIA_DE_POBRE }.into()),
            Fase::Limpia(_) => madame(SIN_LIMPIA.into()),
            Fase::Libre => madame(DESPEDIDA.into()),
        }
    }

    pub fn expresion_madame(&self) -> &'static str {
        // Como amanecio se le nota en la cara cuando no esta pasando nada especial.
        let de_humor = match self.humor {
            Humor::DeMalas => Some("enojado"),
            Humor::Dormida if (self.total * 0.4).sin() > 0.2 => Some("dormida"),
            _ => None,
        };
        match self.fase {
            Fase::Comentario => return de_humor.unwrap_or(if self.humor == Humor::Dramatica { "asustada" } else { "mistica" }),
            Fase::Profecia(1) => {
                return match Destino::de_puntos(self.puntos()) {
                    Destino::Bendicion | Destino::BuenaSuerte => "enamorada",
                    Destino::NiFuNiFa => "serio",
                    _ => "asustada",
                }
            }
            Fase::Limpia(0) | Fase::Limpia(1) => return "mistica",
            Fase::Limpia(_) => return "enojado",
            Fase::Saludo(_) | Fase::Elegir | Fase::Barajar => {
                if let Some(h) = de_humor {
                    return h;
                }
            }
            _ => {}
        }
        match self.fase {
            Fase::Llegada | Fase::Saludo(0) => "guino",
            Fase::Saludo(_) | Fase::Barajar | Fase::Elegir | Fase::Revelar | Fase::Interrumpida | Fase::Profecia(0) => "mistica",
            Fase::Retrogrado(0) => "espiral",
            Fase::Retrogrado(1) => "asustada",
            Fase::Retrogrado(_) => "serio",
            Fase::Lectura if self.actual().is_some_and(|a| a.2) => "asustada",
            Fase::Lectura => "mistica",
            Fase::Asentar | Fase::Profecia(_) => "feliz",
            Fase::Cobro => "guino",
            Fase::Libre => "feliz",
            Fase::Comentario | Fase::Limpia(_) => "mistica",
        }
    }

    pub fn expresion_miku(&self) -> &'static str {
        match self.fase {
            Fase::Retrogrado(0) => "espiral",
            Fase::Retrogrado(1) | Fase::Interrumpida => "asustada",
            Fase::Elegir | Fase::Revelar => "serio",
            Fase::Lectura if self.actual().is_some_and(|a| a.2) => "asustada",
            Fase::Profecia(2) => "serio",
            Fase::Comentario if self.humor == Humor::DeMalas => "enojado",
            Fase::Comentario => "asustada",
            Fase::Profecia(_) if Destino::de_puntos(self.puntos()).es_salazon() => "asustada",
            Fase::Limpia(0) => "serio",
            Fase::Limpia(1) => "espiral",
            Fase::Cobro | Fase::Libre => "enamorada",
            _ => "feliz",
        }
    }

    /// Madame sentada en su cojin, meciendose despacio sobre la bola.
    pub fn madame(&self) -> Mat4 {
        let drama = if self.humor == Humor::Dramatica { 2.2 } else { 1.0 };
        let mece = 0.05 * drama * (self.total * 0.9 * drama).sin();
        let tiembla = if self.fase == Fase::Retrogrado(0) { 0.04 * (self.total * 40.0).sin() } else { 0.0 } + 0.03 * self.relampago * (self.total * 50.0).sin();
        // En trance (leyendo o dando el veredicto) levita sobre su cojin y gira despacito.
        let trance = match self.fase {
            Fase::Interrumpida | Fase::Lectura | Fase::Comentario | Fase::Profecia(1) | Fase::Limpia(1) => suave(self.t / 0.7),
            Fase::Asentar | Fase::Profecia(2) => 1.0 - suave(self.t / 0.5),
            _ => 0.0,
        };
        let levita = trance * (0.16 + 0.03 * (self.total * 2.4).sin());
        let giro = trance * 0.25 * (self.total * 1.3).sin();
        // Dormida: cabecea.
        let cabeceo = if self.humor == Humor::Dormida && trance < 0.5 { 0.12 * ((self.total * 0.4).sin() > 0.2) as i32 as f32 } else { 0.0 };
        Mat4::translation(MADAME_EN + Vec3::new(tiembla, levita, 0.0)) * Mat4::rotation_z(mece * 0.5) * Mat4::rotation_y(mece + giro) * Mat4::rotation_x(cabeceo)
    }

    /// Miku sentada en su cojin, del otro lado de la mesa (mirando a +Z).
    pub fn miku(&self) -> Mat4 {
        let brinco = if self.fase == Fase::Libre { 0.05 * (self.total * 6.0).sin().abs() } else { 0.0 };
        Mat4::translation(MIKU_EN + Vec3::new(0.0, brinco, 0.0)) * Mat4::rotation_y(PI) * Mat4::scale(0.72)
    }

    /// Donde va la carta del lugar `k` del abanico, boca abajo (cuando esta escogida, sale un poco).
    fn en_abanico(&self, k: usize) -> (Vec3, f32) {
        let d = k as f32 - (ABANICO / 2) as f32;
        let escogida = self.fase == Fase::Elegir && k == self.cursor;
        let sale = if escogida { 0.12 + 0.015 * (self.total * 5.0).sin() } else { 0.0 };
        (Vec3::new(d * 0.15, MESA_Y + 0.006 + k as f32 * 0.0015 + sale * 0.5, -0.42 - 0.02 * d * d + sale), -d * 0.12)
    }

    /// Donde flota la carta que se lee (la `i`), frente a la camara.
    fn flotando(&self, i: usize) -> Vec3 {
        Vec3::new((i as f32 - 1.0) * 0.18, MESA_Y + 0.42 + 0.025 * (self.total * 2.0).sin(), -0.2)
    }

    /// El lugar de la carta `i` sobre la mesa, ya leida.
    fn en_mesa(i: usize) -> Vec3 {
        Vec3::new((i as f32 - 1.0) * 0.42, MESA_Y + 0.004, 0.0)
    }

    /// La matriz de una carta: en `pos`, girada `giro` en Y, levantada `angulo` (-90: boca abajo,
    /// 0: de pie viendo a la camara, 90: boca arriba) y con `vuelta` en su eje (PI: invertida).
    fn matriz_carta(pos: Vec3, giro: f32, angulo: f32, vuelta: f32, escala: f32) -> Mat4 {
        Mat4::translation(pos) * Mat4::rotation_y(giro) * Mat4::rotation_x(angulo) * Mat4::rotation_z(vuelta) * Mat4::scale(ESCALA_CARTA * escala)
    }

    /// La carta del lugar `lugar` volando del abanico a flotar (u de 0 a 1): sube, da una vuelta y queda de frente.
    fn volando(&self, lugar: usize, i: usize, invertida: bool, u: f32) -> Mat4 {
        let (desde, giro) = self.en_abanico(lugar);
        let s = suave(u);
        let pos = desde.lerp(self.flotando(i), s) + Vec3::new(0.0, 0.25 * (u * PI).sin(), 0.0);
        let vuelta = if invertida { PI * s } else { 0.0 };
        Self::matriz_carta(pos, giro * (1.0 - s) + TAU * s, -FRAC_PI_2 * (1.0 - s), vuelta, 1.0 + 0.3 * s)
    }

    /// Todas las cartas que se dibujan: (carta en `BARAJA`, matriz). Las boca abajo usan la carta
    /// que les toca igual (el frente queda abajo y no se ve).
    pub fn cartas(&self) -> Vec<(usize, Mat4)> {
        let mut v = Vec::new();
        // Barajando: las siete dan vueltas en circulo sobre la mesa y luego se abren en abanico.
        if self.fase == Fase::Barajar {
            for k in 0..ABANICO {
                let u = self.t / BARAJAR;
                let a = k as f32 / ABANICO as f32 * TAU + u * TAU * 1.5;
                let circulo = Vec3::new(0.32 * a.cos(), MESA_Y + 0.01 + 0.05 * (a * 2.0).sin().abs(), -0.25 + 0.18 * a.sin());
                let (abanico, giro) = self.en_abanico(k);
                let s = suave((u - 0.6) / 0.4);
                v.push((self.abanico[k], Self::matriz_carta(circulo.lerp(abanico, s), (a + FRAC_PI_2) * (1.0 - s) + giro * s, -FRAC_PI_2, 0.0, 1.0)));
            }
            return v;
        }
        let mostrar_abanico = !matches!(self.fase, Fase::Llegada | Fase::Saludo(_)) && !self.terminada();
        if mostrar_abanico {
            for k in 0..ABANICO {
                if self.libre(k) && self.rebobinada.map_or(true, |r| r.lugar != k) {
                    let (pos, giro) = self.en_abanico(k);
                    v.push((self.abanico[k], Self::matriz_carta(pos, giro, -FRAC_PI_2, 0.0, 1.0)));
                }
            }
        }
        for (i, s) in self.sacadas.iter().enumerate() {
            let c = self.abanico[s.lugar];
            let vuelta = if s.invertida { PI } else { 0.0 };
            let ultima = i + 1 == self.sacadas.len();
            let m = match self.fase {
                Fase::Revelar if ultima => self.volando(s.lugar, i, s.invertida, self.t / REVELAR),
                Fase::Interrumpida | Fase::Lectura | Fase::Comentario if ultima => Self::matriz_carta(self.flotando(i), 0.0, 0.0, vuelta, 1.3),
                Fase::Asentar if ultima => {
                    let u = suave(self.t / ASENTAR);
                    Self::matriz_carta(self.flotando(i).lerp(Self::en_mesa(i), u), 0.0, FRAC_PI_2 * u, vuelta, 1.3 - 0.3 * u)
                }
                _ => Self::matriz_carta(Self::en_mesa(i), 0.0, FRAC_PI_2, vuelta, 1.0),
            };
            v.push((c, m));
        }
        // El rebobinado: la carta hace el camino al reves, de flotar al abanico.
        if let (Fase::Retrogrado(0), Some(r)) = (self.fase, self.rebobinada) {
            let u = 1.0 - (self.t / (REBOBINAR - 0.5)).min(1.0);
            v.push((self.abanico[r.lugar], self.volando(r.lugar, self.sacadas.len(), r.invertida, u)));
        }
        v
    }

    /// Donde esta el Sol, afuera de la ventana. En retrogrado hace el rizo: avanza, se regresa y sigue.
    pub fn sol(&self) -> Vec3 {
        if let Fase::Retrogrado(0) = self.fase {
            let u = (self.t / REBOBINAR).min(1.0) * TAU;
            return SOL + Vec3::new(9.0 * u.sin(), 3.0 * (1.0 - u.cos()), 0.0);
        }
        SOL
    }

    /// La camara: (ojo, a donde mira, arriba). En retrogrado la camara se ladea y gira para atras.
    pub fn camara(&self) -> (Vec3, Vec3, Vec3) {
        let arriba = Vec3::new(0.0, 1.0, 0.0);
        let vaiven = 0.12 * (self.total * 0.3).sin();
        let (ojo, mira) = match self.fase {
            // Desde afuera, entrando por la puerta de la carpa.
            Fase::Llegada => {
                let u = suave(self.t / 5.5);
                (Vec3::new(1.5 * (1.0 - u), 2.6 - 1.1 * u, -13.0 + 10.3 * u), Vec3::new(0.0, 1.2 - 0.2 * u, 1.0))
            }
            // Por encima del hombro de Miku, viendo a Madame.
            Fase::Saludo(_) | Fase::Cobro => (Vec3::new(0.9 + vaiven, 1.75, -2.4), Vec3::new(-0.1, 0.95, 0.8)),
            // Desde arriba de la mesa, para escoger.
            Fase::Barajar | Fase::Elegir | Fase::Asentar => (Vec3::new(0.25 + vaiven * 0.5, 2.05, -0.95), Vec3::new(0.0, MESA_Y, 0.1)),
            // Cerca de la carta que flota, con Madame atras.
            Fase::Revelar | Fase::Interrumpida | Fase::Lectura | Fase::Comentario => {
                let i = self.sacadas.len().saturating_sub(1);
                let f = self.flotando(i);
                (Vec3::new(f.x * 0.6 + 0.35, 1.2, -0.72), Vec3::new(f.x * 0.6 - 0.05, 0.98, 0.4))
            }
            Fase::Retrogrado(0) => {
                // Gira hacia atras alrededor de la mesa.
                let a = -FRAC_PI_2 - self.t * 0.9;
                let ojo = Vec3::new(2.3 * a.cos(), 1.6, 2.3 * a.sin());
                let ladeo = 0.35 * (self.t * 2.0).sin();
                return (ojo, Vec3::new(0.0, 0.8, 0.2), Vec3::new(ladeo.sin(), ladeo.cos(), 0.0));
            }
            Fase::Retrogrado(1) => (Vec3::new(-0.3, 1.25, -0.9), Vec3::new(0.0, 1.15, 1.2)),
            // GLADAS habla: se ve el Sol por la ventana.
            Fase::Retrogrado(_) => (Vec3::new(0.0, 1.5, -3.6), Vec3::new(0.0, 1.9, 4.5)),
            // La bola de cristal de cerca.
            Fase::Profecia(0) | Fase::Profecia(1) => (Vec3::new(0.15, 1.05, -0.35), BOLA + Vec3::new(0.0, 0.05, 0.0)),
            // Miku opina: se le ve la cara (desde el lado de Madame).
            Fase::Profecia(_) => (Vec3::new(0.5, 1.15, 0.1), Vec3::new(0.0, 0.95, -1.2)),
            // La limpia: de cerquita a Miku, con Madame pasandole el huevo.
            Fase::Limpia(0) => (Vec3::new(0.9 + vaiven, 1.75, -2.4), Vec3::new(-0.1, 0.95, 0.8)),
            Fase::Limpia(_) => (Vec3::new(0.85, 1.25, -0.2), Vec3::new(0.0, 0.85, -1.2)),
            Fase::Libre => {
                let lado = 1.4 * (self.total * 0.15).sin();
                (Vec3::new(lado, 1.9, -3.3), Vec3::new(0.0, 0.9, 0.5))
            }
        };
        // El rayo sacude la camara.
        let s = self.relampago * self.relampago * 0.05;
        let ojo = ojo + Vec3::new((self.total * 61.0).sin() * s, (self.total * 47.0).cos() * s, 0.0);
        (ojo, mira, arriba)
    }

    /// El humo del incienso, el brillo de la bola de cristal y los destellos de la carta que flota.
    pub fn dibujar_efectos(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4) {
        // Dos varitas de incienso en las esquinas de la mesa, con su humo que sube ondulando.
        for (k, base) in [Vec3::new(0.78, MESA_Y + 0.05, -0.2), Vec3::new(-0.55, MESA_Y + 0.05, 0.6)].iter().enumerate() {
            let mut anterior = None;
            for s in 0..40 {
                let u = s as f32 / 40.0;
                let p = *base + Vec3::new(0.06 * (u * 9.0 - self.total * 1.5 + k as f32).sin() * u * 2.0, u * 1.4, 0.04 * (u * 7.0 - self.total).cos() * u);
                let q = proyectar(p, vista_proyeccion, fb);
                if let (Some(a), Some(b)) = (anterior, q) {
                    let c = 0.22 * (1.0 - u);
                    fb.linea_de_luz(a, b, [c * 0.8, c * 0.7, c], [c * 0.8, c * 0.7, c]);
                }
                anterior = q;
            }
        }
        // Remolinos de luz dentro de la bola de cristal.
        let rojo = if self.en_retrogrado() { 1.0 } else { 0.0 };
        for k in 0..3 {
            let mut anterior = None;
            for s in 0..=24 {
                let a = s as f32 / 24.0 * TAU;
                let r = 0.09 + 0.03 * (a * 3.0 + self.total * (1.0 + k as f32)).sin();
                let giro = self.total * (0.8 + 0.4 * k as f32) * if self.en_retrogrado() { -3.0 } else { 1.0 };
                let p = BOLA + Vec3::new(r * (a + giro).cos(), 0.04 * (a * 2.0 + k as f32).sin(), r * (a + giro).sin());
                let q = proyectar(p, vista_proyeccion, fb);
                if let (Some(x), Some(y)) = (anterior, q) {
                    let c = [0.35 + 0.4 * rojo, 0.25 * (1.0 - rojo), 0.5 * (1.0 - rojo) + 0.05];
                    fb.linea_de_luz(x, y, c, c);
                }
                anterior = q;
            }
        }
        // Destellos alrededor de la carta que flota.
        if matches!(self.fase, Fase::Interrumpida | Fase::Lectura) {
            let f = self.flotando(self.sacadas.len().saturating_sub(1));
            for k in 0..14 {
                let a = k as f32 / 14.0 * TAU + self.total * 0.8;
                let p = f + Vec3::new(0.32 * a.cos(), 0.4 * a.sin(), -0.02);
                let b = 0.5 + 0.5 * (self.total * 6.0 + k as f32 * 1.7).sin();
                if let Some((x, y, z)) = proyectar(p, vista_proyeccion, fb) {
                    let t = 3.0 * b;
                    fb.linea_de_luz((x - t, y, z), (x + t, y, z), [0.0; 3], [b, 0.8 * b, 0.4 * b]);
                    fb.linea_de_luz((x, y - t, z), (x, y + t, z), [0.0; 3], [b, 0.8 * b, 0.4 * b]);
                }
            }
        }
    }

    /// La limpia: el huevo dando vueltas alrededor de Miku y hojitas de ruda con humo de copal.
    pub fn dibujar_limpia(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4) {
        if self.fase != Fase::Limpia(1) {
            return;
        }
        let centro = MIKU_EN + Vec3::new(0.0, 0.55, 0.0);
        let a = self.t * 4.0;
        let huevo = centro + Vec3::new(0.32 * a.cos(), 0.18 * (self.t * 2.0).sin(), 0.32 * a.sin());
        if let Some((x, y, _)) = proyectar(huevo, vista_proyeccion, fb) {
            crate::post::halo(fb, (x, y), 22.0, [0.9, 0.85, 0.7], 1.2);
            crate::post::halo(fb, (x, y), 8.0, [1.0, 1.0, 0.95], 0.6);
        }
        for k in 0..18 {
            let u = ((self.t * 0.35 + k as f32 / 18.0) % 1.0) as f32;
            let b = k as f32 * 2.4;
            let p = centro + Vec3::new(0.4 * (b + u * 3.0).cos() * (1.0 - u * 0.5), -0.5 + u * 1.6, 0.4 * (b + u * 3.0).sin() * (1.0 - u * 0.5));
            if let Some((x, y, _)) = proyectar(p, vista_proyeccion, fb) {
                let vida = (1.0 - u) * u * 4.0;
                let c = if k % 3 == 0 { [0.3 * vida, 0.3 * vida, 0.35 * vida] } else { [0.1 * vida, 0.6 * vida, 0.2 * vida] };
                crate::post::halo(fb, (x, y), 6.0 + 10.0 * u, c, 1.6);
            }
        }
    }

    /// Los resplandores: las velas (que parpadean), la bola de cristal y la gema del turbante.
    pub fn dibujar_luces(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4) {
        // Con salazon las velas se ponen verdes de miedo; con bendicion, doradas y grandotas.
        let veredicto = if matches!(self.fase, Fase::Profecia(1) | Fase::Profecia(2) | Fase::Cobro | Fase::Limpia(_)) { Some(Destino::de_puntos(self.puntos())) } else { None };
        let (color_vela, tam_vela) = match veredicto {
            Some(d) if d.es_salazon() => ([0.15, 0.55, 0.25], 1.0),
            Some(Destino::Bendicion) => ([0.8, 0.6, 0.2], 1.5),
            _ => ([0.55, 0.3, 0.08], 1.0),
        };
        for (k, v) in VELAS.iter().enumerate() {
            let parpadeo = 0.8 + 0.2 * (self.total * 11.0 + k as f32 * 2.3).sin() * (self.total * 6.1 + k as f32).sin();
            let grande = if v.y > 1.0 { 1.0 } else { 0.6 };
            if let Some((x, y, _)) = proyectar(*v + Vec3::new(0.0, 0.05, 0.0), vista_proyeccion, fb) {
                let c = color_vela.map(|c| c * parpadeo);
                crate::post::halo(fb, (x, y), 46.0 * parpadeo * grande * tam_vela, c, 1.7);
            }
        }
        // La carta escogida del abanico brilla.
        if self.fase == Fase::Elegir {
            let (pos, _) = self.en_abanico(self.cursor);
            if let Some((x, y, _)) = proyectar(pos + Vec3::new(0.0, 0.02, 0.0), vista_proyeccion, fb) {
                let b = 0.8 + 0.2 * (self.total * 5.0).sin();
                crate::post::halo(fb, (x, y), 90.0, [0.9 * b, 0.7 * b, 0.2 * b], 1.4);
            }
        }
        let pulso = 0.7 + 0.3 * (self.total * 2.2).sin();
        let color = match veredicto {
            _ if self.en_retrogrado() => [0.7 * pulso, 0.05, 0.08],
            Some(d) if d.es_salazon() => [0.1 * pulso, 0.7 * pulso, 0.3 * pulso],
            Some(Destino::Bendicion | Destino::BuenaSuerte) => [0.9 * pulso, 0.7 * pulso, 0.2 * pulso],
            _ => [0.35 * pulso, 0.18 * pulso, 0.55 * pulso],
        };
        if let Some((x, y, _)) = proyectar(BOLA, vista_proyeccion, fb) {
            crate::post::halo(fb, (x, y), 80.0, color, 1.5);
        }
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, ancho: i32, alto: i32) {
        let (aceptar, salir) = (&ui::boton(e.nombre("aceptar")), &ui::boton(e.nombre("aterrizar")));
        let centrado = |d: &mut RaylibDrawHandle, s: &str, y: i32, tam: i32, color: Color| ui::centrado(d, s, ancho, y, tam, color);
        // La tarjeta del titulo al llegar (con como amanecio hoy Madame).
        if self.fase == Fase::Llegada {
            let a = (suave(self.t / 0.8) * (1.0 - suave((self.t - 4.5) / 1.0)) * 255.0) as u8;
            centrado(d, "MADAME MERCURIO", 70, 60, Color::new(255, 220, 140, a));
            centrado(d, "Tarot, limpias y excusas astronómicas", 145, 24, Color::new(230, 210, 255, a));
            let (humor, color) = match self.humor {
                Humor::DeBuenas => ("hoy amaneció de buenas", Color::new(150, 240, 190, a)),
                Humor::DeMalas => ("hoy amaneció DE MALAS", Color::new(255, 120, 120, a)),
                Humor::Dormida => ("hoy amaneció dormida", Color::new(170, 190, 255, a)),
                Humor::Dramatica => ("hoy amaneció DRAMÁTICA", Color::new(255, 150, 230, a)),
            };
            centrado(d, humor, 182, 22, color);
        }
        // Las tres posiciones arriba, con lo que ya salio (en el veredicto se quitan: cae el sello).
        if !matches!(self.fase, Fase::Llegada | Fase::Saludo(_) | Fase::Profecia(1) | Fase::Profecia(2) | Fase::Cobro | Fase::Limpia(_)) {
            let (w, h) = (210, 62);
            let x0 = (ancho - 3 * w - 2 * 14) / 2;
            for (i, pos) in POSICIONES.iter().enumerate() {
                let x = x0 + i as i32 * (w + 14);
                ui::panel(d, x, 16, w, h);
                let activa = i == self.sacadas.len() && matches!(self.fase, Fase::Elegir);
                ui::texto(d, pos, x + 14, 24, 18, if activa { Color::new(255, 220, 140, 255) } else { ui::TENUE });
                let (nombre, color) = match self.sacadas.get(i) {
                    Some(s) if !(self.fase == Fase::Revelar && i + 1 == self.sacadas.len()) => {
                        let c = BARAJA[self.abanico[s.lugar]].nombre;
                        (if s.invertida { format!("{c} (inv.)") } else { c.to_string() }, if s.invertida { ui::ROJO } else { ui::TEXTO })
                    }
                    _ => ("?".to_string(), ui::TENUE),
                };
                ui::texto(d, &nombre, x + 14, 46, 20, color);
            }
        }
        // El nombre de la carta que flota, grande (y si es la primera vez que sale, se avisa).
        if let (Fase::Interrumpida | Fase::Lectura | Fase::Comentario, Some((i, c, invertida))) = (self.fase, self.actual()) {
            let a = if self.fase == Fase::Comentario { 255 } else { (suave(self.t / 0.4) * 255.0) as u8 };
            let titulo = if invertida { format!("{} · INVERTIDA", c.nombre) } else { c.nombre.to_string() };
            centrado(d, POSICIONES[i], 100, 22, Color::new(220, 200, 255, a));
            centrado(d, &titulo, 126, 44, if invertida { Color::new(255, 120, 140, a) } else { Color::new(255, 225, 140, a) });
            if self.nueva && self.fase != Fase::Interrumpida {
                let latido = 1.0 + 0.08 * (self.total * 6.0).sin();
                let t = (22.0 * latido) as i32;
                let s = "¡CARTA NUEVA!";
                ui::texto_girado(d, s, ((ancho + ui::medir(&titulo, 44)) / 2 + 16) as f32, 120.0, t as f32, -8.0, Color::new(120, 255, 200, a));
            }
        }
        // El veredicto: el sello que cae (y la sal o los destellos).
        if let Some(dest) = matches!(self.fase, Fase::Profecia(1) | Fase::Profecia(2) | Fase::Cobro | Fase::Limpia(_)).then(|| Destino::de_puntos(self.puntos())) {
            let caida = if self.fase == Fase::Profecia(1) { suave(self.t / 0.35) } else { 1.0 };
            self.dibujar_suerte(d, dest, ancho, alto);
            let (fondo, tinta) = match dest {
                Destino::Bendicion => (Color::new(90, 60, 10, 220), Color::new(255, 220, 90, 255)),
                Destino::BuenaSuerte => (Color::new(20, 70, 40, 220), Color::new(150, 255, 170, 255)),
                Destino::NiFuNiFa => (Color::new(50, 45, 60, 220), Color::new(210, 200, 230, 255)),
                _ => (Color::new(80, 10, 20, 230), Color::new(255, 90, 100, 255)),
            };
            let tam = 56.0 * (2.4 - 1.4 * caida);
            let texto = dest.sello();
            let w = ui::medir(texto, tam as i32) as f32;
            let (cx, cy) = (ancho as f32 / 2.0, 150.0);
            let a = (caida * 255.0) as u8;
            let caja = |extra: f32| Rectangle::new(cx, cy, w + 40.0 + extra, tam + 26.0 + extra);
            d.draw_rectangle_pro(caja(10.0), Vector2::new((w + 50.0) / 2.0, (tam + 36.0) / 2.0), -8.0, Color::new(tinta.r, tinta.g, tinta.b, a));
            d.draw_rectangle_pro(caja(0.0), Vector2::new((w + 40.0) / 2.0, (tam + 26.0) / 2.0), -8.0, Color::new(fondo.r, fondo.g, fondo.b, a));
            let rad = (-8.0f32).to_radians();
            let (dx, dy) = (-w / 2.0, -tam / 2.0);
            let (x, y) = (cx + dx * rad.cos() - dy * rad.sin(), cy + dx * rad.sin() + dy * rad.cos());
            ui::texto_girado(d, texto, x, y, tam, -8.0, Color::new(tinta.r, tinta.g, tinta.b, a));
        }
        // El retrogrado: alerta roja y la pantalla de VHS rebobinando.
        if self.fase == Fase::Retrogrado(0) {
            if (self.total * 4.0) as i32 % 2 == 0 {
                centrado(d, "¡MERCURIO RETRÓGRADO!", alto / 2 - 50, 58, Color::new(255, 70, 90, 255));
            }
            let segundos = ((REBOBINAR - self.t) * 7.0 + 12.0).max(0.0);
            let vhs = format!("<< REBOBINANDO   00:{:02}:{:02}", segundos as i32, ((segundos.fract()) * 60.0) as i32);
            ui::texto(d, &vhs, 34, 102, 30, Color::BLACK);
            ui::texto(d, &vhs, 32, 100, 30, Color::WHITE);
        }
        // Lo que se dice, abajo.
        if let (Some((nombre, _, _)), false) = (self.hablante(), texto.is_empty()) {
            let w = 920.min(ancho - 40);
            let h = ui::alto_dialogo(nombre, texto, w);
            let color = match nombre {
                MADAME => Color::new(255, 150, 220, 255),
                MIKU => Color::new(120, 230, 220, 255),
                _ => ui::LILA,
            };
            ui::caja_dialogo(d, nombre, color, texto, (ancho - w) / 2, alto - h - 20, w);
        }
        // Escogiendo: como se juega.
        if self.fase == Fase::Elegir {
            let pos = POSICIONES[self.sacadas.len().min(2)];
            centrado(d, &format!("Escoja la carta de su {}", pos.to_lowercase()), alto - 130, 30, Color::new(255, 225, 140, 255));
            let ayuda = format!("{}: escoger      {aceptar}: sacar la carta      {salir}: despegar", ui::boton("flechas"));
            centrado(d, &ayuda, alto - 80, 20, ui::TEXTO);
        }
        // La limpia: si o no (con lo que se tiene de notas).
        if self.fase == Fase::Limpia(0) {
            let opciones = [format!("Sí, la limpia ({PRECIO_LIMPIA} notas)"), "No, me aguanto la salazón".to_string()];
            let (bw, bh) = (380.min(ancho / 2 - 40), 56);
            let y = alto / 2 - 40;
            for (k, s) in opciones.iter().enumerate() {
                let x = ancho / 2 - bw - 12 + k as i32 * (bw + 24);
                let escogida = (k == 0) == self.limpia_si;
                ui::panel(d, x, y, bw, bh);
                if escogida {
                    d.draw_rectangle_lines_ex(Rectangle::new(x as f32 - 3.0, y as f32 - 3.0, bw as f32 + 6.0, bh as f32 + 6.0), 3.0, Color::new(255, 220, 140, 255));
                }
                let t = 22;
                ui::texto(d, s, x + (bw - ui::medir(s, t)) / 2, y + (bh - t) / 2, t, if escogida { Color::new(255, 225, 140, 255) } else { ui::TENUE });
            }
            let tiene = format!("Tiene {} notas", self.notas);
            centrado(d, &tiene, y + bh + 14, 18, if self.notas >= PRECIO_LIMPIA { ui::TEXTO } else { Color::new(255, 150, 160, 255) });
        }
        if self.fase == Fase::Libre {
            centrado(d, &format!("{aceptar}: otra lectura      {salir}: despegar"), 100, 24, Color::new(255, 220, 255, 255));
            let vistas = (self.descubiertas | self.vistas()).count_ones();
            centrado(d, &format!("Cartas descubiertas: {vistas} de {}", BARAJA.len()), 134, 20, Color::new(220, 200, 255, 255));
            if self.maldicion > 0.0 {
                let s = format!("Maldición del retrógrado: {:.0} segundos al volar", self.maldicion);
                centrado(d, &s, 162, 20, Color::new(255, 130, 140, 255));
            }
        }
    }

    /// La suerte que llueve: sal (con su salero arriba) si salio salazon, destellos dorados si salio
    /// bendicion o buena suerte.
    fn dibujar_suerte(&self, d: &mut RaylibDrawHandle, dest: Destino, ancho: i32, alto: i32) {
        let (w, h) = (ancho as f32, alto as f32);
        if dest.es_salazon() {
            let cuantos = if dest == Destino::SalazonTotal { 160 } else { 80 };
            for k in 0..cuantos {
                let semilla = (k as u32).wrapping_mul(2_654_435_761);
                let x0 = (semilla % 1000) as f32 / 1000.0;
                let vel = 0.25 + ((semilla / 1000) % 100) as f32 / 300.0;
                let y = ((self.total * vel + (semilla / 7 % 1000) as f32 / 1000.0) % 1.0) * h;
                let x = (w * 0.5 + (x0 - 0.5) * w * (0.15 + 0.85 * y / h)) + 6.0 * (self.total * 3.0 + k as f32).sin();
                let lado = 3.0 + (k % 3) as f32;
                d.draw_rectangle_v(Vector2::new(x, y), Vector2::new(lado, lado), Color::new(250, 250, 255, 220));
            }
            // El salero de cabeza, sacudiendose arriba al centro: vidrio con sal y la tapa con hoyitos.
            let giro = 180.0 + 22.0 * (self.total * 9.0).sin();
            let centro = Vector2::new(w * 0.5, 40.0);
            let r = giro.to_radians();
            let en = |x: f32, y: f32| Vector2::new(centro.x + x * r.cos() - y * r.sin(), centro.y + x * r.sin() + y * r.cos());
            d.draw_rectangle_pro(Rectangle::new(centro.x, centro.y, 44.0, 62.0), Vector2::new(22.0, 31.0), giro, Color::new(200, 225, 245, 230));
            d.draw_rectangle_pro(Rectangle::new(en(0.0, 10.0).x, en(0.0, 10.0).y, 36.0, 34.0), Vector2::new(18.0, 17.0), giro, Color::new(250, 250, 255, 255));
            let tapa = en(0.0, -38.0);
            d.draw_rectangle_pro(Rectangle::new(tapa.x, tapa.y, 48.0, 16.0), Vector2::new(24.0, 8.0), giro, Color::new(175, 180, 195, 255));
            for k in -1..=1 {
                d.draw_circle_v(en(k as f32 * 11.0, -40.0), 2.6, Color::new(60, 60, 75, 255));
            }
        } else if matches!(dest, Destino::Bendicion | Destino::BuenaSuerte) {
            let cuantos = if dest == Destino::Bendicion { 70 } else { 30 };
            for k in 0..cuantos {
                let semilla = (k as u32).wrapping_mul(2_246_822_519);
                let x = (semilla % 1000) as f32 / 1000.0 * w;
                let vel = 0.15 + ((semilla / 1000) % 100) as f32 / 500.0;
                let y = h - ((self.total * vel + (semilla / 11 % 1000) as f32 / 1000.0) % 1.0) * h;
                let r = 3.0 + 3.0 * (self.total * 5.0 + k as f32).sin().abs();
                let c = Color::new(255, 220, 120, 200);
                d.draw_line_ex(Vector2::new(x - r, y), Vector2::new(x + r, y), 2.0, c);
                d.draw_line_ex(Vector2::new(x, y - r), Vector2::new(x, y + r), 2.0, c);
            }
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn aceptar() -> Entrada {
        Entrada { aceptar: true, ..Default::default() }
    }

    /// Avanza aceptando hasta que toca escoger una carta.
    fn hasta_elegir(m: &mut Mercurio) {
        for _ in 0..60 {
            if m.fase == Fase::Elegir && m.t > 0.3 {
                return;
            }
            let e = if m.hablante().is_some() { aceptar() } else { Entrada::default() };
            m.actualizar(&e, 0.5);
        }
        panic!("nunca llego a escoger: {:?}", m.fase);
    }

    /// Una lectura completa: tres cartas distintas, el retrogrado una vez, y la carta repetida sale invertida.
    #[test]
    fn la_lectura_tiene_tres_cartas_y_un_retrogrado() {
        for semilla in 1..20 {
            let mut m = Mercurio::new(semilla);
            let mut vio_retro = 0;
            let mut antes = false;
            for _ in 0..400 {
                if m.terminada() {
                    break;
                }
                if m.fase == Fase::Elegir {
                    hasta_elegir(&mut m);
                    m.actualizar(&aceptar(), 0.1);
                } else {
                    let e = if m.hablante().is_some() && m.t > 0.5 { aceptar() } else { Entrada::default() };
                    m.actualizar(&e, 0.1);
                }
                let ahora = m.en_retrogrado();
                if ahora && !antes {
                    vio_retro += 1;
                }
                antes = ahora;
            }
            assert!(m.terminada(), "la lectura {semilla} no termino: {:?}", m.fase);
            assert_eq!(vio_retro, 1, "el retrogrado pasa una vez por lectura");
            assert_eq!(m.sacadas.len(), 3);
            let mut lugares: Vec<usize> = m.sacadas.iter().map(|s| s.lugar).collect();
            lugares.dedup();
            assert_eq!(lugares.len(), 3, "tres cartas distintas");
            assert!(m.sacadas[m.retro_en].invertida, "la carta de despues del retrogrado sale invertida");
            assert!(m.hablante().is_some_and(|h| !h.1.is_empty()));
        }
    }

    /// Con semillas distintas salen cartas y dialogos distintos.
    #[test]
    fn cada_lectura_es_distinta() {
        let a = Mercurio::new(3);
        let b = Mercurio::new(91);
        assert!(a.abanico != b.abanico || a.saludo != b.saludo);
        // Las siete del abanico no se repiten.
        let mut v = a.abanico.to_vec();
        v.sort();
        v.dedup();
        assert_eq!(v.len(), ABANICO);
    }

    /// Con salazon cae la maldicion; la limpia (si alcanzan las notas) la quita y cobra.
    #[test]
    fn la_limpia_quita_la_salazon() {
        let semilla = (1..3000).find(|&s| {
            let mut m = Mercurio::new(s);
            m.jugar_hasta("Limpia");
            matches!(m.fase, Fase::Limpia(0))
        });
        let mut m = Mercurio::new(semilla.expect("alguna lectura sale salada"));
        m.jugar_hasta("Limpia");
        assert!(Destino::de_puntos(m.puntos()).es_salazon());
        assert!(m.maldicion > 0.0, "la salazon trae maldicion");
        m.notas = 99;
        m.actualizar(&aceptar(), 1.0);
        assert_eq!(m.maldicion, 0.0);
        assert_eq!(m.cobrar, PRECIO_LIMPIA);
        assert!(m.limpia_hecha);
    }

    /// Cada humor tiene sus tres comentarios distintos.
    #[test]
    fn los_comentarios_no_se_repiten() {
        for semilla in 1..60 {
            let m = Mercurio::new(semilla);
            if m.humor != Humor::DeBuenas {
                let c = m.comentarios;
                assert!(c[0] != c[1] && c[1] != c[2] && c[0] != c[2], "{:?}: {c:?}", m.humor);
            }
        }
    }

    /// El cursor no se para en una carta que ya se saco.
    #[test]
    fn el_cursor_salta_las_cartas_sacadas() {
        let mut m = Mercurio::new(5);
        hasta_elegir(&mut m);
        m.actualizar(&aceptar(), 0.1);
        let sacada = m.sacadas[0].lugar;
        m.cursor_valido();
        assert_ne!(m.cursor, sacada);
        for _ in 0..10 {
            m.mover_cursor(1);
            assert_ne!(m.cursor, sacada);
        }
    }
}
