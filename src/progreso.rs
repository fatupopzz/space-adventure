//! El progreso que se guarda entre partidas: el PASAPORTE.
//!
//! Cada planeta tiene su sello (se pone al aterrizar) y sus finales y logros
//! (los distintos desenlaces de cada escena: la boda en Marte, el ganso que se
//! roba los anillos, el Oscar de roca en la Luna...). Los que faltan salen
//! como "???" con una pista, para que den ganas de volver. Tambien se cuentan
//! las notas musicales que se recogen volando.
//!
//! Ademas: los ENCARGOS de GLADAS (tres misiones cortas siempre activas: al
//! cumplir una, paga notas y sale otra), las ESTELAS que se compran con notas
//! en el Mirador, y los ajustes del menu de pausa.
//!
//! Se guarda en `partida.txt`, una linea por cosa (`logro <id>`, `notas <n>`...).

use crate::escena::CUERPOS;
use crate::pausa::Ajustes;
use crate::ui;
use raylib::prelude::*;
use std::collections::BTreeSet;
use std::path::PathBuf;

pub const ARCHIVO: &str = "partida.txt";

/// Un final o logro: su id (lo que se guarda), de que planeta es (indice en `CUERPOS`; `None` = del
/// espacio), como se llama y la pista que se ve mientras no se tiene.
pub struct Logro {
    pub id: &'static str,
    pub planeta: Option<usize>,
    pub nombre: &'static str,
    pub pista: &'static str,
}

const fn logro(id: &'static str, planeta: Option<usize>, nombre: &'static str, pista: &'static str) -> Logro {
    Logro { id, planeta, nombre, pista }
}

// Los indices de `CUERPOS` (ver escena.rs).
const MERCURIO: Option<usize> = Some(1);
const VENUS: Option<usize> = Some(2);
const TIERRA: Option<usize> = Some(3);
const LUNA: Option<usize> = Some(4);
const MARTE: Option<usize> = Some(5);
const JUPITER: Option<usize> = Some(6);
const URANO: Option<usize> = Some(8);
const SATURNO: Option<usize> = Some(7);
const NEPTUNO: Option<usize> = Some(9);
const PLUTON: Option<usize> = Some(10);
const HADA: Option<usize> = Some(11);
const VEGETTA: Option<usize> = Some(12);
const CHAPIN: Option<usize> = Some(13);

pub const LOGROS: [Logro; 77] = [
    logro("tarot_lectura", MERCURIO, "Lectura completa", "Que Madame Mercurio le lea las tres cartas."),
    logro("tarot_invertidas", MERCURIO, "Todo al revés", "Que salgan las tres cartas invertidas."),
    logro("tarot_bendicion", MERCURIO, "Bendecida", "Que las tres cartas den BENDICIÓN."),
    logro("tarot_salazon", MERCURIO, "Salazón total", "Que las cartas den la peor suerte posible."),
    logro("tarot_limpia", MERCURIO, "Limpia con huevo", "Pídale a Madame una limpia."),
    logro("tarot_baraja", MERCURIO, "La baraja completa", "Descubra las 22 cartas del tarot."),
    logro("divorcio_3", VENUS, "Saturno conserva sus 3 anillos", "Gane los tres casos del divorcio."),
    logro("divorcio_0", VENUS, "Saturno sin anillos", "Pierda los tres casos. A propósito, se entiende."),
    logro("divorcio_ganso", VENUS, "El collar del juez", "Agote la paciencia del juez hasta que se robe algo."),
    logro("examen_5", TIERRA, "Tres de tres al cien", "Entregue Gráficas, Teoría y Software 2 al 100% a las 23:59."),
    logro("examen_aprobada", TIERRA, "Pasó raspado", "Apruebe los tres proyectos de Koch (61 o más en cada uno)."),
    logro("examen_reglazo", TIERRA, "Dormido en el teclado", "Quedarse sin energía la noche de la entrega (y despertar con «jjjjjjj» en el código)."),
    logro("koch_examen", TIERRA, "Examen sorpresa", "Termine el examen del profe Koch en el salón de Teoría (spin-off de la Tierra)."),
    logro("koch_5", TIERRA, "Cinco de cinco", "Saque 5/5 en el examen del profe Koch. Sin un solo reglazo."),
    logro("koch_reglazo", TIERRA, "Reglazo triple", "Repruebe el examen de Koch por la vía rápida: tres errores, tres reglazos."),
    logro("luna_oscar", LUNA, "Óscar de roca", "Seis de seis en la toma de la Luna."),
    logro("luna_razzie", LUNA, "Peor película del año", "Que la toma salga desastrosa."),
    logro("luna_estrellas", LUNA, "Estrellas de cartulina", "Pídale al director que pinte estrellas."),
    logro("luna_cables", LUNA, "Astronauta colgado", "Ponerle cables al astronauta."),
    logro("cita_boda", MARTE, "¡Boda en Marte!", "Que la cita salga perfecta. Perfectísima."),
    logro("cita_exito", MARTE, "Segunda cita", "Que la cita salga bien."),
    logro("cita_amigos", MARTE, "Solo amigos", "Que la cita salga... más o menos."),
    logro("cita_solo", MARTE, "Rover solito", "Que la cita salga mal."),
    logro("cita_drama", MARTE, "Golpiza en Marte", "Sálgale bien a dos en la misma visita. A ver qué pasa."),
    logro("urano_juega", URANO, "Primer turno", "Trabaje un día en el baño de Doña Chepa, en Urano."),
    logro("urano_campeon", URANO, "Empleado del mes", "Pase los tres días en el baño de Doña Chepa (hasta el domingo de feria)."),
    logro("ritmo_termino", SATURNO, "Primera canción", "Termine una canción de Saturno Ritmo."),
    logro("ritmo_s", SATURNO, "Nota S", "Saque S en Saturno Ritmo."),
    logro("ritmo_dificil", SATURNO, "Terminar en DIFÍCIL", "Termine la canción en DIFÍCIL."),
    logro("ritmo_sin_fallos", SATURNO, "Cero fallos", "Termine una canción sin fallar ni una nota."),
    logro("uno_gana", NEPTUNO, "¡NEPTUNO!", "Gánele a tres héroes de videojuego."),
    logro("uno_pierde", NEPTUNO, "Derrota épica", "Pierda contra un héroe de videojuego."),
    logro("uno_canela", NEPTUNO, "Tom Nook lo busca", "Que Canela se meta a la partida."),
    logro("uno_pluton", NEPTUNO, "Plutón quiere jugar", "Que Plutón llegue rodando a la mesa."),
    logro("junta_si", PLUTON, "Plutón es planeta", "Que el consejo vote que sí."),
    logro("junta_no", PLUTON, "Plutón sigue en su cuarto", "Que el consejo vote que no."),
    logro("vegetta_juega", VEGETTA, "Bienvenido al servidor", "Termine un minijuego en el servidor de Vegetta."),
    logro("vegetta_carrera", VEGETTA, "Le ganó a Willyrex", "Gánele a Willyrex en cualquier minijuego del servidor."),
    logro("vegetta_tnt", VEGETTA, "El último en pie", "Gane TNT Run: que Willy se caiga primero."),
    logro("vegetta_mina", VEGETTA, "Roca madre", "Llegue primero a la roca madre en la minería."),
    logro("vegetta_triple", VEGETTA, "Dueño del servidor", "Gánele a Willyrex en los tres minijuegos."),
    logro("hada_luka", HADA, "La Fuente de Luka", "Toque la Canción de Miku completa."),
    logro("camioneta_terminal", CHAPIN, "¡Capital, capital!", "Llegue a la terminal manejando la camioneta."),
    logro("camioneta_llena", CHAPIN, "Siempre hay campo", "Llegue a la terminal con 40 pasajeros o más."),
    logro("camioneta_tumulos", CHAPIN, "Chofer con cuidado", "Pase cinco túmulos frenando en un solo viaje."),
    // Las maravillas (homenajes; ver maravillas.rs).
    logro("gusano", SATURNO, "Atajo por el gusano", "Entre al agujero de gusano que está junto a Saturno."),
    logro("teseracto", None, "Más allá del horizonte", "Cruce el horizonte de Gargantúa (y salga del teseracto)."),
    logro("rocky", VENUS, "¡Puño mi choque!", "Encuentre a Rocky en su nave, cerca de Venus."),
    logro("monolito", JUPITER, "Está lleno de estrellas", "Encuentre el monolito cerca de Júpiter."),
    logro("huerto", MARTE, "Botánico marciano", "Encuentre el huerto de papas del polo de Marte."),
    logro("no_panico", None, "No entre en pánico", "Que lo rescate la estrella por irse muy lejos."),
    logro("hada_error", HADA, "¡Hey! ¡Escucha!", "Que Navi la regañe por una nota mala."),
    logro("notas_10", None, "Diez notas", "Recoja 10 notas volando."),
    logro("notas_100", None, "Cien notas", "Recoja 100 notas volando."),
    logro("notas_500", None, "Disco de oro", "Recoja 500 notas volando."),
    logro("combo_10", None, "Combo de 10", "Recoja 10 notas seguidas, sin parar."),
    logro("combo_25", None, "Escala completa", "Recoja 25 notas seguidas."),
    logro("cometa", None, "Cometa atrapado", "Alcance un cometa cuando pase."),
    logro("cometa_3", None, "Cazacometas", "Atrape tres cometas."),
    logro("barrel_10", None, "Mareada", "Haga 10 barrel rolls."),
    logro("hiperespacio", None, "Hiperespacio", "Salte al hiperespacio."),
    logro("sol", None, "Demasiado cerca", "Acérquese mucho al Sol."),
    logro("todos_los_sellos", None, "Pasaporte lleno", "Ponga los doce sellos del pasaporte."),
    logro("pipeline", None, "Pipeline por dentro", "Vea el z-buffer, las normales, las UV y el stencil."),
    logro("mirador", None, "Punto de descanso", "Descanse en el Mirador (está en el punto L2 de la Tierra)."),
    logro("historia", None, "La historia del universo", "Véala completa en el Mirador."),
    logro("estela", None, "De compras", "Compre una estela en el Mirador o en la tienda."),
    logro("jupiter_juega", JUPITER, "Jupiter Asteroids", "Juegue una partida de asteroides en Júpiter."),
    logro("jupiter_mancha", JUPITER, "Tormenta calmada", "Derrote a la Gran Mancha Roja."),
    logro("jupiter_5000", JUPITER, "Puntería de la NASA", "Haga 5,000 puntos en Júpiter Asteroids."),
    logro("jupiter_lunas", JUPITER, "Galileo estaría orgulloso", "Atrape las cuatro lunas de Galileo en una partida."),
    logro("asteroide_rojo", None, "Se me cayeron", "Choque con un asteroide caliente (los rojos)."),
    logro("ciencia_5", None, "¡Ciencia!", "Escuche cinco datos de Bill Nye (pase junto a los asteroides dorados)."),
    logro("ciencia_todo", None, "El chico de la ciencia", "Escuche los 32 datos de Bill Nye."),
    logro("skin", None, "Cambio de look", "Compre una nave nueva en la tienda."),
    logro("encargos_5", None, "Mandadera", "Cumpla cinco encargos de GLADAS."),
    logro("encargos_20", None, "Empleada del mes", "Cumpla veinte encargos de GLADAS."),
];

