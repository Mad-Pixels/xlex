use super::classifier::Classifier;
use super::config::Config;
use super::token::{BaseKind, Token, TokenKind};

use std::{borrow::Cow, hash::Hash, str::CharIndices};

pub struct LexerInline<'a, TK, CL>
where
    TK: Copy + Eq + Hash,
    CL: Classifier<Custom = TK>,
{
    config: &'a Config<TK>,
    classifier: &'a CL,

    input: &'a str,
    pos: usize,
}

impl<'a, TK, CL> LexerInline<'a, TK, CL>
where
    TK: Copy + Eq + Hash,
    CL: Classifier<Custom = TK>,
{
    #[inline]
    pub fn new(config: &'a Config<TK>, classifier: &'a CL, input: &'a str) -> Self {
        LexerInline {
            config,
            classifier,
            input,
            pos: 0,
        }
    }

    #[inline]
    fn next_token(&mut self) -> Option<Token<'a, TK>> {
        while self.pos < self.input.len() {
            let start = self.pos;

            let mut i = self.input[self.pos..].char_indices();
            let (_, ch) = i.next().unwrap();
            self.pos += ch.len_utf8();

            let (bk, ck, repl) = self.classifier.classify(ch);
            let kind = TokenKind::new(bk, ck);

            if let Some(repl) = repl {
                return Some(Token {
                    text: Cow::Owned(repl.into_owned()),
                    kind,
                    start,
                });
            }
            if self.config.should_skip(bk, ck) {
                self.consume_while(bk, ck, &mut i);
                continue;
            }
            if (bk == BaseKind::Symbol && self.config.group_symbols) || bk != BaseKind::Symbol {
                self.consume_while(bk, ck, &mut i);
            }

            return Some(Token {
                text: Cow::Borrowed(&self.input[start..self.pos]),
                start,
                kind,
            });
        }

        None
    }

    #[inline]
    fn consume_while(&mut self, base: BaseKind, custom: Option<TK>, i: &mut CharIndices) {
        while let Some((_, ch)) = i.next() {
            let (bk, ck, repl) = self.classifier.classify(ch);
            if repl.is_some() || bk != base || ck != custom {
                break;
            }
            self.pos += ch.len_utf8();
        }
    }
}

impl<'a, TK, CL> Iterator for LexerInline<'a, TK, CL>
where
    TK: Copy + Eq + Hash,
    CL: Classifier<Custom = TK>,
{
    type Item = Token<'a, TK>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::classifier::DefaultClassifier;
    use crate::lexer::config::Config;
    use crate::lexer::token::{classify_base, BaseKind};
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
    fn test_grouping_symbols_default() {
        let cfg = Config::default().with_grouped_symbols();
        let cls = DefaultClassifier;
        let input = "a!!b";
        let out: Vec<_> = LexerInline::new(&cfg, &cls, input).collect();
        assert_eq!(
            out.iter().map(|t| t.text.as_ref()).collect::<Vec<_>>(),
            &["a", "!!", "b"]
        );
    }

    #[test]
    fn test_skipping_tab_custom() {
        let cfg = Config::default()
            .skip_custom([MyCustom::Tab])
            .with_grouped_symbols();
        let cls = MyClassifier;
        let input = "a\t\t7!!b";
        let out: Vec<_> = LexerInline::new(&cfg, &cls, input).collect();
        assert_eq!(
            out.iter().map(|t| t.text.as_ref()).collect::<Vec<_>>(),
            &["A", "<TAB>", "<TAB>", "SEVEN", "!!", "b"]
        );
    }

    #[test]
    fn test_replacement_only_once() {
        let cfg = Config::default();
        let cls = MyClassifier;
        let input = "77";
        let out: Vec<_> = LexerInline::new(&cfg, &cls, input).collect();
        assert_eq!(
            out.iter().map(|t| t.text.as_ref()).collect::<Vec<_>>(),
            &["SEVEN", "SEVEN"]
        );
    }

    #[test]
    fn test_word_token_combination() {
        struct ReplaceLowerS;
        impl Classifier for ReplaceLowerS {
            type Custom = ();

            fn classify(&self, c: char) -> (BaseKind, Option<()>, Option<Cow<'static, str>>) {
                if c == 's' {
                    (BaseKind::Word, None, Some(Cow::Borrowed("S")))
                } else {
                    (classify_base(c), None, None)
                }
            }
        }

        let cfg = Config::default();
        let cls = ReplaceLowerS;
        let input = "sos";
        let out: Vec<_> = LexerInline::new(&cfg, &cls, input).collect();
        assert_eq!(
            out.iter().map(|t| t.text.as_ref()).collect::<Vec<_>>(),
            &["S", "o", "S"]
        );
    }

    #[test]
    fn test_word_token_combination_2() {
        struct ReplaceLowerO;
        impl Classifier for ReplaceLowerO {
            type Custom = ();

            fn classify(&self, c: char) -> (BaseKind, Option<()>, Option<Cow<'static, str>>) {
                if c == 'o' {
                    (BaseKind::Word, None, Some(Cow::Borrowed("O")))
                } else {
                    (classify_base(c), None, None)
                }
            }
        }

        let cfg = Config::default();
        let cls = ReplaceLowerO;
        let input = "sos";
        let out: Vec<_> = LexerInline::new(&cfg, &cls, input).collect();
        assert_eq!(
            out.iter().map(|t| t.text.as_ref()).collect::<Vec<_>>(),
            &["s", "O", "s"]
        );
    }

    #[test]
    fn test_skip_base_kind() {
        let cfg = Config::default().skip_base([BaseKind::Symbol]);
        let cls = DefaultClassifier;
        let input = "abc!";
        let out: Vec<_> = LexerInline::new(&cfg, &cls, input).collect();
        assert_eq!(
            out.iter().map(|t| t.text.as_ref()).collect::<Vec<_>>(),
            &["abc"]
        );
    }
}
