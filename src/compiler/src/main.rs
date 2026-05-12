use gemini_rust::Gemini;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Read};

const SCRIPT_ANALYZER_PROMPT: &str = r#"You are a visual-moments analyzer for a motion-graphics pipeline.
INPUT: a JSON object with two fields:
  - "script": the full narration as a string
  - "wordGroups": an ordered array of short phrases the narration is broken into

TASK: identify 3-8 moments in the narration that should be reinforced
with a small abstract Lottie animation playing behind/alongside the text.

OUTPUT: a JSON array, and NOTHING else. No markdown fences. No prose.
The first character of your response MUST be '[' and the last MUST be ']'.

Each element of the array MUST be an object with EXACTLY these fields:
  - "index"    (integer): 0-based index into wordGroups where the visual begins.
                Must be in [0, wordGroups.length - 1]. Strictly increasing across the array.
  - "concept"  (string):  one or two lowercase words (e.g. "growth").
  - "mood"     (string):  one of: "energetic","calm","tense","hopeful","somber","playful","ominous","triumphant".
  - "metaphor" (string):  2-5 word noun phrase (e.g. "expanding circle").
  - "duration" (integer): frames at 30fps, in [15, 90].

RULES:
- Pick moments of real conceptual weight, not filler.
- Do not place two visual moments closer than 3 wordGroups apart.
- Output ONLY the JSON array."#;

const ANIMATION_SPEC_PROMPT: &str = r#"
System: You are an animation spec writer. Given an animation concept,
write a spec using ONLY the primitives in this vocabulary: [vocabulary].
A spec is an array of layers. Each layer has:
- id: string
- shape: one of [shapes]
- color: hex string
- keyframes: array of { frame, transform, value, easing }
Rules:
- Maximum 5 layers
- Maximum 10 keyframes per layer
- All frame values are integers between 0 and {duration}
- Values for scale are between 0 and 2
- Values for opacity are between 0 and 1
- Values for translate are between -500 and 500
Return only JSON.
"#;

const VALID_MOODS: &[&str] = &[
    "energetic",
    "calm",
    "tense",
    "hopeful",
    "somber",
    "playful",
    "ominous",
    "triumphant",
];

const DURATION_MIN: u32 = 15;
const DURATION_MAX: u32 = 90;

