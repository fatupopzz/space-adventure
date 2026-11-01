//! Los personajes que se pueden escoger (Space Adventure): cada uno pilota la nave,
//! baila, aterriza y hace todo lo que antes hacia Miku. Dos vienen de fabrica; los
//! demas se desbloquean subiendo de nivel en el pase espacial, como en Fortnite.
//!
//! Los modelos los arma `herramientas/generar_personajes.py` con el mismo esqueleto
//! chibi de Miku (mismas partes por nombre), asi que el juego los trata igual.

use crate::sonido::Voz;

pub struct Ficha {
    pub id: &'static str,
    pub nombre: &'static str,
    /// Lo que dice GLADAS de el (sale en el casillero).
    pub texto: &'static str,
    /// Si en los dialogos se le habla en femenino ("Bienvenida").
    pub ella: bool,
    /// Los archivos: `modelos/<archivo>_piloto.obj` y `modelos/<archivo>_baile.obj`.
    pub archivo: &'static str,
}

const fn ficha(id: &'static str, nombre: &'static str, ella: bool, archivo: &'static str, texto: &'static str) -> Ficha {
    Ficha { id, nombre, texto, ella, archivo }
}

pub const PERSONAJES: [Ficha; 26] = [
    ficha("chavo", "El Chavo", false, "personaje_chavo", "Vive en un barril, pero ahora tiene nave. «Fue sin querer queriendo», dijo al despegar."),
    ficha("miku", "Miku", true, "miku", "La piloto original. Canta, baila y maneja; GLADAS dice que es la única que lee el manual."),
    ficha("simi", "Doctor Simi", false, "personaje_simi", "Lo mismo, pero en el espacio. Lleva botiquín, bigote y ganas de bailar en cualquier esquina."),
    ficha("shrek", "Shrek", false, "personaje_shrek", "Salió de su pantano para conocer el universo. Dice que los planetas son como las cebollas: tienen capas."),
    ficha("steve", "Steve", false, "personaje_steve", "Cabeza cuadrada, corazón cuadrado. Quiso picar el Sol para sacarle carbón. No se le dejó."),
    ficha("chilindrina", "La Chilindrina", true, "personaje_chilindrina", "Trenzas, lentes y pecas. Si no la dejan pilotar, llora (y se le olvida a los dos segundos)."),
    ficha("doge", "Doge", false, "personaje_doge", "Wow. Muy nave. Tan espacio. Mucho planeta. Such piloto."),
    ficha("mario", "Mario", false, "personaje_mario", "Vino buscando a una princesa y encontró un sistema solar. Brinca más alto en la Luna."),
    ficha("patricio", "Patricio", false, "personaje_patricio", "¿Esto es el espacio? No, esto es Patricio. Confunde los planetas con medusas."),
    ficha("creeper", "El Creeper", false, "personaje_creeper", "Sssssss... Tranquilo: GLADAS le quitó la pólvora. Casi toda."),
    ficha("kirby", "Kirby", false, "personaje_kirby", "Una bola rosa que se come lo que encuentra. Por favor no se coma la nave otra vez."),
    ficha("gato", "Gato que grita", false, "personaje_gato", "AAAAAAAAAAA. (Es su forma de decir hola. Y adiós. Y todo lo demás.)"),
    ficha("pikachu", "Pikachu", false, "personaje_pikachu", "Carga la nave con sus cachetes. GLADAS lo usa de batería de emergencia, sin decirle."),
    ficha("chapulin", "El Chapulín", false, "personaje_chapulin", "¡No contaban con su astucia! Sus antenitas de vinil detectan cometas (dice él)."),
    ficha("knuckles", "Ugandan Knuckles", false, "personaje_knuckles", "Anda buscando a la reina por todo el sistema solar. Hace clic con la boca. Mucho."),
    ficha("koch", "Profe Koch", false, "personaje_koch", "El de Teoría de la Computación. Trae su regla, sus lentes y un autómata para cada ocasión. Se gana entregándole los tres proyectos al cien la noche de la entrega."),
    ficha("vegetta", "Vegetta777", false, "personaje_vegetta", "El dueño de Planeta Vegetta. Construye una casa en cada planeta que toca. Se gana ganándole un minijuego a Willyrex en su servidor."),
    ficha("willyrex", "Willyrex", false, "personaje_willyrex", "El compa de Vegetta, con audífonos de gamer. Dice que él encontró los diamantes primero. Llega junto con Vegetta."),
    ficha("capibara", "La capibara", true, "personaje_capibara", "Se lleva bien con todos: con los cocodrilos, con los pájaros y ahora con los asteroides. Trae una mandarina en la cabeza y cero preocupaciones."),
    ficha("moodeng", "Moo Deng", true, "personaje_moodeng", "La hipopótamo bebé más famosa de internet. Muerde lo que se le acerca, incluido el volante. Brilla porque está mojada. Siempre está mojada."),
    ficha("pedro", "Pedro", false, "personaje_pedro", "El mapache que baila. Pedro, Pedro, Pedro, Pedro, pe. Se metió a la nave buscando basura y se quedó porque había música."),
    ficha("chillguy", "Chill Guy", false, "personaje_chillguy", "Un perrito tranquilo, de suéter y tenis rojos. Lo persigue un agujero negro y él como si nada. Es un chico chill y ya."),
    ficha("labubu", "Labubu", true, "personaje_labubu", "El monstruito de peluche de la sonrisa de nueve dientes. Salió de una caja sorpresa. Nadie sabe si es tierna o si planea algo. Las dos."),
    ficha("tungtung", "Tung Tung Tung Sahur", false, "personaje_tungtung", "Un tronco con ojos y su bate. Toca la puerta tres veces a la hora del desayuno. Si no le abren, toca tres veces más. Tung tung tung."),
    ficha("bananini", "Chimpanzini Bananini", false, "personaje_bananini", "Un changuito adentro de una banana. Premio final de la Temporada 1 del pase: solo lo tiene quien llegó al nivel 40. Wa wa wa."),
    ficha("rocky", "Rocky", false, "personaje_rocky", "Ingeniero de Erid (Project Hail Mary). No tiene ojos: oye con todo el caparazón. Habla con acordes y arregla la nave a medio vuelo sin que nadie se lo pida. «¡Puño mi choque!»"),
];

