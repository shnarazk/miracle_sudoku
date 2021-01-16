use crate::{get_block_len, get_range, pos::*, Rules};

fn collect_digits(fixed: &[(Pos, usize)], p: Pos, blen: isize) -> Vec<usize> {
    fixed
        .iter()
        .filter(|(q, _)| p.i == q.i || p.j == q.j || ((p.i - 1) / blen == (q.i - 1) / blen && (p.j - 1) / blen == (q.j - 1) / blen))
        .map(|r| r.1)
        .collect::<Vec<usize>>()
}

pub fn sudoku_preset(fixed: &[(Pos, usize)]) -> Rules {
    let range = get_range();
    let mut rules = Vec::new();
    for (p, d) in fixed.iter() {
        for dd in 1..=range as usize {
            rules.push(vec![p.state(dd, *d == dd).as_lit()]);
        }
    }
    rules
}

/// 1. At least one number sholud be assigned on each cell.
/// 2. So a positive assginment should be a trigger to assgin the rest vars negatively.
/// O(n^4)
pub fn sudoku_ident(fixed: &[(Pos, usize)]) -> Rules {
    let range = get_range();
    let mut rules = Vec::new();
    for i in 1..=range {
        for j in 1..=range {
            let p = Pos::at(i, j);
            if fixed.iter().any(|r| p == r.0) {
                continue;
            }
            // at-least constraints
            let v = (1..=(range as usize))
                .map(|d| p.state(d, true).as_lit())
                .collect::<Vec<_>>();
            rules.push(v);
            // at-most constraints
            let presets = collect_digits(fixed, p);
            for d in 1..=(range as usize) {
                if presets.contains(&d) {
                    continue;
                }
                for dd in 1..(range as usize) {
                    if d != dd {
                        rules.push(p.state(d, true).requires(p.state(dd, false)));
                    }
                }
            }
        }
    }
    rules
}

/// 1. At least each number should be assigned on each group once.
/// O(n^2)
pub fn sudoku_ident2() -> Rules {
    let range = get_range();
    let mut rules = Vec::new();

    // squares
    let bsize = (range as f64).sqrt() as isize;
    let mut block_walk = Vec::new();
    for i in 0..bsize {
        for j in 0..bsize {
            block_walk.push(Pos::at(i, j));
        }
    }

    // for values
    for n in 1..=range {
        // rows
        for i in 1..=range {
            rules.push(
                (1..=range)
                    .map(|j| Pos::at(i, j).state(n as usize, true).as_lit())
                    .collect::<Vec<_>>(),
            );
        }
        // columns
        for j in 1..=range {
            rules.push(
                (1..=range)
                    .map(|i| Pos::at(i, j).state(n as usize, true).as_lit())
                    .collect::<Vec<_>>(),
            );
        }
        // squares
        for i in (0..bsize).map(|k| k * bsize + 1) {
            for j in (0..bsize).map(|k| k * bsize + 1) {
                let base = Pos::at(i, j);
                let mut temp = Vec::new();
                for offset in &block_walk {
                    if let Some(p) = (base + *offset).valid(range) {
                        temp.push(p.state(n as usize, true).as_lit());
                    }
                }
                rules.push(temp);
            }
        }
    }
    rules
}

/// 1. In Each row, each number should be assgined at most once.
/// O(n^4)
pub fn sudoku_row(fixed: &[(Pos, usize)]) -> Rules {
    let range = get_range();
    let mut rules = Vec::new();
    for i in 1..=range {
        for j in 1..=range {
            let p = Pos::at(i, j);
            for jj in j + 1..=range {
                let q = Pos::at(i, jj);
                if fixed.iter().any(|r| p == r.0) && fixed.iter().any(|r| q == r.0) {
                    continue;
                } else if let Some((_, d)) = fixed.iter().find(|r| p == r.0) {
                    rules.push(vec![q.state(*d, false).as_lit()]);
                } else if let Some((_, d)) = fixed.iter().find(|r| q == r.0) {
                    rules.push(vec![p.state(*d, false).as_lit()]);
                } else {
                    for d in 1..=(range as usize) {
                        rules.push(p.state(d, true).requires(q.state(d, false)));
                    }
                }
            }
        }
    }
    rules
}

/// 1. In Each column, each number should be assgined at most once.
/// O(n^4)
pub fn sudoku_column(fixed: &[(Pos, usize)]) -> Rules {
    let range = get_range();
    let mut rules = Vec::new();
    for j in 1..=range {
        for i in 1..=range {
            let p = Pos::at(i, j);
            for ii in i + 1..=range {
                let q = Pos::at(ii, j);
                if fixed.iter().any(|r| p == r.0) && fixed.iter().any(|r| q == r.0) {
                    continue;
                } else if let Some((_, d)) = fixed.iter().find(|r| p == r.0) {
                    rules.push(vec![q.state(*d, false).as_lit()]);
                } else if let Some((_, d)) = fixed.iter().find(|r| q == r.0) {
                    rules.push(vec![p.state(*d, false).as_lit()]);
                } else {
                    for d in 1..=(range as usize) {
                        rules.push(p.state(d, true).requires(q.state(d, false)));
                    }
                }
            }
        }
    }
    rules
}

/// 1. In Each square block, each number should be assgined at most once.
/// O(n^4)
pub fn sudoku_block(fixed: &[(Pos, usize)]) -> Rules {
    let range = get_range();
    let bsize = (range as f64).sqrt() as isize;
    let mut rules = Vec::new();
    let mut block_walk = Vec::new();
    for i in 0..bsize {
        for j in 0..bsize {
            block_walk.push(Pos::at(i, j));
        }
    }
    for i in (0..bsize).map(|k| k * bsize + 1) {
        for j in (0..bsize).map(|k| k * bsize + 1) {
            let base = Pos::at(i, j);
            for tail in 0..block_walk.len() {
                if let Some(p) = (base + block_walk[tail]).valid(range) {
                    for offset in &block_walk[tail + 1..] {
                        if let Some(q) = (base + *offset).valid(range) {
                            if fixed.iter().any(|r| p == r.0) && fixed.iter().any(|r| q == r.0) {
                                continue;
                            } else if let Some((_, d)) = fixed.iter().find(|r| p == r.0) {
                                rules.push(vec![q.state(*d, false).as_lit()]);
                            } else if let Some((_, d)) = fixed.iter().find(|r| q == r.0) {
                                rules.push(vec![p.state(*d, false).as_lit()]);
                            } else {
                                for d in 1..=(range as usize) {
                                    rules.push(p.state(d, true).requires(q.state(d, false)));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    rules
}

pub fn veried(ans: &[Vec<usize>]) -> bool {
    let range = get_range();
    let blen = get_block_len() as usize;
    let sorted = (1..=range as usize).collect::<Vec<usize>>();
    for (i, line) in ans.iter().enumerate() {
        let mut l = line.clone();
        l.sort_unstable();
        if l != sorted {
            dbg!((i, l));
            return false;
        }
    } 
    for j in 0..range as usize {
        let mut l = ans.iter().map(|l| l[j]).collect::<Vec<usize>>();
        l.sort_unstable();
        if l != sorted {
            dbg!((j, l));
            return false;
        }
    } 
    for i in 0..blen as usize {
        for j in 0..blen as usize {
            let mut l = Vec::new();
            for line in &ans[i*blen..(i+1)*blen] {
                for jj in j*blen..(j+1)*blen {
                    l.push(line[jj]);
                }
            }
            assert_eq!(l.len(), range as usize);
            l.sort_unstable();
            if l != sorted {
                dbg!((i, j));
                return false;
            }
        }
    }
    true
}