/// Los logros que no cuentan como "final nuevo" para los encargos (si no, un encargo se cumpliria solo).
const NO_CUENTAN: [&str; 4] = ["encargos_5", "encargos_20", "notas_10", "notas_100"];

/// Las misiones cortas de GLADAS.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Encargo {
    Notas,
    Combo,
    Cometa,
    /// Aterrizar en el planeta (indice en `CUERPOS`).
    Aterrizar(usize),
    Barrels,
    /// Conseguir cualquier final o logro nuevo.
    Final,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mision {
    pub tipo: Encargo,
    pub meta: u32,
    pub avance: u32,
}

impl Mision {
    pub fn texto(&self) -> String {
        match self.tipo {
            Encargo::Notas => format!("Recoge {} notas", self.meta),
            Encargo::Combo => format!("Haz un combo de {}", self.meta),
            Encargo::Cometa => "Atrapa un cometa".into(),
            Encargo::Aterrizar(p) => format!("Aterriza en {}", CUERPOS[p].nombre),
            Encargo::Barrels => format!("Haz {} barrel rolls", self.meta),
            Encargo::Final => "Consigue un final o logro nuevo".into(),
        }
    }

    pub fn recompensa(&self) -> u32 {
        match self.tipo {
            Encargo::Notas => self.meta + 5,
            Encargo::Combo => self.meta * 3,
            Encargo::Cometa => 30,
            Encargo::Aterrizar(_) => 20,
            Encargo::Barrels => 10,
            Encargo::Final => 25,
        }
    }

    fn codigo(&self) -> (&'static str, usize) {
        match self.tipo {
            Encargo::Notas => ("notas", 0),
            Encargo::Combo => ("combo", 0),
            Encargo::Cometa => ("cometa", 0),
            Encargo::Aterrizar(p) => ("aterrizar", p),
            Encargo::Barrels => ("barrels", 0),
            Encargo::Final => ("final", 0),
        }
    }
}

/// Lo que puede hacer avanzar un encargo.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hecho {
    Notas(u32),
    Combo(u32),
    Cometa,
    Aterrizo(usize),
    Barrel,
    Final,
}

/// Los planetas con escena (para los encargos de aterrizar).
const CON_ESCENA: [usize; 13] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

/// Las estelas de la nave: id, nombre y precio en notas.
pub const ESTELAS: [(&str, &str, u32); 16] = [
    ("miku", "Audio de puerro", 40),
    ("fuego", "Dejen que cocine", 60),
    ("arcoiris", "RGB da más FPS", 90),
    ("notas", "Mi audio de 7 minutos", 120),
    ("corazones", "Te vi en línea", 150),
    ("aurora", "Entrada con aura", 80),
    ("ion", "Llegando (mentira)", 110),
    ("estelar", "Glitter del caos", 140),
    ("humo", "Camioneta extraurbana", 30),
    ("burbujas", "Hacer burbujas", 50),
    ("pixel", "Modo 8 bits", 70),
    ("confeti", "Fiesta en la nave", 100),
    ("adn", "ADN de campeón", 130),
    ("seisiete", "Six seven", 150),
    ("astrofago", "Astrófago (no lo toque)", 140),
    ("teseracto", "Librero de 5 dimensiones", 180),
];

