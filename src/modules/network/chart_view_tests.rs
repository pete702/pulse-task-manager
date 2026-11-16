//! Network chart tests.

use super::clean_4_axis_max;

#[test]
fn clean_4_axis_max_traffic_windows() {
    assert_eq!(clean_4_axis_max(0.0, 1.0), 1.0);
    assert_eq!(clean_4_axis_max(90.0, 1.0), 100.0);
    assert_eq!(clean_4_axis_max(300.0, 1.0), 360.0);
    assert_eq!(clean_4_axis_max(900.0, 1.0), 1000.0);
    assert_eq!(clean_4_axis_max(1500.0, 1.0), 2000.0);
    assert_eq!(clean_4_axis_max(8500.0, 1.0), 10000.0);
}
