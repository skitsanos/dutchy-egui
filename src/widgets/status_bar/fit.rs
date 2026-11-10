//! Which segments fit on one line, and the width left for an elided one.
use crate::tokens::status_bar as tokens;

/// One segment's natural width, priority and group.
#[derive(Clone, Copy, Debug)]
pub(super) struct Slot {
    pub width: f32,
    pub priority: u8,
    pub end: bool,
}

/// Visible segments, plus the one segment elided to the given width when not
/// even the highest-priority segment fits alone.
#[derive(Debug, PartialEq)]
pub(super) struct Fit {
    pub visible: Vec<bool>,
    pub elide: Option<(usize, f32)>,
}

/// Bar width the visible slots need, including insets and gaps.
pub(super) fn needed(slots: &[Slot], visible: &[bool]) -> f32 {
    let group = |end: bool| {
        let widths: Vec<f32> = slots
            .iter()
            .zip(visible)
            .filter(|(slot, shown)| **shown && slot.end == end)
            .map(|(slot, _)| slot.width)
            .collect();
        let gaps = tokens::GAP * widths.len().saturating_sub(1) as f32;
        (widths.iter().sum::<f32>() + gaps, !widths.is_empty())
    };
    let (start, has_start) = group(false);
    let (end, has_end) = group(true);
    let between = if has_start && has_end {
        tokens::GROUP_GAP
    } else {
        0.0
    };
    2.0 * tokens::INSET + start + end + between
}

/// Hide the lowest priority first, the later slot on ties, until the rest
/// fit. A last slot that still does not fit is elided instead of hidden.
pub(super) fn fit(slots: &[Slot], available: f32) -> Fit {
    let mut visible = vec![true; slots.len()];
    while needed(slots, &visible) > available {
        let shown = visible.iter().filter(|shown| **shown).count();
        if shown <= 1 {
            let elide = visible.iter().position(|shown| *shown);
            return Fit {
                visible,
                elide: elide.map(|index| (index, (available - 2.0 * tokens::INSET).max(0.0))),
            };
        }
        let victim = (0..slots.len())
            .filter(|index| visible[*index])
            .min_by_key(|index| (slots[*index].priority, std::cmp::Reverse(*index)));
        if let Some(index) = victim {
            visible[index] = false;
        }
    }
    Fit {
        visible,
        elide: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(width: f32, priority: u8, end: bool) -> Slot {
        Slot {
            width,
            priority,
            end,
        }
    }

    #[test]
    fn needed_counts_insets_gaps_and_the_group_gap_once() {
        let slots = [
            slot(10.0, 0, false),
            slot(20.0, 0, false),
            slot(5.0, 0, true),
        ];
        assert_eq!(needed(&slots, &[true; 3]), 16.0 + 35.0 + 4.0 + 16.0);
        assert_eq!(needed(&slots, &[true, true, false]), 16.0 + 30.0 + 4.0);
        assert_eq!(needed(&slots, &[false; 3]), 16.0);
    }

    #[test]
    fn lowest_priority_and_then_latest_slots_hide_first() {
        let slots = [
            slot(40.0, 2, false),
            slot(40.0, 0, false),
            slot(40.0, 0, true),
            slot(40.0, 1, true),
        ];
        let all = needed(&slots, &[true; 4]);
        assert_eq!(fit(&slots, all).visible, [true; 4]);
        assert_eq!(fit(&slots, all - 1.0).visible, [true, true, false, true]);
        assert_eq!(
            fit(&slots, needed(&slots, &[true, true, false, true]) - 1.0).visible,
            [true, false, false, true]
        );
    }

    #[test]
    fn a_lone_slot_that_cannot_fit_is_elided_to_the_inner_width() {
        let slots = [slot(400.0, 0, false), slot(10.0, 9, true)];
        assert_eq!(
            fit(&slots, 100.0),
            Fit {
                visible: vec![false, true],
                elide: None
            }
        );
        let slots = [slot(400.0, 0, false)];
        assert_eq!(fit(&slots, 100.0).elide, Some((0, 84.0)));
        assert_eq!(fit(&slots, 0.0).elide, Some((0, 0.0)));
        assert_eq!(fit(&[], 0.0).elide, None);
    }
}
