//! PLANETA CHAPIN: Guatemala hecha planeta. Es el planeta "de laboratorio" del proyecto: aqui
//! estan juntas las tecnicas del menu de puntos que no caben en un `.obj` hecho antes de correr.
//! Es voxel, como todos los planetas del juego, pero no viene de un archivo: lo arma el juego.
//!
//! 1. **Generacion procedural de geometria** (antes de rasterizar): una rejilla de voxels
//!    alrededor de la esfera; cada celda se llena o no segun la altura del terreno en su
//!    direccion (tierra), o queda de agua si esta abajo del nivel del mar. De cada voxel solo se
//!    sacan las caras que dan al aire, y las caras vecinas iguales se juntan en rectangulos
//!    (greedy meshing): muchos menos triangulos con la misma forma.
//! 2. **Deformacion con ruido**: la altura sale de ruido Perlin (`ruido.rs`): fBm para los
//!    continentes, "crestas" para las cordilleras, y una cadena de cuatro VOLCANES (conos con su
//!    crater; los dos activos, con lava y una colada bajando) en un arco, como el cinturon
//!    volcanico. De que es cada voxel (arena, pasto, selva, milpa, monte, roca, ceniza, lava;
//!    tierra en los barrancos) tambien sale de la altura y de campos de ruido.
//! 3. **Niveles de detalle**: la misma superficie con 16 a 64 voxels de lado a lado; el juego
//!    dibuja la que toca segun que tan grande se ve el planeta (de lejos, voxels grandes), con el
//!    mismo criterio que los demas planetas (`planetas::nivel_para`).
//! 4. **Objetos sobre la superficie, repartidos con ruido y semilla**: ceibas, rocas volcanicas,
//!    casitas de Antigua, el Templo I de Tikal y el Arco de Santa Catalina. Se tiran puntos al
//!    azar (con la semilla) sobre la esfera y cada uno se queda o no segun campos de ruido: la
//!    "selva" (fBm), los "pueblos" (Perlin) y la pendiente. Misma semilla, mismas ubicaciones.
//! 5. **Posicion y orientacion sobre la superficie curva**: cada objeto crece hacia afuera del
//!    planeta (su "arriba" es la direccion desde el centro), con un giro al azar, y se para justo
//!    donde esa linea toca los voxels (en cada nivel de detalle). Su matriz se arma en el espacio
//!    del planeta, asi que al girar el planeta (`M * local`) todo gira con el, sin flotar ni
//!    resbalarse.
//!
//! El terreno y el mar usan sus propios shaders (`Shader::Terreno`, con mapa de normales y lava;
//! `Shader::Oceano`, con olas animadas). Los parametros se cambian en vivo (ver `Parametros`).

use crate::matematica::{Mat4, Vec3};
use crate::obj::Modelo;
use crate::obra::Obra;
use crate::ruido::Perlin;
use std::collections::{HashMap, HashSet};
use std::f32::consts::TAU;
use std::sync::{Arc, RwLock};

/// Lo que se puede cambiar del planeta (en vivo, con F5 a F10 en el espacio; ver `main.rs`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parametros {
    /// La semilla del ruido y del reparto: la misma semilla, el mismo planeta.
    pub semilla: u32,
    /// Cuanto sube la cumbre mas alta sobre el mar, como fraccion del radio.
    pub relieve: f32,
    /// Que tan chiquitos son los continentes (frecuencia del ruido).
    pub frecuencia: f32,
    /// Cuantos puntos se tiran para repartir objetos (mas: mas objetos).
    pub puntos: usize,
}

pub const PARAMETROS: Parametros = Parametros { semilla: 502, relieve: 0.13, frecuencia: 1.35, puntos: 900 };

/// Los voxels de lado a lado de cada nivel de detalle (del mas ligero al mas fino). Los demas
/// planetas del juego tienen 16 de lejos y 40 de cerca.
pub const VOXELES: [i32; 5] = [16, 24, 34, 48, 64];

/// Lo ancho de los volcanes y de su crater (en radianes sobre la esfera), y lo hondo del crater.
const ANCHO_VOLCAN: f32 = 0.17;
const CRATER: f32 = 0.05;
const HONDO_CRATER: f32 = 0.22;

/// Los tipos de objeto de la superficie (cada uno es un modelo propio, `generar_chapin.py`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tipo {
    Ceiba,
    Roca,
    Casita,
    Tikal,
    Arco,
}

impl Tipo {
    #[cfg(test)]
    pub const TODOS: [Tipo; 5] = [Tipo::Ceiba, Tipo::Roca, Tipo::Casita, Tipo::Tikal, Tipo::Arco];

