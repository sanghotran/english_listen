//! Server-side mirror of `src/utils/diff.ts`. `record_attempt` recomputes accuracy here
//! instead of trusting whatever number the client sends.

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
    let ref_tokens = tokenize(reference);
    let in_tokens = tokenize(input);
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
}
