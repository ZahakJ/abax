//! Small charts drawn with the painter: bars for daily counts, lines for
//! accuracy and speed. No plotting dependency.

use egui::{pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui};

use crate::theme::{with_alpha, Palette};

pub fn bars(
    ui: &mut Ui,
    pal: &Palette,
    values: &[f32],
    labels: &[String],
    height: f32,
    unit: &str,
) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, height), Sense::hover());
    let p = ui.painter();
    let max = values.iter().cloned().fold(0.0f32, f32::max).max(1.0);
    let n = values.len().max(1) as f32;
    let pad = 6.0;
    let label_h = 16.0;
    let plot = Rect::from_min_max(
        rect.min + vec2(pad, pad + 14.0),
        rect.max - vec2(pad, pad + label_h),
    );
    let slot = plot.width() / n;
    let bar_w = (slot * 0.62).max(2.0);
    // Baseline.
    p.line_segment(
        [
            pos2(plot.left(), plot.bottom()),
            pos2(plot.right(), plot.bottom()),
        ],
        Stroke::new(1.0, with_alpha(pal.ink_soft, 0.5)),
    );
    let font = FontId::proportional(11.0);
    for (i, v) in values.iter().enumerate() {
        let x = plot.left() + slot * (i as f32 + 0.5);
        let h = (v / max) * plot.height();
        let r = Rect::from_min_max(
            pos2(x - bar_w / 2.0, plot.bottom() - h),
            pos2(x + bar_w / 2.0, plot.bottom()),
        );
        let col = if i + 1 == values.len() {
            pal.accent
        } else {
            with_alpha(pal.accent, 0.55)
        };
        if h > 0.0 {
            p.rect(r, 3.0, col, Stroke::NONE, StrokeKind::Inside);
            p.text(
                pos2(x, r.top() - 2.0),
                Align2::CENTER_BOTTOM,
                format!("{}", v.round() as i64),
                font.clone(),
                pal.ink_soft,
            );
        }
        if let Some(l) = labels.get(i) {
            if values.len() <= 14 || i % 2 == 0 {
                p.text(
                    pos2(x, plot.bottom() + 3.0),
                    Align2::CENTER_TOP,
                    l,
                    font.clone(),
                    pal.ink_soft,
                );
            }
        }
    }
    if !unit.is_empty() {
        p.text(
            pos2(rect.left() + pad, rect.top() + 2.0),
            Align2::LEFT_TOP,
            unit,
            font,
            pal.ink_soft,
        );
    }
}

/// A line chart; `range` fixes the y axis (e.g. 0..1 for accuracy) or is
/// None for auto.
pub fn line(
    ui: &mut Ui,
    pal: &Palette,
    values: &[f32],
    height: f32,
    range: Option<(f32, f32)>,
    fmt: &dyn Fn(f32) -> String,
    color: Color32,
) {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(w, height), Sense::hover());
    let p = ui.painter();
    let pad = 8.0;
    let plot = Rect::from_min_max(
        rect.min + vec2(pad + 34.0, pad),
        rect.max - vec2(pad, pad + 4.0),
    );
    let (lo, hi) = range.unwrap_or_else(|| {
        let mn = values.iter().cloned().fold(f32::INFINITY, f32::min);
        let mx = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        if mn.is_finite() && mx.is_finite() && mx > mn {
            (0.0f32.min(mn), mx * 1.1)
        } else {
            (0.0, 1.0)
        }
    });
    let font = FontId::proportional(11.0);
    // Grid: 3 lines.
    for k in 0..=2 {
        let y = plot.bottom() - plot.height() * k as f32 / 2.0;
        p.line_segment(
            [pos2(plot.left(), y), pos2(plot.right(), y)],
            Stroke::new(
                1.0,
                with_alpha(pal.ink_soft, if k == 0 { 0.5 } else { 0.18 }),
            ),
        );
        let v = lo + (hi - lo) * k as f32 / 2.0;
        p.text(
            pos2(plot.left() - 4.0, y),
            Align2::RIGHT_CENTER,
            fmt(v),
            font.clone(),
            pal.ink_soft,
        );
    }
    if values.len() < 2 {
        p.text(
            plot.center(),
            Align2::CENTER_CENTER,
            "Not enough data yet",
            font,
            pal.ink_soft,
        );
        return;
    }
    let n = (values.len() - 1) as f32;
    let pts: Vec<egui::Pos2> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = plot.left() + plot.width() * i as f32 / n;
            let t = ((v - lo) / (hi - lo).max(1e-6)).clamp(0.0, 1.0);
            pos2(x, plot.bottom() - plot.height() * t)
        })
        .collect();
    // Soft area under the line.
    let mut area = pts.clone();
    area.push(pos2(plot.right(), plot.bottom()));
    area.push(pos2(plot.left(), plot.bottom()));
    p.add(egui::Shape::convex_polygon(
        vec![],
        Color32::TRANSPARENT,
        Stroke::NONE,
    ));
    for w in pts.windows(2) {
        let quad = vec![
            w[0],
            w[1],
            pos2(w[1].x, plot.bottom()),
            pos2(w[0].x, plot.bottom()),
        ];
        p.add(egui::Shape::convex_polygon(
            quad,
            with_alpha(color, 0.10),
            Stroke::NONE,
        ));
    }
    p.add(egui::Shape::line(pts.clone(), Stroke::new(2.0, color)));
    for pt in &pts {
        p.circle_filled(*pt, 2.5, color);
    }
    // Hover readout.
    if let Some(hp) = resp.hover_pos() {
        let i = (((hp.x - plot.left()) / plot.width()) * n)
            .round()
            .clamp(0.0, n) as usize;
        let pt = pts[i];
        p.circle_stroke(pt, 5.0, Stroke::new(1.5, pal.ink));
        p.text(
            pos2(pt.x, pt.y - 8.0),
            Align2::CENTER_BOTTOM,
            fmt(values[i]),
            FontId::proportional(12.0),
            pal.ink,
        );
    }
}

/// A horizontal mastery bar with label.
pub fn mastery_bar(
    ui: &mut Ui,
    pal: &Palette,
    label: &str,
    value: f32,
    mastered: bool,
    locked: bool,
) {
    let h = 22.0;
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    let p = ui.painter();
    let label_w = (w * 0.42).min(270.0);
    let col = if locked {
        with_alpha(pal.ink_soft, 0.6)
    } else {
        pal.ink
    };
    p.text(
        pos2(rect.left(), rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(14.0),
        col,
    );
    let bar = Rect::from_min_max(
        pos2(rect.left() + label_w, rect.top() + 5.0),
        pos2(rect.right() - 36.0, rect.bottom() - 5.0),
    );
    p.rect(
        bar,
        4.0,
        with_alpha(pal.ink_soft, 0.18),
        Stroke::NONE,
        StrokeKind::Inside,
    );
    if !locked {
        let fill = Rect::from_min_max(
            bar.min,
            pos2(
                bar.left() + bar.width() * value.clamp(0.0, 1.0),
                bar.bottom(),
            ),
        );
        let c = if mastered { pal.ok } else { pal.accent };
        p.rect(fill, 4.0, c, Stroke::NONE, StrokeKind::Inside);
    }
    let txt = if locked {
        "locked".to_string()
    } else if mastered {
        "done".to_string()
    } else {
        format!("{}%", (value * 100.0).round() as i32)
    };
    p.text(
        pos2(rect.right(), rect.center().y),
        Align2::RIGHT_CENTER,
        txt,
        FontId::proportional(12.0),
        pal.ink_soft,
    );
}
