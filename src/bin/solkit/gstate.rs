use solkit::engine::Pos;

pub struct GameState {
    marked: Pos,
    hints: Vec<Pos>,
}

impl GameState {
    pub fn new() -> GameState {
        GameState { marked: Pos::new(), hints: Vec::new() }
    }

    pub fn mark(&mut self, p: Pos) {
        if p == self.marked {
            self.marked = Pos::new();
        } else {
            self.marked = p;
        }
    }

    pub fn clear_mark(&mut self) {
        self.marked = Pos::new();
    }

    pub fn marked(&self) -> Pos {
        self.marked
    }

    pub fn hint(&mut self, hints: &[Pos]) {
        self.hints.clear();
        for h in hints.iter() {
            self.hints.push(*h);
        }
    }

    pub fn hints(&self) -> &[Pos] {
        &self.hints
    }

    pub fn clear_hints(&mut self) {
        self.hints.clear();
    }
}
