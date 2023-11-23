use std::fmt::Display;

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
        T: Display,
{
    let p1string = target.to_string();
    let p1_s = p1string.split_ascii_whitespace().collect::<Vec<&str>>();
    let p1_s_words = p1_s.len() as isize;
    let p2string = comparison.to_string();
    let p2_s = p2string.split_ascii_whitespace().collect::<Vec<&str>>();
    let p2_s_words = p2_s.len() as isize;
    let mut percent: f64 = if p1_s_words != p2_s_words { (p1_s_words - p2_s_words) as f64 } else { 0.0 };
    let mut words_match: isize = 0;

    for (count, word) in p1_s.into_iter().enumerate() {
        if count < p2_s_words as usize && word == p2_s[count] {
            words_match += 1;
        }
    }

    percent += (words_match as f64 / p1_s_words as f64) * 100.00;

    percent >= min_percent_to_reach as f64
}
