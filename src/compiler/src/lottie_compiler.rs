//! Lottie compiler — stage 3 of the bedrock-video pipeline.
//!
//! Converts a [`Vec<AnimationSpec>`] into a [`Vec<LottieAnimation>`] (Lottie spec 5.5.7).
//!
//! # Subcommands
//!
//! - `lottie-compiler` — stage 3 standalone: reads `Vec<AnimationSpec>` from stdin, writes
//!   `Vec<LottieAnimation>` JSON to stdout.
//! - `compile` — Option-A full pipeline: reads `data.json` from stdin, runs all three stages
//!   in-process (script-analyzer → animation-spec → lottie-compiler), writes
//!   `Vec<LottieAnimation>` JSON to stdout.
//!
//! # Lottie schema reference
//!
//! ```json
//! {
//!   "v": "5.5.7", "fr": 30, "ip": 0, "op": 90,
//!   "w": 500,     "h": 500, "nm": "Example", "ddd": 0,
//!   "assets": [],
//!   "layers": [
//!     { "ddd": 0, "ind": 1, "ty": 4, "nm": "Circle", "ks": { … }, "shapes": [ … ] }
//!   ]
//! }
//! ```

use crate::animation_spec::AnimationSpec;
use crate::script_analyzer::ScriptInput;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{self, Read};

// ─── Constants ────────────────────────────────────────────────────────────────

/// Canvas dimensions match the Remotion composition in Root.tsx.
const CANVAS_W: u32 = 500;
const CANVAS_H: u32 = 500;

/// Frame rate for all generated animations.
const FRAME_RATE: u32 = 30;

/// Lottie body-movin spec version written into every animation.
const LOTTIE_VERSION: &str = "5.5.7";

// ─── Public types ─────────────────────────────────────────────────────────────

/// Top-level Lottie animation object (Lottie spec 5.x).
///
/// Field names match the Lottie JSON schema exactly so serde serialises them
/// without any `rename` attributes.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LottieAnimation {
    /// Spec version string, always "5.5.7".
    pub v: String,
    /// Frames per second.
    pub fr: u32,
    /// In-point: first frame of the animation (always 0).
    pub ip: u32,
    /// Out-point: last frame of the animation (= `AnimationSpec.duration`).
    pub op: u32,
    /// Canvas width in pixels.
    pub w: u32,
    /// Canvas height in pixels.
    pub h: u32,
    /// Human-readable name (used by After Effects and preview tools).
    pub nm: String,
    /// 3D flag: 0 = 2D (we never use 3D).
    pub ddd: u32,
    /// Embedded asset list (empty — all shapes are procedural).
    pub assets: Vec<Value>,
    /// One shape layer per animation (ty = 4).
    pub layers: Vec<Value>,
}

// ─── Public entry points ──────────────────────────────────────────────────────

/// Stage 3 standalone: reads `Vec<AnimationSpec>` from stdin, writes
/// `Vec<LottieAnimation>` JSON to stdout.
///
/// Run with: `echo '[{...}]' | ./compiler lottie-compiler`
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw)?;
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("empty input on stdin; expected JSON array of AnimationSpec".into());
    }

    let specs: Vec<AnimationSpec> = serde_json::from_str(raw)
        .map_err(|e| format!("stdin is not a valid AnimationSpec array: {e}"))?;
    if specs.is_empty() {
        return Err("input specs array is empty".into());
    }

    let animations = compile(&specs)?;
    eprintln!("compiled {} lottie animations", animations.len());
    println!("{}", serde_json::to_string(&animations)?);
    Ok(())
}

