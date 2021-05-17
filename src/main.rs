use sha2::{Digest, Sha256};
use std::convert::TryInto;
use std::sync::Arc;
use std::sync::RwLock;

struct Solution {
    pub prefix: [u8; 4],
    pub result: [u8; 32],
}

impl Solution {
    fn print(&self) {
        println!("{}", hex::encode(&self.result));
        println!("{}", hex::encode(&self.prefix));
    }
}

fn solve(begin: u8, end: u8, mut data: Vec<u8>, solution: Arc<RwLock<Option<Solution>>>) {
    let mut hasher = Sha256::new();

    // This 4x for loops could be replaced by single u32 write to make fewer writes and look much less ugly
    for b0 in begin..=end {
        data[0] = b0;
        for b1 in 0..0xFFu8 {
            data[1] = b1;
            for b2 in 0..0xFFu8 {
                data[2] = b2;
                for b3 in 0..0xFFu8 {
                    data[3] = b3;

                    if solution.read().unwrap().is_some() {
                        // Solved in other thread
                        return;
                    }

                    hasher.update(&data);
                    let result = hasher.finalize_reset();

                    if result[31] == 0xfe && result[30] == 0xca {
                        let mut guard = solution.write().unwrap();
                        if guard.is_none() {
                            // some means that task was already solved in other thread
                            *guard = Some(Solution {
                                prefix: [b0, b1, b2, b3],
                                result: result.as_slice().try_into().expect("Wrong lenght"),
                            })
                        }
                        return;
                    }
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Missing input string");
        return;
    }

    let mut data_in: Vec<u8> = vec![0; 4];
    data_in.append(&mut hex::decode(&args[1]).expect("Decoding failed"));

    let solution = Arc::new(RwLock::new(None));

    // Split the job to multiple tasks
    let tasks_cnt = num_cpus::get() as u8;
    let mut tasks = vec![];

    // Determine ~equal ranges for tasks
    let mut tasks_ranges: Vec<(u8, u8)> = vec![];
    let step = std::u8::MAX / tasks_cnt;
    for i in 0..tasks_cnt {
        let begin = i * step;
        let end = if i == (tasks_cnt - 1) {
            std::u8::MAX
        } else {
            (i + 1) * step - 1
        };
        tasks_ranges.push((begin, end))
    }

    // Fire up tasks
    for range in tasks_ranges {
        let data = data_in.clone();
        let s = solution.clone();

        tasks.push(std::thread::spawn(move || {
            solve(range.0, range.1, data, s);
        }));
    }

    // Wait for finish
    for task in tasks {
        task.join().expect("Task has panicked");
    }

    solution.read().unwrap().as_ref().unwrap().print();
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_solve() {
        let mut data = hex::decode("00000000129df964b701d0b8e72fe7224cc71643cf8e000d122e72f742747708f5e3bb6294c619604e52dcd8f5446da7e9ff7459d1d3cefbcc231dd4c02730a22af9880c").unwrap();
        let mut solution = Arc::new(RwLock::new(None));

        solve(0, 0xFF, data, solution.clone());

        assert!(solution.read().unwrap().is_some());
        assert!(
            hex::encode(&solution.read().unwrap().as_ref().unwrap().prefix)
                .eq_ignore_ascii_case("00003997")
        );
        assert!(
            hex::encode(&solution.read().unwrap().as_ref().unwrap().result).eq_ignore_ascii_case(
                "6681edd1d36af256c615bf6dcfcda03c282c3e0871bd75564458d77c529dcafe"
            )
        );

        data = hex::decode("00000000129df964b701d0b8e72fe7224cc71643cf8e000d122e72f742747708f5e3bb6294c619604e52dcd8f5446da7e9ff7459d1d3cefbcc231dd4c02730a22af98806").unwrap();
        solution = Arc::new(RwLock::new(None));

        solve(0, 0xFF, data, solution.clone());

        assert!(solution.read().unwrap().is_some());
        assert!(
            hex::encode(&solution.read().unwrap().as_ref().unwrap().prefix)
                .eq_ignore_ascii_case("00031f8b")
        );
        assert!(
            hex::encode(&solution.read().unwrap().as_ref().unwrap().result).eq_ignore_ascii_case(
                "b2d6e0e21ae7f478e8d732e1c4dd1c76ae5113dafd36410ddf34ce399c32cafe"
            )
        );
    }
}
