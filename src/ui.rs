//! Cajas de texto encima del render (el HUD, la IA de la nave, el examen).
//! Esto si lo dibuja raylib directo en la ventana: es interfaz, no escena.
//!
//! El texto va en Pixelify Sans (licencia SIL Open Font, ver
//! `assets/fuentes/OFL.txt`), un font pixelado de videojuego, cargado a
//! varios tamanos para que salga nitido. Si no se encuentra, se usa el de raylib.
//! Los titulos van en 04b_30 (de Yuji Oshimoto, 04.jp.org: freeware, ver
//! `assets/fuentes/04B_30-licencia.gif`): letras de pixel gorditas, con contorno, sombra, canto
//! y dos tonos, como los logos de los juegos (ver `titulo`). Ese font no trae tildes, ñ ni ¡¿:
//! esas marcas se dibujan aqui encima de su letra, en la misma cuadricula de pixeles.

use raylib::ffi;
use raylib::prelude::*;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::ffi::CString;

const RUTA_FUENTE: &str = "assets/fuentes/PixelifySans.ttf";
const RUTA_TITULOS: &str = "assets/fuentes/04B_30.ttf";
/// Los tamanos del font de los titulos (son pocos y grandes).
const TAMANOS_TITULO: [i32; 9] = [20, 26, 34, 44, 56, 70, 88, 110, 140];
/// A que tamanos se rasteriza el font: cada texto se dibuja con el atlas de su tamano (o el mas
/// cercano de arriba), asi sale nitido en vez de borroso por escalar uno solo de 48 (lo que pasaba
/// antes: lo chiquito y los numeros se emborronaban).
const TAMANOS: [i32; 24] = [11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 22, 24, 26, 28, 30, 34, 40, 46, 52, 60, 72, 90, 110, 130];

/// El atlas mas grande que se carga (los titulos enormes se agrandan un poquito desde aqui).
const ATLAS_MAXIMO: i32 = 170;

thread_local! {
    /// Cuantos pixeles de pantalla hay por pixel de interfaz (2 en una pantalla Retina): los
    /// atlas se cargan a ese tamano, para que el texto salga nitido y no estirado.
    static ESCALA: Cell<f32> = const { Cell::new(1.0) };
    /// El font cargado a cada tamano (en pixeles de interfaz): (tamano, font).
    static FUENTES: std::cell::RefCell<Vec<(i32, ffi::Font)>> = const { std::cell::RefCell::new(Vec::new()) };
    /// Lo mismo para el font de los titulos.
    static TITULOS: std::cell::RefCell<Vec<(i32, ffi::Font)>> = const { std::cell::RefCell::new(Vec::new()) };
    /// Donde va el retrato de GLADAS este cuadro (x, y, lado), si esta hablando:
    /// lo deja `caja_dialogo` y el bucle principal pega ahi la imagen.
    static RETRATO: Cell<Option<(i32, i32, i32)>> = const { Cell::new(None) };
}

/// Lado del retrato de GLADAS dentro de su caja de dialogo.
pub const LADO_RETRATO: i32 = 112;

/// Olvida el retrato del cuadro anterior (se llama antes de dibujar la interfaz).
pub fn reiniciar_retrato() {
    RETRATO.set(None);
}

/// Donde quedo el retrato de GLADAS en este cuadro, si hablo.
pub fn retrato() -> Option<(i32, i32, i32)> {
    RETRATO.get()
}

/// Carga los fonts una vez, despues de abrir la ventana, a varios tamanos. Se cargan los
/// caracteres de Latin-1 (32 a 255): ahi estan las tildes, la ñ, ¿ y ¡.
/// Se llama a la funcion de C directo porque `load_font_ex` de raylib-rs le
/// pasa a C el largo en bytes del texto en vez de la cantidad de caracteres,
/// y con letras de mas de un byte leeria fuera de la lista.
pub fn cargar_fuente(_hilo: &RaylibThread, escala: f32) {
    let escala = escala.clamp(1.0, 3.0);
    ESCALA.set(escala);
    let mut caracteres: Vec<i32> = (32..=255).collect();
    let ruta = CString::new(RUTA_FUENTE).unwrap();
    let mut cargados = Vec::new();
    for tam in TAMANOS {
        // El atlas a la resolucion de la pantalla (el doble en Retina), con un tope.
        let px = ((tam as f32 * escala).round() as i32).min(ATLAS_MAXIMO);
        if cargados.iter().any(|&(_, p, _): &(i32, i32, ffi::Font)| p == px) {
            continue;
        }
        let f = unsafe { ffi::LoadFontEx(ruta.as_ptr(), px, caracteres.as_mut_ptr(), caracteres.len() as i32) };
        if f.glyphs.is_null() || f.texture.id == 0 {
            eprintln!("no se pudo cargar {RUTA_FUENTE}; se usa el font de raylib");
            break;
        }
        unsafe { ffi::SetTextureFilter(f.texture, TextureFilter::TEXTURE_FILTER_BILINEAR as i32) };
        unsafe { numeros_legibles(&f) };
        cargados.push((tam, px, f));
    }
    FUENTES.with(|f| *f.borrow_mut() = cargados.into_iter().map(|(t, _, f)| (t, f)).collect());
    // Los titulos: solo ASCII (es lo que trae el font; las tildes se dibujan aparte).
    let ruta = CString::new(RUTA_TITULOS).unwrap();
    let mut ascii: Vec<i32> = (32..127).collect();
    let mut titulos = Vec::new();
    for tam in TAMANOS_TITULO {
        let px = ((tam as f32 * escala).round() as i32).min(240);
        let f = unsafe { ffi::LoadFontEx(ruta.as_ptr(), px, ascii.as_mut_ptr(), ascii.len() as i32) };
        if f.glyphs.is_null() || f.texture.id == 0 {
            eprintln!("no se pudo cargar {RUTA_TITULOS}; los titulos van en el font normal");
            break;
        }
        unsafe { ffi::SetTextureFilter(f.texture, TextureFilter::TEXTURE_FILTER_BILINEAR as i32) };
        titulos.push((tam, f));
    }
    TITULOS.with(|t| *t.borrow_mut() = titulos);
}

