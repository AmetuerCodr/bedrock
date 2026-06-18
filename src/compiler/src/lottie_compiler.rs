//! Lottie compiler — stage 3 of the bedrock-video pipeline.
//!
//! Converts a [`Vec<AnimationSpec>`] into a [`Vec<LottieAnimation>`] (Lottie spec 5.5.7).

use crate::animation_spec::{AnimationSpec, LayerSpec, PositionKf};
use crate::script_analyzer::ScriptInput;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{self, Read};

const CANVAS_W: u32 = 500;
const CANVAS_H: u32 = 500;
const FRAME_RATE: u32 = 30;
const LOTTIE_VERSION: &str = "5.5.7";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LottieAnimation {
    pub v: String,
    pub fr: u32,
    pub ip: u32,
    pub op: u32,
    pub w: u32,
    pub h: u32,
    pub nm: String,
    pub ddd: u32,
    pub assets: Vec<Value>,
    pub layers: Vec<Value>,
}

// ─── Public entry points ──────────────────────────────────────────────────────

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

pub async fn run_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw)?;
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("empty input on stdin; expected data.json contents".into());
    }
    let input: ScriptInput =
        serde_json::from_str(raw).map_err(|e| format!("stdin is not valid data.json: {e}"))?;
    eprintln!(
        "stage 1/3: analyzing script ({} word groups)…",
        input.word_groups.len()
    );
    let moments = crate::script_analyzer::analyze(input).await?;
    eprintln!(
        "stage 2/3: designing animation specs for {} moments…",
        moments.len()
    );
    let specs = crate::animation_spec::design(&moments).await?;
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

