use core::f32::consts::PI;

/// Exact ASCII character ramp ordered by optical density on a solid black background:
/// sparse shadows and delicate contours (';', ':') up to luminous crests ('#', '@').
pub const DENSITY_RAMP: [u8; 8] = [b';', b':', b'=', b'+', b'*', b'%', b'#', b'@'];

const MAX_COLS: usize = 240;
const MAX_ROWS: usize = 140;

#[derive(Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    #[inline]
    const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }

    #[inline]
    fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }

    #[inline]
    fn scale(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    #[inline]
    fn dot(self, o: Self) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    #[inline]
    fn cross(self, o: Self) -> Self {
        Self::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    #[inline]
    fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    #[inline]
    fn normalize(self) -> Self {
        let l = self.len();
        if l > 1e-6 {
            self.scale(1.0 / l)
        } else {
            Self::new(0.0, 0.0, 1.0)
        }
    }

    #[inline]
    fn rot_x(self, a: f32) -> Self {
        let (s, c) = a.sin_cos();
        Self::new(self.x, self.y * c - self.z * s, self.y * s + self.z * c)
    }

    #[inline]
    fn rot_y(self, a: f32) -> Self {
        let (s, c) = a.sin_cos();
        Self::new(self.x * c + self.z * s, self.y, -self.x * s + self.z * c)
    }

    #[inline]
    fn rot_z(self, a: f32) -> Self {
        let (s, c) = a.sin_cos();
        Self::new(self.x * c - self.y * s, self.x * s + self.y * c, self.z)
    }
}

#[derive(Clone, Copy)]
struct Rgb {
    r: f32,
    g: f32,
    b: f32,
}

impl Rgb {
    #[inline]
    const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    #[inline]
    fn lerp(self, o: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::new(
            self.r + (o.r - self.r) * t,
            self.g + (o.g - self.g) * t,
            self.b + (o.b - self.b) * t,
        )
    }

    #[inline]
    fn to_u8(self) -> (u8, u8, u8) {
        (
            self.r.clamp(0.0, 255.0) as u8,
            self.g.clamp(0.0, 255.0) as u8,
            self.b.clamp(0.0, 255.0) as u8,
        )
    }
}

#[derive(Clone, Copy)]
struct Palette {
    base_cup: Rgb,
    shadow: Rgb,
    mid: Rgb,
    bright: Rgb,
    highlight: Rgb,
}

impl Palette {
    fn sample(&self, shade: f32, v_cup: f32, spec: f32, rim: f32) -> Rgb {
        let base_mix = ((0.26 - v_cup) / 0.26).clamp(0.0, 1.0) * 0.48;
        let t = shade.clamp(0.0, 1.0);

        let body = if t < 0.48 {
            self.shadow.lerp(self.mid, t / 0.48)
        } else {
            self.mid.lerp(self.bright, (t - 0.48) / 0.52)
        };

        let mut col = body.lerp(self.base_cup, base_mix);

        // White / light-gray luminous highlights on specular crests and rim curves
        let light_touch = (spec * 1.12 + rim * 0.40).clamp(0.0, 1.0);
        if light_touch > 0.22 {
            let k = ((light_touch - 0.22) / 0.78).powf(0.95);
            col = col.lerp(self.highlight, (k * 0.94).clamp(0.0, 0.94));
        }
        col
    }
}

// --- Mixed Pastel Garden Palette (calibrated for solid black #000000 terminal canvas) ---

const PAL_PASTEL_ROSE: Palette = Palette {
    base_cup: Rgb::new(174.0, 72.0, 108.0),
    shadow: Rgb::new(206.0, 102.0, 140.0),
    mid: Rgb::new(250.0, 156.0, 192.0),
    bright: Rgb::new(255.0, 206.0, 226.0),
    highlight: Rgb::new(255.0, 248.0, 252.0),
};

const PAL_PASTEL_PEARL_BLUSH: Palette = Palette {
    base_cup: Rgb::new(166.0, 146.0, 156.0),
    shadow: Rgb::new(192.0, 178.0, 188.0),
    mid: Rgb::new(236.0, 226.0, 234.0),
    bright: Rgb::new(252.0, 246.0, 250.0),
    highlight: Rgb::new(255.0, 255.0, 255.0),
};

const PAL_PASTEL_PEACH: Palette = Palette {
    base_cup: Rgb::new(188.0, 94.0, 68.0),
    shadow: Rgb::new(222.0, 124.0, 94.0),
    mid: Rgb::new(255.0, 172.0, 138.0),
    bright: Rgb::new(255.0, 216.0, 192.0),
    highlight: Rgb::new(255.0, 248.0, 242.0),
};

const PAL_PASTEL_LILAC: Palette = Palette {
    base_cup: Rgb::new(118.0, 78.0, 174.0),
    shadow: Rgb::new(148.0, 108.0, 208.0),
    mid: Rgb::new(200.0, 156.0, 252.0),
    bright: Rgb::new(234.0, 210.0, 255.0),
    highlight: Rgb::new(250.0, 246.0, 255.0),
};

const PAL_PASTEL_BUTTER: Palette = Palette {
    base_cup: Rgb::new(186.0, 148.0, 48.0),
    shadow: Rgb::new(216.0, 180.0, 74.0),
    mid: Rgb::new(254.0, 226.0, 118.0),
    bright: Rgb::new(255.0, 244.0, 178.0),
    highlight: Rgb::new(255.0, 253.0, 242.0),
};

const PAL_LEAF_SAGE: Palette = Palette {
    base_cup: Rgb::new(24.0, 96.0, 56.0),
    shadow: Rgb::new(36.0, 138.0, 78.0),
    mid: Rgb::new(72.0, 210.0, 124.0),
    bright: Rgb::new(146.0, 242.0, 178.0),
    highlight: Rgb::new(230.0, 248.0, 236.0),
};

const PASTEL_STAMEN: Rgb = Rgb::new(255.0, 226.0, 118.0);

#[derive(Clone, Copy)]
struct SubPixel {
    z: f32,
    lum: f32,
    r: u8,
    g: u8,
    b: u8,
    surface_id: u16,
    kind: u8, // 0 = empty, 1 = flower, 2 = stem, 3 = leaf
}

impl SubPixel {
    const EMPTY: Self = Self {
        z: -1e9,
        lum: 0.0,
        r: 0,
        g: 0,
        b: 0,
        surface_id: 0,
        kind: 0,
    };
}

#[derive(Clone, Copy)]
struct CellSummary {
    z: f32,
    lum: f32,
    r: f32,
    g: f32,
    b: f32,
    surface_id: u16,
    kind: u8,
    coverage: u8,
}

impl CellSummary {
    const EMPTY: Self = Self {
        z: -1e9,
        lum: 0.0,
        r: 0.0,
        g: 0.0,
        b: 0.0,
        surface_id: 0,
        kind: 0,
        coverage: 0,
    };
}

#[derive(Clone, Copy)]
struct Star {
    nx: f32,
    ny: f32,
    phase: f32,
    speed: f32,
    base_lum: f32,
    drift_y: f32,
    drift_x: f32,
}

#[derive(Clone, Copy)]
struct TulipSpec {
    base_pos: Vec3,
    waist_pos: Vec3,
    head_pos: Vec3,
    roll: f32,
    pitch: f32,
    yaw: f32,
    radius: f32,
    height: f32,
    stem_radius: f32,
    sway_phase: f32,
    sway_amp: f32,
    bloom_delay: f32,
    palette: Palette,
    id_base: u16,
}

#[derive(Clone, Copy)]
struct PeonySpec {
    base_pos: Vec3,
    waist_pos: Vec3,
    head_pos: Vec3,
    roll: f32,
    pitch: f32,
    yaw: f32,
    radius: f32,
    stem_radius: f32,
    sway_phase: f32,
    sway_amp: f32,
    bloom_delay: f32,
    palette: Palette,
    stamen_col: Rgb,
    id_base: u16,
}

#[derive(Clone, Copy)]
struct FreesiaSpec {
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    stem_radius: f32,
    floret_scale: f32,
    lean_x: f32,
    sway_phase: f32,
    sway_amp: f32,
    bloom_delay: f32,
    palette: Palette,
    throat_col: Rgb,
    id_base: u16,
}

#[derive(Clone, Copy)]
struct LeafSpec {
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    max_width: f32,
    twist_start: f32,
    twist_end: f32,
    fold_depth: f32,
    sway_phase: f32,
    sway_amp: f32,
    id: u16,
}

/// Real-time 3D pastel garden bouquet renderer projecting onto a colorized ASCII grid.
pub struct BouquetRenderer {
    cols: usize,
    rows: usize,
    char_aspect: f32,
    time: f32,
    bloom_age: f32,
    wind_strength: f32,

    pointer_x: f32,
    pointer_y: f32,
    pointer_active: bool,
    pointer_push_x: f32,
    pointer_push_vx: f32,

    sub_buf: Vec<SubPixel>,
    cell_buf: Vec<CellSummary>,
    out_buf: Vec<u8>,
    ascii_text: Vec<u8>,
    stars: Vec<Star>,
}

impl BouquetRenderer {
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut r = Self {
            cols: 0,
            rows: 0,
            char_aspect: 0.54,
            time: 0.0,
            bloom_age: 0.0,
            wind_strength: 1.0,
            pointer_x: 0.0,
            pointer_y: 0.0,
            pointer_active: false,
            pointer_push_x: 0.0,
            pointer_push_vx: 0.0,
            sub_buf: Vec::new(),
            cell_buf: Vec::new(),
            out_buf: Vec::new(),
            ascii_text: Vec::new(),
            stars: Vec::new(),
        };
        r.init_stars();
        r.resize(cols, rows);
        r
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        let c = cols.clamp(40, MAX_COLS);
        let r = rows.clamp(24, MAX_ROWS);
        if c == self.cols && r == self.rows {
            return;
        }
        self.cols = c;
        self.rows = r;
        self.sub_buf.resize((c * 2) * (r * 2), SubPixel::EMPTY);
        self.cell_buf.resize(c * r, CellSummary::EMPTY);
        self.out_buf.resize(c * r * 4, 0);
        self.ascii_text.reserve((c + 1) * r);
    }

    pub fn set_char_aspect(&mut self, aspect: f32) {
        self.char_aspect = aspect.clamp(0.35, 0.85);
    }

    pub fn set_wind(&mut self, strength: f32) {
        self.wind_strength = strength.clamp(0.0, 2.2);
    }

    pub fn set_pointer(&mut self, nx: f32, ny: f32, active: bool) {
        self.pointer_x = nx.clamp(-1.2, 1.2);
        self.pointer_y = ny.clamp(-1.2, 1.2);
        self.pointer_active = active;
    }

    pub fn trigger_bloom(&mut self) {
        self.bloom_age = 0.0;
    }

    pub fn set_bloom_instant(&mut self) {
        self.bloom_age = 10.0;
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn out_bytes(&self) -> &[u8] {
        &self.out_buf
    }

    pub fn build_ascii_text(&mut self) -> &[u8] {
        self.ascii_text.clear();
        for row in 0..self.rows {
            let mut last_non_space = 0usize;
            for col in 0..self.cols {
                let g = self.out_buf[(row * self.cols + col) * 4];
                if g != b' ' {
                    last_non_space = col + 1;
                }
            }
            for col in 0..last_non_space {
                self.ascii_text.push(self.out_buf[(row * self.cols + col) * 4]);
            }
            if row + 1 < self.rows {
                self.ascii_text.push(b'\n');
            }
        }
        &self.ascii_text
    }

    fn init_stars(&mut self) {
        self.stars.clear();
        let total = 122usize;
        let g = 1.324_717_96_f32;
        let a1 = 1.0 / g;
        let a2 = 1.0 / (g * g);

        for i in 1..=total {
            let fi = i as f32;
            let nx = (0.5 + a1 * fi).fract();
            let ny = (0.5 + a2 * fi).fract();
            let h1 = ((fi * 12.9898).sin() * 43_758.545).fract().abs();
            let h2 = ((fi * 78.233).sin() * 29_183.213).fract().abs();

            let is_drifter = i % 5 == 0;
            self.stars.push(Star {
                nx,
                ny,
                phase: h1 * PI * 2.0,
                speed: 0.45 + h2 * 1.15,
                base_lum: 0.34 + h1 * 0.62,
                drift_y: if is_drifter { 0.011 + h2 * 0.014 } else { 0.0 },
                drift_x: if is_drifter { (h1 - 0.5) * 0.008 } else { 0.0 },
            });
        }
    }

    pub fn render(&mut self, dt: f32) {
        let clamped_dt = dt.clamp(0.0, 0.05);
        self.time += clamped_dt;
        self.bloom_age = (self.bloom_age + clamped_dt).min(12.0);

        // Smooth spring-damper pointer breeze
        let target_push = if self.pointer_active {
            let dist_weight = (1.0 - self.pointer_y.abs() * 0.45).clamp(0.2, 1.0);
            (-self.pointer_x * 0.22 * dist_weight).clamp(-0.24, 0.24)
        } else {
            0.0
        };
        let accel = (target_push - self.pointer_push_x) * 14.0 - self.pointer_push_vx * 5.2;
        self.pointer_push_vx += accel * clamped_dt;
        self.pointer_push_x += self.pointer_push_vx * clamped_dt;

        self.sub_buf.fill(SubPixel::EMPTY);

        let light_drift = (self.time * 0.35).sin() * 0.08;
        let key_light = Vec3::new(-0.54 + light_drift, 0.34, 0.77).normalize();
        let fill_light = Vec3::new(0.75, 0.18, 0.64).normalize();

        let t = self.time;
        let wind = self.wind_strength;
        let global_gust = wind
            * (0.55 * (t * 0.78).sin()
                + 0.28 * (t * 1.43 + 0.9).sin()
                + 0.17 * (t * 0.37 - 0.5).cos());

        let rise = ease_out_cubic((self.bloom_age / 1.45).clamp(0.0, 1.0));

        // 1. Botanical foliage (broad tulip leaves + slender freesia blades + clasping waist leaf)
        let leaves = [
            LeafSpec {
                p0: Vec3::new(-0.05, -0.68, 0.02),
                p1: Vec3::new(-0.30, -0.38, 0.08),
                p2: Vec3::new(-0.66, -0.08, 0.10),
                p3: Vec3::new(-0.96, -0.06, 0.01),
                max_width: 0.090,
                twist_start: 0.24,
                twist_end: 0.72,
                fold_depth: 0.025,
                sway_phase: 0.12,
                sway_amp: 0.046,
                id: 701,
            },
            LeafSpec {
                p0: Vec3::new(0.05, -0.68, 0.02),
                p1: Vec3::new(0.32, -0.40, 0.08),
                p2: Vec3::new(0.68, -0.14, 0.10),
                p3: Vec3::new(0.98, -0.12, 0.01),
                max_width: 0.088,
                twist_start: -0.24,
                twist_end: -0.72,
                fold_depth: 0.025,
                sway_phase: 1.08,
                sway_amp: 0.048,
                id: 702,
            },
            LeafSpec {
                p0: Vec3::new(-0.04, -0.60, -0.06),
                p1: Vec3::new(-0.22, -0.18, -0.06),
                p2: Vec3::new(-0.36, 0.20, -0.04),
                p3: Vec3::new(-0.44, 0.50, -0.08),
                max_width: 0.044,
                twist_start: 0.15,
                twist_end: 0.40,
                fold_depth: 0.014,
                sway_phase: 0.34,
                sway_amp: 0.045,
                id: 703,
            },
            LeafSpec {
                p0: Vec3::new(0.04, -0.60, -0.06),
                p1: Vec3::new(0.24, -0.20, -0.06),
                p2: Vec3::new(0.38, 0.16, -0.04),
                p3: Vec3::new(0.48, 0.44, -0.08),
                max_width: 0.044,
                twist_start: -0.15,
                twist_end: -0.42,
                fold_depth: 0.014,
                sway_phase: 0.86,
                sway_amp: 0.046,
                id: 704,
            },
            LeafSpec {
                p0: Vec3::new(-0.07, -0.66, 0.07),
                p1: Vec3::new(-0.04, -0.46, 0.13),
                p2: Vec3::new(0.06, -0.28, 0.14),
                p3: Vec3::new(0.18, -0.14, 0.10),
                max_width: 0.056,
                twist_start: -0.10,
                twist_end: 0.34,
                fold_depth: 0.018,
                sway_phase: 0.60,
                sway_amp: 0.026,
                id: 705,
            },
        ];

        for leaf in &leaves {
            self.render_leaf(leaf, &PAL_LEAF_SAGE, global_gust, rise, key_light, fill_light);
        }

        // 2. Two airy arching FREESIA spikes (butter yellow left, pearl white right)
        let freesias = [
            FreesiaSpec {
                p0: Vec3::new(0.09, -0.94, -0.05),
                p1: Vec3::new(-0.08, -0.44, -0.03),
                p2: Vec3::new(-0.36, 0.48, -0.01),
                p3: Vec3::new(-0.92, 0.78, 0.03),
                stem_radius: 0.020,
                floret_scale: 1.46,
                lean_x: -1.0,
                sway_phase: 0.18,
                sway_amp: 0.074,
                bloom_delay: 0.36,
                palette: PAL_PASTEL_BUTTER,
                throat_col: PASTEL_STAMEN,
                id_base: 800,
            },
            FreesiaSpec {
                p0: Vec3::new(-0.09, -0.94, -0.05),
                p1: Vec3::new(0.09, -0.44, -0.03),
                p2: Vec3::new(0.40, 0.42, -0.01),
                p3: Vec3::new(0.94, 0.68, 0.03),
                stem_radius: 0.020,
                floret_scale: 1.46,
                lean_x: 1.0,
                sway_phase: 1.12,
                sway_amp: 0.076,
                bloom_delay: 0.48,
                palette: PAL_PASTEL_PEARL_BLUSH,
                throat_col: PASTEL_STAMEN,
                id_base: 900,
            },
        ];

        for freesia in &freesias {
            self.render_freesia_spike(freesia, &PAL_LEAF_SAGE, global_gust, rise, key_light, fill_light);
        }

        // 3. Two sculptural TULIPS (pastel lilac left, pastel peach center-top)
        let tulips = [
            TulipSpec {
                base_pos: Vec3::new(0.14, -0.94, -0.02),
                waist_pos: Vec3::new(-0.06, -0.48, 0.01),
                head_pos: Vec3::new(-0.67, 0.14, 0.04),
                roll: 0.36,
                pitch: 0.11,
                yaw: 0.18,
                radius: 0.185,
                height: 0.435,
                stem_radius: 0.019,
                sway_phase: 0.05,
                sway_amp: 0.064,
                bloom_delay: 0.20,
                palette: PAL_PASTEL_LILAC,
                id_base: 100,
            },
            TulipSpec {
                base_pos: Vec3::new(-0.02, -0.95, -0.01),
                waist_pos: Vec3::new(0.02, -0.46, 0.02),
                head_pos: Vec3::new(0.10, 0.40, 0.04),
                roll: -0.08,
                pitch: 0.10,
                yaw: -0.06,
                radius: 0.196,
                height: 0.460,
                stem_radius: 0.020,
                sway_phase: 0.52,
                sway_amp: 0.060,
                bloom_delay: 0.08,
                palette: PAL_PASTEL_PEACH,
                id_base: 200,
            },
        ];

        for tulip in &tulips {
            self.render_tulip(tulip, &PAL_LEAF_SAGE, global_gust, rise, key_light, fill_light);
        }

        // 4. Two lush multi-layered PEONIES (blush rose center-left, pearl blush right)
        let peonies = [
            PeonySpec {
                base_pos: Vec3::new(0.04, -0.95, 0.02),
                waist_pos: Vec3::new(-0.02, -0.46, 0.06),
                head_pos: Vec3::new(-0.17, 0.08, 0.17),
                roll: 0.12,
                pitch: 1.06,
                yaw: 0.10,
                radius: 0.270,
                stem_radius: 0.020,
                sway_phase: 0.35,
                sway_amp: 0.050,
                bloom_delay: 0.0,
                palette: PAL_PASTEL_ROSE,
                stamen_col: PASTEL_STAMEN,
                id_base: 400,
            },
            PeonySpec {
                base_pos: Vec3::new(-0.13, -0.94, 0.01),
                waist_pos: Vec3::new(0.06, -0.48, 0.05),
                head_pos: Vec3::new(0.46, -0.02, 0.15),
                roll: -0.20,
                pitch: 1.02,
                yaw: -0.18,
                radius: 0.236,
                stem_radius: 0.019,
                sway_phase: 0.88,
                sway_amp: 0.056,
                bloom_delay: 0.26,
                palette: PAL_PASTEL_PEARL_BLUSH,
                stamen_col: PASTEL_STAMEN,
                id_base: 550,
            },
        ];

        for peony in &peonies {
            self.render_peony(peony, &PAL_LEAF_SAGE, global_gust, rise, key_light, fill_light);
        }

        self.resolve_grid();
    }

    fn render_stem_tube(
        &mut self,
        p0: Vec3,
        p1: Vec3,
        p2: Vec3,
        p3: Vec3,
        stem_radius: f32,
        bulge_tip: bool,
        leaf_pal: &Palette,
        stem_id: u16,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let stem_steps_v = 84;
        let stem_steps_u = 14;

        for iv in 0..=stem_steps_v {
            let v = iv as f32 / stem_steps_v as f32;
            let c = bezier3(p0, p1, p2, p3, v);
            let tan = bezier3_tangent(p0, p1, p2, p3, v).normalize();
            let side = tan.cross(Vec3::new(0.0, 0.0, 1.0)).normalize();
            let fwd = side.cross(tan).normalize();

            let tip_bulge = if bulge_tip { 0.30 * v.powf(7.0) } else { -0.25 * v };
            let r_stem = (stem_radius * (1.0 - 0.10 * v + tip_bulge)).max(0.005);

            for iu in 0..stem_steps_u {
                let ang = (iu as f32 / stem_steps_u as f32) * PI * 2.0;
                let (sa, ca) = ang.sin_cos();
                let n = side.scale(ca).add(fwd.scale(sa)).normalize();
                if n.z < -0.25 {
                    continue;
                }
                let pos = c.add(n.scale(r_stem));
                let waist_ao = 1.0 - 0.22 * (-(v - 0.42).powi(2) * 32.0).exp();
                let bottom_fade = (v / 0.08).clamp(0.45, 1.0);
                let (lum, col) = shade_surface(
                    n,
                    leaf_pal,
                    v * 0.5,
                    waist_ao * bottom_fade,
                    0.22,
                    key_light,
                    fill_light,
                );
                self.plot_3d(pos, lum, col, stem_id, 2);
            }
        }
    }

    fn render_tulip(
        &mut self,
        spec: &TulipSpec,
        leaf_pal: &Palette,
        global_gust: f32,
        rise: f32,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let t = self.time;
        let wind = self.wind_strength;

        let local_sway = (global_gust
            + wind * 0.36 * (t * 1.12 + spec.sway_phase).sin()
            + wind * 0.14 * (t * 2.18 + spec.sway_phase * 1.7).cos())
            * spec.sway_amp
            + self.pointer_push_x * 0.45;
        let local_depth_sway = wind * 0.022 * (t * 0.92 + spec.sway_phase + 1.1).cos();

        let bloom_raw = ((self.bloom_age - spec.bloom_delay) / 2.2).clamp(0.0, 1.0);
        let bloom_base = ease_out_cubic(bloom_raw);
        let breath = 0.045 * (t * 0.85 + spec.sway_phase).sin() * bloom_base;
        let bloom = (0.14 + 0.86 * bloom_base + breath).clamp(0.10, 1.08);

        let p0 = spec.base_pos;
        let p1 = Vec3::new(
            spec.waist_pos.x + local_sway * 0.14,
            spec.waist_pos.y,
            spec.waist_pos.z + local_depth_sway * 0.15,
        );
        let p3 = Vec3::new(
            spec.head_pos.x + local_sway,
            spec.head_pos.y - local_sway.abs() * 0.14 - (1.0 - rise) * 0.24,
            spec.head_pos.z + local_depth_sway,
        );
        let p2 = Vec3::new(
            spec.waist_pos.x * 0.35 + p3.x * 0.65 + local_sway * 0.25,
            spec.waist_pos.y * 0.32 + p3.y * 0.68,
            spec.waist_pos.z * 0.35 + p3.z * 0.65 + local_depth_sway * 0.5,
        );

        self.render_stem_tube(
            p0,
            p1,
            p2,
            p3,
            spec.stem_radius,
            true,
            leaf_pal,
            spec.id_base + 90,
            key_light,
            fill_light,
        );

        let roll_dyn = spec.roll - local_sway * 1.45;
        let pitch_dyn = spec.pitch + local_depth_sway * 1.6 + (1.0 - bloom_base) * 0.08;
        let yaw_dyn = spec.yaw + local_sway * 0.35;

        let head_scale = 0.72 + 0.28 * rise;
        let radius_s = spec.radius * head_scale;
        let height_s = spec.height * head_scale;

        let petals: [(f32, bool, u16); 6] = [
            (0.0, true, spec.id_base + 1),
            (1.96, true, spec.id_base + 2),
            (-1.96, true, spec.id_base + 3),
            (0.98, false, spec.id_base + 4),
            (-0.98, false, spec.id_base + 5),
            (PI, false, spec.id_base + 6),
        ];

        let u_steps = 40usize;
        let v_steps = 54usize;
        let du = 0.018;
        let dv = 0.014;

        for &(azim, is_outer, petal_id) in &petals {
            for iv in 0..=v_steps {
                let v = iv as f32 / v_steps as f32;
                let row0 = precompute_tulip_row(v, is_outer, bloom, radius_s, height_s);
                let row1 = precompute_tulip_row((v + dv).min(1.0), is_outer, bloom, radius_s, height_s);
                let cup_ao = 0.78 + 0.22 * v.powf(0.42);
                let whorl_ao = if is_outer { 1.0 } else { 0.82 + 0.18 * v };
                let sin_vpi = (v * PI).sin();

                for iu in 0..=u_steps {
                    let u = -1.0 + 2.0 * (iu as f32 / u_steps as f32);
                    let p_loc = eval_tulip_from_row(u, azim, &row0);
                    let pu = eval_tulip_from_row((u + du).min(1.0), azim, &row0);
                    let pv = eval_tulip_from_row(u, azim, &row1);

                    let n_loc = pu.sub(p_loc).cross(pv.sub(p_loc)).normalize();

                    let p_world = p_loc
                        .rot_y(yaw_dyn)
                        .rot_x(pitch_dyn)
                        .rot_z(roll_dyn)
                        .add(p3);
                    let mut n_world = n_loc
                        .rot_y(yaw_dyn)
                        .rot_x(pitch_dyn)
                        .rot_z(roll_dyn)
                        .normalize();

                    let mut interior_factor = 1.0_f32;
                    if n_world.z < 0.0 {
                        n_world = n_world.scale(-1.0);
                        interior_factor = 0.56;
                    }

                    let u_abs = u.abs();
                    let edge_crease = 1.0 - 0.28 * (u_abs * u_abs * u_abs.sqrt());
                    let vein_mod = 1.0 + 0.08 * (u * PI * 2.0).cos() * sin_vpi;
                    let ao = (cup_ao * whorl_ao * interior_factor * edge_crease * vein_mod)
                        .clamp(0.28, 1.12);

                    let (lum, col) = shade_surface(
                        n_world,
                        &spec.palette,
                        v,
                        ao,
                        1.15,
                        key_light,
                        fill_light,
                    );
                    self.plot_3d(p_world, lum, col, petal_id, 1);
                }
            }
        }
    }

    fn render_peony(
        &mut self,
        spec: &PeonySpec,
        leaf_pal: &Palette,
        global_gust: f32,
        rise: f32,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let t = self.time;
        let wind = self.wind_strength;

        let local_sway = (global_gust
            + wind * 0.30 * (t * 1.02 + spec.sway_phase).sin()
            + wind * 0.12 * (t * 1.94 + spec.sway_phase).cos())
            * spec.sway_amp
            + self.pointer_push_x * 0.40;
        let local_depth_sway = wind * 0.018 * (t * 0.88 + spec.sway_phase).cos();

        let bloom_raw = ((self.bloom_age - spec.bloom_delay) / 2.35).clamp(0.0, 1.0);
        let bloom_base = ease_out_cubic(bloom_raw);
        let breath = 0.038 * (t * 0.78 + spec.sway_phase).sin() * bloom_base;
        let bloom = (0.18 + 0.82 * bloom_base + breath).clamp(0.14, 1.06);

        let p0 = spec.base_pos;
        let p1 = Vec3::new(
            spec.waist_pos.x + local_sway * 0.14,
            spec.waist_pos.y,
            spec.waist_pos.z,
        );
        let p3 = Vec3::new(
            spec.head_pos.x + local_sway,
            spec.head_pos.y - local_sway.abs() * 0.12 - (1.0 - rise) * 0.22,
            spec.head_pos.z + local_depth_sway,
        );
        let p2 = Vec3::new(
            spec.waist_pos.x * 0.35 + p3.x * 0.65 + local_sway * 0.22,
            spec.waist_pos.y * 0.35 + p3.y * 0.65,
            spec.waist_pos.z * 0.35 + p3.z * 0.65,
        );

        let stem_top = p3.add(Vec3::new(0.0, -0.05, -0.06));
        self.render_stem_tube(
            p0,
            p1,
            p2,
            stem_top,
            spec.stem_radius,
            true,
            leaf_pal,
            spec.id_base + 140,
            key_light,
            fill_light,
        );

        let roll_dyn = spec.roll - local_sway * 1.15;
        let pitch_dyn = spec.pitch + local_depth_sway * 1.2;
        let yaw_dyn = spec.yaw + local_sway * 0.32;
        let r_scale = spec.radius * (0.68 + 0.32 * rise);

        // 4 concentric ruffled peony petal rings
        let rings: [(usize, f32, f32, f32, f32, f32, f32, f32); 4] = [
            (7, 1.24, 1.04, 0.02, 0.58, 0.15, 0.022, 0.94),
            (7, 0.92, 0.84, 0.09, 0.56, 0.58, 0.024, 1.04),
            (6, 0.60, 0.62, 0.15, 0.60, 0.28, 0.020, 1.10),
            (5, 0.32, 0.40, 0.20, 0.66, 0.72, 0.014, 1.14),
        ];

        let mut pid = spec.id_base + 1;
        for (ring_idx, &(count, tilt_full, span_mult, y_off, half_ang, phase_off, ruffle_amp, ring_ao)) in
            rings.iter().enumerate()
        {
            let open_tilt = 0.22 + (tilt_full - 0.22) * bloom;
            let span = r_scale * span_mult;
            let y_base = r_scale * y_off;
            let u_steps = if ring_idx < 2 { 24 } else { 20 };
            let v_steps = if ring_idx < 2 { 26 } else { 22 };
            let du = 0.032;
            let dv = 0.028;

            for k in 0..count {
                let azim = phase_off + (k as f32 / count as f32) * PI * 2.0;
                let (s_az, c_az) = azim.sin_cos();
                let dome_n_loc = Vec3::new(s_az * 0.46, 0.74, c_az * 0.46).normalize();
                let ruffle_phase = (k as f32) * 1.7 + (ring_idx as f32) * 0.9;
                let petal_id = pid;
                pid += 1;

                for iv in 0..=v_steps {
                    let v = iv as f32 / v_steps as f32;
                    let r0 = precompute_peony_row(v, open_tilt, span, y_base, half_ang, r_scale);
                    let r1 = precompute_peony_row(
                        (v + dv).min(1.0),
                        open_tilt,
                        span,
                        y_base,
                        half_ang,
                        r_scale,
                    );
                    let cup_grad = 0.48 + 0.64 * v.powf(0.75);

                    for iu in 0..=u_steps {
                        let u = -1.0 + 2.0 * (iu as f32 / u_steps as f32);
                        let p_loc = eval_peony_from_row(u, azim, &r0, ruffle_amp, ruffle_phase);
                        let pu = eval_peony_from_row((u + du).min(1.0), azim, &r0, ruffle_amp, ruffle_phase);
                        let pv = eval_peony_from_row(u, azim, &r1, ruffle_amp, ruffle_phase);

                        let geom_n = pu.sub(p_loc).cross(pv.sub(p_loc)).normalize();
                        let n_loc = geom_n.scale(0.55).add(dome_n_loc.scale(0.45)).normalize();

                        let p_world = p_loc
                            .rot_y(yaw_dyn)
                            .rot_x(pitch_dyn)
                            .rot_z(roll_dyn)
                            .add(p3);
                        let mut n_world = n_loc
                            .rot_y(yaw_dyn)
                            .rot_x(pitch_dyn)
                            .rot_z(roll_dyn)
                            .normalize();

                        if n_world.z < 0.0 {
                            n_world = n_world.scale(-1.0);
                        }

                        let ruffle_lip = if v > 0.70 { 1.0 + (v - 0.70) * 0.95 } else { 1.0 };
                        let edge_soft = 1.0 - 0.18 * (u * u * u.abs());
                        let ao = (cup_grad * ring_ao * edge_soft * ruffle_lip).clamp(0.24, 1.32);

                        let (lum, col) = shade_surface(
                            n_world,
                            &spec.palette,
                            0.14 + 0.86 * v,
                            ao,
                            1.22,
                            key_light,
                            fill_light,
                        );
                        self.plot_3d(p_world, lum, col, petal_id, 1);
                    }
                }
            }
        }

        // Soft golden-cream stamen cushion at the peony heart
        if bloom > 0.45 {
            let core_id = spec.id_base + 130;
            let core_r = r_scale * 0.17 * bloom;
            for iv in 0..=10 {
                let v = iv as f32 / 10.0;
                let phi = v * PI * 0.5;
                let (sp, cp) = phi.sin_cos();
                for iu in 0..14 {
                    let ang = (iu as f32 / 14.0) * PI * 2.0;
                    let (sa, ca) = ang.sin_cos();
                    let p_loc = Vec3::new(
                        core_r * sp * sa,
                        r_scale * 0.19 + core_r * 0.55 * cp,
                        core_r * sp * ca,
                    );
                    let p_world = p_loc
                        .rot_y(yaw_dyn)
                        .rot_x(pitch_dyn)
                        .rot_z(roll_dyn)
                        .add(p3);
                    let lum = 0.52 + 0.40 * cp;
                    let col = spec.stamen_col.lerp(Rgb::new(255.0, 252.0, 236.0), cp * 0.45);
                    self.plot_3d(p_world, lum, col, core_id, 1);
                }
            }
        }
    }

    fn render_freesia_spike(
        &mut self,
        spec: &FreesiaSpec,
        leaf_pal: &Palette,
        global_gust: f32,
        rise: f32,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let t = self.time;
        let wind = self.wind_strength;

        let local_sway = (global_gust
            + wind * 0.44 * (t * 1.24 + spec.sway_phase).sin()
            + wind * 0.18 * (t * 2.32 + spec.sway_phase).cos())
            * spec.sway_amp
            + self.pointer_push_x * 0.52;
        let local_depth = wind * 0.025 * (t * 0.96 + spec.sway_phase).cos();

        let p0 = spec.p0;
        let p1 = Vec3::new(spec.p1.x + local_sway * 0.18, spec.p1.y, spec.p1.z);
        let p2 = Vec3::new(
            spec.p2.x + local_sway * 0.65,
            spec.p2.y - (1.0 - rise) * 0.18,
            spec.p2.z + local_depth * 0.5,
        );
        let p3 = Vec3::new(
            spec.p3.x + local_sway * 1.18,
            spec.p3.y - local_sway.abs() * 0.16 - (1.0 - rise) * 0.28,
            spec.p3.z + local_depth,
        );

        self.render_stem_tube(
            p0,
            p1,
            p2,
            p3,
            spec.stem_radius,
            false,
            leaf_pal,
            spec.id_base + 95,
            key_light,
            fill_light,
        );

        let bloom_raw = ((self.bloom_age - spec.bloom_delay) / 2.2).clamp(0.0, 1.0);
        let bloom_base = ease_out_cubic(bloom_raw);
        let breath = 0.04 * (t * 0.92 + spec.sway_phase).sin() * bloom_base;

        // 5 graduated florets arranged along the arching comb, tapering into a tip bud
        let florets: [(f32, f32, f32, f32); 5] = [
            (0.56, 1.06, 1.00, 0.16),
            (0.67, 0.94, 0.92, 0.30),
            (0.78, 0.80, 0.74, 0.45),
            (0.88, 0.66, 0.40, 0.64),
            (0.98, 0.54, 0.08, 0.86),
        ];

        for (idx, &(s, size_m, open_m, tilt_out)) in florets.iter().enumerate() {
            let anchor = bezier3(p0, p1, p2, p3, s).add(Vec3::new(0.0, 0.012, 0.018));
            let scale = spec.floret_scale * size_m * (0.65 + 0.35 * rise);
            let openness = (open_m * (0.18 + 0.82 * bloom_base) + breath * open_m).clamp(0.05, 1.05);

            let roll = -spec.lean_x * tilt_out - local_sway * 1.1;
            let pitch = 0.44 + 0.05 * (idx as f32);
            let yaw = -spec.lean_x * 0.22;

            self.render_freesia_floret(
                anchor,
                scale,
                openness,
                roll,
                pitch,
                yaw,
                &spec.palette,
                spec.throat_col,
                spec.id_base + (idx as u16) * 14 + 1,
                key_light,
                fill_light,
            );
        }
    }

    fn render_freesia_floret(
        &mut self,
        origin: Vec3,
        scale: f32,
        openness: f32,
        roll: f32,
        pitch: f32,
        yaw: f32,
        palette: &Palette,
        throat_col: Rgb,
        id_base: u16,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let len_total = 0.165 * scale;
        let max_r = 0.088 * scale;

        let u_steps = 12usize;
        let v_steps = 20usize;
        let du = 0.04;
        let dv = 0.035;

        for k in 0..6 {
            let azim = (k as f32) * (PI / 3.0) + 0.26;
            let petal_id = id_base + k as u16;

            for iv in 0..=v_steps {
                let v = iv as f32 / v_steps as f32;
                let r0 = precompute_freesia_row(v, openness, max_r, len_total);
                let r1 = precompute_freesia_row((v + dv).min(1.0), openness, max_r, len_total);

                for iu in 0..=u_steps {
                    let u = -1.0 + 2.0 * (iu as f32 / u_steps as f32);
                    let p_loc = eval_freesia_from_row(u, azim, &r0);
                    let pu = eval_freesia_from_row((u + du).min(1.0), azim, &r0);
                    let pv = eval_freesia_from_row(u, azim, &r1);

                    let n_loc = pu.sub(p_loc).cross(pv.sub(p_loc)).normalize();

                    let p_world = p_loc
                        .rot_y(yaw)
                        .rot_x(pitch)
                        .rot_z(roll)
                        .add(origin);
                    let mut n_world = n_loc
                        .rot_y(yaw)
                        .rot_x(pitch)
                        .rot_z(roll)
                        .normalize();

                    if n_world.z < 0.0 {
                        n_world = n_world.scale(-1.0);
                    }

                    let ao = (0.84 + 0.32 * v.powf(0.5)) * (1.0 - 0.14 * u * u);
                    let (lum, mut col) = shade_surface(
                        n_world,
                        palette,
                        v,
                        ao,
                        1.22,
                        key_light,
                        fill_light,
                    );
                    if v < 0.36 && openness > 0.35 {
                        let throat_k = ((0.36 - v) / 0.36) * 0.42 * openness;
                        col = col.lerp(throat_col, throat_k);
                    }
                    self.plot_3d(p_world, lum, col, petal_id, 1);
                }
            }
        }
    }

    fn render_leaf(
        &mut self,
        leaf: &LeafSpec,
        leaf_pal: &Palette,
        global_gust: f32,
        rise: f32,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let t = self.time;
        let wind = self.wind_strength;
        let flutter = (global_gust
            + wind * 0.45 * (t * 1.48 + leaf.sway_phase).sin()
            + wind * 0.18 * (t * 2.65 + leaf.sway_phase).cos())
            * leaf.sway_amp
            + self.pointer_push_x * 0.32;

        let p0 = leaf.p0;
        let p1 = Vec3::new(leaf.p1.x + flutter * 0.25, leaf.p1.y, leaf.p1.z);
        let p2 = Vec3::new(
            leaf.p2.x + flutter * 0.72,
            leaf.p2.y - (1.0 - rise) * 0.12,
            leaf.p2.z + flutter * 0.15,
        );
        let p3 = Vec3::new(
            leaf.p3.x + flutter * 1.25,
            leaf.p3.y - flutter.abs() * 0.12 - (1.0 - rise) * 0.20,
            leaf.p3.z + flutter * 0.28,
        );

        let v_steps = 72usize;
        let u_steps = 22usize;

        for iv in 0..=v_steps {
            let v = iv as f32 / v_steps as f32;
            let center = bezier3(p0, p1, p2, p3, v);
            let tan = bezier3_tangent(p0, p1, p2, p3, v).normalize();

            let width_profile = (v.powf(0.42) * (1.0 - v).powf(0.76) * 2.38).clamp(0.0, 1.0);
            let half_w = leaf.max_width * width_profile * (0.75 + 0.25 * rise);

            let twist = leaf.twist_start + (leaf.twist_end - leaf.twist_start) * v;
            let base_side = tan.cross(Vec3::new(0.0, 0.0, 1.0)).normalize();
            let base_norm = base_side.cross(tan).normalize();

            let (st, ct) = twist.sin_cos();
            let side = base_side.scale(ct).add(base_norm.scale(st)).normalize();
            let up_n = base_norm.scale(ct).sub(base_side.scale(st)).normalize();

            for iu in 0..=u_steps {
                let u = -1.0 + 2.0 * (iu as f32 / u_steps as f32);
                let fold = leaf.fold_depth * (1.0 - u.abs()) * width_profile;
                let pos = center.add(side.scale(u * half_w)).add(up_n.scale(-fold));

                let slope = if u >= 0.0 { 0.42 } else { -0.42 };
                let mut n = up_n.add(side.scale(slope)).normalize();
                if n.z < 0.0 {
                    n = n.scale(-1.0);
                }

                let base_ao = 0.54 + 0.46 * v.powf(0.45);
                let rib_ao = 0.86 + 0.14 * u.abs();
                let (lum, col) = shade_surface(
                    n,
                    leaf_pal,
                    0.18 + 0.68 * v,
                    base_ao * rib_ao,
                    0.70,
                    key_light,
                    fill_light,
                );
                self.plot_3d(pos, lum, col, leaf.id, 3);
            }
        }
    }

    #[inline]
    fn plot_3d(&mut self, world: Vec3, lum: f32, col: Rgb, surface_id: u16, kind: u8) {
        let cam_z = 3.55;
        let dz = cam_z - world.z;
        if dz < 0.4 {
            return;
        }
        let persp = cam_z / dz;

        let sw = (self.cols * 2) as f32;
        let sh = (self.rows * 2) as f32;

        // Fit both landscape desktop and portrait mobile screens without clipping or distortion
        let scale_y_by_h = sh * 0.445;
        let max_scale_x = (sw * 0.44) / 1.12;
        let scale_y_by_w = max_scale_x * self.char_aspect;
        let scale_y = scale_y_by_h.min(scale_y_by_w);
        let scale_x = scale_y / self.char_aspect;

        // Slight upward center lift on tall portrait screens so the bouquet rests gracefully above mobile controls
        let is_portrait = scale_y_by_w < scale_y_by_h * 0.92;
        let center_y_ratio = if is_portrait { 0.495 } else { 0.515 };

        let sx = (sw * 0.5 + world.x * persp * scale_x) as i32;
        let sy = (sh * center_y_ratio - world.y * persp * scale_y) as i32;
        let sw_i = (self.cols * 2) as i32;
        let sh_i = (self.rows * 2) as i32;

        if sx < 0 || sy < 0 || sx >= sw_i || sy >= sh_i {
            return;
        }

        let idx = (sy as usize) * (self.cols * 2) + (sx as usize);
        if world.z > self.sub_buf[idx].z {
            let (r, g, b) = col.to_u8();
            self.sub_buf[idx] = SubPixel {
                z: world.z,
                lum: lum.clamp(0.0, 1.0),
                r,
                g,
                b,
                surface_id,
                kind,
            };
        }
    }

    fn resolve_grid(&mut self) {
        let cols = self.cols;
        let rows = self.rows;
        let sw = cols * 2;

        // 2x2 supersampled downsample onto the regular monospace grid
        for row in 0..rows {
            for col in 0..cols {
                let samples = [
                    self.sub_buf[(row * 2) * sw + (col * 2)],
                    self.sub_buf[(row * 2) * sw + (col * 2 + 1)],
                    self.sub_buf[(row * 2 + 1) * sw + (col * 2)],
                    self.sub_buf[(row * 2 + 1) * sw + (col * 2 + 1)],
                ];

                let mut best_z = -1e9_f32;
                let mut best_id = 0u16;
                let mut best_kind = 0u8;
                let mut total_cov = 0u8;

                for s in &samples {
                    if s.kind != 0 {
                        total_cov += 1;
                        if s.z > best_z {
                            best_z = s.z;
                            best_id = s.surface_id;
                            best_kind = s.kind;
                        }
                    }
                }

                let idx = row * cols + col;
                if total_cov == 0 {
                    self.cell_buf[idx] = CellSummary::EMPTY;
                    continue;
                }

                let mut lum_sum = 0.0_f32;
                let mut r_sum = 0.0_f32;
                let mut g_sum = 0.0_f32;
                let mut b_sum = 0.0_f32;
                let mut count = 0.0_f32;

                for s in &samples {
                    if s.kind != 0 && (s.surface_id == best_id || (best_z - s.z) < 0.045) {
                        lum_sum += s.lum;
                        r_sum += s.r as f32;
                        g_sum += s.g as f32;
                        b_sum += s.b as f32;
                        count += 1.0;
                    }
                }

                let inv = 1.0 / count.max(1.0);
                self.cell_buf[idx] = CellSummary {
                    z: best_z,
                    lum: lum_sum * inv,
                    r: r_sum * inv,
                    g: g_sum * inv,
                    b: b_sum * inv,
                    surface_id: best_id,
                    kind: best_kind,
                    coverage: total_cov,
                };
            }
        }

        // Resolve final ASCII glyph ramp (@ # % * + = : ;) with petal-overlap crease shading
        for row in 0..rows {
            for col in 0..cols {
                let idx = row * cols + col;
                let out_i = idx * 4;
                let cell = self.cell_buf[idx];

                if cell.kind == 0 {
                    self.out_buf[out_i] = b' ';
                    self.out_buf[out_i + 1] = 0;
                    self.out_buf[out_i + 2] = 0;
                    self.out_buf[out_i + 3] = 0;
                    continue;
                }

                let mut lum = cell.lum;
                let mut r = cell.r;
                let mut g = cell.g;
                let mut b = cell.b;

                let mut occluded_by_front = false;
                let mut is_silhouette_edge = false;
                let neighbors: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                for &(dx, dy) in &neighbors {
                    let nx = col as i32 + dx;
                    let ny = row as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= cols as i32 || ny >= rows as i32 {
                        continue;
                    }
                    let nb = self.cell_buf[ny as usize * cols + nx as usize];
                    if nb.kind == 0 {
                        is_silhouette_edge = true;
                    } else if nb.surface_id != cell.surface_id && (nb.z - cell.z) > 0.024 {
                        occluded_by_front = true;
                    }
                }

                if occluded_by_front {
                    lum *= 0.62;
                    r *= 0.78;
                    g *= 0.78;
                    b *= 0.78;
                }

                if is_silhouette_edge && cell.coverage < 4 {
                    let edge_scale = match cell.coverage {
                        1 => 0.36,
                        2 => 0.58,
                        3 => 0.82,
                        _ => 1.0,
                    };
                    lum *= edge_scale;
                }

                let shaped_lum = lum.clamp(0.0, 1.0).powf(1.22);
                let ramp_f = shaped_lum.clamp(0.0, 0.999) * (DENSITY_RAMP.len() as f32);
                let ramp_idx = (ramp_f as usize).min(DENSITY_RAMP.len() - 1);

                self.out_buf[out_i] = DENSITY_RAMP[ramp_idx];
                self.out_buf[out_i + 1] = r.clamp(0.0, 255.0) as u8;
                self.out_buf[out_i + 2] = g.clamp(0.0, 255.0) as u8;
                self.out_buf[out_i + 3] = b.clamp(0.0, 255.0) as u8;
            }
        }

        // Minimalist scattered white starfield and drifting dust particles on the black void
        let t = self.time;
        let wind_drift = self.wind_strength * 0.004;
        for star in &self.stars {
            let fx = (star.nx
                + t * (star.drift_x + if star.drift_y > 0.0 { wind_drift } else { 0.0 }))
            .rem_euclid(1.0);
            let fy = (star.ny - t * star.drift_y).rem_euclid(1.0);

            let col = ((fx * cols as f32) as usize).min(cols - 1);
            let row = ((fy * rows as f32) as usize).min(rows - 1);
            let idx = row * cols + col;

            if self.cell_buf[idx].kind != 0 {
                continue;
            }

            let twinkle = 0.55 + 0.45 * (t * star.speed + star.phase).sin();
            let intensity = (star.base_lum * twinkle).clamp(0.18, 1.0);
            let out_i = idx * 4;
            self.out_buf[out_i] = b'.';
            self.out_buf[out_i + 1] = (intensity * 236.0) as u8;
            self.out_buf[out_i + 2] = (intensity * 240.0) as u8;
            self.out_buf[out_i + 3] = (intensity * 248.0) as u8;
        }
    }
}

#[derive(Clone, Copy)]
struct TulipRow {
    half_ang: f32,
    base_r: f32,
    spoon_amp: f32,
    rib_amp: f32,
    y: f32,
}

#[inline]
fn precompute_tulip_row(
    v: f32,
    is_outer: bool,
    bloom: f32,
    radius_scale: f32,
    height_scale: f32,
) -> TulipRow {
    let width_env = (v.max(1e-4).powf(0.40) * (1.0 - v).max(1e-4).powf(0.68) * 2.30).clamp(0.0, 1.0);
    let max_half_ang = if is_outer { 0.72 } else { 0.68 };
    let half_ang = max_half_ang * width_env;

    let belly = (v * PI * 0.88).sin().max(0.0).powf(0.56);
    let open_bell = 0.54 + 0.46 * bloom;
    let tip_flare = if is_outer {
        0.24 * bloom * v.powf(2.1)
    } else {
        0.10 * bloom * v.powf(2.4)
    };
    let whorl_offset = if is_outer {
        0.026 * (1.0 - 0.30 * v)
    } else {
        -0.014
    };
    let base_r = radius_scale * (0.10 + open_bell * 0.74 * belly + tip_flare) + whorl_offset;

    let sin_vpi = (v * PI).sin();
    let spoon_amp = 0.072 * radius_scale * sin_vpi;
    let rib_amp = 0.011 * radius_scale * sin_vpi;

    let h_mult = if is_outer { 0.90 } else { 1.12 };
    let tip_droop = if is_outer { 0.055 } else { 0.018 } * bloom * (v * v * v);
    let y = height_scale * h_mult * (v.powf(0.86) - tip_droop);

    TulipRow {
        half_ang,
        base_r,
        spoon_amp,
        rib_amp,
        y,
    }
}

#[inline]
fn eval_tulip_from_row(u: f32, azim: f32, row: &TulipRow) -> Vec3 {
    let ang = azim + u * row.half_ang;
    let spoon = (1.0 - u * u) * row.spoon_amp;
    let rib = row.rib_amp * (u * PI * 1.5).cos();
    let r = (row.base_r + spoon + rib).max(0.01);
    let (sa, ca) = ang.sin_cos();
    Vec3::new(r * sa, row.y, r * ca)
}

#[derive(Clone, Copy)]
struct PeonyRow {
    half_ang: f32,
    base_r: f32,
    y: f32,
    spoon_amp: f32,
    ruffle_weight: f32,
}

#[inline]
fn precompute_peony_row(
    v: f32,
    tilt: f32,
    span: f32,
    y_base: f32,
    max_half_ang: f32,
    r_scale: f32,
) -> PeonyRow {
    let width_env = (v.max(1e-4).powf(0.36) * (1.0 - 0.78 * v).max(0.08).powf(0.28) * 1.38)
        .clamp(0.0, 1.0);
    let half_ang = max_half_ang * width_env;

    let (st, ct) = tilt.sin_cos();
    let arc_r = (v * PI * 0.62).sin();
    let base_r = r_scale * 0.09 + span * arc_r * st;

    let cup_lift = span * 0.24 * v * v;
    let y = y_base + span * (v * ct + 0.16 * (v * PI).sin()) + cup_lift * (1.0 - st * 0.4);

    let spoon_amp = 0.036 * r_scale * (v * PI).sin();
    let ruffle_weight = v * v;

    PeonyRow {
        half_ang,
        base_r,
        y,
        spoon_amp,
        ruffle_weight,
    }
}

#[inline]
fn eval_peony_from_row(
    u: f32,
    azim: f32,
    row: &PeonyRow,
    ruffle_amp: f32,
    ruffle_phase: f32,
) -> Vec3 {
    let ang = azim + u * row.half_ang;
    let wave = (u * PI * 2.5 + ruffle_phase).sin() * ruffle_amp * row.ruffle_weight;
    let spoon = (1.0 - u * u) * row.spoon_amp;
    let r = (row.base_r + spoon + wave * 0.55).max(0.008);
    let y = row.y + wave;
    let (sa, ca) = ang.sin_cos();
    Vec3::new(r * sa, y, r * ca)
}

#[derive(Clone, Copy)]
struct FreesiaRow {
    half_ang: f32,
    base_r: f32,
    y: f32,
    spoon_amp: f32,
}

#[inline]
fn precompute_freesia_row(v: f32, openness: f32, max_r: f32, len_total: f32) -> FreesiaRow {
    let lobe_width = if v < 0.36 {
        0.54
    } else {
        let t = (v - 0.36) / 0.64;
        (0.56 * (1.0 - t).max(1e-4).powf(0.62) * (0.45 + 0.75 * (t * PI).sin())).clamp(0.03, 0.58)
    };
    let half_ang = lobe_width * (0.72 + 0.28 * openness);

    let tube_r = max_r * (0.14 + 0.22 * v);
    let flare = if v > 0.34 {
        let t = (v - 0.34) / 0.66;
        max_r * (0.12 + 0.92 * openness) * t.powf(1.45)
    } else {
        0.0
    };
    let bud_taper = if openness < 0.35 && v > 0.55 {
        let t = (v - 0.55) / 0.45;
        1.0 - (1.0 - openness / 0.35) * 0.72 * t * t
    } else {
        1.0
    };
    let base_r = (tube_r + flare) * bud_taper;

    let tip_reflex = 0.20 * openness * v * v * v;
    let y = len_total * (v - tip_reflex);
    let spoon_amp = max_r * 0.08 * (v * PI).sin();

    FreesiaRow {
        half_ang,
        base_r,
        y,
        spoon_amp,
    }
}

#[inline]
fn eval_freesia_from_row(u: f32, azim: f32, row: &FreesiaRow) -> Vec3 {
    let ang = azim + u * row.half_ang;
    let r = (row.base_r + (1.0 - u * u) * row.spoon_amp).max(0.005);
    let (sa, ca) = ang.sin_cos();
    Vec3::new(r * sa, row.y, r * ca)
}

#[inline]
fn shade_surface(
    normal: Vec3,
    palette: &Palette,
    v_cup: f32,
    ao: f32,
    spec_strength: f32,
    key_light: Vec3,
    fill_light: Vec3,
) -> (f32, Rgb) {
    let view = Vec3::new(0.0, 0.0, 1.0);

    let ndl = normal.dot(key_light);
    let wrap_diff = ((ndl + 0.38) / 1.38).clamp(0.0, 1.0);
    let direct_diff = ndl.max(0.0);
    let key = 0.58 * direct_diff + 0.42 * wrap_diff;

    let fill = normal.dot(fill_light).max(0.0) * 0.26;

    let half_vec = key_light.add(view).normalize();
    let spec = normal.dot(half_vec).max(0.0).powf(8.5) * spec_strength;

    let ndv = normal.dot(view).abs().clamp(0.0, 1.0);
    let rim = (1.0 - ndv).powf(2.1) * (0.32 + 0.68 * normal.y.max(0.0));

    let base_shade = (0.15 + key * 0.67 + fill + rim * 0.24) * ao;
    let lum = (base_shade + spec * 0.50).clamp(0.08, 1.0);
    let col = palette.sample(base_shade, v_cup, spec, rim);

    (lum, col)
}

#[inline]
fn bezier3(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let u = 1.0 - t;
    let u2 = u * u;
    let t2 = t * t;
    p0.scale(u2 * u)
        .add(p1.scale(3.0 * u2 * t))
        .add(p2.scale(3.0 * u * t2))
        .add(p3.scale(t2 * t))
}

#[inline]
fn bezier3_tangent(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let u = 1.0 - t;
    p1.sub(p0)
        .scale(3.0 * u * u)
        .add(p2.sub(p1).scale(6.0 * u * t))
        .add(p3.sub(p2).scale(3.0 * t * t))
}

#[inline]
fn ease_out_cubic(x: f32) -> f32 {
    let inv = 1.0 - x.clamp(0.0, 1.0);
    1.0 - inv * inv * inv
}

// ============================================================================
// WebAssembly FFI exports (zero external dependencies, static GitHub Pages)
// ============================================================================

static mut ENGINE: Option<BouquetRenderer> = None;

#[allow(static_mut_refs)]
fn engine_mut() -> &'static mut BouquetRenderer {
    unsafe { ENGINE.get_or_insert_with(|| BouquetRenderer::new(118, 58)) }
}

