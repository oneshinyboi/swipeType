use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub fn get_keyboard_layout() -> HashMap<char, Point> {
    let mut layout = HashMap::new();
    let rows = [
        ("qwertyuiop", 0.0, 0.0),
        ("asdfghjkl", 0.5, 1.0),
        ("zxcvbnm", 1.5, 2.0),
    ];

    for (chars, x_offset, y) in rows {
        for (i, c) in chars.chars().enumerate() {
            layout.insert(c, Point { x: i as f64 + x_offset, y });
        }
    }
    layout
}

pub fn euclidean_dist(p1: &Point, p2: &Point) -> f64 {
    ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt()
}

pub fn get_word_path(word: &str, layout: &HashMap<char, Point>) -> Vec<Point> {
    let key_points: Vec<Point> = word.chars()
        .filter_map(|c| layout.get(&c.to_ascii_lowercase()).cloned())
        .collect();

    if key_points.is_empty() {
        return vec![];
    }

    let step_size = 0.5;
    let mut full_path = vec![key_points[0]];

    for i in 1..key_points.len() {
        let p1 = key_points[i-1];
        let p2 = key_points[i];
        
        let dist = euclidean_dist(&p1, &p2);
        if dist > step_size {
            let num_steps = (dist / step_size) as i32;
            let dx = (p2.x - p1.x) / num_steps as f64;
            let dy = (p2.y - p1.y) / num_steps as f64;
            
            for s in 1..num_steps {
                let new_x = p1.x + dx * s as f64;
                let new_y = p1.y + dy * s as f64;
                full_path.push(Point { x: new_x, y: new_y });
            }
        }
        full_path.push(p2);
    }

    full_path
}

pub fn simplify_path(path: &[Point]) -> Vec<Point> {
    if path.is_empty() {
        return vec![];
    }
    
    let mut new_path = vec![path[0]];
    for p in path.iter().skip(1) {
        if euclidean_dist(p, new_path.last().unwrap()) > 0.01 {
            new_path.push(*p);
        }
    }
    new_path
}

/// Fast DTW with Sakoe-Chiba band constraint and early termination.
/// Uses O(n) space instead of O(n*m).
pub fn dtw_distance_fast(s: &[Point], t: &[Point], window: usize, cutoff: f64) -> f64 {
    let n = s.len();
    let m = t.len();
    if n == 0 || m == 0 {
        return f64::INFINITY;
    }

    // Early reject if lengths are too different
    let len_diff = (n as i64 - m as i64).unsigned_abs() as usize;
    if len_diff > window {
        return f64::INFINITY;
    }

    // Use two rows for O(m) space
    let mut prev = vec![f64::INFINITY; m + 1];
    let mut curr = vec![f64::INFINITY; m + 1];
    prev[0] = 0.0;

    for i in 1..=n {
        curr[0] = f64::INFINITY;
        let j_start = if i > window { i - window } else { 1 };
        let j_end = (i + window).min(m);

        // Reset out-of-band cells
        if j_start > 1 {
            curr[j_start - 1] = f64::INFINITY;
        }

        let mut row_min = f64::INFINITY;
        for j in j_start..=j_end {
            let cost = euclidean_dist(&s[i - 1], &t[j - 1]);
            let prev_min = prev[j].min(curr[j - 1]).min(prev[j - 1]);
            curr[j] = cost + prev_min;
            row_min = row_min.min(curr[j]);
        }

        // Early termination: if entire row exceeds cutoff, no point continuing
        if row_min > cutoff {
            return f64::INFINITY;
        }

        std::mem::swap(&mut prev, &mut curr);
    }

    prev[m]
}

/// Unused
pub fn dtw_distance(s: &[Point], t: &[Point]) -> f64 {
    let n = s.len();
    let m = t.len();
    if n == 0 || m == 0 {
        return f64::INFINITY;
    }

    let mut dtw = vec![vec![f64::INFINITY; m + 1]; n + 1];
    dtw[0][0] = 0.0;

    for i in 1..=n {
        for j in 1..=m {
            let cost = euclidean_dist(&s[i - 1], &t[j - 1]);
            let prev_min = dtw[i - 1][j].min(dtw[i][j - 1]).min(dtw[i - 1][j - 1]);
            dtw[i][j] = cost + prev_min;
        }
    }

    dtw[n][m]
}