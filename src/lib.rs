pub fn shard_work<T, R>(src: Vec<T>, f: fn(T) -> R) -> Vec<R>
where
    T: Sized + Copy + Send + Sync,
    R: Sized + Copy + Default + Send + Sync,
{
    // empty vector
    if src.is_empty() {
        return Vec::new();
    }

    let threshold = 256;
    // less equal than threshold
    if src.len() <= threshold {
        return src.iter().map(|x| -> R { f(*x) }).collect();
    }

    // greater than threshold
    let n = src.len();
    let mut thread_num = 8;
    let mut each = n / thread_num;
    let mut extra = n % thread_num;
    if each < threshold {
        thread_num = (n + threshold - 1) / threshold;
        each = n / thread_num;
        extra = n % thread_num;
    }

    let mut result = vec![R::default(); n];

    let mut src_slice = src.as_slice();
    let mut dst_slice = result.as_mut_slice();

    crossbeam::scope(|scope| {
        while !src_slice.is_empty() {
            let mut end = each + if extra == 0 { 0 } else { 1 };
            if end > src_slice.len() {
                end = src_slice.len();
            }

            extra = if extra == 0 { 0 } else { extra - 1 };

            let (thread_src_slice, remain_src_slice) = src_slice.split_at(end);
            let (thread_dst_slice, remain_dst_slice) = dst_slice.split_at_mut(end);
            scope.spawn(move |_| {
                let mut i = 0;
                for t in thread_src_slice {
                    thread_dst_slice[i] = f(*t);
                    i += 1;
                }
            });

            src_slice = remain_src_slice;
            dst_slice = remain_dst_slice;
        }
    }).unwrap();

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::time;
    const THRESHOLD:usize = 256;
    const THREAD_NUM:usize = 8;

    fn f(i:i32) ->f64 {
        i as f64 + (i % 1000) as f64 / 1000f64
    }
    fn fsleep(i:i32)->f64 {
        // sleep for a while, to make sure scope.spawn really create threads
        // std::thread::sleep(time::Duration::from_millis(100));
        f(i)
    }
    fn fx(i:&i32)->f64 {
        f(*i)
    }
    #[test]
    fn test_empty() {
        let src_slice:Vec<i32> = vec![];
        let dst_vec = shard_work(src_slice, f);
        assert!(dst_vec.is_empty());
    }

    #[test]
    fn test_half_threshold() {
        let src_slice:Vec<i32> = (0..THRESHOLD as i32/2).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, f);
        assert_eq!(exp_dst, dst_vec);
    }

    #[test]
    fn test_threshold() {
        let src_slice:Vec<i32> = (0..THRESHOLD as i32).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, f);
        assert_eq!(exp_dst, dst_vec);
    }
    #[test]
    fn test_threshold_plus_1() {
        let src_slice:Vec<i32> = (0..(THRESHOLD as i32 + 1)).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, f);
        assert_eq!(exp_dst, dst_vec);
    }

    #[test]
    fn test_threshold_minus_1() {
        let src_slice:Vec<i32> = (0..(THRESHOLD as i32 - 1)).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, f);
        assert_eq!(exp_dst, dst_vec);
    }

    #[test]
    fn test_4threshold() {
        let src_slice:Vec<i32> = (0..(THRESHOLD as i32 * 4)).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, f);
        assert_eq!(exp_dst, dst_vec);
    }
    #[test]
    fn test_4threshold_plus_one() {
        let src_slice:Vec<i32> = (0..(THRESHOLD as i32 * 4 + 1)).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, f);
        assert_eq!(exp_dst, dst_vec);
    }
    #[test]
    fn test_4threshold_minus_one() {
        let src_slice:Vec<i32> = (0..(THRESHOLD as i32 * 4 - 1)).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, f);
        assert_eq!(exp_dst, dst_vec);
    }

    #[test]
    fn test_all_threshold() {
        let src_slice:Vec<i32> = (0..(THRESHOLD * THREAD_NUM)as i32).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, f);
        assert_eq!(exp_dst, dst_vec);
    }
    #[test]
    fn test_all_threshold_plus_one() {
        let src_slice:Vec<i32> = (0..(THRESHOLD * THREAD_NUM)as i32).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, f);
        assert_eq!(exp_dst, dst_vec);
    }
    #[test]
    fn test_all_plush_threshold_minus_one() {
        let src_slice:Vec<i32> = (0..((THRESHOLD * (THREAD_NUM+1)-1)) as i32).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, f);
        assert_eq!(exp_dst, dst_vec);
    }
    #[test]
    fn test_2all_plush_threshold_plus_one() {
        let src_slice:Vec<i32> = (0..((THRESHOLD* 2 * (THREAD_NUM+1)+1)) as i32).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, f);
        assert_eq!(exp_dst, dst_vec);
    }
    #[test]
    fn test_2all_plush_threshold_minus_one() {
        let src_slice:Vec<i32> = (0..((THRESHOLD* 2 * (THREAD_NUM+1)-1)) as i32).collect();
        let exp_dst:Vec<f64> = src_slice.iter().map(fx).collect();
        let dst_vec = shard_work(src_slice, fsleep);
        assert_eq!(exp_dst, dst_vec);
    }
}
