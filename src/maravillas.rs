//! Las MARAVILLAS del sistema solar: homenajes a peliculas, libros y juegos, cada uno donde tiene
//! sentido, para encontrarlos volando (GLADAS cuenta que son al acercarse):
//!
//! - **Interstellar**: el AGUJERO DE GUSANO junto a Saturno (como en la pelicula), con la
//!   Endurance girando a su lado; entrar lleva al otro lado del sistema, a GARGANTUA: un agujero
//!   negro con su disco de acrecion y la luz de las estrellas doblada a su alrededor (el lente
//!   gravitacional es un shader de pantalla, `post::lente`). Gargantua jala la nave; si se cruza
//!   el horizonte, se entra al TESERACTO (un librero de cinco dimensiones) y se sale junto a
//!   Saturno, como Cooper.
//! - **Project Hail Mary**: la LINEA PETROVA (el brillo infrarrojo del astrofago, del Sol a Venus)
//!   y la BLIP-A estacionada junto a Venus, con ROCKY en su burbuja. Rocky habla con acordes.
//! - **2001: Odisea del espacio**: el MONOLITO junto a Jupiter (y GLADAS haciendo de HAL).
//! - **The Martian**: el HUERTO DE PAPAS junto al rover, en el polo de Marte (ver `planetas.rs`).
//! - **La guia del viajero intergalactico**: el rescate dice NO ENTRE EN PANICO (en `main.rs`).

use crate::escena::{CUERPOS, JUPITER, MARTE, SATURNO, VENUS};
use crate::framebuffer::Framebuffer;
use crate::matematica::{Mat4, Vec3};
use crate::nave::Nave;
use crate::obj::Modelo;
use crate::obra::Obra;
use crate::pipeline::proyectar;
use crate::sonido::Efecto;
use std::f32::consts::{PI, TAU};

/// Donde esta Gargantua: lejos del Sol y muy arriba del plano de las orbitas (se ve como un
/// faro desde casi todo el sistema, y su gravedad no alcanza a ningun planeta).
pub const GARGANTUA: Vec3 = Vec3::new(-255.0, 260.0, -348.0);
/// El radio de su sombra (el horizonte de eventos, mas o menos) y el de su disco de acrecion.
pub const SOMBRA: f32 = 14.0;
pub const DISCO: (f32, f32) = (1.7, 4.4);
/// Hasta donde jala, y que tan fuerte (a = FUERZA / d^2).
const ALCANCE: f32 = 210.0;
const FUERZA: f32 = 9000.0;
/// El radio del agujero de gusano.
pub const RADIO_GUSANO: f32 = 7.0;
/// Lo que duran el paso por el agujero de gusano y el teseracto (segundos).
const PASAJE: f32 = 1.6;
const TESERACTO: f32 = 4.2;

/// El agujero de gusano: orbita a Saturno despacito, un poco arriba de sus anillos.
pub fn gusano_en(t: f32) -> Vec3 {
    let a = t * 0.03 + 1.0;
    CUERPOS[SATURNO].posicion(t) + Vec3::new(a.cos() * 55.0, 10.0, a.sin() * 55.0)
}

/// La Endurance, al lado del agujero de gusano.
pub fn endurance_en(t: f32) -> Vec3 {
    gusano_en(t) + Vec3::new(17.0, 3.0, 9.0)
}

/// La Blip-A: junto a Venus, del lado del Sol (sobre la linea Petrova).
pub fn blip_en(t: f32) -> Vec3 {
    let venus = CUERPOS[VENUS].posicion(t);
    let afuera = (venus - CUERPOS[0].posicion(t)).normalize();
    venus - afuera * 24.0 + Vec3::new(0.0, 6.0, 0.0)
}

/// El monolito: entre Jupiter y el Sol, un poco arriba (como el de la pelicula, que orbitaba Jupiter).
pub fn monolito_en(t: f32) -> Vec3 {
    let jupiter = CUERPOS[JUPITER].posicion(t);
    let hacia_sol = (CUERPOS[0].posicion(t) - jupiter).normalize();
    jupiter + hacia_sol * 38.0 + Vec3::new(0.0, 6.0, 0.0)
}

