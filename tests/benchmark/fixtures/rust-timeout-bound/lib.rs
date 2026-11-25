pub fn bounded_timeout(requested: u32, maximum: u32) -> u32 {
    requested.max(maximum)
}

#[cfg(test)]
mod tests {
    use super::bounded_timeout;

    #[test]
    fn clamps_above_maximum() {
        assert_eq!(bounded_timeout(500, 100), 100);
    }

    #[test]
    fn preserves_delays_at_or_below_maximum() {
        assert_eq!(bounded_timeout(0, 100), 0);
        assert_eq!(bounded_timeout(42, 100), 42);
        assert_eq!(bounded_timeout(100, 100), 100);
    }
}