    /// Que tan grande es, en radios del planeta.
    fn tamano(self) -> f32 {
        match self {
            Tipo::Ceiba => 0.095,
            Tipo::Roca => 0.05,
            Tipo::Casita => 0.05,
            Tipo::Tikal => 0.17,
            Tipo::Arco => 0.1,
        }
    }

    /// Que tan separados quedan de otros objetos (en radianes sobre la esfera).
    fn separacion(self) -> f32 {
        match self {
            Tipo::Ceiba => 0.075,
            Tipo::Roca => 0.05,
            Tipo::Casita => 0.045,
            Tipo::Tikal | Tipo::Arco => 0.16,
        }
    }
}

/// Un objeto puesto sobre el planeta.
#[derive(Clone, Debug)]
pub struct Instancia {
    pub tipo: Tipo,
    /// Su matriz en el espacio del planeta (radio 1) en cada nivel de detalle (el suelo de voxels
    /// cambia un poquito de uno a otro): el mundo la ve como `M_planeta * locales[nivel]`.
    pub locales: [Mat4; 5],
    /// Donde esta (en el espacio del planeta, sobre el nivel mas fino).
    pub posicion: Vec3,
    /// Para las casitas: de que color es la pared (indice en los colores de Antigua).
    pub color: usize,
}

/// De que es un voxel. Los colores son los de los demas planetas del juego: planos y vivos.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Suelo {
    Agua,
    Arena,
    Pasto,
    Selva,
    Milpa,
    Tierra,
    Monte,
    Roca,
    Ceniza,
    Lava,
}

impl Suelo {
    const TODOS: [Suelo; 10] = [Suelo::Agua, Suelo::Arena, Suelo::Pasto, Suelo::Selva, Suelo::Milpa, Suelo::Tierra, Suelo::Monte, Suelo::Roca, Suelo::Ceniza, Suelo::Lava];

    fn nombre(self) -> &'static str {
        match self {
            Suelo::Agua => "agua",
            Suelo::Arena => "arena",
            Suelo::Pasto => "pasto",
            Suelo::Selva => "selva",
            Suelo::Milpa => "milpa",
            Suelo::Tierra => "tierra",
            Suelo::Monte => "monte",
            Suelo::Roca => "roca",
            Suelo::Ceniza => "ceniza",
            Suelo::Lava => "lava",
        }
    }

    /// (color, emisivo)
    fn colores(self) -> ([f32; 3], [f32; 3]) {
        match self {
            Suelo::Agua => ([0.15, 0.40, 0.90], [0.0; 3]),
            Suelo::Arena => ([0.96, 0.86, 0.60], [0.0; 3]),
            Suelo::Pasto => ([0.36, 0.76, 0.32], [0.0; 3]),
            Suelo::Selva => ([0.16, 0.56, 0.30], [0.0; 3]),
            Suelo::Milpa => ([0.70, 0.78, 0.26], [0.0; 3]),
            Suelo::Tierra => ([0.62, 0.41, 0.26], [0.0; 3]),
            Suelo::Monte => ([0.60, 0.53, 0.44], [0.0; 3]),
            Suelo::Roca => ([0.44, 0.38, 0.52], [0.0; 3]),
            Suelo::Ceniza => ([0.30, 0.27, 0.36], [0.0; 3]),
            Suelo::Lava => ([0.45, 0.12, 0.03], [1.0, 0.42, 0.10]),
        }
    }
}

/// La forma del terreno: con esto se sabe la altura en cualquier direccion (la usan los voxels,
/// el reparto de objetos y las colisiones de la nave).
pub struct Relieve {
    pub params: Parametros,
    continentes: Perlin,
    cordilleras: Perlin,
    /// Los campos de la selva y de los pueblos (deciden que crece y que se construye).
    selva: Perlin,
    pueblos: Perlin,
    /// Los volcanes: su direccion (unitaria) y que tan alto llegan.
    pub volcanes: Vec<(Vec3, f32)>,
    /// Por donde baja la colada de lava de cada volcan (tangente a la esfera).
    coladas: Vec<Vec3>,
}

/// Un generador congruencial (el de siempre), para tirar los dados con la semilla.
struct Dados(u32);

impl Dados {
    fn uno(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 8) as f32 / 16_777_216.0
    }

    /// Un punto al azar sobre la esfera unitaria (uniforme: z al azar en [-1, 1] y un angulo).
    fn en_la_esfera(&mut self) -> Vec3 {
        let z = self.uno() * 2.0 - 1.0;
        let a = self.uno() * TAU;
        let r = (1.0 - z * z).max(0.0).sqrt();
        Vec3::new(r * a.cos(), z, r * a.sin())
    }
}

