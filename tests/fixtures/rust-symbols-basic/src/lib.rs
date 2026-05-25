pub fn top_level_function() {}

pub struct Widget {
    value: i32,
}

pub enum Mode {
    Fast,
    Slow,
}

pub trait Runner {
    fn run(&self);
}

pub type WidgetId = u64;

pub const DEFAULT_LIMIT: usize = 10;

pub static GLOBAL_LIMIT: usize = DEFAULT_LIMIT;

pub mod nested {
    pub fn child() {}
}

impl Widget {
    pub fn new(value: i32) -> Self {
        fn nested_helper(value: i32) -> i32 {
            value
        }

        Self {
            value: nested_helper(value),
        }
    }
}