/// Lo que paga el pase si ya se tenia la skin del premio (de partidas de antes de que fuera exclusiva).
const NOTAS_POR_SKIN_REPETIDA: u32 = 100;

/// Un premio del pase espacial.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Premio {
    Personaje(&'static str),
    Skin(&'static str),
    Estela(&'static str),
    Notas(u32),
    /// Un titulo para el piloto: sale en el HUD, debajo del lugar (se equipa en el pase).
    Titulo(&'static str),
}

/// Los titulos del pase: id, lo que dice y de donde sale (para la vitrina).
pub const TITULOS: [(&str, &str, &str); 8] = [
    ("novato", "Piloto sin licencia", "Pasó el examen de manejo espacial a la tercera. Le regalaron el título por lástima."),
    ("npc", "NPC con aura", "Sigue el mismo camino todos los días y dice las mismas tres frases. Pero con aura."),
    ("delulu", "Delulu galáctico", "Está convencido de que Plutón es planeta y de que Saturno le va a escribir. Delulu es la solución."),
    ("aura", "Aura farmer", "Hace barrel rolls frente a los asteroides aunque nadie lo esté viendo. +1000 de aura cada vez."),
    ("main", "Main character", "La cámara siempre lo sigue a él. Los demás pilotos son extras. Ellos todavía no lo saben."),
    ("sigma", "Sigma del espacio", "No sigue las reglas del sistema solar. Tampoco las de tránsito. GLADAS ya se rindió."),
    ("rizz", "Rizz interestelar", "Le consiguió cita a Curiosity, le guiñó el ojo a Saturno y Neptuno le dejó ganar en cartas."),
    ("goat", "El GOAT del sistema solar", "El mejor de todos los tiempos. Los planetas votaron. Plutón votó dos veces."),
];

/// Lo que dice el titulo con ese id.
pub fn texto_titulo(id: &str) -> &'static str {
    TITULOS.iter().find(|t| t.0 == id).map_or("", |t| t.1)
}

/// El pase espacial, TEMPORADA 1 · ODISEA BRAINROT: el premio de cada nivel (el nivel 1 es
/// `PASE[0]`). Como en Fortnite, se sube jugando: la XP sale de las notas recogidas, los logros,
/// los sellos, los encargos y los cometas. Cada pocos niveles toca un titulo; el gran premio, en
/// el 40, es un piloto que no se vende: Chimpanzini Bananini.
pub const NOMBRE_TEMPORADA: &str = "TEMPORADA 1 · ODISEA BRAINROT";
pub const PASE: [Premio; 40] = [
    Premio::Personaje("simi"),
    Premio::Titulo("novato"),
    Premio::Personaje("shrek"),
    Premio::Skin("chapina"),
    Premio::Personaje("steve"),
    Premio::Notas(40),
    Premio::Personaje("chilindrina"),
    Premio::Estela("fuego"),
    Premio::Personaje("doge"),
    Premio::Titulo("npc"),
    Premio::Personaje("mario"),
    Premio::Skin("camioneta"),
    Premio::Personaje("patricio"),
    Premio::Estela("arcoiris"),
    Premio::Personaje("creeper"),
    Premio::Notas(60),
    Premio::Personaje("kirby"),
    Premio::Skin("shuco"),
    Premio::Personaje("gato"),
    Premio::Titulo("delulu"),
    Premio::Personaje("pikachu"),
    Premio::Skin("quetzal"),
    Premio::Personaje("chapulin"),
    Premio::Estela("corazones"),
    Premio::Personaje("knuckles"),
    Premio::Notas(100),
    Premio::Skin("sombra"),
    Premio::Titulo("aura"),
    Premio::Estela("notas"),
    Premio::Skin("oro"),
    Premio::Notas(120),
    Premio::Titulo("main"),
    Premio::Estela("seisiete"),
    Premio::Titulo("sigma"),
    Premio::Skin("gamer"),
    Premio::Notas(150),
    Premio::Titulo("rizz"),
    Premio::Titulo("goat"),
    Premio::Notas(200),
    Premio::Personaje("bananini"),
];

/// La XP que pide el nivel `n` (para pasar de n-1 a n): cada nivel cuesta un poco mas.
pub fn xp_del_nivel(n: u32) -> u32 {
    80 + 20 * n
}

/// El nombre de un premio, para la pantalla del pase y los avisos.
pub fn nombre_del_premio(p: Premio) -> String {
    match p {
        Premio::Personaje(id) => crate::personajes::PERSONAJES[crate::personajes::indice(id)].nombre.to_string(),
        Premio::Skin(id) => crate::tienda::SKINS[crate::tienda::indice_skin(id)].nombre.to_string(),
        Premio::Estela(id) => ESTELAS.iter().find(|e| e.0 == id).map_or(id, |e| e.1).to_string(),
        Premio::Notas(n) => format!("{n} notas"),
        Premio::Titulo(id) => texto_titulo(id).to_string(),
    }
}

/// Los cuerpos que tienen sello en el pasaporte (todos menos el Sol).
pub const CON_SELLO: [usize; 13] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

pub struct Progreso {
    pub logros: BTreeSet<String>,
    /// Las notas que se tienen (se gastan en estelas) y las que se han recogido en total (para los logros).
    pub notas: u32,
    pub notas_totales: u32,
    pub cometas: u32,
    /// Cuantos datos de Bill Nye se han escuchado (van en orden: asi no se repiten).
    pub datos: u32,
    /// El record de Jupiter Asteroids.
    pub record_jupiter: u32,
    /// La mochila: las lunas de Galileo que se trajeron de Jupiter (Io, Europa, Ganimedes, Calisto).
    /// En el espacio se usan: turbo, escudo, iman y cometa.
    pub mochila: [u32; 4],
    /// Las cartas del tarot de Madame Mercurio que ya se vieron (un bit por carta de `BARAJA`).
    pub cartas_tarot: u32,
    pub encargos: Vec<Mision>,
    pub encargos_cumplidos: u32,
    pub estelas: BTreeSet<String>,
    pub estela: Option<String>,
    /// Las skins de la nave que se compraron en la tienda, y la que se lleva puesta.
    pub skins: BTreeSet<String>,
    pub skin: String,
    /// El titulo que se lleva puesto (de los del pase), si hay.
    pub titulo: Option<String>,
    /// El personaje escogido (id de `personajes::PERSONAJES`) y los que se compraron en la tienda.
    pub personaje: String,
    pub personajes: BTreeSet<String>,
    /// Hasta que nivel del pase ya se entregaron los premios.
    pub pase_cobrado: u32,
    /// Hasta que nivel ya se vio el pase y el casillero (para el puntito de "nuevo" en su pestana).
    pub pase_visto: u32,
    pub personajes_visto: u32,
    pub ajustes: Ajustes,
    semilla: u32,
    ruta: Option<PathBuf>,
    /// Lo recien desbloqueado, para el aviso en pantalla: (texto, segundos que lleva).
    pub avisos: Vec<(String, f32)>,
}

impl Progreso {
    /// Lee la partida guardada (o empieza de cero si no hay).
    pub fn cargar() -> Self {
        Self::desde(Some(PathBuf::from(ARCHIVO)))
    }

    /// Sin archivo: para las capturas y las pruebas.
    pub fn en_memoria() -> Self {
        Self::desde(None)
    }

    fn desde(ruta: Option<PathBuf>) -> Self {
        let semilla = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(7, |d| d.subsec_nanos()) | 1;
        let mut p = Progreso {
            logros: BTreeSet::new(),
            notas: 0,
            notas_totales: 0,
            cometas: 0,
            datos: 0,
            record_jupiter: 0,
            cartas_tarot: 0,
            mochila: [0; 4],
            encargos: Vec::new(),
            encargos_cumplidos: 0,
            estelas: BTreeSet::new(),
            estela: None,
            skins: BTreeSet::new(),
            skin: crate::tienda::SKINS[0].id.to_string(),
            personaje: crate::personajes::GRATIS[0].to_string(),
            personajes: BTreeSet::new(),
            titulo: None,
            pase_cobrado: 0,
            pase_visto: 0,
            personajes_visto: 0,
            ajustes: Ajustes::default(),
            semilla,
            ruta,
            avisos: Vec::new(),
        };
        let mut con_totales = false;
        if let Some(texto) = p.ruta.as_ref().and_then(|r| std::fs::read_to_string(r).ok()) {
            for linea in texto.lines() {
                let partes: Vec<&str> = linea.split_whitespace().collect();
                let num = |k: usize| partes.get(k).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
                match partes.first().copied() {
                    Some("logro") if partes.len() > 1 => {
                        p.logros.insert(partes[1].to_string());
                    }
                    Some("notas") => p.notas = num(1),
                    Some("notas_totales") => {
                        p.notas_totales = num(1);
                        con_totales = true;
                    }
                    Some("cometas") => p.cometas = num(1),
                    Some("datos") => p.datos = num(1),
                    Some("record_jupiter") => p.record_jupiter = num(1),
                    Some("cartas_tarot") => p.cartas_tarot = num(1),
                    Some("mochila") => p.mochila = [num(1), num(2), num(3), num(4)],
                    Some("encargos_cumplidos") => p.encargos_cumplidos = num(1),
                    Some("estela_tiene") if partes.len() > 1 => {
                        p.estelas.insert(partes[1].to_string());
                    }
                    Some("estela_puesta") if partes.len() > 1 => p.estela = Some(partes[1].to_string()),
                    Some("skin_tiene") if partes.len() > 1 => {
                        p.skins.insert(partes[1].to_string());
                    }
                    Some("skin_puesta") if partes.len() > 1 => p.skin = partes[1].to_string(),
                    Some("personaje_tiene") if partes.len() > 1 => {
                        p.personajes.insert(partes[1].to_string());
                    }
                    Some("personaje") if partes.len() > 1 => p.personaje = partes[1].to_string(),
                    Some("pase_cobrado") => p.pase_cobrado = num(1),
                    Some("titulo") if partes.len() > 1 => p.titulo = Some(partes[1].to_string()),
                    Some("pase_visto") => p.pase_visto = num(1),
                    Some("personajes_visto") => p.personajes_visto = num(1),
                    Some("volumen") => p.ajustes.volumen = num(1).min(4) as u8,
                    Some("calidad_rapida") => p.ajustes.calidad_rapida = num(1) == 1,
                    Some("invertir") => p.ajustes.invertir = num(1) == 1,
                    Some("raton") => p.ajustes.raton = num(1) == 1,
                    Some("musica") => p.ajustes.musica = num(1).min(4) as u8,
                    Some("efectos") => p.ajustes.efectos = num(1).min(4) as u8,
                    Some("sensibilidad") => p.ajustes.sensibilidad = num(1).clamp(1, 5) as u8,
                    Some("texto") => p.ajustes.texto = num(1).min(3) as u8,
                    Some("vibracion") => p.ajustes.vibracion = num(1) == 1,
                    Some("dibujado") => p.ajustes.dibujado = num(1) == 1,
                    Some("encargo") if partes.len() >= 5 => {
                        let tipo = match partes[1] {
                            "notas" => Some(Encargo::Notas),
                            "combo" => Some(Encargo::Combo),
                            "cometa" => Some(Encargo::Cometa),
                            "aterrizar" => Some(Encargo::Aterrizar((num(2) as usize).min(CUERPOS.len() - 1))),
                            "barrels" => Some(Encargo::Barrels),
                            "final" => Some(Encargo::Final),
                            _ => None,
                        };
                        if let Some(tipo) = tipo {
                            p.encargos.push(Mision { tipo, meta: num(3).max(1), avance: num(4) });
                        }
                    }
                    _ => {}
                }
            }
        }
        // Las partidas viejas no tenian el total: es lo que se tenia.
        if !con_totales {
            p.notas_totales = p.notas;
        }
        while p.encargos.len() < 3 {
            let m = p.nuevo_encargo();
            p.encargos.push(m);
        }
        p
    }

    fn azar(&mut self, n: u32) -> u32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) % n
    }

    /// Un encargo nuevo, de un tipo distinto a los que ya estan.
    fn nuevo_encargo(&mut self) -> Mision {
        for _ in 0..20 {
            let tipo = match self.azar(6) {
                0 => Encargo::Notas,
                1 => Encargo::Combo,
                2 => Encargo::Cometa,
                3 => Encargo::Aterrizar(CON_ESCENA[self.azar(CON_ESCENA.len() as u32) as usize]),
                4 => Encargo::Barrels,
                _ => Encargo::Final,
            };
            let mismo = |m: &Mision| std::mem::discriminant(&m.tipo) == std::mem::discriminant(&tipo);
            if self.encargos.iter().any(mismo) {
                continue;
            }
            let meta = match tipo {
                Encargo::Notas => 10 + 5 * self.azar(5),
                Encargo::Combo => 5 + self.azar(8),
                Encargo::Barrels => 3 + self.azar(3),
                _ => 1,
            };
            return Mision { tipo, meta, avance: 0 };
        }
        Mision { tipo: Encargo::Notas, meta: 15, avance: 0 }
    }

    /// Algo paso: avanzan los encargos que tienen que ver. Los cumplidos pagan y se cambian por otros.
    pub fn hecho(&mut self, h: Hecho) {
        let mut cumplidos = Vec::new();
        for (k, m) in self.encargos.iter_mut().enumerate() {
            let avanza = match (h, m.tipo) {
                (Hecho::Notas(n), Encargo::Notas) => m.avance + n,
                (Hecho::Combo(c), Encargo::Combo) => m.avance.max(c),
                (Hecho::Cometa, Encargo::Cometa) | (Hecho::Barrel, Encargo::Barrels) | (Hecho::Final, Encargo::Final) => m.avance + 1,
                (Hecho::Aterrizo(p), Encargo::Aterrizar(q)) if p == q => m.avance + 1,
                _ => continue,
            };
            m.avance = avanza.min(m.meta);
            if m.avance >= m.meta {
                cumplidos.push(k);
            }
        }
        for k in cumplidos.into_iter().rev() {
            let m = self.encargos.remove(k);
            let paga = m.recompensa();
            self.notas += paga;
            self.encargos_cumplidos += 1;
            self.avisos.push((format!("¡ENCARGO CUMPLIDO! +{paga} notas"), 0.0));
            let nuevo = self.nuevo_encargo();
            self.encargos.insert(k, nuevo);
            if self.encargos_cumplidos >= 5 {
                self.desbloquear("encargos_5");
            }
            if self.encargos_cumplidos >= 20 {
                self.desbloquear("encargos_20");
            }
        }
        self.guardar();
    }

    /// Compra la estela (si alcanzan las notas) o, si ya se tiene, se la pone (o se la quita).
    /// Devuelve si se pudo.
    pub fn estela_escogida(&mut self, id: &str) -> bool {
        if self.estelas.contains(id) {
            self.estela = if self.estela.as_deref() == Some(id) { None } else { Some(id.to_string()) };
            self.guardar();
            return true;
        }
        if PASE.iter().any(|p| matches!(p, Premio::Estela(premio) if *premio == id)) { return false; }
        let Some(&(_, _, precio)) = ESTELAS.iter().find(|e| e.0 == id) else { return false };
        if self.notas < precio {
            return false;
        }
        self.notas -= precio;
        self.estelas.insert(id.to_string());
        self.estela = Some(id.to_string());
        self.desbloquear("estela");
        self.guardar();
        true
    }

    /// El siguiente dato de Bill Nye (en orden, y vuelve a empezar al acabarse).
    pub fn siguiente_dato(&mut self) -> &'static str {
        let dato = crate::asteroides::DATOS[self.datos as usize % crate::asteroides::DATOS.len()];
        self.datos += 1;
        if self.datos >= 5 {
            self.desbloquear("ciencia_5");
        }
        if self.datos >= crate::asteroides::DATOS.len() as u32 {
            self.desbloquear("ciencia_todo");
        }
        self.guardar();
        dato
    }

    /// Se cayeron notas (un asteroide caliente): se pierden de las que se tienen, no de las totales.
    pub fn perder_notas(&mut self, n: u32) {
        self.notas = self.notas.saturating_sub(n);
        self.desbloquear("asteroide_rojo");
        self.guardar();
    }

    /// La XP del pase: sale de todo lo que se ha hecho (asi una partida vieja ya trae su nivel).
    pub fn xp(&self) -> u32 {
        let sellos = self.logros.iter().filter(|l| l.starts_with("sello_")).count() as u32;
        let logros = self.logros.len() as u32 - sellos;
        self.notas_totales * 5 + logros * 40 + sellos * 120 + self.encargos_cumplidos * 60 + self.cometas * 80
    }

    /// El nivel del pase, la XP que lleva dentro de ese nivel y la que pide el siguiente.
    pub fn nivel_del_pase(&self) -> (u32, u32, u32) {
        let mut resto = self.xp();
        let mut nivel = 0;
        while nivel < PASE.len() as u32 && resto >= xp_del_nivel(nivel + 1) {
            resto -= xp_del_nivel(nivel + 1);
            nivel += 1;
        }
        (nivel, resto, xp_del_nivel(nivel + 1))
    }

    /// Si el personaje ya se puede escoger (de fabrica, comprado, por logro o por el pase).
    pub fn tiene_personaje(&self, id: &str) -> bool {
        if crate::personajes::GRATIS.contains(&id) || self.personajes.contains(id) {
            return true;
        }
        if let Some(logro) = crate::personajes::por_logro(id) {
            return self.tiene(logro);
        }
        let nivel = self.nivel_del_pase().0 as usize;
        PASE[..nivel].iter().any(|p| matches!(p, Premio::Personaje(x) if *x == id))
    }

    /// El nivel del pase en que sale el personaje (0 si es gratis).
    pub fn nivel_del_personaje(id: &str) -> u32 {
        PASE.iter().position(|p| matches!(p, Premio::Personaje(x) if *x == id)).map_or(0, |i| i as u32 + 1)
    }

    /// Entrega los premios de los niveles nuevos del pase (con su aviso). Las skins y estelas
    /// que ya se tenian se cambian por notas.
    pub fn cobrar_pase(&mut self) {
        let nivel = self.nivel_del_pase().0;
        if nivel <= self.pase_cobrado {
            return;
        }
        for n in self.pase_cobrado + 1..=nivel {
            let premio = PASE[n as usize - 1];
            let extra = match premio {
                Premio::Notas(x) => {
                    self.notas += x;
                    String::new()
                }
                Premio::Skin(id) if self.skins.contains(id) => {
                    // Las del pase ya no tienen precio; las partidas viejas pudieron comprarlas.
                    let x = NOTAS_POR_SKIN_REPETIDA;
                    self.notas += x;
                    format!(" (ya la tenía: +{x} notas)")
                }
                Premio::Skin(id) => {
                    self.skins.insert(id.to_string());
                    String::new()
                }
                Premio::Estela(id) if self.estelas.contains(id) => {
                    let x = ESTELAS.iter().find(|e| e.0 == id).map_or(0, |e| e.2) / 2;
                    self.notas += x;
                    format!(" (ya la tenía: +{x} notas)")
                }
                Premio::Estela(id) => {
                    self.estelas.insert(id.to_string());
                    String::new()
                }
                Premio::Personaje(_) | Premio::Titulo(_) => String::new(),
            };
            self.avisos.push((format!("¡NIVEL {n} DEL PASE! {}{extra} · véalo en el menú", nombre_del_premio(premio)), 0.0));
        }
        self.pase_cobrado = nivel;
        self.guardar();
    }

    /// Los puntitos de "nuevo" de las pestanas: (personajes, pase, tienda). Personajes: hay uno
    /// desbloqueado que no se ha visto; pase: se subio de nivel; tienda: alcanza para algo nuevo.
    pub fn novedades(&self) -> (bool, bool, bool) {
        let nivel = self.nivel_del_pase().0;
        let visto = self.personajes_visto.min(nivel) as usize;
        let personajes = PASE[visto..nivel as usize].iter().any(|p| matches!(p, Premio::Personaje(_)));
        let tienda = crate::tienda::skins_tienda().iter().any(|&i| { let k = &crate::tienda::SKINS[i]; !self.tiene_skin(k.id) && k.precio <= self.notas })
            || crate::tienda::estelas_tienda().iter().any(|&i| { let e = &ESTELAS[i]; !self.estelas.contains(e.0) && e.2 <= self.notas })
            || crate::tienda::personajes_tienda().iter().any(|&i| {
                let id = crate::personajes::PERSONAJES[i].id;
                !self.tiene_personaje(id) && crate::personajes::precio(id).is_some_and(|x| x <= self.notas)
            });
        (personajes, nivel > self.pase_visto, tienda)
    }

    /// Si la skin ya es suya (la de fabrica siempre lo es).
    pub fn tiene_skin(&self, id: &str) -> bool {
        id == crate::tienda::SKINS[0].id || self.skins.contains(id)
    }

    /// Compra el personaje en la tienda (si alcanzan las notas) o, si ya se tiene, se lo pone.
    /// Devuelve si se pudo.
    pub fn personaje_escogido(&mut self, id: &str) -> bool {
        if !self.tiene_personaje(id) {
            // Los del pase no se venden: se ganan subiendo de nivel.
            let Some(precio) = crate::personajes::precio(id) else { return false };
            if self.notas < precio {
                return false;
            }
            self.notas -= precio;
            self.personajes.insert(id.to_string());
        }
        self.personaje = id.to_string();
        self.guardar();
        true
    }

    /// Compra la skin (si alcanzan las notas) o, si ya se tiene, se la pone. Devuelve si se pudo.
    pub fn skin_escogida(&mut self, id: &str) -> bool {
        if !self.tiene_skin(id) {
            if PASE.iter().any(|p| matches!(p, Premio::Skin(premio) if *premio == id)) { return false; }
            let Some(precio) = crate::tienda::SKINS.iter().find(|k| k.id == id).map(|k| k.precio) else { return false };
            if self.notas < precio {
                return false;
            }
            self.notas -= precio;
            self.skins.insert(id.to_string());
            self.desbloquear("skin");
        }
        self.skin = id.to_string();
        self.guardar();
        true
    }

    pub fn guardar(&self) {
        if let Some(ruta) = &self.ruta {
            let mut s = String::from("# Space Adventure: el pasaporte (se puede borrar para empezar de cero)\n");
            s += &format!("notas {}\nnotas_totales {}\ncometas {}\nencargos_cumplidos {}\ndatos {}\n", self.notas, self.notas_totales, self.cometas, self.encargos_cumplidos, self.datos);
            s += &format!("record_jupiter {}\nmochila {} {} {} {}\ncartas_tarot {}\n", self.record_jupiter, self.mochila[0], self.mochila[1], self.mochila[2], self.mochila[3], self.cartas_tarot);
            let a = &self.ajustes;
            s += &format!("volumen {}\nmusica {}\nefectos {}\ncalidad_rapida {}\ninvertir {}\nraton {}\n", a.volumen, a.musica, a.efectos, a.calidad_rapida as u8, a.invertir as u8, a.raton as u8);
            s += &format!("sensibilidad {}\ntexto {}\nvibracion {}\ndibujado {}\n", a.sensibilidad, a.texto, a.vibracion as u8, a.dibujado as u8);
            for m in &self.encargos {
                let (tipo, param) = m.codigo();
                s += &format!("encargo {tipo} {param} {} {}\n", m.meta, m.avance);
            }
            for e in &self.estelas {
                s += &format!("estela_tiene {e}\n");
            }
            if let Some(e) = &self.estela {
                s += &format!("estela_puesta {e}\n");
            }
            for k in &self.skins {
                s += &format!("skin_tiene {k}\n");
            }
            for k in &self.personajes {
                s += &format!("personaje_tiene {k}\n");
            }
            s += &format!("skin_puesta {}\n", self.skin);
            if let Some(t) = &self.titulo {
                s += &format!("titulo {t}\n");
            }
            s += &format!("personaje {}\npase_cobrado {}\n", self.personaje, self.pase_cobrado);
            s += &format!("pase_visto {}\npersonajes_visto {}\n", self.pase_visto, self.personajes_visto);
            for id in &self.logros {
                s += &format!("logro {id}\n");
            }
            let _ = std::fs::write(ruta, s);
        }
    }

    pub fn tiene(&self, id: &str) -> bool {
        self.logros.contains(id)
    }

    /// Desbloquea un logro (o un sello, con id `sello_<archivo>`). Devuelve si es nuevo.
    pub fn desbloquear(&mut self, id: &str) -> bool {
        if self.logros.contains(id) {
            return false;
        }
        self.logros.insert(id.to_string());
        let texto = if let Some(archivo) = id.strip_prefix("sello_") {
            let nombre = CUERPOS.iter().find(|c| c.archivo == archivo).map_or(archivo, |c| c.nombre);
            format!("¡SELLO NUEVO! {nombre}")
        } else {
            let nombre = LOGROS.iter().find(|l| l.id == id).map_or(id, |l| l.nombre);
            format!("¡LOGRO! {nombre}")
        };
        self.avisos.push((texto, 0.0));
        // Si el logro gana un personaje secreto, que se sepa (en el casillero no salia).
        for f in crate::personajes::PERSONAJES.iter().filter(|f| crate::personajes::por_logro(f.id) == Some(id)) {
            self.avisos.push((format!("¡PERSONAJE SECRETO! {} ya se puede escoger", f.nombre), 0.0));
        }
        // Un final o logro nuevo avanza el encargo de "final nuevo".
        if !id.starts_with("sello_") && !NO_CUENTAN.contains(&id) {
            self.hecho(Hecho::Final);
        }
        // Con los doce sellos: el pasaporte lleno.
        if CON_SELLO.iter().all(|&i| self.logros.contains(&sello(i))) && !self.logros.contains("todos_los_sellos") {
            self.logros.insert("todos_los_sellos".into());
            self.avisos.push(("¡LOGRO! Pasaporte lleno".into(), 0.0));
        }
        self.guardar();
        true
    }

    /// Un pago (al despegar de un minijuego): suma las notas y lo avisa con su motivo.
    pub fn cobrar(&mut self, n: u32, motivo: &str) {
        if n == 0 {
            return;
        }
        self.sumar_notas(n);
        self.avisos.push((format!("+{n} NOTAS · {motivo}"), 0.0));
    }

    /// Suma notas recogidas, y los logros por cantidad.
    pub fn sumar_notas(&mut self, n: u32) {
        self.notas += n;
        self.notas_totales += n;
        for (meta, id) in [(10, "notas_10"), (100, "notas_100"), (500, "notas_500")] {
            if self.notas_totales >= meta {
                self.desbloquear(id);
            }
        }
        self.hecho(Hecho::Notas(n));
        self.guardar();
    }

    pub fn sumar_cometa(&mut self) {
        self.cometas += 1;
        self.hecho(Hecho::Cometa);
        self.desbloquear("cometa");
        if self.cometas >= 3 {
            self.desbloquear("cometa_3");
        }
        self.guardar();
    }

    /// Cuantos logros y sellos hay, y cuantos se tienen.
    pub fn conteo(&self) -> (usize, usize) {
        let total = LOGROS.len() + CON_SELLO.len();
        let tiene = LOGROS.iter().filter(|l| self.tiene(l.id)).count() + CON_SELLO.iter().filter(|&&i| self.tiene(&sello(i))).count();
        (tiene, total)
    }

    /// Avanza los avisos en pantalla: se muestran de a uno, cada uno unos segundos.
    pub fn actualizar(&mut self, dt: f32) {
        if let Some(a) = self.avisos.first_mut() {
            a.1 += dt;
            if a.1 > AVISO {
                self.avisos.remove(0);
            }
        }
    }
}

