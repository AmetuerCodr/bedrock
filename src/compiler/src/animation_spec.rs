use crate::gemini;
use crate::script_analyzer::VisualMoment;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

const SYSTEM_PROMPT: &str = r##"You are a motion-graphics designer for a Lottie compiler.

INPUT: a JSON object with one field "moments" - an ordered array of visual
moment descriptors. Each moment has:
  - "concept"  (string)
  - "mood"     (string, one of: "energetic","calm","tense","hopeful","somber","playful","ominous","triumphant")
  - "metaphor" (string, short noun phrase)
  - "duration" (integer, frames at 30fps)

TASK: for each moment IN ORDER, design ONE concrete animation specification.

OUTPUT: a JSON array of length EXACTLY equal to the input moments array.
Output ONLY the array. No markdown fences. No prose.

Each element MUST be an object with EXACTLY these fields:
  - "shape"    (string):  one of: "circle","square","triangle","line","arc","bars","dots"
  - "motion"   (string):  one of: "expand","contract","pulse","drift","rotate","shake","appear"
  - "color"    (string):  a hex color in the form "#rrggbb" (lowercase, 7 chars total)
  - "size"     (string):  one of: "small","medium","large"
  - "easing"   (string):  one of: "linear","ease_in","ease_out","ease_in_out","spring"
  - "duration" (integer): frames at 30fps, in [15, 90]. Copy from the input moment.

RULES:
- The shape and motion should be visually consistent with the metaphor.
- The motion and easing should reinforce the mood.
- The color should evoke the mood (warm for hopeful/triumphant/energetic, cool for calm/somber, dark for ominous, etc.).
- Vary the combinations across the output - avoid repeating the same shape+motion twice in a row.
- Output ONLY the JSON array."##;

const VALID_SHAPES: &[&str] = &[
    "circle", "square", "triangle", "line", "arc", "bars", "dots",
];
const VALID_MOTIONS: &[&str] = &[
    "expand", "contract", "pulse", "drift", "rotate", "shake", "appear",
];
const VALID_SIZES: &[&str] = &["small", "medium", "large"];
const VALID_EASINGS: &[&str] = &[
    "linear",
    "ease_in",
    "ease_out",
    "ease_in_out",
    "spring",
];

const DURATION_MIN: u32 = 15;
const DURATION_MAX: u32 = 90;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AnimationSpec {
    pub shape: String,
    pub motion: String,
    pub color: String,
    pub size: String,
    pub easing: String,
    pub duration: u32,
}