/// En Pixelify el 5 es igual a la S y el 0 igual a la O: con numeros chiquitos no se sabe que dice
/// ("Recoge 2S notas"). Se redibujan esos dos digitos en el atlas, en la misma cuadricula de pixeles
/// del font (5 x 7): un 5 con la parte de arriba recta y un 0 con diagonal.
unsafe fn numeros_legibles(f: &ffi::Font) {
    const CINCO: [&str; 7] = ["#####", "#....", "####.", "....#", "....#", "#...#", ".###."];
    const CERO: [&str; 7] = [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."];
    if f.glyphs.is_null() || f.recs.is_null() {
        return;
    }
    let glifos = std::slice::from_raw_parts(f.glyphs, f.glyphCount as usize);
    let recs = std::slice::from_raw_parts(f.recs, f.glyphCount as usize);
    for (letra, dibujo) in [('5', CINCO), ('0', CERO)] {
        let Some(k) = glifos.iter().position(|g| g.value == letra as i32) else { continue };
        let r = recs[k];
        let (w, h) = (r.width as usize, r.height as usize);
        if w < 5 || h < 7 {
            continue;
        }
        // El atlas es gris con alfa (dos bytes por pixel): blanco donde hay pixel del dibujo.
        let mut datos = vec![0u8; w * h * 2];
        for y in 0..h {
            for x in 0..w {
                let (c, f) = (x * 5 / w, y * 7 / h);
                if dibujo[f].as_bytes()[c] == b'#' {
                    datos[(y * w + x) * 2] = 255;
                    datos[(y * w + x) * 2 + 1] = 255;
                }
            }
        }
        ffi::UpdateTextureRec(f.texture, r, datos.as_ptr() as *const std::os::raw::c_void);
    }
}

/// El font para dibujar a `tam` pixeles: el atlas del tamano justo (o el mas cercano de arriba; si
/// no hay, el mas grande), y el espaciado entre letras (un poquito mas en chiquito, para que los
/// numeros no se peguen).
fn fuente(tam: f32) -> (ffi::Font, f32) {
    FUENTES.with(|f| {
        let f = f.borrow();
        let elegido = f.iter().filter(|(n, _)| *n as f32 >= tam - 0.5).min_by_key(|(n, _)| *n).or_else(|| f.iter().max_by_key(|(n, _)| *n));
        let espacio = (tam / 16.0).max(1.0);
        match elegido {
            Some(&(_, font)) => (font, espacio),
            None => (unsafe { ffi::GetFontDefault() }, tam / 10.0),
        }
    })
}

/// El font de los titulos para `tam` pixeles (o el normal, si no se pudo cargar).
fn fuente_titulo(tam: f32) -> (ffi::Font, f32) {
    let hay = TITULOS.with(|t| {
        let t = t.borrow();
        t.iter().filter(|(n, _)| *n as f32 >= tam * 0.9).min_by_key(|(n, _)| *n).or_else(|| t.iter().max_by_key(|(n, _)| *n)).map(|&(_, f)| f)
    });
    match hay {
        Some(f) => (f, tam / 24.0),
        None => fuente(tam),
    }
}

/// Las marcas que el font de los titulos no trae y se dibujan encima de la letra.
#[derive(Clone, Copy, PartialEq)]
enum Marca {
    Nada,
    Tilde,
    Enye,
    Dieresis,
    /// ¡ y ¿: el ! y el ? de cabeza.
    DeCabeza,
}

/// Cada letra del titulo como la trae el font (en mayuscula y sin tilde) y la marca que lleva.
fn letras_de_titulo(s: &str) -> Vec<(char, Marca)> {
    s.chars()
        .map(|c| match c {
            'á' | 'Á' => ('A', Marca::Tilde),
            'é' | 'É' => ('E', Marca::Tilde),
            'í' | 'Í' => ('I', Marca::Tilde),
            'ó' | 'Ó' => ('O', Marca::Tilde),
            'ú' | 'Ú' => ('U', Marca::Tilde),
            'ü' | 'Ü' => ('U', Marca::Dieresis),
            'ñ' | 'Ñ' => ('N', Marca::Enye),
            '¡' => ('!', Marca::DeCabeza),
            '¿' => ('?', Marca::DeCabeza),
            '«' => ('<', Marca::Nada),
            '»' => ('>', Marca::Nada),
            '·' => ('-', Marca::Nada),
            '—' | '–' => ('-', Marca::Nada),
            // Todo en mayusculas (las minusculas del font son versalitas, y se ve disparejo).
            c if (c as u32) < 127 => (c.to_ascii_uppercase(), Marca::Nada),
            _ => ('?', Marca::Nada),
        })
        .collect()
}

/// Lo ancho de cada letra del titulo (sin el espacio entre letras).
fn ancho_letra(font: ffi::Font, c: char, tam: f32) -> f32 {
    let mut b = [0u8; 4];
    let s = CString::new(c.encode_utf8(&mut b).as_bytes()).unwrap_or_default();
    unsafe { ffi::MeasureTextEx(font, s.as_ptr(), tam, 0.0).x }
}

/// Lo ancho que queda un titulo.
pub fn medir_titulo(s: &str, tam: i32) -> i32 {
    let (font, espacio) = fuente_titulo(tam as f32);
    let letras = letras_de_titulo(&con_protagonista(s));
    let total: f32 = letras.iter().map(|&(c, _)| ancho_letra(font, c, tam as f32)).sum();
    (total + espacio * letras.len().saturating_sub(1) as f32) as i32
}

/// Un color mas oscuro (para el contorno de los titulos).
fn oscuro(c: Color, k: f32) -> Color {
    Color::new((c.r as f32 * k) as u8, (c.g as f32 * k) as u8, (c.b as f32 * k) as u8, c.a)
}

/// Un titulo de videojuego: letras gorditas con contorno oscuro, sombra abajo y dos tonos (`arriba`
/// en la mitad de arriba de las letras, `abajo` en la de abajo, con una rayita de brillo en medio).
/// La transparencia sale de `arriba`.
pub fn titulo(d: &mut RaylibDrawHandle, s: &str, x: i32, y: i32, tam: i32, arriba: Color, abajo: Color) {
    titulo_girado(d, s, x as f32, y as f32, tam as f32, 0.0, arriba, abajo);
}

/// Un titulo de un solo color (arriba sale mas clarito, como con brillo).
pub fn titulo_de(d: &mut RaylibDrawHandle, s: &str, x: i32, y: i32, tam: i32, color: Color) {
    titulo(d, s, x, y, tam, claro(color), color);
}

/// Lo mismo, girado.
pub fn titulo_girado_de(d: &mut RaylibDrawHandle, s: &str, x: f32, y: f32, tam: f32, grados: f32, color: Color) {
    titulo_girado(d, s, x, y, tam, grados, color, color);
}

/// El titulo de los menus (girado): blanco con el canto morado del juego.
pub fn titulo_menu(d: &mut RaylibDrawHandle, s: &str, x: f32, y: f32, tam: f32, grados: f32) {
    titulo_girado(d, s, x, y, tam, grados, Color::WHITE, Color::new(150, 90, 255, 255));
}

/// El color mezclado con blanco (para la parte de arriba de los titulos).
fn claro(c: Color) -> Color {
    let m = |v: u8| (v as f32 + (255.0 - v as f32) * 0.5) as u8;
    Color::new(m(c.r), m(c.g), m(c.b), c.a)
}


/// Un titulo girado `grados` alrededor de su esquina de arriba a la izquierda: sombra, contorno
/// grueso, un canto abajo (como letras de bloque) y el relleno. Los dos tonos del relleno solo
/// salen derechos (girado va del color de `arriba`). Se dibuja letra por letra (para ponerle a
/// cada una su tilde, o voltear el ¡ y el ¿).
#[allow(clippy::too_many_arguments)]
pub fn titulo_girado(_d: &mut RaylibDrawHandle, s: &str, x: f32, y: f32, tam: f32, grados: f32, arriba: Color, abajo: Color) {
    // Si no cabe en la ventana, se achica (y si venia centrado, se vuelve a centrar); si se sale
    // por la derecha, se corre hacia adentro.
    if grados == 0.0 {
        let pantalla = unsafe { ffi::GetScreenWidth() } as f32;
        let w = medir_titulo(s, tam as i32) as f32;
        if w > pantalla - 40.0 && tam > 12.0 {
            let nuevo = (tam * (pantalla - 40.0) / w).floor().max(12.0);
            let centrado = (x + w / 2.0 - pantalla / 2.0).abs() < 60.0 || x < 0.0;
            let nx = if centrado { (pantalla - medir_titulo(s, nuevo as i32) as f32) / 2.0 } else { x.max(20.0) };
            return titulo_girado(_d, s, nx, y + (tam - nuevo) * 0.5, nuevo, grados, arriba, abajo);
        }
        if x + w > pantalla - 16.0 && x > 16.0 {
            return titulo_girado(_d, s, (pantalla - 16.0 - w).max(16.0), y, tam, grados, arriba, abajo);
        }
    }
    let (font, espacio) = fuente_titulo(tam);
    let letras = letras_de_titulo(&con_protagonista(s));
    let a = arriba.a;
    let (sen, cos) = grados.to_radians().sin_cos();
    // Hacia adelante (a lo largo del renglon) y hacia abajo, ya girados.
    let (fx, fy) = (cos, sin_o_cero(sen));
    let (bx, by) = (-sen, cos);
    // Donde empieza cada letra y lo ancho que es.
    let mut avance = 0.0;
    let lugares: Vec<(char, Marca, f32, f32)> = letras
        .iter()
        .map(|&(c, m)| {
            let w = ancho_letra(font, c, tam);
            let x0 = avance;
            avance += w + espacio;
            (c, m, x0, w)
        })
        .collect();
    let ancho_total = (avance - espacio).max(0.0);
    if grados == 0.0 && REGISTRO.with(|r| r.borrow().is_some()) {
        anotar(x, y, ancho_total, tam, s, arriba.a);
    }
    // Un pixel del font (los glifos de 04b_30 son de 8 pixeles de alto en 0.82 del tamano).
    let u = tam * 0.103;
    let tope = tam * 0.07;
    let rect = |lx: f32, ly: f32, w: f32, h: f32, dx: f32, dy: f32, color: Color| unsafe {
        let p = (x + dx + fx * lx + bx * ly, y + dy + fy * lx + by * ly);
        ffi::DrawRectanglePro(ffi::Rectangle { x: p.0, y: p.1, width: w, height: h }, ffi::Vector2 { x: 0.0, y: 0.0 }, grados, color.into());
    };
    let dibujar = |dx: f32, dy: f32, color: Color| {
        let mut b = [0u8; 4];
        for &(c, marca, x0, w) in &lugares {
            if c == ' ' {
                continue;
            }
            let texto = CString::new(c.encode_utf8(&mut b).as_bytes()).unwrap_or_default();
            unsafe {
                if marca == Marca::DeCabeza {
                    // Volteado alrededor de su centro.
                    let (cx, cy) = (x0 + w / 2.0, tam / 2.0);
                    let p = ffi::Vector2 { x: x + dx + fx * cx + bx * cy, y: y + dy + fy * cx + by * cy };
                    ffi::DrawTextPro(font, texto.as_ptr(), p, ffi::Vector2 { x: w / 2.0, y: tam / 2.0 }, grados + 180.0, tam, 0.0, color.into());
                } else {
                    let p = ffi::Vector2 { x: x + dx + fx * x0, y: y + dy + fy * x0 };
                    ffi::DrawTextPro(font, texto.as_ptr(), p, ffi::Vector2 { x: 0.0, y: 0.0 }, grados, tam, 0.0, color.into());
                }
            }
            // Las marcas, en pixeles del font, arriba de la letra.
            let cx = x0 + w / 2.0;
            match marca {
                Marca::Tilde => {
                    rect(cx - 0.4 * u, tope - 1.5 * u, 1.3 * u, 1.1 * u, dx, dy, color);
                    rect(cx + 0.7 * u, tope - 2.5 * u, 1.3 * u, 1.1 * u, dx, dy, color);
                }
                Marca::Enye => {
                    for (k, alto) in [0.0f32, -1.0, 0.0, -1.0].iter().enumerate() {
                        rect(cx - 2.0 * u + k as f32 * u, tope - 1.8 * u + alto * u, 1.05 * u, 1.05 * u, dx, dy, color);
                    }
                }
                Marca::Dieresis => {
                    rect(cx - 1.8 * u, tope - 1.7 * u, 1.2 * u, 1.2 * u, dx, dy, color);
                    rect(cx + 0.6 * u, tope - 1.7 * u, 1.2 * u, 1.2 * u, dx, dy, color);
                }
                _ => {}
            }
        }
    };
    let grosor = (tam * 0.07).clamp(2.0, 8.0);
    let canto = (tam * 0.06).clamp(1.0, 7.0);
    let baja = |k: f32| (bx * k, by * k);
    // La sombra, suave y corrida.
    let (sx, sy) = baja(canto + grosor * 1.5);
    dibujar(sx + grosor * 0.5, sy, Color::new(0, 0, 0, (a as f32 * 0.45) as u8));
    // El contorno, alrededor de las letras y de su canto.
    let contorno = Color { a, ..oscuro(abajo, 0.2) };
    for k in 0..12 {
        let ang = k as f32 * std::f32::consts::TAU / 12.0;
        let (ox, oy) = (ang.cos() * grosor, ang.sin() * grosor);
        let (cx, cy) = baja(canto);
        dibujar(ox, oy, contorno);
        dibujar(ox + cx, oy + cy, contorno);
    }
    // El canto (las letras "de bloque").
    let lado = Color { a, ..oscuro(abajo, 0.55) };
    for k in (1..=canto.ceil() as i32).rev() {
        let (cx, cy) = baja(k as f32);
        dibujar(cx, cy, lado);
    }
    if grados != 0.0 {
        dibujar(0.0, 0.0, arriba);
        return;
    }
    // Los dos tonos: todo del color de abajo, y la parte de arriba (recortada) del de arriba.
    dibujar(0.0, 0.0, Color { a, ..abajo });
    let corte = tam * 0.5;
    unsafe { ffi::BeginScissorMode((x - grosor) as i32, (y - tam) as i32, (ancho_total + grosor * 2.0) as i32 + 2, (tam + corte) as i32) };
    dibujar(0.0, 0.0, arriba);
    unsafe { ffi::EndScissorMode() };
    // La rayita de brillo donde cambia el color.
    unsafe { ffi::BeginScissorMode((x - grosor) as i32, (y + corte) as i32, (ancho_total + grosor * 2.0) as i32 + 2, (tam * 0.05).max(1.0) as i32) };
    dibujar(0.0, 0.0, Color::new(255, 255, 255, (a as f32 * 0.45) as u8));
    unsafe { ffi::EndScissorMode() };
}

/// El seno, sin el cero negativo (para que un titulo derecho no se mueva ni medio pixel).
fn sin_o_cero(s: f32) -> f32 {
    if s.abs() < 1e-6 { 0.0 } else { s }
}

/// Un texto normal que se lee sobre cualquier fondo (un cielo claro, una explosion): con un
/// contorno oscuro de un pixel y una sombrita. Para los subtitulos y las ayudas encima del 3D.
pub fn subtitulo(d: &mut RaylibDrawHandle, s: &str, x: i32, y: i32, tam: i32, color: Color) {
    let sombra = Color::new(8, 4, 20, (color.a as f32 * 0.85) as u8);
    // La sombra es solo de las letras: los botones e iconos se dibujan una vez, encima.
    let s = con_protagonista(s);
    escribir(d, &s, x + 2, y + 2, tam, sombra, false);
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        escribir(d, &s, x + dx, y + dy, tam, sombra, false);
    }
    escribir(d, &s, x, y, tam, color, true);
}

