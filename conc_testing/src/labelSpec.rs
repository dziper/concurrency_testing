use regex::Regex;

pub trait LabelTrait {
    fn register(&mut self, label: &str);
    fn reached(&self) -> bool;
    fn reset(&mut self);
}

pub struct StringLabel {
    label: String,
    hit: bool
}
impl StringLabel {
    pub fn new(label: &str) -> StringLabel {
        return StringLabel {
            label: label.to_string(),
            hit: false
        }
    }
}
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
impl RepeatedLabel {
    pub fn new<L:LabelTrait + 'static>(label: L, count: u64) -> RepeatedLabel {
        RepeatedLabel {
            label: Box::new(label),
            count: count,
            current_count: 0
        }
    }
}
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
impl OrLabel {
    pub fn new<L:LabelTrait + 'static>(labels: Vec<L>) -> OrLabel {
        OrLabel {
            labels: labels.into_iter().map(|l| Box::new(l) as Box<dyn LabelTrait>).collect(),
        }
    }
}
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