fn paso_suave(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Gira `v` alrededor del eje unitario `eje` (formula de Rodrigues).
fn girar(v: Vec3, eje: Vec3, angulo: f32) -> Vec3 {
    let (s, c) = angulo.sin_cos();
    v * c + eje.cross(v) * s + eje * (eje.dot(v) * (1.0 - c))
}

impl Relieve {
    pub fn new(params: Parametros) -> Self {
        let mut dados = Dados(params.semilla.wrapping_mul(747_796_405) | 1);
        // Los volcanes, en fila sobre un arco (como el cinturon volcanico del Pacifico):
        // Tajumulco, Agua, Fuego y Pacaya. Fuego y Pacaya llegan a la lava (estan activos).
        let inicio = dados.en_la_esfera();
        let eje = inicio.cross(dados.en_la_esfera()).normalize();
        let alturas = [0.80, 0.84, 1.0, 0.93];
        let volcanes: Vec<(Vec3, f32)> = alturas.iter().enumerate().map(|(k, &alto)| (girar(inicio, eje, k as f32 * 0.24), alto)).collect();
        let coladas = volcanes.iter().map(|&(v, _)| girar(tangentes(v).0, v, dados.uno() * TAU)).collect();
        Relieve {
            params,
            continentes: Perlin::new(params.semilla),
            cordilleras: Perlin::new(params.semilla ^ 0x9e37_79b9),
            selva: Perlin::new(params.semilla ^ 0x00c3_1ba5),
            pueblos: Perlin::new(params.semilla ^ 0x00a7_0a90),
            volcanes,
            coladas,
        }
    }

    /// La altura del terreno en la direccion `p` (unitaria): 0 es el nivel del mar, 1 la cumbre
    /// del volcan mas alto. Lo de abajo de 0 queda bajo el agua.
    pub fn altura(&self, p: Vec3) -> f32 {
        let f = self.params.frecuencia;
        let c = self.continentes.fbm(p * f + Vec3::new(3.1, 1.7, 0.4), 5);
        let tierra = ((c + 0.1) * 2.4).clamp(-1.0, 1.0);
        let adentro = paso_suave(0.0, 0.45, tierra);
        let montes = self.cordilleras.crestas(p * (f * 2.7) + Vec3::new(-2.3, 4.1, 1.9), 4);
        let mut h = tierra * 0.3 + adentro * montes * montes * 0.42;
        // Los volcanes: conos de laderas concavas, con su crater hundido arriba.
        for &(v, alto) in &self.volcanes {
            let angulo = p.dot(v).clamp(-1.0, 1.0).acos();
            if angulo < ANCHO_VOLCAN {
                let k = 1.0 - angulo / ANCHO_VOLCAN;
                h = h.max(0.04) + (alto - 0.12) * k.powf(1.7);
                if angulo < CRATER {
                    h -= HONDO_CRATER * (1.0 - angulo / CRATER);
                }
            }
        }
        h
    }

    /// El radio del terreno en la direccion `p` (unitaria), en radios del planeta.
    pub fn radio(&self, p: Vec3) -> f32 {
        1.0 + self.params.relieve * self.altura(p).clamp(-0.6, 1.3)
    }

    /// El punto del terreno en la direccion `p`.
    pub fn punto(&self, p: Vec3) -> Vec3 {
        p * self.radio(p)
    }

    /// La normal del terreno en `p`: el producto cruz de dos diferencias finitas sobre la superficie.
    pub fn normal(&self, p: Vec3) -> Vec3 {
        let (t, b) = tangentes(p);
        let e = 0.004;
        let dt = self.punto((p + t * e).normalize()) - self.punto((p - t * e).normalize());
        let db = self.punto((p + b * e).normalize()) - self.punto((p - b * e).normalize());
        let n = dt.cross(db).normalize();
        if n.dot(p) < 0.0 { n * -1.0 } else { n }
    }

    /// De que es el voxel en la direccion `p` (unitaria), con el terreno a altura `h` ahi, que
    /// queda `hondo` por debajo de la superficie (en radios), en una rejilla de voxels de `celda`:
    /// la capa de arriba segun la altura, la selva y los pueblos; abajo, tierra (o roca).
    fn suelo(&self, p: Vec3, h: f32, hondo: f32, celda: f32) -> Suelo {
        let arriba = hondo < celda * 0.95;
        // Los volcanes: ceniza en las laderas altas; en los activos, lava en el crater y en la
        // colada que baja por un lado.
        for (&(v, alto), &colada) in self.volcanes.iter().zip(&self.coladas) {
            let angulo = p.dot(v).clamp(-1.0, 1.0).acos();
            if angulo > ANCHO_VOLCAN {
                continue;
            }
            if alto >= 0.93 {
                if angulo < CRATER * 1.15 {
                    return Suelo::Lava;
                }
                let d = p - v * p.dot(v);
                let a_lo_largo = d.dot(colada);
                let de_lado = (d - colada * a_lo_largo).length();
                if arriba && a_lo_largo > 0.0 && a_lo_largo < ANCHO_VOLCAN * 0.75 && de_lado < celda * 0.6 + 0.004 {
                    return Suelo::Lava;
                }
            }
            if h > 0.3 && angulo < ANCHO_VOLCAN * 0.7 {
                return if arriba { Suelo::Ceniza } else { Suelo::Roca };
            }
        }
        if !arriba {
            return if h > 0.62 { Suelo::Roca } else { Suelo::Tierra };
        }
        if h < 0.025 {
            Suelo::Arena
        } else if h > 0.62 {
            Suelo::Roca
        } else if h > 0.48 {
            Suelo::Monte
        } else if self.selva.fbm(p * 3.2, 3) > 0.02 {
            Suelo::Selva
        } else if self.pueblos.ruido(p * 2.4 + Vec3::new(7.0, 1.0, 3.0)) > 0.16 {
            // Alrededor de los pueblos, la milpa.
            Suelo::Milpa
        } else {
            Suelo::Pasto
        }
    }
}

/// Dos direcciones tangentes a la esfera en `p`.
fn tangentes(p: Vec3) -> (Vec3, Vec3) {
    let otro = if p.y.abs() < 0.9 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
    let t = otro.cross(p).normalize();
    (t, p.cross(t))
}

/// El relieve del Chapin que esta en el juego, para las colisiones de la nave (ver `colision.rs`).
static RELIEVE: RwLock<Option<Arc<Relieve>>> = RwLock::new(None);

/// El radio del terreno del Chapin en la direccion `p` (unitaria, en el espacio del planeta).
pub fn radio_superficie(p: Vec3) -> f32 {
    RELIEVE.read().ok().and_then(|r| r.as_ref().map(|r| r.radio(p))).unwrap_or(1.0)
}

/// Lo que mide un voxel del nivel de detalle `nivel` (en radios del planeta).
pub fn celda(nivel: usize) -> f32 {
    2.0 / VOXELES[nivel.min(VOXELES.len() - 1)] as f32
}

type Celda = (i32, i32, i32);

/// La rejilla de voxels del planeta con `lado` voxels de lado a lado: que hay en cada celda de la
/// cascara (lo de mas adentro es solido y no se guarda).
struct Rejilla<'a> {
    relieve: &'a Relieve,
    u: f32,
    /// Todo lo que este mas adentro que esto es tierra (no hace falta preguntarle al ruido).
    r_solido: f32,
    celdas: HashMap<Celda, Suelo>,
}

