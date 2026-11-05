//! LOS PLANETAS SE ARMAN EN EL JUEGO. Ninguno viene de un `.obj` hecho antes de correr: igual que
//! el Planeta Chapin (ver `chapin.rs`), cada cuerpo del sistema solar sale de una RECETA, al
//! arrancar, y pasa por las mismas etapas antes de rasterizar:
//!
//! 1. **Deformacion con ruido**: el radio de la superficie en cada direccion `d` es
//!    `1 + forma(d) + amplitud * calma(d) * fBm_Perlin(d * frecuencia)`. La `forma` es lo propio
//!    de cada planeta (los crateres de Mercurio, los continentes de la Tierra, los mares de la
//!    Luna); el fBm de Perlin (`ruido.rs`) le pone montanas, dunas o burbujas a todos, cada uno con
//!    su amplitud, su frecuencia y su semilla. `calma` deja quieta la superficie donde no debe
//!    moverse (las caras de Neptuno y Pluton, la cueva del Hada, el rover de Marte, el mar).
//! 2. **Generacion de geometria**: una rejilla de voxels alrededor de la esfera; cada celda se
//!    llena si queda adentro del radio deformado, con el material que le toque segun donde esta
//!    (las franjas de Jupiter, el corazon de Pluton...). Despues los extras: el Monte Olimpo y el
//!    rover, las nubes de la Tierra, el anillo de Urano, los hongos del Hada, las prominencias del
//!    Sol. De cada voxel solo salen las caras que dan al aire, y las vecinas iguales se juntan en
//!    rectangulos (greedy meshing).
//! 3. **Niveles de detalle**: cada planeta se arma en cinco niveles, del ligero (voxels grandes,
//!    para de lejos) al mas fino (para pasar rozando), y se dibuja el que toca segun que tan grande
//!    se ve (`nivel_para`, el mismo criterio del Chapin).
//! 4. **En vivo**: junto a cualquier planeta, F5/F6 le cambian la semilla del ruido y F7/F8 lo que
//!    sube el relieve (`Ajuste`): se vuelve a armar ahi mismo (con relieve 0 queda una esfera de
//!    voxels lisa: la generacion sin la deformacion). Las colisiones usan el relieve de ese momento.
//!
//! Lo que NO es el planeta sigue siendo modelo: las caras de Neptuno y Pluton (calcomanias con
//! sus expresiones) y, en Saturno, el escenario de Saturno Ritmo (los anillos de vinil, las notas,
//! las bocinas). Las recetas son las de antes (`herramientas/generar_planetas.py`), pasadas aqui.

use crate::matematica::Vec3;
use crate::obj::{Material, Modelo, Parte, Vertice};
use crate::ruido::Perlin;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use std::sync::{OnceLock, RwLock};

/// Lo que se le cambia en vivo a un planeta (F5 a F8, junto a el).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ajuste {
    /// Se suma a la semilla del ruido de la receta: otra semilla, otro relieve.
    pub semilla: u32,
    /// Multiplica todo el relieve: 1 el de la receta, 0 una esfera lisa, 3 el triple de montanas.
    pub relieve: f32,
}

impl Default for Ajuste {
    fn default() -> Self {
        Ajuste { semilla: 0, relieve: 1.0 }
    }
}

/// Cuantas veces el diametro de la receta lleva cada nivel de detalle (del ligero al mas fino).
const NIVELES: [f32; 5] = [0.75, 1.0, 1.5, 2.5, 3.25];

/// Que nivel de detalle toca si el planeta mide `radio_px` pixeles de radio en pantalla (para
/// todos los planetas, el Chapin incluido).
pub fn nivel_para(radio_px: f32) -> usize {
    // (El nivel 3 entra donde antes entraba el modelo detallado: a los 70 pixeles.)
    match radio_px {
        r if r < 25.0 => 0,
        r if r < 45.0 => 1,
        r if r < 70.0 => 2,
        r if r < 700.0 => 3,
        // Pegado al suelo: los voxels mas chiquitos.
        _ => 4,
    }
}

/// Lo que mide un voxel en el nivel `n` del planeta `archivo` (en el espacio del modelo, donde el
/// radio es 1). `None` si no se arma con receta o no es solo voxels de un color por material.
pub fn celda_de(archivo: &str, n: usize) -> Option<f32> {
    let r = RECETAS.iter().find(|r| r.archivo == archivo)?;
    Some(2.0 / lado_del_nivel(r.diametro, n.min(NIVELES.len() - 1)) as f32)
}

/// Los voxels de lado a lado del nivel `n` de algo que en el nivel normal lleva `diametro`.
fn lado_del_nivel(diametro: i32, n: usize) -> i32 {
    ((diametro as f32 * NIVELES[n] / 2.0).round() as i32 * 2).max(4)
}

type Celda = (i32, i32, i32);

