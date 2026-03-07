fn main() {
    let mut v = vec![
        "a", "ai", "ao", "au", "ay", "e", "eo", "i", "ia", "iu", "iê", "iêu", "o", "oa", "oai", "oao", "oay", "oe", "oi", "oo", "oă", "u", "ua", "uâ", "uă", "uân", "uâng", "uất", "uê", "ui", "uô", "uôn", "uông", "uôi", "uơ", "uy", "uya", "uyê", "y", "ya", "yê", "yêu", "â", "âu", "ây", "ê", "êu", "ô", "ôi", "ông", "ôn", "ă", "ơ", "ơi", "ơm", "ơn", "ơng", "ư", "ưa", "ươ", "ươn", "ương", "ươi", "ưu"
    ];
    v.sort();
    v.dedup();
    
    println!("Sorted VOWEL_CLUSTERS:");
    for s in v {
        println!("\"{}\",", s);
    }
}
