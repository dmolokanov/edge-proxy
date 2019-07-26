use failure::Fail;
use log::{log, Level};

pub fn failure(fail: &dyn Fail) {
    log!(Level::Error, "{}", fail);
    for cause in fail.iter_causes() {
        log!(Level::Error, "\tcaused by: {}", cause);
    }
}
