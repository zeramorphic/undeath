pub fn hconcat(left: &str, right: &str, gap: &str) -> String {
    left.lines()
        .zip(right.lines())
        .map(|(l, r)| format!("{l}{gap}{r}"))
        .collect::<Vec<_>>()
        .join("\n")
}