/// La voz (los "blips" de cuando habla) de cada personaje.
pub fn voz(id: &str) -> Voz {
    match id {
        "chavo" | "mario" | "chapulin" => Voz::Erizo,
        "simi" => Voz::Profe,
        "shrek" | "knuckles" => Voz::Planeta,
        "steve" => Voz::Rover,
        "chilindrina" => Voz::Robotina,
        "doge" | "gato" | "kirby" | "pikachu" => Voz::Animalito,
        "patricio" => Voz::Neptuno,
        "creeper" => Voz::Radio,
        "koch" => Voz::Profe,
        "vegetta" | "willyrex" => Voz::Erizo,
        // Los eridianos hablan con acordes: una voz musical y aguda.
        "rocky" => Voz::Hada,
        "capibara" | "moodeng" | "pedro" | "labubu" | "bananini" => Voz::Animalito,
        "chillguy" => Voz::Planeta,
        "tungtung" => Voz::Grunido,
        _ => Voz::Miku,
    }
}

/// Altura propia aun cuando dos pilotos usan el mismo tipo de sintetizador.
pub fn tono(id: &str) -> f32 {
    match id {
        "chavo" => 0.90, "miku" => 1.0, "simi" => 1.08, "shrek" => 0.72,
        "steve" => 0.85, "chilindrina" => 1.22, "doge" => 0.78, "mario" => 1.18,
        "patricio" => 0.68, "creeper" => 0.80, "kirby" => 1.25, "gato" => 1.50,
        "pikachu" => 1.10, "chapulin" => 1.04, "knuckles" => 0.92, "koch" => 0.78,
        "vegetta" => 1.0, "willyrex" => 0.94, "rocky" => 1.3,
        "capibara" => 0.7, "moodeng" => 1.35, "pedro" => 1.15, "chillguy" => 0.85, "labubu" => 1.4, "tungtung" => 0.9, "bananini" => 1.2, _ => 1.0,
    }
}

/// Lo que dice de su cosecha cada uno, al final de algunas de sus lineas.
fn muletillas(id: &str) -> &'static [&'static str] {
    match id {
        "chavo" => &["Eso, eso, eso.", "Fue sin querer queriendo.", "¡Se me chispoteó!", "Bueno, pero no se enojen."],
        "simi" => &["¡Lo mismo, pero más barato!", "*baila un poquito*", "Tómese su agüita."],
        "shrek" => &["¡Fuera de mi pantano!", "Los ogros son como las cebollas: tienen capas.", "Burro, ¿dónde estás?"],
        "steve" => &["*hmm*", "¿Eso se puede picar?", "Voy por más bloques."],
        "chilindrina" => &["¡Fíjate, fíjate, fíjate!", "*llora* ...ya se me pasó.", "¡Ay, qué bonito!"],
        "mario" => &["¡Wahoo!", "¡Let's-a go!", "¡Mamma mia!"],
        "patricio" => &["¿Esto es el espacio? No, esto es Patricio.", "¡Tengo una idea! ...se me olvidó.", "¿La mayonesa es un planeta?"],
        "chapulin" => &["¡No contaban con mi astucia!", "¡Síganme los buenos!", "¡Que no panda el cúnico!"],
        "knuckles" => &["*clic clic* ¿Conoces el camino?", "¡Hermanos, por aquí!", "*clic clic clic*"],
        "koch" => &["Eso es un autómata finito. Todo es un autómata finito.", "Anótenlo, que viene en el examen.", "Muy bien. Ahora demuéstrelo por inducción."],
        "vegetta" => &["¡Muy buenas a todos, guapísimos!", "¡Esto lo construyo yo en un momentito!", "¡Willy, mira esto!"],
        "willyrex" => &["¡Vegetta, ven a ver esto!", "Yo lo vi primero, que conste.", "¡Qué épico, tío!"],
        "chillguy" => &["Todo bien.", "Es que soy un chico chill.", "*se mete las manos a las bolsas*", "Relax."],
        _ => &[],
    }
}

/// Los que no hablan: dicen su sonido y GLADAS traduce.
fn sonido_propio(id: &str) -> Option<&'static str> {
    match id {
        "kirby" => Some("¡Poyo! ¡Poyo poyo!"),
        "pikachu" => Some("¡Pika pika! ¡Pikachu!"),
        "creeper" => Some("Sssssssss..."),
        "gato" => Some("¡AAAAAAAAAA!"),
        "doge" => Some("Wow. Such frase. Mucho decir."),
        "rocky" => Some("♪ ♫ ♪ ♫"),
        "capibara" => Some("*masca una hojita en paz*"),
        "moodeng" => Some("*ÑAM* (muerde el volante)"),
        "pedro" => Some("Pedro, Pedro, Pedro, pe..."),
        "labubu" => Some("Hi hi hi hi hi."),
        "tungtung" => Some("Tung tung tung tung tung tung tung tung tung... ¡SAHUR!"),
        "bananini" => Some("¡Wa wa wa! ¡Bananini!"),
        _ => None,
    }
}

/// Situaciones del modo historia con una replica escrita para cada piloto.
#[derive(Clone, Copy)]
pub enum Replica {
    Cita, Boda, Cartas, Fuente, Luna, Juicio, Tarot, Golpe,
    Cosmos, Vida, Futuro,
}

