//! Spin-off de la Tierra: EL EXAMEN DE KOCH, en un salon de Teoria de la Computacion de la UVG.
//!
//! Al aterrizar en la Tierra se escoge a donde ir (`PuertaUvg`): el lab del CIT (la noche antes
//! de la entrega) o este salon. Aqui el protagonista se sienta en un pupitre y el profe Koch hace
//! cinco preguntas de automatas, sacadas al azar de un banco. Si la respuesta esta mal, Koch
//! camina hasta el pupitre, alza la regla y... reglazo: la camara tiembla, el protagonista queda
//! con los ojos en espiral y la pantalla se pone roja en los bordes. Con cada error Koch espera
//! mas cerca; con tres, examen reprobado por la via rapida. GLADAS comenta cada respuesta y al
//! final sale la nota (y se puede repetir, con otras preguntas).

use crate::entrada::Entrada;
use crate::matematica::{Mat4, Vec3};
use crate::sonido::Efecto;
use crate::ui;
use raylib::prelude::*;
use std::cell::Cell;

pub struct Pregunta {
    pub texto: &'static str,
    pub opciones: [&'static str; 4],
    pub correcta: usize,
    pub porque: &'static str,
}

const PREGUNTAS: [Pregunta; 20] = [
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
    Pregunta {
        texto: "Un AFD que acepta las cadenas binarias con una cantidad par de unos, ¿cuántos estados necesita como mínimo?",
        opciones: ["1", "2", "3", "4"],
        correcta: 1,
        porque: "Basta recordar si van par o impar de unos: dos estados, y el de «par» es el de aceptación.",
    },
    Pregunta {
        texto: "Una gramática libre de contexto es ambigua si...",
        opciones: ["Tiene más de un símbolo inicial", "Alguna cadena tiene dos árboles de derivación distintos", "Genera un lenguaje infinito", "Tiene producciones épsilon"],
        correcta: 1,
        porque: "Ambigua es que una misma cadena salga con dos árboles (o dos derivaciones por la izquierda) distintos.",
    },
    Pregunta {
        texto: "En la jerarquía de Chomsky, las gramáticas de tipo 3 generan...",
        opciones: ["Lenguajes regulares", "Lenguajes libres de contexto", "Lenguajes sensibles al contexto", "Lenguajes recursivamente enumerables"],
        correcta: 0,
        porque: "Tipo 3 son las regulares, tipo 2 las libres de contexto, tipo 1 las sensibles al contexto y tipo 0 sin restricción.",
    },
    Pregunta {
        texto: "¿Qué cadena NO pertenece al lenguaje de la expresión regular a*b*?",
        opciones: ["aaabb", "bbb", "La cadena vacía", "aba"],
        correcta: 3,
        porque: "a*b* es primero las a y después las b. En «aba» vuelve una a después de la b.",
    },
    Pregunta {
        texto: "Una máquina de Moore se diferencia de una de Mealy en que...",
        opciones: ["Su salida depende solo del estado actual", "No tiene estados", "Lee la cinta para los dos lados", "Acepta lenguajes no regulares"],
        correcta: 0,
        porque: "En Moore la salida va pegada al estado; en Mealy depende del estado y del símbolo que se lee.",
    },
];

/// Cuantas preguntas tiene cada examen.
const POR_CLASE: usize = 5;
/// Con cuantos errores el profe llega al pupitre (y se acaba el examen).
const ERRORES_MAXIMOS: usize = 3;
/// Donde espera el profe (junto al pizarron) y donde termina si se enoja
/// del todo (frente al pupitre del protagonista, que esta en x = 0, z = 0.8).
const PROFE_PIZARRON: Vec3 = Vec3::new(1.4, 0.0, -2.75);
const PROFE_PUPITRE: Vec3 = Vec3::new(0.70, 0.0, 0.15);
/// Segundos del reglazo: cuando pega la regla y cuando se acaba la escena.
const IMPACTO: f32 = 1.5;
const FIN_REGLAZO: f32 = 2.3;

const BIEN: [&str; 4] = [
    "Prueba de aula número {n}: correcta. Lo anoto como accidente estadístico.",
    "Correcto. El profe Koch casi sonríe. Casi. No se acostumbre.",
    "Respuesta aceptada. Igual que una cadena que termina en abb.",
    "Bien. Su promedio respira de nuevo. Con oxígeno prestado.",
];
const MAL: [&str; 4] = [
    "Prueba de aula número {n}: incorrecta. Como se esperaba. Hasta el autómata se rió.",
    "Error. Esta nave tiene más estados que ideas, y aun así le gana.",
    "Respuesta rechazada. Ni un AFN adivinando al azar falla tan bonito.",
    "Incorrecto. Eso pasa por estudiar con un video de 30 segundos a las 3 de la mañana.",
];

enum Etapa {
    Llegada,
    Pregunta,
    Reglazo { elegida: usize },
    Respuesta { elegida: usize },
    Final,
}

pub struct Aula {
    etapa: Etapa,
    orden: Vec<usize>,
    n: usize,
    pub puntaje: usize,
    errores: usize,
    cursor: usize,
    /// Donde va el profe ahora (camina de a poco hacia donde le toca).
    profe: Vec3,
    profe_inicio: Vec3,
    /// Segundos en la etapa actual.
    t: f32,
    /// Segundos desde que se entro al salon (para las animaciones).
    pub total: f32,
    ia: String,
    ojo: Vec3,
    mira: Vec3,
    /// Cuantos examenes se terminaron, y los mejores resultados (para los logros).
    pub terminados: u32,
    pub perfectos: u32,
    pub reprobados_a_reglazos: u32,
    /// Los sonidos que hay que tocar este cuadro.
    pub eventos: Vec<Efecto>,
}

// Donde se pone la camara: viendo el pizarron desde atras del protagonista, o a su cara para ver
// como reacciona.
const PIZARRON: (Vec3, Vec3) = (Vec3::new(0.9, 1.75, 2.7), Vec3::new(0.3, 1.35, -3.4));
// (La cara queda en el tercio de arriba: abajo va el panel de la pregunta.)
const CARA: (Vec3, Vec3) = (Vec3::new(-0.8, 1.55, -0.4), Vec3::new(0.05, 0.42, 0.8));
const ENTRADA: (Vec3, Vec3) = (Vec3::new(-0.6, 2.4, 3.3), Vec3::new(0.0, 1.4, -3.0));
const GOLPE: (Vec3, Vec3) = (Vec3::new(-1.8, 1.7, -1.0), Vec3::new(0.35, 1.15, 0.5));

impl Aula {
    pub fn new(semilla: u32) -> Self {
        Aula {
            etapa: Etapa::Llegada,
            orden: Self::barajar(semilla),
            n: 0,
            puntaje: 0,
            errores: 0,
            cursor: 0,
            profe: PROFE_PIZARRON,
            profe_inicio: PROFE_PIZARRON,
            t: 0.0,
            total: 0.0,
            ia: "Aterrizaje en la Tierra. Salón de Teoría de la Computación, UVG. Hay examen. El profe Koch trae regla y cero paciencia.".into(),
            ojo: ENTRADA.0,
            mira: ENTRADA.1,
            terminados: 0,
            perfectos: 0,
            reprobados_a_reglazos: 0,
            eventos: Vec::new(),
        }
    }