/// Un material: nombre, color, emisivo (si brilla solo) y opacidad.
type Mat = (&'static str, [f32; 3], Option<[f32; 3]>, f32);

/// Manchas suaves y deterministas (suma de senos), las de los dibujos de cada planeta: mas o
/// menos en [-1, 1]. `s` es la semilla.
fn manchas(x: f32, y: f32, z: f32, s: f32) -> f32 {
    (3.1 * x + 1.7 * s).sin() * (2.6 * y + 0.9 * s).sin() * (2.9 * z + 2.3 * s).sin() + 0.5 * (5.3 * x + 4.1 * z + s).sin() * (4.7 * y + 1.3 * s).sin()
}

/// `manchas` en el punto `p` escalado por `k`.
fn mancha(p: Vec3, k: f32, s: f32) -> f32 {
    manchas(k * p.x, k * p.y, k * p.z, s)
}

/// Longitud (0 hacia -Z, positiva hacia +X) y latitud, en radianes.
fn lon_lat(p: Vec3) -> (f32, f32) {
    let r = p.length().max(1e-6);
    (p.x.atan2(-p.z), (p.y / r).clamp(-1.0, 1.0).asin())
}

fn paso_suave(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Los voxels de un cuerpo mientras se arma: cada celda guarda su parte (y cada parte, su material).
pub struct Voxels {
    pub u: f32,
    celdas: HashMap<Celda, u16>,
    partes: Vec<(String, usize)>,
    materiales: Vec<Material>,
}

impl Voxels {
    /// `brillo`: el especular y el exponente de los materiales que no brillan solos.
    fn new(u: f32, materiales: &[Mat], brillo: (f32, f32)) -> Self {
        let materiales = materiales
            .iter()
            .map(|&(nombre, kd, ke, d)| {
                let mut m = Material::nuevo(nombre);
                m.kd = Vec3::new(kd[0], kd[1], kd[2]);
                // Lo que brilla solo no lleva brillo especular (se veria como rayas blancas).
                let ks = if ke.is_some() { 0.0 } else { brillo.0 };
                m.ks = Vec3::new(ks, ks, ks);
                m.ns = brillo.1;
                m.ke = ke.map_or(Vec3::zero(), |c| Vec3::new(c[0], c[1], c[2]));
                m.d = d;
                m
            })
            .collect();
        Voxels { u, celdas: HashMap::new(), partes: Vec::new(), materiales }
    }

    fn parte(&mut self, parte: &str, mat: &str) -> u16 {
        if let Some(k) = self.partes.iter().position(|(n, _)| n == parte) {
            return k as u16;
        }
        let m = self.materiales.iter().position(|m| m.nombre == mat).unwrap_or_else(|| panic!("material {mat} sin registrar"));
        self.partes.push((parte.to_string(), m));
        (self.partes.len() - 1) as u16
    }

    fn put(&mut self, c: Celda, parte: &str, mat: &str) {
        let k = self.parte(parte, mat);
        self.celdas.insert(c, k);
    }

    fn centro(&self, (i, j, k): Celda) -> Vec3 {
        Vec3::new(i as f32 + 0.5, j as f32 + 0.5, k as f32 + 0.5) * self.u
    }

    /// Un voxel en el punto `p` del mundo, sea cual sea el tamano del voxel.
    fn poner(&mut self, p: Vec3, parte: &str, mat: &str) {
        let u = self.u;
        self.put(((p.x / u).floor() as i32, (p.y / u).floor() as i32, (p.z / u).floor() as i32), parte, mat);
    }

    /// Llena con voxels la caja del mundo [x0, x1] x [y0, y1] x [z0, z1] (al menos un voxel).
    #[allow(clippy::too_many_arguments)]
    fn caja(&mut self, parte: &str, mat: &str, x0: f32, x1: f32, y0: f32, y1: f32, z0: f32, z1: f32) {
        let u = self.u;
        let rango = |a: f32, b: f32| {
            let desde = (a / u).floor() as i32;
            desde..=desde.max((b / u).ceil() as i32 - 1)
        };
        for i in rango(x0, x1) {
            for j in rango(y0, y1) {
                for k in rango(z0, z1) {
                    self.put((i, j, k), parte, mat);
                }
            }
        }
    }

    /// Voxels en fila desde `inicio` a lo largo de un eje (0, 1, 2).
    fn columna(&mut self, parte: &str, mat: &str, inicio: [i32; 3], eje: usize, signo: i32, largo: i32) {
        for s in 0..largo {
            let mut p = inicio;
            p[eje] += s * signo;
            self.put((p[0], p[1], p[2]), parte, mat);
        }
    }

    /// El modelo: de cada voxel, las caras que dan al aire (una cara entre dos voxels no se ve),
    /// juntadas en rectangulos por parte, eje y capa. `escala` agranda todo (Saturno es de radio 2).
    pub fn modelo(&self, escala: f32) -> Modelo {
        let mut grupos: HashMap<(u16, usize, i32, i32), HashSet<(i32, i32)>> = HashMap::new();
        for (&(i, j, k), &parte) in &self.celdas {
            let ijk = [i, j, k];
            for eje in 0..3 {
                for signo in [-1, 1] {
                    let mut v = ijk;
                    v[eje] += signo;
                    if self.celdas.contains_key(&(v[0], v[1], v[2])) {
                        continue;
                    }
                    grupos.entry((parte, eje, signo, ijk[eje])).or_default().insert((ijk[(eje + 1) % 3], ijk[(eje + 2) % 3]));
                }
            }
        }
        // En orden, para que el modelo salga igual cada vez (el HashMap no tiene orden).
        let mut claves: Vec<_> = grupos.keys().copied().collect();
        claves.sort_unstable();
        let mut triangulos: Vec<Vec<[Vertice; 3]>> = vec![Vec::new(); self.partes.len()];
        for clave in claves {
            let (parte, eje, signo, capa) = clave;
            let mut n = [0.0; 3];
            n[eje] = signo as f32;
            let normal = Vec3::new(n[0], n[1], n[2]);
            let plano = (capa + if signo > 0 { 1 } else { 0 }) as f32 * self.u * escala;
            for (b0, c0, alto, ancho) in crate::chapin::rectangulos(&grupos[&clave]) {
                let esquina = |b: i32, c: i32| {
                    let mut p = [0.0; 3];
                    p[eje] = plano;
                    p[(eje + 1) % 3] = b as f32 * self.u * escala;
                    p[(eje + 2) % 3] = c as f32 * self.u * escala;
                    Vertice { pos: Vec3::new(p[0], p[1], p[2]), normal, uv: [0.0, 0.0] }
                };
                let q = [esquina(b0, c0), esquina(b0 + alto, c0), esquina(b0 + alto, c0 + ancho), esquina(b0, c0 + ancho)];
                // Las esquinas van en el sentido de las manecillas visto desde +eje: del lado de
                // afuera, antihorario (como pide el pipeline).
                let t = &mut triangulos[parte as usize];
                if signo > 0 {
                    t.push([q[0], q[1], q[2]]);
                    t.push([q[0], q[2], q[3]]);
                } else {
                    t.push([q[0], q[2], q[1]]);
                    t.push([q[0], q[3], q[2]]);
                }
            }
        }
        let partes = self
            .partes
            .iter()
            .zip(triangulos)
            .filter(|(_, t)| !t.is_empty())
            .map(|((nombre, material), triangulos)| Parte { nombre: nombre.clone(), material: *material, triangulos, comentario: String::new() })
            .collect();
        Modelo::nuevo(partes, self.materiales.clone())
    }
}

/// Como se arma un planeta.
#[derive(Clone, Copy)]
struct Receta {
    archivo: &'static str,
    /// Voxels de lado a lado en el nivel ligero (el detallado lleva 2.5 veces mas).
    diametro: i32,
    materiales: &'static [Mat],
    /// De que es el voxel con centro en `p` (el material, que tambien es el nombre de su parte).
    color: fn(Vec3) -> &'static str,
    /// El relieve propio en la direccion `d` (unitaria), en radios: crateres, continentes...
    forma: fn(Vec3) -> f32,
    /// La deformacion con Perlin: amplitud (en radios), frecuencia, octavas y semilla.
    ruido: (f32, f32, u32, u32),
    /// Que tanto se deforma con el ruido en la direccion `d`: 0 nada (las caras, la cueva), 1 todo.
    calma: fn(Vec3) -> f32,
    /// Lo que se le agrega por fuera (en coordenadas del planeta): el volcan, las nubes, el anillo.
    extra: Option<fn(&mut Voxels, bool)>,
}

fn sin_forma(_: Vec3) -> f32 {
    0.0
}

fn sin_calma(_: Vec3) -> f32 {
    1.0
}

/// Cerca de la cara (al frente, hacia -Z y un poco abajo) la superficie no se mueve: si no, el
/// relieve la taparia.
fn calma_cara(d: Vec3) -> f32 {
    1.0 - paso_suave(0.80, 0.92, d.dot(Vec3::new(0.0, -0.2, -1.0).normalize()))
}

// ---------- Sol ----------
// Granulacion (celdas mas calientes y mas frias), manchas solares y prominencias en arco. Su
// color de verdad lo pone el shader de plasma; el ruido le hace la superficie burbujeante.
const MAT_SOL: &[Mat] = &[
    ("sol", [0.35, 0.22, 0.05], Some([0.95, 0.60, 0.15]), 1.0),
    ("sol_caliente", [0.40, 0.30, 0.10], Some([1.00, 0.84, 0.40]), 1.0),
    ("sol_frio", [0.30, 0.15, 0.04], Some([0.85, 0.42, 0.10]), 1.0),
    ("sol_mancha", [0.15, 0.06, 0.02], Some([0.45, 0.16, 0.04]), 1.0),
    ("prominencia", [0.30, 0.10, 0.10], Some([1.00, 0.42, 0.28]), 1.0),
];

fn color_sol(p: Vec3) -> &'static str {
    let d = p.normalize();
    if mancha(d, 1.3, 1.0) > 0.78 && d.y.abs() < 0.6 {
        return "sol_mancha";
    }
    let granulo = mancha(d, 7.0, 3.0) + 0.5 * mancha(d, 13.0, 4.0);
    if granulo > 0.55 {
        "sol_caliente"
    } else if granulo < -0.45 {
        "sol_frio"
    } else {
        "sol"
    }
}

/// Arcos de plasma que salen y vuelven a la superficie.
fn prominencias(vx: &mut Voxels, hd: bool) {
    for (lon0, lat0, giro, alto) in [(0.4f32, 0.3f32, 0.6f32, 0.24f32), (2.3, -0.2, 1.4, 0.2), (4.1, 0.5, 0.2, 0.22)] {
        for k in 0..60 {
            let a = PI * k as f32 / 59.0;
            let (dlon, dlat) = (0.28 * a.cos() * giro.cos(), 0.28 * a.cos() * giro.sin());
            let r = 1.0 + alto * a.sin();
            let (lon, lat) = (lon0 + dlon, lat0 + dlat);
            let p = Vec3::new(r * lat.cos() * lon.sin(), r * lat.sin(), -r * lat.cos() * lon.cos());
            let grosor = vx.u * if hd { 0.7 } else { 0.4 };
            for dx in [-grosor, 0.0, grosor] {
                for dy in [-grosor, 0.0, grosor] {
                    for dz in [-grosor, 0.0, grosor] {
                        if dx.abs() + dy.abs() + dz.abs() <= grosor * 1.5 {
                            vx.poner(p + Vec3::new(dx, dy, dz), "prominencias", "prominencia");
                        }
                    }
                }
            }
        }
    }
}

// ---------- Mercurio: roca gris con crateres hundidos ----------
const MAT_MERCURIO: &[Mat] = &[("roca", [0.62, 0.60, 0.58], None, 1.0), ("crater", [0.40, 0.38, 0.37], None, 1.0)];

fn color_mercurio(p: Vec3) -> &'static str {
    if mancha(p, 1.0, 2.0) > 0.35 { "crater" } else { "roca" }
}

