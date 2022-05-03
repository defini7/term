mod parser;

use std::collections::HashMap;
use parser::lex::lex::TokenKind;
use parser::Node;

#[derive(Debug, Clone)]
pub enum ValueKind {
    Integer(i64),
    Decimal(f64),
    Str(String),
    Identifier(String),
    None
}

pub struct Stack<T> {
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

struct Variable {
    name: String,
    value: ValueKind
}

impl Variable {
    fn new() -> Variable {
        Variable {
            name: String::new(),
            value: ValueKind::None
        }
    }
}

pub struct State {
    pub stack: Stack<ValueKind>,
    pub variables: HashMap<String, ValueKind>
}

impl State {
    pub fn new() -> State {
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

fn visit_node(node: &Node, state: &mut State) -> ValueKind {
    if node.children.len() == 0 {
        return visit_alone_node(node)
    }

    if let TokenKind::Plus | TokenKind::Minus | TokenKind::Asterisk | TokenKind::ForwardSlash | TokenKind::Assign = node.entry {
        if node.children.len() == 1 {
            visit_unaryop_node(node, state)
        } else if node.children.len() == 2 {
            visit_binop_node(node, state)
        } else {
            panic!("Can't visit unexpected node!");
        }
    } else {
        panic!("Unexpected node type: {:?}", node.entry);
    }
}

fn visit_alone_node(node: &Node) -> ValueKind {
    match &node.entry {
        TokenKind::Integer(n) => ValueKind::Integer(n.to_owned()),
        TokenKind::Decimal(n) => ValueKind::Decimal(n.to_owned()),
        TokenKind::Identifier(n) => ValueKind::Identifier(n.to_string()),
        TokenKind::QuotedString(s) => ValueKind::Str(s.to_string()),
        _ => ValueKind::None
    }
}

fn do_number_node(lhs: &ValueKind, rhs: &ValueKind, op: &TokenKind, state: &mut State) -> ValueKind {
    if let ValueKind::Identifier(n1) = &lhs {
        if let ValueKind::Identifier(n2) = &rhs {
            return do_self(&get_var(n1, state), &get_var(n2, state), op)
        } else {
            return do_self(&get_var(n1, state), rhs, op)
        }
    } else {
        if let ValueKind::Identifier(n) = &rhs {
            return do_self(lhs, &get_var(n, state), op)
        }
    }

    return do_self(lhs, rhs, op);

    fn do_self(lhs: &ValueKind, rhs: &ValueKind, op: &TokenKind) -> ValueKind {
        match &lhs {
            &ValueKind::Decimal(ln) => {
                match rhs {
                    &ValueKind::Decimal(rn) => {
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
                    &ValueKind::Integer(rn) => {
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
                            _ => panic!("Unexpected operation: {:?}", op)
                        }
                    },
                    _ => panic!("Right value should be integer or float: {:?}!", rhs)
                }
            },
            &ValueKind::Integer(ln) => {
                match rhs {
                    &ValueKind::Decimal(rn) => {
                        match op {
                            &TokenKind::Plus => ValueKind::Decimal(ln.to_owned() as f64 + rn),
                            &TokenKind::Minus => ValueKind::Decimal(ln.to_owned() as f64 - rn),
                            &TokenKind::Asterisk => ValueKind::Decimal(ln.to_owned() as f64 * rn),
                            &TokenKind::ForwardSlash => {
                                if rn != 0.0 {
                                    ValueKind::Decimal(ln.to_owned() as f64 / rn)
                                } else {
                                    panic!("Can't divide by zero, {} / {}", ln, rn)
                                }
                            },
                            _ => panic!("Unexpected operation: {:?}", op)
                        }
                    },
                    &ValueKind::Integer(rn) => {
                        match op {
                            &TokenKind::Plus => ValueKind::Integer(ln + rn),
                            &TokenKind::Minus => ValueKind::Integer(ln - rn),
                            &TokenKind::Asterisk => ValueKind::Integer(ln * rn),
                            &TokenKind::ForwardSlash => {
                                if rn != 0 {
                                    if ln % rn != 0 {
                                        ValueKind::Decimal(ln.to_owned() as f64 / rn as f64)
                                    } else {
                                        ValueKind::Integer(ln.to_owned() / rn)
                                    }
                                } else {
                                    panic!("Can't divide by zero: {} / {}", ln, rn)
                                }
                            },
                            _ => panic!("Unexpected operation: {:?}", op)
                        }
                    },
                    _ => panic!("Right value should be integer or float: {:?}!", rhs)
                }
            },
            _ => panic!("Left value should be integer or float: {:?}!", lhs)
        }
    }
}

fn get_var(name: &String, state: &mut State) -> ValueKind {
    let new_value = state.variables.get(name).expect(format!("No such variable: {:?}", name).as_str());

    match &new_value {
        &ValueKind::Decimal(v) => ValueKind::Decimal(v.to_owned()),
        &ValueKind::Integer(v) => ValueKind::Integer(v.to_owned()),
        &ValueKind::Str(v) => ValueKind::Str(v.to_string()),
        _ => ValueKind::None
    }
}

fn do_assign_node(lhs: &ValueKind, rhs: &ValueKind, state: &mut State) -> ValueKind {
    if let ValueKind::Identifier(name) = lhs {
        let mut new_var = Variable::new();
        new_var.name = name.to_string();
        
        match rhs {
            ValueKind::Decimal(n) => { new_var.value = ValueKind::Decimal(n.to_owned()); },
            ValueKind::Integer(n) => { new_var.value = ValueKind::Integer(n.to_owned()); },
            ValueKind::Str(n) => { new_var.value = ValueKind::Str(n.to_string()); },
            ValueKind::Identifier(n) => { new_var.value = get_var(n, state); }
            _ => { new_var.value = ValueKind::None; }
        }

        let v = &new_var.value.to_owned();
        state.variables.insert(new_var.name, new_var.value);

        v.to_owned()
        
    } else {
        panic!("Expected identifier on the left side, but got: {:?}", lhs)
    }
}

fn visit_binop_node(node: &Node, state: &mut State) -> ValueKind {
    let lhs = visit_node(&node.children[0], state);
    let rhs = visit_node(&node.children[1], state);

    if let TokenKind::Assign = node.entry {
        return do_assign_node(&lhs, &rhs, state)
    }

    do_number_node(&lhs, &rhs, &node.entry, state)
}

fn visit_unaryop_node(node: &Node, state: &mut State) -> ValueKind {
    let n = visit_node(node, state);

    if let TokenKind::Minus = node.entry {
        do_number_node(&n, &ValueKind::Integer(-1), &TokenKind::Asterisk, state)
    } else {
        n
    }
}

pub fn interpret(src: &str, main_state: &mut State) -> i32 {
    let tree = parser::parse(src).expect("AST(Abstract Syntax Tree) error");

    visit_node(&tree, main_state);

    0
}