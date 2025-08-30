use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rw_parser_rs::renderware::dff::dff_parser::DffParser;
use std::fs;
use std::path::Path;

fn parse_dff_benchmark(c: &mut Criterion) {
    let file_path = Path::new(r"C:\Users\Administrator\Documents\Code\playground-render\assets1\source_skins\supa90.dff");
    let file_buffer = fs::read(file_path).expect("Failed to read file");

    c.bench_function("DFF Parser", |b| {
        b.iter(|| {
            let mut parser = DffParser::new(black_box(&file_buffer));
            let _ = parser.parse();
        })
    });
}

criterion_group!(benches, parse_dff_benchmark);
criterion_main!(benches);