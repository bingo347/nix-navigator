macro_rules! __log_colorized_level {
    (INFO) => {
        concat!("\x1b[1;32m", "INFO", "\x1b[0m")
    };
    (WARN) => {
        concat!("\x1b[1;33m", "WARN", "\x1b[0m")
    };
    (ERR) => {
        concat!("\x1b[1;31m", "ERR", "\x1b[0m")
    };
    ($other:ident) => {
        stringify!($other)
    };
}

macro_rules! log {
    ($level:ident : $template:literal $(, $($args:tt)* )?) => {
        println!("{}: {}", __log_colorized_level!($level), format_args!($template $(, $($args)* )?))
    };
}

macro_rules! info {
    ($template:literal $(, $($args:tt)* )?) => {
        log!(INFO: $template $(, $($args)* )?)
    };
}

macro_rules! warn {
    ($template:literal $(, $($args:tt)* )?) => {
        log!(WARN: $template $(, $($args)* )?)
    };
}

macro_rules! error {
    ($template:literal $(, $($args:tt)* )?) => {
        log!(ERR: $template $(, $($args)* )?)
    };
}
