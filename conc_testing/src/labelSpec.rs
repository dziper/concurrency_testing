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
    fn new(label: &str) -> StringLabel {
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
    fn new(pattern: Regex) -> RegexLabel {
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


pub struct RepeatedLabel<'a> {
    label: &'a mut dyn LabelTrait,
    count: u64,
    current_count: u64
}
impl<'a> RepeatedLabel<'a> {
    fn new(label: &mut impl LabelTrait, count: u64) -> RepeatedLabel<'_> {
        RepeatedLabel {
            label: label,
            count: count,
            current_count: 0
        }
    }
}
impl<'a> LabelTrait for RepeatedLabel<'a> {
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