impl<'a> Rejilla<'a> {
    fn new(relieve: &'a Relieve, lado: i32) -> Self {
        let u = 2.0 / lado as f32;
        let rel = relieve.params.relieve;
        let r_max = 1.0 + rel * 1.3 + u;
        let r_solido = 1.0 - rel * 0.6 - 2.0 * u;
        let n = (r_max / u).ceil() as i32 + 1;
        let mut celdas = HashMap::new();
        for i in -n..n {
            for j in -n..n {
                for k in -n..n {
                    let c = Vec3::new(i as f32 + 0.5, j as f32 + 0.5, k as f32 + 0.5) * u;
                    let r = c.length();
                    if r < r_solido || r > r_max {
                        continue;
                    }
                    let dir = c * (1.0 / r);
                    let h = relieve.altura(dir);
                    let mut alto = 1.0 + rel * h.clamp(-0.6, 1.3);
                    // Bajo el mar, el fondo queda al menos un voxel abajo del agua (si no, en lo
                    // bajito el agua seria mas delgada que un voxel y se veria arena con charcos).
                    if h < 0.0 {
                        alto = alto.min(1.0 - u);
                    }
                    if r <= alto {
                        celdas.insert((i, j, k), relieve.suelo(dir, h, alto - r, u));
                    } else if r <= 1.0 {
                        celdas.insert((i, j, k), Suelo::Agua);
                    }
                }
            }
        }
        Rejilla { relieve, u, r_solido, celdas }
    }

