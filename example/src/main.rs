use fallible::*;

#[fallible]
fn read_config() -> Result<i32, &'static str> {
    Ok(42)
}

fn main() {
    let x = read_config().unwrap();
    println!("{x}");
}