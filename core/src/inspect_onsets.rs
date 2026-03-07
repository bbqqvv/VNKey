use vnkey_core::phonology::ONSETS;

fn main() {
    println!("ONSETS length: {}", ONSETS.len());
    for (i, s) in ONSETS.iter().enumerate() {
        println!("{}: '{}'", i, s);
    }
}
