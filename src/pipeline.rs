//! El pipeline de render por software, etapa por etapa.
//!
//! Cada triangulo de cada modelo pasa por las mismas etapas que en la GPU:
//!
//! 1. **Vertex shader**: lleva cada vertice del modelo al mundo (matriz M) y
//!    al espacio de recorte (matrices V y P).
//! 2. **Primitive assembly**: junta los tres vertices en un triangulo.
//! 3. **Clipping**: descarta lo que queda fuera de la vista y recorta contra
//!    el plano cercano lo que la camara tiene casi encima.
//! 4. **Perspective divide y viewport**: divide entre `w` y pasa a pixeles.
//! 5. **Face culling**: tira los triangulos que dan la espalda a la camara.
//! 6. **Rasterizacion**: recorre los pixeles del triangulo con coordenadas
//!    baricentricas e interpola (con correccion de perspectiva).
//! 7. **Fragment shader**: calcula el color con la luz del Sol y, si el
//!    material tiene textura, la muestrea en la UV interpolada.
//! 8. **Operaciones por fragmento**: stencil test (si el draw call usa la
//!    mascara), depth test con el z-buffer, y blending: lo transparente se
//!    mezcla con lo que ya habia.
//!
//! Los uniforms son lo que es igual para todo un draw call: las matrices,
//! la posicion de la luz y la de la camara.

use crate::animacion::{Animaciones, EstadoAnim};
use crate::framebuffer::{Banda, Framebuffer};
use crate::matematica::{Mat4, Vec3, Vec4};
use crate::obj::{Material, Modelo, Parte, Vertice};


#[derive(Clone, Copy)]
pub struct Uniforms {
    pub modelo: Mat4,
    pub vista: Mat4,
    pub proyeccion: Mat4,
    /// Donde esta el Sol en el mundo: de ahi sale la luz.
    pub luz: Vec3,
    /// Donde esta la camara en el mundo, para el brillo especular.
    pub camara: Vec3,
    /// Luz que reciben las caras que no ven la luz principal. En el espacio
    /// es lila, como la nebulosa; en el salon, la de las paredes.
    pub ambiente: Vec3,
    /// Luz de borde: las caras que la camara ve de canto se encienden con
    /// este color. Separa las siluetas del fondo oscuro.
    pub borde: Vec3,
    /// El color de la luz principal (blanca casi siempre; azulita la de una pantalla).
    pub color_luz: Vec3,
    /// Hasta donde llega la luz (se apaga con la distancia, como un foco); 0: llega a todo (el Sol).
    pub alcance: f32,
    /// Que fragment shader usa este draw call (ver `Shader`).
    pub shader: Shader,
    /// Segundos, para los shaders animados (la lava, las olas, el plasma del Sol).
    pub tiempo: f32,
    /// Para los shaders de planeta: su centro en el mundo, su eje de giro (el "arriba" del
    /// planeta, para la tangente de las olas) y el radio del nivel del mar.
    pub centro: Vec3,
    pub eje: Vec3,
    pub radio: f32,
    /// La fuerza del efecto del shader (el relieve del mapa de normales, lo alto de las olas, lo
    /// vivo del plasma): 0 lo apaga. Se puede cambiar en vivo (ver `chapin.rs`).
    pub intensidad: f32,
    /// Para el terreno voxel: lo que mide un voxel en el espacio del modelo (cada cara lleva el
    /// mapa de normales una vez).
    pub celda: f32,
}

/// Los fragment shaders del pipeline: cada draw call escoge el suyo en sus uniforms. Cada uno
/// calcula el color de otra manera (no es el mismo con otro color).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Shader {
    /// Blinn-Phong con luz de borde y emisivo; en el estilo "dibujado", cel shading en tres bandas.
    #[default]
    Phong,
    /// Terreno de planeta voxel: el color de cada voxel (con variacion por bloque), con MAPA DE
    /// NORMALES en cada cara (bisel y granito en espacio tangente, con su TBN) y lava animada.
    Terreno,
    /// Mar: olas animadas (trenes de ondas que inclinan la normal con el tiempo), reflejo con
    /// Fresnel (de canto refleja el cielo) y el brillo del Sol sobre el agua.
    Oceano,
    /// Plasma del Sol: ruido fBm animado y deformado con otro ruido (*domain warping*), con el borde
    /// mas oscuro (como el Sol de verdad). Brilla solo: no lo ilumina nada.
    Plasma,
}

/// Como se ensamblan las primitivas: triangulos rellenos, solo sus aristas
/// o solo sus vertices.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModoDibujo {
    Relleno,
    Alambre,
    Puntos,
}

/// Que se ve en pantalla: la imagen normal, o los datos que viajan por el
/// pipeline pintados como color (para ver cada etapa por dentro).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vista {
    Color,
    /// El z-buffer: mas claro, mas cerca.
    Profundidad,
    /// La normal interpolada de cada fragmento, de [-1, 1] a [0, 1] en RGB.
    Normales,
    /// La coordenada UV interpolada (rojo = u, verde = v) donde hay textura.
    Uv,
    /// El stencil buffer: lo marcado se ve encima de la imagen en gris.
    Stencil,
}

pub struct Opciones {
    pub modo: ModoDibujo,
    pub culling: bool,
    pub vista: Vista,
}

/// Como usa un draw call el STENCIL BUFFER.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stencil {
    /// No lo toca.
    Nada,
    /// Marca la mascara con este valor donde pasa el depth test, sin pintar
    /// color, y "abre" el z-buffer ahi (lo deja al fondo) para que lo que se
    /// dibuje despues se vea a traves, como por una ventana.
    Marcar(u8),
    /// Solo dibuja donde la mascara vale esto (el stencil test).
    Solo(u8),
    /// Sin depth test: lo que se dibuja despues tapa a lo de antes, este adelante o atras (lo
    /// que pasa si se olvida el z-buffer; es uno de los bugs del minijuego de la Tierra).
    SinZBuffer,
}