/// El id del sello del cuerpo `i`.
pub fn sello(i: usize) -> String {
    format!("sello_{}", CUERPOS[i].archivo)
}

/// Segundos que dura cada aviso.
const AVISO: f32 = 3.2;

/// El color de cada sello (el de su planeta).
fn color_sello(i: usize) -> Color {
    match CUERPOS[i].archivo {
        "mercurio" => Color::new(200, 160, 120, 255),
        "venus" => Color::new(240, 200, 100, 255),
        "tierra" => Color::new(90, 160, 255, 255),
        "luna" => Color::new(210, 210, 225, 255),
        "marte" => Color::new(240, 110, 70, 255),
        "jupiter" => Color::new(230, 170, 120, 255),
        "saturno_ritmo" => Color::new(255, 120, 210, 255),
        "urano" => Color::new(120, 230, 240, 255),
        "neptuno" => Color::new(90, 120, 255, 255),
        "pluton" => Color::new(240, 200, 170, 255),
        "planeta_vegetta" => Color::new(170, 110, 240, 255),
        _ => Color::new(255, 140, 230, 255),
    }
}

/// El aviso de "¡SELLO NUEVO!" que baja de arriba a la derecha.
pub fn dibujar_avisos(d: &mut RaylibDrawHandle, p: &Progreso, ancho: i32) {
    if let Some((texto, t)) = p.avisos.first() {
        let entra = (t / 0.25).min(1.0);
        let sale = ((AVISO - t) / 0.3).clamp(0.0, 1.0);
        let k = entra.min(sale);
        let w = ui::medir(texto, 22) + 70;
        let x = ancho - w - 20;
        let y = (-80.0 + 190.0 * k) as i32;
        ui::panel(d, x, y, w, 56);
        d.draw_circle(x + 30, y + 28, 16.0, Color::new(255, 200, 90, 255));
        d.draw_circle(x + 30, y + 28, 11.0, Color::new(200, 90, 160, 255));
        ui::texto(d, "*", x + 25, y + 16, 26, Color::WHITE);
        ui::texto(d, texto, x + 56, y + 16, 22, Color::new(255, 230, 150, 255));
    }
}