fn forma_mercurio(d: Vec3) -> f32 {
    if mancha(d, 1.0, 2.0) > 0.35 { -0.06 } else { 0.02 * mancha(d, 4.0, 2.5) }
}

// ---------- Venus: nubes amarillas en franjas ----------
const MAT_VENUS: &[Mat] = &[("nube", [0.92, 0.72, 0.38], None, 1.0), ("nube_clara", [1.00, 0.88, 0.60], None, 1.0)];

fn color_venus(p: Vec3) -> &'static str {
    if (7.0 * p.y + 2.0 * mancha(p, 1.0, 3.0)).sin() > 0.2 { "nube_clara" } else { "nube" }
}

// ---------- Tierra: oceanos, continentes con montanas y desiertos, polos y nubes ----------
const MAT_TIERRA: &[Mat] = &[
    ("oceano", [0.15, 0.40, 0.90], None, 1.0),
    ("continente", [0.30, 0.72, 0.30], None, 1.0),
    ("hielo", [0.95, 0.97, 1.00], None, 1.0),
    ("montana", [0.55, 0.50, 0.42], None, 1.0),
    ("desierto", [0.85, 0.75, 0.45], None, 1.0),
    ("nube_tierra", [1.0, 1.0, 1.0], None, 0.7),
];

fn tierra_firme(d: Vec3) -> f32 {
    mancha(d, 1.0, 4.0) + 0.3 * mancha(d, 2.0, 7.0)
}

fn color_tierra(p: Vec3) -> &'static str {
    let r = p.length().max(1e-6);
    let d = p * (1.0 / r);
    if d.y.abs() > 0.82 {
        return "hielo";
    }
    let f = tierra_firme(d);
    if f > 0.55 && r > 1.05 {
        "montana"
    } else if f > 0.12 {
        if d.y.abs() < 0.3 && mancha(p, 3.0, 5.5) > 0.3 { "desierto" } else { "continente" }
    } else {
        "oceano"
    }
}

fn forma_tierra(d: Vec3) -> f32 {
    let f = tierra_firme(d);
    if f <= 0.12 { 0.0 } else { 0.035 + (f - 0.55).max(0.0) * 0.18 }
}

/// El ruido solo levanta los continentes (el mar se queda parejo).
fn calma_tierra(d: Vec3) -> f32 {
    paso_suave(0.12, 0.25, tierra_firme(d))
}

/// Las nubes: una capa delgada que flota sobre todo (solo en el nivel detallado).
fn nubes_tierra(vx: &mut Voxels, hd: bool) {
    if !hd {
        return;
    }
    let n = (1.2 / vx.u).round() as i32;
    for i in -n..n {
        for j in -n..n {
            for k in -n..n {
                let p = vx.centro((i, j, k));
                let r = p.length();
                if (1.12..=1.12 + vx.u * 1.2).contains(&r) && manchas(2.2 * p.x, 3.0 * p.y, 2.2 * p.z, 21.0) > 0.42 && !vx.celdas.contains_key(&(i, j, k)) {
                    vx.put((i, j, k), "nubes", "nube_tierra");
                }
            }
        }
    }
}

// ---------- Luna: gris con mares oscuros ----------
const MAT_LUNA: &[Mat] = &[("luna", [0.78, 0.78, 0.80], None, 1.0), ("mar_lunar", [0.45, 0.45, 0.50], None, 1.0), ("crater", [0.60, 0.60, 0.63], None, 1.0)];

fn color_luna(p: Vec3) -> &'static str {
    if mancha(p, 1.0, 8.0) > 0.4 {
        "mar_lunar"
    } else if mancha(p, 3.0, 9.0) > 0.75 {
        "crater"
    } else {
        "luna"
    }
}

fn forma_luna(d: Vec3) -> f32 {
    if mancha(d, 1.0, 8.0) > 0.4 {
        -0.04
    } else if mancha(d, 3.0, 9.0) > 0.75 {
        -0.06
    } else {
        0.0
    }
}