#[derive(Default)]
pub struct Estadisticas {
    /// Triangulos que entraron al pipeline.
    pub triangulos: usize,
    /// Los que sobrevivieron al clipping y al culling y se rasterizaron.
    pub dibujados: usize,
    /// Objetos (draw calls) que se pidieron dibujar, y cuantos se descartaron enteros por quedar
    /// fuera de la vista (sin pasar ni un vertice por el vertex shader).
    pub objetos: usize,
    pub descartados: usize,
    /// Objetos que ni se pidieron por verse demasiado chiquitos (nivel de detalle por objeto).
    pub por_detalle: usize,
}

/// Un draw call: un modelo, sus uniforms, y que expresion mostrar si el
/// modelo trae caras (`cara_<expresion>_*`).
pub struct DrawCall<'a> {
    pub modelo: &'a Modelo,
    pub uniforms: Uniforms,
    pub expresion: Option<&'a str>,
    /// Las partes que se mueven solas (llamas, turbinas, coletas).
    pub animacion: Option<(&'a Animaciones, EstadoAnim)>,
    /// Partes que no se dibujan, por el principio de su nombre.
    pub ocultar: &'a [&'a str],
    /// Una matriz extra por parte, calculada afuera (el baile de Miku).
    pub por_parte: Option<&'a [Mat4]>,
    /// El modo de dibujo de este objeto, si no es el de todos (Pluton en alambre).
    pub modo: Option<ModoDibujo>,
    pub stencil: Stencil,
}

/// Lo que sale del vertex shader.
#[derive(Clone, Copy)]
struct VerticeRecorte {
    clip: Vec4,
    mundo: Vec3,
    normal: Vec3,
    /// La UV viaja como un varying mas (se guarda en un Vec3 con z = 0).
    uv: Vec3,
}

impl VerticeRecorte {
    fn lerp(self, o: VerticeRecorte, t: f32) -> VerticeRecorte {
        VerticeRecorte {
            clip: self.clip.lerp(o.clip, t),
            mundo: self.mundo.lerp(o.mundo, t),
            normal: self.normal.lerp(o.normal, t),
            uv: self.uv.lerp(o.uv, t),
        }
    }
}

/// Un vertice ya en pixeles. Los atributos van tambien multiplicados por
/// `1/w` para interpolarlos con correccion de perspectiva.
#[derive(Clone, Copy)]
struct VerticePantalla {
    x: f32,
    y: f32,
    /// Profundidad en [0, 1].
    z: f32,
    inv_w: f32,
    mundo: Vec3,
    normal: Vec3,
    uv: Vec3,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pasada {
    Opaca,
    Transparente,
}

/// Un triangulo ya en pantalla, listo para rasterizar.
struct Listo<'a> {
    p: [VerticePantalla; 3],
    mat: &'a Material,
    /// Indice en la lista de uniforms (uno por parte, no por triangulo).
    u: usize,
    pasada: Pasada,
    stencil: Stencil,
}

/// Dibuja todos los draw calls: primero todo lo opaco, despues lo
/// transparente encima (que tiene que ver lo opaco para mezclarse).
///
/// En modo relleno va en dos tiempos. Primero la GEOMETRIA (vertex shader,
/// clipping, viewport, culling) arma la lista de triangulos en pantalla.
/// Despues la RASTERIZACION se reparte: la imagen se parte en franjas
/// horizontales y cada hilo pinta la suya recorriendo la lista entera en el
/// mismo orden, asi lo opaco sigue antes que lo transparente en cada pixel.
pub fn render(fb: &mut Framebuffer, llamadas: &[DrawCall], op: &Opciones) -> Estadisticas {
    let mut st = Estadisticas::default();
    let mut uniforms: Vec<Uniforms> = Vec::new();
    let mut listos: Vec<Listo> = Vec::new();
    for pasada in [Pasada::Opaca, Pasada::Transparente] {
        for ll in llamadas {
            geometria(fb, ll, pasada, op, &mut st, &mut uniforms, &mut listos);
        }
    }
    if listos.is_empty() {
        return st;
    }
    let hilos = std::thread::available_parallelism().map_or(4, |n| n.get());
    let mut bandas = fb.bandas(hilos * 2);
    let (listos, uniforms) = (&listos, &uniforms);
    std::thread::scope(|s| {
        for banda in bandas.iter_mut() {
            s.spawn(move || {
                for t in listos {
                    rellenar(banda, t, &uniforms[t.u], op.vista);
                }
            });
        }
    });
    st
}

/// Las partes `cara_*` solo se dibujan si son de la expresion pedida.
fn visible(parte: &Parte, expresion: Option<&str>) -> bool {
    match parte.nombre.strip_prefix("cara_") {
        None => true,
        Some(resto) => expresion.is_some_and(|e| resto.strip_prefix(e).is_some_and(|r| r.starts_with('_'))),
    }
}

/// Las etapas de geometria de un draw call. En relleno deja los triangulos
/// en `listos`; en alambre y puntos los dibuja de una vez (son modos para
/// ver la malla, no hace falta repartirlos).
fn geometria<'a>(
    fb: &mut Framebuffer,
    ll: &DrawCall<'a>,
    pasada: Pasada,
    op: &Opciones,
    st: &mut Estadisticas,
    uniformes: &mut Vec<Uniforms>,
    listos: &mut Vec<Listo<'a>>,
) {
    let vista_proyeccion = ll.uniforms.proyeccion * ll.uniforms.vista;
    // Descarte de objetos: lo que queda entero fuera de la vista ni entra al vertex shader.
    if pasada == Pasada::Opaca {
        st.objetos += 1;
    }
    if descartar() && fuera_de_vista(ll, &vista_proyeccion) {
        if pasada == Pasada::Opaca {
            st.descartados += 1;
        }
        return;
    }
    for (j, parte) in ll.modelo.partes.iter().enumerate() {
        let mat = &ll.modelo.materiales[parte.material];
        if !visible(parte, ll.expresion)
            || ll.ocultar.iter().any(|o| parte.nombre.starts_with(o))
            || mat.es_transparente() != (pasada == Pasada::Transparente)
        {
            continue;
        }
        // Una parte animada lleva su propia matriz entre la M y el vertice.
        let mut uniforms = ll.uniforms;
        if let Some(m) = ll.animacion.and_then(|(a, e)| a.matriz(j, &e)) {
            uniforms.modelo = uniforms.modelo * m;
        }
        if let Some(m) = ll.por_parte.and_then(|ms| ms.get(j)) {
            uniforms.modelo = uniforms.modelo * *m;
        }
        uniformes.push(uniforms);
        let iu = uniformes.len() - 1;
        let u = &uniforms;
        // Una vez por parte: la matriz de las normales (inversa transpuesta de M).
        let matriz_normal = u.modelo.matriz_normal();
        for tri in &parte.triangulos {
            st.triangulos += 1;
            let vs = tri.map(|v| vertex_shader(&v, u, &matriz_normal, &vista_proyeccion));
            let (recortados, n) = recortar(vs);
            for t in &recortados[..n] {
                let p = t.map(|v| a_pantalla(v, fb));
                if !pasa_culling(p, op) {
                    continue;
                }
                st.dibujados += 1;
                match ll.modo.unwrap_or(op.modo) {
                    ModoDibujo::Relleno => listos.push(Listo { p, mat, u: iu, pasada, stencil: ll.stencil }),
                    ModoDibujo::Alambre => {
                        for i in 0..3 {
                            linea(fb, p[i], p[(i + 1) % 3], mat, u);
                        }
                    }
                    ModoDibujo::Puntos => {
                        for v in p {
                            punto(fb, v, mat, u);
                        }
                    }
                }
            }
        }
    }
}

