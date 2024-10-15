use pmrcore::ac::session::{
    SessionFactory,
    SessionTokenFactory,
};
use test_pmr::is_send_sync;

#[test]
fn check_send_sync() {
    is_send_sync::<SessionFactory>();
    is_send_sync::<SessionTokenFactory>();
}