// ---------- Marte: rojo oxido, el Monte Olimpo, Valles Marineris y un rover en el polo ----------
const MAT_MARTE: &[Mat] = &[
    ("marte", [0.85, 0.38, 0.20], None, 1.0),
    ("marte_oscuro", [0.55, 0.22, 0.14], None, 1.0),
    ("hielo", [0.95, 0.97, 1.00], None, 1.0),
    ("volcan", [0.78, 0.50, 0.32], None, 1.0),
    ("caldera", [0.30, 0.10, 0.06], Some([0.35, 0.08, 0.02]), 1.0),
    ("rover", [0.92, 0.92, 0.95], None, 1.0),
    ("rover_ruedas", [0.15, 0.15, 0.18], None, 1.0),
    ("rover_antena", [0.3, 0.9, 1.0], Some([0.3, 0.9, 1.0]), 1.0),
    ("hab", [0.92, 0.92, 0.95], None, 1.0),
    ("hab_luz", [1.0, 0.9, 0.6], Some([0.9, 0.75, 0.4]), 1.0),
    ("tierra_huerto", [0.42, 0.26, 0.16], None, 1.0),
    ("papa", [0.35, 0.80, 0.30], None, 1.0),
];

fn color_marte(p: Vec3) -> &'static str {
    if p.y > 0.85 {
        "hielo"
    } else if mancha(p, 1.0, 5.0) > 0.3 {
        "marte_oscuro"
    } else {
        "marte"
    }
}

fn forma_marte(d: Vec3) -> f32 {
    if mancha(d, 1.0, 5.0) > 0.3 { 0.0 } else { 0.03 + 0.02 * mancha(d, 5.0, 5.5) }
}

/// En el polo de arriba esta el rover: ahi el suelo se queda quieto.
fn calma_marte(d: Vec3) -> f32 {
    1.0 - paso_suave(0.88, 0.95, d.y)
}

fn volcan_y_rover(vx: &mut Voxels, _hd: bool) {
    // Monte Olimpo: un cono que sale de la superficie hacia `d`, con su caldera.
    let d = Vec3::new(0.55, 0.45, -0.70).normalize();
    let n = (1.45 / vx.u).ceil() as i32;
    for i in -n..n {
        for j in -n..n {
            for k in -n..n {
                let p = vx.centro((i, j, k));
                let a = p.dot(d);
                let perp = (p.dot(p) - a * a).max(0.0).sqrt();
                if perp < 0.42 && (0.85..=1.0 + 0.38 * (1.0 - perp / 0.42)).contains(&a) {
                    let caldera = perp < 0.08 && a > 1.2;
                    let m = if caldera { "caldera" } else { "volcan" };
                    vx.put((i, j, k), m, m);
                }
            }
        }
    }
    // Valles Marineris: un canon largo cerca del ecuador.
    let volcan = vx.partes.iter().position(|(n, _)| n == "volcan").map(|k| k as u16);
    let canon: Vec<Celda> = vx
        .celdas
        .iter()
        .filter(|&(&c, &parte)| {
            let p = vx.centro(c);
            let (lon, lat) = lon_lat(p);
            p.length() > 0.93 && (lat + 0.15 + 0.05 * (4.0 * lon).sin()).abs() < 0.05 && (-1.6..-0.4).contains(&lon) && Some(parte) != volcan
        })
        .map(|(&c, _)| c)
        .collect();
    for c in canon {
        vx.celdas.remove(&c);
    }
    // El rover en el polo de arriba, junto al hielo.
    vx.caja("rover", "rover", -0.12, 0.12, 1.0, 1.08, -0.25, 0.08);
    for (x, z) in [(-0.2, -0.24), (0.12, -0.24), (-0.2, 0.0), (0.12, 0.0)] {
        vx.caja("rover_ruedas", "rover_ruedas", x, x + 0.08, 0.98, 1.06, z, z + 0.08);
    }
    vx.caja("rover_antena", "rover_antena", -0.02, 0.04, 1.08, 1.18, -0.02, 0.04);
    // El huerto de papas de The Martian: el "Hab" (la casita blanca, con su ventana encendida) y
    // los surcos de papas (verde sobre tierra), del otro lado del rover.
    vx.caja("hab", "hab", 0.24, 0.44, 0.98, 1.12, -0.22, 0.0);
    vx.caja("hab_luz", "hab_luz", 0.30, 0.38, 1.04, 1.08, 0.0, 0.02);
    vx.caja("tierra_huerto", "tierra_huerto", -0.50, -0.26, 0.98, 1.03, -0.28, 0.06);
    let u = vx.u;
    let mut x = -0.48;
    while x < -0.27 {
        let mut z = -0.26;
        while z < 0.05 {
            vx.caja("papa", "papa", x, x + u * 0.9, 1.03, 1.07, z, z + u * 0.9);
            z += u * 2.0;
        }
        x += u * 2.0;
    }
}

// ---------- Jupiter: franjas por latitud y la Gran Mancha Roja ----------
const MAT_JUPITER: &[Mat] = &[
    ("jup_crema", [0.78, 0.70, 0.56], None, 1.0),
    ("jup_naranja", [0.92, 0.62, 0.35], None, 1.0),
    ("jup_cafe", [0.62, 0.40, 0.28], None, 1.0),
    ("mancha_roja", [0.85, 0.28, 0.20], None, 1.0),
];
const FRANJAS_JUPITER: [&str; 9] = ["jup_crema", "jup_naranja", "jup_crema", "jup_cafe", "jup_crema", "jup_naranja", "jup_crema", "jup_cafe", "jup_crema"];

fn color_jupiter(p: Vec3) -> &'static str {
    let (lon, lat) = lon_lat(p);
    // La mancha: un ovalo en el hemisferio sur.
    if ((lon - 0.6) / 0.42).powi(2) + ((lat + 0.38) / 0.15).powi(2) <= 1.0 {
        return "mancha_roja";
    }
    let f = ((p.y + 1.0) / 2.0 * FRANJAS_JUPITER.len() as f32 + 0.35 * (6.0 * lon).sin() * 0.3) as i32;
    FRANJAS_JUPITER[f.clamp(0, FRANJAS_JUPITER.len() as i32 - 1) as usize]
}

/// Un gigante de gas: las franjas se inflan y se hunden un poquito (los vientos van por latitud).
fn forma_jupiter(d: Vec3) -> f32 {
    0.015 * (d.y.clamp(-1.0, 1.0).asin() * 11.0).sin()
}

// ---------- Saturno (solo el planeta: el escenario de Saturno Ritmo es otro modelo) ----------
const MAT_SATURNO: &[Mat] = &[("sat_a", [0.98, 0.80, 0.55], None, 1.0), ("sat_b", [0.90, 0.60, 0.55], None, 1.0), ("sat_c", [0.72, 0.58, 0.92], None, 1.0)];
const FRANJAS_SATURNO: [&str; 10] = ["sat_a", "sat_b", "sat_c", "sat_a", "sat_b", "sat_b", "sat_a", "sat_c", "sat_b", "sat_a"];

