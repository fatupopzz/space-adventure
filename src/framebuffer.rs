//! El framebuffer: la imagen que arma el pipeline, con su z-buffer.
//!
//! El color se guarda ya en RGBA de 8 bits, en el mismo orden que la
//! textura de raylib, para subirlo a la GPU sin convertir nada: raylib solo
//! pone la ventana, todo lo que se ve se pinta aca.

pub struct Framebuffer {
    pub ancho: usize,
    pub alto: usize,
    color: Vec<u8>,
    /// Profundidad de cada pixel en [0, 1] (0 = plano cercano). Empieza en
    /// infinito: "no hay nada dibujado" (el cielo no cuenta).
    profundidad: Vec<f32>,
    /// El STENCIL BUFFER: un numero por pixel que funciona como mascara. Un
    /// draw call puede marcarlo y otro dibujar solo donde quedo marcado (el
    /// portal del Planeta Hada).
    stencil: Vec<u8>,
}

impl Framebuffer {
    pub fn new(ancho: usize, alto: usize) -> Self {
        Framebuffer { ancho, alto, color: vec![255; ancho * alto * 4], profundidad: vec![f32::INFINITY; ancho * alto], stencil: vec![0; ancho * alto] }
    }

    pub fn limpiar(&mut self, fondo: [u8; 3]) {
        for px in self.color.chunks_exact_mut(4) {
            px.copy_from_slice(&[fondo[0], fondo[1], fondo[2], 255]);
        }
        self.limpiar_profundidad();
    }

    /// Todo transparente (alfa 0): para dibujar algo suelto, como Miku en los menus, y pegarlo
    /// encima de la interfaz. Lo que se pinta despues queda opaco.
    pub fn limpiar_transparente(&mut self) {
        self.color.fill(0);
        self.limpiar_profundidad();
    }

    /// En un framebuffer transparente, la luz sumada (estelas, halos) no toca el alfa: aqui cada
    /// pixel se vuelve tan opaco como brilla, para que se vea al pegarlo encima de otra cosa.
    pub fn opacar_luz(&mut self) {
        for px in self.color.chunks_exact_mut(4) {
            let brillo = px[0].max(px[1]).max(px[2]);
            px[3] = px[3].max(brillo);
        }
    }

    /// Solo el z-buffer y el stencil: el color lo tapa entero el cielo.
    pub fn limpiar_profundidad(&mut self) {
        self.profundidad.fill(f32::INFINITY);
        self.stencil.fill(0);
    }

    /// Pinta el z-buffer como imagen: mas claro, mas cerca. La profundidad
    /// en perspectiva se amontona cerca de 1, asi que se muestra en escala
    /// logaritmica de (1 - z); lo que no tiene nada (el cielo) queda negro.
    pub fn ver_profundidad(&mut self) {
        for i in 0..self.ancho * self.alto {
            let z = self.profundidad[i];
            let k = if z.is_finite() { (1.0 + (1.0 - z).max(1e-6).log10() / 4.0).clamp(0.0, 1.0) } else { 0.0 };
            self.pintar(i, [k, k, k * 1.1], 1.0);
        }
    }

    /// Pinta el stencil buffer: la imagen en gris oscuro y lo marcado encima en color.
    pub fn ver_stencil(&mut self) {
        for i in 0..self.ancho * self.alto {
            let [r, g, b] = self.leer(i);
            let gris = (r + g + b) / 3.0 * 0.4;
            let rgb = if self.stencil[i] > 0 { [1.0, 0.3 + gris, 0.8] } else { [gris, gris, gris] };
            self.pintar(i, rgb, 1.0);
        }
    }

    /// El DEPTH TEST: si el fragmento en `i` queda delante de lo que ya hay.
    #[inline]
    pub fn pasa_profundidad(&self, i: usize, z: f32) -> bool {
        z < self.profundidad[i]
    }

    #[inline]
    pub fn profundidad(&self, i: usize) -> f32 {
        self.profundidad[i]
    }

    #[inline]
    pub fn escribir_profundidad(&mut self, i: usize, z: f32) {
        self.profundidad[i] = z;
    }

    /// Escribe un color en [0, 1] (lo que pase de 1 se satura). Con `alfa`
    /// menor que 1 hace BLENDING: mezcla con lo que ya estaba,
    /// `nuevo * alfa + viejo * (1 - alfa)`.
    #[inline]
    pub fn pintar(&mut self, i: usize, rgb: [f32; 3], alfa: f32) {
        let px = &mut self.color[i * 4..i * 4 + 3];
        for (c, v) in px.iter_mut().zip(rgb) {
            let nuevo = v.clamp(0.0, 1.0) * 255.0;
            *c = if alfa >= 1.0 { nuevo } else { nuevo * alfa + *c as f32 * (1.0 - alfa) } as u8;
        }
        // Lo pintado se vuelve opaco (solo importa si se limpio transparente).
        let a = &mut self.color[i * 4 + 3];
        *a = (*a).max((alfa.min(1.0) * 255.0) as u8);
    }

