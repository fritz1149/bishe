fn main() {
    let s = String::from("hello world");
    let t = &s;
    say(t);
}

fn say(s: &str) {
    println!("{}",s);
}