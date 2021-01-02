use anyhow::{anyhow, Result};
use crossterm::style::Color;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cell {
    pub fg: Color,
    pub bg: Color,
    pub ch: char,
}

impl Default for Cell {
    fn default() -> Cell {
        Cell { fg: Color::White, bg: Color::Black, ch: '\t' }
    }
}

// for optimized output, each screen flush detects changes cells and prints only them.
// BufIterator goes though all screen cells and yield all the changed ones.
pub struct CellDiff {
    pub cell: Cell,
    pub col: u16,
    pub row: u16,
}

pub struct BufferIterator<'a> {
    buffer: &'a Buffer,
    index: usize,
}

impl<'a> IntoIterator for &'a Buffer {
    type Item = CellDiff;
    type IntoIter = BufferIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BufferIterator { buffer: self, index: 0 }
    }
}

impl<'a> Iterator for BufferIterator<'a> {
    type Item = CellDiff;
    fn next(&mut self) -> Option<CellDiff> {
        let l = self.buffer.back.len();
        while self.index < l {
            if self.buffer.back[self.index] != self.buffer.display[self.index] {
                let row = self.index / usize::from(self.buffer.w);
                let col = self.index - row * usize::from(self.buffer.w);
                let (col, row) = (col as u16, row as u16);
                let cd = CellDiff { cell: self.buffer.back[self.index], col, row };
                let c = Some(cd);
                self.index += 1;
                return c;
            }
            self.index += 1;
        }
        None
    }
}

pub struct Buffer {
    display: Vec<Cell>, // what is shown in the screen
    back: Vec<Cell>,    // back buffer - current screen data
    what: Vec<u16>,     // what is in the every screen cell (for mouse support)
    pub(crate) w: u16,  // screen width
    pub(crate) h: u16,  // screen height
}

impl Buffer {
    pub fn new(w: u16, h: u16) -> Result<Self> {
        if !(60..1500).contains(&w) {
            return Err(anyhow!("Width must be between 60 and 1500"));
        }
        if !(20..500).contains(&h) {
            return Err(anyhow!("Height must be between 20 and 500"));
        }
        let sz = usize::from(w) * usize::from(h);
        Ok(Buffer { w, h, what: vec![0; sz], display: vec![Cell::default(); sz], back: vec![Cell::default(); sz] })
    }

    // copy data from the back buffer to the screen buffer
    pub fn flip(&mut self) {
        for (idx, val) in self.back.iter().enumerate() {
            self.display[idx] = *val;
        }
    }

    // fill the back buffer with spaces and given color attributes
    pub fn clear(&mut self, fg: Color, bg: Color) {
        let sz = usize::from(self.w) * usize::from(self.h);
        let cell = Cell { ch: ' ', fg, bg };
        self.back = vec![cell; sz];
        self.what = vec![0; sz];
    }

    // resize screen and back buffers. Old data is preserved if possible (when the area grows)
    pub fn resize(&mut self, new_w: u16, new_h: u16) -> Result<()> {
        if !(60..1500).contains(&new_w) {
            return Err(anyhow!("Width must be between 60 and 1500"));
        }
        if !(20..500).contains(&new_h) {
            return Err(anyhow!("Height must be between 20 and 500"));
        }
        let def_cell = Cell::default();
        let new_sz = usize::from(new_w) * usize::from(new_h);
        let nd: Vec<Cell> = vec![def_cell; new_sz];
        let mut nb: Vec<Cell> = vec![def_cell; new_sz];
        let mut nw: Vec<u16> = vec![0; new_sz];
        for y in 0..new_h {
            if y >= self.h {
                break;
            }
            let dy = usize::from(y) * usize::from(new_w);
            let dy_old = usize::from(y) * usize::from(self.w);
            for x in 0..new_w {
                if x >= self.w {
                    break;
                }
                let x = usize::from(x);
                nb[dy + x] = self.back[dy_old + x];
                nw[dy + x] = self.what[dy_old + x];
            }
        }
        self.display = nd;
        self.back = nb;
        self.what = nw;
        self.w = new_w;
        self.h = new_h;
        Ok(())
    }

    pub fn write_char(&mut self, ch: char, col: u16, row: u16, fg: Color, bg: Color, kind: u16) {
        if row >= self.h || col >= self.w {
            return;
        }
        let idx = usize::from(row) * usize::from(self.w) + usize::from(col);
        self.back[idx] = Cell { fg, bg, ch };
        self.what[idx] = kind;
    }