/// El huerto de papas en el polo de Marte, en el mundo (gira con el planeta).
pub fn huerto_en(t: f32) -> Vec3 {
    CUERPOS[MARTE].matriz(t).transform_point(Vec3::new(-0.37, 1.04, -0.1))
}

/// Las matrices de lo que se dibuja: la Endurance girando, la Blip-A, Rocky en su burbuja (dando
/// brinquitos si esta contento), el monolito y el disco de Gargantua.
pub struct Matrices {
    pub endurance: Mat4,
    pub blip: Mat4,
    pub rocky: Mat4,
    pub monolito: Mat4,
    pub disco: Mat4,
}

/// Lo que paso con las maravillas este cuadro (lo aplica el juego).
#[derive(Default)]
pub struct Novedades {
    pub mensaje: Option<String>,
    pub efectos: Vec<Efecto>,
    pub logros: Vec<&'static str>,
    pub sacudida: f32,
}

#[derive(Default)]
pub struct Maravillas {
    /// Lo que GLADAS ya conto en esta partida (un bit por cosa).
    dichas: u32,
    /// El paso por el agujero de gusano y el teseracto: cuanto llevan.
    pub pasaje: Option<f32>,
    pub teseracto: Option<f32>,
    /// Rocky da brinquitos (contento) mientras esto sea mayor que 0.
    baile_rocky: f32,
    /// Si GLADAS esta hablando (lo nuevo espera).
    hablando: bool,
}

// Los bits de `dichas`.
const GUSANO: u32 = 1;
const ENDURANCE: u32 = 2;
const LEJOS_GARGANTUA: u32 = 4;
const CERCA_GARGANTUA: u32 = 8;
const PETROVA: u32 = 16;
const ROCKY: u32 = 32;
const MONOLITO: u32 = 64;
const MARTE_PISTA: u32 = 128;
const HUERTO: u32 = 256;

impl Maravillas {
    /// Si hay una secuencia (el gusano o el teseracto): mientras dure, la nave no se maneja.
    pub fn en_secuencia(&self) -> bool {
        self.pasaje.is_some() || self.teseracto.is_some()
    }

    /// Que tanto se ve el paso por el gusano (0 a 1, con el pico a la mitad: ahi se cambia de lugar).
    pub fn efecto_pasaje(&self) -> f32 {
        self.pasaje.map_or(0.0, |u| (u / PASAJE * PI).sin())
    }

    /// Que tanto se ve el teseracto (0 a 1).
    pub fn efecto_teseracto(&self) -> f32 {
        self.teseracto.map_or(0.0, |u| (u / TESERACTO * PI).sin().powf(0.5))
    }

    /// Esta junto al monolito (para la broma de HAL al apretar E).
    pub fn junto_al_monolito(&self, p: Vec3, t: f32) -> bool {
        (p - monolito_en(t)).length() < 30.0
    }

    /// Cuenta algo (una sola vez); si ya hay algo que decir este cuadro o GLADAS esta hablando,
    /// espera su turno (para no cortarla).
    /// Para la grabacion: GLADAS ya no cuenta nada (`todo`), o solo deja de contar la linea
    /// Petrova (que si no sale en casi cualquier toma cerca del Sol).
    pub fn callar(&mut self, todo: bool) {
        self.dichas |= if todo { u32::MAX } else { PETROVA };
    }

    fn contar(&mut self, bit: u32, n: &mut Novedades, mensaje: &str, efecto: Option<Efecto>) -> bool {
        if self.dichas & bit != 0 || n.mensaje.is_some() || self.hablando {
            return false;
        }
        self.dichas |= bit;
        n.mensaje = Some(mensaje.to_string());
        n.efectos.extend(efecto);
        true
    }

