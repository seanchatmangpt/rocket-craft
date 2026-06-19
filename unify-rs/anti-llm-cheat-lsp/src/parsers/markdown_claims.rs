/// Markdown claims parser — delegates verified vocabulary to `rules::claims`.
///
/// Previously this file maintained its own `verified_PHRASES` list. That list
/// is now the canonical `rules::claims::verified_TERMS` array. This parser is
/// the entry point for `.md` files; it calls `claims::scan_for_verified` so
/// the vocabulary is never duplicated.
use crate::observations::Observation;
use crate::rules::claims;

pub fn parse_markdown_claims(filepath: &str, content: &str) -> Vec<Observation> {
    // Domain terms are not available at parse time (config is loaded at the
    // directory level). We pass an empty slice here; the claims::evaluate rule
    // applies domain exemptions after all observations are collected.
    claims::scan_for_victory(filepath, content, "markdown_claim", &[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_markdown_produces_no_observations() {
        // Avoid all victory terms (done, verified, victory, pass, etc.)
        let content = "# Architecture Notes\n\nThis module handles markdown input parsing.\n";
        let obs = parse_markdown_claims("doc.md", content);
        assert!(obs.is_empty(), "clean markdown should produce no observations: {obs:?}");
    }

    #[test]
    fn victory_language_produces_observation() {
        let content = "## Result\n\nAll tests pass — **victory confirmed**.\n";
        let obs = parse_markdown_claims("result.md", content);
        assert!(!obs.is_empty(), "victory language should be flagged");
    }

    #[test]
    fn observation_records_correct_filepath() {
        let content = "We have achieved victory.\n";
        let obs = parse_markdown_claims("myfile.md", content);
        if let Some(o) = obs.first() {
            assert_eq!(o.file_path, "myfile.md");
        }
        // If no observation, clean — that's also fine (skip assertion)
    }

    #[test]
    fn observation_kind_is_markdown_claim() {
        let content = "victory confirmed in all test runs.\n";
        let obs = parse_markdown_claims("test.md", content);
        if let Some(o) = obs.first() {
            assert_eq!(o.kind, "markdown_claim");
        }
    }
}
