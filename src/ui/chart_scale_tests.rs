use super::*;
use egui_plot::GridInput;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn assert_no_duplicate_coords(marks: &[GridMark]) {
    for i in 0..marks.len() {
        for j in (i + 1)..marks.len() {
            assert!(
                (marks[i].value - marks[j].value).abs() > 1e-9,
                "duplicate x value {}",
                marks[i].value
            );
        }
    }
}

#[test]
fn clean_4_axis_max_edges_and_steps() {
    assert!(approx_eq(clean_4_axis_max(f64::NAN, 1.0), 1.0));
    assert!(approx_eq(clean_4_axis_max(0.0, 0.0), 1.0));
    assert!(approx_eq(clean_4_axis_max(3.0, 10.0), 10.0));
    assert!(approx_eq(clean_4_axis_max(300.0, 1.0), 360.0));
    assert!(approx_eq(clean_4_axis_max(2.9, 1.0), 3.2));
}

#[test]
fn format_y_tick_variants() {
    assert_eq!(format_y_tick(1.0, 2.0, "MB/s"), "1.0 MB/s");
    assert_eq!(format_y_tick(50.0, 100.0, "%"), "50 %");
    assert_eq!(format_y_tick(12.3, 10.0, ""), "12.3");
    assert_eq!(format_y_tick(f64::NAN, 10.0, "x"), "0.0 x");
}

#[test]
fn window_secs_to_span_min_clamps_and_maps() {
    assert!(approx_eq(window_secs_to_span_min(0), 1.0));
    assert!(approx_eq(window_secs_to_span_min(15), 1.0));
    assert!(approx_eq(window_secs_to_span_min(60), 1.0));
    assert!(approx_eq(window_secs_to_span_min(300), 5.0));
    assert!(approx_eq(window_secs_to_span_min(900), 15.0));
}

#[test]
fn build_x_grid_marks_bounds_and_no_duplicates() {
    for (span, expected_min) in [(1.0_f64, -1.0_f64), (5.0, -5.0), (15.0, -15.0)] {
        let marks = build_x_grid_marks(span);
        assert!(!marks.is_empty());
        let min_v = marks.iter().map(|m| m.value).fold(f64::INFINITY, f64::min);
        let max_v = marks.iter().map(|m| m.value).fold(f64::NEG_INFINITY, f64::max);
        assert!(min_v <= expected_min + 1e-9, "span {span} min {min_v}");
        assert!(max_v >= -1e-9, "span {span} max {max_v}");
        assert_no_duplicate_coords(&marks);
    }
}

#[test]
fn x_grid_spacer_bounds_and_no_duplicates() {
    let input = GridInput { bounds: (-15.0, 0.0), base_step_size: 1.0 };
    for (span, expected_min) in [(1.0_f64, -1.0_f64), (5.0, -5.0), (15.0, -15.0)] {
        let marks = x_grid_spacer(span, &input);
        assert!(!marks.is_empty());
        let min_v = marks.iter().map(|m| m.value).fold(f64::INFINITY, f64::min);
        let max_v = marks.iter().map(|m| m.value).fold(f64::NEG_INFINITY, f64::max);
        assert!(min_v <= expected_min + 1e-9, "span {span} min {min_v}");
        assert!(max_v >= -1e-9, "span {span} max {max_v}");
        assert_no_duplicate_coords(&marks);
    }
}

#[test]
fn minutes_ago_and_convert() {
    assert!(approx_eq(minutes_ago(120.0, 180.0), -1.0));
    assert!(approx_eq(minutes_ago(180.0, 180.0), 0.0));
    let pts = [[60.0, 1.0], [120.0, 2.0]];
    let out = convert_to_minutes_ago(&pts, 120.0);
    assert_eq!(out, vec![[-1.0, 1.0], [0.0, 2.0]]);
}

#[test]
fn anchored_cases() {
    assert!(anchored(Vec::new()).is_empty());
    let one = anchored(vec![[-0.5, 7.0]]);
    assert_eq!(one.len(), 2);
    assert!(approx_eq(one[0][0], -ONE_SEC_MIN) && approx_eq(one[0][1], 7.0));
    assert!(approx_eq(one[1][0], 0.0) && approx_eq(one[1][1], 7.0));

    let mid = anchored(vec![[-1.0, 1.0], [-0.2, 2.0]]);
    assert_eq!(mid.first().copied(), Some([-1.0, 1.0]));
    assert_eq!(mid.last().copied(), Some([0.0, 2.0]));
    assert_eq!(mid.iter().filter(|p| approx_eq(p[0], 0.0)).count(), 1);

    let at_zero = anchored(vec![[-1.0, 1.0], [0.0, 3.0]]);
    assert_eq!(at_zero.len(), 2);
    assert_eq!(at_zero.last().copied(), Some([0.0, 3.0]));
}

#[test]
fn percent_y_grid_values() {
    let vals: Vec<f64> = PERCENT_Y_GRID.iter().map(|m| m.value).collect();
    assert_eq!(vals, vec![0.0, 25.0, 50.0, 75.0, 100.0]);
    assert!(PERCENT_Y_GRID.iter().all(|m| approx_eq(m.step_size, 25.0)));
}

#[test]
fn x_axis_ticks_values() {
    assert_eq!(x_axis_ticks(60), vec![1.0, 0.75, 0.5, 0.25, 0.0]);
    assert_eq!(x_axis_ticks(300), vec![5.0, 4.0, 3.0, 2.0, 1.0, 0.0]);
    assert_eq!(x_axis_ticks(900), vec![15.0, 12.0, 9.0, 6.0, 3.0, 0.0]);
}

#[test]
fn format_tick_label_values() {
    assert_eq!(format_tick_label(60, 0.0), "0s");
    assert_eq!(format_tick_label(300, 0.0), "0m");
    assert_eq!(format_tick_label(900, 0.0), "0m");
    assert_eq!(format_tick_label(60, 1.0), "60s");
    assert_eq!(format_tick_label(60, 0.25), "15s");
    assert_eq!(format_tick_label(300, 3.0), "3m");
    assert_eq!(format_tick_label(900, 12.0), "12m");
}

#[test]
fn format_minutes_and_axis_label_values() {
    assert_eq!(format_minutes(0.0), "0m");
    assert_eq!(format_minutes(-0.0), "0m");
    assert_eq!(format_minutes(-1.0), "-1m");
    assert_eq!(format_minutes(-2.5), "-2.5m");

    assert_eq!(axis_label(60, -1.0), "-1m");
    assert_eq!(axis_label(60, -0.5), "-0.5m");
    assert_eq!(axis_label(60, 0.0), "0m");
    for mark in [-0.9, -0.8, -0.7, -0.6, -0.4, -0.3, -0.2, -0.1] {
        assert_eq!(axis_label(60, mark), "", "1m axis must hide mark {mark}");
    }
    assert_eq!(axis_label(300, -4.0), "-4m");
    assert_eq!(axis_label(900, 0.0), "0m");
}
