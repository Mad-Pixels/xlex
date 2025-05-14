use super::token::{classify_base, BaseKind};

use std::{borrow::Cow, hash::Hash};

pub trait Classifier {
    type Custom: Copy + Eq + Hash;

    fn classify(&self, c: char) -> (BaseKind, Option<Self::Custom>, Option<Cow<'static, str>>);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NoCustom;

pub struct DefaultClassifier;

impl Classifier for DefaultClassifier {
    type Custom = NoCustom;

    #[inline]
    fn classify(&self, c: char) -> (BaseKind, Option<Self::Custom>, Option<Cow<'static, str>>) {
        (classify_base(c), None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::config::Config;
    use crate::lexer::inline::LexerInline;
    use crate::lexer::token::{classify_base, BaseKind, Token, TokenKind};
    use std::borrow::Cow;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum MyCustom {
        Tab,
        Seven,
        LetterA,
    }

    struct MyClassifier;
    impl Classifier for MyClassifier {
        type Custom = MyCustom;

        fn classify(&self, c: char) -> (BaseKind, Option<MyCustom>, Option<Cow<'static, str>>) {
            match c {
                '\t' => (
                    BaseKind::Space,
                    Some(MyCustom::Tab),
                    Some(Cow::Borrowed("<TAB>")),
                ),
                '7' => (
                    BaseKind::Number,
                    Some(MyCustom::Seven),
                    Some(Cow::Owned("SEVEN".into())),
                ),
                'a' => (
                    BaseKind::Word,
                    Some(MyCustom::LetterA),
                    Some(Cow::Borrowed("A")),
                ),
                _ => (classify_base(c), None, None),
            }
        }
    }

    #[test]
    fn test_replacement_mixed() {
        let cfg = Config::default().with_grouped_symbols();
        let cls = MyClassifier;
        let input = "a\t7!x";
        let tokens: Vec<_> = LexerInline::new(&cfg, &cls, input).collect();

        let expected = vec![
            Token {
                text: Cow::Borrowed("A"),
                kind: TokenKind::new(BaseKind::Word, Some(MyCustom::LetterA)),
                start: 0,
            },
            Token {
                text: Cow::Owned("<TAB>".into()),
                kind: TokenKind::new(BaseKind::Space, Some(MyCustom::Tab)),
                start: 1,
            },
            Token {
                text: Cow::Owned("SEVEN".into()),
                kind: TokenKind::new(BaseKind::Number, Some(MyCustom::Seven)),
                start: 2,
            },
            Token {
                text: Cow::Borrowed("!"),
                kind: TokenKind::SYMBOL,
                start: 3,
            },
            Token {
                text: Cow::Borrowed("x"),
                kind: TokenKind::WORD,
                start: 4,
            },
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_default_classifier_behavior() {
        let cfg = Config::default().with_grouped_symbols();
        let cls = DefaultClassifier;
        let input = "abc 123!";
        let tokens: Vec<_> = LexerInline::new(&cfg, &cls, input).collect();

        let expected = vec![
            Token {
                text: Cow::Borrowed("abc"),
                kind: TokenKind::WORD,
                start: 0,
            },
            Token {
                text: Cow::Borrowed("123"),
                kind: TokenKind::NUMBER,
                start: 4,
            },
            Token {
                text: Cow::Borrowed("!"),
                kind: TokenKind::SYMBOL,
                start: 7,
            },
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_classifier_handles_none_replacement() {
        struct NoReplace;
        impl Classifier for NoReplace {
            type Custom = ();
            fn classify(&self, c: char) -> (BaseKind, Option<()>, Option<Cow<'static, str>>) {
                (classify_base(c), None, None)
            }
        }

        let cfg = Config::default();
        let cls = NoReplace;
        let input = "abc!";
        let tokens: Vec<_> = LexerInline::new(&cfg, &cls, input).collect();

        let expected = vec![
            Token {
                text: Cow::Borrowed("abc"),
                kind: TokenKind::WORD,
                start: 0,
            },
            Token {
                text: Cow::Borrowed("!"),
                kind: TokenKind::SYMBOL,
                start: 3,
            },
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_grouped_symbols_and_replacement() {
        let cfg = Config::default().with_grouped_symbols();
        let cls = MyClassifier;
        let input = "!!7";
        let tokens: Vec<_> = LexerInline::new(&cfg, &cls, input).collect();

        let expected = vec![
            Token {
                text: Cow::Borrowed("!!"),
                kind: TokenKind::SYMBOL,
                start: 0,
            },
            Token {
                text: Cow::Owned("SEVEN".into()),
                kind: TokenKind::new(BaseKind::Number, Some(MyCustom::Seven)),
                start: 2,
            },
        ];

        assert_eq!(tokens, expected);
    }
}
