//! Aterrizaje en URANO: EL BANO DE DONA CHEPA. El unico edificio de Urano es un bano publico, y
//! hoy hay feria: llega todo el sistema solar. Dona Chepa cobra desde su banco; usted hace todo lo
//! demas (estilo *Overcooked*: caos contra el reloj).
//!
//! - Los clientes (aliens) hacen fila en la entrada. En la CAJA se les cobra Q1 (papel incluido) y
//!   se van solos al primer cubiculo libre; si cobro rapido, dejan propina.
//! - Cada cubiculo tiene su papel: cuando se acaba, el que esta adentro grita "¡PAPEL!" y hay que
//!   llevarle un rollo del ESTANTE.
//! - A veces el inodoro se TAPA: el cubiculo no se puede usar hasta destaparlo (con el destapacanos,
//!   apretando varias veces).
//! - En los lavamanos y frente a los cubiculos salen CHARCOS: si un cliente pasa encima se resbala
//!   (y se pierde reputacion). Se secan con el trapeador (manteniendo el boton).
//! - Los clientes pierden la paciencia en la fila (los "urgidos", mas rapido); si se les acaba, se
//!   van enojados.
//! Tres dias (lunes, viernes de quincena y domingo de feria), cada vez mas locos; para pasar el dia
//! hay que juntar la meta de quetzales sin bajar de dos estrellas.
//!
//! Camara fija desde arriba (como en Overcooked); el bano esta en metros, con el frente hacia +Z.

use crate::vida::{Tam, Tono, Vida};
use crate::entrada::Entrada;
use crate::matematica::{Mat4, Vec3, Vec4};
use crate::sonido::{Efecto, Voz};
use crate::ui;
use raylib::prelude::*;
use std::f32::consts::PI;

pub const FOV: f32 = 50.0;
const ASPECTO: f32 = 16.0 / 9.0;
const OJO: Vec3 = Vec3::new(0.0, 9.6, 8.1);
const MIRA: Vec3 = Vec3::new(0.0, 0.0, 0.5);

/// El bano (igual que en `generar_bano.py`).
pub const CUBICULOS: usize = 4;
pub fn x_cubiculo(n: usize) -> f32 {
    -4.4 + (n as f32 + 0.5) * 1.5
}
/// Donde se para uno frente a la puerta de un cubiculo, y donde queda el cliente adentro.
const Z_PUERTA: f32 = -1.85;
const Z_ADENTRO: f32 = -3.0;
/// Donde se para uno para cobrar, y la fila (de frente a atras).
const CAJA: Vec3 = Vec3::new(-3.05, 0.0, 2.6);
const FILA_X: f32 = -4.5;
const FILA_Z: f32 = 2.85;
const PASO_FILA: f32 = 0.8;
/// El carril de salida (por el hueco de la entrada) y afuera.
const SALIDA_X: f32 = -3.6;
const AFUERA: Vec3 = Vec3::new(-4.1, 0.0, 7.5);
/// El estante (lo que hay y a que altura de la pared) y los lavamanos.
const X_ESTANTE: f32 = 3.95;
const ESTANTE: [(Item, f32); 3] = [(Item::Papel, -1.0), (Item::Destapacanos, 0.0), (Item::Trapeador, 1.0)];
const X_LAVAMANOS: f32 = 3.9;
const LAVAMANOS: [f32; 2] = [2.2, 3.3];
/// Donde esta sentada Dona Chepa (mirando hacia la entrada).
pub const CHEPA: Vec3 = Vec3::new(-4.35, 0.0, 0.55);
const NOMBRE_CHEPA: &str = "DOÑA CHEPA";

/// Velocidades (m/s) y radios.
const VEL_JUGADOR: f32 = 3.4;
const VEL_ALIEN: f32 = 1.7;
const ALCANCE: f32 = 1.15;
/// Rollos que trae un cubiculo cuando se le pone papel.
const ROLLOS: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Item {
    Papel,
    Destapacanos,
    Trapeador,
}

impl Item {
    pub fn nombre(self) -> &'static str {
        match self {
            Item::Papel => "papel",
            Item::Destapacanos => "destapacaños",
            Item::Trapeador => "trapeador",
        }
    }
}

/// Que hace un cliente.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Hace {
    /// En la fila (su lugar se calcula cada cuadro).
    Fila,
    /// Caminando a su cubiculo.
    AlCubiculo(usize),
    /// Adentro, con los segundos que le faltan.
    Adentro(usize, f32),
    /// Adentro y sin papel: esperando que le lleven.
    SinPapel(usize),
    /// Saliendo del cubiculo hacia un lavamanos.
    AlLavamanos(usize),
    Lavando(usize, f32),
    /// Yendose (contento).
    Saliendo,
    /// Yendose enojado (se le acabo la paciencia).
    Enojado,
}

#[derive(Clone, Debug)]
pub struct Alien {
    pub pos: Vec3,
    pub rumbo: f32,
    /// Que color de piel tiene (indice en `COLORES_ALIEN`).
    pub color: usize,
    nombre: &'static str,
    hace: Hace,
    camino: Vec<Vec3>,
    /// De 1 (tranquilo) a 0 (se va).
    pub paciencia: f32,
    urgido: bool,
    /// Segundos que le quedan de resbalon (da vueltas en el piso), y de no volverse a resbalar
    /// (para que al levantarse no se caiga otra vez en el mismo charco).
    pub resbalon: f32,
    inmune: f32,
    pub paso: f32,
    propina: u32,
}

/// Los colores de los clientes (para recolorear el modelo).
pub const COLORES_ALIEN: [[f32; 3]; 5] = [[0.45, 0.85, 0.55], [0.55, 0.65, 1.0], [1.0, 0.62, 0.78], [1.0, 0.82, 0.35], [0.75, 0.55, 0.95]];

const NOMBRES: [&str; 12] = [
    "el de Neptuno", "la de Marte", "un pluton", "el de Júpiter", "la de Venus", "un mercuriano", "el de Saturno", "una lunática", "el del cometa",
    "la de Ganímedes", "un tal Titán", "el de Ceres",
];

