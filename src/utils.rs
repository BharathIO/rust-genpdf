use chrono::Utc;

const FORMAT: &str = "%a %b %e %T %Y";

/// log prints a message to stdout with a timestamp
pub fn log(title: &str, msg: &str) {
    println!("{} {:>12} {}", Utc::now().format(FORMAT), title, msg);
}

/// log_msg prints a message to stdout with a timestamp
pub fn log_msg(msg: &str) {
    println!("{:>12} {}", Utc::now().format(FORMAT), msg);
}
