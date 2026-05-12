use gemini_rust::Gemini;
use std::env;

pub async fn prompt(
    user_msg: &str,
    system: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY").map_err(|_| "GEMINI_API_KEY is not set")?;
    let client = Gemini::new(api_key)?;

    let response = client
        .generate_content()
        .with_system_prompt(system.unwrap_or(""))
        .with_user_message(user_msg)
        .execute()
        .await?;

    Ok(response.text())
}

pub fn strip_fences(s: &str) -> &str {
    let s = s.trim();
    let s = s
        .strip_prefix("```json")
        .or_else(|| s.strip_prefix("```"))
        .unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_fences_removes_json_fence() {
        assert_eq!(strip_fences("```json\n[{\"a\":1}]\n```"), "[{\"a\":1}]");
    }

    #[test]
    fn strip_fences_removes_plain_fence() {
        assert_eq!(strip_fences("```\n[1,2]\n```"), "[1,2]");
    }

    #[test]
    fn strip_fences_noop_on_clean_json() {
        assert_eq!(strip_fences("[{\"a\":1}]"), "[{\"a\":1}]");
    }

    #[test]
    fn strip_fences_handles_surrounding_whitespace() {
        assert_eq!(strip_fences("   \n```json\n[]\n```\n  "), "[]");
    }
}