/// Marcas de un boton dentro de una frase (ver `boton`).
const INICIO_BOTON: char = '\u{E010}';
const FIN_BOTON: char = '\u{E011}';
const ESPACIO_BOTON: char = '\u{E012}';

/// Un boton dentro de una frase: `format!("{}: comprar", ui::boton(e.nombre("aceptar")))`. Al
/// escribirse se dibuja con su icono (la tecla, o el boton del control que se este usando), se
/// mide con el y `envolver` no lo parte. Sirven los nombres de `Entrada::nombre`, las letras y
/// numeros, "flechas", "ratón", "clic", "stick"...; si no hay icono, sale como una teclita.
pub fn boton(nombre: &str) -> String {
    format!("{INICIO_BOTON}{}{FIN_BOTON}", nombre.replace(' ', &ESPACIO_BOTON.to_string()))
}

/// Para escribir letra por letra: si `fin` (un indice de `texto`) cae a media marca de un boton,
/// lo corre al final del boton (el icono sale entero, de una vez).
pub fn sin_partir_botones(texto: &str, fin: usize) -> usize {
    let antes = &texto[..fin];
    match (antes.rfind(INICIO_BOTON), antes.rfind(FIN_BOTON)) {
        (Some(i), f) if f.is_none_or(|f| f < i) => texto[i..].find(FIN_BOTON).map_or(texto.len(), |k| i + k + FIN_BOTON.len_utf8()),
        _ => fin,
    }
}

/// El texto partido en pedazos de texto normal (`false`) y nombres de botones (`true`).
fn pedazos(s: &str) -> Vec<(String, bool)> {
    let mut v = Vec::new();
    let mut resto = s;
    while let Some(i) = resto.find(INICIO_BOTON) {
        let (antes, despues) = resto.split_at(i);
        if !antes.is_empty() {
            v.push((antes.to_string(), false));
        }
        let despues = &despues[INICIO_BOTON.len_utf8()..];
        let fin = despues.find(FIN_BOTON).unwrap_or(despues.len());
        v.push((despues[..fin].replace(ESPACIO_BOTON, " "), true));
        resto = despues.get(fin + FIN_BOTON.len_utf8()..).unwrap_or("");
    }
    if !resto.is_empty() {
        v.push((resto.to_string(), false));
    }
    v
}

/// Si el texto es un solo boton marcado (y nada mas), su nombre.
fn solo_boton(s: &str) -> Option<String> {
    match pedazos(s.trim()).as_slice() {
        [(nombre, true)] => Some(nombre.clone()),
        _ => None,
    }
}

/// Lo ancho de un boton dentro de una frase (los iconos ya traen su orilla transparente; la
/// teclita de repuesto, un poquito de aire a los lados).
fn ancho_boton(nombre: &str, tam: i32) -> i32 {
    crate::controles::ancho_en_linea(nombre, tam).unwrap_or_else(|| medir_crudo(nombre, tam * 4 / 5) + tam / 2 + 4)
}

/// Un boton dentro de una frase: su icono o, si no hay, una teclita lila con el nombre.
fn dibujar_boton(d: &mut RaylibDrawHandle, nombre: &str, x: f32, y: f32, tam: i32, alfa: u8) {
    if crate::controles::dibujar_en_linea(nombre, x as i32, y as i32, tam, alfa).is_some() {
        return;
    }
    let t = tam * 4 / 5;
    let (w, h) = (medir_crudo(nombre, t) + tam / 2, tam + tam / 6);
    let caja = Rectangle::new(x + 2.0, y + (tam - h) as f32 / 2.0, w as f32, h as f32);
    d.draw_rectangle_rounded(Rectangle { y: caja.y + 2.0, ..caja }, 0.4, 6, Color::new(150, 130, 205, alfa));
    d.draw_rectangle_rounded(caja, 0.4, 6, Color::new(232, 224, 252, alfa));
    texto_crudo(d, nombre, x as i32 + 2 + tam / 4, y as i32 + (tam - t) / 2, t, Color::new(40, 20, 70, alfa));
}

/// Centrado en la ventana: con letras grandes, como titulo; con chicas, como subtitulo.
pub fn centrado(d: &mut RaylibDrawHandle, s: &str, ancho: i32, y: i32, tam: i32, color: Color) {
    if tam >= 34 {
        titulo_de(d, s, (ancho - medir_titulo(s, tam)) / 2, y, tam, color);
    } else {
        subtitulo(d, s, (ancho - medir(s, tam)) / 2, y, tam, color);
    }
}

/// Escribe `s` con el font del juego. Pide el `RaylibDrawHandle` para que
/// solo se pueda llamar mientras se dibuja.
pub fn texto(d: &mut RaylibDrawHandle, s: &str, x: i32, y: i32, tam: i32, color: Color) {
    texto_crudo(d, &con_protagonista(s), x, y, tam, color);
}

/// Como `texto`, pero tal cual (sin cambiar "Miku" por el personaje: para la lista de personajes).
/// Los simbolos de los botones del control (ver `SIMBOLOS`) se dibujan como figuras.
pub fn texto_crudo(d: &mut RaylibDrawHandle, s: &str, x: i32, y: i32, tam: i32, color: Color) {
    escribir(d, s, x, y, tam, color, true);
}

/// Escribe el texto con sus botones (ver `boton`) y simbolos; sin `iconos` solo las letras (para
/// las sombras), dejando el hueco de cada boton.
fn escribir(d: &mut RaylibDrawHandle, s: &str, x: i32, y: i32, tam: i32, color: Color, iconos: bool) {
    if s.contains(INICIO_BOTON) {
        let mut x = x;
        for (pedazo, es_boton) in pedazos(s) {
            if es_boton {
                if iconos {
                    dibujar_boton(d, &pedazo, x as f32, y as f32, tam, color.a);
                }
                x += ancho_boton(&pedazo, tam);
            } else {
                escribir(d, &pedazo, x, y, tam, color, iconos);
                x += medir_crudo(&pedazo, tam);
            }
        }
        return;
    }
    if !s.chars().any(es_simbolo) {
        corrida(s, x as f32, y as f32, tam, color);
        return;
    }
    let mut x = x as f32;
    let mut resto = s;
    while let Some(i) = resto.find(es_simbolo) {
        let (antes, despues) = resto.split_at(i);
        if !antes.is_empty() {
            corrida(antes, x, y as f32, tam, color);
            x += medir_corrida(antes, tam) as f32;
        }
        let simbolo = despues.chars().next().unwrap_or(' ');
        if iconos {
            dibujar_simbolo(d, simbolo, x, y as f32, tam as f32, color.a);
        }
        x += ancho_simbolo(tam) as f32;
        resto = &despues[simbolo.len_utf8()..];
    }
    if !resto.is_empty() {
        corrida(resto, x, y as f32, tam, color);
    }
}

/// Un pedazo de texto sin simbolos, con el font del juego.
fn corrida(s: &str, x: f32, y: f32, tam: i32, color: Color) {
    let c = CString::new(s).unwrap_or_default();
    let pos = ffi::Vector2 { x, y };
    let (font, espacio) = fuente(tam as f32);
    unsafe { ffi::DrawTextEx(font, c.as_ptr(), pos, tam as f32, espacio, color.into()) };
    if REGISTRO.with(|r| r.borrow().is_some()) {
        let w = unsafe { ffi::MeasureTextEx(font, c.as_ptr(), tam as f32, espacio).x };
        anotar(x, y, w, tam as f32, s, color.a);
    }
}

// ---------- revision de textos encimados (para las fotos de prueba) ----------

thread_local! {
    /// Los textos dibujados en el cuadro (su rectangulo y que dicen), si se esta revisando.
    static REGISTRO: RefCell<Option<Vec<(Rectangle, String)>>> = const { RefCell::new(None) };
}

/// Prende (o apaga) el registro de textos: con el, `revisar_textos` dice que se encima.
pub fn registrar_textos(si: bool) {
    REGISTRO.with(|r| *r.borrow_mut() = if si { Some(Vec::new()) } else { None });
}

/// Olvida los textos del cuadro anterior (se llama al empezar a dibujar la interfaz).
pub fn reiniciar_registro() {
    REGISTRO.with(|r| {
        if let Some(v) = r.borrow_mut().as_mut() {
            v.clear();
        }
    });
}

fn anotar(x: f32, y: f32, w: f32, h: f32, s: &str, alfa: u8) {
    if alfa < 40 || s.trim().is_empty() {
        return;
    }
    REGISTRO.with(|r| {
        if let Some(v) = r.borrow_mut().as_mut() {
            // Los renglones de un texto se miden por su alto real (no el tamano entero del font).
            v.push((Rectangle::new(x, y + h * 0.1, w, h * 0.8), s.to_string()));
        }
    });
}

/// Lo que se encima o se sale de la ventana en el ultimo cuadro: un renglon por problema.
pub fn revisar_textos(ancho: i32, alto: i32) -> Vec<String> {
    let textos = REGISTRO.with(|r| r.borrow().clone().unwrap_or_default());
    let mut problemas = Vec::new();
    for (k, (a, sa)) in textos.iter().enumerate() {
        if a.x < -1.0 || a.y < -1.0 || a.x + a.width > ancho as f32 + 1.0 || a.y + a.height > alto as f32 + 1.0 {
            problemas.push(format!("FUERA  «{sa}» en ({:.0}, {:.0}) de {:.0}", a.x, a.y, a.width));
        }
        for (b, sb) in textos.iter().skip(k + 1) {
            // Las sombras y contornos son el mismo texto corrido unos pixeles: no cuentan.
            if sa == sb {
                continue;
            }
            if let Some(i) = a.get_collision_rec(b) {
                let menor = (a.width * a.height).min(b.width * b.height).max(1.0);
                if i.width * i.height > menor * 0.2 {
                    problemas.push(format!("ENCIMA «{sa}» con «{sb}» en ({:.0}, {:.0})", i.x, i.y));
                }
            }
        }
    }
    problemas.sort();
    problemas.dedup();
    problemas
}

fn medir_corrida(s: &str, tam: i32) -> i32 {
    let c = CString::new(s).unwrap_or_default();
    let (font, espacio) = fuente(tam as f32);
    unsafe { ffi::MeasureTextEx(font, c.as_ptr(), tam as f32, espacio).x as i32 }
}