// ---------- Urano: azul verdoso palido con un anillo delgado ----------
const MAT_URANO: &[Mat] = &[("urano", [0.55, 0.85, 0.88], None, 1.0), ("urano_claro", [0.72, 0.93, 0.94], None, 1.0), ("anillo", [0.35, 0.40, 0.55], None, 1.0)];

fn color_urano(p: Vec3) -> &'static str {
    if p.y.abs() > 0.55 { "urano_claro" } else { "urano" }
}

fn anillo_urano(vx: &mut Voxels, _hd: bool) {
    let n = (1.75 / vx.u).ceil() as i32;
    for i in -n..n {
        for k in -n..n {
            let p = vx.centro((i, 0, k));
            if (1.45..=1.7).contains(&p.x.hypot(p.z)) {
                vx.put((i, 0, k), "anillo", "anillo");
            }
        }
    }
}

// ---------- Neptuno: azul profundo, su gran mancha oscura y nubes blancas ----------
const MAT_NEPTUNO: &[Mat] = &[("neptuno", [0.20, 0.35, 0.95], None, 1.0), ("mancha_oscura", [0.10, 0.15, 0.50], None, 1.0), ("nube_blanca", [0.92, 0.95, 1.00], None, 1.0)];

fn color_neptuno(p: Vec3) -> &'static str {
    let (lon, lat) = lon_lat(p);
    if ((lon + 0.8) / 0.4).powi(2) + ((lat + 0.3) / 0.16).powi(2) <= 1.0 {
        "mancha_oscura"
    } else if (9.0 * lat + 2.0 * (3.0 * lon).sin()).sin().abs() > 0.93 && mancha(p, 1.0, 6.0) > 0.0 {
        "nube_blanca"
    } else {
        "neptuno"
    }
}

// ---------- Pluton: chiquito, beige, con su corazon blanco ----------
const MAT_PLUTON: &[Mat] = &[("pluton", [0.80, 0.68, 0.55], None, 1.0), ("pluton_oscuro", [0.50, 0.35, 0.28], None, 1.0), ("corazon_pluton", [0.98, 0.95, 0.90], None, 1.0)];

/// El corazon (Tombaugh Regio) mirando hacia -Z: la curva (u^2 + v^2 - 1)^3 - u^2 v^3 <= 0.
fn en_el_corazon(p: Vec3) -> bool {
    let (lon, lat) = lon_lat(p);
    let (hu, hv) = (lon / 0.5, (lat + 0.1) / 0.5);
    (hu * hu + hv * hv - 1.0).powi(3) - hu * hu * hv.powi(3) <= 0.0
}

fn color_pluton(p: Vec3) -> &'static str {
    if en_el_corazon(p) {
        "corazon_pluton"
    } else if mancha(p, 1.0, 11.0) > 0.35 {
        "pluton_oscuro"
    } else {
        "pluton"
    }
}

/// Montanas de hielo alrededor; el corazon es una planicie (como Sputnik Planitia) y tiene la cara.
fn calma_pluton(d: Vec3) -> f32 {
    if en_el_corazon(d) { 0.0 } else { calma_cara(d) }
}

// ---------- Planeta Hada: bosque de hongos que brillan y una cueva ----------
const MAT_HADA: &[Mat] = &[
    ("pasto", [0.30, 0.70, 0.45], None, 1.0),
    ("musgo", [0.20, 0.50, 0.40], None, 1.0),
    ("flores", [0.95, 0.60, 0.85], None, 1.0),
    ("cueva", [0.06, 0.04, 0.10], Some([0.20, 0.05, 0.30]), 1.0),
    ("hongo_tallo", [0.95, 0.92, 0.85], None, 1.0),
    ("hongo_rosa", [1.00, 0.45, 0.80], Some([0.70, 0.20, 0.55]), 1.0),
    ("hongo_cian", [0.35, 0.95, 1.00], Some([0.20, 0.65, 0.75]), 1.0),
];

fn color_hada(p: Vec3) -> &'static str {
    let (lon, lat) = lon_lat(p);
    if lon.abs() < 0.3 && lat.abs() < 0.22 {
        "cueva"
    } else if mancha(p, 1.0, 12.0) > 0.45 {
        "flores"
    } else if mancha(p, 2.0, 13.0) > 0.5 {
        "musgo"
    } else {
        "pasto"
    }
}

fn en_la_cueva(d: Vec3) -> bool {
    d.x.atan2(-d.z).abs() < 0.35 && d.y.abs() < 0.3
}

fn forma_hada(d: Vec3) -> f32 {
    if en_la_cueva(d) { 0.0 } else { 0.025 * (1.0 + mancha(d, 4.0, 14.0)) }
}

fn calma_hada(d: Vec3) -> f32 {
    let (lon, lat) = lon_lat(d);
    // Alrededor de la cueva (y de su portal), quieto.
    (paso_suave(0.35, 0.55, lon.abs())).max(paso_suave(0.3, 0.45, lat.abs()))
}

/// Cada hongo crece hacia afuera por el eje dominante de su direccion.
fn hongos(vx: &mut Voxels, _hd: bool) {
    let n = (1.0 / vx.u).round() as i32;
    let escala = n as f32 / 8.0;
    let direcciones = [(0.25, 1.0, 0.2), (-0.5, 0.85, -0.3), (1.0, 0.3, 0.4), (-1.0, 0.2, 0.5), (0.4, -0.2, 1.0), (-0.3, -0.4, 1.0), (0.6, -1.0, -0.2), (-0.7, 0.4, -1.0), (0.8, 0.6, -0.9), (-0.2, -1.0, 0.5)];
    for (m, &(x, y, z)) in direcciones.iter().enumerate() {
        let d = Vec3::new(x, y, z).normalize();
        let c = [d.x, d.y, d.z];
        let eje = (0..3).max_by(|&a, &b| c[a].abs().total_cmp(&c[b].abs())).unwrap_or(1);
        let signo = if c[eje] > 0.0 { 1 } else { -1 };
        // El voxel de la superficie en esa direccion (el redondeo de Python: a la par).
        let p = c.map(|v| (v * (n as f32 - 0.6) - 0.5).round_ties_even() as i32);
        // El tallo arranca un poquito mas afuera: el relieve puede haber subido el suelo.
        let tallo = ((2 + m as i32 % 2) as f32 * escala).round_ties_even() as i32 + 1;
        vx.columna("hongo_tallo", "hongo_tallo", p, eje, signo, tallo);
        let mut cima = p;
        cima[eje] += signo * tallo;
        let mat = if m % 2 == 0 { "hongo_rosa" } else { "hongo_cian" };
        let otros: Vec<usize> = (0..3).filter(|&q| q != eje).collect();
        let ancho = (1.2 * escala).round_ties_even() as i32;
        let capas = (escala.round_ties_even() as i32).max(1);
        for a in -ancho..=ancho {
            for b in -ancho..=ancho {
                for capa in 0..capas {
                    if (a * a + b * b) as f32 <= (ancho as f32 + 0.5).powi(2) - (capa * capa * 2) as f32 {
                        let mut q = cima;
                        q[otros[0]] += a;
                        q[otros[1]] += b;
                        q[eje] += signo * capa;
                        vx.put((q[0], q[1], q[2]), mat, mat);
                    }
                }
            }
        }
        let mut q = cima;
        q[eje] += signo * capas;
        vx.put((q[0], q[1], q[2]), mat, mat);
    }
}

