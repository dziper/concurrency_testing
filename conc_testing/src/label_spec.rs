use regex::Regex;

/// When specifying a Label to run_to, the user can pass an object with the LabelTrait,
/// which can be used to specify a condition for when a label should be hit.
/// Labels can be composed for flexible condition specification
/// 
/// ```rust,ignore
/// run_to("thread0", StringLabel::new("Label 1")).await;
/// 
/// // Run to any label that starts with foo in thread0
/// run_to!("thread0", RegexLabel::new(Regex::new(r"foo*"))).await;
/// 
/// // Run to the fifth hit of Label 1 in thread0
/// run_to!("thread0", RepeatedLabel::new(StringLabel::new("Label 1"), 5)).await;
/// 
/// // Run to the fifth hit of either Label 1 or Label 2 in thread0
/// run_to!("thread0", RepeatedLabel::new(
///     OrLabel::new(vec![
///         StringLabel::new("Label 1"),
///         StringLabel::new("Label 2"),
///     ]), 5)).await;
/// ```
/// 
pub trait LabelTrait {
    /// When the test thread reaches a label, this object's register() function will be called with that label
    fn register(&mut self, label: &str);
    /// Retuns true if the label has been reached. False if otherwise
    fn reached(&self) -> bool;
    /// Resets any internal state (such as reached state)
    fn reset(&mut self);
}

/// Most basic Label that requires an exact label match to be triggered.
pub struct StringLabel {
    label: String,
    hit: bool
}
#[allow(dead_code)]
impl StringLabel {
    pub fn new(label: &str) -> StringLabel {
        return StringLabel {
            label: label.to_string(),
            hit: false
        }
    }
}
#[allow(dead_code)]
impl LabelTrait for StringLabel {
    fn register(&mut self, label: &str) {
        if label == self.label {
            self.hit = true;
        }
    }
    fn reached(&self) -> bool {
        self.hit
    }
    fn reset(&mut self) {
        self.hit = false;
    }
}


/// RegexLabel allows the user to specify a Regex Pattern,
/// that when matched marks the label as reached
pub struct RegexLabel {
    pattern: Regex,
    hit: bool
}
impl RegexLabel {
    pub fn new(pattern: Regex) -> RegexLabel {
        return RegexLabel {
            pattern: pattern,
            hit: false
        }
    }
}
impl LabelTrait for RegexLabel {
    fn register(&mut self, label: &str) {
        if self.pattern.is_match(label) {
            self.hit = true;
        }
    }
    fn reached(&self) -> bool {
        self.hit
    }
    fn reset(&mut self) {
        self.hit = false;
    }
}


pub struct RepeatedLabel {
    label: Box<dyn LabelTrait>,
    count: u64,
    current_count: u64
}
#[allow(dead_code)]
impl RepeatedLabel {
    pub fn new<L:LabelTrait + 'static>(label: L, count: u64) -> RepeatedLabel {
        RepeatedLabel {
            label: Box::new(label),
            count: count,
            current_count: 0
        }
    }
}
#[allow(dead_code)]
impl LabelTrait for RepeatedLabel {
    fn register(&mut self, label: &str) {
        self.label.register(label);
        if self.label.reached() {
            self.current_count += 1;
            self.label.reset();
        }
    }
    fn reached(&self) -> bool {
        self.current_count >= self.count
    }
    fn reset(&mut self) {
        self.current_count = 0;
        self.label.reset();
    }
}

pub struct OrLabel {
    labels: Vec<Box<dyn LabelTrait>>,
}
#[allow(dead_code)]
impl OrLabel {
    pub fn new<L:LabelTrait + 'static>(labels: Vec<L>) -> OrLabel {
        OrLabel {
            labels: labels.into_iter().map(|l| Box::new(l) as Box<dyn LabelTrait>).collect(),
        }
    }
}
#[allow(dead_code)]
impl LabelTrait for OrLabel {
    fn register(&mut self, label: &str) {
        for l in &mut self.labels {
            l.register(label);
        }
    }
    fn reached(&self) -> bool {
        return self.labels.iter().any(|l| l.reached());
    }
    fn reset(&mut self) {
        self.labels.iter_mut().for_each(|l| l.reset());
    }
}