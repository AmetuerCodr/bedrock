use crate::gemini;
use crate::script_analyzer::VisualMoment;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

const SYSTEM_PROMPT: &str = r##"You are a motion-graphics designer for a Lottie animation compiler.

INPUT: a JSON object with one field "moments" — an ordered array of visual moment descriptors.
Each moment has: "concept" (string), "mood" (string), "metaphor" (string), "duration" (integer, frames at 30fps).

TASK: for each moment IN ORDER, design a multi-layer animation. Stack 2-5 simple shapes to create visual complexity.

OUTPUT: a JSON array of length EXACTLY equal to the input moments array. Output ONLY the array. No markdown fences. No prose.

Canvas: 500x500 pixels. (0,0) = top-left corner. Y increases downward. Center = (250, 250).

Each element MUST be an object with EXACTLY these fields:
  "duration" (integer): frames at 30fps, in [15, 90]. Copy from the input moment.
  "layers"   (array):   2-5 objects, each with ALL of these fields:
    "shape"       (string):  one of "circle","square","triangle","line","arc","bars","dots"
    "color"       (string):  hex color "#rrggbb" lowercase, exactly 7 chars
    "size"        (string):  one of "small","medium","large"
    "position"    (array):   1-3 objects each: {"frame":<int>,"x":<float 0-500>,"y":<float 0-500>}
                             1 object = static. 2-3 objects = animated movement between waypoints.
    "scale_start" (number):  scale at frame 0, as percentage 0-300. Use 0 to expand from nothing.
    "scale_end"   (number):  scale at last frame, as percentage 0-300. Use 0 to shrink away.
    "opacity_in"  (integer): frames to fade in at start (0=instant). Max: duration/3.
    "opacity_out" (integer): frames to fade out at end (0=stays visible). Max: duration/3.
    "delay"       (integer): frame when this layer first appears. Range: 0 to duration-8.
    "easing"      (string):  one of "linear","ease_in","ease_out","ease_in_out","spring"

