//! Aterrizaje en la Tierra: 11:59, LA NOCHE ANTES DE LA ENTREGA.
//!
//! Son las 23:00 en el laboratorio de computo del CIT (UVG) y a las 23:59 se entregan los tres
//! proyectos del profe Koch: GRAFICAS, TEORIA DE LA COMPUTACION y SOFTWARE 2. El reloj corre de
//! verdad (una hora del juego son seis minutos). Cada proyecto avanza con su minijuego en el
//! monitor:
//! - Graficas: el render del proyecto sale roto (de cabeza, estirado, sin z-buffer, por dentro,
//!   con la luz al reves...) y hay que encontrar el arreglo. El render es de verdad: lo dibuja el
//!   mismo rasterizador, con el bug puesto.
//! - Teoria: un automata con transiciones que faltan; al completarlo corren las pruebas.
//! - Software 2: conflictos de merge; hay que quedarse con la version que pasa el test.
//!
//! Y mientras tanto: la energia se acaba (hay que ir por cafe a la maquina), el grupo de WhatsApp
//! no se calla, y pasan cosas: se reinicia Windows, se va la luz, se cae GitHub, Koch cambia la
//! rubrica, un companero pide ayuda, llega pizza, Koch hace una pregunta por Teams... A las
//! 23:59 se entrega lo que haya y Koch pone las notas.

use crate::entrada::Entrada;
use crate::matematica::Vec3;
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;
use std::cell::RefCell;

/// Lo que dura la noche en segundos de verdad (de 23:00 a 23:59).
pub const DURACION: f32 = 360.0;
const PASA_CON: u32 = 61;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Curso {
    Graficas,
    Teoria,
    Software,
}

pub const CURSOS: [Curso; 3] = [Curso::Graficas, Curso::Teoria, Curso::Software];

impl Curso {
    pub fn nombre(self) -> &'static str {
        match self {
            Curso::Graficas => "Gráficas por Computadora",
            Curso::Teoria => "Teoría de la Computación",
            Curso::Software => "Ingeniería de Software 2",
        }
    }

    fn pestana(self) -> &'static str {
        match self {
            Curso::Graficas => "proyecto3.rs",
            Curso::Teoria => "automatas.jff",
            Curso::Software => "github · sprint-final",
        }
    }

    fn color(self) -> Color {
        match self {
            Curso::Graficas => Color::new(255, 150, 90, 255),
            Curso::Teoria => Color::new(140, 200, 255, 255),
            Curso::Software => Color::new(150, 240, 150, 255),
        }
    }

    /// Cuanto sube el proyecto con cada cosa resuelta.
    fn avance(self) -> f32 {
        match self {
            Curso::Graficas => 20.0,
            Curso::Teoria => 25.0,
            Curso::Software => 20.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Graficas: el render roto
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bug {
    YInvertida,
    Aspecto,
    Winding,
    SinZBuffer,
    LuzAlReves,
    Alambre,
    Puntos,
    Fov,
    Cercano,
    OrdenMatrices,
}

struct FichaBug {
    bug: Bug,
    sintoma: &'static str,
    /// La linea de codigo sospechosa (lo que se ve en el editor).
    codigo: &'static str,
    arreglo: &'static str,
}

const BUGS: [FichaBug; 10] = [
    FichaBug { bug: Bug::YInvertida, sintoma: "Todo sale de cabeza.", codigo: "let y = (ndc.y + 1.0) * 0.5 * alto;", arreglo: "Voltear la Y al pasar a pixeles: (1 - y)" },
    FichaBug { bug: Bug::Aspecto, sintoma: "Sale estirado a lo ancho.", codigo: "Mat4::perspective(fov, 1.0, cerca, lejos)", arreglo: "Usar ancho / alto como aspecto de la proyección" },
    FichaBug { bug: Bug::Winding, sintoma: "Se ve por dentro: las caras de enfrente desaparecen.", codigo: "if area > 0.0 { continue; } // culling", arreglo: "Descartar las caras con el área al revés (el winding)" },
    FichaBug { bug: Bug::SinZBuffer, sintoma: "Lo de atrás se pinta encima de lo de adelante.", codigo: "fb.pintar(i, color); // y la profundidad?", arreglo: "Hacer el depth test antes de pintar" },
    FichaBug { bug: Bug::LuzAlReves, sintoma: "Todo sale oscuro: solo brilla la orilla de atrás.", codigo: "let l = (fragmento - luz).normalize();", arreglo: "La dirección a la luz es luz - fragmento" },
    FichaBug { bug: Bug::Alambre, sintoma: "Solo se ven las aristas.", codigo: "let modo = ModoDibujo::Alambre;", arreglo: "Rellenar los triángulos (modo relleno)" },
    FichaBug { bug: Bug::Puntos, sintoma: "Solo salen puntitos.", codigo: "let modo = ModoDibujo::Puntos;", arreglo: "Rasterizar el triángulo completo, no solo los vértices" },
    FichaBug { bug: Bug::Fov, sintoma: "Todo se ve deformado, como ojo de pez.", codigo: "let fov = 150f32.to_radians();", arreglo: "Bajar el FOV a unos 60 grados" },
    FichaBug { bug: Bug::Cercano, sintoma: "A todo se le corta el frente.", codigo: "let cerca = 2.9;", arreglo: "Acercar el plano cercano (0.1)" },
    FichaBug { bug: Bug::OrdenMatrices, sintoma: "Da vueltas alrededor del origen en vez de girar en su lugar.", codigo: "let m = rotacion * traslacion;", arreglo: "Multiplicar traslación · rotación, en ese orden" },
];

// ---------------------------------------------------------------------------
// Teoria: el automata incompleto
// ---------------------------------------------------------------------------

pub struct Automata {
    pub lenguaje: &'static str,
    pub estados: usize,
    pub finales: &'static [usize],
    /// A donde va cada estado con a y con b.
    pub delta: [[usize; 2]; 4],
}

const AUTOMATAS: [Automata; 8] = [
    Automata { lenguaje: "las cadenas que terminan en ab", estados: 3, finales: &[2], delta: [[1, 0], [1, 2], [1, 0], [0, 0]] },
    Automata { lenguaje: "las cadenas con un número par de a", estados: 2, finales: &[0], delta: [[1, 0], [0, 1], [0, 0], [0, 0]] },
    Automata { lenguaje: "las cadenas que tienen bb adentro", estados: 3, finales: &[2], delta: [[0, 1], [0, 2], [2, 2], [0, 0]] },
    Automata { lenguaje: "las cadenas que empiezan con a", estados: 3, finales: &[1], delta: [[1, 2], [1, 1], [2, 2], [0, 0]] },
    Automata { lenguaje: "las cadenas de largo par", estados: 2, finales: &[0], delta: [[1, 1], [0, 0], [0, 0], [0, 0]] },
    Automata { lenguaje: "las cadenas sin dos a seguidas", estados: 3, finales: &[0, 1], delta: [[1, 0], [2, 0], [2, 2], [0, 0]] },
    Automata { lenguaje: "las cadenas que terminan en b", estados: 2, finales: &[1], delta: [[0, 1], [0, 1], [0, 0], [0, 0]] },
    Automata { lenguaje: "las cadenas cuyo número de b es múltiplo de 3", estados: 3, finales: &[0], delta: [[0, 1], [1, 2], [2, 0], [0, 0]] },
];

impl Automata {
    fn acepta(&self, delta: &[[usize; 2]; 4], cadena: &str) -> (bool, Vec<usize>) {
        let mut q = 0;
        let mut camino = vec![0];
        for c in cadena.chars() {
            q = delta[q][if c == 'a' { 0 } else { 1 }];
            camino.push(q);
        }
        (self.finales.contains(&q), camino)
    }
}

// ---------------------------------------------------------------------------
// Software 2: los conflictos de merge
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Resolucion {
    Head,
    Rama,
    Ambas,
}

struct Conflicto {
    archivo: &'static str,
    antes: &'static [&'static str],
    head: &'static [&'static str],
    rama: &'static str,
    otra: &'static [&'static str],
    despues: &'static [&'static str],
    test: &'static str,
    correcta: Resolucion,
}

const CONFLICTOS: [Conflicto; 10] = [
    Conflicto {
        archivo: "src/notas.rs",
        antes: &["fn calcular_nota(puntos: u32) -> u32 {"],
        head: &["    puntos * 100 / 70"],
        rama: "feature/nota-maxima",
        otra: &["    (puntos * 100 / 70).min(100)"],
        despues: &["}"],
        test: "test nota_no_pasa_de_100",
        correcta: Resolucion::Rama,
    },
    Conflicto {
        archivo: "src/notas.rs",
        antes: &["fn nota_final(nota: f32) -> u32 {"],
        head: &["    nota.round() as u32"],
        rama: "hotfix/koch-no-redondea",
        otra: &["    nota.floor() as u32"],
        despues: &["}"],
        test: "test un_60_punto_9_se_queda_en_60",
        correcta: Resolucion::Rama,
    },
    Conflicto {
        archivo: "src/parqueo.rs",
        antes: &["fn hay_parqueo(hora: u32) -> bool {"],
        head: &["    hora < 7"],
        rama: "fix/realidad",
        otra: &["    false"],
        despues: &["}"],
        test: "test el_parqueo_de_la_uvg_siempre_esta_lleno",
        correcta: Resolucion::Rama,
    },
    Conflicto {
        archivo: "src/cafeteria.rs",
        antes: &["fn tomar_cafe(&mut self) {"],
        head: &["    self.energia += 50;"],
        rama: "feature/cafe-sin-overflow",
        otra: &["    self.energia = (self.energia + 50).min(100);"],
        despues: &["}"],
        test: "test la_energia_no_pasa_de_100",
        correcta: Resolucion::Rama,
    },
    Conflicto {
        archivo: "src/inscripcion.rs",
        antes: &["fn inscribir(alumno: &Alumno) -> Result<(), Error> {"],
        head: &["    validar_carne(alumno)?;"],
        rama: "feature/validar-horario",
        otra: &["    validar_horario(alumno)?;"],
        despues: &["    Ok(())", "}"],
        test: "test valida_carne_y_horario",
        correcta: Resolucion::Ambas,
    },
    Conflicto {
        archivo: "src/login.rs",
        antes: &["fn intentar(&mut self) {", "    self.intentos += 1;"],
        head: &["    if self.intentos >= 3 { self.bloquear(); }"],
        rama: "refactor/intentos",
        otra: &["    if self.intentos > 3 { self.bloquear(); }"],
        despues: &["}"],
        test: "test bloquea_al_tercer_intento",
        correcta: Resolucion::Head,
    },
    Conflicto {
        archivo: "src/login.rs",
        antes: &["fn es_valida(clave: &str, guardada: &str) -> bool {"],
        head: &["    clave == guardada"],
        rama: "security/hash",
        otra: &["    hash(clave) == guardada"],
        despues: &["}"],
        test: "test compara_hashes_no_claves",
        correcta: Resolucion::Rama,
    },
    Conflicto {
        archivo: "src/lista.rs",
        antes: &["fn sumar(lista: &[u32]) -> u32 {", "    let mut total = 0;"],
        head: &["    for i in 0..=lista.len() { total += lista[i]; }"],
        rama: "fix/indices",
        otra: &["    for i in 0..lista.len() { total += lista[i]; }"],
        despues: &["    total", "}"],
        test: "test no_se_sale_del_arreglo",
        correcta: Resolucion::Rama,
    },
    Conflicto {
        archivo: "src/entrega.rs",
        antes: &["fn a_tiempo(hora: &str) -> bool {"],
        head: &["    hora <= \"23:59\""],
        rama: "please/un-minutito-mas",
        otra: &["    hora <= \"00:30\""],
        despues: &["}"],
        test: "test koch_no_acepta_tarde",
        correcta: Resolucion::Head,
    },
    Conflicto {
        archivo: "src/cafeteria.js",
        antes: &["// el menu de la cafeteria"],
        head: &["import { cafe } from \"./menu\";"],
        rama: "feature/pizza",
        otra: &["import { pizza } from \"./menu\";"],
        despues: &["export const cena = [cafe, pizza];"],
        test: "test la_cena_trae_cafe_y_pizza",
        correcta: Resolucion::Ambas,
    },
];

// ---------------------------------------------------------------------------
// Las preguntas de Koch por Teams
// ---------------------------------------------------------------------------