pub struct Cubiculo {
    pub ocupado: bool,
    pub papel: u32,
    pub tapado: bool,
    /// Cuanto se ha destapado (0 a 1).
    pub destape: f32,
    /// La puerta: 0 cerrada, 1 abierta.
    pub puerta: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Charco {
    pub pos: Vec3,
    /// Su radio (se achica al trapear).
    pub tam: f32,
}

pub struct Jugador {
    pub pos: Vec3,
    pub rumbo: f32,
    vel: Vec3,
    pub carga: Option<Item>,
    pub paso: f32,
}

struct Dia {
    nombre: &'static str,
    lema: &'static str,
    dura: f32,
    cada: (f32, f32),
    tapado: f32,
    charco: f32,
    urgidos: f32,
    meta: u32,
}

const DIAS: [Dia; 3] = [
    Dia { nombre: "DÍA 1 · LUNES", lema: "tranquilo... por ahora", dura: 100.0, cada: (6.0, 8.5), tapado: 0.12, charco: 0.35, urgidos: 0.15, meta: 9 },
    Dia { nombre: "DÍA 2 · VIERNES DE QUINCENA", lema: "todos cenaron shucos", dura: 115.0, cada: (4.2, 6.2), tapado: 0.25, charco: 0.45, urgidos: 0.35, meta: 13 },
    Dia { nombre: "DÍA 3 · DOMINGO DE FERIA", lema: "llega la excursión de Plutón", dura: 130.0, cada: (3.4, 5.0), tapado: 0.3, charco: 0.55, urgidos: 0.5, meta: 17 },
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Fase {
    /// Las frases de llegada (cual va).
    Llegada(usize),
    /// El letrero del dia antes de abrir.
    Abriendo,
    Jugando,
    /// Se acabo el dia: si se paso o no.
    FinDia(bool),
    /// Los tres dias pasados.
    Final,
}

pub enum AccionUrano {
    Nada,
    Despegar,
}

/// Lo que esta enfrente del jugador (con lo que interactua).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Objetivo {
    Caja,
    Estante(Item),
    Cubiculo(usize),
    Charco(usize),
}

const LLEGADA: [(&str, &str, Voz); 3] = [
    (ui::NOMBRE_IA, "Urano: gira acostado a 98 grados. Lo único que hay es un baño público. Y hoy hay feria. Mal día para tener ganas.", Voz::Ia),
    (NOMBRE_CHEPA, "¡Ay, mijo, qué bueno que vino! Hoy viene todo el sistema solar. Usted cobra en la caja: Q1, con papel incluido. Yo superviso desde mi banco.", Voz::Planeta),
    (NOMBRE_CHEPA, "Si se acaba el papel, del estante. Si alguien tapa el inodoro, destapacaños. Y los charcos, con el trapeador, que si alguien se resbala me demandan.", Voz::Planeta),
];

pub struct Urano {
    /// Las chispas de cuando algo sale bien (o mal).
    pub jugo: crate::chispas::Jugo,
    fase: Fase,
    /// Segundos en la fase.
    t: f32,
    pub total: f32,
    pub dia: usize,
    /// Segundos que le quedan al dia.
    reloj: f32,
    hasta_cliente: f32,
    excursion: bool,
    pub aliens: Vec<Alien>,
    pub cubiculos: Vec<Cubiculo>,
    pub charcos: Vec<Charco>,
    pub jugador: Jugador,
    pub quetzales: u32,
    /// De 0 a 5 estrellas.
    pub reputacion: f32,
    atendidos: u32,
    enojados: u32,
    resbalones: u32,
    semilla: u32,
    mensaje: Option<(&'static str, String, Voz, f32)>,
    /// Destello rojo (algo salio mal) y cuanto tiembla la camara.
    pub dano: f32,
    pub sacudida: f32,
    /// Donde quedo la imagen 3D en la ventana (para poner los iconos encima).
    pub imagen: (f32, f32, f32, f32),
    pub partidas: u32,
    /// Los dias cumplidos (meta de quetzales y dos estrellas o mas) en esta visita.
    pub dias_ganados: u32,
    pub campeon: bool,
    pub record: u32,
    /// Los letreros gigantes, el confeti y las sacudidas (ver vida.rs).
    pub vida: Vida,
    pub eventos: Vec<Efecto>,
}

fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

impl Urano {
    pub fn new(semilla: u32) -> Self {
        let mut u = Urano {
            fase: Fase::Llegada(0),
            t: 0.0,
            total: 0.0,
            dia: 0,
            reloj: DIAS[0].dura,
            hasta_cliente: 1.0,
            excursion: false,
            aliens: Vec::new(),
            cubiculos: Vec::new(),
            charcos: Vec::new(),
            jugador: Jugador { pos: Vec3::new(-1.0, 0.0, 1.2), rumbo: 0.0, vel: Vec3::zero(), carga: None, paso: 0.0 },
            quetzales: 0,
            reputacion: 3.5,
            atendidos: 0,
            enojados: 0,
            resbalones: 0,
            semilla: semilla | 1,
            mensaje: None,
            dano: 0.0,
            sacudida: 0.0,
            jugo: Default::default(),
            imagen: (0.0, 0.0, 1.0, 1.0),
            partidas: 0,
            dias_ganados: 0,
            campeon: false,
            record: 0,
            vida: Vida::new("EL BAÑO DE DOÑA CHEPA", "Q1, papel incluido", Tono::Cian),
            eventos: Vec::new(),
        };
        u.abrir_dia(0);
        u.fase = Fase::Llegada(0);
        let (q, t, v) = LLEGADA[0];
        u.decir(q, t, v);
        u
    }

    fn azar(&mut self) -> f32 {
        self.semilla = self.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.semilla >> 8) as f32 / 16_777_216.0
    }

    fn decir(&mut self, quien: &'static str, texto: impl Into<String>, voz: Voz) {
        let texto = texto.into();
        let dura = 2.5 + texto.chars().count() as f32 * 0.045;
        self.mensaje = Some((quien, texto, voz, dura));
    }

    fn chepa(&mut self, texto: impl Into<String>) {
        self.decir(NOMBRE_CHEPA, texto, Voz::Planeta);
    }

    fn cambiar(&mut self, fase: Fase) {
        self.fase = fase;
        self.t = 0.0;
    }

    /// Deja el bano listo para el dia `dia`.
    fn abrir_dia(&mut self, dia: usize) {
        self.dia = dia;
        self.reloj = DIAS[dia].dura;
        self.hasta_cliente = 1.5;
        self.excursion = false;
        self.aliens.clear();
        self.charcos.clear();
        self.cubiculos = (0..CUBICULOS).map(|_| Cubiculo { ocupado: false, papel: ROLLOS, tapado: false, destape: 0.0, puerta: 0.0 }).collect();
        self.jugador = Jugador { pos: Vec3::new(-1.0, 0.0, 1.2), rumbo: 0.0, vel: Vec3::zero(), carga: None, paso: 0.0 };
        self.quetzales = 0;
        self.reputacion = 3.5;
        self.atendidos = 0;
        self.enojados = 0;
        self.resbalones = 0;
        self.cambiar(Fase::Abriendo);
    }

    /// Que el juego esta en marcha (para esconder el cursor y no despegar sin querer).
    pub fn jugando(&self) -> bool {
        matches!(self.fase, Fase::Jugando)
    }

    pub fn hablante(&self) -> Option<(&'static str, String, Voz)> {
        self.mensaje.as_ref().map(|(q, t, v, _)| (*q, t.clone(), *v))
    }

    pub fn despedida(&self) -> String {
        if self.campeon {
            format!("Despegue de Urano. El baño más limpio del sistema solar: {} quetzales en el mejor día. Doña Chepa ya le guardó su gafete de empleado del mes.", self.record)
        } else if self.partidas > 0 {
            "Despegue de Urano. Doña Chepa dice que vuelva mañana. Que traiga trapeador propio.".into()
        } else {
            "Despegue de Urano. Ni entramos al baño. Mejor: la fila llegaba hasta Neptuno.".into()
        }
    }

    // -----------------------------------------------------------------------
    // La camara y la proyeccion (las mismas del render)
    // -----------------------------------------------------------------------

    /// La camara fija desde arriba; se corre un poquito con el jugador y tiembla con los sustos.
    pub fn camara(&self) -> (Vec3, Vec3) {
        let s = self.sacudida * 0.08;
        let temblor = Vec3::new((self.total * 53.0).sin() * s, (self.total * 47.0).cos() * s, 0.0);
        let sigue = Vec3::new(self.jugador.pos.x * 0.12, 0.0, self.jugador.pos.z * 0.08);
        if let Fase::Llegada(_) = self.fase {
            // Al llegar: entrando por la puerta, hasta quedar arriba.
            let u = suave(self.total / 6.0);
            let desde = (Vec3::new(-4.0, 1.6, 9.0), Vec3::new(-1.0, 1.0, 0.0));
            return (desde.0.lerp(OJO, u) + temblor, desde.1.lerp(MIRA, u));
        }
        (OJO + sigue + temblor, MIRA + sigue)
    }

    fn vista_proyeccion(&self) -> Mat4 {
        let (ojo, mira) = self.camara();
        Mat4::perspective(FOV.to_radians(), ASPECTO, 0.1, 60.0) * Mat4::look_at(ojo, mira, Vec3::new(0.0, 1.0, 0.0))
    }

