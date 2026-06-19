use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Defines the mathematical boundary of an admitted typestate transition
/// as extracted from the semantic ontology (O*).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyBoundary {
    pub source: String,
    pub target: String,
    pub max_latency_ns: u64,
    pub frame_window_length: u32,
}

/// A specific mathematical point in the state space permutation matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStatePermutation {
    pub source_state: String,
    pub target_state: String,
    pub simulated_latency_ns: u64,
    pub simulated_frame_window: u32,
    pub coordinate: String,
}

/// The Combinatorial Coordinate Generator:
/// Autonomously brute-forces all mathematical permutations of an admitted state space.
pub struct CombinatorialCoordinateGenerator {
    pub boundaries: Vec<OntologyBoundary>,
}

impl Default for CombinatorialCoordinateGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CombinatorialCoordinateGenerator {
    pub fn new() -> Self {
        Self {
            boundaries: Vec::new(),
        }
    }

    /// Read the ontology boundaries from a TTL file (e.g. nexus-ostar.ttl).
    pub fn load_ontology<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let content = fs::read_to_string(path.as_ref())
            .context("Failed to read ontology file")?;
        
        let mut current_boundary = false;
        let mut source = String::new();
        let mut target = String::new();
        let mut max_latency = 0;
        let mut max_frames = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("ostar:Bound_") {
                if current_boundary && !source.is_empty() && !target.is_empty() {
                    self.boundaries.push(OntologyBoundary {
                        source: source.clone(),
                        target: target.clone(),
                        max_latency_ns: max_latency,
                        frame_window_length: max_frames,
                    });
                }
                source = String::new();
                target = String::new();
                max_latency = 0;
                max_frames = 0;
                current_boundary = true;
            } else if current_boundary {
                if line.starts_with("ostar:source") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        source = parts[1].replace("ostar:", "").trim_end_matches(';').to_string();
                    }
                } else if line.starts_with("ostar:target") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        target = parts[1].replace("ostar:", "").trim_end_matches(';').to_string();
                    }
                } else if line.starts_with("ostar:maxLatencyNs") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        max_latency = parts[1].trim_end_matches(';').parse().unwrap_or(0);
                    }
                } else if line.starts_with("ostar:frameWindowLength") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        max_frames = parts[1].trim_end_matches(';').parse().unwrap_or(0);
                    }
                } else if line.ends_with('.') {
                    if !source.is_empty() && !target.is_empty() {
                        self.boundaries.push(OntologyBoundary {
                            source: source.clone(),
                            target: target.clone(),
                            max_latency_ns: max_latency,
                            frame_window_length: max_frames,
                        });
                    }
                    current_boundary = false;
                }
            }
        }
        
        if current_boundary && !source.is_empty() && !target.is_empty() {
            self.boundaries.push(OntologyBoundary {
                source,
                target,
                max_latency_ns: max_latency,
                frame_window_length: max_frames,
            });
        }
        
        Ok(())
    }

    /// Spits out the multi-dimensional matrix of every possible game state permutation.
    pub fn generate_matrix(&self) -> Vec<GameStatePermutation> {
        let mut matrix = Vec::new();

        for bound in &self.boundaries {
            // Generating a realistic boundary matrix: min, median, max latency limits.
            let latency_steps = if bound.max_latency_ns > 0 {
                vec![0, bound.max_latency_ns / 2, bound.max_latency_ns]
            } else {
                vec![0]
            };
            
            let mut frame_steps = Vec::new();
            if bound.frame_window_length == 0 {
                frame_steps.push(0);
            } else {
                for f in 1..=bound.frame_window_length {
                    frame_steps.push(f);
                }
            }

            for &latency in &latency_steps {
                for &frame in &frame_steps {
                    // Coordinate system convention:
                    // e.g. "sAttack:mParry:L8000000:F3"
                    let coord = format!("s{}:m{}:L{}:F{}", bound.source, bound.target, latency, frame);
                    matrix.push(GameStatePermutation {
                        source_state: bound.source.clone(),
                        target_state: bound.target.clone(),
                        simulated_latency_ns: latency,
                        simulated_frame_window: frame,
                        coordinate: coord,
                    });
                }
            }
        }

        matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_with(boundaries: Vec<OntologyBoundary>) -> CombinatorialCoordinateGenerator {
        CombinatorialCoordinateGenerator { boundaries }
    }

    fn bound(source: &str, target: &str, max_latency_ns: u64, frames: u32) -> OntologyBoundary {
        OntologyBoundary {
            source: source.into(),
            target: target.into(),
            max_latency_ns,
            frame_window_length: frames,
        }
    }

    // ── CombinatorialCoordinateGenerator::new ─────────────────────────────────

    #[test]
    fn new_starts_with_empty_boundaries() {
        let g = CombinatorialCoordinateGenerator::new();
        assert!(g.boundaries.is_empty());
    }

    // ── generate_matrix ───────────────────────────────────────────────────────

    #[test]
    fn empty_boundaries_produces_empty_matrix() {
        let g = gen_with(vec![]);
        assert!(g.generate_matrix().is_empty());
    }

    #[test]
    fn zero_latency_zero_frames_produces_one_permutation() {
        let g = gen_with(vec![bound("Idle", "Attack", 0, 0)]);
        let matrix = g.generate_matrix();
        assert_eq!(matrix.len(), 1);
        assert_eq!(matrix[0].source_state, "Idle");
        assert_eq!(matrix[0].target_state, "Attack");
        assert_eq!(matrix[0].simulated_latency_ns, 0);
        assert_eq!(matrix[0].simulated_frame_window, 0);
    }

    #[test]
    fn nonzero_latency_produces_three_latency_steps() {
        // max_latency=100 → [0, 50, 100]; frames=0 → [0]  → 3 rows
        let g = gen_with(vec![bound("A", "B", 100, 0)]);
        let matrix = g.generate_matrix();
        assert_eq!(matrix.len(), 3);
        let latencies: Vec<u64> = matrix.iter().map(|p| p.simulated_latency_ns).collect();
        assert_eq!(latencies, vec![0, 50, 100]);
    }

    #[test]
    fn frame_window_expands_permutations() {
        // frames=3 → [1,2,3]; latency=0 → [0]  → 3 rows
        let g = gen_with(vec![bound("A", "B", 0, 3)]);
        let matrix = g.generate_matrix();
        assert_eq!(matrix.len(), 3);
        let frames: Vec<u32> = matrix.iter().map(|p| p.simulated_frame_window).collect();
        assert_eq!(frames, vec![1, 2, 3]);
    }

    #[test]
    fn coordinate_string_encodes_source_target_latency_frame() {
        let g = gen_with(vec![bound("Idle", "Parry", 0, 0)]);
        let coord = &g.generate_matrix()[0].coordinate;
        assert!(coord.contains("sIdle"), "should contain source");
        assert!(coord.contains("mParry"), "should contain target");
        assert!(coord.contains("L0"), "should contain latency");
        assert!(coord.contains("F0"), "should contain frame");
    }

    #[test]
    fn multiple_boundaries_are_all_expanded() {
        let g = gen_with(vec![
            bound("A", "B", 0, 0),
            bound("B", "C", 0, 0),
        ]);
        let matrix = g.generate_matrix();
        assert_eq!(matrix.len(), 2);
        assert_eq!(matrix[0].source_state, "A");
        assert_eq!(matrix[1].source_state, "B");
    }

    // ── OntologyBoundary serde ────────────────────────────────────────────────

    #[test]
    fn ontology_boundary_roundtrips_through_json() {
        let b = bound("Idle", "Attack", 8_000_000, 3);
        let json = serde_json::to_string(&b).unwrap();
        let b2: OntologyBoundary = serde_json::from_str(&json).unwrap();
        assert_eq!(b2.source, "Idle");
        assert_eq!(b2.max_latency_ns, 8_000_000);
        assert_eq!(b2.frame_window_length, 3);
    }
}