#[no_mangle]
pub extern "C" fn init(cols: u32, rows: u32) {
    engine_mut().resize(cols as usize, rows as usize);
}

#[no_mangle]
pub extern "C" fn set_aspect(char_aspect: f32) {
    engine_mut().set_char_aspect(char_aspect);
}

#[no_mangle]
pub extern "C" fn set_wind(strength: f32) {
    engine_mut().set_wind(strength);
}

#[no_mangle]
pub extern "C" fn set_pointer(nx: f32, ny: f32, active: u32) {
    engine_mut().set_pointer(nx, ny, active != 0);
}

#[no_mangle]
pub extern "C" fn trigger_bloom() {
    engine_mut().trigger_bloom();
}

#[no_mangle]
pub extern "C" fn set_bloom_instant() {
    engine_mut().set_bloom_instant();
}

#[no_mangle]
pub extern "C" fn render(dt: f32) {
    engine_mut().render(dt);
}

#[no_mangle]
pub extern "C" fn cols() -> u32 {
    engine_mut().cols() as u32
}

#[no_mangle]
pub extern "C" fn rows() -> u32 {
    engine_mut().rows() as u32
}

#[no_mangle]
pub extern "C" fn buffer_ptr() -> *const u8 {
    engine_mut().out_bytes().as_ptr()
}

