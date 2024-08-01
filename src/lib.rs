pub mod log_level;
pub mod log_util;

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use log::{debug, error, info, warn};
    use log_util::LogUtil;

    use super::*;

    lazy_static! {
        static ref BASE_LOG: LogUtil = LogUtil::new("TestLog");
    }

    #[test]
    fn test_log() {
        LogUtil::init_with_logger(&BASE_LOG).unwrap();

        info!("Test");
        error!("Test: {}", 5);
        warn!("Test: {}", "abc");
        let b = "def";
        debug!("Test: {}, {b}", "abc");
    }
}