/// Todas las recetas (Saturno y Planeta Vegetta tienen su propia funcion; el Chapin, su modulo).
const RECETAS: [Receta; 11] = [
    Receta { archivo: "sol", diametro: 16, materiales: MAT_SOL, color: color_sol, forma: sin_forma, ruido: (0.05, 3.0, 3, 11), calma: sin_calma, extra: Some(prominencias) },
    Receta { archivo: "mercurio", diametro: 16, materiales: MAT_MERCURIO, color: color_mercurio, forma: forma_mercurio, ruido: (0.05, 3.2, 4, 12), calma: sin_calma, extra: None },
    Receta { archivo: "venus", diametro: 16, materiales: MAT_VENUS, color: color_venus, forma: sin_forma, ruido: (0.035, 2.2, 3, 13), calma: sin_calma, extra: None },
    Receta { archivo: "tierra", diametro: 16, materiales: MAT_TIERRA, color: color_tierra, forma: forma_tierra, ruido: (0.05, 3.0, 4, 14), calma: calma_tierra, extra: Some(nubes_tierra) },
    Receta { archivo: "luna", diametro: 12, materiales: MAT_LUNA, color: color_luna, forma: forma_luna, ruido: (0.055, 3.0, 4, 15), calma: sin_calma, extra: None },
    Receta { archivo: "marte", diametro: 16, materiales: MAT_MARTE, color: color_marte, forma: forma_marte, ruido: (0.05, 2.6, 4, 16), calma: calma_marte, extra: Some(volcan_y_rover) },
    Receta { archivo: "jupiter", diametro: 24, materiales: MAT_JUPITER, color: color_jupiter, forma: forma_jupiter, ruido: (0.03, 2.0, 3, 17), calma: sin_calma, extra: None },
    Receta { archivo: "urano", diametro: 16, materiales: MAT_URANO, color: color_urano, forma: sin_forma, ruido: (0.035, 2.0, 3, 19), calma: sin_calma, extra: Some(anillo_urano) },
    Receta { archivo: "neptuno", diametro: 16, materiales: MAT_NEPTUNO, color: color_neptuno, forma: sin_forma, ruido: (0.04, 2.4, 3, 20), calma: calma_cara, extra: None },
    Receta { archivo: "pluton", diametro: 18, materiales: MAT_PLUTON, color: color_pluton, forma: sin_forma, ruido: (0.07, 3.0, 4, 21), calma: calma_pluton, extra: None },
    Receta { archivo: "hada", diametro: 16, materiales: MAT_HADA, color: color_hada, forma: forma_hada, ruido: (0.035, 3.0, 3, 22), calma: calma_hada, extra: Some(hongos) },
];

/// Saturno (radio 2) y Planeta Vegetta, que no son esferas de radio 1 como los demas.
const RUIDO_SATURNO: (f32, f32, u32, u32) = (0.035, 2.0, 3, 18);
const RUIDO_VEGETTA: (f32, f32, u32, u32) = (0.0, 0.35, 3, 23);

/// El ajuste en vivo de cada receta y su generador de Perlin (para las colisiones, que preguntan
/// el radio de la superficie cada cuadro).
fn vivos() -> &'static RwLock<Vec<(Ajuste, Perlin)>> {
    static V: OnceLock<RwLock<Vec<(Ajuste, Perlin)>>> = OnceLock::new();
    V.get_or_init(|| RwLock::new(RECETAS.iter().map(|r| (Ajuste::default(), Perlin::new(r.ruido.3))).collect()))
}

impl Receta {
    /// Cuanto sube (o baja) la superficie en la direccion `d` (unitaria), en radios, con el ruido
    /// de `perlin` y el ajuste en vivo.
    fn relieve(&self, d: Vec3, perlin: &Perlin, ajuste: Ajuste) -> f32 {
        let (amplitud, frecuencia, octavas, _) = self.ruido;
        let ruido = perlin.fbm(d * frecuencia + Vec3::new(1.7, -3.1, 2.3), octavas);
        ajuste.relieve * ((self.forma)(d) + amplitud * (self.calma)(d) * ruido)
    }

    /// Los voxels de este planeta, con `lado` voxels de lado a lado.
    fn voxels(&self, lado: i32, ajuste: Ajuste) -> Voxels {
        let perlin = Perlin::new(self.ruido.3.wrapping_add(ajuste.semilla));
        // Los extras finos (las nubes, las prominencias gruesas), del nivel detallado para arriba.
        let hd = lado >= self.diametro * 2;
        let u = 2.0 / lado as f32;
        let mut vx = Voxels::new(u, self.materiales, (0.2, 16.0));
        // Hasta donde puede llegar el relieve (lo propio mas el ruido), y desde donde para adentro
        // todo es solido (ahi ni se pregunta).
        let k = ajuste.relieve.max(0.0);
        let alcance = 1.0 + (0.16 + self.ruido.0 * 1.6) * k.max(1.0) + u;
        let adentro = 1.0 - (0.07 + self.ruido.0 * 1.6) * k.max(1.0) - u;
        let n = (alcance / u).ceil() as i32 + 1;
        for i in -n..n {
            for j in -n..n {
                for k in -n..n {
                    let p = vx.centro((i, j, k));
                    let r = p.length();
                    if r > alcance {
                        continue;
                    }
                    if r <= adentro || r <= 1.0 + self.relieve(p * (1.0 / r.max(1e-6)), &perlin, ajuste) {
                        let m = (self.color)(p);
                        vx.put((i, j, k), m, m);
                    }
                }
            }
        }
        if let Some(extra) = self.extra {
            extra(&mut vx, hd);
        }
        vx
    }
}

/// Saturno sin su escenario: una esfera de voxels de 0.2 (radio 2) con franjas por latitud, las
/// partes `franja_0` a `franja_9` (las que espera Saturno Ritmo), deformada con ruido.
fn saturno(lado: i32, ajuste: Ajuste) -> Modelo {
    let u = 2.0 / lado as f32;
    let mut vx = Voxels::new(u, MAT_SATURNO, (0.3, 16.0));
    let (amplitud, frecuencia, octavas, semilla) = RUIDO_SATURNO;
    let perlin = Perlin::new(semilla.wrapping_add(ajuste.semilla));
    let borde = (lado as f32 * (0.5 + 0.1 * ajuste.relieve.max(0.0))).ceil() as i32 + 1;
    for i in -borde..borde {
        for j in -lado / 2..lado / 2 {
            for k in -borde..borde {
                let p = vx.centro((i, j, k));
                let r = p.length().max(1e-6);
                let d = p * (1.0 / r);
                let relieve = ajuste.relieve * (amplitud * perlin.fbm(d * frecuencia + Vec3::new(1.7, -3.1, 2.3), octavas) + 0.02 * (d.y.asin() * 9.0).sin());
                if r <= 1.0 + relieve {
                    // Diez franjas por latitud, como las espera Saturno Ritmo.
                    let f = ((j + lado / 2) * 10).div_euclid(lado).clamp(0, 9) as usize;
                    vx.put((i, j, k), &format!("franja_{f}"), FRANJAS_SATURNO[f]);
                }
            }
        }
    }
    vx.modelo(2.0)
}

