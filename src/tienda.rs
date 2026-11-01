//! La tienda (desde el menu principal): skins para la nave, estelas y pilotos extra, pagados
//! con las notas que se recogen volando. Solo se venden los EXTRAS: lo que es premio del pase
//! se gana solo en el pase. Cada skin combina una paleta con adornos
//! propios sobre una copia de la nave, conservando las partes animadas.
//!
//! A la derecha, la nave de muestra gira en su vitrina con la skin escogida (aunque
//! no se haya comprado: probar es gratis). En las estelas, vuela en circulos con la
//! estela puesta.

use crate::entrada::Entrada;
use crate::matematica::Vec3;
use crate::obj::Modelo;
use crate::progreso::{Progreso, ESTELAS};
use crate::sonido::Efecto;
use crate::ui;
use raylib::prelude::*;

/// Un color de un material: (material, difuso, emisivo si cambia).
type Cambio = (&'static str, [f32; 3], Option<[f32; 3]>);

pub struct Skin {
    pub id: &'static str,
    pub nombre: &'static str,
    pub precio: u32,
    /// Lo que opina GLADAS (sale debajo de la vitrina).
    pub texto: &'static str,
    pub colores: &'static [Cambio],
    /// Brillo de metal pulido en el cuerpo.
    pub brillante: bool,
}

pub const SKINS: [Skin; 29] = [
    Skin {
        id: "lila",
        nombre: "NPC con licencia",
        precio: 0,
        texto: "Placa NPC y antena de tres barritas. Se cree personaje principal desde que pasó el examen de parqueo. A la tercera.",
        colores: &[],
        brillante: false,
    },
    Skin {
        id: "miku",
        nombre: "Puerro con Wi-Fi",
        precio: 60,
        texto: "Puerro gigante en el techo y carrocería cian. Recibe conciertos, no llamadas de su ex. La señal depende del ritmo.",
        colores: &[
            ("piel_lila", [0.22, 0.78, 0.78], None),
            ("pelo_violeta", [0.08, 0.50, 0.56], None),
            ("rojo", [1.0, 0.32, 0.62], None),
            ("rojo_oscuro", [0.55, 0.10, 0.34], None),
            ("rosa", [0.40, 1.0, 0.95], Some([0.08, 0.22, 0.22])),
            ("tobera_luz", [0.40, 1.0, 0.95], Some([0.30, 0.95, 0.90])),
            ("llama", [0.40, 1.0, 0.95], Some([0.25, 0.90, 0.85])),
        ],
        brillante: false,
    },
    Skin {
        id: "chapina",
        nombre: "Guate sin lag",
        precio: 70,
        texto: "Bandera en el techo, turbina de antorcha y azul patrio. Salió de la Roosevelt hace dos horas. Avanzó hasta Marte.",
        colores: &[
            ("piel_lila", [0.40, 0.68, 0.96], None),
            ("pelo_violeta", [0.95, 0.95, 1.0], None),
            ("rojo", [0.25, 0.50, 0.95], None),
            ("rojo_oscuro", [0.14, 0.30, 0.66], None),
            ("rosa", [1.0, 1.0, 1.0], Some([0.15, 0.15, 0.18])),
            ("llama", [1.0, 0.70, 0.25], Some([1.0, 0.55, 0.15])),
            ("tobera_luz", [1.0, 0.75, 0.30], Some([1.0, 0.60, 0.20])),
        ],
        brillante: false,
    },
    Skin {
        id: "camioneta",
        nombre: "Bus del multiverso",
        precio: 90,
        texto: "Rótulo HAY CAMPO, pintura de camioneta y cero vergüenza. El ayudante cobra hasta por la gravedad. Parada: Saturno.",
        colores: &[
            ("piel_lila", [0.98, 0.78, 0.12], None),
            ("pelo_violeta", [0.10, 0.58, 0.24], None),
            ("rojo", [0.92, 0.12, 0.10], None),
            ("rojo_oscuro", [0.55, 0.06, 0.05], None),
            ("rosa", [1.0, 0.95, 0.40], Some([0.20, 0.18, 0.04])),
            ("metal", [0.90, 0.90, 0.95], None),
            ("llama", [1.0, 0.45, 0.10], Some([1.0, 0.35, 0.05])),
            ("tobera_luz", [1.0, 0.50, 0.15], Some([1.0, 0.40, 0.10])),
        ],
        brillante: false,
    },
    Skin {
        id: "luka",
        nombre: "Delulu espacial",
        precio: 100,
        texto: "Corazón rosa en el techo. Júpiter le dio una vuelta y ya anda planeando la boda. GLADAS ya apartó el juzgado.",
        colores: &[
            ("piel_lila", [0.98, 0.62, 0.76], None),
            ("pelo_violeta", [0.80, 0.34, 0.56], None),
            ("rojo", [0.98, 0.78, 0.30], None),
            ("rojo_oscuro", [0.70, 0.48, 0.12], None),
            ("rosa", [1.0, 0.85, 0.40], Some([0.25, 0.18, 0.05])),
        ],
        brillante: false,
    },
    Skin {
        id: "shuco",
        nombre: "Shuco supersónico",
        precio: 120,
        texto: "Pan, salchicha y mostaza sobre la cabina. CON TODO incluye jalapeño y turbulencia. El combo no trae servilletas.",
        colores: &[
            ("piel_lila", [0.86, 0.60, 0.30], None),
            ("pelo_violeta", [0.42, 0.66, 0.18], None),
            ("rojo", [0.72, 0.24, 0.14], None),
            ("rojo_oscuro", [0.48, 0.14, 0.08], None),
            ("rosa", [1.0, 0.85, 0.15], Some([0.25, 0.20, 0.0])),
            ("metal", [0.92, 0.88, 0.80], None),
            ("llama", [1.0, 0.85, 0.20], Some([0.90, 0.70, 0.10])),
            ("tobera_luz", [1.0, 0.85, 0.25], Some([0.90, 0.70, 0.12])),
        ],
        brillante: false,
    },
    Skin {
        id: "quetzal",
        nombre: "Quetzal con aura",
        precio: 150,
        texto: "Verde quetzal con lentes oscuros. Posa antes de cada aterrizaje. El aura se pierde al chocar, los lentes no.",
        colores: &[
            ("piel_lila", [0.10, 0.66, 0.40], None),
            ("pelo_violeta", [0.04, 0.42, 0.30], None),
            ("rojo", [0.88, 0.10, 0.14], None),
            ("rojo_oscuro", [0.55, 0.05, 0.08], None),
            ("rosa", [0.55, 1.0, 0.60], Some([0.10, 0.25, 0.10])),
            ("llama", [0.50, 1.0, 0.50], Some([0.35, 0.95, 0.40])),
            ("tobera_luz", [0.55, 1.0, 0.55], Some([0.40, 0.95, 0.45])),
        ],
        brillante: false,
    },
    Skin {
        id: "sombra",
        nombre: "Modo no fui yo",
        precio: 180,
        texto: "Negro, neón rojo y lentes de incógnito. El radar la vio chocar. Ella insiste en que era su primo.",
        colores: &[
            ("piel_lila", [0.14, 0.13, 0.17], None),
            ("pelo_violeta", [0.24, 0.20, 0.28], None),
            ("rojo", [1.0, 0.10, 0.18], Some([0.55, 0.02, 0.06])),
            ("rojo_oscuro", [0.60, 0.04, 0.08], None),
            ("rosa", [1.0, 0.12, 0.20], Some([0.70, 0.05, 0.10])),
            ("metal", [0.30, 0.30, 0.34], None),
            ("llama", [1.0, 0.15, 0.20], Some([1.0, 0.08, 0.12])),
            ("tobera_luz", [1.0, 0.15, 0.20], Some([1.0, 0.10, 0.14])),
            ("laser", [1.0, 0.20, 0.25], Some([1.0, 0.10, 0.15])),
        ],
        brillante: true,
    },
    Skin {
        id: "oro",
        nombre: "Premium sin saldo",
        precio: 400,
        texto: "Corona dorada y placa +9999. Parece millonaria hasta que intenta pagar el parqueo con puntos del pase.",
        colores: &[
            ("piel_lila", [0.96, 0.74, 0.22], None),
            ("pelo_violeta", [0.74, 0.50, 0.10], None),
            ("rojo", [1.0, 0.92, 0.55], None),
            ("rojo_oscuro", [0.62, 0.40, 0.08], None),
            ("rosa", [1.0, 0.95, 0.70], Some([0.40, 0.32, 0.10])),
            ("metal", [1.0, 0.86, 0.45], None),
            ("llama", [1.0, 0.85, 0.35], Some([1.0, 0.75, 0.25])),
            ("tobera_luz", [1.0, 0.88, 0.40], Some([1.0, 0.78, 0.30])),
        ],
        brillante: true,
    },
    Skin {
        id: "aurora",
        nombre: "Farmeadora de aura",
        precio: 130,
        texto: "Corona, lentes y verde menta: aterriza de lado para el clip. +67 de aura si nadie vio los tres intentos anteriores.",
        colores: &[
            ("piel_lila", [0.18, 0.86, 0.64], None),
            ("pelo_violeta", [0.3, 0.12, 0.56], None),
            ("rojo", [0.65, 0.35, 1.0], None),
            ("rojo_oscuro", [0.3, 0.12, 0.56], None),
            ("rosa", [0.65, 0.35, 1.0], Some([0.65, 0.35, 1.0])),
            ("llama", [0.65, 0.35, 1.0], Some([0.65, 0.35, 1.0])),
            ("tobera_luz", [0.65, 0.35, 1.0], Some([0.65, 0.35, 1.0])),
        ],
        brillante: true,
    },
    Skin {
        id: "cobre",
        nombre: "Café y ansiedad",
        precio: 160,
        texto: "Taza gigante en el techo, café visible y motor azul. Lleva tres entregas, dos exámenes y cero horas de sueño.",
        colores: &[
            ("piel_lila", [0.78, 0.36, 0.16], None),
            ("pelo_violeta", [0.3, 0.13, 0.08], None),
            ("rojo", [0.18, 0.6, 1.0], None),
            ("rojo_oscuro", [0.3, 0.13, 0.08], None),
            ("rosa", [0.18, 0.6, 1.0], Some([0.18, 0.6, 1.0])),
            ("llama", [0.18, 0.6, 1.0], Some([0.18, 0.6, 1.0])),
            ("tobera_luz", [0.18, 0.6, 1.0], Some([0.18, 0.6, 1.0])),
        ],
        brillante: true,
    },
    Skin {
        id: "hielo",
        nombre: "Modo avión literal",
        precio: 200,
        texto: "Avioncito blanco encima de otra nave. No recibe mensajes: está en modo avión al cuadrado. Igual deja el visto.",
        colores: &[
            ("piel_lila", [0.88, 0.95, 1.0], None),
            ("pelo_violeta", [0.2, 0.42, 0.72], None),
            ("rojo", [0.45, 0.8, 1.0], None),
            ("rojo_oscuro", [0.2, 0.42, 0.72], None),
            ("rosa", [0.45, 0.8, 1.0], Some([0.45, 0.8, 1.0])),
            ("llama", [0.45, 0.8, 1.0], Some([0.45, 0.8, 1.0])),
            ("tobera_luz", [0.45, 0.8, 1.0], Some([0.45, 0.8, 1.0])),
        ],
        brillante: true,
    },
    Skin {
        id: "uva",
        nombre: "67 de ping",
        precio: 240,
        texto: "Un 67 neón montado sobre la cabina morada. No sabe qué significa; lo gritó el copiloto y ya mandaron a hacer la placa.",
        colores: &[
            ("piel_lila", [0.42, 0.13, 0.65], None),
            ("pelo_violeta", [0.18, 0.04, 0.32], None),
            ("rojo", [0.68, 1.0, 0.15], None),
            ("rojo_oscuro", [0.18, 0.04, 0.32], None),
            ("rosa", [0.68, 1.0, 0.15], Some([0.68, 1.0, 0.15])),
            ("llama", [0.68, 1.0, 0.15], Some([0.68, 1.0, 0.15])),
            ("tobera_luz", [0.68, 1.0, 0.15], Some([0.68, 1.0, 0.15])),
        ],
        brillante: true,
    },

    // Las carrocerias: ademas de los colores, cambian la forma de la nave entera.
    Skin {
        id: "platillo",
        nombre: "Platillo de Roswell",
        precio: 180,
        texto: "Carrocería de platillo volador, con anillo y lucecitas. Dicen que lo vieron en Roswell. En la Roosevelt también, pero era un tuk-tuk.",
        colores: &[
            ("piel_lila", [0.72, 0.80, 0.76], None),
            ("pelo_violeta", [0.35, 0.85, 0.45], None),
            ("rojo", [0.30, 0.85, 0.55], None),
            ("rojo_oscuro", [0.15, 0.45, 0.30], None),
            ("rosa", [0.50, 1.0, 0.60], Some([0.20, 0.50, 0.25])),
            ("llama", [0.50, 1.0, 0.60], Some([0.40, 0.95, 0.50])),
            ("tobera_luz", [0.50, 1.0, 0.60], Some([0.40, 0.95, 0.50])),
        ],
        brillante: true,
    },
    Skin {
        id: "cohete",
        nombre: "Cohete del 69",
        precio: 200,
        texto: "Carrocería de cohete: punta a franjas y tres aletas, como en las caricaturas. Llega a la Luna en ocho días. En ocho minutos si no hay tráfico.",
        colores: &[
            ("piel_lila", [0.95, 0.95, 0.95], None),
            ("pelo_violeta", [0.90, 0.15, 0.15], None),
            ("rojo", [0.90, 0.15, 0.15], None),
            ("rojo_oscuro", [0.55, 0.08, 0.08], None),
            ("rosa", [1.0, 0.65, 0.20], Some([0.30, 0.15, 0.02])),
            ("llama", [1.0, 0.60, 0.15], Some([1.0, 0.50, 0.10])),
            ("tobera_luz", [1.0, 0.65, 0.20], Some([1.0, 0.55, 0.15])),
        ],
        brillante: true,
    },
    Skin {
        id: "tiburon",
        nombre: "Tiburón espacial",
        precio: 220,
        texto: "Carrocería de tiburón: aleta, cola y dos filas de dientes. Nada en el vacío. Los asteroides le tienen miedo. Bill Nye no, pero toma notas.",
        colores: &[
            ("piel_lila", [0.48, 0.58, 0.68], None),
            ("pelo_violeta", [0.20, 0.30, 0.45], None),
            ("rojo", [0.85, 0.85, 0.90], None),
            ("rojo_oscuro", [0.35, 0.42, 0.52], None),
            ("rosa", [0.80, 0.95, 1.0], Some([0.10, 0.15, 0.20])),
            ("llama", [0.40, 0.70, 1.0], Some([0.30, 0.60, 1.0])),
            ("tobera_luz", [0.40, 0.70, 1.0], Some([0.30, 0.60, 1.0])),
        ],
        brillante: false,
    },
    Skin {
        id: "dragon",
        nombre: "Dragón del Volcán de Fuego",
        precio: 260,
        texto: "Carrocería de dragón: alas de murciélago, cuernos y cola con punta. Nació en el Volcán de Fuego. Escupe lava, pero solo los domingos.",
        colores: &[
            ("piel_lila", [0.55, 0.12, 0.14], None),
            ("pelo_violeta", [0.12, 0.08, 0.08], None),
            ("rojo", [1.0, 0.50, 0.10], None),
            ("rojo_oscuro", [0.40, 0.06, 0.06], None),
            ("rosa", [1.0, 0.75, 0.15], Some([0.60, 0.35, 0.05])),
            ("llama", [1.0, 0.40, 0.05], Some([1.0, 0.35, 0.02])),
            ("tobera_luz", [1.0, 0.50, 0.10], Some([1.0, 0.40, 0.05])),
        ],
        brillante: false,
    },
    Skin {
        id: "tuktuk",
        nombre: "Tuk-tuk interestelar",
        precio: 150,
        texto: "Carrocería de tuk-tuk: tres llantas, techo de lámina y calcomanías. Le cabe uno adelante, tres atrás y un primo en el techo. Cobra Q5 por planeta.",
        colores: &[
            ("piel_lila", [0.92, 0.22, 0.22], None),
            ("pelo_violeta", [1.0, 0.80, 0.15], None),
            ("rojo", [1.0, 0.80, 0.15], None),
            ("rojo_oscuro", [0.55, 0.10, 0.10], None),
            ("rosa", [0.30, 0.80, 1.0], Some([0.08, 0.20, 0.25])),
        ],
        brillante: false,
    },
    Skin {
        id: "patito",
        nombre: "Patito de hule",
        precio: 170,
        texto: "Carrocería de patito de hule del tamaño de una nave. Flota en el espacio y en la tina. El réferi de Urano lo reconoce de algún lado.",
        colores: &[
            ("piel_lila", [1.0, 0.86, 0.15], None),
            ("pelo_violeta", [1.0, 0.55, 0.10], None),
            ("rojo", [1.0, 0.55, 0.10], None),
            ("rojo_oscuro", [0.70, 0.35, 0.05], None),
            ("rosa", [1.0, 0.95, 0.60], Some([0.25, 0.22, 0.10])),
        ],
        brillante: false,
    },
    // Las de las maravillas (ver maravillas.rs).
    Skin {
        id: "endurance",
        nombre: "No es momento de precauciones",
        precio: 280,
        texto: "El anillo de la Endurance alrededor de la nave, girando para tener gravedad. Doce módulos y ni uno con baño propio. Cada hora aquí son siete años de garantía.",
        colores: &[
            ("piel_lila", [0.90, 0.90, 0.88], None),
            ("pelo_violeta", [0.42, 0.44, 0.50], None),
            ("rojo", [0.62, 0.64, 0.70], None),
            ("rojo_oscuro", [0.30, 0.32, 0.38], None),
            ("rosa", [1.0, 0.85, 0.55], Some([0.9, 0.75, 0.45])),
            ("llama", [0.75, 0.85, 1.0], Some([0.65, 0.8, 1.0])),
            ("tobera_luz", [0.75, 0.85, 1.0], Some([0.65, 0.8, 1.0])),
        ],
        brillante: true,
    },
    Skin {
        id: "blip",
        nombre: "Blip-A",
        precio: 300,
        texto: "Casco de xenonita, motores de astrófago al rojo infrarrojo y la burbuja de Rocky pegada al costado, con Rocky adentro. Le pone acordes a la radio. Amaze, amaze, amaze.",
        colores: &[
            ("piel_lila", [0.36, 0.42, 0.38], None),
            ("pelo_violeta", [0.24, 0.28, 0.26], None),
            ("rojo", [0.55, 0.60, 0.52], None),
            ("rojo_oscuro", [0.20, 0.22, 0.20], None),
            ("rosa", [1.0, 0.25, 0.30], Some([0.9, 0.12, 0.15])),
            ("llama", [1.0, 0.15, 0.20], Some([0.95, 0.10, 0.15])),
            ("tobera_luz", [1.0, 0.15, 0.20], Some([0.95, 0.10, 0.15])),
        ],
        brillante: false,
    },
    // Mas carrocerias.
    Skin {
        id: "barrilete",
        nombre: "Barrilete de Sumpango",
        precio: 220,
        texto: "Un barrilete gigante de Sumpango pegado atrás de la nave: papel de china, flecos y la cola larga. El primero de noviembre vuela solito. Los otros días también, porque es una nave.",
        colores: &[
            ("piel_lila", [0.98, 0.92, 0.80], None),
            ("pelo_violeta", [0.85, 0.20, 0.35], None),
            ("rojo", [0.15, 0.55, 0.85], None),
            ("rojo_oscuro", [0.10, 0.35, 0.60], None),
            ("rosa", [1.0, 0.80, 0.20], Some([0.5, 0.35, 0.05])),
        ],
        brillante: false,
    },
    Skin {
        id: "pinata",
        nombre: "Piñata de siete picos",
        precio: 190,
        texto: "Siete picos de papel con flecos. Adentro trae dulces, según el vendedor. GLADAS le pidió a la piloto que no deje que nadie la agarre a palos en Neptuno.",
        colores: &[
            ("piel_lila", [1.0, 0.45, 0.75], None),
            ("pelo_violeta", [0.95, 0.85, 0.20], None),
            ("rojo", [0.25, 0.85, 0.60], None),
            ("rojo_oscuro", [0.15, 0.55, 0.40], None),
            ("rosa", [1.0, 0.95, 0.35], Some([0.6, 0.5, 0.1])),
        ],
        brillante: false,
    },
    Skin {
        id: "disco",
        nombre: "Fiebre del sábado espacial",
        precio: 210,
        texto: "Una bola disco que gira en el techo, luces de colores y bocinas. Saturno pidió que la estacione lejos del escenario: le roba el público.",
        colores: &[
            ("piel_lila", [0.18, 0.10, 0.30], None),
            ("pelo_violeta", [0.95, 0.35, 0.85], None),
            ("rojo", [0.30, 0.95, 0.95], None),
            ("rojo_oscuro", [0.12, 0.06, 0.20], None),
            ("rosa", [1.0, 0.40, 0.90], Some([1.0, 0.35, 0.85])),
            ("llama", [0.95, 0.40, 1.0], Some([0.9, 0.35, 1.0])),
            ("tobera_luz", [0.95, 0.40, 1.0], Some([0.9, 0.35, 1.0])),
        ],
        brillante: true,
    },
    Skin {
        id: "tars",
        nombre: "Humor al 75%",
        precio: 260,
        texto: "Los paneles de TARS a los lados de la nave, con su pantallita. Honestidad al 90%, humor al 75%. GLADAS pidió bajarle el humor. TARS le dijo que no.",
        colores: &[
            ("piel_lila", [0.55, 0.57, 0.60], None),
            ("pelo_violeta", [0.20, 0.21, 0.24], None),
            ("rojo", [0.35, 0.37, 0.40], None),
            ("rojo_oscuro", [0.15, 0.16, 0.18], None),
            ("rosa", [0.85, 0.95, 1.0], Some([0.6, 0.75, 0.9])),
        ],
        brillante: true,
    },
    // Brainrot italiano (los de 2025).
    Skin {
        id: "tralalero",
        nombre: "Tralalero Tralala",
        precio: 200,
        texto: "Aleta de tiburón, cola y tres tenis azules colgando de la nave. Nadie sabe para qué son los tenis. Nadie se atreve a preguntar.",
        colores: &[
            ("piel_lila", [0.35, 0.58, 0.88], None),
            ("pelo_violeta", [0.92, 0.94, 0.98], None),
            ("rojo", [0.15, 0.40, 0.95], None),
            ("rojo_oscuro", [0.18, 0.32, 0.55], None),
            ("rosa", [0.60, 0.85, 1.0], Some([0.15, 0.3, 0.5])),
        ],
        brillante: false,
    },
    Skin {
        id: "tungtung",
        nombre: "Tung Tung Tung Sahur",
        precio: 220,
        texto: "Carrocería de tronco, anillos de madera arriba y el bate al costado. Pasa a la hora del desayuno tocando la puerta. Tres veces. Siempre tres.",
        colores: &[
            ("piel_lila", [0.72, 0.50, 0.30], None),
            ("pelo_violeta", [0.48, 0.32, 0.18], None),
            ("rojo", [0.85, 0.66, 0.42], None),
            ("rojo_oscuro", [0.40, 0.26, 0.14], None),
            ("rosa", [1.0, 0.85, 0.55], Some([0.3, 0.2, 0.08])),
        ],
        brillante: false,
    },
    Skin {
        id: "bombardiro",
        nombre: "Bombardiro Crocodilo",
        precio: 240,
        texto: "Hocico de cocodrilo, alas de bombardero y dos hélices. En vez de bombas tira notas. GLADAS revisó dos veces. Bueno, casi siempre notas.",
        colores: &[
            ("piel_lila", [0.42, 0.50, 0.25], None),
            ("pelo_violeta", [0.25, 0.62, 0.22], None),
            ("rojo", [0.55, 0.60, 0.32], None),
            ("rojo_oscuro", [0.22, 0.28, 0.12], None),
            ("rosa", [1.0, 0.85, 0.10], Some([0.45, 0.35, 0.0])),
        ],
        brillante: false,
    },
    Skin {
        id: "ballerina",
        nombre: "Ballerina Cappuccina",
        precio: 180,
        texto: "Una taza de capuchino en el techo (con su corazón de espuma) y un tutú rosa en la cintura. Baila sola en la gravedad cero. Huele a café.",
        colores: &[
            ("piel_lila", [1.0, 0.70, 0.82], None),
            ("pelo_violeta", [0.55, 0.33, 0.18], None),
            ("rojo", [1.0, 0.50, 0.72], None),
            ("rojo_oscuro", [0.75, 0.30, 0.50], None),
            ("rosa", [1.0, 0.92, 0.85], Some([0.35, 0.25, 0.2])),
        ],
        brillante: false,
    },
];

/// La skin con ese id (la de fabrica si no existe).
pub fn indice_skin(id: &str) -> usize {
    SKINS.iter().position(|k| k.id == id).unwrap_or(0)
}

/// Una copia de la nave con los colores de la skin.
pub fn vestir(base: &Modelo, skin: &Skin) -> Modelo {
    let mut m = base.clone();
    let v = |c: [f32; 3]| Vec3::new(c[0], c[1], c[2]);
    for mat in &mut m.materiales {
        if let Some(&(_, kd, ke)) = skin.colores.iter().find(|c| c.0 == mat.nombre) {
            mat.kd = v(kd);
            if let Some(ke) = ke {
                mat.ke = v(ke);
            }
        }
        if skin.brillante && matches!(mat.nombre.as_str(), "piel_lila" | "pelo_violeta" | "metal" | "rojo") {
            mat.ks = v([0.9, 0.85, 0.75]);
            mat.ns = 48.0;
        }
    }
    crate::adornos::adornar(&mut m, skin.id);
    m
}

/// Solo se venden los cosmeticos que no son premios del pase.
pub fn skins_tienda() -> Vec<usize> {
    SKINS.iter().enumerate().filter(|(_, s)| !crate::progreso::PASE.contains(&crate::progreso::Premio::Skin(s.id))).map(|(i, _)| i).collect()
}

pub fn estelas_tienda() -> Vec<usize> {
    ESTELAS.iter().enumerate().filter(|(_, e)| !crate::progreso::PASE.contains(&crate::progreso::Premio::Estela(e.0))).map(|(i, _)| i).collect()
}

/// Los pilotos que se venden: los que tienen precio (los extras; ni los del pase ni los de fabrica).
pub fn personajes_tienda() -> Vec<usize> {
    crate::personajes::PERSONAJES.iter().enumerate().filter(|(_, f)| crate::personajes::precio(f.id).is_some()).map(|(i, _)| i).collect()
}

/// Lo que ensena la vitrina 3D (de la tienda, el casillero o el pase).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Muestra {
    /// La nave con esa skin; con `estela`, volando en circulos (`""`: sin estela).
    Nave { skin: usize, estela: Option<&'static str> },
    /// Ese personaje, bailando.
    Personaje(usize),
    /// Algo de la mochila (0..4 las lunas, 4.. los recuerdos), girando; si no se tiene, en sombra.
    Recuerdo { cual: usize, tiene: bool },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pestana {
    Naves,
    Estelas,
    Personajes,
}

pub enum AccionTienda {
    Nada,
    Cerrar,
}

pub struct Tienda {
    pub pestana: Pestana,
    cursor: usize,
    /// Segundos desde que se movio el cursor (la banda entra deslizandose).
    desde: f32,
    /// Segundos desde que se abrio (la lista entra una sola vez).
    abierta: f32,
    pub reloj: f32,
    /// Lo que dice la caja al comprar (o al no alcanzar): texto y segundos que lleva.
    aviso: Option<(String, f32)>,
    pub eventos: Vec<Efecto>,
}

impl Tienda {
    /// Abre con el cursor en la skin que se lleva puesta.
    pub fn new(p: &Progreso) -> Self {
        Tienda { pestana: Pestana::Naves, cursor: skins_tienda().iter().position(|&i| SKINS[i].id == p.skin).unwrap_or(0), desde: 1.0, abierta: 0.0, reloj: 0.0, aviso: None, eventos: Vec::new() }
    }

    fn filas(&self) -> usize {
        match self.pestana {
            Pestana::Naves => skins_tienda().len(),
            // La primera fila de estelas es "Sin estela".
            Pestana::Estelas => estelas_tienda().len() + 1,
            Pestana::Personajes => personajes_tienda().len(),
        }
    }

    /// Lo que se ve en la vitrina: la skin (la escogida en la lista, o la puesta) y la estela.
    pub fn vitrina(&self, p: &Progreso) -> Muestra {
        match self.pestana {
            Pestana::Naves => Muestra::Nave { skin: skins_tienda()[self.cursor], estela: None },
            Pestana::Estelas => {
                let estela = if self.cursor == 0 { None } else { Some(ESTELAS[estelas_tienda()[self.cursor - 1]].0) };
                Muestra::Nave { skin: indice_skin(&p.skin), estela: estela.or(Some("")) }
            }
            Pestana::Personajes => Muestra::Personaje(personajes_tienda()[self.cursor]),
        }
    }

    pub fn actualizar(&mut self, e: &Entrada, p: &mut Progreso, dt: f32) -> AccionTienda {
        self.desde += dt;
        self.abierta += dt;
        self.reloj += dt;
        if let Some((_, t)) = &mut self.aviso {
            *t += dt;
        }
        if e.pausa {
            self.eventos.push(Efecto::Carta);
            return AccionTienda::Cerrar;
        }
        if e.izquierda || e.derecha {
            const ORDEN: [Pestana; 3] = [Pestana::Naves, Pestana::Estelas, Pestana::Personajes];
            let k = ORDEN.iter().position(|&q| q == self.pestana).unwrap_or(0);
            self.pestana = ORDEN[if e.derecha { (k + 1) % 3 } else { (k + 2) % 3 }];
            self.cursor = match self.pestana {
                Pestana::Naves => skins_tienda().iter().position(|&i| SKINS[i].id == p.skin).unwrap_or(0),
                Pestana::Estelas => p.estela.as_deref().and_then(|id| estelas_tienda().iter().position(|&i| ESTELAS[i].0 == id)).map_or(0, |i| i + 1),
                Pestana::Personajes => personajes_tienda().iter().position(|&i| crate::personajes::PERSONAJES[i].id == p.personaje).unwrap_or(0),
            };
            self.desde = 0.0;
            self.aviso = None;
            self.eventos.push(Efecto::Carta);
        }
        let n = self.filas();
        if e.abajo {
            self.cursor = (self.cursor + 1) % n;
            self.desde = 0.0;
            self.eventos.push(Efecto::Carta);
        }
        if e.arriba {
            self.cursor = (self.cursor + n - 1) % n;
            self.desde = 0.0;
            self.eventos.push(Efecto::Carta);
        }
        if e.aceptar {
            self.escoger(p);
        }
        AccionTienda::Nada
    }

    /// Compra (si alcanza) o se pone lo escogido.
    fn escoger(&mut self, p: &mut Progreso) {
        if self.pestana == Pestana::Personajes {
            let f = &crate::personajes::PERSONAJES[personajes_tienda()[self.cursor]];
            if p.personaje == f.id {
                return;
            }
            let ya_era = p.tiene_personaje(f.id);
            let precio = crate::personajes::precio(f.id).unwrap_or(0);
            let texto = if !p.personaje_escogido(f.id) {
                self.eventos.push(Efecto::Fallo);
                format!("Le faltan {} notas. Vaya a volar un rato y regrese.", precio - p.notas.min(precio))
            } else if ya_era {
                self.eventos.push(Efecto::Acierto);
                format!("{}: a bordo.", f.nombre)
            } else {
                self.eventos.push(Efecto::Sello);
                format!("¡{} se sube a la nave! Le quedaron {} notas.", f.nombre, p.notas)
            };
            self.aviso = Some((texto, 0.0));
            return;
        }
        let (ya_era, precio, nombre) = match self.pestana {
            Pestana::Naves => {
                let k = &SKINS[skins_tienda()[self.cursor]];
                if p.skin == k.id {
                    return;
                }
                (p.tiene_skin(k.id), k.precio, k.nombre)
            }
            Pestana::Estelas if self.cursor == 0 => {
                if p.estela.take().is_some() {
                    p.guardar();
                    self.eventos.push(Efecto::Carta);
                    self.aviso = Some(("Sin estela. Discreta, como ninja.".into(), 0.0));
                }
                return;
            }
            Pestana::Estelas => {
                let (id, nombre, precio) = ESTELAS[estelas_tienda()[self.cursor - 1]];
                if p.estela.as_deref() == Some(id) {
                    return;
                }
                (p.estelas.contains(id), precio, nombre)
            }
            Pestana::Personajes => return,
        };
        let pudo = match self.pestana {
            Pestana::Naves => p.skin_escogida(SKINS[skins_tienda()[self.cursor]].id),
            Pestana::Estelas => p.estela_escogida(ESTELAS[estelas_tienda()[self.cursor - 1]].0),
            Pestana::Personajes => false,
        };
        let texto = if !pudo {
            self.eventos.push(Efecto::Fallo);
            format!("Le faltan {} notas. Vaya a volar un rato y regrese.", precio - p.notas.min(precio))
        } else if ya_era {
            self.eventos.push(Efecto::Acierto);
            format!("{nombre}: puesta.")
        } else {
            self.eventos.push(Efecto::Sello);
            format!("¡{nombre} es suya! Le quedaron {} notas.", p.notas)
        };
        self.aviso = Some((texto, 0.0));
    }

    /// Dibuja la tienda encima del menu; `vitrina` es la textura de la nave girando.
    pub fn dibujar(&self, d: &mut RaylibDrawHandle, e: &Entrada, p: &Progreso, vitrina: &Texture2D, ancho: i32, alto: i32) {
        let (w, h) = (ancho as f32, alto as f32);
        d.draw_rectangle(0, 0, ancho, alto, Color::new(6, 4, 22, 255));
        // El titulo, inclinado, con su franja (como los otros menus).
        ui::paralelogramo(d, -40.0, h * 0.13, w * 0.42, h * 0.072, 26.0, Color::new(18, 40, 120, 235));
        let tam_titulo = (h * 0.055).clamp(26.0, 52.0);
        ui::titulo_menu(d, "TIENDA", w * 0.04, h * 0.134, tam_titulo, -3.0);
        // Las notas que se tienen, arriba a la derecha.
        let tam = (h * 0.032).clamp(18.0, 30.0) as i32;
        let cartera = format!("{} notas", p.notas);
        let xc = ancho - ui::medir(&cartera, tam) - (w * 0.04) as i32;
        d.draw_circle(xc - tam / 2 - 6, (h * 0.15) as i32 + tam / 2, tam as f32 * 0.32, Color::new(255, 130, 210, 255));
        ui::texto(d, &cartera, xc, (h * 0.15) as i32, tam, Color::new(255, 230, 150, 255));

        // La lista va a la izquierda y la vitrina a la derecha; las pestanas caben sobre la lista.
        let x_lista = w * 0.05;
        let ancho_lista = (w * 0.40).min(560.0);
        let x_v = x_lista + ancho_lista + w * 0.04;

        // Las pestanas: NAVES, ESTELAS y PERSONAJES, la escogida rellena (mas chicas si no caben).
        const PESTANAS: [(Pestana, &str); 3] = [(Pestana::Naves, "NAVES"), (Pestana::Estelas, "ESTELAS"), (Pestana::Personajes, "PERSONAJES")];
        let y_pest = h * 0.22;
        let ancho_pestanas = |t: i32| PESTANAS.iter().map(|(_, n)| ui::medir(n, t) as f32 + t as f32 * 1.6 + 12.0).sum::<f32>();
        let mut tam_p = tam;
        while tam_p > 12 && ancho_pestanas(tam_p) > x_v - x_lista - 12.0 {
            tam_p -= 1;
        }
        let mut x = x_lista;
        for (pest, nombre) in PESTANAS {
            let activa = pest == self.pestana;
            let relleno = tam_p as f32 * 0.8;
            let ancho_p = ui::medir(nombre, tam_p) as f32 + relleno * 2.0;
            let color = if activa { Color::new(110, 235, 225, 255) } else { Color::new(60, 50, 100, 200) };
            ui::paralelogramo(d, x, y_pest, ancho_p, tam_p as f32 + 14.0, 10.0, color);
            ui::texto(d, nombre, (x + relleno) as i32, (y_pest + 7.0) as i32, tam_p, if activa { Color::new(10, 10, 30, 255) } else { ui::TENUE });
            x += ancho_p + 12.0;
        }

        let sub = if self.pestana == Pestana::Personajes { "Pilotos extra: los secretos sin esperar al logro, y Rocky" } else { "Exclusivos de tienda" };
        ui::texto(d, sub, (w * 0.05) as i32, (h * 0.29) as i32, 15, ui::TENUE);

        // La lista, a la izquierda: con aire entre filas; si no caben todas, se desliza con el
        // cursor (y unas flechitas avisan que hay mas arriba o abajo).
        let y_lista = h * 0.34;
        let tam_fila = (h * 0.032).clamp(16.0, 26.0) as i32;
        let filas = self.filas();
        let alto_lista = h - 84.0 - y_lista;
        let paso = (alto_lista / filas as f32).clamp(tam_fila as f32 * 1.8, tam_fila as f32 * 2.3);
        let visibles = ((alto_lista / paso) as usize).clamp(1, filas);
        let primera = self.cursor.saturating_sub(visibles / 2).min(filas - visibles);
        let flecha = |d: &mut RaylibDrawHandle, y: f32, arriba: bool| {
            let (cx, s) = (x_lista + ancho_lista / 2.0, 7.0);
            let (a, b, c) = if arriba {
                (Vector2::new(cx, y - s), Vector2::new(cx - s * 1.4, y + s), Vector2::new(cx + s * 1.4, y + s))
            } else {
                (Vector2::new(cx - s * 1.4, y - s), Vector2::new(cx, y + s), Vector2::new(cx + s * 1.4, y - s))
            };
            d.draw_triangle(a, b, c, Color::new(190, 180, 220, 200));
        };
        if primera > 0 {
            flecha(d, y_lista - paso * 0.45, true);
        }
        if primera + visibles < filas {
            flecha(d, y_lista + visibles as f32 * paso - paso * 0.2, false);
        }
        // Todas las filas: (nombre, precio, si se tiene, si esta puesta).
        let datos: Vec<(&str, u32, bool, bool)> = (0..filas)
            .map(|k| match self.pestana {
                Pestana::Naves => {
                    let s = &SKINS[skins_tienda()[k]];
                    (s.nombre, s.precio, p.tiene_skin(s.id), p.skin == s.id)
                }
                Pestana::Estelas if k == 0 => ("Sin estela", 0, true, p.estela.is_none()),
                Pestana::Estelas => {
                    let (id, nombre, precio) = ESTELAS[estelas_tienda()[k - 1]];
                    (nombre, precio, p.estelas.contains(id), p.estela.as_deref() == Some(id))
                }
                Pestana::Personajes => {
                    let f = &crate::personajes::PERSONAJES[personajes_tienda()[k]];
                    (f.nombre, crate::personajes::precio(f.id).unwrap_or(0), p.tiene_personaje(f.id), p.personaje == f.id)
                }
            })
            .collect();
        let valor_de = |precio: u32, tiene: bool, puesta: bool| if puesta { "PUESTA".to_string() } else if tiene { "tuya".to_string() } else { format!("{precio} notas") };
        // Un solo tamano para todos los nombres de la lista (el mas grande en que caben todos), y
        // los precios un poquito mas chicos, parejos con los nombres.
        let valor_tam = |tn: i32| (tn * 4 / 5).max(12);
        let mut tn = tam_fila;
        while tn > 12 && datos.iter().any(|&(nombre, precio, tiene, puesta)| ui::medir(nombre, tn) > ancho_lista as i32 - 56 - ui::medir(&valor_de(precio, tiene, puesta), valor_tam(tn))) {
            tn -= 1;
        }
        let tv = valor_tam(tn);
        for (j, k) in (primera..primera + visibles).enumerate() {
            let (nombre, precio, tiene, puesta) = datos[k];
            let y = y_lista + j as f32 * paso;
            let entra = ((self.abierta - j as f32 * 0.03) / 0.2).clamp(0.0, 1.0);
            let a = (entra * 255.0) as u8;
            let escogida = k == self.cursor;
            let xf = x_lista - (1.0 - entra) * 30.0;
            if escogida {
                // La banda, centrada en el texto de la fila (sin tapar las de junto).
                let largo = ancho_lista * (self.desde / 0.12).min(1.0);
                let alto_banda = (paso * 0.84).min(tam_fila as f32 * 1.8);
                ui::paralelogramo(d, xf - 10.0, y + tam_fila as f32 / 2.0 - alto_banda / 2.0, largo, alto_banda, 12.0, Color::new(255, 130, 210, 235));
            }
            let color = if escogida { Color::new(20, 8, 40, a) } else if tiene { Color::new(240, 232, 255, a) } else { Color::new(200, 190, 225, (a as f32 * 0.75) as u8) };
            // A la derecha: el precio, "tuya" o "puesta".
            let valor = valor_de(precio, tiene, puesta);
            let color_valor = if puesta {
                if escogida { Color::new(20, 8, 40, a) } else { ui::VERDE }
            } else if tiene {
                if escogida { Color::new(60, 20, 80, a) } else { ui::TENUE }
            } else if escogida {
                Color::new(60, 20, 80, a)
            } else if p.notas >= precio {
                Color::new(255, 230, 150, a)
            } else {
                Color::new(230, 120, 140, a)
            };
            // Tal cual (sin cambiar "Miku" por el protagonista: aqui Miku es Miku).
            ui::texto_crudo(d, nombre, (xf + 14.0) as i32, y as i32 + (tam_fila - tn) / 2, tn, color);
            ui::texto(d, &valor, (xf + ancho_lista - 24.0) as i32 - ui::medir(&valor, tv), y as i32 + (tam_fila - tv) / 2, tv, color_valor);
        }

        // La vitrina, a la derecha: la nave girando y, abajo, el nombre y lo que dice GLADAS.
        let ancho_v = (w - x_v - w * 0.04).max(200.0);
        let alto_v = (h * 0.34).min(ancho_v * 0.75);
        let y_v = h * 0.18;
        ui::panel(d, x_v as i32, y_v as i32, ancho_v as i32, (h * 0.70) as i32);
        let fuente = Rectangle::new(0.0, 0.0, vitrina.width as f32, vitrina.height as f32);
        let (lado_w, lado_h) = (ancho_v - 24.0, alto_v);
        let escala = (lado_w / vitrina.width as f32).min(lado_h / vitrina.height as f32);
        let (tw, th) = (vitrina.width as f32 * escala, vitrina.height as f32 * escala);
        d.draw_texture_pro(vitrina, fuente, Rectangle::new(x_v + (ancho_v - tw) / 2.0, y_v + 12.0, tw, th), Vector2::zero(), 0.0, Color::WHITE);
        let (titulo, texto) = match self.pestana {
            Pestana::Naves => (SKINS[skins_tienda()[self.cursor]].nombre, SKINS[skins_tienda()[self.cursor]].texto),
            Pestana::Estelas if self.cursor == 0 => ("Sin estela", "Nada detrás de la nave. Como cuando se va sin despedirse."),
            Pestana::Estelas => (ESTELAS[estelas_tienda()[self.cursor - 1]].1, texto_estela(ESTELAS[estelas_tienda()[self.cursor - 1]].0)),
            Pestana::Personajes => {
                let f = &crate::personajes::PERSONAJES[personajes_tienda()[self.cursor]];
                (f.nombre, f.texto)
            }
        };
        let y_t = (y_v + th + 26.0) as i32;
        let mut tam_t = (h * 0.04).clamp(20.0, 34.0) as i32;
        while tam_t > 14 && ui::medir(titulo, tam_t) > ancho_v as i32 - 48 {
            tam_t -= 1;
        }
        ui::texto_crudo(d, titulo, x_v as i32 + 24, y_t, tam_t, Color::new(110, 235, 225, 255));
        let tam_p = (h * 0.024).clamp(15.0, 21.0) as i32;
        let mut y_p = y_t + tam_t + 10;
        for linea in ui::envolver(texto, tam_p, ancho_v as i32 - 48) {
            ui::texto_crudo(d, &linea, x_v as i32 + 24, y_p, tam_p, ui::TEXTO);
            y_p += tam_p + tam_p / 3;
        }
        if let Some((aviso, t)) = &self.aviso {
            let a = (255.0 * (1.0 - ((t - 3.0) / 0.5).clamp(0.0, 1.0))) as u8;
            ui::parrafo(d, aviso, x_v as i32 + 24, y_p + 12, tam_p, ancho_v as i32 - 48, Color::new(255, 230, 150, a));
        }

        ui::ayudas(d, &[
            ("\u{E008} \u{E009}", "Escoger"),
            ("\u{E00A} \u{E00B}", "Categoría"),
            (e.nombre("aceptar"), "Comprar / equipar"),
            (e.nombre("pausa"), "Volver"),
        ], ancho, alto);
    }
}

/// Lo que dice GLADAS de cada estela.
pub fn texto_estela(id: &str) -> &'static str {
    match id {
        "miku" => "Puerros que salen girando detrás de la nave. Nadie sabe de dónde salen. Nadie pregunta.",
        "fuego" => "Llamas que suben y chispas que saltan. El chat puso 'cocina'. GLADAS entendió 'incendio' y sacó el extintor.",
        "arcoiris" => "Seis franjas paralelas y estrellitas, como el gato de internet. Le falta la tostada; la tostada es usted.",
        "notas" => "Corcheas rosas y cian que se van flotando. Un audio de siete minutos que nadie escuchó completo.",
        "corazones" => "Corazones que laten y suben. Uno por cada 'ya llegué' que mandó sin haber salido.",
        "aurora" => "Una cortina verde y violeta que cuelga de la nave y ondula. El aura dura más que la batería.",
        "ion" => "Un rayo en zigzag que cambia de forma doce veces por segundo. Pikachu lo aprueba. Su abogado, no.",
        "estelar" => "Estrellitas de cuatro picos que titilan alrededor. Dejó glitter en Mercurio y el Sol parece piñata.",
        "humo" => "Humo negro de camioneta extraurbana. La única estela que oscurece. ¡Hay lugar, hay lugar!",
        "burbujas" => "Burbujas que suben, crecen y revientan. En el espacio no deberían existir. Pero aquí estamos.",
        "pixel" => "Cuadritos de consola vieja, pegados a la cuadrícula. Sopla el cartucho antes de usarla.",
        "confeti" => "Papelitos de colores que caen meciéndose. Cada vuelo es un cumpleaños. Nadie limpia.",
        "adn" => "Una doble hélice que gira detrás de la nave. Genéticamente diseñada para presumir.",
        _ => "",
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn tienda_y_pase_no_comparten_articulos_y_todo_se_puede_previsualizar() {
        use crate::progreso::{Premio, PASE};
        let mut p = Progreso::en_memoria();
        p.skin = "oro".into(); // Un premio equipado no debe convertirse en indice de tienda.
        let mut t = Tienda::new(&p);
        for _ in 0..t.filas() {
            if let Muestra::Nave { skin, .. } = t.vitrina(&p) {
                assert!(!PASE.contains(&Premio::Skin(SKINS[skin].id)));
            } else { panic!("la tienda muestra naves"); }
            t.actualizar(&Entrada { abajo: true, ..Default::default() }, &mut p, 0.1);
        }
        p.estela = Some("fuego".into());
        t.actualizar(&Entrada { derecha: true, ..Default::default() }, &mut p, 0.1);
        for _ in 0..t.filas() {
            if let Muestra::Nave { estela: Some(id), .. } = t.vitrina(&p) {
                assert!(!PASE.contains(&Premio::Estela(id)));
            }
            t.actualizar(&Entrada { abajo: true, ..Default::default() }, &mut p, 0.1);
        }
        // Los pilotos: solo los extras (Koch, Rocky, Vegetta y Willy), ninguno del pase ni de fabrica.
        t.actualizar(&Entrada { derecha: true, ..Default::default() }, &mut p, 0.1);
        assert_eq!(t.pestana, Pestana::Personajes);
        let mut vistos = Vec::new();
        for _ in 0..t.filas() {
            let Muestra::Personaje(i) = t.vitrina(&p) else { panic!("la pestana de pilotos muestra pilotos") };
            let id = crate::personajes::PERSONAJES[i].id;
            assert!(!PASE.contains(&Premio::Personaje(id)) && !crate::personajes::GRATIS.contains(&id), "{id} no se vende");
            vistos.push(id);
            t.actualizar(&Entrada { abajo: true, ..Default::default() }, &mut p, 0.1);
        }
        vistos.sort();
        assert_eq!(vistos, ["koch", "rocky", "vegetta", "willyrex"]);
    }

    #[test]
    fn los_premios_no_se_compran_y_los_exclusivos_si() {
        let mut p = Progreso::en_memoria();
        // Sin encargos (salen al azar y, al comprar, alguno podria pagar notas).
        p.encargos.clear();
        p.notas = 1000;
        assert!(!p.skin_escogida("oro"));
        assert!(!p.estela_escogida("fuego"));
        assert_eq!(p.notas, 1000);
        assert!(p.skin_escogida("aurora"));
        assert!(p.estela_escogida("ion"));
        assert_eq!(p.skin, "aurora");
        assert_eq!(p.estela.as_deref(), Some("ion"));
        assert_eq!(p.notas, 1000 - 130 - 110);
        assert!(p.skin_escogida("aurora"));
        assert_eq!(p.notas, 760, "equipar algo comprado no vuelve a cobrar");
    }

    #[test]
    fn comprar_una_skin_cuesta_notas_y_se_queda_puesta() {
        let mut p = Progreso::en_memoria();
        p.encargos.clear();
        let mut t = Tienda::new(&p);
        assert_eq!(t.cursor, 0, "empieza en la de fabrica");
        // Sin notas no alcanza.
        t.actualizar(&Entrada { abajo: true, ..Default::default() }, &mut p, 0.1);
        t.actualizar(&Entrada { aceptar: true, ..Default::default() }, &mut p, 0.1);
        assert_eq!(p.skin, "lila");
        p.notas = 100;
        t.actualizar(&Entrada { aceptar: true, ..Default::default() }, &mut p, 0.1);
        assert_eq!(p.skin, SKINS[1].id);
        assert_eq!(p.notas, 100 - SKINS[1].precio);
        assert!(p.tiene("skin"));
        // Volver a la de fabrica no cuesta nada.
        t.actualizar(&Entrada { arriba: true, ..Default::default() }, &mut p, 0.1);
        t.actualizar(&Entrada { aceptar: true, ..Default::default() }, &mut p, 0.1);
        assert_eq!((p.skin.as_str(), p.notas), ("lila", 100 - SKINS[1].precio));
    }

    #[test]
    fn los_ids_de_las_skins_no_se_repiten() {
        let mut ids: Vec<&str> = SKINS.iter().map(|k| k.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), SKINS.len());
        assert_eq!(SKINS[0].precio, 0);
    }
}