    /// Fisher-Yates con un generador congruencial: cinco preguntas distintas del banco.
    fn barajar(semilla: u32) -> Vec<usize> {
        let mut s = semilla | 1;
        let mut orden: Vec<usize> = (0..PREGUNTAS.len()).collect();
        for i in (1..orden.len()).rev() {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            orden.swap(i, (s >> 8) as usize % (i + 1));
        }
        orden.truncate(POR_CLASE);
        orden
    }

    /// Otra vez el examen, con otras preguntas (el profe ya se sabe la regla de memoria).
    pub fn repetir(&mut self) {
        let semilla = self.orden.iter().fold(self.puntaje as u32 * 31 + self.errores as u32 * 7 + 11, |h, &x| h.wrapping_mul(2_654_435_761).wrapping_add(x as u32 + 1));
        // Que no salgan las mismas cinco (si se puede).
        let mut orden = Self::barajar(semilla);
        let mut intentos = 0;
        while orden == self.orden && intentos < 8 {
            orden = Self::barajar(semilla.wrapping_add(intentos * 977 + 1));
            intentos += 1;
        }
        self.orden = orden;
        self.n = 0;
        self.puntaje = 0;
        self.errores = 0;
        self.cursor = 0;
        self.ia = "Examen de recuperación. El profe Koch afiló la regla. Mismas reglas: si la riega, reglazo.".into();
        self.cambiar(Etapa::Pregunta);
        self.eventos.push(Efecto::Acierto);
    }

    fn pregunta(&self) -> &'static Pregunta {
        &PREGUNTAS[self.orden[self.n.min(POR_CLASE - 1)]]
    }

    fn cambiar(&mut self, etapa: Etapa) {
        self.etapa = etapa;
        self.t = 0.0;
    }

