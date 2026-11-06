//! Network chart tests.

use super::*;
use std::sync::Arc;

fn series(v: f64) -> Arc<Vec<[f64; 2]>> {
    Arc::new(vec![[0.0, 0.0], [1.0, v]])
}

#[test]
fn stepped_ceiling_gigabit_and_10g() {
    assert_eq!(stepped_ceiling(&series(1500.0), &series(0.0)), 2500.0);
    assert_eq!(stepped_ceiling(&series(8500.0), &series(0.0)), 10000.0);
    assert_eq!(stepped_ceiling(&series(900.0), &series(0.0)), 1000.0);
    assert_eq!(stepped_ceiling(&series(30.0), &series(30.0)), 50.0);
    assert_eq!(stepped_ceiling(&series(0.0), &series(0.0)), 10.0);
}
