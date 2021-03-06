pub mod lex {
    #[derive(Debug)]
    pub enum TokenKind {
        Integer(i64),
        Decimal(f64),
        Identifier(String),
        QuotedString(String),
        Boolean(bool),
        Plus,
        Minus,
        Asterisk,
        ForwardSlash,
        Dot,
        Assign,
        Lparen,
        Rparen,
        If,
        While,
        NewLine,
        Less,
        Greater,
        IsEquals,
        NotEquals,
        Not
    }

    impl From<i64> for TokenKind {
        fn from(other: i64) -> TokenKind {
            TokenKind::Integer(other)
        }
    }

    impl From<f64> for TokenKind {
        fn from(other: f64) -> TokenKind {
            TokenKind::Decimal(other)
        }
    }

    impl<'a> From<&'a str> for TokenKind {
        fn from(other: &'a str) -> TokenKind {
            TokenKind::Identifier(other.to_string())
        }
    }

    struct Lexer<'a> {
        current: usize,
        remaining: &'a str
    }

    impl<'a> Lexer<'a> {
        fn new(src: &str) -> Lexer {
            Lexer {
                current: 0,
                remaining: src
            }
        }

        fn next_token(&mut self) -> Result<Option<(TokenKind, usize, usize)>, i32> {
            self.skip_whitespace();

            if self.remaining.is_empty() {
                Ok(None)
            } else {
                let start = self.current;
                let tok = self._next_token().expect("Could not read the next token.");
                Ok(Some((tok, start, self.current)))
            }
        }

        fn skip_whitespace(&mut self) {
            self.chomp(skip(self.remaining))
        }

        fn _next_token(&mut self) -> Result<TokenKind, usize> {
            let (tok, bytes_read) = lex_one(self.remaining)?;
            self.chomp(bytes_read);

            Ok(tok)
        }

        fn chomp(&mut self, num_bytes: usize) {
            self.remaining = &self.remaining[num_bytes..];
            self.current += num_bytes;
        }
    }

    fn take_while<F>(data: &str, mut pred: F) -> Result<(&str, usize), usize> where F: FnMut(char) -> bool {
        let mut current: usize = 0;
        
        for c in data.chars() {
            if !pred(c) {
                break;
            }

            current += c.len_utf8();
        }

        if current == 0 {
            Err(0)
        } else {
            Ok((&data[..current], current))
        }
    }

    fn lex_ident(data: &str) -> Result<(TokenKind, usize), usize> {
        match data.chars().next() {
            Some(c) if c.is_digit(10) => panic!("Identifiers can't start with a number"),
            None => panic!("Unexpected EOF"),
            _ => {}
        }

        let (got, bytes_read) = take_while(data, |c| c == '_' || c.is_alphanumeric())?;

        let tok = TokenKind::Identifier(got.to_string());
        Ok((tok, bytes_read))
    }

    fn lex_number(data: &str) -> Result<(TokenKind, usize), usize> {
        let mut was_dot = false;

        let (decimal, bytes_read) = take_while(data, |c| {
            if c.is_digit(10) {
                true
            } else if c == '.' {
                if !was_dot {
                    was_dot = true;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        })?;

        if was_dot {
            let n: f64 = decimal.parse().expect("Can not parse float number.");
            Ok((TokenKind::Decimal(n), bytes_read))
        } else {
            let n: i64 = decimal.parse().expect("Can not parse float number.");
            Ok((TokenKind::Integer(n), bytes_read))
        }
    }

    fn lex_string(data: &str) -> Result<(TokenKind, usize), usize> {
        let mut was_first = false;

        let (string, bytes_read) = take_while(data, |c| {
            if c == '"' && was_first {
                false
            } else {
                was_first = true;
                true
            }
        })?;

        let mut result = String::from(string);
        result.remove(0);

        Ok((TokenKind::QuotedString(result), bytes_read + 1))
    }

    fn lex_equals(data: String) -> (TokenKind, usize) {
        if data.as_bytes()[1] as char == '=' {
            (TokenKind::IsEquals, 2)
        } else {
            (TokenKind::Assign, 1)
        }
    }

    fn lex_not(data: String) -> (TokenKind, usize) {
        if data.as_bytes()[1] as char == '=' {
            (TokenKind::NotEquals, 2)
        } else {
            (TokenKind::Not, 1)
        }
    }

    fn skip_whitespace(data: &str) -> usize {
        match take_while(data, |c| c.is_whitespace()) {
            Ok((_, bytes_skipped)) => bytes_skipped,
            _ => 0
        }
    }

    fn skip_until<'a>(mut src: &'a str, pattern: &str) -> &'a str {
        while !src.is_empty() && !src.starts_with(pattern) {
            let next_char_size = src.chars().next().expect("String can't be empty").len_utf8();
            src = &src[next_char_size..];
        }

        &src[pattern.len()..]
    }

    fn skip_comments(src: &str) -> usize {
        let pairs = [("//", "\n"), ("/*", "*/")];

        for &(pattern, matcher) in &pairs {
            if src.starts_with(pattern) {
                let leftovers = skip_until(src, matcher);
                return src.len() - leftovers.len();
            }
        }

        0
    }

    fn skip(src: &str) -> usize {
        let mut remaining = src;

        loop {
            let ws = skip_whitespace(remaining);
            remaining = &remaining[ws..];
            let comments = skip_comments(remaining);
            remaining = &remaining[comments..];

            if ws + comments == 0 {
                return src.len() - remaining.len();
            }
        }
    }

    fn lex_one(data: &str) -> Result<(TokenKind, usize), usize> {
        let next = match data.chars().next() {
            Some(c) => c,
            None => panic!("Unexpected EOF")
        };

        let (tok, length) = match next {
            '.' => (TokenKind::Dot, 1),
            '=' => lex_equals(data.to_string()),
            '!' => lex_not(data.to_string()),
            '<' => (TokenKind::Less, 1),
            '>' => (TokenKind::Greater, 1),
            '+' => (TokenKind::Plus, 1),
            '-' => (TokenKind::Minus, 1),
            '*' => (TokenKind::Asterisk, 1),
            '/' => (TokenKind::ForwardSlash, 1),
            '(' => (TokenKind::Lparen, 1),
            ')' => (TokenKind::Rparen, 1),
            '\n' => (TokenKind::NewLine, 1),
            '"' => lex_string(data).expect("Couldn't lex a string"),
            '0'..='9' => lex_number(data).expect("Couldn't lex a number"),
            c @ '_' | c if c.is_alphabetic() => {
                let ident = lex_ident(data).expect("Couldn't lex an identifier");

                if let TokenKind::Identifier(i) = &ident.0 {
                    match i.as_str() {
                        "if" => (TokenKind::If, 2),
                        "while" => (TokenKind::While, 5),
                        "true" => (TokenKind::Boolean(true), 4),
                        "false" => (TokenKind::Boolean(false), 5),
                        _ => ident
                    }
                } else {
                    ident
                }
            }
            other => panic!("Unknown character '{}'", other)
        };

        Ok((tok, length))
    }

    pub fn lex(src: &str) -> Result<Vec<TokenKind>, String> {
        let mut lexer = Lexer::new(src);
        let mut tokens = Vec::new();

        while let Some(tok) = lexer.next_token().expect("Can not get next token!") {
            tokens.push(tok.0);
        }

        Ok(tokens)
    }
}