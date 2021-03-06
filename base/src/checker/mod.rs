pub mod node;

use super::ast::{Expression, Grammar};
pub use node::Node;

fn check_expr<'a>(
    input: &'a str,
    expression: &'a Expression,
    grammar: &'a Grammar,
) -> Result<(&'a str, String, Option<Vec<Node>>), ()> {
    match &expression {
        Expression::Alternative {
            first: box first,
            second: box second,
            rest,
        } => {
            match check_expr(input, first, grammar) {
                Ok((rest, output, node)) => return Ok((rest, output, node)),
                Err(()) => {}
            }
            match check_expr(input, second, grammar) {
                Ok((rest, output, node)) => return Ok((rest, output, node)),
                Err(()) => {}
            }
            for expression in rest.iter()
            {
                match check_expr(input, expression, grammar) {
                    Ok((rest, output, node)) => return Ok((rest, output, node)),
                    Err(()) => {}
                }
            }
            Err(())
        }
        Expression::Sequence {
            first: box first,
            second: box second,
            rest,
        } => {
            let mut input = input;
            let mut output = String::new();
            let mut nodes = Vec::new();
            let (inp, output_first, node_first) = check_expr(input, first, grammar)?;
            input = inp;
            output.push_str(&output_first);
            if let Some(n) = node_first {
                nodes.extend(n);
            }
            let (inp, output_second, node_second) = check_expr(input, second, grammar)?;
            input = inp;
            output.push_str(&output_second);
            if let Some(n) = node_second {
                nodes.extend(n);
            }
            for expression in rest.iter()
            {
                let (inp, output_expr, node_expr) = check_expr(input, expression, grammar)?;
                input = inp;
                output.push_str(&output_expr);
                if let Some(n) = node_expr {
                    nodes.extend(n);
                }
            }
            Ok((input, output, Some(nodes)))
        }
        Expression::Optional(box inner) => {
            match check_expr(input, inner, grammar) {
                Ok((rest, output, nodes)) => Ok((rest, output, nodes)),
                Err(()) => Ok((input, String::new(), None)),
            }
        }
        Expression::Repeated(box inner) => {
            let mut input = input;
            let mut output = String::new();
            let mut nodes = Vec::new();
            loop {
                match check_expr(input, inner, grammar) {
                    Ok((rest, out, node_expr)) => {
                        input = rest;
                        output.push_str(&out);
                        if let Some(n) = node_expr {
                            nodes.extend(n);
                        }
                    }
                    Err(()) => return Ok((input, output, Some(nodes))),
                }
            }
        }
        Expression::Factor {
            count,
            primary: box primary,
        } => {
            let mut input = input;
            let mut output = String::new();
            let mut nodes = Vec::new();
            for _ in 0..*count {
                match check_expr(input, primary, grammar) {
                    Ok((rest, out, node_expr)) => {
                        input = rest;
                        output.push_str(&out);
                        if let Some(n) = node_expr {
                            nodes.extend(n);
                        }
                    }
                    Err(()) => return Err(()),
                }
            }
            Ok((input, output, Some(nodes)))
        }
        Expression::Exception {
            subject: box subject,
            restriction: box restriction,
        } => {
            let (input, read_chars, node) = check_expr(input, subject, grammar)?;
            match check_expr(&read_chars, restriction, grammar) {
                Ok((_, matched_chars, _)) if matched_chars == read_chars => Err(()),
                _ => Ok((input, read_chars, node)),
            }
        }
        Expression::Nonterminal(identifier) => {
            let (rest, out, node) = check_prod(input, grammar, identifier)?;
            return Ok((rest, out, Some(vec![node])));
        },
        Expression::Terminal(content) => {
            if input.starts_with(content) {
                let len = content.len();
                Ok((&input[len..], content.clone(), Some(vec![Node::Terminal(content.clone())])))
            } else {
                Err(())
            }
        }
        Expression::Special(_) => Err(()),
        Expression::Empty => Ok((input, String::new(), None)),
    }
}

pub(super) fn check_prod<'a>(
    input: &'a str,
    grammar: &'a Grammar,
    initial_rule: &'a str,
) -> Result<(&'a str, String, Node), ()> {
    let expression = grammar.get(initial_rule).unwrap();
    let (rest, out, nodes) = check_expr(input, expression, grammar)?;
    return Ok((rest, out, Node::Nonterminal(initial_rule.to_owned(), nodes.unwrap_or(Vec::new()))));
}

pub fn check<'a>(
    input: &'a str,
    grammar: &'a Grammar,
    initial_rule: &'a str,
) -> Option<Node> {
    match check_prod(input, grammar, initial_rule) {
        Ok((input, _, node)) if input.is_empty() => Some(node),
        _ => None,
    }
}