    /// Un punto del mundo en la ventana (para los iconos de la interfaz).
    fn en_ventana(&self, p: Vec3) -> Option<(f32, f32)> {
        let c = self.vista_proyeccion().mul_vec4(Vec4::new(p.x, p.y, p.z, 1.0));
        if c.w <= 0.1 {
            return None;
        }
        let (x, y, w, h) = self.imagen;
        Some((x + (c.x / c.w + 1.0) * 0.5 * w, y + (1.0 - c.y / c.w) * 0.5 * h))
    }

    // -----------------------------------------------------------------------
    // El juego
    // -----------------------------------------------------------------------

    pub fn actualizar(&mut self, e: &Entrada, dt: f32) -> AccionUrano {
        self.t += dt;
        self.total += dt;
        self.dano = (self.dano - dt * 1.5).max(0.0);
        self.sacudida = (self.sacudida - dt * 2.0).max(0.0);
        self.jugo.actualizar(dt);
        if let Some(m) = &mut self.mensaje {
            m.3 -= dt;
            if m.3 <= 0.0 {
                self.mensaje = None;
            }
        }
        if e.aterrizar {
            return AccionUrano::Despegar;
        }
        match self.fase {
            Fase::Llegada(i) => {
                if (e.aceptar || e.clic) && self.t > 0.4 || self.mensaje.is_none() {
                    if i + 1 < LLEGADA.len() {
                        let (q, t, v) = LLEGADA[i + 1];
                        self.decir(q, t, v);
                        self.cambiar(Fase::Llegada(i + 1));
                    } else {
                        self.mensaje = None;
                        self.abrir_dia(0);
                    }
                }
            }
            Fase::Abriendo => {
                if self.t > 2.6 || ((e.aceptar || e.clic) && self.t > 0.6) {
                    self.cambiar(Fase::Jugando);
                    self.eventos.push(Efecto::Puerta);
                    let lema = DIAS[self.dia].lema;
                    self.chepa(format!("¡Se abre el baño! Hoy: {lema}. Meta: Q{}.", DIAS[self.dia].meta));
                }
                self.mover_jugador(e, dt);
            }
            Fase::Jugando => self.jugar(e, dt),
            Fase::FinDia(gano) => {
                if (e.aceptar || e.clic) && self.t > 1.2 {
                    match (gano, self.dia + 1 < DIAS.len()) {
                        (true, true) => self.abrir_dia(self.dia + 1),
                        (true, false) => {
                            self.vida.exito("¡EMPLEADO DEL MES!", "Doña Chepa casi llora", Tono::Oro);
                            self.cambiar(Fase::Final);
                        }
                        (false, _) => self.abrir_dia(self.dia),
                    }
                }
            }
            Fase::Final => {
                if (e.aceptar || e.clic) && self.t > 1.5 {
                    self.abrir_dia(0);
                }
            }
        }
        AccionUrano::Nada
    }

    fn jugar(&mut self, e: &Entrada, dt: f32) {
        self.reloj -= dt;
        self.mover_jugador(e, dt);
        self.usar(e, dt);
        // Llegan clientes (cada vez mas seguido); el domingo, la excursion de Pluton de un jalon.
        self.hasta_cliente -= dt;
        if self.hasta_cliente <= 0.0 && self.reloj > 8.0 {
            let (a, b) = DIAS[self.dia].cada;
            self.hasta_cliente = a + (b - a) * self.azar();
            self.llega_cliente();
        }
        if self.dia == 2 && !self.excursion && self.reloj < DIAS[2].dura - 40.0 {
            self.excursion = true;
            for _ in 0..4 {
                self.llega_cliente();
            }
            self.chepa("¡Ahí viene la excursión del colegio de Plutón! Cuatro de un solo. Ni planetas son y ya quieren baño.");
        }
        self.mover_aliens(dt);
        self.animar_puertas(dt);
        if self.reloj <= 0.0 {
            self.terminar_dia();
        }
    }

    fn llega_cliente(&mut self) {
        let urgido = self.azar() < DIAS[self.dia].urgidos;
        let color = (self.azar() * COLORES_ALIEN.len() as f32) as usize % COLORES_ALIEN.len();
        let nombre = NOMBRES[(self.azar() * NOMBRES.len() as f32) as usize % NOMBRES.len()];
        let lugar = self.aliens.iter().filter(|a| a.hace == Hace::Fila).count();
        // Entra desde afuera, por el carril de la fila.
        let pos = Vec3::new(FILA_X, 0.0, 7.5 + lugar as f32 * 0.4);
        self.aliens.push(Alien { pos, rumbo: PI, color, nombre, hace: Hace::Fila, camino: Vec::new(), paciencia: 1.0, urgido, resbalon: 0.0, inmune: 0.0, paso: 0.0, propina: 0 });
        if urgido && self.azar() < 0.5 {
            self.chepa(format!("Ahí viene {nombre}, y viene URGIDO. Atiéndalo rápido o me deja un recuerdo en la fila."));
        }
    }

    /// Los que estan en la fila, en orden (el primero es el de enfrente).
    fn fila(&self) -> Vec<usize> {
        let mut f: Vec<usize> = (0..self.aliens.len()).filter(|&i| self.aliens[i].hace == Hace::Fila).collect();
        f.sort_by(|&a, &b| self.aliens[a].pos.z.total_cmp(&self.aliens[b].pos.z));
        f
    }

    fn lugar_en_fila(k: usize) -> Vec3 {
        Vec3::new(FILA_X, 0.0, FILA_Z + k as f32 * PASO_FILA)
    }

    /// El primer cubiculo que se puede usar (libre, sin tapar).
    fn cubiculo_libre(&self) -> Option<usize> {
        (0..CUBICULOS).find(|&n| !self.cubiculos[n].ocupado && !self.cubiculos[n].tapado)
    }

