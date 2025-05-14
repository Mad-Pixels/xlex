use super::classifier::Classifier;

use std::{borrow::Cow, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a, TK: Copy + Eq + Hash> {
    pub kind: TokenKind<TK>,
    pub text: Cow<'a, str>,
    
    pub start: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BaseKind {
    Number,
    Symbol,
    Space,
    Word,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TokenKind<TK: Copy + Eq + Hash> {
    pub custom: Option<TK>,
    pub base: BaseKind,
}

impl<TK: Copy + Eq + Hash> TokenKind<TK> {
    pub const NUMBER: Self = Self { base: BaseKind::Number, custom: None };
    pub const SYMBOL: Self = Self { base: BaseKind::Symbol, custom: None };
    pub const SPACE: Self = Self { base: BaseKind::Space, custom: None };
    pub const WORD: Self = Self { base: BaseKind::Word, custom: None };

    #[inline]
    pub fn new(base: BaseKind, custom: Option<TK>) -> Self { Self { custom, base } }

    #[inline]
    pub fn matches(&self, c: char, classifier: &impl Classifier<Custom = TK>) -> bool {
        is_match(self.base, self.custom, c, classifier)
    }

    pub fn predicate<'a>(&'a self, classifier: &'a impl Classifier<Custom = TK>) -> impl Fn(char) -> bool + 'a{
        move |ch| is_match(self.base, self.custom, ch, classifier)
    }

    
}

#[inline]
pub fn classify_base(c: char) -> BaseKind {
    if c.is_alphabetic() {
        BaseKind::Word
    } else if c.is_numeric() {
        BaseKind::Number
    } else if c.is_whitespace() {
        BaseKind::Space
    } else {
        BaseKind::Symbol
    }
}

#[inline]
fn is_match<TK: Copy + Eq + Hash>(
    base: BaseKind, 
    custom: Option<TK>, 
    ch: char,
    classifier: &impl Classifier<Custom = TK>,
) -> bool {
    let (b, cust, _repl) = classifier.classify(ch);
    base == b && custom == cust
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum MyCustom { Tab, Newline, Char7 }

    struct TestClassifier;
    impl Classifier for TestClassifier {
        type Custom = MyCustom;

        fn classify(&self, c: char) -> (BaseKind, Option<MyCustom>, Option<Cow<'static, str>>) {
            match c {
                '\t' => (BaseKind::Space, Some(MyCustom::Tab), None),
                '\n' => (BaseKind::Space, Some(MyCustom::Newline), None),
                '7'  => (BaseKind::Number, Some(MyCustom::Char7), None),
                _    => (classify_base(c), None, None),
            }
        }
    }

    #[test]
    fn test_matches_default_kinds() {
        let cls = TestClassifier;

        assert!(TokenKind::WORD.matches('a', &cls));
        assert!(TokenKind::NUMBER.matches('3', &cls));
        assert!(TokenKind::SPACE.matches(' ', &cls));
        assert!(TokenKind::SYMBOL.matches('!', &cls));
    }

    #[test]
    fn test_matches_custom_variants() {
        let cls = TestClassifier;

        let tab_kind = TokenKind::new(BaseKind::Space, Some(MyCustom::Tab));
        assert!(tab_kind.matches('\t', &cls));
        assert!(!tab_kind.matches(' ', &cls));
    }

    #[test]
    fn test_predicate_true_and_false() {
        let cls = TestClassifier;

        let kind = TokenKind::new(BaseKind::Number, Some(MyCustom::Char7));
        let pred = kind.predicate(&cls);
        assert!(pred('7'));
        assert!(!pred('8'));
        assert!(!pred('a')); 
    }

    #[test]
    fn test_classify_base_function() {
        assert_eq!(classify_base('x'), BaseKind::Word);
        assert_eq!(classify_base('3'), BaseKind::Number);
        assert_eq!(classify_base(' '), BaseKind::Space);
        assert_eq!(classify_base('@'), BaseKind::Symbol);
    }
}