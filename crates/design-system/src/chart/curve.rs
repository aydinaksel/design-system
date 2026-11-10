#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlotPoint {
    pub horizontal: f64,
    pub vertical: f64,
}

fn secant(from: PlotPoint, to: PlotPoint) -> f64 {
    let run = to.horizontal - from.horizontal;
    if run.abs() < f64::EPSILON {
        0.0
    } else {
        (to.vertical - from.vertical) / run
    }
}

fn direction(slope: f64) -> f64 {
    if slope > 0.0 {
        1.0
    } else if slope < 0.0 {
        -1.0
    } else {
        0.0
    }
}

// Monotone cubic interpolation after Steffen, "A Simple Method for Monotonic
// Interpolation in One Dimension" (1990), as d3-shape's curveMonotoneX does:
// the curve never overshoots a data point, so a smoothed peak is a real peak.
fn interior_tangent(previous: PlotPoint, current: PlotPoint, next: PlotPoint) -> f64 {
    let left_run = current.horizontal - previous.horizontal;
    let right_run = next.horizontal - current.horizontal;
    let left_slope = secant(previous, current);
    let right_slope = secant(current, next);
    let weighted = (left_slope * right_run + right_slope * left_run) / (left_run + right_run);
    let limit = left_slope
        .abs()
        .min(right_slope.abs())
        .min(0.5 * weighted.abs());
    (direction(left_slope) + direction(right_slope)) * limit
}

fn end_tangent(secant_slope: f64, neighbour_tangent: f64) -> f64 {
    (3.0 * secant_slope - neighbour_tangent) / 2.0
}

fn tangents(points: &[PlotPoint]) -> Vec<f64> {
    let interior: Vec<f64> = points
        .windows(3)
        .filter_map(|window| match window {
            [previous, current, next] => Some(interior_tangent(*previous, *current, *next)),
            _ => None,
        })
        .collect();
    let (Some(first_interior), Some(last_interior)) = (interior.first(), interior.last()) else {
        return Vec::new();
    };
    let (Some(first), Some(second)) = (points.first(), points.get(1)) else {
        return Vec::new();
    };
    let (Some(last), Some(before_last)) = (points.last(), points.iter().rev().nth(1)) else {
        return Vec::new();
    };
    let mut all = Vec::with_capacity(points.len());
    all.push(end_tangent(secant(*first, *second), *first_interior));
    all.extend(interior.iter().copied());
    all.push(end_tangent(secant(*before_last, *last), *last_interior));
    all
}

fn cubic_segment(start: PlotPoint, end: PlotPoint, start_slope: f64, end_slope: f64) -> String {
    let third = (end.horizontal - start.horizontal) / 3.0;
    format!(
        "C{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}",
        start.horizontal + third,
        start.vertical + third * start_slope,
        end.horizontal - third,
        end.vertical - third * end_slope,
        end.horizontal,
        end.vertical,
    )
}

pub(crate) fn monotone_path(points: &[PlotPoint]) -> String {
    let Some(first) = points.first() else {
        return String::new();
    };
    let start = format!("M{:.2},{:.2}", first.horizontal, first.vertical);
    let slopes = tangents(points);
    let segments: Vec<String> = if slopes.is_empty() {
        points
            .iter()
            .skip(1)
            .map(|point| format!("L{:.2},{:.2}", point.horizontal, point.vertical))
            .collect()
    } else {
        points
            .windows(2)
            .zip(slopes.windows(2))
            .filter_map(|pair| match pair {
                ([start, end], [start_slope, end_slope]) => {
                    Some(cubic_segment(*start, *end, *start_slope, *end_slope))
                }
                _ => None,
            })
            .collect()
    };
    start + &segments.concat()
}

#[cfg(test)]
mod tests {
    use super::{monotone_path, PlotPoint};

    fn point(horizontal: f64, vertical: f64) -> PlotPoint {
        PlotPoint {
            horizontal,
            vertical,
        }
    }

    #[test]
    fn two_points_draw_a_straight_line() {
        assert_eq!(
            monotone_path(&[point(0.0, 10.0), point(100.0, 20.0)]),
            "M0.00,10.00L100.00,20.00"
        );
    }

    #[test]
    fn a_flat_neighbour_keeps_the_peak_from_overshooting() {
        let path = monotone_path(&[point(0.0, 50.0), point(50.0, 0.0), point(100.0, 0.0)]);

        assert!(path.ends_with("83.33,0.00,100.00,0.00"), "{path}");
    }
}