    fn centro(&self, c: Celda) -> Vec3 {
        Vec3::new(c.0 as f32 + 0.5, c.1 as f32 + 0.5, c.2 as f32 + 0.5) * self.u
    }

    fn celda_de(&self, p: Vec3) -> Celda {
        ((p.x / self.u).floor() as i32, (p.y / self.u).floor() as i32, (p.z / self.u).floor() as i32)
    }

    /// Que hay en la celda `c` (lo de adentro de la cascara es tierra).
    fn en(&self, c: Celda) -> Option<Suelo> {
        self.celdas.get(&c).copied().or_else(|| (self.centro(c).length() < self.r_solido).then_some(Suelo::Tierra))
    }

    /// Donde la linea que sale del centro en la direccion `dir` (unitaria) toca los voxels: se
    /// baja desde arriba a pasitos de una vigesima de voxel hasta dar con algo (tierra, o agua en
    /// la orilla). Ahi se para un objeto.
    fn superficie(&self, dir: Vec3) -> Vec3 {
        let paso = self.u * 0.05;
        let mut r = 1.0 + self.relieve.params.relieve * 1.4 + self.u;
        while r > 0.3 && self.en(self.celda_de(dir * r)).is_none() {
            r -= paso;
        }
        dir * (r + paso * 0.5)
    }

    /// Los dos modelos: la tierra y el agua. Solo las caras que dan al aire (entre tierra y agua
    /// tampoco: el agua no deja ver lo de abajo), y las vecinas iguales juntas en rectangulos.
    fn modelos(&self) -> (Modelo, Modelo) {
        // Las caras, agrupadas por material, eje, sentido y capa: (b, c) en el plano de la capa.
        let mut grupos: HashMap<(Suelo, usize, i32, i32), HashSet<(i32, i32)>> = HashMap::new();
        for (&(i, j, k), &s) in &self.celdas {
            let ijk = [i, j, k];
            for eje in 0..3 {
                for signo in [-1, 1] {
                    let mut v = ijk;
                    v[eje] += signo;
                    // Cualquier vecino (tierra o agua) la tapa.
                    if self.en((v[0], v[1], v[2])).is_some() {
                        continue;
                    }
                    let (b, c) = ((eje + 1) % 3, (eje + 2) % 3);
                    grupos.entry((s, eje, signo, ijk[eje])).or_default().insert((ijk[b], ijk[c]));
                }
            }
        }
        let mut tierra = Obra::new();
        let mut agua = Obra::new();
        for s in Suelo::TODOS {
            let (kd, ke) = s.colores();
            let obra = if s == Suelo::Agua { &mut agua } else { &mut tierra };
            obra.material(s.nombre(), kd, ke, 0.0, 8.0);
        }
        // En orden, para que el modelo salga igual cada vez (el HashMap no tiene orden).
        let mut claves: Vec<_> = grupos.keys().copied().collect();
        claves.sort_by_key(|&(s, eje, signo, capa)| (s as u8, eje, signo, capa));
        for clave in claves {
            let (s, eje, signo, capa) = clave;
            let obra = if s == Suelo::Agua { &mut agua } else { &mut tierra };
            for (b0, c0, alto, ancho) in rectangulos(&grupos[&clave]) {
                let plano = (capa + if signo > 0 { 1 } else { 0 }) as f32 * self.u;
                let esquina = |b: i32, c: i32| {
                    let mut p = [0.0; 3];
                    p[eje] = plano;
                    p[(eje + 1) % 3] = b as f32 * self.u;
                    p[(eje + 2) % 3] = c as f32 * self.u;
                    Vec3::new(p[0], p[1], p[2])
                };
                let mut normal = [0.0; 3];
                normal[eje] = signo as f32;
                obra.quad(
                    s.nombre(),
                    [esquina(b0, c0), esquina(b0 + alto, c0), esquina(b0 + alto, c0 + ancho), esquina(b0, c0 + ancho)],
                    Vec3::new(normal[0], normal[1], normal[2]),
                );
            }
        }
        (tierra.modelo(), agua.modelo())
    }
}

