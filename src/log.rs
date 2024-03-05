use colored::Colorize;

pub fn info(msg: &str) {
    println!("[{}] {}", "INFO".cyan(), msg);
}

pub fn success(msg: &str) {
    println!("[{}] {}", "SUCCESS".green(), msg);
}

pub fn error(msg: &str) {
    println!("[{}] {}", "ERROR".red(), msg);
}
