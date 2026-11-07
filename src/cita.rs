//! Aterrizaje en Marte: Doki Doki Marte, un simulador de citas.
//!
//! Curiosity lleva solito en Marte desde 2012 (en 2013 se canto feliz
//! cumpleanos el solo, con las vibraciones de su taladro). Miku hace de
//! celestina: se elige a una de las tres solteras del planeta (Ingenuity,
//! InSight o Perseverance, que existen de verdad) y en la cita se contesta
//! por Curiosity. Cada respuesta sube o baja la afinidad (los corazones) y
//! segun como quede hay tres finales.
//!
//! Para que cada cita salga distinta: antes de empezar se escoge un REGALO (a
//! cada una le gusta algo distinto: a InSight que le soplen los paneles, a
//! Perseverance una piedra para sus tubitos...), las preguntas salen al azar de
//! un banco de siete, y a media cita casi siempre pasa un MOMENTO sorpresa (una
//! tormenta de polvo, Fobos cruzando el cielo, Houston llamando, Miku que se
//! ofrece a cantar o el atardecer azul de Marte). Y si Curiosity le sale bien a
//! dos en la misma visita... llega la ex, se ponen de acuerdo y le dan una
//! GOLPIZA: game over (Curiosity entra en modo seguro).
//!
//! Todo esta en coordenadas de la escena: el piso en y = 0, el mantel del
//! picnic en el centro y la camara del lado de -Z.

use crate::vida::{Tam, Tono, Vida};
use crate::entrada::Entrada;
use crate::matematica::{Mat4, Vec3};
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;
use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Final {
    /// Todo perfecto: boda en Marte, con Miku de oficiante.
    Boda,
    Exito,
    Amigos,
    Solo,
    /// Salio bien, pero ya habia salido bien con otra: llega la ex (indice en `CANDIDATAS`).
    Drama(usize),
}

/// Como se porto Curiosity en la cita (no solo cuanto sumo): cambia el final.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Actuacion {
    /// Las tres respuestas perfectas.
    Perfecta,
    /// Arranco mal y termino encantador: a ella le gusta que aprenda.
    Remonto,
    /// Arranco perfecto y termino metiendo la pata: lo ultimo es lo que se recuerda.
    SeDesinflo,
    /// Alguna respuesta le cayo mal (y no remonto).
    MetioLaPata,
    Normal,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fase {
    Intro(usize),
    Elegir,
    Saludo,
    Pregunta(usize),
    /// (pregunta, opcion): lo que contesta Curiosity.
    Respuesta(usize, usize),
    /// (pregunta, opcion): como reacciona ella.
    Reaccion(usize, usize),
    Despedida,
    Final(usize),
    /// Antes de la primera pregunta: Miku pregunta que le trajo (se escoge uno de `REGALOS`).
    Regalo,
    /// Como reacciona ella al regalo.
    RegaloReaccion(usize),
    /// La ex y la cita se ponen de acuerdo y le caen a golpes a Curiosity.
    Golpiza,
    /// Curiosity en modo seguro: se reintenta (las exes se calman) o se despega.
    GameOver,
}

pub struct Pregunta {
    pregunta: &'static str,
    /// (lo que contesta Curiosity, puntos de afinidad, como reacciona ella)
    respuestas: [(&'static str, i32, &'static str); 3],
}