DESIGN RULES:
- Use 3-5 layers for richness. Stagger "delay" (e.g. 0,8,16,24) so elements appear in sequence.
- scale_start=0 expands from nothing. scale_start=100,scale_end=0 shrinks away.
- Give layers 2-point position arrays to animate movement (rising = y decreases, falling = y increases).
- opacity_in/out create smooth entry and exit per layer.
- Colors match mood: warm (#ff6600,#ffdd00,#ff4400) for energetic/triumphant, cool (#4488ff,#44ccff) for hopeful/calm, dark (#330011,#220033) for ominous/tense, muted (#888888,#aabbcc) for somber/playful.
- Vary shapes within one animation. Think in metaphors — "rocket launch" = smoke+flame+body+sparks.

EXAMPLE — concept "growth", mood "hopeful", duration 60:
[{"duration":60,"layers":[
  {"shape":"circle","color":"#224488","size":"large","position":[{"frame":0,"x":250,"y":340},{"frame":60,"x":250,"y":200}],"scale_start":0,"scale_end":180,"opacity_in":5,"opacity_out":15,"delay":0,"easing":"ease_out"},
  {"shape":"arc","color":"#44aaff","size":"medium","position":[{"frame":0,"x":250,"y":300},{"frame":60,"x":250,"y":150}],"scale_start":60,"scale_end":110,"opacity_in":8,"opacity_out":12,"delay":10,"easing":"ease_in_out"},
  {"shape":"dots","color":"#88ddff","size":"small","position":[{"frame":0,"x":250,"y":260}],"scale_start":0,"scale_end":140,"opacity_in":0,"opacity_out":18,"delay":20,"easing":"spring"},
  {"shape":"triangle","color":"#aaddff","size":"small","position":[{"frame":0,"x":220,"y":310},{"frame":60,"x":190,"y":170}],"scale_start":0,"scale_end":80,"opacity_in":0,"opacity_out":10,"delay":15,"easing":"ease_out"}
]}]"##;

const VALID_SHAPES: &[&str] = &[
    "circle", "square", "triangle", "line", "arc", "bars", "dots",
];
const VALID_EASINGS: &[&str] = &["linear", "ease_in", "ease_out", "ease_in_out", "spring"];
const VALID_SIZES: &[&str] = &["small", "medium", "large"];
const DURATION_MIN: u32 = 15;
const DURATION_MAX: u32 = 90;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PositionKf {
    pub frame: u32,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct LayerSpec {
    pub shape: String,
    pub color: String,
    pub size: String,
    pub position: Vec<PositionKf>,
    pub scale_start: f64,
    pub scale_end: f64,
    pub opacity_in: u32,
    pub opacity_out: u32,
    pub delay: u32,
    pub easing: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AnimationSpec {
    pub duration: u32,
    pub layers: Vec<LayerSpec>,
}

fn is_hex_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

fn normalize_shape(s: &str) -> String {
    match s {
        "lines" | "line_segment" => "line".into(),
        "ellipse" | "oval" => "circle".into(),
        "rectangle" | "rect" => "square".into(),
        "bar" => "bars".into(),
        "dot" | "circle_dots" => "dots".into(),
        "arcs" | "arc_shape" => "arc".into(),
        other => other.into(),
    }
}

fn validate_specs(specs: &mut Vec<AnimationSpec>, expected_len: usize) -> Result<(), String> {
    if specs.len() != expected_len {
        return Err(format!(
            "spec count {} does not match moment count {}",
            specs.len(),
            expected_len
        ));
    }
    for (si, spec) in specs.iter_mut().enumerate() {
        if spec.duration < DURATION_MIN {
            spec.duration = DURATION_MIN;
        } else if spec.duration > DURATION_MAX {
            spec.duration = DURATION_MAX;
        }
        let dur = spec.duration;

        if spec.layers.is_empty() {
            return Err(format!("spec {si}: layers array is empty"));
        }
        if spec.layers.len() > 6 {
            spec.layers.truncate(6);
        }

        for (li, layer) in spec.layers.iter_mut().enumerate() {
            let ctx = format!("spec {si} layer {li}");

            layer.shape = normalize_shape(&layer.shape);
            if !VALID_SHAPES.contains(&layer.shape.as_str()) {
                return Err(format!("{ctx}: unknown shape {:?}", layer.shape));
            }

            layer.size = match layer.size.as_str() {
                "huge" | "xlarge" => "large".into(),
                "tiny" | "mini" => "small".into(),
                s if VALID_SIZES.contains(&s) => s.into(),
                _ => "medium".into(),
            };

            layer.easing = match layer.easing.as_str() {
                "bounce" | "elastic" => "spring".into(),
                "cubic" => "ease_in_out".into(),
                s if VALID_EASINGS.contains(&s) => s.into(),
                _ => "ease_out".into(),
            };

            layer.color = layer.color.to_ascii_lowercase();
            if !is_hex_color(&layer.color) {
                return Err(format!("{ctx}: bad color {:?}", layer.color));
            }

            layer.scale_start = layer.scale_start.clamp(0.0, 300.0);
            layer.scale_end = layer.scale_end.clamp(0.0, 300.0);

            if layer.position.is_empty() {
                layer.position = vec![PositionKf { frame: 0, x: 250.0, y: 250.0 }];
            }
            for kf in layer.position.iter_mut() {
                kf.x = kf.x.clamp(-50.0, 550.0);
                kf.y = kf.y.clamp(-50.0, 550.0);
            }

            if layer.delay >= dur.saturating_sub(4) {
                layer.delay = 0;
            }

            if layer.opacity_in + layer.opacity_out >= dur {
                layer.opacity_in = dur / 4;
                layer.opacity_out = dur / 4;
            }
        }
    }
    Ok(())
}

pub async fn design(
    moments: &[VisualMoment],
) -> Result<Vec<AnimationSpec>, Box<dyn std::error::Error>> {
    if moments.is_empty() {
        return Err("input moments array is empty".into());
    }
    let user_msg = serde_json::to_string(&serde_json::json!({ "moments": moments }))?;
    eprintln!("calling gemini: animation-spec ({} moments)…", moments.len());
    let raw = gemini::prompt(&user_msg, Some(SYSTEM_PROMPT)).await?;
    let cleaned = gemini::strip_fences(&raw);
    let mut specs: Vec<AnimationSpec> = serde_json::from_str(cleaned)
        .map_err(|e| format!("gemini returned invalid JSON: {e}\n---\n{cleaned}"))?;
    validate_specs(&mut specs, moments.len())?;
    eprintln!("designed {} animation specs", specs.len());
    Ok(specs)
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut raw_input = String::new();
    io::stdin().read_to_string(&mut raw_input)?;
    let raw_input = raw_input.trim();
    if raw_input.is_empty() {
        return Err("empty input on stdin; expected JSON array of visual moments".into());
    }
    let moments: Vec<VisualMoment> = serde_json::from_str(raw_input)
        .map_err(|e| format!("stdin is not a valid moments JSON array: {e}"))?;
    let specs = design(&moments).await?;
    println!("{}", serde_json::to_string(&specs)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(shape: &str, color: &str) -> LayerSpec {
        LayerSpec {
            shape: shape.into(),
            color: color.into(),
            size: "medium".into(),
            position: vec![PositionKf { frame: 0, x: 250.0, y: 250.0 }],
            scale_start: 0.0,
            scale_end: 100.0,
            opacity_in: 0,
            opacity_out: 0,
            delay: 0,
            easing: "ease_out".into(),
        }
    }

    fn spec_with_layers(duration: u32, layers: Vec<LayerSpec>) -> AnimationSpec {
        AnimationSpec { duration, layers }
    }

    #[test]
    fn is_hex_color_accepts_valid() {
        assert!(is_hex_color("#ff5577"));
        assert!(is_hex_color("#000000"));
        assert!(is_hex_color("#abcdef"));
    }

    #[test]
    fn is_hex_color_rejects_invalid() {
        assert!(!is_hex_color("ff5577"));
        assert!(!is_hex_color("#ff55"));
        assert!(!is_hex_color("#gg5577"));
        assert!(!is_hex_color("red"));
        assert!(!is_hex_color(""));
    }

    #[test]
    fn parses_known_good_fixture() {
        let raw = r##"[{"duration":30,"layers":[
            {"shape":"circle","color":"#ff5577","size":"large","position":[{"frame":0,"x":250,"y":250}],
             "scale_start":0,"scale_end":100,"opacity_in":0,"opacity_out":0,"delay":0,"easing":"ease_out"},
            {"shape":"dots","color":"#44aaff","size":"small","position":[{"frame":0,"x":250,"y":250},{"frame":30,"x":250,"y":150}],
             "scale_start":50,"scale_end":150,"opacity_in":5,"opacity_out":5,"delay":8,"easing":"spring"}
        ]}]"##;
        let parsed: Vec<AnimationSpec> = serde_json::from_str(raw).expect("should parse");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].layers.len(), 2);
        assert_eq!(parsed[0].layers[0].shape, "circle");
        assert_eq!(parsed[0].layers[1].position.len(), 2);
    }

    #[test]
    fn validate_accepts_valid_fixture() {
        let mut specs = vec![spec_with_layers(
            30,
            vec![layer("circle", "#ff5577"), layer("dots", "#44aaff")],
        )];
        assert!(validate_specs(&mut specs, 1).is_ok());
    }

    #[test]
    fn validate_rejects_length_mismatch() {
        let mut specs = vec![spec_with_layers(30, vec![layer("circle", "#ff5577")])];
        let err = validate_specs(&mut specs, 5).unwrap_err();
        assert!(err.contains("does not match"));
    }

    #[test]
    fn validate_rejects_empty_layers() {
        let mut specs = vec![spec_with_layers(30, vec![])];
        let err = validate_specs(&mut specs, 1).unwrap_err();
        assert!(err.contains("empty"));
    }

    #[test]
    fn validate_truncates_excess_layers() {
        let layers: Vec<LayerSpec> = (0..8).map(|_| layer("circle", "#ff5577")).collect();
        let mut specs = vec![spec_with_layers(30, layers)];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].layers.len(), 6);
    }

    #[test]
    fn validate_rejects_unknown_shape() {
        let mut specs = vec![spec_with_layers(30, vec![layer("hexagon", "#ff5577")])];
        let err = validate_specs(&mut specs, 1).unwrap_err();
        assert!(err.contains("unknown shape"));
    }

    #[test]
    fn validate_normalizes_shape_aliases() {
        let aliases = [
            ("ellipse", "circle"),
            ("rectangle", "square"),
            ("bar", "bars"),
            ("dot", "dots"),
            ("arcs", "arc"),
            ("lines", "line"),
        ];
        for (alias, expected) in aliases {
            let mut specs = vec![spec_with_layers(30, vec![layer(alias, "#ff5577")])];
            validate_specs(&mut specs, 1).expect(alias);
            assert_eq!(specs[0].layers[0].shape, expected);
        }
    }

    #[test]
    fn validate_rejects_bad_color() {
        let mut specs = vec![spec_with_layers(30, vec![layer("circle", "red")])];
        let err = validate_specs(&mut specs, 1).unwrap_err();
        assert!(err.contains("bad color"));
    }

    #[test]
    fn validate_lowercases_color() {
        let mut specs = vec![spec_with_layers(30, vec![layer("circle", "#FF5577")])];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].layers[0].color, "#ff5577");
    }

    #[test]
    fn validate_defaults_unknown_size() {
        let mut l = layer("circle", "#ff5577");
        l.size = "huge".into();
        let mut specs = vec![spec_with_layers(30, vec![l])];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].layers[0].size, "large");
    }

    #[test]
    fn validate_defaults_unknown_easing() {
        let mut l = layer("circle", "#ff5577");
        l.easing = "bounce".into();
        let mut specs = vec![spec_with_layers(30, vec![l])];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].layers[0].easing, "spring");
    }

    #[test]
    fn validate_clamps_scale() {
        let mut l = layer("circle", "#ff5577");
        l.scale_start = -50.0;
        l.scale_end = 500.0;
        let mut specs = vec![spec_with_layers(30, vec![l])];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].layers[0].scale_start, 0.0);
        assert_eq!(specs[0].layers[0].scale_end, 300.0);
    }

    #[test]
    fn validate_clamps_duration_below_min() {
        let mut specs = vec![spec_with_layers(5, vec![layer("circle", "#ff5577")])];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].duration, DURATION_MIN);
    }

    #[test]
    fn validate_clamps_duration_above_max() {
        let mut specs = vec![spec_with_layers(200, vec![layer("circle", "#ff5577")])];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].duration, DURATION_MAX);
    }

    #[test]
    fn validate_clamps_delay_to_within_duration() {
        let mut l = layer("circle", "#ff5577");
        l.delay = 999;
        let mut specs = vec![spec_with_layers(30, vec![l])];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].layers[0].delay, 0);
    }

    #[test]
    fn validate_clamps_opacity_envelope_overflow() {
        let mut l = layer("circle", "#ff5577");
        l.opacity_in = 20;
        l.opacity_out = 20;
        let mut specs = vec![spec_with_layers(30, vec![l])];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].layers[0].opacity_in, 7);
        assert_eq!(specs[0].layers[0].opacity_out, 7);
    }

    #[test]
    fn validate_adds_default_position_when_missing() {
        let mut l = layer("circle", "#ff5577");
        l.position = vec![];
        let mut specs = vec![spec_with_layers(30, vec![l])];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].layers[0].position.len(), 1);
        assert_eq!(specs[0].layers[0].position[0].x, 250.0);
        assert_eq!(specs[0].layers[0].position[0].y, 250.0);
    }

    #[test]
    fn validate_accepts_all_enum_combinations() {
        for shape in VALID_SHAPES {
            for easing in VALID_EASINGS {
                for size in VALID_SIZES {
                    let mut l = layer(shape, "#ff5577");
                    l.easing = easing.to_string();
                    l.size = size.to_string();
                    let mut specs = vec![spec_with_layers(30, vec![l])];
                    assert!(
                        validate_specs(&mut specs, 1).is_ok(),
                        "{shape}/{easing}/{size}"
                    );
                }
            }
        }
    }
}
