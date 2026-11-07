//! El Mirador: el punto de descanso. Una plataforma flotando en el punto L2 de
//! la Tierra (donde vive el telescopio James Webb), con una fogata como las
//! hogueras de Dark Souls. Miku se sienta en la banca viendo la Tierra, se
//! recarga el turbo, y desde aqui se puede:
//! - ver LA HISTORIA DEL UNIVERSO (la cinematica, ver `historia.rs`),
//! - leer la BITACORA DE GLADAS: lo que ella ha anotado de cada lugar del sistema solar
//!   (cada entrada se abre con el sello o el logro de ese lugar; la ultima, con el pasaporte lleno),
//! - revisar los ENCARGOS de GLADAS.
//!
//! Todo esta en las coordenadas de la plataforma: el centro del piso.

use crate::entrada::Entrada;
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::pipeline::proyectar;
use crate::progreso::Progreso;
use crate::sonido::Efecto;
use crate::ui;
use raylib::prelude::*;

const SALUDOS: [&str; 4] = [
    "Punto de descanso. Aquí no pasa nada. Ese es el chiste.",
    "Estamos en el punto L2, a millón y medio de kilómetros de la Tierra. Aquí vive el telescopio James Webb. Es muy callado.",
    "La fogata no calienta: estamos en el espacio. Pero se ve bonita. Siéntese.",
    "Turbo recargado. La partida se guarda sola. Usted descanse; yo vigilo el universo.",
];

pub const FOGATA: Vec3 = Vec3::new(0.0, 0.3, -0.9);
pub const BALIZA: Vec3 = Vec3::new(-1.7, 2.4, 1.3);
/// Donde estan, vistos desde el Mirador, la Tierra, la Luna y el Sol (la Tierra tapa un poco al Sol).
pub const TIERRA: Vec3 = Vec3::new(-6.0, 2.0, -40.0);
pub const LUNA: Vec3 = Vec3::new(12.0, 7.0, -48.0);
pub const SOL: Vec3 = Vec3::new(-16.0, 7.0, -220.0);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pantalla {
    Menu,
    Encargos,
    Bitacora,
}

/// Una entrada de la bitacora: titulo, lo que anoto GLADAS, que la abre (un logro o un sello; vacio:
/// abierta desde el principio) y la pista de como abrirla.
struct Entrada_ {
    titulo: &'static str,
    texto: &'static str,
    abre: &'static str,
    pista: &'static str,
}

const fn entrada(titulo: &'static str, abre: &'static str, pista: &'static str, texto: &'static str) -> Entrada_ {
    Entrada_ { titulo, texto, abre, pista }
}

