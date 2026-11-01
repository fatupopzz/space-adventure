//! Vectores y matrices 4x4, escritos a mano.
//!
//! El proyecto es un pipeline de render por software: transformar vertices,
//! proyectarlos y llevarlos a la pantalla ES esta aritmetica. Por eso no hay
//! libreria de algebra lineal: todo lo que hace cada etapa se ve aca.
//!
//! Convenciones (las de OpenGL, que son las de la clase):
//! - Mano derecha, Y arriba, la camara mira hacia -Z.
//! - Vectores columna: `v' = M * v`, asi que `A * B` aplica primero B.
//! - El espacio de recorte de la proyeccion deja la profundidad visible en
//!   `-w <= z <= w`.

use std::ops::{Add, AddAssign, Mul, Neg, Sub};

/// Un punto o una direccion en el espacio.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    pub const fn zero() -> Self {
        Vec3::new(0.0, 0.0, 0.0)
    }

    #[inline]
    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    #[inline]
    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// El vector con largo 1. El vector nulo se queda nulo en vez de
    /// llenarse de NaN.
    #[inline]
    pub fn normalize(self) -> Vec3 {
        let l = self.length();
        if l > 1e-12 {
            self * (1.0 / l)
        } else {
            self
        }
    }

    #[inline]
    pub fn lerp(self, o: Vec3, t: f32) -> Vec3 {
        self + (o - self) * t
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    #[inline]
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, o: Vec3) {
        *self = *self + o;
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    #[inline]
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, s: f32) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}

/// Producto componente a componente: es como se mezclan dos colores.
impl Mul<Vec3> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x * o.x, self.y * o.y, self.z * o.z)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    #[inline]
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

/// Un punto en coordenadas homogeneas. Sale del vertex shader en espacio de
/// recorte; la `w` es la que despues divide (el perspective divide).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vec4 { x, y, z, w }
    }

    #[inline]
    pub fn lerp(self, o: Vec4, t: f32) -> Vec4 {
        Vec4::new(
            self.x + (o.x - self.x) * t,
            self.y + (o.y - self.y) * t,
            self.z + (o.z - self.z) * t,
            self.w + (o.w - self.w) * t,
        )
    }
}

