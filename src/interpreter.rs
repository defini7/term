mod parser;

use std::collections::HashMap;
use parser::lex::lex::TokenKind;
use parser::Node;

#[derive(Debug)]
pub enum ValueKind {
    Integer(i64),
    Decimal(f64),
    Str(String),
    None
}

struct Stack<T> {
    max_size: usize,
    items: Vec<T>
}

impl<T> Stack<T> {
    fn with_capacity(max_size: usize) -> Self {
        Self {
            max_size,
            items: Vec::with_capacity(max_size)
        }
    }

    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    fn push(&mut self, item: T) -> bool {
        if self.items.len() == self.max_size {
            return false
        }
        self.items.push(item);
        true
    }

    fn size(&self) -> usize {
        self.items.len()
    }

    fn peek(&self) -> Option<&T> {
        self.items.last()
    }
}

struct State {
    stack: Stack<ValueKind>,
    variables: HashMap<String, ValueKind>
}

impl State {
    fn new() -> State {
        State {
            stack: Stack::with_capacity(200),
            variables: HashMap::new()
        }
    }

    fn push_stack(&mut self, item: ValueKind) {
        self.stack.push(item);
    }

    fn pop_stack(&mut self) {
        self.stack.pop();
    }

    fn size_stack(&self) -> usize {
        self.stack.size()
    }

    fn peek_stack(&self) -> Option<&ValueKind> {
        self.stack.peek()
    }
}

fn visit_node(node: &Node) -> ValueKind {
    if node.children.len() == 0 {
        return visit_number_node(node)
    }

    if let TokenKind::Plus | TokenKind::Minus | TokenKind::Asterisk | TokenKind::ForwardSlash = node.entry {
        if node.children.len() == 1 {
            visit_unaryop_node(node)
        } else if node.children.len() == 2 {
            visit_binop_node(node)
        } else {
            panic!("Can't visit unexpected node!");
        }
    } else {
        panic!("Unexpected node type: {:?}", node.entry);
    }
}

fn visit_number_node(node: &Node) -> ValueKind {
    match node.entry {
        TokenKind::Integer(n) => ValueKind::Integer(n),
        TokenKind::Decimal(n) => ValueKind::Decimal(n),
        _ => ValueKind::None
    }
}

fn do_number_nodes(lhs: ValueKind, rhs: ValueKind, op: &TokenKind) -> ValueKind {
    match lhs {
        ValueKind::Decimal(ln) => {
            match rhs {
                ValueKind::Decimal(rn) => {
                    match op {
                        &TokenKind::Plus => ValueKind::Decimal(ln + rn),
                        &TokenKind::Minus => ValueKind::Decimal(ln - rn),
                        &TokenKind::Asterisk => ValueKind::Decimal(ln * rn),
                        &TokenKind::ForwardSlash => {
                            if rn != 0.0 {
                                ValueKind::Decimal(ln / rn)
                            } else {
                                panic!("Can't divide by zero, {} / {}", ln, rn)
                            }
                        },
                        _ => panic!("Unexpected operation!")
                    }
                },
                ValueKind::Integer(rn) => {
                    match op {
                        &TokenKind::Plus => ValueKind::Decimal(ln + rn as f64),
                        &TokenKind::Minus => ValueKind::Decimal(ln - rn as f64),
                        &TokenKind::Asterisk => ValueKind::Decimal(ln * rn as f64),
                        &TokenKind::ForwardSlash => {
                            if rn as f64 != 0.0 {
                                ValueKind::Decimal(ln / rn as f64)
                            } else {
                                panic!("Can't divide by zero: {} / {}", ln, rn)
                            }
                        },
                        _ => panic!("Unexpected operation!")
                    }
                },
                _ => panic!("Right value should be integer or float!")
            }
        }
        ValueKind::Integer(ln) => {
            match rhs {
                ValueKind::Decimal(rn) => {
                    match op {
                        &TokenKind::Plus => ValueKind::Decimal(ln as f64 + rn),
                        &TokenKind::Minus => ValueKind::Decimal(ln as f64 - rn),
                        &TokenKind::Asterisk => ValueKind::Decimal(ln as f64 * rn),
                        &TokenKind::ForwardSlash => {
                            if rn != 0.0 {
                                ValueKind::Decimal(ln as f64 / rn)
                            } else {
                                panic!("Can't divide by zero, {} / {}", ln, rn)
                            }
                        },
                        _ => panic!("Unexpected operation!")
                    }
                },
                ValueKind::Integer(rn) => {
                    match op {
                        &TokenKind::Plus => ValueKind::Decimal(ln as f64 + rn as f64),
                        &TokenKind::Minus => ValueKind::Decimal(ln as f64 - rn as f64),
                        &TokenKind::Asterisk => ValueKind::Decimal(ln as f64 * rn as f64),
                        &TokenKind::ForwardSlash => {
                            if rn as f64 != 0.0 {
                                ValueKind::Decimal(ln as f64 / rn as f64)
                            } else {
                                panic!("Can't divide by zero, {} / {}", ln, rn)
                            }
                        },
                        _ => panic!("Unexpected operation!")
                    }
                },
                _ => panic!("Right value should be integer or float!")
            }
        }
        _ => panic!("Left value should be integer or float!")
    }
}

fn visit_binop_node(node: &Node) -> ValueKind {
    let lhs = visit_node(&node.children[0]);
    let rhs = visit_node(&node.children[1]);
    
    do_number_nodes(lhs, rhs, &node.entry)
}

fn visit_unaryop_node(node: &Node) -> ValueKind {
    let n = visit_node(node);

    if let TokenKind::Minus = node.entry {
        do_number_nodes(n, ValueKind::Integer(-1), &TokenKind::Asterisk)
    } else {
        n
    }
}

pub fn interpret(src: &str) -> i32 {
    let tree = parser::parse(src).expect("AST(Abstract Syntax Tree) error");

    let main_state = State::new();

    println!("{:?}", visit_node(&tree));

    0
}