/// Los botones del control que se dibujan como su simbolo (caracteres de uso privado de
/// Unicode, que ningun font trae): los de PlayStation y las letras de colores de Xbox.
pub const CRUZ: &str = "\u{E000}";
pub const CIRCULO: &str = "\u{E001}";
pub const CUADRADO: &str = "\u{E002}";
pub const TRIANGULO: &str = "\u{E003}";
pub const XBOX_A: &str = "\u{E004}";
pub const XBOX_B: &str = "\u{E005}";
pub const XBOX_X: &str = "\u{E006}";
pub const XBOX_Y: &str = "\u{E007}";
pub const SWITCH_B: &str = "\u{E00C}";
pub const SWITCH_A: &str = "\u{E00D}";
pub const SWITCH_Y: &str = "\u{E00E}";
pub const SWITCH_X: &str = "\u{E00F}";
/// Flechas (la fuente no las trae): arriba, abajo, izquierda y derecha.
pub const ARRIBA: &str = "\u{E008}";
pub const ABAJO: &str = "\u{E009}";
pub const IZQUIERDA: &str = "\u{E00A}";
pub const DERECHA: &str = "\u{E00B}";

fn es_simbolo(c: char) -> bool {
    ('\u{E000}'..='\u{E00F}').contains(&c)
}

fn ancho_simbolo(tam: i32) -> i32 {
    tam + tam / 8
}

/// El boton, redondo y oscuro, con su figura de color encima (o la letra, en los de Xbox).
fn dibujar_simbolo(d: &mut RaylibDrawHandle, s: char, x: f32, y: f32, tam: f32, alfa: u8) {
    if crate::controles::en_linea(s, x, y, tam, alfa) { return; }
    let c = Vector2::new(x + tam * 0.5, y + tam * 0.5);
    let r = tam * 0.48;
    let con = |r: u8, g: u8, b: u8| Color::new(r, g, b, alfa);
    let xbox = match s {
        '\u{E004}' => Some(("A", con(80, 190, 60))),
        '\u{E005}' => Some(("B", con(225, 60, 60))),
        '\u{E006}' => Some(("X", con(60, 120, 235))),
        '\u{E007}' => Some(("Y", con(240, 190, 40))),
        '\u{E00C}' => Some(("B", con(100, 140, 180))),
        '\u{E00D}' => Some(("A", con(100, 140, 180))),
        '\u{E00E}' => Some(("Y", con(100, 140, 180))),
        '\u{E00F}' => Some(("X", con(100, 140, 180))),
        _ => None,
    };
    // Las flechas: un triangulo con su palito, del color del texto (sin boton atras).
    if ('\u{E008}'..='\u{E00B}').contains(&s) {
        let k = tam * 0.34;
        let color = con(235, 228, 255);
        let (dx, dy) = match s {
            '\u{E008}' => (0.0, -1.0),
            '\u{E009}' => (0.0, 1.0),
            '\u{E00A}' => (-1.0, 0.0),
            _ => (1.0, 0.0),
        };
        let punta = Vector2::new(c.x + dx * k, c.y + dy * k);
        let (px, py) = (-dy, dx);
        let base = Vector2::new(c.x - dx * k * 0.1, c.y - dy * k * 0.1);
        let (b1, b2) = (Vector2::new(base.x + px * k * 0.8, base.y + py * k * 0.8), Vector2::new(base.x - px * k * 0.8, base.y - py * k * 0.8));
        d.draw_triangle(punta, b1, b2, color);
        d.draw_triangle(punta, b2, b1, color);
        d.draw_line_ex(base, Vector2::new(c.x - dx * k, c.y - dy * k), (tam * 0.13).max(2.0), color);
        return;
    }
    if let Some((letra, color)) = xbox {
        d.draw_circle_v(c, r, color);
        let t = (tam * 0.72) as i32;
        corrida(letra, c.x - medir_corrida(letra, t) as f32 / 2.0, c.y - t as f32 * 0.5, t, con(255, 255, 255));
        return;
    }
    d.draw_circle_v(c, r, con(28, 26, 40));
    d.draw_circle_lines(c.x as i32, c.y as i32, r, con(120, 115, 150));
    let k = tam * 0.22;
    let grueso = (tam * 0.10).max(2.0);
    match s {
        '\u{E000}' => {
            let azul = con(130, 175, 255);
            d.draw_line_ex(Vector2::new(c.x - k, c.y - k), Vector2::new(c.x + k, c.y + k), grueso, azul);
            d.draw_line_ex(Vector2::new(c.x - k, c.y + k), Vector2::new(c.x + k, c.y - k), grueso, azul);
        }
        '\u{E001}' => d.draw_ring(c, k - grueso * 0.5, k + grueso * 0.5, 0.0, 360.0, 24, con(255, 105, 115)),
        '\u{E002}' => d.draw_rectangle_lines_ex(Rectangle::new(c.x - k, c.y - k, 2.0 * k, 2.0 * k), grueso * 0.9, con(255, 209, 64)),
        _ => {
            let verde = con(70, 225, 180);
            let (a, b, e) = (Vector2::new(c.x, c.y - k * 1.1), Vector2::new(c.x - k * 1.1, c.y + k * 0.8), Vector2::new(c.x + k * 1.1, c.y + k * 0.8));
            d.draw_line_ex(a, b, grueso, verde);
            d.draw_line_ex(b, e, grueso, verde);
            d.draw_line_ex(e, a, grueso, verde);
        }
    }
}

thread_local! {
    /// El personaje que se escogio: su nombre y si se le habla en femenino.
    static PROTAGONISTA: RefCell<(String, bool)> = RefCell::new(("Miku".into(), true));
    /// El id del personaje (para su forma de hablar).
    static ID_PROTAGONISTA: RefCell<String> = RefCell::new("miku".into());
}

/// Cambia el protagonista de los textos (cada cuadro, con el personaje escogido).
pub fn poner_protagonista(id: &str, nombre: &str, ella: bool) {
    ID_PROTAGONISTA.with(|p| {
        if *p.borrow() != id {
            *p.borrow_mut() = id.to_string();
        }
    });
    PROTAGONISTA.with(|p| {
        let mut p = p.borrow_mut();
        if p.0 != nombre || p.1 != ella {
            *p = (nombre.to_string(), ella);
        }
    });
}

pub fn protagonista() -> &'static crate::personajes::Ficha {
    ID_PROTAGONISTA.with(|id| &crate::personajes::PERSONAJES[crate::personajes::indice(&id.borrow())])
}

/// Los textos se escribieron con Miku de protagonista: aqui "Miku" se vuelve el personaje
/// escogido (y si es el, "Bienvenida" se vuelve "Bienvenido" y "la piloto", "el piloto").
/// Los creditos (con ©) se quedan como estan.
pub fn con_protagonista(s: &str) -> Cow<'_, str> {
    match con_botones(s) {
        Cow::Borrowed(b) => solo_protagonista(b),
        Cow::Owned(o) => Cow::Owned(solo_protagonista(&o).into_owned()),
    }
}

/// Los marcadores de botones en los textos fijos ("Café con {barrel}"): se cambian por el boton de
/// esa accion en el control que se este usando (ver `Entrada::nombre`), dibujado con su icono.
/// Tambien `{flechas}`, `{izq_der}` y `{arriba_abajo}`. Lo demas entre llaves se queda igual.
fn con_botones(s: &str) -> Cow<'_, str> {
    if !s.contains('{') {
        return Cow::Borrowed(s);
    }
    let mut t = String::with_capacity(s.len() + 16);
    let mut resto = s;
    let mut cambio = false;
    while let Some(i) = resto.find('{') {
        let (antes, despues) = resto.split_at(i);
        t.push_str(antes);
        let fin = despues.find('}');
        match fin.and_then(|f| nombre_de_marca(&despues[1..f]).map(|n| (f, n))) {
            Some((f, nombre)) => {
                t.push_str(&boton(&nombre));
                resto = &despues[f + 1..];
                cambio = true;
            }
            None => {
                t.push('{');
                resto = &despues[1..];
            }
        }
    }
    t.push_str(resto);
    if cambio { Cow::Owned(t) } else { Cow::Borrowed(s) }
}

fn nombre_de_marca(marca: &str) -> Option<String> {
    let e = crate::entrada::Entrada { control: crate::controles::control(), ..Default::default() };
    match marca {
        "aceptar" | "aterrizar" | "barrel" | "turbo" | "pausa" | "pasaporte" | "objetivo" | "info" | "piso" | "aire" | "gatillo" | "apuntar" | "orbitar" | "acelerar" => Some(e.nombre(marca).to_string()),
        "flechas" => Some("flechas".into()),
        // Lo que mueve de lado a lado: el stick con control, A y D en el teclado.
        "mover" => Some(if e.control == crate::entrada::Control::Teclado { "A / D".into() } else { "stick".into() }),
        "izq_der" => Some(format!("{IZQUIERDA} {DERECHA}")),
        "arriba_abajo" => Some(format!("{ARRIBA} {ABAJO}")),
        _ => None,
    }
}

/// El nombre del personaje escogido (y su genero) en lugar de "Miku".
fn solo_protagonista(s: &str) -> Cow<'_, str> {
    PROTAGONISTA.with(|p| {
        let p = p.borrow();
        let cambia_nombre = p.0 != "Miku" && (s.contains("Miku") || s.contains("MIKU")) && !s.contains('©') && !s.contains("Hatsune");
        let cambia_genero = !p.1 && (s.contains("ienvenida") || s.contains("a piloto") || s.contains("bogada") || s.contains("Abogada"));
        if !cambia_nombre && !cambia_genero {
            return Cow::Borrowed(s);
        }
        let mut t = s.to_string();
        if cambia_nombre {
            t = t.replace("Miku", &p.0).replace("MIKU", &p.0.to_uppercase());
        }
        if cambia_genero {
            for (de, a) in [
                ("Bienvenida", "Bienvenido"),
                ("bienvenida", "bienvenido"),
                ("La piloto", "El piloto"),
                ("la piloto", "el piloto"),
                ("la abogada", "el abogado"),
                ("La abogada", "El abogado"),
                ("su abogada", "su abogado"),
                ("Abogada", "Abogado"),
                ("abogada", "abogado"),
            ] {
                t = t.replace(de, a);
            }
        }
        Cow::Owned(t)
    })
}

/// Una linea que dice el protagonista, como la diria el personaje escogido (ver `personajes`).
pub fn como_lo_dice_el_protagonista(texto: &str) -> Option<String> {
    ID_PROTAGONISTA.with(|id| crate::personajes::como_lo_dice(&id.borrow(), texto))
}

/// Lo ancho que queda `s` en pixeles.
pub fn medir(s: &str, tam: i32) -> i32 {
    medir_crudo(&con_protagonista(s), tam)
}

