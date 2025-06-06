/// Checks if two values are alike based on the percentage of matching words.
///
/// The function takes two generic values `target` and `comparison`, which must implement the
/// `Display` trait. It also takes a minimum percentage `min_percent_to_reach` as an `isize`.
///
/// Returns `true` if the percentage of matching words between `target` and `comparison`
/// is greater than or equal to `min_percent_to_reach`, otherwise returns `false`.
///
/// # Examples
///
/// ```
/// use crate::is_alike;
///
/// let target = "Hello, world!";
/// let comparison = "Hello, there!";
/// let min_percent_to_reach = 50;
/// let result = is_alike(target, comparison, min_percent_to_reach);
///
/// assert_eq!(result, true);
/// ```
pub fn is_alike<T>(target: T, comparison: T, min_percent_to_reach: isize) -> bool
where
    T: std::fmt::Display,
{
    let p1string = target.to_string();
    let p2string = comparison.to_string();

    let max_len = std::cmp::max(p1string.len(), p2string.len());
    if max_len == 0 {
        return true;
    }

    let distance = levenshtein(&p1string, &p2string);
    let similarity = 1.0 - (distance as f64 / max_len as f64);
    let percent = similarity * 100.0;

    percent >= min_percent_to_reach as f64
}

/// Computes the Levenshtein distance between two strings.
/// This is a standard dynamic programming implementation.
fn levenshtein(a: &str, b: &str) -> usize {
    let mut costs = (0..=b.len()).collect::<Vec<_>>();

    for (i, ca) in a.chars().enumerate() {
        let mut last_cost = i;
        costs[0] = i + 1;

        for (j, cb) in b.chars().enumerate() {
            let new_cost = if ca == cb {
                last_cost
            } else {
                1 + std::cmp::min(std::cmp::min(costs[j], costs[j + 1]), last_cost)
            };
            last_cost = costs[j + 1];
            costs[j + 1] = new_cost;
        }
    }

    costs[b.len()]
}
