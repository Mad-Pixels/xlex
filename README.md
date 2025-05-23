<picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/Mad-Pixels/.github/raw/main/profile/banner.png">
    <source media="(prefers-color-scheme: light)" srcset="https://github.com/Mad-Pixels/.github/raw/main/profile/banner.png">
    <img
        alt="MadPixels"
        src="https://github.com/Mad-Pixels/.github/raw/main/profile/banner.png">
</picture>

# xlex
...

# Contributing
We're open to any new ideas and contributions. We also have some rules and taboos here, so please read this page and our [Code of Conduct](/CODE_OF_CONDUCT.md) carefully.

## I want to report an issue
If you've found an issue and want to report it, please check our [Issues](https://github.com/Mad-Pixels/xlex/issues) page.

```
// Простая обработка строки
let config = Config::default();
let classifier = DefaultClassifier;
let mut lexer = Lexer::from_str(&config, &classifier, "hello world 123");

lexer.process_tokens(|token| {
    println!("{}: {:?}", token.text, token.kind);
    Ok::<(), io::Error>(())
})?;

// Обработка файла
let mut lexer = Lexer::from_file(&config, &classifier, Path::new("large_text.txt"))?;

// Подсчет статистики
let (words, numbers, symbols, spaces) = lexer.count_token_types()?;
println!("Статистика: {} слов, {} чисел", words, numbers);

// Фильтрация и запись в файл
let mut output = File::create("filtered.txt")?;
lexer.filter(
    |_, kind| kind.base == BaseKind::Word,
    |token| {
        writeln!(output, "{}", token.text)?;
        Ok::<(), io::Error>(())
    }
)?;
```