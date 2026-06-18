use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_patterns::{BatchPipeline, Pipeline, Stage};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _buffer) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

struct DoubleStage;
impl Stage<i32, i32> for DoubleStage {
    fn process(&self, input: i32) -> i32 {
        input * 2
    }
    fn stage_name(&self) -> &'static str {
        "double"
    }
}

struct AddTenStage;
impl Stage<i32, i32> for AddTenStage {
    fn process(&self, input: i32) -> i32 {
        input + 10
    }
    fn stage_name(&self) -> &'static str {
        "add_ten"
    }
}

struct ToStringStage;
impl Stage<i32, String> for ToStringStage {
    fn process(&self, input: i32) -> String {
        format!("value:{}", input)
    }
    fn stage_name(&self) -> &'static str {
        "to_string"
    }
}

#[test]
fn single_stage_pipeline_processes_input() {
    let log = log();
    log.info("Given a Pipeline with a single DoubleStage");
    let p = Pipeline::new("double", DoubleStage);

    log.info("When run with input 5");
    let result = p.run(5);

    log.info("Then output is 10 and stages_count is 1");
    assert_eq!(result, 10);
    assert_eq!(p.stages_count(), 1);
}

#[test]
fn chained_stages_compose_transformations_in_order() {
    let log = log();
    log.info("Given a Pipeline composed of DoubleStage, AddTenStage, and ToStringStage");
    let p = Pipeline::new("double", DoubleStage)
        .then(AddTenStage)
        .then(ToStringStage);

    log.info("When run with input 5");
    let result = p.run(5);

    log.info("Then output is 'value:20' ((5*2)+10=20) and stages_count is 3");
    assert_eq!(result, "value:20");
    assert_eq!(p.stages_count(), 3);
}

#[test]
fn pipeline_output_varies_with_input() {
    let log = log();
    log.info("Given a Pipeline of DoubleStage then AddTenStage");
    let p = Pipeline::new("double", DoubleStage).then(AddTenStage);

    log.info("When run with two different inputs");
    let r1 = p.run(1);
    let r2 = p.run(100);

    log.info("Then the outputs are different — pipeline output depends on input");
    assert_ne!(r1, r2, "pipeline output must depend on input");
}

#[test]
fn batch_pipeline_processes_all_inputs() {
    let log = log();
    log.info("Given a BatchPipeline wrapping a DoubleStage pipeline");
    let p = Pipeline::new("double", DoubleStage);
    let mut bp = BatchPipeline::new(p);

    log.info("When a batch of 5 items is processed");
    let results = bp.process_batch(vec![1, 2, 3, 4, 5]);

    log.info("Then all outputs are doubled and processed_count is 5");
    assert_eq!(results, vec![2, 4, 6, 8, 10]);
    assert_eq!(bp.processed_count(), 5);
}

proptest! {
    #[test]
    fn double_stage_always_produces_even_output(input in 0i32..1000) {
        let log = log();
        log.info("Given a Pipeline with a DoubleStage");
        let p = Pipeline::new("double", DoubleStage);

        log.info("When run with any non-negative integer");
        let result = p.run(input);

        log.info("Then the result is always even");
        prop_assert_eq!(result % 2, 0);
    }

    #[test]
    fn pipeline_composition_matches_explicit_formula(input in -100i32..100) {
        let log = log();
        log.info("Given a Pipeline of DoubleStage then AddTenStage");
        let p = Pipeline::new("double", DoubleStage).then(AddTenStage);

        log.info("When run with any input");
        let result = p.run(input);

        log.info("Then result equals input * 2 + 10");
        let expected = input * 2 + 10;
        prop_assert_eq!(result, expected);
    }
}