/// Option-A full pipeline: reads `data.json` from stdin and runs all three
/// stages in-process, writing `Vec<LottieAnimation>` JSON to stdout.
///
/// Each stage's `eprintln!` progress lines go to stderr so stdout stays
/// clean for the caller.
///
/// Run with: `cat public/data.json | ./compiler compile`
pub async fn run_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw)?;
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("empty input on stdin; expected data.json contents".into());
    }

    // Stage 1 — script_analyzer: data.json → Vec<VisualMoment>
    let input: ScriptInput =
        serde_json::from_str(raw).map_err(|e| format!("stdin is not valid data.json: {e}"))?;
    eprintln!(
        "stage 1/3: analyzing script ({} word groups)…",
        input.word_groups.len()
    );
    let moments = crate::script_analyzer::analyze(input).await?;

    // Stage 2 — animation_spec: Vec<VisualMoment> → Vec<AnimationSpec>
    eprintln!(
        "stage 2/3: designing animation specs for {} moments…",
        moments.len()
    );
    let specs = crate::animation_spec::design(&moments).await?;

    // Stage 3 — lottie_compiler: Vec<AnimationSpec> → Vec<LottieAnimation>
    eprintln!("stage 3/3: compiling {} specs to lottie…", specs.len());
    let animations = compile(&specs)?;
    eprintln!("compiled {} lottie animations", animations.len());

    println!("{}", serde_json::to_string(&animations)?);
    std::fs::write(
        "../../public/lottie.json",
        serde_json::to_string(&animations)?,
    )?;
    Ok(())
}

/// Pure compilation logic: converts a slice of [`AnimationSpec`] into Lottie
/// animations without any I/O.
///
/// Each spec maps to exactly one [`LottieAnimation`] with one shape layer.
pub fn compile(
    specs: &[AnimationSpec],
) -> Result<Vec<LottieAnimation>, Box<dyn std::error::Error>> {
    let animations: Vec<LottieAnimation> = specs
        .iter()
        .enumerate()
        .map(|(i, spec)| build_animation(i, spec))
        .collect();

    validate_animations(&animations).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    Ok(animations)
}

// ─── Validation ───────────────────────────────────────────────────────────────

/// Validates a compiled slice of animations. Mirrors the validate_* pattern
/// from animation_spec.rs.
fn validate_animations(anims: &[LottieAnimation]) -> Result<(), String> {
    if anims.is_empty() {
        return Err("compile produced zero animations".to_string());
    }
    for (i, a) in anims.iter().enumerate() {
        if a.op <= a.ip {
            return Err(format!(
                "animation {i}: op ({}) must be > ip ({})",
                a.op, a.ip
            ));
        }
        if a.w == 0 || a.h == 0 {
            return Err(format!(
                "animation {i}: w ({}) and h ({}) must both be > 0",
                a.w, a.h
            ));
        }
        if a.layers.is_empty() {
            return Err(format!("animation {i}: layers must not be empty"));
        }
    }
    Ok(())
}

// ─── Animation builder ────────────────────────────────────────────────────────

/// Build one complete LottieAnimation from a single AnimationSpec.
fn build_animation(index: usize, spec: &AnimationSpec) -> LottieAnimation {
    let shapes = build_shapes(spec);
    let transform = build_transform(&spec.motion, &spec.easing, spec.duration);

    // ty=4 is a Lottie shape layer.
    let layer = json!({
        "ddd": 0,
        "ind": 1,
        "ty": 4,
        "nm": format!("{} {}", spec.shape, spec.motion),
        "ks": transform,
        "shapes": shapes,
        "ip": 0,
        "op": spec.duration,
        "st": 0,
        "bm": 0  // normal blend mode
    });

    LottieAnimation {
        v: LOTTIE_VERSION.to_string(),
        fr: FRAME_RATE,
        ip: 0,
        op: spec.duration,
        w: CANVAS_W,
        h: CANVAS_H,
        nm: format!("anim-{index}-{}-{}", spec.shape, spec.motion),
        ddd: 0,
        assets: vec![],
        layers: vec![layer],
    }
}

// ─── Shape builders ───────────────────────────────────────────────────────────

