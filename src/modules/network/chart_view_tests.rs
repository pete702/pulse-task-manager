//! Network chart tests.

use crate::modules::memory::downsampling::nice_ceiling;

#[test]
fn nice_ceiling_gigabit_and_10g() {
    assert_eq!(nice_ceiling(1500.0, 1.0), 2000.0);
    assert_eq!(nice_ceiling(8500.0, 1.0), 10000.0);
    assert_eq!(nice_ceiling(900.0, 1.0), 2000.0);
    assert_eq!(nice_ceiling(30.0, 1.0), 50.0);
    assert_eq!(nice_ceiling(0.0, 10.0), 10.0);
}
