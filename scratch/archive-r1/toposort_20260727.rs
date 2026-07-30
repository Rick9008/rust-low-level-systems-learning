#[derive(Debug)]
enum Error {
    CycleError,
    DuplicateName,
}

// What we gonna do is that we store the test into a adjacent list and do a topologic sort
// if there's some dep not exists in the name we just ignore this
// and for whole topologic sort tc and sc:
// TC: O(V + E), where V is the names and E is the edges count
// SC: O(V + E)

use std::collections::{HashMap, HashSet, VecDeque};

fn schedule(tests: &[(String, Vec<String>)]) -> Result<Vec<String>, Error> {
    let mut cnt_tests = tests.len();
    let mut exists_name: HashSet<&str> = HashSet::new();

    for (name, _dep) in tests {
        // sanity test
        if !exists_name.insert(name) {
            return Err(Error::DuplicateName);
        }
    }

    let mut adj_list: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut inde_cnt: HashMap<&str, usize> = HashMap::new();

    for (name, deps) in tests {
        let inde = inde_cnt.entry(name).or_default();
        for dep in deps {
            // get bug by dry run: non-exist name in dep
            if !exists_name.contains(dep.as_str()) {
                continue;
            }
            adj_list.entry(dep).or_default().push(name);
            *inde += 1;
        }
    }
    // topo sort with bfs
    let mut dq = VecDeque::new();

    for (&name, ind) in &inde_cnt {
        if *ind == 0 {
            dq.push_back(name);
        }
    }
    let mut ans = Vec::new();

    while !dq.is_empty() {
        // Invariant: dq is not empty, so we unwrap
        let top = dq.pop_front().unwrap();
        ans.push(top.to_string());
        cnt_tests -= 1;
        if adj_list.get(top).is_none() {
            continue;
        }
        for nxt in &adj_list[top] {
            let ind = inde_cnt.get_mut(nxt).unwrap();
            *ind -= 1;
            if ind == &0 {
                dq.push_back(nxt);
            }
        }
    }

    if cnt_tests > 0 {
        return Err(Error::CycleError);
    }
    Ok(ans)
}

fn validate(tests: &[(String, Vec<String>)], topo: Vec<String>) -> bool {
    if tests.len() != topo.len() {
        return false;
    }
    let mut pos_map: HashMap<&str, usize> = HashMap::new();
    let exists_name: HashSet<&str> = tests.iter().map(|(str, vec)| str.as_str()).collect();
    for (idx, val) in topo.iter().enumerate() {
        if pos_map.contains_key(val.as_str()) {
            return false;
        }
        pos_map.entry(val.as_str()).or_insert(idx);
    }
    for (name, deps) in tests {
        let name = name.as_str();
        for dep in deps {
            if !exists_name.contains(dep.as_str()) {
                continue;
            }
            if pos_map[name] < pos_map[dep.as_str()] {
                return false;
            }
        }
    }

    true
}

fn main() {
    let test_1 = &[
        ("a".to_string(), vec!["b".to_string(), "c".to_string()]),
        ("b".to_string(), vec![]),
        ("c".to_string(), vec!["b".to_string()]),
    ];

    let res = schedule(test_1);
    assert!(res.is_ok());
    assert!(validate(test_1, res.expect("Should return Result")));
    // boundary test
    // 1. all nodes with no dependency
    // 2. a -> b | a -> c | b -> d | c -> d
    // 3. with no name exists dependency

    let test_2 = &[
        ("a".to_string(), vec!["b".to_string(), "c".to_string()]),
        ("b".to_string(), vec!["d".to_string()]),
        ("c".to_string(), vec!["d".to_string()]),
        ("d".to_string(), vec![]),
    ];
    assert!(validate(
        test_2,
        schedule(test_2).expect("Should return Result")
    ));

    let test_3 = &[
        (
            "a".to_string(),
            vec!["b".to_string(), "c".to_string(), "d".to_string()],
        ),
        ("b".to_string(), vec![]),
    ];

    assert!(validate(
        test_3,
        schedule(test_3).expect("Should return Result")
    ));
}

