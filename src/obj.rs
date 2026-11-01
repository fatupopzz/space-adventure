//! Lector de `.obj` y `.mtl`, a mano.
//!
//! Lee lo que usan los modelos del proyecto: `v`, `vn`, `vt`, caras `f` (en
//! formato `v//vn`, `v/vt/vn` o solo `v`), `o` para separar partes,
//! `usemtl`, `mtllib`, y los comentarios `#`, que en estos modelos traen
//! datos utiles (pivotes, ejes, donde va la piloto). Del `.mtl`, ademas de
//! los colores, `map_Kd`: la textura del color difuso.
//!
//! Cada `o` del archivo queda como una `Parte` con su propio material. Eso
//! es lo que permite dibujar solo algunas (las caras de una expresion) o
//! moverlas por separado (las coletas, las aspas de las turbinas).

use crate::matematica::Vec3;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Una textura ya en memoria: los colores de cada texel en [0, 1].
/// raylib solo lee el PNG; el muestreo lo hace el fragment shader del
/// pipeline (`Textura::muestrear`).
pub struct Textura {
    pub ancho: usize,
    pub alto: usize,
    texels: Vec<Vec3>,
}

impl std::fmt::Debug for Textura {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Textura {}x{}", self.ancho, self.alto)
    }
}

impl Textura {
    /// Una textura hecha en el juego (los texels ya en [0, 1], por filas desde arriba).
    pub fn desde(ancho: usize, alto: usize, texels: Vec<Vec3>) -> Textura {
        assert_eq!(texels.len(), ancho * alto);
        Textura { ancho, alto, texels }
    }

    pub fn cargar(ruta: &str) -> Result<Textura, String> {
        let imagen = raylib::core::texture::Image::load_image(ruta).map_err(|e| format!("no se pudo leer {ruta}: {e}"))?;
        let (ancho, alto) = (imagen.width() as usize, imagen.height() as usize);
        let texels = imagen.get_image_data().iter().map(|c| Vec3::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0)).collect();
        Ok(Textura { ancho, alto, texels })
    }

    fn texel(&self, x: isize, y: isize) -> Vec3 {
        let x = x.rem_euclid(self.ancho as isize) as usize;
        let y = y.rem_euclid(self.alto as isize) as usize;
        self.texels[y * self.ancho + x]
    }

    /// El color en la coordenada (u, v) con filtro BILINEAL: se mezclan los
    /// cuatro texels que rodean al punto segun que tan cerca queda de cada
    /// uno. La v del `.obj` crece hacia arriba y las filas de la imagen hacia
    /// abajo, por eso se voltea. Fuera de [0, 1] la textura se repite.
    pub fn muestrear(&self, u: f32, v: f32) -> Vec3 {
        let x = u * self.ancho as f32 - 0.5;
        let y = (1.0 - v) * self.alto as f32 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let (x0, y0) = (x0 as isize, y0 as isize);
        let arriba = self.texel(x0, y0) * (1.0 - fx) + self.texel(x0 + 1, y0) * fx;
        let abajo = self.texel(x0, y0 + 1) * (1.0 - fx) + self.texel(x0 + 1, y0 + 1) * fx;
        arriba * (1.0 - fy) + abajo * fy
    }
}

/// Un material del `.mtl`. Los nombres son los del formato.
#[derive(Clone, Debug)]
pub struct Material {
    pub nombre: String,
    /// Color difuso.
    pub kd: Vec3,
    /// Color especular.
    pub ks: Vec3,
    /// Exponente especular: mas alto, brillo mas chico y mas duro.
    pub ns: f32,
    /// Color emisivo: luz propia, se suma sin importar la iluminacion.
    pub ke: Vec3,
    /// Opacidad: 1 opaco, menos de 1 se dibuja con blending.
    pub d: f32,
    /// La textura del color difuso (`map_Kd`), si tiene: su color se multiplica por `kd`.
    pub textura: Option<Arc<Textura>>,
}