#[derive(Deserialize)]
struct ScriptInput {
    script: String,
    #[serde(rename = "wordGroups")]
    word_groups: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct VisualMoment {
    index: u32,
    concept: String,
    mood: String,
    metaphor: String,
    duration: u32,
}

async fn gemini_prompt(
    prompt: &str,
    system_prompt: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY").map_err(|_| "GEMINI_API_KEY is not set")?;
    let client = Gemini::new(api_key)?;

    let response = client
        .generate_content()
        .with_system_prompt(system_prompt.unwrap_or(""))
        .with_user_message(prompt)
        .execute()
        .await?;

    Ok(response.text())
}

fn strip_fences(s: &str) -> &str {
    let s = s.trim();
    let s = s
        .strip_prefix("```json")
        .or_else(|| s.strip_prefix("```"))
        .unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim()
}

fn validate_moments(moments: &mut Vec<VisualMoment>, n_word_groups: usize) -> Result<(), String> {
    if moments.is_empty() {
        return Err("gemini returned zero visual moments".to_string());
    }
    let n = n_word_groups as u32;
    let mut prev: Option<u32> = None;
    for m in moments.iter_mut() {
        if m.index >= n {
            return Err(format!(
                "index {} out of range for wordGroups.length={}",
                m.index, n
            ));
        }
        if let Some(p) = prev {
            if m.index <= p {
                return Err(format!(
                    "indices must be strictly increasing: {} followed by {}",
                    p, m.index
                ));
            }
        }
        prev = Some(m.index);
        if !VALID_MOODS.contains(&m.mood.as_str()) {
            return Err(format!("unknown mood: {:?}", m.mood));
        }
        if m.duration < DURATION_MIN {
            m.duration = DURATION_MIN;
        } else if m.duration > DURATION_MAX {
            m.duration = DURATION_MAX;
        }
    }
    Ok(())
}

async fn script_analyzer() -> Result<(), Box<dyn std::error::Error>> {
    let mut raw_input = String::new();
    io::stdin().read_to_string(&mut raw_input)?;
    let raw_input = raw_input.trim();
    if raw_input.is_empty() {
        return Err("empty input on stdin; expected data.json contents".into());
    }

    let input: ScriptInput =
        serde_json::from_str(raw_input).map_err(|e| format!("stdin is not valid JSON: {e}"))?;

    if input.word_groups.is_empty() {
        return Err("input wordGroups is empty".into());
    }

    let user_msg = serde_json::to_string(&serde_json::json!({
        "script": input.script,
        "wordGroups": input.word_groups,
    }))?;

    let raw = gemini_prompt(&user_msg, Some(SCRIPT_ANALYZER_PROMPT)).await?;
    let cleaned = strip_fences(&raw);

    let mut moments: Vec<VisualMoment> = serde_json::from_str(cleaned)
        .map_err(|e| format!("gemini returned invalid JSON: {e}\n---\n{cleaned}"))?;

    validate_moments(&mut moments, input.word_groups.len())?;

    eprintln!(
        "analyzed {} wordGroups, produced {} moments",
        input.word_groups.len(),
        moments.len()
    );
    println!("{}", serde_json::to_string(&moments)?);
    Ok(())
}

fn animation_spec() -> Result<(), Box<dyn std::error::Error>> {
    // gemini designs each animation based on a constrained vocabulary syntax
    todo!()
}

fn lottie_compiler() {
    // maps keywords to valid lottie json
    todo!()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    script_analyzer().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moment(index: u32, mood: &str, duration: u32) -> VisualMoment {
        VisualMoment {
            index,
            concept: "growth".into(),
            mood: mood.into(),
            metaphor: "expanding circle".into(),
            duration,
        }
    }

    #[test]
    fn strip_fences_removes_json_fence() {
        let raw = "```json\n[{\"a\":1}]\n```";
        assert_eq!(strip_fences(raw), "[{\"a\":1}]");
    }

    #[test]
    fn strip_fences_removes_plain_fence() {
        let raw = "```\n[1,2]\n```";
        assert_eq!(strip_fences(raw), "[1,2]");
    }

    #[test]
    fn strip_fences_noop_on_clean_json() {
        let raw = "[{\"a\":1}]";
        assert_eq!(strip_fences(raw), "[{\"a\":1}]");
    }

    #[test]
    fn strip_fences_handles_surrounding_whitespace() {
        let raw = "   \n```json\n[]\n```\n  ";
        assert_eq!(strip_fences(raw), "[]");
    }

    #[test]
    fn parses_known_good_fixture() {
        let raw = r#"[
            {"index":0,"concept":"growth","mood":"hopeful","metaphor":"expanding circle","duration":30},
            {"index":5,"concept":"friction","mood":"tense","metaphor":"converging arcs","duration":24}
        ]"#;
        let parsed: Vec<VisualMoment> = serde_json::from_str(raw).expect("should parse");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].concept, "growth");
        assert_eq!(parsed[1].mood, "tense");
    }

    #[test]
    fn validate_accepts_valid_fixture() {
        let mut moments = vec![
            moment(0, "hopeful", 30),
            moment(5, "tense", 24),
            moment(10, "triumphant", 40),
        ];
        assert!(validate_moments(&mut moments, 33).is_ok());
    }

    #[test]
    fn validate_rejects_out_of_range_index() {
        let mut moments = vec![moment(50, "hopeful", 30)];
        let err = validate_moments(&mut moments, 33).unwrap_err();
        assert!(err.contains("out of range"));
    }

    #[test]
    fn validate_rejects_non_monotonic_indices() {
        let mut moments = vec![moment(5, "hopeful", 30), moment(3, "tense", 30)];
        let err = validate_moments(&mut moments, 33).unwrap_err();
        assert!(err.contains("strictly increasing"));
    }

    #[test]
    fn validate_rejects_duplicate_indices() {
        let mut moments = vec![moment(5, "hopeful", 30), moment(5, "tense", 30)];
        let err = validate_moments(&mut moments, 33).unwrap_err();
        assert!(err.contains("strictly increasing"));
    }

    #[test]
    fn validate_rejects_empty_array() {
        let mut moments: Vec<VisualMoment> = vec![];
        let err = validate_moments(&mut moments, 33).unwrap_err();
        assert!(err.contains("zero"));
    }

    #[test]
    fn validate_rejects_unknown_mood() {
        let mut moments = vec![moment(0, "excited", 30)];
        let err = validate_moments(&mut moments, 33).unwrap_err();
        assert!(err.contains("unknown mood"));
    }

    #[test]
    fn validate_clamps_duration_below_min() {
        let mut moments = vec![moment(0, "hopeful", 5)];
        validate_moments(&mut moments, 33).unwrap();
        assert_eq!(moments[0].duration, DURATION_MIN);
    }

    #[test]
    fn validate_clamps_duration_above_max() {
        let mut moments = vec![moment(0, "hopeful", 200)];
        validate_moments(&mut moments, 33).unwrap();
        assert_eq!(moments[0].duration, DURATION_MAX);
    }

    #[test]
    fn validate_accepts_all_mood_values() {
        for mood in VALID_MOODS {
            let mut moments = vec![moment(0, mood, 30)];
            assert!(
                validate_moments(&mut moments, 33).is_ok(),
                "mood {mood} should be valid"
            );
        }
    }

    // #[test]
    // fn unused_functions_still_compile() {
    //     // animation_spec and lottie_compiler are intentionally `todo!()` stubs.
    //     let _ = animation_spec as fn();
    //     let _ = lottie_compiler as fn();
    // }
}
