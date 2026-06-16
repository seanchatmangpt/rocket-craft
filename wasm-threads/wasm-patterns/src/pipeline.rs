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