    /// Avanza la clase. Devuelve `true` cuando hay que despegar.
    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> bool {
        let antes = self.t;
        self.t += dt;
        self.total += dt;
        // Durante el reglazo no se escapa nadie.
        if e.aterrizar && !matches!(self.etapa, Etapa::Reglazo { .. }) {
            return true;
        }
        match self.etapa {
            Etapa::Llegada => {
                if e.aceptar || e.clic {
                    self.ia = "El profe Koch va a preguntar. Se contesta con el número o con {arriba_abajo} y {aceptar}. Copiar no es opción: soy la única que sabe.".into();
                    self.cambiar(Etapa::Pregunta);
                    self.eventos.push(Efecto::Carta);
                }
            }
            Etapa::Pregunta => {
                if e.arriba {
                    self.cursor = (self.cursor + 3) % 4;
                    self.eventos.push(Efecto::Carta);
                }
                if e.abajo {
                    self.cursor = (self.cursor + 1) % 4;
                    self.eventos.push(Efecto::Carta);
                }
                let elegida = e.respuesta.filter(|&r| r < 4).or(if e.aceptar { Some(self.cursor) } else { None });
                if let Some(elegida) = elegida {
                    self.cursor = elegida;
                    let bien = elegida == self.pregunta().correcta;
                    let frases = if bien { &BIEN } else { &MAL };
                    self.puntaje += bien as usize;
                    self.errores += !bien as usize;
                    self.eventos.push(if bien { Efecto::Perfecto } else { Efecto::Fallo });
                    self.ia = frases[(self.n + self.orden[0]) % frases.len()].replace("{n}", &(self.n + 1).to_string());
                    if bien {
                        self.cambiar(Etapa::Respuesta { elegida });
                    } else {
                        self.profe_inicio = self.profe;
                        self.cambiar(Etapa::Reglazo { elegida });
                    }
                }
            }
            Etapa::Reglazo { elegida } => {
                if antes < IMPACTO && self.t >= IMPACTO {
                    self.eventos.push(Efecto::Reglazo);
                }
                if self.t >= FIN_REGLAZO {
                    self.cambiar(Etapa::Respuesta { elegida });
                }
            }
            Etapa::Respuesta { .. } => {
                if e.aceptar || e.clic {
                    self.n += 1;
                    self.cursor = 0;
                    if self.errores >= ERRORES_MAXIMOS {
                        self.ia = "Tres errores y tres reglazos. Examen reprobado por la vía rápida. La próxima vez estudiamos antes de aterrizar.".into();
                        self.terminar();
                    } else if self.n >= POR_CLASE {
                        self.ia = match self.puntaje {
                            5 => "Cinco de cinco. Con honores. Voy a revisar si hubo copia.",
                            3 | 4 => "Aprobó. Raspando, pero aprobó. Como se pasa en la U.",
                            _ => "Reprobó. Nos vemos en el examen de recuperación. Yo manejo, usted estudia.",
                        }
                        .into();
                        self.terminar();
                    } else {
                        self.cambiar(Etapa::Pregunta);
                    }
                }
            }
            // Al final: aceptar repite el examen (con otras preguntas); salir despega.
            Etapa::Final => {
                if (e.aceptar || e.clic) && self.t > 0.5 {
                    self.repetir();
                }
            }
        }
        // La camara va de a poco a donde le toca.
        let (ojo, mira) = match self.etapa {
            Etapa::Reglazo { .. } => GOLPE,
            Etapa::Respuesta { .. } | Etapa::Final => CARA,
            _ => PIZARRON,
        };
        let k = 1.0 - (-5.0 * dt).exp();
        self.ojo = self.ojo.lerp(ojo, k);
        self.mira = self.mira.lerp(mira, k);
        // El profe camina despacio hacia donde lo lleva su enojo.
        if matches!(self.etapa, Etapa::Reglazo { .. }) {
            self.profe = self.profe_inicio.lerp(PROFE_PUPITRE, (self.t / 0.9).clamp(0.0, 1.0));
        } else {
            let destino = PROFE_PIZARRON.lerp(PROFE_PUPITRE, self.errores as f32 / ERRORES_MAXIMOS as f32);
            self.profe = self.profe.lerp(destino, 1.0 - (-1.2 * dt).exp());
        }
        false
    }

    fn terminar(&mut self) {
        self.terminados += 1;
        self.perfectos += (self.puntaje == POR_CLASE) as u32;
        self.reprobados_a_reglazos += (self.errores >= ERRORES_MAXIMOS) as u32;
        self.eventos.push(if self.puntaje >= 3 { Efecto::Sello } else { Efecto::Fallo });
        self.cambiar(Etapa::Final);
    }

    /// Donde esta el profe y hacia donde mira (yaw): siempre hacia el pupitre.
    pub fn profe(&self) -> (Vec3, f32) {
        let al_pupitre = Vec3::new(0.0, 0.0, 0.8) - self.profe;
        (self.profe, f32::atan2(-al_pupitre.x, -al_pupitre.z))
    }

    /// Que tan roja se pone la pantalla, de 0 a 1.
    pub fn enojo(&self) -> f32 {
        self.errores as f32 / ERRORES_MAXIMOS as f32
    }

    /// La respuesta buena de la pregunta actual (para las capturas y las pruebas).
    pub fn correcta(&self) -> usize {
        self.pregunta().correcta
    }

    pub fn camara(&self) -> (Vec3, Vec3) {
        let golpe = self.impacto();
        let sacudida = Vec3::new((self.t * 85.0).sin(), (self.t * 67.0).cos(), 0.0) * (0.045 * golpe);
        (self.ojo + sacudida, self.mira + sacudida * 0.4)
    }

    /// Que tan fuerte se siente el golpe ahorita (1 justo al pegar, y se apaga).
    pub fn impacto(&self) -> f32 {
        if matches!(self.etapa, Etapa::Reglazo { .. }) && self.t >= IMPACTO {
            (1.0 - (self.t - IMPACTO) / 0.55).max(0.0)
        } else {
            0.0
        }
    }

    /// El brazo y la regla comparten el pivote del codo: prepara, golpea y baja.
    pub fn brazo_regla(&self) -> Mat4 {
        let angulo = if matches!(self.etapa, Etapa::Reglazo { .. }) {
            if self.t < 1.3 {
                1.4 * ((self.t - 0.9) / 0.4).clamp(0.0, 1.0)
            } else if self.t < IMPACTO {
                1.4 - 1.42 * ((self.t - 1.3) / (IMPACTO - 1.3)).powi(2)
            } else {
                -0.02 * (1.0 - (self.t - IMPACTO) / (FIN_REGLAZO - IMPACTO)).max(0.0)
            }
        } else {
            0.0
        };
        let pivote = Vec3::new(0.2625, 1.175, 0.0);
        Mat4::translation(pivote) * Mat4::rotation_x(angulo) * Mat4::translation(-pivote)
    }

    /// El protagonista se encoge con el golpe.
    pub fn reaccion(&self) -> Mat4 {
        let golpe = self.impacto();
        Mat4::translation(Vec3::new(-0.08 * golpe, -0.05 * golpe, 0.04 * golpe)) * Mat4::rotation_z(0.16 * golpe)
    }

    /// Como termino el examen, si ya termino: el puntaje y si fue por reglazos.
    pub fn resultado(&self) -> Option<(usize, bool)> {
        matches!(self.etapa, Etapa::Final).then_some((self.puntaje, self.errores >= ERRORES_MAXIMOS))
    }