/// Planeta Vegetta: un cubo de bloques (16 por lado), con pasto arriba, menas y lava adentro, un
/// arbolito y la casita de Vegetta. El ruido le levanta cerros de uno y dos bloques en las caras
/// de los lados y de abajo (arriba queda parejo para la casa).
fn vegetta(lado: i32, ajuste: Ajuste) -> Modelo {
    const MAT: &[Mat] = &[
        ("pasto", [0.37, 0.63, 0.24], None, 1.0),
        ("tierra", [0.53, 0.38, 0.25], None, 1.0),
        ("piedra", [0.49, 0.49, 0.49], None, 1.0),
        ("diamante", [0.35, 0.92, 0.88], Some([0.12, 0.35, 0.33]), 1.0),
        ("oro", [0.98, 0.84, 0.27], Some([0.15, 0.12, 0.02]), 1.0),
        ("lava", [0.95, 0.42, 0.08], Some([0.8, 0.3, 0.05]), 1.0),
        ("tronco", [0.40, 0.30, 0.18], None, 1.0),
        ("hojas", [0.24, 0.52, 0.18], None, 1.0),
        ("tablas", [0.66, 0.51, 0.31], None, 1.0),
        ("morado", [0.47, 0.20, 0.67], None, 1.0),
        ("puerta", [0.35, 0.22, 0.12], None, 1.0),
    ];
    let mut vx = Voxels::new(2.0 / lado as f32, MAT, (0.15, 12.0));
    let medio = lado / 2;
    let mut azar = 777u32.wrapping_add(ajuste.semilla);
    let mut dado = || {
        azar = azar.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (azar >> 8) as f32 / 16_777_216.0
    };
    for i in -medio..medio {
        for j in -medio..medio {
            for k in -medio..medio {
                let borde = (i as f32 + 0.5).abs().max((j as f32 + 0.5).abs()).max((k as f32 + 0.5).abs()) > medio as f32 - 1.5;
                if !borde {
                    continue;
                }
                let m = if j == medio - 1 {
                    "pasto"
                } else if j >= medio / 2 {
                    "tierra"
                } else {
                    let r = dado();
                    if r < 0.04 {
                        "diamante"
                    } else if r < 0.08 {
                        "oro"
                    } else if r < 0.1 && j < -medio / 2 + 1 {
                        "lava"
                    } else {
                        "piedra"
                    }
                };
                vx.put((i, j, k), m, m);
            }
        }
    }
    // Los cerros: en cada cara (menos la de arriba), cuantos bloques sube cada columna sale del
    // ruido de Perlin en ese punto de la cara (en bloques del cubo de 16, para que de lejos y de
    // cerca salgan en el mismo lugar); el ajuste en vivo los sube o los aplana.
    let (_, frecuencia, octavas, semilla) = RUIDO_VEGETTA;
    let perlin = Perlin::new(semilla.wrapping_add(ajuste.semilla));
    let a_16 = 16.0 / lado as f32;
    let tope = (2.0 * ajuste.relieve.max(0.0)).round();
    for eje in 0..3usize {
        for signo in [-1i32, 1] {
            if eje == 1 && signo > 0 {
                continue;
            }
            for a in -medio..medio {
                for b in -medio..medio {
                    let mut p = [0.0f32; 3];
                    p[eje] = signo as f32 * 8.0;
                    p[(eje + 1) % 3] = (a as f32 + 0.5) * a_16;
                    p[(eje + 2) % 3] = (b as f32 + 0.5) * a_16;
                    // (En bloques del cubo de 16; de lejos los bloques son del doble.)
                    let h = ((perlin.fbm(Vec3::new(p[0], p[1], p[2]) * frecuencia, octavas) + 0.1) * 4.0).floor().clamp(0.0, tope) as i32 * lado / 16;
                    for capa in 1..=h {
                        let mut c = [0i32; 3];
                        c[eje] = if signo > 0 { medio - 1 + capa } else { -medio - capa };
                        c[(eje + 1) % 3] = a;
                        c[(eje + 2) % 3] = b;
                        // Los cerros de los lados, con pasto en lo de arriba (como en Minecraft).
                        let m = if eje != 1 && c[1] >= medio - 2 { "pasto" } else if eje != 1 && c[1] >= medio / 2 - 1 { "tierra" } else { "piedra" };
                        vx.put((c[0], c[1], c[2]), m, m);
                    }
                }
            }
        }
    }
    // Un arbolito y la casita de Vegetta (techo morado) arriba (de lejos, con bloques de a dos, ni
    // se verian: solo en el cubo de 16).
    if lado < 16 {
        return vx.modelo(1.0);
    }
    for j in 8..12 {
        vx.put((-4, j, 3), "tronco", "tronco");
    }
    for i in -6..-1 {
        for k in 1..6 {
            for j in 11..13 {
                if (i + 4i32).abs() + (k - 3i32).abs() < 4 {
                    vx.put((i, j, k), "hojas", "hojas");
                }
            }
        }
    }
    let caja = |vx: &mut Voxels, parte: &str, mat: &str, (i0, i1): (i32, i32), (j0, j1): (i32, i32), (k0, k1): (i32, i32)| {
        for i in i0..=i1 {
            for j in j0..=j1 {
                for k in k0..=k1 {
                    vx.put((i, j, k), parte, mat);
                }
            }
        }
    };
    caja(&mut vx, "casa", "tablas", (1, 5), (8, 10), (-5, -1));
    for n in 0..3 {
        caja(&mut vx, "techo", "morado", (1 + n, 5 - n), (11 + n, 11 + n), (-5, -1));
    }
    caja(&mut vx, "puerta", "puerta", (3, 3), (8, 9), (-6, -6));
    vx.modelo(1.0)
}

/// Agrega a `m` las partes de `extra` (con sus materiales): las caras de Neptuno y Pluton, el
/// escenario de Saturno.
pub fn juntar(m: Modelo, extra: &Modelo) -> Modelo {
    let desde = m.materiales.len();
    let mut materiales = m.materiales;
    materiales.extend(extra.materiales.iter().cloned());
    let mut partes = m.partes;
    partes.extend(extra.partes.iter().map(|p| Parte { material: p.material + desde, ..p.clone() }));
    Modelo::nuevo(partes, materiales)
}