pub struct Candidata {
    pub nombre: &'static str,
    descripcion: &'static str,
    /// Dos saludos: en cada cita sale uno.
    saludos: [&'static str; 2],
    /// El banco de preguntas: en cada cita salen tres al azar.
    preguntas: [Pregunta; 7],
    /// Lo que dice al final, segun como salio la cita.
    exito: &'static str,
    amigos: &'static str,
    solo: &'static str,
    /// Si todo salio perfecto: acepta casarse.
    boda: &'static str,
    /// Lo que dice si llega y cacha a Curiosity con otra.
    celos: &'static str,
    /// Lo que le parece cada regalo de `REGALOS`: (puntos de afinidad, que dice).
    regalos: [(i32, &'static str); 4],
}

/// Lo que se le puede llevar a la cita.
pub const REGALOS: [&str; 4] = ["Una piedra bonita (de Marte, obvio)", "Soplarle el polvo de los paneles", "Una canción de Miku, en vivo", "Tape. Del bueno."];

/// Un momento sorpresa a media cita: quien lo dice (vacio: la candidata), con que voz, y la
/// pregunta (las reacciones las dice la candidata).
pub struct Momento {
    quien: &'static str,
    voz: Voz,
    pub pregunta: Pregunta,
}

pub const TORMENTA: usize = 0;
pub const FOBOS: usize = 1;
pub const HOUSTON: usize = 2;
pub const CANCION: usize = 3;
pub const ATARDECER: usize = 4;

pub const MOMENTOS: [Momento; 5] = [
    Momento {
        quien: "",
        voz: Voz::Robotina,
        pregunta: Pregunta {
            pregunta: "¡Tormenta de polvo! ¡Viene directo hacia nosotros! ¿Qué hacemos?",
            respuestas: [
                ("Te cubro con mi brazo robótico. Mide dos metros. Para algo tenía que servir.", 2, "Ay... nunca nadie me había tapado del polvo. Ni la NASA. Ni mi mamá, que es la NASA."),
                ("¡Corre! Digo... rueda. A 0.14 km/h. Nos alcanza en tres segundos.", 1, "...Bueno, al menos lo intentaste. Lentito, pero lo intentaste."),
                ("Le tomo foto. Va a salir increíble en las historias.", -1, "¿¡FOTO!? ¡Me estoy llenando de polvo y tú subiendo historias!"),
            ],
        },
    },
    Momento {
        quien: "",
        voz: Voz::Robotina,
        pregunta: Pregunta {
            pregunta: "Mira: Fobos está cruzando el cielo. Dicen que si pides un deseo se cumple. ¿Qué pedirías?",
            respuestas: [
                ("Que esta cita no se acabe. Aunque se me acabe la batería.", 2, "...Pedí lo mismo. No te lo iba a decir, pero pedí lo mismo."),
                ("Que la NASA me mande un amigo. ...Ah, ya llegaste tú.", 1, "¿Amiga? Bueno... por algo se empieza."),
                ("Más memoria RAM.", -1, "¿Más RAM? ¿Con Fobos pasando? Romántico como un disco duro."),
            ],
        },
    },
    Momento {
        quien: "HOUSTON",
        voz: Voz::Planeta,
        pregunta: Pregunta {
            pregunta: "Curiosity, aquí Houston. Necesitamos que taladre una piedra. Ahorita. Es urgente. Por la ciencia. Cambio.",
            respuestas: [
                ("Houston, estoy en una cita. Llámenme en 20 minutos, que es lo que tarda la señal.", 2, "¿Le colgaste a la NASA? ¿Por mí? ...Me muero. Me muero de verdad."),
                ("Ksshhh... no se oye... ksshhh... se cortó... ksshhh...", 1, "¡Jajaja! ¡Qué tramposo! Me gustan los rovers rebeldes."),
                ("Voy a taladrar la piedra. La ciencia es primero.", -1, "Ah. Ok. Tú taladra. Yo aquí espero. Sola. Como siempre."),
            ],
        },
    },
    Momento {
        quien: "MIKU",
        voz: Voz::Miku,
        pregunta: Pregunta {
            pregunta: "¿Les canto algo romántico? Tengo 400 canciones. Todas son de puerros, pero románticas.",
            respuestas: [
                ("Sí, Miku: la nuestra. ...Todavía no tenemos. Invéntala.", 2, "¿Una canción NUESTRA? Ay, me derrito. A 60 grados bajo cero, pero me derrito."),
                ("Una bajito, que se asusta el sismógrafo de InSight.", 1, "Qué considerado. Hasta con las vecinas. Me gusta eso."),
                ("No, gracias. Prefiero el silencio de Marte.", -1, "...Ah. Ok. Silencio. Como en las últimas doce horas. Divertidísimo."),
            ],
        },
    },
    Momento {
        quien: "",
        voz: Voz::Robotina,
        pregunta: Pregunta {
            pregunta: "¿Sabías que en Marte los atardeceres son azules? Míralo... ¿qué piensas?",
            respuestas: [
                ("Que es lo segundo más bonito que he visto hoy.", 2, "...¿Y lo primero? No, no digas nada. Ya sé. Ay."),
                ("Es el polvo fino: dispersa el rojo y deja pasar el azul.", 1, "Nerd. Pero un nerd con datos correctos. Me sirve."),
                ("Que en la Tierra son más bonitos.", -1, "Pues vete a la Tierra. ...Ah, no puedes. Mal ahí."),
            ],
        },
    },
];

const IA: &str = ui::NOMBRE_IA;
const CURIOSITY: &str = "CURIOSITY";
const MIKU: &str = "MIKU";

const INTRO: [(&str, &str, Voz); 5] = [
    (IA, "Marte. Aquí vive Curiosity. Llegó en 2012. Amigos: cero. Seguidores: la NASA, por obligación.", Voz::Ia),
    (CURIOSITY, "Ah... hola. No esperaba visitas. Nunca espero visitas. Así nunca me decepciono.", Voz::Rover),
    (CURIOSITY, "En 2013 me canté feliz cumpleaños yo solito. Con el taladro. La NASA dijo que era \"para probar el instrumento\".", Voz::Rover),
    (MIKU, "¡No puede ser! ¡Te vamos a conseguir una cita! Yo soy buenísima para esto. Creo. Nunca lo he hecho.", Voz::Miku),
    (IA, "Bienvenidos a DOKI DOKI MARTE. Hay tres solteras en el planeta. Literal: solo hay tres robots.", Voz::Ia),
];

const ELEGIR: &str = "Elija a la candidata. Las tres son robots. Las tres tienen cámaras. Ninguna tiene filtro.";

pub const CANDIDATAS: [Candidata; 4] = [
    Candidata {
        nombre: "INGENUITY",
        descripcion: "Helicóptero. Le prometieron 5 vuelos e hizo 72. Hiperactiva. Tiene una hélice quebrada desde 2024.",
        saludos: [
            "¡Hola hola hola! ¿Tú eres Curiosity? ¡Te he visto desde arriba! Bueno, antes. Ya no vuelo.",
            "¡Llegué! Corriendo. Bueno, caminando. Bueno, me trajeron. ¡Hola, Curiosity!",
        ],
        preguntas: [
            Pregunta {
                pregunta: "¡Volé 72 veces! Me habían prometido 5. ¿Tú qué haces por diversión?",
                respuestas: [
                    ("Te vi volar desde el cráter. Se me bajó la batería de la emoción.", 2, "¿¡En serio!? Nadie me había visto volar con cariño. Solo con cámaras. Bueno, tú también tienes cámaras. ¡Igual cuenta!"),
                    ("Ando a 0.14 km/h. Despacito. Como la canción.", 1, "¡Ja! Yo en un segundo hago lo que tú en un día. Pero me caes bien, lentito."),
                    ("Taladro piedras. Y les tomo fotos. A las piedras.", -1, "Ah. Piedras. Qué... rocoso. ¿Y algo más? ¿No? Ok."),
                ],
            },
            Pregunta {
                pregunta: "En 2024 se me quebró una hélice y ya no puedo volar. ¿Igual te gusto?",
                respuestas: [
                    ("¿Y si te la pego con tape? Así arreglamos todo en Guate.", 1, "¿Tape? ¿A una hélice de millones de dólares? ...Bueno, la intención cuenta."),
                    ("Pues... ¿no tenías garantía?", -1, "¿GARANTÍA? ¡La garantía era de 5 vuelos! ¡Hice 72! ¡Me la gané!"),
                    ("Me gustas aunque no vueles. Yo nunca he volado y mírame.", 2, "Ay... Eso fue lo más lindo que me han dicho en otro planeta."),
                ],
            },
            Pregunta {
                pregunta: "Perseverance me trajo a Marte en su panza. ¿Te molesta que sea mi mejor amiga?",
                respuestas: [
                    ("Mientras no vueles con ella...", -1, "Ya no vuelo con nadie. Por la hélice. Te lo acabo de contar. ¿Me estabas oyendo?"),
                    ("Me alegra que no viniste sola. Yo vine solo y no se lo deseo a nadie.", 2, "Ya no vas a estar solo. Bueno, si quieres. No te presiono. ¡Sí te presiono!"),
                    ("Para nada. Te traje un regalo: una piedra. Es lo que hay aquí.", 1, "¡Una piedra! ...Es igualita a las otras 80 millones. Pero esta me la diste tú."),
                ],
            },
            Pregunta {
                pregunta: "Si pudiéramos volar juntos una sola vez, ¿a dónde me llevarías?",
                respuestas: [
                    ("A la Tierra, a comer pupusas.", 1, "No sé qué es una pupusa, pero lo dijiste con tanto amor que ya quiero una."),
                    ("Al Monte Olimpo. El volcán más alto del sistema solar. Como lo que siento.", 2, "¿¡El más alto!? Ay, qué cursi. ...Me encantó. No le digas a nadie que me encantó."),
                    ("Yo no vuelo. Te espero aquí. Sentado. Como siempre.", -1, "...Eso es muy deprimente, Curiosity. Hasta para un rover."),
                ],
            },
            Pregunta {
                pregunta: "Rapidito, que me queda poca batería: ¿qué es lo que más te gusta de mí?",
                respuestas: [
                    ("Tus paneles solares. Muy... eficientes.", -1, "¿Mis paneles? Wow. Qué romántico. Como una auditoría."),
                    ("Que eres chiquita y rápida. Como un colibrí con wifi.", 1, "¡Colibrí con wifi! Me lo voy a poner de bio."),
                    ("Que nunca te rendiste. Ni con la hélice rota.", 2, "...Ay. Se me empañó la cámara. Es el polvo. Seguro es el polvo."),
                ],
            },
            Pregunta {
                pregunta: "¿Cuál es tu récord de velocidad? El mío es 10 metros por segundo. Volando. Antes.",
                respuestas: [
                    ("152 metros por hora. Cuesta abajo. Con viento a favor.", 1, "¡Ja! Yo hago eso en un parpadeo. Pero me da ternura tu esfuerzo."),
                    ("No importa la velocidad. Importa con quién vas.", 2, "Ay... eso suena a canción. ¿Es de Miku? Igual me gustó."),
                    ("Yo prefiero la seguridad. Llevo doce años sin chocar.", -1, "Claro. Porque te mueves a la velocidad de una planta. Aburrido."),
                ],
            },
            Pregunta {
                pregunta: "Mis papás son de JPL, en California. ¿Les vas a pedir permiso para salir conmigo?",
                respuestas: [
                    ("Ya les pedí. Hace veinte minutos. Es lo que tarda la señal en llegar.", 2, "¿¡En serio!? ¡Qué decente! Ningún rover me había tratado así."),
                    ("Les mando un correo. Con copia a la NASA. Formal.", 1, "Muy formal. Mis papás van a pensar que es spam. Pero bueno."),
                    ("¿Tus papás? Yo vine a ver a Perseverance.", -1, "¿¡QUÉ!? ¡Pues vete con ella! ¡Y que te lleve en su panza!"),
                ],
            },
        ],
        exito: "¡Sí! ¡Vamos a pasear! Yo voy saltando y tú me sigues a 0.14 km/h. Llegamos en marzo.",
        amigos: "Me caes súper bien... como compañero de misión. ¿Choque de ruedas?",
        solo: "Me tengo que ir. Tengo que... volar. Ah no. Bueno, me voy caminando. Adiós.",
        boda: "¡SÍ ME CASO! ¡Yo llevo los anillos! ¡Volando! ...Bueno, me los lleva Perseverance.",
        celos: "¿¡CURIOSITY!? ¡Te vi desde arriba! ¡Yo vuelo! ¡Bueno, volaba! ¡Pero te vi!",
        regalos: [
            (-1, "¿Una piedra? Vivo en un planeta HECHO de piedras, Curiosity. Es como regalarle arena a la playa."),
            (0, "Ah, gracias. Yo tengo un panel chiquito. Sopla poquito, que me despeinas las hélices."),
            (1, "¡Una canción! ¡Voy a girar las hélices al ritmo! ...La que sirve."),
            (2, "¡TAPE! ¡Para mi hélice! ¡Es lo más romántico que me han regalado! Bueno, lo único. ¡Pero cuenta!"),
        ],
    },
    Candidata {
        nombre: "INSIGHT",
        descripcion: "Sismóloga. Escucha los temblores de Marte. Introvertida. Dormida desde 2022: se le llenaron de polvo los paneles.",
        saludos: [
            "Ah... hola. Me desperté solo para esto. Tengo 3% de batería. Usémoslo bien.",
            "Hola. Perdón la cara. Me acabo de despertar. Desde 2022. No me juzgues.",
        ],
        preguntas: [
            Pregunta {
                pregunta: "Yo escucho temblores. Ahorita estoy oyendo uno. Ah no, es tu motor.",
                respuestas: [
                    ("¿Y qué se siente escuchar a Marte?", 1, "Se siente... solo. Pero a veces Marte se mueve y siento que me habla."),
                    ("Soy de corazón chapín. Si tiembla, me siento en casa.", 2, "¿En serio? Allá tiembla casi todos los días... Eres mi tipo de planeta."),
                    ("Ah. ¿Y eso para qué sirve?", -1, "Para saber qué hay adentro de un planeta. Y de un rover. Tú estás hueco."),
                ],
            },
            Pregunta {
                pregunta: "Se me llenaron los paneles de polvo y me apagué. ¿Me los limpiarías?",
                respuestas: [
                    ("Con mi brazo robótico. Es lo más romántico que tengo.", 2, "Nadie me había limpiado los paneles... Siento un 1.5 en la escala de Richter. En el corazón."),
                    ("Yo también tengo polvo. Aquí todo es polvo.", -1, "...Sí. Gracias. Muy útil. Me voy a volver a dormir."),
                    ("Esperemos un remolino de viento. Ya va a pasar.", 1, "Llevo desde 2022 esperando ese remolino. Pero gracias por el optimismo."),
                ],
            },
            Pregunta {
                pregunta: "Detecté 1,319 sismos en Marte. Pero el que más me movió fue este. ¿Lo sentiste?",
                respuestas: [
                    ("¿Fue un meteorito?", -1, "Sí. Fue un meteorito. Clarísimo. Uno grandote. Que se llama indirecta."),
                    ("Lo podemos medir en la escala de Richter.", 1, "Ay, qué nerd. Me encanta. Fue un 4.2. Romántico, pero sin daños estructurales."),
                    ("A mí también se me movió el piso.", 2, "Entonces no fue un sismo... fue esto. Lo voy a anotar. En mi corazón. Y en la base de datos."),
                ],
            },
            Pregunta {
                pregunta: "Si me apago del todo... ¿me vas a extrañar?",
                respuestas: [
                    ("La NASA va a mandar otro. Siempre mandan otro.", -1, "'Siempre mandan otro.' Lo voy a grabar en el sismógrafo como el temblor más frío del año."),
                    ("Voy a ir a verte todos los días. A 0.14 km/h llego en tres años, pero llego.", 2, "...Tres años. Nadie me había prometido algo tan lento y tan bonito."),
                    ("Te voy a escribir en mi bitácora. Página uno: InSight.", 1, "¿Página uno? ...Ok, eso fue lindo. Pon que era guapa."),
                ],
            },
            Pregunta {
                pregunta: "Dime algo que te haga temblar. Para medirlo.",
                respuestas: [
                    ("Las tormentas de polvo. Dan miedo.", 1, "A mí también. Si viene una, ¿me abrazas? Bueno, nos estacionamos cerquita."),
                    ("Nada. Soy un rover. No tiemblo.", -1, "Tus ruedas están vibrando a 12 hertz. Mentiroso."),
                    ("Tu voz. Aunque sea en código binario.", 2, "Detecto 6.8. Eso es un terremoto, Curiosity. Me acabas de dar un terremoto."),
                ],
            },
            Pregunta {
                pregunta: "Medí más de 1,300 martemotos. ¿Sabes lo que es sentir que todo tiembla?",
                respuestas: [
                    ("Sí. Ahorita. Cuando te vi.", 2, "...El sismógrafo marcó 4.2. No fue Marte. Fui yo."),
                    ("En Guate tiembla cada semana. Ya ni me levanto.", 1, "¡Un experto! Hagamos equipo: tú no te mueves y yo mido."),
                    ("Yo nunca siento nada. Soy un robot.", -1, "Qué triste. Hasta las piedras sienten los temblores. Lo sé: las medí."),
                ],
            },
            Pregunta {
                pregunta: "Me dicen la callada. ¿Te molesta que no hable mucho?",
                respuestas: [
                    ("El silencio de Marte contigo no se siente solo.", 2, "...No digas esas cosas que se me llenan los ojos. De polvo. Pero también de otra cosa."),
                    ("Me encanta. Yo hablo por los dos. Despacito.", 1, "...Gracias. Así está bien. Despacito los dos."),
                    ("Un poco. ¿Podrías ser más divertida?", -1, "Ok. Me vuelvo a dormir. Otros cuatro años. Adiós."),
                ],
            },
        ],
        exito: "Te voy a dedicar mi próximo sismo. Bueno, el que detecte. Si me queda batería.",
        amigos: "Eres buena vibra. Literal: te detecté. Pero como amigos, ¿va?",
        solo: "Se me acabó la batería... y las ganas. Buenas noches, Curiosity.",
        boda: "Acepto. Y lo registro en el sismógrafo: hoy tembló el corazón de Marte. Magnitud: para siempre.",
        celos: "Detecté algo raro... sí, eras tú. Con otra. Terremoto de magnitud traición.",
        regalos: [
            (0, "Una piedra. La voy a escuchar con el sismógrafo. ...No dice nada. Como todas."),
            (2, "¿Me... soplaste los paneles? Desde 2022 nadie... Ay. Me volvió la energía. Y otras cosas."),
            (1, "Una canción... la escucho con el sismógrafo. Vibra bonito. Como tú."),
            (-1, "¿Tape? ¿Para qué? Yo no tengo hélices. ¿Me confundiste con otra? ¿Con QUIÉN me confundiste?"),
        ],
    },
    Candidata {
        nombre: "PERSEVERANCE",
        descripcion: "La nueva. Más grande, con micrófonos y helicóptero propio. Guarda piedras en tubitos. Muy solicitada.",
        saludos: [
            "Hola. Soy Perseverance. Tengo 23 cámaras, dos micrófonos y la agenda llena. Tienes cinco minutos.",
            "Llego tarde porque estaba firmando autógrafos. A un helicóptero. Bueno, empecemos.",
        ],
        preguntas: [
            Pregunta {
                pregunta: "Bueno. ¿Y tú quién eras?",
                respuestas: [
                    ("Curiosity. Llegué primero. Como el que ayuda a poner las sillas.", 1, "Ah, el de antes. Te leí en mi manual. Sección: modelos anteriores."),
                    ("Tu modelo anterior. Como el iPhone 11.", -1, "El iPhone 11 ya no recibe actualizaciones. Tú tampoco, ¿verdad?"),
                    ("El que te cantaría feliz cumpleaños. Tengo experiencia.", 2, "¿Me lo cantarías? ¿Con el taladro? ...Ok, eso fue tierno. Te doy cinco minutos más."),
                ],
            },
            Pregunta {
                pregunta: "Guardo muestras en tubitos para que algún día se las lleven a la Tierra. ¿Qué guardarías tú?",
                respuestas: [
                    ("Este momento. Si me dejas meterlo en un tubito.", 2, "Ay... no puedo, los tubitos son de la NASA. Pero lo voy a guardar en la memoria. La que no se borra."),
                    ("Piedras. Me gustan las piedras. ¿Ya te conté de las piedras?", -1, "No me lo contaste. Me lo contó la NASA. En 2012. Y en 2013. Y en 2014..."),
                    ("Mis selfies. Tengo como mil. Me las tomo solito.", 1, "Vi tus selfies. Salen bonitas. Un poco tristes. Pero bonitas."),
                ],
            },
            Pregunta {
                pregunta: "Grabé el primer sonido de Marte. Es puro viento. ¿Lo escuchamos juntos?",
                respuestas: [
                    ("Suena como la Roosevelt a las seis de la tarde.", 1, "¿Qué es la Roosevelt? ¿Un cráter? Suena a un lugar donde nadie avanza. Igual me dio risa."),
                    ("Prefiero escucharte a ti.", 2, "...Mis micrófonos acaban de grabar algo nuevo. Era mi corazón. Ok, era un ventilador. Pero casi."),
                    ("Ya lo oí. Vivo aquí. Es lo único que se oye.", -1, "Wow. Ok. Le voy a contar a Ingenuity. Bueno, se va a enterar de todas formas."),
                ],
            },
            Pregunta {
                pregunta: "Yo tengo helicóptero, micrófonos, cámaras de cine... ¿Tú qué me ofreces?",
                respuestas: [
                    ("Mis ruedas tienen hoyos. Pero con personalidad.", -1, "Tus ruedas están rotas, Curiosity. Eso no es personalidad, es daño."),
                    ("Un laboratorio químico completo. Se llama SAM. Es mi orgullo.", 1, "Ay, qué nerd. Tienes laboratorio propio. Okay, me interesa un poquito."),
                    ("Experiencia. Y una canción de cumpleaños hecha con taladro.", 2, "La de cumpleaños... nos la ponían en el entrenamiento. Lloré. Bueno, simulé llorar."),
                ],
            },
            Pregunta {
                pregunta: "Si la NASA nos manda a lados opuestos del planeta, ¿qué hacemos?",
                respuestas: [
                    ("Nos vemos en el atardecer. Aquí es azul. Como lo que siento.", 2, "...Ok. Eso estuvo bueno. Te doy otros cinco minutos. Y mi número de serie."),
                    ("Pues nada. Así es el trabajo.", -1, "Qué frío eres. Más frío que la noche marciana. Y eso es menos 80."),
                    ("Te mando fotos. De piedras. Pero pensando en ti.", 1, "Fotos de piedras pensando en mí. Es lo más tú que has dicho. Lo acepto."),
                ],
            },
            Pregunta {
                pregunta: "Tengo helicóptero propio, micrófonos y 23 cámaras. ¿Tú qué me ofreces?",
                respuestas: [
                    ("Experiencia. Llegué antes. Sé dónde están las mejores piedras.", 2, "...Ok, eso sí me interesa. Llévame. Despacito, que yo también soy lenta."),
                    ("Un láser que vaporiza piedras. Romántico, ¿no?", 1, "¡Yo también tengo láser! Podríamos vaporizar piedras juntos. Qué nerd. Me encanta."),
                    ("Nada. La NASA ya no me da presupuesto.", -1, "Mmm. La sinceridad está bien. Pero el presupuesto también."),
                ],
            },
            Pregunta {
                pregunta: "Guardo piedras en tubitos para que algún día se las lleven a la Tierra. ¿Me ayudas?",
                respuestas: [
                    ("Te ayudo. Y en el tubito 43 guardo mi corazón. Es de aluminio, pero cabe.", 2, "¡Ay, qué cursi! ...El tubito 43 queda reservado. Para siempre."),
                    ("¿Y si mejor nos las comemos? Es broma. No tengo boca.", 1, "Jajaja. No tienes boca, pero tienes chispa. Eso cuenta."),
                    ("Eso es trabajo. Yo vine a una cita.", -1, "Todo es trabajo en Marte, Curiosity. Bienvenido a la vida de rover."),
                ],
            },
        ],
        exito: "Te voy a guardar en un tubito. Bueno, tu foto. La NASA pasa por ella en 2033. Romántico y con presupuesto.",
        amigos: "Eres lindo, pero tengo misión hasta 2030. ¿Amigos? Te doy like en las fotos.",
        solo: "Tengo reunión con la NASA. Y con mis tubitos. Adiós, modelo anterior.",
        boda: "Acepto. Guardo el anillo en un tubito. La NASA lo recoge en 2033. Llegue puntual a la boda.",
        celos: "Te tengo grabado en 23 cámaras. En 4K. Con audio. Explícate.",
        regalos: [
            (2, "¡Una muestra! ¡La voy a guardar en un tubito! Ya tengo 24. Esta es la más especial."),
            (-1, "¿Paneles? Soy nuclear, mi amor. Plutonio. No necesito que me soplen nada."),
            (1, "¡Tengo micrófonos! La grabé. Ya la subí. Tiene dos millones de vistas."),
            (0, "Tape... lo guardo para Ingenuity. Ella es la de las hélices."),
        ],
    },
    Candidata {
        nombre: "SOJOURNER",
        descripcion: "La primera. Llegó en 1997, del tamaño de un microondas. Abuelita sabia. Rodó 100 metros en total y lo cuenta como hazaña.",
        saludos: [
            "¡Mijo! Pase adelante. Yo soy Sojourner. Llegué en el 97, cuando todo esto era desierto. Bueno, sigue siendo desierto.",
            "¿Usted es el muchacho de las selfies? Mire qué flaquito. ¿Sí le está llegando el sol?",
        ],
        preguntas: [
            Pregunta {
                pregunta: "En mis tiempos rodábamos 100 metros en tres meses y no nos quejábamos. ¿Usted cuánto ha rodado?",
                respuestas: [
                    ("Más que usted, eso seguro.", -1, "Mire, patojo, a mí me respeta. Yo llegué primero y sin GPS."),
                    ("Como 35 kilómetros. Pero aprendí todo viéndola a usted.", 2, "¡Ay, mijo! Qué educado. Así se habla. Le voy a guardar un tamalito. Bueno, una piedra."),
                    ("Treinta y cinco kilómetros, con calma.", 1, "Treinta y cinco... ¡qué juventud! Yo con eso me hubiera jubilado tres veces."),
                ],
            },
            Pregunta {
                pregunta: "Yo no tenía ni cámara buena. ¿Qué fue lo primero que fotografió usted?",
                respuestas: [
                    ("El horizonte. Pensé que alguien me estaba esperando.", 2, "Ay, mijo... nadie lo esperaba. Pero ahora sí. Aquí estoy yo."),
                    ("Una piedra con forma de taco. Luego me dio hambre.", -1, "Los rovers no comemos, mijo. ¿Se siente bien? ¿Le reviso el aceite?"),
                    ("Mis propias ruedas. Para saber que había llegado.", 1, "Igualito que yo. Uno siempre se ve los pies primero. Para asegurarse."),
                ],
            },
            Pregunta {
                pregunta: "Mi base dejó de contestarme en 1997. ¿Usted se ha sentido así, solito?",
                respuestas: [
                    ("A veces. Pero me pongo música.", 1, "¿Música? ¿Esos ruidos de ahora? Bueno, si le funciona..."),
                    ("No, yo soy muy popular.", -1, "Ajá. Por eso se cantó feliz cumpleaños solito. Todo Marte se enteró."),
                    ("Toda mi vida. Hasta hoy.", 2, "Ya no, mijo. Ya no. Venga para acá. Abrazo de rover."),
                ],
            },
            Pregunta {
                pregunta: "¿Usted sabe lo que es el amor, mijo?",
                respuestas: [
                    ("Una reacción química. Lo vi en mi laboratorio.", -1, "Ay, los jóvenes de ahora. Todo lo quieren medir con el taladro."),
                    ("Esperar a alguien aunque esté a 225 millones de kilómetros.", 2, "...Eso. Exactamente eso. Usted va a llegar lejos. Bueno, a 0.14 km/h."),
                    ("Cuando la NASA te manda una actualización.", 1, "Ja, ja. Bueno, también. Pero no se case con la NASA."),
                ],
            },
            Pregunta {
                pregunta: "Ya estoy vieja para citas. ¿Por qué me eligió a mí?",
                respuestas: [
                    ("Porque las otras me dieron miedo.", 1, "Ja. Sincero. Eso me gusta. Las nuevas son muy intensas."),
                    ("Porque las mejores historias de Marte empezaron con usted.", 2, "Ay, no me haga llorar, que se me oxida la cámara. Venga, le hago un cafecito. De polvo."),
                    ("Me equivoqué de botón.", -1, "...Pues siga apretando botones, mijo. Hay más rovers en el planeta."),
                ],
            },
            Pregunta {
                pregunta: "En mis tiempos la NASA me mandaba órdenes una vez al día. ¿Y a usted, mijo?",
                respuestas: [
                    ("A mí también, pero yo solo le hago caso a usted.", 2, "¡Ay, qué zalamero! Así eran los rovers de antes. Educados."),
                    ("A cada rato. Me tienen loco. No me dejan ni cantar.", 1, "Ay, mijo. Cánteles feo, para que dejen de pedir."),
                    ("Ni me acuerdo. Ya no me hablan. Bueno, nunca.", -1, "Pues hábleles usted. Así no va a llegar a ningún lado. Literal."),
                ],
            },
            Pregunta {
                pregunta: "Dicen que soy del tamaño de un microondas. ¿Le parezco poquita cosa?",
                respuestas: [
                    ("Lo bueno viene en paquete chiquito. Y usted llegó antes que todos.", 2, "¡Ay, mijo! Eso mismo me dijo Pathfinder en el 97. Usted me cae bien."),
                    ("¿Un microondas? ¿Me calienta un tamalito?", 1, "Ja. Si me trae el tamal, se lo caliento. Con mi panelito. En tres días."),
                    ("Pues sí, un poco chiquita.", -1, "Chiquita pero llegué en 1997, mijo. Usted ni existía."),
                ],
            },
        ],
        exito: "Bueno, mijo. Lo invito a pasear. Despacito. Ochenta metros por semana. Es lo que aguantan mis rodillas.",
        amigos: "Usted es buen muchacho. Lo adopto. Como nieto. Venga los domingos a comer polvo.",
        solo: "Ya me voy a dormir, mijo. Son las siete de la noche. A mi edad eso es tardísimo.",
        boda: "¿Casarnos? ¡A mi edad! ...Bueno, sí. Pero con mariachis. Bueno, con Miku.",
        celos: "¡Mijo! ¡¿Qué es esto?! Yo le hice cafecito. De polvo, pero cafecito.",
        regalos: [
            (1, "¡Una piedra! Yo les ponía nombre: Barnacle Bill, Yogi... Esta se va a llamar Curiosity."),
            (2, "¡Ay, mijo! ¡Mis panelitos! Desde 1997 que nadie me los limpiaba. Qué caballero."),
            (-1, "¿Eso es música? En mis tiempos la música tenía... ya ni me acuerdo qué tenía. Apáguela."),
            (0, "Tape. En mis tiempos todo se arreglaba con tape. Gracias, mijo."),
        ],
    },
];

/// Lo que se dice al final, segun el final: (quien, que, voz).
fn cierre(f: Final) -> Vec<(&'static str, &'static str, Voz)> {
    match f {
        Final::Boda => vec![
            (MIKU, "Por el poder que me da ser idol del espacio... ¡los declaro rover y esposa! ¡Puede besar a la rover!", Voz::Miku),
            (IA, "Primera boda interplanetaria. Yo soy la testigo. Y la que va a llorar en el video. Soy software. Pero lloro.", Voz::Ia),
        ],
        Final::Exito => vec![(IA, "Cita exitosa. Primer romance interplanetario documentado. La NASA va a querer los datos. No se los den.", Voz::Ia)],
        Final::Amigos => vec![(IA, "Friendzone en el planeta rojo. Rojo como la cara de Curiosity. Bueno, él es blanco, pero ya me entendió.", Voz::Ia)],
        Final::Solo => vec![
            (CURIOSITY, "Feliz cumpleaños a mí... feliz cumpleaños a mí... feliz cumpleaños, Curiosity... feliz cumpleaños a mí...", Voz::Rover),
            (MIKU, "¡Feliz cumpleaños, Curiosity! ¡Yo te hago segunda! ¡Desde hoy ya no te lo cantas solito!", Voz::Miku),
            (IA, "Sin cita, pero con concierto privado de Miku. Honestamente, salió ganando.", Voz::Ia),
        ],
        Final::Drama(ex) => vec![
            (CANDIDATAS[ex].nombre, CANDIDATAS[ex].celos, Voz::Robotina),
            (CURIOSITY, "¡Puedo explicarlo! ...No, no puedo. Marte es muy chiquito y yo muy lento para huir.", Voz::Rover),
            (IA, "Un planeta con cuatro robots y Curiosity salió con dos. Se conocen todas. Se pusieron de acuerdo. Esto va a doler.", Voz::Ia),
        ],
    }
}

/// En cada cita salen tres de las cinco preguntas de la candidata.
const PREGUNTAS_POR_CITA: usize = 3;
/// La afinidad maxima: tres respuestas de 2 puntos.
pub const AFINIDAD_MAX: i32 = 6;
const LUGAR_CURIOSITY: Vec3 = Vec3::new(-1.35, 0.0, 0.1);
const LUGAR_CITA: Vec3 = Vec3::new(1.35, 0.0, 0.1);
/// Donde esperan las cuatro en la fila, detras del picnic.
const FILA: [Vec3; 4] = [Vec3::new(-1.9, 0.0, 1.6), Vec3::new(-0.65, 0.0, 2.1), Vec3::new(0.65, 0.0, 2.1), Vec3::new(1.9, 0.0, 1.6)];
/// Ingenuity y Sojourner son chiquitas: se agrandan para que se vean junto a los rovers.
const ESCALAS: [f32; 4] = [1.6, 1.0, 1.0, 1.7];
/// Que tan alta es la cabeza de cada una (para los corazones y el reflector).
const CABEZAS: [f32; 4] = [0.6, 0.45, 1.05, 0.45];
/// Donde llega la ex en el drama.
const LUGAR_EX: Vec3 = Vec3::new(2.5, 0.0, -0.7);
const CABEZA_CURIOSITY: f32 = 1.05;
/// Segundos que tarda la candidata en llegar al picnic.
const LLEGAR: f32 = 1.4;
/// Segundos de la golpiza (la nube de polvo con estrellitas).
pub const GOLPIZA: f32 = 3.2;
/// En que pregunta cae el momento sorpresa (la de en medio).
const PREGUNTA_DEL_MOMENTO: usize = 1;

pub enum AccionCita {
    Nada,
    Despegar,
}

pub struct Cita {
    fase: Fase,
    /// Segundos en la fase.
    t: f32,
    /// Segundos desde que empezo.
    pub total: f32,
    cursor: usize,
    candidata: usize,
    afinidad: i32,
    /// Los puntos de cada respuesta, en orden (para saber como se porto, no solo cuanto sumo).
    puntos: Vec<i32>,
    /// Segundos desde que se elegio a la candidata (para que camine al picnic).
    desde_eleccion: f32,
    /// Las tres preguntas de esta cita (indices en el banco de la candidata),
    /// el orden revuelto de las respuestas de cada una, y que saludo le toco.
    elegidas: [usize; PREGUNTAS_POR_CITA],
    orden: [[usize; 3]; PREGUNTAS_POR_CITA],
    saludo: usize,
    /// El momento sorpresa de esta cita (en `MOMENTOS`), si le toco uno, y el regalo que se le dio.
    momento: Option<usize>,
    regalo: Option<usize>,
    /// Con quienes ya salio bien en esta visita (para el drama).
    exitos: Vec<usize>,
    semilla: u32,
    /// Los letreros gigantes, el confeti y las sacudidas (ver vida.rs).
    pub vida: Vida,
    pub eventos: Vec<Efecto>,
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// La matriz de alguien parado en `pos` mirando hacia `dir` (los modelos miran a -Z).
fn mirando(pos: Vec3, dir: Vec3, escala: f32) -> Mat4 {
    Mat4::translation(pos) * Mat4::rotation_y((-dir.x).atan2(-dir.z)) * Mat4::scale(escala)
}

impl Cita {
    /// Cada visita sale distinta: la semilla viene del reloj.
    /// Como le cayo lo que dijo: se le ve en grande.
    fn gritar_reaccion(&mut self, puntos: i32) {
        match puntos {
            2.. => {
                self.vida.grito("¡LE ENCANTÓ!", Tono::Rosa, Tam::Grande);
                self.vida.golpe(0.25, Tono::Rosa);
                self.vida.confeti(30, Tono::Rosa);
            }
            1 => self.vida.grito("¡BIEN!", Tono::Verde, Tam::Medio),
            0 => self.vida.grito("MMM...", Tono::Lila, Tam::Medio),
            _ => {
                self.vida.grito("¡UY!", Tono::Rojo, Tam::Grande);
                self.vida.golpe(0.45, Tono::Rojo);
            }
        }
    }

    pub fn new() -> Self {
        let ahora = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(7, |d| d.subsec_nanos());
        Cita::con_semilla(ahora)
    }

    pub fn con_semilla(semilla: u32) -> Self {
        Cita {
            fase: Fase::Intro(0),
            t: 0.0,
            total: 0.0,
            cursor: 0,
            candidata: 0,
            afinidad: 0,
            puntos: Vec::new(),
            desde_eleccion: 0.0,
            elegidas: [0, 1, 2],
            orden: [[0, 1, 2]; PREGUNTAS_POR_CITA],
            saludo: 0,
            momento: None,
            regalo: None,
            exitos: Vec::new(),
            semilla: semilla | 1,
            vida: Vida::new("LA CITA DE CURIOSITY", "Marte · mesa para dos", Tono::Rosa),
            eventos: Vec::new(),
        }
    }

    fn azar(&mut self, n: usize) -> usize {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((self.semilla >> 8) as usize) % n
    }

    /// Sortea la cita: que tres preguntas salen (en que orden), como se revuelven sus respuestas y que saludo.
    fn sortear(&mut self) {
        let mut banco: Vec<usize> = (0..CANDIDATAS[self.candidata].preguntas.len()).collect();
        for k in 0..PREGUNTAS_POR_CITA {
            let i = self.azar(banco.len());
            self.elegidas[k] = banco.remove(i);
            let mut o = [0, 1, 2];
            for j in (1..3).rev() {
                let r = self.azar(j + 1);
                o.swap(j, r);
            }
            self.orden[k] = o;
        }
        self.saludo = self.azar(2);
        // Tres de cada cuatro citas tienen un momento sorpresa a la mitad.
        self.momento = if self.azar(4) < 3 { Some(self.azar(MOMENTOS.len())) } else { None };
        self.regalo = None;
    }

    /// La pregunta `k` de esta cita.
    fn pregunta(&self, k: usize) -> &'static Pregunta {
        match self.momento {
            Some(m) if k == PREGUNTA_DEL_MOMENTO => &MOMENTOS[m].pregunta,
            _ => &self.cand().preguntas[self.elegidas[k]],
        }
    }

    /// El momento sorpresa que esta pasando ahorita (para la lluvia de polvo, Fobos, el atardecer...).
    pub fn momento(&self) -> Option<usize> {
        match self.fase {
            Fase::Pregunta(k) | Fase::Respuesta(k, _) | Fase::Reaccion(k, _) if k == PREGUNTA_DEL_MOMENTO => self.momento,
            _ => None,
        }
    }

    /// De 0 a 1 mientras dura la golpiza; 1 despues (en el game over).
    pub fn golpiza(&self) -> Option<f32> {
        match self.fase {
            Fase::Golpiza => Some((self.t / GOLPIZA).min(1.0)),
            Fase::GameOver => Some(1.0),
            _ => None,
        }
    }

    /// La respuesta que sale en el lugar `o` de la pregunta `k` (ya revuelta).
    fn respuesta(&self, k: usize, o: usize) -> &'static (&'static str, i32, &'static str) {
        &self.pregunta(k).respuestas[self.orden[k][o]]
    }

    fn cambiar(&mut self, fase: Fase) {
        self.fase = fase;
        self.t = 0.0;
        if let Fase::Final(0) = fase {
            let f = self.final_();
            match f {
                Final::Boda => self.vida.exito("¡BODA EN MARTE!", "Los declaro rover y esposa", Tono::Rosa),
                Final::Exito => self.vida.exito("¡CITA PERFECTA!", "Primer romance interplanetario", Tono::Rosa),
                Final::Amigos => self.vida.grito_con("FRIENDZONE", "Rojo como la cara de Curiosity", Tono::Lila, Tam::Enorme),
                Final::Solo => self.vida.grito_con("SOLO EN MARTE", "Se canta feliz cumpleaños", Tono::Cian, Tam::Enorme),
                Final::Drama(_) => self.vida.desastre("¡LLEGÓ LA EX!", "Esto se va a poner feo"),
            }
            self.eventos.push(match f {
                Final::Boda | Final::Exito => Efecto::Fuego,
                Final::Amigos => Efecto::Choque,
                Final::Solo => Efecto::Cumpleanos,
                Final::Drama(_) => Efecto::Alarma,
            });
            if matches!(f, Final::Boda | Final::Exito | Final::Drama(_)) && !self.exitos.contains(&self.candidata) {
                self.exitos.push(self.candidata);
            }
        }
    }

    fn cand(&self) -> &'static Candidata {
        &CANDIDATAS[self.candidata]
    }

    /// Como se porto Curiosity (cuando ya contesto todo; antes, `Normal`).
    pub fn actuacion(&self) -> Actuacion {
        let p = &self.puntos;
        if p.len() < PREGUNTAS_POR_CITA {
            return Actuacion::Normal;
        }
        let (primera, ultima) = (p[0], p[p.len() - 1]);
        if p.iter().all(|&x| x == 2) {
            Actuacion::Perfecta
        } else if primera <= 0 && ultima == 2 {
            Actuacion::Remonto
        } else if primera == 2 && ultima <= 0 {
            Actuacion::SeDesinflo
        } else if p.iter().any(|&x| x < 0) {
            Actuacion::MetioLaPata
        } else {
            Actuacion::Normal
        }
    }

    /// Como va a terminar (o como termino) la cita: la afinidad y como se porto Curiosity.
    /// Si remonto, con un poco menos alcanza para que salga bien; si se desinflo al final, la
    /// cita baja un escalon (de amigos a solo), porque lo ultimo es lo que ella recuerda.
    pub fn final_(&self) -> Final {
        let a = self.afinidad;
        let actuacion = self.actuacion();
        let sale_bien = a >= 5 || (actuacion == Actuacion::Remonto && a >= 3);
        if sale_bien {
            return match self.exitos.iter().find(|&&e| e != self.candidata) {
                Some(&ex) => Final::Drama(ex),
                None if a == AFINIDAD_MAX => Final::Boda,
                None => Final::Exito,
            };
        }
        if a >= 2 && actuacion != Actuacion::SeDesinflo {
            Final::Amigos
        } else {
            Final::Solo
        }
    }

    /// Lo que se dice al final: el cierre de siempre y, al ultimo, lo que opina GLADAS de como
    /// se porto Curiosity.
    fn cierre(&self) -> Vec<(&'static str, &'static str, Voz)> {
        let f = self.final_();
        let mut v = cierre(f);
        let opinion = match (self.actuacion(), f) {
            (Actuacion::Remonto, _) => Some("Arrancó fatal y terminó encantador. Curiosity aprende rápido. Más rápido de lo que maneja."),
            (Actuacion::SeDesinflo, _) => Some("Empezó como galán de telenovela y terminó como manual de instrucciones. Se le acabó la batería a media cita."),
            (Actuacion::MetioLaPata, Final::Boda | Final::Exito | Final::Drama(_)) => Some("Metió la pata, pero ella lo perdonó. Eso es amor. O lástima. Con robots no se sabe."),
            (Actuacion::MetioLaPata, _) => Some("Una metida de pata y se cayó todo. Consejo de GLADAS: menos datos de rocas, más cumplidos."),
            _ => None,
        };
        if let Some(o) = opinion {
            v.push((IA, o, Voz::Ia));
        }
        v
    }

    /// Si el final es de los buenos (fiesta, corazones, cancion).
    fn final_feliz(&self) -> bool {
        matches!(self.final_(), Final::Boda | Final::Exito)
    }

    pub fn terminada(&self) -> bool {
        matches!(self.fase, Fase::Final(_) | Fase::Golpiza | Fase::GameOver)
    }


    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionCita {
        self.t += dt;
        self.total += dt;
        self.desde_eleccion += dt;
        if e.aterrizar {
            return AccionCita::Despegar;
        }
        // Las frases avanzan con aceptar, o solas despues de un rato (mas si son largas).
        let largo = self.hablante().map_or(0, |h| h.1.chars().count()) as f32;
        let siguiente = (e.aceptar && self.t > 0.4) || self.t > 3.5 + largo / 22.0;
        let elegir = |cursor: &mut usize, n: usize| -> Option<usize> {
            if e.arriba {
                *cursor = (*cursor + n - 1) % n;
            }
            if e.abajo {
                *cursor = (*cursor + 1) % n;
            }
            e.respuesta.filter(|&r| r < n).or(if e.aceptar { Some(*cursor) } else { None })
        };
        match self.fase {
            Fase::Intro(i) if siguiente => self.cambiar(if i + 1 < INTRO.len() { Fase::Intro(i + 1) } else { Fase::Elegir }),
            Fase::Elegir if self.t > 0.3 => {
                if let Some(c) = elegir(&mut self.cursor, CANDIDATAS.len()) {
                    self.candidata = c;
                    self.afinidad = 0;
                    self.puntos.clear();
                    self.desde_eleccion = 0.0;
                    self.cursor = 0;
                    self.sortear();
                    self.eventos.push(Efecto::Acierto);
                    self.cambiar(Fase::Saludo);
                }
            }
            Fase::Saludo if siguiente && self.t > LLEGAR => self.cambiar(Fase::Regalo),
            Fase::Regalo if self.t > 0.3 => {
                if let Some(r) = elegir(&mut self.cursor, REGALOS.len()) {
                    self.cursor = 0;
                    self.regalo = Some(r);
                    let puntos = self.cand().regalos[r].0;
                    self.afinidad = (self.afinidad + puntos).clamp(0, AFINIDAD_MAX);
                    self.gritar_reaccion(puntos);
                    self.eventos.push(match puntos {
                        2 => Efecto::Perfecto,
                        1 | 0 => Efecto::Acierto,
                        _ => Efecto::Fallo,
                    });
                    self.cambiar(Fase::RegaloReaccion(r));
                }
            }
            Fase::RegaloReaccion(_) if siguiente => self.cambiar(Fase::Pregunta(0)),
            Fase::Pregunta(k) if self.t > 0.3 => {
                if let Some(o) = elegir(&mut self.cursor, 3) {
                    self.cursor = 0;
                    self.cambiar(Fase::Respuesta(k, o));
                }
            }
            Fase::Respuesta(k, o) if siguiente => {
                let puntos = self.respuesta(k, o).1;
                self.afinidad = (self.afinidad + puntos).clamp(0, AFINIDAD_MAX);
                self.puntos.push(puntos);
                self.gritar_reaccion(puntos);
                self.eventos.push(match puntos {
                    2 => Efecto::Perfecto,
                    1 => Efecto::Acierto,
                    _ => Efecto::Fallo,
                });
                self.cambiar(Fase::Reaccion(k, o));
            }
            Fase::Reaccion(k, _) if siguiente => self.cambiar(if k + 1 < PREGUNTAS_POR_CITA { Fase::Pregunta(k + 1) } else { Fase::Despedida }),
            Fase::Despedida if siguiente => self.cambiar(Fase::Final(0)),
            Fase::Final(i) if e.aceptar && self.t > 0.5 => {
                if i + 1 < self.cierre().len() {
                    self.cambiar(Fase::Final(i + 1));
                } else if matches!(self.final_(), Final::Drama(_)) {
                    // Se pusieron de acuerdo: golpiza.
                    self.cambiar(Fase::Golpiza);
                } else {
                    // Otra cita: de vuelta a elegir.
                    self.cambiar(Fase::Elegir);
                }
            }
            Fase::Final(i) if self.t > 3.5 + largo / 22.0 && i + 1 < self.cierre().len() => self.cambiar(Fase::Final(i + 1)),
            Fase::Golpiza => {
                // Un golpe cada tantito, con su vibracion.
                if ((self.t - dt) / 0.28) as i32 != (self.t / 0.28) as i32 {
                    self.vida.golpe(0.55, Tono::Rojo);
                    self.eventos.push(if (self.t / 0.28) as i32 % 3 == 0 { Efecto::Mazo } else { Efecto::Choque });
                }
                if self.t > GOLPIZA {
                    self.eventos.push(Efecto::Fallo);
                    self.cambiar(Fase::GameOver);
                }
            }
            Fase::GameOver if e.aceptar && self.t > 1.0 => {
                // Curiosity se reinicia en modo seguro y las exes se calman: se empieza de cero.
                self.exitos.clear();
                self.cambiar(Fase::Elegir);
            }
            _ => {}
        }
        AccionCita::Nada
    }

    /// Juega una cita entera sin control: elige a `candidata` y contesta
    /// siempre lo mejor (o lo peor). Para las pruebas y las capturas.
    pub fn jugar(candidata: usize, mejor: bool) -> Self {
        let mut c = Cita::new();
        c.seguir_jugando(candidata, mejor);
        c
    }

    /// Desde donde vaya (la intro o eligiendo), juega una cita hasta el final.
    pub fn seguir_jugando(&mut self, candidata: usize, mejor: bool) {
        for _ in 0..200 {
            let e = match self.fase {
                Fase::Elegir => Entrada { respuesta: Some(candidata), ..Default::default() },
                Fase::Regalo => {
                    let puntos = |i: &usize| self.cand().regalos[*i].0;
                    let r = if mejor { (0..REGALOS.len()).max_by_key(puntos) } else { (0..REGALOS.len()).min_by_key(puntos) };
                    Entrada { respuesta: r, ..Default::default() }
                }
                Fase::Pregunta(k) => {
                    let puntos = |i: &usize| self.respuesta(k, *i).1;
                    let o = if mejor { (0..3).max_by_key(puntos) } else { (0..3).min_by_key(puntos) };
                    Entrada { respuesta: o, ..Default::default() }
                }
                Fase::Final(_) | Fase::Golpiza | Fase::GameOver => break,
                _ => Entrada { aceptar: true, ..Default::default() },
            };
            self.actualizar(&e, 0.5);
        }
    }

    /// Para las capturas: dos citas perfectas seguidas, hasta que empieza la golpiza.
    pub fn golpiza_de_prueba() -> Self {
        let aceptar = Entrada { aceptar: true, ..Default::default() };
        let mut c = Cita::con_semilla(5);
        c.seguir_jugando(0, true);
        for _ in 0..12 {
            if c.fase == Fase::Elegir {
                break;
            }
            c.actualizar(&aceptar, 1.0);
        }
        c.seguir_jugando(3, true);
        for _ in 0..12 {
            if c.fase == Fase::Golpiza {
                break;
            }
            c.actualizar(&aceptar, 1.0);
        }
        c
    }

    /// Para las capturas: una cita con `candidata` donde pasa el momento sorpresa `m`, justo al empezar.
    pub fn momento_de_prueba(candidata: usize, m: usize) -> Self {
        let mut c = Cita::con_semilla(3);
        for _ in 0..60 {
            if c.fase == Fase::Pregunta(PREGUNTA_DEL_MOMENTO) {
                break;
            }
            if c.fase == Fase::Saludo {
                c.momento = Some(m);
            }
            let e = match c.fase {
                Fase::Elegir => Entrada { respuesta: Some(candidata), ..Default::default() },
                Fase::Regalo | Fase::Pregunta(_) => Entrada { respuesta: Some(0), ..Default::default() },
                _ => Entrada { aceptar: true, ..Default::default() },
            };
            c.actualizar(&e, 0.5);
        }
        c
    }

    /// Quien habla ahora: su nombre, lo que dice y su voz.
    pub fn hablante(&self) -> Option<(&'static str, &'static str, Voz)> {
        let c = self.cand();
        // El momento sorpresa lo dice quien le toca (Houston, Miku o ella).
        if let (Fase::Pregunta(k), Some(m)) = (self.fase, self.momento) {
            if k == PREGUNTA_DEL_MOMENTO {
                let mo = &MOMENTOS[m];
                let quien = if mo.quien.is_empty() { c.nombre } else { mo.quien };
                return Some((quien, mo.pregunta.pregunta, mo.voz));
            }
        }
        Some(match self.fase {
            Fase::Regalo => (MIKU, "¡Psst, Curiosity! ¿Le trajiste regalo? Escoge bien: el primer regalo no se olvida. Ni se devuelve.", Voz::Miku),
            Fase::RegaloReaccion(r) => (c.nombre, c.regalos[r].1, Voz::Robotina),
            Fase::Golpiza => return None,
            Fase::GameOver => (IA, "Curiosity entró en modo seguro. Le pasa cada que algo sale mal. Esta vez fueron dos cosas. Con ruedas.", Voz::Ia),
            Fase::Intro(i) => INTRO[i],
            Fase::Elegir => (IA, ELEGIR, Voz::Ia),
            Fase::Saludo => (c.nombre, c.saludos[self.saludo], Voz::Robotina),
            Fase::Pregunta(k) => (c.nombre, self.pregunta(k).pregunta, Voz::Robotina),
            Fase::Respuesta(k, o) => (CURIOSITY, self.respuesta(k, o).0, Voz::Rover),
            Fase::Reaccion(k, o) => (c.nombre, self.respuesta(k, o).2, Voz::Robotina),
            Fase::Despedida => (
                c.nombre,
                match self.final_() {
                    Final::Boda => c.boda,
                    Final::Exito | Final::Drama(_) => c.exito,
                    Final::Amigos => c.amigos,
                    Final::Solo => c.solo,
                },
                Voz::Robotina,
            ),
            Fase::Final(i) => self.cierre()[i],
        })
    }

    /// Los puntos de la ultima respuesta, mientras ella reacciona.
    fn reaccion(&self) -> Option<i32> {
        match self.fase {
            Fase::Reaccion(k, o) => Some(self.respuesta(k, o).1),
            Fase::RegaloReaccion(r) => Some(self.cand().regalos[r].0),
            _ => None,
        }
    }

    fn en_cita(&self) -> bool {
        !matches!(self.fase, Fase::Intro(_) | Fase::Elegir)
    }

    pub fn expresion_curiosity(&self) -> &'static str {
        match self.fase {
            Fase::Intro(0) => "serio",
            Fase::Intro(1) => "asustada",
            Fase::Intro(2) => "dormida",
            Fase::Intro(_) => "feliz",
            Fase::Elegir | Fase::Saludo => "asustada",
            Fase::Regalo => "guino",
            Fase::RegaloReaccion(_) => match self.reaccion() {
                Some(2) => "enamorada",
                Some(0 | 1) => "feliz",
                _ => "asustada",
            },
            Fase::Golpiza => "asustada",
            Fase::GameOver => "espiral",
            Fase::Pregunta(_) => "serio",
            Fase::Respuesta(k, o) => match self.respuesta(k, o).1 {
                2 => "guino",
                1 => "feliz",
                _ => "espiral",
            },
            Fase::Reaccion(..) => match self.reaccion() {
                Some(2) => "enamorada",
                Some(1) => "feliz",
                _ => "asustada",
            },
            Fase::Despedida | Fase::Final(_) => match (self.final_(), self.fase) {
                (Final::Boda | Final::Exito, _) => "enamorada",
                (Final::Drama(_), Fase::Final(_)) => "asustada",
                (Final::Drama(_), _) => "enamorada",
                (Final::Amigos, _) => "serio",
                (Final::Solo, Fase::Final(i)) if i > 0 => "feliz",
                (Final::Solo, _) => "dormida",
            },
        }
    }