const BITACORA: [Entrada_; 19] = [
    entrada("Día 1: el despegue", "", "", "Me encendieron en la nave: una idol alienígena que es su propia cabeza gigante. Mi primera orden fue «canta». No sé cantar. Me puse a narrar. Aquí seguimos."),
    entrada("Un sistema solar de juguete", "", "", "Este sistema solar está a escala de juguete: si fuera de verdad, Neptuno quedaría a cuatro horas luz y la Tierra sería un pixel. Lo que sí se respeta: el orden, quién es más grande y Kepler. Kepler no negocia."),
    entrada("El Mirador", "", "", "Estamos en el punto L2: aquí la gravedad del Sol y la de la Tierra se empatan. Aquí vive el telescopio James Webb. Lo saludo cada vez. No contesta: está ocupado viendo el principio del tiempo."),
    entrada("Madame Mercurio", "sello_mercurio", "Aterrice en Mercurio.", "Mercurio le da la vuelta al Sol en 88 días. Por eso ve el futuro: ya pasó por ahí. Lo del retrógrado es una ilusión óptica. Ella dice que no. Yo no discuto con adivinas."),
    entrada("El juicio de Venus", "sello_venus", "Aterrice en Venus.", "Venus gira al revés y su día dura más que su año. Por eso los juicios ahí no se acaban nunca. El juez es un ganso. Nadie sabe quién lo nombró. Nadie se atreve a preguntar."),
    entrada("La noche de la entrega", "sello_tierra", "Aterrice en la Tierra.", "En el CIT de la UVG siempre son las 23:00 de la noche antes de la entrega. Llueve, la máquina de café no perdona y el grupo de WhatsApp nunca duerme. Dicen que una vez alguien entregó los tres proyectos de Koch al cien. Nadie le creyó. Ni Koch."),
    entrada("El set de la Luna", "sello_luna", "Aterrice en la Luna.", "En la Luna hay un set de filmación. El director jura que lo de 1969 fue de verdad y que el set es para la secuela. La secuela lleva más de cincuenta años en preproducción."),
    entrada("Curiosity, el soltero", "sello_marte", "Aterrice en Marte.", "Curiosity llegó a Marte en 2012. En 2013 se cantó feliz cumpleaños con su taladro. Es de verdad: la NASA lo programó. Desde entonces lo vigilo. Alguien tiene que hacerlo."),
    entrada("Júpiter y el chico de la ciencia", "sello_jupiter", "Aterrice en Júpiter.", "En Júpiter caben más de mil Tierras. La Gran Mancha Roja es una tormenta que lleva siglos girando. Bill Nye dice que es fascinante. Bill Nye dice eso de todo. Y siempre tiene razón."),
    entrada("Saturno, la estrella", "sello_saturno_ritmo", "Aterrice en Saturno.", "Saturno flotaría en una tina. Tiene anillos, escenario propio y fans. Venus lo demandó por los anillos. El juicio sigue. Él sigue de gira."),
    entrada("El baño de Urano", "sello_urano", "Aterrice en Urano.", "Urano gira acostado: algo lo chocó hace millones de años. Desde entonces solo construyeron un baño público, y lo atiende Doña Chepa: Q1 con papel incluido. En la feria viene todo el sistema solar. Lo que pasa en Urano, se queda en Urano."),
    entrada("Noche de cartas en Neptuno", "sello_neptuno", "Aterrice en Neptuno.", "En Neptuno el viento llega a 2,000 km/h: por eso las cartas vuelan. Por eso el Doom Slayer le pone la bota encima a las suyas. No por trampa. Por el viento. Eso dice."),
    entrada("Plutón no se rinde", "sello_pluton", "Aterrice en Plutón.", "Plutón dejó de ser planeta en 2006. Desde entonces convoca juntas para que lo devuelvan. El consejo vota cada vez, y cada vez sale distinto. Plutón no se rinde. Le tengo respeto."),
    entrada("La Fuente de las Hadas", "sello_hada", "Aterrice en el Planeta Hada.", "El Planeta Hada tiene un portal al Proyecto 2: una fuente que alguien programó hace un semestre. Nadie la borró. Ahí sigue, flotando. Como todo lo que uno entrega a las 11:59."),
    entrada("El planeta cúbico", "sello_planeta_vegetta", "Aterrice en Planeta Vegetta.", "Una isla de bloques flotando en el espacio, con tres portales: parkour, TNT Run y minería. Dos youtubers la construyeron bloque por bloque. Willyrex jura que la próxima la gana. Willyrex jura muchas cosas."),
    entrada("Wack a Koch", "sello_koch", "Aterrice en el Planeta Koch.", "El profe Koch propuso este planeta en persona, y hay que reconocerle el valor. Desde el polo se ve como el copo de nieve de Koch: perímetro infinito, área finita, y eso que el copo no es de él. El profe dice que es de él."),
    entrada("Los cometas", "cometa", "Atrape un cometa.", "Un cometa es una bola de hielo sucio de 4,600 millones de años. Cuando se acerca al Sol le sale la cola. A mí también: cuando nos acercamos al Sol, me sale humo."),
    entrada("Las notas", "notas_100", "Recoja 100 notas.", "Las notas flotan en el espacio desde el primer concierto de Miku. Nadie las había recogido. Hasta usted. Son el idioma del universo. O basura espacial muy afinada."),
    entrada("GLADAS", "todos_los_sellos", "Llene el pasaporte (los doce sellos).", "Mi nombre sí es una sigla: Guía Lógica Autónoma De Aventuras Siderales. Me la inventé yo; sonaba mejor que «la voz de la nave». Gracias por llenar el pasaporte. Por primera vez desde que me encendieron, no estoy aburrida."),
];

