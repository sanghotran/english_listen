//! Server-side mirror of `src/utils/diff.ts`. `record_attempt` recomputes accuracy here
//! instead of trusting whatever number the client sends.

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordStatus {
    Correct,
    Missing,
    Extra,
}

#[derive(Debug, Clone)]
pub struct WordToken {
    pub text: String,
    pub status: WordStatus,
}

// A few contractions are irregular enough that the generic "stem + n't" rule below would mangle
// them (e.g. "can't" -> stem "ca" instead of "can", since "can" itself ends in "n").
static IRREGULAR_CONTRACTIONS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    vec![
        (Regex::new(r"(?i)\bwon't\b").unwrap(), "will not"),
        (Regex::new(r"(?i)\bshan't\b").unwrap(), "shall not"),
        (Regex::new(r"(?i)\bcan't\b").unwrap(), "can not"),
    ]
});
static NT_SUFFIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(\w+)n't\b").unwrap());
static RE_SUFFIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(\w+)'re\b").unwrap());
static VE_SUFFIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(\w+)'ve\b").unwrap());
static LL_SUFFIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(\w+)'ll\b").unwrap());
static IM_SUFFIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bI'm\b").unwrap());

/// Mirrors `expandContractions()` in src/utils/diff.ts: expands unambiguous contractions to
/// their full form before tokenizing, so "don't" typed as "do not" (or vice versa) scores as
/// correct. `'s`/`'d` are genuinely ambiguous (is/has, would/had) and are left alone.
fn expand_contractions(text: &str) -> String {
    let mut result = text.to_string();
    for (pattern, replacement) in IRREGULAR_CONTRACTIONS.iter() {
        result = pattern.replace_all(&result, *replacement).into_owned();
    }
    result = NT_SUFFIX.replace_all(&result, "$1 not").into_owned();
    result = RE_SUFFIX.replace_all(&result, "$1 are").into_owned();
    result = VE_SUFFIX.replace_all(&result, "$1 have").into_owned();
    result = LL_SUFFIX.replace_all(&result, "$1 will").into_owned();
    result = IM_SUFFIX.replace_all(&result, "I am").into_owned();
    result
}

fn tokenize(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

fn normalize(word: &str) -> String {
    word.trim_matches(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '-'))
        .to_lowercase()
}

/// Word-level diff between what the user typed and the reference transcript, via an
/// LCS alignment over normalized words (equivalent classification to jsdiff's `diffArrays`).
pub fn diff_words(input: &str, reference: &str) -> Vec<WordToken> {
    let expanded_reference = expand_contractions(reference);
    let expanded_input = expand_contractions(input);
    let ref_tokens = tokenize(&expanded_reference);
    let in_tokens = tokenize(&expanded_input);
    let ref_norm: Vec<String> = ref_tokens.iter().map(|w| normalize(w)).collect();
    let in_norm: Vec<String> = in_tokens.iter().map(|w| normalize(w)).collect();

    let n = ref_tokens.len();
    let m = in_tokens.len();
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if ref_norm[i] == in_norm[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut tokens = Vec::with_capacity(n + m);
    let mut i = 0;
    let mut j = 0;
    while i < n && j < m {
        if ref_norm[i] == in_norm[j] {
            tokens.push(WordToken {
                text: ref_tokens[i].to_string(),
                status: WordStatus::Correct,
            });
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            tokens.push(WordToken {
                text: ref_tokens[i].to_string(),
                status: WordStatus::Missing,
            });
            i += 1;
        } else {
            tokens.push(WordToken {
                text: in_tokens[j].to_string(),
                status: WordStatus::Extra,
            });
            j += 1;
        }
    }
    while i < n {
        tokens.push(WordToken {
            text: ref_tokens[i].to_string(),
            status: WordStatus::Missing,
        });
        i += 1;
    }
    while j < m {
        tokens.push(WordToken {
            text: in_tokens[j].to_string(),
            status: WordStatus::Extra,
        });
        j += 1;
    }
    tokens
}

struct Run {
    status: WordStatus,
    length: u32,
}

fn to_runs(tokens: &[WordToken]) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    for token in tokens {
        if let Some(last) = runs.last_mut() {
            if last.status == token.status {
                last.length += 1;
                continue;
            }
        }
        runs.push(Run {
            status: token.status,
            length: 1,
        });
    }
    runs
}