/// La pantalla del pasaporte: los doce sellos y, del planeta escogido, sus finales y logros.
/// `sellos`: la imagen 3D de cada cuerpo de `CON_SELLO` (y al final, la nave, para "el espacio").
#[allow(clippy::too_many_arguments)]
pub fn dibujar_pasaporte(d: &mut RaylibDrawHandle, p: &Progreso, escogido: usize, ancho: i32, alto: i32, tecla: &str, sellos: &[Texture2D]) {
    d.draw_rectangle(0, 0, ancho, alto, Color::new(8, 4, 20, 210));
    let (w, h) = ((ancho - 80).min(1100), (alto - 80).min(640));
    let (x0, y0) = ((ancho - w) / 2, (alto - h) / 2);
    ui::panel(d, x0, y0, w, h);
    let (tiene, total) = p.conteo();
    ui::titulo_de(d, "PASAPORTE ESPACIAL", x0 + 28, y0 + 22, 30, Color::new(255, 220, 150, 255));
    ui::divisor(d, (x0 + 26) as f32, (y0 + 88) as f32, (w - 52) as f32, 2.0, Color::new(185, 140, 255, 200));
    let resumen = format!("{tiene} de {total}   ·   {} notas   ·   {} cometas", p.notas, p.cometas);
    ui::texto(d, &resumen, x0 + 28, y0 + 62, 20, ui::TENUE);
    let ayuda = format!("{}: planeta      {}: cerrar", ui::boton("\u{E00A} \u{E00B}"), ui::boton(tecla));
    ui::texto(d, &ayuda, x0 + w - ui::medir(&ayuda, 18) - 28, y0 + 30, 18, ui::TENUE);

    // Los sellos: estampas con el planeta adentro (en 3D, pintado por el rasterizador) y su nombre
    // en una cinta. Los que faltan, oscuros: se adivina la silueta.
    let columnas = 3;
    let total_sellos = CON_SELLO.len() + 1;
    let filas = total_sellos.div_ceil(columnas) as i32;
    let paso = ((h - 150) / filas).min(118);
    // El radio deja lugar abajo para la cinta con el nombre.
    let r = (paso as f32 * 0.36).min(42.0);
    let fondo_panel = Color::new(20, 12, 42, 255);
    for (n, &i) in CON_SELLO.iter().chain(std::iter::once(&0)).enumerate() {
        let (c, f) = (n % columnas, n / columnas);
        let (cx, cy) = ((x0 + 70) as f32 + c as f32 * (r * 2.0 + 24.0), (y0 + 112) as f32 + r + 8.0 + f as f32 * paso as f32);
        let es_espacio = n == CON_SELLO.len();
        let puesto = es_espacio || p.tiene(&sello(i));
        let color = if es_espacio { Color::new(170, 140, 255, 255) } else if puesto { color_sello(i) } else { Color::new(60, 55, 82, 255) };
        if n == escogido {
            d.draw_circle_v(Vector2::new(cx, cy), r + 6.0, Color::new(255, 230, 150, 255));
        }
        // La estampa: un circulo con la orilla dentada (como timbre de correo).
        d.draw_circle_v(Vector2::new(cx, cy), r, color);
        let dientes = 18;
        for k in 0..dientes {
            let a = k as f32 / dientes as f32 * std::f32::consts::TAU;
            d.draw_circle_v(Vector2::new(cx + a.cos() * r, cy + a.sin() * r), r * 0.09, if n == escogido { Color::new(255, 230, 150, 255) } else { fondo_panel });
        }
        d.draw_circle_lines(cx as i32, cy as i32, r * 0.80, Color::new(255, 255, 255, if puesto { 120 } else { 30 }));
        // El planeta adentro.
        if let Some(t) = sellos.get(n) {
            let lado = r * 1.45;
            let tinte = if puesto { Color::WHITE } else { Color::new(80, 72, 115, 255) };
            let fuente = Rectangle::new(0.0, 0.0, t.width as f32, t.height as f32);
            d.draw_texture_pro(t, fuente, Rectangle::new(cx - lado / 2.0, cy - lado / 2.0, lado, lado), Vector2::zero(), 0.0, tinte);
        }
        // La cinta con el nombre, abajo.
        let nombre = if es_espacio { "ESPACIO".to_string() } else if puesto { CUERPOS[i].nombre.to_uppercase() } else { "???".to_string() };
        let mut tam = 13;
        while tam > 9 && ui::medir(&nombre, tam) as f32 > r * 2.0 - 6.0 {
            tam -= 1;
        }
        let wc = ui::medir(&nombre, tam) as f32 + 12.0;
        // Abajo del sello (no encima del planeta).
        let yc = cy + r + 4.0;
        d.draw_rectangle_rounded(Rectangle::new(cx - wc / 2.0, yc, wc, tam as f32 + 6.0), 0.4, 6, if puesto { Color::new(30, 15, 40, 230) } else { Color::new(30, 26, 48, 230) });
        ui::texto(d, &nombre, (cx - wc / 2.0 + 6.0) as i32, (yc + 3.0) as i32, tam, if puesto { Color::new(255, 235, 200, 255) } else { ui::TENUE });
    }

    let lado = (r * 2.0 + 10.0) as i32;
    // Los logros del escogido, a la derecha.
    let planeta = CON_SELLO.get(escogido).copied();
    let xl = x0 + 70 + columnas as i32 * (lado + 14) + 10;
    let titulo = planeta.map_or("EL ESPACIO".to_string(), |i| CUERPOS[i].nombre.to_uppercase());
    ui::texto(d, &titulo, xl, y0 + 110, 28, ui::TEXTO);
    let lista: Vec<&Logro> = LOGROS.iter().filter(|l| l.planeta == planeta).collect();
    if lista.is_empty() {
        ui::texto(d, "Aquí todavía no hay escena. Solo el sello.", xl, y0 + 160, 20, ui::TENUE);
        ui::texto(d, "(Por ahora.)", xl, y0 + 188, 20, ui::TENUE);
    }
    // Si la lista no cabe en una columna (el espacio tiene muchos), se parte en dos y, si aun asi
    // no cabe, con letra mas chica; y al final en modo compacto: un renglon por logro (la pista
    // sola, o "nombre: pista" si ya se tiene). Nunca se sale del panel: lo que no cabe se cuenta.
    let ancho_lista = x0 + w - xl - 28;
    let (arriba, abajo) = (y0 + 154, y0 + h - 18);
    let texto_compacto = |l: &Logro| if p.tiene(l.id) { format!("{}: {}", l.nombre, l.pista) } else { l.pista.to_string() };
    // (columnas, letra del nombre, letra de la pista, hueco, compacto)
    let opciones = [(1, 22, 16, 10, false), (2, 22, 16, 10, false), (2, 20, 15, 6, false), (2, 18, 14, 4, false), (2, 0, 15, 8, true), (2, 0, 13, 5, true)];
    let acomodo = |&(cols, tn, tp, hueco, compacto): &(i32, i32, i32, i32, bool)| {
        let ancho_col = (ancho_lista - (cols - 1) * 24) / cols;
        let mut puestos = Vec::new();
        let (mut col, mut y) = (0, arriba);
        for l in &lista {
            let renglones = if compacto { ui::envolver(&texto_compacto(l), tp, ancho_col - 32).len() } else { ui::envolver(l.pista, tp, ancho_col - 32).len() } as i32;
            let alto = if compacto { 0 } else { tn + 6 } + renglones * (tp + tp / 3) + hueco;
            if y + alto > abajo && y > arriba {
                col += 1;
                y = arriba;
            }
            puestos.push((col, xl + col * (ancho_col + 24), y));
            y += alto;
        }
        (col < cols, ancho_col, puestos)
    };
    let escogida = opciones.iter().find(|o| acomodo(o).0).unwrap_or(&opciones[opciones.len() - 1]);
    let (_, ancho_col, puestos) = acomodo(escogida);
    let (cols, tn, tp, _, compacto) = *escogida;
    let mut no_caben = 0;
    for (l, &(col, x, y)) in lista.iter().zip(&puestos) {
        if col >= cols {
            no_caben += 1;
            continue;
        }
        let lo_tiene = p.tiene(l.id);
        // La casilla: un cuadrito, con palomita si ya se tiene.
        let alto_linea = if compacto { tp } else { tn };
        let c = if compacto { 15.0 } else { 18.0 };
        let (xf, yf) = (x as f32, y as f32 + (alto_linea as f32 - c) / 2.0 + 1.0);
        d.draw_rectangle_lines_ex(Rectangle::new(xf, yf, c, c), 2.0, if lo_tiene { ui::VERDE } else { ui::TENUE });
        // La palomita del pack (sale un poquito de la casilla, como hecha a mano).
        if lo_tiene && !ui::palomita(d, xf - 1.0, yf - c * 0.3, c * 1.15, true) {
            d.draw_line_ex(Vector2::new(xf + c * 0.22, yf + c * 0.5), Vector2::new(xf + c * 0.44, yf + c * 0.78), 3.0, ui::VERDE);
            d.draw_line_ex(Vector2::new(xf + c * 0.44, yf + c * 0.78), Vector2::new(xf + c * 0.83, yf + c * 0.17), 3.0, ui::VERDE);
        }
        if compacto {
            ui::parrafo(d, &texto_compacto(l), x + 28, y, tp, ancho_col - 32, if lo_tiene { Color::new(255, 230, 150, 255) } else { ui::TENUE });
        } else {
            let nombre = if lo_tiene { l.nombre } else { "???" };
            ui::texto(d, nombre, x + 32, y, tn, if lo_tiene { Color::new(255, 230, 150, 255) } else { ui::TEXTO });
            ui::parrafo(d, l.pista, x + 32, y + tn + 6, tp, ancho_col - 32, ui::TENUE);
        }
    }
    if no_caben > 0 {
        let s = format!("... y {no_caben} más");
        ui::texto(d, &s, x0 + w - 28 - ui::medir(&s, 15), abajo - 4, 15, ui::TENUE);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// En la tienda se compra un personaje con notas (y si no alcanza, no).
    #[test]
    fn los_personajes_se_compran_en_la_tienda() {
        let mut p = Progreso::en_memoria();
        assert!(!p.tiene_personaje("koch"));
        p.notas = 100;
        assert!(!p.personaje_escogido("koch"), "no alcanza");
        assert_eq!(p.notas, 100);
        p.notas = crate::personajes::precio("koch").unwrap() + 5;
        assert!(p.personaje_escogido("koch"));
        assert!(p.tiene_personaje("koch") && p.personaje == "koch");
        assert_eq!(p.notas, 5);
        // Volverlo a escoger no cobra.
        p.personaje = "chavo".into();
        assert!(p.personaje_escogido("koch"));
        assert_eq!(p.notas, 5);
        // Los del pase son solo del pase: con notas de sobra, Pikachu no se vende.
        p.notas = 5000;
        assert!(!p.personaje_escogido("pikachu"));
        assert!(!p.tiene_personaje("pikachu") && p.notas == 5000);
    }

    /// Lo que se desbloquea se guarda en el archivo y se vuelve a leer igual.
    #[test]
    fn el_pasaporte_se_guarda_y_se_carga() {
        let ruta = std::env::temp_dir().join(format!("space_miku_prueba_{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&ruta);
        let mut p = Progreso::desde(Some(ruta.clone()));
        // Sin encargos (salen al azar y pagarian notas: la cuenta cambiaria).
        p.encargos.clear();
        assert!(p.desbloquear("cita_boda"));
        assert!(!p.desbloquear("cita_boda"), "dos veces no cuenta");
        assert!(p.desbloquear(&sello(3)));
        p.sumar_notas(12);
        let q = Progreso::desde(Some(ruta.clone()));
        assert!(q.tiene("cita_boda") && q.tiene("sello_tierra") && q.tiene("notas_10"));
        assert_eq!(q.notas, 12);
        let _ = std::fs::remove_file(&ruta);
    }

    /// Con los doce sellos sale el logro del pasaporte lleno; los ids no se repiten.
    #[test]
    fn once_sellos_llenan_el_pasaporte() {
        let mut p = Progreso::en_memoria();
        for &i in &CON_SELLO {
            assert!(!p.tiene("todos_los_sellos"));
            p.desbloquear(&sello(i));
        }
        assert!(p.tiene("todos_los_sellos"));
        let mut ids: Vec<&str> = LOGROS.iter().map(|l| l.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), LOGROS.len());
    }
}