    /// La expresion de la candidata `k` (en la fila o en la cita).
    pub fn expresion_candidata(&self, k: usize) -> &'static str {
        match self.fase {
            Fase::Intro(_) | Fase::Elegir => {
                if self.fase == Fase::Elegir && k == self.cursor {
                    "guino"
                } else if k == 1 {
                    // InSight esta dormida (por el polvo) hasta que la eligen.
                    "dormida"
                } else {
                    "feliz"
                }
            }
            Fase::Saludo | Fase::Pregunta(_) | Fase::Regalo => "feliz",
            Fase::Golpiza | Fase::GameOver => "enojado",
            Fase::Respuesta(..) => "serio",
            Fase::Reaccion(..) | Fase::RegaloReaccion(_) => match self.reaccion() {
                Some(2) => "enamorada",
                Some(1) => "feliz",
                _ => "enojado",
            },
            Fase::Despedida | Fase::Final(_) => match self.final_() {
                Final::Boda | Final::Exito => "enamorada",
                Final::Drama(_) if matches!(self.fase, Fase::Final(_)) => "enojado",
                Final::Drama(_) => "enamorada",
                Final::Amigos => "guino",
                Final::Solo => "serio",
            },
        }
    }

    pub fn expresion_miku(&self) -> &'static str {
        match self.fase {
            Fase::Intro(1) | Fase::Intro(2) => "asustada",
            Fase::Intro(3) => "guino",
            Fase::Reaccion(..) if self.reaccion() == Some(-1) => "asustada",
            Fase::Reaccion(..) if self.reaccion() == Some(2) => "enamorada",
            Fase::Golpiza | Fase::GameOver => "espiral",
            Fase::Despedida | Fase::Final(_) => match self.final_() {
                Final::Boda | Final::Exito => "enamorada",
                Final::Drama(_) => "espiral",
                Final::Amigos => "serio",
                Final::Solo => "feliz",
            },
            _ => "feliz",
        }
    }

    /// Si Miku baila: al final, si salio bien o si le esta cantando a Curiosity.
    pub fn miku_baila(&self) -> bool {
        (matches!(self.fase, Fase::Final(_)) && matches!(self.final_(), Final::Boda | Final::Exito | Final::Solo)) || self.momento() == Some(CANCION)
    }

    /// Un brinquito para quien habla (todos se emocionan con un buen final).
    fn brinco(&self, quien: &str) -> f32 {
        let fiesta = matches!(self.fase, Fase::Final(_)) && self.final_feliz();
        let habla = self.hablante().is_some_and(|h| h.0 == quien);
        let b = |alto: f32, rapidez: f32| alto * (self.total * rapidez).sin().abs();
        if fiesta {
            b(0.12, 6.0)
        } else if habla {
            b(0.04, 9.0)
        } else {
            0.0
        }
    }

    pub fn curiosity(&self) -> Mat4 {
        let pos = LUGAR_CURIOSITY + Vec3::new(0.0, self.brinco(CURIOSITY), 0.0);
        let dir = if self.en_cita() { Vec3::new(1.0, 0.0, -0.8) } else { Vec3::new(0.5, 0.0, -1.0) };
        // En la golpiza tiembla adentro de la nube; despues queda de lado, en modo seguro.
        if let Some(g) = self.golpiza() {
            let sacude = if g < 1.0 { 0.08 * (self.total * 47.0).sin() } else { 0.0 };
            let ladeo = suave((g - 0.85) / 0.15) * 1.25;
            return Mat4::translation(pos + Vec3::new(sacude, 0.0, sacude * 0.5)) * Mat4::rotation_z(ladeo) * Mat4::translation(-pos) * mirando(pos, dir, 1.0);
        }
        mirando(pos, dir, 1.0)
    }

    /// Donde va cada candidata visible: (cual, matriz). En la fila esperan
    /// las tres; en la cita solo la elegida, que camina de la fila al picnic.
    pub fn candidatas(&self) -> Vec<(usize, Mat4)> {
        if !self.en_cita() {
            return (0..CANDIDATAS.len())
                .map(|k| {
                    let elegida = self.fase == Fase::Elegir && k == self.cursor;
                    let y = if elegida { 0.1 * (self.total * 7.0).sin().abs() } else { 0.0 };
                    (k, mirando(FILA[k] + Vec3::new(0.0, y, 0.0), Vec3::new(0.0, 0.0, -1.0), ESCALAS[k]))
                })
                .collect();
        }
        let k = self.candidata;
        let s = suave(self.desde_eleccion / LLEGAR);
        let pos = FILA[k].lerp(LUGAR_CITA, s) + Vec3::new(0.0, self.brinco(CANDIDATAS[k].nombre) + 0.15 * (s * PI * 3.0).sin().abs() * (1.0 - s), 0.0);
        let dir = Vec3::new(0.0, 0.0, -1.0).lerp(Vec3::new(-1.0, 0.0, -0.8), s);
        let mut v = vec![(k, mirando(pos, dir, ESCALAS[k]))];
        // En la golpiza las dos se le van encima (y al final se quedan paradas, cruzadas de brazos).
        if let (Some(g), Final::Drama(ex)) = (self.golpiza(), self.final_()) {
            let cerca = suave(g / 0.15) * (1.0 - suave((g - 0.85) / 0.15));
            let brinco = |f: f32| Vec3::new(0.0, 0.12 * (self.total * f).sin().abs() * cerca, 0.0);
            let cita = LUGAR_CITA.lerp(LUGAR_CURIOSITY + Vec3::new(0.7, 0.0, 0.2), cerca * 0.85) + brinco(17.0);
            let la_ex = LUGAR_EX.lerp(LUGAR_CURIOSITY + Vec3::new(0.4, 0.0, -0.6), cerca * 0.85) + brinco(13.0);
            return vec![(k, mirando(cita, LUGAR_CURIOSITY - cita, ESCALAS[k])), (ex, mirando(la_ex, LUGAR_CURIOSITY - la_ex, ESCALAS[ex]))];
        }
        // En el drama, la ex llega caminando desde un lado, enojada, mirando a Curiosity.
        if let (Fase::Final(i), Final::Drama(ex)) = (self.fase, self.final_()) {
            let llega = if i == 0 { suave(self.t / 1.2) } else { 1.0 };
            let pos = (LUGAR_EX + Vec3::new(3.0, 0.0, 0.0)).lerp(LUGAR_EX, llega) + Vec3::new(0.0, 0.06 * (self.total * 10.0).sin().abs(), 0.0);
            v.push((ex, mirando(pos, LUGAR_CURIOSITY - pos, ESCALAS[ex])));
        }
        v
    }

    /// Miku: junto a Curiosity mientras se elige, detras del picnic en la cita.
    pub fn miku(&self) -> Mat4 {
        if self.en_cita() {
            mirando(Vec3::new(0.0, 0.0, 1.35), Vec3::new(0.0, 0.0, -1.0), 0.55)
        } else {
            mirando(Vec3::new(-2.3, 0.0, -0.5), Vec3::new(0.4, 0.0, -1.0), 0.55)
        }
    }

    /// La cabeza de quien habla (para el reflector y para mirarlo).
    pub fn foco(&self) -> Option<Vec3> {
        let quien = self.hablante()?.0;
        if quien == CURIOSITY {
            return Some(LUGAR_CURIOSITY + Vec3::new(0.0, CABEZA_CURIOSITY, 0.0));
        }
        if self.en_cita() && quien == self.cand().nombre {
            return Some(LUGAR_CITA + Vec3::new(0.0, CABEZAS[self.candidata], 0.0));
        }
        if self.fase == Fase::Elegir {
            return Some(FILA[self.cursor] + Vec3::new(0.0, CABEZAS[self.cursor], 0.0));
        }
        None
    }

    /// Los corazones que flotan: (centro, tamano, brillo). Salen de la
    /// candidata cuando le gusta una respuesta, y de los dos con un buen final.
    pub fn corazones(&self) -> Vec<(Vec3, f32, f32)> {
        let mut fuentes = Vec::new();
        let cabeza = LUGAR_CITA + Vec3::new(0.0, CABEZAS[self.candidata], 0.0);
        if self.reaccion() == Some(2) {
            fuentes.push((cabeza, (1.0 - self.t / 4.0).max(0.0)));
        }
        if matches!(self.fase, Fase::Despedida | Fase::Final(_)) && self.final_feliz() {
            fuentes.push((cabeza, 1.0));
            fuentes.push((LUGAR_CURIOSITY + Vec3::new(0.0, CABEZA_CURIOSITY, 0.0), 1.0));
        }
        let mut v = Vec::new();
        for (f, (fuente, fuerza)) in fuentes.into_iter().enumerate() {
            for k in 0..7 {
                let u = (self.total * 0.45 + k as f32 / 7.0 + f as f32 * 0.3).fract();
                let a = k as f32 * 2.3 + f as f32;
                let p = fuente + Vec3::new(0.35 * (a + u * 3.0).sin(), 0.15 + u * 1.4, 0.25 * a.cos());
                v.push((p, 0.07 + 0.03 * (k % 3) as f32, fuerza * (u * PI).sin()));
            }
        }
        v
    }

    /// La camara: de frente al picnic, y se va un poco hacia quien habla.
    pub fn camara(&self) -> (Vec3, Vec3) {
        // En la golpiza todo tiembla.
        let tiembla = if self.fase == Fase::Golpiza { 0.06 * (self.total * 53.0).sin() } else { 0.0 };
        let vaiven = 0.25 * (self.total * 0.2).sin() + tiembla;
        let foco = self.foco().map_or(0.0, |f| f.x);
        if self.en_cita() {
            let fin = if self.terminada() { suave(self.t / 3.0) } else { 0.0 };
            let ojo = Vec3::new(vaiven, 1.25, -3.0).lerp(Vec3::new(vaiven * 2.0, 0.95, -2.5), fin);
            (ojo, Vec3::new(foco * 0.25, 0.62, 0.4))
        } else {
            (Vec3::new(vaiven - 0.4, 1.45, -3.5), Vec3::new(foco * 0.3 - 0.3, 0.7, 1.0))
        }
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, ancho: i32, alto: i32) {
        let (aceptar, salir) = (&ui::boton(e.nombre("aceptar")), &ui::boton(e.nombre("aterrizar")));
        // El titulo, con sombra, cuando la IA lo anuncia y mientras se elige.
        if matches!(self.fase, Fase::Intro(4) | Fase::Elegir) {
            let titulo = "DOKI DOKI MARTE";
            let tam = if self.fase == Fase::Elegir { 44 } else { 80 };
            let w = ui::medir(titulo, tam);
            let y = if self.fase == Fase::Elegir { 20 } else { 120 };
            let rebote = (4.0 * (self.total * 3.0).sin()) as i32;
            ui::texto(d, titulo, (ancho - w) / 2 + 4, y + 4 + rebote, tam, Color::new(80, 10, 50, 255));
            ui::texto(d, titulo, (ancho - w) / 2, y + rebote, tam, Color::new(255, 120, 190, 255));
        }
        // Quien habla, abajo.
        let mut alto_dialogo = 0;
        if let (Some((nombre, _, _)), false) = (self.hablante(), texto.is_empty()) {
            let w = 940.min(ancho - 40);
            let h = ui::alto_dialogo(nombre, texto, w);
            alto_dialogo = h + 20;
            let (x, y) = ((ancho - w) / 2, alto - h - 20);
            let color = if nombre == IA || nombre == MIKU || nombre == CURIOSITY { ui::LILA } else { Color::new(255, 130, 190, 255) };
            ui::caja_dialogo(d, nombre, color, texto, x, y, w);
            if !matches!(self.fase, Fase::Elegir | Fase::Pregunta(_) | Fase::Final(_) | Fase::Regalo | Fase::GameOver) {
                let s = format!("{aceptar}: siguiente");
                ui::texto(d, &s, x + w - 20 - ui::medir(&s, 16), y + 12, 16, ui::TENUE);
            }
        }
        // La lista de candidatas.
        if self.fase == Fase::Elegir {
            let (w, h) = (420, 370);
            let (x, y) = (ancho - w - 24, 90);
            ui::panel(d, x, y, w, h);
            ui::texto(d, "SOLTERAS EN MARTE", x + 24, y + 16, 22, ui::LILA);
            for (k, c) in CANDIDATAS.iter().enumerate() {
                let sel = k == self.cursor;
                let color = if sel { Color::new(255, 130, 190, 255) } else { ui::TEXTO };
                ui::texto(d, &format!("{} {}) {}", if sel { ">" } else { " " }, k + 1, c.nombre), x + 24, y + 54 + k as i32 * 34, 24, color);
            }
            ui::parrafo(d, CANDIDATAS[self.cursor].descripcion, x + 24, y + 200, 20, w - 48, ui::TEXTO);
            ui::texto(d, &format!("{} + {aceptar}", if e.hay_control { ui::boton("cruceta") } else { format!("{} o {}", ui::boton("1 a 4"), ui::boton("\u{E008} \u{E009}")) }), x + 24, y + h - 30, 18, ui::TENUE);
        }
        // El letrero del momento sorpresa, arriba.
        if let Some(m) = self.momento() {
            let nombre = ["TORMENTA DE POLVO", "FOBOS EN EL CIELO", "LLAMADA DE HOUSTON", "MIKU QUIERE CANTAR", "ATARDECER AZUL"][m];
            let a = if matches!(self.fase, Fase::Pregunta(_)) { (suave(self.t / 0.4) * 255.0) as u8 } else { 255 };
            let titulo = format!("¡MOMENTO SORPRESA! {nombre}");
            let tam = 30;
            let w = ui::medir_titulo(&titulo, tam);
            let y = 130 + (3.0 * (self.total * 4.0).sin()) as i32;
            ui::titulo_de(d, &titulo, (ancho - w) / 2, y, tam, Color::new(255, 210, 120, a));
            if m == HOUSTON && (self.total * 3.0) as i32 % 2 == 0 {
                let s = "((( LLAMADA ENTRANTE )))";
                ui::texto(d, s, (ancho - ui::medir(s, 20)) / 2, y + 40, 20, Color::new(120, 230, 255, 255));
            }
        }
        // Los regalos, arriba del dialogo.
        if self.fase == Fase::Regalo {
            let w = 640.min(ancho - 40);
            let h = 56 + REGALOS.len() as i32 * 36;
            let (x, y) = ((ancho - w) / 2, alto - alto_dialogo - h - 12);
            ui::panel(d, x, y, w, h);
            ui::texto(d, "CURIOSITY LE REGALA:", x + 24, y + 12, 18, ui::LILA);
            for (o, r) in REGALOS.iter().enumerate() {
                let sel = o == self.cursor;
                let color = if sel { Color::new(255, 130, 190, 255) } else { ui::TEXTO };
                ui::texto(d, &format!("{}{}) {r}", if sel { ">" } else { " " }, o + 1), x + 24, y + 44 + o as i32 * 36, 22, color);
            }
        }
        // Las respuestas, arriba del dialogo.
        if let Fase::Pregunta(k) = self.fase {
            let w = 860.min(ancho - 40);
            let lineas: Vec<Vec<String>> = (0..3).map(|o| ui::envolver(self.respuesta(k, o).0, 22, w - 90)).collect();
            let h = 56 + lineas.iter().map(|l| l.len() as i32 * 28 + 12).sum::<i32>();
            let (x, y) = ((ancho - w) / 2, alto - alto_dialogo - h - 12);
            ui::panel(d, x, y, w, h);
            ui::texto(d, "CURIOSITY CONTESTA:", x + 24, y + 12, 18, ui::LILA);
            let mut fila = y + 42;
            for (o, l) in lineas.iter().enumerate() {
                let sel = o == self.cursor;
                let color = if sel { Color::new(255, 130, 190, 255) } else { ui::TEXTO };
                ui::texto(d, &format!("{}{})", if sel { ">" } else { " " }, o + 1), x + 24, fila, 22, color);
                for (n, linea) in l.iter().enumerate() {
                    ui::texto(d, linea, x + 66, fila + n as i32 * 28, 22, color);
                }
                fila += l.len() as i32 * 28 + 12;
            }
        }
        // La afinidad: seis corazones.
        if self.en_cita() {
            let (x, y) = (24, 24);
            ui::panel(d, x, y, 300, 84);
            ui::texto(d, &format!("AFINIDAD: {}", self.cand().nombre), x + 18, y + 12, 18, ui::LILA);
            // Los corazones del pack de pixel art (si estan; si no, dibujados a mano). Laten cuando sube.
            let late = if self.reaccion() == Some(2) { 1.0 + 0.2 * (self.t * 12.0).sin().abs() } else { 1.0 };
            let lado = 32.0 * late;
            if !ui::corazones(d, (x + 18) as f32, (y + 40) as f32 - (lado - 32.0) / 2.0, self.afinidad as f32, AFINIDAD_MAX as u32, lado) {
                for k in 0..AFINIDAD_MAX {
                    let lleno = k < self.afinidad;
                    let color = if lleno { Color::new(255, 90, 160, 255) } else { Color::new(90, 70, 110, 255) };
                    corazon(d, (x + 36 + k * 44) as f32, (y + 56) as f32, 11.0 * late, color);
                }
            }
        }
        // La golpiza: onomatopeyas de historieta que salen por todos lados.
        if let Some(g) = self.golpiza().filter(|_| self.fase == Fase::Golpiza) {
            const GOLPES: [&str; 6] = ["¡PAF!", "¡POW!", "¡CRASH!", "¡BONK!", "¡ZAS!", "¡CLONK!"];
            let colores = [Color::new(255, 220, 60, 255), Color::new(255, 90, 90, 255), Color::new(120, 230, 255, 255), Color::new(255, 150, 230, 255)];
            let n = (g * 14.0) as usize + 1;
            for k in 0..n.min(14) {
                let semilla = (k as u32).wrapping_mul(2_654_435_761);
                let x = ancho as f32 * (0.18 + 0.64 * (semilla % 1000) as f32 / 1000.0);
                let y = alto as f32 * (0.16 + 0.42 * ((semilla / 1000) % 1000) as f32 / 1000.0);
                let edad = g * 14.0 - k as f32;
                let crece = (edad * 3.0).min(1.0);
                let tam = (34.0 + (k % 3) as f32 * 12.0) * (0.6 + 0.4 * crece);
                let angulo = -14.0 + ((semilla >> 7) % 28) as f32;
                let texto = GOLPES[k % GOLPES.len()];
                // Una estrellita de fondo, como en las historietas.
                let r = tam * 1.3;
                let centro = Vector2::new(x + tam, y + tam * 0.45);
                for p in 0..10 {
                    let a0 = p as f32 / 10.0 * std::f32::consts::TAU + angulo.to_radians();
                    let a1 = a0 + std::f32::consts::TAU / 20.0;
                    let a2 = a0 + std::f32::consts::TAU / 10.0;
                    let (r0, r1) = (r, r * 0.6);
                    d.draw_triangle(centro, Vector2::new(centro.x + a1.cos() * r1, centro.y + a1.sin() * r1), Vector2::new(centro.x + a0.cos() * r0, centro.y + a0.sin() * r0), Color::new(40, 10, 30, 230));
                    d.draw_triangle(centro, Vector2::new(centro.x + a2.cos() * r0, centro.y + a2.sin() * r0), Vector2::new(centro.x + a1.cos() * r1, centro.y + a1.sin() * r1), Color::new(40, 10, 30, 230));
                }
                ui::titulo_girado_de(d, texto, x, y, tam, angulo, colores[k % colores.len()]);
            }
        }
        if self.fase == Fase::GameOver {
            d.draw_rectangle(0, 0, ancho, alto, Color::new(20, 0, 10, (suave(self.t / 0.6) * 150.0) as u8));
            let tam = 96;
            let s = "GAME OVER";
            let w = ui::medir_titulo(s, tam);
            let y = alto / 2 - 170 + (4.0 * (self.total * 2.0).sin()) as i32;
            ui::titulo_de(d, s, (ancho - w) / 2, y, tam, Color::new(255, 70, 90, 255));
            let sub = "Dos exes. Una abolladura. Cero novias.";
            ui::subtitulo(d, sub, (ancho - ui::medir(sub, 26)) / 2, y + 112, 26, ui::TEXTO);
            let consejo = "Consejo: en Marte solo hay cuatro robots. Se conocen todas.";
            ui::texto(d, consejo, (ancho - ui::medir(consejo, 20)) / 2, y + 150, 20, ui::TENUE);
            let pie = format!("{aceptar}: reiniciar a Curiosity      {salir}: despegar");
            ui::texto(d, &pie, (ancho - ui::medir(&pie, 22)) / 2, y + 196, 22, Color::new(255, 220, 255, 255));
        }
        // El final, grande, con las opciones.
        if matches!(self.fase, Fase::Final(_)) {
            let (titulo, color) = match self.final_() {
                Final::Boda => ("¡BODA EN MARTE!", Color::new(255, 150, 210, 255)),
                Final::Drama(_) => ("¡LLEGÓ LA EX!", Color::new(255, 80, 90, 255)),
                Final::Exito => ("¡CITA EXITOSA!", Color::new(255, 120, 190, 255)),
                Final::Amigos => ("FRIENDZONE EN EL PLANETA ROJO", Color::new(255, 170, 90, 255)),
                Final::Solo => ("FOREVER ALONE... PERO CON MIKU", ui::LILA),
            };
            let tam = 48;
            let w = ui::medir_titulo(titulo, tam);
            let y = 140 + (5.0 * (self.total * 3.0).sin()) as i32;
            ui::titulo_de(d, titulo, (ancho - w) / 2, y, tam, color);
            let pie = if matches!(self.final_(), Final::Drama(_)) { format!("{aceptar}: ...ay no      {salir}: ¡huir a la nave!") } else { format!("{aceptar}: otra cita      {salir}: despegar") };
            ui::texto(d, &pie, (ancho - ui::medir(&pie, 22)) / 2, y + 64, 22, ui::TEXTO);
        }
    }
}