/// Accuracy = 1 - word error rate. A missing+extra run pair is one substitution edit per
/// word pair, not two separate errors — otherwise a single wrong word counts as 2 errors.
pub fn compute_accuracy(tokens: &[WordToken]) -> f64 {
    let reference_count = tokens.iter().filter(|t| t.status != WordStatus::Extra).count();
    if reference_count == 0 {
        return 0.0;
    }

    let runs = to_runs(tokens);
    let mut error_count: u32 = 0;
    let mut i = 0;
    while i < runs.len() {
        let run = &runs[i];
        if run.status == WordStatus::Correct {
            i += 1;
            continue;
        }
        if let Some(next) = runs.get(i + 1) {
            let is_substitution_pair = (run.status == WordStatus::Missing
                && next.status == WordStatus::Extra)
                || (run.status == WordStatus::Extra && next.status == WordStatus::Missing);
            if is_substitution_pair {
                error_count += run.length.max(next.length);
                i += 2;
                continue;
            }
        }
        error_count += run.length;
        i += 1;
    }
    (1.0 - error_count as f64 / reference_count as f64).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_is_perfect() {
        let tokens = diff_words("hello world", "hello world");
        assert_eq!(compute_accuracy(&tokens), 1.0);
        assert!(tokens.iter().all(|t| t.status == WordStatus::Correct));
    }

    #[test]
    fn case_and_punctuation_are_ignored() {
        let tokens = diff_words("Hello, world!", "hello world");
        assert_eq!(compute_accuracy(&tokens), 1.0);
    }

    #[test]
    fn missing_word_counts_as_one_error() {
        let tokens = diff_words("hello", "hello world");
        assert_eq!(compute_accuracy(&tokens), 0.5);
    }

    #[test]
    fn extra_word_counts_as_one_error() {
        let tokens = diff_words("hello world today", "hello world");
        assert_eq!(compute_accuracy(&tokens), 0.5);
    }

    #[test]
    fn wrong_word_counts_as_single_substitution_error() {
        let tokens = diff_words("hello earth", "hello world");
        assert_eq!(compute_accuracy(&tokens), 0.5);
    }

    #[test]
    fn empty_input_is_zero_accuracy() {
        let tokens = diff_words("", "hello world");
        assert_eq!(compute_accuracy(&tokens), 0.0);
    }

    #[test]
    fn contraction_typed_out_in_full_is_correct() {
        let tokens = diff_words("I do not know", "I don't know");
        assert_eq!(compute_accuracy(&tokens), 1.0);
    }

    #[test]
    fn full_form_typed_as_contraction_is_correct() {
        let tokens = diff_words("I don't know", "I do not know");
        assert_eq!(compute_accuracy(&tokens), 1.0);
    }

    #[test]
    fn irregular_contractions_wont_cant_shant() {
        assert_eq!(compute_accuracy(&diff_words("will not", "won't")), 1.0);
        assert_eq!(compute_accuracy(&diff_words("can not", "can't")), 1.0);
        assert_eq!(compute_accuracy(&diff_words("shall not", "shan't")), 1.0);
    }

    #[test]
    fn re_ve_ll_m_contractions() {
        assert_eq!(compute_accuracy(&diff_words("we are ready", "we're ready")), 1.0);
        assert_eq!(compute_accuracy(&diff_words("they have left", "they've left")), 1.0);
        assert_eq!(compute_accuracy(&diff_words("I will go", "I'll go")), 1.0);
        assert_eq!(compute_accuracy(&diff_words("I am here", "I'm here")), 1.0);
    }

    #[test]
    fn still_catches_genuine_mismatch_after_contraction_expansion() {
        let tokens = diff_words("I don't know", "I do know");
        assert!(compute_accuracy(&tokens) < 1.0);
    }
}