/// Greedy meshing de una capa: junta las celdas en rectangulos (b0, c0, alto en b, ancho en c).
/// (Lo usan tambien los demas planetas, `planetas.rs`.)
pub(crate) fn rectangulos(celdas: &HashSet<(i32, i32)>) -> Vec<(i32, i32, i32, i32)> {
    let mut orden: Vec<(i32, i32)> = celdas.iter().copied().collect();
    orden.sort_unstable();
    let mut hechas: HashSet<(i32, i32)> = HashSet::with_capacity(celdas.len());
    let libre = |p: (i32, i32), hechas: &HashSet<(i32, i32)>| celdas.contains(&p) && !hechas.contains(&p);
    let mut salida = Vec::new();
    for (b0, c0) in orden {
        if hechas.contains(&(b0, c0)) {
            continue;
        }
        let mut ancho = 1;
        while libre((b0, c0 + ancho), &hechas) {
            ancho += 1;
        }
        let mut alto = 1;
        while (0..ancho).all(|dc| libre((b0 + alto, c0 + dc), &hechas)) {
            alto += 1;
        }
        for db in 0..alto {
            for dc in 0..ancho {
                hechas.insert((b0 + db, c0 + dc));
            }
        }
        salida.push((b0, c0, alto, ancho));
    }
    salida
}

/// Una matriz que pone un objeto en `posicion`, con su eje Y en `arriba` y su Z en `frente`.
fn colocar(posicion: Vec3, arriba: Vec3, frente: Vec3, escala: f32) -> Mat4 {
    let z = (frente - arriba * frente.dot(arriba)).normalize();
    let x = arriba.cross(z);
    let (x, y, z) = (x * escala, arriba * escala, z * escala);
    Mat4 { m: [[x.x, y.x, z.x, posicion.x], [x.y, y.y, z.y, posicion.y], [x.z, y.z, z.z, posicion.z], [0.0, 0.0, 0.0, 1.0]] }
}

/// El planeta armado: sus niveles de detalle, el mar y los objetos de la superficie (la forma
/// del terreno queda en `RELIEVE`, para las colisiones).
pub struct Chapin {
    /// La tierra con cada numero de voxels de `VOXELES`.
    pub niveles: Vec<Modelo>,
    /// El agua, en los mismos niveles.
    pub mar: Vec<Modelo>,
    pub instancias: Vec<Instancia>,
}

/// Los colores de las paredes de Antigua (las casitas se pintan con uno de estos).
pub const COLORES_ANTIGUA: [[f32; 3]; 5] = [[0.97, 0.76, 0.24], [0.42, 0.72, 0.85], [0.93, 0.50, 0.55], [0.55, 0.82, 0.62], [0.86, 0.45, 0.25]];

impl Chapin {
    pub fn new(params: Parametros) -> Self {
        let relieve = Arc::new(Relieve::new(params));
        if let Ok(mut r) = RELIEVE.write() {
            *r = Some(relieve.clone());
        }
        let rejillas: Vec<Rejilla> = VOXELES.iter().map(|&lado| Rejilla::new(&relieve, lado)).collect();
        let (niveles, mar) = rejillas.iter().map(Rejilla::modelos).unzip();
        let instancias = sembrar(&relieve, &rejillas);
        Chapin { niveles, mar, instancias }
    }

}