    /// Blending ADITIVO: suma luz a lo que ya hay. Es como se ven los
    /// brillos, las estelas y el resplandor: nunca oscurecen.
    #[inline]
    pub fn sumar(&mut self, i: usize, rgb: [f32; 3]) {
        let px = &mut self.color[i * 4..i * 4 + 3];
        for (c, v) in px.iter_mut().zip(rgb) {
            *c = (*c as f32 + v.max(0.0) * 255.0).min(255.0) as u8;
        }
    }

    #[inline]
    pub fn leer(&self, i: usize) -> [f32; 3] {
        let px = &self.color[i * 4..i * 4 + 3];
        [px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0]
    }

    /// Una linea de luz (aditiva) de `a` a `b`, cada punto con su
    /// profundidad: lo que ya este delante la tapa. Para estelas y polvo.
    pub fn linea_de_luz(&mut self, a: (f32, f32, f32), b: (f32, f32, f32), rgb_a: [f32; 3], rgb_b: [f32; 3]) {
        let pasos = (b.0 - a.0).abs().max((b.1 - a.1).abs()).ceil().clamp(1.0, 2000.0) as usize;
        for s in 0..=pasos {
            let t = s as f32 / pasos as f32;
            let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            if x < 0.0 || y < 0.0 || x >= self.ancho as f32 || y >= self.alto as f32 {
                continue;
            }
            let i = y as usize * self.ancho + x as usize;
            if self.pasa_profundidad(i, a.2 + (b.2 - a.2) * t) {
                let c = [0, 1, 2].map(|k| rgb_a[k] + (rgb_b[k] - rgb_a[k]) * t);
                self.sumar(i, c);
            }
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.color
    }

    /// Parte la imagen en `n` franjas horizontales que se pueden pintar al
    /// mismo tiempo, cada una en su hilo: no comparten ningun pixel.
    pub fn bandas(&mut self, n: usize) -> Vec<Banda<'_>> {
        let filas = self.alto.div_ceil(n.max(1));
        let ancho = self.ancho;
        self.color
            .chunks_mut(filas * ancho * 4)
            .zip(self.profundidad.chunks_mut(filas * ancho))
            .zip(self.stencil.chunks_mut(filas * ancho))
            .enumerate()
            .map(|(k, ((color, profundidad), stencil))| Banda {
                ancho,
                y0: k * filas,
                y1: k * filas + profundidad.len() / ancho,
                base: k * filas * ancho,
                color,
                profundidad,
                stencil,
            })
            .collect()
    }
}

/// Unas filas del framebuffer, de `y0` a `y1` (sin incluir). Se indexa con
/// el mismo `i` que el framebuffer entero (`y * ancho + x`).
pub struct Banda<'a> {
    pub ancho: usize,
    pub y0: usize,
    pub y1: usize,
    base: usize,
    color: &'a mut [u8],
    profundidad: &'a mut [f32],
    stencil: &'a mut [u8],
}

impl Banda<'_> {
    #[inline]
    pub fn pasa_profundidad(&self, i: usize, z: f32) -> bool {
        z < self.profundidad[i - self.base]
    }

    #[inline]
    pub fn escribir_profundidad(&mut self, i: usize, z: f32) {
        self.profundidad[i - self.base] = z;
    }

    #[inline]
    pub fn stencil(&self, i: usize) -> u8 {
        self.stencil[i - self.base]
    }

    #[inline]
    pub fn escribir_stencil(&mut self, i: usize, v: u8) {
        self.stencil[i - self.base] = v;
    }

    /// Igual que `Framebuffer::pintar`.
    #[inline]
    pub fn pintar(&mut self, i: usize, rgb: [f32; 3], alfa: f32) {
        let j = (i - self.base) * 4;
        for (c, v) in self.color[j..j + 3].iter_mut().zip(rgb) {
            let nuevo = v.clamp(0.0, 1.0) * 255.0;
            *c = if alfa >= 1.0 { nuevo } else { nuevo * alfa + *c as f32 * (1.0 - alfa) } as u8;
        }
        self.color[j + 3] = self.color[j + 3].max((alfa.min(1.0) * 255.0) as u8);
    }
}