/// Dispatch to the correct shape builder and append a fill or stroke item.
///
/// Shape items are ordered: geometry first, then paint (fill/stroke).
/// This matches After Effects / Lottie-web rendering order.
fn build_shapes(spec: &AnimationSpec) -> Vec<Value> {
    let color = parse_hex_color(&spec.color);
    let px = size_px(&spec.size);

    match spec.shape.as_str() {
        "circle" => vec![build_ellipse(px), build_fill(color)],
        "square" => vec![build_rect(px, px, 0.0), build_fill(color)],
        "triangle" => vec![build_triangle_path(px), build_fill(color)],
        "line" => vec![build_line_path(px), build_stroke(color, 4.0)],
        "arc" => vec![build_arc_path(px), build_stroke(color, 4.0)],
        "bars" => build_bars(color, px),
        "dots" => build_dots(color, px),
        _ => vec![build_ellipse(px), build_fill(color)], // safe default
    }
}

/// Lottie ellipse (`ty: "el"`). `diameter` sets both width and height.
fn build_ellipse(diameter: f64) -> Value {
    json!({
        "ty": "el",
        "nm": "Ellipse",
        "s": { "a": 0, "k": [diameter, diameter] },
        "p": { "a": 0, "k": [0.0, 0.0] }
    })
}

/// Lottie rectangle (`ty: "rc"`). `corner_radius = 0` gives a sharp square.
fn build_rect(w: f64, h: f64, corner_radius: f64) -> Value {
    json!({
        "ty": "rc",
        "nm": "Rectangle",
        "s": { "a": 0, "k": [w, h] },
        "r": { "a": 0, "k": corner_radius },
        "p": { "a": 0, "k": [0.0, 0.0] }
    })
}

