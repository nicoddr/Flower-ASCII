use core::f32::consts::PI;

/// Exact ASCII character ramp ordered by optical density on a solid black background:
/// sparse shadows and delicate contours (';', ':') up to luminous crests ('#', '@').
pub const DENSITY_RAMP: [u8; 8] = [b';', b':', b'=', b'+', b'*', b'%', b'#', b'@'];

const MAX_COLS: usize = 280;
const MAX_ROWS: usize = 180;

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
}

/// Precomputed 3×3 rotation matrix (rot_z(roll) * rot_x(pitch) * rot_y(yaw))
/// Computed once per flower to eliminate thousands of trig calls per frame.
#[derive(Clone, Copy)]
struct Mat3 {
    c0: Vec3,
    c1: Vec3,
    c2: Vec3,
}

impl Mat3 {
    #[inline]
    fn from_ypr(yaw: f32, pitch: f32, roll: f32) -> Self {
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let (sr, cr) = roll.sin_cos();

        // Applies rot_y(yaw), then rot_x(pitch), then rot_z(roll)
        let r00 = cr * cy - sr * sp * sy;
        let r01 = -sr * cp;
        let r02 = cr * sy + sr * sp * cy;

        let r10 = sr * cy + cr * sp * sy;
        let r11 = cr * cp;
        let r12 = sr * sy - cr * sp * cy;

        let r20 = -cp * sy;
        let r21 = sp;
        let r22 = cp * cy;

        Self {
            c0: Vec3::new(r00, r10, r20),
            c1: Vec3::new(r01, r11, r21),
            c2: Vec3::new(r02, r12, r22),
        }
    }

    #[inline]
    fn mul_vec(self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.c0.x * v.x + self.c1.x * v.y + self.c2.x * v.z,
            self.c0.y * v.x + self.c1.y * v.y + self.c2.y * v.z,
            self.c0.z * v.x + self.c1.z * v.y + self.c2.z * v.z,
        )
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
    #[inline]
    fn sample(&self, shade: f32, v_cup: f32, spec: f32, rim: f32, picotee: f32) -> Rgb {
        let base_mix = ((0.28 - v_cup) / 0.28).clamp(0.0, 1.0) * 0.50;
        let t = shade.clamp(0.0, 1.0);

        let body = if t < 0.48 {
            self.shadow.lerp(self.mid, t / 0.48)
        } else {
            self.mid.lerp(self.bright, (t - 0.48) / 0.52)
        };

        let mut col = body.lerp(self.base_cup, base_mix);

        // Delicate pale edge tint along petal tips (natural pastel botanical picotee)
        if picotee > 0.01 {
            col = col.lerp(self.bright.lerp(self.highlight, 0.55), picotee.clamp(0.0, 0.65));
        }

        // Crisp white / light-gray luminous highlights on specular crests and rim curves
        let light_touch = (spec * 1.15 + rim * 0.42).clamp(0.0, 1.0);
        if light_touch > 0.20 {
            let k = ((light_touch - 0.20) / 0.80).powf(0.92);
            col = col.lerp(self.highlight, (k * 0.95).clamp(0.0, 0.95));
        }
        col
    }
}

// --- Fine-Tuned Pastel Botanical Palettes (solid #000000 terminal canvas) ---

const PAL_ROSE_PEONY: Palette = Palette {
    base_cup: Rgb::new(164.0, 58.0, 98.0),
    shadow: Rgb::new(202.0, 94.0, 136.0),
    mid: Rgb::new(250.0, 152.0, 192.0),
    bright: Rgb::new(255.0, 208.0, 228.0),
    highlight: Rgb::new(255.0, 250.0, 253.0),
};

const PAL_PEARL_PEONY: Palette = Palette {
    base_cup: Rgb::new(152.0, 136.0, 148.0),
    shadow: Rgb::new(186.0, 174.0, 186.0),
    mid: Rgb::new(236.0, 226.0, 236.0),
    bright: Rgb::new(252.0, 246.0, 252.0),
    highlight: Rgb::new(255.0, 255.0, 255.0),
};

const PAL_CORAL_BUD: Palette = Palette {
    base_cup: Rgb::new(178.0, 76.0, 94.0),
    shadow: Rgb::new(216.0, 112.0, 130.0),
    mid: Rgb::new(255.0, 166.0, 182.0),
    bright: Rgb::new(255.0, 216.0, 224.0),
    highlight: Rgb::new(255.0, 248.0, 250.0),
};

const PAL_PEACH_TULIP: Palette = Palette {
    base_cup: Rgb::new(184.0, 84.0, 62.0),
    shadow: Rgb::new(222.0, 120.0, 90.0),
    mid: Rgb::new(255.0, 170.0, 134.0),
    bright: Rgb::new(255.0, 220.0, 194.0),
    highlight: Rgb::new(255.0, 250.0, 244.0),
};

const PAL_LILAC_TULIP: Palette = Palette {
    base_cup: Rgb::new(112.0, 72.0, 174.0),
    shadow: Rgb::new(146.0, 104.0, 208.0),
    mid: Rgb::new(200.0, 154.0, 252.0),
    bright: Rgb::new(236.0, 212.0, 255.0),
    highlight: Rgb::new(250.0, 246.0, 255.0),
};

const PAL_BUTTER_TULIP: Palette = Palette {
    base_cup: Rgb::new(180.0, 132.0, 38.0),
    shadow: Rgb::new(214.0, 170.0, 64.0),
    mid: Rgb::new(254.0, 220.0, 108.0),
    bright: Rgb::new(255.0, 244.0, 180.0),
    highlight: Rgb::new(255.0, 253.0, 244.0),
};

const PAL_BUTTER_FREESIA: Palette = Palette {
    base_cup: Rgb::new(188.0, 152.0, 44.0),
    shadow: Rgb::new(220.0, 186.0, 76.0),
    mid: Rgb::new(255.0, 232.0, 120.0),
    bright: Rgb::new(255.0, 248.0, 192.0),
    highlight: Rgb::new(255.0, 254.0, 246.0),
};

const PAL_PEARL_FREESIA: Palette = Palette {
    base_cup: Rgb::new(144.0, 150.0, 166.0),
    shadow: Rgb::new(182.0, 188.0, 204.0),
    mid: Rgb::new(234.0, 238.0, 248.0),
    bright: Rgb::new(250.0, 252.0, 255.0),
    highlight: Rgb::new(255.0, 255.0, 255.0),
};

const PAL_LILAC_FREESIA: Palette = Palette {
    base_cup: Rgb::new(134.0, 104.0, 188.0),
    shadow: Rgb::new(168.0, 138.0, 222.0),
    mid: Rgb::new(216.0, 188.0, 255.0),
    bright: Rgb::new(244.0, 232.0, 255.0),
    highlight: Rgb::new(253.0, 250.0, 255.0),
};

const PAL_LEAF_SAGE: Palette = Palette {
    base_cup: Rgb::new(22.0, 92.0, 54.0),
    shadow: Rgb::new(34.0, 136.0, 76.0),
    mid: Rgb::new(70.0, 208.0, 122.0),
    bright: Rgb::new(148.0, 242.0, 178.0),
    highlight: Rgb::new(230.0, 248.0, 236.0),
};

const PAL_LEAF_DEEP: Palette = Palette {
    base_cup: Rgb::new(16.0, 74.0, 44.0),
    shadow: Rgb::new(26.0, 114.0, 66.0),
    mid: Rgb::new(50.0, 178.0, 104.0),
    bright: Rgb::new(114.0, 226.0, 152.0),
    highlight: Rgb::new(214.0, 242.0, 224.0),
};

const PAL_RIBBON_SILK: Palette = Palette {
    base_cup: Rgb::new(148.0, 134.0, 126.0),
    shadow: Rgb::new(182.0, 168.0, 160.0),
    mid: Rgb::new(228.0, 218.0, 212.0),
    bright: Rgb::new(250.0, 244.0, 240.0),
    highlight: Rgb::new(255.0, 255.0, 252.0),
};

const PASTEL_STAMEN: Rgb = Rgb::new(255.0, 222.0, 108.0);
const TULIP_ANTHER: Rgb = Rgb::new(196.0, 146.0, 78.0);

#[derive(Clone, Copy)]
struct SubPixel {
    z: f32,
    lum: f32,
    r: u8,
    g: u8,
    b: u8,
    surface_id: u16,
    kind: u8, // 0 = empty, 1 = flower, 2 = stem, 3 = leaf, 4 = accent/ribbon
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
struct PollenMote {
    origin_x: f32,
    origin_y: f32,
    speed_y: f32,
    sway_freq: f32,
    sway_amp: f32,
    phase: f32,
    tint: Rgb,
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
    stem_delay: f32,
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
    stem_delay: f32,
    bloom_delay: f32,
    is_bud: bool,
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
    floret_count: usize,
    lean_x: f32,
    sway_phase: f32,
    sway_amp: f32,
    stem_delay: f32,
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
    grow_delay: f32,
    deep_green: bool,
    id: u16,
}

#[derive(Clone, Copy)]
struct GypsoSprig {
    base: Vec3,
    mid: Vec3,
    tip: Vec3,
    sway_phase: f32,
    grow_delay: f32,
    id_base: u16,
}

/// Real-time high-detail 3D pastel garden bouquet renderer projecting onto a colorized ASCII grid.
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

