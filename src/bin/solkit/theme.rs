use crossterm::style::Color;

use solkit::card::Suit;

pub(crate) trait Theme {
    fn base_colors(&self) -> (Color, Color);
    fn forbidden_card(&self) -> (Color, Color);
    fn forbidden_area(&self) -> (Color, Color);
    fn card(&self) -> (Color, Color);
    fn card_back(&self) -> (Color, Color);
    fn empty_card(&self) -> (Color, Color);
    fn hint_card(&self) -> (Color, Color);
    fn selected_card(&self) -> (Color, Color);
    fn suit(&self, s: Suit) -> Color;

    fn hint_letter(&self) -> (Color, Color);
    fn win_msg(&self) -> (Color, Color);
    fn menu_selected_item(&self) -> (Color, Color);
}

pub(crate) struct DarkTheme {
    classic: bool,
}

pub(crate) struct LightTheme {
    classic: bool,
}

impl DarkTheme {
    pub(crate) fn new(classic: bool) -> Self {
        DarkTheme { classic }
    }
}

impl LightTheme {
    pub(crate) fn new(classic: bool) -> Self {
        LightTheme { classic }
    }
}

impl Theme for DarkTheme {
    fn base_colors(&self) -> (Color, Color) {
        (Color::White, Color::Black)
    }
    fn forbidden_card(&self) -> (Color, Color) {
        (Color::DarkGrey, Color::Black)
    }
    fn forbidden_area(&self) -> (Color, Color) {
        (Color::DarkRed, Color::Black)
    }
    fn card(&self) -> (Color, Color) {
        self.base_colors()
    }
    fn card_back(&self) -> (Color, Color) {
        (Color::Blue, Color::Black)
    }
    fn empty_card(&self) -> (Color, Color) {
        (Color::DarkGrey, Color::Black)
    }
    fn hint_card(&self) -> (Color, Color) {
        (Color::Green, Color::Black)
    }
    fn selected_card(&self) -> (Color, Color) {
        (Color::White, Color::DarkGrey)
    }
    fn suit(&self, s: Suit) -> Color {
        if self.classic {
            match s {
                Suit::Spade => Color::Grey,
                Suit::Club => Color::Grey,
                Suit::Diamond => Color::DarkRed,
                Suit::Heart => Color::DarkRed,
                _ => Color::Grey,
            }
        } else {
            match s {
                Suit::Spade => Color::Grey,
                Suit::Club => Color::Cyan,
                Suit::Diamond => Color::DarkRed,
                Suit::Heart => Color::Magenta,
                _ => Color::Grey,
            }
        }
    }

    fn hint_letter(&self) -> (Color, Color) {
        self.base_colors()
    }
    fn win_msg(&self) -> (Color, Color) {
        (Color::Green, Color::Black)
    }
    fn menu_selected_item(&self) -> (Color, Color) {
        (Color::Black, Color::White)
    }
}

impl Theme for LightTheme {
    fn base_colors(&self) -> (Color, Color) {
        (Color::White, Color::DarkGreen)
    }
    fn forbidden_card(&self) -> (Color, Color) {
        (Color::DarkGrey, Color::DarkGreen)
    }
    fn forbidden_area(&self) -> (Color, Color) {
        (Color::DarkRed, Color::DarkGreen)
    }
    fn card(&self) -> (Color, Color) {
        // self.base_colors()
        (Color::Black, Color::White)
    }
    fn card_back(&self) -> (Color, Color) {
        (Color::Blue, Color::DarkGreen)
    }
    fn empty_card(&self) -> (Color, Color) {
        (Color::DarkGrey, Color::DarkGreen)
    }
    fn hint_card(&self) -> (Color, Color) {
        (Color::Green, Color::White)
    }
    fn selected_card(&self) -> (Color, Color) {
        (Color::White, Color::Grey)
    }
    fn suit(&self, s: Suit) -> Color {
        if self.classic {
            match s {
                Suit::Spade => Color::Black,
                Suit::Club => Color::Black,
                Suit::Diamond => Color::Red,
                Suit::Heart => Color::Red,
                _ => Color::DarkGrey,
            }
        } else {
            match s {
                Suit::Spade => Color::Black,
                Suit::Club => Color::Cyan,
                Suit::Diamond => Color::Red,
                Suit::Heart => Color::Magenta,
                _ => Color::DarkGrey,
            }
        }
    }

    fn hint_letter(&self) -> (Color, Color) {
        self.base_colors()
    }
    fn win_msg(&self) -> (Color, Color) {
        (Color::Cyan, Color::DarkGreen)
    }
    fn menu_selected_item(&self) -> (Color, Color) {
        (Color::DarkGreen, Color::White)
    }
}
