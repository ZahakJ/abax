// Abax soroban renderer: one pipeline, one instanced draw per soroban.
//
// Every part of the instrument (board, well, rods, beam, unit dots, glows,
// bead shadows and beads) is one instance of a unit quad, shaded per pixel
// from a small signed-distance shape plus a lighting model. Positions are in
// egui points relative to the callback origin; the vertex shader maps them
// to the full-screen viewport set in `paint`.
//
// Colours arrive as linear RGB. The fragment shader works in linear light
// and encodes at the end: linear output for sRGB targets, gamma-encoded
// for the non-sRGB targets egui prefers. Output is premultiplied alpha,
// blended exactly like egui's own meshes.

struct Globals {
    screen_px: vec2<f32>,
    ppp: f32,
    time: f32,
    origin: vec2<f32>,
    srgb_target: u32,
    _pad: u32,
    // Key light direction, y up, z toward the viewer.
    light: vec4<f32>,
};

@group(0) @binding(0) var<uniform> g: Globals;

const K_BOARD: u32 = 0u;
const K_WELL: u32 = 1u;
const K_ACTIVE_GLOW: u32 = 2u;
const K_ROD: u32 = 3u;
const K_BEAM: u32 = 4u;
const K_DOT: u32 = 5u;
const K_UNDERLINE: u32 = 6u;
const K_HINT_GLOW: u32 = 7u;
const K_SHADOW: u32 = 8u;
const K_BEAD: u32 = 9u;

// Bead silhouette exponent: a superellipse below 2 gives the biconvex lens
// of a soroban bead, pointed left and right with a softened equator.
const LENS_P: f32 = 1.55;

struct Inst {
    @location(0) rect: vec4<f32>,
    @location(1) kind: vec4<u32>,
    @location(2) params: vec4<f32>,
    @location(3) ca: vec4<f32>,
    @location(4) cb: vec4<f32>,
    @location(5) cc: vec4<f32>,
};

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    // Points relative to the instance rect centre (y down).
    @location(0) local: vec2<f32>,
    // Points relative to the callback origin (for grain).
    @location(1) world: vec2<f32>,
    @location(2) @interpolate(flat) half: vec2<f32>,
    @location(3) @interpolate(flat) kind: vec4<u32>,
    @location(4) @interpolate(flat) params: vec4<f32>,
    @location(5) @interpolate(flat) ca: vec4<f32>,
    @location(6) @interpolate(flat) cb: vec4<f32>,
    @location(7) @interpolate(flat) cc: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, inst: Inst) -> VsOut {
    // Two triangles: (0,0) (1,0) (0,1) / (0,1) (1,0) (1,1).
    let cx = select(0.0, 1.0, vi == 1u || vi == 4u || vi == 5u);
    let cy = select(0.0, 1.0, vi == 2u || vi == 3u || vi == 5u);
    // Grow every quad by 1.5 px for the anti-aliased edge; shadows also by
    // their blur radius.
    var pad = 1.5 / g.ppp;
    if inst.kind.x == K_SHADOW {
        pad = pad + inst.params.x;
    }
    let mn = inst.rect.xy - vec2<f32>(pad);
    let sz = inst.rect.zw + vec2<f32>(2.0 * pad);
    let p = mn + vec2<f32>(cx, cy) * sz;
    let centre = inst.rect.xy + inst.rect.zw * 0.5;
    let px = (g.origin + p) * g.ppp;
    var out: VsOut;
    out.pos = vec4<f32>(px.x / g.screen_px.x * 2.0 - 1.0, 1.0 - px.y / g.screen_px.y * 2.0, 0.0, 1.0);
    out.local = p - centre;
    out.world = p;
    out.half = inst.rect.zw * 0.5;
    out.kind = inst.kind;
    out.params = inst.params;
    out.ca = inst.ca;
    out.cb = inst.cb;
    out.cc = inst.cc;
    return out;
}

// ---------------------------------------------------------------- helpers