fn is_hex_color(s: &str) -> bool {
    s.len() == 7
        && s.starts_with('#')
        && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

fn validate_specs(
    specs: &mut Vec<AnimationSpec>,
    expected_len: usize,
) -> Result<(), String> {
    if specs.len() != expected_len {
        return Err(format!(
            "spec count {} does not match moment count {}",
            specs.len(),
            expected_len
        ));
    }
    for s in specs.iter_mut() {
        if !VALID_SHAPES.contains(&s.shape.as_str()) {
            return Err(format!("unknown shape: {:?}", s.shape));
        }
        if !VALID_MOTIONS.contains(&s.motion.as_str()) {
            return Err(format!("unknown motion: {:?}", s.motion));
        }
        if !VALID_SIZES.contains(&s.size.as_str()) {
            return Err(format!("unknown size: {:?}", s.size));
        }
        if !VALID_EASINGS.contains(&s.easing.as_str()) {
            return Err(format!("unknown easing: {:?}", s.easing));
        }
        s.color = s.color.to_ascii_lowercase();
        if !is_hex_color(&s.color) {
            return Err(format!(
                "bad color: {:?} (expected #rrggbb)",
                s.color
            ));
        }
        if s.duration < DURATION_MIN {
            s.duration = DURATION_MIN;
        } else if s.duration > DURATION_MAX {
            s.duration = DURATION_MAX;
        }
    }
    Ok(())
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
    if moments.is_empty() {
        return Err("input moments array is empty".into());
    }

    let user_msg = serde_json::to_string(&serde_json::json!({ "moments": moments }))?;
    let raw = gemini::prompt(&user_msg, Some(SYSTEM_PROMPT)).await?;
    let cleaned = gemini::strip_fences(&raw);

    let mut specs: Vec<AnimationSpec> = serde_json::from_str(cleaned)
        .map_err(|e| format!("gemini returned invalid JSON: {e}\n---\n{cleaned}"))?;

    validate_specs(&mut specs, moments.len())?;

    eprintln!("designed {} animation specs", specs.len());
    println!("{}", serde_json::to_string(&specs)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn is_hex_color_accepts_valid() {
        assert!(is_hex_color("#ff5577"));
        assert!(is_hex_color("#000000"));
        assert!(is_hex_color("#abcdef"));
        assert!(is_hex_color("#123456"));
    }

    #[test]
    fn is_hex_color_rejects_invalid() {
        assert!(!is_hex_color("ff5577")); // missing #
        assert!(!is_hex_color("#ff55")); // too short
        assert!(!is_hex_color("#gg5577")); // non-hex char
        assert!(!is_hex_color("#ff5577cc")); // too long
        assert!(!is_hex_color("")); // empty
        assert!(!is_hex_color("red")); // color name
    }

    #[test]
    fn parses_known_good_fixture() {
        let raw = r##"[
            {"shape":"circle","motion":"expand","color":"#ff5577","size":"large","easing":"ease_out","duration":30},
            {"shape":"line","motion":"drift","color":"#3344aa","size":"medium","easing":"linear","duration":24}
        ]"##;
        let parsed: Vec<AnimationSpec> = serde_json::from_str(raw).expect("should parse");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].shape, "circle");
        assert_eq!(parsed[1].motion, "drift");
    }

    #[test]
    fn validate_accepts_valid_fixture() {
        let mut specs = vec![
            spec("circle", "expand", "#ff5577", "large", "ease_out", 30),
            spec("line", "drift", "#3344aa", "medium", "linear", 24),
        ];
        assert!(validate_specs(&mut specs, 2).is_ok());
    }

    #[test]
    fn validate_rejects_length_mismatch() {
        let mut specs = vec![spec(
            "circle", "expand", "#ff5577", "large", "ease_out", 30,
        )];
        let err = validate_specs(&mut specs, 5).unwrap_err();
        assert!(err.contains("does not match"));
    }

    #[test]
    fn validate_rejects_unknown_shape() {
        let mut specs = vec![spec(
            "hexagon", "expand", "#ff5577", "large", "ease_out", 30,
        )];
        let err = validate_specs(&mut specs, 1).unwrap_err();
        assert!(err.contains("unknown shape"));
    }

    #[test]
    fn validate_rejects_unknown_motion() {
        let mut specs = vec![spec(
            "circle", "wiggle", "#ff5577", "large", "ease_out", 30,
        )];
        let err = validate_specs(&mut specs, 1).unwrap_err();
        assert!(err.contains("unknown motion"));
    }

    #[test]
    fn validate_rejects_unknown_size() {
        let mut specs = vec![spec(
            "circle", "expand", "#ff5577", "huge", "ease_out", 30,
        )];
        let err = validate_specs(&mut specs, 1).unwrap_err();
        assert!(err.contains("unknown size"));
    }

    #[test]
    fn validate_rejects_unknown_easing() {
        let mut specs = vec![spec(
            "circle", "expand", "#ff5577", "large", "bounce", 30,
        )];
        let err = validate_specs(&mut specs, 1).unwrap_err();
        assert!(err.contains("unknown easing"));
    }

    #[test]
    fn validate_rejects_bad_color() {
        let mut specs = vec![spec("circle", "expand", "red", "large", "ease_out", 30)];
        let err = validate_specs(&mut specs, 1).unwrap_err();
        assert!(err.contains("bad color"));
    }

    #[test]
    fn validate_lowercases_color() {
        let mut specs = vec![spec(
            "circle", "expand", "#FF5577", "large", "ease_out", 30,
        )];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].color, "#ff5577");
    }

    #[test]
    fn validate_clamps_duration_below_min() {
        let mut specs = vec![spec("circle", "expand", "#ff5577", "large", "ease_out", 5)];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].duration, DURATION_MIN);
    }

    #[test]
    fn validate_clamps_duration_above_max() {
        let mut specs = vec![spec(
            "circle", "expand", "#ff5577", "large", "ease_out", 200,
        )];
        validate_specs(&mut specs, 1).unwrap();
        assert_eq!(specs[0].duration, DURATION_MAX);
    }

    #[test]
    fn validate_accepts_all_enum_combinations() {
        for shape in VALID_SHAPES {
            for motion in VALID_MOTIONS {
                for size in VALID_SIZES {
                    for easing in VALID_EASINGS {
                        let mut specs =
                            vec![spec(shape, motion, "#ff5577", size, easing, 30)];
                        assert!(
                            validate_specs(&mut specs, 1).is_ok(),
                            "{shape}/{motion}/{size}/{easing} should be valid"
                        );
                    }
                }
            }
        }
    }
}