pub fn medir_crudo(s: &str, tam: i32) -> i32 {
    if s.contains(INICIO_BOTON) {
        return pedazos(s).iter().map(|(p, es_boton)| if *es_boton { ancho_boton(p, tam) } else { medir_crudo(p, tam) }).sum();
    }
    if !s.chars().any(es_simbolo) {
        return medir_corrida(s, tam);
    }
    let simbolos = s.chars().filter(|&c| es_simbolo(c)).count() as i32;
    let resto: String = s.chars().filter(|&c| !es_simbolo(c)).collect();
    medir_corrida(&resto, tam) + simbolos * ancho_simbolo(tam)
}

pub const TEXTO: Color = Color::new(240, 232, 255, 255);
pub const TENUE: Color = Color::new(190, 180, 215, 255);
pub const LILA: Color = Color::new(200, 150, 255, 255);
pub const VERDE: Color = Color::new(120, 235, 160, 255);
pub const ROJO: Color = Color::new(255, 110, 130, 255);

/// Que tan redondas quedan las esquinas para un radio de `radio` pixeles.
fn redondez(w: i32, h: i32, radio: f32) -> f32 {
    (2.0 * radio / w.min(h).max(1) as f32).min(1.0)
}

/// Un panel: esquinas redondas, sombra, fondo morado oscuro translucido,
/// borde lila y una linea de brillo arriba.
pub fn panel(d: &mut RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32) {
    // El panel de pixel art (el pergamino en tono noche), con su sombrita; si no hay sprites, el de antes.
    d.draw_rectangle(x + 4, y + 6, w, h, Color::new(0, 0, 0, 70));
    if pergamino(d, x as f32, y as f32, w as f32, h as f32, Tono::Noche, 2.0, 238) {
        return;
    }
    let r = redondez(w, h, 12.0);
    let caja = Rectangle::new(x as f32, y as f32, w as f32, h as f32);
    d.draw_rectangle_rounded(Rectangle::new(x as f32 + 3.0, y as f32 + 5.0, w as f32, h as f32), r, 8, Color::new(0, 0, 0, 90));
    d.draw_rectangle_rounded(caja, r, 8, Color::new(20, 12, 42, 222));
    d.draw_rectangle_rounded_lines(caja, r, 8, Color::new(185, 140, 255, 150));
    d.draw_rectangle(x + 14, y + 2, (w - 28).max(0), 1, Color::new(255, 255, 255, 40));
}

/// Un velo oscuro sin borde, que se desvanece hacia `hacia` (para el HUD del espacio: que se lea
/// el texto encima de la nebulosa sin tapar el juego con marcos).
#[derive(Clone, Copy, PartialEq)]
pub enum Hacia {
    Derecha,
    Izquierda,
    Abajo,
}

pub fn velo(d: &mut RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32, hacia: Hacia) {
    let (oscuro, nada) = (Color::new(6, 3, 20, 165), Color::new(6, 3, 20, 0));
    match hacia {
        Hacia::Derecha => d.draw_rectangle_gradient_h(x, y, w, h, oscuro, nada),
        Hacia::Izquierda => d.draw_rectangle_gradient_h(x, y, w, h, nada, oscuro),
        Hacia::Abajo => d.draw_rectangle_gradient_v(x, y, w, h, oscuro, nada),
    }
}

/// Un texto que no cabe en `ancho` pixeles, cortado con "..." al final.
pub fn recortado(s: &str, tam: i32, ancho: i32) -> String {
    if medir(s, tam) <= ancho {
        return s.to_string();
    }
    let mut t: String = s.to_string();
    while !t.is_empty() && medir(&format!("{}...", t.trim_end()), tam) > ancho {
        t.pop();
    }
    format!("{}...", t.trim_end())
}

/// Una tecla dibujada como tecla de teclado (o boton de control): devuelve lo ancho que quedo.
pub fn tecla(d: &mut RaylibDrawHandle, s: &str, x: i32, y: i32, tam: i32, resaltada: bool) -> i32 {
    let alto = tam + tam / 2;
    // Si llega ya marcado como boton (de una frase partida en "tecla: accion"), se desmarca; si
    // trae varios botones con texto ("W / S o stick"), va tal cual, con sus iconos.
    if s.contains(INICIO_BOTON) {
        if let Some(nombre) = solo_boton(s) {
            return tecla(d, &nombre, x, y, tam, resaltada);
        }
        escribir(d, s, x, y + (alto - tam) / 2, tam, TEXTO, true);
        return medir_crudo(s, tam);
    }
    if let Some(w) = crate::controles::dibujar(s, x, y + (alto - crate::controles::lado(tam)) / 2, tam, resaltada) {
        // Los dos botones centrales de PlayStation tienen la misma forma: conservar su nombre.
        if matches!(s, "Create" | "Options") {
            texto(d, s, x + w + 3, y + tam / 4, tam, TEXTO);
            return w + 3 + medir(s, tam);
        }
        return w;
    }
    let w = medir(s, tam) + tam;
    let h = tam + tam / 2;
    let r = redondez(w, h, 6.0);
    let (fondo, borde) = if resaltada { (Color::new(255, 200, 240, 255), Color::new(255, 120, 200, 255)) } else { (Color::new(230, 222, 250, 255), Color::new(150, 130, 200, 255)) };
    d.draw_rectangle_rounded(Rectangle::new(x as f32, y as f32 + 3.0, w as f32, h as f32), r, 6, borde);
    d.draw_rectangle_rounded(Rectangle::new(x as f32, y as f32, w as f32, h as f32), r, 6, fondo);
    texto(d, s, x + tam / 2, y + tam / 4, tam, Color::new(40, 20, 70, 255));
    w
}

/// Una accion: la tecla y lo que hace (`[E] Aterrizar en Tierra`). Devuelve el ancho total.
pub fn accion(d: &mut RaylibDrawHandle, boton: &str, que: &str, x: i32, y: i32, tam: i32, resaltada: bool) -> i32 {
    if boton.is_empty() {
        subtitulo(d, que, x, y + tam / 4, tam, TENUE);
        return medir(que, tam);
    }
    let w = tecla(d, boton, x, y, tam, resaltada);
    let color = if resaltada { Color::new(255, 220, 245, 255) } else { TEXTO };
    subtitulo(d, que, x + w + 8, y + tam / 4, tam, color);
    w + 8 + medir(que, tam)
}

/// Lo ancho de una accion, para centrar una fila de ellas.
pub fn medir_accion(boton: &str, que: &str, tam: i32) -> i32 {
    if boton.is_empty() { medir(que, tam) } else { medir_tecla(boton, tam) + 8 + medir(que, tam) }
}

pub fn medir_tecla(boton: &str, tam: i32) -> i32 {
    if boton.contains(INICIO_BOTON) {
        return solo_boton(boton).map_or_else(|| medir_crudo(boton, tam), |nombre| medir_tecla(&nombre, tam));
    }
    match crate::controles::ancho(boton, tam) {
        Some(w) if matches!(boton, "Create" | "Options") => w + 3 + medir(boton, tam),
        Some(w) => w,
        None => medir(boton, tam) + tam,
    }
}

/// Ayudas al pie: grupos de icono + accion, con salto de linea si la ventana es angosta.
pub fn ayudas(d: &mut RaylibDrawHandle, acciones: &[(&str, &str)], ancho: i32, alto: i32) {
    if acciones.is_empty() { return; }
    let margen = (ancho / 30).clamp(16, 40);
    let disponible = ancho - margen * 2;
    let tam = if ancho < 800 { 13 } else { 15 };
    let paso = 36;
    let separacion = 22;
    let filas = filas_de_ayudas(acciones, ancho);
    let alto_barra = filas.len() as i32 * paso + 14;
    let y0 = alto - alto_barra;
    d.draw_rectangle_gradient_v(0, y0, ancho, alto_barra, Color::new(11, 14, 35, 240), Color::new(7, 9, 24, 255));
    d.draw_rectangle(margen, y0, disponible, 1, Color::new(110, 185, 210, 90));
    for (fila, acciones) in filas.iter().enumerate() {
        let total = acciones.iter().map(|a| a.2).sum::<i32>() + (acciones.len() as i32 - 1) * separacion;
        let mut x = (ancho - total) / 2;
        let y = y0 + 11 + fila as i32 * paso;
        for &(boton, descripcion, w) in acciones {
            let max_texto = (disponible - medir_tecla(boton, tam) - 8).max(20);
            let texto = recortado(descripcion, tam, max_texto);
            accion(d, boton, &texto, x, y, tam, false);
            x += w + separacion;
        }
    }
}

/// Las ayudas acomodadas en filas (las que no caben pasan a la siguiente), con su ancho.
fn filas_de_ayudas<'a>(acciones: &[(&'a str, &'a str)], ancho: i32) -> Vec<Vec<(&'a str, &'a str, i32)>> {
    let disponible = ancho - (ancho / 30).clamp(16, 40) * 2;
    let tam = if ancho < 800 { 13 } else { 15 };
    let separacion = 22;
    let mut filas: Vec<Vec<(&str, &str, i32)>> = vec![Vec::new()];
    let mut ocupado = 0;
    for &(boton, descripcion) in acciones {
        let w = medir_accion(boton, descripcion, tam).min(disponible);
        if ocupado > 0 && ocupado + separacion + w > disponible {
            filas.push(Vec::new());
            ocupado = 0;
        }
        filas.last_mut().unwrap().push((boton, descripcion, w));
        ocupado += w + separacion;
    }
    filas
}

/// Lo alto de la barra de `ayudas` (0 si no hay), para dejarle lugar a lo de encima.
pub fn alto_ayudas(acciones: &[(&str, &str)], ancho: i32) -> i32 {
    if acciones.is_empty() { 0 } else { filas_de_ayudas(acciones, ancho).len() as i32 * 36 + 14 }
}

/// Adapta las ayudas existentes con formato "tecla: accion     tecla: accion".
pub fn ayuda(d: &mut RaylibDrawHandle, texto: &str, ancho: i32, alto: i32) {
    let acciones: Vec<_> = texto.split("     ").filter_map(|p| p.split_once(':')).map(|(b, a)| (b.trim(), a.trim())).collect();
    ayudas(d, &acciones, ancho, alto);
}

/// Lo alto que queda una caja de dialogo de ancho `w`.
pub fn alto_dialogo(nombre: &str, texto: &str, w: i32) -> i32 {
    let ancho_texto = w - 40 - if nombre == NOMBRE_IA { LADO_RETRATO + 16 } else { 0 };
    let lineas = envolver(texto, 22, ancho_texto).len().max(1) as i32;
    let h = 46 + lineas * 29;
    if nombre == NOMBRE_IA { h.max(LADO_RETRATO + 24) } else { h }
}

