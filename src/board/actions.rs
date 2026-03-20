/// Outcome of a cell click or drag action.
#[derive(Debug, Clone)]
pub enum ClickAction {
    None,
    Selected(usize),
    Deselected,
    Merged {
        #[allow(dead_code)]
        source: usize,
        #[allow(dead_code)]
        target: usize,
        result: String,
    },
    Moved {
        #[allow(dead_code)]
        from: usize,
        #[allow(dead_code)]
        to: usize,
        item: String,
    },
    Swapped {
        #[allow(dead_code)]
        from: usize,
        #[allow(dead_code)]
        to: usize,
    },
    GeneratorActivated(usize, String),
}