/// Los cinco niveles de detalle del cuerpo `archivo` (del ligero al mas fino), armados aqui con
/// el ajuste en vivo, cada nivel en su hilo. `None` si no es de los que se arman con receta (el
/// Chapin tiene su propio modulo).
pub fn armar(archivo: &str, ajuste: Ajuste) -> Option<Vec<Modelo>> {
    let nivel = |n: usize| -> Modelo {
        match archivo {
            "saturno_ritmo" => saturno(lado_del_nivel(20, n), ajuste),
            // El cubo de bloques: de lejos de 8 por lado, de cerca de 16 (sus bloques de siempre).
            "planeta_vegetta" => vegetta(if n < 2 { 8 } else { 16 }, ajuste),
            _ => {
                let r = RECETAS.iter().find(|r| r.archivo == archivo).expect("receta");
                r.voxels(lado_del_nivel(r.diametro, n), ajuste).modelo(1.0)
            }
        }
    };
    if !matches!(archivo, "saturno_ritmo" | "planeta_vegetta") {
        let k = RECETAS.iter().position(|r| r.archivo == archivo)?;
        if let Ok(mut v) = vivos().write() {
            v[k] = (ajuste, Perlin::new(RECETAS[k].ruido.3.wrapping_add(ajuste.semilla)));
        }
    }
    Some(std::thread::scope(|hilos| {
        let tareas: Vec<_> = (0..NIVELES.len()).map(|n| hilos.spawn(move || nivel(n))).collect();
        tareas.into_iter().map(|t| t.join().expect("no se pudo armar un nivel")).collect()
    }))
}

/// El radio de la superficie del cuerpo `archivo` en la direccion `d` (unitaria, en el espacio
/// del planeta), en radios del modelo, con el relieve de ese momento: para que la nave choque con
/// las montanas de verdad.
pub fn radio_superficie(archivo: &str, d: Vec3) -> Option<f32> {
    let k = RECETAS.iter().position(|r| r.archivo == archivo)?;
    let v = vivos().read().ok()?;
    let (ajuste, perlin) = &v[k];
    Some(1.0 + RECETAS[k].relieve(d, perlin, *ajuste).max(-0.1))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn triangulos(m: &Modelo) -> usize {
        m.partes.iter().map(|p| p.triangulos.len()).sum()
    }

    /// Todos se arman en sus cinco niveles, con mas detalle en los finos, y siempre igual.
    #[test]
    fn todos_los_planetas_se_arman_con_sus_cinco_niveles() {
        for archivo in RECETAS.iter().map(|r| r.archivo).chain(["saturno_ritmo", "planeta_vegetta"]) {
            let niveles = armar(archivo, Ajuste::default()).expect("sin receta");
            assert_eq!(niveles.len(), 5);
            let t: Vec<usize> = niveles.iter().map(triangulos).collect();
            assert!(t[0] > 50, "{archivo} casi vacio");
            assert!(t[0] <= t[2] && t[2] <= t[4] && t[0] < t[4], "{archivo}: los niveles no crecen: {t:?}");
            let otra = armar(archivo, Ajuste::default()).unwrap();
            assert_eq!(triangulos(&otra[3]), t[3], "{archivo} cambia de una vez a otra");
        }
        let saturno = armar("saturno_ritmo", Ajuste::default()).unwrap();
        assert!(saturno.iter().all(|m| m.partes.iter().filter(|p| p.nombre.starts_with("franja_")).count() == 10), "Saturno Ritmo espera diez franjas");
    }

    /// Un punto de la esfera (Fibonacci): `k` de `n`.
    fn punto(k: usize, n: usize) -> Vec3 {
        let z = 1.0 - 2.0 * (k as f32 + 0.5) / n as f32;
        let a = k as f32 * 2.399_963;
        Vec3::new((1.0 - z * z).sqrt() * a.cos(), z, (1.0 - z * z).sqrt() * a.sin())
    }

    /// Cada planeta esta deformado por el ruido: sin el ruido, la superficie seria otra; y con otra
    /// semilla (F5/F6), tambien.
    #[test]
    fn el_ruido_deforma_a_todos() {
        for r in &RECETAS {
            let perlin = Perlin::new(r.ruido.3);
            let sin_ruido = Receta { ruido: (0.0, r.ruido.1, r.ruido.2, r.ruido.3), ..*r };
            let normal = Ajuste::default();
            let distintos = (0..400).filter(|&k| (r.relieve(punto(k, 400), &perlin, normal) - sin_ruido.relieve(punto(k, 400), &perlin, normal)).abs() > 0.005).count();
            // (En la Tierra el ruido solo levanta los continentes, como la cuarta parte: el mar
            // se queda parejo.)
            assert!(distintos > 60, "{}: el ruido casi no lo toca ({distintos} de 400)", r.archivo);
            // Y el nivel detallado lo nota (otra geometria), tambien con otra semilla.
            let lado = lado_del_nivel(r.diametro, 3);
            let con = triangulos(&r.voxels(lado, normal).modelo(1.0));
            assert_ne!(con, triangulos(&sin_ruido.voxels(lado, normal).modelo(1.0)), "{}: la geometria no cambio con el ruido", r.archivo);
            assert_ne!(con, triangulos(&r.voxels(lado, Ajuste { semilla: 1, ..normal }).modelo(1.0)), "{}: otra semilla, la misma geometria", r.archivo);
        }
    }

    /// La deformacion es una etapa aparte de la generacion: con el relieve en 0 (F7), cada planeta
    /// sale como una esfera de voxels lisa (sin sus extras, que van por fuera).
    #[test]
    fn sin_relieve_cada_planeta_es_una_esfera() {
        for r in &RECETAS {
            let lado = lado_del_nivel(r.diametro, 3);
            let u = 2.0 / lado as f32;
            let solo = Receta { extra: None, ..*r };
            let lejos = |ajuste: Ajuste| solo.voxels(lado, ajuste).modelo(1.0).partes.iter().flat_map(|p| p.triangulos.iter().flatten()).map(|v| v.pos.length()).fold(0.0f32, f32::max);
            let lisa = lejos(Ajuste { relieve: 0.0, ..Ajuste::default() });
            assert!(lisa <= 1.0 + u * 0.9, "{}: sin relieve llega a {lisa}", r.archivo);
            assert!(lejos(Ajuste { relieve: 3.0, ..Ajuste::default() }) > lisa, "{}: con mas relieve no crece", r.archivo);
        }
    }

    /// Las caras que salen dan al aire: ninguna queda entre dos voxels.
    #[test]
    fn solo_salen_las_caras_que_dan_al_aire() {
        let vx = RECETAS[5].voxels(lado_del_nivel(RECETAS[5].diametro, 3), Ajuste::default());
        let m = vx.modelo(1.0);
        for p in &m.partes {
            for t in &p.triangulos {
                let centro = (t[0].pos + t[1].pos + t[2].pos) * (1.0 / 3.0);
                let afuera = centro + t[0].normal * (vx.u * 0.5);
                let c = ((afuera.x / vx.u).floor() as i32, (afuera.y / vx.u).floor() as i32, (afuera.z / vx.u).floor() as i32);
                assert!(!vx.celdas.contains_key(&c), "una cara de {} da a otro voxel", p.nombre);
            }
        }
    }
}