pub fn replica(id: &str, situacion: Replica) -> &'static str {
    let lineas = match id {
        "chavo" => [
            "¡Le conseguimos una cita, Curiosity! Yo invito las tortas... si alguien me presta para las tortas.",
            "Los declaro rover y esposa. ¡Eso, eso, eso! Ahora sí, que pase el pastel.",
            "¡Voy a ganar estas cartas! ¿El premio es una torta de jamón?",
            "¡Un corazón! Lo guardo en el barril para que no se me pierda.",
            "¿La Luna es de mentiritas? ¡Con razón no vendían tortas en la esquina!",
            "Saturno, yo lo defiendo. ¡Y si mienten, les digo que no me tienen paciencia!",
            "¿Ese futuro incluye una torta de jamón? Porque con hambre no puedo pensar.",
            "¡Ay! ¡Fue sin querer queriendo, profe! No me tiene paciencia.",
            "¿Todo eso cabe en el cielo? Mi barril ya se llenó con dos cobijas.",
            "Tanto tiempo para llegar aquí... ¿y todavía no es hora del recreo?",
            "Si se apagan las estrellas, les prestamos la lámpara de la vecindad.",
        ],
        "miku" => [
            "¡Curiosity, vamos a encontrarte una cita!",
            "¡Los declaro rover y esposa! ¡Que empiece la música!",
            "¡Voy a ganar! ¡Treinta y nueve es mi número de la suerte!",
            "¡Un corazón! ¡Lo voy a poner en mi próximo concierto!",
            "¿Cómo que la llegada? ¿Esto es un set?",
            "¡Tranquilo, Saturno! Si alguien miente, le grito ¡PROTESTO!",
            "¿Las cartas dicen algo de mi próximo concierto?",
            "¡Ay, profe! ¡Me desafinó hasta los ceros y unos!",
            "Todas esas estrellas... el escenario más grande que he visto.",
            "De polvo de estrellas a gente que canta. Me gusta esa evolución.",
            "Entonces hay tiempo para otro concierto. ¡Hasta la última estrella!",
        ],
        "simi" => [
            "Curiosity, le receto una cita y dos vasos de agua. La consulta va por mi cuenta.",
            "Los declaro rover y esposa. ¡Lo mismo, pero con chequeo de pareja!",
            "¡Vamos con las cartas! Si pierdo, les regalo un baile, no una receta.",
            "Un corazón nuevo. ¡Ese sí se cuida con ejercicio y unos pasitos!",
            "¿Esto es un set? Y yo que traje protector solar para toda la producción.",
            "Saturno, vamos a revisar los síntomas de esta mentira. ¡PROTESTO!",
            "El futuro se ve bien, pero igual tómese su agüita.",
            "¡Ay! Eso necesita hielo, profe. Y usted necesita vacaciones.",
            "Todo ese calcio salió de estrellas. La farmacia del universo sí está surtida.",
            "La vida llegó lejos. Ahora falta que todos recuerden tomar agua.",
            "Le quedan años al universo. Vuelva a control antes de que se apague.",
        ],
        "shrek" => [
            "Está bien, Curiosity, te buscamos pareja. Pero la cita se hace lejos de mi pantano.",
            "Los declaro rover y esposa. Si Burro pregunta, yo no organicé la boda.",
            "Voy a ganar y después vuelvo al pantano. ¡Nada de tocar mis cartas!",
            "Un corazón. Los ogros también tenemos uno, debajo de todas las capas.",
            "¿Una Luna falsa? Tiene menos capas que una cebolla de plástico.",
            "Saturno, esa mentira tiene capas. Vamos a pelarlas una por una.",
            "¿Las cartas dicen cuándo me dejan volver tranquilo a mi pantano?",
            "¡Oiga! La regla fuera de mi pantano... y de mi cabeza.",
            "Estrellas, galaxias, planetas... ya dije que todo tiene capas.",
            "Millones de años de evolución para que Burro siga sin callarse.",
            "Cuando el Sol se hinche, nos mudamos a un pantano más lejos.",
        ],
        "steve" => [
            "Curiosity, construyo una mesa para dos. Tú consigue las flores; yo pongo los bloques.",
            "Rover y esposa. Punto de reaparición compartido. Boda construida.",
            "Tengo cartas, tengo mesa. Falta craftear una victoria.",
            "Un corazón extra. Al inventario; esto sirve cuando aparecen esqueletos.",
            "Ese suelo parece construido en creativo. ¿Quién puso los bloques de la Luna?",
            "Saturno, esa declaración no encaja. Voy a desmontarla bloque por bloque.",
            "¿El futuro tiene diamantes? Necesito saber a qué altura minar.",
            "Uf. Perdí medio corazón. Debí traer casco de diamante.",
            "Buen mundo. Mucha piedra, pocas mesas de crafteo.",
            "Primero agua, luego árboles. Ya puedo empezar una base.",
            "Si se acaba este mundo, guardo el inventario y genero otro.",
        ],
        "chilindrina" => [
            "¡Fíjate, Curiosity! Yo te consigo una cita, pero tú pagas los refrescos.",
            "¡Rover y esposa! Yo llevo los anillos. Pero no me los pidan de vuelta.",
            "¡Voy a ganar! Y no vale mirar mis cartas por encima de los lentes.",
            "¡Un corazón para mí! No lo presto. Bueno... un ratito.",
            "¡Fíjate, fíjate! Ese ventilador no es de la Luna. ¡Nos están engañando!",
            "Saturno, yo sé cuándo alguien inventa excusas. ¡Papá me enseñó todas!",
            "¿Mi futuro? Espero que diga que ya no me toca hacer mandados.",
            "¡Ay, mi papá se va a enterar! ...Después de que se me pase.",
            "¡Tantas estrellas! Y ninguna me presta para un helado.",
            "La Tierra aprendió a tener vida antes que el Chavo a hacer la tarea.",
            "¡Todavía falta muchísimo! Entonces sí me da tiempo de salir a jugar.",
        ],
        "doge" => [
            "Wow. Mucho romance. Curiosity tendrá cita. Such cupido.",
            "Muy boda. Tan rover. Wow. Se permite besar.",
            "Such cartas. Muy estrategia. Próxima victoria: Doge.",
            "Wow. Corazón brillante. Mucho tesoro. Guardar con cuidado.",
            "Mucho set. Cero Luna real. Such engaño. Wow.",
            "Muy sospechoso. Saturno inocente. Such protesta.",
            "Wow. Cartas del futuro. Tan misterio. ¿Mucho premio?",
            "¡Au! Muy regla. Poco estudio. Such dolor.",
            "Mucho universo. Tan estrellas. Wow. Very espacio.",
            "Such evolución. Mucho planeta. Al final: perritos.",
            "Tan futuro. Muchas estrellas pendientes. Mucho tiempo para jugar.",
        ],
        "mario" => [
            "¡Let's-a go, Curiosity! Esta vez buscamos una cita, no otro castillo.",
            "¡Wahoo! ¡Rover y esposa! La princesa habría traído un pastel.",
            "¡Esta partida es mía! Una estrella de suerte y la última carta.",
            "¡Un corazón! Parece una vida extra. ¡Wahoo!",
            "¿Un set? ¡Mamma mia! La Luna verdadera está en otro castillo.",
            "¡Saturno, vamos a recuperar esos anillos! Como monedas, pero más grandes.",
            "¿Las cartas dicen dónde está la princesa? Ya revisé todos los castillos.",
            "¡Mamma mia! ¡Eso me quitó el power-up!",
            "¡Tantas estrellas! Con una de esas cruzo toda la galaxia saltando.",
            "Un planeta con agua y hongos. ¡Este nivel me gusta!",
            "Todavía quedan muchos niveles. ¡Let's-a go!",
        ],
        "patricio" => [
            "Curiosity, yo sé de citas. Una vez tuve una con el dentista. No salió bien.",
            "Los declaro... ¿rover y mayonesa? Ah, esposa. Eso.",
            "¡Voy a ganar! ¿Las cartas se comen antes o después de repartir?",
            "Un corazón. Yo pensaba que el mío estaba en la rodilla.",
            "¿Esto es un set? Yo pensé que era la fila del supermercado.",
            "Saturno, seré tu abogado. Primera pregunta: ¿qué es un abogado?",
            "¿Mi futuro sigue debajo de una roca? Es que no quiero mudarme.",
            "¡Ay! ¿La respuesta correcta era no recibir el reglazo?",
            "¿Todas esas estrellas son mis primos? Somos una familia grande.",
            "¿Antes de los peces no había mariscos? Qué época tan difícil.",
            "¿Cuántos millones de años faltan para el almuerzo?",
        ],
        "creeper" => [
            "Sss... (GLADAS: propone una cita. Sin explosiones románticas.)",
            "Sssss. (GLADAS: los declara rover y esposa. Nadie encienda las velas.)",
            "Sss-ss. (GLADAS: va por la victoria. El mazo sigue intacto.)",
            "Sss... (GLADAS: guardará el corazón lejos de la pólvora.)",
            "Sssss? (GLADAS: sospecha que esta Luna fue construida en creativo.)",
            "SSS. (GLADAS: protesta. Le aconsejo al juez mantener la calma.)",
            "Sss? (GLADAS: pregunta si su futuro tiene mecha corta.)",
            "¡SSS! (GLADAS: le dolió. No lo hagan explotar de vergüenza.)",
            "Sssss... (GLADAS: insiste en que esa explosión no fue suya.)",
            "Sss. (GLADAS: aprecia que la vida sobreviviera a tantas explosiones.)",
            "Sss... (GLADAS: promete no adelantar el final del universo.)",
        ],
        "kirby" => [
            "¡Poyo! (GLADAS: ayudará a Curiosity. Pregunta si la cita incluye comida.)",
            "¡Poyo poyo! (GLADAS: rover y esposa. Ahora exige el pastel.)",
            "¡Poyo! (GLADAS: quiere ganar sin tragarse las cartas.)",
            "¡Poyooo! (GLADAS: le encanta el corazón. Esta vez no se lo va a comer.)",
            "¿Poyo? (GLADAS: dice que esa Luna sabe a cartón.)",
            "¡POYO! (GLADAS: objeción. La mentira no le pasa ni con agua.)",
            "¿Poyo poyo? (GLADAS: pregunta si ve un bufé en su futuro.)",
            "¡Poy! (GLADAS: la regla no era un aperitivo.)",
            "¡Poyo! (GLADAS: las estrellas se ven deliciosas. Le dije que no.)",
            "Poyo... (GLADAS: pregunta en qué capítulo inventaron el pastel.)",
            "¡Poyo! (GLADAS: hay tiempo para una merienda antes del final.)",
        ],
        "gato" => [
            "¡AAAA! (GLADAS: acepta ayudar con la cita. No garantiza hablar bajito.)",
            "¡AAAAAAAA! (GLADAS: los declara rover y esposa. Fue el brindis.)",
            "¡AAAAAA! (GLADAS: está anunciando que va a ganar.)",
            "¡AAAH! (GLADAS: le gusta el corazón. Ese fue el grito de agradecimiento.)",
            "¿AAAA? (GLADAS: encontró un set. Ahora entiende el cable del micrófono.)",
            "¡AAAAAAAA! (GLADAS: protesta. El micrófono solicita vacaciones.)",
            "¿AAAAA? (GLADAS: quiere saber si mañana también podrá gritar.)",
            "¡AAAAAAAAAAAA! (GLADAS: ese sí fue de dolor.)",
            "¡AAAA! (GLADAS: saluda al universo. Por suerte el vacío no lleva sonido.)",
            "¡AAAH! (GLADAS: celebra que la evolución haya inventado las cuerdas vocales.)",
            "¡AAAA! (GLADAS: promete gritar hasta que se apague la última estrella.)",
        ],
        "pikachu" => [
            "¡Pika pika! (GLADAS: ayudará a que haya chispa entre los rovers.)",
            "¡Pikachu! (GLADAS: rover y esposa. Beso sin descarga, por favor.)",
            "¡Pika! (GLADAS: va a ganar. Cargar el mazo de electricidad sería trampa.)",
            "¡Pika pi! (GLADAS: guardará el corazón junto a su batería.)",
            "¿Pika? (GLADAS: detectó enchufes. Definitivamente es un set.)",
            "¡Pikachu! (GLADAS: protesta. Las pruebas son electrizantes.)",
            "¿Pika pika? (GLADAS: pregunta si su futuro tiene cargador.)",
            "¡Pika! (GLADAS: recibió un reglazo. La madera no conduce la venganza.)",
            "¡Pika! (GLADAS: todas esas estrellas tienen muchísima energía.)",
            "¡Pika pi! (GLADAS: la Tierra necesitó vida antes de tener Pokémon.)",
            "¡Pikachu! (GLADAS: intentará recargar el Sol. Le dije que no alcanza.)",
        ],
        "chapulin" => [
            "¡No contaban con mi astucia! Curiosity, mis antenitas detectan una cita.",
            "¡Los declaro rover y esposa! Todos mis movimientos nupciales están calculados.",
            "¡Síganme los buenos! Bueno, mis cartas. Las suyas no las veo.",
            "¡Un corazón! Lo guardaré junto a las pastillas de chiquitolina.",
            "¡Lo sospeché desde un principio! Mis antenitas detectaron el set.",
            "¡Que no panda el cúnico, Saturno! Mi astucia defenderá tus anillos.",
            "¿El futuro? Lo sospechaba desde un principio... pero cuéntemelo otra vez.",
            "¡Que no panda el cúnico! El reglazo estaba fríamente calculado... por el profe.",
            "¡Mis antenitas de vinil detectan actividad cósmica! Muchísima.",
            "La vida siempre encontró cómo seguir. ¡No contaban con su astucia!",
            "Si el universo necesita ayuda, ¡síganme los buenos!",
        ],
        "knuckles" => [
            "*clic clic* Curiosity, conozco el camino hacia una cita. ¡Sígueme!",
            "¡Rover y esposa! Encontraron el camino. *clic clic*",
            "*clic clic* Conozco el camino a la victoria. Pasa esas cartas.",
            "¡Un corazón para la reina! Esto sí merece seguir el camino.",
            "Esta no es la Luna. Nos enseñaron el camino equivocado. *clic*",
            "Saturno, las pruebas muestran el camino. ¡Que hable el testigo!",
            "¿Esas cartas conocen el camino? Porque yo todavía busco a la reina.",
            "¡Ay! Ese no era el camino, profe. *clic triste*",
            "Tantas galaxias y todavía nadie me dice dónde queda la reina.",
            "La vida recorrió un camino larguísimo. Respeto. *clic clic*",
            "Queda universo por recorrer. ¡Hermanos, por este camino!",
        ],
        "koch" => [
            "Curiosity, una cita es un problema de búsqueda. Vamos a recorrer el árbol de candidatas.",
            "Los declaro rover y esposa. Queda demostrado. Q.E.D.",
            "Las cartas son un lenguaje regular. Ya armé el autómata. Voy a ganar.",
            "Un corazón. Lo agrego al conjunto de cosas aceptadas. Estado final.",
            "Esta Luna es falsa. Su gramática no genera estrellas. Contraejemplo.",
            "Saturno, una mentira es una cadena que el autómata rechaza. Vamos a rechazarla.",
            "¿El futuro? Es indecidible. Lo siento, Madame: Turing lo demostró.",
            "¡Ay! ...Bueno, ahora entiendo cómo se siente del otro lado de la regla.",
            "13,800 millones de años. Y todavía no termina de correr. Típico de un algoritmo sin caso base.",
            "La vida: un sistema que se copia a sí mismo. Von Neumann estaría orgulloso.",
            "El universo se va a apagar. Igual el examen es el jueves.",
        ],
        "vegetta" => [
            "¡Curiosity, guapísimo! ¡Te vamos a conseguir una cita épica, ya verás!",
            "¡Los declaro rover y esposa! ¡Y les construyo una casa en Marte de regalo!",
            "¡Vamos a por las cartas! Willy siempre me gana, pero hoy no, ¿eh?",
            "¡Un corazón! ¡Esto en Minecraft valdría un montón!",
            "¿Un set de grabación? ¡Esto lo construí mejor yo en Karmaland!",
            "Saturno, tranquilo. ¡Si mienten, les cae todo el peso de Planeta Vegetta!",
            "¿Las cartas dicen si Willy me va a devolver los diamantes?",
            "¡Au! ¡Eso me lo apunto para el próximo video, profe!",
            "¡Cuántas estrellas, tío! Aquí cabe un mundo nuevo enterito.",
            "De polvo de estrellas a gente que construye casas. ¡Qué progreso!",
            "Si se apagan las estrellas, ponemos antorchas. Solucionado.",
        ],
        "willyrex" => [
            "Curiosity, confía en mí: yo de citas sé. Bueno, de citas en videojuegos.",
            "¡Rover y esposa! Vegetta, ¿por qué lloras? ...No, yo tampoco lloro.",
            "Estas cartas las gano yo. Vegetta siempre dice que no, y siempre pierde.",
            "Un corazón. Yo lo vi primero, que conste en el video.",
            "¿Esto es un set? Ya decía yo que la Luna tenía mucha luz de estudio.",
            "Saturno, yo te defiendo. He visto todos los juegos de abogados. Dos veces.",
            "¿Mi futuro? Que sea con más suscriptores que Vegetta, por favor.",
            "¡Au! Profe, eso es daño crítico.",
            "Qué épico, tío. El universo entero y ningún lag.",
            "Tanto tiempo evolucionando para terminar jugando videojuegos. Lo apruebo.",
            "Que se apaguen las estrellas, que yo tengo la pantalla prendida.",
        ],
        // Rocky habla con acordes y GLADAS traduce: frases cortas, repite para enfatizar, termina
        // las preguntas con "¿pregunta?" y todo le parece "amaze".
        "rocky" => [
            "♪♫ (Traducción: «Rover solo. Rover triste. Rocky ingeniero. Rocky arregla cita. Fácil fácil fácil.»)",
            "♪♫♪ (Traducción: «¡Rover y esposa! ¡Amaze amaze amaze! ¿Ahora comen, pregunta?»)",
            "♪♪ (Traducción: «Juego de cartas es ciencia de probabilidad. Rocky gana. Rocky siempre gana. Bueno, a veces.»)",
            "♫ (Traducción: «Corazón brilla. En Erid no hay luz, pero Rocky siente que brilla. Amaze.»)",
            "♪♫♪ (Traducción: «¿Luna es falsa, pregunta? Rocky oye eco de estudio. Mal mal mal. Mentira.»)",
            "♫♪ (Traducción: «Saturno tranquilo. Rocky ayuda. Humanos y eridianos, mismo equipo.»)",
            "♪ (Traducción: «Futuro no se sabe. Se calcula. Rocky trae calculadora.»)",
            "♫♫ (Traducción: «¡Au! Profe pega fuerte. Caparazón aguanta. Profe no tan amaze.»)",
            "♪♫♪♫ (Traducción: «Tantas estrellas. Todas con astrófago, pregunta? Rocky espera que no.»)",
            "♫ (Traducción: «Vida empieza chiquita. Vida termina construyendo naves. Amaze.»)",
            "♪♪♫ (Traducción: «Si estrellas se apagan, Rocky y amigo las arreglan. Puño mi choque.»)",
        ],
        // Cita, Boda, Cartas, Fuente, Luna, Juicio, Tarot, Golpe, Cosmos, Vida, Futuro.
        "capibara" => [
            "*masca su hojita* (Traducción: «Curiosity, relájate. El amor llega solo. Como el agua al río.»)",
            "*se mete al charco de la boda* (Traducción: «Los declaro rover y esposa. Ahora todos a la poza.»)",
            "*pone una carta y cierra los ojos* (Traducción: «Gane o pierda, igual voy a estar en paz.»)",
            "*se pone el corazón en la cabeza, junto a la mandarina* (Traducción: «Ahora traigo dos cosas en la cabeza. Bueno, tres.»)",
            "*bosteza* (Traducción: «¿La Luna es falsa? Igual se ve bonita. Me quedo.»)",
            "*se sienta junto a Saturno* (Traducción: «Tranquilo, amigo. Todos le caemos bien a alguien. Yo le caigo bien a todos.»)",
            "*se duerme durante la lectura* (Traducción: «El futuro llega igual. Mejor descansada.»)",
            "*ni se inmuta* (Traducción: «¿Eso fue un reglazo? Estoy muy en paz para sentirlo.»)",
            "*mira las estrellas, masticando* (Traducción: «Tanto universo y ni una poza. Le falta.»)",
            "*asiente despacito* (Traducción: «De bacterias a capibaras. Lo mejor llegó al final, obvio.»)",
            "*se acomoda la mandarina* (Traducción: «Si se apagan las estrellas, me echo una siesta.»)",
        ],
        "moodeng" => [
            "*ÑAM* (Traducción: «Curiosity necesita cita. Yo le muerdo a los que no sirvan.»)",
            "*ÑAM ÑAM* (Traducción: «¡Los declaro rover y esposa! ¿Hay pastel? Me lo como.»)",
            "*muerde la baraja* (Traducción: «Ya no hay cartas. Gané.»)",
            "*intenta morder el corazón* (Traducción: «No se deja. Lo respeto. Un poquito.»)",
            "*muerde la escenografía* (Traducción: «Cartón. La Luna es de cartón. Sabe a cartón.»)",
            "*se sienta encima del abogado* (Traducción: «Objeción denegada. Pesa más el hipopótamo.»)",
            "*muerde las cartas del tarot* (Traducción: «Ahora el futuro tiene dientes.»)",
            "*le muerde la regla al profe* (Traducción: «Empate.»)",
            "*bufa al cielo* (Traducción: «Tantas estrellas y ninguna se deja morder.»)",
            "*se revuelca en el lodo* (Traducción: «La vida empezó en el lodo. Aquí sigo, orgullosa.»)",
            "*bosteza enorme* (Traducción: «Si se apaga el universo, yo sigo siendo famosa.»)",
        ],
        "pedro" => [
            "Pedro, Pedro, Pe... (Traducción: «Curiosity, la clave es el baile. Nadie se resiste a dar vueltas.»)",
            "*da vueltas en la boda* (Traducción: «Los declaro rover y esposa. Pista de baile abierta.»)",
            "*baila sobre la mesa* (Traducción: «No sé las reglas, pero las bailo.»)",
            "*se roba el corazón y lo esconde* (Traducción: «Era basura. Todo lo bonito es basura para un mapache.»)",
            "*hurga en la utilería* (Traducción: «La Luna es de utilería. Hay muchas cosas buenas en esta basura.»)",
            "*gira frente al jurado* (Traducción: «Saturno es inocente. Lo baila mi conciencia.»)",
            "*se come una carta* (Traducción: «El futuro sabe a papel. Buen papel.»)",
            "*sigue bailando con el chichón* (Traducción: «Pedro, Pedro, Pedro, ¡AY!, pe.»)",
            "*da vueltas en gravedad cero* (Traducción: «Aquí se gira sin parar. Es mi casa.»)",
            "*baila* (Traducción: «De células a mapaches que bailan. Valió la pena.»)",
            "*sigue girando* (Traducción: «Cuando se apague todo, que me encuentren bailando.»)",
        ],
        "chillguy" => [
            "Curiosity, relax. Si la cita sale mal, igual eres un rover con mucho aura. Todo bien.",
            "Los declaro rover y esposa. Qué chill. Ojalá a todos les vaya así.",
            "Juego tranquilo. Si gano, bien; si no, también. Es que soy un chico chill.",
            "Un corazón. Lo guardo en la bolsa del suéter, junto con mis manos.",
            "¿La Luna es falsa? Bueno. Sigue siendo una buena vista. Relax.",
            "Saturno, tranquilo. Nadie se estresa en este tribunal. Bueno, el juez sí.",
            "Mi futuro: seguir chill. Ya me lo sé, Madame.",
            "Ay. Bueno. Pasa. Todo bien, profe.",
            "Tantas galaxias y yo aquí, con las manos en las bolsas. Perfecto.",
            "Millones de años de evolución para llegar a un perrito en suéter. Lo aceptamos.",
            "Si se apagan las estrellas, me pongo otro suéter. Relax.",
        ],
        "labubu" => [
            "Hi hi hi. (Traducción: «Curiosity, si la cita se porta mal, yo me encargo. Con mis nueve dientes.»)",
            "Hi hi hi hi. (Traducción: «Rover y esposa. Qué tierno. Demasiado tierno. Sospechoso.»)",
            "Hi hi. (Traducción: «Salí de una caja sorpresa. Las cartas son mi especialidad.»)",
            "Hi hi hi. (Traducción: «Un corazón. Lo voy a guardar en mi caja. Junto con otras cosas.»)",
            "Hi hi hi hi hi. (Traducción: «La Luna es falsa. Como la mitad de los Labubus. Yo soy original.»)",
            "Hi. (Traducción: «Saturno, sonríe. Así. Más dientes.»)",
            "Hi hi hi. (Traducción: «Mi futuro: agotarme en todas las tiendas.»)",
            "...hi. (Traducción: «Eso dolió. No se me olvida, profe.»)",
            "Hi hi hi. (Traducción: «Tantas estrellas. Todas en cajas sorpresa. Quiero abrirlas todas.»)",
            "Hi hi. (Traducción: «De criaturas raras a coleccionables. La evolución tiene buen gusto.»)",
            "Hi hi hi hi. (Traducción: «Cuando todo se apague, yo voy a seguir sonriendo.»)",
        ],
        "tungtung" => [
            "Tung tung tung. (Traducción: «Curiosity. A la cita se llega temprano. Tres toques a la puerta.»)",
            "TUNG. (Traducción: «Rover y esposa. Ya pueden desayunar.»)",
            "Tung tung. (Traducción: «Barajo con el bate. Nadie hace trampa.»)",
            "Tung. (Traducción: «Un corazón. Me lo pongo en el tronco. Ahora tengo sentimientos.»)",
            "Tung tung tung tung. (Traducción: «Toqué la Luna tres veces. Suena hueca. Es de mentira.»)",
            "TUNG TUNG TUNG. (Traducción: «Orden en la sala. Traigo mi propio mazo.»)",
            "Tung tung. (Traducción: «Mi futuro: tocar puertas. Siempre.»)",
            "...tung. (Traducción: «Usted me pega con una regla. Yo traigo bate. Pensémoslo bien, profe.»)",
            "Tung tung tung. (Traducción: «Tantas estrellas. Tantas puertas por tocar.»)",
            "Tung. (Traducción: «Antes era árbol. Ahora tengo ojos. Progreso.»)",
            "Tung tung tung... ¡SAHUR! (Traducción: «Aunque se apague el Sol, alguien tiene que despertar a todos.»)",
        ],
        "bananini" => [
            "¡Wa wa wa! (Traducción: «Curiosity, invítala a comer banana. Funciona siempre.»)",
            "¡Bananini! (Traducción: «Los declaro rover y esposa. Arroz no tengo, tiro cáscaras.»)",
            "¡Wa wa! (Traducción: «Gané el pase entero. Unas cartitas no me asustan.»)",
            "¡Wa! (Traducción: «Un corazón. Más dulce que una banana. Casi.»)",
            "¡Wa wa wa wa! (Traducción: «La Luna es falsa: no tiene ni una mata de banano.»)",
            "¡Bananini! (Traducción: «Saturno, cuéntame todo. Pero pélalo despacito.»)",
            "¡Wa wa! (Traducción: «Mi futuro: más bananas. El tarot lo confirma.»)",
            "¡Wa...! (Traducción: «Ay. La cáscara amortiguó. Más o menos.»)",
            "¡Wa wa wa! (Traducción: «Tanto espacio y ni un supermercado.»)",
            "¡Bananini! (Traducción: «Primero fue la banana, luego el mono. O al revés. La ciencia no se decide.»)",
            "¡Wa wa! (Traducción: «Si se apaga todo, yo tengo potasio para rato.»)",
        ],
        _ => return replica("chavo", situacion),
    };
    lineas[situacion as usize]
}