    fn mover_aliens(&mut self, dt: f32) {
        let fila = self.fila();
        for (k, &i) in fila.iter().enumerate() {
            self.aliens[i].camino = vec![Self::lugar_en_fila(k)];
        }
        let mut se_van: Vec<usize> = Vec::new();
        for i in 0..self.aliens.len() {
            let delante = fila.first() == Some(&i);
            let mut a = self.aliens[i].clone();
            // La paciencia: en la fila (mas al frente, mas desesperado) y adentro sin papel.
            let gasto = match a.hace {
                Hace::Fila => (if a.urgido { 1.0 / 30.0 } else { 1.0 / 50.0 }) * if delante { 1.3 } else { 1.0 },
                Hace::SinPapel(_) => 1.0 / 18.0,
                _ => 0.0,
            };
            a.paciencia = (a.paciencia - gasto * dt).max(0.0);
            if a.paciencia <= 0.0 && matches!(a.hace, Hace::Fila | Hace::SinPapel(_)) {
                if let Hace::SinPapel(n) = a.hace {
                    self.cubiculos[n].ocupado = false;
                    a.camino = vec![Vec3::new(x_cubiculo(n), 0.0, Z_PUERTA), Vec3::new(SALIDA_X, 0.0, 3.4), Vec3::new(SALIDA_X, 0.0, 4.6), AFUERA];
                } else {
                    a.camino = vec![Vec3::new(SALIDA_X, 0.0, a.pos.z.max(3.2)), Vec3::new(SALIDA_X, 0.0, 4.6), AFUERA];
                }
                a.hace = Hace::Enojado;
                self.enojados += 1;
                self.reputacion = (self.reputacion - 0.8).max(0.0);
                self.dano = 1.0;
                self.vida.grito("¡SE FUE ENOJADO!", Tono::Rojo, Tam::Medio);
                self.vida.golpe(0.35, Tono::Rojo);
                self.eventos.push(Efecto::Fallo);
                let nombre = a.nombre;
                self.chepa(format!("¡Se fue {nombre}, bien enojado! Eso me cuesta una estrella en la app."));
            }
            // El resbalon: da vueltas un rato, sin avanzar.
            if a.resbalon > 0.0 {
                a.resbalon -= dt;
                self.aliens[i] = a;
                continue;
            }
            let llego = Self::caminar(&mut a, dt);
            a.inmune = (a.inmune - dt).max(0.0);
            // Pisar un charco: ¡resbalon! (solo caminando, y no dos veces seguidas).
            let avanzando = a.camino.first().is_some_and(|&d| (d - a.pos).length() > 0.05);
            if avanzando && a.inmune <= 0.0 {
                if let Some(c) = self.charcos.iter().position(|c| (c.pos - a.pos).length() < c.tam * 0.8) {
                    a.resbalon = 1.3;
                    a.inmune = 3.5;
                    self.charcos[c].tam *= 0.7;
                    self.resbalones += 1;
                    self.reputacion = (self.reputacion - 0.5).max(0.0);
                    self.sacudida = 0.6;
                    self.vida.grito("¡SE RESBALÓ!", Tono::Rojo, Tam::Grande);
                    self.eventos.push(Efecto::Choque);
                    let nombre = a.nombre;
                    self.chepa(format!("¡AY! ¡Se me resbaló {nombre}! ¡El trapeador, mijo, el trapeador!"));
                }
            }
            if llego {
                match a.hace {
                    Hace::AlCubiculo(n) => {
                        if self.cubiculos[n].papel == 0 {
                            a.hace = Hace::SinPapel(n);
                            self.vida.grito("¡NO HAY PAPEL!", Tono::Oro, Tam::Grande);
                            self.vida.golpe(0.25, Tono::Oro);
                            self.eventos.push(Efecto::Alarma);
                            self.chepa(format!("¡Gritan del {}! ¡NO HAY PAPEL! Del estante, rápido.", n + 1));
                        } else {
                            self.cubiculos[n].papel -= 1;
                            a.hace = Hace::Adentro(n, 3.5 + 3.0 * self.azar());
                        }
                    }
                    Hace::AlLavamanos(k) => a.hace = Hace::Lavando(k, 1.5),
                    Hace::Saliendo | Hace::Enojado => se_van.push(i),
                    _ => {}
                }
            }
            match a.hace {
                Hace::Adentro(n, t) => {
                    let t = t - dt;
                    if t > 0.0 {
                        a.hace = Hace::Adentro(n, t);
                    } else {
                        // Sale: libera el cubiculo (y a veces lo deja tapado).
                        self.cubiculos[n].ocupado = false;
                        if self.azar() < DIAS[self.dia].tapado {
                            self.cubiculos[n].tapado = true;
                            self.cubiculos[n].destape = 0.0;
                            self.vida.grito("¡SE TAPÓ!", Tono::Rojo, Tam::Grande);
                            self.vida.golpe(0.25, Tono::Rojo);
                            self.eventos.push(Efecto::Alarma);
                            let nombre = a.nombre;
                            self.chepa(format!("¡Se tapó el {}! ¿Qué le darían de comer a {nombre}? ¡Destapacaños!", n + 1));
                        }
                        // A veces deja un charco en la puerta.
                        if self.azar() < DIAS[self.dia].charco * 0.4 {
                            self.charcos.push(Charco { pos: Vec3::new(x_cubiculo(n) + 0.3, 0.0, Z_PUERTA + 0.35), tam: 0.55 });
                        }
                        let k = (self.azar() * 2.0) as usize % 2;
                        a.camino = vec![Vec3::new(x_cubiculo(n), 0.0, Z_PUERTA), Vec3::new(X_LAVAMANOS - 0.3, 0.0, LAVAMANOS[k] - 0.6), Vec3::new(X_LAVAMANOS, 0.0, LAVAMANOS[k])];
                        a.hace = Hace::AlLavamanos(k);
                        self.atendidos += 1;
                        self.quetzales += a.propina;
                        if a.propina > 0 {
                            self.vida.grito("¡PROPINA!", Tono::Oro, Tam::Medio);
                            self.eventos.push(Efecto::Sello);
                        }
                        self.reputacion = (self.reputacion + 0.25).min(5.0);
                    }
                }
                Hace::Lavando(k, t) => {
                    let t = t - dt;
                    if t > 0.0 {
                        a.hace = Hace::Lavando(k, t);
                    } else {
                        if self.azar() < DIAS[self.dia].charco {
                            self.charcos.push(Charco { pos: Vec3::new(X_LAVAMANOS - 0.75, 0.0, LAVAMANOS[k] - 0.15), tam: 0.6 });
                        }
                        a.camino = vec![Vec3::new(2.6, 0.0, 3.5), Vec3::new(SALIDA_X, 0.0, 3.4), Vec3::new(SALIDA_X, 0.0, 4.6), AFUERA];
                        a.hace = Hace::Saliendo;
                    }
                }
                _ => {}
            }
            self.aliens[i] = a;
        }
        // Los que ya salieron se van del todo.
        for i in se_van.into_iter().rev() {
            self.aliens.remove(i);
        }
    }

    /// Camina hacia el siguiente punto del camino. Devuelve si ya llego al ultimo.
    fn caminar(a: &mut Alien, dt: f32) -> bool {
        let Some(&destino) = a.camino.first() else { return true };
        let d = destino - a.pos;
        let largo = d.length();
        let paso = VEL_ALIEN * if a.urgido { 1.25 } else { 1.0 } * dt;
        if largo <= paso.max(0.02) {
            a.pos = destino;
            a.camino.remove(0);
            return a.camino.is_empty();
        }
        a.pos += d * (paso / largo);
        a.rumbo = f32::atan2(d.x, d.z);
        a.paso += dt * 9.0;
        false
    }

    fn animar_puertas(&mut self, dt: f32) {
        for n in 0..CUBICULOS {
            let puerta = Vec3::new(x_cubiculo(n), 0.0, Z_PUERTA - 0.4);
            // Abierta si alguien esta entrando o saliendo (cerca de la puerta) o si esta tapado.
            let pasando = self.aliens.iter().any(|a| (a.pos - puerta).length() < 1.0 && !matches!(a.hace, Hace::Adentro(..)));
            let objetivo = if pasando || self.cubiculos[n].tapado { 1.0 } else { 0.0 };
            let c = &mut self.cubiculos[n];
            c.puerta += (objetivo - c.puerta) * (1.0 - (-8.0 * dt).exp());
        }
    }

    fn mover_jugador(&mut self, e: &Entrada, dt: f32) {
        // El stick o WASD/flechas: arriba en la pantalla es -Z.
        let mut quiere = Vec3::new(e.horizontal.clamp(-1.0, 1.0), 0.0, -(e.empuje + e.cabecear).clamp(-1.0, 1.0));
        if quiere.length() > 1.0 {
            quiere = quiere.normalize();
        }
        let j = &mut self.jugador;
        // Arranca y frena suave (sin patinar): la velocidad persigue a la pedida.
        j.vel = j.vel.lerp(quiere * VEL_JUGADOR, 1.0 - (-12.0 * dt).exp());
        let mut p = j.pos + j.vel * dt;
        // Las paredes, el mostrador, el estante, los lavamanos y los cubiculos.
        p.x = p.x.clamp(-4.7, 3.95);
        p.z = p.z.clamp(-2.05, 3.7);
        let dentro = |p: Vec3, x0: f32, x1: f32, z0: f32, z1: f32| p.x > x0 && p.x < x1 && p.z > z0 && p.z < z1;
        if dentro(p, -5.2, -3.15, 0.9, 2.45) {
            // El mostrador de la caja: se queda del lado de donde venia.
            if j.pos.x >= -3.15 {
                p.x = -3.15;
            } else if j.pos.z >= 2.45 {
                p.z = 2.45;
            } else {
                p.z = 0.9;
            }
        }
        j.pos = p;
        let rapidez = j.vel.length();
        if rapidez > 0.3 {
            j.rumbo = f32::atan2(j.vel.x, j.vel.z);
            j.paso += dt * rapidez * 3.2;
        }
    }

