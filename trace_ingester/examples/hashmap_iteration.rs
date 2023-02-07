use std::collections::HashMap;

fn main() {
    let mut a = HashMap::new();

    a.insert("a", 1);
    a.insert("d", 15);
    a.insert("x", 1);
    a.insert("t", 11);

    let over_ten = a
        .values() //
        .filter(|&&v| v > 10);

    for i in over_ten {
        println!("{i}");
    }
}
