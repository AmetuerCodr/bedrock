// =====================================================================
// BLUEPRINT: merging the pipeline
// =====================================================================
//
// The three Rust modules form a 3-stage pipeline:
//
//     stdin (data.json)
//        |
//        v
//   script_analyzer  --->  Vec<VisualMoment>
//        |
//        v
//   animation_spec   --->  Vec<AnimationSpec>
//        |
//        v
//   lottie_compiler  --->  Vec<lottie::Animation>  (this module)
//        |
//        v
//   stdout (JSON: array of Lottie animation files)
//
// Today the three stages are exposed as separate CLI subcommands in
// main.rs (script-analyzer | animation-spec | lottie-compiler), and the
// TS side only invokes `script-analyzer`. To merge them into one end-to-end
// run, you have two reasonable shapes:
//
// ---------------------------------------------------------------------
// OPTION A - new `compile` subcommand that runs all three in-process
// ---------------------------------------------------------------------
// Add a `pub async fn run_pipeline()` here (or in main.rs) that calls
// each stage's pure-logic functions directly, skipping stdin/stdout
// between stages. Each existing `run()` keeps reading stdin and writing
// stdout for individual debugging.
//
//   pub async fn run_pipeline() -> Result<(), Box<dyn Error>> {
//       // 1. Read data.json from stdin (same as script_analyzer::run).
//       let input: ScriptInput = read_stdin_json()?;
//
//       // 2. script_analyzer logic -> Vec<VisualMoment>.
//       //    Refactor script_analyzer::run() into:
//       //      - read_stdin_json()    (pure I/O)
//       //      - analyze(input)       (pure logic, returns Vec<VisualMoment>)
//       //      - print_stdout(...)    (pure I/O)
//       //    so analyze() is reusable here.
//       let moments = crate::script_analyzer::analyze(input).await?;
//
//       // 3. animation_spec logic -> Vec<AnimationSpec>.
//       //    Same refactor: extract `design(moments)` from `run()`.
//       let specs = crate::animation_spec::design(&moments).await?;
//
//       // 4. lottie_compiler logic -> Vec<lottie::Animation>.
//       let animations = compile(&specs)?;
//
//       // 5. Emit final JSON.
//       println!("{}", serde_json::to_string(&animations)?);
//       Ok(())
//   }
//
// Then in main.rs add: "compile" => lottie_compiler::run_pipeline().await
//
// Pros: one Gemini round trip per stage, no JSON serialize/parse between
//       stages, no process boundary. Faster, simpler to debug.
// Cons: requires extracting the pure logic out of each `run()` so it can
//       be called without touching stdin/stdout.
//
// ---------------------------------------------------------------------
// OPTION B - shell-style pipe from TS
// ---------------------------------------------------------------------
// Keep each stage as a separate subprocess and chain them from
// lottiecompiler.ts using Bun.spawn pipes:
//
//   ts: data.json -> stdin of (./compiler script-analyzer)
//                 -> stdout to stdin of (./compiler animation-spec)
//                 -> stdout to stdin of (./compiler lottie-compiler)
//                 -> stdout parsed as Vec<lottie::Animation>
//
// Pros: zero Rust changes needed beyond implementing this module's
//       `run()`; each stage stays a clean single-purpose binary;
//       observable between stages.
// Cons: 3 process spawns per render, 3 JSON parse rounds.
//
// Recommendation: Option A. The stages already share types (VisualMoment
// is imported from script_analyzer into animation_spec); extracting the
// pure functions is a small refactor that also makes the modules more
// testable end-to-end without a real Gemini call (just mock the pure
// logic boundaries).
//
// ---------------------------------------------------------------------
// What `run()` in THIS module still needs to do
// ---------------------------------------------------------------------
// Independent of A vs B, implement the leaf stage first:
//
//   1. Read Vec<AnimationSpec> from stdin (mirror animation_spec::run).
//   2. For each spec, build a Lottie JSON object:
//        - Define `struct LottieAnimation { v, fr, ip, op, w, h, layers, ... }`
//          matching the Lottie schema (v=5.5.7, fr=30, w/h=1080 or 500).
//        - Map AnimationSpec.shape  -> Lottie shape layer (ty=4 + sh)
//          using a HashMap<&str, fn(&AnimationSpec) -> serde_json::Value>
//          or a `match spec.shape.as_str() { "circle" => ..., ... }`.
//        - Map AnimationSpec.motion -> keyframe sequence on the layer's
//          transform (position/scale/rotation animatable properties).
//        - Map AnimationSpec.easing -> bezier control points on the
//          keyframes (ease_in_out -> [0.42, 0, 0.58, 1], etc.).
//        - Set ip=0, op=spec.duration, and use spec.color for fills.
//   3. Validate: required Lottie fields present, op > ip, w/h > 0.
//      Reuse the validate_* pattern from animation_spec.rs.
//   4. Print Vec<LottieAnimation> as JSON to stdout.
//
// Keep the commented Lottie JSON skeleton from the original main.rs
// nearby as a reference for the field shape:
//
//   {
//     "v": "5.5.7", "fr": 30, "ip": 0, "op": 90,
//     "w": 500, "h": 500, "nm": "Example", "ddd": 0,
//     "assets": [],
//     "layers": [
//       { "ddd": 0, "ind": 1, "ty": 4, "nm": "Circle", "ks": {...}, "shapes": [...] }
//     ]
//   }
//
// Add unit tests mirroring animation_spec.rs:
//   - parses a known-good Lottie fixture
//   - validate rejects missing fields / bad op<=ip / bad w,h
//   - each shape variant produces a non-empty layers array
//   - each motion variant produces non-empty keyframes
// =====================================================================

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // maps keywords to valid lottie json
    todo!()
}