/// La caja de dialogo de todos los personajes: el nombre arriba y el texto.
/// Si habla GLADAS, le deja lugar a su retrato a la izquierda (y avisa donde).
/// Devuelve lo alto que quedo.
pub fn caja_dialogo(d: &mut RaylibDrawHandle, nombre: &str, color_nombre: Color, texto: &str, x: i32, y: i32, w: i32) -> i32 {
    let h = alto_dialogo(nombre, texto, w);
    panel(d, x, y, w, h);
    let mut tx = x + 20;
    if nombre == NOMBRE_IA {
        let (rx, ry) = (x + 12, y + (h - LADO_RETRATO) / 2);
        d.draw_rectangle_rounded(Rectangle::new(rx as f32 - 2.0, ry as f32 - 2.0, LADO_RETRATO as f32 + 4.0, LADO_RETRATO as f32 + 4.0), 0.15, 6, Color::new(185, 140, 255, 160));
        RETRATO.set(Some((rx, ry, LADO_RETRATO)));
        tx = rx + LADO_RETRATO + 16;
    }
    self::texto(d, nombre, tx, y + 12, 16, color_nombre);
    parrafo(d, texto, tx, y + 34, 22, x + w - 20 - tx, TEXTO);
    h
}

/// Parte el texto en lineas que quepan en `ancho` pixeles.
pub fn envolver(texto: &str, tam: i32, ancho: i32) -> Vec<String> {
    let mut lineas = Vec::new();
    let mut actual = String::new();
    for palabra in texto.split_whitespace() {
        let prueba = if actual.is_empty() { palabra.to_string() } else { format!("{actual} {palabra}") };
        if medir(&prueba, tam) > ancho && !actual.is_empty() {
            lineas.push(std::mem::replace(&mut actual, palabra.to_string()));
        } else {
            actual = prueba;
        }
    }
    if !actual.is_empty() {
        lineas.push(actual);
    }
    lineas
}

/// Escribe el texto partido en lineas y devuelve la `y` donde termino.
pub fn parrafo(d: &mut RaylibDrawHandle, texto: &str, x: i32, y: i32, tam: i32, ancho: i32, color: Color) -> i32 {
    let mut y = y;
    for l in envolver(texto, tam, ancho) {
        self::texto(d, &l, x, y, tam, color);
        y += tam + tam / 3;
    }
    y
}

/// Como se llama la IA de la nave (la que narra, opina y regana).
pub const NOMBRE_IA: &str = "GLADAS";

/// Un mensaje de GLADAS, la IA de la nave: caja abajo al centro.
pub fn mensaje_ia(d: &mut RaylibDrawHandle, texto: &str, ancho_ventana: i32, alto_ventana: i32) {
    mensaje_ia_entre(d, texto, 20, ancho_ventana - 20, alto_ventana);
}

/// Un mensaje de GLADAS abajo, centrado entre `x0` y `x1` (para no tapar el tablero).
pub fn mensaje_ia_entre(d: &mut RaylibDrawHandle, texto: &str, x0: i32, x1: i32, alto_ventana: i32) {
    if texto.is_empty() {
        return;
    }
    let w = 900.min(x1 - x0);
    let h = alto_dialogo(NOMBRE_IA, texto, w);
    caja_dialogo(d, NOMBRE_IA, LILA, texto, x0 + (x1 - x0 - w) / 2, alto_ventana - h - 20, w);
}

// ---------- menus con estilo (inspirado en los de Persona 3, en los colores del juego) ----------

/// Texto girado `grados` alrededor de su esquina de arriba a la izquierda.
pub fn texto_girado(_d: &mut RaylibDrawHandle, s: &str, x: f32, y: f32, tam: f32, grados: f32, color: Color) {
    let c = CString::new(&*con_protagonista(s)).unwrap_or_default();
    let pos = ffi::Vector2 { x, y };
    let origen = ffi::Vector2 { x: 0.0, y: 0.0 };
    let (font, espacio) = fuente(tam);
    unsafe { ffi::DrawTextPro(font, c.as_ptr(), pos, origen, grados, tam, espacio, color.into()) };
}

/// Un paralelogramo inclinado (la esquina de arriba corrida `inclinacion` pixeles a la derecha).
pub fn paralelogramo(d: &mut RaylibDrawHandle, x: f32, y: f32, w: f32, h: f32, inclinacion: f32, color: Color) {
    let (a, b) = (Vector2::new(x + inclinacion, y), Vector2::new(x + w + inclinacion, y));
    let (c, e) = (Vector2::new(x + w, y + h), Vector2::new(x, y + h));
    // Raylib solo pinta los triangulos en un sentido: se mandan en los dos.
    for (p, q, r) in [(a, e, c), (a, c, b)] {
        d.draw_triangle(p, q, r, color);
        d.draw_triangle(p, r, q, color);
    }
}

/// Una fila de un menu con estilo: nombre y (opcional) valor a la derecha.
pub struct Fila {
    pub nombre: String,
    pub valor: String,
    /// Si se ve apagada (por ejemplo, algo que no alcanza a comprarse).
    pub apagada: bool,
}

/// Un menu con estilo: fondo azul noche con franjas diagonales, el titulo gigante girado, las opciones
/// en escalera, y la escogida sobre una banda blanca inclinada que entra deslizandose (`desde` son los
/// segundos desde que se movio el cursor). `reloj` anima las franjas.
#[allow(clippy::too_many_arguments)]
pub fn menu_estilo(d: &mut RaylibDrawHandle, titulo: &str, sub: &str, filas: &[Fila], cursor: usize, desde: f32, reloj: f32, ayuda: &str, ancho: i32, alto: i32) {
    d.draw_rectangle(0, 0, ancho, alto, Color::new(4, 6, 28, 215));
    menu_con_franjas(d, titulo, sub, filas, cursor, desde, reloj, ayuda, ancho, alto, 1.0);
}

/// El mismo menu pero sin tapar la escena (las franjas medio transparentes, sin ayuda abajo).
#[allow(clippy::too_many_arguments)]
pub fn menu_estilo_suave(d: &mut RaylibDrawHandle, titulo: &str, sub: &str, filas: &[Fila], cursor: usize, desde: f32, reloj: f32, ancho: i32, alto: i32) {
    menu_con_franjas(d, titulo, sub, filas, cursor, desde, reloj, "", ancho, alto, 0.6);
}

#[allow(clippy::too_many_arguments)]
fn menu_con_franjas(d: &mut RaylibDrawHandle, titulo: &str, sub: &str, filas: &[Fila], cursor: usize, desde: f32, reloj: f32, ayuda: &str, ancho: i32, alto: i32, opacidad: f32) {
    let (w, h) = (ancho as f32, alto as f32);
    let a = |v: u8| (v as f32 * opacidad) as u8;
    // Las franjas diagonales: el panel azul noche donde va la lista, y arriba, bajo el titulo, una
    // lila y una cian delgada (que corre). Ninguna cruza las opciones.
    let corre = (reloj * 18.0) % 80.0;
    paralelogramo(d, -w * 0.2, h * 0.24, w * 0.86, h * 0.70, w * 0.18, Color::new(14, 32, 100, a(215)));
    paralelogramo(d, -w * 0.25, h * 0.215, w * 0.95, h * 0.022, w * 0.06, Color::new(120, 70, 220, a(220)));
    paralelogramo(d, -w * 0.3 + corre, h * 0.245, w * 0.9, h * 0.006, w * 0.06, Color::new(80, 230, 230, a(230)));
    // El titulo, girado, con su sombra lila (arriba, lejos de la lista).
    // (Debajo de la barra de pestanas del menu del juego.)
    titulo_menu(d, titulo, w * 0.05, h * 0.1, h * 0.068, -5.0);
    if !sub.is_empty() {
        texto_girado(d, sub, w * 0.06, h * 0.1 + h * 0.085, 20.0, -5.0, Color::new(150, 230, 240, 255));
    }
    // Las opciones. Pocas: en escalera, como en Persona. Muchas (los ajustes): una lista recta, con
    // los valores en su columna; si no, se pierden. Las letras se achican para que quepan.
    let escalera = filas.len() <= 6;
    let tam = (h * 0.045).min(h * 0.62 / (filas.len().max(1) as f32 * 1.5)).clamp(16.0, 34.0);
    let paso = tam * 1.5;
    let y0 = h * 0.30;
    let x0 = w * 0.09;
    // La lista deja libre la derecha (ahi va Miku).
    let ancho_fila = (w * 0.5).min(660.0);
    for (k, f) in filas.iter().enumerate() {
        let y = y0 + k as f32 * paso;
        let x = x0 + if escalera { k as f32 * tam * 0.55 } else { 0.0 };
        let escogida = k == cursor;
        let derecha = x + ancho_fila - 30.0;
        if escogida {
            // La banda blanca entra deslizandose desde la izquierda.
            let entra = (desde / 0.12).clamp(0.0, 1.0);
            let largo = ancho_fila * (0.4 + 0.6 * entra);
            paralelogramo(d, x - 24.0 + 6.0, y - 4.0 + 6.0, largo, tam + 10.0, 12.0, Color::new(80, 230, 230, 200));
            paralelogramo(d, x - 24.0, y - 4.0, largo, tam + 10.0, 12.0, Color::WHITE);
        } else if !escalera {
            // En la lista recta, una rayita tenue entre fila y fila.
            d.draw_line_ex(Vector2::new(x - 10.0, y + tam + 5.0), Vector2::new(derecha + 10.0, y + tam + 5.0), 1.0, Color::new(120, 150, 255, 40));
        }
        let color = if escogida {
            Color::new(10, 16, 60, 255)
        } else if f.apagada {
            Color::new(130, 140, 180, 255)
        } else {
            Color::new(225, 235, 255, 255)
        };
        if !escogida {
            texto(d, &f.nombre, x as i32 + 2, y as i32 + 2, tam as i32, Color::new(5, 10, 50, 255));
        }
        texto(d, &f.nombre, x as i32, y as i32 + escogida as i32, tam as i32, color);
        if !f.valor.is_empty() {
            let color_valor = if escogida { Color::new(110, 40, 190, 255) } else { Color::new(160, 200, 255, 255) };
            valor_de_fila(d, &f.valor, derecha, y + tam * 0.5, tam, color_valor);
        }
    }
    if !ayuda.is_empty() {
        self::ayuda(d, ayuda, ancho, alto);
    }
}

