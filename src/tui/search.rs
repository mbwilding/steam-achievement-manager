pub fn fuzzy_score(haystack: &str, needle: &str) -> Option<i64> {
    let needle = needle.trim();
    if needle.is_empty() {
        return None;
    }

    if haystack.contains(needle) {
        return Some(1_000_000 - (haystack.len() as i64 - needle.len() as i64));
    }

    let mut score: i64 = 0;
    let mut last_match_index: Option<usize> = None;

    let mut hay_chars = haystack.chars().enumerate();
    let mut prev_char: Option<char> = None;

    for needle_char in needle.chars() {
        let mut found: Option<(usize, char, Option<char>)> = None;

        for (i, h) in hay_chars.by_ref() {
            if h == needle_char {
                found = Some((i, h, prev_char));
                prev_char = Some(h);
                break;
            }
            prev_char = Some(h);
        }

        let (i, h, prev) = found?;

        score += 10;

        if let Some(last) = last_match_index {
            if i == last + 1 {
                score += 15;
            } else {
                score -= (i.saturating_sub(last + 1) as i64) * 2;
            }
        }

        if is_word_boundary(prev, h) {
            score += 20;
        }

        last_match_index = Some(i);
    }

    if let Some(first) = last_match_index {
        score -= first as i64;
    }

    Some(score)
}

fn is_word_boundary(prev: Option<char>, current: char) -> bool {
    match prev {
        None => true,
        Some(p) => {
            (p == ' ' || p == '_' || p == '-' || p == ':' || p == '/' || p == '(' || p == '[')
                || (p.is_ascii_lowercase() && current.is_ascii_uppercase())
        }
    }
}
