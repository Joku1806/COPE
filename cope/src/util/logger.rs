
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) =>  {
        let local: DateTime<Local> = Local::now();
        let time = local.format("%H:%M:%S.%f\t").to_string();
        println!("{}{}{}{}",
            "[".blue().bold(),
            time.blue().bold(),
            "] LOG ".blue().bold(),
            format!($($arg)*)
        );
    }
}