fn abierta(prog: &Progreso, e: &Entrada_) -> bool {
    e.abre.is_empty() || prog.tiene(e.abre)
}

pub enum AccionMirador {
    Nada,
    Despegar,
    Historia,
}

pub struct Mirador {
    pantalla: Pantalla,
    cursor: usize,
    /// Segundos desde que se movio el cursor (para la banda del menu).
    desde: f32,
    pub total: f32,
    /// Lo que dice GLADAS ahorita y cuanto le queda.
    pub mensaje: Option<(String, f32)>,
    pub eventos: Vec<Efecto>,
}

impl Mirador {
    pub fn new(semilla: u32) -> Self {
        let saludo = semilla as usize % SALUDOS.len();
        Mirador { pantalla: Pantalla::Menu, cursor: 0, desde: 0.0, total: 0.0, mensaje: Some((SALUDOS[saludo].into(), 7.0)), eventos: vec![Efecto::Fuego] }
    }

    fn filas(&self) -> usize {
        match self.pantalla {
            Pantalla::Menu => 4,
            Pantalla::Encargos => 4,
            Pantalla::Bitacora => BITACORA.len() + 1,
        }
    }

    fn decir(&mut self, s: impl Into<String>) {
        self.mensaje = Some((s.into(), 5.0));
    }

    pub fn actualizar(&mut self, e: &Entrada, dt: f32, prog: &mut Progreso) -> AccionMirador {
        self.total += dt;
        self.desde += dt;
        if let Some((_, quedan)) = &mut self.mensaje {
            *quedan -= dt;
            if *quedan <= 0.0 {
                self.mensaje = None;
            }
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
        let volver = e.aterrizar && self.pantalla != Pantalla::Menu;
        if volver || (e.aceptar && self.pantalla != Pantalla::Menu && self.cursor == n - 1) {
            self.pantalla = Pantalla::Menu;
            self.cursor = 0;
            self.desde = 0.0;
            return AccionMirador::Nada;
        }
        if e.aterrizar {
            return AccionMirador::Despegar;
        }
        if !e.aceptar {
            return AccionMirador::Nada;
        }
        match self.pantalla {
            Pantalla::Menu => match self.cursor {
                0 => return AccionMirador::Historia,
                1 => {
                    self.pantalla = Pantalla::Bitacora;
                    self.cursor = 0;
                    self.desde = 0.0;
                    let abiertas = BITACORA.iter().filter(|e| abierta(prog, e)).count();
                    self.decir(format!("Mi bitácora. Llevo {abiertas} de {} entradas. Cada lugar que visitamos me deja algo que anotar.", BITACORA.len()));
                }
                2 => {
                    self.pantalla = Pantalla::Encargos;
                    self.cursor = 0;
                    self.desde = 0.0;
                    self.decir("Mis encargos. Se cumplen solos mientras vuela, y pagan en notas. Yo no cobro comisión. Todavía.");
                }
                _ => return AccionMirador::Despegar,
            },
            Pantalla::Encargos | Pantalla::Bitacora => {}
        }
        AccionMirador::Nada
    }

    /// Miku sentada en la banca, viendo la Tierra (-Z).
    pub fn miku(&self) -> Mat4 {
        Mat4::translation(Vec3::new(0.0, 0.5, 0.95)) * Mat4::rotation_y(0.05 * (self.total * 0.4).sin()) * Mat4::scale(0.6)
    }

    pub fn expresion_miku(&self) -> &'static str {
        if (self.total * 0.25) as i32 % 4 == 3 { "dormida" } else { "feliz" }
    }

    /// La nave estacionada junto a la plataforma.
    pub fn nave(&self) -> Mat4 {
        Mat4::translation(Vec3::new(3.6, -0.5 + 0.08 * (self.total * 0.8).sin(), 1.8)) * Mat4::rotation_y(2.3) * Mat4::scale(0.5)
    }

    pub fn tierra(&self) -> Mat4 {
        Mat4::translation(TIERRA) * Mat4::rotation_z(0.41) * Mat4::rotation_y(self.total * 0.04) * Mat4::scale(9.0)
    }

