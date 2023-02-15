fn main() {
    let raw = std::fs::read_to_string("resources/monitor.yml").unwrap();
    println!("{}", raw);
}