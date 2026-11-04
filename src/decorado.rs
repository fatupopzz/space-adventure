//! Lo que rodea a los minijuegos: fondos de bloques para que ninguna escena flote en el vacio.
//! - Vegetta: nubes e islas flotantes alrededor del servidor.
//! - Urano: las bandas de nubes del planeta, su anillo y lo que se le ha volado a Dona Chepa.
//! - La Luna: el estudio de verdad (telon con foquitos, rieles y reflectores) y sus crateres.

use crate::matematica::Vec3;
use crate::obj::Modelo;
use crate::obra::Obra;

pub struct Decorado {
    pub nubes: [Modelo; 2],
    pub islas: [Modelo; 3],
    pub bandas_urano: Modelo,
    pub anillo_urano: Modelo,
    pub rollo: Modelo,
    pub patito: Modelo,
    pub hielo: Modelo,
    pub estudio: Modelo,
    pub crateres: Modelo,
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn paleta(o: &mut Obra) {
    const COLORES: [(&str, [f32; 3], f32); 21] = [
        ("nube", [0.96, 0.96, 0.98], 0.06),
        ("nube_sombra", [0.76, 0.80, 0.92], 0.04),
        ("pasto", [0.30, 0.72, 0.26], 0.0),
        ("pasto_b", [0.24, 0.60, 0.25], 0.0),
        ("tierra", [0.52, 0.34, 0.20], 0.0),
        ("piedra", [0.48, 0.48, 0.54], 0.0),
        ("tronco", [0.38, 0.25, 0.14], 0.0),
        ("hojas", [0.14, 0.50, 0.20], 0.0),
        ("flor", [1.0, 0.36, 0.66], 0.12),
        ("cian_a", [0.17, 0.50, 0.58], 0.3),
        ("cian_b", [0.13, 0.42, 0.54], 0.3),
        ("cian_c", [0.22, 0.57, 0.60], 0.3),
        ("cian_d", [0.10, 0.35, 0.50], 0.3),
        ("anillo_a", [0.52, 0.68, 0.84], 0.25),
        ("anillo_b", [0.30, 0.40, 0.72], 0.25),
        ("blanco", [0.97, 0.97, 0.94], 0.1),
        ("amarillo", [1.0, 0.82, 0.12], 0.12),
        ("naranja", [1.0, 0.48, 0.08], 0.1),
        ("hielo", [0.74, 0.96, 1.0], 0.3),
        ("telon_a", [0.05, 0.05, 0.13], 0.0),
        ("telon_b", [0.08, 0.07, 0.18], 0.0),
    ];
    for (nombre, kd, brillo) in COLORES {
        o.material(nombre, kd, kd.map(|c| c * brillo), 0.0, 1.0);
    }
    o.material("metal", [0.44, 0.44, 0.50], [0.0; 3], 0.7, 50.0);
    o.material("foco", [1.0, 0.94, 0.72], [1.0, 0.88, 0.56], 0.0, 1.0);
    o.material("foquito", [1.0, 0.96, 0.80], [0.85, 0.78, 0.55], 0.0, 1.0);
    o.material("regolito", [0.36, 0.36, 0.38], [0.0; 3], 0.0, 1.0);
    o.material("regolito_b", [0.20, 0.20, 0.23], [0.0; 3], 0.0, 1.0);
}

fn nube(partes: &[(f32, f32, f32, f32, f32, f32)]) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    for (k, &(x, y, z, a, h, l)) in partes.iter().enumerate() {
        o.caja(if k % 3 == 2 { "nube_sombra" } else { "nube" }, v(x - a, y - h, z - l), v(x + a, y + h, z + l));
    }
    o.modelo()
}

