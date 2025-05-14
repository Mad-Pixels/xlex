use super::token::BaseKind;

use std::{collections::HashSet, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config<TK: Copy + Eq + Hash> {
    pub skip_base: HashSet<BaseKind>,
    pub skip_custom: HashSet<Option<TK>>,

    pub group_symbols: bool,
}

impl<TK: Copy + Eq + Hash> Default for Config<TK> {
    #[inline]
    fn default() -> Self {
        let mut skip_base = HashSet::new();
        skip_base.insert(BaseKind::Space);

        Self {
            skip_base,
            skip_custom: HashSet::new(),
            group_symbols: false,
        }
    }
}

impl<TK: Copy + Eq + Hash> Config<TK> {
    #[inline]
    pub fn skip_base<IT>(mut self, kinds: IT) -> Self
    where
        IT: IntoIterator<Item = BaseKind>,
    {
        self.skip_base.extend(kinds);
        self
    }

    #[inline]
    pub fn skip_custom<IT>(mut self, kinds: IT) -> Self
    where
        IT: IntoIterator<Item = TK>,
    {
        self.skip_custom.extend(kinds.into_iter().map(Some));
        self
    }

    #[inline]
    pub fn should_skip(&self, base: BaseKind, custom: Option<TK>) -> bool {
        self.skip_base.contains(&base) || self.skip_custom.contains(&custom)
    }

    #[inline]
    pub fn with_grouped_symbols(mut self) -> Self {
        self.group_symbols = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::BaseKind;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum MyCustom {
        Tab,
    }

    #[test]
    fn test_default_skips_space() {
        let cfg: Config<MyCustom> = Config::default();
        assert!(cfg.should_skip(BaseKind::Space, None));
        assert!(!cfg.should_skip(BaseKind::Word, None));
    }

    #[test]
    fn test_skip_base_adds_additional_kinds() {
        let cfg: Config<MyCustom> = Config::default().skip_base([BaseKind::Symbol]);
        assert!(cfg.should_skip(BaseKind::Space, None));
        assert!(cfg.should_skip(BaseKind::Symbol, None));
        assert!(!cfg.should_skip(BaseKind::Number, None));
    }

    #[test]
    fn test_skip_custom_subtypes() {
        let cfg = Config::default().skip_custom([MyCustom::Tab]);
        assert!(cfg.should_skip(BaseKind::Space, Some(MyCustom::Tab)));
    }

    #[test]
    fn test_group_symbols_flag() {
        let mut cfg: Config<MyCustom> = Config::default();
        assert!(!cfg.group_symbols);

        cfg = cfg.with_grouped_symbols();
        assert!(cfg.group_symbols);
    }
}