/// El reparto de los objetos: puntos al azar (con la semilla) y campos de ruido que deciden que
/// va en cada uno (o si no va nada). Cada objeto se para sobre los voxels de cada nivel.
fn sembrar(relieve: &Relieve, rejillas: &[Rejilla]) -> Vec<Instancia> {
    let params = relieve.params;
    let mut dados = Dados(params.semilla.wrapping_mul(2_891_336_453) ^ 0x5eed);
    struct Candidato {
        p: Vec3,
        h: f32,
        pendiente: f32,
        selva: f32,
        pueblo: f32,
        volcan: bool,
    }
    let candidatos: Vec<Candidato> = (0..params.puntos)
        .map(|_| {
            let p = dados.en_la_esfera();
            let h = relieve.altura(p);
            let n = relieve.normal(p);
            Candidato {
                p,
                h,
                pendiente: 1.0 - n.dot(p),
                selva: relieve.selva.fbm(p * 3.2, 3),
                pueblo: relieve.pueblos.ruido(p * 2.4 + Vec3::new(7.0, 1.0, 3.0)),
                volcan: relieve.volcanes.iter().any(|&(v, _)| p.dot(v).clamp(-1.0, 1.0).acos() < 0.2),
            }
        })
        .collect();
    let mut puestos: Vec<Instancia> = Vec::new();
    let poner = |tipo: Tipo, c: &Candidato, giro: f32, color: usize, puestos: &mut Vec<Instancia>| {
        // Que no quede encima de otro.
        let libre = puestos.iter().all(|o| o.posicion.normalize().dot(c.p).clamp(-1.0, 1.0).acos() > tipo.separacion().max(o.tipo.separacion()) * 0.8);
        if !libre {
            return false;
        }
        // Crecen hacia afuera del planeta (su "arriba" es la direccion desde el centro), con un
        // giro al azar alrededor de ese eje.
        let arriba = c.p;
        let frente = girar(tangentes(c.p).0, c.p, giro);
        let escala = tipo.tamano() * (0.8 + 0.45 * ((giro * 7.3).sin() * 0.5 + 0.5));
        // Paradas donde su linea toca los voxels de cada nivel, un poquito enterradas (los
        // voxels son escalones: asi no queda flotando una orilla de la base).
        let locales: [Mat4; 5] = std::array::from_fn(|n| colocar(rejillas[n].superficie(c.p) - arriba * (escala * 0.1), arriba, frente, escala));
        let posicion = Vec3::new(locales[4].m[0][3], locales[4].m[1][3], locales[4].m[2][3]);
        puestos.push(Instancia { tipo, locales, posicion, color });
        true
    };
    // Los monumentos primero: Tikal en lo mas metido de la selva; el Arco, en el pueblo mas grande.
    let en_tierra = |c: &&Candidato| c.h > 0.06 && c.h < 0.42 && c.pendiente < 0.12 && !c.volcan;
    if let Some(c) = candidatos.iter().filter(en_tierra).max_by(|a, b| a.selva.total_cmp(&b.selva)) {
        poner(Tipo::Tikal, c, 0.3, 0, &mut puestos);
    }
    if let Some(c) = candidatos.iter().filter(en_tierra).max_by(|a, b| a.pueblo.total_cmp(&b.pueblo)) {
        poner(Tipo::Arco, c, 1.1, 0, &mut puestos);
    }
    for (k, c) in candidatos.iter().enumerate() {
        if c.h < 0.05 {
            continue; // agua o playa
        }
        let azar = dados.uno();
        let giro = dados.uno() * TAU;
        let tipo = if c.volcan || c.pendiente > 0.3 || c.h > 0.62 {
            (azar < 0.55).then_some(Tipo::Roca)
        } else if c.pueblo > 0.22 && c.pendiente < 0.15 && c.h < 0.4 {
            Some(Tipo::Casita)
        } else if c.selva > 0.02 {
            (azar < 0.85).then_some(Tipo::Ceiba)
        } else {
            (azar < 0.18).then_some(Tipo::Roca)
        };
        if let Some(tipo) = tipo {
            poner(tipo, c, giro, k % COLORES_ANTIGUA.len(), &mut puestos);
        }
    }
    puestos
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn los_rectangulos_cubren_cada_celda_una_vez() {
        let celdas: HashSet<(i32, i32)> = [(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2), (3, 3), (2, 0)].into_iter().collect();
        let r = rectangulos(&celdas);
        let mut cubiertas = HashSet::new();
        for (b0, c0, alto, ancho) in r.iter().copied() {
            for b in b0..b0 + alto {
                for c in c0..c0 + ancho {
                    assert!(celdas.contains(&(b, c)), "se salio");
                    assert!(cubiertas.insert((b, c)), "encimado");
                }
            }
        }
        assert_eq!(cubiertas, celdas);
        assert!(r.len() <= 3, "el bloque de 2 x 3 va junto: {r:?}");
    }

    #[test]
    fn la_misma_semilla_da_el_mismo_planeta_y_otra_semilla_otro() {
        let (a, b) = (Chapin::new(PARAMETROS), Chapin::new(PARAMETROS));
        assert_eq!(a.instancias.len(), b.instancias.len());
        assert!(a.instancias.iter().zip(&b.instancias).all(|(x, y)| x.tipo == y.tipo && (x.posicion - y.posicion).length() < 1e-6));
        assert_eq!(a.niveles[2].partes.iter().map(|p| p.triangulos.len()).sum::<usize>(), b.niveles[2].partes.iter().map(|p| p.triangulos.len()).sum::<usize>());
        let c = Chapin::new(Parametros { semilla: 77, ..PARAMETROS });
        assert!(c.instancias.iter().zip(&a.instancias).any(|(x, y)| (x.posicion - y.posicion).length() > 1e-3));
    }

    /// Cada objeto, en cada nivel de detalle, se para donde su linea toca los voxels (abajo hay
    /// suelo y arriba aire), y crece hacia afuera del planeta.
    #[test]
    fn hay_de_todos_los_tipos_parados_sobre_los_voxels() {
        let ch = Chapin::new(PARAMETROS);
        let relieve = Relieve::new(PARAMETROS);
        for tipo in Tipo::TODOS {
            assert!(ch.instancias.iter().any(|i| i.tipo == tipo), "falta {tipo:?}");
        }
        let rejillas: Vec<Rejilla> = VOXELES.iter().map(|&l| Rejilla::new(&relieve, l)).collect();
        for i in &ch.instancias {
            // Los objetos solo se dibujan de cerca (del nivel 2 para arriba; ver `main.rs`).
            for (n, rejilla) in rejillas.iter().enumerate().skip(2) {
                let m = &i.locales[n];
                let y = Vec3::new(m.m[0][1], m.m[1][1], m.m[2][1]);
                let escala = y.length();
                let y = y * (1.0 / escala);
                // El punto donde toca (sin lo enterrado).
                let toca = Vec3::new(m.m[0][3], m.m[1][3], m.m[2][3]) + y * (escala * 0.1);
                assert!(y.dot(toca.normalize()) > 0.999, "{:?} no crece hacia afuera", i.tipo);
                let abajo = rejilla.celda_de(toca - y * (rejilla.u * 0.1));
                let encima = rejilla.celda_de(toca + y * (rejilla.u * 0.1));
                assert!(rejilla.en(abajo).is_some(), "{:?} flota en el nivel {n}", i.tipo);
                assert!(rejilla.en(encima).is_none(), "{:?} esta enterrado en el nivel {n}", i.tipo);
            }
            assert!(relieve.altura(i.posicion.normalize()) > 0.0);
        }
    }

    #[test]
    fn los_niveles_de_detalle_tienen_cada_vez_mas_triangulos() {
        let ch = Chapin::new(PARAMETROS);
        let cuantos: Vec<usize> = ch.niveles.iter().map(|m| m.partes.iter().map(|p| p.triangulos.len()).sum()).collect();
        assert!(cuantos.windows(2).all(|w| w[0] < w[1]), "{cuantos:?}");
        // Y todos los materiales aparecen en el nivel fino (la lava de los volcanes incluida).
        for s in Suelo::TODOS.iter().filter(|s| **s != Suelo::Agua) {
            assert!(ch.niveles[4].partes.iter().any(|p| p.nombre == s.nombre()), "falta {s:?}");
        }
        assert!(!ch.mar[4].partes.is_empty());
    }

    /// La deformacion y la generacion son dos etapas: con relieve 0 la rejilla genera una esfera
    /// de voxels lisa (nada pasa de radio 1); con relieve, el ruido la deforma (montanas y
    /// volcanes salen de la esfera) con la misma generacion.
    #[test]
    fn sin_relieve_es_una_esfera_y_el_ruido_la_deforma() {
        let mas_lejos = |relieve: f32| {
            let r = Relieve::new(Parametros { relieve, ..PARAMETROS });
            let (tierra, _) = Rejilla::new(&r, VOXELES[3]).modelos();
            tierra.partes.iter().flat_map(|p| p.triangulos.iter().flatten()).map(|v| v.pos.length()).fold(0.0f32, f32::max)
        };
        let u = celda(3);
        // Una esfera de voxels: sus esquinas llegan a lo mucho a medio voxel en diagonal.
        let lisa = mas_lejos(0.0);
        assert!(lisa <= 1.0 + u * 0.9, "sin relieve llega a {lisa}");
        let deformada = mas_lejos(PARAMETROS.relieve);
        assert!(deformada > 1.0 + PARAMETROS.relieve * 0.7, "con relieve solo llega a {deformada}");
        // Los extremos de F7/F8 (en vivo) arman el planeta completo, con sus objetos.
        for relieve in [0.0, 0.4] {
            let t = std::time::Instant::now();
            let ch = Chapin::new(Parametros { relieve, ..PARAMETROS });
            assert!(!ch.instancias.is_empty() && ch.niveles.iter().all(|m| !m.partes.is_empty()));
            assert!(t.elapsed().as_secs_f32() < 2.0, "tardo {:?} en armarse", t.elapsed());
        }
    }

    /// Las caras de la tierra no se meten bajo el agua ni hay caras entre dos voxels de tierra.
    #[test]
    fn solo_se_ven_las_caras_que_dan_al_aire() {
        let relieve = Relieve::new(PARAMETROS);
        let rejilla = Rejilla::new(&relieve, VOXELES[1]);
        let (tierra, _) = rejilla.modelos();
        for parte in &tierra.partes {
            for t in &parte.triangulos {
                let centro = (t[0].pos + t[1].pos + t[2].pos) * (1.0 / 3.0);
                let afuera = rejilla.celda_de(centro + t[0].normal * (rejilla.u * 0.5));
                assert!(rejilla.en(afuera).is_none(), "una cara de {} da a {:?}", parte.nombre, rejilla.en(afuera));
            }
        }
    }
}