// ---------- 1. vertex shader ----------

/// Recibe el vertice con sus ATRIBUTOS (posicion, normal, UV) y los uniforms,
/// y devuelve la posicion en clip space (lo que en OpenGL seria
/// `gl_Position`) mas los varyings que siguen hacia el fragment shader.
fn vertex_shader(v: &Vertice, u: &Uniforms, matriz_normal: &Mat4, vista_proyeccion: &Mat4) -> VerticeRecorte {
    let mundo = u.modelo.transform_point(v.pos);
    // Las normales van con la inversa transpuesta de M, no con M.
    let normal = matriz_normal.transform_dir(v.normal).normalize();
    let clip = vista_proyeccion.mul_vec4(Vec4::new(mundo.x, mundo.y, mundo.z, 1.0));
    VerticeRecorte { clip, mundo, normal, uv: Vec3::new(v.uv[0], v.uv[1], 0.0) }
}

// ---------- 2 y 3. primitive assembly y clipping ----------

/// Recorta el triangulo. Devuelve 0, 1 o 2 triangulos: si un vertice queda
/// detras del plano cercano, el pedazo de adelante es un cuadrilatero.
fn recortar(t: [VerticeRecorte; 3]) -> ([[VerticeRecorte; 3]; 2], usize) {
    let vacio = ([t, t], 0);
    let c = t.map(|v| v.clip);
    // Fuera de la vista entero (los tres del mismo lado de un plano): se tira.
    let fuera = |f: fn(&Vec4) -> bool| c.iter().all(f);
    if fuera(|v| v.x > v.w) || fuera(|v| v.x < -v.w) || fuera(|v| v.y > v.w) || fuera(|v| v.y < -v.w)
        || fuera(|v| v.z > v.w) || fuera(|v| v.z < -v.w)
    {
        return vacio;
    }
    // Distancia con signo al plano cercano (z = -w); adentro si es >= 0.
    let d = c.map(|v| v.z + v.w);
    if d.iter().all(|&x| x >= 0.0) {
        return ([t, t], 1);
    }
    // Sutherland-Hodgman contra el plano cercano.
    let mut poli = [t[0]; 4];
    let mut n = 0;
    for i in 0..3 {
        let j = (i + 1) % 3;
        if d[i] >= 0.0 {
            poli[n] = t[i];
            n += 1;
        }
        if (d[i] >= 0.0) != (d[j] >= 0.0) {
            poli[n] = t[i].lerp(t[j], d[i] / (d[i] - d[j]));
            n += 1;
        }
    }
    match n {
        3 => ([[poli[0], poli[1], poli[2]], [poli[0], poli[1], poli[2]]], 1),
        4 => ([[poli[0], poli[1], poli[2]], [poli[0], poli[2], poli[3]]], 2),
        _ => vacio,
    }
}

// ---------- 4. perspective divide y viewport ----------

fn a_pantalla(v: VerticeRecorte, fb: &Framebuffer) -> VerticePantalla {
    let inv_w = 1.0 / v.clip.w;
    let (nx, ny, nz) = (v.clip.x * inv_w, v.clip.y * inv_w, v.clip.z * inv_w);
    VerticePantalla {
        x: (nx + 1.0) * 0.5 * fb.ancho as f32,
        // En pantalla Y crece hacia abajo.
        y: (1.0 - ny) * 0.5 * fb.alto as f32,
        z: nz * 0.5 + 0.5,
        inv_w,
        mundo: v.mundo * inv_w,
        normal: v.normal * inv_w,
        uv: v.uv * inv_w,
    }
}

/// Funcion de arista: positiva de un lado de la recta a-b, negativa del
/// otro, y en valor absoluto el doble del area del triangulo (a, b, p).
#[inline]
fn arista(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)
}

/// Regla arriba-izquierda: un pixel justo sobre una arista compartida es
/// de UN solo triangulo. Sin esto se pinta dos veces, y en lo transparente
/// se nota como una linea mas oscura en cada union.
#[inline]
fn arriba_izquierda(a: (f32, f32), b: (f32, f32)) -> bool {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    (dy == 0.0 && dx > 0.0) || dy < 0.0
}

// ---------- 5 a 8. culling, rasterizacion, fragment shader, depth test, blending ----------

/// FACE CULLING: si el triangulo sigue (no esta degenerado y, con culling
/// prendido, da la cara a la camara).
fn pasa_culling(p: [VerticePantalla; 3], op: &Opciones) -> bool {
    let area = arista((p[0].x, p[0].y), (p[1].x, p[1].y), (p[2].x, p[2].y));
    // CCW en el mundo queda con area negativa, porque en pantalla Y esta
    // al reves: eso es una cara de frente.
    area != 0.0 && !(op.culling && area > 0.0)
}