/// Una isla flotante: pasto, tierra y una punta de piedra hacia abajo, con arbol o flores.
fn isla(radio: f32, con_arbol: bool) -> Modelo {
    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("pasto", v(-radio, -0.8, -radio * 0.8), v(radio, 0.0, radio * 0.8));
    o.caja("pasto_b", v(-radio * 0.72, -0.8, -radio), v(radio * 0.72, 0.0, radio));
    for k in 1..5 {
        let e = 1.0 - k as f32 * 0.2;
        let (y0, y1) = (-0.8 - k as f32 * 1.5, -0.8 - (k - 1) as f32 * 1.5);
        o.caja(if k < 3 { "tierra" } else { "piedra" }, v(-radio * e, y0, -radio * e * 0.8), v(radio * e, y1, radio * e * 0.8));
    }
    if con_arbol {
        o.caja("tronco", v(-0.5, 0.0, -0.5), v(0.5, 4.0, 0.5));
        o.caja("hojas", v(-2.6, 3.4, -2.6), v(2.6, 6.0, 2.6));
        o.caja("hojas", v(-1.5, 6.0, -1.5), v(1.5, 7.2, 1.5));
    } else {
        for (x, z) in [(-2.2, 1.0), (1.4, -1.6), (2.4, 1.8), (-1.0, -2.2)] {
            o.caja("flor", v(x - 0.3, 0.0, z - 0.3), v(x + 0.3, 0.6, z + 0.3));
        }
    }
    o.modelo()
}

