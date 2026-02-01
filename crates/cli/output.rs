use std::io::Write;

pub fn line(message: impl AsRef<str>) {
    println!("{}", message.as_ref());
}

pub fn error(message: impl AsRef<str>) {
    let mut stderr = std::io::stderr().lock();
    let _ = writeln!(stderr, "{}", message.as_ref());
}