fn rellenar(fb: &mut Banda, t: &Listo, u: &Uniforms, vista: Vista) {
    let (p, mat, pasada) = (t.p, t.mat, t.pasada);
    // Se ordenan para que el area sea positiva; asi "adentro" es siempre
    // las tres aristas >= 0.
    let [a, mut b, mut c] = p;
    if arista((a.x, a.y), (b.x, b.y), (c.x, c.y)) < 0.0 {
        std::mem::swap(&mut b, &mut c);
    }
    let (pa, pb, pc) = ((a.x, a.y), (b.x, b.y), (c.x, c.y));
    // Si los tres vertices traen la misma normal (caras planas, como todo lo
    // voxel), se normaliza una vez por triangulo y no en cada pixel.
    let na = a.normal * (1.0 / a.inv_w);
    let plana = (na - b.normal * (1.0 / b.inv_w)).length() < 1e-3 && (na - c.normal * (1.0 / c.inv_w)).length() < 1e-3;
    let normal_plana = na.normalize();
    let area = arista(pa, pb, pc);
    let tl = [arriba_izquierda(pb, pc), arriba_izquierda(pc, pa), arriba_izquierda(pa, pb)];

    let x0 = a.x.min(b.x).min(c.x).floor().max(0.0) as usize;
    // Solo las filas de esta franja.
    let y0 = (a.y.min(b.y).min(c.y).floor().max(0.0) as usize).max(fb.y0);
    let x1 = (a.x.max(b.x).max(c.x).ceil() as isize).min(fb.ancho as isize - 1);
    let y1 = (a.y.max(b.y).max(c.y).ceil() as isize).min(fb.y1 as isize - 1);
    if x1 < 0 || y1 < 0 {
        return;
    }

    let aristas = [(pb, pc), (pc, pa), (pa, pb)];
    for y in y0..=y1 as usize {
        // En cada fila, cada arista es una recta en x: se despeja de donde a
        // donde queda del lado de adentro y solo se recorre ese tramo. Recorrer
        // la caja entera gasta la mitad de las vueltas en pixeles de afuera.
        let cy = y as f32 + 0.5;
        let (mut desde, mut hasta) = (x0 as f32, x1 as f32 + 1.0);
        for &(p, q) in &aristas {
            // arista(p, q, (x, cy)) = pendiente * x + base
            let pendiente = -(q.1 - p.1);
            let base = (q.0 - p.0) * (cy - p.1) + (q.1 - p.1) * p.0;
            if pendiente > 0.0 {
                desde = desde.max(-base / pendiente - 0.5);
            } else if pendiente < 0.0 {
                hasta = hasta.min(-base / pendiente - 0.5);
            } else if base < 0.0 {
                hasta = desde - 1.0;
            }
        }
        if hasta < desde {
            continue;
        }
        // Un pixel de margen a cada lado: la prueba exacta de abajo decide.
        let xa = (desde.floor() as isize - 1).max(x0 as isize) as usize;
        let xb = (hasta.ceil() as isize + 1).min(x1);
        if xb < xa as isize {
            continue;
        }
        for x in xa..=xb as usize {
            let centro = (x as f32 + 0.5, cy);
            let e = [arista(pb, pc, centro), arista(pc, pa, centro), arista(pa, pb, centro)];
            if (0..3).any(|k| e[k] < 0.0 || (e[k] == 0.0 && !tl[k])) {
                continue;
            }
            // Coordenadas baricentricas: cuanto pesa cada vertice aca.
            let (w0, w1, w2) = (e[0] / area, e[1] / area, e[2] / area);
            let z = w0 * a.z + w1 * b.z + w2 * c.z;
            let i = y * fb.ancho + x;
            // Operaciones por fragmento: primero el STENCIL TEST, despues el DEPTH TEST.
            if let Stencil::Solo(r) = t.stencil {
                if fb.stencil(i) != r {
                    continue;
                }
            }
            if !(0.0..=1.0).contains(&z) || (!fb.pasa_profundidad(i, z) && t.stencil != Stencil::SinZBuffer) {
                continue;
            }
            if let Stencil::Marcar(r) = t.stencil {
                // La mascara: marca el pixel y abre el z-buffer, sin pintar.
                fb.escribir_stencil(i, r);
                fb.escribir_profundidad(i, 1.0);
                continue;
            }
            // Correccion de perspectiva: se interpola atributo/w y 1/w, y se divide.
            let inv_w = w0 * a.inv_w + w1 * b.inv_w + w2 * c.inv_w;
            let mundo = (a.mundo * w0 + b.mundo * w1 + c.mundo * w2) * (1.0 / inv_w);
            let normal = if plana { normal_plana } else { (a.normal * w0 + b.normal * w1 + c.normal * w2).normalize() };
            let uv = (a.uv * w0 + b.uv * w1 + c.uv * w2) * (1.0 / inv_w);
            let color = match vista {
                Vista::Normales => [normal.x * 0.5 + 0.5, normal.y * 0.5 + 0.5, normal.z * 0.5 + 0.5],
                Vista::Uv if mat.textura.is_some() => [uv.x.rem_euclid(1.0), uv.y.rem_euclid(1.0), 0.0],
                Vista::Uv => [0.15, 0.15, 0.18],
                _ => fragment_shader(mundo, normal, uv, mat, u),
            };
            match pasada {
                Pasada::Opaca => {
                    fb.pintar(i, color, 1.0);
                    fb.escribir_profundidad(i, z);
                }
                // Lo transparente se mezcla pero no tapa: no escribe profundidad.
                Pasada::Transparente => fb.pintar(i, color, mat.d),
            }
        }
    }
}