pub fn armar() -> Decorado {
    let nubes = [
        nube(&[(0.0, 0.0, 0.0, 12.0, 2.0, 7.0), (-7.0, 1.2, 1.0, 6.0, 1.8, 4.0), (6.0, -0.8, -1.0, 8.0, 1.6, 5.0), (1.0, 3.0, 0.0, 5.0, 1.6, 4.0)]),
        nube(&[(0.0, 0.0, 0.0, 8.0, 1.5, 5.0), (5.0, 1.0, 0.0, 4.0, 1.5, 3.0), (-4.0, -0.5, 1.0, 5.0, 1.2, 4.0)]),
    ];
    let islas = [isla(6.0, true), isla(4.2, false), isla(8.0, true)];

    // Urano visto desde arriba: franjas de nubes de varios cianes, que no se acaban.
    let mut o = Obra::new();
    paleta(&mut o);
    const CIANES: [&str; 4] = ["cian_a", "cian_b", "cian_c", "cian_d"];
    for k in 0..16 {
        let (z0, ancho) = (-120.0 + k as f32 * 16.0, 16.0);
        let mat = CIANES[(k * 3 + k / 3) % 4];
        o.quad(mat, [v(-260.0, 0.0, z0), v(260.0, 0.0, z0), v(260.0, 0.0, z0 + ancho), v(-260.0, 0.0, z0 + ancho)], v(0.0, 1.0, 0.0));
        // Remolinos de bloques, para que la franja no se vea lisa.
        for j in 0..7 {
            let x = -200.0 + ((j * 53 + k * 29) % 400) as f32;
            let otro = CIANES[(k * 3 + k / 3 + 1 + j) % 4];
            o.caja(otro, v(x, 0.0, z0 + 3.0), v(x + 26.0, 1.0, z0 + 9.0));
        }
    }
    let bandas_urano = o.modelo();

    let mut o = Obra::new();
    paleta(&mut o);
    for k in 0..56 {
        let a = k as f32 / 56.0 * std::f32::consts::TAU;
        let r = 30.0 + ((k * 7) % 3) as f32 * 0.8;
        let (x, z) = (a.cos() * r, a.sin() * r);
        o.caja(if k % 2 == 0 { "anillo_a" } else { "anillo_b" }, v(x - 1.8, -0.2, z - 1.8), v(x + 1.8, 0.2, z + 1.8));
    }
    let anillo_urano = o.modelo();

    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("blanco", v(-0.3, -0.42, -0.2), v(0.3, 0.42, 0.2));
    o.caja("blanco", v(-0.2, -0.42, -0.3), v(0.2, 0.42, 0.3));
    o.caja("tronco", v(-0.08, 0.42, -0.08), v(0.08, 0.44, 0.08));
    let rollo = o.modelo();

    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("amarillo", v(-0.32, -0.25, -0.45), v(0.32, 0.2, 0.45));
    o.caja("amarillo", v(-0.2, 0.2, -0.5), v(0.2, 0.58, -0.12));
    o.caja("naranja", v(-0.12, 0.3, -0.68), v(0.12, 0.42, -0.5));
    let patito = o.modelo();

    let mut o = Obra::new();
    paleta(&mut o);
    o.caja("hielo", v(-0.6, -0.6, -0.6), v(0.6, 0.6, 0.6));
    let hielo = o.modelo();

    // El estudio: telon con pliegues y foquitos de "estrellas", y la armazon de luces de arriba.
    let mut o = Obra::new();
    paleta(&mut o);
    let (fondo, ancho) = (205.0, 300.0);
    for k in 0..40 {
        let x = -ancho + k as f32 * ancho * 2.0 / 40.0;
        let z = fondo + if k % 2 == 0 { 0.0 } else { 4.0 };
        o.caja(if k % 2 == 0 { "telon_a" } else { "telon_b" }, v(x, -2.0, z), v(x + ancho * 2.0 / 40.0, 190.0, z + 4.0));
    }
    for k in 0..46 {
        // Las "estrellas" son foquitos: de cerca se nota, pero nadie se acerca.
        let x = -230.0 + ((k * 137) % 460) as f32;
        let y = 40.0 + ((k * 83) % 130) as f32;
        o.caja("foquito", v(x - 0.5, y - 0.5, fondo - 0.6), v(x + 0.5, y + 0.5, fondo));
    }
    for (z, y) in [(16.0, 9.0), (30.0, 13.0)] {
        o.caja("metal", v(-36.0, y, z - 0.25), v(36.0, y + 0.5, z + 0.25));
        o.caja("metal", v(-36.0, y + 1.4, z - 0.25), v(36.0, y + 1.8, z + 0.25));
        for k in 0..18 {
            let x = -34.0 + k as f32 * 4.0;
            o.caja("metal", v(x - 0.1, y, z - 0.1), v(x + 0.1, y + 1.5, z + 0.1));
        }
    }
    for x in [-22.0, 22.0] {
        o.caja("metal", v(x - 0.3, -2.0, 29.7), v(x + 0.3, 14.0, 30.3));
    }
    let estudio = o.modelo();

    // Crateres y piedras, para romper el piso liso.
    let mut o = Obra::new();
    paleta(&mut o);
    for (x, z, r) in [(-7.5, 3.0, 2.0), (6.5, 7.0, 2.8), (-3.5, 12.0, 3.2), (11.0, 1.0, 1.7), (-12.0, 9.0, 2.5)] {
        o.caja("regolito", v(x - r, 0.0, z - r * 0.7), v(x + r, 0.28, z + r * 0.7));
        o.caja("regolito", v(x - r * 0.7, 0.0, z - r), v(x + r * 0.7, 0.28, z + r));
        o.caja("regolito_b", v(x - r * 0.68, 0.0, z - r * 0.5), v(x + r * 0.68, 0.3, z + r * 0.5));
        o.caja("regolito_b", v(x - r * 0.5, 0.0, z - r * 0.68), v(x + r * 0.5, 0.3, z + r * 0.68));
    }
    for (x, z, r) in [(-5.0, 6.0, 0.4), (4.0, 3.2, 0.3), (8.5, 11.0, 0.6), (-9.0, 1.0, 0.35), (2.0, 14.0, 0.5)] {
        o.caja("regolito", v(x - r, 0.0, z - r), v(x + r, r * 1.3, z + r));
    }
    let crateres = o.modelo();

    Decorado { nubes, islas, bandas_urano, anillo_urano, rollo, patito, hielo, estudio, crateres }
}