    /// Lo que tiene enfrente el jugador.
    fn objetivo(&self) -> Option<Objetivo> {
        let j = &self.jugador;
        // Con el trapeador, el charco donde esta parado manda.
        if j.carga == Some(Item::Trapeador) {
            if let Some(i) = self.charcos.iter().position(|c| (c.pos - j.pos).length() < c.tam + 0.35) {
                return Some(Objetivo::Charco(i));
            }
        }
        let mut cerca: Vec<(Objetivo, f32)> = vec![(Objetivo::Caja, (CAJA - j.pos).length())];
        for (item, z) in ESTANTE {
            cerca.push((Objetivo::Estante(item), (Vec3::new(X_ESTANTE, 0.0, z) - j.pos).length()));
        }
        for n in 0..CUBICULOS {
            cerca.push((Objetivo::Cubiculo(n), (Vec3::new(x_cubiculo(n), 0.0, Z_PUERTA) - j.pos).length()));
        }
        cerca.into_iter().filter(|c| c.1 < ALCANCE).min_by(|a, b| a.1.total_cmp(&b.1)).map(|c| c.0)
    }

    /// Lo que pasaria si se aprieta el boton ahorita (para el letrero de ayuda).
    fn que_hace(&self) -> Option<String> {
        let j = &self.jugador;
        Some(match self.objetivo()? {
            Objetivo::Caja => match (self.fila().first(), self.cubiculo_libre()) {
                (None, _) => "nadie en la fila".into(),
                (Some(_), None) => "no hay cubículo libre".into(),
                (Some(_), Some(n)) => format!("cobrar Q1 (al {})", n + 1),
            },
            Objetivo::Estante(item) => match j.carga {
                Some(c) if c == item => format!("dejar el {}", item.nombre()),
                _ => format!("agarrar el {}", item.nombre()),
            },
            Objetivo::Cubiculo(n) => {
                let c = &self.cubiculos[n];
                match j.carga {
                    _ if c.tapado && j.carga == Some(Item::Destapacanos) => "¡destapar! (apriete varias veces)".into(),
                    _ if c.tapado => "está tapado: traiga el destapacaños".into(),
                    Some(Item::Papel) if c.papel < ROLLOS => "poner papel".into(),
                    _ if c.papel == 0 => "sin papel: traiga un rollo".into(),
                    _ => format!("cubículo {} ({} rollos)", n + 1, c.papel),
                }
            }
            Objetivo::Charco(_) => "trapear (mantener)".into(),
        })
    }

    fn usar(&mut self, e: &Entrada, dt: f32) {
        let objetivo = self.objetivo();
        // Trapear: mantener el boton sobre el charco.
        if let (Some(Objetivo::Charco(i)), true) = (objetivo, e.disparando || e.raton_apretado) {
            self.charcos[i].tam -= dt * 0.6;
            if self.total % 0.35 < dt {
                self.eventos.push(Efecto::Chorro);
            }
            if self.charcos[i].tam < 0.15 {
                let p = self.charcos[i].pos;
                self.charcos.remove(i);
                self.jugo.estallar(p + Vec3::new(0.0, 0.2, 0.0), [0.5, 0.95, 1.0], 18, 4.0);
                self.jugo.onda(p, [0.5, 0.95, 1.0], 2.0);
                self.vida.grito("¡LIMPIO!", Tono::Cian, Tam::Chico);
                self.eventos.push(Efecto::Acierto);
            }
            return;
        }
        if !(e.aceptar || e.clic) {
            return;
        }
        let Some(objetivo) = objetivo else { return };
        match objetivo {
            Objetivo::Caja => {
                let fila = self.fila();
                match (fila.first(), self.cubiculo_libre()) {
                    (Some(&i), Some(n)) => {
                        // Cobra Q1 (y la propina segun lo que espero) y lo manda a su cubiculo.
                        let a = &mut self.aliens[i];
                        a.propina = if a.paciencia > 0.7 { 2 } else if a.paciencia > 0.4 { 1 } else { 0 };
                        a.hace = Hace::AlCubiculo(n);
                        let xc = x_cubiculo(n);
                        a.camino = vec![Vec3::new(-3.0, 0.0, 2.95), Vec3::new(-3.0, 0.0, 0.4), Vec3::new(xc, 0.0, Z_PUERTA + 0.2), Vec3::new(xc, 0.0, Z_ADENTRO)];
                        self.cubiculos[n].ocupado = true;
                        self.quetzales += 1;
                        let p = self.jugador.pos + Vec3::new(0.0, 1.2, 0.0);
                        self.jugo.estallar(p, [1.0, 0.84, 0.25], 14, 3.5);
                        self.vida.grito("¡Q1!", Tono::Verde, Tam::Chico);
                        self.eventos.push(Efecto::Acierto);
                    }
                    (Some(_), None) => {
                        self.eventos.push(Efecto::Fallo);
                        self.chepa("No hay cubículo libre. ¡Destape o espere, pero no me cobre al aire!");
                    }
                    (None, _) => {}
                }
            }
            Objetivo::Estante(item) => {
                self.jugador.carga = if self.jugador.carga == Some(item) { None } else { Some(item) };
                self.eventos.push(Efecto::Carta);
            }
            Objetivo::Cubiculo(n) => {
                let carga = self.jugador.carga;
                let c = &mut self.cubiculos[n];
                if c.tapado && carga == Some(Item::Destapacanos) {
                    c.destape += 0.25;
                    self.eventos.push(Efecto::Pop);
                    self.sacudida = self.sacudida.max(0.2);
                    if c.destape >= 1.0 {
                        c.tapado = false;
                        c.destape = 0.0;
                        let p = Vec3::new(x_cubiculo(n), 0.8, Z_PUERTA);
                        self.jugo.estallar(p, [0.55, 1.0, 0.7], 30, 6.0);
                        self.jugo.onda(p, [0.55, 1.0, 0.7], 3.0);
                        self.vida.grito("¡GLUP!", Tono::Verde, Tam::Grande);
                        self.vida.golpe(0.35, Tono::Verde);
                        self.vida.confeti(20, Tono::Verde);
                        self.eventos.push(Efecto::Acierto);
                        self.chepa(format!("¡GLUP! Destapado el {}. Usted tiene futuro en esto.", n + 1));
                    }
                } else if carga == Some(Item::Papel) && c.papel < ROLLOS {
                    c.papel = ROLLOS;
                    self.jugador.carga = None;
                    self.jugo.estallar(Vec3::new(x_cubiculo(n), 1.0, Z_PUERTA), [1.0, 1.0, 1.0], 16, 3.5);
                    self.vida.grito("¡PAPEL!", Tono::Blanco, Tam::Medio);
                    self.eventos.push(Efecto::Sello);
                    // Si alguien esperaba el papel adentro, sigue en lo suyo.
                    if let Some(a) = self.aliens.iter_mut().find(|a| a.hace == Hace::SinPapel(n)) {
                        a.hace = Hace::Adentro(n, 2.5);
                        self.cubiculos[n].papel -= 1;
                    }
                } else {
                    self.eventos.push(Efecto::Fallo);
                }
            }
            Objetivo::Charco(_) => {}
        }
    }

