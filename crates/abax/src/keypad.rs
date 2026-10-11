//! On-screen numeric keypad and keyboard mapping.

use egui::{Key, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Digit(u8),
    Backspace,
    Clear,
    Enter,
}

/// Read digit and control keys from the keyboard this frame.
pub fn keyboard_actions(ui: &Ui) -> Vec<KeyAction> {
    let mut out = Vec::new();
    ui.input(|i| {
        for ev in &i.events {
            if let egui::Event::Key {
                key, pressed: true, ..
            } = ev
            {
                let a = match key {
                    Key::Num0 => Some(KeyAction::Digit(0)),
                    Key::Num1 => Some(KeyAction::Digit(1)),
                    Key::Num2 => Some(KeyAction::Digit(2)),
                    Key::Num3 => Some(KeyAction::Digit(3)),
                    Key::Num4 => Some(KeyAction::Digit(4)),
                    Key::Num5 => Some(KeyAction::Digit(5)),
                    Key::Num6 => Some(KeyAction::Digit(6)),
                    Key::Num7 => Some(KeyAction::Digit(7)),
                    Key::Num8 => Some(KeyAction::Digit(8)),
                    Key::Num9 => Some(KeyAction::Digit(9)),
                    Key::Backspace => Some(KeyAction::Backspace),
                    Key::Delete | Key::Escape => Some(KeyAction::Clear),
                    Key::Enter | Key::Space => Some(KeyAction::Enter),
                    _ => None,
                };
                if let Some(a) = a {
                    out.push(a);
                }
            }
        }
    });
    out
}

/// Width the keypad takes for a given key size.
pub fn keypad_width(key_size: Vec2) -> f32 {
    3.0 * key_size.x + 2.0 * 6.0
}

/// Draw the keypad centred in the available width.
pub fn keypad_centered(ui: &mut Ui, left_handed: bool, key_size: Vec2) -> Option<KeyAction> {
    let pad = ((ui.available_width() - keypad_width(key_size)) / 2.0).max(0.0);
    ui.horizontal(|ui| {
        ui.add_space(pad);
        keypad(ui, left_handed, key_size)
    })
    .inner
}

/// Draw the keypad; returns the action pressed this frame, if any.
pub fn keypad(ui: &mut Ui, left_handed: bool, key_size: Vec2) -> Option<KeyAction> {
    let mut action = None;
    let rows: [[&str; 3]; 4] = [
        ["7", "8", "9"],
        ["4", "5", "6"],
        ["1", "2", "3"],
        ["Del", "0", "Enter"],
    ];
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
        for row in rows {
            ui.horizontal(|ui| {
                let mut cells: Vec<&str> = row.to_vec();
                if left_handed {
                    cells.reverse();
                }
                for label in cells {
                    let is_enter = label == "Enter";
                    let size = if label.len() > 1 {
                        key_size.y * 0.3
                    } else {
                        key_size.y * 0.42
                    };
                    let btn =
                        egui::Button::new(egui::RichText::new(label).size(size)).min_size(key_size);
                    let btn = if is_enter {
                        btn.fill(ui.visuals().selection.bg_fill)
                    } else {
                        btn
                    };
                    if ui.add(btn).clicked() {
                        action = Some(match label {
                            "Del" => KeyAction::Backspace,
                            "Enter" => KeyAction::Enter,
                            d => KeyAction::Digit(d.parse().unwrap_or(0)),
                        });
                    }
                }
            });
        }
    });
    action
}

/// A digit buffer the keypad edits.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DigitBuffer {
    pub text: String,
}

impl DigitBuffer {
    pub fn apply(&mut self, a: KeyAction) {
        match a {
            KeyAction::Digit(d) => {
                if self.text.len() < 30 && !(self.text.is_empty() && d == 0 && false) {
                    self.text.push((b'0' + d) as char);
                }
            }
            KeyAction::Backspace => {
                self.text.pop();
            }
            KeyAction::Clear => self.text.clear(),
            KeyAction::Enter => {}
        }
    }

    pub fn value(&self) -> Option<u128> {
        if self.text.is_empty() {
            None
        } else {
            self.text.parse().ok()
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_edits() {
        let mut b = DigitBuffer::default();
        assert_eq!(b.value(), None);
        b.apply(KeyAction::Digit(4));
        b.apply(KeyAction::Digit(0));
        b.apply(KeyAction::Digit(2));
        assert_eq!(b.value(), Some(402));
        b.apply(KeyAction::Backspace);
        assert_eq!(b.value(), Some(40));
        b.apply(KeyAction::Clear);
        assert!(b.is_empty());
    }
}