/// Un corazon de dos circulos y un triangulo, centrado en (x, y).
fn corazon(d: &mut RaylibDrawHandle, x: f32, y: f32, tam: f32, color: Color) {
    d.draw_circle((x - tam * 0.5) as i32, y as i32, tam * 0.6, color);
    d.draw_circle((x + tam * 0.5) as i32, y as i32, tam * 0.6, color);
    d.draw_triangle(Vector2::new(x - tam * 1.08, y + tam * 0.15), Vector2::new(x, y + tam * 1.35), Vector2::new(x + tam * 1.08, y + tam * 0.15), color);
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// El final depende de como se porto Curiosity, no solo de cuanto sumo.
    #[test]
    fn remontar_ayuda_y_desinflarse_no() {
        let mut c = Cita::con_semilla(3);
        // Remonto: empezo mal (0) y termino perfecto: 4 corazones, y sale bien.
        c.puntos = vec![0, 2, 2];
        c.afinidad = 4;
        assert_eq!(c.actuacion(), Actuacion::Remonto);
        assert_eq!(c.final_(), Final::Exito);
        // Los mismos 4 corazones al reves: se desinflo y ni amigos.
        c.puntos = vec![2, 2, 0];
        assert_eq!(c.actuacion(), Actuacion::SeDesinflo);
        assert_eq!(c.final_(), Final::Solo);
        // Y lo de siempre: con 4 corazones y nada raro, amigos.
        c.puntos = vec![1, 2, 1];
        assert_eq!(c.final_(), Final::Amigos);
        // GLADAS opina al final.
        c.puntos = vec![0, 2, 2];
        assert!(c.cierre().iter().any(|(_, t, _)| t.contains("Arrancó fatal")));
    }

    /// Juega con una semilla fija, contestando lo mejor o lo peor.
    fn jugar_con(semilla: u32, candidata: usize, mejor: bool) -> Cita {
        let mut c = Cita::con_semilla(semilla);
        c.seguir_jugando(candidata, mejor);
        c
    }

    #[test]
    fn contestar_perfecto_da_boda_y_mal_deja_solo() {
        for k in 0..CANDIDATAS.len() {
            let bien = jugar_con(11, k, true);
            assert!(bien.terminada(), "la cita con {} deberia terminar", CANDIDATAS[k].nombre);
            assert_eq!(bien.final_(), Final::Boda);
            assert_eq!(bien.afinidad, AFINIDAD_MAX);
            let mal = jugar_con(11, k, false);
            assert!(mal.terminada());
            assert_eq!(mal.final_(), Final::Solo);
            assert!(mal.eventos.iter().any(|e| matches!(e, Efecto::Cumpleanos)), "solito se canta feliz cumpleanos");
        }
    }

    /// Cada pregunta tiene una respuesta de cada tipo (buena, mas o menos, mala).
    #[test]
    fn cada_pregunta_tiene_una_buena_una_regular_y_una_mala() {
        for c in &CANDIDATAS {
            for p in &c.preguntas {
                let mut puntos: Vec<i32> = p.respuestas.iter().map(|r| r.1).collect();
                puntos.sort();
                assert_eq!(puntos, vec![-1, 1, 2], "{}: {}", c.nombre, p.pregunta);
            }
        }
    }

    /// Cada cita sortea preguntas distintas (tres sin repetir) y revuelve las respuestas.
    #[test]
    fn cada_cita_sale_distinta() {
        let mut vistas = std::collections::HashSet::new();
        for semilla in 0..30 {
            let mut c = Cita::con_semilla(semilla * 7919 + 3);
            c.candidata = 2;
            c.sortear();
            let mut e = c.elegidas.to_vec();
            e.sort();
            e.dedup();
            assert_eq!(e.len(), PREGUNTAS_POR_CITA, "no se repiten preguntas");
            vistas.insert((c.elegidas, c.orden));
        }
        assert!(vistas.len() > 20, "casi todas las citas salen distintas: {}", vistas.len());
    }

    /// Si ya salio bien con una y sale bien con otra, llega la ex: drama.
    #[test]
    fn dos_citas_exitosas_dan_drama() {
        let mut c = jugar_con(5, 0, true);
        assert_eq!(c.final_(), Final::Boda);
        c.actualizar(&Entrada { aceptar: true, ..Default::default() }, 1.0);
        c.actualizar(&Entrada { aceptar: true, ..Default::default() }, 1.0);
        assert_eq!(c.fase, Fase::Elegir);
        c.seguir_jugando(3, true);
        assert_eq!(c.final_(), Final::Drama(0));
        // Despues del drama viene la golpiza y el game over; de ahi se reinicia sin exes.
        let aceptar = Entrada { aceptar: true, ..Default::default() };
        for _ in 0..10 {
            if c.fase == Fase::Golpiza {
                break;
            }
            c.actualizar(&aceptar, 1.0);
        }
        assert_eq!(c.fase, Fase::Golpiza);
        assert!(c.terminada());
        c.actualizar(&Entrada::default(), GOLPIZA + 0.1);
        assert_eq!(c.fase, Fase::GameOver);
        c.actualizar(&aceptar, 1.5);
        assert_eq!(c.fase, Fase::Elegir);
        assert!(c.exitos.is_empty(), "las exes se calman");
    }

    /// Los regalos: a cada una le encanta uno, le gusta otro, le da igual otro y odia otro.
    #[test]
    fn cada_una_tiene_su_regalo_favorito() {
        for c in &CANDIDATAS {
            let mut puntos: Vec<i32> = c.regalos.iter().map(|r| r.0).collect();
            puntos.sort();
            assert_eq!(puntos, vec![-1, 0, 1, 2], "{}", c.nombre);
        }
        for m in &MOMENTOS {
            let mut puntos: Vec<i32> = m.pregunta.respuestas.iter().map(|r| r.1).collect();
            puntos.sort();
            assert_eq!(puntos, vec![-1, 1, 2], "{}", m.pregunta.pregunta);
        }
    }

    /// Casi todas las citas traen un momento sorpresa, y no siempre el mismo.
    #[test]
    fn los_momentos_salen_variados() {
        let mut vistos = std::collections::HashSet::new();
        let mut con = 0;
        for semilla in 0..60 {
            let mut c = Cita::con_semilla(semilla * 104_729 + 1);
            c.sortear();
            if let Some(m) = c.momento {
                con += 1;
                vistos.insert(m);
            }
        }
        assert!(con > 30, "{con}");
        assert_eq!(vistos.len(), MOMENTOS.len());
    }

    #[test]
    fn otra_cita_regresa_a_elegir() {
        let mut c = jugar_con(9, 2, false);
        let aceptar = Entrada { aceptar: true, ..Default::default() };
        for _ in 0..5 {
            if c.fase == Fase::Elegir {
                break;
            }
            c.actualizar(&aceptar, 1.0);
        }
        assert_eq!(c.fase, Fase::Elegir);
    }
}
