//! Las partes que se mueven solas: las llamas, las turbinas, la estrella
//! de la antena, la helice de la colita y las coletas de Miku.
//!
//! Cada una lleva en su comentario del `.obj` su pivote o desde donde
//! crece (los escriben los generadores de `herramientas/`). Aca se leen una
//! vez al cargar, y cada cuadro se arma una matriz extra por parte que va
//! entre la M del modelo y el vertice: `M * animacion * v`.

use crate::matematica::{Mat4, Vec3};
use crate::obj::Modelo;

/// Lo que cambia cuadro a cuadro y mueve las partes.
#[derive(Clone, Copy, Default)]
pub struct EstadoAnim {
    pub t: f32,
    /// Angulo acumulado de las turbinas (giran mas rapido a mas velocidad).
    pub giro_turbinas: f32,
    /// Que tanto empujan los motores, de 0 a 1 (mas en el hiperespacio).
    pub empuje: f32,
    /// Hacia donde se esta girando, de -1 (derecha) a 1 (izquierda).
    pub viraje: f32,
    /// El golpe del ritmo: 1 en cada beat y se apaga hasta el siguiente.
    pub ritmo: f32,
}

enum Anim {
    /// Gira alrededor de un eje que pasa por el pivote.
    Girar { pivote: Vec3, eje: char, rapidez: f32, por_motor: bool },
    /// Se estira a lo largo de un eje desde un plano (las llamas).
    Estirar { desde: f32, eje: char, fase: f32, minimo: f32 },
    /// Se mece con el viento desde la liga.
    Coleta { pivote: Vec3, lado: f32 },
    /// Barra del ecualizador de Saturno: crece en Y desde su base con el ritmo.
    Barra { desde: f32, fase: f32 },
}

pub struct Animaciones {
    partes: Vec<Option<Anim>>,
}

/// Los numeros que aparecen en un comentario, en orden.
fn numeros(texto: &str) -> Vec<f32> {
    texto.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).filter_map(|s| s.parse().ok()).collect()
}

fn punto(texto: &str) -> Vec3 {
    match numeros(texto)[..] {
        [x, y, z, ..] => Vec3::new(x, y, z),
        _ => Vec3::zero(),
    }
}

impl Animaciones {
    /// Sin nada que se mueva solo (los personajes de pie: los mueve el baile).
    pub fn ninguna() -> Self {
        Animaciones { partes: Vec::new() }
    }

