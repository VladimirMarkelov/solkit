use std::io::Write;

use anyhow::Result;
use crossterm::{cursor, queue, style, style::Color};

use crate::buffer::Buffer;

// TODO: long term ones
//   - support double-width?

const SINGLE_FRM: [char; 6] = ['│', '─', '┌', '┐', '└', '┘'];
const DOUBLE_FRM: [char; 6] = ['║', '═', '╔', '╗', '╚', '╝'];
const VLINE: usize = 0;
const HLINE: usize = 1;
const TL_CORNER: usize = 2;
const TR_CORNER: usize = 3;
const DL_CORNER: usize = 4;
const DR_CORNER: usize = 5;

pub(crate) struct Screen {
    buf: Buffer,
    fg: Color,
    bg: Color,
    kd: u16,
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum Border {
    None,
    Single,
    Double,
}

impl Screen {
    pub(crate) fn new(w: u16, h: u16) -> Result<Self> {
        let buf = Buffer::new(w, h)?;
        Ok(Screen { buf, fg: Color::White, bg: Color::Black, kd: 0 })
    }

    pub(crate) fn kind(&mut self, k: u16) {
        self.kd = k;
    }

    pub(crate) fn colors(&mut self, fg: Color, bg: Color) {
        self.fg = fg;
        self.bg = bg;
    }

    pub(crate) fn width(&self) -> u16 {
        self.buf.w
    }

    pub(crate) fn height(&self) -> u16 {
        self.buf.h
    }

    pub(crate) fn write_string(&mut self, s: &str, col: u16, row: u16) {
        if s.is_empty() {
            return;
        }
        if row >= self.buf.h || col >= self.buf.w {
            return;
        }
        let mut dx = col;
        for ch in s.chars() {
            self.buf.write_char(ch, dx, row, self.fg, self.bg, self.kd);
            dx += 1;
            if dx >= self.buf.w {
                break;
            }
        }
    }

    pub(crate) fn write_string_highlight(&mut self, s: &str, col: u16, row: u16, ext_color: Color) {
        if s.is_empty() {
            return;
        }
        if row >= self.buf.h || col >= self.buf.w {
            return;
        }
        let mut dx = col;
        let save_color = self.fg;
        for ch in s.chars() {
            if ch == '`' {
                self.fg = if save_color == self.fg { ext_color } else { save_color };
                continue;
            }
            self.buf.write_char(ch, dx, row, self.fg, self.bg, self.kd);
            dx += 1;
            if dx >= self.buf.w {
                break;
            }
        }
        self.fg = save_color;
    }

    /*
    pub(crate) fn write_string_vertical(&mut self, s: &str, col: u16, row: u16) {
        if row >= self.buf.h || col >= self.buf.w {
            return;
        }
        let mut dy = row;
        for ch in s.chars() {
            if dy >= self.buf.h {
                break;
            }
            self.buf.write_char(ch, col, dy, self.fg, self.bg, self.kd);
            dy+=1;
        }
    }
    */

    pub(crate) fn write_char(&mut self, ch: char, col: u16, row: u16) {
        self.buf.write_char(ch, col, row, self.fg, self.bg, self.kd);
    }

    pub(crate) fn write_hline(&mut self, x: u16, y: u16, w: u16, border: Border) {
        if y >= self.buf.h || x >= self.buf.w {
            return;
        }
        let ch = match border {
            Border::None => ' ',
            Border::Single => SINGLE_FRM[HLINE],
            Border::Double => DOUBLE_FRM[HLINE],
        };
        for dx in x..x + w {
            if dx >= self.buf.w {
                break;
            }
            self.buf.write_char(ch, dx, y, self.fg, self.bg, self.kd);
        }
    }

    pub(crate) fn write_vline(&mut self, col: u16, row: u16, h: u16, border: Border) {
        if row >= self.buf.h || col >= self.buf.w {
            return;
        }
        let ch = match border {
            Border::None => ' ',
            Border::Single => SINGLE_FRM[VLINE],
            Border::Double => DOUBLE_FRM[VLINE],
        };
        for dy in row..row + h {
            if dy >= self.buf.h {
                break;
            }
            self.buf.write_char(ch, col, dy, self.fg, self.bg, self.kd);
        }
    }

    pub(crate) fn fill_rect(&mut self, col: u16, row: u16, w: u16, h: u16, ch: char) {
        for dy in row..row + h {
            if dy >= self.buf.h {
                break;
            }
            for dx in col..col + w {
                if dx >= self.buf.w {
                    break;
                }
                self.buf.write_char(ch, dx, dy, self.fg, self.bg, self.kd);
            }
        }
    }

