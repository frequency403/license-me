use std::fmt::Display;

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
    let mut percent: f64 = 0.0;
    let mut words_match: isize = 0;
    if p1_s_words != p2_s_words {
        percent = (p1_s_words - p2_s_words) as f64;
    }
    for (count, word) in p1_s.into_iter().enumerate() {
        if count < p2_s_words as usize && word == p2_s[count] {
            words_match += 1;
        }
    }

    percent = (words_match as f64 / p1_s_words as f64) * 100.00;

    percent >= min_percent_to_reach as f64

}
