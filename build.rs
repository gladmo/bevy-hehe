fn main() {
    let utc8 = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
    let now = chrono::Utc::now().with_timezone(&utc8);
    println!(
        "cargo:rustc-env=BUILD_TIME={}",
        now.format("%Y-%m-%d %H:%M:%S")
    );
}
