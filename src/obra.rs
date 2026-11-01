//! La OBRA: un ayudante para armar modelos en el juego (sin `.obj`), con cajas y cuadrilateros.
//! Lo usan las pistas de parkour de Planeta Vegetta, que se arman distintas en cada visita. Cada material es una parte; con textura, cada cara
//! lleva sus coordenadas UV.

use crate::matematica::Vec3;
use crate::obj::{Material, Modelo, Parte, Textura, Vertice};
use std::sync::Arc;

pub struct Obra {
    partes: Vec<Parte>,
    materiales: Vec<Material>,
}

impl Obra {
    pub fn new() -> Self {
        Obra { partes: Vec::new(), materiales: Vec::new() }
    }

    /// Registra un material (y su parte, con el mismo nombre).
    pub fn material(&mut self, nombre: &str, kd: [f32; 3], ke: [f32; 3], ks: f32, ns: f32) {
        self.material_con_textura(nombre, kd, ke, ks, ns, None);
    }

    pub fn material_con_textura(&mut self, nombre: &str, kd: [f32; 3], ke: [f32; 3], ks: f32, ns: f32, textura: Option<Arc<Textura>>) {
        self.materiales.push(Material {
            nombre: nombre.to_string(),
            kd: Vec3::new(kd[0], kd[1], kd[2]),
            ks: Vec3::new(ks, ks, ks),
            ns,
            ke: Vec3::new(ke[0], ke[1], ke[2]),
            d: 1.0,
            textura,
        });
        self.partes.push(Parte { nombre: nombre.to_string(), material: self.materiales.len() - 1, triangulos: Vec::new(), comentario: String::new() });
    }

    /// Hace transparente un material ya registrado (opacidad `d` de 0 a 1).
    pub fn transparente(&mut self, nombre: &str, d: f32) {
        if let Some(m) = self.materiales.iter_mut().find(|m| m.nombre == nombre) {
            m.d = d;
        }
    }

    fn parte(&mut self, nombre: &str) -> &mut Parte {
        let i = self.partes.iter().position(|p| p.nombre == nombre).unwrap_or_else(|| panic!("material {nombre} sin registrar"));
        &mut self.partes[i]
    }

    /// Un cuadrilatero que da la cara hacia `normal` (los puntos en orden alrededor).
    pub fn quad(&mut self, mat: &str, p: [Vec3; 4], normal: Vec3) {
        self.quad_uv(mat, p, [[0.0; 2]; 4], normal);
    }

    /// Como `quad`, con la coordenada de textura de cada esquina.
    pub fn quad_uv(&mut self, mat: &str, p: [Vec3; 4], uv: [[f32; 2]; 4], normal: Vec3) {
        let n = normal.normalize();
        let v = |k: usize| Vertice { pos: p[k], normal: n, uv: uv[k] };
        let horario = (p[1] - p[0]).cross(p[2] - p[0]).dot(n) < 0.0;
        let parte = self.parte(mat);
        if horario {
            parte.triangulos.push([v(0), v(2), v(1)]);
            parte.triangulos.push([v(0), v(3), v(2)]);
        } else {
            parte.triangulos.push([v(0), v(1), v(2)]);
            parte.triangulos.push([v(0), v(2), v(3)]);
        }
    }

    /// Una caja alineada a los ejes, con sus seis caras hacia afuera.
    pub fn caja(&mut self, mat: &str, a: Vec3, b: Vec3) {
        let (x0, y0, z0, x1, y1, z1) = (a.x.min(b.x), a.y.min(b.y), a.z.min(b.z), a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));
        let p = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
        self.quad(mat, [p(x0, y1, z0), p(x1, y1, z0), p(x1, y1, z1), p(x0, y1, z1)], Vec3::new(0.0, 1.0, 0.0));
        self.quad(mat, [p(x0, y0, z0), p(x1, y0, z0), p(x1, y0, z1), p(x0, y0, z1)], Vec3::new(0.0, -1.0, 0.0));
        self.quad(mat, [p(x0, y0, z0), p(x1, y0, z0), p(x1, y1, z0), p(x0, y1, z0)], Vec3::new(0.0, 0.0, -1.0));
        self.quad(mat, [p(x0, y0, z1), p(x1, y0, z1), p(x1, y1, z1), p(x0, y1, z1)], Vec3::new(0.0, 0.0, 1.0));
        self.quad(mat, [p(x0, y0, z0), p(x0, y0, z1), p(x0, y1, z1), p(x0, y1, z0)], Vec3::new(-1.0, 0.0, 0.0));
        self.quad(mat, [p(x1, y0, z0), p(x1, y0, z1), p(x1, y1, z1), p(x1, y1, z0)], Vec3::new(1.0, 0.0, 0.0));
    }

    pub fn modelo(self) -> Modelo {
        Modelo::nuevo(self.partes.into_iter().filter(|p| !p.triangulos.is_empty()).collect(), self.materiales)
    }
}
