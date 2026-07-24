//! Splits a lesson transcript into sentence-group segments sized by CEFR level, so a learner
//! dictates a few sentences at a time instead of an entire ~5 minute transcript in one go.
//! Mirrored exactly in `src/utils/segments.ts` — `record_attempt` scores against the segment
//! text computed here (server-authoritative), so the two implementations must agree on
//! boundaries for a given transcript.

/// More sentences per segment as the level goes up, since higher levels are expected to
/// handle longer working-memory spans.
fn chunk_size_for_level(level: &str) -> usize {
    match level {
        "A1" => 2,
        "A2" => 3,
        "B1" => 4,
        _ => 3,
    }
}

/// Splits on `.`/`!`/`?` runs (e.g. "...", "?!"), keeping the punctuation attached to the
/// sentence it closes. No lookbehind in the `regex` crate, so this walks char-by-char instead
/// of a single pattern — kept simple on purpose (abbreviations like "U.S." will split early;
/// good enough for chunking dictation practice, not a linguistic sentence boundary detector).
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        current.push(c);
        if c == '.' || c == '!' || c == '?' {
            while i + 1 < chars.len() && matches!(chars[i + 1], '.' | '!' | '?') {
                i += 1;
                current.push(chars[i]);
            }
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
        i += 1;
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }
    sentences
}

/// Groups `transcript`'s sentences into level-sized chunks. Always returns at least one
/// segment for non-empty input (mirrors `split_sentences` always producing >= 1 sentence).
pub fn segment_transcript(transcript: &str, level: &str) -> Vec<String> {
    let sentences = split_sentences(transcript);
    let size = chunk_size_for_level(level);
    sentences
        .chunks(size.max(1))
        .map(|group| group.join(" "))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_sentence_terminators() {
        let sentences = split_sentences("One. Two! Three? Four.");
        assert_eq!(sentences, vec!["One.", "Two!", "Three?", "Four."]);
    }

    #[test]
    fn keeps_multi_punctuation_together() {
        let sentences = split_sentences("Wait... really?! Yes.");
        assert_eq!(sentences, vec!["Wait...", "really?!", "Yes."]);
    }

    #[test]
    fn trailing_text_without_terminator_is_its_own_sentence() {
        let sentences = split_sentences("One. trailing fragment");
        assert_eq!(sentences, vec!["One.", "trailing fragment"]);
    }

    #[test]
    fn empty_input_has_no_sentences() {
        assert!(split_sentences("").is_empty());
        assert!(segment_transcript("", "B1").is_empty());
    }

    #[test]
    fn groups_by_level_chunk_size() {
        let transcript = "One. Two. Three. Four. Five. Six. Seven. Eight.";
        assert_eq!(
            segment_transcript(transcript, "A1"),
            vec!["One. Two.", "Three. Four.", "Five. Six.", "Seven. Eight."]
        );
        assert_eq!(
            segment_transcript(transcript, "A2"),
            vec!["One. Two. Three.", "Four. Five. Six.", "Seven. Eight."]
        );
        assert_eq!(
            segment_transcript(transcript, "B1"),
            vec!["One. Two. Three. Four.", "Five. Six. Seven. Eight."]
        );
    }
}
