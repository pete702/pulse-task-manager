//! Network chart tests.

use crate::modules::memory::downsampling::nice_ceiling;

#[test]
fn nice_ceiling_gigabit_and_10g() {
    assert_eq!(nice_ceiling(1500.0, 1.0), 1500.0);
    assert_eq!(nice_ceiling(8500.0, 1.0), 10000.0);
    assert_eq!(nice_ceiling(900.0, 1.0), 1000.0);
    assert_eq!(nice_ceiling(30.0, 1.0), 30.0);
    assert_eq!(nice_ceiling(0.0, 10.0), 10.0);
}

#[test]
fn nice_ceiling_100mbps_lock() {
    assert_eq!(nice_ceiling(90.0, 1.0), 100.0);
    assert_eq!(nice_ceiling(100.0, 1.0), 100.0);
    assert_eq!(nice_ceiling(105.0, 1.0), 100.0);
}

#[test]
fn nice_ceiling_gigabit_lock() {
    assert_eq!(nice_ceiling(900.0, 1.0), 1000.0);
    assert_eq!(nice_ceiling(1000.0, 1.0), 1000.0);
    assert_eq!(nice_ceiling(1050.0, 1.0), 1000.0);
}

#[test]
fn nice_ceiling_ping_latency() {
    assert_eq!(nice_ceiling(4.0, 5.0), 5.0);
    assert_eq!(nice_ceiling(12.0, 5.0), 15.0);
    assert_eq!(nice_ceiling(19.0, 5.0), 20.0);
    assert_eq!(nice_ceiling(23.0, 5.0), 25.0);
}
