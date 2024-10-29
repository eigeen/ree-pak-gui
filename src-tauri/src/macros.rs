#[macro_export]
macro_rules! warp_result_elapsed {
    ($expr:expr) => {{
        let start = std::time::Instant::now();
        let result = $expr.map_err(|e| e.to_string());
        let elapsed = start.elapsed();
        println!("Wrapped command spent {} ms", elapsed.as_millis());
        result
    }};

    ($expr:expr, $msg:expr) => {{
        let start = std::time::Instant::now();
        let result = $expr.map_err(|e| e.to_string());
        let elapsed = start.elapsed();
        println!($msg, elapsed.as_millis());
        result
    }};
}
