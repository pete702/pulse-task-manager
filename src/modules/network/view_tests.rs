//! Network view tests.

use super::*;
#[test]
fn set_scope_all_persists_after_view_pass() {
    {
        let mut g = state().lock().unwrap_or_else(|e| e.into_inner());
        g.selected = Some(Arc::from("Eth0"));
    }
    set_scope_all();
    {
        let g = state().lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(g.selected, None);
    }
    set_chart_mode_value(2);
    assert_eq!(chart_mode_value(), 2);
    set_chart_mode_value(1);
    assert_eq!(chart_mode_value(), 1);
    set_chart_mode_value(0);
    assert_eq!(chart_mode_value(), 0);
}
