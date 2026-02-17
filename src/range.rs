
struct Range {
    pub start: f32,
    pub end: f32,
    
}

impl Range {
    pub fn new(start: f32, end: f32) -> Range {
        Range { start, end }
    }

    pub fn contains_inclusive(&self, x: f32) -> bool {
        self.start <= x && x <= self.end
    }

    pub fn contains_exclusive(&self, x: f32) -> bool {
        self.start < x && x < self.end
    }
}