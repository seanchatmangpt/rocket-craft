pub mod receipt {
    #[derive(Debug, Clone)]
    pub struct Receipt {
        pub description: String,
    }

    impl Receipt {
        pub fn new(description: impl Into<String>) -> Self {
            Self {
                description: description.into(),
            }
        }
    }
}
