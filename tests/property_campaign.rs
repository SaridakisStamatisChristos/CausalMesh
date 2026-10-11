use causalmesh::{sync_pair, Replica};
use tempfile::TempDir;

fn sync_indices(replicas: &mut [Replica], i: usize, j: usize) {
    assert_ne!(i, j);
    if i < j {
        let (left, right) = replicas.split_at_mut(j);
        sync_pair(&mut left[i], &mut right[0]).unwrap();
    } else {
        let (left, right) = replicas.split_at_mut(i);
        sync_pair(&mut right[0], &mut left[j]).unwrap();
    }
}

fn next(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed
}

#[test]
fn deterministic_random_histories_converge_after_eventual_delivery() {
    for initial_seed in 0u64..64 {
        let dirs = [TempDir::new().unwrap(), TempDir::new().unwrap(), TempDir::new().unwrap()];
        let paths = [
            dirs[0].path().to_path_buf(),
            dirs[1].path().to_path_buf(),
            dirs[2].path().to_path_buf(),
        ];
        let mut replicas: Vec<Replica> = paths.iter().map(|p| Replica::open(p).unwrap()).collect();
        let mut rng = initial_seed ^ 0x9e3779b97f4a7c15;

        for step in 0..80u64 {
            let r = next(&mut rng);
            let idx = (r as usize) % 3;
            let key = format!("k{}", (r >> 8) % 3);
            match (r >> 16) % 7 {
                0..=2 => {
                    replicas[idx]
                        .put(key, format!("s{initial_seed}-{step}-{r}").into_bytes())
                        .unwrap();
                }
                3 => {
                    replicas[idx].delete(key).unwrap();
                }
                4 | 5 => {
                    let mut other = ((r >> 24) as usize) % 3;
                    if other == idx {
                        other = (other + 1) % 3;
                    }
                    sync_indices(&mut replicas, idx, other);
                }
                _ => {
                    replicas[idx].snapshot().unwrap();
                    replicas[idx] = Replica::open(&paths[idx]).unwrap();
                }
            }
        }

        for _ in 0..3 {
            sync_indices(&mut replicas, 0, 1);
            sync_indices(&mut replicas, 1, 2);
            sync_indices(&mut replicas, 0, 2);
        }
        let s0 = replicas[0].logical_state().unwrap();
        assert_eq!(s0, replicas[1].logical_state().unwrap(), "seed {initial_seed}");
        assert_eq!(s0, replicas[2].logical_state().unwrap(), "seed {initial_seed}");
    }
}