    /// Un cuadro. Con una secuencia en curso la avanza (y mueve la nave cuando toca); si no, la
    /// gravedad de Gargantua, entrar al gusano o cruzar el horizonte, y lo que se cuenta al acercarse.
    pub fn actualizar(&mut self, nave: &mut Nave, deriva: &mut Vec3, t: f32, dt: f32, hablando: bool) -> Novedades {
        let mut n = Novedades::default();
        self.hablando = hablando;
        self.baile_rocky = (self.baile_rocky - dt).max(0.0);
        // El paso por el gusano: a la mitad, la nave aparece del otro lado, viendo a Gargantua.
        if let Some(u) = self.pasaje {
            let ahora = u + dt;
            if u < PASAJE * 0.5 && ahora >= PASAJE * 0.5 {
                let lado = (CUERPOS[0].posicion(t) - GARGANTUA).normalize();
                colocar(nave, GARGANTUA + (lado + Vec3::new(0.0, 0.35, 0.0)).normalize() * 170.0, GARGANTUA);
                *deriva = Vec3::zero();
                n.mensaje = Some("¡Salimos del otro lado! Eso de enfrente es Gargantúa. Mire cómo dobla la luz de las estrellas. No se le acerque mucho: jala.".into());
            }
            self.pasaje = (ahora < PASAJE).then_some(ahora);
            return n;
        }
        // El teseracto: a la mitad, la nave sale junto a Saturno (como Cooper).
        if let Some(u) = self.teseracto {
            let ahora = u + dt;
            if u < TESERACTO * 0.5 && ahora >= TESERACTO * 0.5 {
                let saturno = CUERPOS[SATURNO].posicion(t);
                let lado = (CUERPOS[0].posicion(t) - saturno).normalize();
                colocar(nave, saturno + (lado + Vec3::new(0.0, 0.3, 0.0)).normalize() * 70.0, saturno);
                *deriva = Vec3::zero();
            }
            if ahora >= TESERACTO {
                self.teseracto = None;
                n.mensaje = Some("Salimos junto a Saturno. Como Cooper. Alguien avísele a Murph. Y a la NASA. Y a mi mamá.".into());
                n.logros.push("teseracto");
            } else {
                self.teseracto = Some(ahora);
            }
            return n;
        }
        let p = nave.pos;
        // Gargantua jala (cada vez mas fuerte); cruzar el horizonte es entrar al teseracto.
        let hacia = GARGANTUA - p;
        let d = hacia.length();
        if d < ALCANCE {
            let orilla = ((ALCANCE - d) / (ALCANCE * 0.3)).clamp(0.0, 1.0);
            *deriva += hacia * (FUERZA / (d * d).max(1.0) * orilla / d.max(1.0) * dt);
            n.sacudida = (1.0 - d / 60.0).clamp(0.0, 0.6);
        }
        if d < SOMBRA * 1.1 {
            self.teseracto = Some(0.0);
            *deriva = Vec3::zero();
            nave.velocidad = 0.0;
            n.mensaje = Some("¡Cruzamos el horizonte! Estamos en... ¿un librero? ¿En cinco dimensiones? Alguien del otro lado está tirando libros.".into());
            n.efectos.push(Efecto::Teseracto);
            return n;
        }
        if d < 260.0 {
            self.contar(LEJOS_GARGANTUA, &mut n, "Gargantúa: un agujero negro de verdad. ¿Ve el anillo? Es la luz de atrás, doblada por la gravedad. Cerca de él el tiempo va más lento: cada hora aquí son siete años en la Tierra. Su tarea de las 11:59 ya venció.", None);
        }
        if d < 80.0 {
            self.contar(CERCA_GARGANTUA, &mut n, "Nos está jalando. Si cruzamos el horizonte no hay vuelta atrás. Bueno, en las películas sí.", Some(Efecto::Alarma));
        }
        // El agujero de gusano: verlo y, si se entra, el paso.
        let gusano = (p - gusano_en(t)).length();
        if gusano < RADIO_GUSANO * 0.9 {
            self.pasaje = Some(0.0);
            n.efectos.push(Efecto::Gusano);
            n.logros.push("gusano");
            n.mensaje = Some("¡Adentro del agujero de gusano! Agárrese de algo. De lo que sea. De mí no, que soy software.".into());
            return n;
        }
        if gusano < 70.0 {
            self.contar(GUSANO, &mut n, "Eso que parece una bola de cristal es un agujero de gusano. Junto a Saturno, como en Interstellar. Del otro lado se ve otra parte del cielo. Si entra, sale al otro lado del sistema.", Some(Efecto::Cometa));
        }
        if (p - endurance_en(t)).length() < 40.0 {
            self.contar(ENDURANCE, &mut n, "Es la Endurance. Gira para tener gravedad: doce módulos y ni uno con baño propio. Si la ve dando vueltas sin control, no es momento para precauciones.", None);
        }
        // Project Hail Mary: la linea Petrova cerca de Venus y Rocky en la Blip-A.
        if (p - CUERPOS[VENUS].posicion(t)).length() < 110.0 {
            self.contar(PETROVA, &mut n, "¿Ve esa línea roja del Sol a Venus? Es la línea Petrova: astrófago, bichitos que comen luz de estrella. Si el Sol se empieza a apagar, ya sabemos a quién reclamarle.", None);
        }
        if (p - blip_en(t)).length() < 45.0 && self.contar(ROCKY, &mut n, "¡Una nave eridiana! Es la Blip-A, y en la burbuja está Rocky. Los eridianos hablan con acordes... Traduzco: «¡Amigo! ¡Puño mi choque!». Dice que la música es ciencia. Amaze, amaze, amaze.", Some(Efecto::Rocky)) {
            self.baile_rocky = 4.0;
            n.logros.push("rocky");
        }
        // 2001.
        if (p - monolito_en(t)).length() < 40.0 && self.contar(MONOLITO, &mut n, "Un monolito. Negro, liso, de uno por cuatro por nueve: los cuadrados de uno, dos y tres. Alguien lo dejó junto a Júpiter. Dios mío... está lleno de estrellas.", Some(Efecto::Monolito)) {
            n.logros.push("monolito");
        }
        // The Martian: primero la pista (cerca de Marte), luego el huerto (pasando por el polo).
        let marte = &CUERPOS[MARTE];
        if (p - marte.posicion(t)).length() < marte.radio_visual() * 3.5 {
            self.contar(MARTE_PISTA, &mut n, "En el polo de arriba de Marte hay algo verde junto al rover. En Marte. Verde. Vaya a ver, yo no tengo ojos.", None);
        }
        if (p - huerto_en(t)).length() < 9.0 && self.contar(HUERTO, &mut n, "¡Un huerto de papas en Marte, junto al rover! Alguien tuvo que ponerse muy científico para sobrevivir aquí. Abonó con... mejor no pregunte.", Some(Efecto::Sello)) {
            n.logros.push("huerto");
        }
        n
    }

