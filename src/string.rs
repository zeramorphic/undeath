pub fn hconcat(left: &str, right: &str, gap: &str) -> String {
    left.lines()
        .zip(right.lines())
        .map(|(l, r)| format!("{l}{gap}{r}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn large_number(number: usize) -> String {
    if number < 1_000 {
        number.to_string()
    } else if number < 1_000_000 {
        format!("{} {:03}", number / 1_00, number % 1_000)
    } else if number < 1_000_000_000 {
        format!("{}.{:03} million", number / 1_000_000, (number % 1_000_000) / 1_000)
    } else {
        format!(
            "{}.{:03} billion",
            number / 1_000_000_000,
            (number % 1_000_000_000) / 1_000_000
        )
    }
}