fn validate_animations(anims: &[LottieAnimation]) -> Result<(), String> {
    if anims.is_empty() {
        return Err("compile produced zero animations".to_string());
    }
    for (i, a) in anims.iter().enumerate() {
        if a.op <= a.ip {
            return Err(format!("animation {i}: op ({}) must be > ip ({})", a.op, a.ip));
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

fn build_animation(index: usize, spec: &AnimationSpec) -> LottieAnimation {
    let layers: Vec<Value> = spec
        .layers
        .iter()
        .enumerate()
        .map(|(i, layer)| build_layer(i + 1, layer, spec.duration))
        .collect();

    LottieAnimation {
        v: LOTTIE_VERSION.to_string(),
        fr: FRAME_RATE,
        ip: 0,
        op: spec.duration,
        w: CANVAS_W,
        h: CANVAS_H,
        nm: format!("anim-{index}"),
        ddd: 0,
        assets: vec![],
        layers,
    }
}

fn build_layer(ind: usize, layer: &LayerSpec, duration: u32) -> Value {
    let color = parse_hex_color(&layer.color);
    let px = size_px(&layer.size);
    let shapes = build_shapes(&layer.shape, color, px);
    let pos = build_position(&layer.position, &layer.easing);
    let scale = build_scale(layer.scale_start, layer.scale_end, duration, &layer.easing);
    let opacity = build_opacity(layer.opacity_in, layer.opacity_out, duration);

    json!({
        "ddd": 0,
        "ind": ind,
        "ty": 4,
        "nm": format!("{} {}", layer.shape, ind),
        "ip": layer.delay,
        "op": duration,
        "st": 0,
        "bm": 0,
        "ks": {
            "a": { "a": 0, "k": [0.0, 0.0, 0.0] },
            "p": pos,
            "s": scale,
            "r": { "a": 0, "k": 0 },
            "o": opacity,
        },
        "shapes": shapes,
    })
}

// ─── Property builders ────────────────────────────────────────────────────────

fn build_position(position: &[PositionKf], easing: &str) -> Value {
    if position.len() < 2 {
        let p = position.first().cloned().unwrap_or(PositionKf { frame: 0, x: 250.0, y: 250.0 });
        return json!({ "a": 0, "k": [p.x, p.y, 0.0] });
    }
    let (o_bez, i_bez) = easing_bezier(easing);
    let mut keyframes: Vec<Value> = position
        .windows(2)
        .map(|w| {
            let a = &w[0];
            let b = &w[1];
            kf(
                a.frame as f64,
                json!([a.x, a.y, 0.0]),
                json!([b.x, b.y, 0.0]),
                o_bez.clone(),
                i_bez.clone(),
            )
        })
        .collect();
    let last = position.last().unwrap();
    keyframes.push(kf_end(last.frame as f64, json!([last.x, last.y, 0.0])));
    json!({ "a": 1, "k": keyframes })
}

fn build_scale(start: f64, end: f64, duration: u32, easing: &str) -> Value {
    if (start - end).abs() < 0.1 {
        return json!({ "a": 0, "k": [start, start, 100.0] });
    }
    let (o_bez, i_bez) = easing_bezier(easing);
    let dur = duration as f64;
    json!({ "a": 1, "k": [
        kf(0.0, json!([start, start, 100.0]), json!([end, end, 100.0]), o_bez, i_bez),
        kf_end(dur, json!([end, end, 100.0]))
    ]})
}

fn build_opacity(opacity_in: u32, opacity_out: u32, duration: u32) -> Value {
    let dur = duration as f64;
    let lin_o = json!({"x":[0.0],"y":[0.0]});
    let lin_i = json!({"x":[1.0],"y":[1.0]});

    match (opacity_in, opacity_out) {
        (0, 0) => json!({ "a": 0, "k": 100 }),
        (fi, 0) => json!({ "a": 1, "k": [
            kf(0.0, json!([0.0]), json!([100.0]), lin_o, lin_i),
            kf_end(fi as f64, json!([100.0]))
        ]}),
        (0, fo) => {
            let fade_start = dur - fo as f64;
            json!({ "a": 1, "k": [
                kf(fade_start, json!([100.0]), json!([0.0]), lin_o, lin_i),
                kf_end(dur, json!([0.0]))
            ]})
        }
        (fi, fo) => {
            let fade_start = dur - fo as f64;
            json!({ "a": 1, "k": [
                kf(0.0, json!([0.0]), json!([100.0]), lin_o.clone(), lin_i.clone()),
                kf(fi as f64, json!([100.0]), json!([100.0]), lin_o.clone(), lin_i.clone()),
                kf(fade_start, json!([100.0]), json!([0.0]), lin_o, lin_i),
                kf_end(dur, json!([0.0]))
            ]})
        }
    }
}

// ─── Shape builders ───────────────────────────────────────────────────────────

fn build_shapes(shape: &str, color: [f64; 4], px: f64) -> Vec<Value> {
    match shape {
        "circle" => vec![build_ellipse(px), build_fill(color)],
        "square" => vec![build_rect(px, px, 0.0), build_fill(color)],
        "triangle" => vec![build_triangle_path(px), build_fill(color)],
        "line" => vec![build_line_path(px), build_stroke(color, 4.0)],
        "arc" => vec![build_arc_path(px), build_stroke(color, 4.0)],
        "bars" => build_bars(color, px),
        "dots" => build_dots(color, px),
        _ => vec![build_ellipse(px), build_fill(color)],
    }
}

fn build_ellipse(diameter: f64) -> Value {
    json!({
        "ty": "el", "nm": "Ellipse",
        "s": { "a": 0, "k": [diameter, diameter] },
        "p": { "a": 0, "k": [0.0, 0.0] }
    })
}

fn build_rect(w: f64, h: f64, corner_radius: f64) -> Value {
    json!({
        "ty": "rc", "nm": "Rectangle",
        "s": { "a": 0, "k": [w, h] },
        "r": { "a": 0, "k": corner_radius },
        "p": { "a": 0, "k": [0.0, 0.0] }
    })
}

fn build_triangle_path(size: f64) -> Value {
    let half_h = size / 2.0;
    let half_w = size / 2.0;
    json!({
        "ty": "sh", "nm": "Triangle",
        "ks": { "a": 0, "k": {
            "c": true,
            "v": [[0.0, -half_h], [half_w, half_h], [-half_w, half_h]],
            "i": [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
            "o": [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]
        }}
    })
}

fn build_line_path(length: f64) -> Value {
    let half = length / 2.0;
    json!({
        "ty": "sh", "nm": "Line",
        "ks": { "a": 0, "k": {
            "c": false,
            "v": [[-half, 0.0], [half, 0.0]],
            "i": [[0.0, 0.0], [0.0, 0.0]],
            "o": [[0.0, 0.0], [0.0, 0.0]]
        }}
    })
}

fn build_arc_path(size: f64) -> Value {
    let r = size / 2.0;
    let k = r * 0.5523;
    json!({
        "ty": "sh", "nm": "Arc",
        "ks": { "a": 0, "k": {
            "c": false,
            "v": [[-r, 0.0], [0.0, -r], [r, 0.0]],
            "i": [[0.0, 0.0], [-k, 0.0], [0.0, 0.0]],
            "o": [[-k, 0.0], [k, 0.0], [0.0, 0.0]]
        }}
    })
}

fn build_bars(color: [f64; 4], size: f64) -> Vec<Value> {
    let bar_w = (size / 5.0).max(4.0);
    let bar_h = size;
    let gap = size / 3.0;
    let mut items: Vec<Value> = (0..3_i32)
        .map(|i| {
            let x = (i as f64 - 1.0) * gap;
            json!({
                "ty": "rc", "nm": format!("Bar {}", i + 1),
                "s": { "a": 0, "k": [bar_w, bar_h] },
                "r": { "a": 0, "k": 0 },
                "p": { "a": 0, "k": [x, 0.0] }
            })
        })
        .collect();
    items.push(build_fill(color));
    items
}

fn build_dots(color: [f64; 4], size: f64) -> Vec<Value> {
    let dot_d = (size / 4.0).max(8.0);
    let gap = size / 3.0;
    let mut items: Vec<Value> = (0..3_i32)
        .map(|i| {
            let x = (i as f64 - 1.0) * gap;
            json!({
                "ty": "el", "nm": format!("Dot {}", i + 1),
                "s": { "a": 0, "k": [dot_d, dot_d] },
                "p": { "a": 0, "k": [x, 0.0] }
            })
        })
        .collect();
    items.push(build_fill(color));
    items
}

fn build_fill(color: [f64; 4]) -> Value {
    json!({
        "ty": "fl", "nm": "Fill",
        "c": { "a": 0, "k": [color[0], color[1], color[2], color[3]] },
        "o": { "a": 0, "k": 100 }
    })
}

fn build_stroke(color: [f64; 4], width: f64) -> Value {
    json!({
        "ty": "st", "nm": "Stroke",
        "c": { "a": 0, "k": [color[0], color[1], color[2], color[3]] },
        "o": { "a": 0, "k": 100 },
        "w": { "a": 0, "k": width }
    })
}

// ─── Keyframe & easing helpers ────────────────────────────────────────────────

fn kf(t: f64, s: Value, e: Value, o: Value, i: Value) -> Value {
    json!({ "t": t, "s": s, "e": e, "o": o, "i": i })
}

fn kf_end(t: f64, s: Value) -> Value {
    json!({ "t": t, "s": s })
}

fn easing_bezier(easing: &str) -> (Value, Value) {
    match easing {
        "ease_in" => (json!({"x":[0.42],"y":[0.0]}), json!({"x":[1.0],"y":[1.0]})),
        "ease_out" => (json!({"x":[0.0],"y":[0.0]}), json!({"x":[0.42],"y":[1.0]})),
        "ease_in_out" => (json!({"x":[0.42],"y":[0.0]}), json!({"x":[0.42],"y":[1.0]})),
        "spring" => (json!({"x":[0.5],"y":[0.0]}), json!({"x":[0.1],"y":[1.5]})),
        _ => (json!({"x":[0.0],"y":[0.0]}), json!({"x":[1.0],"y":[1.0]})),
    }
}

fn size_px(size: &str) -> f64 {
    match size {
        "small" => 80.0,
        "large" => 250.0,
        _ => 150.0,
    }
}

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
    use crate::animation_spec::{AnimationSpec, LayerSpec, PositionKf};

    fn base_layer(shape: &str) -> LayerSpec {
        LayerSpec {
            shape: shape.into(),
            color: "#ff5577".into(),
            size: "medium".into(),
            position: vec![PositionKf { frame: 0, x: 250.0, y: 250.0 }],
            scale_start: 0.0,
            scale_end: 100.0,
            opacity_in: 0,
            opacity_out: 0,
            delay: 0,
            easing: "linear".into(),
        }
    }

    fn spec_with(duration: u32, layers: Vec<LayerSpec>) -> AnimationSpec {
        AnimationSpec { duration, layers }
    }

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
        let raw = r#"{"v":"5.5.7","fr":30,"ip":0,"op":30,"w":500,"h":500,"nm":"test","ddd":0,"assets":[],"layers":[]}"#;
        let parsed: LottieAnimation = serde_json::from_str(raw).expect("should parse");
        assert_eq!(parsed.v, "5.5.7");
        assert_eq!(parsed.fr, 30);
        assert_eq!(parsed.op, 30);
        assert_eq!(parsed.w, 500);
    }

    // ── Validation ────────────────────────────────────────────────────────────

    #[test]
    fn validate_rejects_op_equal_to_ip() {
        let anim = minimal_anim(30, 30, CANVAS_W, CANVAS_H, vec![json!({"ty":4})]);
        assert!(validate_animations(&[anim]).unwrap_err().contains("op"));
    }

    #[test]
    fn validate_rejects_zero_width() {
        let anim = minimal_anim(0, 30, 0, CANVAS_H, vec![json!({"ty":4})]);
        assert!(validate_animations(&[anim]).unwrap_err().contains("w"));
    }

    #[test]
    fn validate_rejects_zero_height() {
        let anim = minimal_anim(0, 30, CANVAS_W, 0, vec![json!({"ty":4})]);
        assert!(validate_animations(&[anim]).unwrap_err().contains("h"));
    }

    #[test]
    fn validate_rejects_empty_layers() {
        let anim = minimal_anim(0, 30, CANVAS_W, CANVAS_H, vec![]);
        assert!(validate_animations(&[anim]).unwrap_err().contains("layers"));
    }

    #[test]
    fn validate_rejects_empty_animations_slice() {
        assert!(validate_animations(&[]).unwrap_err().contains("zero"));
    }

    // ── Shape variants ────────────────────────────────────────────────────────

    #[test]
    fn each_shape_produces_non_empty_layers() {
        let shapes = ["circle", "square", "triangle", "line", "arc", "bars", "dots"];
        for shape in shapes {
            let specs = vec![spec_with(30, vec![base_layer(shape)])];
            let result = compile(&specs).unwrap();
            assert!(!result[0].layers.is_empty(), "shape '{shape}' gave empty layers");
        }
    }

    #[test]
    fn each_shape_produces_at_least_two_shape_items() {
        let shapes = ["circle", "square", "triangle", "line", "arc", "bars", "dots"];
        for shape in shapes {
            let specs = vec![spec_with(30, vec![base_layer(shape)])];
            let result = compile(&specs).unwrap();
            let items = result[0].layers[0]["shapes"].as_array().unwrap();
            assert!(items.len() >= 2, "shape '{shape}' has < 2 shape items");
        }
    }

    // ── Position builder ──────────────────────────────────────────────────────

    #[test]
    fn static_position_produces_a0() {
        let pos = vec![PositionKf { frame: 0, x: 250.0, y: 250.0 }];
        let result = build_position(&pos, "linear");
        assert_eq!(result["a"], json!(0));
    }

    #[test]
    fn animated_position_produces_a1() {
        let pos = vec![
            PositionKf { frame: 0, x: 250.0, y: 300.0 },
            PositionKf { frame: 30, x: 250.0, y: 150.0 },
        ];
        let result = build_position(&pos, "ease_out");
        assert_eq!(result["a"], json!(1));
        assert!(result["k"].as_array().unwrap().len() >= 2);
    }

    // ── Scale builder ─────────────────────────────────────────────────────────

    #[test]
    fn equal_scale_produces_static() {
        let result = build_scale(100.0, 100.0, 30, "linear");
        assert_eq!(result["a"], json!(0));
    }

    #[test]
    fn different_scale_produces_animated() {
        let result = build_scale(0.0, 100.0, 30, "ease_out");
        assert_eq!(result["a"], json!(1));
    }

    // ── Opacity builder ───────────────────────────────────────────────────────

    #[test]
    fn no_opacity_envelope_is_static() {
        let result = build_opacity(0, 0, 30);
        assert_eq!(result["a"], json!(0));
        assert_eq!(result["k"], json!(100));
    }

    #[test]
    fn fade_in_only_is_animated() {
        let result = build_opacity(10, 0, 30);
        assert_eq!(result["a"], json!(1));
        assert_eq!(result["k"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn both_envelopes_produce_four_keyframes() {
        let result = build_opacity(8, 8, 60);
        assert_eq!(result["a"], json!(1));
        assert_eq!(result["k"].as_array().unwrap().len(), 4);
    }

    // ── Multi-layer ───────────────────────────────────────────────────────────

    #[test]
    fn multi_layer_spec_produces_multiple_lottie_layers() {
        let specs = vec![spec_with(
            60,
            vec![base_layer("circle"), base_layer("dots"), base_layer("arc")],
        )];
        let result = compile(&specs).unwrap();
        assert_eq!(result[0].layers.len(), 3);
    }

    #[test]
    fn layer_delay_sets_ip_field() {
        let mut l = base_layer("circle");
        l.delay = 15;
        let specs = vec![spec_with(60, vec![l])];
        let result = compile(&specs).unwrap();
        assert_eq!(result[0].layers[0]["ip"], json!(15));
    }

    // ── compile() contract ────────────────────────────────────────────────────

    #[test]
    fn compile_produces_one_animation_per_spec() {
        let specs = vec![
            spec_with(30, vec![base_layer("circle"), base_layer("dots")]),
            spec_with(45, vec![base_layer("arc"), base_layer("triangle")]),
            spec_with(60, vec![base_layer("bars")]),
        ];
        assert_eq!(compile(&specs).unwrap().len(), 3);
    }

    #[test]
    fn compile_sets_correct_metadata() {
        let specs = vec![spec_with(45, vec![base_layer("circle")])];
        let result = compile(&specs).unwrap();
        assert_eq!(result[0].v, LOTTIE_VERSION);
        assert_eq!(result[0].fr, FRAME_RATE);
        assert_eq!(result[0].w, CANVAS_W);
        assert_eq!(result[0].h, CANVAS_H);
        assert_eq!(result[0].ip, 0);
        assert_eq!(result[0].op, 45);
        assert_eq!(result[0].ddd, 0);
    }

    // ── Color parsing ─────────────────────────────────────────────────────────

    #[test]
    fn parse_hex_color_normalises_correctly() {
        let [r, g, b, a] = parse_hex_color("#ff8000");
        assert!((r - 1.0).abs() < 0.005);
        assert!((g - 0.502).abs() < 0.005);
        assert!(b.abs() < 0.005);
        assert!((a - 1.0).abs() < 0.005);
    }

    #[test]
    fn parse_hex_color_handles_extremes() {
        let [r, g, b, _] = parse_hex_color("#000000");
        assert_eq!(r, 0.0); assert_eq!(g, 0.0); assert_eq!(b, 0.0);
        let [r, g, b, _] = parse_hex_color("#ffffff");
        assert!((r - 1.0).abs() < 0.005);
        assert!((g - 1.0).abs() < 0.005);
        assert!((b - 1.0).abs() < 0.005);
    }

    // ── Easing ────────────────────────────────────────────────────────────────

    #[test]
    fn easing_bezier_returns_all_variants_without_panic() {
        for easing in ["linear", "ease_in", "ease_out", "ease_in_out", "spring", "unknown"] {
            let (o, i) = easing_bezier(easing);
            assert!(o.get("x").is_some(), "out tangent missing x for {easing}");
            assert!(i.get("x").is_some(), "in tangent missing x for {easing}");
        }
    }
}