    pub(crate) fn draw_frame(&mut self, col: u16, row: u16, w: u16, h: u16, border: Border) {
        if border == Border::None {
            self.write_hline(col, row, w, border);
            self.write_hline(col, row + h - 1, w, border);
            self.write_vline(col, row + 1, h - 2, border);
            self.write_vline(col + w - 1, row + 1, h - 2, border);
        } else {
            let frm = if border == Border::Single { &SINGLE_FRM } else { &DOUBLE_FRM };
            self.buf.write_char(frm[TL_CORNER], col, row, self.fg, self.bg, self.kd);
            self.buf.write_char(frm[TR_CORNER], col + w - 1, row, self.fg, self.bg, self.kd);
            self.buf.write_char(frm[DL_CORNER], col, row + h - 1, self.fg, self.bg, self.kd);
            self.buf.write_char(frm[DR_CORNER], col + w - 1, row + h - 1, self.fg, self.bg, self.kd);
            self.write_hline(col + 1, row, w - 2, border);
            self.write_hline(col + 1, row + h - 1, w - 2, border);
            self.write_vline(col, row + 1, h - 2, border);
            self.write_vline(col + w - 1, row + 1, h - 2, border);
        }
    }

    pub(crate) fn clear(&mut self) {
        self.buf.clear(self.fg, self.bg);
    }

    pub(crate) fn resize(&mut self, new_w: u16, new_h: u16) -> Result<()> {
        self.buf.resize(new_w, new_h)
    }

    pub(crate) fn flush<W>(&mut self, w: &mut W) -> Result<()>
    where
        W: Write,
    {
        let mut cnt = 0;
        let mut fg = Color::Black;
        let mut bg = Color::Black;
        let mut text = String::new();
        let mut col = 0u16;
        let mut row = 0u16;
        let mut len = 0u16;

        for change in self.buf.into_iter() {
            if change.col == self.buf.w - 1 && change.row == self.buf.h - 1 {
                continue;
            }
            cnt += 1;
            if text.is_empty() {
                text.push(change.cell.ch);
                col = change.col;
                row = change.row;
                fg = change.cell.fg;
                bg = change.cell.bg;
                len = 1;
                continue;
            }
            if fg == change.cell.fg && bg == change.cell.bg && change.row == row && change.col == len + col {
                text.push(change.cell.ch);
                len += 1;
                continue;
            }
            queue!(
                w,
                cursor::MoveTo(col, row),
                style::SetForegroundColor(fg),
                style::SetBackgroundColor(bg),
                style::Print(&text),
            )?;
            text.clear();
            text.push(change.cell.ch);
            col = change.col;
            row = change.row;
            fg = change.cell.fg;
            bg = change.cell.bg;
            len = 1;
        }
        if !text.is_empty() {
            queue!(
                w,
                cursor::MoveTo(col, row),
                style::SetForegroundColor(fg),
                style::SetBackgroundColor(bg),
                style::Print(text),
            )?;
        }
        if cnt != 0 {
            self.buf.flip();
        }
        Ok(())
    }

    pub(crate) fn what_at(&self, col: u16, row: u16) -> u16 {
        self.buf.what_at(col, row)
    }
}

#[cfg(test)]
mod buf_test {
    use super::*;

    #[test]
    fn iter_test() {
        let mut scr = Screen::new(80, 25).unwrap();
        scr.write_string("one", 10, 20);
        scr.write_string("two", 15, 10);
        scr.write_string("big", 78, 24);
        // first run: "two" must go first, "big" must be truncated
        let mut s = String::new();
        for cd in scr.buf.into_iter() {
            s.push(cd.cell.ch);
        }
        assert_eq!(s, "twoonebi".to_string());
        // second run must equal first one
        let mut s2 = String::new();
        for cd in scr.buf.into_iter() {
            s2.push(cd.cell.ch);
        }
        assert_eq!(s2, "twoonebi".to_string());
        // no changes after flip
        scr.buf.flip();
        let mut s3 = String::new();
        for cd in scr.buf.into_iter() {
            s3.push(cd.cell.ch);
        }
        assert_eq!(s3, String::new());
        scr.write_char('z', 16, 18);
        let mut cnt = 0;
        for cd in scr.buf.into_iter() {
            assert_eq!(cd.cell.ch, 'z');
            assert_eq!(cd.col, 16);
            assert_eq!(cd.row, 18);
            cnt += 1;
        }
        assert_eq!(cnt, 1);
    }

    #[test]
    fn what_test() {
        let mut scr = Screen::new(80, 25).unwrap();
        scr.kind(1);
        scr.write_string("one", 10, 20);
        scr.kind(2);
        scr.write_string("two", 15, 10);
        scr.kind(3);
        scr.write_string("big", 78, 24);

        struct Kw {
            x: u16,
            y: u16,
            w: u16,
        }
        let whats: Vec<Kw> = vec![
            Kw { x: 5, y: 5, w: 0 },
            Kw { x: 500, y: 500, w: 0 },
            Kw { x: 15, y: 10, w: 2 },
            Kw { x: 16, y: 10, w: 2 },
            Kw { x: 11, y: 20, w: 1 },
            Kw { x: 12, y: 20, w: 1 },
            Kw { x: 78, y: 24, w: 3 },
            Kw { x: 79, y: 24, w: 3 },
            Kw { x: 80, y: 24, w: 0 },
        ];
        for kw in whats.iter() {
            let what = scr.what_at(kw.x, kw.y);
            assert_eq!(what, kw.w);
        }
    }
}