    /// Las matrices de todo lo que se dibuja, en el tiempo `t` (y `reloj` para lo que siempre se
    /// mueve aunque el mundo se congele).
    pub fn matrices(&self, t: f32, reloj: f32) -> Matrices {
        let blip = blip_en(t);
        // La Blip-A ve hacia el Sol (la nariz va hacia -Z del modelo).
        let hacia_sol = (CUERPOS[0].posicion(t) - blip).normalize();
        let rumbo = (-hacia_sol.x).atan2(-hacia_sol.z);
        let m_blip = Mat4::translation(blip) * Mat4::rotation_y(rumbo) * Mat4::scale(2.2);
        // Rocky en el centro de su burbuja (ver generar_referencias.py), en el piso de la burbuja.
        let brinco = if self.baile_rocky > 0.0 { (reloj * 9.0).sin().abs() * 0.22 } else { 0.03 * (reloj * 2.0).sin() };
        let m_rocky = m_blip * Mat4::translation(Vec3::new(2.75, 0.15 + brinco, 0.625)) * Mat4::rotation_y(reloj * if self.baile_rocky > 0.0 { 3.0 } else { 0.4 }) * Mat4::scale(0.9);
        let monolito = monolito_en(t);
        let m_monolito = Mat4::translation(monolito) * Mat4::rotation_y(reloj * 0.08) * Mat4::scale(4.0);
        Matrices {
            // Inclinada para que se lea el anillo (de canto parece una fila de cajas).
            endurance: Mat4::translation(endurance_en(t)) * Mat4::rotation_z(0.5) * Mat4::rotation_x(1.05) * Mat4::rotation_y(reloj * 0.6) * Mat4::scale(2.4),
            blip: m_blip,
            rocky: m_rocky,
            monolito: m_monolito,
            // El disco, casi de canto (como se ve en la pelicula) y girando.
            disco: Mat4::translation(GARGANTUA) * Mat4::rotation_x(0.12) * Mat4::rotation_z(-0.18) * Mat4::rotation_y(reloj * 0.25) * Mat4::scale(SOMBRA),
        }
    }
}

