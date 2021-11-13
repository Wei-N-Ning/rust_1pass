use lib_rust_1pass::*;

fn main() {
    let conf = OnePasswordConfig::new("".to_string());
    println!("{:?}", conf.inst_config.is_none());
}