    /// Lo que dice GLADAS al despegar.
    pub fn despedida(&self) -> String {
        if self.terminados == 0 {
            return "Despegue de la UVG. Nos fuimos antes del examen. Técnicamente eso es un cero. Técnicamente.".into();
        }
        format!("Despegue de la UVG. Nota del examen: {}/{}. {}", self.puntaje, POR_CLASE, self.ia)
    }

    pub fn expresion_protagonista(&self) -> &'static str {
        match self.etapa {
            Etapa::Llegada => "feliz",
            // Si tarda mucho en contestar, se duerme.
            Etapa::Pregunta if self.t > 15.0 => "dormida",
            Etapa::Pregunta => "feliz",
            Etapa::Reglazo { .. } if self.impacto() > 0.0 => "espiral",
            Etapa::Reglazo { .. } => "asustada",
            Etapa::Respuesta { elegida } if elegida == self.pregunta().correcta => "enamorada",
            Etapa::Respuesta { .. } => "asustada",
            Etapa::Final if self.puntaje >= 3 => "enamorada",
            Etapa::Final => "serio",
        }
    }

    pub fn expresion_profe(&self) -> &'static str {
        match self.etapa {
            Etapa::Reglazo { .. } => "enojado",
            Etapa::Respuesta { elegida } if elegida != self.pregunta().correcta => "enojado",
            Etapa::Final if self.puntaje >= 3 => "feliz",
            Etapa::Final => "enojado",
            _ if self.errores > 0 => "serio",
            _ => "feliz",
        }
    }

    /// Lo que dice GLADAS ahora.
    pub fn ia(&self) -> &str {
        &self.ia
    }

    /// Lo que dice el protagonista despues del reglazo (a su manera).
    pub fn replica_golpe(&self) -> &'static str {
        if (matches!(self.etapa, Etapa::Reglazo { .. }) && self.t >= IMPACTO) || matches!(self.etapa, Etapa::Respuesta { elegida } if elegida != self.pregunta().correcta) {
            crate::personajes::replica(ui::protagonista().id, crate::personajes::Replica::Golpe)
        } else {
            ""
        }
    }

    /// Lo que pregunta el profe ahora (vacio si no esta preguntando).
    pub fn texto_profe(&self) -> &'static str {
        match self.etapa {
            Etapa::Pregunta | Etapa::Respuesta { .. } => self.pregunta().texto,
            _ => "",
        }
    }

    /// Para `--captura` y `--foto-ui`: el examen en un momento (`aula`: la llegada; `aula_pregunta`;
    /// `aula_bien`; `aula_mal`: a media regla; `aula_reglazo`: el golpe; `aula_final`).
    pub fn preparar_captura(&mut self, modo: &str) {
        let aceptar = Entrada { aceptar: true, ..Default::default() };
        if modo == "aula" {
            return;
        }
        self.actualizar(&aceptar, 0.0);
        let mal = Entrada { respuesta: Some((self.correcta() + 1) % 4), ..Default::default() };
        match modo {
            "aula_bien" => {
                self.actualizar(&Entrada { respuesta: Some(self.correcta()), ..Default::default() }, 0.0);
            }
            "aula_mal" => {
                self.actualizar(&mal, 0.0);
                self.actualizar(&Entrada::default(), 1.2);
            }
            "aula_reglazo" => {
                self.actualizar(&mal, 0.0);
                self.actualizar(&Entrada::default(), IMPACTO + 0.05);
            }
            "aula_final" => {
                while self.resultado().is_none() {
                    let r = if self.n.is_multiple_of(2) { self.correcta() } else { (self.correcta() + 1) % 4 };
                    self.actualizar(&Entrada { respuesta: Some(r), ..Default::default() }, 0.0);
                    self.actualizar(&Entrada::default(), FIN_REGLAZO);
                    self.actualizar(&aceptar, 0.0);
                }
            }
            _ => {}
        }
        // La camara ya en su lugar.
        for _ in 0..90 {
            self.actualizar(&Entrada::default(), 1.0 / 60.0);
            if matches!(self.etapa, Etapa::Reglazo { .. }) {
                break;
            }
        }
        self.eventos.clear();
    }

    /// Las ayudas de abajo, segun la etapa.
    fn ayudas(&self, e: &Entrada) -> Vec<(&'static str, &'static str)> {
        let (aceptar, salir) = (e.nombre("aceptar"), e.nombre("aterrizar"));
        match self.etapa {
            Etapa::Llegada => vec![(aceptar, "Empezar el examen"), (salir, "Despegar")],
            Etapa::Pregunta if e.hay_control => vec![("\u{E008} \u{E009}", "Escoger"), (aceptar, "Contestar"), (salir, "Despegar")],
            Etapa::Pregunta => vec![("\u{E008} \u{E009}", "Escoger"), ("1 a 4", "Contestar directo"), (aceptar, "Contestar"), (salir, "Despegar")],
            Etapa::Reglazo { .. } => Vec::new(),
            Etapa::Respuesta { .. } => vec![(aceptar, "Siguiente pregunta"), (salir, "Despegar")],
            Etapa::Final => vec![(aceptar, "Repetir el examen"), (salir, "Despegar")],
        }
    }

    /// `ia`, `profe` y `piloto` son lo que ya se ve de cada texto (aparecen letra por letra).
    /// Todo va en un panel abajo (encima de las ayudas); la caja de GLADAS de arriba solo sale
    /// si cabe sin encimarse (en una ventana chiquita, el panel manda).
    #[allow(clippy::too_many_arguments)]
    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, ia: &str, profe: &str, piloto: &str, ancho: i32, alto: i32) {
        let acciones = self.ayudas(e);
        let pie = ui::alto_ayudas(&acciones, ancho);
        let angosta = ancho < 900 || alto < 560;
        let px = (ancho / 30).clamp(16, 40);
        let pw = ancho - px * 2;
        let interior = pw - 48;
        let fondo = alto - pie - 12;
        let nombre = ui::protagonista().nombre;
        // Lo alto del panel de abajo, con los textos completos (aunque todavia se esten escribiendo).
        let (tam_p, tam_o, tam_q) = if angosta { (18, 16, 15) } else { (24, 20, 17) };
        let (paso_p, paso_q) = (tam_p + tam_p / 3, tam_q + tam_q / 3);
        let alto_opcion = tam_o + if angosta { 8 } else { 14 };
        let lineas = |texto: &str, tam: i32| ui::envolver(texto, tam, interior).len() as i32;
        let comentario = format!("{}: {}", ui::NOMBRE_IA, self.ia);
        let replica = self.replica_golpe();
        let h = match self.etapa {
            Etapa::Llegada => 22 + if angosta { 24 } else { 32 } + 14 + 24 + 14,
            Etapa::Reglazo { .. } => 0,
            Etapa::Pregunta | Etapa::Respuesta { .. } => {
                let p = self.pregunta();
                let mut h = 40 + lineas(p.texto, tam_p).max(1) * paso_p + 8 + 4 * alto_opcion + 12;
                if matches!(self.etapa, Etapa::Respuesta { .. }) {
                    h += lineas(p.porque, tam_q) * paso_q + 6 + lineas(&comentario, tam_q) * paso_q + 4;
                    if !replica.is_empty() {
                        h += lineas(&format!("{nombre}: {replica}"), tam_q) * paso_q + 4;
                    }
                }
                h
            }
            Etapa::Final => 22 + if angosta { 26 } else { 34 } + 16 + 30 + 16 + 28 + 18,
        };
        let y0 = fondo - h;
        // Arriba a la izquierda: GLADAS (en la llegada y al final), si cabe.
        if matches!(self.etapa, Etapa::Llegada | Etapa::Final) && !ia.is_empty() {
            let w = (ancho - 40).min(860);
            if 20 + ui::alto_dialogo(ui::NOMBRE_IA, &self.ia, w) <= y0 - 10 {
                ui::caja_dialogo(d, ui::NOMBRE_IA, ui::LILA, ia, 20, 20, w);
            }
        }
        match self.etapa {
            Etapa::Llegada => {
                let tam_t = if angosta { 24 } else { 32 };
                ui::panel(d, px, y0, pw, h);
                ui::titulo_de(d, "SALÓN DE TEORÍA", px + 24, y0 + 22, tam_t, Color::new(110, 235, 225, 255));
                let sub = "UVG · spin-off: el examen del profe Koch. Si la riega, reglazo.";
                ui::texto(d, &ui::recortado(sub, 18, interior), px + 24, y0 + 22 + tam_t + 14, 18, ui::TENUE);
            }
            Etapa::Reglazo { .. } => {
                let caja = if piloto.is_empty() { 0 } else { ui::alto_dialogo(nombre, replica, pw) };
                if self.t < IMPACTO {
                    let s = "El profe Koch prepara la regla...";
                    ui::subtitulo(d, s, (ancho - ui::medir(s, 26)) / 2, fondo - caja - 60, 26, ui::ROJO);
                } else {
                    let tam = if angosta { 48 } else { 72 };
                    let s = "¡REGLAZO!";
                    let temblor = (self.impacto() * 8.0 * (self.t * 60.0).sin()) as i32;
                    ui::titulo_de(d, s, (ancho - ui::medir_titulo(s, tam)) / 2 + temblor, (alto as f32 * 0.28) as i32, tam, Color::new(255, 80, 95, 255));
                }
                if !piloto.is_empty() {
                    ui::caja_dialogo(d, nombre, ui::LILA, piloto, px, fondo - caja, pw);
                }
            }
            Etapa::Pregunta | Etapa::Respuesta { .. } => {
                let p = self.pregunta();
                let elegida = if let Etapa::Respuesta { elegida } = self.etapa { Some(elegida) } else { None };
                ui::panel(d, px, y0, pw, h);
                // La cabecera: quien pregunta, en que pregunta va, la nota y los reglazos que van.
                ui::texto(d, "EL PROFE KOCH PREGUNTA", px + 24, y0 + 14, 14, ui::LILA);
                let reglas_x = px + pw - 24 - ERRORES_MAXIMOS as i32 * 14;
                for k in 0..ERRORES_MAXIMOS {
                    regla(d, reglas_x + k as i32 * 14, y0 + 10, 22, k < self.errores);
                }
                let marcador = format!("PREGUNTA {}/{}   NOTA {}/{}", (self.n + 1).min(POR_CLASE), POR_CLASE, self.puntaje, POR_CLASE);
                ui::texto(d, &marcador, reglas_x - 16 - ui::medir(&marcador, 14), y0 + 14, 14, ui::TENUE);
                ui::parrafo(d, profe, px + 24, y0 + 38, tam_p, interior, ui::TEXTO);
                let y_opciones = y0 + 40 + lineas(p.texto, tam_p).max(1) * paso_p + 8;
                for (i, op) in p.opciones.iter().enumerate() {
                    let y = y_opciones + i as i32 * alto_opcion;
                    let escogida = elegida.is_none() && i == self.cursor;
                    if escogida {
                        ui::paralelogramo(d, (px + 14) as f32, (y - 4) as f32, (pw - 28) as f32, (tam_o + 8) as f32, 10.0, Color::new(255, 130, 210, 90));
                    }
                    let color = match elegida {
                        Some(_) if i == p.correcta => ui::VERDE,
                        Some(el) if i == el => ui::ROJO,
                        Some(_) => ui::TENUE,
                        None if escogida => Color::WHITE,
                        None => ui::TEXTO,
                    };
                    // Con teclado, el numero es su tecla; con control, se escoge con la cruceta.
                    let marca = if e.hay_control { format!("{})", i + 1) } else { ui::boton(&(i + 1).to_string()) };
                    ui::texto(d, &format!("{marca} {op}"), px + 28, y, tam_o, color);
                    if elegida.is_some() && (i == p.correcta || elegida == Some(i)) {
                        ui::palomita(d, (px + pw - 52) as f32, (y + 1) as f32, tam_o as f32, i == p.correcta);
                    }
                }
                if elegida.is_some() {
                    let mut y = ui::parrafo(d, p.porque, px + 24, y_opciones + 4 * alto_opcion + 6, tam_q, interior, ui::TENUE) + 4;
                    y = ui::parrafo(d, &format!("{}: {ia}", ui::NOMBRE_IA), px + 24, y, tam_q, interior, ui::LILA) + 4;
                    if !piloto.is_empty() {
                        ui::parrafo(d, &format!("{nombre}: {piloto}"), px + 24, y, tam_q, interior, Color::new(255, 200, 230, 255));
                    }
                }
            }
            Etapa::Final => {
                let tam_t = if angosta { 26 } else { 34 };
                ui::panel(d, px, y0, pw, h);
                ui::titulo_de(d, &format!("NOTA FINAL {}/{}", self.puntaje, POR_CLASE), px + 24, y0 + 22, tam_t, Color::WHITE);
                ui::estrellas(d, (px + 24) as f32, (y0 + 22 + tam_t + 16) as f32, self.puntaje as u32, POR_CLASE as u32, 28.0);
                let (veredicto, color) = match (self.puntaje >= 3, self.errores >= ERRORES_MAXIMOS) {
                    (_, true) => ("REPROBÓ POR LA VÍA RÁPIDA (tres reglazos)", ui::ROJO),
                    (true, _) if self.puntaje == POR_CLASE => ("¡APROBÓ CON CIEN! Koch está confundido", ui::VERDE),
                    (true, _) => ("APROBÓ", ui::VERDE),
                    (false, _) => ("REPROBÓ", ui::ROJO),
                };
                let tam_v = if angosta { 18 } else { 24 };
                ui::texto(d, &ui::recortado(veredicto, tam_v, interior), px + 24, y0 + 22 + tam_t + 16 + 30 + 16, tam_v, color);
            }
        }
        ui::ayudas(d, &acciones, ancho, alto);
    }
}