/// Lo que dice el personaje `id` cuando le toca la linea `texto` (escrita para Miku): los que no
/// hablan, su sonido con la traduccion; los demas tienen replicas propias y muletillas.
/// Siempre igual para la misma linea (no cambia cada vez que se dibuja).
pub fn como_lo_dice(id: &str, texto: &str) -> Option<String> {
    if texto.is_empty() || id == "miku" { return None; }
    let situacion = match texto {
        t if t.starts_with("¡No puede ser! ¡Te vamos a conseguir una cita!") => Some(Replica::Cita),
        t if t.starts_with("Por el poder que me da ser idol") => Some(Replica::Boda),
        t if t.starts_with("¡Yo voy a ganar! Tengo 39 razones") => Some(Replica::Cartas),
        t if t.starts_with("¡Un corazón! ¡Lo voy a poner") => Some(Replica::Fuente),
        t if t.starts_with("¿Cómo que la ") => Some(Replica::Luna),
        t if t.starts_with("¡Tranquilo, Saturno!") => Some(Replica::Juicio),
        t if t.starts_with("O sea... ¿que no le escriba") || t.starts_with("¿Y mi próximo concierto?") || t.starts_with("Me gustó la parte de mi futuro.") => Some(Replica::Tarot),
        _ => None,
    };
    if let Some(situacion) = situacion { return Some(replica(id, situacion).into()); }
    // Las pruebas del juicio y las instrucciones conservan toda su informacion.
    if let Some(sonido) = sonido_propio(id) {
        return Some(format!("{sonido} (Traducción de GLADAS: «{texto}»)"));
    }
    let m = muletillas(id);
    if m.is_empty() { return None; }
    let suerte = texto.chars().fold(7u32, |h, c| h.wrapping_mul(31).wrapping_add(c as u32));
    Some(format!("{texto} {}", m[suerte as usize % m.len()]))
}