    fn terminar_dia(&mut self) {
        let meta = DIAS[self.dia].meta;
        let gano = self.quetzales >= meta && self.reputacion >= 2.0;
        self.partidas += 1;
        self.dias_ganados += gano as u32;
        self.record = self.record.max(self.quetzales);
        if gano && self.dia + 1 == DIAS.len() {
            self.campeon = true;
        }
        self.eventos.push(if gano { Efecto::Sello } else { Efecto::Fallo });
        self.chepa(match (gano, self.quetzales >= meta) {
            (true, _) => "¡Cerramos! Ni un charco y la caja llena. Venga, le invito un atol.",
            (false, true) => "Juntamos el pisto, pero la gente se fue enojada. Las estrellas también cuentan, mijo.",
            (false, false) => "No llegamos a la meta. Mañana otra vez... bueno, hoy otra vez.",
        });
        self.aliens.clear();
        if gano {
            self.vida.exito("¡DÍA SUPERADO!", "Ni un charco y la caja llena", Tono::Oro);
        } else {
            self.vida.desastre("¡CERRADO!", "Doña Chepa cuenta el pisto");
        }
        self.cambiar(Fase::FinDia(gano));
    }

    // -----------------------------------------------------------------------
    // Lo que se dibuja en 3D (lo arma `main.rs`)
    // -----------------------------------------------------------------------

    /// Cada cliente: su matriz de modelo, su color y su cara.
    pub fn aliens_3d(&self) -> Vec<(Mat4, usize, &'static str)> {
        self.aliens
            .iter()
            .filter(|a| !matches!(a.hace, Hace::Adentro(..) | Hace::SinPapel(_)))
            .map(|a| {
                let brinco = (a.paso).sin().abs() * 0.06;
                let resbalon = if a.resbalon > 0.0 {
                    // Patas arriba y dando vueltas.
                    Mat4::translation(Vec3::new(0.0, 0.25, 0.0)) * Mat4::rotation_y(a.resbalon * 9.0) * Mat4::rotation_z(PI * 0.5 * (a.resbalon / 1.3).min(1.0))
                } else {
                    Mat4::identity()
                };
                let cara = match a.hace {
                    Hace::Enojado => "enojado",
                    _ if a.resbalon > 0.0 || a.paciencia < 0.4 || a.urgido => "urgido",
                    _ => "feliz",
                };
                (Mat4::translation(a.pos + Vec3::new(0.0, brinco, 0.0)) * Mat4::rotation_y(a.rumbo) * resbalon, a.color, cara)
            })
            .collect()
    }

    /// La matriz de la puerta del cubiculo `n` (gira en su bisagra, hacia afuera).
    pub fn puerta(&self, n: usize, bisagra: Vec3) -> Mat4 {
        let angulo = -1.6 * self.cubiculos[n].puerta;
        Mat4::translation(bisagra) * Mat4::rotation_y(angulo) * Mat4::translation(-bisagra)
    }

    /// Donde va lo que carga el jugador (en la mano, enfrente).
    pub fn carga_3d(&self) -> Option<(Item, Mat4)> {
        let j = &self.jugador;
        let item = j.carga?;
        let frente = Vec3::new(j.rumbo.sin(), 0.0, j.rumbo.cos());
        let trapeando = matches!(self.objetivo(), Some(Objetivo::Charco(_)));
        let vaiven = if trapeando { (self.total * 14.0).sin() * 0.35 } else { 0.0 };
        let pos = j.pos + frente * 0.42 + Vec3::new(0.0, if item == Item::Papel { 0.55 } else { 0.05 }, 0.0);
        Some((item, Mat4::translation(pos) * Mat4::rotation_y(j.rumbo + vaiven) * Mat4::rotation_x(if item == Item::Papel { 0.0 } else { -0.25 })))
    }

    // -----------------------------------------------------------------------
    // Para las capturas
    // -----------------------------------------------------------------------

    /// `urano`: la llegada. `urano_juego`: a media manana del viernes, con fila, un tapado, un
    /// charco y alguien sin papel. `urano_final`: los tres dias pasados.
    pub fn preparar_captura(&mut self, modo: &str) {
        if modo == "urano" {
            return;
        }
        self.mensaje = None;
        match modo {
            "urano_final" => {
                self.abrir_dia(2);
                self.campeon = true;
                self.record = 21;
                self.quetzales = 21;
                self.reputacion = 4.5;
                self.cambiar(Fase::Final);
            }
            _ => {
                self.abrir_dia(1);
                self.cambiar(Fase::Jugando);
                for _ in 0..5 {
                    self.llega_cliente();
                }
                for _ in 0..240 {
                    self.mover_aliens(1.0 / 60.0);
                }
                // Uno ya adentro sin papel, uno tapado, un charco en el lavamanos.
                self.cubiculos[0].papel = 0;
                let i = self.fila()[0];
                self.aliens[i].hace = Hace::SinPapel(0);
                self.aliens[i].pos = Vec3::new(x_cubiculo(0), 0.0, Z_ADENTRO);
                self.cubiculos[0].ocupado = true;
                self.cubiculos[2].tapado = true;
                self.cubiculos[2].destape = 0.5;
                self.charcos.push(Charco { pos: Vec3::new(X_LAVAMANOS - 0.75, 0.0, LAVAMANOS[0] - 0.15), tam: 0.6 });
                self.jugador.pos = Vec3::new(1.2, 0.0, -0.4);
                self.jugador.rumbo = PI * 0.75;
                self.jugador.carga = Some(Item::Destapacanos);
                self.quetzales = 7;
                self.reputacion = 3.0;
                self.reloj = 61.0;
                self.mover_aliens(0.0);
            }
        }
    }

    // -----------------------------------------------------------------------
    // La interfaz
    // -----------------------------------------------------------------------

    /// `texto` es lo que ya se ve de lo que se esta diciendo (aparece letra por letra).
    pub fn dibujar_ui(&self, d: &mut RaylibDrawHandle, e: &Entrada, texto: &str, imagen: Rectangle, ancho: i32, alto: i32) {
        let _ = imagen;
        let acciones: Vec<(&str, &str)> = match self.fase {
            Fase::Jugando | Fase::Abriendo => vec![(if e.hay_control { "stick" } else { "W / A / S / D" }, "Moverse"), (e.nombre("aceptar"), "Usar / agarrar"), (e.nombre("aterrizar"), "Despegar")],
            Fase::FinDia(true) => vec![(e.nombre("aceptar"), if self.dia + 1 < DIAS.len() { "Siguiente día" } else { "Ver el final" }), (e.nombre("aterrizar"), "Despegar")],
            Fase::FinDia(false) => vec![(e.nombre("aceptar"), "Repetir el día"), (e.nombre("aterrizar"), "Despegar")],
            Fase::Final => vec![(e.nombre("aceptar"), "Otra vez desde el lunes"), (e.nombre("aterrizar"), "Despegar")],
            Fase::Llegada(_) => vec![(e.nombre("aceptar"), "Seguir"), (e.nombre("aterrizar"), "Despegar")],
        };
        let pie = ui::alto_ayudas(&acciones, ancho);
        let angosta = ancho < 900;
        if matches!(self.fase, Fase::Jugando | Fase::Abriendo) {
            self.dibujar_iconos(d, e);
            self.dibujar_marcador(d, ancho, angosta);
        }
        match self.fase {
            Fase::Abriendo => {
                let dia = &DIAS[self.dia];
                let tam = if angosta { 30 } else { 46 };
                let y = (alto as f32 * 0.36) as i32;
                ui::titulo_de(d, dia.nombre, (ancho - ui::medir_titulo(dia.nombre, tam)) / 2, y, tam, Color::new(110, 235, 225, 255));
                let lema = format!("{} · meta Q{}", dia.lema, dia.meta);
                ui::subtitulo(d, &lema, (ancho - ui::medir(&lema, 22)) / 2, y + tam + 14, 22, ui::TEXTO);
            }
            Fase::FinDia(gano) => self.dibujar_fin_dia(d, gano, ancho, alto, pie),
            Fase::Final => {
                let tam = if angosta { 24 } else { 36 };
                let titulo = "EL BAÑO MÁS LIMPIO";
                let y = (alto as f32 * 0.25) as i32;
                ui::titulo_de(d, titulo, (ancho - ui::medir_titulo(titulo, tam)) / 2, y, tam, Color::new(255, 220, 120, 255));
                let sub = "del sistema solar · empleado del mes de Doña Chepa";
                ui::subtitulo(d, sub, (ancho - ui::medir(sub, 20)) / 2, y + tam + 12, 20, ui::TEXTO);
                ui::insignia(d, ui::Insignia::AlasRoja, ancho as f32 / 2.0, y as f32 + tam as f32 + 110.0, 3.0, Color::WHITE);
            }
            _ => {}
        }
        // Lo que dice Dona Chepa (o GLADAS al llegar), abajo, encima de las ayudas.
        if let Some((quien, completo, _)) = self.hablante() {
            if !texto.is_empty() {
                let w = (ancho - 80).min(900);
                let h = ui::alto_dialogo(quien, &completo, w);
                ui::caja_dialogo(d, quien, ui::LILA, texto, (ancho - w) / 2, alto - pie - h - 14, w);
            }
        }
        ui::ayudas(d, &acciones, ancho, alto);
    }

