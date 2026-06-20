pub trait Stage<I, O> {
    fn process(&self, input: I) -> O;
    fn stage_name(&self) -> &'static str;
}

pub struct Pipeline<I, O> {
    name: String,
    stages_count: usize,
    processor: Box<dyn Fn(I) -> O>,
}

impl<I: 'static, O: 'static> Pipeline<I, O> {
    pub fn new<S: Stage<I, O> + 'static>(name: impl Into<String>, stage: S) -> Self {
        let proc = move |input| stage.process(input);
        Self { name: name.into(), stages_count: 1, processor: Box::new(proc) }
    }

    pub fn then<O2: 'static, S: Stage<O, O2> + 'static>(self, stage: S) -> Pipeline<I, O2> {
        let proc = self.processor;
        let count = self.stages_count + 1;
        let name = self.name.clone();
        Pipeline {
            name,
            stages_count: count,
            processor: Box::new(move |input| stage.process(proc(input))),
        }
    }

    pub fn run(&self, input: I) -> O { (self.processor)(input) }
    pub fn name(&self) -> &str { &self.name }
    pub fn stages_count(&self) -> usize { self.stages_count }
}

pub struct BatchPipeline<I: Clone, O> {
    pipeline: Pipeline<I, O>,
    processed: usize,
}

impl<I: Clone + 'static, O: 'static> BatchPipeline<I, O> {
    pub fn new(pipeline: Pipeline<I, O>) -> Self { Self { pipeline, processed: 0 } }
    pub fn process_batch(&mut self, inputs: Vec<I>) -> Vec<O> {
        self.processed += inputs.len();
        inputs.into_iter().map(|i| self.pipeline.run(i)).collect()
    }
    pub fn processed_count(&self) -> usize { self.processed }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Double;
    impl Stage<i32, i32> for Double {
        fn process(&self, input: i32) -> i32 { input * 2 }
        fn stage_name(&self) -> &'static str { "double" }
    }

    struct Stringify;
    impl Stage<i32, String> for Stringify {
        fn process(&self, input: i32) -> String { input.to_string() }
        fn stage_name(&self) -> &'static str { "stringify" }
    }

    #[test]
    fn single_stage_pipeline_runs_correctly() {
        let p = Pipeline::new("p", Double);
        assert_eq!(p.run(5), 10);
    }

    #[test]
    fn pipeline_name_and_stages_count() {
        let p = Pipeline::new("my-pipe", Double);
        assert_eq!(p.name(), "my-pipe");
        assert_eq!(p.stages_count(), 1);
    }

    #[test]
    fn chained_pipeline_composes_stages() {
        let p = Pipeline::new("p", Double).then(Stringify);
        assert_eq!(p.run(3), "6");
        assert_eq!(p.stages_count(), 2);
    }

    #[test]
    fn batch_pipeline_processes_all_inputs() {
        let mut bp = BatchPipeline::new(Pipeline::new("b", Double));
        let results = bp.process_batch(vec![1, 2, 3]);
        assert_eq!(results, vec![2, 4, 6]);
        assert_eq!(bp.processed_count(), 3);
    }

    #[test]
    fn batch_pipeline_accumulates_processed_count() {
        let mut bp = BatchPipeline::new(Pipeline::new("b", Double));
        bp.process_batch(vec![1, 2]);
        bp.process_batch(vec![3]);
        assert_eq!(bp.processed_count(), 3);
    }

    #[test]
    fn batch_pipeline_empty_batch_is_no_op() {
        let mut bp = BatchPipeline::new(Pipeline::new("b", Double));
        let results = bp.process_batch(vec![]);
        assert!(results.is_empty());
        assert_eq!(bp.processed_count(), 0);
    }
}
