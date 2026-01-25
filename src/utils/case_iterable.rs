pub trait CaseIterable: Sized {
    fn all_cases() -> &'static [Self];
}
