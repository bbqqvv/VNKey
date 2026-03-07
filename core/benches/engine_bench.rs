use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vietnamese_ime_engine::{Engine, InputMode};

fn bench_process_key(c: &mut Criterion) {
    let mut engine = Engine::new(InputMode::Telex);
    
    c.bench_function("process_key_telex_h", |b| b.iter(|| {
        engine.process_key(black_box('h'));
        engine.reset();
    }));

    c.bench_function("process_key_telex_hoas", |b| b.iter(|| {
        engine.process_key(black_box('h'));
        engine.process_key(black_box('o'));
        engine.process_key(black_box('a'));
        engine.process_key(black_box('s'));
        engine.reset();
    }));
}

criterion_group!(benches, bench_process_key);
criterion_main!(benches);