    // Precomputed per-frame shimmer wave parameters during the bloom finale
    shimmer_x: f32,
    shimmer_strength: f32,

    sub_buf: Vec<SubPixel>,
    cell_buf: Vec<CellSummary>,
    out_buf: Vec<u8>,
    ascii_text: Vec<u8>,
    stars: Vec<Star>,
    pollen: Vec<PollenMote>,
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
            shimmer_x: -3.0,
            shimmer_strength: 0.0,
            sub_buf: Vec::new(),
            cell_buf: Vec::new(),
            out_buf: Vec::new(),
            ascii_text: Vec::new(),
            stars: Vec::new(),
            pollen: Vec::new(),
        };
        r.init_stars();
        r.init_pollen();
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
        self.bloom_age = 12.0;
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
        let total = 152usize;
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
                speed: 0.42 + h2 * 1.15,
                base_lum: 0.30 + h1 * 0.66,
                drift_y: if is_drifter { 0.010 + h2 * 0.013 } else { 0.0 },
                drift_x: if is_drifter { (h1 - 0.5) * 0.007 } else { 0.0 },
            });
        }
    }

    fn init_pollen(&mut self) {
        self.pollen.clear();
        let origins: [(f32, f32, Rgb); 14] = [
            (-0.20, 0.14, Rgb::new(255.0, 218.0, 232.0)),
            (0.06, 0.46, Rgb::new(255.0, 224.0, 200.0)),
            (0.40, 0.02, Rgb::new(255.0, 248.0, 232.0)),
            (-0.62, 0.24, Rgb::new(236.0, 214.0, 255.0)),
            (0.58, 0.26, Rgb::new(255.0, 244.0, 178.0)),
            (-0.82, 0.64, Rgb::new(255.0, 248.0, 196.0)),
            (0.82, 0.60, Rgb::new(248.0, 250.0, 255.0)),
            (-0.34, 0.78, Rgb::new(240.0, 228.0, 255.0)),
            (-0.08, 0.22, Rgb::new(255.0, 238.0, 168.0)),
            (0.24, 0.20, Rgb::new(255.0, 246.0, 248.0)),
            (-0.46, 0.42, Rgb::new(255.0, 248.0, 244.0)),
            (0.42, 0.44, Rgb::new(255.0, 250.0, 224.0)),
            (-0.14, 0.58, Rgb::new(255.0, 236.0, 218.0)),
            (0.16, 0.06, Rgb::new(255.0, 232.0, 242.0)),
        ];
        for (i, &(ox, oy, tint)) in origins.iter().enumerate() {
            let fi = (i + 1) as f32;
            self.pollen.push(PollenMote {
                origin_x: ox,
                origin_y: oy,
                speed_y: 0.055 + (fi * 0.137).fract() * 0.045,
                sway_freq: 0.9 + (fi * 0.271).fract() * 0.9,
                sway_amp: 0.04 + (fi * 0.193).fract() * 0.05,
                phase: fi * 1.31,
                tint,
            });
        }
    }

    pub fn render(&mut self, dt: f32) {
        let clamped_dt = dt.clamp(0.0, 0.05);
        self.time += clamped_dt;
        self.bloom_age = (self.bloom_age + clamped_dt).min(20.0);

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

        // Act IV luminous shimmer wave sweeping gracefully across the bouquet as full bloom completes
        let shimmer_phase = ((self.bloom_age - 3.35) / 1.55).clamp(0.0, 1.0);
        if shimmer_phase > 0.0 && shimmer_phase < 1.0 {
            self.shimmer_x = -1.25 + 2.50 * ease_in_out_sine(shimmer_phase);
            self.shimmer_strength = (shimmer_phase * PI).sin() * 0.34;
        } else {
            self.shimmer_x = -3.0;
            self.shimmer_strength = 0.0;
        }

        // Key light slightly sweeps in softly during the opening act, then gently breathes
        let light_drift = (self.time * 0.35).sin() * 0.08;
        let key_light = Vec3::new(-0.52 + light_drift, 0.36, 0.77).normalize();
        let fill_light = Vec3::new(0.75, 0.18, 0.64).normalize();

        // Breeze ramps in smoothly as the bouquet rises & blooms (calm during germination -> lush sway)
        let wind_ramp = 0.20 + 0.80 * smoothstep(1.1, 3.8, self.bloom_age);
        let t = self.time;
        let wind = self.wind_strength * wind_ramp;
        let global_gust = wind
            * (0.55 * (t * 0.78).sin()
                + 0.28 * (t * 1.43 + 0.9).sin()
                + 0.17 * (t * 0.37 - 0.5).cos());

        // ====================================================================
        // 1. MULTI-SPECIES BOTANICAL FOLIAGE (7 leaves + peony collar sprigs)
        // ====================================================================
        let leaves = [
            // Broad outer-left tulip leaf
            LeafSpec {
                p0: Vec3::new(-0.05, -0.64, 0.02),
                p1: Vec3::new(-0.30, -0.36, 0.08),
                p2: Vec3::new(-0.68, -0.10, 0.10),
                p3: Vec3::new(-0.98, -0.10, 0.01),
                max_width: 0.092,
                twist_start: 0.24,
                twist_end: 0.72,
                fold_depth: 0.026,
                sway_phase: 0.10,
                sway_amp: 0.044,
                grow_delay: 0.32,
                deep_green: false,
                id: 701,
            },
            // Broad outer-right tulip leaf
            LeafSpec {
                p0: Vec3::new(0.05, -0.64, 0.02),
                p1: Vec3::new(0.32, -0.38, 0.08),
                p2: Vec3::new(0.70, -0.14, 0.10),
                p3: Vec3::new(0.99, -0.14, 0.01),
                max_width: 0.090,
                twist_start: -0.24,
                twist_end: -0.72,
                fold_depth: 0.026,
                sway_phase: 1.06,
                sway_amp: 0.046,
                grow_delay: 0.38,
                deep_green: false,
                id: 702,
            },
            // Mid-left graceful arching tulip leaf (higher tier)
            LeafSpec {
                p0: Vec3::new(-0.04, -0.58, -0.02),
                p1: Vec3::new(-0.26, -0.24, -0.01),
                p2: Vec3::new(-0.58, 0.04, 0.01),
                p3: Vec3::new(-0.88, 0.14, -0.04),
                max_width: 0.068,
                twist_start: 0.20,
                twist_end: 0.56,
                fold_depth: 0.020,
                sway_phase: 0.42,
                sway_amp: 0.048,
                grow_delay: 0.48,
                deep_green: true,
                id: 703,
            },
            // Mid-right graceful arching tulip leaf (higher tier)
            LeafSpec {
                p0: Vec3::new(0.04, -0.58, -0.02),
                p1: Vec3::new(0.28, -0.26, -0.01),
                p2: Vec3::new(0.60, 0.02, 0.01),
                p3: Vec3::new(0.90, 0.10, -0.04),
                max_width: 0.068,
                twist_start: -0.20,
                twist_end: -0.56,
                fold_depth: 0.020,
                sway_phase: 0.92,
                sway_amp: 0.048,
                grow_delay: 0.54,
                deep_green: true,
                id: 704,
            },
            // Slender upright freesia blade (inner-left)
            LeafSpec {
                p0: Vec3::new(-0.03, -0.58, -0.07),
                p1: Vec3::new(-0.20, -0.14, -0.07),
                p2: Vec3::new(-0.34, 0.26, -0.05),
                p3: Vec3::new(-0.44, 0.58, -0.09),
                max_width: 0.040,
                twist_start: 0.14,
                twist_end: 0.36,
                fold_depth: 0.013,
                sway_phase: 0.28,
                sway_amp: 0.044,
                grow_delay: 0.58,
                deep_green: false,
                id: 705,
            },
            // Slender upright freesia blade (inner-right)
            LeafSpec {
                p0: Vec3::new(0.03, -0.58, -0.07),
                p1: Vec3::new(0.22, -0.16, -0.07),
                p2: Vec3::new(0.36, 0.22, -0.05),
                p3: Vec3::new(0.46, 0.52, -0.09),
                max_width: 0.040,
                twist_start: -0.14,
                twist_end: -0.38,
                fold_depth: 0.013,
                sway_phase: 0.84,
                sway_amp: 0.045,
                grow_delay: 0.64,
                deep_green: false,
                id: 706,
            },
            // Foreground clasping leaf embracing the gathered waist
            LeafSpec {
                p0: Vec3::new(-0.06, -0.66, 0.07),
                p1: Vec3::new(-0.03, -0.44, 0.14),
                p2: Vec3::new(0.06, -0.26, 0.15),
                p3: Vec3::new(0.18, -0.12, 0.11),
                max_width: 0.054,
                twist_start: -0.08,
                twist_end: 0.32,
                fold_depth: 0.017,
                sway_phase: 0.60,
                sway_amp: 0.024,
                grow_delay: 0.45,
                deep_green: false,
                id: 707,
            },
        ];

        for leaf in &leaves {
            self.render_leaf(leaf, global_gust, wind, key_light, fill_light);
        }

        // Peony compound foliage sprays peeking delicately behind the hero & companion peonies
        self.render_peony_leaflet_spray(
            Vec3::new(-0.34, -0.22, 0.04),
            0.52,
            0.19,
            0.68,
            global_gust,
            wind,
            720,
            key_light,
            fill_light,
        );
        self.render_peony_leaflet_spray(
            Vec3::new(0.56, -0.20, 0.04),
            -0.58,
            0.18,
            0.76,
            global_gust,
            wind,
            730,
            key_light,
            fill_light,
        );

        // ====================================================================
        // 2. THREE AIRY ARCHING FREESIA SPIKES (butter yellow, pearl white, mist lilac)
        // ====================================================================
        let freesias = [
            FreesiaSpec {
                p0: Vec3::new(0.09, -0.95, -0.05),
                p1: Vec3::new(-0.07, -0.46, -0.03),
                p2: Vec3::new(-0.38, 0.44, -0.01),
                p3: Vec3::new(-0.94, 0.72, 0.03),
                stem_radius: 0.018,
                floret_scale: 1.42,
                floret_count: 6,
                lean_x: -1.0,
                sway_phase: 0.18,
                sway_amp: 0.070,
                stem_delay: 0.22,
                bloom_delay: 1.50,
                palette: PAL_BUTTER_FREESIA,
                throat_col: PASTEL_STAMEN,
                id_base: 800,
            },
            FreesiaSpec {
                p0: Vec3::new(-0.09, -0.95, -0.05),
                p1: Vec3::new(0.08, -0.46, -0.03),
                p2: Vec3::new(0.40, 0.40, -0.01),
                p3: Vec3::new(0.95, 0.66, 0.03),
                stem_radius: 0.018,
                floret_scale: 1.42,
                floret_count: 6,
                lean_x: 1.0,
                sway_phase: 1.12,
                sway_amp: 0.072,
                stem_delay: 0.28,
                bloom_delay: 1.62,
                palette: PAL_PEARL_FREESIA,
                throat_col: PASTEL_STAMEN,
                id_base: 900,
            },
            // Tall whisper-lilac freesia sprig adding asymmetrical crest height
            FreesiaSpec {
                p0: Vec3::new(0.03, -0.95, -0.07),
                p1: Vec3::new(-0.04, -0.44, -0.06),
                p2: Vec3::new(-0.14, 0.56, -0.05),
                p3: Vec3::new(-0.42, 0.88, -0.03),
                stem_radius: 0.015,
                floret_scale: 1.18,
                floret_count: 5,
                lean_x: -0.75,
                sway_phase: 0.68,
                sway_amp: 0.076,
                stem_delay: 0.34,
                bloom_delay: 1.76,
                palette: PAL_LILAC_FREESIA,
                throat_col: PASTEL_STAMEN,
                id_base: 1000,
            },
        ];

        for freesia in &freesias {
            self.render_freesia_spike(freesia, global_gust, wind, key_light, fill_light);
        }

        // ====================================================================
        // 3. THREE SCULPTURAL TULIPS (peach-apricot hero, lilac left, butter-yellow right)
        // ====================================================================
        let tulips = [
            TulipSpec {
                base_pos: Vec3::new(0.13, -0.95, -0.02),
                waist_pos: Vec3::new(-0.05, -0.48, 0.01),
                head_pos: Vec3::new(-0.65, 0.17, 0.05),
                roll: 0.36,
                pitch: 0.12,
                yaw: 0.18,
                radius: 0.176,
                height: 0.420,
                stem_radius: 0.018,
                sway_phase: 0.05,
                sway_amp: 0.060,
                stem_delay: 0.12,
                bloom_delay: 1.38,
                palette: PAL_LILAC_TULIP,
                id_base: 100,
            },
            TulipSpec {
                base_pos: Vec3::new(-0.01, -0.96, -0.01),
                waist_pos: Vec3::new(0.01, -0.46, 0.02),
                head_pos: Vec3::new(0.06, 0.41, 0.06),
                roll: -0.05,
                pitch: 0.11,
                yaw: -0.06,
                radius: 0.190,
                height: 0.455,
                stem_radius: 0.019,
                sway_phase: 0.48,
                sway_amp: 0.056,
                stem_delay: 0.0,
                bloom_delay: 1.24,
                palette: PAL_PEACH_TULIP,
                id_base: 200,
            },
            TulipSpec {
                base_pos: Vec3::new(-0.12, -0.95, -0.02),
                waist_pos: Vec3::new(0.06, -0.48, 0.01),
                head_pos: Vec3::new(0.60, 0.21, 0.04),
                roll: -0.34,
                pitch: 0.13,
                yaw: -0.20,
                radius: 0.170,
                height: 0.402,
                stem_radius: 0.018,
                sway_phase: 0.94,
                sway_amp: 0.062,
                stem_delay: 0.18,
                bloom_delay: 1.52,
                palette: PAL_BUTTER_TULIP,
                id_base: 300,
            },
        ];

        for tulip in &tulips {
            self.render_tulip(tulip, global_gust, wind, key_light, fill_light);
        }

        // ====================================================================
        // 4. THREE PEONIES (Hero blush-rose, pearl-blush companion, + coral-pink bud with sepals)
        // ====================================================================
        let peonies = [
            PeonySpec {
                base_pos: Vec3::new(0.04, -0.96, 0.03),
                waist_pos: Vec3::new(-0.02, -0.48, 0.06),
                head_pos: Vec3::new(-0.18, 0.08, 0.21),
                roll: 0.12,
                pitch: 1.08,
                yaw: 0.10,
                radius: 0.258,
                stem_radius: 0.020,
                sway_phase: 0.35,
                sway_amp: 0.046,
                stem_delay: 0.06,
                bloom_delay: 1.30,
                is_bud: false,
                palette: PAL_ROSE_PEONY,
                stamen_col: PASTEL_STAMEN,
                id_base: 400,
            },
            PeonySpec {
                base_pos: Vec3::new(-0.11, -0.95, 0.02),
                waist_pos: Vec3::new(0.05, -0.48, 0.06),
                head_pos: Vec3::new(0.39, -0.04, 0.18),
                roll: -0.22,
                pitch: 1.04,
                yaw: -0.18,
                radius: 0.226,
                stem_radius: 0.018,
                sway_phase: 0.86,
                sway_amp: 0.050,
                stem_delay: 0.16,
                bloom_delay: 1.56,
                is_bud: false,
                palette: PAL_PEARL_PEONY,
                stamen_col: PASTEL_STAMEN,
                id_base: 520,
            },
            // Plump coral-pink peony bud cradled by green calyx sepals
            PeonySpec {
                base_pos: Vec3::new(0.08, -0.94, 0.02),
                waist_pos: Vec3::new(-0.07, -0.50, 0.07),
                head_pos: Vec3::new(-0.53, -0.13, 0.15),
                roll: 0.42,
                pitch: 0.56,
                yaw: 0.28,
                radius: 0.126,
                stem_radius: 0.016,
                sway_phase: 0.22,
                sway_amp: 0.052,
                stem_delay: 0.26,
                bloom_delay: 1.85,
                is_bud: true,
                palette: PAL_CORAL_BUD,
                stamen_col: PASTEL_STAMEN,
                id_base: 640,
            },
        ];

        for peony in &peonies {
            self.render_peony(peony, global_gust, wind, key_light, fill_light);
        }

        // ====================================================================
        // 5. DELICATE BABY'S-BREATH (GYPSOPHILA) STARRY MICRO-SPRIGS
        // ====================================================================
        let gypso_sprigs = [
            GypsoSprig {
                base: Vec3::new(-0.06, -0.46, 0.00),
                mid: Vec3::new(-0.24, 0.18, 0.02),
                tip: Vec3::new(-0.37, 0.48, 0.04),
                sway_phase: 0.40,
                grow_delay: 2.10,
                id_base: 1120,
            },
            GypsoSprig {
                base: Vec3::new(0.06, -0.46, 0.00),
                mid: Vec3::new(0.24, 0.16, 0.02),
                tip: Vec3::new(0.36, 0.47, 0.04),
                sway_phase: 1.02,
                grow_delay: 2.28,
                id_base: 1160,
            },
            GypsoSprig {
                base: Vec3::new(-0.08, -0.52, 0.02),
                mid: Vec3::new(-0.46, -0.06, 0.06),
                tip: Vec3::new(-0.77, 0.06, 0.08),
                sway_phase: 0.15,
                grow_delay: 2.45,
                id_base: 1200,
            },
            GypsoSprig {
                base: Vec3::new(0.08, -0.52, 0.02),
                mid: Vec3::new(0.48, -0.10, 0.06),
                tip: Vec3::new(0.77, 0.03, 0.08),
                sway_phase: 1.35,
                grow_delay: 2.60,
                id_base: 1240,
            },
        ];

        for sprig in &gypso_sprigs {
            self.render_gypsophila(sprig, global_gust, wind, key_light, fill_light);
        }

        // ====================================================================
        // 6. SUBTLE SILK RIBBON TIE AT THE BOUQUET WAIST
        // ====================================================================
        self.render_silk_tie(key_light, fill_light);

        // ====================================================================
        // 7. RESOLVE 2×2 Z-BUFFER -> REGULAR ASCII GRID + STARS + FLOATING POLLEN
        // ====================================================================
        self.resolve_grid();
    }

    /// Renders a cylindrical stem up to `grow_t ∈ [0, 1]` so stems literally grow upward in Act I.
    fn render_stem_tube(
        &mut self,
        p0: Vec3,
        p1: Vec3,
        p2: Vec3,
        p3: Vec3,
        stem_radius: f32,
        bulge_tip: bool,
        grow_t: f32,
        leaf_pal: &Palette,
        stem_id: u16,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        if grow_t <= 0.01 {
            return;
        }
        let stem_steps_v = ((84.0 * grow_t) as usize).max(6);
        let stem_steps_u = 12usize;

        for iv in 0..=stem_steps_v {
            let v = (iv as f32 / stem_steps_v as f32) * grow_t;
            let c = bezier3(p0, p1, p2, p3, v);
            let tan = bezier3_tangent(p0, p1, p2, p3, v).normalize();
            let side = tan.cross(Vec3::new(0.0, 0.0, 1.0)).normalize();
            let fwd = side.cross(tan).normalize();

            let tip_bulge = if bulge_tip { 0.30 * v.powf(7.0) } else { -0.22 * v };
            // Taper the very tip gently while still actively growing
            let growing_taper = if grow_t < 0.98 && iv + 3 >= stem_steps_v {
                0.65
            } else {
                1.0
            };
            let r_stem = (stem_radius * (1.0 - 0.10 * v + tip_bulge) * growing_taper).max(0.004);

            for iu in 0..stem_steps_u {
                let ang = (iu as f32 / stem_steps_u as f32) * PI * 2.0;
                let (sa, ca) = ang.sin_cos();
                let n = side.scale(ca).add(fwd.scale(sa)).normalize();
                if n.z < -0.25 {
                    continue;
                }
                let pos = c.add(n.scale(r_stem));
                let waist_ao = 1.0 - 0.24 * (-(v - 0.42).powi(2) * 32.0).exp();
                let bottom_fade = (v / 0.07).clamp(0.45, 1.0);
                let (lum, col) = shade_surface(
                    n,
                    leaf_pal,
                    v * 0.5,
                    waist_ao * bottom_fade,
                    0.22,
                    0.0,
                    key_light,
                    fill_light,
                );
                self.plot_3d(pos, lum, col, stem_id, 2);
            }
        }
    }

    /// Subtle ivory-silk botanical wrap tying the converged stems at the bouquet waist.
    fn render_silk_tie(&mut self, key_light: Vec3, fill_light: Vec3) {
        let tie_prog = ease_out_cubic(((self.bloom_age - 0.55) / 0.65).clamp(0.0, 1.0));
        if tie_prog <= 0.02 {
            return;
        }
        let center_y = -0.58;
        let band_h = 0.068 * tie_prog;
        let rad_x = 0.105 * (0.75 + 0.25 * tie_prog);
        let rad_z = 0.095;

        for iv in 0..=16 {
            let v = iv as f32 / 16.0;
            let y = center_y + (v - 0.5) * band_h;
            let pinch = 1.0 + 0.14 * ((v - 0.5) * 2.0).powi(2);
            for iu in 0..=30 {
                let u = -1.0 + 2.0 * (iu as f32 / 30.0);
                let ang = u * PI * 0.62 * tie_prog;
                let (sa, ca) = ang.sin_cos();
                let pos = Vec3::new(rad_x * pinch * sa, y, 0.03 + rad_z * pinch * ca);
                let n = Vec3::new(sa * 0.85, (v - 0.5) * 0.35, ca).normalize();
                let fold = 0.84 + 0.16 * (v * PI * 3.0).sin().abs();
                let (lum, col) = shade_surface(
                    n,
                    &PAL_RIBBON_SILK,
                    v,
                    fold,
                    0.95,
                    0.0,
                    key_light,
                    fill_light,
                );
                self.plot_3d(pos, lum, col, 1350, 4);
            }
        }

        // Two delicate short draped silk ribbon tails falling slightly from the knot
        let tail_prog = ease_out_cubic(((self.bloom_age - 0.85) / 0.70).clamp(0.0, 1.0));
        if tail_prog > 0.02 {
            let tails = [
                (
                    Vec3::new(-0.02, -0.60, 0.12),
                    Vec3::new(-0.08, -0.68, 0.13),
                    Vec3::new(-0.13, -0.78, 0.12),
                    Vec3::new(-0.17, -0.87, 0.10),
                    1351u16,
                ),
                (
                    Vec3::new(0.02, -0.60, 0.12),
                    Vec3::new(0.07, -0.67, 0.13),
                    Vec3::new(0.11, -0.76, 0.12),
                    Vec3::new(0.14, -0.84, 0.10),
                    1352u16,
                ),
            ];
            for &(p0, p1, p2, p3, id) in &tails {
                let steps = ((24.0 * tail_prog) as usize).max(3);
                for iv in 0..=steps {
                    let v = (iv as f32 / steps as f32) * tail_prog;
                    let c = bezier3(p0, p1, p2, p3, v);
                    let w = 0.017 * (1.0 - 0.32 * v);
                    for iu in -3..=3 {
                        let u = iu as f32 / 3.0;
                        let pos = Vec3::new(c.x + u * w, c.y, c.z + (1.0 - u * u) * 0.005);
                        let n = Vec3::new(u * 0.45, 0.15, 0.88).normalize();
                        let (lum, col) = shade_surface(
                            n,
                            &PAL_RIBBON_SILK,
                            1.0 - v * 0.4,
                            0.88 - 0.15 * v,
                            0.85,
                            0.0,
                            key_light,
                            fill_light,
                        );
                        self.plot_3d(pos, lum, col, id, 4);
                    }
                }
            }
        }
    }

    fn render_tulip(
        &mut self,
        spec: &TulipSpec,
        global_gust: f32,
        wind: f32,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let t = self.time;
        let local_sway = (global_gust
            + wind * 0.36 * (t * 1.12 + spec.sway_phase).sin()
            + wind * 0.14 * (t * 2.18 + spec.sway_phase * 1.7).cos())
            * spec.sway_amp
            + self.pointer_push_x * 0.45;
        let local_depth_sway = wind * 0.022 * (t * 0.92 + spec.sway_phase + 1.1).cos();

        // Act I: stem grows smoothly upward with a subtle arrival bounce
        let stem_raw = ((self.bloom_age - spec.stem_delay) / 1.18).clamp(0.0, 1.0);
        let stem_grow = ease_out_cubic(stem_raw);
        let arrive_nod = ((self.bloom_age - spec.stem_delay - 0.95).max(0.0) * 5.2).sin()
            * (-((self.bloom_age - spec.stem_delay - 0.95).max(0.0) * 2.6)).exp()
            * 0.055;

        let p0 = spec.base_pos;
        let p1 = Vec3::new(
            spec.waist_pos.x + local_sway * 0.14,
            spec.waist_pos.y,
            spec.waist_pos.z + local_depth_sway * 0.15,
        );
        let p3 = Vec3::new(
            spec.head_pos.x + local_sway + arrive_nod * 0.4,
            spec.head_pos.y - local_sway.abs() * 0.14,
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
            stem_grow,
            &PAL_LEAF_SAGE,
            spec.id_base + 90,
            key_light,
            fill_light,
        );

        // Act II: bud emerges smoothly at the exact growing tip once stem reaches 72%
        let bud_emerge = ease_out_back_soft(((stem_raw - 0.68) / 0.36).clamp(0.0, 1.0));
        if bud_emerge <= 0.02 {
            return;
        }
        let head_anchor = bezier3(p0, p1, p2, p3, stem_grow);

        // Act III: outer whorl opens slightly before inner crown petals
        let outer_open = ease_out_cubic(((self.bloom_age - spec.bloom_delay) / 1.85).clamp(0.0, 1.0));
        let inner_open = ease_out_cubic(((self.bloom_age - spec.bloom_delay - 0.26) / 1.95).clamp(0.0, 1.0));
        let breath = 0.042 * (t * 0.85 + spec.sway_phase).sin() * inner_open;

        let roll_dyn = spec.roll - local_sway * 1.45 - arrive_nod * 1.6;
        let pitch_dyn = spec.pitch + local_depth_sway * 1.6 + (1.0 - outer_open) * 0.16;
        let yaw_dyn = spec.yaw + local_sway * 0.35;
        let rot = Mat3::from_ypr(yaw_dyn, pitch_dyn, roll_dyn);

        let radius_s = spec.radius * (0.28 + 0.72 * bud_emerge);
        let height_s = spec.height * (0.32 + 0.68 * bud_emerge);

        let petals: [(f32, bool, u16); 6] = [
            (0.0, true, spec.id_base + 1),
            (1.96, true, spec.id_base + 2),
            (-1.96, true, spec.id_base + 3),
            (0.98, false, spec.id_base + 4),
            (-0.98, false, spec.id_base + 5),
            (PI, false, spec.id_base + 6),
        ];

        let u_steps = 36usize;
        let v_steps = 48usize;
        let du = 0.020;
        let dv = 0.016;

        for &(azim, is_outer, petal_id) in &petals {
            let open_prog = if is_outer { outer_open } else { inner_open };
            let bloom = (0.04 + 0.96 * open_prog + breath).clamp(0.03, 1.08);

            for iv in 0..=v_steps {
                let v = iv as f32 / v_steps as f32;
                let row0 = precompute_tulip_row(v, is_outer, bloom, radius_s, height_s);
                let row1 = precompute_tulip_row((v + dv).min(1.0), is_outer, bloom, radius_s, height_s);
                let cup_ao = 0.76 + 0.24 * v.powf(0.42);
                let whorl_ao = if is_outer { 1.0 } else { 0.82 + 0.18 * v };
                let sin_vpi = (v * PI).sin();
                // Soft pastel cream rim accent along upper tulip petal lip
                let picotee = ((v - 0.72) / 0.28).clamp(0.0, 1.0) * 0.36;

                for iu in 0..=u_steps {
                    let u = -1.0 + 2.0 * (iu as f32 / u_steps as f32);
                    let p_loc = eval_tulip_from_row(u, azim, &row0);
                    let pu = eval_tulip_from_row((u + du).min(1.0), azim, &row0);
                    let pv = eval_tulip_from_row(u, azim, &row1);

                    let n_loc = pu.sub(p_loc).cross(pv.sub(p_loc)).normalize();
                    let p_world = rot.mul_vec(p_loc).add(head_anchor);
                    let mut n_world = rot.mul_vec(n_loc).normalize();

                    let mut interior_factor = 1.0_f32;
                    if n_world.z < 0.0 {
                        n_world = n_world.scale(-1.0);
                        interior_factor = 0.56;
                    }

                    let u_abs = u.abs();
                    let edge_crease = 1.0 - 0.28 * (u_abs * u_abs * u_abs.sqrt());
                    // Fine longitudinal tulip petal veining
                    let vein_mod = 1.0
                        + 0.075 * (u * PI * 2.0).cos() * sin_vpi
                        + 0.035 * (u * PI * 5.0).cos() * sin_vpi;
                    let ao = (cup_ao * whorl_ao * interior_factor * edge_crease * vein_mod)
                        .clamp(0.26, 1.14);

                    let (lum, col) = shade_surface(
                        n_world,
                        &spec.palette,
                        v,
                        ao,
                        1.18,
                        picotee,
                        key_light,
                        fill_light,
                    );
                    self.plot_3d(p_world, lum, col, petal_id, 1);
                }
            }
        }

        // Subtle tulip pistil & 6 stamens peeking inside the chalice once opened
        if inner_open > 0.45 {
            let core_id = spec.id_base + 50;
            let reveal = ((inner_open - 0.45) / 0.55).clamp(0.0, 1.0);
            for k in 0..6 {
                let ang = (k as f32) * (PI / 3.0) + 0.4;
                let (sa, ca) = ang.sin_cos();
                for iv in 0..=10 {
                    let v = iv as f32 / 10.0;
                    let r = radius_s * (0.04 + 0.14 * v * reveal);
                    let y = height_s * (0.14 + 0.42 * v * reveal);
                    let p_world = rot.mul_vec(Vec3::new(r * sa, y, r * ca)).add(head_anchor);
                    let col = if v > 0.72 {
                        TULIP_ANTHER.lerp(PASTEL_STAMEN, 0.55)
                    } else {
                        Rgb::new(236.0, 232.0, 198.0)
                    };
                    self.plot_3d(p_world, 0.42 + 0.36 * v, col, core_id, 1);
                }
            }
        }
    }

    fn render_peony(
        &mut self,
        spec: &PeonySpec,
        global_gust: f32,
        wind: f32,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let t = self.time;
        let local_sway = (global_gust
            + wind * 0.30 * (t * 1.02 + spec.sway_phase).sin()
            + wind * 0.12 * (t * 1.94 + spec.sway_phase).cos())
            * spec.sway_amp
            + self.pointer_push_x * 0.40;
        let local_depth_sway = wind * 0.018 * (t * 0.88 + spec.sway_phase).cos();

        let stem_raw = ((self.bloom_age - spec.stem_delay) / 1.15).clamp(0.0, 1.0);
        let stem_grow = ease_out_cubic(stem_raw);

        let p0 = spec.base_pos;
        let p1 = Vec3::new(
            spec.waist_pos.x + local_sway * 0.14,
            spec.waist_pos.y,
            spec.waist_pos.z,
        );
        let p3 = Vec3::new(
            spec.head_pos.x + local_sway,
            spec.head_pos.y - local_sway.abs() * 0.12,
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
            stem_grow,
            &PAL_LEAF_DEEP,
            spec.id_base + 115,
            key_light,
            fill_light,
        );

        let bud_emerge = ease_out_back_soft(((stem_raw - 0.66) / 0.38).clamp(0.0, 1.0));
        if bud_emerge <= 0.02 {
            return;
        }
        let head_anchor = bezier3(p0, p1, p2, p3, stem_grow);

        let roll_dyn = spec.roll - local_sway * 1.12;
        let pitch_dyn = spec.pitch + local_depth_sway * 1.2;
        let yaw_dyn = spec.yaw + local_sway * 0.30;
        let rot = Mat3::from_ypr(yaw_dyn, pitch_dyn, roll_dyn);

        let r_scale = spec.radius * (0.28 + 0.72 * bud_emerge);

        // Curved green calyx sepals cradling the underside of the peony bud & bloom
        let sepal_open = ease_out_cubic(((self.bloom_age - spec.bloom_delay + 0.15) / 1.4).clamp(0.0, 1.0));
        for s in 0..5 {
            let az = (s as f32) * (PI * 2.0 / 5.0) + 0.3;
            let sepal_tilt = if spec.is_bud {
                0.52 + 0.42 * sepal_open
            } else {
                0.70 + 0.75 * sepal_open
            };
            let sepal_span = r_scale * if spec.is_bud { 1.08 } else { 0.88 };
            for iv in 0..=12 {
                let v = iv as f32 / 12.0;
                let w_env = (v.max(1e-3).powf(0.42) * (1.0 - v).max(1e-3).powf(0.70) * 2.1).clamp(0.0, 1.0);
                let (st, ct) = sepal_tilt.sin_cos();
                let base_r = r_scale * 0.12 + sepal_span * (v * PI * 0.62).sin() * st;
                let y = -r_scale * 0.10 + sepal_span * (v * ct - 0.14 * v * v);
                for iu in 0..=8 {
                    let u = -1.0 + 2.0 * (iu as f32 / 8.0);
                    let ang = az + u * 0.34 * w_env;
                    let (sa, ca) = ang.sin_cos();
                    let p_loc = Vec3::new(base_r * sa, y, base_r * ca);
                    let p_world = rot.mul_vec(p_loc).add(head_anchor);
                    let n_world = rot.mul_vec(Vec3::new(sa * 0.7, -0.35, ca * 0.7).normalize()).normalize();
                    let (lum, col) = shade_surface(
                        if n_world.z < 0.0 { n_world.scale(-1.0) } else { n_world },
                        &PAL_LEAF_DEEP,
                        v,
                        0.72 + 0.25 * v,
                        0.45,
                        0.0,
                        key_light,
                        fill_light,
                    );
                    self.plot_3d(p_world, lum, col, spec.id_base + 98 + s as u16, 3);
                }
            }
        }

        // 5 concentric ruffled rings on full peonies; 3 tightly-cupped rings on the peony bud!
        // (count, tilt_closed, tilt_open, span_mult, y_off, half_ang, phase_off, ruffle_amp, ring_ao)
        let full_rings: [(usize, f32, f32, f32, f32, f32, f32, f32, f32); 5] = [
            (8, 0.36, 1.32, 1.06, 0.00, 0.54, 0.14, 0.024, 0.90),
            (8, 0.30, 1.06, 0.90, 0.07, 0.52, 0.54, 0.026, 1.00),
            (7, 0.24, 0.80, 0.74, 0.13, 0.56, 0.26, 0.024, 1.06),
            (6, 0.18, 0.54, 0.56, 0.18, 0.60, 0.68, 0.018, 1.12),
            (5, 0.12, 0.30, 0.38, 0.22, 0.66, 0.10, 0.013, 1.15),
        ];
        let bud_rings: [(usize, f32, f32, f32, f32, f32, f32, f32, f32); 3] = [
            (6, 0.22, 0.56, 0.98, 0.02, 0.62, 0.12, 0.012, 0.96),
            (5, 0.16, 0.38, 0.80, 0.09, 0.64, 0.58, 0.010, 1.04),
            (4, 0.10, 0.22, 0.58, 0.15, 0.68, 0.28, 0.008, 1.08),
        ];
        let rings: &[(usize, f32, f32, f32, f32, f32, f32, f32, f32)] =
            if spec.is_bud { &bud_rings } else { &full_rings };

        let mut pid = spec.id_base + 1;
        let mut inner_ring_open = 0.0_f32;

        for (ring_idx, &(count, tilt_closed, tilt_open, span_mult, y_off, half_ang, phase_off, ruffle_amp, ring_ao)) in
            rings.iter().enumerate()
        {
            // Act III signature: outer guard petals peel open first, followed ring-by-ring inward
            // with a gentle botanical helical unspiral!
            let ring_delay = spec.bloom_delay + (ring_idx as f32) * 0.25;
            let ring_raw = ((self.bloom_age - ring_delay) / 1.45).clamp(0.0, 1.0);
            let ring_open = ease_out_cubic(ring_raw);
            inner_ring_open = ring_open;

            let breath = 0.036 * (t * 0.78 + spec.sway_phase + ring_idx as f32 * 0.25).sin() * ring_open;
            let open_tilt = tilt_closed + (tilt_open - tilt_closed) * (ring_open + breath).clamp(0.0, 1.06);
            // Graceful helical swirl unwinds as each ring opens
            let swirl_twist = (1.0 - ring_open) * (0.62 - ring_idx as f32 * 0.08);

            let span = r_scale * span_mult * (0.68 + 0.32 * ring_open);
            let y_base = r_scale * y_off;
            let u_steps = if spec.is_bud { 14 } else if ring_idx < 2 { 20 } else { 16 };
            let v_steps = if spec.is_bud { 16 } else if ring_idx < 2 { 22 } else { 18 };
            let du = 0.036;
            let dv = 0.032;

            for k in 0..count {
                let azim = phase_off + swirl_twist + (k as f32 / count as f32) * PI * 2.0;
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
                    let cup_grad = 0.46 + 0.66 * v.powf(0.75);
                    let picotee = ((v - 0.70) / 0.30).clamp(0.0, 1.0) * 0.45;
                    let ruffle_lip = if v > 0.70 { 1.0 + (v - 0.70) * 0.95 } else { 1.0 };

                    for iu in 0..=u_steps {
                        let u = -1.0 + 2.0 * (iu as f32 / u_steps as f32);
                        let p_loc = eval_peony_from_row(u, azim, &r0, ruffle_amp, ruffle_phase);
                        let pu = eval_peony_from_row((u + du).min(1.0), azim, &r0, ruffle_amp, ruffle_phase);
                        let pv = eval_peony_from_row(u, azim, &r1, ruffle_amp, ruffle_phase);

                        let geom_n = pu.sub(p_loc).cross(pv.sub(p_loc)).normalize();
                        let n_loc = geom_n.scale(0.55).add(dome_n_loc.scale(0.45)).normalize();

                        let p_world = rot.mul_vec(p_loc).add(head_anchor);
                        let mut n_world = rot.mul_vec(n_loc).normalize();
                        if n_world.z < 0.0 {
                            n_world = n_world.scale(-1.0);
                        }

                        let edge_soft = 1.0 - 0.18 * (u * u * u.abs());
                        let ao = (cup_grad * ring_ao * edge_soft * ruffle_lip).clamp(0.24, 1.32);

                        let (lum, col) = shade_surface(
                            n_world,
                            &spec.palette,
                            0.14 + 0.86 * v,
                            ao,
                            1.22,
                            picotee,
                            key_light,
                            fill_light,
                        );
                        self.plot_3d(p_world, lum, col, petal_id, 1);
                    }
                }
            }
        }

        // Detailed golden-cream stamen crown & tiny radiating anthers revealed when inner ring opens
        if !spec.is_bud && inner_ring_open > 0.25 {
            let core_reveal = ease_out_cubic(((inner_ring_open - 0.25) / 0.75).clamp(0.0, 1.0));
            let core_id = spec.id_base + 92;
            let core_r = r_scale * 0.18 * core_reveal;
            for iv in 0..=10 {
                let v = iv as f32 / 10.0;
                let phi = v * PI * 0.5;
                let (sp, cp) = phi.sin_cos();
                for iu in 0..16 {
                    let ang = (iu as f32 / 16.0) * PI * 2.0;
                    let (sa, ca) = ang.sin_cos();
                    let p_loc = Vec3::new(
                        core_r * sp * sa,
                        r_scale * 0.22 + core_r * 0.55 * cp,
                        core_r * sp * ca,
                    );
                    let p_world = rot.mul_vec(p_loc).add(head_anchor);
                    let lum = 0.56 + 0.40 * cp;
                    let col = spec.stamen_col.lerp(Rgb::new(255.0, 253.0, 238.0), cp * 0.52);
                    self.plot_3d(p_world, lum, col, core_id, 1);
                }
            }
            // Tiny golden filament crown tips around the peony center
            for f in 0..10 {
                let ang = (f as f32) * (PI * 2.0 / 10.0);
                let (sa, ca) = ang.sin_cos();
                let tip_r = core_r * 1.25;
                let p_world = rot
                    .mul_vec(Vec3::new(tip_r * sa, r_scale * 0.25 + core_r * 0.42, tip_r * ca))
                    .add(head_anchor);
                self.plot_3d(p_world, 0.92, Rgb::new(255.0, 240.0, 164.0), core_id + 1, 1);
            }
        }
    }

    fn render_freesia_spike(
        &mut self,
        spec: &FreesiaSpec,
        global_gust: f32,
        wind: f32,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let t = self.time;
        let local_sway = (global_gust
            + wind * 0.44 * (t * 1.24 + spec.sway_phase).sin()
            + wind * 0.18 * (t * 2.32 + spec.sway_phase).cos())
            * spec.sway_amp
            + self.pointer_push_x * 0.52;
        let local_depth = wind * 0.025 * (t * 0.96 + spec.sway_phase).cos();

        let stem_raw = ((self.bloom_age - spec.stem_delay) / 1.30).clamp(0.0, 1.0);
        let stem_grow = ease_out_cubic(stem_raw);

        let p0 = spec.p0;
        let p1 = Vec3::new(spec.p1.x + local_sway * 0.18, spec.p1.y, spec.p1.z);
        let p2 = Vec3::new(
            spec.p2.x + local_sway * 0.65,
            spec.p2.y,
            spec.p2.z + local_depth * 0.5,
        );
        let p3 = Vec3::new(
            spec.p3.x + local_sway * 1.18,
            spec.p3.y - local_sway.abs() * 0.16,
            spec.p3.z + local_depth,
        );

        self.render_stem_tube(
            p0,
            p1,
            p2,
            p3,
            spec.stem_radius,
            false,
            stem_grow,
            &PAL_LEAF_SAGE,
            spec.id_base + 95,
            key_light,
            fill_light,
        );

        let count = spec.floret_count;
        for idx in 0..count {
            let frac = idx as f32 / (count - 1).max(1) as f32;
            let s = 0.52 + frac * 0.46; // 0.52 -> 0.98 along the arching comb
            if stem_grow < s - 0.04 {
                continue;
            }

            // Bud appears right as the growing stem reaches position `s`
            let emerge = ease_out_back_soft(((stem_grow - (s - 0.04)) / 0.16).clamp(0.0, 1.0));
            if emerge <= 0.02 {
                continue;
            }

            // Act III domino wave: florets burst open sequentially along the comb (base -> tip)
            let floret_bloom_t = ((self.bloom_age - (spec.bloom_delay + idx as f32 * 0.20)) / 1.10).clamp(0.0, 1.0);
            let floret_open = ease_out_cubic(floret_bloom_t);
            let breath = 0.04 * (t * 0.92 + spec.sway_phase + idx as f32 * 0.3).sin() * floret_open;

            let size_m = 1.06 - frac * 0.52;
            // Base florets open into wide star-trumpets; tip stays a poised pointed bud
            let target_open = if idx + 1 == count {
                0.08
            } else if idx + 2 == count {
                0.42
            } else {
                (1.02 - frac * 0.28).clamp(0.5, 1.02)
            };
            let openness = (0.06 + (target_open - 0.06) * floret_open + breath * target_open).clamp(0.05, 1.06);
            let tilt_out = 0.14 + frac * 0.72;

            let anchor = bezier3(p0, p1, p2, p3, s).add(Vec3::new(0.0, 0.012, 0.018));
            let scale = spec.floret_scale * size_m * (0.35 + 0.65 * emerge);

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
        let rot = Mat3::from_ypr(yaw, pitch, roll);

        let u_steps = 10usize;
        let v_steps = 18usize;
        let du = 0.045;
        let dv = 0.038;

        for k in 0..6 {
            let azim = (k as f32) * (PI / 3.0) + 0.26;
            let petal_id = id_base + k as u16;

            for iv in 0..=v_steps {
                let v = iv as f32 / v_steps as f32;
                let r0 = precompute_freesia_row(v, openness, max_r, len_total);
                let r1 = precompute_freesia_row((v + dv).min(1.0), openness, max_r, len_total);
                let is_calyx_bract = v < 0.16;

                for iu in 0..=u_steps {
                    let u = -1.0 + 2.0 * (iu as f32 / u_steps as f32);
                    let p_loc = eval_freesia_from_row(u, azim, &r0);
                    let pu = eval_freesia_from_row((u + du).min(1.0), azim, &r0);
                    let pv = eval_freesia_from_row(u, azim, &r1);

                    let n_loc = pu.sub(p_loc).cross(pv.sub(p_loc)).normalize();
                    let p_world = rot.mul_vec(p_loc).add(origin);
                    let mut n_world = rot.mul_vec(n_loc).normalize();
                    if n_world.z < 0.0 {
                        n_world = n_world.scale(-1.0);
                    }

                    let ao = (0.84 + 0.32 * v.powf(0.5)) * (1.0 - 0.14 * u * u);
                    let pal_ref = if is_calyx_bract { &PAL_LEAF_SAGE } else { palette };
                    let (lum, mut col) = shade_surface(
                        n_world,
                        pal_ref,
                        v,
                        ao,
                        1.22,
                        if v > 0.75 { 0.30 } else { 0.0 },
                        key_light,
                        fill_light,
                    );
                    if !is_calyx_bract && v < 0.38 && openness > 0.35 {
                        let throat_k = ((0.38 - v) / 0.22).clamp(0.0, 1.0) * 0.45 * openness;
                        col = col.lerp(throat_col, throat_k);
                    }
                    self.plot_3d(p_world, lum, col, petal_id, if is_calyx_bract { 3 } else { 1 });
                }
            }
        }
    }

    /// Delicate branching Baby's-Breath (Gypsophila) sprigs with tiny starry white micro-blossoms.
    fn render_gypsophila(
        &mut self,
        sprig: &GypsoSprig,
        global_gust: f32,
        wind: f32,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let grow = ease_out_cubic(((self.bloom_age - sprig.grow_delay) / 0.95).clamp(0.0, 1.0));
        if grow <= 0.02 {
            return;
        }

        let t = self.time;
        let sway = (global_gust + wind * 0.48 * (t * 1.45 + sprig.sway_phase).sin()) * 0.068
            + self.pointer_push_x * 0.46;

        let p0 = sprig.base;
        let p1 = Vec3::new(sprig.mid.x + sway * 0.45, sprig.mid.y, sprig.mid.z);
        let p2 = Vec3::new(sprig.tip.x + sway, sprig.tip.y, sprig.tip.z);

        // Delicate main twig (quadratic Bézier sampled thinly)
        let steps = ((34.0 * grow) as usize).max(4);
        for i in 0..=steps {
            let s = (i as f32 / steps as f32) * grow;
            let u = 1.0 - s;
            let pos = p0
                .scale(u * u)
                .add(p1.scale(2.0 * u * s))
                .add(p2.scale(s * s));
            let col = Rgb::new(44.0, 152.0, 88.0);
            self.plot_3d(pos, 0.34, col, sprig.id_base, 2);
        }

        // 5 tiny branching forks tipped with luminous starry pearl-white gypsophila blossoms
        let forks: [(f32, f32, f32, f32); 5] = [
            (0.62, -0.09, 0.07, 0.02),
            (0.74, 0.08, 0.08, 0.03),
            (0.84, -0.07, 0.09, 0.02),
            (0.93, 0.07, 0.06, 0.02),
            (1.00, 0.00, 0.05, 0.03),
        ];

        for (k, &(s, ox, oy, oz)) in forks.iter().enumerate() {
            if grow < s - 0.05 {
                continue;
            }
            let u = 1.0 - s;
            let root = p0
                .scale(u * u)
                .add(p1.scale(2.0 * u * s))
                .add(p2.scale(s * s));
            let fork_grow = ease_out_cubic(((grow - (s - 0.05)) / 0.22).clamp(0.0, 1.0));
            let tip = root.add(Vec3::new(ox * fork_grow, oy * fork_grow, oz * fork_grow));

            // Fine twiglet
            for j in 1..=8 {
                let f = j as f32 / 8.0;
                let pos = root.lerp_pos(tip, f);
                self.plot_3d(pos, 0.30, Rgb::new(48.0, 160.0, 94.0), sprig.id_base + 1, 2);
            }

            // Starlet blossom pops gently into existence right after twiglet arrives
            let pop = ease_out_back_soft(
                ((self.bloom_age - (sprig.grow_delay + 0.55 + k as f32 * 0.14)) / 0.55).clamp(0.0, 1.0),
            );
            if pop <= 0.05 {
                continue;
            }
            let rad = 0.026 * pop;
            let blossom_id = sprig.id_base + 10 + k as u16;
            for ring in 0..=3 {
                let rf = ring as f32 / 3.0;
                let pts = if ring == 0 { 1 } else { 7 };
                for p in 0..pts {
                    let ang = (p as f32 / pts as f32) * PI * 2.0 + k as f32;
                    let (sa, ca) = ang.sin_cos();
                    let pos = tip.add(Vec3::new(rad * rf * sa, rad * rf * ca, rad * (1.0 - rf * 0.6)));
                    let n = Vec3::new(sa * rf * 0.6, ca * rf * 0.6, 0.85).normalize();
                    let (lum, col) = shade_surface(
                        n,
                        &PAL_PEARL_FREESIA,
                        1.0 - rf * 0.3,
                        0.96 - rf * 0.22,
                        1.25,
                        0.3,
                        key_light,
                        fill_light,
                    );
                    self.plot_3d(pos, lum, col, blossom_id, 1);
                }
            }
        }
    }

    /// 3-leaflet scalloped peony foliage collar spray peeking behind the peonies.
    fn render_peony_leaflet_spray(
        &mut self,
        origin: Vec3,
        base_angle: f32,
        len: f32,
        grow_delay: f32,
        global_gust: f32,
        wind: f32,
        id_base: u16,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let grow = ease_out_cubic(((self.bloom_age - grow_delay) / 1.15).clamp(0.0, 1.0));
        if grow <= 0.02 {
            return;
        }
        let sway = (global_gust + wind * 0.32 * (self.time * 1.2 + base_angle).sin()) * 0.036
            + self.pointer_push_x * 0.25;

        let leaflets: [(f32, f32, f32); 3] = [
            (base_angle - 0.44, len * 0.82, 0.042),
            (base_angle, len * 1.04, 0.050),
            (base_angle + 0.44, len * 0.82, 0.042),
        ];

        for (idx, &(ang, l_len, max_w)) in leaflets.iter().enumerate() {
            let a = ang - sway * 1.2;
            let (sa, ca) = a.sin_cos();
            let dir = Vec3::new(-sa, ca, 0.05).normalize();
            let side = Vec3::new(ca, sa, 0.0).normalize();
            let up = Vec3::new(0.0, 0.0, 1.0);

            let v_steps = ((24.0 * grow) as usize).max(4);
            for iv in 0..=v_steps {
                let v = (iv as f32 / v_steps as f32) * grow;
                let w_env = (v.max(1e-3).powf(0.45) * (1.0 - v).max(1e-3).powf(0.65) * 2.15).clamp(0.0, 1.0);
                // Subtle notched margin characteristic of peony leaves
                let notch = 1.0 + 0.10 * (v * PI * 6.0).sin();
                let hw = max_w * w_env * notch * (0.35 + 0.65 * grow);
                let center = origin.add(dir.scale(l_len * v)).add(Vec3::new(0.0, 0.0, -0.02 * v * v));

                for iu in 0..=10 {
                    let u = -1.0 + 2.0 * (iu as f32 / 10.0);
                    let pos = center.add(side.scale(u * hw)).add(up.scale(-0.012 * (1.0 - u.abs())));
                    let slope = if u >= 0.0 { 0.38 } else { -0.38 };
                    let n = up.add(side.scale(slope)).normalize();
                    let (lum, col) = shade_surface(
                        n,
                        &PAL_LEAF_DEEP,
                        0.2 + 0.7 * v,
                        0.68 + 0.30 * v,
                        0.55,
                        0.0,
                        key_light,
                        fill_light,
                    );
                    self.plot_3d(pos, lum, col, id_base + idx as u16, 3);
                }
            }
        }
    }

    fn render_leaf(
        &mut self,
        leaf: &LeafSpec,
        global_gust: f32,
        wind: f32,
        key_light: Vec3,
        fill_light: Vec3,
    ) {
        let grow = ease_out_cubic(((self.bloom_age - leaf.grow_delay) / 1.35).clamp(0.0, 1.0));
        if grow <= 0.02 {
            return;
        }

        let t = self.time;
        let flutter = (global_gust
            + wind * 0.45 * (t * 1.48 + leaf.sway_phase).sin()
            + wind * 0.18 * (t * 2.65 + leaf.sway_phase).cos())
            * leaf.sway_amp
            + self.pointer_push_x * 0.32;

        // Leaves uncurl gracefully outward as they grow in Act I
        let uncurl = ease_out_cubic(grow);
        let p0 = leaf.p0;
        let p1 = Vec3::new(
            p0.x + (leaf.p1.x - p0.x) * (0.45 + 0.55 * uncurl) + flutter * 0.25,
            leaf.p1.y,
            leaf.p1.z,
        );
        let p2 = Vec3::new(
            p0.x + (leaf.p2.x - p0.x) * (0.35 + 0.65 * uncurl) + flutter * 0.72,
            leaf.p2.y,
            leaf.p2.z + flutter * 0.15,
        );
        let p3 = Vec3::new(
            p0.x + (leaf.p3.x - p0.x) * (0.30 + 0.70 * uncurl) + flutter * 1.25,
            leaf.p3.y - flutter.abs() * 0.12,
            leaf.p3.z + flutter * 0.28,
        );

        let v_steps = ((68.0 * grow) as usize).max(6);
        let u_steps = 20usize;
        let pal = if leaf.deep_green { &PAL_LEAF_DEEP } else { &PAL_LEAF_SAGE };

        for iv in 0..=v_steps {
            let v = (iv as f32 / v_steps as f32) * grow;
            let center = bezier3(p0, p1, p2, p3, v);
            let tan = bezier3_tangent(p0, p1, p2, p3, v).normalize();

            let width_profile = (v.powf(0.42) * (1.0 - v).powf(0.76) * 2.38).clamp(0.0, 1.0);
            let half_w = leaf.max_width * width_profile * (0.35 + 0.65 * uncurl);

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
                let rib_ao = 0.85 + 0.15 * u.abs() + 0.04 * (u * PI * 3.0).cos();
                let (lum, col) = shade_surface(
                    n,
                    pal,
                    0.18 + 0.68 * v,
                    base_ao * rib_ao,
                    0.68,
                    0.0,
                    key_light,
                    fill_light,
                );
                self.plot_3d(pos, lum, col, leaf.id, 3);
            }
        }
    }

    #[inline]
    fn plot_3d(&mut self, world: Vec3, mut lum: f32, mut col: Rgb, surface_id: u16, kind: u8) {
        let cam_z = 3.55;
        let dz = cam_z - world.z;
        if dz < 0.4 {
            return;
        }
        let persp = cam_z / dz;

        // Subtle luminous shimmer wave sweeping across petals during the Act IV bloom finale
        if self.shimmer_strength > 0.001 && kind == 1 {
            let dx = world.x - self.shimmer_x;
            let wave = (-(dx * dx) * 14.0).exp() * self.shimmer_strength;
            if wave > 0.01 {
                lum = (lum + wave).clamp(0.0, 1.0);
                col = col.lerp(Rgb::new(255.0, 254.0, 250.0), wave * 1.15);
            }
        }

        let sw = (self.cols * 2) as f32;
        let sh = (self.rows * 2) as f32;

        // Fit both landscape desktop and portrait mobile screens without clipping or distortion
        let scale_y_by_h = sh * 0.450;
        let max_scale_x = (sw * 0.44) / 1.13;
        let scale_y_by_w = max_scale_x * self.char_aspect;
        let scale_y = scale_y_by_h.min(scale_y_by_w);
        let scale_x = scale_y / self.char_aspect;

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

        // 2×2 supersampled downsample onto the regular monospace grid
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
                    if s.kind != 0 && (s.surface_id == best_id || (best_z - s.z) < 0.040) {
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
                    } else if nb.surface_id != cell.surface_id && (nb.z - cell.z) > 0.022 {
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

        // Delicate luminous pollen motes rising softly out of the freshly bloomed bouquet (Act IV+)
        let pollen_alpha = smoothstep(2.8, 4.2, self.bloom_age);
        if pollen_alpha > 0.02 {
            let sw_f = cols as f32;
            let sh_f = rows as f32;
            let scale_y_by_h = sh_f * 0.450;
            let scale_y_by_w = ((sw_f * 0.44) / 1.13) * self.char_aspect;
            let scale_y = scale_y_by_h.min(scale_y_by_w);
            let scale_x = scale_y / self.char_aspect;
            let center_y_ratio = if scale_y_by_w < scale_y_by_h * 0.92 { 0.495 } else { 0.515 };

            for mote in &self.pollen {
                let cycle = ((self.bloom_age - 2.6).max(0.0) * mote.speed_y + mote.phase * 0.17).fract();
                let wx = mote.origin_x
                    + (t * mote.sway_freq + mote.phase).sin() * mote.sway_amp
                    + cycle * self.wind_strength * 0.14;
                let wy = mote.origin_y + cycle * 0.58;
                let fade = (cycle * PI).sin() * pollen_alpha;
                if fade < 0.12 {
                    continue;
                }

                let cx = (sw_f * 0.5 + wx * scale_x) as i32;
                let cy = (sh_f * center_y_ratio - wy * scale_y) as i32;
                if cx < 0 || cy < 0 || cx >= cols as i32 || cy >= rows as i32 {
                    continue;
                }
                let idx = cy as usize * cols + cx as usize;
                if self.cell_buf[idx].kind != 0 {
                    continue;
                }
                let out_i = idx * 4;
                self.out_buf[out_i] = b'.';
                self.out_buf[out_i + 1] = (mote.tint.r * fade).clamp(0.0, 255.0) as u8;
                self.out_buf[out_i + 2] = (mote.tint.g * fade).clamp(0.0, 255.0) as u8;
                self.out_buf[out_i + 3] = (mote.tint.b * fade).clamp(0.0, 255.0) as u8;
            }
        }
    }
}