/// Una reglita de 30 cm (de `alto` pixeles): roja si ya cayo ese reglazo.
fn regla(d: &mut RaylibDrawHandle, x: i32, y: i32, alto: i32, usada: bool) {
    let (cuerpo, raya) = if usada { (Color::new(240, 95, 110, 255), Color::new(120, 24, 44, 255)) } else { (Color::new(84, 72, 120, 255), Color::new(48, 40, 74, 255)) };
    let ancho = (alto / 4).max(7);
    d.draw_rectangle(x + 2, y + 2, ancho, alto, Color::new(0, 0, 0, 90));
    d.draw_rectangle(x, y, ancho, alto, cuerpo);
    for k in (4..alto - 2).step_by(5) {
        let largo = if k % 10 == 4 { ancho * 2 / 3 } else { ancho / 3 };
        d.draw_rectangle(x, y + k, largo, 1, raya);
    }
}

// ---------------------------------------------------------------------------
// La entrada de la UVG: a donde se va al aterrizar en la Tierra
// ---------------------------------------------------------------------------

/// Lo que se escoge en la entrada de la UVG.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DestinoUvg {
    /// El lab del CIT: 11:59, la noche antes de la entrega.
    Lab,
    /// El salon de Teoria: el examen de Koch (el spin-off).
    Salon,
    /// Mejor no: la nave despega.
    Despegar,
}