impl Material {
    pub fn nuevo(nombre: &str) -> Self {
        Material {
            nombre: nombre.to_string(),
            kd: Vec3::new(0.8, 0.8, 0.8),
            ks: Vec3::zero(),
            ns: 1.0,
            ke: Vec3::zero(),
            d: 1.0,
            textura: None,
        }
    }

    pub fn es_transparente(&self) -> bool {
        self.d < 1.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vertice {
    pub pos: Vec3,
    pub normal: Vec3,
    /// Coordenada de textura (u, v); (0, 0) si el modelo no trae.
    pub uv: [f32; 2],
}

/// Un grupo `o` del archivo: un nombre, un material y sus triangulos.
#[derive(Clone, Debug)]
pub struct Parte {
    pub nombre: String,
    pub material: usize,
    pub triangulos: Vec<[Vertice; 3]>,
    /// El comentario que viene justo despues de su `o` (pivotes, ejes).
    pub comentario: String,
}

#[derive(Clone, Debug)]
pub struct Modelo {
    pub partes: Vec<Parte>,
    pub materiales: Vec<Material>,
    /// Las lineas de comentario del `.obj`, sin el `#`.
    pub comentarios: Vec<String>,
    /// La esfera que lo envuelve, calculada la primera vez que se pide (ver `esfera`). Quien le
    /// cambie las partes despues, la reinicia (`OnceLock::new()`).
    pub esfera_guardada: std::sync::OnceLock<(Vec3, f32)>,
}

impl Modelo {
    /// La esfera que envuelve todo el modelo (centro y radio, en coordenadas del modelo): la usa
    /// el pipeline para descartar objetos enteros que quedan fuera de la vista.
    pub fn esfera(&self) -> (Vec3, f32) {
        *self.esfera_guardada.get_or_init(|| self.calcular_esfera())
    }

    /// Un modelo hecho en el juego (sin `.obj`).
    pub fn nuevo(partes: Vec<Parte>, materiales: Vec<Material>) -> Modelo {
        Modelo { partes, materiales, comentarios: Vec::new(), esfera_guardada: Default::default() }
    }

    fn calcular_esfera(&self) -> (Vec3, f32) {
        let mut minimo = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut maximo = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in self.partes.iter().flat_map(|p| p.triangulos.iter()).flat_map(|t| t.iter()) {
            minimo = Vec3::new(minimo.x.min(v.pos.x), minimo.y.min(v.pos.y), minimo.z.min(v.pos.z));
            maximo = Vec3::new(maximo.x.max(v.pos.x), maximo.y.max(v.pos.y), maximo.z.max(v.pos.z));
        }
        if minimo.x > maximo.x {
            return (Vec3::zero(), 0.0);
        }
        let centro = (minimo + maximo) * 0.5;
        let radio = self.partes.iter().flat_map(|p| p.triangulos.iter()).flat_map(|t| t.iter()).map(|v| (v.pos - centro).length()).fold(0.0, f32::max);
        (centro, radio)
    }

    pub fn cargar(ruta: &str) -> Result<Modelo, String> {
        let texto = fs::read_to_string(ruta).map_err(|e| format!("no se pudo leer {ruta}: {e}"))?;
        let carpeta = Path::new(ruta).parent().unwrap_or(Path::new("."));

        let mut posiciones: Vec<Vec3> = Vec::new();
        let mut normales: Vec<Vec3> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut materiales: Vec<Material> = Vec::new();
        let mut partes: Vec<Parte> = Vec::new();
        let mut comentarios = Vec::new();
        let mut nombre_actual = String::from("sin_nombre");

        // Busca un material por nombre; si el .mtl no lo trae, lo crea gris
        // para que el modelo igual se vea.
        let indice_material = |materiales: &mut Vec<Material>, nombre: &str| -> usize {
            if let Some(i) = materiales.iter().position(|m| m.nombre == nombre) {
                return i;
            }
            materiales.push(Material::nuevo(nombre));
            materiales.len() - 1
        };

        for (n, linea) in texto.lines().enumerate() {
            let linea = linea.trim();
            if let Some(c) = linea.strip_prefix('#') {
                // Un comentario antes de la primera cara de una parte es de esa parte.
                match partes.last_mut() {
                    Some(p) if p.triangulos.is_empty() => p.comentario = c.trim().to_string(),
                    _ => comentarios.push(c.trim().to_string()),
                }
                continue;
            }
            let mut campos = linea.split_whitespace();
            let Some(tipo) = campos.next() else { continue };
            let resto: Vec<&str> = campos.collect();
            let error = |que: &str| format!("{ruta}:{}: {que}", n + 1);

            match tipo {
                "v" | "vn" => {
                    let xyz: Vec<f32> = resto.iter().take(3).map(|s| s.parse().unwrap_or(0.0)).collect();
                    if xyz.len() < 3 {
                        return Err(error("faltan coordenadas"));
                    }
                    let v = Vec3::new(xyz[0], xyz[1], xyz[2]);
                    if tipo == "v" { posiciones.push(v) } else { normales.push(v.normalize()) }
                }
                "vt" => {
                    let u: Vec<f32> = resto.iter().take(2).map(|s| s.parse().unwrap_or(0.0)).collect();
                    uvs.push([u.first().copied().unwrap_or(0.0), u.get(1).copied().unwrap_or(0.0)]);
                }
                "mtllib" => {
                    if let Some(archivo) = resto.first() {
                        let ruta_mtl = carpeta.join(archivo);
                        cargar_mtl(&ruta_mtl.to_string_lossy(), &mut materiales)?;
                    }
                }
                "o" | "g" => {
                    nombre_actual = resto.first().unwrap_or(&"sin_nombre").to_string();
                    let material = partes.last().map_or(0, |p| p.material);
                    partes.push(Parte { nombre: nombre_actual.clone(), material, triangulos: Vec::new(), comentario: String::new() });
                }
                "usemtl" => {
                    let i = indice_material(&mut materiales, resto.first().unwrap_or(&"default"));
                    match partes.last_mut() {
                        // Cambia el material antes de la primera cara: es el de la parte.
                        Some(p) if p.triangulos.is_empty() => p.material = i,
                        // Cambia a la mitad: sigue la misma parte con otro material.
                        _ => partes.push(Parte { nombre: nombre_actual.clone(), material: i, triangulos: Vec::new(), comentario: String::new() }),
                    }
                }
                "f" => {
                    if partes.is_empty() {
                        let i = indice_material(&mut materiales, "default");
                        partes.push(Parte { nombre: nombre_actual.clone(), material: i, triangulos: Vec::new(), comentario: String::new() });
                    }
                    let mut vs = Vec::with_capacity(resto.len());
                    for campo in &resto {
                        let mut idx = campo.split('/');
                        let iv = resolver(idx.next(), posiciones.len()).ok_or_else(|| error("indice de vertice invalido"))?;
                        let ivt = resolver(idx.next(), uvs.len());
                        let inor = resolver(idx.next(), normales.len());
                        vs.push((posiciones[iv], inor.map(|i| normales[i]), ivt.map_or([0.0, 0.0], |i| uvs[i])));
                    }
                    if vs.len() < 3 {
                        return Err(error("cara con menos de 3 vertices"));
                    }
                    // Poligonos de mas de 3 lados: abanico desde el primero.
                    for k in 1..vs.len() - 1 {
                        let (a, b, c) = (vs[0], vs[k], vs[k + 1]);
                        // Sin normal en el archivo: la de la cara, con el orden CCW.
                        let cara = (b.0 - a.0).cross(c.0 - a.0).normalize();
                        let v = |p: (Vec3, Option<Vec3>, [f32; 2])| Vertice { pos: p.0, normal: p.1.unwrap_or(cara), uv: p.2 };
                        partes.last_mut().unwrap().triangulos.push([v(a), v(b), v(c)]);
                    }
                }
                _ => {}
            }
        }

        partes.retain(|p| !p.triangulos.is_empty());
        if materiales.is_empty() {
            materiales.push(Material::nuevo("default"));
        }
        Ok(Modelo { partes, materiales, comentarios, esfera_guardada: Default::default() })
    }

    /// El texto de un comentario que empieza con `clave:`, por ejemplo
    /// `piloto:` en `nave_lila.obj`.
    pub fn comentario(&self, clave: &str) -> Option<&str> {
        self.comentarios
            .iter()
            .find_map(|c| c.strip_prefix(clave).and_then(|r| r.strip_prefix(':')).map(str::trim))
    }
}

/// Indice del `.obj` (empieza en 1, negativo cuenta desde el final) a indice
/// de arreglo. `None` si no hay o se sale.
fn resolver(campo: Option<&str>, total: usize) -> Option<usize> {
    let i: i64 = campo.filter(|s| !s.is_empty())?.parse().ok()?;
    let i = if i < 0 { total as i64 + i } else { i - 1 };
    (0..total as i64).contains(&i).then_some(i as usize)
}

fn cargar_mtl(ruta: &str, materiales: &mut Vec<Material>) -> Result<(), String> {
    let texto = fs::read_to_string(ruta).map_err(|e| format!("no se pudo leer {ruta}: {e}"))?;
    let carpeta = Path::new(ruta).parent().unwrap_or(Path::new("."));
    for linea in texto.lines() {
        let campos: Vec<&str> = linea.split_whitespace().collect();
        let Some((&tipo, resto)) = campos.split_first() else { continue };
        let num = |i: usize| resto.get(i).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
        let color = || Vec3::new(num(0), num(1), num(2));
        if tipo == "newmtl" {
            materiales.push(Material::nuevo(resto.first().unwrap_or(&"default")));
            continue;
        }
        let Some(m) = materiales.last_mut() else { continue };
        match tipo {
            "Kd" => m.kd = color(),
            "Ks" => m.ks = color(),
            "Ke" => m.ke = color(),
            "Ns" => m.ns = num(0),
            "d" => m.d = num(0),
            "Tr" => m.d = 1.0 - num(0),
            "map_Kd" => {
                if let Some(archivo) = resto.first() {
                    m.textura = Some(Arc::new(Textura::cargar(&carpeta.join(archivo).to_string_lossy())?));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Una textura de 2x2 hecha a mano: rojo, verde / azul, blanco.
    fn cuadros() -> Textura {
        let c = |r, g, b| Vec3::new(r, g, b);
        Textura { ancho: 2, alto: 2, texels: vec![c(1.0, 0.0, 0.0), c(0.0, 1.0, 0.0), c(0.0, 0.0, 1.0), c(1.0, 1.0, 1.0)] }
    }

    #[test]
    fn muestreo_de_texturas() {
        let t = cuadros();
        // En el centro de cada texel sale su color (v crece hacia arriba: la fila de arriba es v alta).
        assert!((t.muestrear(0.25, 0.75) - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-5);
        assert!((t.muestrear(0.75, 0.25) - Vec3::new(1.0, 1.0, 1.0)).length() < 1e-5);
        // Entre dos texels, el bilineal los mezcla a la mitad.
        assert!((t.muestrear(0.5, 0.75) - Vec3::new(0.5, 0.5, 0.0)).length() < 1e-5);
    }

    /// El pizarron del salon trae coordenadas UV y su textura.
    #[test]
    fn el_pizarron_trae_uv_y_textura() {
        let aula = Modelo::cargar("modelos/aula_uvg.obj").unwrap();
        let p = aula.partes.iter().find(|p| p.nombre == "pizarra_dibujo").expect("falta el pizarron");
        let m = &aula.materiales[p.material];
        assert!(m.textura.as_ref().is_some_and(|t| t.ancho == 1024 && t.alto == 288));
        let us: Vec<f32> = p.triangulos.iter().flatten().map(|v| v.uv[0]).collect();
        assert!(us.iter().any(|&u| u == 0.0) && us.iter().any(|&u| u == 1.0));
    }
}
