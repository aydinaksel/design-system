pub(crate) fn spread_label_positions(desired_percents: &[f64], spacing_percent: f64) -> Vec<f64> {
    let mut placed: Vec<f64> = Vec::with_capacity(desired_percents.len());
    for desired in desired_percents {
        let lowest_free = placed
            .last()
            .map_or(0.0, |previous| previous + spacing_percent);
        placed.push(desired.max(lowest_free));
    }
    let mut highest_free = 100.0;
    for position in placed.iter_mut().rev() {
        *position = position.min(highest_free);
        highest_free = *position - spacing_percent;
    }
    placed
}

#[cfg(test)]
mod tests {
    use super::spread_label_positions;

    const SPACING_PERCENT: f64 = 8.0;

    #[test]
    fn labels_far_apart_stay_beside_their_marks() {
        assert_eq!(
            spread_label_positions(&[10.0, 60.0], SPACING_PERCENT),
            vec![10.0, 60.0]
        );
    }

    #[test]
    fn labels_that_would_overlap_are_pushed_apart() {
        assert_eq!(
            spread_label_positions(&[40.0, 42.0], SPACING_PERCENT),
            vec![40.0, 48.0]
        );
    }

    #[test]
    fn labels_crowded_at_the_baseline_move_up() {
        assert_eq!(
            spread_label_positions(&[99.0, 100.0], SPACING_PERCENT),
            vec![92.0, 100.0]
        );
    }
}
