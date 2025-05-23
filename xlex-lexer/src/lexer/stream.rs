use super::classifier::Classifier;
use super::config::Config;
use super::token::{BaseKind, Token, TokenKind};

use std::borrow::Cow;
use std::io::BufRead;
use std::{hash::Hash, str};

pub struct LexerStream<'a, TK, CL, BR>
where
    BR: BufRead,
    TK: Copy + Eq + Hash,
    CL: Classifier<Custom = TK>,
{
    config: &'a Config<TK>,
    classifier: &'a CL,

    reader: BR,
    pos: usize,
}

impl<'a, TK, CL, BR> LexerStream<'a, TK, CL, BR>
where
    BR: BufRead,
    TK: Copy + Eq + Hash,
    CL: Classifier<Custom = TK>,
{
    #[inline]
    pub fn new(config: &'a Config<TK>, classifier: &'a CL, reader: BR) -> Self {
        LexerStream {
            config,
            classifier,
            reader,
            pos: 0,
        }
    }
}

impl<'a, TK, CL, BR> Iterator for LexerStream<'a, TK, CL, BR>
where
    BR: BufRead,
    TK: Copy + Eq + Hash,
    CL: Classifier<Custom = TK>,
{
    type Item = Token<'static, TK>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let buf = match self.reader.fill_buf() {
                Ok(b) if !b.is_empty() => b,
                _ => return None,
            };

            let position = self.pos;
            let char_len = utf_char_len(buf[0]);

            if char_len > buf.len() {
                self.reader.consume(1);
                self.pos += 1;
                continue;
            }
            
            let ch = match str::from_utf8(&buf[..char_len])
                .ok()
                .and_then(|s| s.chars().next())
            {
                Some(c) => c,
                None => {
                    self.reader.consume(char_len);
                    self.pos += char_len;
                    continue;
                }
            };

            let (bk, ck, repl) = self.classifier.classify(ch);
            let kind = TokenKind::new(bk, ck);

            if let Some(repl) = repl {
                self.reader.consume(char_len);
                self.pos += char_len;
                
                return Some(Token {
                    text: Cow::Owned(repl.into_owned()),
                    kind,
                    start: position,
                });
            }
            if self.config.should_skip(bk, ck) {
                let token_len = scan_token(buf, char_len, bk, ck, self.classifier);
                self.reader.consume(token_len);
                self.pos += token_len;
                continue;
            }

            let token_len;
            if (bk == BaseKind::Symbol && self.config.group_symbols) || bk != BaseKind::Symbol {
                token_len = scan_token(buf, char_len, bk, ck, self.classifier);
            } else {
                token_len = char_len;
            }

            let text = match str::from_utf8(&buf[..token_len]) {
                Ok(s) => Cow::Owned(s.to_string()),
                Err(_) => {
                    self.reader.consume(token_len);
                    self.pos += token_len;
                    continue;
                }
            };

            self.reader.consume(token_len);
            self.pos += token_len;
            return Some(Token {
                text,
                kind,
                start: position,
            });
        }
    }
}

#[inline]
fn utf_char_len(f: u8) -> usize {
    if f < 0x80 {
        1
    } else if f < 0xE0 {
        2
    } else if f < 0xF0 {
        3
    } else {
        4
    }
}

#[inline]
fn scan_token<TK, CL>(
    buf: &[u8],
    mut token_len: usize,
    base: BaseKind,
    custom: Option<TK>,
    classifier: &CL,
) -> usize
where
    TK: Copy + Eq + Hash,
    CL: Classifier<Custom = TK>,
{
    while token_len < buf.len() {
        let len = utf_char_len(buf[token_len]);
        if token_len + len > buf.len() {
            break;
        }

        let ch = match str::from_utf8(&buf[token_len..token_len + len])
            .ok()
            .and_then(|s| s.chars().next())
        {
            Some(c) => c,
            None => break,
        };

        let (bk, ck, next_repl) = classifier.classify(ch);
        if next_repl.is_some() || bk != base || ck != custom {
            break;
        }
        
        token_len += len;
    }
    
    token_len
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::classifier::{Classifier, DefaultClassifier};
    use crate::lexer::{
        config::Config,
        token::{classify_base, BaseKind},
    };
    use std::borrow::Cow;
    use std::io::Cursor;

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

    fn run(
        input: &str,
        config: Config<MyCustom>,
        classifier: &impl Classifier<Custom = MyCustom>,
    ) -> Vec<String> {
        let reader = Cursor::new(input.as_bytes());
        LexerStream::new(&config, classifier, reader)
            .map(|t| t.text.into_owned())
            .collect()
    }

    #[test]
    fn test_grouping_symbols_default() {
        let cfg = Config::default().with_grouped_symbols();
        let cls = DefaultClassifier;
        let reader = Cursor::new("a!!b");
        let tokens: Vec<_> = LexerStream::new(&cfg, &cls, reader)
            .map(|t| t.text.into_owned())
            .collect();
        assert_eq!(tokens, ["a", "!!", "b"]);
    }

    #[test]
    fn test_skipping_tab_custom() {
        let cfg = Config::default()
            .skip_custom([MyCustom::Tab])
            .with_grouped_symbols();
        let cls = MyClassifier;
        let tokens = run("a\t\t7!!b", cfg, &cls);
        assert_eq!(tokens, ["A", "<TAB>", "<TAB>", "SEVEN", "!!", "b"]);
    }

    #[test]
    fn test_replacement_only_once() {
        let cfg = Config::default();
        let cls = MyClassifier;
        let tokens = run("77", cfg, &cls);
        assert_eq!(tokens, ["SEVEN", "SEVEN"]);
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
        let reader = Cursor::new("sos");
        let out: Vec<_> = LexerStream::new(&cfg, &cls, reader)
            .map(|t| t.text.into_owned())
            .collect();
        assert_eq!(out, ["S", "o", "S"]);
    }

    #[test]
    fn test_skip_base_kind() {
        let cfg = Config::default().skip_base([BaseKind::Symbol]);
        let cls = DefaultClassifier;
        let reader = Cursor::new("abc!");
        let out: Vec<_> = LexerStream::new(&cfg, &cls, reader)
            .map(|t| t.text.into_owned())
            .collect();
        assert_eq!(out, ["abc"]);
    }
}
