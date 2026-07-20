//! 驗收:challenges::ds::dsu。完成後移除 #[ignore]。

use challenges::ds::dsu::Dsu;

/// 基本連通性與 components 計數。
#[test]
#[ignore = "完成 challenge 後移除"]
fn basic_union_connected() {
    let mut d = Dsu::new(5);
    assert_eq!(d.components(), 5);
    assert!(d.union(0, 1));
    assert!(d.union(2, 3));
    assert!(!d.connected(0, 2));
    assert!(d.union(1, 3));
    assert!(d.connected(0, 2));
    assert_eq!(d.components(), 2); // {0,1,2,3} {4}
}

/// boundary:自反與重複 union 都是 no-op。
#[test]
#[ignore = "完成 challenge 後移除"]
fn self_and_duplicate() {
    let mut d = Dsu::new(3);
    assert!(!d.union(1, 1));
    assert!(d.union(0, 1));
    assert!(!d.union(1, 0));
    assert_eq!(d.components(), 2);
}

/// boundary:深鏈——沒做優化(或用了遞迴)這裡會超時/爆 stack。
#[test]
#[ignore = "完成 challenge 後移除"]
fn deep_chain_performance() {
    const N: usize = 200_000;
    let mut d = Dsu::new(N);
    for i in 0..N - 1 {
        d.union(i, i + 1);
    }
    // 全部 find 一遍:攤銷近常數的話瞬間完成
    let root = d.find(0);
    for i in 0..N {
        assert_eq!(d.find(i), root);
    }
    assert_eq!(d.components(), 1);
}

/// 與天真標號模型對照(隨機 union 序列)。
#[test]
#[ignore = "完成 challenge 後移除"]
fn matches_naive_labeling() {
    let n = 20;
    let mut d = Dsu::new(n);
    let mut label: Vec<usize> = (0..n).collect();
    let mut x: u64 = 7;
    for _ in 0..60 {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let a = (x >> 33) as usize % n;
        let b = (x >> 45) as usize % n;
        d.union(a, b);
        let (la, lb) = (label[a], label[b]);
        if la != lb {
            for l in label.iter_mut() {
                if *l == lb {
                    *l = la;
                }
            }
        }
    }
    for i in 0..n {
        for j in 0..n {
            assert_eq!(d.connected(i, j), label[i] == label[j], "({i},{j})");
        }
    }
}