    /// Arriba: el dia y su reloj, la caja contra la meta y las estrellas.
    fn dibujar_marcador(&self, d: &mut RaylibDrawHandle, ancho: i32, angosta: bool) {
        let dia = &DIAS[self.dia];
        let tam = if angosta { 14 } else { 18 };
        ui::velo(d, 0, 0, ancho, 96, ui::Hacia::Abajo);
        ui::titulo_de(d, dia.nombre, 24, 16, tam + 4, Color::WHITE);
        let llenura = (self.reloj / dia.dura).clamp(0.0, 1.0);
        ui::barra(d, 24.0, 48.0, 220.0, 16.0, llenura, if llenura < 0.2 { ui::ColorBarra::Rojo } else { ui::ColorBarra::Azul });
        // La caja, a la derecha del reloj del dia.
        let caja = format!("Q{} / meta Q{}", self.quetzales, dia.meta);
        let color = if self.quetzales >= dia.meta { ui::VERDE } else { Color::new(255, 230, 150, 255) };
        ui::subtitulo(d, &caja, 258, 44, tam + 4, color);
        let estrellas = self.reputacion.round() as u32;
        ui::estrellas(d, ancho as f32 - 24.0 - 5.0 * 30.0, 22.0, estrellas, 5, 24.0);
        let quejas = format!("atendidos {}  ·  enojados {}  ·  resbalones {}", self.atendidos, self.enojados, self.resbalones);
        ui::subtitulo(d, &quejas, ancho - 24 - ui::medir(&quejas, 13), 56, 13, ui::TENUE);
    }

    /// Encima del bano: la paciencia de la fila, lo que pasa en cada cubiculo, lo que se carga y
    /// lo que se puede hacer.
    fn dibujar_iconos(&self, d: &mut RaylibDrawHandle, e: &Entrada) {
        // La paciencia de los de la fila (y quien esta urgido).
        for a in &self.aliens {
            if a.hace != Hace::Fila {
                continue;
            }
            if let Some((x, y)) = self.en_ventana(a.pos + Vec3::new(0.0, 1.35, 0.0)) {
                let color = if a.paciencia > 0.5 { ui::ColorBarra::Verde } else if a.paciencia > 0.25 { ui::ColorBarra::Amarillo } else { ui::ColorBarra::Rojo };
                ui::barra(d, x - 22.0, y, 44.0, 8.0, a.paciencia, color);
                if a.urgido && (self.total * 4.0).sin() > 0.0 {
                    ui::subtitulo(d, "!", x as i32 + 26, y as i32 - 8, 18, ui::ROJO);
                }
            }
        }
        // Los cubiculos: tapado (con lo que falta), sin papel, cuantos rollos.
        for n in 0..CUBICULOS {
            let c = &self.cubiculos[n];
            let Some((x, y)) = self.en_ventana(Vec3::new(x_cubiculo(n), 2.7, Z_PUERTA - 0.2)) else { continue };
            let sin_papel = self.aliens.iter().any(|a| a.hace == Hace::SinPapel(n));
            let (texto, color) = if c.tapado {
                ("¡TAPADO!", Color::new(255, 110, 120, 255))
            } else if sin_papel {
                ("¡PAPEL!", Color::new(255, 230, 120, 255))
            } else if c.ocupado {
                ("ocupado", ui::TENUE)
            } else {
                ("libre", ui::VERDE)
            };
            let parpadea = (c.tapado || sin_papel) && (self.total * 5.0).sin() < -0.2;
            if !parpadea {
                ui::subtitulo(d, texto, x as i32 - ui::medir(texto, 15) / 2, y as i32 - 18, 15, color);
            }
            if c.tapado {
                ui::barra(d, x - 24.0, y + 2.0, 48.0, 7.0, c.destape, ui::ColorBarra::Azul);
            } else {
                // Los rollos que le quedan (puntitos).
                for k in 0..ROLLOS {
                    let lleno = k < c.papel;
                    d.draw_circle((x - 10.0 + k as f32 * 10.0) as i32, y as i32 + 6, 3.0, if lleno { Color::new(250, 248, 240, 255) } else { Color::new(80, 70, 110, 200) });
                }
            }
        }
        // El estante: un letrero sobre cada cosa, para saber donde esta todo. El que hace falta
        // ahorita brilla y brinca (papel si alguien grita, destapacanos si hay un tapado, trapeador
        // si hay charcos).
        for (item, z) in ESTANTE {
            let Some((x, y)) = self.en_ventana(Vec3::new(X_ESTANTE, 2.05, z)) else { continue };
            let hace_falta = match item {
                Item::Papel => self.aliens.iter().any(|a| matches!(a.hace, Hace::SinPapel(_))),
                Item::Destapacanos => self.cubiculos.iter().any(|c| c.tapado),
                Item::Trapeador => !self.charcos.is_empty(),
            };
            let lo_lleva = self.jugador.carga == Some(item);
            let nombre = match item {
                Item::Papel => "PAPEL",
                Item::Destapacanos => "DESTAPACAÑOS",
                Item::Trapeador => "TRAPEADOR",
            };
            let tam = 15;
            let brinco = if hace_falta && !lo_lleva { ((self.total * 6.0).sin().abs() * 6.0) as i32 } else { 0 };
            let w = ui::medir(nombre, tam) + 18;
            let (bx, by) = (x as i32 - w / 2, y as i32 - 30 - brinco);
            let (fondo, letra) = if lo_lleva {
                (Color::new(40, 30, 70, 150), ui::TENUE)
            } else if hace_falta {
                (Color::new(255, 200, 70, 235), Color::new(40, 20, 10, 255))
            } else {
                (Color::new(30, 20, 60, 215), Color::WHITE)
            };
            d.draw_rectangle(bx, by, w, tam + 10, fondo);
            d.draw_rectangle_lines(bx, by, w, tam + 10, Color::new(255, 255, 255, 120));
            ui::subtitulo(d, nombre, bx + 9, by + 5, tam, letra);
            // La flechita que apunta a la repisa.
            let (fx, fy) = (x, (by + tam + 10) as f32);
            d.draw_triangle(Vector2::new(fx - 7.0, fy), Vector2::new(fx, fy + 9.0), Vector2::new(fx + 7.0, fy), fondo);
        }
        // Lo que lleva el jugador y lo que pasaria con el boton.
        let j = &self.jugador;
        if let Some((x, y)) = self.en_ventana(j.pos + Vec3::new(0.0, 1.7, 0.0)) {
            if let Some(que) = self.que_hace() {
                let s = format!("{}: {que}", ui::boton(e.nombre("aceptar")));
                ui::subtitulo(d, &s, x as i32 - ui::medir(&s, 15) / 2, y as i32 - 26, 15, Color::new(255, 230, 245, 255));
            }
            if let Some(item) = j.carga {
                let s = format!("con el {}", item.nombre());
                ui::subtitulo(d, &s, x as i32 - ui::medir(&s, 13) / 2, y as i32 - 6, 13, ui::TENUE);
            }
        }
    }