    pub fn what_at(&self, col: u16, row: u16) -> u16 {
        if col >= self.w || row >= self.h {
            return 0;
        }
        let idx = usize::from(row) * usize::from(self.w) + usize::from(col);
        self.what[idx]
    }

    pub fn cell(&self, col: u16, row: u16) -> Option<Cell> {
        if col >= self.w || row >= self.h {
            return None;
        }
        let idx = (self.w * row + col) as usize;
        Some(self.display[idx])
    }
}

#[cfg(test)]
mod buf_test {
    use super::*;
    use crossterm::style::Color;

    #[test]
    fn h_resize_test() {
        let mut scr = Buffer::new(60, 30).unwrap();
        scr.clear(Color::White, Color::Black);
        struct Dt {
            c: char,
            x: u16,
            y: u16,
        }
        let data: Vec<Dt> = vec![
            Dt { c: 'a', x: 0, y: 0 },
            Dt { c: 'b', x: 1, y: 0 },
            Dt { c: 'c', x: 2, y: 0 },
            Dt { c: 'd', x: 1, y: 1 },
            Dt { c: 'e', x: 2, y: 1 },
            Dt { c: 'f', x: 3, y: 1 },
            Dt { c: 'g', x: 3, y: 4 },
            Dt { c: 'h', x: 4, y: 4 },
            Dt { c: 'i', x: 5, y: 4 },
        ];
        for d in data.iter() {
            scr.write_char(d.c, d.x, d.y, Color::White, Color::Black, 0);
        }
        scr.flip();
        let res = scr.resize(70, 30);
        scr.flip();
        assert!(res.is_ok());
        for d in data.iter() {
            let c = scr.cell(d.x, d.y);
            assert!(c.is_some());
            let c = c.unwrap();
            assert_eq!(d.c, c.ch);
        }
    }

    #[test]
    fn v_resize_test() {
        let mut scr = Buffer::new(60, 30).unwrap();
        scr.clear(Color::White, Color::Black);
        struct Dt {
            c: char,
            x: u16,
            y: u16,
        }
        let data: Vec<Dt> = vec![
            Dt { c: 'a', x: 0, y: 0 },
            Dt { c: 'b', x: 1, y: 0 },
            Dt { c: 'c', x: 2, y: 0 },
            Dt { c: 'd', x: 1, y: 1 },
            Dt { c: 'e', x: 2, y: 1 },
            Dt { c: 'f', x: 3, y: 1 },
            Dt { c: 'g', x: 3, y: 4 },
            Dt { c: 'h', x: 4, y: 4 },
            Dt { c: 'i', x: 5, y: 4 },
        ];
        for d in data.iter() {
            scr.write_char(d.c, d.x, d.y, Color::White, Color::Black, 0);
        }
        scr.flip();
        let res = scr.resize(60, 35);
        scr.flip();
        assert!(res.is_ok());
        for d in data.iter() {
            let c = scr.cell(d.x, d.y);
            assert!(c.is_some());
            let c = c.unwrap();
            assert_eq!(d.c, c.ch);
        }
    }

    #[test]
    fn both_resize_test() {
        let mut scr = Buffer::new(60, 30).unwrap();
        scr.clear(Color::White, Color::Black);
        struct Dt {
            c: char,
            x: u16,
            y: u16,
        }
        let data: Vec<Dt> = vec![
            Dt { c: 'a', x: 0, y: 0 },
            Dt { c: 'b', x: 1, y: 0 },
            Dt { c: 'c', x: 2, y: 0 },
            Dt { c: 'd', x: 1, y: 1 },
            Dt { c: 'e', x: 2, y: 1 },
            Dt { c: 'f', x: 3, y: 1 },
            Dt { c: 'g', x: 3, y: 4 },
            Dt { c: 'h', x: 4, y: 4 },
            Dt { c: 'i', x: 5, y: 4 },
        ];
        for d in data.iter() {
            scr.write_char(d.c, d.x, d.y, Color::White, Color::Black, 0);
        }
        scr.flip();
        let res = scr.resize(70, 35);
        scr.flip();
        assert!(res.is_ok());
        for d in data.iter() {
            let c = scr.cell(d.x, d.y);
            assert!(c.is_some());
            let c = c.unwrap();
            assert_eq!(d.c, c.ch);
        }
    }
}
