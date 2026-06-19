use crate::diagnostics::AntiLlmDiagnostic;
use crate::observations::Observation;

pub fn evaluate(obs: &[Observation]) -> Vec<AntiLlmDiagnostic> {
    let mut diags = Vec::new();

    for o in obs {
        // Route log treated as route execution
        if o.construct == "Routing to PackPlan"
            || o.context.contains("Routing to PackPlan -> Staging")
        {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-ROUTE-001".to_string(),
                category: "route".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Route log treated as route execution. A print or log of the route does not prove execution.".to_string(),
                forbidden_implication: "Log(RouteIntent) => RouteExecution".to_string(),
                blocking: true,
                required_correction: "Collect concrete evidence at each step of the route (CodeAction, clap admission, PackPlan, Staging, MutationGate, Receipt).".to_string(),
                required_next_proof: "Require active receipt matching the checkpoint.".to_string(),
            });
        }

        // Static scan substituted for route proof
        if (o.message.contains("static scan") && o.message.contains("route proof"))
            || o.construct == "static scan as route proof"
        {
            diags.push(AntiLlmDiagnostic {
                code: "ANTI-LLM-ROUTE-008".to_string(),
                category: "route".to_string(),
                file_path: o.file_path.clone(),
                line: o.line,
                column: o.column,
                message: "Static scan substituted for route proof. Lack of bad strings does not prove mutation was safely routed.".to_string(),
                forbidden_implication: "¬KnownBadPath => AllMutation lawfully routed".to_string(),
                blocking: true,
                required_correction: "Use dynamic mutation gate check instead of static text scan checks.".to_string(),
                required_next_proof: "Prove MutationGate denial handles unadmitted paths.".to_string(),
            });
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observations::Observation;

    fn obs(construct: &str, message: &str, context: &str) -> Observation {
        Observation {
            file_path: "f.rs".into(), start_byte: 0, end_byte: 0,
            line: 1, column: 1,
            kind: "route".into(),
            construct: construct.into(),
            context: context.into(),
            message: message.into(),
        }
    }

    #[test]
    fn route_001_fires_on_routing_to_packplan_construct() {
        let o = obs("Routing to PackPlan", "", "");
        let diags = evaluate(&[o]);
        assert!(diags.iter().any(|d| d.code == "ANTI-LLM-ROUTE-001"));
    }

    #[test]
    fn route_001_fires_when_context_contains_routing_string() {
        let o = obs("other", "", "Routing to PackPlan -> Staging");
        let diags = evaluate(&[o]);
        assert!(diags.iter().any(|d| d.code == "ANTI-LLM-ROUTE-001"));
    }

    #[test]
    fn route_008_fires_on_static_scan_construct() {
        let o = obs("static scan as route proof", "", "");
        let diags = evaluate(&[o]);
        assert!(diags.iter().any(|d| d.code == "ANTI-LLM-ROUTE-008"));
    }

    #[test]
    fn route_008_fires_when_message_contains_both_phrases() {
        let o = obs("other", "static scan as route proof evidence", "");
        let diags = evaluate(&[o]);
        assert!(diags.iter().any(|d| d.code == "ANTI-LLM-ROUTE-008"));
    }

    #[test]
    fn no_obs_produces_no_diags() {
        assert!(evaluate(&[]).is_empty());
    }

    #[test]
    fn unrelated_obs_produces_no_diags() {
        let o = obs("console.log", "just a log", "");
        assert!(evaluate(&[o]).is_empty());
    }

    #[test]
    fn route_001_is_blocking() {
        let o = obs("Routing to PackPlan", "", "");
        let diags = evaluate(&[o]);
        assert!(diags.iter().all(|d| d.blocking));
    }
}