    fn dibujar_fin_dia(&self, d: &mut RaylibDrawHandle, gano: bool, ancho: i32, alto: i32, pie: i32) {
        let dia = &DIAS[self.dia];
        let (w, h) = ((ancho - 80).min(620), 230);
        let (x, y) = ((ancho - w) / 2, (alto - pie - h) / 2);
        ui::panel(d, x, y, w, h);
        let titulo = if gano { "¡DÍA SUPERADO!" } else { "SE CERRÓ EL BAÑO" };
        let tam = if ancho < 900 { 24 } else { 32 };
        ui::titulo_de(d, titulo, x + 28, y + 24, tam, if gano { ui::VERDE } else { ui::ROJO });
        ui::texto(d, dia.nombre, x + 28, y + 24 + tam + 10, 16, ui::TENUE);
        let filas = [
            format!("Caja: Q{} (meta Q{})", self.quetzales, dia.meta),
            format!("Clientes atendidos: {}   ·   se fueron enojados: {}", self.atendidos, self.enojados),
            format!("Resbalones: {}", self.resbalones),
        ];
        for (k, f) in filas.iter().enumerate() {
            ui::texto(d, f, x + 28, y + 90 + k as i32 * 28, 18, ui::TEXTO);
        }
        ui::estrellas(d, (x + w - 28 - 5 * 28) as f32, (y + 30) as f32, self.reputacion.round() as u32, 5, 22.0);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Un empleado automatico (para las pruebas): atiende lo urgente primero (tapados, papel,
    /// charcos) y si no hay nada, cobra. Camina con el "stick" como cualquiera.
    fn empleado(u: &Urano) -> Entrada {
        let j = &u.jugador;
        let (meta, item): (Vec3, Option<Item>) = if let Some(n) = (0..CUBICULOS).find(|&n| u.cubiculos[n].tapado) {
            (Vec3::new(x_cubiculo(n), 0.0, Z_PUERTA), Some(Item::Destapacanos))
        } else if let Some(n) = (0..CUBICULOS).find(|&n| u.cubiculos[n].papel == 0) {
            (Vec3::new(x_cubiculo(n), 0.0, Z_PUERTA), Some(Item::Papel))
        } else if let Some(c) = u.charcos.first() {
            (c.pos, Some(Item::Trapeador))
        } else {
            (CAJA, None)
        };
        // Primero, ir por la herramienta si no la trae.
        let (destino, va_por) = match item {
            Some(it) if j.carga != Some(it) => (Vec3::new(X_ESTANTE, 0.0, ESTANTE.iter().find(|e| e.0 == it).unwrap().1), true),
            _ => (meta, false),
        };
        let d = destino - j.pos;
        let mut e = Entrada::default();
        if d.length() > 0.4 {
            let dir = d.normalize();
            e.horizontal = dir.x;
            e.empuje = -dir.z;
        } else {
            e.aceptar = (u.total * 10.0) as i32 % 2 == 0 && !va_por || (va_por && (u.total * 3.0) as i32 % 3 == 0);
            e.disparando = true;
            if va_por {
                e.aceptar = j.carga != item && (u.total * 3.0) as i32 % 3 == 0;
            }
        }
        e
    }

    fn arrancar() -> Urano {
        let mut u = Urano::new(7);
        u.abrir_dia(0);
        u.cambiar(Fase::Jugando);
        u
    }

    #[test]
    fn cobrar_manda_al_cliente_a_un_cubiculo_y_suma_un_quetzal() {
        let mut u = arrancar();
        u.llega_cliente();
        for _ in 0..600 {
            u.mover_aliens(1.0 / 60.0);
        }
        u.jugador.pos = CAJA;
        u.actualizar(&Entrada { aceptar: true, ..Default::default() }, 1.0 / 60.0);
        assert_eq!(u.quetzales, 1);
        assert!(matches!(u.aliens[0].hace, Hace::AlCubiculo(0)));
        assert!(u.cubiculos[0].ocupado);
    }

    #[test]
    fn sin_papel_grita_y_con_papel_sigue() {
        let mut u = arrancar();
        u.cubiculos[0].papel = 0;
        u.llega_cliente();
        for _ in 0..600 {
            u.mover_aliens(1.0 / 60.0);
        }
        u.jugador.pos = CAJA;
        u.actualizar(&Entrada { aceptar: true, ..Default::default() }, 1.0 / 60.0);
        for _ in 0..600 {
            u.mover_aliens(1.0 / 60.0);
        }
        assert_eq!(u.aliens[0].hace, Hace::SinPapel(0));
        u.jugador.carga = Some(Item::Papel);
        u.jugador.pos = Vec3::new(x_cubiculo(0), 0.0, Z_PUERTA);
        u.actualizar(&Entrada { aceptar: true, ..Default::default() }, 1.0 / 60.0);
        assert!(matches!(u.aliens[0].hace, Hace::Adentro(0, _)));
        assert_eq!(u.cubiculos[0].papel, ROLLOS - 1);
        assert_eq!(u.jugador.carga, None);
    }

    #[test]
    fn el_destapacanos_destapa_en_cuatro_apretones() {
        let mut u = arrancar();
        u.cubiculos[1].tapado = true;
        u.jugador.carga = Some(Item::Destapacanos);
        u.jugador.pos = Vec3::new(x_cubiculo(1), 0.0, Z_PUERTA);
        for k in 0..4 {
            assert!(u.cubiculos[1].tapado, "se destapo antes de tiempo ({k})");
            u.actualizar(&Entrada { aceptar: true, ..Default::default() }, 1.0 / 60.0);
        }
        assert!(!u.cubiculos[1].tapado);
    }

    #[test]
    fn el_charco_hace_resbalar_y_el_trapeador_lo_seca() {
        let mut u = arrancar();
        u.charcos.push(Charco { pos: Vec3::new(FILA_X, 0.0, 4.0), tam: 0.6 });
        u.llega_cliente();
        for _ in 0..600 {
            u.mover_aliens(1.0 / 60.0);
        }
        assert_eq!(u.resbalones, 1);
        let c = u.charcos[0].pos;
        u.jugador.pos = c;
        u.jugador.carga = Some(Item::Trapeador);
        for _ in 0..240 {
            u.actualizar(&Entrada { disparando: true, ..Default::default() }, 1.0 / 60.0);
        }
        assert!(u.charcos.is_empty());
    }

    /// Un dia bien atendido se pasa; sin hacer nada, la gente se va enojada y no.
    #[test]
    fn atendiendo_bien_se_pasa_el_lunes_y_sin_hacer_nada_no() {
        let mut u = arrancar();
        while matches!(u.fase, Fase::Jugando) {
            let e = empleado(&u);
            u.actualizar(&e, 1.0 / 30.0);
        }
        assert_eq!(u.fase, Fase::FinDia(true), "caja {} meta {}, estrellas {}", u.quetzales, DIAS[0].meta, u.reputacion);

        let mut v = arrancar();
        while matches!(v.fase, Fase::Jugando) {
            v.actualizar(&Entrada::default(), 1.0 / 30.0);
        }
        assert_eq!(v.fase, Fase::FinDia(false));
        assert!(v.enojados > 0);
    }
}