/// Pone la nave en `pos`, quieta, viendo hacia `mira`.
fn colocar(nave: &mut Nave, pos: Vec3, mira: Vec3) {
    nave.pos = pos;
    nave.velocidad = 0.0;
    (nave.yaw, nave.pitch) = crate::salto::rumbo_hacia(mira - pos);
}

/// El disco de acrecion de Gargantua: un anillo plano (de las dos caras) entre los radios de
/// `DISCO`, en unidades de la sombra. Se dibuja con el shader de plasma (los remolinos de gas).
pub fn disco() -> Modelo {
    let mut o = Obra::new();
    o.material("disco", [1.0, 0.7, 0.3], [1.0, 0.6, 0.2], 0.0, 1.0);
    let lados = 96;
    let anillos = 4;
    for i in 0..lados {
        let (a0, a1) = (i as f32 / lados as f32 * TAU, (i + 1) as f32 / lados as f32 * TAU);
        for k in 0..anillos {
            let r0 = DISCO.0 + (DISCO.1 - DISCO.0) * k as f32 / anillos as f32;
            let r1 = DISCO.0 + (DISCO.1 - DISCO.0) * (k + 1) as f32 / anillos as f32;
            let p = |a: f32, r: f32| Vec3::new(a.cos() * r, 0.0, a.sin() * r);
            let q = [p(a0, r0), p(a1, r0), p(a1, r1), p(a0, r1)];
            o.quad("disco", q, Vec3::new(0.0, 1.0, 0.0));
            o.quad("disco", q, Vec3::new(0.0, -1.0, 0.0));
        }
    }
    o.modelo()
}

/// La linea Petrova: un hilo de luz infrarroja del Sol a Venus que titila (aditiva, detras de lo
/// que tenga enfrente).
pub fn dibujar_petrova(fb: &mut Framebuffer, vista_proyeccion: &Mat4, t: f32, reloj: f32) {
    let sol = CUERPOS[0].posicion(t);
    let venus = CUERPOS[VENUS].posicion(t);
    let (desde, hasta) = (sol + (venus - sol).normalize() * CUERPOS[0].escala * 1.05, venus - (venus - sol).normalize() * CUERPOS[VENUS].escala * 1.1);
    let pasos = 48;
    for k in 0..pasos {
        let (u0, u1) = (k as f32 / pasos as f32, (k + 1) as f32 / pasos as f32);
        let (a, b) = (desde.lerp(hasta, u0), desde.lerp(hasta, u1));
        if let (Some(pa), Some(pb)) = (proyectar(a, vista_proyeccion, fb), proyectar(b, vista_proyeccion, fb)) {
            // Titila a lo largo (el astrofago viaja en grumos).
            let k = 0.55 + 0.45 * ((u0 * 40.0 - reloj * 3.0).sin() * 0.5 + 0.5);
            let c = [0.55 * k, 0.04 * k, 0.10 * k];
            fb.linea_de_luz(pa, pb, c, [0.25 * k, 0.0, 0.05 * k]);
            fb.linea_de_luz((pa.0, pa.1 + 1.0, pa.2), (pb.0, pb.1 + 1.0, pb.2), c.map(|v| v * 0.5), [0.0; 3]);
        }
    }
}