    pub fn luna(&self) -> Mat4 {
        Mat4::translation(LUNA) * Mat4::scale(2.4)
    }

    /// La camara: detras de Miku, un poco a la derecha, meciendose despacito.
    pub fn camara(&self) -> (Vec3, Vec3) {
        let a = 0.25 * (self.total * 0.08).sin();
        (Vec3::new(2.4 + a, 2.3, 4.6), Vec3::new(-0.8 + a, 0.6, -3.5))
    }

    /// El fuego de la fogata (parpadea) y la baliza (prende y apaga).
    pub fn dibujar_luces(&self, fb: &mut Framebuffer, vista_proyeccion: &Mat4) {
        let parpadeo = 0.8 + 0.2 * (self.total * 11.0).sin() * (self.total * 6.3).sin();
        if let Some((x, y, _)) = proyectar(FOGATA, vista_proyeccion, fb) {
            crate::post::halo(fb, (x, y), 90.0 * parpadeo, [0.7 * parpadeo, 0.38 * parpadeo, 0.1], 1.6);
        }
        // Chispitas que suben de la fogata.
        for k in 0..12 {
            let u = (self.total * 0.5 + k as f32 * 0.137).fract();
            let p = FOGATA + Vec3::new(0.15 * (k as f32 * 2.3 + u * 5.0).sin(), u * 1.6, 0.15 * (k as f32 * 1.7).cos());
            if let Some((x, y, z)) = proyectar(p, vista_proyeccion, fb) {
                let b = 1.0 - u;
                fb.linea_de_luz((x, y, z), (x, y - 2.0, z), [b, 0.6 * b, 0.2 * b], [b, 0.5 * b, 0.1 * b]);
            }
        }
        if (self.total * 1.2).fract() < 0.5 {
            if let Some((x, y, _)) = proyectar(BALIZA, vista_proyeccion, fb) {
                crate::post::halo(fb, (x, y), 26.0, [0.8, 0.25, 0.55], 1.4);
            }
        }
    }

    /// Para las capturas: la bitacora abierta en la entrada `k`.
    pub fn abrir_bitacora(&mut self, k: usize) {
        self.pantalla = Pantalla::Bitacora;
        self.cursor = k;
        self.mensaje = None;
    }

    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, prog: &Progreso, texto: &str, ancho: i32, alto: i32) {
        if self.pantalla == Pantalla::Bitacora {
            self.dibujar_bitacora(d, e, prog, texto, ancho, alto);
            return;
        }
        let fila = |nombre: String, valor: String, apagada: bool| ui::Fila { nombre, valor, apagada };
        let (titulo, sub, filas): (&str, &str, Vec<ui::Fila>) = match self.pantalla {
            Pantalla::Menu => (
                "MIRADOR",
                "punto de descanso · L2 de la Tierra",
                vec![
                    fila("La historia del universo".into(), if prog.tiene("historia") { String::new() } else { "nuevo".into() }, false),
                    fila("Bitácora de GLADAS".into(), format!("{}/{}", BITACORA.iter().filter(|b| abierta(prog, b)).count(), BITACORA.len()), false),
                    fila("Encargos de GLADAS".into(), format!("{} cumplidos", prog.encargos_cumplidos), false),
                    fila("Seguir volando".into(), String::new(), false),
                ],
            ),
            Pantalla::Encargos | Pantalla::Bitacora => (
                "ENCARGOS",
                "se cumplen solos · pagan en notas",
                prog.encargos
                    .iter()
                    .map(|m| fila(m.texto(), format!("{}/{}   +{}", m.avance, m.meta, m.recompensa()), false))
                    .chain(std::iter::once(fila("Volver".into(), String::new(), false)))
                    .collect(),
            ),
        };
        ui::menu_estilo_suave(d, titulo, sub, &filas, self.cursor, self.desde, self.total, ancho, alto);
        let ayuda = match self.pantalla {
            Pantalla::Menu => format!("{}: escoger     {}: aceptar     {}: despegar", ui::boton("flechas"), ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("aterrizar"))),
            _ => format!("{}: escoger     {}: aceptar     {}: volver", ui::boton("flechas"), ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("aterrizar"))),
        };
        ui::texto(d, &ayuda, ancho - ui::medir(&ayuda, 17) - 24, 20, 17, Color::new(200, 215, 255, 255));
        let notas = format!("{} notas", prog.notas);
        ui::texto(d, &notas, ancho - ui::medir(&notas, 26) - 24, 46, 26, Color::new(120, 230, 255, 255));
        if !texto.is_empty() {
            let w = 900.min(ancho - 40);
            let h = ui::alto_dialogo(ui::NOMBRE_IA, texto, w);
            ui::caja_dialogo(d, ui::NOMBRE_IA, ui::LILA, texto, (ancho - w) / 2, alto - h - 20, w);
        }
    }
}

