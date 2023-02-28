use chrono::Utc;

/// log prints a message to stdout with a timestamp
pub fn log(title: &str, msg: &str) {
    println!(
        "{} {:>12} {}",
        Utc::now().format("%a %b %e %T %Y"),
        title,
        msg
    );
}

/// log_msg prints a message to stdout with a timestamp
pub fn log_msg(msg: &str) {
    println!("{:>12} {}", Utc::now().format("%a %b %e %T %Y"), msg);
}
