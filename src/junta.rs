//! Aterrizaje en Pluton: la junta de emergencia.
//!
//! Pluton aprieta el boton de emergencia y reune al consejo de planetas
//! alrededor de la mesa (homenaje a los juegos de impostores): cada planeta
//! es un tripulante frijolito con visor, con el traje de su color y su
//! planetita flotando arriba de la cabeza. Da un discurso dramatico, cada planeta
//! opina, y se vota si vuelve a ser planeta. Los votos del consejo quedan
//! empatados, asi que decide la piloto.
//!
//! Lo que luce del pipeline: Pluton se dibuja en ALAMBRE (primitive
//! assembly en modo lineas) porque "ya no lo rellenan". Si gana el si,
//! vuelve a dibujarse relleno.
//!
//! Todo esta en las coordenadas de Pluton: radio 1 en el origen.

use crate::entrada::Entrada;
use crate::escena::CUERPOS;
use crate::matematica::{Mat4, Vec3};
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Voto {
    Si,
    No,
    Saltar,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fase {
    Alarma,
    Discurso(usize),
    Debate(usize),
    Votacion,
    Conteo,
    Resultado,
}

const DISCURSO: [&str; 5] = [
    "Hermanos planetas... y Urano.",
    "En 2006 me quitaron el título. Ni una llamada. Ni un mensaje. Me enteré por las noticias, como todos.",
    "Tengo cinco lunas. ¡Cinco! Mercurio no tiene ni una y nadie le hace junta.",
    "Tengo un corazón en la cara. Literal. ¿Qué más quieren de mí?",
    "Solo pido una cosa: que me vuelvan a rellenar. Desde 2006 me dibujan solo con rayitas.",
];

/// El consejo: quien es (indice en `CUERPOS`), que dice, como vota y su escala aqui.
const CONSEJO: [(usize, &str, Voto, f32); 8] = [
    (1, "Yo no tengo lunas y no ando llorando. Bueno, sí, pero en privado.", Voto::No, 0.42),
    (2, "Yo giro al revés y nadie me hizo junta. Voto... ¿al revés?", Voto::Si, 0.55),
    (3, "Yo tengo examen de autómatas. No tengo tiempo para esto.", Voto::Saltar, 0.58),
    (5, "Mi rover dice que sí. Mi rover dice que sí a todo, está solito.", Voto::Si, 0.5),
    (6, "Yo soy mil trescientas Tierras. Plutón cabe en mi mancha roja. Con todo respeto.", Voto::No, 0.95),
    (7, "Si Plutón vuelve, ¿le presto el escenario? Ya tengo el ecualizador listo.", Voto::Si, 0.36),
    (8, "No entiendo qué está pasando. Nunca entiendo qué está pasando.", Voto::Saltar, 0.58),
    (9, "Plutón se mete en mi órbita sin permiso hace millones de años. Solo digo.", Voto::No, 0.56),
];

const OPCIONES: [(Voto, &str); 3] = [(Voto::Si, "SÍ es planeta"), (Voto::No, "NO es planeta"), (Voto::Saltar, "Saltar voto")];
/// Centro de la mesa de la junta, en el piso de la sala (Pluton queda sentado en la orilla).
pub const MESA: Vec3 = Vec3::new(0.0, -1.0, -2.0);
const RADIO_RONDA: f32 = 1.75;
const ESCALA_TRIPULANTE: f32 = 1.5;
/// Para dibujar un solo tripulante de `tripulantes.obj`: se ocultan los otros siete.
pub const OCULTAR_TRIPULANTE: [[&str; 7]; 8] = [
    ["t1_", "t2_", "t3_", "t4_", "t5_", "t6_", "t7_"],
    ["t0_", "t2_", "t3_", "t4_", "t5_", "t6_", "t7_"],
    ["t0_", "t1_", "t3_", "t4_", "t5_", "t6_", "t7_"],
    ["t0_", "t1_", "t2_", "t4_", "t5_", "t6_", "t7_"],
    ["t0_", "t1_", "t2_", "t3_", "t5_", "t6_", "t7_"],
    ["t0_", "t1_", "t2_", "t3_", "t4_", "t6_", "t7_"],
    ["t0_", "t1_", "t2_", "t3_", "t4_", "t5_", "t7_"],
    ["t0_", "t1_", "t2_", "t3_", "t4_", "t5_", "t6_"],
];
/// Segundos entre voto y voto al contarlos.
const PASO_CONTEO: f32 = 0.5;

pub enum AccionJunta {
    Nada,
    Despegar,
}

pub struct Junta {
    fase: Fase,
    /// Segundos en la fase.
    t: f32,
    /// Segundos desde que empezo la junta.
    pub total: f32,
    cursor: usize,
    voto: Option<Voto>,
    resultado: Option<Voto>,
    pub eventos: Vec<Efecto>,
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

impl Junta {
    pub fn new() -> Self {
        Junta { fase: Fase::Alarma, t: 0.0, total: 0.0, cursor: 0, voto: None, resultado: None, eventos: vec![Efecto::Alarma] }
    }

    fn cambiar(&mut self, fase: Fase) {
        self.fase = fase;
        self.t = 0.0;
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionJunta {
        self.t += dt;
        self.total += dt;
        if e.aterrizar {
            return AccionJunta::Despegar;
        }
        // Las frases avanzan con aceptar (o solas, si nadie aprieta nada).
        let siguiente = (e.aceptar && self.t > 0.4) || self.t > 7.0;
        match self.fase {
            Fase::Alarma if self.t > 2.8 => self.cambiar(Fase::Discurso(0)),
            Fase::Discurso(i) if siguiente => {
                self.cambiar(if i + 1 < DISCURSO.len() { Fase::Discurso(i + 1) } else { Fase::Debate(0) });
            }
            Fase::Debate(i) if siguiente => {
                self.cambiar(if i + 1 < CONSEJO.len() { Fase::Debate(i + 1) } else { Fase::Votacion });
            }
            Fase::Votacion => {
                if e.arriba {
                    self.cursor = (self.cursor + OPCIONES.len() - 1) % OPCIONES.len();
                }
                if e.abajo {
                    self.cursor = (self.cursor + 1) % OPCIONES.len();
                }
                let elegido = e.respuesta.filter(|&r| r < OPCIONES.len()).or(if e.aceptar { Some(self.cursor) } else { None });
                if let Some(i) = elegido {
                    self.voto = Some(OPCIONES[i].0);
                    self.cambiar(Fase::Conteo);
                }
            }
            Fase::Conteo => {
                let antes = ((self.t - dt) / PASO_CONTEO) as usize;
                let ahora = (self.t / PASO_CONTEO) as usize;
                if ahora > antes && ahora <= CONSEJO.len() + 1 {
                    self.eventos.push(Efecto::Acierto);
                }
                if self.t > PASO_CONTEO * (CONSEJO.len() as f32 + 2.0) + 0.8 {
                    let r = self.contar();
                    self.resultado = Some(r);
                    self.eventos.push(match r {
                        Voto::Si => Efecto::Perfecto,
                        Voto::No => Efecto::Fallo,
                        Voto::Saltar => Efecto::Choque,
                    });
                    if r == Voto::Si {
                        self.eventos.push(Efecto::Fuego);
                    }
                    self.cambiar(Fase::Resultado);
                }
            }
            Fase::Resultado if e.aceptar && self.t > 0.6 => {
                // Otra votacion, por si se arrepintio.
                self.resultado = None;
                self.voto = None;
                self.cambiar(Fase::Votacion);
            }
            _ => {}
        }
        AccionJunta::Nada
    }

    /// Gana lo que tenga mas votos; el consejo esta empatado 3 a 3, asi que decide la piloto.
    fn contar(&self) -> Voto {
        let mut cuenta = [0; 3];
        for v in CONSEJO.iter().map(|c| c.2).chain(self.voto) {
            cuenta[v as usize] += 1;
        }
        let max = *cuenta.iter().max().unwrap();
        let ganadores: Vec<usize> = (0..3).filter(|&i| cuenta[i] == max).collect();
        match ganadores[..] {
            [i] => [Voto::Si, Voto::No, Voto::Saltar][i],
            _ => Voto::Saltar,
        }
    }

    /// Si Pluton ya se dibuja relleno (gano el si).
    /// Lo que voto el consejo, si ya se conto.
    pub fn resultado(&self) -> Option<Voto> {
        self.resultado
    }

    pub fn relleno(&self) -> bool {
        self.resultado == Some(Voto::Si)
    }

    /// El destello blanco cuando Pluton vuelve a rellenarse.
    pub fn destello(&self) -> f32 {
        if self.fase == Fase::Resultado && self.resultado == Some(Voto::Si) { (0.8 - self.t * 1.6).max(0.0) } else { 0.0 }
    }

    /// Que tan roja se pone la pantalla con la alarma.
    pub fn alarma(&self) -> f32 {
        if self.fase == Fase::Alarma { 0.5 + 0.5 * (self.t * 8.0).sin() } else { 0.0 }
    }

    /// Quien habla ahora: su nombre, lo que dice y su voz. Pluton es el 0 del consejo hablado.
    pub fn hablante(&self) -> Option<(&'static str, &'static str, Voz)> {
        match self.fase {
            Fase::Alarma => Some((ui::NOMBRE_IA, "Alguien presionó el botón de emergencia. Fue Plutón. Obviamente.", Voz::Ia)),
            Fase::Discurso(i) => Some(("PLUTÓN", DISCURSO[i], Voz::Pluton)),
            Fase::Debate(i) => Some((CUERPOS[CONSEJO[i].0].nombre, CONSEJO[i].1, Voz::Planeta)),
            Fase::Votacion => Some((ui::NOMBRE_IA, "Hora de votar. El consejo está empatado. Usted decide. Sin presión. Bueno, un poquito.", Voz::Ia)),
            Fase::Conteo => None,
            Fase::Resultado => Some((
                ui::NOMBRE_IA,
                match self.resultado {
                    Some(Voto::Si) => "Plutón vuelve a ser planeta. La Unión Astronómica no fue consultada. Nos van a demandar.",
                    Some(Voto::No) => "Plutón sigue de planeta enano. Se fue a su cuarto y puso música triste.",
                    _ => "Se saltó el voto. Como junta de la U: dos horas y no se decidió nada.",
                },
                Voz::Ia,
            )),
        }
    }

    pub fn expresion_pluton(&self) -> &'static str {
        match self.fase {
            Fase::Alarma => "asustada",
            Fase::Discurso(i) if i % 2 == 0 => "serio",
            Fase::Discurso(_) => "enojado",
            Fase::Debate(_) => "serio",
            Fase::Votacion | Fase::Conteo => "asustada",
            Fase::Resultado => match self.resultado {
                Some(Voto::Si) => "enamorada",
                Some(Voto::No) => "dormida",
                _ => "espiral",
            },
        }
    }

    pub fn expresion_miku(&self) -> &'static str {
        match (self.fase, self.resultado) {
            (Fase::Resultado, Some(Voto::Si)) => "enamorada",
            (Fase::Resultado, Some(Voto::No)) => "serio",
            (Fase::Alarma, _) => "asustada",
            _ => "feliz",
        }
    }

    /// Si Miku baila (gano el si) o solo saluda.
    pub fn miku_baila(&self) -> bool {
        self.relleno()
    }

    /// Donde para el tripulante `k`: alrededor de la mesa, cuatro de cada lado,
    /// de cerca de la camara hasta junto a Pluton.
    fn lugar(k: usize) -> Vec3 {
        let (lado, paso) = if k < 4 { (1.0, k) } else { (-1.0, 7 - k) };
        let a = (68.0 + 72.0 * paso as f32 / 3.0).to_radians();
        MESA + Vec3::new(lado * RADIO_RONDA * a.sin(), 0.0, -RADIO_RONDA * a.cos())
    }

    /// Cuanto brinca el tripulante `k` ahora.
    fn brinco(&self, k: usize) -> f32 {
        let habla = matches!(self.fase, Fase::Debate(j) if j == k);
        let fiesta = self.fase == Fase::Resultado && self.resultado == Some(Voto::Si);
        let b = |alto: f32, rapidez: f32| alto * (self.total * rapidez + k as f32 * 0.9).sin().abs();
        if fiesta {
            b(0.3, 7.0)
        } else if habla {
            b(0.14, 6.0)
        } else if self.fase == Fase::Alarma {
            b(0.08, 11.0)
        } else {
            b(0.02, 2.0)
        }
    }

    /// El consejo: cada planeta es un tripulante con su traje, parado
    /// alrededor de la mesa, y su planetita flotando arriba de la cabeza.
    /// Devuelve (indice en `CUERPOS`, matriz del tripulante, matriz del planetita).
    pub fn consejo(&self) -> Vec<(usize, Mat4, Mat4)> {
        let mareados = self.fase == Fase::Resultado && self.resultado == Some(Voto::Saltar);
        CONSEJO
            .iter()
            .enumerate()
            .map(|(k, &(i, _, _, escala))| {
                let habla = matches!(self.fase, Fase::Debate(j) if j == k);
                let pos = Self::lugar(k) + Vec3::new(0.0, self.brinco(k), 0.0);
                // Miran a la mesa, un poquito hacia la camara.
                let d = MESA + Vec3::new(0.0, 0.0, -2.2) - pos;
                let giro = (-d.x).atan2(-d.z) + if mareados { self.t * 4.0 + k as f32 } else { 0.0 };
                let tripulante = Mat4::translation(pos) * Mat4::rotation_y(giro) * Mat4::scale(ESCALA_TRIPULANTE);
                let r = escala * 0.32 * if habla { 1.4 } else { 1.0 };
                let arriba = pos + Vec3::new(0.0, 0.7 * ESCALA_TRIPULANTE + 0.12 + r + 0.05 * (self.total * 1.7 + k as f32).sin(), 0.0);
                let c = &CUERPOS[i];
                let planeta = Mat4::translation(arriba)
                    * Mat4::rotation_z(c.inclinacion.to_radians())
                    * Mat4::rotation_y(self.total * 0.6 + k as f32)
                    * Mat4::scale(r);
                (i, tripulante, planeta)
            })
            .collect()
    }

    /// Donde esta quien habla (para la camara y el halo).
    pub fn foco(&self) -> Vec3 {
        match self.fase {
            Fase::Debate(k) => Self::lugar(k) + Vec3::new(0.0, 0.7 * ESCALA_TRIPULANTE, 0.0),
            _ => Vec3::zero(),
        }
    }

    /// El boton de emergencia, arriba de la mesa (brilla en la alarma).
    pub fn boton(&self) -> Option<Vec3> {
        (self.fase == Fase::Alarma).then_some(MESA + Vec3::new(0.0, 0.55, 0.0))
    }

    /// Pluton gira despacio, pero siempre vuelve a mirar a la camara.
    pub fn pluton(&self) -> Mat4 {
        Mat4::rotation_y(0.35 * (self.total * 0.5).sin()) * Mat4::rotation_z(0.08 * (self.total * 0.7).sin())
    }

    /// Miku parada sobre Pluton, a un lado de la cara.
    pub fn miku(&self) -> Mat4 {
        let normal = Vec3::new(-0.62, 0.7, -0.35).normalize();
        let x = normal.cross(Vec3::new(0.0, 0.0, 1.0)).normalize();
        let z = x.cross(normal);
        Mat4::translation(normal * 1.0) * Mat4::from_basis(x, normal, z) * Mat4::rotation_y(-0.5) * Mat4::scale(0.5)
    }

    pub fn camara(&self) -> (Vec3, Vec3) {
        let lejos = suave(if matches!(self.fase, Fase::Votacion | Fase::Conteo) { self.t / 1.2 } else { 0.0 });
        let vaiven = 0.4 * (self.total * 0.15).sin();
        let ojo = Vec3::new(vaiven, 1.7, -6.3).lerp(Vec3::new(0.0, 3.0, -8.6), lejos);
        let mira = Vec3::new(0.0, -0.05, -1.3).lerp(self.foco(), 0.3);
        (ojo, mira)
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, ancho: i32, alto: i32) {
        let (aceptar, salir) = (&ui::boton(e.nombre("aceptar")), &ui::boton(e.nombre("aterrizar")));
        // La alarma: letrero grande que parpadea.
        if self.fase == Fase::Alarma {
            let titulo = "¡JUNTA DE EMERGENCIA!";
            let tam = 72;
            let w = ui::medir_titulo(titulo, tam);
            let a = if (self.t * 4.0) as i32 % 2 == 0 { 255 } else { 120 };
            ui::titulo_de(d, titulo, (ancho - w) / 2, 160, tam, Color::new(255, 70, 90, a));
        }
        // Quien habla, abajo.
        if let (Some((nombre, _, _)), false) = (self.hablante(), texto.is_empty()) {
            let w = 940.min(ancho - 40);
            let h = ui::alto_dialogo(nombre, texto, w);
            let (x, y) = ((ancho - w) / 2, alto - h - 20);
            ui::caja_dialogo(d, nombre, ui::LILA, texto, x, y, w);
            if matches!(self.fase, Fase::Discurso(_) | Fase::Debate(_)) {
                let s = format!("{aceptar}: siguiente");
                ui::texto(d, &s, x + w - 20 - ui::medir(&s, 16), y + 12, 16, ui::TENUE);
            }
        }
        // La boleta.
        if matches!(self.fase, Fase::Votacion | Fase::Conteo | Fase::Resultado) {
            let (w, h) = (560, 440);
            let (x, y) = ((ancho - w) / 2, 70);
            ui::panel(d, x, y, w, h);
            ui::texto(d, "¿PLUTÓN ES PLANETA?", x + 28, y + 18, 34, ui::TEXTO);
            let revelados = if self.fase == Fase::Votacion { 0 } else { ((self.t / PASO_CONTEO) as usize).min(CONSEJO.len() + 1) };
            let etiqueta = |v: Voto| match v {
                Voto::Si => ("SÍ", ui::VERDE),
                Voto::No => ("NO", ui::ROJO),
                Voto::Saltar => ("SALTÓ", ui::TENUE),
            };
            for (k, c) in CONSEJO.iter().enumerate() {
                let fila = y + 70 + k as i32 * 30;
                ui::texto(d, CUERPOS[c.0].nombre, x + 36, fila, 22, ui::TEXTO);
                let (t, color) = if k < revelados && self.fase != Fase::Votacion { etiqueta(c.2) } else { ("?", ui::TENUE) };
                ui::texto(d, t, x + 240, fila, 22, color);
            }
            let fila = y + 70 + CONSEJO.len() as i32 * 30;
            ui::texto(d, "TÚ", x + 36, fila, 22, ui::LILA);
            if let (Some(v), true) = (self.voto, revelados > CONSEJO.len()) {
                let (t, color) = etiqueta(v);
                ui::texto(d, t, x + 240, fila, 22, color);
            }
            if self.fase == Fase::Votacion {
                for (k, (_, nombre)) in OPCIONES.iter().enumerate() {
                    let color = if k == self.cursor { ui::LILA } else { ui::TEXTO };
                    let flecha = if k == self.cursor { ">" } else { " " };
                    ui::texto(d, &format!("{flecha} {}) {nombre}", k + 1), x + 330, y + 90 + k as i32 * 40, 24, color);
                }
                ui::texto(d, &format!("{} + {aceptar}", if e.hay_control { ui::boton("cruceta") } else { format!("{} o {}", ui::boton("1 a 3"), ui::boton("\u{E008} \u{E009}")) }), x + 330, y + 220, 18, ui::TENUE);
            }
        }
        // El resultado, grande.
        if self.fase == Fase::Resultado {
            let (texto, color) = match self.resultado {
                Some(Voto::Si) => ("PLUTÓN VUELVE A SER PLANETA", ui::VERDE),
                Some(Voto::No) => ("PLUTÓN NO ERA EL IMPOSTOR... NI PLANETA", ui::ROJO),
                _ => ("NADIE FUE EXPULSADO (VOTO SALTADO)", ui::TENUE),
            };
            // Arriba de la boleta, y las opciones adentro, abajo.
            let tam = 38;
            let w = ui::medir(texto, tam);
            ui::texto(d, texto, (ancho - w) / 2 + 3, 19, tam, Color::new(20, 5, 35, 255));
            ui::texto(d, texto, (ancho - w) / 2, 16, tam, color);
            let pie = format!("{aceptar}: votar otra vez      {salir}: despegar");
            ui::texto(d, &pie, (ancho - ui::medir(&pie, 20)) / 2, 70 + 440 - 36, 20, ui::LILA);
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Pasa el discurso y el debate apretando aceptar, vota y espera el conteo.
    fn votar(respuesta: usize) -> Junta {
        let mut j = Junta::new();
        let aceptar = Entrada { aceptar: true, ..Default::default() };
        for _ in 0..60 {
            if j.fase == Fase::Votacion {
                break;
            }
            j.actualizar(&aceptar, 0.5);
        }
        assert!(j.fase == Fase::Votacion, "deberia llegar a la votacion");
        j.actualizar(&Entrada { respuesta: Some(respuesta), ..Default::default() }, 0.0);
        for _ in 0..(10.0 * 60.0) as usize {
            j.actualizar(&Entrada::default(), 1.0 / 60.0);
        }
        assert!(j.fase == Fase::Resultado);
        j
    }

    #[test]
    fn el_voto_de_la_piloto_desempata() {
        let si = votar(0);
        assert!(si.relleno(), "con el si, Pluton vuelve a rellenarse");
        assert!(si.eventos.iter().any(|e| matches!(e, Efecto::Perfecto)));
        let no = votar(1);
        assert!(!no.relleno());
        assert_eq!(no.resultado, Some(Voto::No));
        // Saltar deja un triple empate 3-3-3... nadie es expulsado.
        assert_eq!(votar(2).resultado, Some(Voto::Saltar));
    }

    #[test]
    fn hay_un_tripulante_por_planeta_del_consejo() {
        let j = Junta::new();
        assert_eq!(j.consejo().len(), OCULTAR_TRIPULANTE.len());
        // Nadie se para encima de la mesa ni adentro de Pluton.
        for k in 0..CONSEJO.len() {
            let p = Junta::lugar(k);
            assert!((p - MESA).length() > 1.2, "el tripulante {k} esta en la mesa");
            assert!(p.length() > 1.3, "el tripulante {k} esta adentro de Pluton");
        }
    }
}