/// Los que no salen en el pase: se ganan con un logro (el profe, aprobando su examen con 5 de 5;
/// Vegetta y Willyrex, ganando la carrera de diamantes en Planeta Vegetta).
pub fn por_logro(id: &str) -> Option<&'static str> {
    match id {
        "koch" => Some("examen_5"),
        "vegetta" | "willyrex" => Some("vegetta_carrera"),
        _ => None,
    }
}

/// Como se gana uno que no sale en el pase (para el casillero).
pub fn como_se_gana(id: &str) -> Option<&'static str> {
    match id {
        "koch" => Some("Entregue los tres proyectos de Koch al 100% a las 23:59 (en la Tierra), o cómprelo en la tienda."),
        "vegetta" | "willyrex" => Some("Gánele a Willyrex un minijuego del servidor de Planeta Vegetta (parkour, TNT Run o minería), o cómprelo en la tienda."),
        "rocky" => Some("Cómprelo en la tienda. (Antes vaya a saludarlo: está en la Blip-A, junto a Venus.)"),
        "capibara" | "moodeng" | "pedro" | "chillguy" | "labubu" | "tungtung" => Some("Cómprelo en la tienda, en la pestaña de personajes."),
        _ => None,
    }
}

/// Lo que cuesta en la tienda (en notas). Solo se venden los extras (los que no son del pase ni
/// de fabrica); los del pase son SOLO del pase, y para esos no hay precio.
pub fn precio(id: &str) -> Option<u32> {
    match id {
        "koch" => Some(400),
        "vegetta" | "willyrex" => Some(300),
        "rocky" => Some(350),
        "capibara" => Some(220),
        "moodeng" => Some(240),
        "pedro" => Some(260),
        "chillguy" => Some(280),
        "labubu" => Some(320),
        "tungtung" => Some(300),
        _ => None,
    }
}