/// Las aristas del triangulo, para el modo alambre. Los atributos se
/// interpolan sin correccion de perspectiva: en una linea no se nota.
fn linea(fb: &mut Framebuffer, a: VerticePantalla, b: VerticePantalla, mat: &Material, u: &Uniforms) {
    let pasos = (b.x - a.x).abs().max((b.y - a.y).abs()).ceil().max(1.0) as usize;
    for s in 0..=pasos {
        let t = s as f32 / pasos as f32;
        let (x, y) = (a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
        let z = a.z + (b.z - a.z) * t;
        if x < 0.0 || y < 0.0 || x >= fb.ancho as f32 || y >= fb.alto as f32 || !(0.0..=1.0).contains(&z) {
            continue;
        }
        let i = y as usize * fb.ancho + x as usize;
        if fb.pasa_profundidad(i, z) {
            let inv_w = a.inv_w + (b.inv_w - a.inv_w) * t;
            let mundo = a.mundo.lerp(b.mundo, t) * (1.0 / inv_w);
            let normal = a.normal.lerp(b.normal, t);
            let uv = a.uv.lerp(b.uv, t) * (1.0 / inv_w);
            fb.pintar(i, fragment_shader(mundo, normal.normalize(), uv, mat, u), 1.0);
            fb.escribir_profundidad(i, z);
        }
    }
}

/// Un vertice como un cuadrito de 2x2, para el modo puntos.
fn punto(fb: &mut Framebuffer, v: VerticePantalla, mat: &Material, u: &Uniforms) {
    if !(0.0..=1.0).contains(&v.z) {
        return;
    }
    let color = fragment_shader(v.mundo * (1.0 / v.inv_w), v.normal.normalize(), v.uv * (1.0 / v.inv_w), mat, u);
    for dy in 0..2 {
        for dx in 0..2 {
            let (x, y) = (v.x as isize + dx - 1, v.y as isize + dy - 1);
            if x < 0 || y < 0 || x >= fb.ancho as isize || y >= fb.alto as isize {
                continue;
            }
            let i = y as usize * fb.ancho + x as usize;
            if fb.pasa_profundidad(i, v.z) {
                fb.pintar(i, color, 1.0);
                fb.escribir_profundidad(i, v.z);
            }
        }
    }
}

// ---------- 7. fragment shader ----------

/// Blinn-Phong con la luz saliendo del Sol: cada cara se ilumina segun que
/// tanto lo mira. El color emisivo (`Ke`) se suma aparte, asi el Sol y los
/// brillos se ven aunque no les llegue luz. Si el material tiene textura,
/// el color base sale de MUESTREARLA en la UV del fragmento.
/// El estilo "dibujado" (cel shading, como caricatura): la luz va en tres bandas (sombra, medio
/// tono y luz) en vez de degradado, y el brillo es una manchita nitida. Se cambia en los ajustes.
pub static DIBUJADO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Descartar objetos enteros fuera de la vista (frustum culling por objeto). Se apaga con `V` en el
/// espacio para comparar cuantos triangulos procesa el pipeline con y sin descarte.
pub static DESCARTE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

fn descartar() -> bool {
    DESCARTE.load(std::sync::atomic::Ordering::Relaxed)
}

/// Si el objeto queda entero fuera de la vista: la esfera que envuelve su modelo, llevada al
/// mundo con su matriz M, contra los seis planos del frustum. Los planos salen de las filas de
/// `P * V` (metodo de Gribb y Hartmann): un punto esta adentro si `w + x >= 0`, `w - x >= 0`, y lo
/// mismo con `y` y `z`; cada plano es la fila 4 mas o menos la fila 1, 2 o 3.
fn fuera_de_vista(ll: &DrawCall, vista_proyeccion: &Mat4) -> bool {
    let (centro, radio) = ll.modelo.esfera();
    if radio <= 0.0 {
        return false;
    }
    let m = &ll.uniforms.modelo;
    let c = m.transform_point(centro);
    // La escala mas grande de M (el largo de sus columnas) agranda la esfera.
    let escala = (0..3).map(|j| Vec3::new(m.m[0][j], m.m[1][j], m.m[2][j]).length()).fold(0.0, f32::max);
    // Lo que se anima o se mueve por partes se puede salir un poco de su esfera: holgura.
    let holgura = if ll.animacion.is_some() || ll.por_parte.is_some() { 1.6 } else { 1.05 };
    let r = radio * escala * holgura;
    let f = &vista_proyeccion.m;
    for k in 0..3 {
        for signo in [1.0, -1.0] {
            let plano = [0, 1, 2, 3].map(|j| f[3][j] + signo * f[k][j]);
            let largo = (plano[0] * plano[0] + plano[1] * plano[1] + plano[2] * plano[2]).sqrt();
            if (plano[0] * c.x + plano[1] * c.y + plano[2] * c.z + plano[3]) / largo < -r {
                return true;
            }
        }
    }
    false
}

fn dibujado() -> bool {
    DIBUJADO.load(std::sync::atomic::Ordering::Relaxed)
}

fn paso_suave(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// La luz en bandas: la sombra (0), el medio tono y la luz entera, con orillas apenas suaves (para
/// que no se vean escalones de pixeles).
fn bandas(x: f32) -> f32 {
    0.5 * paso_suave(0.03, 0.09, x) + 0.5 * paso_suave(0.42, 0.5, x)
}

/// El fragment shader: cada draw call usa el que pidio en sus uniforms.
fn fragment_shader(mundo: Vec3, n: Vec3, uv: Vec3, m: &Material, u: &Uniforms) -> [f32; 3] {
    match u.shader {
        Shader::Phong => phong(mundo, n, uv, m, u),
        Shader::Terreno => terreno(mundo, n, m, u),
        Shader::Oceano => oceano(mundo, n, m, u),
        Shader::Plasma => plasma(mundo, n, u),
    }
}

/// La tangente y la bitangente de una superficie de planeta: la tangente apunta "al este" (la
/// cruz del eje del planeta con la normal) y la bitangente "al norte". Con la normal arman la
/// TBN, que lleva lo que esta en espacio tangente (el mapa de normales, las olas) al mundo.
fn tangentes(eje: Vec3, n: Vec3) -> (Vec3, Vec3) {
    let mut t = eje.cross(n);
    if t.length() < 1e-3 {
        // En los polos el eje y la normal son paralelos: cualquier otra direccion sirve.
        t = Vec3::new(1.0, 0.0, 0.0).cross(n);
    }
    let t = t.normalize();
    (t, n.cross(t))
}

/// Un mapa de normales hecho al arrancar (128 x 128): UNA cara de voxel, con la orilla biselada
/// (asi cada bloque se lee como bloque) y un granito de ruido adentro. Cada texel guarda una
/// normal en espacio tangente, de [-1, 1] pasada a [0, 1] (por eso los mapas de normales se ven
/// morados). Sale de un campo de alturas: la normal es (-dh/du, -dh/dv, 1); la v de la textura
/// crece hacia arriba y las filas hacia abajo (ver `Textura::muestrear`).
fn mapa_normales() -> &'static crate::obj::Textura {
    static MAPA: std::sync::OnceLock<crate::obj::Textura> = std::sync::OnceLock::new();
    MAPA.get_or_init(|| {
        const N: i32 = 128;
        let altura = |x: i32, y: i32| {
            let (x, y) = (x.clamp(0, N - 1), y.clamp(0, N - 1));
            // El bisel: sube en el primer 12% desde cada orilla.
            let orilla = x.min(N - 1 - x).min(y.min(N - 1 - y)) as f32 / N as f32;
            let bisel = paso_suave(0.0, 0.12, orilla);
            let mut grano = 0.0;
            let mut amplitud = 1.0;
            for periodo in [8, 16, 32] {
                let k = periodo as f32 / N as f32;
                grano += amplitud * crate::ruido::valor2_periodico(x as f32 * k, y as f32 * k, periodo);
                amplitud *= 0.5;
            }
            bisel * 0.12 + grano * 0.006
        };
        let mut texels = Vec::with_capacity((N * N) as usize);
        for y in 0..N {
            for x in 0..N {
                let dx = altura(x + 1, y) - altura(x - 1, y);
                let dfila = altura(x, y + 1) - altura(x, y - 1);
                let n = Vec3::new(-dx * 40.0, dfila * 40.0, 1.0).normalize();
                texels.push(n * 0.5 + Vec3::new(0.5, 0.5, 0.5));
            }
        }
        crate::obj::Textura::desde(N as usize, N as usize, texels)
    })
}

/// Terreno de planeta voxel (ver `Shader::Terreno`): el color es el del material del voxel, con
/// un poquito de variacion por bloque (como los pixeles de los demas planetas), y el relieve de
/// cada cara sale del mapa de normales, llevado al mundo con su TBN. La lava corre y late.
fn terreno(mundo: Vec3, n: Vec3, m: &Material, u: &Uniforms) -> [f32; 3] {
    // Al espacio del planeta (sin moverlo, girarlo ni escalarlo): ahi los voxels estan alineados a
    // los ejes. La matriz es escala * giro, asi que su transpuesta entre la escala al cuadrado la
    // deshace.
    let mm = &u.modelo.m;
    let al_planeta = |v: Vec3| Vec3::new(mm[0][0] * v.x + mm[1][0] * v.y + mm[2][0] * v.z, mm[0][1] * v.x + mm[1][1] * v.y + mm[2][1] * v.z, mm[0][2] * v.x + mm[1][2] * v.y + mm[2][2] * v.z);
    let al_mundo = |v: Vec3| Vec3::new(mm[0][0] * v.x + mm[0][1] * v.y + mm[0][2] * v.z, mm[1][0] * v.x + mm[1][1] * v.y + mm[1][2] * v.z, mm[2][0] * v.x + mm[2][1] * v.y + mm[2][2] * v.z);
    let escala2 = (mm[0][0] * mm[0][0] + mm[1][0] * mm[1][0] + mm[2][0] * mm[2][0]).max(1e-8);
    let p = al_planeta(mundo - u.centro) * (1.0 / escala2);
    let nl = al_planeta(n).normalize();
    // La cara mira a uno de los seis ejes; los otros dos son su tangente y su bitangente.
    let (ax, ay, az) = (nl.x.abs(), nl.y.abs(), nl.z.abs());
    let eje = if ax >= ay && ax >= az { 0 } else if ay >= az { 1 } else { 2 };
    let unitario = |k: usize| match k {
        0 => Vec3::new(1.0, 0.0, 0.0),
        1 => Vec3::new(0.0, 1.0, 0.0),
        _ => Vec3::new(0.0, 0.0, 1.0),
    };
    let celda = u.celda.max(1e-4);
    let q = p * (1.0 / celda);
    let comp = |v: Vec3, k: usize| match k {
        0 => v.x,
        1 => v.y,
        _ => v.z,
    };
    let (ib, ic) = ((eje + 1) % 3, (eje + 2) % 3);
    let texel = mapa_normales().muestrear(comp(q, ib).rem_euclid(1.0), comp(q, ic).rem_euclid(1.0));
    let k = u.intensidad;
    let tangente = unitario(ib) * ((texel.x * 2.0 - 1.0) * k) + unitario(ic) * ((texel.y * 2.0 - 1.0) * k) + nl * (texel.z * 2.0 - 1.0);
    let n2 = al_mundo(tangente).normalize();
    // Cada bloque un poquito mas claro o mas oscuro (el voxel es el de atras de la cara).
    let dentro = q - nl * 0.5;
    let variacion = 0.92 + 0.14 * crate::ruido::hash3(dentro.x.floor() as i32, dentro.y.floor() as i32, dentro.z.floor() as i32);
    let base = m.kd * variacion;
    let hacia_luz = (u.luz - mundo).normalize();
    // La luz mezcla la normal de la cara con la de la esfera: cada bloque tiene sus caras, pero
    // el dia y la noche son del planeta (si no, de tres cuartos sale un tablero de ajedrez: una
    // cara al Sol y la de al lado negra). La noche empieza donde la esfera le da la espalda al Sol.
    let esfera = (mundo - u.centro).normalize();
    let dia = paso_suave(-0.12, 0.15, esfera.dot(hacia_luz));
    let lambert_de = |normal: Vec3| (normal * 0.55 + esfera * 0.45).normalize().dot(hacia_luz).max(0.0) * dia;
    let (lisa, con_mapa) = (lambert_de(n), lambert_de(n2));
    // De dia, las caras que no ven al Sol reciben la luz del cielo (rebotada): sombra, no negro.
    let relleno = 0.24 * dia;
    // En el estilo dibujado la luz va en bandas, pero lo que pone el mapa de normales se suma
    // suave encima (si no, las bandas se lo comen y el bisel no se veria).
    let difusa = if dibujado() { bandas(lisa) * 0.86 + (con_mapa - lisa) * 0.9 } else { con_mapa * 0.82 }.max(relleno);
    // De noche, luz de luna (azulita): se ve que es de noche, pero el planeta no desaparece contra
    // el cielo negro (y la lava de los volcanes resalta).
    let luz = u.color_luz * difusa + luz_de_luna(dia, n.dot(esfera));
    let mut c = Vec3::new(base.x * (u.ambiente.x + luz.x), base.y * (u.ambiente.y + luz.y), base.z * (u.ambiente.z + luz.z));
    // La lava: un fBm que corre con el tiempo por encima de los bloques y late.
    if m.ke.x > 0.0 {
        let flujo = crate::ruido::valor_fbm(p * 9.0 + Vec3::new(0.0, -u.tiempo * 0.7, u.tiempo * 0.35), 3);
        let brillo = paso_suave(0.35, 0.75, flujo) * (0.75 + 0.25 * (u.tiempo * 3.1).sin());
        c += m.ke * (0.55 + 1.5 * brillo);
    }
    // La luz de borde, como en los demas.
    let v = (u.camara - mundo).normalize();
    c += u.borde * (1.0 - n.dot(v).max(0.0)).powi(3);
    [c.x, c.y, c.z]
}

/// La luz de luna de los planetas voxel del lado de noche (`dia` de 0 a 1), mas fuerte en las
/// caras que miran hacia afuera (`afuera`: la normal contra la direccion desde el centro).
fn luz_de_luna(dia: f32, afuera: f32) -> Vec3 {
    Vec3::new(0.10, 0.14, 0.30) * ((1.0 - dia) * (0.55 + 0.45 * afuera.max(0.0)))
}

/// Mar (ver `Shader::Oceano`).
fn oceano(mundo: Vec3, n: Vec3, m: &Material, u: &Uniforms) -> [f32; 3] {
    let v = (u.camara - mundo).normalize();
    // El agua es voxel: todas las caras que miran al mismo lado tienen la misma normal, y el
    // brillo del Sol pintaria baldosas enteras de blanco. La normal que se ilumina mezcla la de la
    // cara con la de la esfera: el reflejo es una mancha sobre el planeta y los bloques se notan.
    let esfera = (mundo - u.centro).normalize();
    let base = (n * 0.4 + esfera * 0.6).normalize();
    let (t, b) = tangentes(u.eje, base);
    // Las olas: tres trenes de ondas en el plano tangente, con distinto largo, rumbo y velocidad.
    // Lo que se mueve es la normal (su pendiente), no los vertices: brillos que corren.
    let p = (mundo - u.centro) * (1.0 / u.radio.max(1e-4));
    let (a, c) = (p.dot(t), p.dot(b));
    let s = u.tiempo;
    let k = 0.06 * u.intensidad;
    let gx = k * ((a * 70.0 + s * 1.6).cos() + 0.5 * ((a + c) * 115.0 - s * 2.2).cos());
    let gy = k * ((c * 60.0 - s * 1.3).cos() + 0.5 * ((a - c) * 135.0 + s * 2.7).cos());
    let n2 = (base - t * gx - b * gy).normalize();
    // Fresnel (Schlick): de frente se ve el agua honda; de canto, el reflejo del cielo (sin
    // llegar a blanco: si no, de canto el mar parece hielo).
    let fresnel = (0.04 + 0.96 * (1.0 - n2.dot(v).max(0.0)).powi(5)) * 0.6;
    let hacia_luz = (u.luz - mundo).normalize();
    let lambert = base.dot(hacia_luz).max(0.0);
    // El agua honda es del color del material (el azul de los demas planetas); de noche, con la
    // luz de luna.
    let esfera_luz = esfera.dot(hacia_luz);
    let luna = luz_de_luna(paso_suave(-0.12, 0.15, esfera_luz), 1.0);
    let hondo = m.kd * (0.35 * paso_suave(-0.3, 0.1, esfera_luz) + 0.75 * lambert)
        + Vec3::new(m.kd.x * (u.ambiente.x + luna.x), m.kd.y * (u.ambiente.y + luna.y), m.kd.z * (u.ambiente.z + luna.z));
    let cielo = Vec3::new(0.38, 0.68, 1.0) * (0.3 + 0.6 * lambert);
    let mut color = hondo.lerp(cielo, fresnel);
    // El brillo del Sol sobre las olas (Blinn-Phong, muy concentrado).
    let medio = (hacia_luz + v).normalize();
    color += u.color_luz * (n2.dot(medio).max(0.0).powi(160) * 1.6 * lambert.sqrt());
    [color.x, color.y, color.z]
}

/// Plasma del Sol (ver `Shader::Plasma`).
fn plasma(mundo: Vec3, n: Vec3, u: &Uniforms) -> [f32; 3] {
    let d = (mundo - u.centro).normalize();
    let s = u.tiempo * 0.15;
    // Domain warping: un ruido decide donde se lee el otro, y eso hace los remolinos. El desvio
    // sale de dos fBm de dos octavas (la tercera componente es su diferencia): con el Sol llenando
    // la pantalla, son 7 lecturas de ruido por pixel en vez de 12.
    let (qa, qb) = (
        crate::ruido::valor_fbm(d * 2.4 + Vec3::new(s, 0.0, 0.0), 2),
        crate::ruido::valor_fbm(d * 2.4 + Vec3::new(5.2, 1.3 + s, 0.0), 2),
    );
    let q = Vec3::new(qa, qb, qa - qb + 0.5);
    let f = crate::ruido::valor_fbm(d * 3.2 + q * (2.6 * u.intensidad.max(0.1)) + Vec3::new(0.0, 0.0, -s * 1.5), 3);
    let (rojo, naranja, amarillo) = (Vec3::new(0.62, 0.10, 0.02), Vec3::new(1.0, 0.48, 0.06), Vec3::new(1.0, 0.95, 0.62));
    let c = if f < 0.5 { rojo.lerp(naranja, f * 2.0) } else { naranja.lerp(amarillo, (f - 0.5) * 2.0) };
    // Oscurecimiento al borde del disco (limb darkening).
    let v = (u.camara - mundo).normalize();
    let c = c * (0.55 + 0.75 * n.dot(v).max(0.0).sqrt());
    [c.x, c.y, c.z]
}

/// Blinn-Phong (ver `Shader::Phong`).
fn phong(mundo: Vec3, n: Vec3, uv: Vec3, m: &Material, u: &Uniforms) -> [f32; 3] {
    let hacia_luz = u.luz - mundo;
    let l = hacia_luz.normalize();
    // Un foco (con alcance) se apaga con la distancia, suavecito: 1 / (1 + (d/alcance)^2).
    let atenuacion = if u.alcance > 0.0 {
        let k = hacia_luz.length() / u.alcance;
        1.0 / (1.0 + k * k)
    } else {
        1.0
    };
    // La luz directa no llega a 1: asi lo claro (la piel, lo lila) no se quema a blanco.
    let toon = dibujado();
    let lambert = n.dot(l).max(0.0);
    let difusa = if toon { bandas(lambert) * 0.86 } else { lambert * 0.82 } * atenuacion;
    let luz = u.color_luz * difusa;
    let base = match &m.textura {
        Some(t) => {
            let texel = t.muestrear(uv.x, uv.y);
            Vec3::new(texel.x * m.kd.x, texel.y * m.kd.y, texel.z * m.kd.z)
        }
        None => m.kd,
    };
    // Con textura, el emisivo tambien la usa: asi una pantalla o un pizarron brillan con su dibujo.
    let propio = if m.textura.is_some() { Vec3::new(base.x * m.ke.x, base.y * m.ke.y, base.z * m.ke.z) } else { m.ke };
    let mut c = Vec3::new(base.x * (u.ambiente.x + luz.x), base.y * (u.ambiente.y + luz.y), base.z * (u.ambiente.z + luz.z)) + propio;
    // El brillo especular y la luz de borde necesitan la direccion a la
    // camara; si el material o la escena no los usan, ni se calcula.
    let con_brillo = difusa > 0.0 && m.ks.x + m.ks.y + m.ks.z > 0.0;
    let con_borde = u.borde.x + u.borde.y + u.borde.z > 0.0;
    if con_brillo || con_borde {
        let v = (u.camara - mundo).normalize();
        if con_brillo {
            let brillo = n.dot((l + v).normalize()).max(0.0).powi(m.ns.max(1.0) as i32);
            let brillo = if toon { paso_suave(0.45, 0.55, brillo) * 0.55 } else { brillo } * atenuacion;
            c += Vec3::new(m.ks.x * u.color_luz.x, m.ks.y * u.color_luz.y, m.ks.z * u.color_luz.z) * brillo;
        }
        if con_borde {
            let canto = (1.0 - n.dot(v).max(0.0)).powi(3);
            c += u.borde * if toon { paso_suave(0.2, 0.35, canto) * 0.8 } else { canto };
        }
    }
    [c.x, c.y, c.z]
}

/// Un punto del mundo en pixeles, con su profundidad en [0, 1]. `None` si
/// queda detras de la camara. Es el mismo camino que un vertice (V, P,
/// divide, viewport), para lo que se dibuja fuera de los modelos.
pub fn proyectar(p: Vec3, vista_proyeccion: &Mat4, fb: &Framebuffer) -> Option<(f32, f32, f32)> {
    let c = vista_proyeccion.mul_vec4(Vec4::new(p.x, p.y, p.z, 1.0));
    if c.w <= 0.01 {
        return None;
    }
    let v = a_pantalla(VerticeRecorte { clip: c, mundo: p, normal: Vec3::zero(), uv: Vec3::zero() }, fb);
    Some((v.x, v.y, v.z))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Un cuadrado de lado `2 * medio` en z = 0, mirando a +Z, de un color emisivo.
    fn cuadrado(medio: f32, color: [f32; 3]) -> Modelo {
        let ruta = std::env::temp_dir().join(format!("cuadrado_{}_{}.obj", medio, color[0]));
        let mtl = ruta.with_extension("mtl");
        std::fs::write(&mtl, format!("newmtl c\nKd 0 0 0\nKe {} {} {}\n", color[0], color[1], color[2])).unwrap();
        let m = medio;
        let obj = format!(
            "mtllib {}\no cuadro\nusemtl c\nv {} {} 0\nv {m} {} 0\nv {m} {m} 0\nv {} {m} 0\nf 1 2 3 4\n",
            mtl.file_name().unwrap().to_string_lossy(), -m, -m, -m, -m
        );
        std::fs::write(&ruta, obj).unwrap();
        Modelo::cargar(&ruta.to_string_lossy()).unwrap()
    }

    /// El stencil test: lo que se dibuja con `Solo(1)` solo aparece donde otro
    /// draw call marco la mascara con `Marcar(1)`.
    #[test]
    fn el_stencil_recorta_lo_que_se_dibuja() {
        let mut fb = Framebuffer::new(64, 64);
        fb.limpiar([0, 0, 0]);
        let vista = Mat4::look_at(Vec3::new(0.0, 0.0, 3.0), Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
        let proyeccion = Mat4::perspective(60f32.to_radians(), 1.0, 0.1, 10.0);
        let u = Uniforms {
            modelo: Mat4::identity(),
            vista,
            proyeccion,
            luz: Vec3::new(0.0, 0.0, 5.0),
            camara: Vec3::new(0.0, 0.0, 3.0),
            ambiente: Vec3::zero(),
            borde: Vec3::zero(),
            color_luz: Vec3::new(1.0, 1.0, 1.0),
            alcance: 0.0,
            shader: Shader::Phong,
            tiempo: 0.0,
            centro: Vec3::zero(),
            eje: Vec3::new(0.0, 1.0, 0.0),
            radio: 1.0,
            intensidad: 1.0,
            celda: 0.0,
        };
        let (chico, grande) = (cuadrado(0.4, [0.0, 0.0, 0.0]), cuadrado(1.5, [1.0, 0.0, 0.0]));
        let op = Opciones { modo: ModoDibujo::Relleno, culling: true, vista: Vista::Color };
        let llamada = |modelo, stencil| DrawCall { modelo, uniforms: u, expresion: None, animacion: None, ocultar: &[], por_parte: None, modo: None, stencil };
        // La mascara, un poquito adelante (mas cerca de la camara).
        let u_mascara = Uniforms { modelo: Mat4::translation(Vec3::new(0.0, 0.0, 0.1)), ..u };
        render(&mut fb, &[DrawCall { uniforms: u_mascara, ..llamada(&chico, Stencil::Marcar(1)) }, llamada(&grande, Stencil::Solo(1))], &op);
        let rojo = |x: usize, y: usize| fb.leer(y * 64 + x)[0] > 0.5;
        assert!(rojo(32, 32), "en medio (adentro de la mascara) se dibuja");
        assert!(!rojo(32, 14), "afuera de la mascara no, aunque el cuadro grande pase por ahi");
    }
}
