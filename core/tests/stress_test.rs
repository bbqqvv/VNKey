use std::time::Instant;
use vnkey_core::{Engine, InputMode};

#[test]
fn test_engine_throughput_and_latency() {
    let mut engine = Engine::new(InputMode::Telex);

    // 1. Generate a stress corpus (approx 10,000 characters)
    let sentence = "Chào bạn, đây là một bài kiểm tra hiệu năng cho bộ gõ VNKey được viết bằng ngôn ngữ lập trình Rust. ";
    let mut corpus = String::new();
    for _ in 0..100 {
        corpus.push_str(sentence);
    }

    // 2. Benchmark processing
    let start = Instant::now();
    let mut results_len = 0;
    for c in corpus.chars() {
        let out = engine.process_key(c);
        results_len += out.len();
    }
    let duration = start.elapsed();

    // 3. Report metrics
    let char_count = corpus.chars().count();
    let avg_latency = duration.as_micros() as f64 / char_count as f64;

    println!("\n--- STRESS TEST REPORT ---");
    println!("Total characters processed: {}", char_count);
    println!("Total time elapsed: {:?}", duration);
    println!("Average latency per character: {:.4} µs", avg_latency);
    println!(
        "Throughput: {:.2} characters/sec",
        char_count as f64 / duration.as_secs_f64()
    );

    // Performance Requirement: Average latency must be < 500µs (0.5ms)
    assert!(
        avg_latency < 500.0,
        "Latency too high: {:.2}µs",
        avg_latency
    );
}

#[test]
fn test_long_word_buffer_safety() {
    let mut engine = Engine::new(InputMode::Telex);

    // Feed extremely long sequence of alphanumeric characters without spaces (500 chars)
    // Engine has a safety limit of 50 chars per word buffer.
    engine.reset();
    for _ in 0..51 {
        engine.process_key('a');
    }

    // After 50, it should have reset, so buffer length should be 0
    // (since 51st char is returned but not added to new buffer in current logic)
    assert_eq!(engine.buffer().len(), 0);
}