/// Los que vienen de fabrica (los demas salen en el pase).
pub const GRATIS: [&str; 2] = ["chavo", "miku"];

/// El indice del personaje con ese id (el primero si no existe).
pub fn indice(id: &str) -> usize {
    PERSONAJES.iter().position(|p| p.id == id).unwrap_or(0)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn las_escenas_tienen_dialogos_propios_sin_perder_las_pruebas() {
        let boda = "Por el poder que me da ser idol del espacio... ¡los declaro rover y esposa! ¡Puede besar a la rover!";
        let prueba = "¡PROTESTO! El anillo B es más del 90% hielo de agua.";
        let mut replicas = std::collections::BTreeSet::new();
        for p in &PERSONAJES {
            let texto = como_lo_dice(p.id, boda).unwrap_or_else(|| boda.into());
            assert!(replicas.insert(texto.clone()), "{} repite dialogo", p.id);
            if p.id != "miku" { assert!(!texto.contains("idol")); }
            assert!(como_lo_dice(p.id, prueba).unwrap_or_else(|| prueba.into()).contains(prueba));
            assert_eq!(como_lo_dice(p.id, ""), None);
        }
    }

    #[test]
    fn cada_uno_habla_a_su_manera() {
        let linea = "¡Esa declaración es mentira!";
        assert_eq!(como_lo_dice("miku", linea), None, "Miku dice sus lineas tal cual");
        let kirby = como_lo_dice("kirby", linea).unwrap();
        assert!(kirby.starts_with("¡Poyo!") && kirby.contains(linea), "Kirby: su sonido y la traduccion");
        // Siempre lo mismo para la misma linea (no cambia cada cuadro).
        assert_eq!(como_lo_dice("chavo", linea), como_lo_dice("chavo", linea));
        // Tambien las lineas sin una replica especial tienen la voz del personaje.
        let con = (0..40).filter(|k| como_lo_dice("shrek", &format!("linea {k}")).is_some()).count();
        assert_eq!(con, 40, "cada linea del protagonista debe conservar su personalidad");
    }

    #[test]
    fn los_ids_no_se_repiten_y_todos_salen_en_el_pase() {
        let mut ids: Vec<&str> = PERSONAJES.iter().map(|p| p.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), PERSONAJES.len());
        for p in &PERSONAJES {
            let en_pase = crate::progreso::PASE.iter().any(|r| matches!(r, crate::progreso::Premio::Personaje(id) if *id == p.id));
            let maneras = GRATIS.contains(&p.id) as u32 + en_pase as u32 + por_logro(p.id).is_some() as u32;
            // Gratis, del pase o por un logro (solo una), o si no, que se pueda comprar (Rocky).
            assert!(maneras == 1 || (maneras == 0 && precio(p.id).is_some()), "{} tiene que ser gratis, salir en el pase, ganarse con un logro o venderse", p.id);
            // Los del pase y los de fabrica no se venden.
            assert!(!(en_pase || GRATIS.contains(&p.id)) || precio(p.id).is_none(), "{} es del pase o de fabrica y tiene precio", p.id);
        }
    }
}