impl Vec3 {
    #[inline]
    fn lerp_pos(self, o: Self, t: f32) -> Self {
        Self::new(
            self.x + (o.x - self.x) * t,
            self.y + (o.y - self.y) * t,
            self.z + (o.z - self.z) * t,
        )
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
    let open_bell = 0.44 + 0.56 * bloom;
    let tip_flare = if is_outer {
        0.26 * bloom * v.powf(2.1)
    } else {
        0.11 * bloom * v.powf(2.4)
    };
    let whorl_offset = if is_outer {
        0.026 * (1.0 - 0.30 * v)
    } else {
        -0.014
    };
    let base_r = radius_scale * (0.09 + open_bell * 0.74 * belly + tip_flare) + whorl_offset;

    let sin_vpi = (v * PI).sin();
    let spoon_amp = 0.072 * radius_scale * sin_vpi;
    let rib_amp = 0.012 * radius_scale * sin_vpi;

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
    let base_r = r_scale * 0.08 + span * arc_r * st;

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
    picotee: f32,
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
    let col = palette.sample(base_shade, v_cup, spec, rim, picotee);

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

#[inline]
fn ease_in_out_sine(x: f32) -> f32 {
    0.5 * (1.0 - (PI * x.clamp(0.0, 1.0)).cos())
}

#[inline]
fn ease_out_back_soft(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    let c1 = 0.85_f32;
    let c3 = c1 + 1.0;
    let t = x - 1.0;
    (1.0 + c3 * t * t * t + c1 * t * t).clamp(0.0, 1.06)
}

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ============================================================================
// WebAssembly FFI exports (zero external dependencies, static GitHub Pages)
// ============================================================================

static mut ENGINE: Option<BouquetRenderer> = None;

#[allow(static_mut_refs)]
fn engine_mut() -> &'static mut BouquetRenderer {
    unsafe { ENGINE.get_or_insert_with(|| BouquetRenderer::new(152, 74)) }
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

    fn count_bouquet_cells(renderer: &BouquetRenderer) -> ([usize; 8], usize, usize) {
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
        (glyph_counts, stars, bouquet_cells)
    }

    #[test]
    fn renders_high_detail_pastel_bouquet_and_choreographed_opening() {
        let mut renderer = BouquetRenderer::new(150, 72);

        // Act I (0.65s): stems & leaves are actively growing upward
        renderer.trigger_bloom();
        for _ in 0..13 {
            renderer.render(0.05);
        }
        let (_, _, act1_cells) = count_bouquet_cells(&renderer);

        // Act III (2.4s): petals actively unfurling
        for _ in 0..35 {
            renderer.render(0.05);
        }
        let (_, _, act3_cells) = count_bouquet_cells(&renderer);

        // Act IV+ (5.2s): full lush bloom with gypsophila & ribbon
        for _ in 0..56 {
            renderer.render(0.05);
        }
        let (glyphs, stars, full_cells) = count_bouquet_cells(&renderer);

        assert!(act1_cells > 120, "Act I should show growing stems/leaves ({})", act1_cells);
        assert!(act3_cells > act1_cells + 500, "Act III should bloom significantly ({} -> {})", act1_cells, act3_cells);
        assert!(full_cells > act3_cells, "Full bloom should expand further ({} -> {})", act3_cells, full_cells);
        assert!(full_cells > 2800, "Expected rich high-detail bouquet ({})", full_cells);
        assert!(stars > 60, "Expected rich starry background ({})", stars);
        for (i, &cnt) in glyphs.iter().enumerate() {
            assert!(cnt > 30, "Symbol '{}' underrepresented ({})", DENSITY_RAMP[i] as char, cnt);
        }

        let preview = String::from_utf8(renderer.build_ascii_text().to_vec()).unwrap();
        println!("=== HIGH-DETAIL BOUQUET (150x72) ===\n{}", preview);
        println!(
            "Opening progression: Act1={} -> Act3={} -> Full={} | glyphs={:?} | stars={}",
            act1_cells, act3_cells, full_cells, glyphs, stars
        );
    }

    #[test]
    fn fits_cleanly_within_portrait_phone_grid_without_horizontal_clipping() {
        let mut renderer = BouquetRenderer::new(88, 86);
        renderer.set_char_aspect(0.54);
        renderer.set_bloom_instant();
        renderer.render(0.016);

        let cols = renderer.cols();
        let rows = renderer.rows();
        let bytes = renderer.out_bytes();
        let mut bouquet_cells = 0usize;
        let mut left_edge = 0usize;
        let mut right_edge = 0usize;

        for row in 0..rows {
            for col in 0..cols {
                let g = bytes[(row * cols + col) * 4];
                if g != b' ' && g != b'.' {
                    bouquet_cells += 1;
                    if col == 0 {
                        left_edge += 1;
                    }
                    if col + 1 == cols {
                        right_edge += 1;
                    }
                }
            }
        }

        assert!(bouquet_cells > 1100, "Portrait mobile bouquet too sparse ({})", bouquet_cells);
        assert_eq!(left_edge, 0, "Bouquet clipped on left phone edge");
        assert_eq!(right_edge, 0, "Bouquet clipped on right phone edge");
    }
}
