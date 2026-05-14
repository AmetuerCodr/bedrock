mod animation_spec;
mod gemini;
mod lottie_compiler;
mod script_analyzer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subcommand = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "script-analyzer".to_string());

    match subcommand.as_str() {
        // Stage-by-stage subcommands (useful for debugging individual stages).
        "script-analyzer" => script_analyzer::run().await,
        "animation-spec"  => animation_spec::run().await,
        "lottie-compiler" => lottie_compiler::run().await,
        // Option-A full pipeline: data.json → Vec<LottieAnimation> in one process.
        "compile"         => lottie_compiler::run_pipeline().await,
        other => Err(format!(
            "unknown subcommand: {other:?} (expected script-analyzer, animation-spec, lottie-compiler, or compile)"
        )
        .into()),
    }
}