fn hash2(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash2(i);
    let b = hash2(i + vec2<f32>(1.0, 0.0));
    let c = hash2(i + vec2<f32>(0.0, 1.0));
    let d = hash2(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn sd_round_box(p: vec2<f32>, half: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half + vec2<f32>(r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

// Coverage of a signed distance in points; `fw` is points per pixel.
fn cover(d: f32, fw: f32) -> f32 {
    return clamp(0.5 - d / fw, 0.0, 1.0);
}

// Wood grain in 0..1 (1 = a dark grain line). Lines run along x, the long
// axis of the frame, wavering slowly; `scale` > 1 packs them finer.
fn wood(w: vec2<f32>, scale: f32) -> vec2<f32> {
    let y = w.y * scale;
    let warp = 2.6 * sin(w.x * 0.009 + y * 0.045) + 3.0 * vnoise(vec2<f32>(w.x * 0.006, y * 0.05));
    let rings = 0.5 + 0.5 * sin(y * 0.62 + warp * 1.4);
    let fine = 0.5 + 0.5 * sin(y * 2.1 + warp * 2.3 + vnoise(vec2<f32>(w.x * 0.02, y * 0.4)) * 2.0);
    let fibre = vnoise(vec2<f32>(w.x * 0.035, y * 1.3));
    let lines = smoothstep(0.55, 0.95, rings) * 0.7 + smoothstep(0.7, 1.0, fine) * 0.3;
    return vec2<f32>(clamp(lines * (0.6 + 0.6 * fibre), 0.0, 1.0), fibre);
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

// ---------------------------------------------------------------- parts

// Lacquered board: grain, a lit/shaded bevel, rounded corners and a soft
// diagonal sheen. params: radius, bevel, sheen, grain; cc.a: grain scale.
fn shade_board(in: VsOut, fw: f32) -> vec4<f32> {
    let r = in.params.x;
    let bevel = in.params.y;
    let d = sd_round_box(in.local, in.half, r);
    let cov = cover(d, fw);
    let wg = wood(in.world, in.cc.a);
    var col = mix(in.ca.rgb, in.cc.rgb, wg.x * in.params.w);
    col = mix(col, in.cb.rgb, (wg.y - 0.5) * 0.12 * in.params.w);
    // Bevel: the outward normal of the nearest edge against a top-left light.
    let inside = -d;
    let eps = 0.75;
    let grad = normalize(vec2<f32>(
        sd_round_box(in.local + vec2<f32>(eps, 0.0), in.half, r) - sd_round_box(in.local - vec2<f32>(eps, 0.0), in.half, r),
        sd_round_box(in.local + vec2<f32>(0.0, eps), in.half, r) - sd_round_box(in.local - vec2<f32>(0.0, eps), in.half, r)) + vec2<f32>(1e-5));
    let lit = dot(grad, normalize(vec2<f32>(-0.55, -0.85)));
    let band = 1.0 - smoothstep(0.0, bevel, inside);
    col = select(mix(col, col * 0.45, band * -lit * 0.85), mix(col, in.cb.rgb * 1.15, band * lit * 0.8), lit > 0.0);
    // A hairline catch-light right at the outer edge.
    col = col + in.cb.rgb * 0.25 * (1.0 - smoothstep(0.0, 1.2, inside)) * max(lit, 0.0);
    // Lacquer sheen: a broad soft diagonal band, brighter toward the top.
    let uv = in.local / in.half;
    let diag = uv.x * 0.35 + uv.y;
    let sheen = exp(-pow((diag + 0.55) / 0.5, 2.0)) * in.params.z;
    col = col + in.cb.rgb * sheen * 0.7;
    return vec4<f32>(col, cov);
}

// The recessed well: darker lacquer, an inner shadow cast by the frame
// (stronger on the lit top-left side), a vignette and the shadow under the
// beam. params: radius, beam top, beam bottom (local y), grain.
fn shade_well(in: VsOut, fw: f32) -> vec4<f32> {
    let r = in.params.x;
    let d = sd_round_box(in.local, in.half, r);
    let cov = cover(d, fw);
    let wg = wood(in.world + vec2<f32>(37.0, 11.0), in.cc.a);
    var col = mix(in.ca.rgb, in.cc.rgb, wg.x * in.params.w * 0.8);
    let inside = -d;
    // Inner shadow: the frame lip occludes light near every edge; the top
    // and left edges face away from the light and get a wider shadow.
    let top = in.local.y + in.half.y;
    let left = in.local.x + in.half.x;
    let sh_edge = 1.0 - smoothstep(0.0, 7.0, inside);
    let sh_cast = (1.0 - smoothstep(0.0, 12.0, top)) * 0.6 + (1.0 - smoothstep(0.0, 9.0, left)) * 0.4;
    col = col * (1.0 - 0.45 * sh_edge - 0.35 * sh_cast);
    // Vignette toward the corners.
    let uv = in.local / in.half;
    let vig = smoothstep(0.55, 1.45, length(uv * vec2<f32>(0.75, 1.0)));
    col = col * (1.0 - 0.35 * vig);
    // The beam floats just above the well: a soft shadow below it.
    let below = in.local.y - in.params.z;
    let beam_sh = select(0.0, exp(-below / 5.0), below > -0.5);
    col = col * (1.0 - 0.45 * beam_sh);
    // Lit lower/right lip.
    let bottom = in.half.y - in.local.y;
    let right = in.half.x - in.local.x;
    let lip = (1.0 - smoothstep(0.0, 1.5, bottom)) + (1.0 - smoothstep(0.0, 1.5, right)) * 0.5;
    col = col + in.cb.rgb * 0.18 * clamp(lip, 0.0, 1.0);
    return vec4<f32>(col, cov);
}

// A thin cylinder: bamboo, brass or steel. variant (kind.y): 0 bamboo,
// 1 brass, 2 steel. ca: base, cb: highlight colour.
fn shade_metal_or_bamboo(variant: u32, base: vec3<f32>, hi: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let L = normalize(g.light.xyz);
    let H = normalize(L + vec3<f32>(0.0, 0.0, 1.0));
    let ndl = dot(n, L);
    let ndh = max(dot(n, H), 0.0);
    if variant == 0u {
        let diff = max((ndl + 0.4) / 1.4, 0.0);
        let spec = pow(ndh, 9.0) * 0.22;
        return base * (0.42 + 0.7 * diff) + hi * spec;
    }
    // Metal: environment-ish banding plus a narrow bright highlight.
    let x = n.x;
    let band = exp(-pow((x - 0.38) / 0.16, 2.0));
    let env = 0.55 + 0.45 * smoothstep(-1.0, -0.2, x) - 0.5 * band;
    let spec = pow(ndh, 48.0) * 1.0 + pow(ndh, 8.0) * 0.14;
    let tint = select(vec3<f32>(1.0, 0.92, 0.75), vec3<f32>(0.92, 0.97, 1.05), variant == 2u);
    return base * env * (0.45 + 0.6 * max(ndl, 0.0)) + hi * tint * spec;
}

fn shade_rod(in: VsOut, fw: f32) -> vec4<f32> {
    let xr = clamp(in.local.x / in.half.x, -1.0, 1.0);
    let n = vec3<f32>(xr, 0.0, sqrt(max(1.0 - xr * xr, 0.0)));
    let d = abs(in.local.x) - in.half.x;
    let cov = cover(d, fw) * cover(abs(in.local.y) - in.half.y, fw);
    var col = shade_metal_or_bamboo(in.kind.y, in.ca.rgb, in.cb.rgb, n);
    // Occlusion where the rod enters the frame.
    let end = in.half.y - abs(in.local.y);
    col = col * (0.55 + 0.45 * smoothstep(0.0, 8.0, end));
    return vec4<f32>(col, cov);
}

// The reckoning bar. variant 0: lacquered wood with a bevel and faint
// grain; 1: metal. ca: beam, cb: beam light, cc: grain dark (a: scale).
fn shade_beam(in: VsOut, fw: f32) -> vec4<f32> {
    let d = sd_round_box(in.local, in.half, 1.5);
    let cov = cover(d, fw);
    let v = in.local.y / in.half.y;
    var col: vec3<f32>;
    if in.kind.y == 1u {
        // A rounded metal bar: the normal tilts up toward the top edge.
        // The cylinder helper bands on n.x, so feed it the tilt there.
        let n = normalize(vec3<f32>(v * 0.7, -v * 0.7, 1.0));
        col = shade_metal_or_bamboo(1u, in.ca.rgb, in.cb.rgb, n);
        // Brushed streaks.
        col = col * (0.94 + 0.08 * vnoise(vec2<f32>(in.world.x * 0.05, in.world.y * 3.0)));
    } else {
        let wg = wood(in.world * vec2<f32>(1.0, 1.6), in.cc.a * 1.4);
        col = mix(in.ca.rgb, in.cc.rgb, wg.x * 0.22);
        col = col * (1.04 - 0.16 * v);
    }
    // Bevel: lit top edge, shaded bottom edge (2.5 pt).
    let top = in.local.y + in.half.y;
    let bottom = in.half.y - in.local.y;
    col = mix(col, in.cb.rgb * 1.12, (1.0 - smoothstep(0.0, 2.5, top)) * 0.75);
    col = col * (1.0 - 0.45 * (1.0 - smoothstep(0.0, 2.5, bottom)));
    return vec4<f32>(col, cov);
}

// A unit dot: a small inset pin. ca: ring (dark), cb: pin (light).
fn shade_dot(in: VsOut, fw: f32) -> vec4<f32> {
    let rad = in.half.x;
    let l = length(in.local);
    let cov = cover(l - rad, fw);
    let pin = 1.0 - smoothstep(rad * 0.12, rad * 0.5, length(in.local + vec2<f32>(rad * 0.2, rad * 0.2)));
    var col = mix(in.ca.rgb, in.cb.rgb, pin * 0.8);
    // Recessed: the lower-right inner lip catches light.
    let lip = smoothstep(rad * 0.55, rad * 0.95, l) * smoothstep(0.0, 0.8, (in.local.x + in.local.y) / rad);
    col = col + in.cb.rgb * 0.25 * lip;
    return vec4<f32>(col, cov);
}

// Soft accent strip along the active rod. ca: accent (a: strength).
fn shade_active_glow(in: VsOut) -> vec4<f32> {
    let x = in.local.x / in.half.x;
    let y = abs(in.local.y) / in.half.y;
    let a = exp(-x * x * 3.2) * (1.0 - smoothstep(0.7, 1.0, y)) * in.ca.a;
    return vec4<f32>(in.ca.rgb, a);
}

fn shade_underline(in: VsOut, fw: f32) -> vec4<f32> {
    let d = sd_round_box(in.local, in.half, min(in.half.x, in.half.y));
    return vec4<f32>(in.ca.rgb, cover(d, fw) * in.ca.a);
}

// Radial halo behind a bead to move. ca: accent (a: strength incl. pulse).
fn shade_hint_glow(in: VsOut) -> vec4<f32> {
    let d = length(in.local / in.half);
    let a = exp(-d * d * 2.6) * (1.0 - smoothstep(0.75, 1.0, d)) * in.ca.a;
    return vec4<f32>(in.ca.rgb, a);
}

// Implicit lens: returns (approximate signed distance in points, t = |y|/H).
fn lens_sd(p: vec2<f32>, half: vec2<f32>) -> f32 {
    let ax = max(abs(p.x), 1e-4) / half.x;
    let ay = max(abs(p.y), 1e-4) / half.y;
    let f = pow(ax, LENS_P) + pow(ay, LENS_P) - 1.0;
    let gx = LENS_P * pow(ax, LENS_P - 1.0) / half.x;
    let gy = LENS_P * pow(ay, LENS_P - 1.0) / half.y;
    return f / max(length(vec2<f32>(gx, gy)), 1e-4);
}

// Contact shadow: the lens silhouette, blurred. params: blur, alpha.
fn shade_shadow(in: VsOut) -> vec4<f32> {
    let d = lens_sd(in.local, in.half);
    let blur = max(in.params.x, 0.5);
    let a = (1.0 - smoothstep(-blur, blur, d)) * in.params.y;
    return vec4<f32>(0.0, 0.0, 0.0, a);
}

// The bead: a surface of revolution about the rod. Returns premultiplied
// linear colour.
// params: gap above, gap below (points), flash (0..1), bore half-width.
// ca: albedo (a: body alpha), cb: shadow tint (a: highlight alpha),
// cc: specular colour (a: overall alpha). kind.y: 0 bone, 1 ebony, 2 slate.
fn shade_bead(in: VsOut, fw: f32) -> vec4<f32> {
    let W = in.half.x;
    let Hh = in.half.y;
    let p = in.local;
    let d = lens_sd(p, in.half);
    let cov = cover(d, fw);
    let t = min(abs(p.y) / Hh, 0.995);
    let k = max(1.0 - pow(t, LENS_P), 1e-4);
    let r = W * pow(k, 1.0 / LENS_P);
    // dr/dt for the superellipse profile, then the slope in y-up points.
    let drdt = -W * pow(max(t, 1e-4), LENS_P - 1.0) * pow(k, 1.0 / LENS_P - 1.0);
    let yup = -p.y;
    // Soften the ridge at the equator a little (real beads are slightly
    // rounded there), so the lit upper cone blends into the shaded lower.
    let drdy = drdt * sign(yup) / Hh * mix(0.3, 1.0, smoothstep(0.0, 0.28, t));
    let x = clamp(p.x, -r, r);
    let z = sqrt(max(r * r - x * x, 0.0));
    // Flatten the depth so the bead reads as a lens, not a ball.
    var n = normalize(vec3<f32>(x / W, -r * drdy / W, z / W * 0.9));

    // Material constants by kind.
    var wrap = 0.35;
    var shin = 60.0;
    var spec_k = 0.75;
    var broad = 0.10;
    if in.kind.y == 1u {
        wrap = 0.1;
        shin = 40.0;
        spec_k = 0.55;
        broad = 0.12;
    } else if in.kind.y == 2u {
        wrap = 0.12;
        shin = 18.0;
        spec_k = 0.32;
        broad = 0.10;
    }
    let L = normalize(g.light.xyz);
    let V = vec3<f32>(0.0, 0.0, 1.0);
    let ndl = dot(n, L);
    let diff = max((ndl + wrap) / (1.0 + wrap), 0.0);
    let Hv = normalize(L + V);
    let ndh = max(dot(n, Hv), 0.0);
    let spec = pow(ndh, shin) * spec_k + pow(ndh, 6.0) * broad;
    let fres = pow(1.0 - max(n.z, 0.0), 3.0);
    let sky = 0.5 + 0.5 * n.y;

    let albedo = in.ca.rgb;
    let tint = in.cb.rgb;
    // Shaded regions lean toward the warmer shadow tint, as bone does.
    var col = mix(tint * 0.62, albedo, clamp(diff * 1.1, 0.0, 1.0));
    col = col * (0.7 + 0.3 * sky) + albedo * 0.06;
    // Rim darkening at grazing angles, and a slightly darker lower half.
    col = col * (1.0 - 0.38 * fres);
    col = col * (1.0 - 0.10 * smoothstep(0.0, 1.0, -yup / Hh));

    // Ambient occlusion where a neighbour or the beam touches.
    let near_top = smoothstep(0.55, 0.98, t) * select(0.0, 1.0, yup > 0.0);
    let near_bot = smoothstep(0.55, 0.98, t) * select(0.0, 1.0, yup < 0.0);
    let touch_top = 1.0 - smoothstep(0.0, 3.0, in.params.x);
    let touch_bot = 1.0 - smoothstep(0.0, 3.0, in.params.y);
    let centre = 1.0 - 0.5 * abs(x) / W;
    col = col * (1.0 - 0.42 * (near_top * touch_top + near_bot * touch_bot) * centre);

    // Bore: the rod enters at the top and bottom centre.
    let bore = (1.0 - smoothstep(in.params.w * 0.7, in.params.w * 1.5, abs(p.x))) * smoothstep(0.8, 0.97, t);
    col = col * (1.0 - 0.45 * bore);

    // Click flash: brief brightening.
    col = col + (albedo * 0.35 + vec3<f32>(0.04)) * in.params.z;

    // Highlights stay longest when the bead turns to glass.
    let hl = in.cc.rgb * spec + in.cc.rgb * fres * 0.35 * (1.0 - in.ca.a);
    let body_a = in.ca.a;
    let hl_a = in.cb.a;
    let overall = in.cc.a * cov;
    let pm = col * body_a + hl * hl_a;
    let a = clamp(body_a + luma(hl) * hl_a, 0.0, 1.0);
    return vec4<f32>(pm, a) * overall;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Points per pixel. Taken once here, in uniform control flow (WGSL
    // forbids derivatives inside the per-kind branches below).
    let fw = max(fwidth(in.local.x), 1e-4);
    // Straight (unpremultiplied) linear colour + alpha, except beads which
    // return premultiplied.
    var c = vec4<f32>(0.0);
    var premult = false;
    let kind = in.kind.x;
    if kind == K_BOARD {
        c = shade_board(in, fw);
    } else if kind == K_WELL {
        c = shade_well(in, fw);
    } else if kind == K_ACTIVE_GLOW {
        c = shade_active_glow(in);
    } else if kind == K_ROD {
        c = shade_rod(in, fw);
    } else if kind == K_BEAM {
        c = shade_beam(in, fw);
    } else if kind == K_DOT {
        c = shade_dot(in, fw);
    } else if kind == K_UNDERLINE {
        c = shade_underline(in, fw);
    } else if kind == K_HINT_GLOW {
        c = shade_hint_glow(in);
    } else if kind == K_SHADOW {
        c = shade_shadow(in);
    } else {
        c = shade_bead(in, fw);
        premult = true;
    }
    var a = clamp(c.a, 0.0, 1.0);
    var rgb = c.rgb;
    if premult {
        rgb = select(vec3<f32>(0.0), rgb / max(a, 1e-4), a > 1e-4);
    }
    rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    if g.srgb_target == 0u {
        rgb = to_srgb(rgb);
    }
    return vec4<f32>(rgb * a, a);
}