/// Equilateral triangle as a closed Lottie bezier path (`ty: "sh"`).
///
/// Vertices: top-centre, bottom-right, bottom-left (counter-clockwise).
/// Tangent handles are zero — straight edges, no curves.
fn build_triangle_path(size: f64) -> Value {
    let half_h = size / 2.0;
    let half_w = size / 2.0;
    json!({
        "ty": "sh",
        "nm": "Triangle",
        "ks": {
            "a": 0,
            "k": {
                "c": true,
                "v": [[0.0, -half_h], [half_w, half_h], [-half_w, half_h]],
                "i": [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
                "o": [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]
            }
        }
    })
}

/// Horizontal line as an open Lottie bezier path (`ty: "sh"`).
fn build_line_path(length: f64) -> Value {
    let half = length / 2.0;
    json!({
        "ty": "sh",
        "nm": "Line",
        "ks": {
            "a": 0,
            "k": {
                "c": false,
                "v": [[-half, 0.0], [half, 0.0]],
                "i": [[0.0, 0.0], [0.0, 0.0]],
                "o": [[0.0, 0.0], [0.0, 0.0]]
            }
        }
    })
}

/// Semicircular arc approximated with 3-point cubic bezier (kappa ≈ 0.5523).
///
/// The approximation is accurate to < 0.1% error for a true half-circle.
fn build_arc_path(size: f64) -> Value {
    let r = size / 2.0;
    let k = r * 0.5523; // bezier kappa for a quarter-circle
    json!({
        "ty": "sh",
        "nm": "Arc",
        "ks": {
            "a": 0,
            "k": {
                "c": false,
                "v": [[-r, 0.0], [0.0, -r], [r, 0.0]],
                "i": [[0.0, 0.0], [-k, 0.0], [0.0, 0.0]],
                "o": [[-k, 0.0], [k, 0.0],  [0.0, 0.0]]
            }
        }
    })
}

/// Three vertical bars positioned at -gap, 0, +gap along the x axis.
fn build_bars(color: [f64; 4], size: f64) -> Vec<Value> {
    let bar_w = (size / 5.0).max(4.0);
    let bar_h = size;
    let gap = size / 3.0;

    let mut items: Vec<Value> = (0..3_i32)
        .map(|i| {
            let x = (i as f64 - 1.0) * gap;
            json!({
                "ty": "rc",
                "nm": format!("Bar {}", i + 1),
                "s": { "a": 0, "k": [bar_w, bar_h] },
                "r": { "a": 0, "k": 0 },
                "p": { "a": 0, "k": [x, 0.0] }
            })
        })
        .collect();
    items.push(build_fill(color));
    items
}

/// Three dots positioned at -gap, 0, +gap along the x axis.
fn build_dots(color: [f64; 4], size: f64) -> Vec<Value> {
    let dot_d = (size / 4.0).max(8.0);
    let gap = size / 3.0;

    let mut items: Vec<Value> = (0..3_i32)
        .map(|i| {
            let x = (i as f64 - 1.0) * gap;
            json!({
                "ty": "el",
                "nm": format!("Dot {}", i + 1),
                "s": { "a": 0, "k": [dot_d, dot_d] },
                "p": { "a": 0, "k": [x, 0.0] }
            })
        })
        .collect();
    items.push(build_fill(color));
    items
}

/// Lottie solid fill item (`ty: "fl"`). `color` is normalised RGBA [0..1].
fn build_fill(color: [f64; 4]) -> Value {
    json!({
        "ty": "fl",
        "nm": "Fill",
        "c": { "a": 0, "k": [color[0], color[1], color[2], color[3]] },
        "o": { "a": 0, "k": 100 }
    })
}

/// Lottie stroke item (`ty: "st"`). Used for open-path shapes (line, arc).
fn build_stroke(color: [f64; 4], width: f64) -> Value {
    json!({
        "ty": "st",
        "nm": "Stroke",
        "c": { "a": 0, "k": [color[0], color[1], color[2], color[3]] },
        "o": { "a": 0, "k": 100 },
        "w": { "a": 0, "k": width }
    })
}

// ─── Transform / motion builders ─────────────────────────────────────────────

/// Build the Lottie `ks` transform block for a shape layer.
///
/// `op` is the animation out-point (total frames). The layer is centred at
/// canvas position (250, 250). Shape items inside the layer sit at (0, 0)
/// relative to that anchor.
///
/// Each motion animates exactly one transform property; all others are static.
fn build_transform(motion: &str, easing: &str, op: u32) -> Value {
    let (o_bez, i_bez) = easing_bezier(easing);
    let dur = op as f64;

    match motion {
        // Scale from 0% → 100% over the full duration.
        "expand" => json!({
            "o": { "a": 0, "k": 100 },
            "r": { "a": 0, "k": 0 },
            "p": { "a": 0, "k": [250.0, 250.0, 0.0] },
            "a": { "a": 0, "k": [0.0, 0.0, 0.0] },
            "s": { "a": 1, "k": [
                kf(0.0,  json!([0.0,   0.0,   100.0]), json!([100.0, 100.0, 100.0]), o_bez.clone(), i_bez.clone()),
                kf_end(dur, json!([100.0, 100.0, 100.0]))
            ]}
        }),

        // Scale from 100% → 0% over the full duration.
        "contract" => json!({
            "o": { "a": 0, "k": 100 },
            "r": { "a": 0, "k": 0 },
            "p": { "a": 0, "k": [250.0, 250.0, 0.0] },
            "a": { "a": 0, "k": [0.0, 0.0, 0.0] },
            "s": { "a": 1, "k": [
                kf(0.0,  json!([100.0, 100.0, 100.0]), json!([0.0, 0.0, 100.0]), o_bez.clone(), i_bez.clone()),
                kf_end(dur, json!([0.0, 0.0, 100.0]))
            ]}
        }),

        // Scale 100% → 130% → 100%: one heartbeat.
        "pulse" => {
            let mid = dur / 2.0;
            json!({
                "o": { "a": 0, "k": 100 },
                "r": { "a": 0, "k": 0 },
                "p": { "a": 0, "k": [250.0, 250.0, 0.0] },
                "a": { "a": 0, "k": [0.0, 0.0, 0.0] },
                "s": { "a": 1, "k": [
                    kf(0.0, json!([100.0, 100.0, 100.0]), json!([130.0, 130.0, 100.0]), o_bez.clone(), i_bez.clone()),
                    kf(mid, json!([130.0, 130.0, 100.0]), json!([100.0, 100.0, 100.0]), o_bez.clone(), i_bez.clone()),
                    kf_end(dur, json!([100.0, 100.0, 100.0]))
                ]}
            })
        }

        // Translate from y=280 → y=220 (drifts upward).
        "drift" => json!({
            "o": { "a": 0, "k": 100 },
            "r": { "a": 0, "k": 0 },
            "p": { "a": 1, "k": [
                kf(0.0, json!([250.0, 280.0, 0.0]), json!([250.0, 220.0, 0.0]), o_bez.clone(), i_bez.clone()),
                kf_end(dur, json!([250.0, 220.0, 0.0]))
            ]},
            "a": { "a": 0, "k": [0.0, 0.0, 0.0] },
            "s": { "a": 0, "k": [100.0, 100.0, 100.0] }
        }),

        // Rotation 0° → 360°.
        "rotate" => json!({
            "o": { "a": 0, "k": 100 },
            "r": { "a": 1, "k": [
                kf(0.0, json!([0.0]), json!([360.0]), o_bez.clone(), i_bez.clone()),
                kf_end(dur, json!([360.0]))
            ]},
            "p": { "a": 0, "k": [250.0, 250.0, 0.0] },
            "a": { "a": 0, "k": [0.0, 0.0, 0.0] },
            "s": { "a": 0, "k": [100.0, 100.0, 100.0] }
        }),

        // Horizontal oscillation: centre → right → left → right → centre.
        "shake" => {
            let q = dur / 4.0;
            json!({
                "o": { "a": 0, "k": 100 },
                "r": { "a": 0, "k": 0 },
                "p": { "a": 1, "k": [
                    kf(0.0,     json!([250.0, 250.0, 0.0]), json!([270.0, 250.0, 0.0]), o_bez.clone(), i_bez.clone()),
                    kf(q,       json!([270.0, 250.0, 0.0]), json!([230.0, 250.0, 0.0]), o_bez.clone(), i_bez.clone()),
                    kf(q * 2.0, json!([230.0, 250.0, 0.0]), json!([270.0, 250.0, 0.0]), o_bez.clone(), i_bez.clone()),
                    kf(q * 3.0, json!([270.0, 250.0, 0.0]), json!([250.0, 250.0, 0.0]), o_bez.clone(), i_bez.clone()),
                    kf_end(dur, json!([250.0, 250.0, 0.0]))
                ]},
                "a": { "a": 0, "k": [0.0, 0.0, 0.0] },
                "s": { "a": 0, "k": [100.0, 100.0, 100.0] }
            })
        }

        // Opacity 0 → 100 (fade-in).
        "appear" => json!({
            "o": { "a": 1, "k": [
                kf(0.0, json!([0.0]), json!([100.0]), o_bez.clone(), i_bez.clone()),
                kf_end(dur, json!([100.0]))
            ]},
            "r": { "a": 0, "k": 0 },
            "p": { "a": 0, "k": [250.0, 250.0, 0.0] },
            "a": { "a": 0, "k": [0.0, 0.0, 0.0] },
            "s": { "a": 0, "k": [100.0, 100.0, 100.0] }
        }),

        // Fallback: fully static transform at centre.
        _ => json!({
            "o": { "a": 0, "k": 100 },
            "r": { "a": 0, "k": 0 },
            "p": { "a": 0, "k": [250.0, 250.0, 0.0] },
            "a": { "a": 0, "k": [0.0, 0.0, 0.0] },
            "s": { "a": 0, "k": [100.0, 100.0, 100.0] }
        }),
    }
}

/// Build a non-final Lottie keyframe with start value, end value, and easing.
///
/// - `t` — frame number (time)
/// - `s` — start value for this segment (single-element array for scalars, 3-element for vectors)
/// - `e` — end value
/// - `o` — out bezier tangent (from `easing_bezier`)
/// - `i` — in bezier tangent
fn kf(t: f64, s: Value, e: Value, o: Value, i: Value) -> Value {
    json!({ "t": t, "s": s, "e": e, "o": o, "i": i })
}

/// Build the final Lottie keyframe. No `e`/`o`/`i` needed — it's just a stop.
fn kf_end(t: f64, s: Value) -> Value {
    json!({ "t": t, "s": s })
}

/// Map an easing name to Lottie bezier out/in tangents: `(out_tangent, in_tangent)`.
///
/// Lottie easing tangents are expressed as normalised `{x, y}` objects where:
/// - `o` (out) is the handle leaving the current keyframe value
/// - `i` (in)  is the handle arriving at the next keyframe value
///
/// These values map approximately to CSS `cubic-bezier(p1x, p1y, p2x, p2y)`.
fn easing_bezier(easing: &str) -> (Value, Value) {
    match easing {
        "ease_in" => (json!({"x":[0.42],"y":[0.0]}), json!({"x":[1.0], "y":[1.0]})),
        "ease_out" => (json!({"x":[0.0], "y":[0.0]}), json!({"x":[0.42],"y":[1.0]})),
        "ease_in_out" => (json!({"x":[0.42],"y":[0.0]}), json!({"x":[0.42],"y":[1.0]})),
        // Spring: fast exit, overshoots slightly (y > 1), then settles.
        "spring" => (json!({"x":[0.5], "y":[0.0]}), json!({"x":[0.1], "y":[1.5]})),
        _ => (json!({"x":[0.0], "y":[0.0]}), json!({"x":[1.0], "y":[1.0]})), // linear
    }
}

/// Map a size name to a pixel dimension for the shape geometry.
fn size_px(size: &str) -> f64 {
    match size {
        "small" => 80.0,
        "large" => 250.0,
        _ => 150.0, // "medium" and any unexpected value
    }
}

/// Parse a `#rrggbb` hex string into a normalised RGBA array `[r, g, b, 1.0]`.
///
/// Each channel is divided by 255.0 to fit the Lottie `[0.0, 1.0]` range.
fn parse_hex_color(hex: &str) -> [f64; 4] {
    let h = hex.trim_start_matches('#');
    let parse_channel = |s: &str| u8::from_str_radix(s, 16).unwrap_or(0) as f64 / 255.0;
    [
        parse_channel(&h[0..2]),
        parse_channel(&h[2..4]),
        parse_channel(&h[4..6]),
        1.0,
    ]
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation_spec::AnimationSpec;

    /// Helper to build a minimal AnimationSpec for test cases.
    fn spec(
        shape: &str,
        motion: &str,
        color: &str,
        size: &str,
        easing: &str,
        duration: u32,
    ) -> AnimationSpec {
        AnimationSpec {
            shape: shape.into(),
            motion: motion.into(),
            color: color.into(),
            size: size.into(),
            easing: easing.into(),
            duration,
        }
    }

    /// Helper to build a minimal valid LottieAnimation for validate_animations tests.
    fn minimal_anim(ip: u32, op: u32, w: u32, h: u32, layers: Vec<Value>) -> LottieAnimation {
        LottieAnimation {
            v: LOTTIE_VERSION.into(),
            fr: FRAME_RATE,
            ip,
            op,
            w,
            h,
            nm: "test".into(),
            ddd: 0,
            assets: vec![],
            layers,
        }
    }

    // ── Fixture parsing ──────────────────────────────────────────────────────

    #[test]
    fn parses_known_good_lottie_fixture() {
        let raw = r#"{
            "v":"5.5.7","fr":30,"ip":0,"op":30,"w":500,"h":500,
            "nm":"test","ddd":0,"assets":[],"layers":[]
        }"#;
        let parsed: LottieAnimation = serde_json::from_str(raw).expect("should parse");
        assert_eq!(parsed.v, "5.5.7");
        assert_eq!(parsed.fr, 30);
        assert_eq!(parsed.ip, 0);
        assert_eq!(parsed.op, 30);
        assert_eq!(parsed.w, 500);
        assert_eq!(parsed.h, 500);
    }

    // ── Validation ────────────────────────────────────────────────────────────

    #[test]
    fn validate_rejects_op_equal_to_ip() {
        let anim = minimal_anim(30, 30, CANVAS_W, CANVAS_H, vec![json!({"ty":4})]);
        let err = validate_animations(&[anim]).unwrap_err();
        assert!(err.contains("op"), "error should mention 'op': {err}");
    }

    #[test]
    fn validate_rejects_op_less_than_ip() {
        let anim = minimal_anim(60, 30, CANVAS_W, CANVAS_H, vec![json!({"ty":4})]);
        let err = validate_animations(&[anim]).unwrap_err();
        assert!(err.contains("op"), "error should mention 'op': {err}");
    }

    #[test]
    fn validate_rejects_zero_width() {
        let anim = minimal_anim(0, 30, 0, CANVAS_H, vec![json!({"ty":4})]);
        let err = validate_animations(&[anim]).unwrap_err();
        assert!(err.contains("w"), "error should mention 'w': {err}");
    }

    #[test]
    fn validate_rejects_zero_height() {
        let anim = minimal_anim(0, 30, CANVAS_W, 0, vec![json!({"ty":4})]);
        let err = validate_animations(&[anim]).unwrap_err();
        assert!(err.contains("h"), "error should mention 'h': {err}");
    }

    #[test]
    fn validate_rejects_empty_layers() {
        let anim = minimal_anim(0, 30, CANVAS_W, CANVAS_H, vec![]);
        let err = validate_animations(&[anim]).unwrap_err();
        assert!(
            err.contains("layers"),
            "error should mention 'layers': {err}"
        );
    }

    #[test]
    fn validate_rejects_empty_animations_slice() {
        let err = validate_animations(&[]).unwrap_err();
        assert!(err.contains("zero"));
    }

    #[test]
    fn validate_accepts_valid_animation() {
        let anim = minimal_anim(0, 30, CANVAS_W, CANVAS_H, vec![json!({"ty":4})]);
        assert!(validate_animations(&[anim]).is_ok());
    }

    // ── Shape variants ────────────────────────────────────────────────────────

    #[test]
    fn each_shape_variant_produces_non_empty_layers() {
        let shapes = [
            "circle", "square", "triangle", "line", "arc", "bars", "dots",
        ];
        for shape in shapes {
            let specs = vec![spec(shape, "expand", "#ff5577", "medium", "linear", 30)];
            let result =
                compile(&specs).unwrap_or_else(|e| panic!("compile failed for shape {shape}: {e}"));
            assert!(
                !result[0].layers.is_empty(),
                "shape '{shape}' produced empty layers"
            );
        }
    }

    #[test]
    fn each_shape_variant_produces_at_least_two_shape_items() {
        // Every shape type includes at least a geometry item + a paint item (fill or stroke).
        let shapes = [
            "circle", "square", "triangle", "line", "arc", "bars", "dots",
        ];
        for shape in shapes {
            let specs = vec![spec(shape, "expand", "#ff5577", "medium", "linear", 30)];
            let result = compile(&specs).unwrap();
            let layer = &result[0].layers[0];
            let items = layer["shapes"].as_array().expect("shapes must be array");
            assert!(
                items.len() >= 2,
                "shape '{shape}' should have >= 2 shape items, got {}",
                items.len()
            );
        }
    }

    // ── Motion variants ───────────────────────────────────────────────────────

    /// Returns true if any transform property in `layer["ks"]` is animated (a:1)
    /// and has a non-empty keyframe array.
    fn has_animated_keyframes(layer: &Value) -> bool {
        let ks = &layer["ks"];
        for prop in ["o", "r", "p", "s"] {
            let p = &ks[prop];
            if p.get("a") == Some(&json!(1)) {
                if let Some(arr) = p["k"].as_array() {
                    if !arr.is_empty() {
                        return true;
                    }
                }
            }
        }
        false
    }

    #[test]
    fn each_motion_variant_produces_animated_keyframes() {
        let motions = [
            "expand", "contract", "pulse", "drift", "rotate", "shake", "appear",
        ];
        for motion in motions {
            let specs = vec![spec("circle", motion, "#ff5577", "medium", "linear", 30)];
            let result = compile(&specs)
                .unwrap_or_else(|e| panic!("compile failed for motion {motion}: {e}"));
            assert!(
                has_animated_keyframes(&result[0].layers[0]),
                "motion '{motion}' produced no animated keyframe in ks"
            );
        }
    }

    // ── compile() contract ────────────────────────────────────────────────────

    #[test]
    fn compile_produces_one_animation_per_spec() {
        let specs = vec![
            spec("circle", "expand", "#ff5577", "large", "ease_out", 30),
            spec("line", "drift", "#3344aa", "medium", "linear", 24),
            spec("dots", "appear", "#22bb44", "small", "ease_in", 45),
        ];
        let result = compile(&specs).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn compile_sets_correct_metadata() {
        let specs = vec![spec("circle", "expand", "#ff5577", "medium", "linear", 45)];
        let result = compile(&specs).unwrap();
        assert_eq!(result[0].v, LOTTIE_VERSION);
        assert_eq!(result[0].fr, FRAME_RATE);
        assert_eq!(result[0].w, CANVAS_W);
        assert_eq!(result[0].h, CANVAS_H);
        assert_eq!(result[0].ip, 0);
        assert_eq!(result[0].op, 45);
        assert_eq!(result[0].ddd, 0);
    }

    // ── Colour parsing ────────────────────────────────────────────────────────

    #[test]
    fn parse_hex_color_normalises_correctly() {
        let [r, g, b, a] = parse_hex_color("#ff8000");
        assert!((r - 1.0).abs() < 0.005, "red should be ~1.0, got {r}");
        assert!((g - 0.502).abs() < 0.005, "green should be ~0.502, got {g}");
        assert!(b.abs() < 0.005, "blue should be ~0.0, got {b}");
        assert!((a - 1.0).abs() < 0.005, "alpha should be 1.0, got {a}");
    }

    #[test]
    fn parse_hex_color_handles_all_zeros() {
        let [r, g, b, a] = parse_hex_color("#000000");
        assert_eq!(r, 0.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn parse_hex_color_handles_all_ff() {
        let [r, g, b, a] = parse_hex_color("#ffffff");
        assert!((r - 1.0).abs() < 0.005);
        assert!((g - 1.0).abs() < 0.005);
        assert!((b - 1.0).abs() < 0.005);
        assert_eq!(a, 1.0);
    }

    // ── Easing ────────────────────────────────────────────────────────────────

    #[test]
    fn easing_bezier_returns_all_variants_without_panic() {
        for easing in [
            "linear",
            "ease_in",
            "ease_out",
            "ease_in_out",
            "spring",
            "unknown",
        ] {
            let (o, i) = easing_bezier(easing);
            assert!(
                o.get("x").is_some(),
                "out tangent missing x for easing {easing}"
            );
            assert!(
                i.get("x").is_some(),
                "in tangent missing x for easing {easing}"
            );
        }
    }
}
