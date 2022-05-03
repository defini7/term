pub mod lex;

pub use lex::lex::TokenKind;
pub use lex::lex::lex;

#[derive(Debug)]
pub struct Node {
    pub children: Vec<Node>,
    pub entry: TokenKind
}

impl Node {
    pub fn new() -> Node {
        Node {
            children: Vec::new(),
            entry: TokenKind::Lparen
        }
    }
}

fn parse_expr(tokens: &Vec<TokenKind>, pos: usize) -> Result<(Node, usize), String> {
    let (node_summand, next_pos) = parse_summand(tokens, pos)?;

    let t = tokens.get(next_pos);

    if let Some(tk) = t {
        let mut new_node = Node::new();

        match tk {
            &TokenKind::Plus => { new_node.entry = TokenKind::Plus; }
            &TokenKind::Minus => { new_node.entry = TokenKind::Minus; }
            &TokenKind::Assign => { new_node.entry = TokenKind::Assign; }
            _ => return Ok((node_summand, next_pos))
        };

        new_node.children.push(node_summand);
        let (rhs, i) = parse_expr(tokens, next_pos + 1)?;
        new_node.children.push(rhs);
        Ok((new_node, i))
    } else {
        Ok((node_summand, next_pos))
    }
}

fn parse_summand(tokens: &Vec<TokenKind>, pos: usize) -> Result<(Node, usize), String> {
    let (node_term, next_pos) = parse_term(tokens, pos)?;

    let t = tokens.get(next_pos);

    let mut new_node = Node::new();

    match t {
        Some(&TokenKind::Asterisk) => { new_node.entry = TokenKind::Asterisk; },
        Some(&TokenKind::ForwardSlash) => { new_node.entry = TokenKind::ForwardSlash; },
        _ => return Ok((node_term, next_pos))
    };

    new_node.children.push(node_term);
    let (rhs, i) = parse_summand(tokens, next_pos + 1)?;
    new_node.children.push(rhs);
    Ok((new_node, i))
}

fn parse_term(tokens: &Vec<TokenKind>, pos: usize) -> Result<(Node, usize), String> {
    let t = tokens.get(pos).ok_or(String::from("Unexpected EOF, expected paren or number"))?;

    match &*t {
        TokenKind::Integer(n) => {
            let mut node = Node::new();
            node.entry = TokenKind::Integer(n.to_owned());
            Ok((node, pos + 1))
        }
        TokenKind::Decimal(n) => {
            let mut node = Node::new();
            node.entry = TokenKind::Decimal(n.to_owned());
            Ok((node, pos + 1))
        }
        TokenKind::QuotedString(s) => {
            let mut node = Node::new();
            node.entry = TokenKind::QuotedString(s.to_owned());
            Ok((node, pos + 1))
        }
        TokenKind::Identifier(name) => {
            let mut node = Node::new();
            node.entry = TokenKind::Identifier(name.to_owned());
            Ok((node, pos + 1))
        }
        TokenKind::Lparen => {
            parse_expr(tokens, pos + 1).and_then(|(node, next_pos)| {
                if let Some(tok) = tokens.get(next_pos) {
                    if let TokenKind::Rparen = tok {
                        return Ok((node, next_pos + 1))
                    } else {
                        Err(format!("Expected ) but found {:?} at {}", tok, next_pos))
                    }
                } else {
                    Err(format!("Expected ) but found {:#?} at {}", tokens.get(next_pos), next_pos))
                }
            })
        }
        TokenKind::Plus => {
            parse_expr(tokens, pos + 1).and_then(|(node, next_pos)| {
                // 0 + node
                let mut unary = Node::new();
                unary.entry = TokenKind::Plus;
                unary.children.push(Node {
                    children: Vec::new(),
                    entry: TokenKind::Integer(0)
                });
                unary.children.push(node);

                return Ok((unary, next_pos))
            })
        }
        TokenKind::Minus => {
            parse_summand(tokens, pos + 1).and_then(|(node, next_pos)| {
                // 0 - node
                let mut unary = Node::new();
                unary.entry = TokenKind::Minus;
                unary.children.push(Node {
                    children: Vec::new(),
                    entry: TokenKind::Integer(0)
                });
                unary.children.push(node);

                return Ok((unary, next_pos))
            })
        }
        _ => {
            Err(format!("Unexpected token {:?} at {}", t, pos))
        }
    }
}

pub fn parse(src: &str) -> Result<Node, String> {
    let tokens = lex(src)?;

    parse_expr(&tokens, 0).and_then(|(n, i)| if i >= tokens.len() {
        Ok(n)
    } else {
        Err(format!("Expected EOF, happened on {:?} at {}", tokens[i], i))
    })
}