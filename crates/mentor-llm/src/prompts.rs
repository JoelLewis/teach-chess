/// Format a system + user message pair in Gemma 4 instruction format.
///
/// Gemma 4 replaced Gemma 2/3's `<start_of_turn>` markers with
/// `<|turn>{role}\n{content}<turn|>\n` turns and a `<|turn>model\n` generation
/// prompt (see `common_chat_params_init_gemma4` in llama.cpp's `common/chat.cpp`).
/// llama.cpp's C-side `llama_chat_apply_template` heuristics do not know this
/// template (only its Jinja engine does, which llama-cpp-2 does not expose),
/// so the format is applied here instead of via the model's chat template.
///
/// The BOS token is intentionally omitted — tokenization adds it (`AddBos::Always`).
///
/// Prompt *content* (system prompts, fact verbalization, domain grammars) is
/// deliberately app-side; this crate only owns the chat format.
pub fn format_chat(system: &str, user: &str) -> String {
    format!("<|turn>system\n{system}<turn|>\n<|turn>user\n{user}<turn|>\n<|turn>model\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_chat_has_gemma4_markers() {
        let prompt = format_chat("system text", "user text");
        assert!(prompt.starts_with("<|turn>system\nsystem text<turn|>\n"));
        assert!(prompt.contains("<|turn>user\nuser text<turn|>\n"));
        assert!(prompt.ends_with("<|turn>model\n"));
    }
}
