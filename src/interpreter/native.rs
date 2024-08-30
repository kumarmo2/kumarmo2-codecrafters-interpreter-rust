use std::time::{SystemTime, UNIX_EPOCH};

use super::Object;
pub(crate) fn clock(args: Option<Box<dyn Iterator<Item = Object>>>) -> Object {
    let inner_fn = || {
        Object::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_secs_f64(),
        )
    };

    inner_fn()
}