struct Pregunta {
    texto: &'static str,
    opciones: [&'static str; 4],
    correcta: usize,
    porque: &'static str,
}

/// Las de Teoria de la Computacion (las de siempre de Koch).
const TEORIA: [Pregunta; 15] = [
    Pregunta {
        texto: "¿Qué tipo de lenguajes reconoce un autómata finito determinista (AFD)?",
        opciones: ["Lenguajes regulares", "Lenguajes libres de contexto", "Lenguajes recursivamente enumerables", "Solo lenguajes finitos"],
        correcta: 0,
        porque: "Un AFD reconoce exactamente los lenguajes regulares: los mismos que describen las expresiones regulares.",
    },
    Pregunta {
        texto: "¿Para qué sirve el lema de bombeo de los lenguajes regulares?",
        opciones: ["Para construir el AFD mínimo", "Para demostrar que un lenguaje NO es regular", "Para convertir un AFN en AFD", "Para demostrar que un lenguaje SÍ es regular"],
        correcta: 1,
        porque: "Solo sirve para probar que algo no es regular. Que un lenguaje lo cumpla no garantiza que sea regular.",
    },
    Pregunta {
        texto: "El lenguaje { a^n b^n | n >= 0 } es...",
        opciones: ["Regular", "Libre de contexto, pero no regular", "No es libre de contexto", "Finito"],
        correcta: 1,
        porque: "Un autómata finito no puede contar cuántas a vio; un autómata de pila sí.",
    },
    Pregunta {
        texto: "Al convertir un AFN de n estados en AFD con la construcción de subconjuntos, ¿cuántos estados puede tener el AFD en el peor caso?",
        opciones: ["n", "2n", "n²", "2^n"],
        correcta: 3,
        porque: "Cada estado del AFD es un subconjunto de estados del AFN, y hay 2^n subconjuntos.",
    },
    Pregunta {
        texto: "Para aplicar el algoritmo CYK, la gramática tiene que estar en...",
        opciones: ["Forma Normal de Greibach", "Forma Normal de Chomsky", "Forma de Backus-Naur", "Cualquier forma"],
        correcta: 1,
        porque: "CYK arma la tabla con reglas A -> BC y A -> a: eso es la Forma Normal de Chomsky.",
    },
    Pregunta {
        texto: "¿Qué máquina reconoce los lenguajes libres de contexto?",
        opciones: ["Un autómata finito", "Un autómata de pila", "Una máquina de Mealy", "Un transductor finito"],
        correcta: 1,
        porque: "La pila es la memoria que le falta al autómata finito para emparejar cosas como paréntesis.",
    },
    Pregunta {
        texto: "¿Qué hace una transición épsilon en un AFN?",
        opciones: ["Consume cualquier símbolo", "Cambia de estado sin consumir ningún símbolo", "Rechaza la cadena", "Regresa al estado inicial"],
        correcta: 1,
        porque: "Épsilon es la cadena vacía: el autómata se mueve sin leer nada de la entrada.",
    },
    Pregunta {
        texto: "El problema de la parada (halting problem) es...",
        opciones: ["Decidible en tiempo polinomial", "Decidible, pero lento", "Indecidible", "Regular"],
        correcta: 2,
        porque: "Turing demostró que ninguna máquina puede decidir, para todo programa y entrada, si se va a detener.",
    },
    Pregunta {
        texto: "Mire el pizarrón. ¿Qué cadenas acepta la expresión regular (a|b)*abb?",
        opciones: ["Las que empiezan con abb", "Las que terminan en abb", "Las que tienen exactamente una a", "Solo la cadena abb"],
        correcta: 1,
        porque: "Cualquier cosa con a y b, siempre que termine en abb: justo el AFD del pizarrón.",
    },
    Pregunta {
        texto: "La construcción de Thompson sirve para...",
        opciones: ["Minimizar un AFD", "Pasar de expresión regular a AFN", "Pasar de gramática a autómata de pila", "Demostrar indecidibilidad"],
        correcta: 1,
        porque: "Thompson arma un AFN con transiciones épsilon pieza por pieza de la expresión regular.",
    },
    Pregunta {
        texto: "¿Qué complejidad tiene el algoritmo CYK para una cadena de largo n?",
        opciones: ["O(n)", "O(n log n)", "O(n²)", "O(n³)"],
        correcta: 3,
        porque: "Llena una tabla de n² celdas y cada una prueba hasta n formas de partir la subcadena.",
    },
    Pregunta {
        texto: "Los lenguajes regulares son cerrados bajo...",
        opciones: ["Unión, intersección y complemento", "Solo la unión", "Solo la concatenación", "Ninguna operación"],
        correcta: 0,
        porque: "Con autómatas producto y cambiando estados de aceptación salen las tres.",
    },
    Pregunta {
        texto: "Los lenguajes libres de contexto NO son cerrados bajo...",
        opciones: ["La unión", "La concatenación", "La estrella de Kleene", "La intersección"],
        correcta: 3,
        porque: "a^n b^n c^m y a^m b^n c^n son libres de contexto, pero su intersección a^n b^n c^n no lo es.",
    },
    Pregunta {
        texto: "Minimizar un AFD consiste en...",
        opciones: ["Quitar las transiciones épsilon", "Juntar los estados equivalentes", "Agregar un estado de error", "Convertirlo en AFN"],
        correcta: 1,
        porque: "Dos estados son equivalentes si ninguna cadena los distingue; se juntan y el lenguaje no cambia.",
    },
    Pregunta {
        texto: "¿Qué tiene una máquina de Turing que no tiene un autómata de pila?",
        opciones: ["Estados finales", "Una cinta infinita que lee, escribe y se mueve a los dos lados", "Transiciones épsilon", "Un alfabeto"],
        correcta: 1,
        porque: "La pila solo se usa por arriba; la cinta se recorre y se reescribe donde sea. Como el cartel de la pared.",
    },
];

/// Las de Graficas por Computadora (el curso de este proyecto).
const GRAFICAS: [Pregunta; 12] = [
    Pregunta {
        texto: "¿Para qué sirve el z-buffer?",
        opciones: ["Para guardar las texturas", "Para saber qué fragmento queda adelante en cada píxel", "Para recortar lo que queda fuera de la cámara", "Para calcular las sombras"],
        correcta: 1,
        porque: "Guarda la profundidad de lo que ya se dibujó en cada píxel: lo que queda detrás no se pinta.",
    },
    Pregunta {
        texto: "¿Qué etapa del pipeline convierte los triángulos en fragmentos?",
        opciones: ["El vertex shader", "El clipping", "La rasterización", "El z-buffer"],
        correcta: 2,
        porque: "Rasterizar es decidir qué píxeles cubre cada triángulo e interpolar sus datos.",
    },
    Pregunta {
        texto: "¿Qué matriz lleva los vértices del mundo a las coordenadas de la cámara?",
        opciones: ["La de proyección", "La de modelo", "La de viewport", "La de vista"],
        correcta: 3,
        porque: "Modelo: del objeto al mundo. Vista: del mundo a la cámara. Proyección: de la cámara al clip.",
    },
    Pregunta {
        texto: "En el modelo de Phong, ¿qué componente depende de dónde está la cámara?",
        opciones: ["La especular", "La difusa", "La ambiental", "Ninguna"],
        correcta: 0,
        porque: "El brillo especular depende del ángulo entre el reflejo de la luz y la dirección a la cámara.",
    },
    Pregunta {
        texto: "¿Qué hace el back-face culling?",
        opciones: ["Quita los triángulos lejanos", "No dibuja los triángulos que le dan la espalda a la cámara", "Junta triángulos iguales", "Voltea las normales"],
        correcta: 1,
        porque: "Si la cara mira para otro lado no se puede ver: se descarta antes de rasterizar.",
    },
    Pregunta {
        texto: "Las coordenadas baricéntricas de un punto DENTRO del triángulo...",
        opciones: ["Siempre son enteras", "Suman 0", "Son positivas y suman 1", "Dependen de la cámara"],
        correcta: 2,
        porque: "Son los pesos de cada vértice: si alguno es negativo, el punto quedó afuera.",
    },
    Pregunta {
        texto: "¿Para qué se divide entre w después de la proyección en perspectiva?",
        opciones: ["Para que lo lejano se vea más pequeño", "Para normalizar las normales", "Para corregir el color", "Para quitar el z-fighting"],
        correcta: 0,
        porque: "w guarda la distancia: dividir entre ella encoge lo que está lejos.",
    },
    Pregunta {
        texto: "¿Qué es el z-fighting?",
        opciones: ["Un error de texturas", "Cuando la cámara atraviesa un objeto", "Un tipo de antialiasing", "Dos superficies casi a la misma profundidad que parpadean"],
        correcta: 3,
        porque: "La precisión del z-buffer no alcanza para decidir cuál va adelante, y cambia de cuadro a cuadro.",
    },
    Pregunta {
        texto: "El filtro bilineal de una textura...",
        opciones: ["Usa solo el texel más cercano", "Mezcla los cuatro texels que rodean al punto", "Repite la textura", "Comprime la textura"],
        correcta: 1,
        porque: "Promedia los cuatro texels vecinos según qué tan cerca queda el punto de cada uno.",
    },
    Pregunta {
        texto: "¿Qué guarda el stencil buffer?",
        opciones: ["La profundidad de cada píxel", "El color final", "Un número por píxel que sirve de máscara", "Las normales"],
        correcta: 2,
        porque: "Se marca donde se puede dibujar y donde no: así funcionan los portales y los espejos.",
    },
    Pregunta {
        texto: "Con escala NO uniforme, las normales se transforman con...",
        opciones: ["La misma matriz de modelo", "La inversa transpuesta de la matriz de modelo", "La matriz de proyección", "La identidad"],
        correcta: 1,
        porque: "Con la matriz de modelo se inclinarían mal; la inversa transpuesta las deja perpendiculares.",
    },
    Pregunta {
        texto: "¿Qué hace el clipping?",
        opciones: ["Quita las caras traseras", "Ordena los triángulos por profundidad", "Calcula la iluminación", "Recorta lo que queda fuera del volumen de visión"],
        correcta: 3,
        porque: "Lo que se sale del frustum se recorta antes de rasterizar (y no se divide entre w negativo).",
    },
];

const WHATSAPP: [(&str, &str); 22] = [
    ("Andrea", "¿alguien entendió el proyecto de teoría?"),
    ("Diego", "koch dijo que se puede entregar mañana? porfa"),
    ("Koch", "No."),
    ("Majo", "pásenme el link del repo porfa"),
    ("Pablo", "¿qué es un z-buffer?"),
    ("Sofi", "ya lo entregué jaja"),
    ("Diego", "sofi te odio"),
    ("Luis", "alguien tiene cargador tipo C?"),
    ("Andrea", "el autómata me acepta todo, ¿es normal?"),
    ("Pablo", "en mi máquina sí compila"),
    ("Majo", "se fue el wifi del CIT o soy yo"),
    ("Luis", "*manda un sticker de un gato llorando*"),
    ("Diego", "¿la cafetería sigue abierta?"),
    ("Sofi", "no, cerró a las 9"),
    ("Pablo", "mi render sale de cabeza jajaja ayuda"),
    ("Andrea", "merge conflict de 400 líneas x_x"),
    ("Luis", "¿alguien hizo el de software? pregunto por un amigo"),
    ("Majo", "¿el parcial es mañana o pasado?"),
    ("Diego", "MAÑANA????"),
    ("Pablo", "ya no siento los ojos"),
    ("Sofi", "pongan todo en una rama, no en main"),
    ("Andrea", "demasiado tarde"),
];

const RUBRICAS: [&str; 4] = [
    "Asunto: cambio en la rúbrica. Ahora el proyecto debe traer documentación. En LaTeX. Saludos.",
    "Asunto: aclaración. Los comentarios en el código cuentan. Los que dicen «no sé por qué funciona», no.",
    "Asunto: rúbrica actualizada. Se revisará el historial de commits. «asdf», «ya» y «ahora sí» no son mensajes.",
    "Asunto: recordatorio. Las pruebas unitarias valen 15 puntos. Las que fallan, menos.",
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Evento {
    /// Windows quiere reiniciar: hay que guardar antes de que llegue a cero.
    WindowsUpdate,
    /// Se esta reiniciando (pantalla azul con el porcentaje).
    Reiniciando,
    SinLuz,
    GithubCaido,
    Rubrica(usize, usize),
    /// Un companero pide ayuda (y si se le ayuda, al final pasa sus apuntes).
    Companero(usize),
    /// Explicandole al companero.
    Ayudando,
    /// Koch por Teams, con una pregunta (de Teoria o de Graficas).
    Teams(bool, usize),
    CafeAgotado,
    CiFallo,
    Pizza,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Fase {
    Llegada(usize),
    Trabajando,
    /// Yendo por cafe a la maquina.
    Cafe,
    /// Dormido sobre el teclado.
    Dormido,
    Evento(Evento),
    /// 23:59: se entrega lo que haya.
    Final,
}

const LLEGADA: [(&str, &str, Voz); 3] = [
    (ui::NOMBRE_IA, "La Tierra. La UVG, el CIT, laboratorio de cómputo. Son las 23:00. Hay tres entregas a las 23:59. Nadie ha empezado.", Voz::Ia),
    ("PROFE KOCH", "Buenas noches. Les recuerdo: Gráficas, Teoría y Software 2 se entregan hoy a las 23:59. A las 00:00:01 ya es tarde. Suerte.", Voz::Profe),
    (ui::NOMBRE_IA, "Cambie de proyecto con {izq_der}. Café con {barrel}, silenciar el grupo con {turbo}. Yo cuento los minutos. Usted, los bugs.", Voz::Ia),
];

/// Lo que se puede tocar con el raton (lo deja la interfaz al dibujarse; se usa el cuadro siguiente).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Zona {
    Opcion(usize),
    Pestana(usize),
    Boton(usize),
    Cafe,
    Silenciar,
}

/// Como esta el protagonista (para dibujarlo).
pub enum Prota {
    /// Sentado en su compu, tecleando (de 0 a 1: que tan rapido).
    Sentado(f32),
    /// Caminando por el lab (donde, hacia donde mira y el paso).
    Caminando(Vec3, f32, f32),
    /// Dormido sobre el teclado.
    Dormido,
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn mezclar(a: (Vec3, Vec3), b: (Vec3, Vec3), u: f32) -> (Vec3, Vec3) {
    (a.0.lerp(b.0, u), a.1.lerp(b.1, u))
}

pub enum AccionEntrega {
    Nada,
    Despegar,
}

struct Azar(u32);

impl Azar {
    fn n(&mut self, k: usize) -> usize {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((self.0 >> 8) as usize) % k.max(1)
    }
}

/// La ronda de Graficas: el bug, las cuatro opciones (indices en `BUGS`) y si ya se arreglo.
struct RondaGraficas {
    bug: usize,
    opciones: [usize; 4],
    modelo: usize,
}

struct RondaTeoria {
    automata: usize,
    /// Las transiciones que faltan: (estado, simbolo) y lo que se contesto.
    faltan: Vec<((usize, usize), Option<usize>)>,
    /// Las pruebas (cadena, si la acepta el bueno) y hasta donde va la animacion.
    pruebas: Vec<(String, bool)>,
    probando: Option<f32>,
}

pub struct Entrega {
    fase: Fase,
    t: f32,
    pub total: f32,
    /// Segundos de la noche (0: las 23:00; DURACION: las 23:59).
    pub reloj: f32,
    semilla: u32,
    az: Azar,
    pub progreso: [f32; 3],
    pub pestana: usize,
    cursor: usize,
    pub energia: f32,
    /// Despues de un error, unos segundos sin poder contestar.
    bloqueo: f32,
    /// Cuando se resolvio algo: (bien o mal, segundos) para el letrero.
    resultado: Option<(bool, f32)>,
    graficas: RondaGraficas,
    teoria: RondaTeoria,
    software: usize,
    ultimos: [usize; 3],
    // Las distracciones y lo que pasa.
    pub sin_leer: u32,
    silencio: f32,
    proximo_mensaje: f32,
    mensajes: Vec<(usize, f32)>,
    leidos: usize,
    proximo_evento: f32,
    github_caido: f32,
    sin_cafe: f32,
    karma: bool,
    apuntes: bool,
    eleccion: usize,
    pub se_durmio: bool,
    ultimo_evento: Option<Evento>,
    /// Un relampago (de 1 a 0) cuando truena.
    pub relampago: f32,
    mensaje: Option<(&'static str, String, Voz, f32)>,
    /// Lo que se dice despues del mensaje de ahorita (la replica del protagonista).
    cola: Option<(&'static str, String, Voz)>,
    aviso: Option<(String, f32)>,
    pub eventos: Vec<Efecto>,
    pub notas: Option<[u32; 3]>,
    /// El trueno llega un ratito despues del relampago.
    trueno_en: Option<f32>,
    /// Donde estaba el raton (para saber si se movio) y lo que se puede tocar.
    puntero_antes: Option<(f32, f32)>,
    zonas: RefCell<Vec<(Rectangle, Zona)>>,
}

impl Entrega {
    pub fn new(semilla: u32) -> Self {
        let mut az = Azar(semilla | 1);
        let graficas = RondaGraficas { bug: 0, opciones: [0, 1, 2, 3], modelo: 0 };
        let teoria = RondaTeoria { automata: 0, faltan: Vec::new(), pruebas: Vec::new(), probando: None };
        let primer_evento = 22.0 + az.n(10) as f32;
        let mut e = Entrega {
            fase: Fase::Llegada(0),
            t: 0.0,
            total: 0.0,
            reloj: 0.0,
            semilla: semilla | 1,
            az,
            progreso: [0.0; 3],
            pestana: 0,
            cursor: 0,
            energia: 0.85,
            bloqueo: 0.0,
            resultado: None,
            graficas,
            teoria,
            software: 0,
            ultimos: [99; 3],
            sin_leer: 0,
            silencio: 0.0,
            proximo_mensaje: 4.0,
            mensajes: Vec::new(),
            leidos: 0,
            proximo_evento: primer_evento,
            github_caido: 0.0,
            sin_cafe: 0.0,
            karma: false,
            apuntes: false,
            eleccion: 0,
            se_durmio: false,
            ultimo_evento: None,
            relampago: 0.0,
            mensaje: None,
            cola: None,
            aviso: None,
            eventos: Vec::new(),
            notas: None,
            trueno_en: None,
            puntero_antes: None,
            zonas: RefCell::new(Vec::new()),
        };
        e.nueva_ronda(Curso::Graficas);
        e.nueva_ronda(Curso::Teoria);
        e.nueva_ronda(Curso::Software);
        let (q, t, v) = LLEGADA[0];
        e.decir(q, t, v);
        e
    }

    fn decir(&mut self, quien: &'static str, texto: &str, voz: Voz) {
        let largo = texto.chars().count() as f32;
        self.mensaje = Some((quien, texto.to_string(), voz, 2.5 + largo / 18.0));
    }

    fn avisar(&mut self, texto: &str) {
        self.aviso = Some((texto.to_string(), 0.0));
    }

    fn cambiar(&mut self, f: Fase) {
        self.fase = f;
        self.t = 0.0;
        self.eleccion = 0;
    }

    pub fn curso(&self) -> Curso {
        CURSOS[self.pestana]
    }

    /// Las 23:xx que marca el reloj.
    pub fn minutos(&self) -> u32 {
        ((self.reloj / DURACION) * 59.0).floor().min(59.0) as u32
    }

    /// Una ronda nueva del minijuego de ese curso (sin repetir la anterior).
    fn nueva_ronda(&mut self, c: Curso) {
        let k = c as usize;
        match c {
            Curso::Graficas => {
                let mut b = self.az.n(BUGS.len());
                if b == self.ultimos[k] {
                    b = (b + 1) % BUGS.len();
                }
                self.ultimos[k] = b;
                let mut otros: Vec<usize> = (0..BUGS.len()).filter(|&i| i != b).collect();
                let mut ops = [b, 0, 0, 0];
                for slot in ops.iter_mut().skip(1) {
                    *slot = otros.remove(self.az.n(otros.len()));
                }
                for i in (1..4).rev() {
                    let j = self.az.n(i + 1);
                    ops.swap(i, j);
                }
                self.graficas = RondaGraficas { bug: b, opciones: ops, modelo: self.az.n(3) };
            }
            Curso::Teoria => {
                let mut a = self.az.n(AUTOMATAS.len());
                if a == self.ultimos[k] {
                    a = (a + 1) % AUTOMATAS.len();
                }
                self.ultimos[k] = a;
                let au = &AUTOMATAS[a];
                // Faltan una o dos transiciones (dos cuando ya va avanzado).
                let cuantas = if self.progreso[k] >= 40.0 { 2 } else { 1 };
                let mut todas: Vec<(usize, usize)> = (0..au.estados).flat_map(|q| [(q, 0), (q, 1)]).collect();
                let mut faltan = Vec::new();
                for _ in 0..cuantas {
                    let f = todas.remove(self.az.n(todas.len()));
                    faltan.push((f, None));
                }
                // Tres pruebas: al menos una que acepte y una que no.
                let mut pruebas: Vec<(String, bool)> = Vec::new();
                for intento in 0..200 {
                    let largo = 2 + self.az.n(4);
                    let cadena: String = (0..largo).map(|_| if self.az.n(2) == 0 { 'a' } else { 'b' }).collect();
                    let acepta = au.acepta(&au.delta, &cadena).0;
                    let hay = |v: bool| pruebas.iter().any(|p| p.1 == v);
                    if pruebas.iter().any(|p| p.0 == cadena) {
                        continue;
                    }
                    if pruebas.len() < 2 && hay(acepta) && intento < 150 {
                        continue;
                    }
                    pruebas.push((cadena, acepta));
                    if pruebas.len() == 3 {
                        break;
                    }
                }
                self.teoria = RondaTeoria { automata: a, faltan, pruebas, probando: None };
            }
            Curso::Software => {
                let mut s = self.az.n(CONFLICTOS.len());
                if s == self.ultimos[k] {
                    s = (s + 1) % CONFLICTOS.len();
                }
                self.ultimos[k] = s;
                self.software = s;
            }
        }
        self.cursor = 0;
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionEntrega {
        self.t += dt;
        self.total += dt;
        self.relampago = (self.relampago - dt * 2.5).max(0.0);
        if let Some(t) = &mut self.trueno_en {
            *t -= dt;
            if *t <= 0.0 {
                self.trueno_en = None;
                self.eventos.push(Efecto::Trueno);
            }
        }
        if let Some(m) = &mut self.mensaje {
            m.3 -= dt;
            if m.3 <= 0.0 {
                self.mensaje = None;
                if let Some((q, t, v)) = self.cola.take() {
                    self.decir(q, &t, v);
                }
            }
        }
        if let Some(a) = &mut self.aviso {
            a.1 += dt;
            if a.1 > 3.5 {
                self.aviso = None;
            }
        }
        for m in &mut self.mensajes {
            m.1 += dt;
        }
        self.mensajes.retain(|m| m.1 < 3.5);
        if let Some(r) = &mut self.resultado {
            r.1 += dt;
        }
        if e.aterrizar {
            return AccionEntrega::Despegar;
        }
        // La noche corre (menos en la llegada y al final).
        let corre = !matches!(self.fase, Fase::Llegada(_) | Fase::Final);
        if corre {
            self.reloj += dt;
            self.energia = (self.energia - dt / 125.0).max(0.0);
            self.bloqueo = (self.bloqueo - dt).max(0.0);
            self.github_caido = (self.github_caido - dt).max(0.0);
            self.sin_cafe = (self.sin_cafe - dt).max(0.0);
            self.silencio = (self.silencio - dt).max(0.0);
            self.whatsapp(dt);
            // Truena de vez en cuando (llueve sobre el campus): el relampago y, un ratito despues, el trueno.
            if self.az.n(1100) == 0 && self.relampago <= 0.0 {
                self.relampago = 1.0;
                self.trueno_en = Some(0.3 + self.az.n(6) as f32 * 0.12);
            }
            // Las 23:55: si se ayudo al companero, pasa sus apuntes.
            if self.karma && !self.apuntes && self.reloj > DURACION - 40.0 {
                self.apuntes = true;
                let peor = (0..3).min_by(|&a, &b| self.progreso[a].total_cmp(&self.progreso[b])).unwrap_or(0);
                self.progreso[peor] = (self.progreso[peor] + 12.0).min(100.0);
                self.avisar(&format!("Su compañero le pasó sus apuntes: +12% en {}", CURSOS[peor].nombre()));
                self.eventos.push(Efecto::Acierto);
            }
            if self.reloj >= DURACION {
                self.entregar();
                return AccionEntrega::Nada;
            }
            // El ultimo minuto: el corazon a mil.
            if self.reloj > DURACION - 30.0 && ((self.reloj - dt) * 1.5) as i32 != (self.reloj * 1.5) as i32 {
                self.eventos.push(Efecto::Latido);
            }
        }
        let siguiente = (e.aceptar && self.t > 0.4) || (self.mensaje.is_none() && self.t > 0.6);
        let puntero_antes = self.puntero_antes;
        self.puntero_antes = e.puntero;
        match self.fase {
            Fase::Llegada(i) if siguiente => {
                if i + 1 < LLEGADA.len() {
                    let (q, t, v) = LLEGADA[i + 1];
                    self.decir(q, t, v);
                    self.cambiar(Fase::Llegada(i + 1));
                } else {
                    self.mensaje = None;
                    self.cambiar(Fase::Trabajando);
                }
            }
            Fase::Trabajando => self.trabajar(e, dt, puntero_antes),
            Fase::Llegada(_) if e.clic => {
                self.mensaje = None;
                self.cambiar(Fase::Trabajando);
            }
            Fase::Cafe => {
                if self.t > 4.0 {
                    self.energia = (self.energia + 0.6).min(1.0);
                    self.eventos.push(Efecto::Acierto);
                    self.avisar("Café de máquina: +60% de energía. Sabe a examen.");
                    self.cambiar(Fase::Trabajando);
                }
            }
            Fase::Dormido => {
                if self.t > 8.0 {
                    self.energia = 0.35;
                    self.avisar("¡Se durmió sobre el teclado! Ahora el código dice «jjjjjjjjjjjjjjjj».");
                    self.cambiar(Fase::Trabajando);
                }
            }
            Fase::Evento(ev) => self.atender(ev, e, puntero_antes),
            Fase::Final if (e.aceptar || e.clic) && self.t > 2.5 => {
                // Otra noche (distinta).
                *self = Entrega::new(self.semilla.wrapping_mul(2_654_435_761).wrapping_add(59));
                self.mensaje = None;
                self.cambiar(Fase::Trabajando);
            }
            _ => {}
        }
        AccionEntrega::Nada
    }

    fn whatsapp(&mut self, dt: f32) {
        self.proximo_mensaje -= dt;
        if self.proximo_mensaje > 0.0 {
            return;
        }
        self.proximo_mensaje = 6.0 + self.az.n(5) as f32;
        if self.silencio > 0.0 {
            return;
        }
        self.sin_leer += 1;
        let k = self.leidos % WHATSAPP.len();
        self.leidos += 1;
        // Con muchos sin leer, las notificaciones tapan la pantalla.
        if self.sin_leer >= 3 {
            self.mensajes.push((k, 0.0));
            self.eventos.push(Efecto::Whatsapp);
        }
    }

    /// Lo que hay bajo el raton.
    fn bajo_el_raton(&self, e: &Entrada) -> Option<Zona> {
        let (x, y) = e.puntero?;
        self.zonas.borrow().iter().rev().find(|(r, _)| r.check_collision_point_rec(Vector2::new(x, y))).map(|(_, z)| *z)
    }

    /// Si el raton se movio este cuadro (el cursor sigue al raton solo si se esta usando).
    fn raton_se_movio(e: &Entrada, antes: Option<(f32, f32)>) -> bool {
        match (e.puntero, antes) {
            (Some(a), Some(b)) => (a.0 - b.0).abs() + (a.1 - b.1).abs() > 0.5,
            _ => false,
        }
    }

    fn trabajar(&mut self, e: &Entrada, dt: f32, antes: Option<(f32, f32)>) {
        let zona = self.bajo_el_raton(e);
        let clic = if e.clic { zona } else { None };
        // Sin energia: se duerme.
        if self.energia <= 0.0 {
            self.se_durmio = true;
            self.eventos.push(Efecto::Fallo);
            self.cambiar(Fase::Dormido);
            return;
        }
        if let Some(Zona::Pestana(k)) = clic {
            if k != self.pestana {
                self.pestana = k;
                self.cursor = 0;
                self.resultado = None;
                self.eventos.push(Efecto::Carta);
            }
        }
        if e.izquierda {
            self.pestana = (self.pestana + 2) % 3;
            self.cursor = 0;
            self.resultado = None;
            self.eventos.push(Efecto::Carta);
        }
        if e.derecha {
            self.pestana = (self.pestana + 1) % 3;
            self.cursor = 0;
            self.resultado = None;
            self.eventos.push(Efecto::Carta);
        }
        if e.barrel || clic == Some(Zona::Cafe) {
            if self.sin_cafe > 0.0 {
                self.avisar("La máquina dice: AGOTADO. Como usted.");
                self.eventos.push(Efecto::Fallo);
            } else {
                self.eventos.push(Efecto::Puerta);
                self.cambiar(Fase::Cafe);
                return;
            }
        }
        if (e.turbo || clic == Some(Zona::Silenciar)) && (self.sin_leer > 0 || !self.mensajes.is_empty()) {
            self.sin_leer = 0;
            self.mensajes.clear();
            self.silencio = 40.0;
            self.avisar("Grupo silenciado por 40 segundos. Paz.");
        }
        // Los desastres.
        self.proximo_evento -= dt;
        if self.proximo_evento <= 0.0 {
            self.proximo_evento = 28.0 + self.az.n(18) as f32;
            self.lanzar_evento();
            return;
        }
        // El minijuego de la pestana.
        let c = self.curso();
        if self.progreso[c as usize] >= 100.0 {
            return;
        }
        // Despues de resolver, un ratito mostrando como quedo; luego la siguiente.
        if let Some((bien, t)) = self.resultado {
            let espera = if c == Curso::Teoria { 3.4 } else { 1.4 };
            if t > espera {
                self.resultado = None;
                if bien || c == Curso::Teoria {
                    self.nueva_ronda(c);
                }
            }
            return;
        }
        if self.bloqueo > 0.0 || (c == Curso::Software && self.github_caido > 0.0) {
            return;
        }
        let opciones = match c {
            Curso::Graficas => 4,
            Curso::Teoria => AUTOMATAS[self.teoria.automata].estados,
            Curso::Software => 3,
        };
        if e.arriba {
            self.cursor = (self.cursor + opciones - 1) % opciones;
        }
        if e.abajo {
            self.cursor = (self.cursor + 1) % opciones;
        }
        // El raton: pasar por encima escoge, el clic contesta.
        if let (Some(Zona::Opcion(k)), true) = (zona, Self::raton_se_movio(e, antes)) {
            if k < opciones {
                self.cursor = k;
            }
        }
        let por_clic = match clic {
            Some(Zona::Opcion(k)) if k < opciones => Some(k),
            _ => None,
        };
        let elegida = e.respuesta.filter(|&r| r < opciones).or(if e.aceptar { Some(self.cursor) } else { por_clic });
        // Con poca energia, a veces no le atina al boton.
        let elegida = if self.energia < 0.15 && elegida.is_some() && self.az.n(3) == 0 {
            self.avisar("...zzz... ¿qué? ¿dónde estaba?");
            None
        } else {
            elegida
        };
        if let Some(r) = elegida {
            self.contestar(c, r);
        }
    }

    fn contestar(&mut self, c: Curso, r: usize) {
        let k = c as usize;
        let bien = match c {
            Curso::Graficas => self.graficas.opciones[r] == self.graficas.bug,
            Curso::Software => {
                let correcta = CONFLICTOS[self.software].correcta;
                [Resolucion::Head, Resolucion::Rama, Resolucion::Ambas][r] == correcta
            }
            Curso::Teoria => {
                // Se contesta la siguiente transicion que falta; al completarlas, corren las pruebas.
                if let Some(f) = self.teoria.faltan.iter_mut().find(|f| f.1.is_none()) {
                    f.1 = Some(r);
                }
                self.cursor = 0;
                if self.teoria.faltan.iter().any(|f| f.1.is_none()) {
                    self.eventos.push(Efecto::Carta);
                    return;
                }
                self.teoria.probando = Some(0.0);
                let au = &AUTOMATAS[self.teoria.automata];
                self.teoria.faltan.iter().all(|((q, s), elegido)| *elegido == Some(au.delta[*q][*s]))
            }
        };
        self.resultado = Some((bien, 0.0));
        self.eventos.push(Efecto::Teclas);
        if bien {
            self.progreso[k] = (self.progreso[k] + c.avance()).min(100.0);
            self.eventos.push(Efecto::Perfecto);
            if self.progreso[k] >= 100.0 {
                self.avisar(&format!("¡{} listo! Subido y entregado.", c.nombre()));
                self.eventos.push(Efecto::Sello);
            }
        } else {
            self.progreso[k] = (self.progreso[k] - 4.0).max(0.0);
            self.bloqueo = 2.0;
            self.eventos.push(Efecto::Fallo);
        }
    }

    fn lanzar_evento(&mut self) {
        // Koch por Teams sale seguro una vez; lo demas, al azar (sin repetir el anterior).
        let opciones: [Evento; 9] = [
            Evento::WindowsUpdate,
            Evento::SinLuz,
            Evento::GithubCaido,
            Evento::Rubrica(self.az.n(3), self.az.n(RUBRICAS.len())),
            Evento::Companero(self.az.n(4)),
            Evento::Teams(self.az.n(2) == 0, self.az.n(15)),
            Evento::CafeAgotado,
            Evento::CiFallo,
            Evento::Pizza,
        ];
        let mut ev = opciones[self.az.n(opciones.len())];
        if let Some(antes) = self.ultimo_evento {
            if std::mem::discriminant(&antes) == std::mem::discriminant(&ev) {
                ev = Evento::Teams(self.az.n(2) == 0, self.az.n(15));
            }
        }
        self.ultimo_evento = Some(ev);
        match ev {
            Evento::GithubCaido => {
                self.github_caido = 20.0;
                self.avisar("GitHub está caído. Otra vez. El unicornio dice que lo siente.");
                self.eventos.push(Efecto::Fallo);
            }
            Evento::CafeAgotado => {
                self.sin_cafe = 35.0;
                self.avisar("La máquina de café se quedó sin café. Hay agua caliente. Y tristeza.");
                self.eventos.push(Efecto::Fallo);
            }
            Evento::CiFallo => {
                let k = Curso::Software as usize;
                self.progreso[k] = (self.progreso[k] - 8.0).max(0.0);
                self.avisar("El build de main falló: «funciona en mi máquina». -8% en Software 2.");
                self.eventos.push(Efecto::Alarma);
            }
            Evento::Pizza => {
                self.energia = (self.energia + 0.4).min(1.0);
                self.avisar("¡Alguien pidió pizza para todo el lab! +40% de energía.");
                self.eventos.push(Efecto::Perfecto);
            }
            _ => {
                self.eventos.push(if matches!(ev, Evento::SinLuz) { Efecto::Choque } else { Efecto::Alarma });
                if matches!(ev, Evento::SinLuz) {
                    self.relampago = 1.0;
                }
                self.cambiar(Fase::Evento(ev));
            }
        }
    }

    fn atender(&mut self, ev: Evento, e: &Entrada, antes: Option<(f32, f32)>) {
        let zona = self.bajo_el_raton(e);
        let movio = Self::raton_se_movio(e, antes);
        // Con el raton: pasar por encima escoge y el clic confirma (como si se apretara aceptar).
        let clic = e.clic && matches!(zona, Some(Zona::Boton(_) | Zona::Opcion(_)));
        let elegir = |s: &mut Self, n: usize| {
            if e.izquierda || e.arriba {
                s.eleccion = (s.eleccion + n - 1) % n;
            }
            if e.derecha || e.abajo {
                s.eleccion = (s.eleccion + 1) % n;
            }
            if let Some(Zona::Boton(k) | Zona::Opcion(k)) = zona {
                if (movio || clic) && k < n {
                    s.eleccion = k;
                }
            }
        };
        let aceptar = e.aceptar || clic || (e.clic && matches!(ev, Evento::Rubrica(..)));
        match ev {
            Evento::WindowsUpdate => {
                elegir(self, 2);
                if (aceptar && self.t > 0.3) || self.t > 8.0 {
                    if self.t <= 8.0 && self.eleccion == 0 {
                        self.avisar("Guardado. Windows dice que se reinicia «más tarde». Mentira, pero bueno.");
                        self.eventos.push(Efecto::Acierto);
                        self.cambiar(Fase::Trabajando);
                    } else {
                        let k = self.pestana;
                        self.progreso[k] = (self.progreso[k] - 12.0).max(0.0);
                        self.eventos.push(Efecto::Fallo);
                        self.cambiar(Fase::Evento(Evento::Reiniciando));
                    }
                }
            }
            Evento::Reiniciando => {
                if self.t > 5.0 {
                    self.avisar(&format!("Volvió. Sin guardar: -12% en {}.", self.curso().nombre()));
                    self.cambiar(Fase::Trabajando);
                }
            }
            Evento::SinLuz => {
                if self.t > 6.0 {
                    self.avisar("Volvió la luz. Las computadoras del CIT tienen UPS. Su dignidad, no.");
                    self.cambiar(Fase::Trabajando);
                }
            }
            Evento::Rubrica(k, _) => {
                if aceptar && self.t > 0.6 {
                    self.progreso[k] = (self.progreso[k] - 15.0).max(0.0);
                    self.avisar(&format!("Rúbrica nueva: -15% en {}.", CURSOS[k].nombre()));
                    self.cambiar(Fase::Trabajando);
                }
            }
            Evento::Companero(_) => {
                elegir(self, 2);
                if aceptar && self.t > 0.3 {
                    if self.eleccion == 0 {
                        self.karma = true;
                        self.eventos.push(Efecto::Acierto);
                        self.cambiar(Fase::Evento(Evento::Ayudando));
                    } else {
                        self.avisar("«Ahorita no». Lo dice todo el CIT. Siempre.");
                        self.cambiar(Fase::Trabajando);
                    }
                }
            }
            Evento::Ayudando => {
                if self.t > 6.0 {
                    self.avisar("Le explicó. Le debe una. (Las deudas en el CIT se pagan.)");
                    self.cambiar(Fase::Trabajando);
                }
            }
            Evento::Teams(teoria, i) => {
                let p = if teoria { &TEORIA[i % TEORIA.len()] } else { &GRAFICAS[i % GRAFICAS.len()] };
                elegir(self, 4);
                let elegida = e.respuesta.filter(|&r| r < 4).or(if aceptar && self.t > 0.4 { Some(self.eleccion) } else { None });
                if let Some(r) = elegida.filter(|_| self.t > 0.3) {
                    let k = if teoria { Curso::Teoria } else { Curso::Graficas } as usize;
                    if r == p.correcta {
                        self.progreso[k] = (self.progreso[k] + 8.0).min(100.0);
                        self.decir("PROFE KOCH", "Correcto. Ocho puntos más. No se acostumbre.", Voz::Profe);
                        self.eventos.push(Efecto::Perfecto);
                    } else {
                        let porque = format!("Mmm. No. {} Anótelo.", p.porque);
                        self.decir("PROFE KOCH", &porque, Voz::Profe);
                        // Y el protagonista contesta como contesta cada quien ("¡Ay, profe!...").
                        let yo = ui::protagonista();
                        self.cola = Some((yo.nombre, crate::personajes::replica(yo.id, crate::personajes::Replica::Golpe).to_string(), Voz::Miku));
                        self.eventos.push(Efecto::Fallo);
                    }
                    self.cambiar(Fase::Trabajando);
                } else if self.t > 14.0 {
                    self.decir("PROFE KOCH", "¿Hola? ¿Se me fue la conexión o se me fue usted?", Voz::Profe);
                    self.cambiar(Fase::Trabajando);
                }
            }
            _ => self.cambiar(Fase::Trabajando),
        }
    }

    /// 23:59: se entrega lo que haya, y Koch pone las notas.
    fn entregar(&mut self) {
        let notas = self.progreso.map(|p| p.round().clamp(0.0, 100.0) as u32);
        self.notas = Some(notas);
        let promedio = notas.iter().sum::<u32>() / 3;
        let linea = if notas.iter().all(|&n| n >= 100) {
            "Tres de tres al cien. Voy a revisar si hubo copia. ...No hubo. Felicidades. De verdad."
        } else if notas.iter().all(|&n| n >= PASA_CON) {
            "Aprobado. En los tres. Raspando en alguno, pero aprobado. Así se pasa en la U."
        } else if promedio >= PASA_CON {
            "El promedio pasa, pero hay un curso perdido. Nos vemos en la recuperación."
        } else {
            "Mmm. Nos vemos el próximo semestre. Traiga café."
        };
        self.decir("PROFE KOCH", linea, Voz::Profe);
        self.eventos.push(if promedio >= PASA_CON { Efecto::Fuego } else { Efecto::Fallo });
        self.cambiar(Fase::Final);
    }

    // -----------------------------------------------------------------------
    // Para dibujar y para el juego
    // -----------------------------------------------------------------------

    pub fn hablante(&self) -> Option<(&'static str, String, Voz)> {
        self.mensaje.as_ref().map(|(q, t, v, _)| (*q, t.clone(), *v))
    }

    pub fn terminado(&self) -> bool {
        self.fase == Fase::Final
    }

    pub fn sin_luz(&self) -> bool {
        matches!(self.fase, Fase::Evento(Evento::SinLuz))
    }

    pub fn dormido(&self) -> bool {
        self.fase == Fase::Dormido
    }

    pub fn llegando(&self) -> bool {
        matches!(self.fase, Fase::Llegada(_))
    }

    /// El render que hay que mostrar en el monitor (Graficas): el bug, si ya se arreglo, y que modelo.
    pub fn vista_bug(&self) -> Option<(Bug, bool, usize)> {
        let trabajando = matches!(self.fase, Fase::Trabajando | Fase::Evento(Evento::Teams(..)) | Fase::Evento(Evento::Rubrica(..)) | Fase::Evento(Evento::Companero(_)));
        if !trabajando || self.curso() != Curso::Graficas || self.progreso[0] >= 100.0 {
            return None;
        }
        let arreglado = matches!(self.resultado, Some((true, _)));
        Some((BUGS[self.graficas.bug].bug, arreglado, self.graficas.modelo))
    }

    /// Las notas (si ya se entrego) y si se durmio, para los logros.
    pub fn resultado_final(&self) -> Option<[u32; 3]> {
        self.notas
    }

    /// El guardia del CIT: donde va, hacia donde mira y su paso (si anda por ahi). Hace su ronda
    /// dos veces en la noche (de la puerta de atras al fondo y de regreso) y entra con su linterna
    /// cuando se va la luz.
    pub fn guardia(&self) -> Option<(Vec3, f32, f32)> {
        let puerta = Vec3::new(-3.9, 0.0, 4.9);
        let pasillo = Vec3::new(-3.9, 0.0, 4.0);
        let fondo = Vec3::new(2.6, 0.0, 4.0);
        let caminar = |a: Vec3, b: Vec3, u: f32, t: f32| {
            let d = b - a;
            (a.lerp(b, u.clamp(0.0, 1.0)), (-d.x).atan2(-d.z), t * 7.0)
        };
        if self.sin_luz() {
            let u = (self.t / 3.0).min(1.0);
            let adentro = Vec3::new(-2.6, 0.0, 3.6);
            return Some(if u < 1.0 { caminar(puerta, adentro, u, self.t) } else { (adentro, -0.25, 0.0) });
        }
        for inicio in [DURACION * 0.3, DURACION * 0.68] {
            let t = self.reloj - inicio;
            if (0.0..16.0).contains(&t) {
                let tramo = |a: f32, b: f32| ((t - a) / (b - a)).clamp(0.0, 1.0);
                return Some(match t {
                    t if t < 1.0 => caminar(puerta, pasillo, tramo(0.0, 1.0), t),
                    t if t < 7.0 => caminar(pasillo, fondo, tramo(1.0, 7.0), t),
                    // Se queda viendo el lab un ratito (con cara de "ya vamos a cerrar").
                    t if t < 9.0 => (fondo, 0.0, 0.0),
                    t if t < 15.0 => caminar(fondo, pasillo, tramo(9.0, 15.0), t),
                    t => caminar(pasillo, puerta, tramo(15.0, 16.0), t),
                });
            }
        }
        None
    }

    pub fn sin_cafe(&self) -> bool {
        self.sin_cafe > 0.0
    }

    /// De 0 a 1 en el ultimo minuto: la musica se acelera y la pantalla late en rojo.
    pub fn nervios(&self) -> f32 {
        if self.terminado() {
            return 0.0;
        }
        ((self.reloj - (DURACION - 60.0)) / 60.0).clamp(0.0, 1.0)
    }

    /// Que tan rapido va la musica (se acelera en el ultimo minuto).
    pub fn velocidad_musica(&self) -> f32 {
        1.0 + 0.14 * self.nervios()
    }

    /// Los segundos que lleva la llamada de Koch por Teams, si esta llamando.
    pub fn teams(&self) -> Option<f32> {
        matches!(self.fase, Fase::Evento(Evento::Teams(..))).then_some(self.t)
    }

    /// Los angulos de las manecillas del reloj de pared (horas, minutos), en radianes desde las 12.
    pub fn manecillas(&self) -> (f32, f32) {
        let m = (self.reloj / DURACION * 59.0).min(59.0) + if self.terminado() { 1.0 } else { 0.0 };
        let tau = std::f32::consts::TAU;
        ((11.0 + m / 60.0) / 12.0 * tau, m / 60.0 * tau)
    }

    /// Como esta el protagonista. `mio`: su escritorio (en el piso).
    pub fn prota(&self, mio: Vec3) -> Prota {
        match self.fase {
            Fase::Dormido => Prota::Dormido,
            Fase::Cafe => {
                // Va a la maquina de cafe del fondo (por el pasillo), espera y regresa.
                let silla = mio + Vec3::new(0.0, 0.0, 0.55);
                let camino = [silla, Vec3::new(0.0, 0.0, silla.z), Vec3::new(0.0, 0.0, 3.9), Vec3::new(3.75, 0.0, 4.3)];
                let (ida, vuelta) = (1.7, 2.6);
                let (u, regreso) = if self.t < ida { (self.t / ida, false) } else if self.t < vuelta { (1.0, false) } else { (1.0 - (self.t - vuelta) / 1.4, true) };
                let u = u.clamp(0.0, 1.0) * 3.0;
                let k = (u.floor() as usize).min(2);
                let (a, b) = (camino[k], camino[k + 1]);
                let p = a.lerp(b, u - k as f32);
                let dir = if regreso { a - b } else { b - a };
                let rumbo = (-dir.x).atan2(-dir.z);
                let quieto = self.t >= ida && self.t < vuelta;
                Prota::Caminando(p, if quieto { std::f32::consts::PI } else { rumbo }, if quieto { 0.0 } else { self.t * 9.0 })
            }
            _ => {
                // Teclea: suave pensando, rapidisimo justo despues de contestar.
                let fuerza = match self.resultado {
                    Some((_, t)) if t < 0.6 => 1.0,
                    _ if self.fase == Fase::Trabajando => 0.45,
                    _ => 0.15,
                };
                Prota::Sentado(fuerza)
            }
        }
    }

    /// La camara: atras del hombro del protagonista mientras trabaja; entrando por la puerta al
    /// llegar; hacia la maquina de cafe; al proyector cuando llama Koch; y de lejos al final.
    pub fn camara(&self, mio: Vec3) -> (Vec3, Vec3) {
        let respira = Vec3::new((self.total * 0.37).sin() * 0.025, (self.total * 0.53).sin() * 0.018, 0.0);
        let hombro = (mio + Vec3::new(1.15, 1.95, 2.05), mio + Vec3::new(-0.3, 1.0, -0.55));
        let (ojo, mira) = match self.fase {
            Fase::Llegada(_) => {
                // Entra por la puerta de atras, cruza el lab oscuro y se acerca a la compu.
                let u = suave(self.total / 12.0);
                let puerta = (Vec3::new(-3.9, 1.85, 4.7), Vec3::new(-1.6, 1.3, -3.5));
                let medio = (Vec3::new(-0.2, 2.1, 2.6), Vec3::new(-1.0, 1.2, -2.0));
                if u < 0.5 { mezclar(puerta, medio, suave(u * 2.0)) } else { mezclar(medio, hombro, suave(u * 2.0 - 1.0)) }
            }
            Fase::Cafe => {
                let cafetera = (Vec3::new(1.4, 1.75, 1.2), Vec3::new(3.6, 1.0, 4.6));
                mezclar(hombro, cafetera, suave(self.t / 0.9) * (1.0 - suave((self.t - 3.3) / 0.8)))
            }
            Fase::Dormido => {
                let cerca = (mio + Vec3::new(1.0, 1.25, 0.6), mio + Vec3::new(0.0, 0.95, 0.15));
                mezclar(hombro, cerca, suave(self.t / 1.2))
            }
            Fase::Evento(Evento::Teams(..)) => {
                // Hacia el proyector, dejando a Koch a la izquierda (la llamada va a la derecha).
                let proyector = (mio + Vec3::new(1.2, 1.75, 1.3), Vec3::new(2.6, 1.55, -5.4));
                mezclar(hombro, proyector, suave(self.t / 0.8))
            }
            Fase::Final => {
                let lejos = (Vec3::new(1.2, 2.85, 5.0), Vec3::new(-0.9, 0.9, -2.2));
                mezclar(hombro, lejos, suave(self.t / 3.0))
            }
            _ => hombro,
        };
        (ojo + respira, mira + respira * 0.5)
    }

    /// Para las capturas: deja la noche en un momento (`entrega_graficas`, `entrega_teams`...).
    pub fn preparar_captura(&mut self, modo: &str) {
        if modo == "entrega" {
            return;
        }
        self.mensaje = None;
        self.cambiar(Fase::Trabajando);
        self.proximo_evento = 1e9;
        self.proximo_mensaje = 1e9;
        self.reloj = DURACION * 0.55;
        self.progreso = [40.0, 25.0, 60.0];
        self.energia = 0.62;
        self.sin_leer = 5;
        match modo {
            "entrega_teoria" => self.pestana = 1,
            "entrega_software" => self.pestana = 2,
            "entrega_teams" => {
                self.cambiar(Fase::Evento(Evento::Teams(true, 3)));
            }
            "entrega_windows" => self.cambiar(Fase::Evento(Evento::WindowsUpdate)),
            "entrega_luz" => {
                self.relampago = 1.0;
                self.cambiar(Fase::Evento(Evento::SinLuz));
            }
            "entrega_cafe" => self.cambiar(Fase::Cafe),
            "entrega_sueno" => self.cambiar(Fase::Dormido),
            "entrega_companero" => self.cambiar(Fase::Evento(Evento::Companero(1))),
            "entrega_rubrica" => self.cambiar(Fase::Evento(Evento::Rubrica(0, 2))),
            "entrega_whatsapp" => {
                for k in 0..3 {
                    self.mensajes.push((k + 4, 0.5));
                }
            }
            "entrega_nervios" => self.reloj = DURACION - 20.0,
            "entrega_final" => {
                self.progreso = [100.0, 74.0, 58.0];
                self.reloj = DURACION;
                self.entregar();
            }
            _ => {}
        }
    }

    pub fn despedida(&self) -> String {
        match self.notas {
            Some(n) => format!("Despegue de la Tierra. Notas: Gráficas {}, Teoría {}, Software 2 {}. Koch ya está calificando lo de mañana.", n[0], n[1], n[2]),
            None => format!("Despegue de emergencia a las 23:{:02}. Los proyectos se quedaron en el CIT. Koch no acepta excusas espaciales.", self.minutos()),
        }
    }
}

// ---------------------------------------------------------------------------
// La interfaz
// ---------------------------------------------------------------------------

/// Una flecha de `a` a `b` curvada (`curva` px a la izquierda del camino), con la punta al final
/// y la etiqueta a la mitad.
fn flecha(d: &mut RaylibDrawHandle, a: Vector2, b: Vector2, curva: f32, etiqueta: &str, color: Color, punteada: bool) {
    let medio = Vector2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let largo = (dx * dx + dy * dy).sqrt().max(1.0);
    let normal = Vector2::new(-dy / largo, dx / largo);
    let control = Vector2::new(medio.x + normal.x * curva, medio.y + normal.y * curva);
    let punto = |t: f32| {
        let u = 1.0 - t;
        Vector2::new(u * u * a.x + 2.0 * u * t * control.x + t * t * b.x, u * u * a.y + 2.0 * u * t * control.y + t * t * b.y)
    };
    let pasos = 18;
    for k in 0..pasos {
        if punteada && k % 2 == 1 {
            continue;
        }
        d.draw_line_ex(punto(k as f32 / pasos as f32), punto((k + 1) as f32 / pasos as f32), 2.5, color);
    }
    let fin = punto(1.0);
    let antes = punto(0.9);
    let (ux, uy) = (fin.x - antes.x, fin.y - antes.y);
    let l = (ux * ux + uy * uy).sqrt().max(0.01);
    let (ux, uy) = (ux / l, uy / l);
    let izq = Vector2::new(fin.x - ux * 13.0 + uy * 6.5, fin.y - uy * 13.0 - ux * 6.5);
    let der = Vector2::new(fin.x - ux * 13.0 - uy * 6.5, fin.y - uy * 13.0 + ux * 6.5);
    d.draw_triangle(fin, izq, der, color);
    d.draw_triangle(fin, der, izq, color);
    if !etiqueta.is_empty() {
        let et = punto(0.5);
        let w = ui::medir(etiqueta, 17) + 10;
        d.draw_rectangle_rounded(Rectangle::new(et.x - w as f32 / 2.0, et.y - 24.0, w as f32, 22.0), 0.5, 6, Color::new(24, 26, 36, 230));
        ui::texto(d, etiqueta, et.x as i32 - ui::medir(etiqueta, 17) / 2, et.y as i32 - 22, 17, color);
    }
}

/// Los colores de la compu (un tema oscuro como el de los editores).
const BISEL: Color = Color::new(14, 14, 18, 255);
const FONDO_EDITOR: Color = Color::new(30, 31, 38, 255);
const BARRA: Color = Color::new(40, 41, 50, 255);
const BARRA_OSCURA: Color = Color::new(24, 25, 31, 255);
const LINEA: Color = Color::new(58, 60, 74, 255);
const ACENTO: Color = Color::new(110, 140, 255, 255);
const BIEN: Color = Color::new(110, 240, 140, 255);
const MAL: Color = Color::new(255, 105, 115, 255);

fn con_alfa(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, (c.a as f32 * a.clamp(0.0, 1.0)) as u8)
}

/// Un rectangulo de esquinas redondas con radio en pixeles.
fn caja(d: &mut RaylibDrawHandle, x: f32, y: f32, w: f32, h: f32, radio: f32, color: Color) {
    let r = (2.0 * radio / w.min(h).max(1.0)).min(1.0);
    d.draw_rectangle_rounded(Rectangle::new(x, y, w, h), r, 8, color);
}

impl Entrega {
    /// Apunta una zona que se puede tocar con el raton.
    fn zona(&self, x: f32, y: f32, w: f32, h: f32, z: Zona) {
        self.zonas.borrow_mut().push((Rectangle::new(x, y, w, h), z));
    }

    /// Si el raton esta encima de esta zona.
    fn encima(e: &Entrada, x: f32, y: f32, w: f32, h: f32) -> bool {
        e.puntero.is_some_and(|(px, py)| px >= x && px < x + w && py >= y && py < y + h)
    }

    /// `monitor`: el render del proyecto de Graficas (con el bug); `koch`: su retrato (para Teams).
    #[allow(clippy::too_many_arguments)]
    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, monitor: &Texture2D, koch: Option<&Texture2D>, ancho: i32, alto: i32) {
        self.zonas.borrow_mut().clear();
        let (w, h) = (ancho as f32, alto as f32);
        let (aceptar, salir) = (&ui::boton(e.nombre("aceptar")), &ui::boton(e.nombre("aterrizar")));
        if self.llegando() {
            self.dibujar_llegada(d, e, ancho, alto);
        } else if self.se_ve_la_compu() {
            self.dibujar_monitor(d, e, monitor, ancho, alto);
        }
        // El reloj grande, arriba (rojo y latiendo en el ultimo minuto).
        if !self.llegando() && !self.terminado() && !matches!(self.fase, Fase::Evento(Evento::Reiniciando)) {
            let nervios = self.nervios();
            let latido = 1.0 + 0.12 * nervios * (self.total * 1.5 * std::f32::consts::TAU).sin().max(0.0).powi(4);
            let tam = ((h * 0.075).max(34.0) * latido) as i32;
            let reloj = format!("23:{:02}", self.minutos());
            let color = if nervios > 0.0 { Color::new(255, 80, 80, 255) } else { Color::new(255, 235, 190, 255) };
            ui::titulo_de(d, &reloj, (ancho - ui::medir_titulo(&reloj, tam)) / 2, (h * 0.012) as i32, tam, color);
            let falta = 59 - self.minutos().min(59);
            let sub = if falta == 0 { "¡ÚLTIMO MINUTO!".to_string() } else { format!("faltan {falta} min para la entrega") };
            let ts = 16;
            let cs = if nervios > 0.0 { Color::new(255, 150, 150, 255) } else { Color::new(220, 210, 240, 220) };
            ui::texto(d, &sub, (ancho - ui::medir(&sub, ts)) / 2, (h * 0.012) as i32 + tam + 2, ts, cs);
        }
        match self.fase {
            Fase::Cafe => {
                let s = if self.sin_cafe() { "La máquina dice AGOTADO..." } else { "Yendo por café a la máquina del fondo..." };
                let tam = 30;
                ui::titulo_de(d, s, (ancho - ui::medir_titulo(s, tam)) / 2, (h * 0.8) as i32, tam, Color::new(255, 205, 140, 255));
                let bw = (w * 0.3) as i32;
                let (bx, by) = ((ancho - bw) / 2, (h * 0.8) as i32 + 46);
                caja(d, bx as f32, by as f32, bw as f32, 10.0, 5.0, Color::new(0, 0, 0, 150));
                caja(d, bx as f32, by as f32, bw as f32 * (self.t / 4.0).min(1.0), 10.0, 5.0, Color::new(255, 190, 110, 255));
            }
            Fase::Dormido => {
                d.draw_rectangle(0, 0, ancho, alto, Color::new(0, 0, 8, (suave(self.t / 0.8) * 190.0) as u8));
                for k in 0..4 {
                    let sube = (self.t * 0.6 + k as f32 * 0.25).fract();
                    let x = w * 0.5 + 30.0 * k as f32 + 20.0 * (self.t * 2.0 + k as f32).sin();
                    let y = h * 0.55 - sube * h * 0.3;
                    let tam = 36 + k * 14;
                    ui::titulo_de(d, "Z", x as i32, y as i32, tam, con_alfa(Color::new(190, 200, 255, 255), 1.0 - sube));
                }
                let s = "Se durmió sobre el teclado...";
                ui::titulo_de(d, s, (ancho - ui::medir_titulo(s, 34)) / 2, (h * 0.68) as i32, 34, ui::TEXTO);
                let s2 = "(la energía llegó a cero: tome café antes)";
                ui::texto(d, s2, (ancho - ui::medir(s2, 20)) / 2, (h * 0.68) as i32 + 48, 20, ui::TENUE);
            }
            Fase::Evento(Evento::SinLuz) => {
                let parpadeo = if (self.t * 9.0).sin() > -0.6 || self.t > 1.2 { 1.0 } else { 0.3 };
                let s = "SE FUE LA LUZ EN EL CIT";
                let tam = (h * 0.07) as i32;
                ui::titulo_de(d, s, (ancho - ui::medir_titulo(s, tam)) / 2, (h * 0.36) as i32, tam, con_alfa(Color::new(255, 225, 140, 255), parpadeo));
                let s2 = "alguien grita «¡guarden!» en la oscuridad · el guardia viene con su linterna";
                ui::texto(d, s2, (ancho - ui::medir(s2, 20)) / 2, (h * 0.36) as i32 + tam + 12, 20, ui::TENUE);
                let bw = (w * 0.3) as i32;
                let (bx, by) = ((ancho - bw) / 2, (h * 0.36) as i32 + tam + 46);
                caja(d, bx as f32, by as f32, bw as f32, 8.0, 4.0, Color::new(0, 0, 0, 160));
                caja(d, bx as f32, by as f32, bw as f32 * (self.t / 6.0).min(1.0), 8.0, 4.0, Color::new(255, 220, 140, 255));
            }
            Fase::Evento(ev) => self.dibujar_evento(d, e, ev, koch, ancho, alto),
            Fase::Final => self.dibujar_final(d, aceptar, salir, ancho, alto),
            _ => {}
        }
        // Las notificaciones del grupo (entran de la derecha, como en el celular).
        for (n, (k, t)) in self.mensajes.iter().enumerate() {
            let (quien, msg) = WHATSAPP[*k];
            let entra = 1.0 - (1.0 - (t / 0.3).min(1.0)).powi(3);
            let sale = ((t - 3.1) / 0.4).clamp(0.0, 1.0);
            let (bw, bh) = (400.min(ancho / 2) as f32, 72.0);
            let x = w - (bw + 24.0) * (entra - sale);
            let y = h - 150.0 - n as f32 * (bh + 10.0) - bh;
            caja(d, x + 3.0, y + 5.0, bw, bh, 16.0, Color::new(0, 0, 0, 110));
            caja(d, x, y, bw, bh, 16.0, Color::new(28, 34, 32, 245));
            d.draw_circle((x + 34.0) as i32, (y + bh / 2.0) as i32, 20.0, Color::new(37, 211, 102, 255));
            d.draw_circle((x + 34.0) as i32, (y + bh / 2.0) as i32, 12.0, Color::new(28, 34, 32, 255));
            d.draw_circle((x + 34.0) as i32, (y + bh / 2.0) as i32, 9.0, Color::new(37, 211, 102, 255));
            ui::texto(d, &format!("Grupo Koch 2026 · {quien}"), (x + 66.0) as i32, (y + 10.0) as i32, 15, Color::new(120, 230, 160, 255));
            let mut corto = msg.to_string();
            while ui::medir(&corto, 19) > bw as i32 - 84 && corto.chars().count() > 4 {
                corto.pop();
            }
            ui::texto(d, &corto, (x + 66.0) as i32, (y + 36.0) as i32, 19, Color::WHITE);
            ui::texto(d, "ahora", (x + bw - 52.0) as i32, (y + 10.0) as i32, 13, ui::TENUE);
        }
        // Lo que se dice va abajo (ahi no tapa las pestanas ni las opciones de la compu); el aviso,
        // encima de el si estan los dos.
        let dialogo = match (self.hablante(), texto.is_empty()) {
            (Some((nombre, _, _)), false) => Some((nombre, ui::alto_dialogo(nombre, texto, 880.min(ancho - 40)))),
            _ => None,
        };
        // Los avisos (abajo, en una pastilla).
        if let Some((s, t)) = &self.aviso {
            let a = (t / 0.15).min(1.0) * (1.0 - ((t - 3.0) / 0.5).clamp(0.0, 1.0));
            let tam = 21;
            let wa = ui::medir(s, tam) + 44;
            let base = dialogo.map_or(h * 0.915, |(_, hd)| (alto - hd - 20 - 52) as f32);
            let y = base - 8.0 * (1.0 - (t / 0.2).min(1.0));
            caja(d, (ancho - wa) as f32 / 2.0, y - 8.0, wa as f32, 40.0, 20.0, Color::new(16, 12, 34, (a * 235.0) as u8));
            d.draw_rectangle_rounded_lines(Rectangle::new((ancho - wa) as f32 / 2.0, y - 8.0, wa as f32, 40.0), 1.0, 8, Color::new(255, 215, 140, (a * 160.0) as u8));
            ui::texto(d, s, (ancho - ui::medir(s, tam)) / 2, y as i32 + 1, tam, Color::new(255, 235, 175, (a * 255.0) as u8));
        }
        if let Some((nombre, hd)) = dialogo {
            let wd = 880.min(ancho - 40);
            let color = if nombre == "PROFE KOCH" { Color::new(255, 150, 120, 255) } else { ui::LILA };
            ui::caja_dialogo(d, nombre, color, texto, (ancho - wd) / 2, alto - hd - 20, wd);
        }
    }

    /// Si se ve la compu (no se ve yendo por cafe, dormido, sin luz, en la pantalla azul, ni en la llamada).
    fn se_ve_la_compu(&self) -> bool {
        matches!(self.fase, Fase::Trabajando | Fase::Evento(Evento::WindowsUpdate | Evento::Rubrica(..) | Evento::Companero(_) | Evento::Ayudando))
    }

    /// La llegada: franjas de cine y el titulo, con su glitch.
    fn dibujar_llegada(&self, d: &mut RaylibDrawHandle, e: &Entrada, ancho: i32, alto: i32) {
        let (w, h) = (ancho as f32, alto as f32);
        let franja = (h * 0.1) as i32;
        d.draw_rectangle(0, 0, ancho, franja, Color::BLACK);
        d.draw_rectangle(0, alto - franja, ancho, franja, Color::BLACK);
        let a = suave(self.total / 1.2) * (1.0 - suave((self.total - 7.5) / 1.0));
        if a <= 0.0 {
            return;
        }
        let tam = (h * 0.2) as i32;
        let t1 = "11:59";
        let x = (ancho - ui::medir_titulo(t1, tam)) / 2;
        let y = (h * 0.2) as i32;
        // El glitch: copias corridas en rojo y cian que tiemblan de vez en cuando.
        let tiembla = if (self.total * 3.0).fract() < 0.12 { 6.0 } else { 2.0 };
        let dx = (tiembla * (self.total * 50.0).sin()) as i32;
        ui::titulo(d, t1, x - 4 + dx, y, tam, con_alfa(Color::new(255, 40, 60, 255), a * 0.6), con_alfa(Color::new(120, 0, 20, 255), a * 0.6));
        ui::titulo(d, t1, x + 4 - dx, y, tam, con_alfa(Color::new(60, 230, 255, 255), a * 0.5), con_alfa(Color::new(0, 80, 120, 255), a * 0.5));
        ui::titulo(d, t1, x, y, tam, con_alfa(Color::new(255, 245, 235, 255), a), con_alfa(Color::new(255, 90, 90, 255), a));
        let t2 = "LA NOCHE ANTES DE LA ENTREGA";
        let t2m = 30;
        ui::titulo_de(d, t2, (ancho - ui::medir_titulo(t2, t2m)) / 2, y + tam + 4, t2m, con_alfa(Color::new(255, 210, 140, 255), a));
        let t3 = "Laboratorio del CIT · UVG · Gráficas, Teoría y Software 2 se entregan a las 23:59";
        ui::texto(d, t3, (ancho - ui::medir(t3, 20)) / 2, y + tam + 48, 20, con_alfa(Color::new(225, 220, 245, 255), a));
        let s = format!("{}: saltar", ui::boton(e.nombre("aceptar")));
        ui::texto(d, &s, ancho - ui::medir(&s, 16) - 24, alto - franja / 2 - 8, 16, Color::new(200, 200, 220, 200));
        let _ = w;
    }

    /// El monitor: el editor con las pestanas de los tres proyectos (con su avance), el minijuego de
    /// la escogida, la barra de estado y la barra de tareas (energia, WhatsApp, cafe y la hora).
    fn dibujar_monitor(&self, d: &mut RaylibDrawHandle, e: &Entrada, monitor: &Texture2D, ancho: i32, alto: i32) {
        let (w, h) = (ancho as f32, alto as f32);
        // Con sueño, la pantalla se mece y se pone borrosa en las orillas.
        let mece = if self.energia < 0.25 { (self.total * 1.7).sin() * 8.0 * (1.0 - self.energia * 4.0) } else { 0.0 };
        let (px, py, pw, ph) = (w * 0.085 + mece, h * 0.135, w * 0.83, h * 0.77);
        // El bisel del monitor, su sombra y la lucecita de encendido.
        caja(d, px - 16.0, py - 14.0, pw + 32.0, ph + 36.0, 18.0, Color::new(0, 0, 0, 120));
        caja(d, px - 14.0, py - 14.0, pw + 28.0, ph + 30.0, 16.0, BISEL);
        d.draw_circle((px + pw / 2.0) as i32, (py + ph + 8.0) as i32, 3.0, Color::new(80, 200, 255, 255));
        d.draw_rectangle(px as i32, py as i32, pw as i32, ph as i32, FONDO_EDITOR);
        // La barra de titulo.
        d.draw_rectangle(px as i32, py as i32, pw as i32, 28, BARRA_OSCURA);
        for (k, c) in [Color::new(255, 95, 87, 255), Color::new(254, 188, 46, 255), Color::new(40, 200, 64, 255)].iter().enumerate() {
            d.draw_circle((px + 16.0 + k as f32 * 18.0) as i32, (py + 14.0) as i32, 6.0, *c);
        }
        let titulo = format!("{} - proyectos-koch - Visual Studio Code", self.curso().pestana());
        ui::texto(d, &titulo, (px + pw / 2.0) as i32 - ui::medir(&titulo, 14) / 2, py as i32 + 6, 14, Color::new(170, 170, 190, 255));
        // Las pestanas, con su avance.
        let ty = py + 28.0;
        let tw = (pw / 3.0).min(330.0);
        d.draw_rectangle(px as i32, ty as i32, pw as i32, 50, BARRA);
        for (k, c) in CURSOS.iter().enumerate() {
            let x = px + k as f32 * tw;
            let activa = k == self.pestana;
            let encima = Self::encima(e, x, ty, tw, 50.0);
            self.zona(x, ty, tw, 50.0, Zona::Pestana(k));
            d.draw_rectangle(x as i32, ty as i32, tw as i32 - 1, 50, if activa { FONDO_EDITOR } else if encima { Color::new(48, 50, 62, 255) } else { BARRA });
            if activa {
                d.draw_rectangle(x as i32, ty as i32, tw as i32 - 1, 3, c.color());
            }
            d.draw_circle((x + 16.0) as i32, (ty + 17.0) as i32, 5.0, c.color());
            ui::texto(d, c.pestana(), (x + 28.0) as i32, (ty + 8.0) as i32, 16, if activa { Color::WHITE } else { ui::TENUE });
            let p = self.progreso[k];
            let s = if p >= 100.0 { "ENTREGADO".to_string() } else { format!("{:.0}%", p) };
            ui::texto(d, &s, (x + tw - 14.0) as i32 - ui::medir(&s, 14), (ty + 9.0) as i32, 14, if p >= 100.0 { BIEN } else { ui::TENUE });
            let color_barra = match c {
                Curso::Graficas => ui::ColorBarra::Rojo,
                Curso::Teoria => ui::ColorBarra::Azul,
                Curso::Software => ui::ColorBarra::Verde,
            };
            ui::barra(d, x + 12.0, ty + 28.0, tw - 24.0, 18.0, p / 100.0, if p >= 100.0 { ui::ColorBarra::Verde } else { color_barra });
        }
        // El contenido.
        let (cx, cy, cw, chh) = (px + 18.0, ty + 64.0, pw - 36.0, ph - 28.0 - 64.0 - 26.0 - 46.0);
        let c = self.curso();
        if self.progreso[c as usize] >= 100.0 {
            let s = format!("{} · entregado", c.nombre());
            let tam = 34;
            ui::titulo_de(d, &s, (cx + (cw - ui::medir_titulo(&s, tam) as f32) / 2.0) as i32, (cy + chh / 2.0 - 40.0) as i32, tam, BIEN);
            let s2 = format!("{}: hay más proyectos", ui::boton(&format!("{} {}", ui::IZQUIERDA, ui::DERECHA)));
            ui::texto(d, &s2, (cx + (cw - ui::medir(&s2, 18) as f32) / 2.0) as i32, (cy + chh / 2.0 + 10.0) as i32, 18, ui::TENUE);
        } else {
            match c {
                Curso::Graficas => self.dibujar_graficas(d, e, monitor, cx, cy, cw, chh),
                Curso::Teoria => self.dibujar_teoria(d, e, cx, cy, cw, chh),
                Curso::Software => self.dibujar_software(d, e, cx, cy, cw, chh),
            }
        }
        // El sello de lo que paso (con su rebote).
        if let Some((bien, t)) = self.resultado {
            let mostrar = match c {
                Curso::Teoria => self.teoria.probando.is_some() && t > 3.0,
                Curso::Software => t > 0.6,
                Curso::Graficas => true,
            };
            let desde = match c {
                Curso::Teoria => 3.0,
                Curso::Software => 0.6,
                Curso::Graficas => 0.0,
            };
            if mostrar && self.progreso[c as usize] < 100.0 || (mostrar && bien) {
                let k = ((t - desde) / 0.18).clamp(0.0, 1.0);
                let rebote = 1.0 + 0.5 * (1.0 - k) * (1.0 - k);
                let (s, col) = if bien {
                    (match c {
                        Curso::Graficas => "¡ARREGLADO!",
                        Curso::Teoria => "¡PASAN LAS PRUEBAS!",
                        Curso::Software => "¡CI EN VERDE!",
                    }, BIEN)
                } else {
                    (match c {
                        Curso::Graficas => "SIGUE ROTO",
                        Curso::Teoria => "FALLAN LAS PRUEBAS",
                        Curso::Software => "CI EN ROJO",
                    }, MAL)
                };
                let tam = ((h * 0.07) * rebote) as i32;
                let x = (cx + (cw - ui::medir_titulo(s, tam) as f32) / 2.0) as i32;
                let y = (cy + chh * 0.5) as i32 - tam / 2;
                ui::titulo_de(d, s, x, y, tam, col);
                let sub = if bien { format!("+{:.0}% en {}", c.avance(), c.nombre()) } else { "-4% y a compilar otra vez".to_string() };
                ui::texto(d, &sub, (cx + (cw - ui::medir(&sub, 20) as f32) / 2.0) as i32, y + tam + 4, 20, col);
            }
        }
        if self.bloqueo > 0.0 {
            let s = format!("Compilando... {:.0}", self.bloqueo.ceil());
            ui::texto(d, &s, (cx + cw) as i32 - ui::medir(&s, 17), (cy + chh - 20.0) as i32, 17, MAL);
        }
        // La barra de estado (azul, como la del editor).
        let sy = py + ph - 46.0 - 26.0;
        d.draw_rectangle(px as i32, sy as i32, pw as i32, 26, Color::new(0, 110, 200, 255));
        let rama = format!("main*   {} errores   {} advertencias", 3 - self.progreso.iter().filter(|&&p| p >= 100.0).count(), (self.total * 0.3) as i32 % 9 + 2);
        ui::texto(d, &rama, px as i32 + 12, sy as i32 + 5, 15, Color::WHITE);
        let derecha = format!("Ln {}, Col {}   UTF-8   Rust", 142 + (self.total * 2.0) as i32 % 40, 8 + (self.total * 5.0) as i32 % 30);
        ui::texto(d, &derecha, (px + pw) as i32 - 12 - ui::medir(&derecha, 15), sy as i32 + 5, 15, Color::WHITE);
        // La barra de tareas: inicio, las apps, el cafe (la energia), WhatsApp y la hora.
        let by = py + ph - 46.0;
        d.draw_rectangle(px as i32, by as i32, pw as i32, 46, Color::new(20, 22, 30, 255));
        d.draw_rectangle(px as i32, by as i32, pw as i32, 1, LINEA);
        for (k, c) in [Color::new(0, 120, 215, 255), Color::new(30, 150, 240, 255), Color::new(240, 80, 60, 255)].iter().enumerate() {
            let x = px + 14.0 + k as f32 * 40.0;
            caja(d, x, by + 9.0, 28.0, 28.0, 6.0, *c);
        }
        // El cafe: una tacita que se vacia (es la energia), con su vapor.
        let tx = px + 150.0;
        let encima_cafe = Self::encima(e, tx - 6.0, by + 2.0, 250.0, 42.0);
        self.zona(tx - 6.0, by + 2.0, 250.0, 42.0, Zona::Cafe);
        if encima_cafe {
            caja(d, tx - 6.0, by + 4.0, 250.0, 38.0, 8.0, Color::new(50, 44, 60, 255));
        }
        let baja = self.energia < 0.25;
        let color_cafe = if baja { MAL } else { Color::new(200, 140, 80, 255) };
        caja(d, tx, by + 12.0, 22.0, 24.0, 4.0, Color::new(230, 230, 235, 255));
        d.draw_ring(Vector2::new(tx + 24.0, by + 23.0), 4.0, 7.0, -90.0, 90.0, 12, Color::new(230, 230, 235, 255));
        let lleno = (20.0 * self.energia).max(0.0);
        d.draw_rectangle((tx + 2.0) as i32, (by + 14.0 + 20.0 - lleno) as i32, 18, lleno as i32, color_cafe);
        for k in 0..2 {
            let x = tx + 6.0 + k as f32 * 8.0 + 2.0 * (self.total * 3.0 + k as f32).sin();
            d.draw_line_ex(Vector2::new(x, by + 10.0), Vector2::new(x + 2.0, by + 3.0), 1.5, Color::new(220, 220, 230, (120.0 * self.energia) as u8));
        }
        let energia = format!("{:.0}%", self.energia * 100.0);
        ui::texto(d, &energia, (tx + 38.0) as i32, (by + 13.0) as i32, 18, if baja { MAL } else { ui::TEXTO });
        let cafe = if self.sin_cafe() { "café: AGOTADO".to_string() } else { format!("{}: café", ui::boton(e.nombre("barrel"))) };
        ui::texto(d, &cafe, (tx + 90.0) as i32, (by + 14.0) as i32, 16, if self.sin_cafe() { MAL } else { ui::TENUE });
        // WhatsApp, con su globito de no leidos.
        let wx = px + pw * 0.52;
        let encima_wa = Self::encima(e, wx - 22.0, by + 2.0, 220.0, 42.0);
        self.zona(wx - 22.0, by + 2.0, 220.0, 42.0, Zona::Silenciar);
        if encima_wa {
            caja(d, wx - 22.0, by + 4.0, 220.0, 38.0, 8.0, Color::new(40, 56, 48, 255));
        }
        d.draw_circle(wx as i32, (by + 23.0) as i32, 14.0, Color::new(37, 211, 102, 255));
        d.draw_circle(wx as i32, (by + 23.0) as i32, 8.0, Color::new(20, 22, 30, 255));
        d.draw_circle(wx as i32, (by + 23.0) as i32, 6.0, Color::new(37, 211, 102, 255));
        if self.sin_leer > 0 {
            d.draw_circle((wx + 12.0) as i32, (by + 11.0) as i32, 10.0, Color::new(235, 60, 60, 255));
            let n = self.sin_leer.min(99).to_string();
            ui::texto(d, &n, (wx + 12.0) as i32 - ui::medir(&n, 13) / 2, (by + 4.0) as i32, 13, Color::WHITE);
        }
        let silencio = if self.silencio > 0.0 { format!("silenciado {:.0}s", self.silencio.ceil()) } else { format!("{}: silenciar", ui::boton(e.nombre("turbo"))) };
        ui::texto(d, &silencio, (wx + 30.0) as i32, (by + 14.0) as i32, 16, ui::TENUE);
        // La hora de la compu (con los segundos).
        let segundos = ((self.reloj / DURACION * 59.0).fract() * 60.0) as u32;
        let hora = format!("23:{:02}:{:02}", self.minutos(), segundos);
        ui::texto(d, &hora, (px + pw) as i32 - 16 - ui::medir(&hora, 17), (by + 13.0) as i32, 17, ui::TEXTO);
        // Las teclas, abajo del monitor (si no hay un aviso ahi mismo).
        if self.aviso.is_some() {
            return;
        }
        let teclas = format!("{}: proyecto    {}: escoger    {}: hacer    {}: salir del lab", ui::boton(&format!("{} {}", ui::IZQUIERDA, ui::DERECHA)), ui::boton(&format!("{} {}", ui::ARRIBA, ui::ABAJO)), ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("aterrizar")));
        ui::texto(d, &teclas, (ancho - ui::medir(&teclas, 15)) / 2, (py + ph + 18.0) as i32, 15, Color::new(200, 200, 225, 200));
    }

    /// Las opciones como botones (con su numero), de la `y` para abajo: la escogida resaltada.
    #[allow(clippy::too_many_arguments)]
    fn botones(&self, d: &mut RaylibDrawHandle, e: &Entrada, textos: &[String], x: f32, y: f32, w: f32, tam: i32, escogida: Option<usize>) -> f32 {
        let mut y = y;
        for (k, s) in textos.iter().enumerate() {
            let lineas = ui::envolver(s, tam, (w - 56.0) as i32);
            let bh = 14.0 + lineas.len().max(1) as f32 * (tam as f32 + 4.0);
            let activo = escogida == Some(k);
            let encima = Self::encima(e, x, y, w, bh);
            self.zona(x, y, w, bh, Zona::Opcion(k));
            let fondo = if activo { Color::new(70, 80, 160, 255) } else if encima { Color::new(52, 54, 70, 255) } else { Color::new(40, 42, 54, 255) };
            caja(d, x, y, w, bh, 8.0, fondo);
            if activo && !ui::seleccion(d, x, y, w, bh, self.total, Color::new(255, 230, 160, 255)) {
                d.draw_rectangle_rounded_lines(Rectangle::new(x, y, w, bh), 0.2, 8, ACENTO);
            }
            caja(d, x + 8.0, y + 8.0, 26.0, 24.0, 6.0, if activo { ACENTO } else { Color::new(60, 62, 80, 255) });
            let n = (k + 1).to_string();
            ui::texto(d, &n, (x + 21.0) as i32 - ui::medir(&n, 16) / 2, (y + 11.0) as i32, 16, Color::WHITE);
            for (i, l) in lineas.iter().enumerate() {
                ui::texto(d, l, (x + 44.0) as i32, (y + 8.0 + i as f32 * (tam as f32 + 4.0)) as i32, tam, if activo { Color::WHITE } else { ui::TEXTO });
            }
            y += bh + 8.0;
        }
        y
    }

    #[allow(clippy::too_many_arguments)]
    fn dibujar_graficas(&self, d: &mut RaylibDrawHandle, e: &Entrada, monitor: &Texture2D, x: f32, y: f32, w: f32, h: f32) {
        // A la izquierda, el render de verdad (con el bug); a la derecha, el codigo y los arreglos.
        let rw = (w * 0.47).min(h * monitor.width as f32 / monitor.height as f32);
        let rh = rw * monitor.height as f32 / monitor.width as f32;
        caja(d, x - 2.0, y - 2.0, rw + 4.0, rh + 30.0, 6.0, Color::new(20, 20, 26, 255));
        ui::texto(d, "Vista previa · cargo run --release", x as i32 + 8, y as i32 + 4, 14, ui::TENUE);
        let fuente = Rectangle::new(0.0, 0.0, monitor.width as f32, monitor.height as f32);
        d.draw_texture_pro(monitor, fuente, Rectangle::new(x, y + 26.0, rw, rh), Vector2::zero(), 0.0, Color::WHITE);
        let ficha = &BUGS[self.graficas.bug];
        let (bx, bw) = (x + rw + 24.0, w - rw - 24.0);
        ui::texto(d, "render.rs · el proyecto 3 sale raro:", bx as i32, y as i32, 16, ui::TENUE);
        let fin = ui::parrafo(d, ficha.sintoma, bx as i32, y as i32 + 22, 22, bw as i32, Color::new(255, 205, 150, 255));
        // La linea sospechosa, resaltada como en el editor (con su numero de linea y el subrayado rojo).
        let ly = fin as f32 + 8.0;
        caja(d, bx, ly, bw, 34.0, 6.0, Color::new(20, 20, 26, 255));
        ui::texto(d, "142", bx as i32 + 10, ly as i32 + 9, 15, Color::new(100, 100, 120, 255));
        let mut codigo = ficha.codigo.to_string();
        while ui::medir(&codigo, 16) > bw as i32 - 60 && codigo.chars().count() > 6 {
            codigo.pop();
        }
        ui::texto(d, &codigo, bx as i32 + 46, ly as i32 + 8, 16, Color::new(255, 150, 175, 255));
        let ancho_codigo = ui::medir(&codigo, 16) as f32;
        let mut zx = bx + 46.0;
        while zx < bx + 46.0 + ancho_codigo {
            d.draw_line_ex(Vector2::new(zx, ly + 29.0), Vector2::new(zx + 3.0, ly + 26.0), 1.5, MAL);
            d.draw_line_ex(Vector2::new(zx + 3.0, ly + 26.0), Vector2::new(zx + 6.0, ly + 29.0), 1.5, MAL);
            zx += 6.0;
        }
        ui::texto(d, "¿Cómo se arregla?", bx as i32, ly as i32 + 44, 16, ui::TENUE);
        let textos: Vec<String> = self.graficas.opciones.iter().map(|&i| BUGS[i].arreglo.to_string()).collect();
        let escogida = self.resultado.is_none().then_some(self.cursor);
        self.botones(d, e, &textos, bx, ly + 68.0, bw, 17, escogida);
    }

    /// Donde va cada estado del automata en la pantalla.
    fn lugares(&self, au: &Automata, x: f32, y: f32, w: f32) -> Vec<Vector2> {
        (0..au.estados).map(|k| Vector2::new(x + 90.0 + (w - 180.0) * k as f32 / (au.estados.max(2) - 1) as f32, y + 170.0)).collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn dibujar_teoria(&self, d: &mut RaylibDrawHandle, e: &Entrada, x: f32, y: f32, w: f32, h: f32) {
        let ronda = &self.teoria;
        let au = &AUTOMATAS[ronda.automata];
        ui::texto(d, "JFLAP · complete el AFD que acepta", x as i32, y as i32, 16, ui::TENUE);
        ui::texto(d, au.lenguaje, x as i32, y as i32 + 22, 24, Color::new(150, 210, 255, 255));
        let dw = w * 0.64;
        // La hoja de trabajo (cuadriculada, como la de JFLAP).
        caja(d, x - 4.0, y + 56.0, dw + 8.0, h - 60.0, 8.0, Color::new(24, 26, 34, 255));
        let mut gx = x;
        while gx < x + dw {
            d.draw_line(gx as i32, (y + 60.0) as i32, gx as i32, (y + h - 8.0) as i32, Color::new(32, 35, 46, 255));
            gx += 24.0;
        }
        let lugares = self.lugares(au, x, y, dw);
        let delta_jugador = {
            let mut t = au.delta;
            for ((q, s), r) in &ronda.faltan {
                t[*q][*s] = r.unwrap_or(99);
            }
            t
        };
        // Las flechas (juntando a y b cuando van al mismo lado).
        for q in 0..au.estados {
            for destino in 0..au.estados {
                let simbolos: Vec<&str> = (0..2).filter(|&s| delta_jugador[q][s] == destino).map(|s| ["a", "b"][s]).collect();
                if simbolos.is_empty() {
                    continue;
                }
                let etiqueta = simbolos.join(",");
                let contestada = ronda.faltan.iter().any(|((fq, fs), r)| *fq == q && r.is_some() && delta_jugador[*fq][*fs] == destino);
                let color = if contestada { Color::new(255, 205, 100, 255) } else { Color::new(200, 210, 235, 255) };
                let (a, b) = (lugares[q], lugares[destino]);
                if q == destino {
                    let c = Vector2::new(a.x, a.y - 46.0);
                    d.draw_ring(c, 16.0, 18.5, 0.0, 360.0, 24, color);
                    ui::texto(d, &etiqueta, c.x as i32 - ui::medir(&etiqueta, 17) / 2, c.y as i32 - 42, 17, color);
                } else {
                    // Las de ida van por arriba y las de regreso por abajo (asi no se enciman).
                    let dir = if destino > q { 1.0 } else { -1.0 };
                    let (aa, bb) = (Vector2::new(a.x + 32.0 * dir, a.y - 8.0 * dir), Vector2::new(b.x - 32.0 * dir, b.y - 8.0 * dir));
                    flecha(d, aa, bb, -38.0 * (1.0 + (destino as f32 - q as f32).abs() * 0.45), &etiqueta, color, false);
                }
            }
        }
        // Las que faltan: una flecha punteada con "?" (la que se esta contestando, parpadea).
        let actual = ronda.faltan.iter().position(|f| f.1.is_none());
        for (n, ((q, s), r)) in ronda.faltan.iter().enumerate() {
            if r.is_some() {
                continue;
            }
            let a = lugares[*q];
            let p = Vector2::new(a.x + 34.0 + n as f32 * 26.0, a.y + 78.0);
            let brilla = Some(n) == actual && (self.total * 4.0).sin() > 0.0;
            let color = if brilla { Color::new(255, 100, 100, 255) } else { Color::new(200, 80, 90, 255) };
            flecha(d, Vector2::new(a.x + 8.0, a.y + 30.0), p, 12.0, &format!("{} ?", ["a", "b"][*s]), color, true);
        }
        // Los estados (los finales con doble circulo) y la flecha de inicio.
        for (k, p) in lugares.iter().enumerate() {
            d.draw_circle_v(Vector2::new(p.x + 3.0, p.y + 4.0), 30.0, Color::new(0, 0, 0, 90));
            d.draw_circle_v(*p, 29.0, Color::new(255, 236, 160, 255));
            d.draw_ring(*p, 27.0, 30.0, 0.0, 360.0, 32, Color::new(70, 60, 30, 255));
            if au.finales.contains(&k) {
                d.draw_ring(*p, 21.0, 23.0, 0.0, 360.0, 32, Color::new(70, 60, 30, 255));
            }
            let n = format!("q{k}");
            ui::texto(d, &n, p.x as i32 - ui::medir(&n, 20) / 2, p.y as i32 - 11, 20, Color::new(40, 30, 10, 255));
        }
        flecha(d, Vector2::new(lugares[0].x - 82.0, lugares[0].y), Vector2::new(lugares[0].x - 32.0, lugares[0].y), 0.0, "", Color::new(200, 210, 235, 255), false);
        if let Some(p) = self.token(au, &lugares) {
            d.draw_circle_v(p, 12.0, Color::new(110, 240, 255, 120));
            d.draw_circle_v(p, 8.0, Color::new(110, 240, 255, 255));
        }
        // La pregunta y las opciones (o las pruebas corriendo), a la derecha.
        let (ox, ow) = (x + dw + 22.0, w - dw - 22.0);
        if let Some(n) = actual {
            let ((q, s), _) = ronda.faltan[n];
            let pregunta = format!("q{q} -{}-> ?", ["a", "b"][s]);
            ui::titulo_de(d, &pregunta, ox as i32, y as i32 + 62, 28, Color::new(255, 205, 120, 255));
            if ronda.faltan.len() > 1 {
                let s = format!("transición {} de {}", n + 1, ronda.faltan.len());
                ui::texto(d, &s, ox as i32, y as i32 + 102, 15, ui::TENUE);
            }
            let textos: Vec<String> = (0..au.estados).map(|k| format!("q{k}{}", if au.finales.contains(&k) { "  (final)" } else { "" })).collect();
            self.botones(d, e, &textos, ox, y + 128.0, ow, 18, Some(self.cursor));
        } else {
            ui::texto(d, "cargo test", ox as i32, y as i32 + 62, 18, ui::TENUE);
            let avance = self.resultado.map_or(0.0, |r| r.1);
            for (k, (cadena, espera)) in ronda.pruebas.iter().enumerate() {
                let yy = y + 96.0 + k as f32 * 40.0;
                caja(d, ox, yy - 4.0, ow, 32.0, 6.0, Color::new(36, 38, 50, 255));
                ui::texto(d, &format!("\"{cadena}\""), ox as i32 + 10, yy as i32 + 2, 19, ui::TEXTO);
                if avance > (k + 1) as f32 {
                    let acepta = au.acepta(&delta_jugador_seguro(au, ronda), cadena).0;
                    let ok = acepta == *espera;
                    let s = if ok { if acepta { "acepta" } else { "rechaza" } } else { "¡FALLA!" };
                    let icono = if ui::palomita(d, (ox + ow) - 34.0, yy + 1.0, 22.0, ok) { 40 } else { 12 };
                    ui::texto(d, s, (ox + ow) as i32 - icono - ui::medir(s, 18), yy as i32 + 3, 18, if ok { BIEN } else { MAL });
                } else if avance > k as f32 {
                    let puntos = ".".repeat(1 + (self.total * 6.0) as usize % 3);
                    ui::texto(d, &puntos, (ox + ow) as i32 - 40, yy as i32 + 2, 19, ui::TENUE);
                }
            }
        }
    }

    /// Donde va la bolita que recorre el automata con la prueba que se esta corriendo.
    fn token(&self, au: &Automata, lugares: &[Vector2]) -> Option<Vector2> {
        let r = self.resultado?;
        let avance = r.1;
        let k = avance.floor() as usize;
        let (cadena, _) = self.teoria.pruebas.get(k)?;
        let delta = delta_jugador_seguro(au, &self.teoria);
        let (_, camino) = au.acepta(&delta, cadena);
        let u = (avance.fract() * (camino.len() as f32)).min(camino.len() as f32 - 1.0);
        let i = u.floor() as usize;
        let (a, b) = (lugares[camino[i].min(au.estados - 1)], lugares[camino[(i + 1).min(camino.len() - 1)].min(au.estados - 1)]);
        let f = u.fract();
        Some(Vector2::new(a.x + (b.x - a.x) * f, a.y + (b.y - a.y) * f - 34.0 * (f * std::f32::consts::PI).sin()))
    }

    #[allow(clippy::too_many_arguments)]
    fn dibujar_software(&self, d: &mut RaylibDrawHandle, e: &Entrada, x: f32, y: f32, w: f32, h: f32) {
        if self.github_caido > 0.0 {
            // El unicornio de GitHub caido (dibujado con figuras) y el 503.
            let (ux, uy) = (x + w / 2.0, y + h * 0.3);
            caja(d, ux - 60.0, uy - 30.0, 120.0, 60.0, 30.0, Color::new(240, 220, 255, 255));
            d.draw_circle((ux + 60.0) as i32, (uy - 30.0) as i32, 26.0, Color::new(240, 220, 255, 255));
            d.draw_triangle(Vector2::new(ux + 66.0, uy - 52.0), Vector2::new(ux + 74.0, uy - 90.0), Vector2::new(ux + 78.0, uy - 50.0), Color::new(255, 210, 90, 255));
            for k in 0..4 {
                d.draw_rectangle((ux - 50.0 + k as f32 * 30.0) as i32, (uy + 24.0) as i32, 12, 34, Color::new(240, 220, 255, 255));
            }
            for (k, c) in [Color::new(255, 120, 160, 255), Color::new(255, 200, 90, 255), Color::new(120, 220, 255, 255)].iter().enumerate() {
                d.draw_rectangle((ux - 100.0 - k as f32 * 6.0) as i32, (uy - 26.0 + k as f32 * 12.0) as i32, 44, 10, *c);
            }
            let s = "503 · GitHub no responde";
            ui::titulo_de(d, s, (x + (w - ui::medir_titulo(s, 36) as f32) / 2.0) as i32, (y + h * 0.58) as i32, 36, MAL);
            let s2 = format!("El unicornio está cansado. Vuelva en {:.0} segundos (o avance otro proyecto).", self.github_caido.ceil());
            ui::texto(d, &s2, (x + (w - ui::medir(&s2, 19) as f32) / 2.0) as i32, (y + h * 0.58) as i32 + 50, 19, ui::TENUE);
            return;
        }
        let c = &CONFLICTOS[self.software];
        ui::texto(d, &format!("git merge {} · conflicto en {}", c.rama, c.archivo), x as i32, y as i32, 16, ui::TENUE);
        let mut lineas: Vec<(String, Color, Option<Color>)> = Vec::new();
        let normal = Color::new(220, 220, 232, 255);
        for l in c.antes {
            lineas.push((l.to_string(), normal, None));
        }
        lineas.push(("<<<<<<< HEAD (lo suyo)".into(), Color::new(255, 140, 140, 255), Some(Color::new(70, 34, 40, 255))));
        for l in c.head {
            lineas.push((l.to_string(), Color::new(255, 200, 180, 255), Some(Color::new(52, 32, 36, 255))));
        }
        lineas.push(("=======".into(), Color::new(200, 200, 200, 255), None));
        for l in c.otra {
            lineas.push((l.to_string(), Color::new(170, 230, 255, 255), Some(Color::new(28, 44, 62, 255))));
        }
        lineas.push((format!(">>>>>>> {} (lo del compañero)", c.rama), Color::new(130, 190, 255, 255), Some(Color::new(28, 40, 70, 255))));
        for l in c.despues {
            lineas.push((l.to_string(), normal, None));
        }
        let cw = w * 0.6;
        caja(d, x, y + 24.0, cw, lineas.len() as f32 * 26.0 + 16.0, 8.0, Color::new(20, 21, 27, 255));
        for (k, (l, col, fondo)) in lineas.iter().enumerate() {
            let yy = y + 32.0 + k as f32 * 26.0;
            if let Some(f) = fondo {
                d.draw_rectangle(x as i32 + 2, yy as i32 - 2, cw as i32 - 4, 26, *f);
            }
            ui::texto(d, &format!("{:>2}", k + 1), x as i32 + 8, yy as i32 + 2, 15, Color::new(90, 92, 112, 255));
            let mut l = l.clone();
            while ui::medir(&l, 17) > cw as i32 - 56 && l.chars().count() > 6 {
                l.pop();
            }
            ui::texto(d, &l, x as i32 + 40, yy as i32 + 1, 17, *col);
        }
        let (ox, ow) = (x + cw + 22.0, w - cw - 22.0);
        caja(d, ox, y + 24.0, ow, 64.0, 8.0, Color::new(60, 30, 36, 255));
        ui::texto(d, "FALLA:", ox as i32 + 12, y as i32 + 32, 15, MAL);
        ui::parrafo(d, c.test, ox as i32 + 12, y as i32 + 52, 16, ow as i32 - 24, Color::new(255, 220, 160, 255));
        let textos = vec!["Quedarse con lo suyo (HEAD)".to_string(), format!("Quedarse con lo del compañero ({})", c.rama), "Las dos (una y luego la otra)".to_string()];
        let escogida = self.resultado.is_none().then_some(self.cursor);
        self.botones(d, e, &textos, ox, y + 100.0, ow, 17, escogida);
        if let Some((_, t)) = self.resultado {
            if t < 0.6 {
                let puntos = ".".repeat(1 + (self.total * 6.0) as usize % 3);
                let s = format!("Corriendo el CI{puntos}");
                let con_anillo = ui::cargando(d, ox + 14.0, y + h - 20.0, 28.0, self.total);
                ui::texto(d, &s, ox as i32 + if con_anillo { 34 } else { 0 }, (y + h - 30.0) as i32, 19, ui::TENUE);
            }
        }
    }

    fn dibujar_evento(&self, d: &mut RaylibDrawHandle, e: &Entrada, ev: Evento, koch: Option<&Texture2D>, ancho: i32, alto: i32) {
        let (w, h) = (ancho as f32, alto as f32);
        let (mw, mh) = ((w * 0.46).max(460.0), (h * 0.4).max(260.0));
        let (mx, my) = ((w - mw) / 2.0, (h - mh) / 2.0 + h * 0.03);
        let entra = suave(self.t / 0.2);
        let tinta = Color::new(32, 32, 44, 255);
        let ventana = |d: &mut RaylibDrawHandle, titulo: &str, color: Color| {
            d.draw_rectangle(0, 0, ancho, alto, Color::new(4, 2, 14, (165.0 * entra) as u8));
            let s = 0.92 + 0.08 * entra;
            let (vw, vh) = (mw * s, mh * s);
            let (vx, vy) = (mx + (mw - vw) / 2.0, my + (mh - vh) / 2.0);
            caja(d, vx + 6.0, vy + 10.0, vw, vh, 12.0, Color::new(0, 0, 0, 110));
            caja(d, vx, vy, vw, vh, 12.0, Color::new(246, 246, 250, 255));
            caja(d, vx, vy, vw, 46.0, 12.0, color);
            d.draw_rectangle(vx as i32, (vy + 30.0) as i32, vw as i32, 16, color);
            ui::texto(d, titulo, (vx + 18.0) as i32, (vy + 13.0) as i32, 19, Color::WHITE);
        };
        let botones = |d: &mut RaylibDrawHandle, a: &str, b: &str, eleccion: usize| {
            for (k, s) in [a, b].iter().enumerate() {
                let bw = mw / 2.0 - 30.0;
                let x = mx + 20.0 + k as f32 * (bw + 20.0);
                let y = my + mh - 66.0;
                let escogido = k == eleccion;
                let encima = Self::encima(e, x, y, bw, 46.0);
                self.zona(x, y, bw, 46.0, Zona::Boton(k));
                caja(d, x, y, bw, 46.0, 8.0, if escogido { Color::new(30, 105, 225, 255) } else if encima { Color::new(222, 226, 236, 255) } else { Color::new(210, 212, 222, 255) });
                ui::texto(d, s, (x + (bw - ui::medir(s, 18) as f32) / 2.0) as i32, (y + 13.0) as i32, 18, if escogido { Color::WHITE } else { tinta });
            }
        };
        match ev {
            Evento::WindowsUpdate => {
                ventana(d, "Windows Update", Color::new(0, 103, 192, 255));
                let n = (8.0 - self.t).max(0.0);
                // El circulito de la cuenta regresiva.
                let (cx, cy) = (mx + 60.0, my + 110.0);
                d.draw_ring(Vector2::new(cx, cy), 26.0, 31.0, 0.0, 360.0, 40, Color::new(220, 224, 236, 255));
                d.draw_ring(Vector2::new(cx, cy), 26.0, 31.0, -90.0, -90.0 + 360.0 * n / 8.0, 40, Color::new(0, 103, 192, 255));
                let s = format!("{:.0}", n.ceil());
                ui::titulo_de(d, &s, cx as i32 - ui::medir_titulo(&s, 28) / 2, cy as i32 - 17, 28, Color::new(0, 103, 192, 255));
                ui::parrafo(d, "Su equipo se reiniciará para instalar actualizaciones importantes (que nadie pidió). Lo que no esté guardado, se pierde.", (mx + 110.0) as i32, (my + 74.0) as i32, 19, (mw - 130.0) as i32, tinta);
                botones(d, "Guardar y posponer", "Reiniciar ahora", self.eleccion);
            }
            Evento::Reiniciando => {
                d.draw_rectangle(0, 0, ancho, alto, Color::new(0, 90, 200, 255));
                // Los puntitos que dan vueltas (el anillo de carga del pack, si esta).
                if !ui::cargando(d, w / 2.0, h * 0.4, 72.0, self.t) {
                    for k in 0..6 {
                        let a = self.t * 4.0 - k as f32 * 0.35;
                        let (x, y) = (w / 2.0 + a.cos() * 26.0, h * 0.4 + a.sin() * 26.0);
                        d.draw_circle(x as i32, y as i32, 4.0 - k as f32 * 0.4, Color::new(255, 255, 255, 255 - k * 30));
                    }
                }
                let pct = ((self.t / 5.0) * 100.0).min(100.0);
                let s = format!("Trabajando en las actualizaciones  {pct:.0}%");
                ui::texto(d, &s, (ancho - ui::medir(&s, 28)) / 2, (h * 0.5) as i32, 28, Color::WHITE);
                let s2 = "No apague el equipo. (No guardó. Ya sabe lo que pasa.)";
                ui::texto(d, s2, (ancho - ui::medir(s2, 20)) / 2, (h * 0.5) as i32 + 46, 20, Color::new(200, 222, 255, 255));
            }
            Evento::Rubrica(k, cual) => {
                ventana(d, "Outlook · Bandeja de entrada (1)", Color::new(120, 40, 50, 255));
                d.draw_circle((mx + 44.0) as i32, (my + 88.0) as i32, 22.0, Color::new(200, 90, 70, 255));
                ui::titulo_de(d, "K", (mx + 36.0) as i32, (my + 72.0) as i32, 28, Color::WHITE);
                ui::texto(d, "Profe Koch", (mx + 78.0) as i32, (my + 70.0) as i32, 19, tinta);
                ui::texto(d, &format!("Para: {} · Sección 10 · 23:{:02}", CURSOS[k].nombre(), self.minutos()), (mx + 78.0) as i32, (my + 94.0) as i32, 15, Color::new(110, 110, 130, 255));
                ui::parrafo(d, RUBRICAS[cual], (mx + 24.0) as i32, (my + 128.0) as i32, 20, (mw - 48.0) as i32, tinta);
                let s = format!("{}: ...ok (-15%)", ui::boton(e.nombre("aceptar")));
                ui::texto(d, &s, (mx + mw) as i32 - 24 - ui::medir(&s, 18), (my + mh) as i32 - 36, 18, Color::new(150, 50, 60, 255));
            }
            Evento::Companero(quien) => {
                let nombres = ["Pablo", "Andrea", "Diego", "Majo"];
                ventana(d, &format!("{} (el de la compu de al lado)", nombres[quien]), Color::new(40, 130, 80, 255));
                let pedidos = [
                    "¿Me explicás qué es el z-buffer? Porfa porfa. Te invito un café. (No tengo pisto.)",
                    "¿Me ayudás con el autómata? Me acepta todo. Hasta mi nombre.",
                    "Mi merge borró la carpeta del proyecto. ¿Es grave? Decime que no es grave.",
                    "¿Cómo se pone el FOV? Mi render parece pez.",
                ];
                // Como globo de chat.
                caja(d, mx + 24.0, my + 66.0, mw - 48.0, mh - 160.0, 14.0, Color::new(225, 245, 230, 255));
                ui::parrafo(d, pedidos[quien], (mx + 40.0) as i32, (my + 80.0) as i32, 20, (mw - 80.0) as i32, tinta);
                ui::texto(d, "(si le ayuda, le debe una... y las deudas en el CIT se pagan)", (mx + 24.0) as i32, (my + mh - 92.0) as i32, 15, Color::new(100, 110, 120, 255));
                botones(d, "Ayudarle", "Ahorita no", self.eleccion);
            }
            Evento::Ayudando => {
                let s = "Explicándole a su compañero...";
                ui::titulo_de(d, s, (ancho - ui::medir_titulo(s, 30)) / 2, (h * 0.78) as i32, 30, Color::new(170, 255, 190, 255));
                let bw = w * 0.3;
                caja(d, (w - bw) / 2.0, h * 0.78 + 46.0, bw, 10.0, 5.0, Color::new(0, 0, 0, 160));
                caja(d, (w - bw) / 2.0, h * 0.78 + 46.0, bw * (self.t / 6.0).min(1.0), 10.0, 5.0, BIEN);
            }
            Evento::Teams(teoria, i) => self.dibujar_teams(d, e, teoria, i, koch, ancho, alto),
            _ => {}
        }
    }

    /// La llamada de Teams: a la derecha (Koch sale en el proyector, a la izquierda).
    #[allow(clippy::too_many_arguments)]
    fn dibujar_teams(&self, d: &mut RaylibDrawHandle, e: &Entrada, teoria: bool, i: usize, koch: Option<&Texture2D>, ancho: i32, alto: i32) {
        let (w, h) = (ancho as f32, alto as f32);
        let p = if teoria { &TEORIA[i % TEORIA.len()] } else { &GRAFICAS[i % GRAFICAS.len()] };
        let entra = 1.0 - (1.0 - suave(self.t / 0.35)).powi(2);
        let (tw, th) = ((w * 0.44).max(440.0), h * 0.78);
        let (tx, ty) = (w - (tw + 24.0) * entra, h * 0.12);
        caja(d, tx + 6.0, ty + 10.0, tw, th, 14.0, Color::new(0, 0, 0, 120));
        caja(d, tx, ty, tw, th, 14.0, Color::new(36, 36, 52, 250));
        caja(d, tx, ty, tw, 50.0, 14.0, Color::new(98, 100, 167, 255));
        d.draw_rectangle(tx as i32, (ty + 34.0) as i32, tw as i32, 16, Color::new(98, 100, 167, 255));
        let duracion = format!("{:02}:{:02}", (self.t as i32) / 60, (self.t as i32) % 60);
        ui::texto(d, "Teams · Profe Koch", (tx + 18.0) as i32, (ty + 14.0) as i32, 20, Color::WHITE);
        ui::texto(d, &duracion, (tx + tw) as i32 - 20 - ui::medir(&duracion, 18), (ty + 15.0) as i32, 18, Color::new(230, 230, 255, 255));
        // El cuadrito de video de Koch, con el anillo de "esta hablando".
        let lado = 120.0;
        let (vx, vy) = (tx + 20.0, ty + 66.0);
        let habla = 0.5 + 0.5 * (self.total * 9.0).sin();
        caja(d, vx - 4.0, vy - 4.0, lado + 8.0, lado + 8.0, 10.0, Color::new(140, 140, 230, (120.0 + 120.0 * habla) as u8));
        if let Some(t) = koch {
            let fuente = Rectangle::new(0.0, 0.0, t.width as f32, t.height as f32);
            d.draw_texture_pro(t, fuente, Rectangle::new(vx, vy, lado, lado), Vector2::zero(), 0.0, Color::WHITE);
        } else {
            caja(d, vx, vy, lado, lado, 8.0, Color::new(200, 90, 70, 255));
        }
        let encabezado = if teoria { "Pregunta rápida de Teoría (+8%):" } else { "Pregunta rápida de Gráficas (+8%):" };
        ui::texto(d, encabezado, (vx + lado + 18.0) as i32, vy as i32, 16, Color::new(190, 190, 230, 255));
        let fin = ui::parrafo(d, p.texto, (vx + lado + 18.0) as i32, vy as i32 + 24, 20, (tw - lado - 58.0) as i32, ui::TEXTO);
        let oy = (fin as f32).max(vy + lado) + 18.0;
        let textos: Vec<String> = p.opciones.iter().map(|s| s.to_string()).collect();
        self.botones(d, e, &textos, tx + 20.0, oy, tw - 40.0, 18, Some(self.eleccion));
        // La cuenta regresiva (una barra que se acaba).
        let resta = (1.0 - self.t / 14.0).clamp(0.0, 1.0);
        caja(d, tx + 20.0, ty + th - 26.0, tw - 40.0, 8.0, 4.0, Color::new(20, 20, 30, 255));
        caja(d, tx + 20.0, ty + th - 26.0, (tw - 40.0) * resta, 8.0, 4.0, if resta < 0.3 { MAL } else { Color::new(160, 160, 255, 255) });
    }

    fn dibujar_final(&self, d: &mut RaylibDrawHandle, aceptar: &str, salir: &str, ancho: i32, alto: i32) {
        let Some(notas) = self.notas else { return };
        let (w, h) = (ancho as f32, alto as f32);
        let paso = notas.iter().all(|&n| n >= PASA_CON);
        d.draw_rectangle(0, 0, ancho, alto, Color::new(8, 6, 22, (suave(self.t / 0.8) * 170.0) as u8));
        // Confeti si paso todo.
        if paso {
            let colores = [Color::new(255, 110, 150, 255), Color::new(110, 220, 255, 255), Color::new(255, 220, 90, 255), Color::new(140, 255, 150, 255), Color::new(200, 140, 255, 255)];
            for k in 0..90 {
                let semilla = k as f32 * 12.9898;
                let x = (semilla.sin() * 43758.545).fract().abs() * w;
                let caida = ((self.t * (0.18 + (semilla * 0.7).sin().abs() * 0.12)) + (semilla * 1.7).cos().abs()).fract();
                let y = caida * (h + 40.0) - 20.0;
                let giro = self.t * 4.0 + semilla;
                let ancho_c = 10.0 * giro.cos().abs() + 2.0;
                d.draw_rectangle((x + 20.0 * (self.t + semilla).sin()) as i32, y as i32, ancho_c as i32, 6, colores[k % colores.len()]);
            }
        }
        let titulo = "23:59 · ENTREGADO";
        let tam = (h * 0.085) as i32;
        let k0 = suave(self.t / 0.4);
        ui::titulo_de(d, titulo, (ancho - ui::medir_titulo(titulo, tam)) / 2, (h * 0.05 - 30.0 * (1.0 - k0)) as i32, tam, con_alfa(Color::new(255, 220, 140, 255), k0));
        let (cw, ch) = ((w * 0.24).min(330.0), h * 0.36);
        let total = 3.0 * cw + 2.0 * 24.0;
        for (k, c) in CURSOS.iter().enumerate() {
            // Cada tarjeta entra volteandose (de canto a de frente), una tras otra.
            let u = ((self.t - 0.4 - 0.35 * k as f32) / 0.35).clamp(0.0, 1.0);
            if u <= 0.0 {
                continue;
            }
            let giro = suave(u);
            let x0 = (w - total) / 2.0 + k as f32 * (cw + 24.0);
            let y = h * 0.23;
            let ancho_t = cw * giro;
            let x = x0 + (cw - ancho_t) / 2.0;
            caja(d, x + 5.0, y + 8.0, ancho_t, ch, 14.0, Color::new(0, 0, 0, 120));
            caja(d, x, y, ancho_t, ch, 14.0, Color::new(30, 28, 50, 250));
            caja(d, x, y, ancho_t, 8.0, 4.0, c.color());
            if giro < 0.6 {
                continue;
            }
            let nombre_y = y as i32 + (h * 0.07) as i32;
            ui::parrafo(d, c.nombre(), x as i32 + 18, nombre_y, 18, ancho_t as i32 - 36, Color::new(230, 230, 250, 255));
            let n = notas[k];
            let nota = n.to_string();
            let tn = (h * 0.13) as i32;
            let color = if n >= 90 { BIEN } else if n >= PASA_CON { Color::new(255, 225, 110, 255) } else { MAL };
            ui::titulo_de(d, &nota, (x + (ancho_t - ui::medir_titulo(&nota, tn) as f32) / 2.0) as i32, (y + ch * 0.58) as i32 - tn / 2, tn, color);
            // La insignia de la nota (con alas si fue excelente), arriba de la tarjeta.
            let cual = if n >= 100 {
                ui::Insignia::AlasAzul
            } else if n >= 90 {
                ui::Insignia::AlasRoja
            } else if n >= PASA_CON {
                ui::Insignia::GemaVerde
            } else {
                ui::Insignia::GemaGris
            };
            let escala = (h / 300.0).max(1.5) * if matches!(cual, ui::Insignia::AlasAzul | ui::Insignia::AlasRoja) { 1.0 } else { 1.2 };
            ui::insignia(d, cual, x + ancho_t / 2.0, y - 4.0, escala, Color::WHITE);
            let veredicto = if n >= 100 { "PERFECTO" } else if n >= 90 { "EXCELENTE" } else if n >= PASA_CON { "APROBADO" } else { "REPROBADO" };
            ui::titulo_de(d, veredicto, (x + (ancho_t - ui::medir_titulo(veredicto, 24) as f32) / 2.0) as i32, (y + ch) as i32 - 50, 24, color);
        }
        if self.t > 1.6 {
            let promedio = notas.iter().sum::<u32>() as f32 / 3.0;
            let s = format!("Promedio: {promedio:.1}");
            ui::titulo_de(d, &s, (ancho - ui::medir_titulo(&s, 30)) / 2, (h * 0.62) as i32, 30, if paso { BIEN } else { ui::TEXTO });
        }
        if self.t > 2.5 {
            let pie = format!("{aceptar}: otra noche (distinta)      {salir}: despegar");
            ui::subtitulo(d, &pie, (ancho - ui::medir(&pie, 22)) / 2, (h * 0.69) as i32, 22, Color::new(255, 220, 255, 255));
        }
    }
}

/// La tabla con lo que contesto el jugador (lo que falta y no se contesto se queda como el bueno).
fn delta_jugador_seguro(au: &Automata, ronda: &RondaTeoria) -> [[usize; 2]; 4] {
    let mut t = au.delta;
    for ((q, s), r) in &ronda.faltan {
        if let Some(r) = r {
            t[*q][*s] = *r;
        }
    }
    t
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn trabajar(e: &mut Entrega) {
        e.mensaje = None;
        e.cambiar(Fase::Trabajando);
        e.proximo_evento = 1e9;
    }

    #[test]
    fn los_automatas_aceptan_lo_que_dicen() {
        let ab = &AUTOMATAS[0];
        assert!(ab.acepta(&ab.delta, "aab").0 && ab.acepta(&ab.delta, "bab").0);
        assert!(!ab.acepta(&ab.delta, "aba").0 && !ab.acepta(&ab.delta, "b").0);
        let par = &AUTOMATAS[1];
        assert!(par.acepta(&par.delta, "abab").0 && !par.acepta(&par.delta, "ab").0);
        let bb = &AUTOMATAS[2];
        assert!(bb.acepta(&bb.delta, "abba").0 && !bb.acepta(&bb.delta, "abab").0);
        let sin_aa = &AUTOMATAS[5];
        assert!(sin_aa.acepta(&sin_aa.delta, "abab").0 && !sin_aa.acepta(&sin_aa.delta, "baab").0);
        let b3 = &AUTOMATAS[7];
        assert!(b3.acepta(&b3.delta, "bbab").0 && !b3.acepta(&b3.delta, "bb").0);
        for a in &AUTOMATAS {
            for fila in &a.delta[..a.estados] {
                assert!(fila.iter().all(|&q| q < a.estados), "{}: transicion a un estado que no existe", a.lenguaje);
            }
        }
    }

    #[test]
    fn contestar_bien_sube_el_proyecto() {
        let mut e = Entrega::new(7);
        trabajar(&mut e);
        // Graficas: la opcion buena.
        let buena = e.graficas.opciones.iter().position(|&i| i == e.graficas.bug).unwrap();
        e.contestar(Curso::Graficas, buena);
        assert_eq!(e.progreso[0], 20.0);
        // Software: la buena segun el conflicto.
        let buena = [Resolucion::Head, Resolucion::Rama, Resolucion::Ambas].iter().position(|r| *r == CONFLICTOS[e.software].correcta).unwrap();
        e.contestar(Curso::Software, buena);
        assert_eq!(e.progreso[2], 20.0);
        // Teoria: contestando lo que falta con la tabla buena.
        let faltan: Vec<(usize, usize)> = e.teoria.faltan.iter().map(|f| f.0).collect();
        for (q, s) in faltan {
            e.contestar(Curso::Teoria, AUTOMATAS[e.teoria.automata].delta[q][s]);
        }
        assert_eq!(e.progreso[1], 25.0);
    }

    #[test]
    fn equivocarse_baja_y_bloquea() {
        let mut e = Entrega::new(9);
        trabajar(&mut e);
        e.progreso[0] = 30.0;
        let mala = e.graficas.opciones.iter().position(|&i| i != e.graficas.bug).unwrap();
        e.contestar(Curso::Graficas, mala);
        assert!(e.progreso[0] < 30.0 && e.bloqueo > 0.0);
    }

    #[test]
    fn a_las_23_59_se_entrega() {
        let mut e = Entrega::new(3);
        trabajar(&mut e);
        e.progreso = [100.0, 70.0, 40.0];
        e.reloj = DURACION - 0.01;
        e.energia = 1.0;
        e.actualizar(&Entrada::default(), 0.1);
        assert!(e.terminado());
        assert_eq!(e.resultado_final(), Some([100, 70, 40]));
    }

    #[test]
    fn sin_energia_se_duerme() {
        let mut e = Entrega::new(5);
        trabajar(&mut e);
        e.energia = 0.0001;
        e.actualizar(&Entrada::default(), 0.1);
        assert!(e.dormido() && e.se_durmio);
    }

    #[test]
    fn la_noche_trae_desastres_variados() {
        let mut vistos = std::collections::HashSet::new();
        for semilla in 1..80 {
            let mut e = Entrega::new(semilla * 131);
            trabajar(&mut e);
            e.lanzar_evento();
            if let Some(ev) = e.ultimo_evento {
                vistos.insert(std::mem::discriminant(&ev));
            }
        }
        assert!(vistos.len() >= 7, "salen pocos tipos de desastre: {}", vistos.len());
    }
}
