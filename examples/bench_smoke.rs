use causalmesh::{sync_pair, Replica};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!("causalmesh-bench-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let a_path = root.join("a");
    let b_path = root.join("b");
    let mut a = Replica::open(&a_path)?;
    let mut b = Replica::open(&b_path)?;

    let n = 2_000u64;
    let start = Instant::now();
    for i in 0..n {
        a.put(format!("k{i}"), format!("v{i}").into_bytes())?;
    }
    let put_elapsed = start.elapsed();

    let start = Instant::now();
    for i in 0..n {
        let _ = a.get(&format!("k{i}"))?;
    }
    let get_elapsed = start.elapsed();

    let start = Instant::now();
    let delta = sync_pair(&mut a, &mut b)?;
    let delta_sync = start.elapsed();

    let start = Instant::now();
    let zero = sync_pair(&mut a, &mut b)?;
    let zero_sync = start.elapsed();

    drop(a);
    let start = Instant::now();
    let reopened = Replica::open(&a_path)?;
    let reopen = start.elapsed();

    println!("environment_os={}", std::env::consts::OS);
    println!("operations={n}");
    println!("put_total_ms={:.3}", put_elapsed.as_secs_f64() * 1000.0);
    println!("get_total_ms={:.3}", get_elapsed.as_secs_f64() * 1000.0);
    println!(
        "delta_sync_ms={:.3} sent={}",
        delta_sync.as_secs_f64() * 1000.0,
        delta.sent_a_to_b
    );
    println!(
        "zero_sync_ms={:.3} sent={}",
        zero_sync.as_secs_f64() * 1000.0,
        zero.sent_a_to_b + zero.sent_b_to_a
    );
    println!(
        "reopen_ms={:.3} recovered_ops={}",
        reopen.as_secs_f64() * 1000.0,
        reopened.status()?.operation_count
    );

    std::fs::remove_dir_all(root)?;
    Ok(())
}