/// Al aterrizar en la Tierra: dos tarjetas encima de la nave posada, el lab o el salon.
pub struct PuertaUvg {
    pub cursor: usize,
    t: f32,
    /// Donde quedaron las tarjetas al dibujarse (para el raton; se usan el cuadro siguiente).
    tarjetas: Cell<[(f32, f32, f32, f32); 2]>,
    pub eventos: Vec<Efecto>,
}

impl Default for PuertaUvg {
    fn default() -> Self {
        Self::new()
    }
}

impl PuertaUvg {
    pub fn new() -> Self {
        PuertaUvg { cursor: 0, t: 0.0, tarjetas: Cell::new([(0.0, 0.0, 0.0, 0.0); 2]), eventos: Vec::new() }
    }

    fn destino(k: usize) -> DestinoUvg {
        if k == 0 { DestinoUvg::Lab } else { DestinoUvg::Salon }
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> Option<DestinoUvg> {
        self.t += dt;
        let antes = self.cursor;
        if e.izquierda || e.arriba {
            self.cursor = 0;
        }
        if e.derecha || e.abajo {
            self.cursor = 1;
        }
        // El raton: pasar encima escoge, y el clic entra.
        if let Some((mx, my)) = e.puntero {
            if let Some(k) = self.tarjetas.get().iter().position(|&(x, y, w, h)| w > 0.0 && mx >= x && mx <= x + w && my >= y && my <= y + h) {
                self.cursor = k;
                if e.clic && self.t > 0.3 {
                    self.eventos.push(Efecto::Acierto);
                    return Some(Self::destino(k));
                }
            }
        }
        if self.cursor != antes {
            self.eventos.push(Efecto::Carta);
        }
        if let Some(k) = e.respuesta.filter(|&r| r < 2) {
            self.cursor = k;
            self.eventos.push(Efecto::Acierto);
            return Some(Self::destino(k));
        }
        if e.aceptar && self.t > 0.3 {
            self.eventos.push(Efecto::Acierto);
            return Some(Self::destino(self.cursor));
        }
        if e.aterrizar {
            return Some(DestinoUvg::Despegar);
        }
        None
    }

    /// Donde va cada tarjeta: lado a lado, o una encima de otra si la ventana es angosta.
    fn acomodo(ancho: i32, alto: i32, pie: i32) -> [(f32, f32, f32, f32); 2] {
        let (w, h) = (ancho as f32, alto as f32);
        let margen = (w * 0.06).max(18.0);
        let arriba = h * 0.30;
        let abajo = h - pie as f32 - 24.0;
        if w >= 760.0 {
            let hueco = 28.0;
            let ancho_t = ((w - margen * 2.0 - hueco) / 2.0).min(460.0);
            let x0 = (w - ancho_t * 2.0 - hueco) / 2.0;
            let alto_t = (abajo - arriba).clamp(150.0, 230.0);
            [(x0, arriba, ancho_t, alto_t), (x0 + ancho_t + hueco, arriba, ancho_t, alto_t)]
        } else {
            let hueco = 14.0;
            let alto_t = ((abajo - arriba - hueco) / 2.0).clamp(110.0, 200.0);
            [(margen, arriba, w - margen * 2.0, alto_t), (margen, arriba + alto_t + hueco, w - margen * 2.0, alto_t)]
        }
    }

    pub fn dibujar(&self, d: &mut RaylibDrawHandle, e: &Entrada, ancho: i32, alto: i32) {
        let acciones = [("\u{E00A} \u{E00B}", "Escoger"), (e.nombre("aceptar"), "Entrar"), (e.nombre("aterrizar"), "Despegar")];
        let pie = ui::alto_ayudas(&acciones, ancho);
        let entra = (self.t / 0.35).clamp(0.0, 1.0);
        d.draw_rectangle(0, 0, ancho, alto, Color::new(6, 4, 22, (175.0 * entra) as u8));
        // El titulo, centrado.
        let mut tam = ((alto as f32 * 0.06) as i32).clamp(24, 48);
        let titulo = "¿A DÓNDE VAMOS?";
        while tam > 18 && ui::medir_titulo(titulo, tam) > ancho - 40 {
            tam -= 1;
        }
        let y_titulo = (alto as f32 * 0.11) as i32;
        ui::titulo_de(d, titulo, (ancho - ui::medir_titulo(titulo, tam)) / 2, y_titulo, tam, Color::new(110, 235, 225, 255));
        let sub = "Universidad del Valle de Guatemala · la nave quedó en el parqueo";
        let tam_sub = if ancho < 760 { 15 } else { 18 };
        ui::subtitulo(d, sub, (ancho - ui::medir(sub, tam_sub)) / 2, y_titulo + tam + 14, tam_sub, ui::TENUE);

        let tarjetas = Self::acomodo(ancho, alto, pie);
        self.tarjetas.set(tarjetas);
        const CONTENIDO: [(&str, &str, &str, &str, Color); 2] = [
            ("LA AVENTURA", "LAB DEL CIT", "11:59 · la noche antes de la entrega", "Tres proyectos de Koch y una hora: Gráficas, Teoría y Software 2. Café, WhatsApp y el guardia.", Color::new(255, 200, 120, 255)),
            ("SPIN-OFF", "SALÓN DE TEORÍA", "El examen del profe Koch", "Cinco preguntas de autómatas. Si la riega, Koch viene con la regla. Tres reglazos y reprueba.", Color::new(255, 120, 200, 255)),
        ];
        for (k, (&(x, y, w, h), &(etiqueta, nombre, lema, texto, color))) in tarjetas.iter().zip(CONTENIDO.iter()).enumerate() {
            let escogida = k == self.cursor;
            let y = y + if escogida { -6.0 } else { 0.0 } + (1.0 - entra) * 30.0;
            ui::panel(d, x as i32, y as i32, w as i32, h as i32);
            let (xi, wi) = (x as i32 + 22, w as i32 - 44);
            ui::texto(d, etiqueta, xi, y as i32 + 18, 14, color);
            let mut tam_n = if ancho < 760 { 22 } else { 28 };
            while tam_n > 14 && ui::medir_titulo(nombre, tam_n) > wi {
                tam_n -= 1;
            }
            ui::titulo_de(d, nombre, xi, y as i32 + 40, tam_n, if escogida { Color::WHITE } else { Color::new(200, 190, 230, 255) });
            let y_lema = y as i32 + 40 + tam_n + 12;
            ui::texto(d, lema, xi, y_lema, if ancho < 760 { 15 } else { 18 }, color);
            // La descripcion, si cabe.
            let tam_d = if ancho < 760 { 14 } else { 16 };
            let mut yd = y_lema + 30;
            for linea in ui::envolver(texto, tam_d, wi) {
                if yd + tam_d > (y + h) as i32 - 12 {
                    break;
                }
                ui::texto(d, &linea, xi, yd, tam_d, ui::TEXTO);
                yd += tam_d + tam_d / 3;
            }
            if escogida {
                ui::seleccion(d, x - 6.0, y - 6.0, w + 12.0, h + 12.0, self.t, Color::WHITE);
            } else {
                d.draw_rectangle(x as i32, y as i32, w as i32, h as i32, Color::new(6, 4, 22, 90));
            }
        }
        ui::ayudas(d, &acciones, ancho, alto);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn empezar() -> Aula {
        let mut a = Aula::new(7);
        a.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.0);
        a
    }

    #[test]
    fn el_banco_de_preguntas_esta_bien_armado() {
        let mut textos: Vec<&str> = PREGUNTAS.iter().map(|p| p.texto).collect();
        textos.sort();
        textos.dedup();
        assert_eq!(textos.len(), PREGUNTAS.len(), "hay preguntas repetidas");
        assert!(PREGUNTAS.iter().all(|p| p.correcta < 4 && !p.porque.is_empty()));
        assert!(POR_CLASE <= PREGUNTAS.len());
        // Cinco distintas en cada examen.
        let mut a = Aula::new(12345);
        let mut orden = a.orden.clone();
        orden.dedup();
        assert_eq!(orden.len(), POR_CLASE);
        a.repetir();
        assert_eq!(a.orden.len(), POR_CLASE);
    }

    /// Al terminar se puede repetir el examen (aceptar no despega).
    #[test]
    fn se_puede_repetir_el_examen() {
        let mut a = empezar();
        while a.resultado().is_none() {
            a.actualizar(&Entrada { respuesta: Some(a.correcta()), ..Default::default() }, 0.0);
            a.actualizar(&Entrada::default(), 2.0);
            a.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.0);
        }
        assert_eq!((a.terminados, a.perfectos), (1, 1));
        a.actualizar(&Entrada::default(), 1.0);
        assert!(!a.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.1), "aceptar no despega");
        assert!(a.resultado().is_none() && a.n == 0 && a.errores == 0 && a.puntaje == 0);
        assert!(matches!(a.etapa, Etapa::Pregunta));
    }

    #[test]
    fn cada_error_da_un_solo_reglazo_antes_de_seguir() {
        let mut a = empezar();
        for error in 1..=3 {
            a.actualizar(&Entrada { respuesta: Some((a.correcta() + 1) % 4), ..Default::default() }, 0.0);
            assert_eq!(a.errores, error);
            assert!(matches!(a.etapa, Etapa::Reglazo { .. }));
            // Aceptar, salir o contestar otra vez no saltan el golpe ni duplican el error.
            assert!(!a.actualizar(&Entrada { aceptar: true, aterrizar: true, respuesta: Some(0), ..Default::default() }, 0.1));
            assert_eq!(a.errores, error);
            a.actualizar(&Entrada::default(), IMPACTO - 0.09);
            assert!(a.impacto() > 0.9);
            assert!((a.profe - PROFE_PUPITRE).length() < 0.001);
            assert_eq!(a.eventos.iter().filter(|e| matches!(e, Efecto::Reglazo)).count(), 1);
            a.actualizar(&Entrada::default(), 1.0);
            assert!(matches!(a.etapa, Etapa::Respuesta { .. }));
            assert_eq!(a.eventos.iter().filter(|e| matches!(e, Efecto::Reglazo)).count(), 1);
            a.eventos.clear();
            a.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.0);
        }
        assert_eq!(a.resultado(), Some((0, true)));
        assert_eq!(a.reprobados_a_reglazos, 1);
    }

    #[test]
    fn una_partida_perfecta_no_da_reglazos() {
        let mut a = empezar();
        for _ in 0..POR_CLASE {
            a.actualizar(&Entrada { respuesta: Some(a.correcta()), ..Default::default() }, 0.0);
            assert!(matches!(a.etapa, Etapa::Respuesta { .. }));
            a.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.0);
        }
        assert_eq!(a.resultado(), Some((5, false)));
        assert!(!a.eventos.iter().any(|e| matches!(e, Efecto::Reglazo)));
    }

    #[test]
    fn un_cuadro_largo_no_pierde_el_sonido_del_golpe() {
        let mut a = empezar();
        a.actualizar(&Entrada { respuesta: Some((a.correcta() + 1) % 4), ..Default::default() }, 0.0);
        a.actualizar(&Entrada::default(), 3.0);
        assert!(matches!(a.etapa, Etapa::Respuesta { .. }));
        assert_eq!(a.eventos.iter().filter(|e| matches!(e, Efecto::Reglazo)).count(), 1);
    }

    /// En la entrada de la UVG: el lab por defecto, el salon a la derecha, o despegar.
    #[test]
    fn en_la_uvg_se_escoge_el_lab_o_el_salon() {
        let mut p = PuertaUvg::new();
        assert_eq!(p.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.0), None, "un aceptar al llegar no entra solo");
        assert_eq!(p.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.5), Some(DestinoUvg::Lab));
        let mut p = PuertaUvg::new();
        p.actualizar(&Entrada { derecha: true, ..Default::default() }, 0.5);
        assert_eq!(p.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.1), Some(DestinoUvg::Salon));
        let mut p = PuertaUvg::new();
        assert_eq!(p.actualizar(&Entrada { respuesta: Some(1), ..Default::default() }, 0.0), Some(DestinoUvg::Salon));
        assert_eq!(PuertaUvg::new().actualizar(&Entrada { aterrizar: true, ..Default::default() }, 0.0), Some(DestinoUvg::Despegar));
    }
}
