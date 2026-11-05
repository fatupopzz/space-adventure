//! Space Adventure: un sistema solar de juguete que la Nave Lila recorre
//! libremente, dibujado con un rasterizador por software.
//!
//! raylib solo pone la ventana, el teclado, el control y una textura: cada
//! cuadro el pipeline (ver `pipeline.rs`) pinta el framebuffer en memoria y
//! se sube entero a esa textura.
//!
//! Escenas: la intro (Miku baila y sube a la nave), el espacio (volar entre
//! planetas, saltar al hiperespacio) y los aterrizajes: el salon de la UVG
//! en la Tierra y el concierto en Saturno. Entre una y otra la pantalla se
//! funde a negro.

mod adornos;
mod chapin;
#[cfg(haptica_apple)]
mod haptica;
mod demo;
mod ruido;
mod aula;
mod animacion;
mod aterrizaje;
mod asteroides;
mod baile;
mod cancion;
mod casillero;
mod chispas;
mod decorado;
mod personalidad;
mod cielo;
mod cita;
mod divorcio;
mod colision;
mod concierto;
mod entrada;
mod controles;
mod entrega;
mod escena;
mod estelas;
mod framebuffer;
mod historia;
mod fuente;
mod intro;
mod jupiter;
mod junta;
mod camioneta;
mod luna;
mod maravillas;
mod matematica;
mod menu;
mod mochila;
mod mercurio;
mod mirador;
mod nave;
mod notas;
mod neptuno;
mod obj;
mod obra;
mod planetas;
mod pausa;
mod personajes;
mod pipeline;
mod polvo;
mod post;
mod progreso;
mod grabacion;
mod ritmo;
mod salto;
mod sonido;
mod tienda;
mod ui;
mod urano;
mod vegetta;
mod vibracion;

use animacion::{Animaciones, EstadoAnim};
use aterrizaje::Aterrizaje;
use baile::Paso;
use aula::{Aula, DestinoUvg, PuertaUvg};
use entrega::{AccionEntrega, Entrega};
use urano::{AccionUrano, Urano};
use vegetta::{AccionVegetta, Vegetta};
use baile::Baile;
use cielo::Cielo;
use concierto::{AccionConcierto, Concierto, OCULTAR_HAZ};
use entrada::Entrada;
use escena::{CHAPIN, CUERPOS, HADA, JUPITER, LUNA, MARTE, MERCURIO, NEPTUNO, PLUTON, SATURNO, TIERRA, URANO, VEGETTA, VENUS};
use framebuffer::Framebuffer;
use intro::{Intro, OCULTAR_FAN};
use cita::{AccionCita, Cita, Final};
use junta::{AccionJunta, Junta, MESA, OCULTAR_TRIPULANTE};
use neptuno::{AccionNeptuno, Neptuno};
use fuente::{AccionFuente, Fuente};
use mercurio::{AccionMercurio, Mercurio};
use jupiter::{AccionJupiter, Jupiter};
use divorcio::{AccionDivorcio, Divorcio};
use camioneta::{AccionCamioneta, Camioneta};
use luna::{AccionLuna, Luna};
use notas::Coleccionables;
use progreso::{sello, Hecho, Progreso};
use historia::{AccionHistoria, Historia};
use mirador::{AccionMirador, Mirador};
use pausa::{AccionPausa, Pausa};
use casillero::{Casillero, Pase, Submenu};
use tienda::{Muestra, Tienda};
use menu::{AccionMenu, Menu, Opcion};
use matematica::{Mat4, Vec3};
use nave::{Camara, Nave};
use obj::Modelo;
use pipeline::{proyectar, DrawCall, Estadisticas, ModoDibujo, Opciones, Shader, Stencil, Uniforms, Vista};
use polvo::Polvo;
use post::Post;
use raylib::prelude::*;
use ritmo::{Accion, Ritmo};
use salto::Salto;
use sonido::{Dialogo, Efecto, Pista, Sonidos, Voz};

const VENTANA_W: i32 = 1280;
const VENTANA_H: i32 = 720;
/// El framebuffer: tres cuartos de la ventana, agrandado con filtro bilineal.
const RENDER_W: usize = 960;
const RENDER_H: usize = 540;
const FOV: f32 = 60.0;
const CERCA: f32 = 0.1;
const LEJOS: f32 = 2000.0;
/// Desde cuantos pixeles de radio en pantalla se dibuja un planeta con su modelo detallado (LOD).
/// La constante de la gravedad del juego (ver `Juego::actualizar`): en la superficie de un planeta
/// de radio `r` la nave cae con `GRAVEDAD * r` unidades/s².
const GRAVEDAD: f32 = 0.32;
/// La expresion de la nave y de la piloto mientras vuelan normal.
const EXPRESION: &str = "feliz";
/// Fuerza del resplandor del bloom.
const BLOOM: f32 = 0.9;
/// El framebuffer de la grabacion (`--grabar`): mas pixeles que al jugar, para el video en 1080p.
const GRABAR_RENDER: (usize, usize) = (1280, 720);
/// Que tan cerca de la superficie hay que estar para aterrizar (mas un poco segun el tamano del planeta).
const DISTANCIA_ATERRIZAJE: f32 = 10.0;
/// Segundos del fundido a negro entre escenas (de ida y de vuelta).
const FUNDIDO: f32 = 0.45;
/// Lo que no se dibuja de la piloto cuando no va en la nave.
const SIN_NAVE: [&str; 2] = ["volante", "asiento"];
/// El ritmo de todo el sistema solar (Saturno late a este paso).
const BPM: f32 = 120.0;

const LUZ_ESPACIO: (Vec3, Vec3) = (Vec3::new(0.10, 0.08, 0.15), Vec3::new(0.30, 0.20, 0.50));
const LUZ_CONCIERTO: (Vec3, Vec3) = (Vec3::new(0.16, 0.10, 0.24), Vec3::new(0.45, 0.20, 0.55));
/// Atardecer en Marte: ambiente calientito y borde azul (alla los atardeceres son azules).
const LUZ_MARTE: (Vec3, Vec3) = (Vec3::new(0.30, 0.20, 0.20), Vec3::new(0.25, 0.40, 0.75));
/// La Fuente de las Hadas: ambiente lila, borde rosado (de noche, con aurora).
const LUZ_FUENTE: (Vec3, Vec3) = (Vec3::new(0.32, 0.28, 0.42), Vec3::new(0.55, 0.35, 0.75));
/// La carpa de Madame Mercurio: ambiente de velas (calientito) y borde morado.
const LUZ_MERCURIO: (Vec3, Vec3) = (Vec3::new(0.42, 0.33, 0.36), Vec3::new(0.50, 0.25, 0.65));
/// El juzgado de Saturno: ambiente de madera calientita y borde dorado (las nubes de Saturno).
const LUZ_JUZGADO: (Vec3, Vec3) = (Vec3::new(0.34, 0.29, 0.26), Vec3::new(0.45, 0.34, 0.20));
/// La Luna: sin aire, el ambiente es poquito y las sombras son duras.
const LUZ_LUNA: (Vec3, Vec3) = (Vec3::new(0.17, 0.17, 0.19), Vec3::new(0.10, 0.10, 0.13));
/// La tormenta de Neptuno: ambiente azul y borde celeste.
const LUZ_NEPTUNO: (Vec3, Vec3) = (Vec3::new(0.24, 0.27, 0.40), Vec3::new(0.30, 0.50, 0.90));

fn cargar(ruta: &str) -> Modelo {
    Modelo::cargar(ruta).unwrap_or_else(|e| {
        eprintln!("{e}\n(hay que correrlo desde la raiz del proyecto)");
        std::process::exit(1);
    })
}

/// Los numeros de un comentario de cabecera como `piloto: x y z escala s`.
fn numeros(m: &Modelo, clave: &str) -> Vec<f32> {
    m.comentario(clave).unwrap_or("").split_whitespace().filter_map(|s| s.parse().ok()).collect()
}

/// Donde va la piloto dentro de la burbuja (comentario `piloto:` de la nave).
fn lugar_de_la_piloto(nave: &Modelo) -> Mat4 {
    match numeros(nave, "piloto")[..] {
        [x, y, z, s, ..] => Mat4::translation(Vec3::new(x, y, z)) * Mat4::scale(s),
        _ => Mat4::translation(Vec3::new(0.0, 1.1, 0.3)) * Mat4::scale(0.5),
    }
}

/// Las esquinas del vidrio de las pantallas de la cabina (comentarios `pantalla_*:`), en
/// coordenadas de la camara: izquierda, centro y derecha.
fn pantallas_de_la_cabina(cabina: &Modelo) -> [[Vec3; 4]; 3] {
    ["pantalla_izq", "pantalla_centro", "pantalla_der"].map(|n| {
        let v = numeros(cabina, n);
        std::array::from_fn(|k| if v.len() >= 12 { Vec3::new(v[3 * k], v[3 * k + 1], v[3 * k + 2]) } else { Vec3::zero() })
    })
}

/// Todo lo que se prepara una vez al arrancar.
struct Recursos {
    /// La nave con cada skin de la tienda (mismo modelo, otros colores) y cual lleva puesta.
    naves: Vec<Modelo>,
    skin: std::cell::Cell<usize>,
    /// Los personajes (sentado y de pie, con su baile) y cual se escogio.
    elenco: Vec<Elenco>,
    /// Los asteroides: los tres modelos de roca, en gris, rojo (caliente) y oro (con un dato).
    rocas: [Vec<Modelo>; 3],
    /// Las cuatro lunas de Galileo (Io, Europa, Ganimedes y Calisto): los comodines de Jupiter.
    lunas_galileo: Vec<Modelo>,
    /// Bill Nye, el chico de la ciencia (cuenta los datos del universo): de pie y su baile.
    bill: Elenco,
    /// La cabina de la nave por dentro (Jupiter, en primera persona), su palanca de mando y las
    /// esquinas del vidrio de sus tres pantallas (ahi se dibuja el HUD).
    cabina: Modelo,
    palanca: Modelo,
    /// Los recuerdos de la mochila, en 3D (en el orden de `mochila::RECUERDOS`).
    recuerdos: Vec<Modelo>,
    pantallas_cabina: [[Vec3; 4]; 3],
    personaje: std::cell::Cell<usize>,
    sin_animacion: Animaciones,
    lugar_piloto: Mat4,
    /// Los cinco niveles de detalle de cada cuerpo (del ligero al mas fino), que arma el juego
    /// (ver planetas.rs y chapin.rs), y lo que se les junta que no es planeta (las caras de
    /// Neptuno y Pluton, el escenario de Saturno Ritmo).
    planetas: Vec<Vec<Modelo>>,
    extras_planetas: Vec<Option<Modelo>>,
    plataforma: Modelo,
    /// El lab del CIT (la Tierra): el modelo, el eje de las manecillas del reloj y los escritorios
    /// (en el piso; el 1 es el del protagonista).
    lab: Modelo,
    reloj_lab: Vec3,
    escritorios: Vec<Vec3>,
    /// El guardia del CIT, que hace su ronda (y entra con linterna si se va la luz).
    guardia: Modelo,
    guardia_baile: Baile,
    /// Urano: el baño de Doña Chepa (con las bisagras de las puertas de los cubiculos: la parte y
    /// donde gira), los clientes (uno por color), Doña Chepa, lo que se carga y el charco.
    bano_chepa: Modelo,
    bisagras: Vec<(usize, Vec3)>,
    aliens: Vec<Modelo>,
    chepa: Modelo,
    items_urano: [Modelo; 3],
    charco: Modelo,
    /// El Planeta Chapin: los modelos de la camioneta y de su carretera (ver camioneta.rs).
    camioneta: camioneta::Modelos,
    /// Los fondos de bloques de los minijuegos (ver decorado.rs).
    decorado: decorado::Decorado,
    /// Las maravillas (ver maravillas.rs): Rocky, la Blip-A, la Endurance, el monolito y el disco de
    /// acrecion de Gargantua.
    rocky: Modelo,
    blip_a: Modelo,
    endurance: Modelo,
    monolito: Modelo,
    disco_gargantua: Modelo,
    /// Planeta Vegetta: el atlas de los bloques (para armar los mundos), el diamante, un cubo
    /// suelto de cada bloque (los que se caen y los pedacitos, en el orden de `Bloque::TODOS`) y
    /// las grietas de lo que se esta picando (de menos a mas).
    atlas_bloques: Option<std::sync::Arc<obj::Textura>>,
    diamante_mc: Modelo,
    cubos: Vec<Modelo>,
    grietas: [Modelo; 4],
    profe: Modelo,
    /// Tierra (spin-off): el salon de Teoria de la UVG del examen de Koch.
    aula: Modelo,
    /// El Planeta Chapin (su malla en varios niveles de detalle y sus objetos). En una celda:
    /// se vuelve a armar en vivo al cambiarle la semilla o el relieve.
    chapin: std::cell::RefCell<chapin::Chapin>,
    objetos_chapin: ObjetosChapin,
    luna: Modelo,
    asiento_luna: Vec3,
    haz: Modelo,
    fans: Modelo,
    notas_ritmo: Modelo,
    tripulantes: Modelo,
    mesa_junta: Modelo,
    curiosity: Modelo,
    /// Ingenuity, InSight, Perseverance y Sojourner, en el orden de `cita::CANDIDATAS`.
    candidatas: Vec<Modelo>,
    cita_marte: Modelo,
    carta: Modelo,
    diamante: Modelo,
    /// Los rivales de las cartas en Neptuno: Doom Slayer, Master Chief y Cloud.
    rivales: Vec<Modelo>,
    /// Canela, que se mete a la partida de cartas.
    canela: Modelo,
    /// La Gran Fuente de las Hadas (el Proyecto 2) y sus personajes (Luka sale de la fuente).
    fuente_hadas: Modelo,
    luka: Modelo,
    navi: Modelo,
    contenedor: Modelo,
    /// El consultorio de tarot de Mercurio: la carpa, Madame Mercurio y la carta (con textura).
    carpa_tarot: Modelo,
    madame: Modelo,
    tarot: Modelo,
    /// El juicio de divorcio de Saturno en Venus: la sala, Saturno chiquito, los tres ex (en el orden de
    /// `divorcio::EXES`), el ganso juez y Phoenix Wright.
    juzgado: Modelo,
    saturno_cliente: Modelo,
    exes: Vec<Modelo>,
    ganso: Modelo,
    phoenix: Modelo,
    /// El set de filmacion de la Luna: el set, la tela de la bandera, el actor astronauta y el director.
    set_luna: Modelo,
    /// La nota musical que flota en el espacio (para recogerla).
    nota_espacio: Modelo,
    /// El Mirador, el punto de descanso en el L2 de la Tierra.
    mirador: Modelo,
    bandera: Modelo,
    astronauta: Modelo,
    director: Modelo,
    /// El portal del Planeta Hada (mascara del stencil y su aro) y la dimension que se ve por el.
    portal: Modelo,
    dimension: Modelo,
    /// GLADAS, la IA de la nave (sale en su retrato cuando habla).
    gladas: Modelo,
    /// Que partes de `fans` son la barra de luz, y la mano donde gira.
    fan_barra: Vec<bool>,
    fan_mano: Vec3,
    /// Las animaciones de la nave con cada skin (las de la base, mas los adornos que se mueven,
    /// como el anillo de la Endurance).
    anim_naves: Vec<Animaciones>,
    anim_planetas: Vec<Vec<Animaciones>>,
    cielo: Cielo,
    polvo: Polvo,
}

/// Un personaje cargado: sentado (para pilotar) y de pie (para bailar), como Miku.
struct Elenco {
    piloto: Modelo,
    anim_piloto: Animaciones,
    de_pie: Modelo,
    baile: Baile,
}

impl Recursos {
    /// El personaje escogido, sentado manejando.
    fn piloto(&self) -> &Modelo {
        &self.elenco[self.personaje.get()].piloto
    }
    fn anim_piloto(&self) -> &Animaciones {
        &self.elenco[self.personaje.get()].anim_piloto
    }
    /// El personaje escogido, de pie (para bailar).
    fn miku_baile(&self) -> &Modelo {
        &self.elenco[self.personaje.get()].de_pie
    }
    fn baile(&self) -> &Baile {
        &self.elenco[self.personaje.get()].baile
    }

    /// La nave, con la skin que se lleva puesta.
    fn nave(&self) -> &Modelo {
        &self.naves[self.skin.get()]
    }

    /// Las animaciones de la nave con la skin que se lleva puesta.
    fn anim_nave(&self) -> &Animaciones {
        &self.anim_naves[self.skin.get()]
    }

    fn cargar() -> Self {
        let nave = cargar("modelos/nave_lila.obj");
        let elenco: Vec<Elenco> = personajes::PERSONAJES
            .iter()
            .map(|p| {
                let piloto = cargar(&format!("modelos/{}_piloto.obj", p.archivo));
                let de_pie = cargar(&format!("modelos/{}_baile.obj", p.archivo));
                Elenco { anim_piloto: Animaciones::de(&piloto), baile: Baile::de(&de_pie), piloto, de_pie }
            })
            .collect();
        let rocas_grises: Vec<Modelo> = (1..=3).map(|k| cargar(&format!("modelos/asteroide_{k}.obj"))).collect();
        // Las rojas: roca oscura con grietas de lava que brillan. Las doradas: oro que brilla.
        let recolor = |m: &Modelo, f: &dyn Fn(&str) -> (Vec3, Vec3)| {
            let mut m = m.clone();
            for mat in &mut m.materiales {
                let (kd, ke) = f(&mat.nombre);
                mat.kd = kd;
                mat.ke = ke;
            }
            m
        };
        let rocas_rojas: Vec<Modelo> = rocas_grises
            .iter()
            .map(|m| recolor(m, &|n| if n == "crater" { (Vec3::new(1.0, 0.35, 0.08), Vec3::new(0.95, 0.30, 0.05)) } else if n == "roca_clara" { (Vec3::new(0.85, 0.18, 0.08), Vec3::new(0.45, 0.06, 0.02)) } else { (Vec3::new(0.42, 0.10, 0.08), Vec3::new(0.10, 0.01, 0.0)) }))
            .collect();
        let rocas_doradas: Vec<Modelo> = rocas_grises
            .iter()
            .map(|m| recolor(m, &|n| if n == "crater" { (Vec3::new(1.0, 0.85, 0.35), Vec3::new(0.6, 0.45, 0.1)) } else { (Vec3::new(0.95, 0.72, 0.22), Vec3::new(0.25, 0.18, 0.02)) }))
            .collect();
        let bill_de_pie = cargar("modelos/personaje_bill_baile.obj");
        let bill_piloto = cargar("modelos/personaje_bill_piloto.obj");
        let cabina = cargar("modelos/cabina.obj");
        let lab = cargar("modelos/lab_cit.obj");
        let guardia = cargar("modelos/personaje_guardia_baile.obj");
        let bill = Elenco { anim_piloto: Animaciones::de(&bill_piloto), baile: Baile::de(&bill_de_pie), piloto: bill_piloto, de_pie: bill_de_pie };
        let bano_chepa = cargar("modelos/bano_chepa.obj");
        let atlas_bloques = obj::Textura::cargar("modelos/texturas/bloques.png").ok().map(std::sync::Arc::new);
        let alien = cargar("modelos/alien_urano.obj");
        // Ningun planeta viene de un `.obj`: el juego los arma al arrancar, con voxels y su
        // deformacion con ruido (el Chapin en chapin.rs; los demas en planetas.rs), en paralelo.
        // Lo que no es el planeta sigue siendo modelo: las caras de Neptuno y Pluton y el
        // escenario de Saturno Ritmo, que se le juntan.
        let chapin = chapin::Chapin::new(chapin::PARAMETROS);
        let extras_planetas: Vec<Option<Modelo>> = CUERPOS
            .iter()
            .map(|c| match c.archivo {
                "neptuno" | "pluton" => Some(cargar(&format!("modelos/{}_cara.obj", c.archivo))),
                "saturno_ritmo" => Some(cargar("modelos/saturno_escenario.obj")),
                _ => None,
            })
            .collect();
        let planetas: Vec<Vec<Modelo>> = std::thread::scope(|hilos| {
            let tareas: Vec<_> = CUERPOS
                .iter()
                .zip(&extras_planetas)
                .map(|(c, extra)| {
                    let chapin = &chapin;
                    hilos.spawn(move || if c.archivo == "chapin" { chapin.niveles.clone() } else { armar_cuerpo(c.archivo, planetas::Ajuste::default(), extra.as_ref()) })
                })
                .collect();
            tareas.into_iter().map(|t| t.join().expect("no se pudo armar un planeta")).collect()
        });
        // Sus objetos: un modelo por tipo, y la casita en los colores de Antigua.
        let casita = cargar("modelos/chapin_casita.obj");
        let objetos_chapin = ObjetosChapin {
            ceiba: cargar("modelos/chapin_ceiba.obj"),
            roca: cargar("modelos/chapin_roca.obj"),
            ceiba_lejos: cargar("modelos/chapin_ceiba_lejos.obj"),
            roca_lejos: cargar("modelos/chapin_roca_lejos.obj"),
            casitas: chapin::COLORES_ANTIGUA
                .iter()
                .map(|&[r, g, b]| {
                    let mut m = casita.clone();
                    for mat in m.materiales.iter_mut().filter(|mat| mat.nombre == "pared") {
                        mat.kd = Vec3::new(r, g, b);
                    }
                    m
                })
                .collect(),
            tikal: cargar("modelos/chapin_tikal.obj"),
            arco: cargar("modelos/chapin_arco.obj"),
        };
        let luna = cargar("modelos/luna_columpio.obj");
        let fans = cargar("modelos/fans.obj");
        let asiento_luna = match numeros(&luna, "asiento")[..] {
            [x, y, z, ..] => Vec3::new(x, y, z),
            _ => Vec3::new(0.0, -1.79, 0.0),
        };
        Recursos {
            chapin: std::cell::RefCell::new(chapin),
            objetos_chapin,
            lugar_piloto: lugar_de_la_piloto(&nave),
            anim_naves: tienda::SKINS.iter().map(|k| Animaciones::de(&tienda::vestir(&nave, k))).collect(),
            anim_planetas: planetas.iter().map(|niveles| niveles.iter().map(Animaciones::de).collect()).collect(),
            extras_planetas,
            naves: tienda::SKINS.iter().map(|k| tienda::vestir(&nave, k)).collect(),
            skin: std::cell::Cell::new(0),
            elenco,
            rocas: [rocas_grises, rocas_rojas, rocas_doradas],
            lunas_galileo: ["io", "europa", "ganimedes", "calisto"].iter().map(|n| cargar(&format!("modelos/{n}.obj"))).collect(),
            bill,
            pantallas_cabina: pantallas_de_la_cabina(&cabina),
            cabina,
            palanca: cargar("modelos/cabina_palanca.obj"),
            recuerdos: (0..mochila::CUANTOS).map(|k| cargar(&format!("modelos/recuerdo_{k:02}.obj"))).collect(),
            personaje: std::cell::Cell::new(0),
            sin_animacion: Animaciones::ninguna(),
            planetas,
            plataforma: cargar("modelos/plataforma.obj"),
            reloj_lab: match numeros(&lab, "reloj")[..] {
                [x, y, z, ..] => Vec3::new(x, y, z),
                _ => Vec3::new(3.9, 2.45, -5.44),
            },
            escritorios: (0..12)
                .map(|n| match numeros(&lab, &format!("escritorio {n}"))[..] {
                    [x, z, ..] => Vec3::new(x, 0.0, z),
                    _ => Vec3::new(-4.2 + 2.8 * (n % 4) as f32, 0.0, -1.6 + 2.2 * (n / 4) as f32),
                })
                .collect(),
            lab,
            guardia_baile: Baile::de(&guardia),
            guardia,
            bisagras: (0..urano::CUBICULOS)
                .map(|n| {
                    let i = bano_chepa.partes.iter().position(|p| p.nombre == format!("puerta_{n}")).unwrap_or(usize::MAX);
                    let nums: Vec<f32> = bano_chepa.partes.get(i).map_or(Vec::new(), |p| p.comentario.split_whitespace().filter_map(|s| s.parse().ok()).collect());
                    (i, match nums[..] { [x, y, z, ..] => Vec3::new(x, y, z), _ => Vec3::zero() })
                })
                .collect(),
            bano_chepa,
            aliens: urano::COLORES_ALIEN
                .iter()
                .map(|&[r, g, b]| {
                    let mut m = alien.clone();
                    for mat in m.materiales.iter_mut().filter(|mat| mat.nombre == "piel") {
                        mat.kd = Vec3::new(r, g, b);
                    }
                    m
                })
                .collect(),
            chepa: cargar("modelos/chepa.obj"),
            items_urano: [cargar("modelos/item_papel.obj"), cargar("modelos/item_destapacanos.obj"), cargar("modelos/item_trapeador.obj")],
            charco: charco(),
            cubos: vegetta::Bloque::TODOS.iter().map(|&b| vegetta::cubo(b, atlas_bloques.clone())).collect(),
            grietas: std::array::from_fn(grieta),
            atlas_bloques,
            diamante_mc: cargar("modelos/diamante_mc.obj"),
            camioneta: camioneta::modelos(),
            decorado: decorado::armar(),
            rocky: cargar("modelos/rocky.obj"),
            blip_a: cargar("modelos/blip_a.obj"),
            endurance: cargar("modelos/endurance.obj"),
            monolito: cargar("modelos/monolito.obj"),
            disco_gargantua: maravillas::disco(),
            profe: cargar("modelos/profe.obj"),
            aula: cargar("modelos/aula_uvg.obj"),
            luna,
            asiento_luna,
            haz: cargar("modelos/haz_luz.obj"),
            notas_ritmo: cargar("modelos/notas_ritmo.obj"),
            tripulantes: cargar("modelos/tripulantes.obj"),
            mesa_junta: cargar("modelos/mesa_junta.obj"),
            curiosity: cargar("modelos/curiosity.obj"),
            candidatas: ["ingenuity", "insight", "perseverance", "sojourner"].iter().map(|n| cargar(&format!("modelos/{n}.obj"))).collect(),
            cita_marte: cargar("modelos/cita_marte.obj"),
            carta: cargar("modelos/carta.obj"),
            diamante: cargar("modelos/diamante.obj"),
            rivales: ["doom_slayer", "master_chief", "cloud"].iter().map(|n| cargar(&format!("modelos/{n}.obj"))).collect(),
            portal: cargar("modelos/portal.obj"),
            canela: cargar("modelos/canela.obj"),
            fuente_hadas: cargar("modelos/fuente_hadas.obj"),
            luka: cargar("modelos/luka.obj"),
            navi: cargar("modelos/navi.obj"),
            contenedor: cargar("modelos/contenedor.obj"),
            carpa_tarot: cargar("modelos/carpa_tarot.obj"),
            madame: cargar("modelos/madame_mercurio.obj"),
            tarot: cargar("modelos/tarot.obj"),
            juzgado: cargar("modelos/juzgado.obj"),
            saturno_cliente: cargar("modelos/saturno_cliente.obj"),
            exes: ["sonic", "gollum", "sinluz"].iter().map(|n| cargar(&format!("modelos/{n}.obj"))).collect(),
            ganso: cargar("modelos/ganso_juez.obj"),
            phoenix: cargar("modelos/phoenix.obj"),
            set_luna: cargar("modelos/set_luna.obj"),
            nota_espacio: cargar("modelos/nota_espacio.obj"),
            mirador: cargar("modelos/mirador.obj"),
            bandera: cargar("modelos/bandera.obj"),
            astronauta: cargar("modelos/astronauta.obj"),
            director: cargar("modelos/director.obj"),
            dimension: cargar("modelos/dimension.obj"),
            gladas: cargar("modelos/gladas.obj"),
            fan_barra: fans.partes.iter().map(|p| p.nombre.ends_with("_barra")).collect(),
            fan_mano: match numeros(&fans, "pivote mano")[..] {
                [x, y, z, ..] => Vec3::new(x, y, z),
                _ => Vec3::zero(),
            },
            fans,
            cielo: Cielo::new(),
            polvo: Polvo::new(220),
        }
    }
}

impl Recursos {
    /// El modelo del planeta `i` y sus animaciones, en el nivel de detalle que toca si en
    /// pantalla mide `radio_en_pantalla` pixeles de radio.
    fn planeta(&self, i: usize, radio_en_pantalla: f32) -> (&Modelo, &Animaciones) {
        let n = planetas::nivel_para(radio_en_pantalla).min(self.planetas[i].len() - 1);
        (&self.planetas[i][n], &self.anim_planetas[i][n])
    }

    /// El planeta `i` de cerca (en las escenas donde sale grande): el nivel detallado.
    fn planeta_detallado(&self, i: usize) -> (&Modelo, &Animaciones) {
        let n = 3.min(self.planetas[i].len() - 1);
        (&self.planetas[i][n], &self.anim_planetas[i][n])
    }

    /// El planeta `i` como modelo normal (el nivel de antes: el ligero de cerca), para las escenas
    /// donde sale chiquito o se usa entero (Saturno Ritmo).
    fn planeta_normal(&self, i: usize) -> (&Modelo, &Animaciones) {
        (&self.planetas[i][1], &self.anim_planetas[i][1])
    }

    /// Vuelve a armar el planeta `i` con otro ajuste (en vivo, F5 a F8).
    fn rearmar(&mut self, i: usize, ajuste: planetas::Ajuste) {
        let niveles = armar_cuerpo(CUERPOS[i].archivo, ajuste, self.extras_planetas[i].as_ref());
        self.anim_planetas[i] = niveles.iter().map(Animaciones::de).collect();
        self.planetas[i] = niveles;
    }

    /// El Chapin, despues de rearmarlo (su modulo lo arma: aqui solo se copian sus niveles).
    fn rearmar_chapin(&mut self) {
        let niveles = self.chapin.borrow().niveles.clone();
        self.anim_planetas[CHAPIN] = niveles.iter().map(Animaciones::de).collect();
        self.planetas[CHAPIN] = niveles;
    }

    /// Las matrices por parte de un fan: la barra de luz se mece sobre la mano.
    fn matrices_fan(&self, angulo: f32) -> Vec<Mat4> {
        let barra = Mat4::translation(self.fan_mano) * Mat4::rotation_z(angulo) * Mat4::translation(-self.fan_mano);
        self.fan_barra.iter().map(|&b| if b { barra } else { Mat4::identity() }).collect()
    }
}

/// Los cinco niveles del cuerpo `archivo` con el ajuste en vivo, con lo que se le junta que no es
/// planeta (las caras, el escenario de Saturno).
fn armar_cuerpo(archivo: &str, ajuste: planetas::Ajuste, extra: Option<&Modelo>) -> Vec<Modelo> {
    let niveles = planetas::armar(archivo, ajuste).unwrap_or_else(|| panic!("{archivo} no tiene receta"));
    match extra {
        Some(extra) => niveles.into_iter().map(|m| planetas::juntar(m, extra)).collect(),
        None => niveles,
    }
}

/// El golpe del ritmo en el tiempo `t`: 1 en cada beat y se apaga hasta el siguiente.
fn ritmo(t: f32) -> f32 {
    (-(t * BPM / 60.0).fract() * 5.0).exp()
}

/// Un draw call sin nada especial.
fn llamada<'a>(modelo: &'a Modelo, uniforms: Uniforms) -> DrawCall<'a> {
    DrawCall { modelo, uniforms, expresion: None, animacion: None, ocultar: &[], por_parte: None, modo: None, stencil: Stencil::Nada }
}

/// El estado del espacio.
struct Juego {
    /// La intensidad de los shaders de planeta (relieve del mapa de normales, olas, plasma); se
    /// cambia en vivo con F9 y F10.
    intensidad: f32,
    /// Los parametros del Planeta Chapin (se cambian en vivo con F5 a F8).
    params_chapin: chapin::Parametros,
    /// Lo que se le cambio en vivo a cada planeta (F5 a F8, junto a el): semilla y relieve.
    ajustes: [planetas::Ajuste; CUERPOS.len()],
    /// A que velocidad corre el sistema solar: las orbitas y los giros (en vivo, con F3 y F4).
    rapidez_orbitas: f32,
    /// Cuantas veces se ha saludado cada planeta, si ya saludo en esta visita, y hace cuanto que
    /// el piloto no toca nada (para que GLADAS opine).
    saludos: [u32; CUERPOS.len()],
    saludo_dicho: [bool; CUERPOS.len()],
    ocio: f32,
    veces_ocio: usize,
    /// La velocidad que junta la gravedad de los planetas (se suma a la de la nave).
    deriva: Vec3,
    /// El planeta que mas jala ahorita y con cuanta aceleracion (para el tablero).
    gravedad: Option<(usize, f32)>,
    /// El modo orbita: alrededor de que planeta y a que altura sobre su superficie.
    orbita: Option<(usize, f32)>,
    nave: Nave,
    camara: Camara,
    salto: Salto,
    opciones: Opciones,
    t: f32,
    giro_turbinas: f32,
    /// Lo que dice la IA de la nave y cuantos segundos le quedan en pantalla.
    mensaje: Option<(String, f32)>,
    /// Si el salto en curso es para aterrizar: al llegar, se baja solo.
    aterrizar_al_llegar: bool,
    /// Que tanto tiembla la camara despues de un choque (se apaga sola).
    sacudida: f32,
    /// Segundos desde el ultimo regano de la IA por chocar.
    desde_regano: f32,
    /// Los sonidos que hay que tocar este cuadro.
    eventos: Vec<Efecto>,
    /// La animacion de aterrizaje en curso (bajando, posada o despegando).
    aterrizaje: Option<Aterrizaje>,
    /// Que tanto va la nave pegada a la orbita del planeta mas cercano (0 a 1).
    sincronia: f32,
    /// Cuantos barrel rolls van (para que la IA no repita el chiste).
    barrels: usize,
    /// El reloj de las animaciones (llamas, turbinas, temblor): siempre corre.
    /// `t` es el de las orbitas, y se congela durante la cutscene del aterrizaje
    /// (como en los juegos: el mundo se queda quieto y la nave es la que se mueve).
    reloj: f32,
    /// Las notas que flotan entre las orbitas y el cometa que pasa cada tanto.
    coleccion: Coleccionables,
    /// El jugo de recoger: chispas, ondas, letreros, el hit-stop y el destello (ver chispas.rs).
    jugo: chispas::Jugo,
    /// Los homenajes del sistema (Interstellar, Project Hail Mary, 2001...; ver maravillas.rs).
    maravillas: maravillas::Maravillas,
    /// Logros que paso algo en el vuelo (el juego los desbloquea en el progreso).
    logros_nuevos: Vec<&'static str>,
    /// Donde se poso la nave (para los sellos del pasaporte).
    visitas: Vec<usize>,
    /// A que planeta va el hiperespacio (T o LB para escoger); `None`: al siguiente.
    objetivo: Option<usize>,
    /// Por donde paso la nave hace poquito (para dibujar la estela) y cada cuanto se apunta.
    rastro: Vec<Vec3>,
    rastro_t: f32,
    /// La estela que lleva puesta (id de `progreso::ESTELAS`).
    estela: Option<&'static str>,
    /// Los asteroides de alrededor (grises, rojos que tumban notas y dorados con datos).
    campo: asteroides::Campo,
    /// Lo que paso con los asteroides este cuadro: notas que se cayeron y si toca un dato de Bill.
    perdidas: u32,
    dato_bill: bool,
    /// Lo que se uso de la mochila y cuanto le queda (segundos): el escudo de Europa (las rocas
    /// rojas no tumban notas), el iman (las notas cercanas vienen solas) y el turbo de Io.
    escudo: f32,
    iman: f32,
    turbo_io: f32,
    /// La maldicion del retrogrado (del tarot de Mercurio, con salazon): segundos en que los
    /// controles de la nave van al reves.
    maldicion: f32,
    /// El rescate de cuando uno se va muy lejos (como en Mario Galaxy): segundos desde que
    /// empezo (la estrella cierra la pantalla, la nave vuelve y la estrella abre).
    rescate: Option<f32>,
    /// Si ya se aviso que se esta alejando (para no repetirlo cada cuadro).
    aviso_lejos: bool,
}

/// Desde donde se avisa que uno se esta saliendo del sistema solar, y donde lo rescatan.
const ORILLA_SISTEMA: f32 = 660.0;
const FIN_SISTEMA: f32 = 760.0;
/// Lo que dura el rescate (la estrella cierra, la nave regresa, la estrella abre).
const RESCATE: f32 = 2.4;

impl Juego {
    fn new(pos: Vec3, rumbo: f32) -> Self {
        let mut nave = Nave::new(pos);
        nave.yaw = rumbo;
        Juego {
            camara: Camara::detras_de(&nave),
            nave,
            salto: Salto::new(),
            opciones: Opciones { modo: ModoDibujo::Relleno, culling: true, vista: Vista::Color },
            t: 0.0,
            giro_turbinas: 0.0,
            mensaje: None,
            aterrizar_al_llegar: false,
            sacudida: 0.0,
            desde_regano: 10.0,
            eventos: Vec::new(),
            aterrizaje: None,
            sincronia: 0.0,
            intensidad: 1.0,
            params_chapin: chapin::PARAMETROS,
            ajustes: [planetas::Ajuste::default(); CUERPOS.len()],
            rapidez_orbitas: 1.0,
            saludos: [0; CUERPOS.len()],
            saludo_dicho: [false; CUERPOS.len()],
            ocio: 0.0,
            veces_ocio: 0,
            deriva: Vec3::zero(),
            gravedad: None,
            orbita: None,
            barrels: 0,
            reloj: 0.0,
            coleccion: Coleccionables::new(),
            jugo: chispas::Jugo::default(),
            maravillas: maravillas::Maravillas::default(),
            logros_nuevos: Vec::new(),
            campo: asteroides::Campo::new(),
            perdidas: 0,
            dato_bill: false,
            escudo: 0.0,
            iman: 0.0,
            turbo_io: 0.0,
            maldicion: 0.0,
            rescate: None,
            aviso_lejos: false,
            visitas: Vec::new(),
            objetivo: None,
            rastro: Vec::new(),
            rastro_t: 0.0,
            estela: None,
        }
    }

    /// El destino del hiperespacio: el escogido, o el que sigue del mas cercano.
    fn destino_del_salto(&self) -> usize {
        self.objetivo.unwrap_or_else(|| siguiente_destino(self.nave.pos, self.t))
    }

    fn decir(&mut self, texto: impl Into<String>) {
        self.mensaje = Some((texto.into(), 5.0));
    }

    /// Avanza `dt` segundos. Devuelve el planeta donde se pidio aterrizar.
    fn actualizar(&mut self, e: &Entrada, dt: f32) -> Option<usize> {
        // El hit-stop (al atrapar el cometa, al chocar): el mundo se detiene un instante, como si
        // pesara; las chispas y el destello siguen.
        if self.jugo.pausa > 0.0 {
            self.jugo.pausa -= dt;
            self.reloj += dt;
            self.jugo.actualizar(dt);
            return None;
        }
        self.reloj += dt;
        if self.aterrizaje.is_none() {
            self.t += dt * self.rapidez_orbitas;
        }
        self.personalidad(e, dt);
        // T o LB: escoger el destino del hiperespacio (pasa por todos y luego "ninguno").
        if e.objetivo {
            self.objetivo = match self.objetivo {
                None => Some(1),
                Some(i) if i + 1 < CUERPOS.len() => Some(i + 1),
                Some(_) => None,
            };
            self.eventos.push(Efecto::Carta);
        }
        // El rastro para la estela.
        self.rastro_t += dt;
        if self.rastro_t > 0.03 {
            self.rastro_t = 0.0;
            self.rastro.push(self.nave.pos - self.nave.adelante() * 0.5);
            if self.rastro.len() > 50 {
                self.rastro.remove(0);
            }
        }
        // Las notas y el cometa: se recogen volando (no bajando ni en el hiperespacio).
        let libre = self.aterrizaje.is_none() && !self.salto.activo();
        // El iman de la mochila: las notas cercanas vienen solas a la nave.
        if self.iman > 0.0 {
            self.iman -= dt;
            for n in self.coleccion.notas.iter_mut().filter(|n| n.viva) {
                let a = self.nave.pos - n.pos;
                let d = a.length();
                if d < 30.0 && d > 0.1 {
                    n.pos = n.pos + a * (dt * 3.0 / d.max(1.0)).min(1.0) * 8.0;
                }
            }
        }
        self.escudo = (self.escudo - dt).max(0.0);
        self.turbo_io = (self.turbo_io - dt).max(0.0);
        if self.aterrizaje.is_none() {
            self.maldicion = (self.maldicion - dt).max(0.0);
        }
        if self.turbo_io > 0.0 {
            self.nave.energia = 1.0;
        }
        self.coleccion.actualizar(self.nave.pos, dt, libre);
        self.eventos.append(&mut self.coleccion.eventos);
        // El jugo de recoger: chispas del color de la nota, una onda y el "+1" que sube; cada cinco
        // del combo, el letrero grande y chispas doradas. El cometa, ademas, detiene el mundo un
        // instante (hit-stop), destella y sacude la camara.
        for (p, rosa, combo) in std::mem::take(&mut self.coleccion.recogidas_en) {
            let color = if rosa { chispas::ROSA } else { chispas::CIAN };
            self.jugo.estallar(p, color, 14, 9.0);
            self.jugo.onda(p, color, 2.5);
            self.jugo.letrero(p, "+1", color, false);
            self.jugo.latido_notas = 1.0;
            if combo >= 5 && combo % 5 == 0 {
                self.jugo.letrero(p + Vec3::new(0.0, 1.6, 0.0), format!("¡COMBO x{combo}!"), chispas::DORADO, true);
                self.jugo.estallar(p, chispas::DORADO, 24, 14.0);
                self.jugo.onda(p, chispas::DORADO, 6.0);
            }
        }
        if let Some(p) = self.coleccion.cometa_en.take() {
            self.jugo.estallar(p, [0.8, 0.95, 1.0], 40, 22.0);
            self.jugo.estallar(p, chispas::DORADO, 20, 14.0);
            self.jugo.onda(p, [0.8, 0.95, 1.0], 12.0);
            self.jugo.letrero(p + Vec3::new(0.0, 2.0, 0.0), "+25 ¡COMETA!", [0.7, 0.95, 1.0], true);
            self.jugo.pausa = 0.09;
            self.jugo.destello = 0.6;
            self.jugo.latido_notas = 1.0;
            self.sacudida = self.sacudida.max(0.6);
        }
        self.jugo.actualizar(dt);
        // Los asteroides: los grises rebotan, los rojos tumban notas, los dorados traen un dato.
        self.perdidas = 0;
        self.dato_bill = false;
        let adelante = self.nave.adelante();
        self.campo.actualizar(&mut self.nave.pos, adelante, dt, libre && self.salto.puede_maniobrar(), self.escudo > 0.0);
        self.eventos.append(&mut self.campo.eventos);
        self.dato_bill = self.campo.dato;
        if let Some(clase) = self.campo.choque {
            // El golpe pesa: chispas donde pego y el mundo se detiene un instante.
            let (color, n) = if clase == asteroides::Clase::Roja { ([1.0, 0.45, 0.15], 26) } else { ([0.85, 0.82, 0.95], 14) };
            self.jugo.estallar(self.nave.pos + adelante * 1.2, color, n, 11.0);
            self.jugo.pausa = self.jugo.pausa.max(0.06);
            self.nave.velocidad = -self.nave.velocidad * 0.35;
            self.sacudida = self.sacudida.max(if clase == asteroides::Clase::Roja { 0.9 } else { 0.5 });
            if clase == asteroides::Clase::Roja {
                // Se caen notas: un tanto fijo, mas una parte de las que trae (se pueden volver a recoger).
                self.perdidas = 6;
                let donde = self.campo.donde;
                self.coleccion.soltar(donde, 6);
                self.decir("¡Asteroide caliente! Se le cayeron notas. Recójalas rápido, que se derriten en cuatro segundos.");
            } else if self.desde_regano > 4.0 {
                self.desde_regano = 0.0;
                self.decir("Choque con un asteroide. Es una roca. Las rocas no se mueven. Bueno, estas sí, pero poquito.");
            }
        }
        if let Some(aviso) = self.coleccion.aviso {
            self.decir(aviso);
        }
        if let Some((_, quedan)) = &mut self.mensaje {
            *quedan -= dt;
            if *quedan <= 0.0 {
                self.mensaje = None;
            }
        }
        // Aterrizando, posada o despegando: manda la animacion.
        if let Some(a) = &mut self.aterrizaje {
            let mut escena = None;
            // ESPACIO salta la cutscene.
            if e.aceptar {
                a.saltar(self.t);
            }
            match a.actualizar(&mut self.nave, e.aterrizar, dt, self.t) {
                // El golpe al tocar el suelo: sonido y sacudida fuerte.
                aterrizaje::Evento::Toque => {
                    self.eventos.push(Efecto::Choque);
                    self.sacudida = self.sacudida.max(0.9);
                }
                aterrizaje::Evento::Posada => {
                    let i = a.planeta;
                    self.visitas.push(i);
                    if CON_ATERRIZAJE.contains(&i) {
                        escena = Some(i);
                    } else {
                        let texto = format!("{} {}: despegar.", aterrizaje::dato(i), ui::boton(e.nombre("aterrizar")));
                        self.mensaje = Some((texto, 9.0));
                    }
                }
                aterrizaje::Evento::Despego => self.aterrizaje = None,
                aterrizaje::Evento::Nada => {}
            }
            // La entrada a la atmosfera y los motores hacen temblar la camara.
            self.sacudida = (self.sacudida * (-4.0 * dt).exp()).max(self.aterrizaje.as_ref().map_or(0.0, |a| a.temblor()));
            if let Some(a) = &self.aterrizaje {
                (self.camara.ojo, self.camara.mira) = a.camara(&self.nave, self.t);
            } else {
                self.camara = Camara::detras_de(&self.nave);
            }
            self.giro_turbinas += dt * 8.0;
            return escena;
        }

        // El rescate (Mario Galaxy): la estrella cierra la pantalla, la nave aparece junto al
        // planeta de la orilla, y la estrella abre otra vez.
        if let Some(antes) = self.rescate {
            let ahora = antes + dt;
            self.rescate = Some(ahora);
            let mitad = RESCATE * 0.5;
            if antes < mitad && ahora >= mitad {
                let (cerca, _) = mas_cercano(self.nave.pos, self.t);
                let planeta = CUERPOS[cerca].posicion(self.t);
                let hacia_sol = (CUERPOS[0].posicion(self.t) - planeta).normalize();
                self.nave.pos = planeta + hacia_sol * (CUERPOS[cerca].radio_visual() * 3.0 + 12.0) + Vec3::new(0.0, 3.0, 0.0);
                self.nave.velocidad = 0.0;
                // Mirando hacia el planeta (de espaldas al Sol).
                let mira = planeta - self.nave.pos;
                self.nave.yaw = (-mira.x).atan2(-mira.z);
                self.nave.pitch = 0.0;
                self.camara = Camara::detras_de(&self.nave);
                self.eventos.push(Efecto::Cometa);
                self.decir("¡Uy! Casi se nos va al espacio profundo. Lo traje de vuelta con la estrella de rescate, como en Mario Galaxy. De nada.");
            }
            if ahora >= RESCATE {
                self.rescate = None;
                self.aviso_lejos = false;
            }
            self.camara.seguir(&self.nave, dt, 5.0);
            return None;
        }
        // Las maravillas (ver maravillas.rs): en el paso por el gusano o en el teseracto la nave no
        // se maneja; cuando cambia de lugar, la camara salta con ella.
        if self.maravillas.en_secuencia() {
            let antes = self.nave.pos;
            let novedades = self.maravillas.actualizar(&mut self.nave, &mut self.deriva, self.t, dt, false);
            self.aplicar_maravillas(novedades);
            if (self.nave.pos - antes).length() > 50.0 {
                self.camara = Camara::detras_de(&self.nave);
            }
            self.camara.seguir(&self.nave, dt, 5.0);
            return None;
        }
        // Lejos del Sol: primero un aviso y un jaloncito de regreso; mas alla, el rescate.
        let lejos = (self.nave.pos - CUERPOS[0].posicion(self.t)).length();
        if !self.salto.activo() && self.aterrizaje.is_none() {
            if lejos > FIN_SISTEMA {
                self.rescate = Some(0.0);
                self.eventos.push(Efecto::Salto);
                self.logros_nuevos.push("no_panico");
            } else if lejos > ORILLA_SISTEMA {
                if !self.aviso_lejos {
                    self.aviso_lejos = true;
                    self.decir("Nos estamos saliendo del sistema solar. Afuera no hay nada: ni planetas, ni notas, ni wifi. Mejor regresemos.");
                    self.eventos.push(Efecto::Alarma);
                }
                // La gravedad del sistema jala de regreso, cada vez mas fuerte.
                let k = (lejos - ORILLA_SISTEMA) / (FIN_SISTEMA - ORILLA_SISTEMA);
                let adentro = (CUERPOS[0].posicion(self.t) - self.nave.pos).normalize();
                self.nave.pos += adentro * (k * k * 40.0 * dt);
            } else if lejos < ORILLA_SISTEMA - 40.0 {
                self.aviso_lejos = false;
            }
        }
        if self.salto.activo() {
            self.salto.actualizar(&mut self.nave, dt, self.t);
            // Un salto pedido con E termina aterrizando solo.
            if self.aterrizar_al_llegar && self.salto.puede_maniobrar() {
                self.aterrizar_al_llegar = false;
                self.empezar_aterrizaje(self.salto.destino);
            }
        } else {
            let (cerca, distancia) = mas_cercano(self.nave.pos, self.t);
            if self.nave.actualizar(e, dt, nave::limite_de_velocidad(distancia)) {
                const BARRELS: [&str; 4] = [
                    "¡Barrel roll! Así se hace. Un conejo con audífonos estaría orgulloso.",
                    "Otro barrel roll. Miku dice que ya vio su desayuno pasar. Y no desayunó.",
                    "Barrel roll número tres. Los planetas no aplauden porque no tienen manos.",
                    "Ya van varios barrel rolls. Yo no me mareo. Soy software. Pero me preocupo.",
                ];
                if self.barrels < BARRELS.len() {
                    self.decir(BARRELS[self.barrels]);
                }
                self.barrels += 1;
                self.eventos.push(Efecto::Salto);
            }
            // Sincronizar orbita: cerca de un planeta la nave se mueve con el, como si
            // la jalara su gravedad; asi el planeta no se le escapa mientras se acerca.
            let c = &CUERPOS[cerca];
            let alcance = c.radio_visual() * 1.5 + 12.0;
            // Completa en la mitad de cerca del alcance, y se apaga hacia la orilla.
            let objetivo = ((alcance - distancia) / (alcance * 0.5)).clamp(0.0, 1.0);
            self.sincronia += (objetivo - self.sincronia) * (1.0 - (-3.0 * dt).exp());
            let arrastre = (c.posicion(self.t) - c.posicion(self.t - dt)) * self.sincronia;
            self.nave.pos += arrastre;
            // MODO ORBITA (R, o B en el control) junto a un planeta: la nave sigue la curva del
            // planeta a la misma altura, con el rumbo siempre tangente a la superficie. A y D giran
            // como siempre (cambian el plano de la orbita) y las flechas suben o bajan.
            if e.orbitar {
                self.orbita = match self.orbita {
                    Some(_) => {
                        self.decir("Modo órbita apagado. Vuelo libre, con gravedad y todo.");
                        None
                    }
                    None if distancia < c.radio_visual() * 1.5 + 12.0 => {
                        let altura = (self.nave.pos - c.posicion(self.t)).length() - c.radio_cuerpo();
                        self.decir(format!("Modo órbita en {}: la nave sigue la curva a esta altura. {{arriba_abajo}}: más alto o más bajo. {{orbitar}}: salir.", c.nombre));
                        Some((cerca, altura.max(1.5)))
                    }
                    None => {
                        self.decir("Para el modo órbita hay que estar cerca de un planeta. Acérquese y vuelva a probar.");
                        None
                    }
                };
            }
            if let Some((i, altura)) = &mut self.orbita {
                let p = &CUERPOS[*i];
                let centro = p.posicion(self.t);
                *altura = (*altura + e.cabecear * 3.0 * dt).clamp(1.2, p.radio_visual() + 12.0);
                let arriba = (self.nave.pos - centro).normalize();
                self.nave.pos = centro + arriba * (p.radio_cuerpo() + *altura);
                let adelante = self.nave.adelante();
                let tangente = adelante - arriba * adelante.dot(arriba);
                if tangente.length() > 1e-3 {
                    (self.nave.yaw, self.nave.pitch) = salto::rumbo_hacia(tangente);
                }
                self.deriva = Vec3::zero();
                self.gravedad = None;
            } else {
                // GRAVEDAD: cada planeta cercano jala la nave hacia su centro con a = G·M/d², y la
                // masa crece con el volumen (asi en la superficie jala mas el planeta mas grande). El
                // jalon se junta en una velocidad de deriva que se va apagando (rozamiento), y esa
                // deriva mueve la nave: la trayectoria se curva hacia el planeta.
                let mut jalon = Vec3::zero();
                let mut mas_fuerte: Option<(usize, f32)> = None;
                for (i, p) in CUERPOS.iter().enumerate().skip(1) {
                    let hacia = p.posicion(self.t) - self.nave.pos;
                    let d = hacia.length();
                    let r = p.radio_cuerpo();
                    let alcance = r * 5.0 + 20.0;
                    if d < alcance && d > r * 0.5 {
                        // Se apaga suave hacia la orilla de su alcance (sin tirones al entrar).
                        let orilla = ((alcance - d) / (alcance * 0.4)).clamp(0.0, 1.0);
                        let a = GRAVEDAD * r * (r / d).powi(2) * orilla;
                        jalon += hacia * (a / d);
                        if mas_fuerte.is_none_or(|(_, x)| a > x) {
                            mas_fuerte = Some((i, a));
                        }
                    }
                }
                self.deriva += jalon * dt;
                self.deriva = self.deriva * (-0.8 * dt).exp();
                self.nave.pos += self.deriva * dt;
                self.gravedad = mas_fuerte;
            }
            if e.aceptar {
                let destino = self.destino_del_salto();
                self.salto.iniciar(&self.nave, destino);
                self.deriva = Vec3::zero();
                self.orbita = None;
                self.eventos.push(Efecto::Salto);
                self.decir(format!("Salto al hiperespacio. Destino: {}. Abróchese el cinturón, que no tenemos.", CUERPOS[destino].nombre));
            }
        }
        // Las maravillas: la gravedad de Gargantua, entrar al gusano y lo que se cuenta al acercarse.
        if !self.salto.activo() {
            let hablando = self.mensaje.is_some();
            let novedades = self.maravillas.actualizar(&mut self.nave, &mut self.deriva, self.t, dt, hablando);
            self.aplicar_maravillas(novedades);
        }
        // E junto al monolito: GLADAS hace de HAL (no hay donde aterrizar).
        let junto_al_monolito = self.maravillas.junto_al_monolito(self.nave.pos, self.t) && aterrizable(self.nave.pos, self.t).is_none();
        if e.aterrizar && junto_al_monolito {
            self.decir("«Abre la compuerta, GLADAS.» «Lo siento, piloto. Me temo que no puedo hacer eso.» ... Es broma: no tiene compuerta. Es un monolito.");
        }
        // E se escucha tambien durante el destello de llegada de un salto.
        if e.aterrizar && !junto_al_monolito && self.salto.puede_maniobrar() && self.aterrizaje.is_none() {
            match aterrizable(self.nave.pos, self.t) {
                Some(i) => self.empezar_aterrizaje(i),
                // Lejos de todo: salta al planeta escogido (o al mas cercano) y baja solo al llegar.
                None => {
                    let destino = self.objetivo.unwrap_or_else(|| mas_cercano(self.nave.pos, self.t).0.max(1));
                    self.salto = Salto::new();
                    self.salto.iniciar(&self.nave, destino);
                    self.aterrizar_al_llegar = true;
                    self.eventos.push(Efecto::Salto);
                    self.decir(format!("Rumbo a {} para aterrizar. Usted relájese, yo hago todo. Como siempre.", CUERPOS[destino].nombre));
                }
            }
        }
        // Los planetas son solidos (menos durante el viaje del salto, que es una escena).
        self.desde_regano += dt;
        self.sacudida *= (-4.0 * dt).exp();
        if self.salto.puede_maniobrar() {
            if let Some(i) = colision::resolver(&mut self.nave.pos, self.t) {
                // Rebota hacia atras con un tercio de la velocidad (y la gravedad deja de empujar).
                self.nave.velocidad = -self.nave.velocidad * 0.35;
                self.deriva = Vec3::zero();
                if self.sacudida < 0.2 {
                    self.eventos.push(Efecto::Choque);
                }
                self.sacudida = self.sacudida.max(0.5);
                if self.desde_regano > 4.0 {
                    self.desde_regano = 0.0;
                    let frase = match i {
                        0 => "Eso era el Sol. Se recomienda no tocar el Sol. Ni con la nave, ni con la mirada.".to_string(),
                        _ => format!("Choque con {}. El seguro no cubre planetas. Tampoco pilotos. Ni a mí.", CUERPOS[i].nombre),
                    };
                    self.decir(frase);
                }
            }
        }
        self.camara.seguir(&self.nave, dt, 5.0 + 35.0 * self.salto.estiramiento());
        self.giro_turbinas += dt * (3.0 + self.nave.vel.length() * 0.6);
        None
    }

    /// Usa una luna de la mochila: Io (turbo infinito), Europa (escudo), Ganimedes (iman) o
    /// Calisto (llama al cometa).
    fn usar_luna(&mut self, k: usize) {
        match k {
            0 => {
                self.turbo_io = mochila::DURA_USO[0];
                self.decir("Luna Io activada: turbo infinito por 30 segundos. Tiene 400 volcanes; energía le sobra.");
            }
            1 => {
                self.escudo = mochila::DURA_USO[1];
                self.decir("Escudo de Europa: un minuto sin que los asteroides rojos le tumben notas. Hielo de la mejor calidad.");
            }
            2 => {
                self.iman = mochila::DURA_USO[2];
                self.decir("Imán de Ganímedes: las notas de cerca vienen solas. Vuele cerca de ellas y no haga nada. Como siempre.");
            }
            _ => {
                self.coleccion.llamar_cometa();
                self.decir("Calisto llamó al cometa. Ya viene. Alcáncelo antes de que se arrepienta.");
            }
        }
        self.eventos.push(Efecto::Perfecto);
    }

    fn empezar_aterrizaje(&mut self, planeta: usize) {
        self.deriva = Vec3::zero();
        self.orbita = None;
        // En los planetas sin escena, Miku se baja a saludar.
        self.aterrizaje = Some(Aterrizaje::new(planeta, &self.nave, !CON_ATERRIZAJE.contains(&planeta)));
        self.eventos.push(Efecto::Aterrizaje);
        // En la cutscene habla la tarjeta del planeta; GLADAS espera a que se pose.
        self.mensaje = None;
    }

    /// Lo que se mueve la camara por la sacudida de un choque.
    fn temblor(&self) -> Vec3 {
        let s = self.sacudida * 0.25;
        Vec3::new((self.reloj * 53.0).sin() * s, (self.reloj * 47.0).cos() * s, (self.reloj * 61.0).sin() * s * 0.5)
    }

    fn estado_anim(&self) -> EstadoAnim {
        EstadoAnim {
            t: self.reloj,
            giro_turbinas: self.giro_turbinas,
            // Los motores nunca se apagan del todo: aunque este quieta, la nave esta arrancada.
            empuje: ((self.nave.velocidad.max(0.0) / 18.0).min(1.0) * 0.8 + self.turbo_encendido() * 0.6 + self.salto.empuje() + self.aterrizaje.as_ref().map_or(0.0, |a| a.empuje())).max(0.2),
            viraje: self.nave.roll / nave::LADEO,
            ritmo: ritmo(self.reloj),
        }
    }

    /// Que tanto cubre la estrella del rescate la pantalla (0: nada; 1: toda), si hay rescate.
    fn estrella_de_rescate(&self) -> Option<f32> {
        self.rescate.map(|r| {
            let u = r / (RESCATE * 0.5);
            if u < 1.0 { u } else { 2.0 - u }
        })
    }

    /// Que tan cerca de la orilla del sistema esta (0: adentro; 1: ya casi lo rescatan).
    fn en_la_orilla(&self) -> f32 {
        let lejos = (self.nave.pos - CUERPOS[0].posicion(self.t)).length();
        ((lejos - ORILLA_SISTEMA) / (FIN_SISTEMA - ORILLA_SISTEMA)).clamp(0.0, 1.0)
    }

    /// Grados que se abre el FOV: con el salto, y un poco con la velocidad (se siente la rapidez).
    fn fov_extra(&self) -> f32 {
        self.salto.fov_extra() + 16.0 * (self.nave.velocidad / nave::VELOCIDAD_CRUCERO).clamp(0.0, 1.0)
    }

    /// 1 si el turbo esta jalando ahorita (para las llamas).
    fn turbo_encendido(&self) -> f32 {
        if self.nave.velocidad > 20.0 && self.nave.energia < 0.999 { 1.0 } else { 0.0 }
    }

    /// Lo que paso con las maravillas: lo que dice GLADAS, los sonidos, los logros y la sacudida.
    fn aplicar_maravillas(&mut self, n: maravillas::Novedades) {
        if let Some(m) = n.mensaje {
            self.decir(m);
        }
        self.eventos.extend(n.efectos);
        self.logros_nuevos.extend(n.logros);
        self.sacudida = self.sacudida.max(n.sacudida);
    }

    /// Los planetas saludan cuando uno llega, y GLADAS se aburre si uno no hace nada. Nunca le
    /// quitan la palabra a un mensaje que este saliendo.
    fn personalidad(&mut self, e: &Entrada, dt: f32) {
        let toco = e.empuje != 0.0 || e.girar != 0.0 || e.cabecear != 0.0 || e.horizontal != 0.0 || e.aceptar || e.turbo || e.barrel || e.aterrizar || e.orbitar;
        self.ocio = if toco || self.nave.velocidad > 2.0 { 0.0 } else { self.ocio + dt };
        for (i, c) in CUERPOS.iter().enumerate() {
            let d = (c.posicion(self.t) - self.nave.pos).length();
            let radio = c.radio_visual();
            if d > radio * 7.0 + 30.0 {
                self.saludo_dicho[i] = false;
            } else if d < radio * 3.2 + 6.0 && !self.saludo_dicho[i] && self.mensaje.is_none() && self.aterrizaje.is_none() {
                let frases = personalidad::saludos(c.archivo);
                if !frases.is_empty() {
                    self.saludo_dicho[i] = true;
                    self.decir(frases[self.saludos[i] as usize % frases.len()]);
                    self.saludos[i] += 1;
                }
            }
        }
        if self.ocio > 24.0 && self.mensaje.is_none() {
            self.ocio = 0.0;
            self.decir(personalidad::OCIO[self.veces_ocio % personalidad::OCIO.len()]);
            self.veces_ocio += 1;
        }
    }

    /// Al volver de una escena: la nave esta posada sobre el planeta y despega.
    fn despegar_de(&mut self, planeta: usize, mensaje: String) {
        self.aterrizaje = Some(Aterrizaje::despegando(planeta, &self.nave));
        self.eventos.push(Efecto::Salto);
        self.decir(mensaje);
    }
}

/// El planeta mas cercano a un punto (indice en `CUERPOS`) y que tan lejos
/// esta su superficie.
fn mas_cercano(p: Vec3, t: f32) -> (usize, f32) {
    CUERPOS
        .iter()
        .enumerate()
        .map(|(i, c)| (i, (c.posicion(t) - p).length() - c.radio_visual()))
        .fold((0, f32::INFINITY), |a, b| if b.1 < a.1 { b } else { a })
}

/// La linea del panel de TAB del planeta mas cercano: en que nivel de detalle se esta dibujando y
/// con que semilla y relieve (lo que cambian F5 a F8), y la intensidad de los shaders.
fn panel_planeta(juego: &Juego) -> String {
    let (cerca, _) = mas_cercano(juego.nave.pos, juego.t);
    let c = &CUERPOS[cerca];
    let pixeles_por_unidad = RENDER_H as f32 * 0.5 / ((FOV + juego.fov_extra()).to_radians() * 0.5).tan();
    let radio_px = c.radio_visual() / (c.posicion(juego.t) - juego.camara.ojo).length().max(0.1) * pixeles_por_unidad;
    let nivel = planetas::nivel_para(radio_px) + 1;
    let ajuste = if cerca == CHAPIN {
        format!("semilla {}, relieve {:.2}", juego.params_chapin.semilla, juego.params_chapin.relieve)
    } else {
        let a = juego.ajustes[cerca];
        format!("semilla {:+}, relieve ×{:.2}", a.semilla as i32, a.relieve)
    };
    format!("{}: nivel {nivel}/5, {ajuste}, radio ×{:.1}, órbitas ×{}, shaders {:.2}", c.nombre, escena::factor_radio(cerca), juego.rapidez_orbitas, juego.intensidad)
}

/// Los planetas que tienen aterrizaje.
/// Una semilla distinta en cada corrida (para que las lecturas de tarot cambien).
fn semilla_del_reloj() -> u32 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(7, |d| d.subsec_nanos())
}

const CON_ATERRIZAJE: [usize; 13] = [MERCURIO, VENUS, TIERRA, LUNA, MARTE, JUPITER, SATURNO, URANO, NEPTUNO, PLUTON, HADA, VEGETTA, CHAPIN];

/// En que planeta se puede aterrizar desde aqui, si en alguno. Si hay uno con
/// aterrizaje en rango gana ese, aunque otro quede mas cerca: junto a la
/// Tierra casi siempre esta la Luna, y si no, nunca se podria bajar.
fn aterrizable(p: Vec3, t: f32) -> Option<usize> {
    let en_rango: Vec<(usize, f32)> = CUERPOS
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, c)| (i, (c.posicion(t) - p).length() - c.radio_visual()))
        .filter(|&(i, d)| d < alcance_de_aterrizaje(i))
        .collect();
    let mas_cerca = |lista: &mut dyn Iterator<Item = &(usize, f32)>| lista.min_by(|a, b| a.1.total_cmp(&b.1)).map(|x| x.0);
    mas_cerca(&mut en_rango.iter().filter(|(i, _)| CON_ATERRIZAJE.contains(i))).or_else(|| mas_cerca(&mut en_rango.iter()))
}

/// Que tan cerca de la superficie hay que estar para aterrizar en el cuerpo `i`. Las lunas (la nuestra)
/// estan pegadas a su planeta y son chiquitas: hay que llegarles de cerca, si no al apretar E junto a la
/// Tierra se aterrizaria en la Luna.
fn alcance_de_aterrizaje(i: usize) -> f32 {
    let c = &CUERPOS[i];
    if c.padre.is_some() { c.radio_visual() * 2.0 } else { DISTANCIA_ATERRIZAJE + c.radio_visual() * 0.8 }
}

/// Donde esta el Mirador (el punto de descanso): en el punto L2 de la Tierra, del lado contrario
/// al Sol, como el telescopio James Webb. Se mueve con la Tierra.
fn mirador_en(t: f32) -> Vec3 {
    let tierra = CUERPOS[TIERRA].posicion(t);
    let afuera = (tierra - CUERPOS[0].posicion(t)).normalize();
    tierra + afuera * 24.0 + Vec3::new(0.0, 4.0, 0.0)
}

/// Que tan cerca hay que estar del Mirador para descansar ahi.
const DISTANCIA_MIRADOR: f32 = 12.0;

/// A donde lleva el salto: al cuerpo que sigue del mas cercano (sin el Sol).
fn siguiente_destino(p: Vec3, t: f32) -> usize {
    mas_cercano(p, t).0 % (CUERPOS.len() - 1) + 1
}

fn base_uniforms(vista: Mat4, proyeccion: Mat4, luz: Vec3, camara: Vec3, (ambiente, borde): (Vec3, Vec3)) -> Uniforms {
    Uniforms {
        modelo: Mat4::identity(),
        vista,
        proyeccion,
        luz,
        camara,
        ambiente,
        borde,
        color_luz: Vec3::new(1.0, 1.0, 1.0),
        alcance: 0.0,
        shader: Shader::Phong,
        tiempo: 0.0,
        centro: Vec3::zero(),
        eje: Vec3::new(0.0, 1.0, 0.0),
        radio: 1.0,
        intensidad: 1.0,
        celda: 0.0,
    }
}

/// El color del aire de cada planeta (para el anillo de su atmosfera en el espacio).
const ATMOSFERAS: [(usize, [f32; 3]); 10] = [
    (VENUS, [0.95, 0.75, 0.35]),
    (TIERRA, [0.35, 0.65, 1.0]),
    (MARTE, [1.0, 0.5, 0.3]),
    (JUPITER, [0.95, 0.75, 0.5]),
    (SATURNO, [1.0, 0.82, 0.5]),
    (URANO, [0.45, 0.95, 1.0]),
    (NEPTUNO, [0.3, 0.5, 1.0]),
    (HADA, [1.0, 0.55, 0.9]),
    (VEGETTA, [0.55, 0.95, 0.5]),
    (CHAPIN, [0.40, 0.90, 0.85]),
];

/// Un cuadro del espacio. Con `intro`, la nave esta en la plataforma y Miku
/// baila afuera hasta que se sube.
fn dibujar_espacio(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, juego: &Juego, intro: Option<&Intro>) -> Estadisticas {
    let t = juego.t;
    let fov = (FOV + juego.fov_extra()).to_radians();
    let (ojo, mira) = match intro {
        Some(i) => i.camara(&juego.nave, t),
        None => (juego.camara.ojo + juego.temblor(), juego.camara.mira + juego.temblor() * 0.5),
    };
    let arriba = if intro.is_some() { Vec3::new(0.0, 1.0, 0.0) } else { juego.camara.arriba() };
    let vista = Mat4::look_at(ojo, mira, arriba);
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, CERCA, LEJOS);
    let vista_proyeccion = proyeccion * vista;

    // ---------- aplicacion (CPU): un draw call por objeto ----------
    let base = base_uniforms(vista, proyeccion, CUERPOS[0].posicion(t), ojo, LUZ_ESPACIO);
    let anim = juego.estado_anim();
    // Nivel de detalle: que tan grande se ve cada planeta decide que modelo se dibuja.
    let pixeles_por_unidad = fb.alto as f32 * 0.5 / (fov * 0.5).tan();
    let chapin = rec.chapin.borrow();
    // Los uniforms de los shaders de planeta (centro, eje, mar, reloj, intensidad).
    let de_planeta = |c: &escena::Cuerpo| Uniforms {
        modelo: c.matriz(t),
        centro: c.posicion(t),
        eje: c.eje(),
        radio: c.escala_viva(),
        tiempo: juego.reloj,
        intensidad: juego.intensidad,
        ..base
    };
    // El nivel de detalle del Chapin: uno solo para la tierra, el agua y los objetos (la tierra
    // esconde las caras que dan al agua de SU rejilla: con otro nivel, quedarian huecos).
    let c_chapin = &CUERPOS[CHAPIN];
    let radio_chapin = c_chapin.radio_visual() / (c_chapin.posicion(t) - ojo).length().max(0.1) * pixeles_por_unidad;
    let nivel_chapin = planetas::nivel_para(radio_chapin);
    let mut llamadas: Vec<DrawCall> = CUERPOS
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let distancia = (c.posicion(t) - ojo).length().max(0.1);
            let radio_px = c.radio_visual() / distancia * pixeles_por_unidad;
            match i {
                // El Sol: el plasma animado (brilla solo, sin luz de borde).
                0 => llamada(rec.planeta(0, radio_px).0, Uniforms { shader: Shader::Plasma, borde: Vec3::zero(), ..de_planeta(c) }),
                // El Chapin: la tierra en el nivel de detalle (tamano de voxel) que toca.
                CHAPIN => llamada(&chapin.niveles[nivel_chapin], Uniforms { shader: Shader::Terreno, celda: chapin::celda(nivel_chapin), ..de_planeta(c) }),
                _ => {
                    let (m, a) = rec.planeta(i, radio_px);
                    // Los planetas de voxels llevan el shader de terreno: cada bloque con su
                    // relieve (mapa de normales) y su variacion, y la noche es la del planeta.
                    let nivel = planetas::nivel_para(radio_px);
                    let uniforms = match planetas::celda_de(c.archivo, nivel) {
                        Some(celda) => Uniforms { shader: Shader::Terreno, celda, ..de_planeta(c) },
                        None => Uniforms { modelo: c.matriz(t), ..base },
                    };
                    DrawCall { animacion: Some((a, anim)), ..llamada(m, uniforms) }
                }
            }
        })
        .collect();
    // El Chapin: el mar (con su shader de olas) y los objetos de la superficie, que giran con el.
    let u_chapin = de_planeta(c_chapin);
    llamadas.push(llamada(&chapin.mar[nivel_chapin], Uniforms { shader: Shader::Oceano, ..u_chapin }));
    let (mut detras_del_chapin, mut por_detalle) = (0, 0);
    // Los objetos, solo de cerca (en los niveles gruesos un voxel es mas grande que un arbol).
    if nivel_chapin >= 2 {
        let hacia_ojo = (ojo - u_chapin.centro).normalize();
        for inst in &chapin.instancias {
            // Nivel de detalle por objeto: lo que mediria menos de 3 pixeles ni se pide.
            let donde = u_chapin.modelo.transform_point(inst.posicion);
            let local = &inst.locales[nivel_chapin];
            let tamano = Vec3::new(local.m[0][1], local.m[1][1], local.m[2][1]).length() * c_chapin.escala_viva();
            let pixeles = tamano / (donde - ojo).length().max(0.1) * pixeles_por_unidad;
            if pixeles < 3.0 {
                por_detalle += 1;
                continue;
            }
            // Del otro lado del planeta no se ven: se descartan antes de entrar al pipeline.
            if u_chapin.modelo.transform_dir(inst.posicion).normalize().dot(hacia_ojo) < -0.12 {
                detras_del_chapin += 1;
                continue;
            }
            llamadas.push(llamada(rec.objetos_chapin.de(inst, pixeles), Uniforms { modelo: u_chapin.modelo * *local, ..base }));
        }
    }
    // Las maravillas: la Endurance girando, la Blip-A con Rocky en su burbuja, el monolito y el
    // disco de acrecion de Gargantua (con el shader de plasma: remolinos de gas al rojo blanco).
    if intro.is_none() {
        let mm = juego.maravillas.matrices(t, juego.reloj);
        llamadas.push(llamada(&rec.endurance, Uniforms { modelo: mm.endurance, ..base }));
        llamadas.push(llamada(&rec.blip_a, Uniforms { modelo: mm.blip, ..base }));
        llamadas.push(llamada(&rec.rocky, Uniforms { modelo: mm.rocky, ..base }));
        llamadas.push(llamada(&rec.monolito, Uniforms { modelo: mm.monolito, ..base }));
        llamadas.push(llamada(
            &rec.disco_gargantua,
            Uniforms { modelo: mm.disco, shader: Shader::Plasma, centro: maravillas::GARGANTUA, borde: Vec3::zero(), tiempo: juego.reloj, intensidad: juego.intensidad, ..base },
        ));
    }
    llamadas.push(llamada(
        &rec.plataforma,
        Uniforms { modelo: Mat4::translation(intro::plataforma(t)) * Mat4::scale(intro::ESCALA_PLATAFORMA), ..base },
    ));
    let m_nave = juego.nave.matriz();
    llamadas.push(DrawCall {
        expresion: Some(intro.map_or(EXPRESION, |i| i.expresion_nave())),
        animacion: Some((rec.anim_nave(), anim)),
        ..llamada(rec.nave(), Uniforms { modelo: m_nave, ..base })
    });
    let asiento = m_nave * rec.lugar_piloto;
    // Miku afuera, parada sobre el planeta donde se aterrizo.
    let miku_afuera = juego.aterrizaje.as_ref().and_then(|a| a.miku(asiento.transform_point(Vec3::zero()), t));
    let saludo = rec.baile().matrices(t, Paso::Saludo, Paso::Saludo, 1.0, 0.0);
    if let Some(m) = miku_afuera {
        llamadas.push(DrawCall {
            expresion: Some(if (t * 0.7) as i32 % 2 == 0 { "guino" } else { "feliz" }),
            por_parte: Some(&saludo),
            ..llamada(rec.miku_baile(), Uniforms { modelo: m, ..base })
        });
    }
    if intro.is_none_or(|i| i.piloto_a_bordo()) && miku_afuera.is_none() {
        llamadas.push(DrawCall {
            expresion: Some(EXPRESION),
            animacion: Some((rec.anim_piloto(), anim)),
            ..llamada(rec.piloto(), Uniforms { modelo: asiento, ..base })
        });
    }
    // Miku bailando en la plataforma (solo en la intro).
    let baile = intro.and_then(|i| {
        let (antes, ahora, mezcla) = i.paso();
        i.miku(asiento.transform_point(Vec3::zero()), t).map(|(m, salto)| (m, rec.baile().matrices(i.t, antes, ahora, mezcla, salto), i.expresion_miku()))
    });
    if let Some((m, partes, expresion)) = &baile {
        llamadas.push(DrawCall {
            expresion: Some(expresion),
            por_parte: Some(partes),
            ..llamada(rec.miku_baile(), Uniforms { modelo: *m, ..base })
        });
    }

    // El publico y los reflectores de la estacion (solo en la intro).
    let barras: Vec<(Mat4, usize, Vec<Mat4>)> = intro.map_or(Vec::new(), |i| {
        i.fans(t).into_iter().map(|(m, color, barra)| (m, color, rec.matrices_fan(barra))).collect()
    });
    for (m, color, partes) in &barras {
        llamadas.push(DrawCall { ocultar: &OCULTAR_FAN[*color], por_parte: Some(partes), ..llamada(&rec.fans, Uniforms { modelo: *m, ..base }) });
    }
    for (m, color) in intro.map_or(Vec::new(), |i| i.haces(t)) {
        llamadas.push(DrawCall { ocultar: &OCULTAR_HAZ[color], ..llamada(&rec.haz, Uniforms { modelo: m, ..base }) });
    }

    // El Mirador (el punto de descanso), con la banca mirando hacia la Tierra.
    let mirador = mirador_en(t);
    if (mirador - ojo).length() < 400.0 {
        let a_tierra = CUERPOS[TIERRA].posicion(t) - mirador;
        let giro = f32::atan2(-a_tierra.x, -a_tierra.z);
        llamadas.push(llamada(&rec.mirador, Uniforms { modelo: Mat4::translation(mirador) * Mat4::rotation_y(giro), ..base }));
    }
    // Las notas para recoger (de un color o del otro).
    let notas = if intro.is_none() { juego.coleccion.visibles(ojo, juego.reloj) } else { Vec::new() };
    for (m, rosa) in &notas {
        let ocultar: &[&str] = if *rosa { &["nota_cian"] } else { &["nota_rosa"] };
        llamadas.push(DrawCall { ocultar, ..llamada(&rec.nota_espacio, Uniforms { modelo: *m, ..base }) });
    }

    // Los asteroides (los rojos y dorados brillan solos).
    let rocas = if intro.is_none() { juego.campo.visibles(ojo) } else { Vec::new() };
    for (m, modelo, clase, _) in &rocas {
        let c = match clase {
            asteroides::Clase::Gris => 0,
            asteroides::Clase::Roja => 1,
            asteroides::Clase::Dorada => 2,
        };
        llamadas.push(llamada(&rec.rocas[c][*modelo], Uniforms { modelo: *m, ..base }));
    }

    // ---------- fondo, modelos y polvo ----------
    fb.limpiar_profundidad();
    rec.cielo.dibujar_fondo(fb, &vista, fov);
    rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, t, juego.salto.estiramiento());
    rec.cielo.dibujar_adornos(fb, &vista, &proyeccion, t, juego.salto.estiramiento());
    let mut st = pipeline::render(fb, &llamadas, &juego.opciones);
    // Los objetos del Chapin que estaban del otro lado tambien cuentan como descartados.
    st.objetos += detras_del_chapin;
    st.descartados += detras_del_chapin;
    st.por_detalle += por_detalle;
    // El portal de la cueva del Planeta Hada, con el STENCIL BUFFER: el casquete
    // del portal marca la mascara (y abre el z-buffer ahi), y la dimension de
    // bolsillo solo se dibuja donde quedo marcado. Por fuera de la ventana no
    // existe: esta adentro del planeta.
    let hada = &CUERPOS[HADA];
    if (hada.posicion(t) - ojo).length() < 90.0 {
        let m_hada = hada.matriz(t);
        let interior = rec.baile().matrices(t, Paso::Baile, Paso::Baile, 1.0, 0.0);
        let miku_adentro = m_hada * Mat4::translation(Vec3::new(0.0, -0.22, -0.35)) * Mat4::rotation_y(0.3 * (t * 0.8).sin()) * Mat4::scale(0.16);
        let portal = [
            DrawCall { stencil: Stencil::Marcar(1), ocultar: &["portal_borde"], ..llamada(&rec.portal, Uniforms { modelo: m_hada, ..base }) },
            DrawCall { ocultar: &["portal_mascara"], ..llamada(&rec.portal, Uniforms { modelo: m_hada, ..base }) },
            DrawCall { stencil: Stencil::Solo(1), ..llamada(&rec.dimension, Uniforms { modelo: m_hada * Mat4::rotation_y(t * 0.15), ..base }) },
            DrawCall {
                stencil: Stencil::Solo(1),
                expresion: Some("guino"),
                por_parte: Some(&interior),
                ..llamada(rec.miku_baile(), Uniforms { modelo: miku_adentro, ..base })
            },
        ];
        let st_portal = pipeline::render(fb, &portal, &juego.opciones);
        st.triangulos += st_portal.triangulos;
        st.dibujados += st_portal.dibujados;
    }
    rec.polvo.dibujar(fb, &vista_proyeccion, ojo, juego.nave.vel);
    if intro.is_none() {
        juego.coleccion.dibujar_luces(fb, &vista_proyeccion, ojo, juego.reloj);
        juego.jugo.dibujar(fb, &vista_proyeccion, ojo);
        // El brillo de los asteroides calientes y de los dorados (para verlos venir de lejos).
        for (m, _, clase, encendida) in &rocas {
            let centro = Vec3::new(m.m[0][3], m.m[1][3], m.m[2][3]);
            let d = (centro - ojo).length().max(5.0);
            let (color, tam) = match (clase, encendida) {
                (asteroides::Clase::Roja, true) => ([0.55, 0.12, 0.02], 1.0),
                (asteroides::Clase::Dorada, true) => ([0.7, 0.55, 0.12], 1.6 + 0.4 * (juego.reloj * 4.0).sin()),
                _ => continue,
            };
            if let Some((x, y, _)) = proyectar(centro, &vista_proyeccion, fb) {
                post::halo(fb, (x, y), tam * (6.0 + 900.0 / d), color, 1.6);
            }
        }
        if let Some(estela) = juego.estela {
            dibujar_estela(fb, &vista_proyeccion, &juego.rastro, estela, juego.reloj);
        }
        // La baliza del Mirador parpadea (para encontrarlo de lejos) y su fogata brilla.
        if (juego.reloj * 1.2).fract() < 0.5 {
            if let Some((x, y, _)) = proyectar(mirador + Vec3::new(0.0, 2.6, 0.0), &vista_proyeccion, fb) {
                let d = (mirador - ojo).length().max(5.0);
                post::halo(fb, (x, y), 8.0 + 1500.0 / d, [0.8, 0.25, 0.55], 1.4);
            }
        }
    }
    if let Some(i) = intro {
        i.dibujar_efectos(fb, &vista_proyeccion, t);
    }

    // ---------- el Sol: corona y destellos, si se ve ----------
    let sol = &CUERPOS[0];
    let centro_sol = sol.posicion(t);
    let distancia = -vista.transform_point(centro_sol).z;
    if let (Some((x, y, _)), true) = (proyectar(centro_sol, &vista_proyeccion, fb), distancia > 0.0) {
        let radio = sol.escala_viva() / distancia * (fb.alto as f32 * 0.5) / (fov * 0.5).tan();
        // Un punto justo delante de su superficie: lo que este mas cerca, lo tapa.
        let frente = centro_sol + (ojo - centro_sol).normalize() * (sol.escala_viva() * 1.15);
        let z_frente = proyectar(frente, &vista_proyeccion, fb).map_or(0.0, |p| p.2);
        // Si el Sol ya llena la pantalla, la corona y los destellos se apagan (si no, todo se quema).
        let lleno = (1.0 - (radio / fb.alto as f32 - 0.25) / 0.35).clamp(0.0, 1.0);
        let visible = post::visibilidad_sol(fb, (x, y), radio, z_frente) * lleno;
        post::sol(fb, (x, y), radio, visible);
        // Los rayos de luz del Sol (los cortan los planetas y la nave que se le pongan enfrente).
        post::rayos_de_luz(fb, (x, y), [1.0, 0.78, 0.5], 0.9 * lleno.max(0.3));
    }
    // La atmosfera de los planetas: un anillo de luz de su color en la orilla.
    for (i, color) in ATMOSFERAS {
        let c = &CUERPOS[i];
        let centro = c.posicion(t);
        let distancia = -vista.transform_point(centro).z;
        if distancia <= 0.5 {
            continue;
        }
        if let Some((x, y, _)) = proyectar(centro, &vista_proyeccion, fb) {
            let radio = c.radio_visual() / distancia * (fb.alto as f32 * 0.5) / (fov * 0.5).tan();
            let frente = centro + (ojo - centro).normalize() * (c.radio_visual() * 1.05);
            let z_frente = proyectar(frente, &vista_proyeccion, fb).map_or(0.0, |p| p.2);
            post::atmosfera(fb, (x, y), radio, color, 0.55, z_frente);
        }
    }
    // Aterrizando: el resplandor de entrada a la atmosfera y el polvo al posarse.
    if let Some(a) = &juego.aterrizaje {
        let k = a.entrada();
        if let (Some((x, y, _)), true) = (proyectar(juego.nave.pos, &vista_proyeccion, fb), k > 0.0) {
            let c = a.color_entrada();
            post::halo(fb, (x, y), 60.0 + 60.0 * k, [c[0] * k * 0.9, c[1] * k * 0.9, c[2] * k * 0.9], 1.4);
        }
        a.dibujar_polvo(fb, &vista_proyeccion, juego.nave.pos, t);
    }
    // Las chispas de cuando Miku entra a la burbuja.
    if let Some(k) = intro.map(|i| i.chispas()).filter(|&k| k > 0.0) {
        if let Some((x, y, _)) = proyectar(asiento.transform_point(Vec3::new(0.0, 0.5, 0.0)), &vista_proyeccion, fb) {
            post::halo(fb, (x, y), 30.0 + 80.0 * (1.0 - k), [k * 0.6, k * 0.9, k], 1.5);
        }
    }

    // ---------- las maravillas: la luz doblada, la linea Petrova, el teseracto ----------
    if intro.is_none() {
        // Gargantua y el agujero de gusano doblan la luz (el lente gravitacional, ver post::lente).
        let pixeles_por_unidad = fb.alto as f32 * 0.5 / (fov * 0.5).tan();
        for (centro, radio, gusano) in [(maravillas::GARGANTUA, maravillas::SOMBRA, false), (maravillas::gusano_en(t), maravillas::RADIO_GUSANO, true)] {
            let distancia = -vista.transform_point(centro).z;
            if distancia <= radio {
                continue;
            }
            if let Some((x, y, z)) = proyectar(centro, &vista_proyeccion, fb) {
                let r = radio / distancia * pixeles_por_unidad;
                let tope = fb.alto as f32 * 1.1;
                if gusano {
                    post::lente(fb, (x, y), z, 0.0, r, (r * 2.4).min(tope), Some((r, [0.75, 0.95, 1.35])));
                    post::halo(fb, (x, y), r * 1.6, [0.10, 0.18, 0.30], 1.3);
                } else {
                    post::lente(fb, (x, y), z, r, r * 1.5, (r * 6.0).min(tope), None);
                }
            }
        }
        maravillas::dibujar_petrova(fb, &vista_proyeccion, t, juego.reloj);
        let mm = juego.maravillas.matrices(t, juego.reloj);
        // El monolito a contraluz: un brillo frio DETRAS de el (la losa negra se recorta).
        let centro = mm.monolito.transform_point(Vec3::new(0.0, 1.1, 0.0));
        let distancia = -vista.transform_point(centro).z;
        if let (Some((x, y, z)), true) = (proyectar(centro, &vista_proyeccion, fb), distancia > 1.0) {
            let r = 9.0 / distancia * pixeles_por_unidad;
            post::halo_detras(fb, (x, y), r * 1.3, [0.30, 0.34, 0.45], 1.6, z);
        }
        // El motor de astrofago de la Blip-A: brilla en infrarrojo (rojo profundo).
        let motor = mm.blip.transform_point(Vec3::new(0.0, 0.0, 3.6));
        let distancia = -vista.transform_point(motor).z;
        if let (Some((x, y, _)), true) = (proyectar(motor, &vista_proyeccion, fb), distancia > 1.0) {
            let r = 5.0 / distancia * pixeles_por_unidad;
            post::halo(fb, (x, y), r * 1.6, [0.55, 0.05, 0.08], 1.4);
        }
    }

    // ---------- post-procesado ----------
    post::sombras_de_contacto(fb, 5, 0.4);
    // Exposicion automatica: si la imagen ya es muy brillante (el Sol de cerca), el bloom baja.
    post.bloom(fb, BLOOM * post::exposicion(fb));
    // El teseracto, encima de todo (ya con el bloom: lineas nitidas).
    maravillas::dibujar_teseracto(fb, juego.maravillas.efecto_teseracto(), juego.reloj);
    // La gradacion del espacio: colores vivos, sombras hacia el violeta y luces calidas.
    post::gradacion(fb, 1.2, 0.22, [0.012, 0.0, 0.03], [0.02, 0.01, -0.01]);
    // Cerca del Sol, un tinte de calor naranja hacia las orillas.
    let (cerca, distancia) = mas_cercano(juego.nave.pos, t);
    let calor = if cerca == 0 && intro.is_none() { (1.0 - distancia / 45.0).clamp(0.0, 1.0) } else { 0.0 };
    if calor > 0.0 && !juego.salto.activo() {
        post::terminar(fb, [1.0, 0.45, 0.1], calor * 0.8, 0.0);
    } else {
        // El paso por el gusano se ve como un salto: la imagen se estira y destella a la mitad.
        let pasaje = juego.maravillas.efecto_pasaje();
        post::terminar(fb, [0.45, 0.65, 1.0], juego.salto.estiramiento().max(pasaje), juego.salto.destello().max(juego.jugo.destello).max(((pasaje - 0.7) * 3.0).max(0.0)));
    }
    st
}

/// Lo que muestra cada pantalla del lab (en el orden de los escritorios, como en
/// `generar_lab.py`): el color con el que brilla.
const PANTALLAS_LAB: [[f32; 3]; 12] = [
    [0.18, 0.28, 0.45],
    [0.30, 0.40, 0.60],
    [0.30, 0.65, 0.30],
    [0.18, 0.28, 0.45],
    [0.0, 0.0, 0.0],
    [0.18, 0.28, 0.45],
    [0.75, 0.15, 0.15],
    [0.18, 0.28, 0.45],
    [0.18, 0.28, 0.45],
    [0.15, 0.35, 0.95],
    [0.30, 0.65, 0.30],
    [0.18, 0.28, 0.45],
];

/// El contenido de las pantallas del lab: cada una lo suyo (en el orden de los escritorios, como
/// en `generar_lab.py`): codigo con colores, YouTube, la pantalla azul de Windows, un jueguito...
/// La del protagonista (la 1) tiene codigo que va bajando.
fn pantallas_del_lab(fb: &mut Framebuffer, vp: &Mat4, escritorios: &[Vec3], t: f32, sin_luz: bool) {
    #[derive(Clone, Copy, PartialEq)]
    enum Que {
        Codigo,
        Mia,
        Juego,
        Negra,
        Youtube,
        Azul,
    }
    const QUE: [Que; 12] = [Que::Codigo, Que::Mia, Que::Juego, Que::Codigo, Que::Negra, Que::Codigo, Que::Youtube, Que::Codigo, Que::Codigo, Que::Azul, Que::Juego, Que::Codigo];
    let colores = [[0.55, 0.35, 0.85], [0.30, 0.55, 0.95], [0.90, 0.60, 0.30], [0.40, 0.85, 0.50], [0.75, 0.75, 0.80]];
    let fuerza = if sin_luz { 1.4 } else { 1.0 };
    for (k, lugar) in escritorios.iter().enumerate() {
        let que = QUE[k % QUE.len()];
        if que == Que::Negra {
            continue;
        }
        // Un punto del vidrio: (u, v) de 0 a 1, de la esquina de arriba a la izquierda.
        let punto = |u: f32, v: f32| *lugar + Vec3::new(-0.35 + 0.7 * u, 1.28 - 0.36 * v, -0.236);
        let linea = |fb: &mut Framebuffer, a: (f32, f32), b: (f32, f32), c: [f32; 3]| {
            if let (Some(p), Some(q)) = (proyectar(punto(a.0, a.1), vp, fb), proyectar(punto(b.0, b.1), vp, fb)) {
                let c = c.map(|v| v * fuerza * 0.9);
                fb.linea_de_luz(p, q, c, c);
            }
        };
        match que {
            Que::Codigo | Que::Mia => {
                let baja = if que == Que::Mia { (t * 1.2) as usize } else { (t * 0.15) as usize };
                for i in 0..9 {
                    let n = i + baja + k * 3;
                    let sangria = ((n * 7) % 4) as f32 * 0.07;
                    let largo = 0.15 + ((n * 13) % 7) as f32 * 0.08;
                    let v = 0.08 + i as f32 * 0.1;
                    let c = colores[(n * 5) % colores.len()];
                    linea(fb, (0.06 + sangria, v), ((0.06 + sangria + largo).min(0.95), v), c);
                    // Una segunda palabra de otro color en algunas lineas.
                    if n % 3 == 0 {
                        let x0 = (0.1 + sangria + largo).min(0.9);
                        linea(fb, (x0, v), (x0 + 0.12, v), colores[(n * 3 + 1) % colores.len()]);
                    }
                }
                // El cursor parpadeando.
                if que == Que::Mia && (t * 2.0).fract() < 0.5 {
                    linea(fb, (0.5, 0.86), (0.5, 0.94), [1.0, 1.0, 1.0]);
                }
            }
            Que::Youtube => {
                // El video (un cuadro rojo y gris), la flechita de play y la barra roja de avance.
                for i in 0..6 {
                    let v = 0.12 + i as f32 * 0.1;
                    linea(fb, (0.08, v), (0.92, v), [0.35, 0.12, 0.12]);
                }
                for i in 0..5 {
                    let d = i as f32 * 0.02;
                    linea(fb, (0.44 + d, 0.28 + d), (0.44 + d, 0.52 - d), [1.0, 1.0, 1.0]);
                }
                let avance = (t * 0.03 + 0.3).fract();
                linea(fb, (0.08, 0.8), (0.08 + 0.84 * avance, 0.8), [1.0, 0.15, 0.15]);
                linea(fb, (0.08 + 0.84 * avance, 0.8), (0.92, 0.8), [0.3, 0.3, 0.3]);
            }
            Que::Azul => {
                // La pantalla azul: la carita triste y unas lineas de texto (y el porcentaje).
                linea(fb, (0.1, 0.18), (0.1, 0.24), [1.0, 1.0, 1.0]);
                linea(fb, (0.1, 0.32), (0.1, 0.38), [1.0, 1.0, 1.0]);
                linea(fb, (0.16, 0.16), (0.2, 0.28), [1.0, 1.0, 1.0]);
                linea(fb, (0.2, 0.28), (0.16, 0.4), [1.0, 1.0, 1.0]);
                for (i, l) in [0.7f32, 0.55, 0.62, 0.3].iter().enumerate() {
                    let v = 0.55 + i as f32 * 0.1;
                    linea(fb, (0.1, v), (0.1 + l * 0.8, v), [0.8, 0.85, 1.0]);
                }
            }
            Que::Juego => {
                // Un jueguito de plataformas: el piso, unos bloques y el monito brincando.
                linea(fb, (0.05, 0.85), (0.95, 0.85), [0.3, 0.9, 0.3]);
                for b in 0..3 {
                    let x = ((b as f32 * 0.37 - t * 0.12).rem_euclid(1.0)) * 0.9 + 0.05;
                    linea(fb, (x, 0.6), (x + 0.08, 0.6), [0.9, 0.7, 0.2]);
                }
                let brinco = ((t * 3.0 + k as f32).sin()).abs() * 0.25;
                linea(fb, (0.3, 0.84 - brinco), (0.3, 0.72 - brinco), [1.0, 0.3, 0.3]);
                linea(fb, (0.32, 0.84 - brinco), (0.32, 0.72 - brinco), [1.0, 0.3, 0.3]);
            }
            Que::Negra => {}
        }
    }
}

/// Los brazos de un personaje sentado tecleando (de su modelo `piloto`): suben y bajan
/// alternados, mas rapido con mas `fuerza`.
fn tecleando(modelo: &Modelo, t: f32, fuerza: f32) -> Vec<Mat4> {
    modelo
        .partes
        .iter()
        .map(|p| {
            let n = p.nombre.as_str();
            let brazo = n.starts_with("brazo") || n.starts_with("manga") || n.starts_with("mano");
            let lado = if n.contains("_izq") { Some(0.0) } else if n.contains("_der") { Some(1.9) } else { None };
            match (brazo, lado) {
                (true, Some(fase)) => Mat4::translation(Vec3::new(0.0, 0.016 * fuerza * (t * (14.0 + 10.0 * fuerza) + fase).sin().max(0.0), 0.0)),
                _ => Mat4::identity(),
            }
        })
        .collect()
}

/// Un cuadro del lab del CIT la noche de la entrega (la Tierra): el lab de noche con su lluvia,
/// los relampagos, las pantallas brillando, el reloj de pared con la hora de verdad, los
/// companeros en sus compus y el protagonista en la suya. Si se va la luz, solo quedan las
/// pantallas (con UPS) y el letrero de SALIDA; si Koch llama por Teams, sale en el proyector
/// como holograma.
/// Los modelos de los objetos del Planeta Chapin (ver `generar_chapin.py`).
struct ObjetosChapin {
    ceiba: Modelo,
    roca: Modelo,
    /// Las versiones de lejos (nivel de detalle bajo: unas cuantas cajas).
    ceiba_lejos: Modelo,
    roca_lejos: Modelo,
    /// La casita, una por cada color de `chapin::COLORES_ANTIGUA`.
    casitas: Vec<Modelo>,
    tikal: Modelo,
    arco: Modelo,
}

impl ObjetosChapin {
    /// El modelo de un objeto, segun cuantos pixeles mide en pantalla (de lejos, el ligero).
    fn de(&self, i: &chapin::Instancia, pixeles: f32) -> &Modelo {
        let lejos = pixeles < 14.0;
        match i.tipo {
            chapin::Tipo::Ceiba if lejos => &self.ceiba_lejos,
            chapin::Tipo::Roca if lejos => &self.roca_lejos,
            chapin::Tipo::Ceiba => &self.ceiba,
            chapin::Tipo::Roca => &self.roca,
            chapin::Tipo::Casita => &self.casitas[i.color % self.casitas.len()],
            chapin::Tipo::Tikal => &self.tikal,
            chapin::Tipo::Arco => &self.arco,
        }
    }
}

/// Un cuadro del salon de Teoria (el examen de Koch): el salon, el profe con su regla (el brazo y
/// la regla giran juntos en el codo para el reglazo) y el protagonista en su pupitre.
fn dibujar_aula(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, aula: &Aula, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira) = aula.camara();
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(FOV.to_radians(), fb.ancho as f32 / fb.alto as f32, CERCA, 60.0);
    // La luz sale de las lamparas del techo; el ambiente es el gris tibio del salon.
    let base = base_uniforms(vista, proyeccion, Vec3::new(0.0, 2.8, 0.5), ojo, (Vec3::new(0.30, 0.28, 0.28), Vec3::new(0.08, 0.06, 0.07)));
    let anim = EstadoAnim { t: aula.total, ..Default::default() };
    let (pos_profe, yaw_profe) = aula.profe();
    let brazo_regla: Vec<Mat4> = rec
        .profe
        .partes
        .iter()
        .map(|p| if p.nombre == "regla" || p.nombre.starts_with("brazo_regla") { aula.brazo_regla() } else { Mat4::identity() })
        .collect();
    let llamadas = [
        llamada(&rec.aula, base),
        DrawCall {
            expresion: Some(aula.expresion_profe()),
            por_parte: Some(&brazo_regla),
            ..llamada(&rec.profe, Uniforms { modelo: Mat4::translation(pos_profe) * Mat4::rotation_y(yaw_profe), ..base })
        },
        DrawCall {
            expresion: Some(aula.expresion_protagonista()),
            animacion: Some((rec.anim_piloto(), anim)),
            ocultar: &SIN_NAVE,
            ..llamada(rec.piloto(), Uniforms { modelo: Mat4::translation(Vec3::new(0.0, 0.46, 0.8)) * aula.reaccion() * Mat4::scale(0.75), ..base })
        },
    ];
    fb.limpiar([20, 18, 24]);
    let st = pipeline::render(fb, &llamadas, opciones);
    post::sombras_de_contacto(fb, 6, 0.5);
    post.bloom(fb, BLOOM * 0.3);
    post::gradacion(fb, 1.16, 0.2, [0.008, 0.0, 0.016], [0.02, 0.01, -0.012]);
    // Mientras mas errores, mas rojo en los bordes; el golpe, un destello blanco.
    post::terminar(fb, [1.0, 0.08, 0.1], aula.enojo() * 0.9, aula.impacto() * 0.25);
    st
}

fn dibujar_entrega(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, en: &Entrega, opciones: &Opciones) -> Estadisticas {
    let mio = rec.escritorios.get(1).copied().unwrap_or(Vec3::new(-1.4, 0.0, -1.6));
    let (ojo, mira) = en.camara(mio);
    let fov = 60f32.to_radians();
    let (cerca, lejos) = (0.05, 40.0);
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, cerca, lejos);
    let vp = proyeccion * vista;
    let t = en.total;
    let sin_luz = en.sin_luz();
    let r = en.relampago;
    let nervios = en.nervios();
    // La luz: la de las lamparas (arriba, sobre la fila del protagonista); sin luz, la de su
    // pantalla (le pega en la cara desde abajo, azulita). Con el relampago, todo se ilumina de azul.
    let (luz, ambiente, borde, color_luz, alcance) = if sin_luz {
        (mio + Vec3::new(0.0, 1.1, -0.05), Vec3::new(0.012, 0.016, 0.035), Vec3::new(0.05, 0.10, 0.28), Vec3::new(0.5, 0.65, 1.15), 1.5)
    } else {
        (Vec3::new(-0.8, 2.95, -0.8), Vec3::new(0.11, 0.12, 0.17), Vec3::new(0.06, 0.08, 0.17), Vec3::new(1.05, 0.98, 0.88), 6.5)
    };
    let ambiente = ambiente + Vec3::new(0.5, 0.55, 0.8) * (r * r);
    let base = Uniforms { color_luz, alcance, ..base_uniforms(vista, proyeccion, luz, ojo, (ambiente, borde)) };
    fb.limpiar([5, 5, 10]);
    let anim = EstadoAnim { t, ..Default::default() };
    // Las manecillas del reloj de pared, con la hora de la noche.
    let (horas, minutos) = en.manecillas();
    let eje = rec.reloj_lab;
    let gira = |a: f32| Mat4::translation(eje) * Mat4::rotation_z(-a) * Mat4::translation(-eje);
    let partes_lab: Vec<Mat4> = rec
        .lab
        .partes
        .iter()
        .map(|p| {
            if p.nombre.starts_with("reloj_minutero") {
                gira(minutos)
            } else if p.nombre.starts_with("reloj_horario") {
                gira(horas)
            } else {
                Mat4::identity()
            }
        })
        .collect();
    let ocultar_lab: &[&str] = if sin_luz { &["lamparas"] } else { &[] };
    let mut llamadas = vec![DrawCall { ocultar: ocultar_lab, por_parte: Some(&partes_lab), ..llamada(&rec.lab, base) }];
    // Los companeros (otros personajes, que no sean el protagonista ni Koch), en sus compus. El de
    // la fila de atras se quedo dormido.
    let elegido = rec.personaje.get();
    let koch = personajes::indice("koch");
    let n = rec.elenco.len();
    let otros: Vec<usize> = (1..n).map(|k| (elegido + k * 5) % n).filter(|&k| k != elegido && k != koch).take(3).collect();
    let sentado = |escritorio: Vec3| Mat4::translation(escritorio + Vec3::new(0.0, 0.5, 0.5)) * Mat4::scale(1.15);
    let ocultar_piloto: &[&str] = &["asiento", "volante"];
    let lugares_companeros = [2usize, 5, 10];
    let partes_companeros: Vec<Vec<Mat4>> = otros.iter().enumerate().map(|(k, &p)| tecleando(&rec.elenco[p].piloto, t + k as f32 * 1.3, if k == 2 { 0.0 } else { 0.5 })).collect();
    for (k, &p) in otros.iter().enumerate() {
        let lugar = rec.escritorios.get(lugares_companeros[k]).copied().unwrap_or(mio);
        let dormido = k == 2;
        let m = if dormido { sentado(lugar) * Mat4::rotation_x(-0.5) } else { sentado(lugar) };
        llamadas.push(DrawCall {
            expresion: Some(if dormido { "espiral" } else { "serio" }),
            ocultar: ocultar_piloto,
            por_parte: Some(&partes_companeros[k]),
            animacion: Some((&rec.sin_animacion, anim)),
            ..llamada(&rec.elenco[p].piloto, Uniforms { modelo: m, ..base })
        });
    }
    // El protagonista: sentado tecleando, caminando por cafe o dormido sobre el teclado.
    let elenco = &rec.elenco[elegido];
    let expresion = if nervios > 0.5 { "asustada" } else if en.terminado() { "feliz" } else { "serio" };
    let (partes_prota, m_prota, de_pie) = match en.prota(mio) {
        entrega::Prota::Sentado(fuerza) => (tecleando(&elenco.piloto, t, fuerza), sentado(mio), false),
        entrega::Prota::Dormido => (tecleando(&elenco.piloto, t, 0.0), sentado(mio) * Mat4::rotation_x(-0.55), false),
        entrega::Prota::Caminando(p, rumbo, paso) => (elenco.baile.caminar(paso), Mat4::translation(p) * Mat4::rotation_y(rumbo) * Mat4::scale(1.05), true),
    };
    llamadas.push(DrawCall {
        expresion: Some(if en.dormido() { "espiral" } else { expresion }),
        ocultar: if de_pie { &[] } else { ocultar_piloto },
        por_parte: Some(&partes_prota),
        animacion: Some((&rec.sin_animacion, anim)),
        ..llamada(if de_pie { &elenco.de_pie } else { &elenco.piloto }, Uniforms { modelo: m_prota, ..base })
    });
    // El guardia haciendo su ronda.
    let ronda = en.guardia();
    let partes_guardia = ronda.map(|(_, _, paso)| rec.guardia_baile.caminar(paso)).unwrap_or_default();
    if let Some((p, rumbo, _)) = ronda {
        llamadas.push(DrawCall {
            expresion: Some("serio"),
            por_parte: Some(&partes_guardia),
            animacion: Some((&rec.sin_animacion, anim)),
            ..llamada(&rec.guardia, Uniforms { modelo: Mat4::translation(p) * Mat4::rotation_y(rumbo) * Mat4::scale(1.1), ..base })
        });
    }
    // Koch por Teams: sale en el proyector, como holograma (aplastado contra la tela, azulito y
    // parpadeando).
    let tela = Vec3::new(0.8, 0.92, -5.36);
    let holograma = en.teams().map(|tt| {
        let k = suave_main(tt / 0.6);
        let parpadeo = 0.85 + 0.15 * (t * 37.0).sin() * (t * 13.0).sin();
        (k, parpadeo)
    });
    if let Some((k, parpadeo)) = holograma {
        let m = Mat4::translation(tela) * Mat4::rotation_y(std::f32::consts::PI) * Mat4::scale_xyz(Vec3::new(0.85 * k, 0.8, 0.05));
        llamadas.push(DrawCall {
            expresion: Some("serio"),
            ..llamada(&rec.profe, Uniforms { modelo: m, ambiente: Vec3::new(0.7, 1.25, 1.8) * parpadeo, borde: Vec3::new(0.6, 1.0, 1.4), alcance: 0.0, ..base })
        });
    }
    let st = pipeline::render(fb, &llamadas, opciones);
    // La niebla del fondo del lab (de noche, lo lejano se pierde).
    post::niebla(fb, cerca, lejos, 6.0, 17.0, [0.012, 0.012, 0.03], false);
    // Lo que tiene cada pantalla del lab (codigo, YouTube, la pantalla azul, un juego), dibujado
    // con lineas de luz sobre el vidrio.
    pantallas_del_lab(fb, &vp, &rec.escritorios, t, sin_luz);
    // La lluvia escurriendo por las ventanas.
    for (v, &kz) in [-3.6f32, -0.6, 2.4].iter().enumerate() {
        for g in 0..18 {
            let semilla = (v * 31 + g * 17) as f32;
            let z = kz - 1.1 + (semilla * 0.618).fract() * 2.2;
            let caida = (t * (0.9 + (semilla * 0.37).fract() * 0.8) + semilla * 0.37).fract();
            let y = 2.55 - caida * 1.5;
            let (a, b) = (Vec3::new(6.40, y, z), Vec3::new(6.40, y - 0.16, z + 0.01));
            if let (Some(pa), Some(pb)) = (proyectar(a, &vp, fb), proyectar(b, &vp, fb)) {
                let brillo = 0.6 + 2.0 * r;
                fb.linea_de_luz(pa, pb, [0.16 * brillo, 0.2 * brillo, 0.28 * brillo], [0.02, 0.03, 0.05]);
            }
        }
        // El relampago: las ventanas se prenden de blanco azulado.
        if r > 0.01 {
            if let Some((x, y, _)) = proyectar(Vec3::new(6.4, 1.8, kz), &vp, fb) {
                post::halo(fb, (x, y), fb.alto as f32 * 0.5 * r, [0.55 * r, 0.6 * r, 0.9 * r], 1.2);
            }
        }
    }
    // Lo que brilla solo se ve si no hay nada enfrente (se compara con el z-buffer).
    let se_ve = |fb: &Framebuffer, p: Vec3| -> Option<(f32, f32, f32)> {
        let (x, y, z) = proyectar(p, &vp, fb)?;
        if x < 0.0 || y < 0.0 || x >= fb.ancho as f32 || y >= fb.alto as f32 {
            return None;
        }
        let i = y as usize * fb.ancho + x as usize;
        (z <= fb.profundidad(i) + 0.003).then_some((x, y, (p - ojo).length()))
    };
    // Las lamparas prendidas (si hay luz).
    if !sin_luz {
        for x in [-4.0f32, 0.0, 4.0] {
            for z in [-3.5f32, 0.0] {
                if let Some((sx, sy, d)) = se_ve(fb, Vec3::new(x, 3.08, z)) {
                    post::halo(fb, (sx, sy), 70.0 / d.max(1.0) + 12.0, [0.22, 0.23, 0.2], 1.6);
                }
            }
        }
    }
    // Las pantallas: cada una con su color (sin luz brillan mas, porque es lo unico que hay).
    let fuerza_pantallas = if sin_luz { 1.6 } else { 0.7 };
    for (k, lugar) in rec.escritorios.iter().enumerate() {
        let c = if k == 1 { [0.35, 0.45, 0.7] } else { PANTALLAS_LAB[k % PANTALLAS_LAB.len()] };
        if c == [0.0; 3] {
            continue;
        }
        if let Some((sx, sy, d)) = se_ve(fb, *lugar + Vec3::new(0.0, 1.1, -0.235)) {
            let parpadea = 0.9 + 0.1 * (t * 3.0 + k as f32).sin();
            post::halo(fb, (sx, sy), 90.0 / d.max(0.8) + 10.0, c.map(|v| v * fuerza_pantallas * 0.5 * parpadea), 1.5);
        }
    }
    // El letrero de SALIDA, la luz de la maquina de cafe y los focos de los CPU.
    let salida = if sin_luz { 1.0 } else { 0.35 };
    if let Some((sx, sy, d)) = se_ve(fb, Vec3::new(-3.9, 2.48, 5.36)) {
        post::halo(fb, (sx, sy), 60.0 / d.max(1.0) + 14.0, [0.05 * salida, 0.5 * salida, 0.15 * salida], 1.5);
    }
    if let Some((sx, sy, d)) = se_ve(fb, Vec3::new(3.75, 1.4, 4.66)) {
        let sin_cafe = if en.sin_cafe() { 0.3 } else { 1.0 };
        post::halo(fb, (sx, sy), 80.0 / d.max(1.0) + 10.0, [0.5 * sin_cafe, 0.3 * sin_cafe, 0.08 * sin_cafe], 1.5);
    }
    // El proyector: el haz de luz (lineas tenues de la lente a la tela) y el brillo de la tela.
    if let Some((k, parpadeo)) = holograma {
        let lente = Vec3::new(0.8, 2.9, -1.53);
        for (dx, dy) in [(-1.8f32, 0.8f32), (1.8, 0.8), (-1.8, -0.8), (1.8, -0.8), (0.0, 0.0), (-0.9, 0.4), (0.9, -0.4)] {
            if let (Some(a), Some(b)) = (proyectar(lente, &vp, fb), proyectar(Vec3::new(0.8 + dx, 1.7 + dy, -5.4), &vp, fb)) {
                fb.linea_de_luz(a, b, [0.12 * k, 0.2 * k, 0.3 * k], [0.02, 0.05, 0.08]);
            }
        }
        if let Some((sx, sy, _)) = proyectar(Vec3::new(0.8, 1.7, -5.4), &vp, fb) {
            post::halo(fb, (sx, sy), fb.alto as f32 * 0.22 * k, [0.04 * parpadeo, 0.1 * parpadeo, 0.18 * parpadeo], 1.4);
        }
        if let Some((sx, sy, _)) = proyectar(lente, &vp, fb) {
            post::halo(fb, (sx, sy), 30.0, [0.5, 0.7, 1.0], 1.5);
        }
    }
    // La linterna del guardia (cuando se va la luz): el foco y su haz.
    if let (true, Some((p, rumbo, _))) = (sin_luz, ronda) {
        let adelante = Vec3::new(-rumbo.sin(), 0.0, -rumbo.cos());
        let mano = p + Vec3::new(0.0, 1.0, 0.0) + adelante * 0.35;
        if let Some((sx, sy, _)) = proyectar(mano, &vp, fb) {
            post::halo(fb, (sx, sy), 40.0, [0.9, 0.85, 0.6], 1.5);
        }
        for k in 0..9 {
            let lado = Vec3::new(adelante.z, 0.0, -adelante.x) * ((k as f32 - 4.0) * 0.18);
            let fin = mano + adelante * 4.5 + lado + Vec3::new(0.0, -0.9, 0.0);
            if let (Some(a), Some(b)) = (proyectar(mano, &vp, fb), proyectar(fin, &vp, fb)) {
                fb.linea_de_luz(a, b, [0.22, 0.2, 0.14], [0.0, 0.0, 0.0]);
            }
        }
    }
    post.bloom(fb, BLOOM * if sin_luz { 1.3 } else { 0.85 });
    // El color de la noche: las sombras tiran a azul y lo claro a calido (como en las peliculas).
    for i in 0..fb.ancho * fb.alto {
        let c = fb.leer(i);
        let luma = 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2];
        let k = (1.0 - luma * 1.6).clamp(0.0, 1.0);
        fb.pintar(i, [c[0] * (1.0 - 0.12 * k) + 0.01 * k, c[1] * (1.0 - 0.05 * k) + 0.012 * k, c[2] * (1.0 + 0.18 * k) + 0.03 * k], 1.0);
    }
    // Los ultimos segundos: la orilla de la pantalla late en rojo (con el corazon).
    let latido = (t * 1.5 * std::f32::consts::TAU).sin().max(0.0).powi(4);
    post::terminar(fb, [1.0, 0.08, 0.08], nervios * (0.35 + 0.45 * latido), r * 0.12);
    st
}

fn suave_main(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// El "proyecto 3" que se ve en el monitor (Graficas): un modelo girando, dibujado por el mismo
/// rasterizador CON EL BUG puesto (de cabeza, estirado, sin z-buffer, por dentro, con la luz al
/// reves, en alambre, en puntos, con el FOV a 150, con el plano cercano lejisimos o con las
/// matrices al reves). Ya arreglado, se ve bien.
fn dibujar_bug(fb: &mut Framebuffer, rec: &Recursos, (bug, arreglado, cual): (entrega::Bug, bool, usize), t: f32) {
    use entrega::Bug;
    // El fondo del visor: un degradado azul oscuro con una cuadricula, como el de un editor 3D.
    for y in 0..fb.alto {
        let k = y as f32 / fb.alto as f32;
        let c = [0.05 + 0.05 * k, 0.06 + 0.06 * k, 0.11 + 0.08 * k];
        for x in 0..fb.ancho {
            let linea = x % 24 == 0 || y % 24 == 0;
            fb.pintar(y * fb.ancho + x, if linea { c.map(|v| v + 0.035) } else { c }, 1.0);
        }
    }
    fb.limpiar_profundidad();
    let roto = |b: Bug| bug == b && !arreglado;
    let ojo = Vec3::new(0.0, 0.7, 3.4);
    let mira = Vec3::new(0.0, 0.1, 0.0);
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let fov = if roto(Bug::Fov) { 150f32 } else { 48.0 }.to_radians();
    let aspecto = if roto(Bug::Aspecto) { 1.0 } else { fb.ancho as f32 / fb.alto as f32 };
    let cerca = if roto(Bug::Cercano) { 3.05 } else { 0.1 };
    let mut proyeccion = Mat4::perspective(fov, aspecto, cerca, 40.0);
    if roto(Bug::YInvertida) {
        proyeccion = Mat4::scale_xyz(Vec3::new(1.0, -1.0, 1.0)) * proyeccion;
    }
    let luz = Vec3::new(2.5, 3.0, 3.5);
    let luz = if roto(Bug::LuzAlReves) { -luz } else { luz };
    let base = base_uniforms(vista, proyeccion, luz, ojo, (Vec3::new(0.16, 0.16, 0.22), Vec3::new(0.15, 0.25, 0.4)));
    let giro = t * 0.7;
    let (modelo, ajuste) = match cual {
        0 => (&rec.naves[rec.skin.get().min(rec.naves.len() - 1)], Mat4::rotation_x(0.25) * Mat4::scale(0.5)),
        1 => (rec.planeta_normal(SATURNO).0, Mat4::rotation_x(0.35) * Mat4::scale(0.36)),
        _ => (&rec.elenco[rec.personaje.get()].de_pie, Mat4::translation(Vec3::new(0.0, -0.75, 0.0)) * Mat4::scale(1.25)),
    };
    // Con las matrices al reves, gira alrededor del origen (lejos) en vez de en su lugar.
    let mut m = if roto(Bug::OrdenMatrices) { Mat4::rotation_y(giro * 1.5) * Mat4::translation(Vec3::new(1.4, 0.0, 0.0)) * ajuste } else { Mat4::rotation_y(giro) * ajuste };
    if roto(Bug::Winding) {
        // El espejo voltea el orden de los vertices: el culling quita las caras de enfrente.
        m = m * Mat4::scale_xyz(Vec3::new(-1.0, 1.0, 1.0));
    }
    let modo = if roto(Bug::Alambre) {
        ModoDibujo::Alambre
    } else if roto(Bug::Puntos) {
        ModoDibujo::Puntos
    } else {
        ModoDibujo::Relleno
    };
    let sin_z = roto(Bug::SinZBuffer);
    let op = Opciones { modo, culling: !roto(Bug::YInvertida) && !sin_z, vista: Vista::Color };
    let anim = EstadoAnim { t, ..Default::default() };
    pipeline::render(
        fb,
        &[DrawCall { stencil: if sin_z { Stencil::SinZBuffer } else { Stencil::Nada }, animacion: Some((&rec.sin_animacion, anim)), ..llamada(modelo, Uniforms { modelo: m, ..base }) }],
        &op,
    );
    // Arreglado: un brillito verde de "compilo".
    if arreglado {
        post::halo(fb, (fb.ancho as f32 * 0.5, fb.alto as f32 * 0.5), fb.ancho as f32 * 0.45, [0.0, 0.12, 0.04], 1.3);
    }
}

/// Las grietas de un bloque que se esta picando (`n` de 0 a 3, de menos a mas): un cubo oscuro
/// translucido, un poquito mas grande que el bloque, que se va poniendo mas negro.
fn grieta(n: usize) -> Modelo {
    let mut o = obra::Obra::new();
    o.material("grieta", [0.05, 0.04, 0.04], [0.0; 3], 0.0, 1.0);
    o.transparente("grieta", 0.1 + 0.12 * n as f32);
    o.caja("grieta", Vec3::new(-0.51, -0.51, -0.51), Vec3::new(0.51, 0.51, 0.51));
    o.modelo()
}

/// Un cuadro de Planeta Vegetta: el cielo del minijuego (de dia el parkour, de tarde TNT Run, de
/// noche la mina; en el lobby el dia pasa), el mundo de bloques, lo que se cae, los pedacitos,
/// Willyrex, Vegetta y el protagonista (en tercera persona). En la mina alumbran las antorchas.
fn dibujar_vegetta(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, v: &Vegetta, opciones: &Opciones) -> Estadisticas {
    use vegetta::{Bloque, Modo};
    let (ojo, mira) = v.camara();
    let fov = vegetta::FOV.to_radians();
    let (cerca, lejos) = (0.05, 240.0);
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, cerca, lejos);
    let vp = proyeccion * vista;
    let t = v.total;
    let adelante = (mira - ojo).normalize();
    let colores = v.cielo();
    let cielo = colores[1];
    cielo_degradado(fb, adelante, fov, colores);
    let noche = cielo.iter().sum::<f32>() < 0.9;
    // En la mina se esta adentro de la tierra: oscuridad de cueva, sin estrellas.
    if v.bajo_tierra() {
        fb.limpiar([7, 5, 10]);
    } else if noche {
        rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, t, 0.0);
    }
    // El sol cuadrado (de tarde, grande y naranja; de noche, la luna), lejos.
    let (direccion, lado, centro, orilla) = match (noche, v.modo) {
        (true, _) => (Vec3::new(0.5, 0.55, 1.0), 16i32, [0.92, 0.94, 1.0], [0.70, 0.74, 0.86]),
        (_, Modo::Tnt) => (Vec3::new(-0.35, 0.16, 1.0), 34, [1.0, 0.80, 0.50], [1.0, 0.55, 0.30]),
        _ => (Vec3::new(-0.4, 0.5, 1.0), 26, [1.0, 1.0, 0.82], [1.0, 0.92, 0.55]),
    };
    let sol = if v.bajo_tierra() { None } else { proyectar(ojo + direccion.normalize() * 120.0, &vp, fb) };
    if let Some((x, y, _)) = sol {
        let k = if noche { 0.35 } else { 1.0 };
        post::halo(fb, (x, y), 90.0, [0.35 * k, 0.33 * k, 0.2 * k], 1.6);
        for dy in -lado..lado {
            for dx in -lado..lado {
                let (px, py) = (x as i32 + dx, y as i32 + dy);
                if px >= 0 && py >= 0 && (px as usize) < fb.ancho && (py as usize) < fb.alto {
                    let borde = dx.abs() > lado - 5 || dy.abs() > lado - 5;
                    fb.pintar(py as usize * fb.ancho + px as usize, if borde { orilla } else { centro }, 1.0);
                }
            }
        }
    }
    fb.limpiar_profundidad();
    // La luz: el sol (o la luna); en la mina, las antorchas de los mineros (enfrente del vidrio,
    // titilando, y se apagan con la distancia).
    let mut base = if v.bajo_tierra() {
        let y = v.prota.pos.y.min(v.willy.pos.y + 6.0) + 2.0;
        let mut b = base_uniforms(vista, proyeccion, Vec3::new(0.5, y, 4.5), ojo, (Vec3::new(0.20, 0.18, 0.24), Vec3::new(0.12, 0.08, 0.05)));
        b.color_luz = Vec3::new(1.2, 0.88, 0.58);
        b.alcance = 10.5 + 0.6 * (t * 11.0).sin() + 0.3 * (t * 23.0).sin();
        b
    } else if noche {
        let mut b = base_uniforms(vista, proyeccion, Vec3::new(40.0, 60.0, 70.0), ojo, (Vec3::new(0.22, 0.24, 0.36), Vec3::new(0.10, 0.12, 0.22)));
        b.color_luz = Vec3::new(0.55, 0.62, 0.88);
        b
    } else if v.modo == Modo::Tnt {
        let mut b = base_uniforms(vista, proyeccion, Vec3::new(-40.0, 25.0, 110.0), ojo, (Vec3::new(0.40, 0.33, 0.40), Vec3::new(0.30, 0.16, 0.12)));
        b.color_luz = Vec3::new(1.05, 0.84, 0.64);
        b
    } else {
        base_uniforms(vista, proyeccion, Vec3::new(-50.0, 70.0, 60.0), ojo, (Vec3::new(0.42, 0.44, 0.50), Vec3::new(0.15, 0.18, 0.25)))
    };
    base.tiempo = t;
    let anim = EstadoAnim { t, ..Default::default() };
    let cubo = |b: Bloque| &rec.cubos[Bloque::TODOS.iter().position(|&o| o == b).unwrap_or(0)];
    let mut llamadas = vec![llamada(&v.modelo, base)];
    // Alrededor del servidor: islas flotantes y un mar de nubes. Siguen de lejos a la camara, asi
    // no estorban en la pista y siempre hay algo que ver.
    if !v.bajo_tierra() {
        let de = &rec.decorado;
        let cerca_de = |x: f32, y: f32, z: f32| Vec3::new(ojo.x * 0.86 + x, ojo.y + y, ojo.z * 0.86 + z);
        for (k, (x, y, z, cual)) in [(-62.0, -8.0, 48.0, 0), (58.0, -2.0, 62.0, 2), (-38.0, 10.0, 88.0, 1), (86.0, -12.0, 12.0, 1), (-92.0, -4.0, -14.0, 2), (46.0, -10.0, -78.0, 0), (-48.0, 6.0, -70.0, 1), (12.0, 14.0, 104.0, 0)].into_iter().enumerate() {
            let flota = (t * 0.5 + k as f32 * 1.7).sin() * 0.8;
            llamadas.push(llamada(&rec.decorado.islas[cual], Uniforms { modelo: Mat4::translation(cerca_de(x, y + flota, z)) * Mat4::rotation_y(k as f32 * 0.8), ..base }));
        }
        for k in 0..18 {
            let semilla = k as f32 * 2.399;
            let pasa = (t * 0.7 + semilla * 40.0).rem_euclid(260.0) - 130.0;
            let (lado, y) = if k < 11 { ((semilla * 13.0).sin() * 120.0, -46.0 - (k % 3) as f32 * 4.0) } else { ((semilla * 13.0).sin() * 130.0, 28.0 + (k % 4) as f32 * 8.0) };
            llamadas.push(llamada(&de.nubes[k % 2], Uniforms { modelo: Mat4::translation(cerca_de(pasa, y, lado)) * Mat4::scale(1.0 + (k % 3) as f32 * 0.4), ..base }));
        }
    }
    for m in v.matrices_diamantes() {
        llamadas.push(DrawCall { animacion: Some((&rec.sin_animacion, anim)), ..llamada(&rec.diamante_mc, Uniforms { modelo: m, ..base }) });
    }
    // Lo que se cae en TNT Run (girando un poquito) y los pedacitos de lo que se pica.
    for (k, &(b, p, vel)) in v.cayendo.iter().enumerate() {
        let giro = vel * 0.04 * if k % 2 == 0 { 1.0 } else { -1.0 };
        llamadas.push(llamada(cubo(b), Uniforms { modelo: Mat4::translation(p) * Mat4::rotation_x(giro) * Mat4::rotation_z(giro * 0.7), ..base }));
    }
    for (k, pd) in v.particulas.iter().enumerate() {
        llamadas.push(llamada(cubo(pd.bloque), Uniforms { modelo: Mat4::translation(pd.pos) * Mat4::rotation_y(k as f32) * Mat4::scale(pd.tam), ..base }));
    }
    if let Some(((x, y, z), avance)) = v.picando {
        let etapa = ((avance * 4.0) as usize).min(3);
        llamadas.push(llamada(&rec.grietas[etapa], Uniforms { modelo: Mat4::translation(Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5)), ..base }));
    }
    // El protagonista: caminando en el piso (o picando); en el aire, con los brazos abiertos.
    let elenco = &rec.elenco[rec.personaje.get()];
    let partes_prota = if v.picando.is_some() {
        elenco.baile.caminar(t * 9.0)
    } else if v.en_suelo() {
        elenco.baile.caminar(v.paso() * 3.2)
    } else {
        elenco.baile.matrices(t, Paso::Giro, Paso::Giro, 1.0, 0.0)
    };
    llamadas.push(DrawCall { expresion: Some(if v.en_meta() { "feliz" } else { "serio" }), por_parte: Some(&partes_prota), animacion: Some((&rec.sin_animacion, anim)), ..llamada(&elenco.de_pie, Uniforms { modelo: v.matriz_prota(), ..base }) });
    let willy = &rec.elenco[personajes::indice("willyrex")];
    let (m_willy, paso_willy) = v.willy();
    let partes_willy = willy.baile.caminar(paso_willy);
    llamadas.push(DrawCall { expresion: Some("feliz"), por_parte: Some(&partes_willy), animacion: Some((&rec.sin_animacion, anim)), ..llamada(&willy.de_pie, Uniforms { modelo: m_willy, ..base }) });
    let vegetta = &rec.elenco[personajes::indice("vegetta")];
    let partes_vegetta = vegetta.baile.matrices(t, Paso::Saludo, Paso::Baile, 0.5 + 0.5 * (t * 0.5).sin(), 0.0);
    llamadas.push(DrawCall { expresion: Some("guino"), por_parte: Some(&partes_vegetta), animacion: Some((&rec.sin_animacion, anim)), ..llamada(&vegetta.de_pie, Uniforms { modelo: v.vegetta(), ..base }) });
    let st = pipeline::render(fb, &llamadas, opciones);
    // Niebla del color del cielo de lejos (sin taparlo); en la mina no hay.
    if !v.bajo_tierra() {
        post::niebla(fb, cerca, lejos, 70.0, 210.0, cielo, true);
    }
    // El brillo de los diamantes que faltan y de los portales del lobby.
    for m in v.matrices_diamantes() {
        let p = m.transform_point(Vec3::new(0.0, 0.25, 0.0));
        if (p - ojo).length() < 40.0 {
            if let Some((x, y, _)) = proyectar(p, &vp, fb) {
                post::halo(fb, (x, y), 22.0, [0.1, 0.35 + 0.1 * (t * 4.0).sin(), 0.35], 1.4);
            }
        }
    }
    for (p, _, _) in v.letreros_portales() {
        if let Some((x, y, _)) = proyectar(p - Vec3::new(0.0, 3.3, 0.0), &vp, fb) {
            post::halo(fb, (x, y), 60.0, [0.20 + 0.05 * (t * 3.0).sin(), 0.06, 0.32], 1.5);
        }
    }
    // La antorcha del protagonista en la mina: un brillito calido en la mano.
    if v.bajo_tierra() {
        if let Some((x, y, _)) = proyectar(v.prota.pos + Vec3::new(0.0, 1.0, 0.4), &vp, fb) {
            post::halo(fb, (x, y), 46.0 + 4.0 * (t * 13.0).sin(), [0.42, 0.24, 0.06], 1.5);
        }
    }
    post::sombras_de_contacto(fb, 5, 0.45);
    v.jugo.dibujar(fb, &vp, ojo);
    post.bloom(fb, BLOOM * if v.bajo_tierra() { 0.9 } else { 0.5 });
    post::gradacion(fb, 1.14, 0.12, [0.0, 0.0, 0.015], [0.012, 0.006, 0.0]);
    post::terminar(fb, [1.0, 1.0, 1.0], 0.0, 0.0);
    st
}

/// Un charco: un disco de agua translucido de radio 1, pegado al piso (el juego lo escala).
fn charco() -> Modelo {
    let mut agua = obj::Material::nuevo("charco");
    agua.kd = Vec3::new(0.30, 0.58, 0.95);
    agua.ks = Vec3::new(0.9, 0.9, 0.9);
    agua.ns = 80.0;
    agua.ke = Vec3::new(0.05, 0.10, 0.18);
    agua.d = 0.7;
    let arriba = Vec3::new(0.0, 1.0, 0.0);
    let v = |p: Vec3| obj::Vertice { pos: p, normal: arriba, uv: [0.0, 0.0] };
    let lados = 18;
    let triangulos = (0..lados)
        .map(|k| {
            // Un poquito irregular, como un charco de verdad.
            let borde = |k: usize| {
                let a = k as f32 / lados as f32 * std::f32::consts::TAU;
                let r = 1.0 + 0.18 * (a * 3.0).sin() + 0.1 * (a * 5.0 + 1.0).cos();
                Vec3::new(a.cos() * r, 0.0, a.sin() * r)
            };
            [v(Vec3::zero()), v(borde(k + 1)), v(borde(k))]
        })
        .collect();
    Modelo::nuevo(vec![obj::Parte { nombre: "charco".into(), material: 0, triangulos, comentario: String::new() }], vec![agua])
}

/// Un cuadro del baño de Doña Chepa: el baño visto desde arriba, las puertas de los cubiculos
/// (giran en su bisagra), Doña Chepa en su banco, los clientes, el protagonista con lo que carga
/// y los charcos.
fn dibujar_urano(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, u: &Urano, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira) = u.camara();
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(urano::FOV.to_radians(), fb.ancho as f32 / fb.alto as f32, 0.1, 220.0);
    let t = u.total;
    // La luz sale de la lampara del fondo; el ambiente es el blanco frio del baño.
    let base = base_uniforms(vista, proyeccion, Vec3::new(0.0, 6.0, 1.5), ojo, (Vec3::new(0.34, 0.36, 0.42), Vec3::new(0.10, 0.12, 0.16)));
    fb.limpiar([26, 120, 140]);
    let anim = EstadoAnim { t, ..Default::default() };
    let mut partes_bano = vec![Mat4::identity(); rec.bano_chepa.partes.len()];
    for (n, &(i, bisagra)) in rec.bisagras.iter().enumerate() {
        if let Some(m) = partes_bano.get_mut(i) {
            *m = u.puerta(n, bisagra);
        }
    }
    let mut llamadas = vec![DrawCall { por_parte: Some(&partes_bano), ..llamada(&rec.bano_chepa, base) }];
    // El baño flota sobre las nubes de Urano, con su anillo y todo lo que se le ha volado a la Chepa.
    let de = &rec.decorado;
    let corre = (t * 0.8).rem_euclid(32.0);
    llamadas.push(llamada(&de.bandas_urano, Uniforms { modelo: Mat4::translation(Vec3::new(corre, -34.0, 0.0)), ..base }));
    llamadas.push(llamada(&de.anillo_urano, Uniforms { modelo: Mat4::translation(Vec3::new(0.0, -9.0, 0.0)) * Mat4::rotation_y(t * 0.04) * Mat4::rotation_x(0.1), ..base }));
    for k in 0..16 {
        let semilla = k as f32 * 2.399;
        let a = semilla + t * 0.035;
        let r = 12.0 + (semilla * 5.0).sin().abs() * 11.0;
        let p = Vec3::new(a.cos() * r * 1.25, -2.5 - (k % 5) as f32 * 1.5 + (t * 0.6 + semilla).sin() * 0.35, a.sin() * r);
        let modelo = match k % 3 { 0 => &de.rollo, 1 => &de.hielo, _ => &de.patito };
        llamadas.push(llamada(modelo, Uniforms { modelo: Mat4::translation(p) * Mat4::rotation_y(t * 0.5 + semilla) * Mat4::rotation_x(t * 0.3 + semilla) * Mat4::scale(1.5), ..base }));
    }
    // Doña Chepa en su banco, detras del mostrador, viendo a la fila.
    llamadas.push(llamada(&rec.chepa, Uniforms { modelo: Mat4::translation(urano::CHEPA), ..base }));
    let aliens = u.aliens_3d();
    for (m, color, cara) in &aliens {
        llamadas.push(DrawCall { expresion: Some(cara), ..llamada(&rec.aliens[*color % rec.aliens.len()], Uniforms { modelo: *m, ..base }) });
    }
    // El protagonista caminando (el modelo mira a -Z: de ahi la media vuelta).
    let elenco = &rec.elenco[rec.personaje.get()];
    let partes_prota = elenco.baile.caminar(u.jugador.paso);
    let m_prota = Mat4::translation(u.jugador.pos) * Mat4::rotation_y(u.jugador.rumbo + std::f32::consts::PI);
    llamadas.push(DrawCall { expresion: Some("feliz"), por_parte: Some(&partes_prota), animacion: Some((&rec.sin_animacion, anim)), ..llamada(&elenco.de_pie, Uniforms { modelo: m_prota, ..base }) });
    if let Some((item, m)) = u.carga_3d() {
        llamadas.push(llamada(&rec.items_urano[item as usize], Uniforms { modelo: m, ..base }));
    }
    for c in &u.charcos {
        llamadas.push(llamada(&rec.charco, Uniforms { modelo: Mat4::translation(c.pos + Vec3::new(0.0, 0.012, 0.0)) * Mat4::scale(c.tam), ..base }));
    }
    let st = pipeline::render(fb, &llamadas, opciones);
    post::sombras_de_contacto(fb, 6, 0.55);
    u.jugo.dibujar(fb, &(proyeccion * vista), ojo);
    post.bloom(fb, BLOOM * 0.35);
    post::gradacion(fb, 1.12, 0.12, [0.0, 0.006, 0.014], [0.0, 0.0, 0.0]);
    post::terminar(fb, [1.0, 0.2, 0.2], u.sacudida * 0.2 + u.dano * 0.35, 0.0);
    st
}

/// Un cuadro del concierto en Saturno (en las coordenadas de Saturno).
fn dibujar_concierto(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, c: &Concierto, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira) = c.camara();
    let fov = FOV.to_radians();
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.05, 200.0);
    let golpe = c.ritmo();
    // El ambiente se enciende un poco en cada golpe.
    let luz = (LUZ_CONCIERTO.0 * (1.0 + 0.6 * golpe), LUZ_CONCIERTO.1);
    let base = base_uniforms(vista, proyeccion, Vec3::new(3.0, 8.0, -6.0), ojo, luz);
    let anim = EstadoAnim { t: c.t(), ritmo: golpe, viraje: c.vaiven() * 3.0, ..Default::default() };
    let mut llamadas = vec![
        DrawCall {
            animacion: Some((rec.planeta_normal(SATURNO).1, anim)),
            ocultar: &["microfono"],
            ..llamada(rec.planeta_normal(SATURNO).0, base)
        },
        llamada(&rec.luna, Uniforms { modelo: c.columpio(), ..base }),
        DrawCall {
            expresion: Some(c.expresion_miku()),
            animacion: Some((rec.anim_piloto(), anim)),
            ocultar: &SIN_NAVE,
            ..llamada(rec.piloto(), Uniforms { modelo: c.miku(rec.asiento_luna), ..base })
        },
    ];
    for (m, color) in c.haces() {
        llamadas.push(DrawCall { ocultar: &OCULTAR_HAZ[color], ..llamada(&rec.haz, Uniforms { modelo: m, ..base }) });
    }
    fb.limpiar_profundidad();
    rec.cielo.dibujar_fondo(fb, &vista, fov);
    rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, c.t(), 0.0);
    rec.cielo.dibujar_adornos(fb, &vista, &proyeccion, c.t(), 0.0);
    let st = pipeline::render(fb, &llamadas, opciones);
    c.dibujar_chispas(fb, &(proyeccion * vista));
    post.bloom(fb, 0.55 + 0.35 * golpe);
    post::terminar(fb, [1.0, 0.4, 0.8], 0.25 * golpe, 0.0);
    st
}

/// Un cuadro de Saturno Ritmo (en las coordenadas de Saturno, como el concierto).
fn dibujar_ritmo(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, r: &Ritmo, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira) = r.camara();
    let fov = (FOV + r.fov_extra()).to_radians();
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.05, 200.0);
    let golpe = r.golpe();
    let luz = (LUZ_CONCIERTO.0 * (1.0 + 0.6 * golpe), LUZ_CONCIERTO.1);
    let base = base_uniforms(vista, proyeccion, Vec3::new(3.0, 8.0, -6.0), ojo, luz);
    let anim = EstadoAnim { t: r.t(), ritmo: golpe, giro_turbinas: r.t() * 6.0, empuje: 0.6, ..Default::default() };
    let t = r.t();
    let mut llamadas = vec![
        // Los bichitos del modelo se ocultan: los del minijuego van aparte.
        DrawCall { animacion: Some((rec.planeta_normal(SATURNO).1, anim)), ocultar: &["microfono", "nota_"], ..llamada(rec.planeta_normal(SATURNO).0, base) },
        // Miku canta arriba en su luna mientras la nave juega.
        llamada(&rec.luna, Uniforms { modelo: concierto::columpio_en(t), ..base }),
        DrawCall {
            expresion: Some("feliz"),
            animacion: Some((rec.anim_piloto(), anim)),
            ocultar: &SIN_NAVE,
            ..llamada(rec.piloto(), Uniforms { modelo: concierto::miku_en(t, rec.asiento_luna), ..base })
        },
        DrawCall {
            expresion: Some(r.expresion_miku()),
            animacion: Some((rec.anim_nave(), anim)),
            ..llamada(rec.nave(), Uniforms { modelo: r.nave(), ..base })
        },
    ];
    for (m, aire) in r.notas_visibles() {
        let ocultar: &[&str] = if aire { &["nota_piso"] } else { &["nota_aire"] };
        llamadas.push(DrawCall { ocultar, ..llamada(&rec.notas_ritmo, Uniforms { modelo: m, ..base }) });
    }
    for (m, color) in concierto::haces_en(t) {
        llamadas.push(DrawCall { ocultar: &OCULTAR_HAZ[color], ..llamada(&rec.haz, Uniforms { modelo: m, ..base }) });
    }
    fb.limpiar_profundidad();
    rec.cielo.dibujar_fondo(fb, &vista, fov);
    rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, t, 0.0);
    rec.cielo.dibujar_adornos(fb, &vista, &proyeccion, t, 0.0);
    let st = pipeline::render(fb, &llamadas, opciones);
    r.dibujar_efectos(fb, &(proyeccion * vista));
    post.bloom(fb, 0.6 + 0.35 * golpe);
    post::terminar(fb, [1.0, 0.4, 0.8], 0.2 * golpe, 0.0);
    st
}

/// Un cuadro de la junta de emergencia (en las coordenadas de Pluton).
fn dibujar_junta(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, j: &Junta, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira) = j.camara();
    let fov = FOV.to_radians();
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.05, 300.0);
    let base = base_uniforms(vista, proyeccion, Vec3::new(-3.0, 6.0, -8.0), ojo, LUZ_ESPACIO);
    let t = j.total;
    let anim = EstadoAnim { t, ritmo: ritmo(t), giro_turbinas: t * 3.0, ..Default::default() };
    let mut llamadas = vec![
        // Pluton en alambre: "ya no lo rellenan". Si gana el si, vuelve a rellenarse.
        DrawCall {
            expresion: Some(j.expresion_pluton()),
            modo: if j.relleno() { None } else { Some(ModoDibujo::Alambre) },
            ..llamada(rec.planeta_detallado(PLUTON).0, Uniforms { modelo: j.pluton(), ..base })
        },
        // La nave estacionada atras de Pluton.
        DrawCall {
            expresion: Some("feliz"),
            animacion: Some((rec.anim_nave(), anim)),
            ..llamada(rec.nave(), Uniforms { modelo: Mat4::translation(Vec3::new(2.3, 1.1, 1.8)) * Mat4::rotation_y(-0.6) * Mat4::scale(0.45), ..base })
        },
    ];
    // La sala: piso a cuadros y la mesa con el boton.
    llamadas.push(llamada(&rec.mesa_junta, Uniforms { modelo: Mat4::translation(MESA), ..base }));
    for (k, (i, tripulante, planeta)) in j.consejo().into_iter().enumerate() {
        llamadas.push(DrawCall { ocultar: &OCULTAR_TRIPULANTE[k], ..llamada(&rec.tripulantes, Uniforms { modelo: tripulante, ..base }) });
        let (modelo_planeta, anim_planeta) = rec.planeta_normal(i);
        llamadas.push(DrawCall { animacion: Some((anim_planeta, anim)), ..llamada(modelo_planeta, Uniforms { modelo: planeta, ..base }) });
    }
    let pasos = if j.miku_baila() { (Paso::Baile, Paso::Baile) } else { (Paso::Saludo, Paso::Saludo) };
    let partes = rec.baile().matrices(t, pasos.0, pasos.1, 1.0, 0.0);
    llamadas.push(DrawCall {
        expresion: Some(j.expresion_miku()),
        por_parte: Some(&partes),
        ..llamada(rec.miku_baile(), Uniforms { modelo: j.miku(), ..base })
    });
    fb.limpiar_profundidad();
    rec.cielo.dibujar_fondo(fb, &vista, fov);
    rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, t, 0.0);
    rec.cielo.dibujar_adornos(fb, &vista, &proyeccion, t, 0.0);
    let st = pipeline::render(fb, &llamadas, opciones);
    post::sombras_de_contacto(fb, 5, 0.45);
    // Un reflector sobre quien habla.
    let vista_proyeccion = proyeccion * vista;
    if j.foco().length() > 0.1 {
        if let Some((x, y, _)) = proyectar(j.foco(), &vista_proyeccion, fb) {
            post::halo(fb, (x, y), 70.0, [0.35, 0.3, 0.12], 1.6);
        }
    }
    if let Some(Some((x, y, _))) = j.boton().map(|b| proyectar(b, &vista_proyeccion, fb)) {
        post::halo(fb, (x, y), 60.0, [0.9 * j.alarma(), 0.05, 0.05], 1.6);
    }
    post.bloom(fb, BLOOM);
    post::terminar(fb, [1.0, 0.05, 0.1], j.alarma(), j.destello());
    st
}

/// Un cuadro de la cita en Marte, al atardecer.
fn dibujar_cita(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, c: &Cita, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira) = c.camara();
    let fov = FOV.to_radians();
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.05, 300.0);
    let base = base_uniforms(vista, proyeccion, Vec3::new(-4.0, 6.0, -6.0), ojo, LUZ_MARTE);
    let t = c.total;
    // El cielo: de color caramelo en el horizonte a morado oscuro arriba (con el atardecer azul
    // del momento sorpresa, azul cerca del Sol; con la tormenta, todo naranja de polvo).
    let adelante = (mira - ojo).normalize();
    let colores = match c.momento() {
        Some(cita::ATARDECER) => [[0.45, 0.62, 0.95], [0.40, 0.32, 0.55], [0.06, 0.05, 0.18]],
        Some(cita::TORMENTA) => [[0.95, 0.60, 0.35], [0.80, 0.48, 0.32], [0.45, 0.25, 0.20]],
        _ => [[0.86, 0.58, 0.42], [0.50, 0.30, 0.42], [0.08, 0.05, 0.16]],
    };
    cielo_degradado(fb, adelante, fov, colores);
    fb.limpiar_profundidad();
    rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, t, 0.0);
    // El Sol chiquito de Marte, bajito, con su resplandor azul.
    let vista_proyeccion = proyeccion * vista;
    if let Some((x, y, _)) = proyectar(Vec3::new(14.0, 3.2, 60.0), &vista_proyeccion, fb) {
        if c.momento() == Some(cita::ATARDECER) {
            post::halo(fb, (x, y), 700.0, [0.15, 0.35, 0.9], 1.2);
        }
        post::halo(fb, (x, y), 320.0, [0.10, 0.28, 0.75], 1.4);
        post::halo(fb, (x, y), 90.0, [0.25, 0.45, 1.0], 1.6);
        post::halo(fb, (x, y), 10.0, [0.75, 0.85, 1.0], 0.6);
    }
    let anim = EstadoAnim { t, ..Default::default() };
    let mut llamadas = vec![
        llamada(&rec.cita_marte, base),
        DrawCall { expresion: Some(c.expresion_curiosity()), ..llamada(&rec.curiosity, Uniforms { modelo: c.curiosity(), ..base }) },
    ];
    for (k, m) in c.candidatas() {
        llamadas.push(DrawCall { expresion: Some(c.expresion_candidata(k)), ..llamada(&rec.candidatas[k], Uniforms { modelo: m, ..base }) });
    }
    let paso = if c.miku_baila() { Paso::Baile } else { Paso::Saludo };
    let partes = rec.baile().matrices(t, paso, paso, 1.0, 0.0);
    llamadas.push(DrawCall {
        expresion: Some(c.expresion_miku()),
        por_parte: Some(&partes),
        animacion: Some((&rec.sin_animacion, anim)),
        ..llamada(rec.miku_baile(), Uniforms { modelo: c.miku(), ..base })
    });
    let st = pipeline::render(fb, &llamadas, opciones);
    // Un reflector suavecito sobre quien habla.
    if let Some((x, y, _)) = c.foco().and_then(|f| proyectar(f, &vista_proyeccion, fb)) {
        post::halo(fb, (x, y), 80.0, [0.3, 0.15, 0.2], 1.8);
    }
    let (w, h) = (fb.ancho as f32, fb.alto as f32);
    match c.momento() {
        // La tormenta de polvo: rafagas de polvo que cruzan la pantalla.
        Some(cita::TORMENTA) => {
            for k in 0..60u32 {
                let semilla = k.wrapping_mul(2_654_435_761);
                let y = (semilla % 1000) as f32 / 1000.0 * h;
                let vel = 0.4 + ((semilla / 1000) % 100) as f32 / 120.0;
                let x = ((t * vel + (semilla / 7 % 1000) as f32 / 1000.0) % 1.0) * w * 1.2 - w * 0.1;
                post::halo(fb, (x, y), 18.0 + (k % 5) as f32 * 8.0, [0.25, 0.14, 0.06], 1.4);
            }
        }
        // Fobos cruzando el cielo, con estrellas fugaces.
        Some(cita::FOBOS) => {
            let u = (t * 0.05) % 1.0;
            post::halo(fb, (w * (0.1 + 0.8 * u), h * 0.12), 26.0, [0.6, 0.55, 0.5], 1.2);
            post::halo(fb, (w * (0.1 + 0.8 * u), h * 0.12), 8.0, [0.9, 0.85, 0.8], 0.6);
            for k in 0..4u32 {
                let f = (t * 0.7 + k as f32 * 0.27) % 1.0;
                let x0 = w * (0.2 + 0.2 * k as f32);
                let (a, b) = ((x0 + f * 120.0, h * 0.05 + f * 60.0), (x0 + f * 120.0 - 40.0, h * 0.05 + f * 60.0 - 20.0));
                let brillo = (1.0 - f) * 0.9;
                fb.linea_de_luz((a.0, a.1, 0.0), (b.0, b.1, 0.0), [brillo, brillo, brillo * 1.1], [0.0; 3]);
            }
        }
        _ => {}
    }
    // La golpiza: una nube de polvo enorme con estrellitas y pedazos de rover volando.
    if let Some(g) = c.golpiza().filter(|&g| g < 1.0) {
        if let Some((x, y, _)) = proyectar(Vec3::new(-0.9, 0.6, 0.0), &vista_proyeccion, fb) {
            let r = h * 0.22 * (0.6 + 0.4 * (g * 8.0).min(1.0));
            for k in 0..14 {
                let a = k as f32 / 14.0 * std::f32::consts::TAU + t * 3.0;
                let rr = r * (0.55 + 0.25 * (t * 7.0 + k as f32).sin());
                post::halo(fb, (x + a.cos() * rr, y + a.sin() * rr * 0.6), r * 0.45, [0.30, 0.20, 0.12], 1.3);
            }
            for k in 0..8 {
                let a = k as f32 * 2.4 + t * 9.0;
                post::halo(fb, (x + a.cos() * r * 0.8, y + a.sin() * r * 0.5), 10.0, [1.0, 0.95, 0.5], 0.8);
            }
        }
    }
    // Los corazones de luz, siempre de frente a la camara.
    let derecha = adelante.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
    let arriba = derecha.cross(adelante);
    for (centro, tam, brillo) in c.corazones() {
        corazon_de_luz(fb, &vista_proyeccion, centro, derecha, arriba, tam, [1.0 * brillo, 0.3 * brillo, 0.6 * brillo]);
    }
    post.bloom(fb, BLOOM);
    let rosa = if c.terminada() && matches!(c.final_(), Final::Exito | Final::Boda) { 0.15 } else { 0.05 };
    post::terminar(fb, [1.0, 0.4, 0.7], rosa, 0.0);
    st
}

/// Tamano (en pixeles del framebuffer propio) del retrato de GLADAS.
const RETRATO: usize = 160;

/// La cara que pone GLADAS: casi siempre sarcastica, a ratos contenta o
/// guinando, y parpadea cada tantito.
fn expresion_gladas(t: f32) -> &'static str {
    if (t % 3.3) < 0.12 {
        return "dormida";
    }
    match (t % 7.0) as i32 {
        0..=3 => "serio",
        4 | 5 => "feliz",
        _ => "guino",
    }
}

/// RENDER TO TEXTURE: GLADAS se dibuja con el mismo pipeline, pero en un
/// framebuffer propio (chiquito y con su propia camara); despues ese
/// framebuffer se sube a una textura aparte y la interfaz la pega junto a lo
/// que dice. La cabeza se mece, y mas cuando esta hablando.
fn dibujar_retrato(fb: &mut Framebuffer, rec: &Recursos, t: f32, hablando: bool) {
    fb.limpiar([22, 16, 44]);
    let ojo = Vec3::new(0.38, 0.28, -1.55);
    let vista = Mat4::look_at(ojo, Vec3::new(0.05, 0.26, 0.2), Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(40f32.to_radians(), 1.0, 0.05, 20.0);
    let base = base_uniforms(vista, proyeccion, Vec3::new(-1.5, 2.5, -3.0), ojo, (Vec3::new(0.30, 0.26, 0.38), Vec3::new(0.35, 0.30, 0.6)));
    let animo = if hablando { 1.0 } else { 0.4 };
    let m = Mat4::translation(Vec3::new(0.0, 0.03 * (t * 2.1).sin() * animo, 0.0))
        * Mat4::rotation_y(0.15 * (t * 0.9).sin() * animo)
        * Mat4::rotation_z(0.10 * (t * 1.3).sin() * animo)
        * Mat4::rotation_x(0.06 * (t * 7.0).sin().abs() * hablando as i32 as f32);
    let op = Opciones { modo: ModoDibujo::Relleno, culling: true, vista: Vista::Color };
    pipeline::render(fb, &[DrawCall { expresion: Some(expresion_gladas(t)), ..llamada(&rec.gladas, Uniforms { modelo: m, ..base }) }], &op);
}

/// Miku de cuerpo entero bailando, con el fondo transparente: para ponerla en grande en los menus
/// (como los personajes en los menus de Persona). Va en su propio framebuffer.
fn dibujar_miku_menu(fb: &mut Framebuffer, rec: &Recursos, t: f32) {
    fb.limpiar_transparente();
    let ojo = Vec3::new(-0.5, 1.0, -3.3);
    let vista = Mat4::look_at(ojo, Vec3::new(0.0, 0.66, 0.0), Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(38f32.to_radians(), fb.ancho as f32 / fb.alto as f32, 0.05, 20.0);
    let base = base_uniforms(vista, proyeccion, Vec3::new(-1.5, 2.5, -3.0), ojo, (Vec3::new(0.26, 0.25, 0.34), Vec3::new(0.18, 0.45, 0.55)));
    let partes = rec.baile().matrices(t, Paso::Baile, Paso::Baile, 1.0, 0.0);
    let expresion = if (t * 0.5) as i32 % 3 == 2 { "guino" } else { "feliz" };
    let op = Opciones { modo: ModoDibujo::Relleno, culling: true, vista: Vista::Color };
    let anim = EstadoAnim { t, ..Default::default() };
    pipeline::render(
        fb,
        &[DrawCall { expresion: Some(expresion), por_parte: Some(&partes), animacion: Some((&rec.sin_animacion, anim)), ..llamada(rec.miku_baile(), base) }],
        &op,
    );
}

/// Pega a Miku (la de `dibujar_miku_menu`) en grande a la derecha, con su sombra de color corrida.
fn pegar_miku(d: &mut RaylibDrawHandle, textura: &Texture2D, ancho: i32, alto: i32) {
    let h = alto as f32 * 0.98;
    let w = h * textura.width as f32 / textura.height as f32;
    let x = ancho as f32 - w * 0.86;
    let y = alto as f32 - h * 0.97;
    let fuente = Rectangle::new(0.0, 0.0, textura.width as f32, textura.height as f32);
    d.draw_texture_pro(textura, fuente, Rectangle::new(x + 18.0, y + 10.0, w, h), Vector2::zero(), 0.0, Color::new(80, 230, 230, 150));
    d.draw_texture_pro(textura, fuente, Rectangle::new(x, y, w, h), Vector2::zero(), 0.0, Color::WHITE);
}

/// La vitrina de la tienda, el casillero y el pase: la nave o un personaje.
fn dibujar_vitrina(fb: &mut Framebuffer, rec: &Recursos, muestra: Muestra, t: f32) {
    match muestra {
        Muestra::Nave { skin, estela } => dibujar_vitrina_nave(fb, rec, skin, estela, t),
        Muestra::Personaje(i) => dibujar_vitrina_personaje(fb, rec, i, t),
        Muestra::Recuerdo { cual, tiene } => {
            fondo_de_vitrina(fb, t);
            post::halo(fb, (fb.ancho as f32 * 0.5, fb.alto as f32 * 0.55), fb.ancho as f32 * 0.25, [0.12, 0.08, 0.26], 1.6);
            dibujar_recuerdo(fb, rec, cual, t);
            if !tiene {
                // Si todavia no se tiene: solo su sombra (un misterio).
                for i in 0..fb.ancho * fb.alto {
                    if fb.profundidad(i).is_finite() {
                        fb.pintar(i, [0.09, 0.06, 0.18], 1.0);
                    }
                }
            }
        }
    }
}

/// Como se pone cada recuerdo de la mochila para la foto: cuanto se gira de lado, hacia
/// adelante y de canto (las lunas no estan: esas giran solas).
const POSE_RECUERDOS: [(f32, f32, f32); mochila::CUANTOS] = [
    (0.35, 0.0, 0.08),   // la carta del tarot
    (0.45, 0.15, 0.0),   // el anillo de Saturno
    (0.40, 0.25, -0.40), // la regla de Koch, de diagonal
    (0.40, 0.10, 0.0),   // la claqueta
    (0.30, 0.0, 0.12),   // la foto de Marte
    (0.40, 0.40, 0.0),   // el frasco (desde arriba se ve la tapa)
    (0.35, 0.10, 0.0),   // el vinil
    (0.35, 0.0, 0.10),   // la carta +4
    (0.30, 0.0, -0.06),  // la credencial de Pluton
    (0.45, 0.10, 0.0),   // el Contenedor de Corazon
    (0.50, 0.35, 0.0),   // el hielo de cometa
    (0.30, 0.0, 0.0),    // el autografo de Bill
    (0.00, 0.20, 0.0),   // el cinturon de campeon
    (0.70, 0.50, 0.0),   // el bloque de diamante
    (-0.75, 0.15, 0.0),  // el patito del referi (de tres cuartos)
];

/// Una luna (0..4) o un recuerdo (4..) de la mochila, encuadrado por lo que mide y meciendose
/// con `t`, sobre lo que ya tenga el framebuffer.
fn dibujar_recuerdo(fb: &mut Framebuffer, rec: &Recursos, cual: usize, t: f32) {
    let modelo = if cual < 4 { &rec.lunas_galileo[cual] } else { &rec.recuerdos[cual - 4] };
    let (giro, adelante, canto) = if cual < 4 { (t * 0.8, 0.35, 0.0) } else { POSE_RECUERDOS[cual - 4] };
    let giro = giro + if cual < 4 { 0.0 } else { 0.45 * (t * 0.9).sin() };
    let (mut min, mut max) = (Vec3::new(f32::MAX, f32::MAX, f32::MAX), Vec3::new(f32::MIN, f32::MIN, f32::MIN));
    for v in modelo.partes.iter().flat_map(|p| p.triangulos.iter().flatten()) {
        min = Vec3::new(min.x.min(v.pos.x), min.y.min(v.pos.y), min.z.min(v.pos.z));
        max = Vec3::new(max.x.max(v.pos.x), max.y.max(v.pos.y), max.z.max(v.pos.z));
    }
    let (centro, radio) = ((min + max) * 0.5, ((max - min).length() * 0.5).max(0.01));
    let flota = 0.04 * (t * 1.7).sin();
    let m = Mat4::translation(Vec3::new(0.0, flota, 0.0)) * Mat4::rotation_y(giro) * Mat4::rotation_x(adelante) * Mat4::rotation_z(canto) * Mat4::scale(1.0 / radio) * Mat4::translation(centro * -1.0);
    let ojo = Vec3::new(0.0, 0.25, -4.0);
    let vista = Mat4::look_at(ojo, Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(30f32.to_radians(), fb.ancho as f32 / fb.alto as f32, 0.05, 20.0);
    let base = base_uniforms(vista, proyeccion, Vec3::new(-2.0, 3.0, -4.0), ojo, (Vec3::new(0.32, 0.30, 0.40), Vec3::new(0.25, 0.45, 0.60)));
    let op = Opciones { modo: ModoDibujo::Relleno, culling: true, vista: Vista::Color };
    let anim = EstadoAnim { t, ..Default::default() };
    pipeline::render(fb, &[DrawCall { animacion: Some((&rec.sin_animacion, anim)), ..llamada(modelo, Uniforms { modelo: m, ..base }) }], &op);
}

/// El fondo de la vitrina: un degradado lila oscuro con estrellitas que titilan.
fn fondo_de_vitrina(fb: &mut Framebuffer, t: f32) {
    for y in 0..fb.alto {
        let k = y as f32 / fb.alto as f32;
        let rgb = [0.05 + 0.10 * k, 0.03 + 0.04 * k, 0.14 + 0.16 * k];
        for x in 0..fb.ancho {
            fb.pintar(y * fb.ancho + x, rgb, 1.0);
        }
    }
    fb.limpiar_profundidad();
    let (ancho, alto) = (fb.ancho as f32, fb.alto as f32);
    for k in 0..40u32 {
        // Estrellitas fijas (un hash del indice), que titilan.
        let h = k.wrapping_mul(2654435761);
        let (x, y) = ((h % 1000) as f32 / 1000.0 * ancho, ((h / 1000) % 1000) as f32 / 1000.0 * alto * 0.7);
        let brillo = 0.25 + 0.2 * (t * 2.0 + k as f32).sin();
        post::halo(fb, (x, y), 2.5, [brillo, brillo, brillo * 1.2], 2.0);
    }
}

/// El retrato de un personaje (la cabeza, de frente y sonriendo) para su tarjeta del casillero.
fn dibujar_retrato_personaje(fb: &mut Framebuffer, rec: &Recursos, i: usize) {
    dibujar_retrato_de(fb, &rec.elenco[i].de_pie);
}

/// El retrato (la cabeza, de frente y sonriendo) de un modelo de pie con el esqueleto chibi.
fn dibujar_retrato_de(fb: &mut Framebuffer, de_pie: &Modelo) {
    fb.limpiar_transparente();
    struct P<'a> {
        de_pie: &'a Modelo,
    }
    let p = P { de_pie };
    // A la altura de su cabeza (Kirby es pura cabeza y esta abajo; el Creeper la tiene mas arriba).
    let (mut abajo, mut arriba) = (f32::MAX, f32::MIN);
    for v in p.de_pie.partes.iter().filter(|q| q.nombre.starts_with("cabeza")).flat_map(|q| q.triangulos.iter().flatten()) {
        abajo = abajo.min(v.pos.y);
        arriba = arriba.max(v.pos.y);
    }
    let centro = if abajo < arriba { (abajo + arriba) * 0.5 - 0.02 } else { 0.84 };
    let ojo = Vec3::new(-0.35, centro + 0.16, -1.75);
    let vista = Mat4::look_at(ojo, Vec3::new(0.0, centro, 0.0), Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(30f32.to_radians(), fb.ancho as f32 / fb.alto as f32, 0.05, 20.0);
    let base = base_uniforms(vista, proyeccion, Vec3::new(-1.5, 2.5, -3.0), ojo, (Vec3::new(0.30, 0.28, 0.38), Vec3::new(0.18, 0.45, 0.55)));
    let op = Opciones { modo: ModoDibujo::Relleno, culling: true, vista: Vista::Color };
    pipeline::render(fb, &[DrawCall { expresion: Some("feliz"), ..llamada(&p.de_pie, base) }], &op);
}

/// Un personaje bailando en la vitrina, de cuerpo entero, sobre su pedestal de luz.
fn dibujar_vitrina_personaje(fb: &mut Framebuffer, rec: &Recursos, i: usize, t: f32) {
    fondo_de_vitrina(fb, t);
    let p = &rec.elenco[i];
    let ojo = Vec3::new(-0.45, 0.85, -2.6);
    let mira = Vec3::new(0.0, 0.62, 0.0);
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(36f32.to_radians(), fb.ancho as f32 / fb.alto as f32, 0.05, 20.0);
    if let Some(c) = proyectar(Vec3::zero(), &(proyeccion * vista), fb) {
        post::halo(fb, (c.0, c.1), fb.ancho as f32 * 0.22, [0.10, 0.30, 0.34], 1.6);
    }
    let base = base_uniforms(vista, proyeccion, Vec3::new(-1.5, 2.5, -3.0), ojo, (Vec3::new(0.26, 0.25, 0.34), Vec3::new(0.18, 0.45, 0.55)));
    let partes = p.baile.matrices(t, Paso::Baile, Paso::Baile, 1.0, 0.0);
    let expresion = if (t * 0.5) as i32 % 3 == 2 { "guino" } else { "feliz" };
    let op = Opciones { modo: ModoDibujo::Relleno, culling: true, vista: Vista::Color };
    let anim = EstadoAnim { t, ..Default::default() };
    pipeline::render(
        fb,
        &[DrawCall { expresion: Some(expresion), por_parte: Some(&partes), animacion: Some((&rec.sin_animacion, anim)), ..llamada(&p.de_pie, base) }],
        &op,
    );
}

/// La nave con la skin `skin` girando sobre un pedestal de luz. Con `estela` (en la pestana de
/// estelas), en cambio vuela en circulos dejando esa estela.
fn dibujar_vitrina_nave(fb: &mut Framebuffer, rec: &Recursos, skin: usize, estela: Option<&str>, t: f32) {
    let nave = &rec.naves[skin];
    // Lo que mide la nave, para encuadrarla (sea cual sea su escala en el .obj).
    let (mut min, mut max) = (Vec3::new(f32::MAX, f32::MAX, f32::MAX), Vec3::new(f32::MIN, f32::MIN, f32::MIN));
    for v in nave.partes.iter().flat_map(|p| p.triangulos.iter().flatten()) {
        min = Vec3::new(min.x.min(v.pos.x), min.y.min(v.pos.y), min.z.min(v.pos.z));
        max = Vec3::new(max.x.max(v.pos.x), max.y.max(v.pos.y), max.z.max(v.pos.z));
    }
    let centro = (min + max) * 0.5;
    let r = (max - min).length() * 0.5;
    fondo_de_vitrina(fb, t);
    let (ancho, alto) = (fb.ancho as f32, fb.alto as f32);
    let vuela = estela.is_some();
    let ojo = if vuela { Vec3::new(0.0, r * 1.7, r * 3.7) } else { Vec3::new(0.0, r * 0.55, r * 2.7) };
    let vista = Mat4::look_at(ojo, Vec3::new(0.0, if vuela { -r * 0.2 } else { -r * 0.1 }, 0.0), Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(36f32.to_radians(), ancho / alto, 0.05 * r, 40.0 * r);
    let base = base_uniforms(vista, proyeccion, Vec3::new(-3.0 * r, 4.0 * r, 5.0 * r), ojo, (Vec3::new(0.24, 0.22, 0.34), Vec3::new(0.30, 0.55, 0.70)));
    // El pedestal: un resplandor cian debajo de la nave.
    if !vuela {
        if let Some(p) = proyectar(Vec3::new(0.0, -r * 0.75, 0.0), &(proyeccion * vista), fb) {
            post::halo(fb, (p.0, p.1), ancho * 0.30, [0.10, 0.30, 0.34], 1.6);
        }
    }
    let anim = EstadoAnim { t, giro_turbinas: t * 3.0, ..Default::default() };
    let radio_vuelo = r * 1.5;
    let camino = |a: f32| Vec3::new(radio_vuelo * a.cos(), r * 0.25 * (2.0 * a).sin(), radio_vuelo * a.sin());
    let angulo = t * 0.9;
    let modelo = if vuela {
        // Hacia adelante (su -Z) va por la tangente del circulo, ladeada hacia adentro.
        Mat4::translation(camino(angulo)) * Mat4::rotation_y(std::f32::consts::PI - angulo) * Mat4::rotation_z(-0.35) * Mat4::translation(centro * -1.0)
    } else {
        Mat4::rotation_y(t * 0.7) * Mat4::rotation_x(0.08 * (t * 1.3).sin()) * Mat4::translation(centro * -1.0)
    };
    let op = Opciones { modo: ModoDibujo::Relleno, culling: true, vista: Vista::Color };
    pipeline::render(fb, &[DrawCall { expresion: Some(EXPRESION), animacion: Some((&rec.anim_naves[skin], anim)), ..llamada(nave, Uniforms { modelo, ..base }) }], &op);
    if let Some(estela) = estela.filter(|e| !e.is_empty()) {
        let rastro: Vec<Vec3> = (0..48).map(|k| camino(angulo - (47 - k) as f32 * 0.06)).collect();
        // Dos veces (y una corrida a un lado): en la vitrina la nave se ve chica y la estela, delgada.
        let vp = proyeccion * vista;
        dibujar_estela(fb, &vp, &rastro, estela, t);
        let corrida: Vec<Vec3> = rastro.iter().map(|&p| p + Vec3::new(0.0, r * 0.03, 0.0)).collect();
        dibujar_estela(fb, &vp, &corrida, estela, t);
    }
}

/// Un planeta (o luna) para su sello del pasaporte: de tres cuartos, iluminado de lado, con fondo
/// transparente. Se encuadra solo, por lo que mida el modelo (Saturno con todo y anillos).
fn dibujar_miniatura_planeta(fb: &mut Framebuffer, modelo: &Modelo, anim: &Animaciones) {
    fb.limpiar_transparente();
    let (mut min, mut max) = (Vec3::new(f32::MAX, f32::MAX, f32::MAX), Vec3::new(f32::MIN, f32::MIN, f32::MIN));
    for v in modelo.partes.iter().filter(|p| !p.nombre.starts_with("cara_")).flat_map(|p| p.triangulos.iter().flatten()) {
        min = Vec3::new(min.x.min(v.pos.x), min.y.min(v.pos.y), min.z.min(v.pos.z));
        max = Vec3::new(max.x.max(v.pos.x), max.y.max(v.pos.y), max.z.max(v.pos.z));
    }
    let centro = (min + max) * 0.5;
    let r = (max - min).length() * 0.5;
    let ojo = Vec3::new(r * 0.9, r * 0.9, -r * 2.4);
    let vista = Mat4::look_at(ojo, Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(36f32.to_radians(), 1.0, 0.05 * r, 40.0 * r);
    let base = base_uniforms(vista, proyeccion, Vec3::new(-4.0 * r, 3.0 * r, -3.0 * r), ojo, (Vec3::new(0.16, 0.14, 0.24), Vec3::new(0.30, 0.40, 0.60)));
    let op = Opciones { modo: ModoDibujo::Relleno, culling: true, vista: Vista::Color };
    let modelo_m = Mat4::translation(centro * -1.0);
    pipeline::render(fb, &[DrawCall { expresion: Some("feliz"), animacion: Some((anim, EstadoAnim::default())), ..llamada(modelo, Uniforms { modelo: modelo_m, ..base }) }], &op);
}

/// La miniatura de una skin (la nave de tres cuartos) o de una estela (la nave chiquita dejando
/// la estela en curva), con fondo transparente: para las tarjetas del pase.
fn dibujar_miniatura_nave(fb: &mut Framebuffer, rec: &Recursos, skin: usize, estela: Option<&str>) {
    fb.limpiar_transparente();
    let nave = &rec.naves[skin];
    let (mut min, mut max) = (Vec3::new(f32::MAX, f32::MAX, f32::MAX), Vec3::new(f32::MIN, f32::MIN, f32::MIN));
    for v in nave.partes.iter().flat_map(|p| p.triangulos.iter().flatten()) {
        min = Vec3::new(min.x.min(v.pos.x), min.y.min(v.pos.y), min.z.min(v.pos.z));
        max = Vec3::new(max.x.max(v.pos.x), max.y.max(v.pos.y), max.z.max(v.pos.z));
    }
    let centro = (min + max) * 0.5;
    let r = (max - min).length() * 0.5;
    let lejos = if estela.is_some() { 3.6 } else { 2.3 };
    let ojo = Vec3::new(r * 0.35 * lejos, r * 0.25 * lejos, r * lejos);
    let mira = if estela.is_some() { Vec3::new(-r * 0.6, -r * 0.4, 0.0) } else { Vec3::zero() };
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(36f32.to_radians(), fb.ancho as f32 / fb.alto as f32, 0.05 * r, 40.0 * r);
    let base = base_uniforms(vista, proyeccion, Vec3::new(-3.0 * r, 4.0 * r, 5.0 * r), ojo, (Vec3::new(0.26, 0.24, 0.36), Vec3::new(0.30, 0.55, 0.70)));
    // Con estela, la nave va saliendo de una curva (de abajo a la izquierda hacia arriba a la derecha).
    let modelo = if estela.is_some() {
        Mat4::rotation_y(-2.3) * Mat4::rotation_z(0.3) * Mat4::translation(centro * -1.0)
    } else {
        // De frente (su cara mira a -Z) y un poco de lado.
        Mat4::rotation_y(std::f32::consts::PI + 0.55) * Mat4::rotation_x(-0.12) * Mat4::translation(centro * -1.0)
    };
    let op = Opciones { modo: ModoDibujo::Relleno, culling: true, vista: Vista::Color };
    pipeline::render(fb, &[DrawCall { expresion: Some(EXPRESION), animacion: Some((&rec.anim_naves[skin], EstadoAnim { t: 1.2, ..Default::default() })), ..llamada(nave, Uniforms { modelo, ..base }) }], &op);
    if let Some(estela) = estela {
        let vp = proyeccion * vista;
        let curva = |k: f32| {
            // De la cola (k = 0) a la nave (k = 1): una curva cuadratica.
            let (a, b, c) = (Vec3::new(-r * 2.6, -r * 2.0, r * 0.6), Vec3::new(-r * 0.4, -r * 2.0, r * 0.3), Vec3::zero());
            a * ((1.0 - k) * (1.0 - k)) + b * (2.0 * k * (1.0 - k)) + c * (k * k)
        };
        for dy in [-0.06, -0.02, 0.02, 0.06] {
            let rastro: Vec<Vec3> = (0..40).map(|k| curva(k as f32 / 39.0) + Vec3::new(0.0, r * dy, 0.0)).collect();
            dibujar_estela(fb, &vp, &rastro, estela, 1.0);
        }
    }
    fb.opacar_luz();
}

/// Pinta el cielo de una escena con un degradado segun la altura de cada fila
/// sobre el horizonte: `colores` son el del horizonte, el de en medio y el de arriba.
fn cielo_degradado(fb: &mut Framebuffer, adelante: Vec3, fov: f32, [horizonte, medio, arriba]: [[f32; 3]; 3]) {
    let focal = fb.alto as f32 * 0.5 / (fov * 0.5).tan();
    let inclinacion = adelante.y.asin();
    for y in 0..fb.alto {
        let altura = inclinacion + ((fb.alto as f32 * 0.5 - y as f32) / focal).atan();
        let k = (altura / 0.55).clamp(0.0, 1.0);
        let rgb: [f32; 3] = if k < 0.4 {
            [0, 1, 2].map(|i| horizonte[i] + (medio[i] - horizonte[i]) * k / 0.4)
        } else {
            [0, 1, 2].map(|i| medio[i] + (arriba[i] - medio[i]) * (k - 0.4) / 0.6)
        };
        for x in 0..fb.ancho {
            fb.pintar(y * fb.ancho + x, rgb, 1.0);
        }
    }
}

/// Un cuadro de la noche de cartas en Neptuno, en plena tormenta.
fn dibujar_neptuno(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, n: &Neptuno, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira) = n.camara();
    let fov = FOV.to_radians();
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.05, 300.0);
    let vista_proyeccion = proyeccion * vista;
    let base = base_uniforms(vista, proyeccion, Vec3::new(-2.0, 7.0, -6.0), ojo, LUZ_NEPTUNO);
    let t = n.total;
    cielo_degradado(fb, (mira - ojo).normalize(), fov, [[0.20, 0.42, 0.85], [0.08, 0.18, 0.55], [0.02, 0.03, 0.14]]);
    fb.limpiar_profundidad();
    rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, t, 0.0);

    let anim = EstadoAnim { t, ..Default::default() };
    let mut llamadas = vec![
        // Neptuno, el anfitrion, con su cara.
        DrawCall {
            expresion: Some(n.expresion_neptuno()),
            ..llamada(rec.planeta_detallado(NEPTUNO).0, Uniforms { modelo: n.neptuno(), ..base })
        },
        // La mesa (la de la junta, sin el boton ni el piso).
        DrawCall { ocultar: &["boton", "piso"], ..llamada(&rec.mesa_junta, Uniforms { modelo: Mat4::scale(neptuno::ESCALA_MESA), ..base }) },
    ];
    for (figura, m, expresion) in n.figuras() {
        let modelo = match figura {
            neptuno::Figura::Rival(j) => &rec.rivales[j - 1],
            neptuno::Figura::Canela => &rec.canela,
            neptuno::Figura::Pluton => rec.planeta_detallado(PLUTON).0,
        };
        llamadas.push(DrawCall { expresion, ..llamada(modelo, Uniforms { modelo: m, ..base }) });
    }
    let paso = if n.miku_baila() { Paso::Baile } else { Paso::Saludo };
    let partes = rec.baile().matrices(t, paso, paso, 1.0, 0.0);
    llamadas.push(DrawCall {
        expresion: Some(n.expresion_miku()),
        por_parte: Some(&partes),
        animacion: Some((&rec.sin_animacion, anim)),
        ..llamada(rec.miku_baile(), Uniforms { modelo: n.miku(), ..base })
    });
    // La lluvia de diamantes, lejos, cayendo en diagonal con el viento.
    for k in 0..10 {
        let u = (t * (0.1 + 0.02 * (k % 3) as f32) * n.viento().min(2.0) + k as f32 * 0.37).fract();
        let p = Vec3::new(9.0 - 18.0 * u + (k as f32 * 2.9) % 5.0, 7.0 - 9.0 * u, 5.0 + (k % 4) as f32 * 2.0);
        let m = Mat4::translation(p) * Mat4::rotation_y(t * 2.0 + k as f32) * Mat4::rotation_z(0.6) * Mat4::scale(0.6);
        llamadas.push(llamada(&rec.diamante, Uniforms { modelo: m, ..base }));
    }
    let cartas = n.cartas();
    for (m, ocultar) in &cartas {
        llamadas.push(DrawCall { ocultar, ..llamada(&rec.carta, Uniforms { modelo: *m, ..base }) });
    }
    let st = pipeline::render(fb, &llamadas, opciones);
    // Las rayas del viento de 2,000 km/h (mas largas en una rafaga).
    let viento = n.viento();
    for k in 0..40 {
        let rapidez = 9.0 + (k % 5) as f32 * 2.0;
        let x = 10.0 - ((t * rapidez * viento + k as f32 * 7.3) % 20.0);
        let y = -0.5 + (k as f32 * 1.37) % 6.5;
        let z = 3.0 + (k as f32 * 0.91) % 9.0;
        let largo = 0.8 * viento;
        if let (Some(a), Some(b)) = (proyectar(Vec3::new(x, y, z), &vista_proyeccion, fb), proyectar(Vec3::new(x + largo, y, z), &vista_proyeccion, fb)) {
            let c = 0.12 * viento.min(2.5);
            fb.linea_de_luz(a, b, [c * 0.6, c * 0.8, c * 1.2], [0.0; 3]);
        }
    }
    // Un reflector sobre el que esta pensando su jugada.
    if let Some((x, y, _)) = n.foco().and_then(|f| proyectar(f, &vista_proyeccion, fb)) {
        post::halo(fb, (x, y), 90.0, [0.25, 0.22, 0.1], 1.8);
    }
    post::sombras_de_contacto(fb, 5, 0.45);
    post.bloom(fb, BLOOM * 0.55);
    post::gradacion(fb, 1.1, 0.14, [0.0, 0.0, 0.015], [0.0, 0.0, 0.0]);
    post::terminar(fb, [0.3, 0.5, 1.0], 0.04, 0.0);
    st
}

/// Una aurora boreal en el cielo, como la del Proyecto 2: cortinas de luz
/// que ondulan, con rayos verticales, de verde a rosa. Se suma (luz aditiva)
/// solo arriba del horizonte.
fn aurora(fb: &mut Framebuffer, t: f32) {
    let (w, h) = (fb.ancho as f32, fb.alto as f32);
    for y in 0..(fb.alto as f32 * 0.55) as usize {
        let v = y as f32 / h;
        for x in 0..fb.ancho {
            let u = x as f32 / w;
            let mut luz = 0.0;
            for (k, (alto, ancho)) in [(0.16, 0.05), (0.27, 0.07)].iter().enumerate() {
                let centro = alto + 0.05 * (u * 6.0 + t * 0.35 + k as f32 * 2.0).sin() + 0.025 * (u * 17.0 - t * 0.8).sin();
                let d = (v - centro) / ancho;
                let cortina = (-d * d).exp() * if d > 0.0 { 1.0 } else { (1.0 + d * 0.5).max(0.0) };
                let rayos = 0.55 + 0.45 * (u * 90.0 + (u * 13.0 + t).sin() * 3.0).sin();
                luz += cortina * rayos * (0.6 - 0.25 * k as f32);
            }
            if luz > 0.01 {
                let mezcla = (0.5 + 0.5 * (u * 3.0 + t * 0.2).sin()).clamp(0.0, 1.0);
                let color = [0.15 + 0.85 * mezcla, 0.9 - 0.5 * mezcla, 0.55 + 0.25 * mezcla];
                fb.sumar(y * fb.ancho + x, color.map(|c| c * luz * 0.7));
            }
        }
    }
}

/// Un cuadro de la Gran Fuente de las Hadas.
fn dibujar_fuente(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, f: &Fuente, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira) = f.camara();
    let fov = FOV.to_radians();
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.05, 300.0);
    let vista_proyeccion = proyeccion * vista;
    let base = base_uniforms(vista, proyeccion, Vec3::new(-4.0, 9.0, -6.0), ojo, LUZ_FUENTE);
    let t = f.total;
    // El cielo: mar de nubes rosado abajo, noche morada arriba, estrellas y la aurora.
    cielo_degradado(fb, (mira - ojo).normalize(), fov, [[0.62, 0.40, 0.66], [0.16, 0.10, 0.34], [0.03, 0.02, 0.10]]);
    fb.limpiar_profundidad();
    rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, t, 0.0);
    aurora(fb, t);
    let anim = EstadoAnim { t, ..Default::default() };
    let ocultar = f.ocultar();
    let paso = f.paso_miku();
    let partes = rec.baile().matrices(t, paso, paso, 1.0, 0.0);
    let mut llamadas = vec![
        DrawCall { ocultar: &ocultar, ..llamada(&rec.fuente_hadas, base) },
        DrawCall {
            expresion: Some(f.expresion_miku()),
            por_parte: Some(&partes),
            animacion: Some((&rec.sin_animacion, anim)),
            // Su propia luz, del lado de la camara: cuando la Gran Hada le da el corazon, la camara
            // la ve de frente y la luz de la escena le queda atras (se veia toda negra).
            ..llamada(rec.miku_baile(), Uniforms { modelo: f.miku(), luz: ojo + Vec3::new(1.5, 4.0, 0.0), ..base })
        },
        llamada(&rec.navi, Uniforms { modelo: Mat4::translation(f.navi()) * Mat4::rotation_y(t * 6.0) * Mat4::scale(0.8), ..base }),
    ];
    if let Some(m) = f.hada() {
        llamadas.push(DrawCall { expresion: Some(f.expresion_hada()), ..llamada(&rec.luka, Uniforms { modelo: m, ..base }) });
    }
    if let Some(m) = f.contenedor() {
        llamadas.push(llamada(&rec.contenedor, Uniforms { modelo: m, ..base }));
    }
    let st = pipeline::render(fb, &llamadas, opciones);
    f.dibujar_efectos(fb, &vista_proyeccion);
    f.dibujar_luces(fb, &vista_proyeccion);
    post.bloom(fb, BLOOM * post::exposicion(fb) * 0.6);
    post::gradacion(fb, 1.12, 0.16, [0.01, 0.0, 0.02], [0.0, 0.0, 0.0]);
    post::terminar(fb, [1.0, 0.6, 0.9], 0.08, 0.0);
    st
}

/// Un cuadro del consultorio de tarot de Madame Mercurio.
fn dibujar_mercurio(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, m: &Mercurio, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira, arriba) = m.camara();
    let fov = FOV.to_radians();
    let vista = Mat4::look_at(ojo, mira, arriba);
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.05, 300.0);
    let vista_proyeccion = proyeccion * vista;
    let base = base_uniforms(vista, proyeccion, Vec3::new(3.0, 7.0, -1.0), ojo, LUZ_MERCURIO);
    let t = m.total;
    // Afuera: Mercurio no tiene atmosfera, el cielo es negro con estrellas... y el Sol enorme.
    fb.limpiar([4, 3, 10]);
    rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, t, 0.0);
    let sol = proyectar(m.sol(), &vista_proyeccion, fb);
    if let Some((x, y, _)) = sol {
        let radio = fb.alto as f32 * 0.32;
        post::halo(fb, (x, y), radio * 2.6, [0.9, 0.45, 0.12], 2.0);
        post::halo(fb, (x, y), radio, [1.0, 0.62, 0.22], 0.25);
    }
    fb.limpiar_profundidad();
    let anim = EstadoAnim { t, ..Default::default() };
    // Una carta por draw call: se ocultan los frentes de las demas cartas.
    let cartas = m.cartas();
    let ocultas: Vec<Vec<&str>> = cartas.iter().map(|(c, _)| mercurio::FRENTES.iter().enumerate().filter(|(k, _)| k != c).map(|(_, f)| *f).collect()).collect();
    let mut llamadas = vec![
        llamada(&rec.carpa_tarot, base),
        DrawCall { expresion: Some(m.expresion_madame()), ..llamada(&rec.madame, Uniforms { modelo: m.madame(), luz: Vec3::new(-2.0, 5.0, -5.0), ..base }) },
        DrawCall {
            expresion: Some(m.expresion_miku()),
            ocultar: &["volante", "volante_columna", "asiento"],
            animacion: Some((rec.anim_piloto(), anim)),
            // Su propia luz principal, de frente: Madame y Miku se ven de lados contrarios.
            ..llamada(rec.piloto(), Uniforms { modelo: m.miku(), luz: Vec3::new(2.0, 6.0, 4.0), ..base })
        },
    ];
    for ((_, modelo), ocultar) in cartas.iter().zip(&ocultas) {
        llamadas.push(DrawCall { ocultar, ..llamada(&rec.tarot, Uniforms { modelo: *modelo, ..base }) });
    }
    let st = pipeline::render(fb, &llamadas, opciones);
    m.dibujar_efectos(fb, &vista_proyeccion);
    m.dibujar_limpia(fb, &vista_proyeccion);
    m.dibujar_luces(fb, &vista_proyeccion);
    // El resplandor del Sol que entra por la ventana (solo si se ve).
    if let Some((x, y, z)) = sol {
        let visible = post::visibilidad_sol(fb, (x, y), fb.alto as f32 * 0.3, z.min(0.9999));
        post::halo(fb, (x, y), fb.alto as f32 * 0.9, [0.3 * visible, 0.16 * visible, 0.05 * visible], 1.8);
    }
    post.bloom(fb, BLOOM * post::exposicion(fb));
    let retro = m.rebobinando();
    // El relampago de las cartas malas: un flash blanco.
    post::terminar(fb, [1.0, 0.1, 0.3], retro * 0.6, m.relampago * m.relampago * 0.7);
    post::vhs(fb, t, retro);
    st
}

/// Un cuadro del Mirador: Miku en la banca junto a la fogata, con la Tierra, la Luna y el Sol enfrente.
fn dibujar_mirador(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, m: &Mirador, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira) = m.camara();
    let fov = FOV.to_radians();
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.05, 600.0);
    let vista_proyeccion = proyeccion * vista;
    // La luz del Sol viene de atras de la Tierra; la fogata alumbra calientito (el ambiente).
    let base = base_uniforms(vista, proyeccion, Vec3::new(-1.0, 1.5, -6.0), ojo, (Vec3::new(0.30, 0.22, 0.20), Vec3::new(0.35, 0.30, 0.60)));
    let t = m.total;
    fb.limpiar([2, 2, 8]);
    rec.cielo.dibujar_fondo(fb, &vista, fov);
    rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, t, 0.0);
    // El Sol, detras de la Tierra (el Webb lo ve asi, desde L2).
    if let Some((x, y, _)) = proyectar(mirador::SOL, &vista_proyeccion, fb) {
        post::halo(fb, (x, y), fb.alto as f32 * 0.5, [0.7, 0.45, 0.2], 2.0);
        post::halo(fb, (x, y), 40.0, [1.5, 1.2, 0.8], 0.4);
    }
    fb.limpiar_profundidad();
    let anim = EstadoAnim { t, giro_turbinas: t * 2.0, ..Default::default() };
    let (tierra, anim_tierra) = rec.planeta_detallado(TIERRA);
    let llamadas = [
        llamada(&rec.mirador, base),
        DrawCall {
            expresion: Some(m.expresion_miku()),
            ocultar: &["volante", "volante_columna", "asiento"],
            animacion: Some((rec.anim_piloto(), anim)),
            ..llamada(rec.piloto(), Uniforms { modelo: m.miku(), luz: Vec3::new(1.0, 3.0, -2.0), ..base })
        },
        DrawCall { expresion: Some(EXPRESION), animacion: Some((rec.anim_nave(), anim)), ..llamada(rec.nave(), Uniforms { modelo: m.nave(), ..base }) },
        DrawCall { animacion: Some((anim_tierra, anim)), ..llamada(tierra, Uniforms { modelo: m.tierra(), luz: Vec3::new(-1.0, 0.5, -3.0), ..base }) },
        llamada(rec.planeta_normal(escena::LUNA).0, Uniforms { modelo: m.luna(), luz: Vec3::new(-1.0, 0.5, -3.0), ..base }),
    ];
    let st = pipeline::render(fb, &llamadas, opciones);
    m.dibujar_luces(fb, &vista_proyeccion);
    post.bloom(fb, BLOOM * post::exposicion(fb));
    post::terminar(fb, [1.0, 0.6, 0.3], 0.03, 0.0);
    st
}

/// Un cuadro de la historia del universo.
fn dibujar_historia(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, h: &Historia, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira) = h.camara();
    let fov = FOV.to_radians();
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.05, 600.0);
    let vista_proyeccion = proyeccion * vista;
    let base = base_uniforms(vista, proyeccion, Vec3::new(-3.0, 2.0, -4.0), ojo, LUZ_ESPACIO);
    let t = h.total;
    fb.limpiar([0, 0, 0]);
    let estrellas = h.estrellas();
    if estrellas > 0.0 {
        rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, t * estrellas, 0.0);
    }
    fb.limpiar_profundidad();
    let anim = EstadoAnim { t, giro_turbinas: t * 4.0, empuje: 0.8, ..Default::default() };
    let partes = rec.baile().matrices(t, Paso::Baile, Paso::Baile, 1.0, 0.0);
    let cosas = h.cosas();
    let llamadas: Vec<DrawCall> = cosas
        .iter()
        .map(|&(cosa, m, expresion)| match cosa {
            // Con el modelo detallado: en la historia los planetas se ven de cerca.
            historia::Cosa::Planeta(i) => {
                let (modelo, anim_planeta) = rec.planeta_detallado(i);
                DrawCall { animacion: Some((anim_planeta, anim)), ..llamada(modelo, Uniforms { modelo: m, luz: Vec3::new(-3.0, 3.0, -4.0), ..base }) }
            }
            historia::Cosa::Miku => DrawCall {
                expresion,
                por_parte: Some(&partes),
                animacion: Some((&rec.sin_animacion, anim)),
                ..llamada(rec.miku_baile(), Uniforms { modelo: m, luz: Vec3::new(1.0, 3.0, -4.0), ..base })
            },
            historia::Cosa::Nave => DrawCall { expresion, animacion: Some((rec.anim_nave(), anim)), ..llamada(rec.nave(), Uniforms { modelo: m, ..base }) },
            historia::Cosa::Bandera => llamada(&rec.bandera, Uniforms { modelo: m, ..base }),
            historia::Cosa::Astronauta => llamada(&rec.astronauta, Uniforms { modelo: m, ..base }),
        })
        .collect();
    let st = pipeline::render(fb, &llamadas, opciones);
    h.dibujar_efectos(fb, &vista_proyeccion);
    post.bloom(fb, BLOOM * post::exposicion(fb));
    post::terminar(fb, [0.3, 0.4, 1.0], 0.03, h.destello());
    st
}

/// La estela de la nave: lineas de luz por el rastro, con el estilo que se compro.
fn dibujar_estela(fb: &mut Framebuffer, vista_proyeccion: &Mat4, rastro: &[Vec3], estela: &str, reloj: f32) {
    estelas::dibujar(fb, vista_proyeccion, rastro, estela, reloj);
}

/// Jupiter de fondo, pintado pixel por pixel (no con triangulos: de tan cerca, los voxels se
/// veian toscos): un disco enorme abajo con sus bandas que se mueven a distinta velocidad,
/// turbulencia en las orillas de las bandas, la Gran Mancha Roja y oscurecido hacia el borde.
fn fondo_de_jupiter(fb: &mut Framebuffer, t: f32, (cx, cy, r): (f32, f32, f32)) {
    let h = fb.alto as f32;
    // Los colores de las bandas, de norte a sur (crema, cafe, naranja...).
    const BANDAS: [[f32; 3]; 8] = [
        [0.86, 0.80, 0.68], [0.62, 0.42, 0.28], [0.92, 0.86, 0.72], [0.74, 0.52, 0.34],
        [0.95, 0.90, 0.78], [0.58, 0.38, 0.26], [0.85, 0.70, 0.52], [0.66, 0.48, 0.34],
    ];
    let y0 = ((cy - r).max(0.0)) as usize;
    for y in y0..fb.alto {
        let dy = (y as f32 + 0.5 - cy) / r;
        for x in 0..fb.ancho {
            let dx = (x as f32 + 0.5 - cx) / r;
            let d2 = dx * dx + dy * dy;
            if d2 >= 1.0 {
                continue;
            }
            let z = (1.0 - d2).sqrt();
            // Latitud (con la inclinacion de la vista) y longitud que gira con el tiempo.
            let lat = (dy * 0.92 + 0.18).asin();
            let lon = dx.atan2(z) + t * 0.025;
            // Las bandas se tuercen un poquito: turbulencia.
            let onda = (lon * 9.0 + lat * 20.0).sin() * 0.025 + (lon * 23.0 - t * 0.3).sin() * 0.012 * (lat * 14.0).cos();
            let u = (lat + onda) * 7.5 + 4.0;
            let i = u.floor() as i32;
            let f = u - u.floor();
            let a = BANDAS[i.rem_euclid(8) as usize];
            let b = BANDAS[(i + 1).rem_euclid(8) as usize];
            let k = f * f * (3.0 - 2.0 * f);
            let mut c = [0, 1, 2].map(|q| a[q] + (b[q] - a[q]) * k);
            // La Gran Mancha Roja: un ovalo que da la vuelta con el planeta.
            let (ml, mo) = (lat + 0.28, (lon - 0.35).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI);
            let mancha = (mo / 0.32).powi(2) + (ml / 0.11).powi(2);
            if mancha < 1.0 {
                let remolino = 0.5 + 0.5 * ((mancha.sqrt() * 12.0) - t * 2.0 + mo.atan2(ml) * 2.0).sin();
                let m = (1.0 - mancha).powf(0.4);
                c = [0, 1, 2].map(|q| c[q] * (1.0 - m) + [0.78, 0.30 + 0.08 * remolino, 0.18][q] * m);
            }
            // Oscurecido hacia el borde (como los planetas de verdad) y una luz de lado.
            let luz = (0.25 + 0.75 * z.powf(0.6)) * (0.75 + 0.25 * (-dx * 0.8 + z * 0.6).max(0.0));
            let i = y * fb.ancho + x;
            // Un poco mas de contraste y saturacion (si no, con el bloom se ven deslavadas).
            let gris = (c[0] + c[1] + c[2]) / 3.0;
            let c = c.map(|v| ((v - gris) * 1.35 + gris - 0.5) * 1.15 + 0.5);
            fb.pintar(i, c.map(|v| v * luz * 0.78), 1.0);
        }
    }
    // La atmosfera: un brillo azulado en la orilla.
    for y in y0..fb.alto {
        for x in 0..fb.ancho {
            let d = (((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt() - r) / (h * 0.03);
            if d.abs() < 1.0 {
                fb.sumar(y * fb.ancho + x, [0.10, 0.12, 0.20].map(|v| v * (1.0 - d.abs())));
            }
        }
    }
}

/// Un cuadro de Jupiter Asteroids, en primera persona: Jupiter enorme al frente, las rayitas
/// de velocidad, las rocas que vienen, los laseres y las explosiones (la cabina va en la UI).
fn dibujar_jupiter(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, j: &Jupiter, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira, arriba) = j.camara();
    let fov = 70f32.to_radians();
    let vista = Mat4::look_at(ojo, mira, arriba);
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.1, 600.0);
    let vp = proyeccion * vista;
    let t = j.total;
    let base = base_uniforms(vista, proyeccion, ojo + Vec3::new(-30.0, 40.0, 20.0), ojo, (Vec3::new(0.42, 0.38, 0.44), Vec3::new(1.0, 0.8, 0.6)));
    fb.limpiar([5, 3, 12]);
    rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, t, 0.0);
    // Jupiter al frente: casi no se mueve (esta lejisimos), solo un poquito con la nave.
    let (w, h) = (fb.ancho as f32, fb.alto as f32);
    fondo_de_jupiter(fb, t, (w * 0.5 - j.nave.x * 3.0, h * 0.95 + j.nave.y * 3.0, h * 0.85));
    fb.limpiar_profundidad();
    // Las rayitas de velocidad: puntos en un tunel alrededor del camino que vienen hacia la nave.
    let avance = j.avance();
    for k in 0..90u32 {
        let semilla = k.wrapping_mul(2_654_435_761);
        let a = (semilla % 1000) as f32 / 1000.0 * std::f32::consts::TAU;
        let r = 8.0 + ((semilla / 1000) % 1000) as f32 / 1000.0 * 28.0;
        let z = -(((semilla / 7) % 2000) as f32 / 10.0 - avance).rem_euclid(200.0);
        let p0 = Vec3::new(a.cos() * r, a.sin() * r * 0.6, z);
        let p1 = p0 + Vec3::new(0.0, 0.0, 3.0 + avance * 0.0);
        if let (Some(a), Some(b)) = (proyectar(p0, &vp, fb), proyectar(p1, &vp, fb)) {
            let luz = (1.0 + z / 200.0).clamp(0.0, 1.0) * 0.5;
            fb.linea_de_luz(a, b, [0.4 * luz, 0.5 * luz, 0.9 * luz], [0.8 * luz, 0.85 * luz, 1.0 * luz]);
        }
    }
    let anim = EstadoAnim { t, ..Default::default() };
    let mut llamadas = Vec::new();
    for r in &j.rocas {
        let c = match r.clase {
            jupiter::Clase::Gris => 0,
            jupiter::Clase::Roja => 1,
            jupiter::Clase::Dorada => 2,
        };
        llamadas.push(llamada(&rec.rocas[c][r.modelo], Uniforms { modelo: Jupiter::matriz_roca(r), ..base }));
    }
    for l in &j.lunas {
        let m = Mat4::translation(l.pos) * Mat4::rotation_y(t * 1.5 + l.cual as f32) * Mat4::rotation_x(0.4) * Mat4::scale(1.8);
        llamadas.push(DrawCall { animacion: Some((&rec.sin_animacion, anim)), ..llamada(&rec.lunas_galileo[l.cual], Uniforms { modelo: m, ..base }) });
    }
    let mut st = pipeline::render(fb, &llamadas, opciones);
    // La Gran Mancha Roja: una tormenta en espiral al fondo; el ojo brilla cuando se abre.
    if let Some(m) = &j.mancha {
        if let Some((cx, cy, _)) = proyectar(m.pos, &vp, fb) {
            let escala = h * 2.2 / (ojo - m.pos).length().max(10.0) * 10.0;
            let golpe = if m.golpe > 0.0 { 1.7 } else { 1.0 };
            post::halo(fb, (cx, cy), escala * 1.3, [0.5 * golpe, 0.1, 0.04], 1.2);
            for k in 0..70 {
                let u = k as f32 / 70.0;
                for brazo in 0..3 {
                    let a = u * 7.5 + brazo as f32 * std::f32::consts::TAU / 3.0 - m.t * 2.4;
                    let r = u * escala;
                    let (x, y) = (cx + a.cos() * r * 1.3, cy + a.sin() * r * 0.75);
                    post::halo(fb, (x, y), escala * (0.05 + 0.08 * (1.0 - u)), [1.0 * (1.0 - u) * golpe, 0.3 * (1.0 - u), 0.1 * (1.0 - u)], 1.3);
                }
            }
            if m.abierta() {
                post::halo(fb, (cx, cy), escala * 0.35 + 6.0 * (t * 10.0).sin().abs(), [1.0, 0.85, 0.3], 1.6);
            }
        }
    }
    // Los laseres (de fuego con Io, gordos con Ganimedes).
    for b in &j.balas {
        let color = if b.fuego { [1.0, 0.5, 0.15] } else if b.gorda { [0.95, 0.85, 0.6] } else { [0.4, 1.0, 0.9] };
        if let (Some(a), Some(c)) = (proyectar(b.pos - b.vel * 0.035, &vp, fb), proyectar(b.pos, &vp, fb)) {
            for d in [-1.0, 0.0, 1.0] {
                fb.linea_de_luz((a.0 + d, a.1, a.2), (c.0 + d, c.1, c.2), color, color);
            }
            if b.gorda {
                post::halo(fb, (c.0, c.1), 10.0, color.map(|v| v * 0.6), 1.5);
            }
        }
    }
    // Las chispas de las explosiones.
    for c in &j.chispas {
        if let Some((x, y, _)) = proyectar(c.pos, &vp, fb) {
            let d = (c.pos - ojo).length().max(1.0);
            post::halo(fb, (x, y), (2.0 + 4.0 * c.vida) * 30.0 / d, c.color.map(|v| v * c.vida.min(1.0)), 1.5);
        }
    }
    // El brillo de las lunas y de las rocas rojas y doradas (para verlas venir).
    for l in &j.lunas {
        if let Some((x, y, _)) = proyectar(l.pos, &vp, fb) {
            let d = (l.pos - ojo).length().max(1.0);
            post::halo(fb, (x, y), 600.0 / d + 4.0 * (t * 5.0).sin(), jupiter::COLOR_LUNA[l.cual].map(|v| v * 0.6), 1.6);
        }
    }
    for r in &j.rocas {
        let color = match r.clase {
            jupiter::Clase::Roja => [0.6, 0.15, 0.03],
            jupiter::Clase::Dorada => [0.8, 0.6, 0.15],
            _ => continue,
        };
        if let Some((x, y, _)) = proyectar(r.pos, &vp, fb) {
            post::halo(fb, (x, y), 400.0 / (r.pos - ojo).length().max(1.0), color, 1.6);
        }
    }
    // La cabina, al final y encima de todo (va pegada a la camara): el tablero con sus pantallas,
    // la palanca de mando (se inclina con el viraje) y, en la visera, el muñequito cabezon del
    // personaje que se escogio, bamboleandose con cada vuelta y cada golpe.
    fb.limpiar_profundidad();
    let cab = j.cabina();
    let luz_cabina = cab.transform_point(Vec3::new(-0.6, 2.2, 0.6));
    let base_cab = Uniforms { luz: luz_cabina, ambiente: Vec3::new(0.30, 0.25, 0.40), borde: Vec3::new(0.55, 0.40, 0.75), ..base };
    let (giro, cabeceo) = j.palanca();
    let m_palanca = cab * Mat4::translation(Vec3::new(0.0, -0.93, -1.02)) * Mat4::rotation_x(cabeceo - 0.15) * Mat4::rotation_z(giro);
    let (bz, bx) = j.cabezon();
    let elenco = &rec.elenco[rec.personaje.get()];
    let partes = elenco.baile.cabezon(bz, bx);
    let m_munequito = cab * Mat4::translation(Vec3::new(0.74, -0.272, -1.47)) * Mat4::rotation_y(std::f32::consts::PI - 0.45) * Mat4::rotation_z(bz * 0.25) * Mat4::scale(0.13);
    let cabina = [
        llamada(&rec.cabina, Uniforms { modelo: cab, ..base_cab }),
        llamada(&rec.palanca, Uniforms { modelo: m_palanca, ..base_cab }),
        DrawCall {
            expresion: Some("feliz"),
            por_parte: Some(&partes),
            animacion: Some((&rec.sin_animacion, anim)),
            ..llamada(&elenco.de_pie, Uniforms { modelo: m_munequito, ..base_cab })
        },
    ];
    let st_cab = pipeline::render(fb, &cabina, opciones);
    st.triangulos += st_cab.triangulos;
    st.dibujados += st_cab.dibujados;
    // Reflejos en el vidrio de la burbuja: dos rayas diagonales muy tenues.
    let (w, h) = (fb.ancho as f32, fb.alto as f32);
    for (x0, ancho) in [(0.12, 0.05), (0.22, 0.02)] {
        let (a, b) = ((w * x0, 0.0), (w * (x0 - 0.18), h * 0.7));
        for d in 0..(w * ancho) as i32 {
            let c = [0.035, 0.035, 0.05];
            fb.linea_de_luz((a.0 + d as f32, a.1, 0.0), (b.0 + d as f32, b.1, 0.0), c, [0.0, 0.0, 0.0]);
        }
    }
    post.bloom(fb, BLOOM * post::exposicion(fb));
    let tinte = if j.poder[3] > 0.0 { ([0.4, 0.5, 1.0], 0.5) } else { ([1.0, 0.6, 0.3], 0.05) };
    post::terminar(fb, tinte.0, tinte.1, j.destello * 0.5);
    st
}

/// Un cuadro del set de filmacion de la Luna.
/// Mezcla de tres colores del camino: el del principio, el de la mitad y el del final.
fn tres_tiempos(a: [f32; 3], b: [f32; 3], c: [f32; 3], k: f32) -> [f32; 3] {
    let (d, e, u) = if k < 0.5 { (a, b, k * 2.0) } else { (b, c, k * 2.0 - 1.0) };
    [0, 1, 2].map(|i| d[i] + (e[i] - d[i]) * u)
}

/// Un cuadro de la camioneta: el cielo que se va haciendo de noche, los volcanes de bloques, la
/// carretera con el suelo de cada zona, lo de la orilla, los puntos fijos de la ruta (el Arco, la
/// Torre, la terminal), lo que viene en el camino, la camioneta y sus cubitos.
fn dibujar_camioneta(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, c: &Camioneta, opciones: &Opciones) -> Estadisticas {
    use camioneta::{Deco, Tipo, Zona, ARCO, INICIO_ANTIGUA, INICIO_CAPITAL, RUTA, TORRE, TRAMO};
    let (ojo, mira) = c.camara();
    // Entre mas rapido, mas abierta la camara: ayuda a sentir la velocidad.
    let fov = (56.0 + c.vel * 0.36).to_radians();
    let (cerca, lejos) = (0.1, 900.0);
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, cerca, lejos);
    let vp = proyeccion * vista;
    // La tarde dorada, el atardecer lila y la noche de la capital.
    let k = c.atardecer();
    let horizonte = tres_tiempos([1.0, 0.72, 0.46], [1.0, 0.46, 0.36], [0.62, 0.22, 0.52], k);
    let medio = tres_tiempos([0.98, 0.52, 0.66], [0.78, 0.30, 0.64], [0.22, 0.12, 0.42], k);
    let cenit = tres_tiempos([0.46, 0.36, 0.96], [0.27, 0.17, 0.66], [0.05, 0.04, 0.17], k);
    cielo_degradado(fb, (mira - ojo).normalize(), fov, [horizonte, medio, cenit]);
    fb.limpiar_profundidad();
    if k > 0.68 {
        rec.cielo.dibujar_estrellas(fb, &vista, &proyeccion, c.total, 0.0);
        fb.limpiar_profundidad();
    }
    // El sol va bajando atras de los volcanes.
    let sol_pos = Vec3::new(-85.0, 92.0 - k * 118.0, -640.0);
    if let Some((x, y, _)) = proyectar(sol_pos, &vp, fb) {
        let fuerza = (1.0 - k * 1.15).max(0.0);
        post::halo(fb, (x, y), 190.0, [0.55 * fuerza, 0.28 * fuerza, 0.20 * fuerza], 1.5);
    }
    let ambiente = tres_tiempos([0.56, 0.50, 0.56], [0.50, 0.40, 0.56], [0.34, 0.30, 0.52], k);
    let borde = tres_tiempos([0.26, 0.16, 0.18], [0.30, 0.14, 0.26], [0.18, 0.14, 0.42], k);
    let base = base_uniforms(vista, proyeccion, Vec3::new(-60.0, 80.0, 30.0), ojo, (Vec3::new(ambiente[0], ambiente[1], ambiente[2]), Vec3::new(borde[0], borde[1], borde[2])));
    let m = &rec.camioneta;
    let en = |x: f32, y: f32, z: f32| Mat4::translation(Vec3::new(x, y, z));
    let mut llamadas = vec![llamada(&m.llano, base), llamada(&m.volcanes, base), llamada(&m.lava, base)];
    if k < 1.0 {
        llamadas.push(llamada(&m.sol, Uniforms { modelo: Mat4::translation(sol_pos), ..base }));
    }
    // Las nubes pasan despacio, y las fumarolas del Fuego suben y se abren.
    for i in 0..8 {
        let semilla = i as f32 * 2.399;
        let x = ((semilla * 91.0).sin() * 300.0 + c.total * 1.6 + c.recorrido * 0.015).rem_euclid(760.0) - 380.0;
        let y = 62.0 + (semilla * 17.0).sin().abs() * 75.0;
        let z = -210.0 - (semilla * 41.0).cos().abs() * 230.0;
        llamadas.push(llamada(&m.nube, Uniforms { modelo: en(x, y, z) * Mat4::scale(0.9 + (semilla * 7.0).sin().abs()), ..base }));
    }
    for i in 0..7 {
        let t = (c.total * 0.09 + i as f32 / 7.0).fract();
        let x = 38.0 + (i as f32 * 2.1).sin() * 3.0 + t * 26.0;
        let tam = 8.0 + t * 26.0;
        llamadas.push(llamada(&m.cubos[0], Uniforms { modelo: en(x, 150.0 + t * 78.0, -380.0) * Mat4::scale(tam), ..base }));
    }
    // La carretera: cada tramo con el suelo de su zona y turnando franjas.
    let corrido = c.recorrido.rem_euclid(TRAMO);
    let primero = (c.recorrido / TRAMO).floor() as i32;
    for k in -1..30 {
        let z = -(k as f32) * TRAMO + corrido;
        let metros = c.recorrido - z + TRAMO * 0.5;
        let indice = primero + k;
        let zona = camioneta::zona_de(metros);
        let cual = match zona { Zona::Campo => 0, Zona::Antigua => 1, Zona::Capital => 2 };
        llamadas.push(llamada(&m.suelos[cual][indice.rem_euclid(2) as usize], Uniforms { modelo: en(0.0, 0.0, z), ..base }));
        if zona == Zona::Antigua && indice.rem_euclid(9) == 4 {
            llamadas.push(llamada(&m.alfombra, Uniforms { modelo: en(0.0, 0.0, z), ..base }));
        }
    }
    // Lo de la orilla. Lo que tiene cara de calle la voltea hacia la carretera.
    let pi = std::f32::consts::PI;
    for a in &c.adornos {
        let hacia_calle = if a.x > 0.0 { Mat4::identity() } else { Mat4::rotation_y(pi) };
        let donde = en(a.x, 0.0, a.z);
        let (modelo, matriz) = match a.tipo {
            Deco::Ceiba => (&rec.objetos_chapin.ceiba, donde * Mat4::rotation_y(a.color as f32) * Mat4::scale(4.5)),
            Deco::Milpa => (&m.milpa, donde),
            Deco::Casita => (&m.casitas[a.color % 6], donde * hacia_calle),
            Deco::Barrilete => (&m.barriletes[a.color % 3], donde),
            Deco::Poste => (&m.poste, donde),
            Deco::Puesto => (&m.puesto, donde * hacia_calle),
            Deco::Casa => (&m.casas[a.color % 6], donde * hacia_calle),
            Deco::Iglesia => (&m.iglesia, donde),
            Deco::PapelPicado => (&m.papel, en(0.0, (c.total * 2.5 + a.z).sin() * 0.05, a.z)),
            Deco::Edificio => (&m.edificios[a.color % 4], donde * hacia_calle),
            Deco::Valla => (&m.vallas[a.color % 3], donde),
            Deco::Farol => (&m.farol, donde * if a.x > 0.0 { Mat4::identity() } else { Mat4::rotation_y(pi) }),
        };
        llamadas.push(llamada(modelo, Uniforms { modelo: matriz, ..base }));
    }
    // Los puntos fijos de la ruta.
    for (metros, modelo, x) in [(INICIO_ANTIGUA, &m.portada_antigua, 0.0), (ARCO, &m.arco, 0.0), (INICIO_CAPITAL, &m.portada_capital, 0.0), (TORRE, &m.torre, -21.0), (RUTA, &m.terminal, 0.0)] {
        let z = c.recorrido - metros;
        if z < 14.0 && z > -300.0 {
            llamadas.push(llamada(modelo, Uniforms { modelo: en(x, 0.0, z), ..base }));
        }
    }
    for cosa in &c.cosas {
        let donde = en(cosa.x, 0.0, cosa.z);
        match cosa.tipo {
            Tipo::TucTuc => llamadas.push(llamada(&m.tuctuc, Uniforms { modelo: donde, ..base })),
            Tipo::Perro => {
                let trote = (c.total * 14.0).sin().abs() * 0.08;
                let mira_a = if cosa.mov > 0.0 { -std::f32::consts::FRAC_PI_2 } else { std::f32::consts::FRAC_PI_2 };
                llamadas.push(llamada(&m.perro, Uniforms { modelo: en(cosa.x, trote, cosa.z) * Mat4::rotation_y(mira_a), ..base }));
            }
            Tipo::Bache => llamadas.push(llamada(&m.bache, Uniforms { modelo: donde, ..base })),
            Tipo::Tumulo => {
                llamadas.push(llamada(&m.tumulo, Uniforms { modelo: en(0.0, 0.0, cosa.z), ..base }));
                for x in [-4.9, 4.9] {
                    llamadas.push(llamada(&m.senal_tumulo, Uniforms { modelo: en(x, 0.0, cosa.z - 14.0), ..base }));
                }
            }
            Tipo::Parada(gente) => {
                llamadas.push(llamada(&m.parada, Uniforms { modelo: en(0.0, 0.0, cosa.z), ..base }));
                if gente > 0 {
                    let pintura = if c.vel < camioneta::DESPACIO { &m.zona_lenta } else { &m.zona_rapida };
                    llamadas.push(llamada(pintura, Uniforms { modelo: en(0.0, 0.0, cosa.z), ..base }));
                }
                for p in 0..gente {
                    let x = 4.5 + (p % 3) as f32 * 0.45;
                    let z = cosa.z - 1.0 + (p / 3) as f32 * 0.7 + (p % 2) as f32 * 0.2;
                    // Saludan (brinquitos) para que el chofer los vea.
                    let brinco = ((c.total * 6.0 + p as f32).sin()).max(0.0) * 0.15;
                    llamadas.push(llamada(&m.personas[p as usize % 3], Uniforms { modelo: en(x, brinco, z) * Mat4::rotation_y(std::f32::consts::FRAC_PI_2), ..base }));
                }
            }
        }
    }
    // La camioneta, con el ayudante colgado saludando y la gallina en la parrilla.
    let bus = c.matriz();
    llamadas.push(llamada(&m.camioneta, Uniforms { modelo: bus, ..base }));
    if c.frenando {
        llamadas.push(llamada(&m.frenos, Uniforms { modelo: bus, ..base }));
    }
    let meneo = (c.total * 7.0).sin();
    llamadas.push(llamada(&m.ayudante, Uniforms { modelo: bus * Mat4::translation(Vec3::new(0.63, 0.0, 3.68)) * Mat4::rotation_z(meneo * 0.04), ..base }));
    llamadas.push(llamada(&m.brazo, Uniforms { modelo: bus * Mat4::translation(Vec3::new(0.86, 1.62, 3.68)) * Mat4::rotation_z(-0.9 + meneo * 0.55), ..base }));
    let brinca = (c.total * 9.0).sin().max(0.0) * 0.1;
    llamadas.push(llamada(&m.gallina, Uniforms { modelo: bus * Mat4::translation(Vec3::new(0.5, 3.06 + brinca, 0.68)) * Mat4::rotation_y(0.7), ..base }));
    for ch in &c.chispas {
        llamadas.push(llamada(&m.cubos[ch.color], Uniforms { modelo: Mat4::translation(ch.pos) * Mat4::rotation_y(ch.vida * 4.0) * Mat4::scale(ch.tam), ..base }));
    }
    let st = pipeline::render(fb, &llamadas, opciones);
    post::sombras_de_contacto(fb, 5, 0.42);
    // La neblina de lejos, del color del horizonte (asi lo que aparece no sale de golpe).
    post::niebla(fb, cerca, lejos, 130.0, 760.0, [horizonte[0] * 0.9, horizonte[1] * 0.75, horizonte[2] * 0.95], true);
    post.bloom(fb, BLOOM * (0.55 + k * 0.5));
    post::terminar(fb, [1.0, 0.85, 0.6], 0.03, c.destello);
    st
}

fn dibujar_luna(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, l: &Luna, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira) = l.camara();
    let fov = FOV.to_radians();
    let vista = Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0));
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.05, 600.0);
    let vista_proyeccion = proyeccion * vista;
    // El Sol de lado y bajito: sombras largas, como en las fotos del Apolo.
    let base = base_uniforms(vista, proyeccion, Vec3::new(-4.0, 3.0, -2.5), ojo, LUZ_LUNA);
    let t = l.total;
    // El cielo de la Luna: negro. Sin estrellas (como en las fotos de verdad).
    fb.limpiar([2, 2, 5]);
    fb.limpiar_profundidad();
    let anim = EstadoAnim { t, giro_turbinas: t * 2.0, ..Default::default() };
    let ocultar = l.ocultar();
    let paso = l.paso_miku();
    let partes = rec.baile().matrices(t, paso, paso, 1.0, 0.0);
    let (tierra, anim_tierra) = rec.planeta_detallado(TIERRA);
    let mut llamadas = vec![
        DrawCall { ocultar: &ocultar, ..llamada(&rec.set_luna, base) },
        llamada(&rec.decorado.crateres, base),
        llamada(&rec.decorado.estudio, base),
        llamada(&rec.bandera, Uniforms { modelo: l.bandera(), ..base }),
        llamada(&rec.astronauta, Uniforms { modelo: l.astronauta(), ..base }),
        DrawCall { expresion: Some(l.expresion_director()), ..llamada(&rec.director, Uniforms { modelo: l.director(), luz: Vec3::new(-2.0, 4.0, 4.0), ..base }) },
        DrawCall {
            expresion: Some(l.expresion_miku()),
            por_parte: Some(&partes),
            animacion: Some((&rec.sin_animacion, anim)),
            ..llamada(rec.miku_baile(), Uniforms { modelo: l.miku(), luz: Vec3::new(-2.0, 4.0, 4.0), ..base })
        },
        // La nave, estacionada a un lado del set.
        DrawCall {
            expresion: Some(EXPRESION),
            animacion: Some((rec.anim_nave(), anim)),
            ..llamada(rec.nave(), Uniforms { modelo: Mat4::translation(Vec3::new(6.0, 1.0, -3.5)) * Mat4::rotation_y(-0.9) * Mat4::scale(0.6), ..base })
        },
        DrawCall { animacion: Some((anim_tierra, anim)), ..llamada(tierra, Uniforms { modelo: l.tierra(), ..base }) },
    ];
    if let Some(m) = l.profe() {
        llamadas.push(DrawCall { expresion: Some("feliz"), ..llamada(&rec.profe, Uniforms { modelo: m, ..base }) });
    }
    let st = pipeline::render(fb, &llamadas, opciones);
    post::sombras_de_contacto(fb, 6, 0.5);
    l.dibujar_efectos(fb, &vista_proyeccion);
    // Los reflectores del estudio, colgados de la armazon, con su resplandor.
    for k in 0..7 {
        let x = -24.0 + k as f32 * 8.0;
        let parpadeo = 1.0 + 0.04 * (t * 3.0 + x).sin();
        if let Some((px, py, _)) = proyectar(Vec3::new(x, 8.8, 16.0), &vista_proyeccion, fb) {
            post::halo(fb, (px, py), 46.0 * parpadeo, [0.62, 0.56, 0.38], 1.5);
            post::halo(fb, (px, py), 8.0, [1.0, 0.95, 0.75], 1.0);
        }
    }
    post.bloom(fb, BLOOM * post::exposicion(fb));
    post::terminar(fb, [0.6, 0.6, 0.7], 0.02, 0.0);
    st
}

/// Un cuadro del juicio de divorcio de Saturno, en el juzgado de Venus.
fn dibujar_divorcio(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, dv: &Divorcio, opciones: &Opciones) -> Estadisticas {
    let (ojo, mira, arriba) = dv.camara();
    let fov = FOV.to_radians();
    let vista = Mat4::look_at(ojo, mira, arriba);
    let proyeccion = Mat4::perspective(fov, fb.ancho as f32 / fb.alto as f32, 0.05, 600.0);
    let vista_proyeccion = proyeccion * vista;
    let base = base_uniforms(vista, proyeccion, Vec3::new(-2.0, 6.0, -4.0), ojo, LUZ_JUZGADO);
    let t = dv.total;
    // El cielo de Venus: nubes espesas amarillas (de acido sulfurico) que corren al reves.
    cielo_degradado(fb, (mira - ojo).normalize(), fov, [[0.95, 0.80, 0.40], [0.80, 0.58, 0.24], [0.40, 0.24, 0.12]]);
    fb.limpiar_profundidad();
    dv.dibujar_cielo(fb, &vista_proyeccion);
    let anim = EstadoAnim { t, ..Default::default() };
    let paso = dv.paso_miku();
    let partes = rec.baile().matrices(t, paso, paso, 1.0, 0.0);
    let perdidos = dv.anillos_perdidos();
    let (juez, _) = dv.juez();
    let mut llamadas = vec![
        llamada(&rec.juzgado, base),
        llamada(&rec.ganso, Uniforms { modelo: juez, ..base }),
        // Miku y Phoenix ven hacia el centro: cada uno con su luz de frente.
        DrawCall {
            expresion: Some(dv.expresion_miku()),
            por_parte: Some(&partes),
            animacion: Some((&rec.sin_animacion, anim)),
            ..llamada(rec.miku_baile(), Uniforms { modelo: dv.miku(), luz: Vec3::new(-4.0, 6.0, -2.0), ..base })
        },
        DrawCall { expresion: Some(dv.expresion_phoenix()), ..llamada(&rec.phoenix, Uniforms { modelo: dv.phoenix(), luz: Vec3::new(4.0, 6.0, -2.0), ..base }) },
        DrawCall {
            expresion: Some(dv.expresion_saturno()),
            ocultar: &perdidos,
            ..llamada(&rec.saturno_cliente, Uniforms { modelo: dv.saturno(), ..base })
        },
    ];
    for (ex, m, expresion) in dv.exes() {
        llamadas.push(DrawCall { expresion, ..llamada(&rec.exes[ex], Uniforms { modelo: m, ..base }) });
    }
    // Los anillos que se robo el ganso, de collar: Saturno sin el planeta ni los otros anillos.
    let collares = dv.collares();
    let solo_anillo: Vec<Vec<&str>> =
        collares.iter().map(|(k, _)| ["planeta", "rostro"].into_iter().chain((0..3).filter(|j| j != k).map(|j| divorcio::PARTES_ANILLO[j])).collect()).collect();
    for ((_, m), ocultar) in collares.iter().zip(&solo_anillo) {
        llamadas.push(DrawCall { ocultar, ..llamada(&rec.saturno_cliente, Uniforms { modelo: *m, ..base }) });
    }
    let st = pipeline::render(fb, &llamadas, opciones);
    dv.dibujar_luces(fb, &vista_proyeccion);
    post.bloom(fb, BLOOM * post::exposicion(fb));
    post::terminar(fb, [1.0, 0.8, 0.5], 0.03, dv.destello());
    st
}

/// El contorno de un corazon con lineas de luz, en el plano de `derecha` y `arriba`.
fn corazon_de_luz(fb: &mut Framebuffer, vista_proyeccion: &Mat4, centro: Vec3, derecha: Vec3, arriba: Vec3, tam: f32, color: [f32; 3]) {
    let punto = |a: f32, s: f32| {
        let x = 16.0 * a.sin().powi(3);
        let y = 13.0 * a.cos() - 5.0 * (2.0 * a).cos() - 2.0 * (3.0 * a).cos() - (4.0 * a).cos();
        centro + derecha * (x / 16.0 * tam * s) + arriba * (y / 16.0 * tam * s)
    };
    for s in [1.0, 0.9] {
        for i in 0..28 {
            let (a0, a1) = (i as f32 / 28.0 * std::f32::consts::TAU, (i + 1) as f32 / 28.0 * std::f32::consts::TAU);
            if let (Some(p0), Some(p1)) = (proyectar(punto(a0, s), vista_proyeccion, fb), proyectar(punto(a1, s), vista_proyeccion, fb)) {
                fb.linea_de_luz(p0, p1, color, color);
            }
        }
    }
}

enum Escena {
    Intro(Intro),
    Espacio,
    Entrega(Box<Entrega>),
    /// El spin-off de la Tierra: el examen de Koch en el salon de Teoria.
    Aula(Box<Aula>),
    Urano(Box<Urano>),
    Vegetta(Box<Vegetta>),
    Concierto(Concierto),
    Ritmo(Ritmo),
    Junta(Junta),
    Cita(Cita),
    Neptuno(Neptuno),
    Fuente(Fuente),
    Mercurio(Mercurio),
    Divorcio(Divorcio),
    Jupiter(Jupiter),
    Luna(Luna),
    /// El Planeta Chapin: la camioneta extraurbana.
    Camioneta(Box<Camioneta>),
    Mirador(Mirador),
    Historia(Historia),
    /// El menu principal (lo primero que se ve).
    Menu(Menu),
}

/// `--captura archivo.ppm [segundos] [x y z rumbo] [alambre|puntos|salto|portal|entrega|entrega_teams|intro|concierto|ritmo|...|aterrizar:<planeta>] [zbuffer|normales|uv|stencil]`:
/// dibuja un cuadro sin abrir la ventana y lo guarda como PPM. Sin posicion,
/// la nave esta en su lugar de salida; el rumbo va en grados. Con `salto`,
/// el cuadro es a mitad del salto al hiperespacio; con `entrega`, en el lab del CIT la noche de
/// la entrega; con `entrega_teams`, con Koch llamando por Teams; con `intro` y
/// `concierto`, en ese segundo de la intro o del show.
/// El juego y la escena de una captura (o de una foto de la interfaz): ver `captura`.
fn preparar(t: f32, resto: &[String]) -> (Juego, Escena) {
    let n: Vec<f32> = resto.iter().filter_map(|s| s.parse().ok()).collect();
    let mut juego = match n[..] {
        [x, y, z, rumbo, ..] => Juego::new(Vec3::new(x, y, z), rumbo.to_radians()),
        _ => Juego::new(intro::nave_parada(t), 0.0),
    };
    juego.t = t;
    let nada = Entrada::default();
    let mut escena = Escena::Espacio;
    // Las vistas de depuracion se pueden pedir junto con cualquier escena (`entrega zbuffer`).
    let vista = |s: &str| match s {
        "zbuffer" => Some(Vista::Profundidad),
        "normales" => Some(Vista::Normales),
        "uv" => Some(Vista::Uv),
        "stencil" => Some(Vista::Stencil),
        _ => None,
    };
    if let Some(v) = resto.iter().find_map(|s| vista(s)) {
        juego.opciones.vista = v;
    }
    match resto.iter().rev().find(|s| vista(s).is_none()).map(String::as_str) {
        Some("alambre") => juego.opciones.modo = ModoDibujo::Alambre,
        Some("portal") => {
            // La nave frente a la cueva del Planeta Hada, mirando el portal.
            let hada = &CUERPOS[HADA];
            let m = hada.matriz(t);
            let (centro, cueva) = (hada.posicion(t), m.transform_point(Vec3::new(0.0, 0.0, -1.0)));
            let afuera = (cueva - centro).normalize();
            juego.nave = Nave::new(cueva + afuera * 7.0 + Vec3::new(0.0, 1.0, 0.0));
            (juego.nave.yaw, juego.nave.pitch) = salto::rumbo_hacia(cueva - juego.nave.pos);
            juego.camara = Camara::detras_de(&juego.nave);
        }
        Some("puntos") => juego.opciones.modo = ModoDibujo::Puntos,
        Some("cometa") => {
            // Llega un cometa y se deja avanzar unos segundos (sin atraparlo).
            let p = juego.nave.pos;
            juego.coleccion.actualizar(p, 100.0, true);
            juego.coleccion.actualizar(p, 9.0, false);
        }
        Some("salto") => {
            juego.salto.iniciar(&juego.nave, siguiente_destino(juego.nave.pos, juego.t));
            for _ in 0..60 {
                juego.actualizar(&nada, 1.0 / 60.0);
            }
        }
        Some("rescate") => {
            // A media estrella de rescate (Mario Galaxy), afuera del sistema.
            juego.rescate = Some(RESCATE * 0.36 - 2.5);
        }
        Some("orilla") => {
            // En la orilla del sistema solar (con el aviso).
            juego.nave.pos = CUERPOS[0].posicion(juego.t) + Vec3::new(0.0, 0.0, ORILLA_SISTEMA + 60.0);
        }
        Some("intro") => {
            let mut i = Intro::new();
            juego.t = 0.0;
            while i.t < t {
                juego.t += 1.0 / 60.0;
                i.actualizar(&mut juego.nave, false, 1.0 / 60.0, juego.t);
            }
            escena = Escena::Intro(i);
        }
        Some(cual @ ("chapin" | "chapin_orbita" | "chapin_cerca")) => {
            // `chapin`: la nave frente al Planeta Chapin, mirandolo. `chapin_cerca`: casi tocando
            // la selva. `chapin_orbita`: ya en modo orbita, rodeandolo. Luego espera `t` segundos.
            let c = &CUERPOS[CHAPIN];
            juego.t = 40.0;
            let distancia = match cual {
                "chapin" => c.radio_cuerpo() * 2.6,
                _ => c.radio_cuerpo() + 3.0,
            };
            let lado = demo::lado_de_dia(juego.t);
            juego.nave.pos = c.posicion(juego.t) + lado * distancia;
            let mirar = if cual == "chapin_orbita" { lado.cross(Vec3::new(0.0, 1.0, 0.0)).normalize() } else { lado * -1.0 + Vec3::new(0.0, -0.15, 0.0) };
            (juego.nave.yaw, juego.nave.pitch) = salto::rumbo_hacia(mirar);
            juego.camara = Camara::detras_de(&juego.nave);
            juego.sincronia = 1.0;
            if cual == "chapin_orbita" {
                juego.actualizar(&Entrada { orbitar: true, ..Default::default() }, 1.0 / 60.0);
            }
            let mut ahora = 0.0;
            while ahora < t {
                juego.actualizar(&Entrada { empuje: if cual == "chapin_orbita" { 0.6 } else { 0.0 }, ..Default::default() }, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            juego.mensaje = None;
        }
        Some(cual) if cual.starts_with("maravilla:") => {
            // `maravilla:<gargantua|gusano|blip|monolito|huerto|teseracto>`: la nave frente a esa
            // maravilla, mirandola (sin que GLADAS hable).
            juego.t = 40.0;
            let t = juego.t;
            let sol = CUERPOS[0].posicion(t);
            let (donde, distancia, alto) = match &cual["maravilla:".len()..] {
                "gargantua" | "teseracto" => (maravillas::GARGANTUA, 150.0, 0.25),
                "gusano" => (maravillas::gusano_en(t), 34.0, 0.2),
                "blip" => (maravillas::blip_en(t) + Vec3::new(4.0, 0.0, 0.0), 22.0, 0.25),
                "monolito" => (maravillas::monolito_en(t) + Vec3::new(0.0, 4.0, 0.0), 32.0, 0.05),
                _ => (maravillas::huerto_en(t), 9.0, 0.6),
            };
            let lado = ((sol - donde).normalize() + Vec3::new(0.0, alto, 0.0)).normalize();
            juego.nave.pos = donde + lado * distancia;
            // La nariz un poquito a un lado: la camara va detras de la nave y si no la nave la tapa.
            let costado = lado.cross(Vec3::new(0.0, 1.0, 0.0)).normalize() * (distancia * 0.28);
            (juego.nave.yaw, juego.nave.pitch) = salto::rumbo_hacia(donde + costado - juego.nave.pos);
            juego.camara = Camara::detras_de(&juego.nave);
            juego.sincronia = 1.0;
            if cual.ends_with("teseracto") {
                juego.maravillas.teseracto = Some(1.0);
            }
            juego.mensaje = None;
        }
        Some("recoger") => {
            // La nave entrando a un arco de notas (para ver el jugo de recoger): se pone antes de
            // la primera nota del arco mas cercano al Sol, mirando a la siguiente, y vuela un poco.
            let notas: Vec<Vec3> = juego.coleccion.notas.iter().map(|n| n.pos).collect();
            let (a, b) = (notas[0], notas[1]);
            let rumbo = (b - a).normalize();
            juego.nave.pos = a - rumbo * 6.0;
            (juego.nave.yaw, juego.nave.pitch) = salto::rumbo_hacia(rumbo);
            juego.camara = Camara::detras_de(&juego.nave);
            juego.coleccion.combo = 3;
            let mut ahora = 0.0;
            while ahora < t.max(0.9) {
                let hacia = juego.coleccion.notas.iter().filter(|n| n.viva).map(|n| n.pos).min_by(|p, q| (*p - juego.nave.pos).length().total_cmp(&(*q - juego.nave.pos).length())).unwrap_or(b);
                let (yaw, pitch) = salto::rumbo_hacia(hacia - juego.nave.pos);
                juego.nave.yaw = yaw;
                juego.nave.pitch = pitch;
                juego.actualizar(&Entrada { empuje: 1.0, ..Default::default() }, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            juego.mensaje = None;
        }
        Some(cual) if cual.starts_with("demo:") => {
            // `demo:<segundos>`: la demo del video corrida con el juego de verdad hasta ese momento
            // (para revisar cada parte del recorrido sin grabar).
            let hasta: f32 = cual["demo:".len()..].parse().unwrap_or(0.0);
            let (pos, rumbo) = demo::inicio(0.0);
            juego = Juego::new(pos, rumbo);
            let mut dm = demo::Demo::new();
            let mut ahora = 0.0;
            // (`actualizar` avanza el reloj del mundo, como en el juego.)
            while ahora < hasta && dm.fase != demo::Fase::Fin {
                let e = dm.entrada(&juego.nave, juego.orbita.is_some(), juego.t, 1.0 / 60.0);
                let antes = (juego.camara.ojo, juego.camara.mira);
                juego.actualizar(&e, 1.0 / 60.0);
                dm.mover_camara(&mut juego.camara, antes, &juego.nave, juego.t, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            juego.mensaje = None;
        }
        Some(cual) if cual.starts_with("cerca:") => {
            // `cerca:<archivo>` (sol, tierra, saturno_ritmo, chapin...): la nave pegada a ese
            // cuerpo, del lado de dia, mirandolo (para revisar como se ve y cuanto cuesta de cerca).
            let nombre = &cual["cerca:".len()..];
            let i = CUERPOS.iter().position(|c| c.archivo == nombre).unwrap_or(CHAPIN);
            let c = &CUERPOS[i];
            juego.t = 40.0;
            let centro = c.posicion(juego.t);
            // (El Sol se ve desde cualquier lado.)
            let hacia_sol = if i == 0 { Vec3::new(0.0, 0.0, 1.0) } else { (CUERPOS[0].posicion(juego.t) - centro).normalize() };
            let lado = (hacia_sol * 0.8 + hacia_sol.cross(Vec3::new(0.0, 1.0, 0.0)).normalize() * 0.55 + Vec3::new(0.0, 0.2, 0.0)).normalize();
            juego.nave.pos = centro + lado * (c.radio_visual() + 3.0);
            (juego.nave.yaw, juego.nave.pitch) = salto::rumbo_hacia(lado * -1.0 + Vec3::new(0.0, -0.15, 0.0));
            juego.camara = Camara::detras_de(&juego.nave);
            juego.sincronia = 1.0;
            juego.mensaje = None;
        }
        Some(cual) if cual.starts_with("aterrizar:") => {
            // `aterrizar:<planeta>`: aprieta E junto a ese planeta y mira `t` segundos despues.
            let nombre = &cual["aterrizar:".len()..];
            let i = CUERPOS.iter().position(|c| c.archivo == nombre).unwrap_or(TIERRA);
            juego.t = 40.0;
            juego.nave.pos = CUERPOS[i].posicion(40.0) + Vec3::new(CUERPOS[i].radio_visual() + alcance_de_aterrizaje(i).min(5.0) * 0.6, 1.0, 0.0);
            juego.camara = Camara::detras_de(&juego.nave);
            juego.actualizar(&Entrada { aterrizar: true, ..Default::default() }, 1.0 / 60.0);
            let mut ahora = 0.0;
            while ahora < t {
                juego.actualizar(&nada, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
        }
        Some(cual @ ("junta" | "junta_si")) => {
            // `junta`: deja que avance sola `t` segundos. `junta_si`: pasa todo, vota que si y espera `t`.
            let mut j = Junta::new();
            if cual == "junta_si" {
                for _ in 0..40 {
                    j.actualizar(&Entrada { aceptar: true, ..Default::default() }, 0.5);
                    if j.hablante().is_some_and(|h| h.1.starts_with("Hora de votar")) {
                        break;
                    }
                }
                j.actualizar(&Entrada { respuesta: Some(0), ..Default::default() }, 0.0);
                for _ in 0..(8.0 / (1.0 / 60.0)) as usize {
                    j.actualizar(&nada, 1.0 / 60.0);
                }
            }
            let mut ahora = 0.0;
            while ahora < t {
                j.actualizar(&nada, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            escena = Escena::Junta(j);
        }
        Some(cual) if cual.starts_with("cita") => {
            // `cita`: deja que avance sola `t` segundos. `cita_bien:<n>` / `cita_mal:<n>`: juega
            // toda la cita con la candidata `n` (0 a 2) contestando lo mejor o lo peor, y espera `t`.
            // `cita_golpiza`: dos citas perfectas y la golpiza. `cita_momento:<m>`: el momento sorpresa `m`.
            let mut c = match cual.split_once(':') {
                _ if cual == "cita_golpiza" => Cita::golpiza_de_prueba(),
                Some(("cita_momento", m)) => Cita::momento_de_prueba(2, m.parse().unwrap_or(0)),
                Some((modo, n)) => Cita::jugar(n.parse().unwrap_or(0), modo == "cita_bien"),
                None => Cita::new(),
            };
            let mut ahora = 0.0;
            while ahora < t {
                c.actualizar(&nada, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            escena = Escena::Cita(c);
        }
        Some(cual @ ("neptuno" | "neptuno_juego" | "neptuno_reto")) => {
            // `neptuno`: la intro, `t` segundos. `neptuno_juego`: Miku juega sola (como bot) `t` segundos.
            let mut n = Neptuno::new(2024);
            n.miku_automatica = cual == "neptuno_juego";
            let mut ahora = 0.0;
            while ahora < t {
                let e = if n.miku_automatica { Entrada { aceptar: true, ..Default::default() } } else { Entrada::default() };
                n.actualizar(&e, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            n.miku_automatica = false;
            if cual == "neptuno_reto" {
                n.preparar_reto();
            }
            escena = Escena::Neptuno(n);
        }
        Some(cual @ ("jupiter" | "jupiter_jefe")) => {
            // `jupiter`: juega solo `t` segundos. `jupiter_jefe`: lo mismo, contra la Gran Mancha Roja.
            let mut j = Jupiter::new(7, 12_000);
            j.jugar_solo(t, cual == "jupiter_jefe");
            escena = Escena::Jupiter(j);
        }
        Some(cual @ ("fuente" | "fuente_hada")) => {
            // `fuente`: deja correr `t` segundos. `fuente_hada`: toca la cancion y espera `t`.
            let mut f = Fuente::new();
            let aceptar = Entrada { aceptar: true, ..Default::default() };
            if cual == "fuente_hada" {
                for _ in 0..20 {
                    f.actualizar(&aceptar, 0.5);
                    if f.hablante().is_none() {
                        break;
                    }
                }
                for n in fuente::CANCION {
                    let e = match n {
                        fuente::Nota::A => Entrada { aceptar: true, ..Default::default() },
                        fuente::Nota::Arriba => Entrada { arriba: true, ..Default::default() },
                        fuente::Nota::Abajo => Entrada { abajo: true, ..Default::default() },
                        fuente::Nota::Izquierda => Entrada { izquierda: true, ..Default::default() },
                        fuente::Nota::Derecha => Entrada { derecha: true, ..Default::default() },
                    };
                    f.actualizar(&e, 0.5);
                }
            }
            let mut ahora = 0.0;
            while ahora < t {
                f.actualizar(&nada, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            escena = Escena::Fuente(f);
        }
        Some("mirador_bitacora") => {
            // La bitacora de GLADAS abierta (en la tercera entrada).
            let mut m = Mirador::new(1);
            m.abrir_bitacora(2);
            escena = Escena::Mirador(m);
        }
        Some("mirador") => {
            let mut m = Mirador::new(1);
            let mut prog = Progreso::en_memoria();
            let mut ahora = 0.0;
            while ahora < t {
                m.actualizar(&nada, 1.0 / 60.0, &mut prog);
                ahora += 1.0 / 60.0;
            }
            escena = Escena::Mirador(m);
        }
        Some(modo) if modo.starts_with("historia") => {
            // `historia:<capitulo>`: ese capitulo (desde 1), a los `t` segundos.
            let mut h = Historia::new();
            h.capitulo = modo.split_once(':').and_then(|(_, n)| n.parse::<usize>().ok()).unwrap_or(1).saturating_sub(1);
            h.tc = t;
            h.total = t;
            escena = Escena::Historia(h);
        }
        Some("espacio_mirador") => {
            // La nave llegando al Mirador, con una estela de arcoiris.
            let m = mirador_en(juego.t);
            let desde = m + Vec3::new(14.0, 3.0, 10.0);
            juego.nave = Nave::new(desde);
            (juego.nave.yaw, juego.nave.pitch) = salto::rumbo_hacia(m - desde);
            juego.camara = Camara::detras_de(&juego.nave);
            juego.estela = Some("arcoiris");
            let atras = (desde - m).normalize();
            juego.rastro = (0..50).rev().map(|k| desde + atras * (k as f32 * 0.35) + Vec3::new(0.0, 0.6 * (k as f32 * 0.3).sin(), 0.0)).collect();
        }
        Some(modo) if modo.starts_with("camioneta") => {
            // `camioneta`: la llegada. `camioneta_juego`: a media ruta, con gente.
            let mut c = Camioneta::new(11);
            c.preparar_captura(modo);
            let mut ahora = 0.0;
            while ahora < t {
                c.actualizar(&nada, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            escena = Escena::Camioneta(Box::new(c));
        }
        Some(modo) if modo.starts_with("luna") => {
            // `luna`: deja correr `t` segundos. `luna:<fase>` juega solo hasta esa fase (con `_mal`, contestando
            // mal; con `#n`, la n-esima vez) y luego espera `t`.
            let mut l = Luna::new(7);
            if let Some((_, fase)) = modo.split_once(':') {
                l.jugar_hasta(fase);
            }
            let mut ahora = 0.0;
            while ahora < t {
                l.actualizar(&nada, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            escena = Escena::Luna(l);
        }
        Some(modo) if modo.starts_with("divorcio") => {
            // `divorcio`: deja correr `t` segundos. `divorcio:<fase>` juega solo hasta esa fase (con `_mal`,
            // protestando mal; con `#n`, la n-esima vez) y luego espera `t`.
            let mut dv = Divorcio::new(7);
            if let Some((_, fase)) = modo.split_once(':') {
                dv.jugar_hasta(fase);
            }
            let mut ahora = 0.0;
            while ahora < t {
                dv.actualizar(&nada, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            escena = Escena::Divorcio(dv);
        }
        Some(modo) if modo.starts_with("mercurio") => {
            // `mercurio`: deja correr `t` segundos. `mercurio:<fase>` juega solo hasta esa fase y luego espera `t`.
            // Con `~salazon`, `~bendicion` o `~nifu` al final, busca una lectura que termine asi.
            let (modo, quiere) = modo.split_once('~').map_or((modo, None), |(m, q)| (m, Some(q)));
            let preparar = |semilla: u32| {
                let mut m = Mercurio::new(semilla);
                if let Some((_, fase)) = modo.split_once(':') {
                    m.jugar_hasta(fase);
                }
                m
            };
            let mut m = preparar(7);
            if let Some(q) = quiere {
                for semilla in 1..2000 {
                    let mut prueba = preparar(semilla);
                    prueba.jugar_hasta("Cobro");
                    let d = mercurio::Destino::de_puntos(prueba.puntos());
                    let sirve = match q {
                        "salazon" => d == mercurio::Destino::SalazonTotal,
                        "bendicion" => d == mercurio::Destino::Bendicion,
                        _ => d == mercurio::Destino::NiFuNiFa,
                    };
                    if sirve {
                        m = preparar(semilla);
                        break;
                    }
                }
            }
            let mut ahora = 0.0;
            while ahora < t {
                m.actualizar(&nada, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            escena = Escena::Mercurio(m);
        }
        Some("ritmo") => {
            // Juega solo: aprieta cada nota justo a tiempo.
            let mut r = Ritmo::new();
            r.empezar(ritmo::Dificultad::Normal);
            let notas = ritmo::notas_del_juego(r.dificultad);
            let paso = 1.0 / 120.0;
            let mut ahora = 0.0f32;
            while ahora < t {
                let toca = |aire: bool| notas.iter().any(|&(tn, a)| a == aire && tn > ahora && tn <= ahora + paso);
                r.actualizar(&Entrada { piso: toca(false), aire: toca(true), ..Default::default() }, paso);
                ahora += paso;
            }
            escena = Escena::Ritmo(r);
        }
        Some("concierto") => {
            let mut c = Concierto::new();
            c.actualizar(&nada, t);
            escena = Escena::Concierto(c);
        }
        Some(modo) if modo.starts_with("vegetta") => {
            // `vegetta`: la llegada. `vegetta_carrera`: el parkour, corriendo solo `t` segundos.
            // `vegetta_lobby`, `vegetta_tnt` y `vegetta_mina`: el lobby y los otros minijuegos.
            let atlas = obj::Textura::cargar("modelos/texturas/bloques.png").ok().map(std::sync::Arc::new);
            let mut v = Vegetta::new(2024, atlas);
            if modo == "vegetta_carrera" {
                v.correr_solo(t);
            } else if modo != "vegetta" {
                v.preparar_captura(modo);
            } else {
                let mut ahora = 0.0;
                while ahora < t {
                    v.actualizar(&nada, 1.0 / 60.0);
                    ahora += 1.0 / 60.0;
                }
            }
            escena = Escena::Vegetta(Box::new(v));
        }
        Some(modo) if modo.starts_with("urano") => {
            // `urano`: la llegada. `urano_juego`: a media ronda, con plaga por todos lados.
            // `urano_jefe`: con el Ladrillo. `urano_final`: el baño limpio. Luego espera `t` segundos.
            let mut u = Urano::new(77);
            u.preparar_captura(modo);
            let mut ahora = 0.0;
            while ahora < t {
                u.actualizar(&nada, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            escena = Escena::Urano(Box::new(u));
        }
        Some(modo) if modo.starts_with("aula") => {
            // `aula`: la llegada. `aula_pregunta`, `aula_bien`, `aula_mal` (Koch con la regla en
            // alto), `aula_reglazo` (el golpe) y `aula_final`. Luego espera `t` segundos.
            let mut a = Aula::new(4242);
            a.preparar_captura(modo);
            let mut ahora = 0.0;
            while ahora < t {
                a.actualizar(&nada, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            escena = Escena::Aula(Box::new(a));
        }
        Some(modo) if modo.starts_with("entrega") => {
            // `entrega`: la llegada. `entrega_graficas`, `entrega_teoria`, `entrega_software`,
            // `entrega_teams`, `entrega_windows`, `entrega_luz`, `entrega_cafe`, `entrega_sueno`,
            // `entrega_final`: ya en ese momento. Luego espera `t` segundos.
            let mut en = Entrega::new(4242);
            en.preparar_captura(modo);
            let mut ahora = 0.0;
            while ahora < t {
                en.actualizar(&nada, 1.0 / 60.0);
                ahora += 1.0 / 60.0;
            }
            escena = Escena::Entrega(Box::new(en));
        }
        _ => {}
    }
    (juego, escena)
}

fn captura(archivo: &str, t: f32, resto: &[String]) {
    let mut rec = Recursos::cargar();
    // `relieve:<x>` y `semilla:<n>` junto con `cerca:<planeta>`: el planeta con ese ajuste, como
    // si se le hubiera cambiado en vivo (F5 a F8).
    let pedido = |prefijo: &str| resto.iter().find_map(|s| s.strip_prefix(prefijo).and_then(|v| v.parse::<f32>().ok()));
    if let (Some(cual), true) = (resto.iter().find_map(|s| s.strip_prefix("cerca:")), pedido("relieve:").is_some() || pedido("semilla:").is_some()) {
        if let Some(i) = CUERPOS.iter().position(|c| c.archivo == cual) {
            let ajuste = planetas::Ajuste { semilla: pedido("semilla:").unwrap_or(0.0) as i32 as u32, relieve: pedido("relieve:").unwrap_or(1.0) };
            rec.rearmar(i, ajuste);
        }
    }
    let rec = rec;
    let resto: Vec<String> = resto.iter().filter(|s| !s.starts_with("relieve:") && !s.starts_with("semilla:")).cloned().collect();
    let resto = &resto[..];
    // `personaje:<id>` (al final o en cualquier lado): con ese personaje en vez del de fabrica.
    if let Some(id) = resto.iter().find_map(|s| s.strip_prefix("personaje:")) {
        rec.personaje.set(personajes::indice(id));
    }
    let resto: Vec<String> = resto.iter().filter(|s| !s.starts_with("personaje:")).cloned().collect();
    let resto = &resto[..];
    let (juego, escena) = preparar(t, resto);
    let mut fb = Framebuffer::new(RENDER_W, RENDER_H);
    if resto.last().is_some_and(|s| s == "estelas" || s == "naves") {
        // Todas las estelas (o todas las naves de la tienda) en una hoja, para revisarlas.
        let naves = resto.last().is_some_and(|s| s == "naves");
        let (lado, columnas) = (240usize, 5usize);
        let cuantas = if naves { tienda::SKINS.len() } else { progreso::ESTELAS.len() };
        let filas = cuantas.div_ceil(columnas);
        let mut hoja = vec![0u8; lado * columnas * lado * filas * 3];
        for k in 0..cuantas {
            let mut fb = Framebuffer::new(lado, lado);
            fb.limpiar([12, 8, 30]);
            if naves {
                dibujar_miniatura_nave(&mut fb, &rec, k, None);
            } else {
                dibujar_miniatura_nave(&mut fb, &rec, 0, Some(progreso::ESTELAS[k].0));
            }
            let (cx, cy) = (k % columnas, k / columnas);
            for (i, px) in fb.bytes().chunks_exact(4).enumerate() {
                let (x, y) = (i % lado, i / lado);
                let j = ((cy * lado + y) * lado * columnas + cx * lado + x) * 3;
                let a = px[3] as f32 / 255.0;
                for q in 0..3 {
                    hoja[j + q] = (px[q] as f32 * a + [12.0, 8.0, 30.0][q] * (1.0 - a)) as u8;
                }
            }
        }
        let mut ppm = format!("P6\n{} {}\n255\n", lado * columnas, lado * filas).into_bytes();
        ppm.extend(hoja);
        std::fs::write(archivo, ppm).expect("no se pudo guardar la captura");
        return;
    }
    if let Some(ids) = resto.last().and_then(|s| s.strip_prefix("vitrina:")) {
        // Uno o varios personajes (separados por comas) en la vitrina del casillero, lado a lado.
        let ids: Vec<&str> = ids.split(',').collect();
        let (w, h) = (480usize, 540usize);
        let mut hoja = vec![0u8; w * ids.len() * h * 3];
        for (n, id) in ids.iter().enumerate() {
            let mut fb = Framebuffer::new(w, h);
            dibujar_vitrina_personaje(&mut fb, &rec, personajes::indice(id), t);
            for (i, px) in fb.bytes().chunks_exact(4).enumerate() {
                let (x, y) = (i % w, i / w);
                let j = (y * w * ids.len() + n * w + x) * 3;
                hoja[j..j + 3].copy_from_slice(&px[..3]);
            }
        }
        let mut ppm = format!("P6\n{} {}\n255\n", w * ids.len(), h).into_bytes();
        ppm.extend(hoja);
        std::fs::write(archivo, ppm).expect("no se pudo guardar la captura");
        return;
    }
    if resto.last().is_some_and(|s| s == "miku_menu") {
        // Solo Miku para los menus, en su framebuffer (lo transparente sale negro).
        let mut fb = Framebuffer::new(300, 450);
        dibujar_miku_menu(&mut fb, &rec, t);
        let mut ppm = format!("P6\n{} {}\n255\n", fb.ancho, fb.alto).into_bytes();
        ppm.extend(fb.bytes().chunks_exact(4).flat_map(|px| [px[0], px[1], px[2]]));
        std::fs::write(archivo, ppm).expect("no se pudo guardar la captura");
        return;
    }
    if resto.last().is_some_and(|s| s == "gladas") {
        // Solo el retrato de GLADAS, en su framebuffer chiquito.
        fb = Framebuffer::new(RETRATO, RETRATO);
        dibujar_retrato(&mut fb, &rec, t, true);
        let mut ppm = format!("P6\n{} {}\n255\n", fb.ancho, fb.alto).into_bytes();
        ppm.extend(fb.bytes().chunks_exact(4).flat_map(|px| [px[0], px[1], px[2]]));
        std::fs::write(archivo, ppm).expect("no se pudo guardar la captura");
        return;
    }
    let mut post = Post::new(&fb);
    let empezo = std::time::Instant::now();
    let st = dibujar(&mut fb, &mut post, &rec, &juego, &escena);
    let ms = empezo.elapsed().as_secs_f32() * 1000.0;
    let mut ppm = format!("P6\n{} {}\n255\n", fb.ancho, fb.alto).into_bytes();
    ppm.extend(fb.bytes().chunks_exact(4).flat_map(|px| [px[0], px[1], px[2]]));
    std::fs::write(archivo, ppm).expect("no se pudo guardar la captura");
    println!("{archivo}: {} triangulos, {} dibujados, {ms:.1} ms", st.triangulos, st.dibujados);
}

fn dibujar(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, juego: &Juego, escena: &Escena) -> Estadisticas {
    let st = dibujar_escena(fb, post, rec, juego, escena);
    // Las vistas de depuracion que leen los buffers directo.
    match juego.opciones.vista {
        Vista::Profundidad => fb.ver_profundidad(),
        Vista::Stencil => fb.ver_stencil(),
        _ => {}
    }
    st
}

/// Los finales y logros que ya se cumplieron en la escena (se revisan cada cuadro; desbloquear dos
/// veces no hace nada).
fn logros_de_escena(escena: &Escena) -> Vec<&'static str> {
    let mut v = Vec::new();
    match escena {
        Escena::Vegetta(ve) => {
            if ve.terminadas > 0 {
                v.push("vegetta_juega");
            }
            if ve.ganadas > 0 {
                v.push("vegetta_carrera");
            }
            // El marcador: (tu, Willy) en parkour, TNT Run y mineria.
            if ve.marcador[1].0 > 0 {
                v.push("vegetta_tnt");
            }
            if ve.marcador[2].0 > 0 {
                v.push("vegetta_mina");
            }
            if ve.marcador.iter().all(|m| m.0 > 0) {
                v.push("vegetta_triple");
            }
        }
        Escena::Urano(u) => {
            if u.partidas > 0 {
                v.push("urano_juega");
            }
            if u.campeon {
                v.push("urano_campeon");
            }
        }
        Escena::Aula(a) => {
            if a.terminados > 0 {
                v.push("koch_examen");
            }
            if a.perfectos > 0 {
                v.push("koch_5");
            }
            if a.reprobados_a_reglazos > 0 {
                v.push("koch_reglazo");
            }
        }
        Escena::Entrega(en) => {
            if let Some(notas) = en.resultado_final() {
                if notas.iter().all(|&n| n >= 100) {
                    v.push("examen_5");
                }
                if notas.iter().all(|&n| n >= 61) {
                    v.push("examen_aprobada");
                }
            }
            if en.se_durmio {
                v.push("examen_reglazo");
            }
        }
        Escena::Cita(c) if c.terminada() => v.push(match c.final_() {
            Final::Boda => "cita_boda",
            Final::Exito => "cita_exito",
            Final::Amigos => "cita_amigos",
            Final::Solo => "cita_solo",
            Final::Drama(_) => "cita_drama",
        }),
        Escena::Ritmo(r) => {
            if let Some((dificultad, nota, fallos)) = r.resultado() {
                v.push("ritmo_termino");
                if nota == "S" {
                    v.push("ritmo_s");
                }
                if dificultad == ritmo::Dificultad::Dificil {
                    v.push("ritmo_dificil");
                }
                if fallos == 0 {
                    v.push("ritmo_sin_fallos");
                }
            }
        }
        Escena::Neptuno(n) => {
            match n.ganador {
                Some(neptuno::MIKU) => v.push("uno_gana"),
                Some(_) => v.push("uno_pierde"),
                None => {}
            }
            let (canela, pluton) = n.visitas();
            if canela {
                v.push("uno_canela");
            }
            if pluton {
                v.push("uno_pluton");
            }
        }
        Escena::Junta(j) => match j.resultado() {
            Some(junta::Voto::Si) => v.push("junta_si"),
            Some(_) => v.push("junta_no"),
            None => {}
        },
        Escena::Fuente(f) => {
            if f.hada_afuera() {
                v.push("hada_luka");
            }
            if f.reganando() {
                v.push("hada_error");
            }
        }
        Escena::Mercurio(m) if m.terminada() => {
            v.push("tarot_lectura");
            if m.invertidas() == 3 {
                v.push("tarot_invertidas");
            }
            match m.destino() {
                Some(mercurio::Destino::Bendicion) => v.push("tarot_bendicion"),
                Some(mercurio::Destino::SalazonTotal) => v.push("tarot_salazon"),
                _ => {}
            }
            if m.limpia_hecha {
                v.push("tarot_limpia");
            }
        }
        Escena::Divorcio(d) => {
            if d.cuantos(divorcio::Destino::Ganso) > 0 {
                v.push("divorcio_ganso");
            }
            if d.terminado() {
                match d.cuantos(divorcio::Destino::Saturno) {
                    3 => v.push("divorcio_3"),
                    0 => v.push("divorcio_0"),
                    _ => {}
                }
            }
        }
        Escena::Mirador(_) => v.push("mirador"),
        Escena::Camioneta(c) => {
            if c.llegadas > 0 {
                v.push("camioneta_terminal");
            }
            if c.mejor_pasajeros >= 40 {
                v.push("camioneta_llena");
            }
            if c.mejor_tumulos >= 5 {
                v.push("camioneta_tumulos");
            }
        }
        Escena::Luna(l) if l.calificada() => {
            match l.aciertos() {
                6 => v.push("luna_oscar"),
                0 | 1 => v.push("luna_razzie"),
                _ => {}
            }
            if l.estrellas_pintadas {
                v.push("luna_estrellas");
            }
            if l.cables {
                v.push("luna_cables");
            }
        }
        _ => {}
    }
    v
}

/// Lo que paga un minijuego al despegar (en notas) y por que, segun que tan bien se jugo en esta
/// visita. Asi todos los minijuegos sirven para juntar para la tienda, no solo volar. (Jupiter,
/// Mercurio y los diamantes de Vegetta ya pagan en el momento; aqui va lo demas.)
fn pago_de_escena(escena: &Escena) -> Option<(u32, String)> {
    let pago = match escena {
        Escena::Urano(u) if u.partidas > 0 => {
            let n = u.partidas * 8 + u.dias_ganados * 15 + if u.campeon { 30 } else { 0 };
            (n, format!("baño de Doña Chepa: {} día{} cumplido{}", u.dias_ganados, if u.dias_ganados == 1 { "" } else { "s" }, if u.dias_ganados == 1 { "" } else { "s" }))
        }
        Escena::Aula(a) if a.terminados > 0 => (10 + a.puntaje as u32 * 6 + if a.perfectos > 0 { 20 } else { 0 }, format!("examen de Koch: {} de 5", a.puntaje)),
        Escena::Entrega(en) => {
            let notas = en.resultado_final()?;
            let promedio = notas.iter().sum::<u32>() / 3;
            (promedio * 45 / 100 + if notas.iter().all(|&n| n >= 100) { 20 } else { 0 }, format!("la entrega: promedio {promedio}"))
        }
        Escena::Cita(c) if c.terminada() => {
            let (n, que) = match c.final_() {
                Final::Boda => (50, "¡boda!"),
                Final::Exito => (40, "cita exitosa"),
                Final::Amigos => (25, "quedaron de amigos"),
                Final::Drama(_) => (20, "hubo drama"),
                Final::Solo => (10, "Curiosity sigue solo"),
            };
            (n, format!("la cita de Marte: {que}"))
        }
        Escena::Ritmo(r) => {
            let (dificultad, nota, fallos) = r.resultado()?;
            let base = match nota {
                "S" => 50,
                "A" => 38,
                "B" => 26,
                _ => 15,
            };
            let n = base * if dificultad == ritmo::Dificultad::Dificil { 3 } else { 2 } / 2 + if fallos == 0 { 10 } else { 0 };
            (n, format!("Saturno Ritmo: nota {nota}"))
        }
        Escena::Neptuno(n) => match n.ganador {
            Some(neptuno::MIKU) => (40, "¡le ganó a los héroes en cartas!".to_string()),
            Some(_) => (12, "las cartas de Neptuno (por participar)".to_string()),
            None => return None,
        },
        Escena::Junta(j) if j.resultado().is_some() => (25, "la junta de Plutón".to_string()),
        Escena::Fuente(f) if f.hada_afuera() => (35, "la canción de la Fuente".to_string()),
        Escena::Mercurio(m) if m.terminada() => (15, "la lectura de Madame Mercurio".to_string()),
        Escena::Divorcio(d) if d.terminado() => (15 + 8 * d.cuantos(divorcio::Destino::Saturno) as u32, "el divorcio de Venus".to_string()),
        Escena::Luna(l) if l.calificada() => (l.aciertos() as u32 * 7, format!("el set de la Luna: {} de 6", l.aciertos())),
        Escena::Camioneta(c) if c.partidas > 0 => (c.partidas * 5 + c.mejor_pasajeros + if c.llegadas > 0 { 15 } else { 0 }, format!("la camioneta: {} pasajeros a la capital", c.mejor_pasajeros)),
        Escena::Vegetta(v) if v.terminadas > 0 => (v.terminadas * 5 + v.ganadas * 15, format!("el servidor de Vegetta: {} de {} ganados", v.ganadas, v.terminadas)),
        _ => return None,
    };
    (pago.0 > 0).then_some(pago)
}

fn dibujar_escena(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, juego: &Juego, escena: &Escena) -> Estadisticas {
    let st = dibujar_escena_sin_trazo(fb, post, rec, juego, escena);
    // El estilo dibujado: las sombras de contacto (solo con la vista normal; las vistas del
    // pipeline, como el z-buffer, se ven tal cual).
    if pipeline::DIBUJADO.load(std::sync::atomic::Ordering::Relaxed) && juego.opciones.vista == Vista::Color && juego.opciones.modo == ModoDibujo::Relleno {
        post::sombras_de_contacto(fb, 7, 0.4);
    }
    st
}

fn dibujar_escena_sin_trazo(fb: &mut Framebuffer, post: &mut Post, rec: &Recursos, juego: &Juego, escena: &Escena) -> Estadisticas {
    match escena {
        Escena::Intro(i) => dibujar_espacio(fb, post, rec, juego, Some(i)),
        Escena::Espacio => dibujar_espacio(fb, post, rec, juego, None),
        Escena::Entrega(en) => dibujar_entrega(fb, post, rec, en, &juego.opciones),
        Escena::Aula(a) => dibujar_aula(fb, post, rec, a, &juego.opciones),
        Escena::Urano(u) => dibujar_urano(fb, post, rec, u, &juego.opciones),
        Escena::Vegetta(v) => dibujar_vegetta(fb, post, rec, v, &juego.opciones),
        Escena::Concierto(c) => dibujar_concierto(fb, post, rec, c, &juego.opciones),
        Escena::Ritmo(r) => dibujar_ritmo(fb, post, rec, r, &juego.opciones),
        Escena::Junta(j) => dibujar_junta(fb, post, rec, j, &juego.opciones),
        Escena::Cita(c) => dibujar_cita(fb, post, rec, c, &juego.opciones),
        Escena::Neptuno(n) => dibujar_neptuno(fb, post, rec, n, &juego.opciones),
        Escena::Fuente(f) => dibujar_fuente(fb, post, rec, f, &juego.opciones),
        Escena::Mercurio(m) => dibujar_mercurio(fb, post, rec, m, &juego.opciones),
        Escena::Divorcio(d) => dibujar_divorcio(fb, post, rec, d, &juego.opciones),
        Escena::Jupiter(j) => dibujar_jupiter(fb, post, rec, j, &juego.opciones),
        Escena::Luna(l) => dibujar_luna(fb, post, rec, l, &juego.opciones),
        Escena::Camioneta(c) => dibujar_camioneta(fb, post, rec, c, &juego.opciones),
        Escena::Mirador(m) => dibujar_mirador(fb, post, rec, m, &juego.opciones),
        Escena::Historia(h) => dibujar_historia(fb, post, rec, h, &juego.opciones),
        // Atras del menu principal, el sistema solar girando (con la camara dando vueltas a la Tierra).
        Escena::Menu(_) => dibujar_espacio(fb, post, rec, juego, None),
    }
}

/// Guarda lo que hay en la ventana como imagen, a la resolucion real de la pantalla. (La de raylib,
/// `TakeScreenshot`, lee mal la imagen en pantallas Retina: multiplica dos veces por la escala.)
fn foto_de_la_ventana(rl: &RaylibHandle, archivo: &str) {
    use raylib::ffi;
    let (w, h) = (rl.get_render_width(), rl.get_render_height());
    let ruta = std::ffi::CString::new(archivo).unwrap_or_default();
    unsafe {
        let datos = ffi::rlReadScreenPixels(w, h);
        let imagen = ffi::Image { data: datos as *mut std::os::raw::c_void, width: w, height: h, mipmaps: 1, format: ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32 };
        ffi::ExportImage(imagen, ruta.as_ptr());
        ffi::MemFree(datos as *mut std::os::raw::c_void);
    }
}

/// El juego lee `modelos/` y `assets/` desde la carpeta actual. Si se abrio desde otro lado (doble clic
/// al ejecutable en `target/release`, o un acceso directo), busca la carpeta del juego: junto al
/// ejecutable, en las carpetas de arriba, o donde se compilo, y se cambia ahi.
fn ir_a_la_carpeta_del_juego() {
    let es_la_buena = |d: &std::path::Path| d.join("modelos").join("nave_lila.obj").exists();
    if es_la_buena(std::path::Path::new(".")) {
        return;
    }
    let mut candidatas: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        candidatas.extend(exe.ancestors().skip(1).take(5).map(|p| p.to_path_buf()));
    }
    candidatas.push(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    if let Some(d) = candidatas.into_iter().find(|d| es_la_buena(d)) {
        let _ = std::env::set_current_dir(d);
    }
}

fn main() {
    ir_a_la_carpeta_del_juego();
    // Solo advertencias y errores de raylib (sin una linea por cada sonido que carga).
    unsafe { raylib::ffi::SetTraceLogLevel(raylib::ffi::TraceLogLevel::LOG_WARNING as i32) };
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--captura") {
        let archivo = args.get(i + 1).map_or("captura.ppm", |s| s.as_str());
        let t = args.get(i + 2).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        captura(archivo, t, args.get(i + 3..).unwrap_or(&[]));
        return;
    }
    // `--foto-ui archivo.png [segundos] [modo...] [mensaje] [info] [completa]`: abre el juego en esa escena
    // (como `--captura`), deja correr dos segundos y medio, saca una foto de la ventana CON la
    // interfaz y se cierra. `mensaje` pone a GLADAS a hablar; `info` abre la info del pipeline.
    let foto: Option<(String, f32, Vec<String>)> = args.iter().position(|a| a == "--foto-ui").map(|i| {
        let archivo = args.get(i + 1).cloned().unwrap_or_else(|| "foto.png".into());
        let t = args.get(i + 2).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        (archivo, t, args.get(i + 3..).unwrap_or(&[]).to_vec())
    });
    // `--grabar video.mp4 [desde] [hasta] [--limpio]`: el recorrido por todo el juego, grabado cuadro por cuadro con
    // ffmpeg (ver `grabacion.rs`). Por dentro es como una foto de la interfaz que no se acaba: la
    // partida es de mentira, el teclado no cuenta y cada toma arma su escena.
    let mut grabar = args.iter().position(|a| a == "--grabar").map(|i| {
        let mut g = grabacion::Grabacion::new(args.get(i + 1).map_or("space_adventure.mp4", |s| s.as_str()));
        g.toma = args.get(i + 2).and_then(|s| s.parse().ok()).unwrap_or(0);
        g.hasta = args.get(i + 3).and_then(|s| s.parse().ok()).unwrap_or(grabacion::GUION.len());
        g.limpio = args.iter().any(|a| a == "--limpio");
        g
    });
    let foto = foto.or_else(|| grabar.as_ref().map(|_| (String::new(), 0.0, Vec::new())));
    // `--probar-vibracion sin-ventana`: la misma prueba, antes de que raylib abra la ventana (y con
    // ella el control): asi se ve si los dos se pelean por el.
    if args.iter().any(|a| a == "--probar-vibracion") && args.iter().any(|a| a == "sin-ventana") {
        vibracion::diagnostico(&|| {});
        return;
    }
    let mut rec = Recursos::cargar();

    // En pantallas de alta densidad (Retina) se dibuja a la resolucion real de la pantalla; si no,
    // macOS estira la ventana al doble y todo (sobre todo el texto) se ve borroso.
    unsafe { raylib::ffi::SetConfigFlags(raylib::ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32) };
    // Solo advertencias y errores de raylib (sin una linea por cada sonido que carga).
    let (mut rl, thread) = raylib::init().size(VENTANA_W, VENTANA_H).title("Space Adventure").resizable().log_level(TraceLogLevel::LOG_WARNING).build();
    rl.set_window_min_size(640, 360);
    rl.set_target_fps(60);
    // ESC abre la pausa (raylib, por defecto, cerraria el juego de golpe).
    rl.set_exit_key(None);
    // La ventana cabe siempre en la pantalla: a lo mas el 90% del monitor, en 16:9 y centrada.
    let (monitor_w, monitor_h) = unsafe {
        let m = raylib::ffi::GetCurrentMonitor();
        (raylib::ffi::GetMonitorWidth(m), raylib::ffi::GetMonitorHeight(m))
    };
    if monitor_w > 0 && monitor_h > 0 {
        let escala = (monitor_w as f32 * 0.9 / VENTANA_W as f32).min(monitor_h as f32 * 0.82 / VENTANA_H as f32).min(1.0);
        let (w, h) = ((VENTANA_W as f32 * escala) as i32, (VENTANA_H as f32 * escala) as i32);
        rl.set_window_size(w, h);
        rl.set_window_position((monitor_w - w) / 2, ((monitor_h - h) / 2).max(30));
    }
    // Tamaño explicito para revisar las ayudas a distintas resoluciones sin tocar la partida.
    if let Some((_, _, opciones)) = &foto {
        if let Some((w, h)) = opciones.iter().find_map(|o| {
            let (w, h) = o.strip_prefix("ventana:")?.split_once('x')?;
            Some((w.parse::<i32>().ok()?, h.parse::<i32>().ok()?))
        }) {
            rl.set_window_size(w.clamp(640, 1920), h.clamp(360, 1080));
        }
    }
    // La grabacion, siempre al tamano de diseno (en Retina sale al doble de pixeles).
    if grabar.is_some() {
        rl.set_window_size(VENTANA_W, VENTANA_H);
        rl.set_window_position(20, 40);
    }
    ui::cargar_fuente(&thread, rl.get_window_scale_dpi().x);
    ui::cargar_sprites(&thread);
    controles::cargar(&thread);
    // El audio es opcional: sin dispositivo, el juego sigue en silencio.
    let audio = RaylibAudio::init_audio_device().ok();
    let sonidos = audio.as_ref().map(|a| Sonidos::cargar(a, grabar.is_some()));
    let son = sonidos.as_ref();
    // La vibracion del control (si hay SDL2 instalado; si no, el juego sigue igual).
    if args.iter().any(|a| a == "--probar-vibracion") {
        #[cfg(haptica_apple)]
        haptica::Haptica::diagnostico(&|| unsafe { raylib::ffi::PollInputEvents() });
        vibracion::diagnostico(&|| unsafe { raylib::ffi::PollInputEvents() });
        return;
    }
    // (Las fotos no abren SDL: si el juego esta abierto con el control, se pelearian por el.)
    let vibra = if foto.is_some() || args.iter().any(|a| a == "sin-sdl") { None } else { vibracion::Vibrador::abrir() };
    // `--probar-control [segundos]`: dice que controles ve el juego y que se esta apretando.
    if let Some(i) = args.iter().position(|a| a == "--probar-control") {
        let segundos: f32 = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(8.0);
        let mut pasado = 0.0;
        let mut antes = String::new();
        println!("PRUEBA del control: {segundos} s. Vibrador abierto: {}", vibra.is_some());
        while pasado < segundos && !rl.window_should_close() {
            let mut linea = String::new();
            for g in 0..4 {
                if rl.is_gamepad_available(g) {
                    let nombre = rl.get_gamepad_name(g).unwrap_or_default();
                    let ejes = rl.get_gamepad_axis_count(g);
                    let boton = unsafe { raylib::ffi::GetGamepadButtonPressed() };
                    linea += &format!("[{g}] {nombre} ({ejes} ejes) ");
                    for k in 0..ejes.min(6) {
                        linea += &format!("{:+.1} ", unsafe { raylib::ffi::GetGamepadAxisMovement(g, k) });
                    }
                    linea += &format!("boton {boton} ");
                }
            }
            #[cfg(haptica_apple)]
            if let Some(l) = haptica::leer() {
                linea += &format!("APPLE {} ", l.nombre);
                for v in l.ejes {
                    linea += &format!("{v:+.1} ");
                }
                linea += &format!("botones {:#x}", l.botones);
            }
            if linea.is_empty() {
                linea = "ningun control disponible".into();
            }
            if linea != antes {
                println!("{pasado:5.1} s: {linea}");
                antes = linea;
            }
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);
            d.draw_text("Prueba del control: mueva los sticks y apriete botones", 40, 40, 24, Color::WHITE);
            d.draw_text(&antes, 40, 90, 18, Color::GREEN);
            drop(d);
            pasado += rl.get_frame_time();
        }
        return;
    }
    let vibrar_activo = std::cell::Cell::new(true);
    // Cada sonido del juego, y su vibracion (los golpes, el turbo, el ¡PROTESTO!...).
    let tocar = |e: Efecto| {
        if let Some(s) = son {
            s.efecto(e);
        }
        if let (true, Some(v), Some((grave, agudo, ms))) = (vibrar_activo.get(), &vibra, vibracion::de_efecto(e)) {
            v.vibrar(grave, agudo, ms);
        }
    };
    if let Some(s) = son {
        s.reiniciar(Pista::Cancion);
    }
    let (mut dialogo_ia, mut dialogo_profe, mut dialogo_miku) = (Dialogo::new(), Dialogo::new(), Dialogo::new());
    // Bill Nye: lo que esta diciendo (y cuanto lleva) y su caja de dialogo.
    let mut bill_dice: Option<(String, f32)> = None;
    let mut dialogo_bill = Dialogo::new();

    let mut fb = Framebuffer::new(RENDER_W, RENDER_H);
    let mut post = Post::new(&fb);
    let imagen = Image::gen_image_color(RENDER_W as i32, RENDER_H as i32, Color::BLACK);
    let mut textura = rl.load_texture_from_image(&thread, &imagen).expect("no se pudo crear la textura");
    textura.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    // El retrato de GLADAS: su propio framebuffer y su propia textura.
    let mut fb_retrato = Framebuffer::new(RETRATO, RETRATO);
    let imagen_retrato = Image::gen_image_color(RETRATO as i32, RETRATO as i32, Color::BLACK);
    let mut textura_retrato = rl.load_texture_from_image(&thread, &imagen_retrato).expect("no se pudo crear la textura del retrato");
    textura_retrato.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    // Si GLADAS hablaba en el cuadro anterior (el retrato se dibuja antes que la interfaz).
    let mut gladas_hablaba = false;
    // Miku en grande para los menus: su propio framebuffer (transparente) y su textura.
    let mut fb_miku = Framebuffer::new(300, 450);
    // Su propio reloj: en la pausa el del juego se congela, pero ella sigue bailando.
    let mut reloj_menu = 0.0f32;
    let mut textura_miku = rl.load_texture_from_image(&thread, &Image::gen_image_color(300, 450, Color::BLANK)).expect("no se pudo crear la textura de Miku");
    textura_miku.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);

    // La vitrina de la tienda: la nave girando con la skin escogida.
    let mut fb_vitrina = Framebuffer::new(480, 340);
    let mut textura_vitrina = rl.load_texture_from_image(&thread, &Image::gen_image_color(480, 340, Color::BLANK)).expect("no se pudo crear la vitrina");
    textura_vitrina.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    // Los retratos de las tarjetas del casillero: se pintan una vez al arrancar.
    // El retrato de Bill Nye, para su caja de dialogo.
    let retrato_bill = {
        let mut fb_r = Framebuffer::new(160, 160);
        dibujar_retrato_de(&mut fb_r, &rec.bill.de_pie);
        let mut t = rl.load_texture_from_image(&thread, &Image::gen_image_color(160, 160, Color::BLANK)).expect("no se pudo crear el retrato de Bill");
        t.update_texture(fb_r.bytes()).expect("no se pudo pintar el retrato de Bill");
        t.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        t
    };
    let retratos: Vec<Texture2D> = (0..personajes::PERSONAJES.len())
        .map(|i| {
            let mut fb_r = Framebuffer::new(160, 160);
            dibujar_retrato_personaje(&mut fb_r, &rec, i);
            let mut t = rl.load_texture_from_image(&thread, &Image::gen_image_color(160, 160, Color::BLANK)).expect("no se pudo crear un retrato");
            t.update_texture(fb_r.bytes()).expect("no se pudo pintar un retrato");
            t.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            t
        })
        .collect();
    // Los sellos del pasaporte: cada planeta con sello, y al final la nave (el sello del espacio).
    let mut sellos: Vec<Texture2D> = progreso::CON_SELLO
        .iter()
        .map(|&i| {
            let mut fb_s = Framebuffer::new(128, 128);
            let (modelo, anim) = rec.planeta_detallado(i);
            dibujar_miniatura_planeta(&mut fb_s, modelo, anim);
            let mut t = rl.load_texture_from_image(&thread, &Image::gen_image_color(128, 128, Color::BLANK)).expect("no se pudo crear un sello");
            t.update_texture(fb_s.bytes()).expect("no se pudo pintar un sello");
            t.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            t
        })
        .collect();
    // Las miniaturas de los premios del pase: cada skin de nave y cada estela.
    let mut miniatura = |skin: usize, estela: Option<&str>| {
        let mut fb_m = Framebuffer::new(160, 160);
        dibujar_miniatura_nave(&mut fb_m, &rec, skin, estela);
        let mut t = rl.load_texture_from_image(&thread, &Image::gen_image_color(160, 160, Color::BLANK)).expect("no se pudo crear una miniatura");
        t.update_texture(fb_m.bytes()).expect("no se pudo pintar una miniatura");
        t.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        t
    };
    let miniaturas_naves: Vec<Texture2D> = (0..tienda::SKINS.len()).map(|k| miniatura(k, None)).collect();
    let miniaturas_estelas: Vec<Texture2D> = progreso::ESTELAS.iter().map(|e| miniatura(0, Some(e.0))).collect();
    sellos.push(miniatura(0, None));
    // Las fotos de la mochila: las cuatro lunas y cada recuerdo, en 3D.
    let fotos_mochila: Vec<Texture2D> = (0..4 + mochila::CUANTOS)
        .map(|k| {
            let mut fb_m = Framebuffer::new(160, 160);
            fb_m.limpiar_transparente();
            dibujar_recuerdo(&mut fb_m, &rec, k, 0.0);
            let mut t = rl.load_texture_from_image(&thread, &Image::gen_image_color(160, 160, Color::BLANK)).expect("no se pudo crear la foto de un recuerdo");
            t.update_texture(fb_m.bytes()).expect("no se pudo pintar un recuerdo");
            t.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            t
        })
        .collect();
    let imagenes = casillero::Imagenes { retratos: &retratos, naves: &miniaturas_naves, estelas: &miniaturas_estelas, recuerdos: &fotos_mochila };
    // La tienda, el casillero o el pase, si hay uno abierto encima del menu.
    let mut submenu: Option<Submenu> = None;

    let mut juego = Juego::new(intro::nave_parada(0.0), 0.0);
    let mut escena = Escena::Intro(Intro::new());
    // El fundido a negro: `cambio` es a donde se va cuando la pantalla llegue a negro.
    let mut fundido = 1.0f32;
    let mut cuadros_foto = 0;
    // `--demo`: el recorrido de la entrega con piloto automatico (para grabar el video): vuelo,
    // acercamiento al Planeta Chapin, una vuelta en modo orbita y alejamiento.
    let mut demo: Option<demo::Demo> = None;
    if args.iter().any(|a| a == "--demo") {
        let (pos, rumbo) = demo::inicio(0.0);
        juego = Juego::new(pos, rumbo);
        juego.camara = Camara::detras_de(&juego.nave);
        escena = Escena::Espacio;
        fundido = 0.0;
        demo = Some(demo::Demo::new());
    }
    let mut cambio: Option<Escena> = None;
    // Al aterrizar en la Tierra: la entrada de la UVG (el lab del CIT o el salon de Koch).
    let mut puerta_uvg: Option<PuertaUvg> = None;
    // El pasaporte: lo guardado entre partidas. Las fotos de la interfaz no tocan la partida.
    let mut prog = if foto.is_some() { Progreso::en_memoria() } else { Progreso::cargar() };
    // P: el pasaporte abierto, y que planeta se ve en el.
    let mut pasaporte: Option<usize> = None;
    let mut vistas_vistas = [false; 4];
    // ESC: el menu de pausa (y los ajustes). El raton atrapado para volar, si se pidio.
    let mut pausa: Option<Pausa> = None;
    let mut raton_atrapado = false;
    // En Urano se apunta con la mira de la manguera: la flechita del sistema estorba.
    let mut cursor_oculto = false;
    // Para los encargos: el combo y los barrel rolls que ya se contaron.
    let (mut combo_contado, mut barrels_contados) = (0u32, 0usize);
    if let Some(a) = &audio {
        a.set_master_volume(prog.ajustes.volumen as f32 / 4.0);
    }
    // El juego arranca en el menu principal (las fotos de la interfaz, en la escena que se pidio).
    if foto.is_none() {
        escena = Escena::Menu(Menu::new(prog.notas_totales > 0 || !prog.logros.is_empty()));
    }
    let mut hud = true;
    // TAB: la info del pipeline y los controles.
    let mut info = false;

    // Arma la escena de una foto de la interfaz o de una toma de la grabacion: la escena de
    // `preparar` y lo que se abre encima (el menu, la pausa, el pasaporte, la tienda...).
    macro_rules! montar {
        ($t:expr, $resto:expr) => {{
            let (t, resto): (f32, &[String]) = ($t, $resto);
            let modos: Vec<String> = resto.iter().filter(|s| !matches!(s.as_str(), "mensaje" | "info" | "completa" | "menu" | "menu_lista" | "pausa" | "ajustes" | "pasaporte" | "pasaporte_espacio" | "tienda" | "tienda_estelas" | "tienda_personajes" | "uvg" | "casillero" | "pase" | "control_ps" | "control_xbox" | "bill" | "mochila" | "vuelo_demo") && !s.starts_with("personaje:") && !s.starts_with("ventana:")).cloned().collect();
            if resto.iter().any(|s| s == "completa") {
                unsafe { raylib::ffi::ToggleBorderlessWindowed() };
            }
            (juego, escena) = preparar(t, &modos);
            fundido = 0.0;
            if resto.iter().any(|s| s == "mensaje") {
                juego.decir("Eso era el Sol. Se recomienda no tocar el Sol. Ni con la nave, ni con la mirada.");
            }
            // `vuelo_demo`: el recorrido de `--demo`, desde el principio.
            if resto.iter().any(|s| s == "vuelo_demo") {
                let (pos, rumbo) = demo::inicio(0.0);
                juego = Juego::new(pos, rumbo);
                juego.camara = Camara::detras_de(&juego.nave);
                escena = Escena::Espacio;
                demo = Some(demo::Demo::new());
            }
            let r = resto;
            info = r.iter().any(|s| s == "info");
            // Para revisar cada pantalla de la interfaz: el menu principal (con o sin la lista), la pausa y el pasaporte.
            if r.iter().any(|s| s == "menu" || s == "menu_lista") {
                let mut m = Menu::new(true);
                if r.iter().any(|s| s == "menu_lista") {
                    m.iniciar();
                }
                escena = Escena::Menu(m);
            }
            if r.iter().any(|s| s == "pausa") {
                pausa = Some(Pausa::new());
            }
            if r.iter().any(|s| s == "ajustes") {
                pausa = Some(Pausa::ajustes());
            }
            if r.iter().any(|s| s == "pasaporte") {
                pasaporte = Some(3);
            }
            // Con algunos sellos puestos, para ver las estampas a color.
            if r.iter().any(|s| s == "pasaporte" || s == "pasaporte_espacio") {
                for i in [1, 2, 3, 5, 7, 11] {
                    prog.logros.insert(progreso::sello(i));
                }
            }
            if r.iter().any(|s| s == "pasaporte_espacio") {
                pasaporte = Some(progreso::CON_SELLO.len());
            }
            // `personaje:<id>`: con ese personaje escogido. Con partida de mentira (para ver el pase a medias).
            if let Some(id) = r.iter().find_map(|s| s.strip_prefix("personaje:")) {
                prog.personaje = id.to_string();
            }
            if r.iter().any(|s| s == "casillero" || s == "pase") {
                prog.notas_totales = 300;
                prog.encargos_cumplidos = 8;
                prog.cometas = 2;
                for id in ["sello_mercurio", "sello_venus", "sello_marte", "notas_10", "notas_100", "combo_10"] {
                    prog.logros.insert(id.into());
                }
                prog.pase_cobrado = prog.nivel_del_pase().0;
                let mut c = Casillero::new(&prog);
                let mut pase = Pase::new(&prog);
                let derecha = Entrada { derecha: true, ..Default::default() };
                c.actualizar(&derecha, &mut prog, 0.5);
                c.actualizar(&derecha, &mut prog, 0.5);
                pase.actualizar(&Entrada::default(), &mut prog, 0.5);
                submenu = Some(if r.iter().any(|s| s == "pase") { Submenu::Pase(pase) } else { Submenu::Casillero(c) });
            }
            // `mochila`: la mochila con algunas lunas y recuerdos.
            if r.iter().any(|s| s == "mochila") {
                prog.mochila = [3, 1, 0, 2];
                for id in ["tarot_lectura", "examen_5", "cita_boda", "jupiter_mancha", "hada_luka", "ciencia_5", "divorcio_3", "luna_oscar", "ritmo_termino", "uno_gana", "junta_si", "cometa"] {
                    prog.logros.insert(id.into());
                }
                submenu = Some(Submenu::Mochila(mochila::Mochila::new(true)));
            }
            // `bill`: Bill Nye contando un dato.
            if r.iter().any(|s| s == "bill") {
                bill_dice = Some((asteroides::DATOS[1].to_string(), 0.5));
            }
            // `uvg`: la entrada de la UVG abierta (con `aterrizar:tierra`, encima de la nave posada).
            if r.iter().any(|s| s == "uvg") {
                puerta_uvg = Some(PuertaUvg::new());
            }
            // La tienda (con notas de mentira para ver los precios que alcanzan y los que no).
            if r.iter().any(|s| s.starts_with("tienda")) {
                prog.notas = 130;
                let mut t = Tienda::new(&prog);
                let flecha = |c: bool| Entrada { abajo: !c, derecha: c, ..Default::default() };
                if r.iter().any(|s| s == "tienda_estelas" || s == "tienda_personajes") {
                    t.actualizar(&flecha(true), &mut prog, 0.5);
                }
                if r.iter().any(|s| s == "tienda_personajes") {
                    t.actualizar(&flecha(true), &mut prog, 0.5);
                }
                for _ in 0..3 {
                    t.actualizar(&flecha(false), &mut prog, 0.5);
                }
                submenu = Some(Submenu::Tienda(t));
            }
        }};
    }
    if let Some((_, t, resto)) = &foto {
        montar!(*t, resto);
    }
    while !rl.window_should_close() {
        // La grabacion: al empezar cada toma se arma su escena desde cero (con partida nueva).
        if let Some(g) = &grabar {
            if g.termino() {
                break;
            }
            if g.nueva {
                let toma = g.actual();
                (pausa, submenu, pasaporte, bill_dice, puerta_uvg, demo, cambio) = (None, None, None, None, None, None, None);
                hud = true;
                prog = Progreso::en_memoria();
                let modos: Vec<String> = toma.modos.iter().map(|s| s.to_string()).collect();
                montar!(toma.desde, &modos);
                // Cada maravilla con lo suyo (la linea Petrova solo con la Blip-A y en el vuelo).
                match toma.modos[0] {
                    "vuelo_demo" | "maravilla:blip" => {}
                    m if m.starts_with("maravilla:") => juego.maravillas.callar(false),
                    _ => juego.maravillas.callar(true),
                }
            }
        }
        // La foto de la interfaz: despues de unos cuadros, se guarda y se sale.
        if let (Some((archivo, _, _)), None) = (&foto, &grabar) {
            cuadros_foto += 1;
            if cuadros_foto == 149 {
                ui::registrar_textos(true);
            }
            if cuadros_foto > 150 {
                // La revision: los textos que se enciman o se salen de la ventana.
                for p in ui::revisar_textos(rl.get_screen_width(), rl.get_screen_height()) {
                    eprintln!("{p}");
                }
                foto_de_la_ventana(&rl, archivo);
                break;
            }
        }
        // Con el cuadro trabado (arrastrar la ventana) no se teletransporta nada.
        let dt_real = if grabar.is_some() { grabacion::DT } else { rl.get_frame_time().min(0.05) };
        let mut e = if cambio.is_some() || foto.is_some() { Entrada::default() } else { Entrada::leer(&rl) };
        // En las fotos, `control_ps` / `control_xbox` hacen como que hay ese control conectado.
        if let Some((_, _, r)) = &foto {
            for (modo, control) in [("control_ps", entrada::Control::PlayStation), ("control_xbox", entrada::Control::Xbox), ("control_switch", entrada::Control::Switch)] {
                if r.iter().any(|s| s == modo) {
                    e.hay_control = true;
                    e.control = control;
                }
            }
        }
        controles::usar(e.control);
        if let Some(g) = &grabar {
            e = entrada_grabada(g, &mut juego, &escena);
            // Si el sistema le cambia el tamano a la ventana (mosaicos, Stage Manager...), se
            // regresa al de diseno: el video no puede cambiar de tamano a medias.
            if (rl.get_screen_width(), rl.get_screen_height()) != (VENTANA_W, VENTANA_H) {
                rl.set_window_size(VENTANA_W, VENTANA_H);
            }
        }
        // ESC o start: la pausa. Tambien se pausa sola si la ventana pierde el foco.
        // En el menu principal no hay nada que pausar: ahi ESC es del menu (cierra los creditos).
        let en_el_menu = matches!(escena, Escena::Menu(_));
        // El menu del juego tiene pestanas (JUGAR o INICIO, PERSONAJES, PASE, TIENDA, AJUSTES):
        // con Q / E o L1 / R1 se pasa a la de al lado. En el menu principal tambien sirven desde
        // la lista (que es la pestana INICIO).
        let en_la_lista = matches!(&escena, Escena::Menu(m) if !m.creditos && m.iniciado()) && pasaporte.is_none();
        if let Some(k) = pestana_abierta(&pausa, &submenu, en_la_lista) {
            if foto.is_none() && (e.pestana_izq || e.pestana_der) {
                let n = casillero::PESTANAS.len();
                let nueva = if e.pestana_der { (k + 1) % n } else { (k + n - 1) % n };
                abrir_pestana(nueva, en_el_menu, &mut pausa, &mut submenu, &mut prog);
                tocar(Efecto::Carta);
                e = Entrada { hay_control: e.hay_control, control: e.control, ..Default::default() };
            }
        }
        // La tienda, el casillero o el pase (abiertos desde el menu o desde la pausa): la entrada
        // es suya y ESC los cierra. Si se abrieron desde la pausa, se regresa a la pausa.
        let submenu_abierto = submenu.is_some();
        if let Some(t) = &mut submenu {
            let cerrar = t.actualizar(&e, &mut prog, dt_real);
            for ev in t.eventos() {
                tocar(ev);
            }
            // Una luna de la mochila que se uso: su efecto en el vuelo.
            if let Submenu::Mochila(m) = t {
                if let Some(k) = m.usar.take() {
                    juego.usar_luna(k);
                }
            }
            if cerrar {
                submenu = None;
            }
        }
        if foto.is_none() && !submenu_abierto {
            if e.pausa && pasaporte.is_some() {
                // ESC cierra el pasaporte (si esta abierto) en vez de pausar.
                pasaporte = None;
            } else if e.pausa {
                // Con la pausa abierta, ESC regresa un paso (de los ajustes a la pausa, o cierra).
                match pausa.as_mut().map(|p| p.atras() == AccionPausa::Continuar) {
                    Some(true) => pausa = None,
                    Some(false) => {}
                    None if en_el_menu => {}
                    None => pausa = Some(Pausa::new()),
                }
            } else if pausa.is_none() && !en_el_menu && !rl.is_window_focused() {
                pausa = Some(Pausa::new());
            }
        }
        // Si un menu estaba abierto en este cuadro, la tecla es suya: si no, el Enter con que se
        // escoge "Continuar" tambien le llegaria al juego (y saltaria al hiperespacio).
        let menu_abierto = pausa.is_some() || submenu_abierto;
        let accion_pausa = if submenu_abierto { None } else { pausa.as_mut().map(|p| p.actualizar(&e, &mut prog.ajustes, dt_real)) };
        match accion_pausa {
            Some(AccionPausa::Continuar) => pausa = None,
            Some(AccionPausa::Ajustes) => {
                if let Some(a) = &audio {
                    a.set_master_volume(prog.ajustes.volumen as f32 / 4.0);
                }
                prog.guardar();
            }
            Some(AccionPausa::PantallaCompleta) => unsafe { raylib::ffi::ToggleBorderlessWindowed() },
            Some(AccionPausa::Hud) => hud = !hud,
            Some(AccionPausa::Menu) => {
                pausa = None;
                if !matches!(escena, Escena::Menu(_)) {
                    cambio = Some(Escena::Menu(Menu::new(true)));
                }
            }
            Some(AccionPausa::Salir) => {
                prog.guardar();
                break;
            }
            _ => {}
        }
        // Los ajustes que se aplican cada cuadro: volumenes, texto y vibracion.
        if let Some(s) = son {
            s.volumenes(prog.ajustes.musica as f32 / 4.0, prog.ajustes.efectos as f32 / 4.0);
        }
        sonido::velocidad_del_texto(prog.ajustes.letras_por_segundo());
        vibrar_activo.set(prog.ajustes.vibracion);
        pipeline::DIBUJADO.store(prog.ajustes.dibujado, std::sync::atomic::Ordering::Relaxed);
        if let Some(v) = &vibra {
            v.actualizar(dt_real);
        }
        // En pausa el juego no avanza ni escucha (el menu ya uso la entrada).
        let dt = if pausa.is_some() || submenu.is_some() { 0.0 } else { dt_real };
        if pausa.is_some() || submenu.is_some() || menu_abierto {
            e = Entrada { hay_control: e.hay_control, control: e.control, ..Default::default() };
        }
        // La calidad: el tamano del framebuffer (rapida = menos pixeles que pintar).
        let (rw, rh) = if grabar.is_some() {
            GRABAR_RENDER
        } else if prog.ajustes.calidad_rapida {
            (640, 360)
        } else {
            (RENDER_W, RENDER_H)
        };
        if fb.ancho != rw {
            fb = Framebuffer::new(rw, rh);
            post = Post::new(&fb);
            textura = rl.load_texture_from_image(&thread, &Image::gen_image_color(rw as i32, rh as i32, Color::BLACK)).expect("no se pudo crear la textura");
            textura.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        }
        // Volar con el raton: se atrapa el cursor y su movimiento gira la nave (clic: acelerar; derecho: turbo).
        let quiere_raton = prog.ajustes.raton && matches!(escena, Escena::Espacio | Escena::Vegetta(_)) && pausa.is_none() && pasaporte.is_none() && foto.is_none() && rl.is_window_focused();
        if quiere_raton != raton_atrapado {
            if quiere_raton {
                rl.disable_cursor();
            } else {
                rl.enable_cursor();
            }
            raton_atrapado = quiere_raton;
        }
        let ocultar = !raton_atrapado && matches!(&escena, Escena::Urano(u) if u.jugando()) && pausa.is_none() && foto.is_none();
        if ocultar != cursor_oculto {
            if ocultar {
                rl.hide_cursor();
            } else {
                rl.show_cursor();
            }
            cursor_oculto = ocultar;
        }
        // El raton suelto: donde esta y si se hizo clic (la compu del lab se usa tambien con el raton).
        if !raton_atrapado && foto.is_none() && cambio.is_none() {
            let m = rl.get_mouse_position();
            e.puntero = Some((m.x, m.y));
            e.clic = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
            e.raton_apretado = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
            e.picar |= e.raton_apretado;
        }
        if raton_atrapado {
            // El raton mueve el angulo directo (radianes por pixel), no la velocidad de giro.
            let m = rl.get_mouse_delta();
            let k = 0.0008 * prog.ajustes.sensibilidad as f32;
            e.raton = (-m.x * k, -m.y * k);
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                e.empuje = e.empuje.max(1.0);
            }
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
                e.empuje = e.empuje.max(1.0);
                e.turbo = true;
            }
        }
        // Cabeceo invertido (como en los simuladores: abajo sube la nariz).
        if prog.ajustes.invertir {
            e.cabecear = -e.cabecear;
            e.raton.1 = -e.raton.1;
        }
        // La maldicion del retrogrado (salazon en el tarot): volando, todo va al reves.
        if juego.maldicion > 0.0 && matches!(escena, Escena::Espacio) {
            e.girar = -e.girar;
            e.cabecear = -e.cabecear;
            e.raton = (-e.raton.0, -e.raton.1);
        }
        juego.estela = prog.estela.as_deref().and_then(|id| progreso::ESTELAS.iter().find(|x| x.0 == id).map(|x| x.0));
        if e.info {
            info = !info;
        }
        // F11 o Alt+Enter: pantalla completa (ventana sin bordes del tamano del monitor).
        if e.pantalla_completa {
            unsafe { raylib::ffi::ToggleBorderlessWindowed() };
        }
        if e.hud {
            hud = !hud;
        }
        // P: abre y cierra el pasaporte (solo volando).
        if e.pasaporte && submenu.is_none() && matches!(escena, Escena::Espacio | Escena::Menu(_)) {
            pasaporte = if pasaporte.is_some() { None } else { Some(0) };
        }

        match &mut escena {
            Escena::Intro(intro) => {
                juego.t += dt;
                let antes = intro.t;
                intro.actualizar(&mut juego.nave, e.aceptar, dt, juego.t);
                for ev in intro.eventos_desde(antes) {
                    tocar(ev);
                }
                juego.giro_turbinas += dt * 3.0;
                if intro.termino() {
                    // La camara del juego arranca justo donde termino la de la intro.
                    let (ojo, mira) = intro.camara(&juego.nave, juego.t);
                    juego.camara = Camara::new(ojo, mira);
                    // Ya va enganchada a la orbita de la Tierra (con el muelle): sin jalon al empezar.
                    juego.sincronia = 1.0;
                    juego.decir(format!("Soy GLADAS, la IA de la nave. Piloto a bordo: usted maneja, yo opino. {}: hiperespacio.  {}: aterrizar.", ui::boton(e.nombre("aceptar")), ui::boton(e.nombre("aterrizar"))));
                    escena = Escena::Espacio;
                }
            }
            // Con el pasaporte abierto, el juego se pausa y las flechas cambian de planeta.
            Escena::Espacio | Escena::Menu(_) if pasaporte.is_some() => {
                let n = progreso::CON_SELLO.len() + 1;
                if let Some(k) = pasaporte.as_mut() {
                    if e.izquierda || e.arriba {
                        *k = (*k + n - 1) % n;
                    }
                    if e.derecha || e.abajo {
                        *k = (*k + 1) % n;
                    }
                }
            }
            Escena::Espacio => {
                // La demo maneja sola; con cualquier tecla de vuelo se toma el control.
                if demo.is_some() && (e.aceptar || e.aterrizar || e.barrel || e.empuje.abs() > 0.2 || e.girar.abs() > 0.2 || e.cabecear.abs() > 0.2) {
                    demo = None;
                    juego.decir("Fin de la demo: ahora maneja usted.");
                }
                let mut fin_demo = false;
                if let Some(dm) = &mut demo {
                    let auto = dm.entrada(&juego.nave, juego.orbita.is_some(), juego.t, dt);
                    e = Entrada { hay_control: e.hay_control, control: e.control, info: e.info, hud: e.hud, ..auto };
                    fin_demo = dm.fase == demo::Fase::Fin;
                }
                if fin_demo {
                    demo = None;
                }
                // En la entrada de la UVG se escoge a donde ir; la nave (posada) solo escucha para despegar.
                if let Some(p) = &mut puerta_uvg {
                    let destino = p.actualizar(&e, dt);
                    for ev in p.eventos.drain(..) {
                        tocar(ev);
                    }
                    let semilla = semilla_del_reloj() ^ (juego.t * 1000.0) as u32;
                    match destino {
                        Some(DestinoUvg::Lab) => cambio = Some(Escena::Entrega(Box::new(Entrega::new(semilla)))),
                        Some(DestinoUvg::Salon) => cambio = Some(Escena::Aula(Box::new(Aula::new(semilla)))),
                        _ => {}
                    }
                    if destino.is_some() {
                        puerta_uvg = None;
                    }
                    e = Entrada { aterrizar: destino == Some(DestinoUvg::Despegar), hay_control: e.hay_control, control: e.control, ..Default::default() };
                }
                // 1 2 3: como se ensamblan las primitivas. 4 a 7: ver por dentro el
                // z-buffer, las normales, las UV y el stencil. select: pasa al siguiente.
                const VISTAS: [(ModoDibujo, Vista); 7] = [
                    (ModoDibujo::Relleno, Vista::Color),
                    (ModoDibujo::Alambre, Vista::Color),
                    (ModoDibujo::Puntos, Vista::Color),
                    (ModoDibujo::Relleno, Vista::Profundidad),
                    (ModoDibujo::Relleno, Vista::Normales),
                    (ModoDibujo::Relleno, Vista::Uv),
                    (ModoDibujo::Relleno, Vista::Stencil),
                ];
                let actual = VISTAS.iter().position(|&(m, v)| m == juego.opciones.modo && v == juego.opciones.vista).unwrap_or(0);
                let nueva = e.modo.or(e.siguiente_modo.then_some((actual + 1) % VISTAS.len()));
                if let Some(k) = nueva {
                    (juego.opciones.modo, juego.opciones.vista) = VISTAS[k];
                }
                if e.culling {
                    juego.opciones.culling = !juego.opciones.culling;
                }
                // V: el descarte de objetos fuera de la vista, prendido o apagado (para comparar los
                // triangulos que procesa el pipeline en el panel de TAB).
                if e.descarte {
                    let antes = pipeline::DESCARTE.load(std::sync::atomic::Ordering::Relaxed);
                    pipeline::DESCARTE.store(!antes, std::sync::atomic::Ordering::Relaxed);
                    juego.decir(if antes {
                        "Descarte de objetos APAGADO: todo entra al pipeline aunque no se vea. Compare los triángulos en el panel de TAB."
                    } else {
                        "Descarte de objetos prendido: lo que queda fuera de la vista ni entra al vertex shader."
                    });
                }
                // F5 a F8: el planeta de junto (cualquiera) en vivo: la semilla del ruido y lo que sube
                // el relieve; se vuelve a armar ahi mismo. F9 y F10: la intensidad de los shaders.
                if let Some(k) = e.ajuste {
                    let (cerca, distancia) = mas_cercano(juego.nave.pos, juego.t);
                    let nombre = CUERPOS[cerca].nombre;
                    if k >= 8 {
                        // F3 y F4: la velocidad de las orbitas (y de los giros), de todo el sistema.
                        juego.rapidez_orbitas = if k == 8 { (juego.rapidez_orbitas * 0.5).max(0.125) } else { (juego.rapidez_orbitas * 2.0).min(16.0) };
                        juego.decir(format!("Velocidad de las órbitas: ×{}. Depende del tiempo, no de los cuadros.", juego.rapidez_orbitas));
                    } else if k >= 6 {
                        // F1 y F2: el radio del planeta de junto (el dibujo y las colisiones lo siguen).
                        if distancia > CUERPOS[cerca].radio_visual() * 4.0 + 40.0 {
                            juego.decir("Para cambiarle el radio a un planeta, hay que acercarse a uno.");
                        } else {
                            let f = (escena::factor_radio(cerca) + if k == 6 { -0.1 } else { 0.1 }).clamp(0.4, 2.5);
                            escena::poner_factor_radio(cerca, f);
                            juego.decir(format!("{nombre} en vivo: radio ×{f:.1} ({:.2} unidades).", CUERPOS[cerca].radio_visual()));
                        }
                    } else if k >= 4 {
                        juego.intensidad = if k == 4 { (juego.intensidad - 0.25).max(0.0) } else { (juego.intensidad + 0.25).min(3.0) };
                        juego.decir(format!("Intensidad de los shaders: {:.2} (0 los apaga: sin olas, sin relieve en los bloques).", juego.intensidad));
                    } else if distancia > CUERPOS[cerca].radio_visual() * 4.0 + 40.0 {
                        juego.decir("Para cambiarle la semilla o el relieve a un planeta, hay que acercarse a uno.");
                    } else if cerca == CHAPIN {
                        let mut p = juego.params_chapin;
                        match k {
                            0 => p.semilla = p.semilla.wrapping_sub(1),
                            1 => p.semilla = p.semilla.wrapping_add(1),
                            2 => p.relieve = (p.relieve - 0.03).max(0.0),
                            _ => p.relieve = (p.relieve + 0.03).min(0.4),
                        }
                        juego.params_chapin = p;
                        *rec.chapin.borrow_mut() = chapin::Chapin::new(p);
                        rec.rearmar_chapin();
                        let objetos = rec.chapin.borrow().instancias.len();
                        juego.decir(format!("{nombre} en vivo: semilla {}, relieve {:.2}, {objetos} objetos{}.", p.semilla, p.relieve, if p.relieve == 0.0 { " (esfera lisa: la generación sin la deformación)" } else { "" }));
                    } else {
                        let mut a = juego.ajustes[cerca];
                        match k {
                            0 => a.semilla = a.semilla.wrapping_sub(1),
                            1 => a.semilla = a.semilla.wrapping_add(1),
                            2 => a.relieve = (a.relieve - 0.25).max(0.0),
                            _ => a.relieve = (a.relieve + 0.25).min(3.0),
                        }
                        juego.ajustes[cerca] = a;
                        rec.rearmar(cerca, a);
                        juego.decir(format!("{nombre} en vivo: semilla {:+}, relieve ×{:.2}{}.", a.semilla as i32, a.relieve, if a.relieve == 0.0 { " (esfera lisa: la generación sin la deformación)" } else { "" }));
                    }
                }
                // E junto al Mirador: a descansar (la nave se queda quieta y con el turbo lleno).
                let junto_al_mirador = (juego.nave.pos - mirador_en(juego.t)).length() < DISTANCIA_MIRADOR && juego.aterrizaje.is_none() && !juego.salto.activo();
                if junto_al_mirador && e.aterrizar {
                    e.aterrizar = false;
                    juego.nave.velocidad = 0.0;
                    juego.nave.energia = 1.0;
                    cambio = Some(Escena::Mirador(Mirador::new(semilla_del_reloj())));
                    tocar(Efecto::Aterrizaje);
                }
                // Solo se pone un cambio cuando se pide: si se asignara `None` en
                // cada cuadro, el cuadro siguiente al aterrizaje cancelaria el fundido.
                let camara_antes = (juego.camara.ojo, juego.camara.mira);
                let aterrizo = juego.actualizar(&e, dt);
                // La demo: en la vuelta, la camara de cine (con el planeta en cuadro).
                if let Some(dm) = &demo {
                    dm.mover_camara(&mut juego.camara, camara_antes, &juego.nave, juego.t, dt);
                }
                match aterrizo {
                    // En la Tierra se escoge a donde: el lab (la entrega) o el salon (el examen de Koch).
                    Some(TIERRA) => puerta_uvg = Some(PuertaUvg::new()),
                    Some(URANO) => cambio = Some(Escena::Urano(Box::new(Urano::new(semilla_del_reloj() ^ (juego.t * 1000.0) as u32)))),
                    Some(VEGETTA) => cambio = Some(Escena::Vegetta(Box::new(Vegetta::new(semilla_del_reloj() ^ (juego.t * 1000.0) as u32, rec.atlas_bloques.clone())))),
                    Some(SATURNO) => cambio = Some(Escena::Concierto(Concierto::new())),
                    Some(PLUTON) => cambio = Some(Escena::Junta(Junta::new())),
                    Some(MARTE) => cambio = Some(Escena::Cita(Cita::new())),
                    Some(NEPTUNO) => {
                        let mut n = Neptuno::new((juego.t * 1000.0) as u32 ^ semilla_del_reloj());
                        // Si Canela o Pluton ya vinieron alguna vez, ya no llegan siempre.
                        n.ya_vinieron = (prog.tiene("uno_canela"), prog.tiene("uno_pluton"));
                        n.reescoger_visita();
                        cambio = Some(Escena::Neptuno(n));
                    }
                    Some(HADA) => cambio = Some(Escena::Fuente(Fuente::new())),
                    Some(LUNA) => cambio = Some(Escena::Luna(Luna::new(semilla_del_reloj() ^ (juego.t * 1000.0) as u32))),
                    Some(CHAPIN) => cambio = Some(Escena::Camioneta(Box::new(Camioneta::new(semilla_del_reloj() ^ (juego.t * 1000.0) as u32)))),
                    Some(VENUS) => cambio = Some(Escena::Divorcio(Divorcio::new(semilla_del_reloj() ^ (juego.t * 1000.0) as u32))),
                    Some(JUPITER) => cambio = Some(Escena::Jupiter(Jupiter::new(semilla_del_reloj(), prog.record_jupiter))),
                    Some(MERCURIO) => cambio = Some(Escena::Mercurio(Mercurio::new(semilla_del_reloj() ^ (juego.t * 1000.0) as u32))),
                    _ => {}
                }
                if cambio.is_some() {
                    tocar(Efecto::Aterrizaje);
                }
                for ev in juego.eventos.drain(..) {
                    tocar(ev);
                }
                // El turbo retumba suavecito mientras se usa.
                if let (true, Some(v)) = (vibrar_activo.get() && juego.turbo_encendido() > 0.0 && juego.nave.velocidad > 30.0, &vibra) {
                    v.vibrar(0.12, 0.22, 120);
                }
            }
            Escena::Vegetta(v) => {
                juego.t += dt;
                if let AccionVegetta::Despegar = v.actualizar(&e, dt) {
                    // Los diamantes que se trajeron: dos notas cada uno.
                    prog.sumar_notas(v.diamantes() as u32 * 2);
                    juego.despegar_de(VEGETTA, v.despedida());
                    cambio = Some(Escena::Espacio);
                }
                for ev in v.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Urano(u) => {
                juego.t += dt;
                // Donde va la imagen en la ventana (el raton apunta sobre ella).
                let r = rect_imagen(rl.get_screen_width(), rl.get_screen_height());
                u.imagen = (r.x, r.y, r.width, r.height);
                if let AccionUrano::Despegar = u.actualizar(&e, dt) {
                    juego.despegar_de(URANO, u.despedida());
                    cambio = Some(Escena::Espacio);
                }
                for ev in u.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Aula(a) => {
                // El sistema solar sigue girando mientras se esta en el salon.
                juego.t += dt;
                let salir = a.actualizar(&e, dt);
                for ev in a.eventos.drain(..) {
                    tocar(ev);
                }
                if salir {
                    juego.despegar_de(TIERRA, a.despedida());
                    cambio = Some(Escena::Espacio);
                }
            }
            Escena::Entrega(en) => {
                // El sistema solar sigue girando mientras se esta en el lab.
                juego.t += dt;
                let accion = en.actualizar(&e, dt);
                for ev in en.eventos.drain(..) {
                    tocar(ev);
                }
                if let AccionEntrega::Despegar = accion {
                    juego.despegar_de(TIERRA, en.despedida());
                    cambio = Some(Escena::Espacio);
                }
            }
            Escena::Concierto(c) => {
                juego.t += dt;
                match c.actualizar(&e, dt) {
                    Some(AccionConcierto::Despegar) => {
                        juego.despegar_de(SATURNO, "Despegue de Saturno. Fin del concierto. Nadie pidió otra, pero igual volvemos.".into());
                        cambio = Some(Escena::Espacio);
                    }
                    Some(AccionConcierto::Jugar) => cambio = Some(Escena::Ritmo(Ritmo::new())),
                    None => {}
                }
            }
            Escena::Junta(j) => {
                juego.t += dt;
                if let AccionJunta::Despegar = j.actualizar(&e, dt) {
                    let despedida = if j.relleno() {
                        "Despegue de Plutón. Ya es planeta otra vez. Se puso a llorar de la emoción. Yo no. Bueno, un poco."
                    } else {
                        "Despegue de Plutón. Nos despide desde su cuarto. Con la puerta cerrada."
                    };
                    juego.despegar_de(PLUTON, despedida.into());
                    cambio = Some(Escena::Espacio);
                }
                for ev in j.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Cita(c) => {
                juego.t += dt;
                if let AccionCita::Despegar = c.actualizar(&e, dt) {
                    let despedida = match (c.terminada(), c.final_()) {
                        (true, Final::Boda) => "Despegue de Marte. Dejamos a los recién casados. Pidieron no molestar. Por 200 años.",
                        (true, Final::Exito) => "Despegue de Marte. Curiosity ya tiene pareja. Usted no. Solo digo.",
                        (true, Final::Drama(_)) => "Despegue de Marte. Huimos del drama. Curiosity se quedó. No puede huir: va a 0.14 km/h.",
                        (true, Final::Amigos) => "Despegue de Marte. Curiosity tiene una amiga nueva. Algo es algo.",
                        (true, Final::Solo) => "Despegue de Marte. Curiosity nos despide con el taladro. Creo que es de cariño.",
                        _ => "Despegue de Marte. Nos fuimos a media cita. Curiosity sigue esperando. Como siempre.",
                    };
                    juego.despegar_de(MARTE, despedida.into());
                    cambio = Some(Escena::Espacio);
                }
                for ev in c.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Neptuno(n) => {
                juego.t += dt;
                if let AccionNeptuno::Despegar = n.actualizar(&e, dt) {
                    let despedida = match n.ganador {
                        Some(neptuno::MIKU) => "Despegue de Neptuno. Le ganamos a tres leyendas. Con cartas. Eso cuenta. Creo.",
                        Some(_) => "Despegue de Neptuno. Perdimos con dignidad. Bueno, perdimos.",
                        None => "Despegue de Neptuno. Nos fuimos a media partida. Cloud dice que no le interesa. Mentira.",
                    };
                    juego.despegar_de(NEPTUNO, despedida.into());
                    cambio = Some(Escena::Espacio);
                }
                for ev in n.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Fuente(f) => {
                juego.t += dt;
                if let AccionFuente::Despegar = f.actualizar(&e, dt) {
                    let despedida = if f.hada_afuera() {
                        "Despegue del Planeta Hada. Llevamos un corazón nuevo, brillantina en toda la nave y un dueto pendiente con Luka."
                    } else {
                        "Despegue del Planeta Hada. Nos fuimos sin tocar la canción. Luka no salió. Su spin-off se canceló antes de empezar."
                    };
                    juego.despegar_de(HADA, despedida.into());
                    cambio = Some(Escena::Espacio);
                }
                for ev in f.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Menu(m) => {
                // Atras, el sistema solar sigue girando y la camara le da vueltas a la Tierra.
                juego.t += dt;
                juego.reloj += dt;
                let tierra = CUERPOS[TIERRA].posicion(juego.t);
                let a = m.reloj * 0.05;
                juego.nave.pos = intro::nave_parada(juego.t);
                juego.camara = Camara::new(tierra + Vec3::new(30.0 * a.cos(), 9.0, 30.0 * a.sin()), tierra + Vec3::new(0.0, 2.0, 0.0));
                // Con la tienda (o el casillero, o el pase) abierta, la entrada es suya (ESC la cierra).
                let accion = if submenu_abierto { AccionMenu::Nada } else { m.actualizar(&e, dt) };
                for ev in m.eventos.drain(..) {
                    tocar(ev);
                }
                match accion {
                    AccionMenu::Escoger(Opcion::Continuar) => {
                        // De vuelta al espacio: flotando en el centro del muelle, ya arrancada (sin la intro).
                        let t = juego.t;
                        juego = Juego::new(intro::nave_en_el_muelle(t), 0.0);
                        juego.t = t;
                        // Enganchada a la orbita de la Tierra desde el principio: si no, el muelle se
                        // adelanta y la nave lo alcanza de un jalon.
                        juego.sincronia = 1.0;
                        juego.decir("Bienvenida de vuelta. Sus notas, sus sellos y sus encargos siguen aquí. Yo también. No me fui a ningún lado.");
                        cambio = Some(Escena::Espacio);
                    }
                    AccionMenu::Escoger(Opcion::Nueva) => {
                        juego = Juego::new(intro::nave_parada(0.0), 0.0);
                        cambio = Some(Escena::Intro(Intro::new()));
                    }
                    AccionMenu::Escoger(Opcion::Historia) => {
                        let mut h = Historia::new();
                        h.desde_menu = true;
                        cambio = Some(Escena::Historia(h));
                    }
                    AccionMenu::Escoger(Opcion::Pasaporte) => pasaporte = Some(0),
                    AccionMenu::Escoger(Opcion::Salir) => {
                        prog.guardar();
                        break;
                    }
                    _ => {}
                }
            }
            Escena::Mirador(m) => {
                match m.actualizar(&e, dt, &mut prog) {
                    AccionMirador::Despegar => {
                        // La nave sale del Mirador viendo hacia la Tierra, quietecita.
                        let p = mirador_en(juego.t);
                        let a_tierra = (CUERPOS[TIERRA].posicion(juego.t) - p).normalize();
                        juego.nave.pos = p + a_tierra * 6.0 + Vec3::new(0.0, 1.0, 0.0);
                        (juego.nave.yaw, juego.nave.pitch) = salto::rumbo_hacia(a_tierra);
                        juego.nave.velocidad = 0.0;
                        juego.camara = Camara::detras_de(&juego.nave);
                        juego.decir("Descanso terminado. Turbo al cien. A volar, que las notas no se recogen solas.");
                        cambio = Some(Escena::Espacio);
                    }
                    AccionMirador::Historia => cambio = Some(Escena::Historia(Historia::new())),
                    AccionMirador::Nada => {}
                }
                for ev in m.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Historia(h) => {
                if let AccionHistoria::Terminar = h.actualizar(&e, dt) {
                    if h.completa {
                        prog.desbloquear("historia");
                    }
                    if cambio.is_none() {
                        cambio = Some(if h.desde_menu { Escena::Menu(Menu::new(true)) } else { Escena::Mirador(Mirador::new(semilla_del_reloj())) });
                    }
                }
                for ev in h.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Camioneta(c) => {
                juego.t += dt;
                if let AccionCamioneta::Despegar = c.actualizar(&e, dt) {
                    juego.despegar_de(CHAPIN, c.despedida());
                    cambio = Some(Escena::Espacio);
                }
                for ev in c.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Luna(l) => {
                juego.t += dt;
                if let AccionLuna::Despegar = l.actualizar(&e, dt) {
                    let despedida = if !l.terminada() {
                        "Despegue de la Luna. Nos fuimos a media toma. El director dice que nunca vuelve a trabajar con idols."
                    } else if l.aciertos() >= 4 {
                        "Despegue de la Luna. La toma quedó perfecta. Repito: la llegada de 1969 fue real. Esto era otra película."
                    } else {
                        "Despegue de la Luna. Dejamos un ventilador, una taza y al profe Koch en la Luna. Que los encuentre la NASA."
                    };
                    juego.despegar_de(LUNA, despedida.into());
                    cambio = Some(Escena::Espacio);
                }
                for ev in l.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Jupiter(j) => {
                juego.t += dt;
                if let AccionJupiter::Despegar = j.actualizar(&e, dt) {
                    let despedida = if j.puntos == 0 {
                        "Despegue de Júpiter. Ni un asteroide. Bill Nye dice que la ciencia es paciencia. Yo digo que es puntería."
                    } else if j.jefes > 0 {
                        "Despegue de Júpiter. Calmamos la Gran Mancha Roja. Bueno, por un rato. Lleva siglos así."
                    } else {
                        "Despegue de Júpiter. Órbita un poco más limpia. Júpiter no dio las gracias. Es un gigante gaseoso, no tiene modales."
                    };
                    juego.despegar_de(JUPITER, despedida.into());
                    cambio = Some(Escena::Espacio);
                }
                // Al terminar una partida: las notas, el record y las lunas para la mochila.
                if j.terminado() && !j.cobrado {
                    j.cobrado = true;
                    prog.sumar_notas(j.notas_ganadas());
                    prog.record_jupiter = prog.record_jupiter.max(j.puntos);
                    for k in 0..4 {
                        prog.mochila[k] += j.recogidas[k];
                    }
                    prog.desbloquear("jupiter_juega");
                    if j.jefes > 0 {
                        prog.desbloquear("jupiter_mancha");
                    }
                    if j.puntos >= 5000 {
                        prog.desbloquear("jupiter_5000");
                    }
                    if j.recogidas.iter().all(|&n| n > 0) {
                        prog.desbloquear("jupiter_lunas");
                    }
                    prog.guardar();
                }
                // Una roca dorada: Bill cuenta un dato del universo.
                if j.pide_dato {
                    let dato = prog.siguiente_dato();
                    j.decir(dato);
                }
                for ev in j.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Divorcio(d) => {
                juego.t += dt;
                match d.actualizar(&e, dt) {
                    AccionDivorcio::Despegar => {
                        let despedida = if !d.terminado() {
                            "Despegue de Venus. Nos fuimos a media audiencia. El ganso nos va a mandar una multa. Por honk."
                        } else {
                            match d.cuantos(divorcio::Destino::Saturno) {
                                3 => "Despegue de Venus. Saturno se fue divorciado y con sus tres anillos. Se ve radiante. Literalmente: es hielo.",
                                0 => "Despegue de Venus. Saturno se fue divorciado y sin anillos. Parece Júpiter beige. No se lo diga.",
                                _ => "Despegue de Venus. Saturno se fue divorciado, con algunos anillos menos. Dice que así se ve más moderno.",
                            }
                        };
                        juego.despegar_de(VENUS, despedida.into());
                        cambio = Some(Escena::Espacio);
                    }
                    AccionDivorcio::OtroJuicio => *d = d.otra_vez(semilla_del_reloj()),
                    AccionDivorcio::Nada => {}
                }
                for ev in d.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Mercurio(m) => {
                juego.t += dt;
                // Madame sabe cuantas notas trae uno (para la limpia) y que cartas ya se vieron.
                m.notas = prog.notas;
                m.descubiertas = prog.cartas_tarot;
                let accion = m.actualizar(&e, dt);
                let vistas = prog.cartas_tarot | m.vistas();
                if vistas != prog.cartas_tarot {
                    prog.cartas_tarot = vistas;
                    if vistas.count_ones() as usize >= mercurio::BARAJA.len() {
                        prog.desbloquear("tarot_baraja");
                    }
                    prog.guardar();
                }
                // Lo que dejo el veredicto: la luna de regalo, la propina y el cobro de la limpia.
                if let Some(k) = m.regalo.take() {
                    prog.mochila[k] += 1;
                    prog.guardar();
                }
                if m.propina > 0 {
                    prog.sumar_notas(std::mem::take(&mut m.propina));
                    prog.guardar();
                }
                if m.cobrar > 0 {
                    prog.notas = prog.notas.saturating_sub(std::mem::take(&mut m.cobrar));
                    prog.guardar();
                }
                if let AccionMercurio::Despegar = accion {
                    let despedida = if m.maldicion > 0.0 {
                        format!("Despegue de Mercurio. Traemos la maldición del retrógrado: por {:.0} segundos, los controles van al revés. Gracias, Madame.", m.maldicion)
                    } else {
                        match m.leidas() {
                            0 => "Despegue de Mercurio. Nos fuimos sin leer las cartas. Madame dice que ya sabía. Obvio: es adivina.",
                            1 | 2 => "Despegue de Mercurio. Lectura a medias. Si algo sale mal, ya sabemos a quién culpar.",
                            _ => "Despegue de Mercurio. Mercurio sigue en retrógrado. Yo no le escribiría a nadie hasta Saturno.",
                        }
                        .to_string()
                    };
                    juego.maldicion = m.maldicion;
                    juego.despegar_de(MERCURIO, despedida);
                    cambio = Some(Escena::Espacio);
                }
                for ev in m.eventos.drain(..) {
                    tocar(ev);
                }
            }
            Escena::Ritmo(r) => {
                juego.t += dt;
                match r.actualizar(&e, dt) {
                    Accion::OtraVez => *r = r.otra_vez(),
                    Accion::Despegar => {
                        juego.despegar_de(SATURNO, "Despegue de Saturno. Los bichitos-nota se despiden. Algunos con rencor.".into());
                        cambio = Some(Escena::Espacio);
                    }
                    Accion::Nada => {}
                }
                // La cancion arranca desde cero justo cuando empieza la partida (las notas van con ella).
                if std::mem::take(&mut r.empezar_cancion) {
                    if let Some(s) = son {
                        s.reiniciar(Pista::Cancion);
                    }
                }
                for ev in r.eventos.drain(..) {
                    tocar(ev);
                }
            }
        }
        // El pasaporte: sellos al posarse, finales de cada escena, notas y logros de volar.
        let antes = (prog.logros.len(), prog.encargos_cumplidos);
        for i in juego.visitas.drain(..) {
            prog.desbloquear(&sello(i));
            prog.hecho(Hecho::Aterrizo(i));
        }
        for id in logros_de_escena(&escena) {
            prog.desbloquear(id);
        }
        if let Escena::Espacio = escena {
            let recogidas = juego.coleccion.recogidas;
            if recogidas > 0 {
                prog.sumar_notas(recogidas);
                juego.coleccion.recogidas = 0;
            }
            if juego.perdidas > 0 {
                prog.perder_notas(std::mem::take(&mut juego.perdidas));
                if let Some(v) = &vibra {
                    v.vibrar(1.0, 0.6, 450);
                }
            }
            // Un asteroide dorado: Bill Nye cuenta un dato.
            if std::mem::take(&mut juego.dato_bill) {
                bill_dice = Some((prog.siguiente_dato().to_string(), 0.0));
            }
            if std::mem::take(&mut juego.coleccion.cometa_atrapado) {
                prog.sumar_cometa();
            }
            let combo = juego.coleccion.combo;
            if combo > combo_contado {
                prog.hecho(Hecho::Combo(combo));
            }
            combo_contado = combo;
            while barrels_contados < juego.barrels {
                barrels_contados += 1;
                prog.hecho(Hecho::Barrel);
            }
            for (meta, id) in [(10, "combo_10"), (25, "combo_25")] {
                if combo >= meta {
                    prog.desbloquear(id);
                }
            }
            if juego.barrels >= 10 {
                prog.desbloquear("barrel_10");
            }
            if juego.salto.activo() {
                prog.desbloquear("hiperespacio");
            }
            for id in std::mem::take(&mut juego.logros_nuevos) {
                // Rocky regala notas la primera vez (dice que la musica es ciencia).
                if prog.desbloquear(id) && id == "rocky" {
                    prog.sumar_notas(40);
                }
            }
            if let (0, d) = mas_cercano(juego.nave.pos, juego.t) {
                if d < 25.0 {
                    prog.desbloquear("sol");
                }
            }
            for (k, v) in [Vista::Profundidad, Vista::Normales, Vista::Uv, Vista::Stencil].iter().enumerate() {
                vistas_vistas[k] |= juego.opciones.vista == *v;
            }
            if vistas_vistas.iter().all(|v| *v) {
                prog.desbloquear("pipeline");
            }
        }
        if (prog.logros.len(), prog.encargos_cumplidos) != antes {
            tocar(Efecto::Sello);
        }
        prog.actualizar(dt);
        if cambio.is_some() {
            fundido += dt / FUNDIDO;
            if fundido >= 1.0 {
                fundido = 1.0;
                // Al despegar de un minijuego, su pago.
                if matches!(cambio, Some(Escena::Espacio)) {
                    if let Some((n, motivo)) = pago_de_escena(&escena) {
                        prog.cobrar(n, &motivo);
                    }
                }
                escena = cambio.take().unwrap();
                puerta_uvg = None;
                // El concierto arranca la cancion desde el principio (el minijuego
                // la arranca el solo, al elegir la dificultad).
                if let (Some(s), Escena::Concierto(_)) = (son, &escena) {
                    s.reiniciar(Pista::Cancion);
                }
            }
        } else {
            fundido = (fundido - dt / FUNDIDO).max(0.0);
        }

        // La musica de cada escena, con fundidos entre una y otra.
        if let Some(s) = son {
            let (pista, bucle) = match &escena {
                Escena::Intro(_) => (Some(Pista::Cancion), false),
                // Mientras se elige la dificultad suena la musica tranquila; la cancion arranca al jugar.
                Escena::Ritmo(r) if !r.jugando() => (Some(Pista::Espacio), true),
                Escena::Ritmo(_) => (Some(Pista::Cancion), false),
                Escena::Concierto(_) => (Some(Pista::Cancion), true),
                Escena::Espacio => (Some(Pista::Espacio), true),
                Escena::Entrega(_) | Escena::Aula(_) => (Some(Pista::Lofi), true),
                Escena::Urano(_) => (Some(Pista::Arcade), true),
                Escena::Vegetta(_) => (Some(Pista::Cancion), true),
                // En la junta hay silencio tenso... hasta que Pluton vuelve a ser planeta.
                Escena::Junta(j) => (j.relleno().then_some(Pista::Cancion), true),
                // La cita va con la musica tranquila; si sale bien, con la cancion.
                Escena::Cita(c) => (Some(if c.terminada() && matches!(c.final_(), Final::Exito | Final::Boda) { Pista::Cancion } else { Pista::Espacio }), true),
                // Las cartas van con la musica tranquila; si gana Miku, con la cancion.
                Escena::Neptuno(n) => (Some(if n.miku_baila() { Pista::Cancion } else { Pista::Espacio }), true),
                // El arpa de la fuente, siempre.
                Escena::Fuente(_) => (Some(Pista::Fuente), true),
                // El tarot, y al reves mientras Mercurio rebobina.
                Escena::Mercurio(m) => (Some(if m.rebobinando() > 0.0 { Pista::TarotReves } else { Pista::Tarot }), true),
                // El juicio, tenso; en el veredicto ya suena la fiesta.
                Escena::Luna(_) => (Some(Pista::Espacio), true),
                Escena::Camioneta(_) => (Some(Pista::Marimba), true),
                // En el Mirador, el arpa tranquila (como junto a una fogata); la historia, con la musica del espacio.
                Escena::Mirador(_) => (Some(Pista::Fuente), true),
                Escena::Historia(_) => (Some(Pista::Espacio), true),
                Escena::Menu(_) => (Some(Pista::Cancion), true),
                Escena::Divorcio(d) => (Some(if d.de_fiesta() { Pista::Cancion } else { Pista::Juicio }), true),
                Escena::Jupiter(_) => (Some(Pista::Arcade), true),
            };
            s.musica(pista, bucle, dt);
            // El lo-fi de la entrega se acelera en el ultimo minuto.
            s.ritmo(Pista::Lofi, if let Escena::Entrega(en) = &escena { en.velocidad_musica() } else { 1.0 });
        }

        let empezo = std::time::Instant::now();
        rec.skin.set(tienda::indice_skin(&prog.skin));
        // El personaje escogido: su modelo en todas las escenas y su nombre en los textos.
        let elegido = personajes::indice(&prog.personaje);
        rec.personaje.set(elegido);
        let ficha = &personajes::PERSONAJES[elegido];
        ui::poner_protagonista(ficha.id, ficha.nombre, ficha.ella);
        sonido::poner_voz_del_protagonista(personajes::voz(ficha.id), personajes::tono(ficha.id));
        // Los premios de los niveles nuevos del pase (con su aviso).
        if foto.is_none() {
            prog.cobrar_pase();
        }
        let st = dibujar(&mut fb, &mut post, &rec, &juego, &escena);
        let ms = empezo.elapsed().as_secs_f32() * 1000.0;
        textura.update_texture(fb.bytes()).expect("no se pudo actualizar la textura");
        // Miku en grande, solo si hay un menu que la ensena.
        let con_miku = pausa.is_some() || matches!(&escena, Escena::Menu(m) if !m.creditos && pasaporte.is_none() && submenu.is_none());
        reloj_menu += dt_real;
        if let Some(t) = &submenu {
            let (muestra, reloj) = t.muestra(&prog);
            dibujar_vitrina(&mut fb_vitrina, &rec, muestra, reloj);
            textura_vitrina.update_texture(fb_vitrina.bytes()).expect("no se pudo actualizar la vitrina");
        } else if let Escena::Entrega(en) = &escena {
            // El "proyecto 3" en el monitor del lab, con su bug (lo dibuja el mismo rasterizador).
            if let Some(bug) = en.vista_bug() {
                dibujar_bug(&mut fb_vitrina, &rec, bug, en.total);
                textura_vitrina.update_texture(fb_vitrina.bytes()).expect("no se pudo actualizar la vitrina");
            }
        }
        if con_miku {
            dibujar_miku_menu(&mut fb_miku, &rec, reloj_menu);
            textura_miku.update_texture(fb_miku.bytes()).expect("no se pudo actualizar a Miku");
        }
        if gladas_hablaba {
            dibujar_retrato(&mut fb_retrato, &rec, juego.t, true);
            textura_retrato.update_texture(fb_retrato.bytes()).expect("no se pudo actualizar el retrato");
        }

        let (ancho, alto) = (rl.get_screen_width(), rl.get_screen_height());
        let (pixeles_w, pixeles_h) = (rl.get_render_width(), rl.get_render_height());
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        let imagen = rect_imagen(ancho, alto);
        d.draw_texture_pro(
            &textura,
            Rectangle::new(0.0, 0.0, fb.ancho as f32, fb.alto as f32),
            imagen,
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        // La caja de dialogo de GLADAS avisa (en `ui`) donde va su retrato.
        ui::reiniciar_retrato();
        ui::reiniciar_registro();
        match &escena {
            Escena::Intro(intro) => {
                // Franjas de cine arriba y abajo, y la IA narrando.
                d.draw_rectangle(0, 0, ancho, 64, Color::BLACK);
                d.draw_rectangle(0, alto - 64, ancho, 64, Color::BLACK);
                let saltar = format!("{}: saltar", ui::boton(e.nombre("aceptar")));
                ui::texto(&mut d, &saltar, ancho - ui::medir(&saltar, 18) - 20, 24, 18, ui::TENUE);
                let texto = dialogo_ia.mostrar(intro.mensaje(), Voz::Ia, dt, son);
                ui::mensaje_ia(&mut d, texto, ancho, alto - 70);
                let (ojo, mira) = intro.camara(&juego.nave, juego.t);
                // Corazones y notitas que flotan alrededor de Miku.
                let cantidad = intro.corazones();
                if let (Some((x, y)), true) = (proyectar_ventana(intro.cabeza_miku(juego.t), ojo, mira, imagen), cantidad > 0.0) {
                    for k in 0..12 {
                        let u = (intro.t * 0.35 + k as f32 / 12.0).fract();
                        let a = ((1.0 - u) * 255.0 * cantidad) as u8;
                        let px = x + (k as f32 * 2.1 + intro.t * 1.5).sin() * 70.0 * (0.4 + u);
                        let py = y - u * 230.0;
                        let tam = 7.0 + 9.0 * u;
                        if k % 2 == 0 {
                            dibujar_corazon(&mut d, px, py, tam, Color::new(255, 110, 190, a));
                        } else {
                            dibujar_nota(&mut d, px, py, tam, Color::new(110, 240, 230, a));
                        }
                    }
                }
                // Lo que dice Miku, en un globito junto a su cabeza.
                if let (Some(g), Some((x, y))) = (intro.globo(), proyectar_ventana(intro.cabeza_miku(juego.t), ojo, mira, imagen)) {
                    let visible = dialogo_miku.mostrar(g, Voz::Miku, dt, son);
                    globo(&mut d, x + 40.0, y - 90.0, visible);
                }
                // El titulo al principio, con letras que rebotan.
                if intro.t < intro::T_TITULO + 0.6 {
                    let alfa = (intro.t / 0.6).min(1.0) * ((intro::T_TITULO + 0.6 - intro.t) / 0.6).min(1.0);
                    titulo(&mut d, "SPACE ADVENTURE", ancho, intro.t, alfa);
                }
                // La nave dormida ronca: tres "Z" que suben y se desvanecen.
                if let Some((x, y)) = intro.ronquido(juego.t).and_then(|p| proyectar_ventana(p, ojo, mira, imagen)) {
                    for k in 0..3 {
                        let u = (juego.t * 0.45 + k as f32 / 3.0).fract();
                        let a = ((1.0 - u) * 255.0) as u8;
                        let tam = 18 + (u * 22.0) as i32;
                        ui::texto(&mut d, "Z", (x + u * 40.0) as i32, (y - u * 90.0) as i32, tam, Color::new(230, 220, 255, a));
                    }
                }
            }
            Escena::Vegetta(v) => {
                let (texto, voz) = v.hablante().map_or((String::new(), Voz::Ia), |(_, t, voz)| (t, voz));
                let visible = dialogo_ia.mostrar(&texto, voz, dt, son);
                let (ojo_v, mira_v) = v.camara();
                let en_ventana = |p: Vec3| {
                    let vista = Mat4::look_at(ojo_v, mira_v, Vec3::new(0.0, 1.0, 0.0));
                    let proyeccion = Mat4::perspective(vegetta::FOV.to_radians(), RENDER_W as f32 / RENDER_H as f32, 0.05, 150.0);
                    let c = (proyeccion * vista).mul_vec4(matematica::Vec4::new(p.x, p.y, p.z, 1.0));
                    (c.w > 0.01).then(|| (imagen.x + (c.x / c.w + 1.0) * 0.5 * imagen.width, imagen.y + (1.0 - c.y / c.w) * 0.5 * imagen.height))
                };
                v.dibujar_ui(&mut d, &e, visible, ancho, alto, &en_ventana);
            }
            Escena::Urano(u) => {
                let (texto, voz) = u.hablante().map_or((String::new(), Voz::Ia), |(_, t, v)| (t, v));
                let visible = dialogo_ia.mostrar(&texto, voz, dt, son);
                u.dibujar_ui(&mut d, &e, visible, imagen, ancho, alto);
            }
            Escena::Aula(a) => {
                let ia = dialogo_ia.mostrar(a.ia(), Voz::Ia, dt, son).to_string();
                let profe = dialogo_profe.mostrar(a.texto_profe(), Voz::Profe, dt, son).to_string();
                let piloto = dialogo_miku.mostrar(a.replica_golpe(), Voz::Miku, dt, son);
                a.dibujar_ui(&mut d, &e, &ia, &profe, piloto, ancho, alto);
            }
            Escena::Entrega(en) => {
                let (texto, voz) = en.hablante().map_or((String::new(), Voz::Ia), |(_, t, v)| (t, v));
                let visible = dialogo_profe.mostrar(&texto, voz, dt, son);
                let koch = retratos.get(personajes::indice("koch"));
                en.dibujar_ui(&mut d, &e, visible, &textura_vitrina, koch, ancho, alto);
            }
            Escena::Concierto(c) => {
                let ia = dialogo_ia.mostrar(c.comentario(), Voz::Ia, dt, son);
                c.dibujar_ui(&mut d, &e, ia, ancho, alto);
            }
            Escena::Ritmo(r) => {
                let ia = dialogo_ia.mostrar(r.ia(), Voz::Ia, dt, son);
                r.dibujar_ui(&mut d, &e, ia, ancho, alto);
            }
            Escena::Junta(j) => {
                let (texto, voz) = j.hablante().map_or(("", Voz::Ia), |(_, t, v)| (t, v));
                let visible = dialogo_ia.mostrar(texto, voz, dt, son);
                j.dibujar_ui(&mut d, &e, visible, ancho, alto);
            }
            Escena::Cita(c) => {
                let (texto, voz) = c.hablante().map_or(("", Voz::Ia), |(_, t, v)| (t, v));
                let visible = dialogo_ia.mostrar(texto, voz, dt, son);
                c.dibujar_ui(&mut d, &e, visible, ancho, alto);
            }
            Escena::Fuente(f) => {
                let (texto, voz) = f.hablante().map_or(("", Voz::Ia), |(_, t, v)| (t, v));
                let visible = dialogo_ia.mostrar(texto, voz, dt, son);
                f.dibujar_ui(&mut d, &e, visible, ancho, alto);
            }
            Escena::Menu(m) => {
                // Con los ajustes abiertos, el menu de atras no se dibuja (se asomaria detras).
                if submenu.is_none() && pausa.is_none() {
                    m.dibujar(&mut d, &e, prog.notas, prog.conteo(), ancho, alto);
                }
                if !m.creditos && pasaporte.is_none() && pausa.is_none() && submenu.is_none() {
                    pegar_miku(&mut d, &textura_miku, ancho, alto);
                }
            }
            Escena::Mirador(m) => {
                let texto = m.mensaje.as_ref().map_or("", |x| x.0.as_str());
                let visible = dialogo_ia.mostrar(texto, Voz::Ia, dt, son);
                m.dibujar_ui(&mut d, &e, &prog, visible, ancho, alto);
            }
            Escena::Historia(h) => {
                let (nombre, texto, voz) = h.dialogo();
                let visible = dialogo_ia.mostrar_literal(texto, voz, dt, son);
                if !visible.is_empty() {
                    let w = 900.min(ancho - 40);
                    let alto_caja = ui::alto_dialogo(nombre, visible, w);
                    ui::caja_dialogo(&mut d, nombre, ui::LILA, visible, (ancho - w) / 2, alto - alto_caja - 20, w);
                }
                h.dibujar_ui(&mut d, &e, ancho, alto);
            }
            Escena::Camioneta(c) => {
                let (texto, voz) = c.hablante().map_or((String::new(), Voz::Ia), |(_, t, v)| (t, v));
                let visible = dialogo_ia.mostrar(&texto, voz, dt, son);
                c.dibujar_ui(&mut d, &e, visible, ancho, alto);
            }
            Escena::Luna(l) => {
                let (texto, voz) = l.hablante().map_or(("", Voz::Ia), |(_, t, v)| (t, v));
                let visible = dialogo_ia.mostrar(texto, voz, dt, son);
                l.dibujar_ui(&mut d, &e, visible, ancho, alto);
            }
            Escena::Jupiter(j) => {
                // Los letreros de puntos van donde paso (se proyectan igual que la escena).
                let (ojo, mira, arriba) = j.camara();
                let vp = Mat4::perspective(70f32.to_radians(), fb.ancho as f32 / fb.alto as f32, 0.1, 600.0) * Mat4::look_at(ojo, mira, arriba);
                let (ex, ey) = (imagen.width / fb.ancho as f32, imagen.height / fb.alto as f32);
                let a_pantalla = |p: Vec3| proyectar(p, &vp, &fb).map(|(x, y, _)| (imagen.x + x * ex, imagen.y + y * ey));
                j.dibujar_ui(&mut d, &e, &a_pantalla, &rec.pantallas_cabina, ancho, alto);
                if let Some(t) = j.bill() {
                    let visible = dialogo_bill.mostrar(t, Voz::Profe, dt, son).to_string();
                    dibujar_bill_en(&mut d, &retrato_bill, &visible, 1.0, ancho, Some(alto));
                }
            }
            Escena::Divorcio(dv) => {
                let habla = dv.hablante();
                let (texto, voz) = habla.as_ref().map_or(("", Voz::Ia), |(_, t, v)| (t.as_str(), *v));
                let visible = dialogo_ia.mostrar(texto, voz, dt, son);
                dv.dibujar_ui(&mut d, &e, visible, ancho, alto);
            }
            Escena::Mercurio(m) => {
                let habla = m.hablante();
                let (texto, voz) = habla.as_ref().map_or(("", Voz::Ia), |(_, t, v)| (t.as_str(), *v));
                let visible = dialogo_ia.mostrar(texto, voz, dt, son);
                m.dibujar_ui(&mut d, &e, visible, ancho, alto);
            }
            Escena::Neptuno(n) => {
                let (texto, voz) = n.hablante().map_or(("", Voz::Ia), |(_, t, v)| (t, v));
                let visible = dialogo_ia.mostrar(texto, voz, dt, son);
                let (ojo, mira) = n.camara();
                let cabezas: Vec<Option<(f32, f32)>> = n.cabezas().iter().map(|&p| proyectar_ventana(p, ojo, mira, imagen)).collect();
                n.dibujar_ui(&mut d, &e, visible, &cabezas, ancho, alto);
            }
            Escena::Espacio => {
                let cine = juego.aterrizaje.as_ref().filter(|a| a.en_cine());
                if cine.is_none() && pasaporte.is_none() && pausa.is_none() && submenu.is_none() && puerta_uvg.is_none() {
                    barra_de_acciones(&mut d, &juego, &e, hud);
                }
                // Con un menu encima (pausa, ajustes, tienda, pasaporte, la entrada de la UVG...) el HUD
                // se esconde: si no, sus letras se transparentan debajo del menu.
                let con_menu = pausa.is_some() || submenu.is_some() || pasaporte.is_some() || puerta_uvg.is_some();
                if hud && cine.is_none() && !con_menu {
                    // Primero los nombres de los planetas: el HUD (y el panel de TAB) va encima.
                    marcadores(&mut d, &juego, imagen, ancho, alto);
                    letreros(&mut d, &juego, imagen);
                    hud_espacio(&mut d, &juego, &e, &st, ms, info, &prog);
                    // El marcador del cometa: un circulo encima, o en la orilla de la pantalla si esta fuera.
                    if let Some(c) = &juego.coleccion.cometa {
                        let margen = 40.0;
                        let (x, y, fuera) = match proyectar_ladeado(c.pos, juego.camara.ojo, juego.camara.mira, juego.camara.arriba(), imagen) {
                            Some((x, y)) => {
                                let (cx, cy) = (x.clamp(margen, ancho as f32 - margen), y.clamp(margen + 150.0, alto as f32 - margen));
                                (cx, cy, cx != x || cy != y)
                            }
                            None => (ancho as f32 * 0.5, alto as f32 - margen - 60.0, true),
                        };
                        let latido = 1.0 + 0.2 * (juego.reloj * 8.0).sin();
                        d.draw_circle_lines(x as i32, y as i32, 22.0 * latido, Color::new(150, 230, 255, 255));
                        d.draw_circle_lines(x as i32, y as i32, 18.0 * latido, Color::new(150, 230, 255, 160));
                        let distancia = format!("{:.0}", (c.pos - juego.nave.pos).length() * 10.0);
                        let s = if fuera { format!("COMETA {distancia} km") } else { distancia + " km" };
                        ui::texto(&mut d, &s, x as i32 - ui::medir(&s, 16) / 2, y as i32 + 28, 16, Color::new(150, 230, 255, 255));
                    }
                }
                // En la orilla del sistema: el borde de la pantalla se pone azul oscuro y titila un aviso.
                let orilla = juego.en_la_orilla();
                if orilla > 0.0 && juego.rescate.is_none() {
                    let a = (orilla * 200.0) as u8;
                    let g = (alto as f32 * 0.25) as i32;
                    d.draw_rectangle_gradient_v(0, 0, ancho, g, Color::new(10, 20, 80, a), Color::new(10, 20, 80, 0));
                    d.draw_rectangle_gradient_v(0, alto - g, ancho, g, Color::new(10, 20, 80, 0), Color::new(10, 20, 80, a));
                    if (juego.reloj * 3.0).fract() < 0.6 {
                        let s = "¡FUERA DEL SISTEMA SOLAR!";
                        let tam = 34;
                        ui::titulo_de(&mut d, s, (ancho - ui::medir_titulo(s, tam)) / 2, (alto as f32 * 0.24) as i32, tam, Color::new(140, 200, 255, 255));
                    }
                }
                // El rescate: una estrella que cierra la pantalla y la vuelve a abrir (Mario Galaxy).
                if let Some(k) = juego.estrella_de_rescate() {
                    estrella_de_rescate(&mut d, k, juego.reloj, ancho, alto);
                }
                // La cutscene del aterrizaje: franjas de cine y la tarjeta con el nombre del planeta.
                if let Some(a) = &juego.aterrizaje {
                    let franja = (a.franjas() * alto as f32 * 0.11) as i32;
                    if franja > 0 {
                        d.draw_rectangle(0, 0, ancho, franja, Color::BLACK);
                        d.draw_rectangle(0, alto - franja, ancho, franja, Color::BLACK);
                    }
                    let (titulo, subtitulo, alfa) = a.tarjeta();
                    if alfa > 0.0 && !titulo.is_empty() {
                        let a8 = (alfa * 255.0) as u8;
                        let tam = 64;
                        let w = ui::medir_titulo(titulo, tam);
                        let y = (alto as f32 * 0.16) as i32;
                        ui::titulo_de(&mut d, titulo, (ancho - w) / 2, y, tam, Color::new(250, 240, 255, a8));
                        // El subtitulo en una cinta (como las tarjetas de nivel), que se abre con la tarjeta.
                        let w2 = ui::medir(subtitulo, 22);
                        let ancho_cinta = (w2 as f32 + 110.0) * (0.3 + 0.7 * alfa);
                        let yc = (y + tam + 14) as f32;
                        if ui::cinta(&mut d, (ancho as f32 - ancho_cinta) / 2.0, yc, ancho_cinta, 44.0, Color::new(255, 255, 255, a8)) {
                            if alfa > 0.8 {
                                ui::texto(&mut d, subtitulo, (ancho - w2) / 2, yc as i32 + 11, 22, Color::new(90, 50, 30, a8));
                            }
                        } else {
                            ui::texto(&mut d, subtitulo, (ancho - w2) / 2, y + tam + 18, 24, Color::new(215, 200, 240, a8));
                        }
                    }
                    if a.en_cine() && franja > 10 {
                        let s = format!("{}: saltar", ui::boton(e.nombre("aceptar")));
                        ui::texto(&mut d, &s, ancho - ui::medir(&s, 16) - 20, alto - franja + (franja - 16) / 2, 16, ui::TENUE);
                    }
                }
                if let (Some((texto, _)), None) = (&juego.mensaje, &cine) {
                    let visible = dialogo_ia.mostrar(texto, Voz::Ia, dt, son);
                    let derecha = if hud { ancho - ANCHO_TABLERO - 40 } else { ancho - 20 };
                    ui::mensaje_ia_entre(&mut d, visible, 20, derecha, alto);
                }
                // La demo: el rotulo de la parte del recorrido, arriba al centro (en las tomas en crudo, no).
                if let Some((titulo, sub)) = demo.as_ref().and_then(|dm| dm.rotulo()).filter(|_| !grabar.as_ref().is_some_and(|g| g.limpio)) {
                    let tam = 30;
                    let y = (alto as f32 * 0.15) as i32;
                    ui::titulo_de(&mut d, titulo, (ancho - ui::medir_titulo(titulo, tam)) / 2, y, tam, Color::new(110, 235, 225, 255));
                    ui::subtitulo(&mut d, sub, (ancho - ui::medir(sub, 18)) / 2, y + tam + 10, 18, ui::TEXTO);
                }
                // La entrada de la UVG, encima de todo (menos de la pausa).
                if let (Some(p), true) = (&puerta_uvg, pausa.is_none() && submenu.is_none()) {
                    p.dibujar(&mut d, &e, ancho, alto);
                }
            }
        }
        // El retrato de GLADAS (lo que salio de su framebuffer propio), en su caja de dialogo.
        let retrato = ui::retrato();
        if let (Some((x, y, lado)), true) = (retrato, gladas_hablaba) {
            d.draw_texture_pro(
                &textura_retrato,
                Rectangle::new(0.0, 0.0, RETRATO as f32, RETRATO as f32),
                Rectangle::new(x as f32, y as f32, lado as f32, lado as f32),
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }
        gladas_hablaba = retrato.is_some();
        if fundido > 0.0 {
            d.draw_rectangle(0, 0, ancho, alto, Color::new(0, 0, 0, (fundido * 255.0) as u8));
        }
        // Encima de todo: el pasaporte (si esta abierto) y el aviso de sello nuevo.
        if let (Some(k), Escena::Espacio | Escena::Menu(_)) = (pasaporte, &escena) {
            progreso::dibujar_pasaporte(&mut d, &prog, k, ancho, alto, e.nombre("pasaporte"), &sellos);
        }
        // La tienda, el casillero o el pase, encima de todo (del menu o del juego en pausa).
        if let Some(t) = &submenu {
            t.dibujar(&mut d, &e, &prog, &textura_vitrina, &imagenes, ancho, alto);
        }
        // Bill Nye cuenta su dato (arriba al centro, con su retrato), mientras se vuela.
        if let Some((dato, t)) = &mut bill_dice {
            *t += dt_real;
            let largo = dato.chars().count() as f32;
            if *t > 4.0 + largo / 20.0 || !matches!(escena, Escena::Espacio) {
                bill_dice = None;
            } else {
                let visible = dialogo_bill.mostrar(dato, Voz::Profe, dt_real, son).to_string();
                dibujar_bill(&mut d, &retrato_bill, &visible, *t, ancho);
            }
        }
        progreso::dibujar_avisos(&mut d, &prog, ancho);
        if let Some(p) = &pausa {
            let estado_vibracion = (vibra.is_some(), vibra.as_ref().is_some_and(|v| v.hay_control()));
            p.dibujar(&mut d, &prog.ajustes, hud, estado_vibracion, &e, ancho, alto);
            pegar_miku(&mut d, &textura_miku, ancho, alto);
        }
        // Encima de todo, la barra de pestanas del menu del juego (si hay menu).
        let en_la_lista = matches!(&escena, Escena::Menu(m) if !m.creditos && m.iniciado()) && pasaporte.is_none();
        if let Some(k) = pestana_abierta(&pausa, &submenu, en_la_lista) {
            casillero::dibujar_pestanas(&mut d, &e, k, matches!(escena, Escena::Menu(_)), prog.novedades(), reloj_menu, ancho, alto);
        }
        // La grabacion: el rotulo de la toma, su fundido, y el cuadro (ya con todo) a ffmpeg.
        if let Some(g) = &mut grabar {
            if let Some(((titulo, sub), alfa)) = g.rotulo() {
                let (tam, tam_sub) = (34, 19);
                let y = (alto as f32 * if g.al_final() { 0.74 } else { 0.13 }) as i32;
                let w = ui::medir_titulo(titulo, tam).max(ui::medir(sub, tam_sub)) + 56;
                d.draw_rectangle_rounded(Rectangle::new(((ancho - w) / 2) as f32, (y - 16) as f32, w as f32, (tam + tam_sub + 44) as f32), 0.3, 8, Color::new(10, 6, 24, (alfa * 190.0) as u8));
                let a = (alfa * 255.0) as u8;
                ui::titulo_de(&mut d, titulo, (ancho - ui::medir_titulo(titulo, tam)) / 2, y, tam, Color::new(110, 235, 225, a));
                ui::subtitulo(&mut d, sub, (ancho - ui::medir(sub, tam_sub)) / 2, y + tam + 10, tam_sub, Color::new(240, 230, 255, a));
            }
            let negro = g.negro();
            if negro > 0.0 {
                d.draw_rectangle(0, 0, ancho, alto, Color::new(0, 0, 0, (negro * 255.0) as u8));
            }
            unsafe {
                raylib::ffi::rlDrawRenderBatchActive();
                let datos = raylib::ffi::rlReadScreenPixels(pixeles_w, pixeles_h);
                g.cuadro(pixeles_w, pixeles_h, std::slice::from_raw_parts(datos, (pixeles_w * pixeles_h * 4) as usize));
                raylib::ffi::MemFree(datos as *mut std::os::raw::c_void);
            }
            if let Some(s) = son {
                s.grabar(grabacion::DT);
            }
            g.avanzar();
        }
    }
    if let Some(g) = &mut grabar {
        g.terminar(son.and_then(|s| s.mezcla().map(|m| m.wav())));
        // Las tomas en crudo llevan aparte los efectos sin la musica (para editarlas con otra).
        if let (true, Some(m)) = (g.limpio, son.and_then(|s| s.mezcla())) {
            std::fs::write(format!("{}.efectos.wav", g.archivo), m.wav_efectos()).expect("no se pudo guardar los efectos");
        }
    }
    controles::descargar();
}

/// Lo que "aprieta" la grabacion en este cuadro: el piloto automatico de cada toma.
fn entrada_grabada(g: &grabacion::Grabacion, juego: &mut Juego, escena: &Escena) -> Entrada {
    use grabacion::Auto;
    let t = g.t;
    match (g.actual().auto, escena) {
        (Auto::Notas, Escena::Espacio) => {
            // De nota en nota, como en la captura `recoger`.
            let nave = juego.nave.pos;
            if let Some(hacia) = juego.coleccion.notas.iter().filter(|n| n.viva).map(|n| n.pos).min_by(|p, q| (*p - nave).length().total_cmp(&(*q - nave).length())) {
                (juego.nave.yaw, juego.nave.pitch) = salto::rumbo_hacia(hacia - nave);
            }
            Entrada { empuje: 1.0, ..Default::default() }
        }
        (Auto::Orbita, Escena::Espacio) => Entrada { empuje: 0.6, ..Default::default() },
        (Auto::Dialogo, _) => Entrada { aceptar: g.pulso(2.4, 1.2), ..Default::default() },
        (Auto::Camioneta, Escena::Camioneta(c)) => c.chofer(),
        (Auto::Parkour, Escena::Vegetta(v)) if v.en_parkour() => v.entrada_parkour(),
        (Auto::Tnt, Escena::Vegetta(_)) => Entrada { empuje: 1.0, girar: 0.55, disparando: true, ..Default::default() },
        (Auto::Mina, Escena::Vegetta(v)) if v.en_mina() => Entrada { picar: true, ..Default::default() },
        (Auto::Asteroides, Escena::Jupiter(_)) => Entrada { girar: (t * 0.7).sin(), cabecear: (t * 0.5).cos() * 0.6, disparando: true, ..Default::default() },
        (Auto::Ritmo, Escena::Ritmo(r)) => {
            let (piso, aire) = r.golpes_solos(grabacion::DT);
            Entrada { piso, aire, ..Default::default() }
        }
        (Auto::Abajo, _) => Entrada { abajo: g.pulso(0.9, 0.6), ..Default::default() },
        (Auto::Derecha, _) => Entrada { derecha: g.pulso(0.8, 0.6), ..Default::default() },
        _ => Entrada::default(),
    }
}

/// La pestana del menu del juego que esta abierta (None si no hay menu): 0 JUGAR / INICIO (la
/// pausa, o la lista del menu principal), 1 PERSONAJES, 2 MOCHILA, 3 PASE, 4 TIENDA y 5 AJUSTES.
fn pestana_abierta(pausa: &Option<Pausa>, submenu: &Option<Submenu>, en_la_lista: bool) -> Option<usize> {
    match (pausa, submenu) {
        (_, Some(Submenu::Casillero(_))) => Some(1),
        (_, Some(Submenu::Mochila(_))) => Some(2),
        (_, Some(Submenu::Pase(_))) => Some(3),
        (_, Some(Submenu::Tienda(_))) => Some(4),
        (Some(p), None) if p.es_ajustes() => Some(5),
        (Some(_), None) => Some(0),
        (None, None) if en_la_lista => Some(0),
        (None, None) => None,
    }
}

/// Abre la pestana `k` del menu del juego. La 0, en el menu principal, es la lista (cierra todo);
/// jugando, es la pausa. Al abrir el pase o los personajes, sus novedades quedan vistas.
fn abrir_pestana(k: usize, en_menu: bool, pausa: &mut Option<Pausa>, submenu: &mut Option<Submenu>, prog: &mut Progreso) {
    *pausa = None;
    *submenu = None;
    let nivel = prog.nivel_del_pase().0;
    match k {
        0 if en_menu => {}
        0 => *pausa = Some(Pausa::new()),
        1 => {
            *submenu = Some(Submenu::Casillero(Casillero::new(prog)));
            prog.personajes_visto = nivel;
            prog.guardar();
        }
        2 => *submenu = Some(Submenu::Mochila(mochila::Mochila::new(!en_menu))),
        3 => {
            *submenu = Some(Submenu::Pase(Pase::new(prog)));
            prog.pase_visto = nivel;
            prog.guardar();
        }
        4 => *submenu = Some(Submenu::Tienda(Tienda::new(prog))),
        _ => *pausa = Some(Pausa::ajustes()),
    }
}

/// La caja de Bill Nye: su retrato a la izquierda, su nombre y el dato, arriba al centro.
fn dibujar_bill(d: &mut RaylibDrawHandle, retrato: &Texture2D, texto: &str, t: f32, ancho: i32) {
    dibujar_bill_en(d, retrato, texto, t, ancho, None);
}

/// La caja de Bill Nye; con `abajo = Some(alto)`, pegada abajo (en Jupiter, para no tapar la arena).
fn dibujar_bill_en(d: &mut RaylibDrawHandle, retrato: &Texture2D, texto: &str, t: f32, ancho: i32, abajo: Option<i32>) {
    // A la derecha del panel de la izquierda del HUD (no lo tapa).
    let x = 330.max((ancho - 780) / 2);
    let w = 780.min(ancho - x - 24);
    let entra = (t / 0.25).clamp(0.0, 1.0);
    let lineas = ui::envolver(texto, 19, w - 170).len().max(2) as i32;
    let h = 52 + lineas * 25;
    let y = match abajo {
        Some(alto) => alto - h - 44 + ((1.0 - entra) * 30.0) as i32,
        None => 84 - ((1.0 - entra) * 30.0) as i32,
    };
    ui::panel(d, x, y, w, h);
    // El retrato en su cuadrito azul de laboratorio.
    let lado = (h - 20).min(118);
    d.draw_rectangle_rounded(Rectangle::new((x + 12) as f32, (y + 10) as f32, lado as f32, lado as f32), 0.15, 6, Color::new(60, 110, 170, 255));
    let fuente = Rectangle::new(0.0, 0.0, retrato.width as f32, retrato.height as f32);
    d.draw_texture_pro(retrato, fuente, Rectangle::new((x + 12) as f32, (y + 10) as f32, lado as f32, lado as f32), Vector2::zero(), 0.0, Color::WHITE);
    let tx = x + lado + 28;
    ui::texto(d, "BILL NYE · EL CHICO DE LA CIENCIA", tx, y + 14, 16, Color::new(130, 190, 255, 255));
    ui::parrafo(d, texto, tx, y + 40, 19, w - lado - 44, ui::TEXTO);
}

/// Lo que se puede hacer ahorita, como botones arriba al centro (aterrizar, el salto, el destino).
/// Se ve siempre, aunque el HUD este oculto: si no, no hay como saber que E aterriza.
fn barra_de_acciones(d: &mut RaylibDrawHandle, juego: &Juego, e: &Entrada, hud: bool) {
    let ancho = d.get_screen_width();
    let (cerca, _) = mas_cercano(juego.nave.pos, juego.t);
    let (aceptar, aterrizar) = (e.nombre("aceptar"), e.nombre("aterrizar"));
    // Lo que se puede hacer, como botones, arriba al centro.
    let mut acciones: Vec<(&str, String, bool)> = Vec::new();
    match &juego.aterrizaje {
        Some(a) if a.posado() => acciones.push((aterrizar, format!("Despegar de {}", CUERPOS[a.planeta].nombre), true)),
        Some(_) => {}
        None if juego.salto.activo() => acciones.push(("", format!("Hiperespacio hacia {}", CUERPOS[juego.salto.destino].nombre), false)),
        None => {
            acciones.push((aceptar, format!("Salto a {}", CUERPOS[juego.destino_del_salto()].nombre), false));
            let ir_a = juego.objetivo.unwrap_or(cerca.max(1));
            match aterrizable(juego.nave.pos, juego.t) {
                _ if (juego.nave.pos - mirador_en(juego.t)).length() < DISTANCIA_MIRADOR => acciones.push((aterrizar, "Descansar en el Mirador".to_string(), true)),
                Some(i) if !juego.aterrizar_al_llegar => acciones.push((aterrizar, format!("Aterrizar en {}", CUERPOS[i].nombre), true)),
                _ => acciones.push((aterrizar, format!("Ir a {} y aterrizar", CUERPOS[ir_a].nombre), false)),
            }
            acciones.push((e.nombre("objetivo"), "Destino".to_string(), false));
            // Junto a un planeta: el modo orbita.
            if juego.orbita.is_some() {
                acciones.push((e.nombre("orbitar"), "Salir de la órbita".to_string(), true));
            } else if juego.sincronia > 0.3 {
                acciones.push((e.nombre("orbitar"), "Órbita".to_string(), false));
            }
        }
    }
    // Cabe entre el panel de la izquierda (si el HUD esta) y "TAB: info y controles". Si no cabe, se achica; si
    // aun asi no cabe, se quita lo menos importante (el destino).
    let pista_info = ui::medir_accion(e.nombre("info"), "Controles", 16);
    let (izq, der) = if hud { (360, ancho - pista_info - 40) } else { (20, ancho - 20) };
    let disponible = der - izq - 32;
    let medida = |acciones: &[(&str, String, bool)], tam: i32| {
        let sep = tam + 8;
        acciones.iter().map(|(b, q, _)| if b.is_empty() { ui::medir(q, tam) } else { ui::medir_accion(b, q, tam) }).sum::<i32>() + sep * (acciones.len() as i32 - 1).max(0)
    };
    let mut tam = 20;
    while tam > 14 && medida(&acciones, tam) > disponible {
        tam -= 2;
    }
    if medida(&acciones, tam) > disponible && acciones.len() > 2 {
        acciones.pop();
    }
    let total = medida(&acciones, tam);
    if !acciones.is_empty() {
        let x0 = izq + 16 + ((disponible - total) / 2).max(0);
        let alto_barra = tam * 2 + 12;
        // Sin marco: un velo arriba, que se desvanece hacia abajo (las teclas ya se ven solas).
        ui::velo(d, x0 - 60, 0, total + 120, alto_barra + 40, ui::Hacia::Abajo);
        let mut x = x0;
        for (boton, que, resaltada) in &acciones {
            // La accion importante late un poquito.
            let latido = *resaltada && (juego.t * 3.0).sin() > 0.0;
            let y = 20 + (alto_barra - tam * 3 / 2) / 2;
            x += if boton.is_empty() { ui::subtitulo(d, que, x, y + tam / 4, tam, Color::new(150, 220, 255, 255)); ui::medir(que, tam) } else { ui::accion(d, boton, que, x, y, tam, latido) } + tam + 8;
        }
    }
}

/// Los marcadores de los planetas y del Mirador: su nombre y la distancia, sobre cada uno. El destino
/// del hiperespacio va resaltado, y si esta fuera de la pantalla, una flecha en la orilla apunta hacia el.
fn marcadores(d: &mut RaylibDrawHandle, juego: &Juego, imagen: Rectangle, ancho: i32, alto: i32) {
    let (ojo, mira, ladeo) = (juego.camara.ojo, juego.camara.mira, juego.camara.arriba());
    let arriba = Vec3::new(0.0, 1.0, 0.0);
    let mut marcas: Vec<(Vec3, &str, bool, f32)> = (1..CUERPOS.len())
        .map(|i| {
            let c = &CUERPOS[i];
            (c.posicion(juego.t) + arriba * (c.radio_visual() + 1.5), c.nombre, juego.objetivo == Some(i), c.radio_visual())
        })
        .collect();
    marcas.push((mirador_en(juego.t) + arriba * 3.5, "Mirador", false, 2.0));
    // Las maravillas tambien tienen nombre (para encontrarlas).
    marcas.push((maravillas::gusano_en(juego.t) + arriba * 9.0, "Agujero de gusano", false, 7.0));
    marcas.push((maravillas::GARGANTUA + arriba * 22.0, "Gargantúa", false, 30.0));
    marcas.push((maravillas::blip_en(juego.t) + arriba * 5.0, "Blip-A", false, 4.0));
    marcas.push((maravillas::monolito_en(juego.t) + arriba * 11.0, "Monolito", false, 4.0));
    let centro = (ancho as f32 * 0.5, alto as f32 * 0.5);
    // Las etiquetas ya puestas (para no encimar una sobre otra). El destino va primero: siempre se ve.
    marcas.sort_by_key(|m| !m.2);
    let mut ocupados: Vec<Rectangle> = Vec::new();
    for (p, nombre, escogido, radio) in marcas {
        let distancia = (p - juego.nave.pos).length();
        // De cerca ya se ve solo: no hace falta marca (salvo el destino).
        if distancia < radio * 2.5 + 10.0 && !escogido {
            continue;
        }
        let color = if escogido { Color::new(255, 130, 210, 255) } else { Color::new(210, 220, 255, 235) };
        let km = format!("{:.0} km", distancia * 10.0);
        let en_pantalla = proyectar_ladeado(p, ojo, mira, ladeo, imagen).filter(|&(x, y)| x > 20.0 && y > 20.0 && x < ancho as f32 - 20.0 && y < alto as f32 - 20.0);
        match en_pantalla {
            Some((x, y)) => {
                // Solo el nombre y la distancia, centrados sobre el planeta (sin figuritas).
                if escogido || distancia < 450.0 {
                    let tam = if escogido { 18 } else { 14 };
                    let s = if escogido { format!("{}  {km}", nombre.to_uppercase()) } else { format!("{nombre}  {km}") };
                    let w = ui::medir(&s, tam);
                    let tx = x as i32 - w / 2;
                    let caja = Rectangle::new(tx as f32 - 2.0, y - tam as f32 * 0.6, w as f32 + 4.0, tam as f32 * 1.2);
                    if escogido || !ocupados.iter().any(|o| o.check_collision_recs(&caja)) {
                        ocupados.push(caja);
                        ui::texto(d, &s, tx + 1, y as i32 - tam / 2 + 1, tam, Color::new(0, 0, 0, 200));
                        ui::texto(d, &s, tx, y as i32 - tam / 2, tam, color);
                    }
                }
            }
            None if escogido => {
                // Fuera de la pantalla (o atras): una flecha en la orilla, hacia donde queda.
                let atras = ojo * 2.0 - p;
                let (sx, sy) = match proyectar_ladeado(p, ojo, mira, ladeo, imagen) {
                    Some(q) => q,
                    None => proyectar_ladeado(atras, ojo, mira, ladeo, imagen).map_or((centro.0, alto as f32), |(x, y)| (2.0 * centro.0 - x, 2.0 * centro.1 - y)),
                };
                let (dx, dy) = (sx - centro.0, sy - centro.1);
                let largo = (dx * dx + dy * dy).sqrt().max(1.0);
                let (ux, uy) = (dx / largo, dy / largo);
                let k = ((centro.0 - 60.0) / ux.abs().max(0.001)).min((centro.1 - 60.0) / uy.abs().max(0.001));
                let (x, y) = (centro.0 + ux * k, centro.1 + uy * k);
                let punta = Vector2::new(x + ux * 16.0, y + uy * 16.0);
                let (a, b) = (Vector2::new(x - uy * 10.0, y + ux * 10.0), Vector2::new(x + uy * 10.0, y - ux * 10.0));
                d.draw_triangle(punta, a, b, color);
                d.draw_triangle(punta, b, a, color);
                let s = format!("{}  {km}", nombre.to_uppercase());
                let tx = (x as i32 - ui::medir(&s, 16) / 2).clamp(10, ancho - ui::medir(&s, 16) - 10);
                ui::texto(d, &s, tx, y as i32 + if uy > 0.0 { -34 } else { 20 }, 16, color);
            }
            None => {}
        }
    }
}

/// Un globito de dialogo con colita hacia abajo a la izquierda.
fn globo(d: &mut RaylibDrawHandle, x: f32, y: f32, texto: &str) {
    if texto.is_empty() {
        return;
    }
    let tam = 26;
    let w = ui::medir(texto, tam) as f32 + 36.0;
    // El globo de pixel art (papel lila con su colita); si no hay sprites, el redondo de antes.
    if ui::globo(d, x, y, w + 16.0, 2.0, ui::Tono::Papel) {
        ui::texto(d, texto, (x + 20.0) as i32, (y + 18.0) as i32, tam, Color::new(70, 30, 110, 255));
        return;
    }
    let caja = Rectangle::new(x, y, w, 50.0);
    d.draw_rectangle_rounded(caja, 0.5, 8, Color::new(255, 250, 255, 240));
    d.draw_rectangle_rounded_lines(caja, 0.5, 8, Color::new(255, 120, 200, 255));
    let (a, b, c) = (Vector2::new(x + 16.0, y + 48.0), Vector2::new(x + 6.0, y + 70.0), Vector2::new(x + 36.0, y + 48.0));
    d.draw_triangle(a, b, c, Color::new(255, 250, 255, 240));
    d.draw_triangle(a, c, b, Color::new(255, 250, 255, 240));
    ui::texto(d, texto, (x + 18.0) as i32, (y + 12.0) as i32, tam, Color::new(90, 30, 110, 255));
}

/// Un corazon de dos circulos y un triangulo, centrado en (x, y).
fn dibujar_corazon(d: &mut RaylibDrawHandle, x: f32, y: f32, tam: f32, color: Color) {
    d.draw_circle((x - tam * 0.5) as i32, y as i32, tam * 0.55, color);
    d.draw_circle((x + tam * 0.5) as i32, y as i32, tam * 0.55, color);
    let (a, b, c) = (Vector2::new(x - tam * 1.02, y + tam * 0.15), Vector2::new(x, y + tam * 1.3), Vector2::new(x + tam * 1.02, y + tam * 0.15));
    // raylib pide el orden antihorario; se dibuja en los dos para no pensarlo.
    d.draw_triangle(a, b, c, color);
    d.draw_triangle(a, c, b, color);
}

/// Una corchea: cabeza, palito y banderita.
fn dibujar_nota(d: &mut RaylibDrawHandle, x: f32, y: f32, tam: f32, color: Color) {
    d.draw_circle(x as i32, (y + tam) as i32, tam * 0.6, color);
    d.draw_rectangle((x + tam * 0.45) as i32, (y - tam) as i32, (tam * 0.25).max(2.0) as i32, (tam * 2.0) as i32, color);
    d.draw_rectangle((x + tam * 0.45) as i32, (y - tam) as i32, (tam * 0.9) as i32, (tam * 0.35).max(2.0) as i32, color);
}

/// El titulo de la intro: cada letra rebota a su ritmo y cambia de color.
fn titulo(d: &mut RaylibDrawHandle, texto: &str, ancho: i32, t: f32, alfa: f32) {
    // 80 de alto, o menos si el titulo no cabe en la ventana.
    let medida = ui::medir_titulo(texto, 80) + 80 / 12 * texto.chars().count() as i32;
    let tam = if medida > ancho - 80 { 80 * (ancho - 80) / medida } else { 80 };
    let colores = [Color::new(255, 130, 200, 255), Color::new(190, 150, 255, 255), Color::new(110, 235, 225, 255)];
    let total = ui::medir_titulo(texto, tam) + tam / 12 * texto.chars().count() as i32;
    let mut x = (ancho - total) / 2;
    let a = (alfa.clamp(0.0, 1.0) * 255.0) as u8;
    for (k, letra) in texto.chars().enumerate() {
        let s = letra.to_string();
        let y = 96 + ((t * 5.0 + k as f32 * 0.6).sin() * 10.0) as i32;
        let c = colores[(k + (t * 2.0) as usize) % colores.len()];
        ui::titulo_de(d, &s, x, y, tam, Color::new(c.r, c.g, c.b, a));
        x += ui::medir_titulo(&s, tam) + tam / 12;
    }
    let sub = "un viaje por el sistema solar";
    let w = ui::medir(sub, 28);
    ui::subtitulo(d, sub, (ancho - w) / 2, 214, 28, Color::new(240, 230, 255, a));
}

/// La transicion de la estrella (como en Mario Galaxy): todo negro menos una estrella de cinco
/// picos en el centro, que se cierra (`cubre` va de 0 a 1) girando, con un brillo dorado en la orilla.
fn estrella_de_rescate(d: &mut RaylibDrawHandle, cubre: f32, reloj: f32, ancho: i32, alto: i32) {
    let centro = Vector2::new(ancho as f32 * 0.5, alto as f32 * 0.5);
    let k = cubre.clamp(0.0, 1.0);
    let grande = (ancho as f32).hypot(alto as f32) * 1.2;
    let radio = grande * (1.0 - k).powf(1.6);
    let giro = reloj * 1.5;
    // Las puntas y los entrantes de la estrella.
    let puntos: Vec<Vector2> = (0..10)
        .map(|i| {
            let a = giro - std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
            let r = if i % 2 == 0 { radio } else { radio * 0.45 };
            Vector2::new(centro.x + a.cos() * r, centro.y + a.sin() * r)
        })
        .collect();
    // Lo de afuera de la estrella, negro: un cuadrilatero por cada lado, hasta muy lejos.
    let lejos = |p: Vector2| Vector2::new(centro.x + (p.x - centro.x) * 60.0 + 1.0, centro.y + (p.y - centro.y) * 60.0 + 1.0);
    let negro = Color::new(4, 2, 16, 255);
    for i in 0..10 {
        let (a, b) = (puntos[i], puntos[(i + 1) % 10]);
        let (fa, fb) = (lejos(a), lejos(b));
        for (p, q, r) in [(a, b, fb), (a, fb, fa)] {
            d.draw_triangle(p, q, r, negro);
            d.draw_triangle(p, r, q, negro);
        }
    }
    if radio < 1.0 {
        d.draw_rectangle(0, 0, ancho, alto, negro);
    }
    // La orilla dorada de la estrella.
    for i in 0..10 {
        d.draw_line_ex(puntos[i], puntos[(i + 1) % 10], 4.0, Color::new(255, 220, 90, (200.0 * (1.0 - k * 0.5)) as u8));
    }
    // Con la pantalla cerrada, el consejo de la Guia del viajero intergalactico: en letras
    // grandes y amistosas.
    if k > 0.7 {
        let a = (((k - 0.7) / 0.3).min(1.0) * 255.0) as u8;
        let tam = (alto as f32 * 0.075) as i32;
        let s = "NO ENTRE EN PÁNICO";
        ui::titulo_de(d, s, (ancho - ui::medir_titulo(s, tam)) / 2, alto / 2 - tam / 2, tam, Color::new(255, 200, 90, a));
    }
}

/// Un punto del mundo en pixeles de la ventana, con la camara `ojo`, `mira`
/// y el FOV normal (lo mismo que hace el pipeline, para poner texto encima).
/// `imagen` es donde quedo la imagen del juego en la ventana (ver `rect_imagen`).
fn proyectar_ventana(p: Vec3, ojo: Vec3, mira: Vec3, imagen: Rectangle) -> Option<(f32, f32)> {
    proyectar_ladeado(p, ojo, mira, Vec3::new(0.0, 1.0, 0.0), imagen)
}

/// Lo mismo, con la camara ladeada (`arriba` no es el de siempre; ver `Camara::arriba`).
fn proyectar_ladeado(p: Vec3, ojo: Vec3, mira: Vec3, arriba: Vec3, imagen: Rectangle) -> Option<(f32, f32)> {
    let vista = Mat4::look_at(ojo, mira, arriba);
    let proyeccion = Mat4::perspective(FOV.to_radians(), RENDER_W as f32 / RENDER_H as f32, CERCA, LEJOS);
    let c = (proyeccion * vista).mul_vec4(matematica::Vec4::new(p.x, p.y, p.z, 1.0));
    (c.w > 0.01).then(|| (imagen.x + (c.x / c.w + 1.0) * 0.5 * imagen.width, imagen.y + (1.0 - c.y / c.w) * 0.5 * imagen.height))
}

/// Donde va la imagen del juego en una ventana de `ancho` x `alto`: lo mas
/// grande que quepa sin deformarse (16:9), centrada. Si la ventana tiene otra
/// proporcion (pantalla completa en 16:10), quedan franjas negras.
fn rect_imagen(ancho: i32, alto: i32) -> Rectangle {
    let escala = (ancho as f32 / RENDER_W as f32).min(alto as f32 / RENDER_H as f32);
    let (w, h) = (RENDER_W as f32 * escala, RENDER_H as f32 * escala);
    Rectangle::new((ancho as f32 - w) * 0.5, (alto as f32 - h) * 0.5, w, h)
}

/// Los letreros del jugo ("+1", "¡COMBO x10!", el cometa) donde estan en el mundo: suben y se
/// desvanecen; los grandes salen rebotando (crecen de mas y se acomodan).
fn letreros(d: &mut RaylibDrawHandle, juego: &Juego, imagen: Rectangle) {
    let (ojo, mira, ladeo) = (juego.camara.ojo, juego.camara.mira, juego.camara.arriba());
    for l in &juego.jugo.letreros {
        let Some((x, y)) = proyectar_ladeado(l.pos, ojo, mira, ladeo, imagen) else { continue };
        let u = l.avance();
        let alfa = ((1.0 - u) * 1.8).clamp(0.0, 1.0);
        let tam = if l.grande {
            // Un resorte: sale de cero, se pasa y se acomoda.
            let s = u * 1.4;
            (34.0 * (1.0 - (-s * 9.0).exp() * (s * 22.0).cos())).max(4.0) as i32
        } else {
            22
        };
        let c = Color::new((l.color[0] * 255.0).min(255.0) as u8, (l.color[1] * 255.0).min(255.0) as u8, (l.color[2] * 255.0).min(255.0) as u8, (alfa * 255.0) as u8);
        let w = ui::medir_titulo(&l.texto, tam);
        ui::titulo_de(d, &l.texto, x as i32 - w / 2, y as i32 - tam / 2, tam, c);
    }
}

/// Lo que mide de ancho el tablero de la nave (abajo a la derecha): los
/// mensajes de GLADAS se acomodan para no taparlo.
const ANCHO_TABLERO: i32 = 300;

/// El HUD del espacio: donde estas (arriba a la izquierda), que se puede
/// hacer (arriba al centro, como botones), el tablero de la nave (abajo a la
/// derecha) y, con TAB, la info del pipeline y los controles.
fn hud_espacio(d: &mut RaylibDrawHandle, juego: &Juego, e: &Entrada, st: &Estadisticas, ms: f32, info: bool, prog: &Progreso) {
    let (ancho, alto) = (d.get_screen_width(), d.get_screen_height());
    let (cerca, dist) = mas_cercano(juego.nave.pos, juego.t);

    // El velo de la izquierda (sin marcos: un degradado oscuro que se desvanece hacia el centro).
    ui::velo(d, 0, 0, 380, 330, ui::Hacia::Derecha);
    // Donde estas.
    ui::subtitulo(d, if cerca == 0 { "CERCA DEL" } else { "CERCA DE" }, 36, 26, 16, ui::TENUE);
    ui::titulo_de(d, &CUERPOS[cerca].nombre.to_uppercase(), 36, 46, 26, Color::new(240, 232, 255, 255));
    let km = format!("{:.0} km", dist.max(0.0) * 10.0);
    ui::subtitulo(d, &km, 36 + ui::medir_titulo(&CUERPOS[cerca].nombre.to_uppercase(), 26) + 14, 54, 18, ui::TENUE);
    // El titulo del pase que se lleva puesto.
    if let Some(t) = &prog.titulo {
        ui::subtitulo(d, &format!("« {} »", progreso::texto_titulo(t)), 36, 80, 15, Color::new(140, 255, 235, 255));
    }

    // Las notas recogidas (con el combo) y el pasaporte.
    // Tres renglones: el titulo (y el cometa), las notas con su combo al lado, y el pasaporte.
    ui::subtitulo(d, "NOTAS", 36, 104, 16, ui::TENUE);
    let notas = prog.notas.to_string();
    // Al recoger, el numero late (crece y se aclara un instante).
    let latido = juego.jugo.latido_notas;
    let tam = (26.0 * (1.0 + 0.3 * latido)) as i32;
    let claro = (latido * 135.0) as u8;
    ui::titulo_de(d, &notas, 36, 124 - (tam - 26) / 2, tam, Color::new(120 + claro, 230 + claro / 6, 255, 255));
    let combo = juego.coleccion.combo;
    if combo >= 2 {
        let s = format!("combo x{combo}");
        let latido = 1.0 + 0.12 * (juego.reloj * 12.0).sin().abs();
        let x = 36 + ui::medir_titulo(&notas, 26) + 16;
        ui::subtitulo(d, &s, x, 130, (20.0 * latido) as i32, Color::new(255, 150, 220, 255));
    }
    let (tiene, total) = prog.conteo();
    ui::accion(d, e.nombre("pasaporte"), &format!("Pasaporte  {tiene}/{total}"), 36, 158, 16, false);
    if juego.coleccion.cometa.is_some() {
        let s = "¡COMETA!";
        if (juego.reloj * 3.0) as i32 % 2 == 0 {
            ui::subtitulo(d, s, 200, 104, 18, Color::new(150, 230, 255, 255));
        }
    }

    // Los encargos de GLADAS: siempre hay tres, con su avance.
    ui::subtitulo(d, "ENCARGOS", 36, 200, 16, ui::TENUE);
    for (k, m) in prog.encargos.iter().enumerate() {
        let y = 222 + k as i32 * 24;
        let avance = format!("{}/{}", m.avance, m.meta);
        let w_avance = ui::medir(&avance, 16);
        let texto = ui::recortado(&m.texto(), 16, 300 - w_avance - 14);
        let hecho = m.avance >= m.meta;
        ui::subtitulo(d, &texto, 36, y, 16, if hecho { ui::VERDE } else { ui::TEXTO });
        ui::subtitulo(d, &avance, 336 - w_avance, y, 16, Color::new(120, 230, 255, 255));
    }

    ui::accion(d, e.nombre("info"), "Controles", ancho - ui::medir_accion(e.nombre("info"), "Controles", 16) - 20, 20, 16, false);

    // El tablero de la nave, abajo a la derecha: modo de vuelo, velocidad y turbo.
    let (w, h) = (ANCHO_TABLERO, 120);
    let (x, y) = (ancho - w - 20, alto - h - 20);
    ui::velo(d, ancho - w - 120, y - 30, w + 120, h + 50, ui::Hacia::Izquierda);
    let limite = nave::limite_de_velocidad(dist);
    let (estado, color) = if let Some(a) = &juego.aterrizaje {
        (a.etapa().to_string(), Color::new(255, 190, 120, 255))
    } else if juego.salto.activo() {
        ("HIPERESPACIO".to_string(), Color::new(150, 220, 255, 255))
    } else if cerca == 0 && dist < 40.0 {
        ("¡CALOR DEL SOL!".to_string(), Color::new(255, 170, 80, 255))
    } else if let Some((i, altura)) = juego.orbita {
        // Lo mas completo que quepa en el tablero (los nombres largos no caben con todo).
        let nombre = CUERPOS[i].nombre.to_uppercase();
        let opciones = [format!("MODO ÓRBITA · {nombre} · altura {altura:.1}"), format!("ÓRBITA · {nombre} · {altura:.1}"), format!("ÓRBITA · {nombre}")];
        let cabe = opciones.iter().find(|o| ui::medir(o, 18) <= w - 36).unwrap_or(&opciones[2]).clone();
        (cabe, Color::new(130, 255, 220, 255))
    } else if juego.sincronia > 0.5 {
        (format!("ÓRBITA DE {}", CUERPOS[cerca].nombre.to_uppercase()), Color::new(120, 240, 190, 255))
    } else if limite >= nave::VELOCIDAD_CRUCERO * 0.6 {
        ("CRUCERO".to_string(), Color::new(255, 200, 120, 255))
    } else {
        (format!("GRAVEDAD DE {}", CUERPOS[cerca].nombre.to_uppercase()), Color::new(255, 150, 200, 255))
    };
    // Si aun asi no cabe, la letra se achica un poco.
    let mut tam = 18;
    while tam > 12 && ui::medir(&estado, tam) > w - 36 {
        tam -= 1;
    }
    ui::subtitulo(d, &estado, x + 18, y + 12 + (18 - tam) / 2, tam, color);
    if let Some(a) = &juego.aterrizaje {
        // Aterrizando: el altimetro y la velocidad vertical, como en los juegos de naves.
        let altitud = format!("{:.0}", a.altitud(juego.nave.pos, juego.t) * 100.0);
        ui::titulo_de(d, &altitud, x + 18, y + 36, 36, ui::TEXTO);
        ui::subtitulo(d, "m", x + 30 + ui::medir_titulo(&altitud, 36), y + 52, 18, ui::TENUE);
        let vertical = format!("{:+.0} m/s", juego.nave.vel.y * 100.0);
        let color_v = if juego.nave.vel.y < -400.0 { Color::new(255, 120, 120, 255) } else { ui::TENUE };
        ui::subtitulo(d, &vertical, x + w - 18 - ui::medir(&vertical, 16), y + 54, 16, color_v);
    } else {
        let velocidad = format!("{:.0}", juego.nave.velocidad.abs() * 10.0);
        ui::titulo_de(d, &velocidad, x + 18, y + 36, 36, ui::TEXTO);
        ui::subtitulo(d, "km/s", x + 30 + ui::medir_titulo(&velocidad, 36), y + 52, 18, ui::TENUE);
        let maximo = format!("máx {:.0}", limite * 10.0);
        ui::subtitulo(d, &maximo, x + w - 18 - ui::medir(&maximo, 16), y + 54, 16, ui::TENUE);
        // La gravedad que esta curvando la trayectoria, a la derecha del estado.
        if let (None, Some((_, a))) = (juego.orbita, juego.gravedad) {
            if a > 0.05 {
                let s = format!("jalón {a:.1} m/s²");
                ui::subtitulo(d, &s, x + w - 18 - ui::medir(&s, 15), y + 14, 15, Color::new(150, 230, 255, 255));
            }
        }
    }
    // Lo que esta activo (las lunas de la mochila y la maldicion del tarot), arriba del tablero:
    // una bolita de su color con los segundos que le quedan.
    let activos = [
        (juego.turbo_io, jupiter::COLOR_LUNA[0], "Io"),
        (juego.escudo, jupiter::COLOR_LUNA[1], "Europa"),
        (juego.iman, jupiter::COLOR_LUNA[2], "Ganímedes"),
        (juego.maldicion, [0.85, 0.25, 0.45], "¡Maldición!"),
    ];
    let mut xa = x as f32 + w as f32;
    for (seg, c, nombre) in activos.iter().rev().filter(|a| a.0 > 0.0) {
        let s = format!("{nombre} {:.0}", seg.ceil());
        let ancho_chip = ui::medir(&s, 16) as f32 + 38.0;
        xa -= ancho_chip + 8.0;
        let color = Color::new((c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, 255);
        let parpadea = *seg < 5.0 && (juego.reloj * 6.0).sin() < 0.0;
        d.draw_rectangle_rounded(Rectangle::new(xa, (y - 40) as f32, ancho_chip, 30.0), 0.5, 8, Color::new(20, 12, 40, 220));
        d.draw_circle_v(Vector2::new(xa + 16.0, (y - 25) as f32), 8.0, if parpadea { Color::new(color.r, color.g, color.b, 90) } else { color });
        ui::subtitulo(d, &s, xa as i32 + 30, y - 34, 16, if *nombre == "¡Maldición!" { Color::new(255, 150, 170, 255) } else { ui::TEXTO });
    }
    // La barra de turbo (de pixel art: azul, roja cuando ya casi no queda).
    ui::subtitulo(d, "TURBO", x + 18, y + 88, 16, ui::TENUE);
    let color_turbo = if juego.nave.energia < 0.25 { ui::ColorBarra::Rojo } else { ui::ColorBarra::Azul };
    ui::barra(d, (x + 84) as f32, (y + 84) as f32, (w - 100) as f32, 22.0, juego.nave.energia, color_turbo);

    // Con TAB: la info del pipeline y los controles.
    if info {
        let vista = match (juego.opciones.modo, juego.opciones.vista) {
            (_, Vista::Profundidad) => "z-buffer",
            (_, Vista::Normales) => "normales",
            (_, Vista::Uv) => "coordenadas UV",
            (_, Vista::Stencil) => "stencil buffer",
            (ModoDibujo::Relleno, _) => "relleno",
            (ModoDibujo::Alambre, _) => "alambre",
            (ModoDibujo::Puntos, _) => "puntos",
        };
        let n = |a| e.nombre(a);
        let con_control;
        let controles: &[(&str, &str)] = if e.hay_control {
            con_control = [("stick", "volar"), (n("acelerar"), "acelerar / frenar"), (n("turbo"), "turbo"), (n("barrel"), "barrel roll"), (n("orbitar"), "modo órbita"), (n("objetivo"), "destino del salto"), (n("vista"), "vista"), (n("pasaporte"), "pasaporte"), (n("pausa"), "pausa y ajustes")];
            &con_control
        } else {
            &[
                ("W / S", "acelerar / frenar"),
                ("A / D", "girar"),
                ("arriba / abajo", "cabecear"),
                ("shift", "turbo"),
                ("Q", "barrel roll"),
                ("R", "modo órbita"),
                ("1 a 7", "vista"),
                ("T", "destino del salto"),
                ("P", "pasaporte"),
                ("ESC", "pausa y ajustes"),
                ("C", "culling"),
                ("V", "descarte de objetos"),
                ("F5 / F6", "semilla del planeta de junto"),
                ("F7 / F8", "relieve del planeta de junto"),
                ("F9 / F10", "intensidad de shaders"),
                ("H", "ocultar HUD"),
            ]
        };
        let descarte = pipeline::DESCARTE.load(std::sync::atomic::Ordering::Relaxed);
        let lineas = [
            format!("{:.0} FPS   {ms:.1} ms por cuadro", 1000.0 / ms.max(0.1)),
            format!("{} triángulos procesados, {} dibujados", st.triangulos, st.dibujados),
            format!("objetos: {}  fuera de vista: {}  lejos: {}", st.objetos, st.descartados, st.por_detalle),
            format!("descarte de objetos: {}", if descarte { "sí" } else { "NO (comparando)" }),
            format!("vista: {vista}   culling: {}", if juego.opciones.culling { "sí" } else { "no" }),
            panel_planeta(juego),
        ];
        // A la derecha, debajo de la barra de acciones de arriba (y de "TAB: Controles").
        let arriba = 76;
        // Si no cabe en una columna (ventana chiquita), los controles van en dos.
        let alto_lineas = 52 + lineas.len() as i32 * 24;
        let columnas = if arriba + alto_lineas + controles.len() as i32 * 30 > alto - 20 { 2 } else { 1 };
        let filas = (controles.len() as i32 + columnas - 1) / columnas;
        // Tan ancho como la linea mas larga (los numeros cambian: que no se salgan del panel).
        let ancho_lineas = lineas.iter().map(|l| ui::medir(l, 17)).max().unwrap_or(0) + 32;
        let (w, h) = ((340 * columnas).max(ancho_lineas), alto_lineas + filas * 30);
        let x = ancho - w - 20;
        ui::panel(d, x, arriba, w, h);
        ui::texto(d, "PIPELINE", x + 16, arriba + 12, 16, ui::LILA);
        for (k, l) in lineas.iter().enumerate() {
            ui::texto(d, l, x + 16, arriba + 34 + k as i32 * 24, 17, ui::TEXTO);
        }
        let y0 = arriba + 42 + lineas.len() as i32 * 24;
        for (k, (b, que)) in controles.iter().enumerate() {
            let (col, fila) = (k as i32 / filas, k as i32 % filas);
            ui::accion(d, b, que, x + 16 + col * 340, y0 + fila * 30, 16, false);
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// La demo hace el recorrido completo con el juego de verdad (gravedad, sincronia y orbita).
    #[test]
    fn la_demo_hace_el_recorrido_con_el_juego_de_verdad() {
        let (pos, rumbo) = demo::inicio(0.0);
        let mut juego = Juego::new(pos, rumbo);
        let mut dm = demo::Demo::new();
        let mut fases = vec![dm.fase];
        let mut vuelta_en_orbita = 0;
        // Como en el juego: `actualizar` avanza el reloj del mundo (una sola vez por cuadro).
        let mut lejos = 0.0f32;
        for _ in 0..(60 * 240) {
            let e = dm.entrada(&juego.nave, juego.orbita.is_some(), juego.t, 1.0 / 60.0);
            juego.actualizar(&e, 1.0 / 60.0);
            vuelta_en_orbita += juego.orbita.is_some() as usize;
            lejos = lejos.max(juego.nave.pos.length());
            if fases.last() != Some(&dm.fase) {
                fases.push(dm.fase);
                eprintln!("demo: {:.1} s -> {:?}", juego.t, dm.fase);
            }
            if dm.fase == demo::Fase::Fin {
                break;
            }
        }
        assert_eq!(*fases.last().unwrap(), demo::Fase::Fin, "se quedo en {fases:?}");
        assert!(vuelta_en_orbita > 60 * 5, "casi no estuvo en orbita: {vuelta_en_orbita} cuadros");
        // Sin salirse del sistema solar (el rescate la regresaria de golpe en pleno video).
        assert!(lejos < ORILLA_SISTEMA, "se fue hasta {lejos}");
    }

    /// Los minijuegos pagan al despegar segun como se jugo; sin jugar, nada.
    #[test]
    fn los_minijuegos_pagan_segun_como_se_jugo() {
        let mut u = Urano::new(7);
        assert!(pago_de_escena(&Escena::Urano(Box::new(Urano::new(7)))).is_none(), "sin jugar no se paga");
        u.partidas = 1;
        let mut un_dia = Urano::new(7);
        un_dia.partidas = 1;
        let (uno, _) = pago_de_escena(&Escena::Urano(Box::new(un_dia))).unwrap();
        u.partidas = 3;
        u.dias_ganados = 3;
        u.campeon = true;
        let (campeon, motivo) = pago_de_escena(&Escena::Urano(Box::new(u))).unwrap();
        assert!(campeon > uno * 3, "ganar los tres dias paga mucho mas: {uno} vs {campeon}");
        assert!(motivo.contains("3 días"));
        let mut n = Neptuno::new(3);
        assert!(pago_de_escena(&Escena::Neptuno(Neptuno::new(3))).is_none());
        n.ganador = Some(neptuno::MIKU);
        let (gano, _) = pago_de_escena(&Escena::Neptuno(n)).unwrap();
        let mut n = Neptuno::new(3);
        n.ganador = Some(1);
        let (perdio, _) = pago_de_escena(&Escena::Neptuno(n)).unwrap();
        assert!(gano > perdio && perdio > 0);
    }

    /// La gravedad curva la trayectoria: una nave quieta junto al Chapin termina mas cerca de el,
    /// y lejos de todo no le pasa nada.
    #[test]
    fn la_gravedad_jala_hacia_el_planeta() {
        let t = 10.0;
        let c = &CUERPOS[CHAPIN];
        let inicio = c.posicion(t) + Vec3::new(c.radio_cuerpo() + 4.0, 0.0, 0.0);
        let mut juego = Juego::new(inicio, 0.0);
        juego.t = t;
        juego.sincronia = 1.0;
        let antes = (juego.nave.pos - c.posicion(juego.t)).length();
        for _ in 0..120 {
            juego.actualizar(&Entrada::default(), 1.0 / 60.0);
        }
        let despues = (juego.nave.pos - c.posicion(juego.t)).length();
        assert!(despues < antes - 0.3, "no lo jalo: {antes} -> {despues}");
        assert_eq!(juego.gravedad.map(|g| g.0), Some(CHAPIN));
    }

    /// En modo orbita la nave le da la vuelta al planeta a la misma altura, y R lo apaga.
    #[test]
    fn el_modo_orbita_sigue_la_curva_a_la_misma_altura() {
        let t = 10.0;
        let c = &CUERPOS[CHAPIN];
        let inicio = c.posicion(t) + Vec3::new(c.radio_cuerpo() + 5.0, 0.0, 0.0);
        // Mirando hacia -Z: tangente a la superficie en ese punto.
        let mut juego = Juego::new(inicio, 0.0);
        juego.t = t;
        juego.sincronia = 1.0;
        juego.actualizar(&Entrada { orbitar: true, ..Default::default() }, 1.0 / 60.0);
        let (planeta, altura) = juego.orbita.expect("no entro al modo orbita");
        assert_eq!(planeta, CHAPIN);
        // Lo que va rodeando, sumado cuadro a cuadro (puede dar varias vueltas).
        let mut antes = (juego.nave.pos - c.posicion(juego.t)).normalize();
        let mut recorrido = 0.0;
        for _ in 0..600 {
            juego.actualizar(&Entrada { empuje: 1.0, ..Default::default() }, 1.0 / 60.0);
            let d = (juego.nave.pos - c.posicion(juego.t)).length() - c.radio_cuerpo();
            assert!((d - altura).abs() < 0.05, "se salio de la altura: {d} en vez de {altura}");
            let ahora = (juego.nave.pos - c.posicion(juego.t)).normalize();
            recorrido += ahora.dot(antes).clamp(-1.0, 1.0).acos();
            antes = ahora;
        }
        assert!(recorrido > 1.0, "no avanzo alrededor del planeta: {recorrido} radianes");
        juego.actualizar(&Entrada { orbitar: true, ..Default::default() }, 1.0 / 60.0);
        assert!(juego.orbita.is_none());
    }

    /// Junto a la Tierra siempre se puede aterrizar en ella, este donde este la Luna.
    #[test]
    fn se_aterriza_en_la_tierra_aunque_la_luna_este_cerca() {
        for k in 0..400 {
            let t = k as f32 * 0.5;
            let tierra = CUERPOS[TIERRA].posicion(t);
            for dir in [Vec3::new(1.0, 0.2, 0.0), Vec3::new(0.0, 0.3, 1.0), Vec3::new(-0.7, 0.1, -0.7)] {
                let p = tierra + dir.normalize() * 8.0;
                assert_eq!(aterrizable(p, t), Some(TIERRA), "t = {t}");
            }
        }
    }

    /// Apretar E junto a la Tierra baja la nave y entra al salon, tambien justo despues de un salto.
    #[test]
    fn apretar_e_junto_a_la_tierra_aterriza() {
        let e = Entrada { aterrizar: true, ..Default::default() };
        let nada = Entrada::default();
        // Volando cerca.
        let mut juego = Juego::new(SALIDA_PRUEBA, 0.0);
        juego.t = 30.0;
        juego.nave.pos = CUERPOS[TIERRA].posicion(30.0) + Vec3::new(6.0, 1.0, 0.0);
        let mut aterrizo = juego.actualizar(&e, 1.0 / 60.0);
        for _ in 0..400 {
            aterrizo = aterrizo.or(juego.actualizar(&nada, 1.0 / 60.0));
        }
        assert_eq!(aterrizo, Some(TIERRA));
        // Llegando con el hiperespacio desde el Sol.
        let mut juego = Juego::new(SALIDA_PRUEBA, 0.0);
        juego.salto.iniciar(&juego.nave, TIERRA);
        let mut aterrizo = None;
        for _ in 0..900 {
            let entrada = if juego.salto.activo() || juego.aterrizaje.is_some() { &nada } else { &e };
            aterrizo = aterrizo.or(juego.actualizar(entrada, 1.0 / 60.0));
        }
        assert_eq!(aterrizo, Some(TIERRA));
    }

    /// Irse lejisimos: primero jala de regreso, y si se sigue, la estrella de rescate la trae de
    /// vuelta junto a un planeta (como en Mario Galaxy).
    #[test]
    fn irse_muy_lejos_trae_la_nave_de_regreso() {
        let acelerar = Entrada { empuje: 1.0, ..Default::default() };
        let mut juego = Juego::new(SALIDA_PRUEBA, 0.0);
        juego.t = 40.0;
        let sol = CUERPOS[0].posicion(40.0);
        juego.nave.pos = sol + Vec3::new(0.0, 0.0, FIN_SISTEMA + 5.0);
        juego.actualizar(&acelerar, 1.0 / 60.0);
        assert!(juego.rescate.is_some(), "empieza el rescate");
        for _ in 0..(RESCATE * 60.0) as usize + 10 {
            juego.actualizar(&Entrada::default(), 1.0 / 60.0);
        }
        assert!(juego.rescate.is_none());
        assert!((juego.nave.pos - CUERPOS[0].posicion(juego.t)).length() < ORILLA_SISTEMA, "quedo adentro del sistema");
    }

    /// En Urano la nave se posa y, al terminar la cutscene, abre el baño publico (su escena).
    #[test]
    fn aterrizar_en_urano_abre_el_bano() {
        let e = Entrada { aterrizar: true, ..Default::default() };
        let nada = Entrada::default();
        let mut juego = Juego::new(SALIDA_PRUEBA, 0.0);
        juego.t = 40.0;
        juego.nave.pos = CUERPOS[URANO].posicion(40.0) + Vec3::new(CUERPOS[URANO].radio_visual() + 4.0, 1.0, 0.0);
        let mut aterrizo = juego.actualizar(&e, 1.0 / 60.0);
        for _ in 0..900 {
            aterrizo = aterrizo.or(juego.actualizar(&nada, 1.0 / 60.0));
        }
        assert_eq!(aterrizo, Some(URANO));
    }

    /// Al posarse en cada planeta (y en la Luna) se entra a su escena.
    #[test]
    fn aterrizar_abre_la_escena_de_cada_planeta() {
        let e = Entrada { aterrizar: true, ..Default::default() };
        let nada = Entrada::default();
        for planeta in [MERCURIO, VENUS, LUNA, PLUTON, MARTE, NEPTUNO] {
            let mut juego = Juego::new(SALIDA_PRUEBA, 0.0);
            juego.t = 40.0;
            juego.nave.pos = CUERPOS[planeta].posicion(40.0) + Vec3::new(CUERPOS[planeta].radio_visual() + alcance_de_aterrizaje(planeta).min(5.0) * 0.6, 1.0, 0.0);
            let mut escena = juego.actualizar(&e, 1.0 / 60.0);
            for _ in 0..400 {
                escena = escena.or(juego.actualizar(&nada, 1.0 / 60.0));
            }
            assert_eq!(escena, Some(planeta), "deberia entrar a la escena de {}", CUERPOS[planeta].nombre);
        }
    }

    /// Crucero y mass lock: lejos de todo se puede ir muy rapido; junto a un planeta, no.
    #[test]
    fn la_velocidad_maxima_depende_de_la_distancia() {
        let acelerar = Entrada { empuje: 1.0, ..Default::default() };
        let mut lejos = Juego::new(Vec3::new(0.0, 250.0, 0.0), 0.0);
        for _ in 0..300 {
            lejos.actualizar(&acelerar, 1.0 / 60.0);
        }
        assert!(lejos.nave.velocidad > 100.0, "en crucero iba a {}", lejos.nave.velocidad);
        let jupiter = CUERPOS.iter().position(|c| c.archivo == "jupiter").unwrap();
        let mut cerca = Juego::new(CUERPOS[jupiter].posicion(0.0) + Vec3::new(0.0, CUERPOS[jupiter].radio_visual() + 3.0, 0.0), 0.0);
        for _ in 0..120 {
            cerca.actualizar(&acelerar, 1.0 / 60.0);
        }
        assert!(cerca.nave.velocidad < lejos.nave.velocidad * 0.3, "junto a Jupiter iba a {}", cerca.nave.velocidad);
    }

    /// Sin tocar nada, cerca de la Tierra la nave se queda junto a ella aunque la Tierra orbite.
    #[test]
    fn la_nave_sigue_la_orbita_del_planeta_cercano() {
        let nada = Entrada::default();
        let lado = Vec3::new(0.0, CUERPOS[TIERRA].radio_visual() + 3.0, 0.0);
        let mut juego = Juego::new(CUERPOS[TIERRA].posicion(0.0) + lado, 0.0);
        // Diez segundos (las orbitas son lentas: en menos casi no se mueve).
        for _ in 0..600 {
            juego.actualizar(&nada, 1.0 / 60.0);
        }
        let tierra = CUERPOS[TIERRA].posicion(juego.t);
        let se_movio = (CUERPOS[TIERRA].posicion(0.0) - tierra).length();
        let deriva = (juego.nave.pos - (tierra + lado)).length();
        assert!(se_movio > 15.0, "la Tierra casi no se movio: {se_movio}");
        assert!(deriva < 3.0, "la nave se quedo atras: {deriva}");
    }

    const SALIDA_PRUEBA: Vec3 = Vec3::new(0.0, 3.0, 26.0);

    /// E lejos de todo salta solo al planeta mas cercano y se posa en el.
    #[test]
    fn e_lejos_salta_y_aterriza() {
        let e = Entrada { aterrizar: true, ..Default::default() };
        let nada = Entrada::default();
        for inicio in [Vec3::new(0.0, 3.0, 26.0), Vec3::new(80.0, 5.0, -40.0)] {
            let mut juego = Juego::new(inicio, 0.0);
            juego.t = 12.0;
            juego.actualizar(&e, 1.0 / 60.0);
            for _ in 0..600 {
                if juego.actualizar(&nada, 1.0 / 60.0).is_some() {
                    break;
                }
            }
            assert!(juego.aterrizaje.as_ref().is_some_and(|a| a.posado()), "desde {inicio:?}");
        }
    }

    /// Volando derecho hacia la Tierra, la nave rebota y nunca queda adentro.
    #[test]
    fn la_nave_no_atraviesa_planetas() {
        let mut juego = Juego::new(Vec3::zero(), 0.0);
        juego.t = 5.0;
        let tierra = CUERPOS[TIERRA].posicion(juego.t);
        juego.nave.pos = tierra + Vec3::new(0.0, 0.0, 6.0);
        let acelerar = Entrada { empuje: 1.0, ..Default::default() };
        for _ in 0..240 {
            juego.actualizar(&acelerar, 1.0 / 60.0);
            let d = (juego.nave.pos - CUERPOS[TIERRA].posicion(juego.t)).length();
            assert!(d >= CUERPOS[TIERRA].radio_cuerpo() + colision::RADIO_NAVE - 0.01, "quedo adentro: {d}");
        }
    }

    /// La cancion dura lo que dice y cada nota del minijuego cae en una corchea.
    #[test]
    fn la_cancion_y_sus_notas_cuadran() {
        let pista = cancion::sintetizar();
        assert!(pista.len() as f32 >= cancion::DURACION * cancion::FRECUENCIA as f32);
        assert!(pista.iter().all(|m| m.is_finite() && m.abs() <= 1.0));
        let notas = cancion::notas();
        assert!(notas.len() > 40);
        for (t, _) in notas {
            let corcheas = t / cancion::CORCHEA;
            assert!((corcheas - corcheas.round()).abs() < 1e-3);
            assert!(t >= 4.0, "las notas empiezan despues de la cuenta regresiva");
        }
    }

    /// El dialogo muestra el texto de a poco, sin partir letras con tilde.
    #[test]
    fn el_dialogo_aparece_letra_por_letra() {
        let mut d = Dialogo::new();
        let texto = "¡Hola, ñandú!";
        let a = d.mostrar(texto, Voz::Miku, 0.1, None).to_string();
        assert!(a.len() < texto.len() && texto.starts_with(&a));
        let b = d.mostrar(texto, Voz::Miku, 5.0, None).to_string();
        assert_eq!(b, texto);
    }

    /// Jugando perfecto, Saturno Ritmo da nota S y ningun fallo.
    #[test]
    fn jugar_perfecto_da_nota_s() {
        let mut r = Ritmo::new();
        r.empezar(ritmo::Dificultad::Dificil);
        let notas = ritmo::notas_del_juego(r.dificultad);
        let paso = 1.0 / 240.0;
        let mut ahora = 0.0f32;
        while ahora < cancion::DURACION + 1.0 {
            let toca = |aire: bool| notas.iter().any(|&(tn, a)| a == aire && tn > ahora && tn <= ahora + paso);
            r.actualizar(&Entrada { piso: toca(false), aire: toca(true), ..Default::default() }, paso);
            ahora += paso;
        }
        assert!(r.ia().starts_with("Nota S"), "{}", r.ia());
    }
}

#[cfg(test)]
mod estres {
    //! Botones al azar en todos los minijuegos, miles de cuadros y con cuadros chicos y grandes:
    //! nada debe tronar. No revisa que se gane: revisa que ninguna combinacion rompa el juego.
    use super::*;
    use crate::entrada::Entrada;

    struct Azar(u32);
    impl Azar {
        fn f(&mut self) -> f32 {
            self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (self.0 >> 8) as f32 / 16_777_216.0
        }
        fn b(&mut self, p: f32) -> bool {
            self.f() < p
        }
    }

    /// Una entrada al azar. `aterrizar` y `pausa` casi nunca (no sacan del juego en estas pruebas).
    fn entrada(a: &mut Azar) -> Entrada {
        Entrada {
            girar: a.f() * 2.0 - 1.0,
            cabecear: a.f() * 2.0 - 1.0,
            empuje: a.f() * 2.0 - 1.0,
            horizontal: a.f() * 2.0 - 1.0,
            aceptar: a.b(0.25),
            arriba: a.b(0.08),
            abajo: a.b(0.08),
            izquierda: a.b(0.1),
            derecha: a.b(0.1),
            barrel: a.b(0.05),
            disparando: a.b(0.3),
            gatillo: a.b(0.1),
            clic: a.b(0.1),
            raton_apretado: a.b(0.2),
            piso: a.b(0.2),
            aire: a.b(0.2),
            picar: a.b(0.3),
            turbo: a.b(0.1),
            respuesta: if a.b(0.08) { Some((a.f() * 6.0) as usize) } else { None },
            ..Default::default()
        }
    }

    fn dt(a: &mut Azar) -> f32 {
        // Cuadros normales, y a veces un tiron de medio segundo.
        if a.b(0.01) { 0.5 } else { 1.0 / 30.0 + a.f() * 0.03 }
    }

    macro_rules! estres {
        ($nombre:ident, $crear:expr) => {
            #[test]
            fn $nombre() {
                let mut total = 0.0;
                for semilla in [1u32, 77, 2026] {
                    let mut a = Azar(semilla.wrapping_mul(2_654_435_761) | 1);
                    let mut juego = $crear(semilla);
                    for _ in 0..6000 {
                        let e = entrada(&mut a);
                        let d = dt(&mut a);
                        let _ = juego.actualizar(&e, d);
                        juego.eventos.clear();
                        total += d;
                    }
                }
                // Que de verdad haya corrido el tiempo (si no, la prueba no probo nada).
                assert!(total > 3.0 * 6000.0 / 30.0, "corrio {total} s");
                eprintln!("{}: {total:.0} s de juego", stringify!($nombre));
            }
        };
    }

    /// F1 y F2: al cambiar el radio de un planeta, la colision se mueve con el dibujo.
    #[test]
    fn el_radio_en_vivo_mueve_la_colision() {
        use crate::escena::{factor_radio, poner_factor_radio};
        let c = &CUERPOS[PLUTON];
        let antes = c.radio_visual();
        poner_factor_radio(PLUTON, 2.0);
        assert!((c.radio_visual() - antes * 2.0).abs() < 1e-4);
        assert!((c.radio_cuerpo() - c.escala * 2.0).abs() < 1e-4);
        poner_factor_radio(PLUTON, 1.0);
        assert_eq!(factor_radio(PLUTON), 1.0);
    }

    estres!(camioneta, Camioneta::new);
    estres!(neptuno, Neptuno::new);
    estres!(urano, Urano::new);
    estres!(vegetta, |s| Vegetta::new(s, None));
    estres!(jupiter, |s| Jupiter::new(s, 0));
    estres!(ritmo, |_| Ritmo::new());
    estres!(divorcio, Divorcio::new);
    estres!(mercurio, Mercurio::new);
    estres!(luna, Luna::new);
    estres!(cita, |_| Cita::new());
    estres!(junta, |_| Junta::new());
    estres!(aula, Aula::new);
    estres!(entrega, Entrega::new);
    estres!(historia, |_| Historia::new());
}