    pub fn de(modelo: &Modelo) -> Self {
        let comentario = |nombre: &str| {
            modelo.partes.iter().find(|p| p.nombre == nombre).map_or("", |p| p.comentario.as_str())
        };
        let partes = modelo
            .partes
            .iter()
            .map(|p| {
                let n = p.nombre.as_str();
                let c = p.comentario.as_str();
                let lado = if n.contains("_der") { 1.0 } else { -1.0 };
                if n.ends_with("_aspas") {
                    Some(Anim::Girar { pivote: punto(c), eje: 'x', rapidez: 1.0, por_motor: true })
                } else if n == "antena_estrella" {
                    Some(Anim::Girar { pivote: punto(c), eje: 'y', rapidez: 2.0, por_motor: false })
                } else if n == "colita_helice" {
                    Some(Anim::Girar { pivote: punto(c), eje: 'z', rapidez: 1.4, por_motor: true })
                } else if n.starts_with("llama_") {
                    let desde = numeros(c).first().copied().unwrap_or(0.0);
                    Some(Anim::Estirar { desde, eje: 'z', fase: lado * 1.7, minimo: 0.35 })
                } else if n.starts_with("bota_") && n.ends_with("_llama") {
                    let desde = numeros(c).first().copied().unwrap_or(0.0);
                    Some(Anim::Estirar { desde, eje: 'y', fase: lado * 2.9, minimo: 0.6 })
                } else if n.starts_with("vinil_") {
                    // Los anillos de Saturno giran como un disco.
                    Some(Anim::Girar { pivote: Vec3::zero(), eje: 'y', rapidez: 0.5, por_motor: false })
                } else if n.contains("_helicez") {
                    // Helices de los adornos (Bombardiro): giran con el motor, en su pivote.
                    Some(Anim::Girar { pivote: punto(c), eje: 'z', rapidez: 1.6, por_motor: true })
                } else if n.contains("_giray") {
                    // Lo que gira sobre su eje vertical (la bola disco).
                    Some(Anim::Girar { pivote: Vec3::zero(), eje: 'y', rapidez: 1.2, por_motor: false })
                } else if n.contains("_gira") {
                    // Los adornos que giran sobre el eje de vuelo (el anillo de la Endurance).
                    Some(Anim::Girar { pivote: Vec3::zero(), eje: 'z', rapidez: 0.6, por_motor: false })
                } else if n.starts_with("nota_") {
                    // Los bichitos-nota avanzan por el anillo hacia la puerta.
                    Some(Anim::Girar { pivote: Vec3::zero(), eje: 'y', rapidez: 0.3, por_motor: false })
                } else if n.starts_with("eq_") {
                    let indice: f32 = n.rsplit('_').next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    let desde = numeros(c).first().copied().unwrap_or(0.0);
                    Some(Anim::Barra { desde, fase: indice * 0.9 + if n.contains("sur") { 2.0 } else { 0.0 } })
                } else if n.starts_with("coleta_") {
                    // Las puntas usan el pivote de su coleta.
                    let base = if lado > 0.0 { "coleta_der" } else { "coleta_izq" };
                    Some(Anim::Coleta { pivote: punto(comentario(base)), lado })
                } else {
                    None
                }
            })
            .collect();
        Animaciones { partes }
    }

    /// La matriz extra de la parte `i`, o `None` si no se mueve.
    pub fn matriz(&self, i: usize, e: &EstadoAnim) -> Option<Mat4> {
        let alrededor = |pivote: Vec3, r: Mat4| Mat4::translation(pivote) * r * Mat4::translation(-pivote);
        Some(match self.partes.get(i)?.as_ref()? {
            Anim::Girar { pivote, eje, rapidez, por_motor } => {
                let a = if *por_motor { e.giro_turbinas } else { e.t } * rapidez;
                let r = match eje {
                    'x' => Mat4::rotation_x(a),
                    'y' => Mat4::rotation_y(a),
                    _ => Mat4::rotation_z(a),
                };
                alrededor(*pivote, r)
            }
            Anim::Estirar { desde, eje, fase, minimo } => {
                // Crece con el empuje y parpadea rapido, como fuego.
                let f = (minimo + (1.0 - minimo) * e.empuje * 1.4) * (1.0 + 0.16 * (e.t * 41.0 + fase).sin());
                let (escala, p) = match eje {
                    'y' => (Vec3::new(1.0, f, 1.0), Vec3::new(0.0, *desde, 0.0)),
                    _ => (Vec3::new(1.0, 1.0, f), Vec3::new(0.0, 0.0, *desde)),
                };
                alrededor(p, Mat4::scale_xyz(escala))
            }
            Anim::Barra { desde, fase } => {
                // Cada barra a su propio paso, todas golpean juntas en el beat.
                let altura = 0.5 + 0.35 * (e.t * 2.7 + fase).sin().abs() + 0.45 * e.ritmo;
                let p = Vec3::new(0.0, *desde, 0.0);
                alrededor(p, Mat4::scale_xyz(Vec3::new(1.0, altura, 1.0)))
            }
            Anim::Coleta { pivote, lado } => {
                // Hacia atras con la velocidad, hacia afuera de la curva al girar,
                // y un vaiven suave siempre.
                let atras = -(0.12 + 0.35 * e.empuje.min(1.0)) - 0.05 * (e.t * 3.1 + lado).sin();
                let afuera = e.viraje * 0.35 + 0.04 * (e.t * 2.3 + lado * 1.3).sin();
                alrededor(*pivote, Mat4::rotation_z(afuera) * Mat4::rotation_x(atras))
            }
        })
    }
}