/// El valor de una fila, alineado a la derecha en `x`, centrado en `y`. Los niveles (`[##--]`) se
/// dibujan como barritas: llenas las que tiene, vacias las que no.
fn valor_de_fila(d: &mut RaylibDrawHandle, valor: &str, x: f32, y: f32, tam: f32, color: Color) {
    let nivel = valor.strip_prefix('[').and_then(|v| v.strip_suffix(']')).filter(|v| !v.is_empty() && v.chars().all(|c| c == '#' || c == '-'));
    if let Some(barras) = nivel {
        let n = barras.chars().count() as f32;
        let (bw, bh, sep) = (tam * 0.42, tam * 0.62, tam * 0.16);
        let x0 = x - n * (bw + sep) + sep;
        for (k, c) in barras.chars().enumerate() {
            let caja = Rectangle::new(x0 + k as f32 * (bw + sep), y - bh * 0.5, bw, bh);
            if c == '#' {
                d.draw_rectangle_rec(caja, color);
            } else {
                d.draw_rectangle_lines_ex(caja, 2.0, Color::new(color.r, color.g, color.b, 110));
            }
        }
        return;
    }
    let v = (tam * 0.72) as i32;
    // Los si / no van con un interruptor (y lo que diga de mas, a su izquierda).
    let si = valor == "sí" || valor.starts_with("sí ") || valor.starts_with("sí:");
    if si || valor == "no" {
        let alto = tam * 0.62;
        let ancho = interruptor(d, x - alto * 16.0 / 9.0, y - alto * 0.5, alto, si);
        if ancho > 0.0 {
            let resto = if si { valor.trim_start_matches("sí").trim_start_matches(':').trim() } else { "" };
            if !resto.is_empty() {
                texto(d, resto, x as i32 - ancho as i32 - 12 - medir(resto, v), (y - v as f32 * 0.55) as i32, v, color);
            }
            return;
        }
    }
    texto(d, valor, x as i32 - medir(valor, v), (y - v as f32 * 0.55) as i32, v, color);
}

// ---------- sprites de interfaz (el "Pixel UI pack 3": barras, estrellas, insignias...) ----------

/// Las hojas de sprites de la interfaz, en `assets/ui/`: las del "Pixel UI pack 3" (botones,
/// insignias, carga y barras) y las del "Complete UI Essential Pack" de Crusenho (CC BY 4.0: los
/// corchetes de seleccion, la palomita, la tacha, el interruptor y la cinta), y el pergamino, el
/// globo, el marco con diamantes y el divisor del "Humble Gift" de HumblePixel, pasados a los
/// colores del juego (ver `herramientas/recolorear_pergamino.py`).
const HOJAS: [&str; 11] = [
    "assets/ui/botones.png",
    "assets/ui/insignias.png",
    "assets/ui/carga.png",
    "assets/ui/barras.png",
    "assets/ui/seleccion.png",
    "assets/ui/palomita.png",
    "assets/ui/tacha.png",
    "assets/ui/prendido.png",
    "assets/ui/apagado.png",
    "assets/ui/cinta.png",
    "assets/ui/pergamino.png",
];
const BOTONES: usize = 0;
const INSIGNIAS: usize = 1;
const CARGA: usize = 2;
const BARRAS: usize = 3;
const SELECCION: usize = 4;
const PALOMITA: usize = 5;
const TACHA: usize = 6;
const PRENDIDO: usize = 7;
const APAGADO: usize = 8;
const CINTA: usize = 9;
const PERGAMINO: usize = 10;

thread_local! {
    static SPRITES: RefCell<Vec<ffi::Texture2D>> = const { RefCell::new(Vec::new()) };
}

/// Carga las hojas de sprites (despues de abrir la ventana). Con filtro de punto: es pixel art, y
/// agrandado con filtro bilineal se ve borroso. Si falta alguna, se dibuja lo de antes (figuras).
pub fn cargar_sprites(_hilo: &RaylibThread) {
    let mut hojas = Vec::new();
    for ruta in HOJAS {
        let c = CString::new(ruta).unwrap();
        let t = unsafe { ffi::LoadTexture(c.as_ptr()) };
        if t.id == 0 {
            eprintln!("no se pudo cargar {ruta}; la interfaz va sin sprites");
            return;
        }
        unsafe { ffi::SetTextureFilter(t, TextureFilter::TEXTURE_FILTER_POINT as i32) };
        hojas.push(t);
    }
    SPRITES.with(|s| *s.borrow_mut() = hojas);
}

fn hoja(k: usize) -> Option<ffi::Texture2D> {
    SPRITES.with(|s| s.borrow().get(k).copied())
}

/// Un pedazo `fuente` (en pixeles de la hoja) dibujado en `destino`.
fn sprite(t: ffi::Texture2D, fuente: (f32, f32, f32, f32), destino: (f32, f32, f32, f32), tinte: Color) {
    let (sx, sy, sw, sh) = fuente;
    let (dx, dy, dw, dh) = destino;
    unsafe {
        ffi::DrawTexturePro(
            t,
            ffi::Rectangle { x: sx, y: sy, width: sw, height: sh },
            ffi::Rectangle { x: dx, y: dy, width: dw, height: dh },
            ffi::Vector2 { x: 0.0, y: 0.0 },
            0.0,
            tinte.into(),
        )
    };
}

/// Un sprite horizontal estirado sin deformar las puntas (tres partes: la punta izquierda y la
/// derecha a su tamaño, y lo de en medio estirado). `punta` en pixeles de la hoja; `escala`: cuantos
/// pixeles de pantalla por pixel de la hoja.
fn tres_partes(t: ffi::Texture2D, fuente: (f32, f32, f32, f32), punta: f32, x: f32, y: f32, ancho: f32, escala: f32, tinte: Color) {
    let (sx, sy, sw, sh) = fuente;
    let p = punta * escala;
    let medio = (ancho - 2.0 * p).max(0.0);
    sprite(t, (sx, sy, punta, sh), (x, y, p, sh * escala), tinte);
    sprite(t, (sx + punta, sy, sw - 2.0 * punta, sh), (x + p, y, medio, sh * escala), tinte);
    sprite(t, (sx + sw - punta, sy, punta, sh), (x + p + medio, y, p, sh * escala), tinte);
}

/// El color de una barra del pack.
#[derive(Clone, Copy, PartialEq)]
pub enum ColorBarra {
    Rojo,
    Amarillo,
    Azul,
    Verde,
}

/// Una barra de pixel art con su marco redondo plateado: `lleno` de 0 a 1, `alto` en pixeles de
/// pantalla (el ancho que se quiera: se estira sin deformar las puntas). Sin sprites, una barra
/// sencilla del color. Devuelve si se dibujo con sprites.
pub fn barra(d: &mut RaylibDrawHandle, x: f32, y: f32, ancho: f32, alto: f32, lleno: f32, color: ColorBarra) -> bool {
    let lleno = lleno.clamp(0.0, 1.0);
    let Some(t) = hoja(BARRAS) else {
        let c = match color {
            ColorBarra::Rojo => Color::new(235, 70, 80, 255),
            ColorBarra::Amarillo => Color::new(250, 210, 60, 255),
            ColorBarra::Azul => Color::new(60, 170, 240, 255),
            ColorBarra::Verde => Color::new(90, 210, 110, 255),
        };
        d.draw_rectangle(x as i32, y as i32, ancho as i32, alto as i32, Color::new(30, 26, 40, 255));
        d.draw_rectangle(x as i32, y as i32, (ancho * lleno) as i32, alto as i32, c);
        return false;
    };
    // El marco mide 11 de alto y la barra 5 adentro (3 de orilla arriba y abajo).
    let escala = (alto / 11.0).max(1.0);
    let fila = match color {
        ColorBarra::Rojo => 133.0,
        ColorBarra::Azul => 149.0,
        ColorBarra::Amarillo => 165.0,
        ColorBarra::Verde => 181.0,
    };
    tres_partes(t, (6.0, 131.0, 38.0, 11.0), 5.0, x, y, ancho, escala, Color::WHITE);
    let (bx, by, bw) = (x + 3.0 * escala, y + 3.0 * escala, ancho - 6.0 * escala);
    // La barra vacia de fondo, y la llena encima, recortada a lo que va.
    tres_partes(t, (297.0, fila, 32.0, 5.0), 2.0, bx, by, bw, escala, Color::WHITE);
    if lleno > 0.0 {
        unsafe { ffi::BeginScissorMode(bx as i32, by as i32, (bw * lleno).ceil() as i32, (5.0 * escala).ceil() as i32 + 1) };
        tres_partes(t, (57.0, fila, 32.0, 5.0), 2.0, bx, by, bw, escala, Color::WHITE);
        unsafe { ffi::EndScissorMode() };
    }
    true
}

/// Estrellas en fila (`llenas` de `total`), de `lado` pixeles: las grandes del pack (vacias grises,
/// llenas doradas). Sin sprites, no dibuja nada y devuelve false.
pub fn estrellas(d: &mut RaylibDrawHandle, x: f32, y: f32, llenas: u32, total: u32, lado: f32) -> bool {
    let Some(t) = hoja(BOTONES) else { return false };
    let _ = d;
    let ancho = lado * 18.0 / 17.0;
    for k in 0..total {
        let fuente = if k < llenas { (135.0, 135.0, 18.0, 17.0) } else { (7.0, 135.0, 18.0, 17.0) };
        sprite(t, fuente, (x + k as f32 * (ancho + lado * 0.2), y, ancho, lado), Color::WHITE);
    }
    true
}

/// Corazones en fila (`llenos` de `total`, con medios), de `lado` pixeles.
pub fn corazones(d: &mut RaylibDrawHandle, x: f32, y: f32, llenos: f32, total: u32, lado: f32) -> bool {
    let Some(t) = hoja(BOTONES) else { return false };
    let _ = d;
    for k in 0..total {
        let v = (llenos - k as f32).clamp(0.0, 1.0);
        let fuente = if v >= 0.75 { (114.0, 114.0, 12.0, 12.0) } else if v >= 0.25 { (98.0, 114.0, 12.0, 12.0) } else { (82.0, 114.0, 12.0, 12.0) };
        sprite(t, fuente, (x + k as f32 * lado * 1.15, y, lado, lado), Color::WHITE);
    }
    true
}

/// Las insignias del pack (para notas, rangos y premios).
#[derive(Clone, Copy, PartialEq)]
pub enum Insignia {
    /// Con alas: la roja (dorada por dentro) y la azul.
    AlasRoja,
    AlasAzul,
    /// Gemas sin alas: naranja, verde, azul y gris.
    GemaNaranja,
    GemaVerde,
    GemaAzul,
    GemaGris,
}