#[no_mangle]
pub extern "C" fn buffer_len() -> u32 {
    engine_mut().out_bytes().len() as u32
}

#[no_mangle]
pub extern "C" fn copy_ascii_ptr() -> *const u8 {
    engine_mut().build_ascii_text().as_ptr()
}

#[no_mangle]
pub extern "C" fn copy_ascii_len() -> u32 {
    engine_mut().ascii_text.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_pastel_garden_bouquet_with_exact_ascii_ramp() {
        let mut renderer = BouquetRenderer::new(118, 56);
        renderer.set_bloom_instant();
        renderer.render(0.016);

        let mut glyph_counts = [0usize; 8];
        let mut stars = 0usize;
        let mut bouquet_cells = 0usize;

        for chunk in renderer.out_bytes().chunks_exact(4) {
            let g = chunk[0];
            if g == b' ' {
                continue;
            }
            if g == b'.' {
                stars += 1;
                continue;
            }
            let pos = DENSITY_RAMP.iter().position(|&c| c == g);
            assert!(pos.is_some(), "Unexpected character outside ASCII ramp: {:?}", g as char);
            glyph_counts[pos.unwrap()] += 1;
            bouquet_cells += 1;
        }

        for (i, &cnt) in glyph_counts.iter().enumerate() {
            assert!(cnt > 10, "Symbol '{}' underrepresented ({})", DENSITY_RAMP[i] as char, cnt);
        }
        assert!(stars > 40, "Expected starry background dots ({})", stars);
        assert!(bouquet_cells > 1200, "Expected rich pastel bouquet ({})", bouquet_cells);
    }

    #[test]
    fn fits_cleanly_within_portrait_phone_grid_without_horizontal_clipping() {
        let mut renderer = BouquetRenderer::new(72, 68);
        renderer.set_char_aspect(0.55);
        renderer.set_bloom_instant();
        renderer.render(0.016);

        let cols = renderer.cols();
        let rows = renderer.rows();
        let bytes = renderer.out_bytes();
        let mut bouquet_cells = 0usize;
        let mut left_edge_bouquet = 0usize;
        let mut right_edge_bouquet = 0usize;

        for row in 0..rows {
            for col in 0..cols {
                let g = bytes[(row * cols + col) * 4];
                if g != b' ' && g != b'.' {
                    bouquet_cells += 1;
                    if col == 0 {
                        left_edge_bouquet += 1;
                    }
                    if col + 1 == cols {
                        right_edge_bouquet += 1;
                    }
                }
            }
        }

        assert!(bouquet_cells > 650, "Portrait mobile bouquet too sparse ({})", bouquet_cells);
        assert_eq!(left_edge_bouquet, 0, "Bouquet clipped on left phone edge");
        assert_eq!(right_edge_bouquet, 0, "Bouquet clipped on right phone edge");
    }
}
