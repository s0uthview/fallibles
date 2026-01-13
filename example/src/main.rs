use fallible::*;

#[derive(Debug, FallibleError)]
#[fallible(message = "configuration error")]
struct ConfigError {
    message: String,
}

#[derive(Debug, FallibleError)]
enum NetworkError {
    #[fallible]
    Timeout { message: String },
    ConnectionRefused,
    InvalidResponse,
}

#[fallible]
fn read_config() -> Result<i32, &'static str> {
    Ok(42)
}

#[fallible]
fn fetch_data() -> Result<&'static str, &'static str> {
    Ok("Hello, Fallible!")
}

#[fallible]
fn load_settings() -> Result<String, ConfigError> {
    Ok("settings loaded".to_string())
}

#[fallible]
fn network_request() -> Result<String, NetworkError> {
    Ok("response data".to_string())
}

fn main() {
    println!("Without failure injection:");
    match read_config() {
        Ok(x) => println!("read_config succeeded: {x}"),
        Err(_) => println!("read_config failed!"),
    }
    match fetch_data() {
        Ok(msg) => println!("fetch_data succeeded: {msg}"),
        Err(_) => println!("fetch_data failed!"),
    }

    println!("\nWith 50% probability:");
    fallible_core::configure_failures(
        fallible_core::FailureConfig::new()
            .with_probability(0.5)
    );

    for i in 0..10 {
        match read_config() {
            Ok(x) => println!("Attempt {}: read_config succeeded: {x}", i),
            Err(_) => println!("Attempt {}: read_config failed!", i),
        }
    }

    println!("\nWith trigger_every(3):");
    fallible_core::configure_failures(
        fallible_core::FailureConfig::new()
            .trigger_every(3)
    );

    for i in 0..10 {
        match fetch_data() {
            Ok(msg) => println!("Attempt {}: fetch_data succeeded: {msg}", i),
            Err(_) => println!("Attempt {}: fetch_data failed!", i),
        }
    }

    println!("\nTesting custom error types:");
    fallible_core::configure_failures(
        fallible_core::FailureConfig::new()
            .with_probability(0.5)
    );

    for i in 0..5 {
        match load_settings() {
            Ok(s) => println!("Attempt {}: load_settings succeeded: {s}", i),
            Err(e) => println!("Attempt {}: load_settings failed: {:?}", i, e),
        }
    }

    for i in 0..5 {
        match network_request() {
            Ok(s) => println!("Attempt {}: network_request succeeded: {s}", i),
            Err(e) => println!("Attempt {}: network_request failed: {:?}", i, e),
        }
    }

    fallible_core::clear_failure_config();

    println!("\nWith observability:");
    fallible_core::configure_failures(
        fallible_core::FailureConfig::new()
            .with_probability(0.5)
            .on_check(|fp| {
                println!("  [CHECK] {} at {}:{}", fp.function, fp.file, fp.line);
            })
            .on_failure(|fp| {
                println!("  [FAILURE TRIGGERED] {} (id: {:?})", fp.function, fp.id);
            })
    );

    println!("Testing with callbacks:");
    for i in 0..3 {
        println!("Attempt {}:", i);
        match read_config() {
            Ok(x) => println!("  Result: succeeded with {}", x),
            Err(_) => println!("  Result: failed"),
        }
    }

    if let Some(stats) = fallible_core::get_failure_stats() {
        println!("\nStatistics:");
        println!("  Total checks: {}", stats.total_checks);
        println!("  Total failures: {}", stats.total_failures);
        println!("  Failure rate: {:.1}%", 
            (stats.total_failures as f64 / stats.total_checks as f64) * 100.0);
    }

    fallible_core::clear_failure_config();
}