/// Matriz 4x4 guardada por filas: `m[fila][columna]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    pub const fn identity() -> Self {
        Mat4 {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn translation(t: Vec3) -> Self {
        let mut r = Mat4::identity();
        r.m[0][3] = t.x;
        r.m[1][3] = t.y;
        r.m[2][3] = t.z;
        r
    }

    /// Escala uniforme. Solo se usa uniforme a proposito: asi la normal se
    /// transforma con la misma matriz (sin la inversa transpuesta) y basta
    /// con normalizarla.
    pub fn scale(s: f32) -> Self {
        let mut r = Mat4::identity();
        r.m[0][0] = s;
        r.m[1][1] = s;
        r.m[2][2] = s;
        r
    }

    /// Escala distinta en cada eje. Solo para partes sin luz propia que
    /// importe (las llamas): con escala no uniforme la normal ya no se
    /// transforma bien con la misma matriz.
    pub fn scale_xyz(s: Vec3) -> Self {
        let mut r = Mat4::identity();
        r.m[0][0] = s.x;
        r.m[1][1] = s.y;
        r.m[2][2] = s.z;
        r
    }

    pub fn rotation_x(a: f32) -> Self {
        let (s, c) = a.sin_cos();
        let mut r = Mat4::identity();
        r.m[1][1] = c;
        r.m[1][2] = -s;
        r.m[2][1] = s;
        r.m[2][2] = c;
        r
    }

    pub fn rotation_y(a: f32) -> Self {
        let (s, c) = a.sin_cos();
        let mut r = Mat4::identity();
        r.m[0][0] = c;
        r.m[0][2] = s;
        r.m[2][0] = -s;
        r.m[2][2] = c;
        r
    }

    pub fn rotation_z(a: f32) -> Self {
        let (s, c) = a.sin_cos();
        let mut r = Mat4::identity();
        r.m[0][0] = c;
        r.m[0][1] = -s;
        r.m[1][0] = s;
        r.m[1][1] = c;
        r
    }

    /// La matriz cuyas columnas son los ejes `x`, `y`, `z`: lleva el eje Y
    /// del modelo a `y`. Sirve para apuntar algo (un haz de luz) a donde sea.
    pub fn from_basis(x: Vec3, y: Vec3, z: Vec3) -> Self {
        Mat4 {
            m: [
                [x.x, y.x, z.x, 0.0],
                [x.y, y.y, z.y, 0.0],
                [x.z, y.z, z.z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// La matriz de vista V: lleva el mundo al espacio de la camara, con la
    /// camara en el origen mirando hacia -Z.
    pub fn look_at(ojo: Vec3, objetivo: Vec3, arriba: Vec3) -> Self {
        let f = (objetivo - ojo).normalize();
        let r = f.cross(arriba).normalize();
        let u = r.cross(f);
        Mat4 {
            m: [
                [r.x, r.y, r.z, -r.dot(ojo)],
                [u.x, u.y, u.z, -u.dot(ojo)],
                [-f.x, -f.y, -f.z, f.dot(ojo)],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// La matriz de proyeccion P en perspectiva. Lo que queda entre `cerca`
    /// y `lejos` termina con `-w <= z <= w`, y la `w` de salida es la
    /// distancia a la camara: la que hace que lo lejano se vea chiquito
    /// cuando se divide.
    pub fn perspective(fov_y: f32, aspecto: f32, cerca: f32, lejos: f32) -> Self {
        let f = 1.0 / (fov_y * 0.5).tan();
        Mat4 {
            m: [
                [f / aspecto, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (lejos + cerca) / (cerca - lejos), 2.0 * lejos * cerca / (cerca - lejos)],
                [0.0, 0.0, -1.0, 0.0],
            ],
        }
    }

    #[inline]
    pub fn mul_vec4(&self, v: Vec4) -> Vec4 {
        let m = &self.m;
        Vec4::new(
            m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z + m[0][3] * v.w,
            m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z + m[1][3] * v.w,
            m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z + m[2][3] * v.w,
            m[3][0] * v.x + m[3][1] * v.y + m[3][2] * v.z + m[3][3] * v.w,
        )
    }

    /// Un punto (w = 1): le aplica rotacion, escala y traslacion.
    #[inline]
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        let r = self.mul_vec4(Vec4::new(p.x, p.y, p.z, 1.0));
        Vec3::new(r.x, r.y, r.z)
    }

    /// Una direccion (w = 0): rota y escala, pero no se traslada.
    #[inline]
    pub fn transform_dir(&self, d: Vec3) -> Vec3 {
        let r = self.mul_vec4(Vec4::new(d.x, d.y, d.z, 0.0));
        Vec3::new(r.x, r.y, r.z)
    }

    /// La matriz para transformar NORMALES: la inversa transpuesta de la
    /// parte 3x3 de esta matriz, n' = (M^-1)^T n. Con escala uniforme da lo
    /// mismo que M, pero con escala distinta en cada eje (las llamas que se
    /// estiran, las barras del ecualizador) la normal con M ya no seria
    /// perpendicular a la superficie. Se calcula como cofactores / determinante.
    pub fn matriz_normal(&self) -> Mat4 {
        let a = |i: usize, j: usize| self.m[i][j];
        // Cofactor (i, j) de la 3x3.
        let cof = |i: usize, j: usize| {
            let (r0, r1) = ((i + 1) % 3, (i + 2) % 3);
            let (c0, c1) = ((j + 1) % 3, (j + 2) % 3);
            a(r0, c0) * a(r1, c1) - a(r0, c1) * a(r1, c0)
        };
        let det = a(0, 0) * cof(0, 0) + a(0, 1) * cof(0, 1) + a(0, 2) * cof(0, 2);
        let inv_det = if det.abs() > 1e-12 { 1.0 / det } else { 0.0 };
        let mut r = Mat4::identity();
        for i in 0..3 {
            for j in 0..3 {
                // (M^-1)^T = cofactores / det (la adjunta ya viene transpuesta).
                r.m[i][j] = cof(i, j) * inv_det;
            }
        }
        r
    }
}

impl Mul for Mat4 {
    type Output = Mat4;
    fn mul(self, o: Mat4) -> Mat4 {
        let mut r = [[0.0; 4]; 4];
        for (i, fila) in r.iter_mut().enumerate() {
            for (j, celda) in fila.iter_mut().enumerate() {
                *celda = (0..4).map(|k| self.m[i][k] * o.m[k][j]).sum();
            }
        }
        Mat4 { m: r }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Con escala distinta en cada eje, la normal transformada con M deja de
    /// ser perpendicular; con la inversa transpuesta sigue siendolo.
    #[test]
    fn las_normales_van_con_la_inversa_transpuesta() {
        let m = Mat4::rotation_y(0.7) * Mat4::scale_xyz(Vec3::new(3.0, 1.0, 0.5));
        // Un plano con tangente (1, -1, 0) y normal (1, 1, 0).
        let tangente = m.transform_dir(Vec3::new(1.0, -1.0, 0.0));
        let con_m = m.transform_dir(Vec3::new(1.0, 1.0, 0.0)).normalize();
        let bien = m.matriz_normal().transform_dir(Vec3::new(1.0, 1.0, 0.0)).normalize();
        assert!(con_m.dot(tangente.normalize()).abs() > 0.3, "con M deberia torcerse");
        assert!(bien.dot(tangente.normalize()).abs() < 1e-5, "con la inversa transpuesta es perpendicular");
        // Y con escala uniforme da lo mismo que M.
        let r = Mat4::rotation_x(0.4) * Mat4::scale(2.0);
        let n = Vec3::new(0.0, 0.6, 0.8);
        assert!((r.matriz_normal().transform_dir(n).normalize() - r.transform_dir(n).normalize()).length() < 1e-5);
    }
}