/// Una insignia centrada en (`cx`, `cy`), `escala` pixeles de pantalla por pixel de la hoja.
pub fn insignia(d: &mut RaylibDrawHandle, cual: Insignia, cx: f32, cy: f32, escala: f32, tinte: Color) -> bool {
    let Some(t) = hoja(INSIGNIAS) else { return false };
    let _ = d;
    let fuente = match cual {
        Insignia::AlasRoja => (80.0, 100.0, 80.0, 40.0),
        Insignia::AlasAzul => (176.0, 100.0, 80.0, 40.0),
        Insignia::GemaNaranja => (86.0, 56.0, 36.0, 32.0),
        Insignia::GemaVerde => (134.0, 56.0, 36.0, 32.0),
        Insignia::GemaAzul => (182.0, 56.0, 36.0, 32.0),
        Insignia::GemaGris => (230.0, 56.0, 36.0, 32.0),
    };
    let (w, h) = (fuente.2 * escala, fuente.3 * escala);
    sprite(t, fuente, (cx - w / 2.0, cy - h / 2.0, w, h), tinte);
    true
}

/// El anillo de "cargando" del pack (los petalos que giran), centrado, de `lado` pixeles; `t` en
/// segundos (cambia de cuadro cada 0.1 s).
pub fn cargando(d: &mut RaylibDrawHandle, cx: f32, cy: f32, lado: f32, t: f32) -> bool {
    let Some(hoja_carga) = hoja(CARGA) else { return false };
    let _ = d;
    let cuadro = (t * 10.0) as usize % 5;
    sprite(hoja_carga, (cuadro as f32 * 48.0, 48.0, 48.0, 48.0), (cx - lado / 2.0, cy - lado / 2.0, lado, lado), Color::WHITE);
    true
}

/// Los corchetes de seleccion (como el cursor de los juegos): una esquinita en cada esquina de
/// lo escogido, que "respira" hacia adentro y hacia afuera. `t` en segundos.
pub fn seleccion(d: &mut RaylibDrawHandle, x: f32, y: f32, ancho: f32, alto: f32, t: f32, tinte: Color) -> bool {
    let Some(h) = hoja(SELECCION) else { return false };
    let _ = d;
    let escala = (alto / 22.0).clamp(2.0, 3.0).round();
    let lado = 10.0 * escala;
    let respira = (0.5 + 0.5 * (t * 6.0).sin()) * 2.0 * escala;
    let afuera = 3.0 * escala - respira;
    let (x0, y0, x1, y1) = (x - afuera, y - afuera, x + ancho + afuera - lado, y + alto + afuera - lado);
    sprite(h, (0.0, 0.0, 10.0, 10.0), (x0, y0, lado, lado), tinte);
    sprite(h, (22.0, 0.0, 10.0, 10.0), (x1, y0, lado, lado), tinte);
    sprite(h, (0.0, 22.0, 10.0, 10.0), (x0, y1, lado, lado), tinte);
    sprite(h, (22.0, 22.0, 10.0, 10.0), (x1, y1, lado, lado), tinte);
    true
}

/// Una palomita verde (o una tacha roja, con `bien` en false) de `alto` pixeles.
pub fn palomita(d: &mut RaylibDrawHandle, x: f32, y: f32, alto: f32, bien: bool) -> bool {
    let Some(h) = hoja(if bien { PALOMITA } else { TACHA }) else { return false };
    let _ = d;
    let (w, hh) = if bien { (17.0, 14.0) } else { (15.0, 15.0) };
    let escala = alto / hh;
    sprite(h, (0.0, 0.0, w, hh), (x, y, w * escala, alto), Color::WHITE);
    true
}

/// Un interruptor (prendido: la perilla a la derecha, verde; apagado: a la izquierda, gris), de
/// `alto` pixeles. Devuelve lo ancho que quedo (0 si no hay sprites).
pub fn interruptor(d: &mut RaylibDrawHandle, x: f32, y: f32, alto: f32, prendido: bool) -> f32 {
    let Some(h) = hoja(if prendido { PRENDIDO } else { APAGADO }) else { return 0.0 };
    let _ = d;
    let escala = alto / 9.0;
    let tinte = if prendido { Color::new(140, 240, 160, 255) } else { Color::new(150, 145, 175, 255) };
    sprite(h, (0.0, 0.0, 16.0, 9.0), (x, y, 16.0 * escala, alto), tinte);
    16.0 * escala
}

/// Una cinta (el liston crema con las puntas dobladas) de `ancho` x `alto`, para un letrero.
pub fn cinta(d: &mut RaylibDrawHandle, x: f32, y: f32, ancho: f32, alto: f32, tinte: Color) -> bool {
    let Some(h) = hoja(CINTA) else { return false };
    let _ = d;
    tres_partes(h, (0.0, 0.0, 64.0, 20.0), 13.0, x, y, ancho, alto / 20.0, tinte);
    true
}

/// Un sprite estirado sin deformar sus orillas (nueve partes: las esquinas a su tamaño, las
/// orillas estiradas a lo largo y el centro estirado en los dos sentidos). Bordes en pixeles de la
/// hoja: izquierda, derecha, arriba y abajo.
#[allow(clippy::too_many_arguments)]
fn nueve_partes(t: ffi::Texture2D, fuente: (f32, f32, f32, f32), bordes: (f32, f32, f32, f32), x: f32, y: f32, ancho: f32, alto: f32, escala: f32, tinte: Color) {
    let (sx, sy, sw, sh) = fuente;
    let (bi, bd, ba, bb) = bordes;
    let cols = [(sx, bi, bi * escala), (sx + bi, sw - bi - bd, (ancho - (bi + bd) * escala).max(0.0)), (sx + sw - bd, bd, bd * escala)];
    let filas = [(sy, ba, ba * escala), (sy + ba, sh - ba - bb, (alto - (ba + bb) * escala).max(0.0)), (sy + sh - bb, bb, bb * escala)];
    let mut py = y;
    for &(fy, fh, dh) in &filas {
        let mut px = x;
        for &(fx, fw, dw) in &cols {
            if dw > 0.0 && dh > 0.0 {
                sprite(t, (fx, fy, fw, fh), (px, py, dw, dh), tinte);
            }
            px += dw;
        }
        py += dh;
    }
}

/// El tono de los paneles de pixel art: papel lila (para lo que es "de papel": la bitacora, los
/// globos) o noche (los paneles de siempre).
#[derive(Clone, Copy, PartialEq)]
pub enum Tono {
    Papel,
    Noche,
}

/// Un panel de pixel art (el pergamino del pack, en los colores del juego), de cualquier tamaño.
/// `escala`: pixeles de pantalla por pixel del sprite (el grosor de la orilla). Sin sprites, no
/// dibuja nada y devuelve false.
#[allow(clippy::too_many_arguments)]
pub fn pergamino(d: &mut RaylibDrawHandle, x: f32, y: f32, ancho: f32, alto: f32, tono: Tono, escala: f32, alfa: u8) -> bool {
    let Some(t) = hoja(PERGAMINO) else { return false };
    let _ = d;
    let sx = if tono == Tono::Papel { 0.0 } else { 184.0 };
    nueve_partes(t, (sx, 0.0, 183.0, 280.0), (8.0, 8.0, 8.0, 8.0), x, y, ancho, alto, escala, Color::new(255, 255, 255, alfa));
    true
}

/// El marco fino con diamantes en las esquinas (y un hueco arriba, para un titulo), teñido.
pub fn marco_diamantes(d: &mut RaylibDrawHandle, x: f32, y: f32, ancho: f32, alto: f32, escala: f32, tinte: Color) -> bool {
    let Some(t) = hoja(PERGAMINO) else { return false };
    let _ = d;
    nueve_partes(t, (368.0, 80.0, 159.0, 249.0), (46.0, 46.0, 16.0, 16.0), x, y, ancho, alto, escala, tinte);
    true
}

/// Un divisor horizontal (una linea con un diamantito en cada punta), teñido.
pub fn divisor(d: &mut RaylibDrawHandle, x: f32, y: f32, ancho: f32, escala: f32, tinte: Color) -> bool {
    let Some(t) = hoja(PERGAMINO) else { return false };
    let _ = d;
    tres_partes(t, (0.0, 290.0, 56.0, 12.0), 8.0, x, y, ancho, escala, tinte);
    true
}

/// Un globo de dialogo de pixel art (con su colita abajo a la izquierda) de `ancho`; mide 34
/// pixeles del sprite de alto (por `escala`). Devuelve false si no hay sprites.
pub fn globo(d: &mut RaylibDrawHandle, x: f32, y: f32, ancho: f32, escala: f32, tono: Tono) -> bool {
    let Some(t) = hoja(PERGAMINO) else { return false };
    let _ = d;
    let sy = if tono == Tono::Papel { 0.0 } else { 40.0 };
    // Las puntas: la izquierda con la colita, la derecha con la rayita.
    let (sx, sw, sh) = (368.0, 109.0, 34.0);
    let (pi, pd) = (18.0, 16.0);
    let medio = (ancho - (pi + pd) * escala).max(0.0);
    sprite(t, (sx, sy, pi, sh), (x, y, pi * escala, sh * escala), Color::WHITE);
    sprite(t, (sx + pi, sy, sw - pi - pd, sh), (x + pi * escala, y, medio, sh * escala), Color::WHITE);
    sprite(t, (sx + sw - pd, sy, pd, sh), (x + pi * escala + medio, y, pd * escala, sh * escala), Color::WHITE);
    true
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Un boton metido en una frase se vuelve a leer entero (con sus espacios), sin partirse.
    #[test]
    fn los_botones_dentro_del_texto_se_leen_enteros() {
        let frase = format!("{}: correr   {}: saltar", boton("W / S"), boton("ESPACIO"));
        let p = pedazos(&frase);
        assert_eq!(p, vec![("W / S".to_string(), true), (": correr   ".to_string(), false), ("ESPACIO".to_string(), true), (": saltar".to_string(), false)]);
        // Las palabras no tienen espacios por dentro: `envolver` no puede partir un boton.
        assert!(!boton("W / S").chars().any(char::is_whitespace));
        // Letra por letra: si el corte cae a media marca, sale el boton entero.
        let medio = frase.char_indices().nth(3).map(|(i, _)| i).unwrap();
        assert_eq!(&frase[..sin_partir_botones(&frase, medio)], boton("W / S"));
        assert_eq!(sin_partir_botones(&frase, 0), 0);
    }

    /// Los marcadores de los textos fijos se cambian por el boton del control que se usa.
    #[test]
    fn los_marcadores_de_los_dialogos_se_vuelven_botones() {
        crate::controles::usar(crate::entrada::Control::Teclado);
        assert_eq!(con_botones("Café con {barrel}."), format!("Café con {}.", boton("Q")));
        crate::controles::usar(crate::entrada::Control::PlayStation);
        assert_eq!(con_botones("Café con {barrel}."), format!("Café con {}.", boton(TRIANGULO)));
        crate::controles::usar(crate::entrada::Control::Teclado);
        assert_eq!(con_botones("{nada} y {flechas"), "{nada} y {flechas");
        assert!(matches!(con_botones("sin llaves"), Cow::Borrowed(_)));
    }
}