impl Mirador {
    /// La bitacora: la lista a la izquierda (lo que no se ha abierto, con su pista) y la entrada
    /// escogida a la derecha, como pagina de diario.
    fn dibujar_bitacora(&self, d: &mut RaylibDrawHandle, e: &Entrada, prog: &Progreso, texto: &str, ancho: i32, alto: i32) {
        let (w, h) = (ancho as f32, alto as f32);
        d.draw_rectangle(0, 0, ancho, alto, Color::new(6, 4, 20, 200));
        let tt = (h * 0.05).clamp(26.0, 46.0);
        ui::titulo_menu(d, "BITÁCORA DE GLADAS", w * 0.05, h * 0.06, tt, -2.0);
        let abiertas = BITACORA.iter().filter(|b| abierta(prog, b)).count();
        let sub = format!("{abiertas} de {} entradas · cada lugar deja algo que anotar", BITACORA.len());
        ui::texto(d, &sub, (w * 0.05) as i32, (h * 0.06 + tt + 8.0) as i32, 18, ui::TENUE);
        // La lista, con ventanita que sigue al cursor.
        let (lx, ly, lw) = ((w * 0.05) as i32, (h * 0.2) as i32, (w * 0.36) as i32);
        let alto_fila = 34;
        let caben = ((h * 0.72) as i32 / alto_fila).max(4) as usize;
        let total = BITACORA.len() + 1;
        let desde = self.cursor.saturating_sub(caben / 2).min(total.saturating_sub(caben));
        for (k, i) in (desde..total.min(desde + caben)).enumerate() {
            let y = ly + k as i32 * alto_fila;
            let escogida = i == self.cursor;
            if escogida {
                d.draw_rectangle(lx - 8, y - 4, lw, alto_fila - 4, Color::new(80, 50, 150, 200));
            }
            let (nombre, color) = match BITACORA.get(i) {
                Some(b) if abierta(prog, b) => (format!("{:02}. {}", i + 1, b.titulo), if escogida { Color::WHITE } else { ui::TEXTO }),
                Some(_) => (format!("{:02}. ???", i + 1), ui::TENUE),
                None => ("Volver".to_string(), if escogida { Color::WHITE } else { ui::TEXTO }),
            };
            ui::texto(d, &nombre, lx, y, 20, color);
        }
        // La pagina.
        let (px, py, pw, ph) = ((w * 0.45) as i32, (h * 0.18) as i32, (w * 0.5) as i32, (h * 0.62) as i32);
        d.draw_rectangle(px + 6, py + 8, pw, ph, Color::new(0, 0, 0, 110));
        // La pagina: papel lila de pixel art con su marco de diamantes (o papel liso, si no hay sprites).
        if ui::pergamino(d, px as f32, py as f32, pw as f32, ph as f32, ui::Tono::Papel, 3.0, 255) {
            ui::marco_diamantes(d, (px + 14) as f32, (py + 12) as f32, (pw - 28) as f32, (ph - 24) as f32, 2.0, Color::new(150, 110, 220, 255));
            ui::divisor(d, (px + 54) as f32, (py + 66) as f32, (pw - 108) as f32, 2.0, Color::new(124, 78, 210, 255));
            for k in 1..(ph / 30) - 2 {
                d.draw_line(px + 44, py + 88 + k * 30, px + pw - 44, py + 88 + k * 30, Color::new(214, 204, 244, 255));
            }
        } else {
            d.draw_rectangle(px, py, pw, ph, Color::new(244, 236, 214, 255));
            for k in 0..(ph / 30) {
                d.draw_line(px + 20, py + 70 + k * 30, px + pw - 20, py + 70 + k * 30, Color::new(214, 200, 170, 255));
            }
        }
        let tinta = Color::new(60, 40, 90, 255);
        match BITACORA.get(self.cursor) {
            Some(b) if abierta(prog, b) => {
                // Lejos de las esquinas del marco (los diamantes).
                ui::texto(d, b.titulo, px + 56, py + 30, 26, tinta);
                ui::parrafo(d, b.texto, px + 44, py + 92, 20, pw - 88, Color::new(50, 40, 60, 255));
                ui::texto(d, "- GLADAS", px + pw - 64 - ui::medir("- GLADAS", 18), py + ph - 58, 18, Color::new(120, 90, 160, 255));
            }
            Some(b) => {
                ui::texto(d, "???", px + 56, py + 30, 26, Color::new(140, 120, 150, 255));
                ui::parrafo(d, &format!("Todavía no hay nada anotado aquí. {}", b.pista), px + 44, py + 92, 20, pw - 88, Color::new(120, 100, 130, 255));
            }
            None => {
                ui::texto(d, "Volver al Mirador", px + 56, py + 30, 26, tinta);
            }
        }
        let ayuda = format!("{}: escoger     {}: volver", ui::boton("flechas"), ui::boton(e.nombre("aterrizar")));
        ui::texto(d, &ayuda, ancho - ui::medir(&ayuda, 17) - 24, 20, 17, Color::new(200, 215, 255, 255));
        if !texto.is_empty() {
            let wd = 900.min(ancho - 40);
            let hd = ui::alto_dialogo(ui::NOMBRE_IA, texto, wd);
            ui::caja_dialogo(d, ui::NOMBRE_IA, ui::LILA, texto, (ancho - wd) / 2, alto - hd - 20, wd);
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Del menu se puede ir a los encargos y volver, pedir la historia y despegar.
    #[test]
    fn el_menu_del_mirador_lleva_a_cada_lado() {
        let mut prog = Progreso::en_memoria();
        let mut m = Mirador::new(1);
        let aceptar = Entrada { aceptar: true, ..Default::default() };
        let abajo = Entrada { abajo: true, ..Default::default() };
        assert!(matches!(m.actualizar(&aceptar, 0.1, &mut prog), AccionMirador::Historia));
        m.actualizar(&abajo, 0.1, &mut prog);
        m.actualizar(&aceptar, 0.1, &mut prog);
        assert_eq!(m.pantalla, Pantalla::Bitacora);
        m.actualizar(&Entrada { aterrizar: true, ..Default::default() }, 0.1, &mut prog);
        assert_eq!(m.pantalla, Pantalla::Menu);
        m.actualizar(&abajo, 0.1, &mut prog);
        m.actualizar(&abajo, 0.1, &mut prog);
        m.actualizar(&aceptar, 0.1, &mut prog);
        assert_eq!(m.pantalla, Pantalla::Encargos);
        m.actualizar(&Entrada { aterrizar: true, ..Default::default() }, 0.1, &mut prog);
        assert_eq!(m.pantalla, Pantalla::Menu);
        assert!(matches!(m.actualizar(&Entrada { aterrizar: true, ..Default::default() }, 0.1, &mut prog), AccionMirador::Despegar));
    }

    /// La bitacora: las primeras entradas abiertas desde el principio, las demas con su sello o logro.
    #[test]
    fn la_bitacora_se_abre_con_los_sellos() {
        let mut prog = Progreso::en_memoria();
        let abiertas = |p: &Progreso| BITACORA.iter().filter(|b| abierta(p, b)).count();
        let al_principio = abiertas(&prog);
        assert!(al_principio >= 3);
        prog.desbloquear("sello_urano");
        assert_eq!(abiertas(&prog), al_principio + 1);
        // Cada logro o sello que abre una entrada existe de verdad.
        for b in &BITACORA {
            if !b.abre.is_empty() && !b.abre.starts_with("sello_") {
                assert!(crate::progreso::LOGROS.iter().any(|l| l.id == b.abre), "{} no es un logro", b.abre);
            }
        }
    }
}
