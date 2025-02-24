## Rust Log util
A log tool written in Rust that can work in conjunction with the log component of the standard library. It features synchronized log file recording and log file rolling capabilities.

## Quick start
```rust
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
```
## Run this test, you will see console print:
![image](https://github.com/user-attachments/assets/52f5d24c-2110-4be7-b3a3-5acea45ed528) <br>
*Debug Log display only when Set environment variable `RUST_LOG="debug"`* <br>
And you will got log/TestLog with 2 log files: `TestLog.log`, `TestLog_xxxxxxxx.log`

## No global instance LogUtil(No log file out)
If you don't want to use LogUtil with a global variable instance, init LogUtil with:
```rust
LogUtil::init().unwrap()
```
Log messages will only print to console, no log file output.