/// El teseracto (Interstellar): un pasillo de cuadros dorados (los estantes de un librero de cinco
/// dimensiones) que vienen hacia la camara, sobre la imagen oscurecida. `k`: que tanto se ve.
pub fn dibujar_teseracto(fb: &mut Framebuffer, k: f32, reloj: f32) {
    if k <= 0.0 {
        return;
    }
    let (w, h) = (fb.ancho as f32, fb.alto as f32);
    // Oscurecer lo de atras.
    for i in 0..fb.ancho * fb.alto {
        let c = fb.leer(i);
        fb.pintar(i, c.map(|v| v * (1.0 - 0.85 * k)), 1.0);
    }
    let (cx, cy) = (w * 0.5, h * 0.5);
    let dorado = [1.0 * k, 0.78 * k, 0.4 * k];
    // Los marcos: cuadros que crecen desde el centro (como si se avanzara por el pasillo).
    for n in 0..14 {
        let u = ((n as f32 / 14.0) + reloj * 0.35).fract();
        let escala = 0.04 + u * u * 1.3;
        let (mw, mh) = (w * escala, h * escala);
        let brillo = u * (1.0 - u) * 4.0;
        let c = dorado.map(|v| v * brillo);
        let esquinas = [(cx - mw, cy - mh), (cx + mw, cy - mh), (cx + mw, cy + mh), (cx - mw, cy + mh)];
        for e in 0..4 {
            let (a, b) = (esquinas[e], esquinas[(e + 1) % 4]);
            fb.linea_de_luz((a.0, a.1, 0.0), (b.0, b.1, 0.0), c, c.map(|v| v * 0.3));
        }
        // Los estantes: rayitas verticales dentro de cada marco (los lomos de los libros).
        for s in 0..8 {
            let x = cx - mw + (s as f32 + 0.5) / 8.0 * mw * 2.0;
            fb.linea_de_luz((x, cy - mh, 0.0), (x, cy - mh * 0.6, 0.0), c.map(|v| v * 0.6), [0.0; 3]);
        }
    }
    // Las lineas que se van al fondo (la perspectiva del pasillo).
    for (x, y) in [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h), (cx, 0.0), (cx, h), (0.0, cy), (w, cy)] {
        fb.linea_de_luz((x, y, 0.0), (cx, cy, 0.0), dorado.map(|v| v * 0.5), [0.0; 3]);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Entrar al gusano lleva junto a Gargantua; cruzar el horizonte, al teseracto y de ahi a
    /// Saturno. Cada cosa se cuenta una sola vez.
    #[test]
    fn el_gusano_y_el_teseracto_llevan_a_donde_deben() {
        let mut m = Maravillas::default();
        let mut nave = Nave::new(gusano_en(10.0));
        let mut deriva = Vec3::zero();
        let mut logros = Vec::new();
        let mut t = 10.0;
        let n = m.actualizar(&mut nave, &mut deriva, t, 1.0 / 60.0, false);
        logros.extend(n.logros);
        assert!(m.pasaje.is_some(), "no entro al gusano");
        while m.en_secuencia() {
            t += 1.0 / 60.0;
            m.actualizar(&mut nave, &mut deriva, t, 1.0 / 60.0, false);
        }
        assert!((nave.pos - GARGANTUA).length() < 200.0, "no salio junto a Gargantua");
        // Gargantua jala: dejada quieta, la deriva va hacia el.
        let antes = (nave.pos - GARGANTUA).length();
        for _ in 0..120 {
            m.actualizar(&mut nave, &mut deriva, t, 1.0 / 60.0, false);
            nave.pos += deriva * (1.0 / 60.0);
        }
        assert!((nave.pos - GARGANTUA).length() < antes, "Gargantua no jala");
        // Cruzando el horizonte: el teseracto, y se sale junto a Saturno.
        nave.pos = GARGANTUA + Vec3::new(SOMBRA, 0.0, 0.0);
        m.actualizar(&mut nave, &mut deriva, t, 1.0 / 60.0, false);
        assert!(m.teseracto.is_some());
        while m.en_secuencia() {
            t += 1.0 / 60.0;
            logros.extend(m.actualizar(&mut nave, &mut deriva, t, 1.0 / 60.0, false).logros);
        }
        assert!((nave.pos - CUERPOS[SATURNO].posicion(t)).length() < 100.0, "no salio junto a Saturno");
        assert!(logros.contains(&"gusano") && logros.contains(&"teseracto"));
    }

    /// Rocky, el monolito y el huerto: se encuentran acercandose (una vez cada uno).
    #[test]
    fn se_encuentran_acercandose() {
        let t = 30.0;
        for (donde, logro) in [(blip_en(t), "rocky"), (monolito_en(t), "monolito"), (huerto_en(t), "huerto")] {
            let mut m = Maravillas::default();
            let mut nave = Nave::new(donde + Vec3::new(0.0, 3.0, 0.0));
            let mut deriva = Vec3::zero();
            let mut todos = Vec::new();
            for _ in 0..3 {
                todos.extend(m.actualizar(&mut nave, &mut deriva, t, 1.0 / 60.0, false).logros);
            }
            assert_eq!(todos.iter().filter(|l| **l == logro).count(), 1, "{logro}: {todos:?}");
        }
    }
}
