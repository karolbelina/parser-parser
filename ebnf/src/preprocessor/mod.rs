pub mod error;
#[cfg(test)]
mod tests;

use super::parser::{Expression, Grammar, Production};
use super::span::{Span, Spanned, Spanning};
use error::Error;
use std::collections::HashMap;

fn is_failing(
    expression: &Spanned<Expression>,
    rules: &HashMap<String, &Spanned<Expression>>,
    trace: &mut Vec<String>,
) -> bool {
    match &expression.node {
        Expression::Alternative {
            first: box first,
            second: box second,
            rest,
        } => {
            if !is_failing(first, rules, trace) {
                return false;
            }
            if !is_failing(second, rules, trace) {
                return false;
            }
            for expression in rest.iter() {
                if !is_failing(expression, rules, trace) {
                    return false;
                }
            }
            true
        }
        Expression::Sequence {
            first: box first,
            second: box second,
            rest,
        } => {
            if is_failing(first, rules, trace) {
                return true;
            }
            if is_failing(second, rules, trace) {
                return true;
            }
            for expression in rest.iter() {
                if is_failing(expression, rules, trace) {
                    return true;
                }
            }
            false
        }
        Expression::Optional(_) => true,
        Expression::Repeated(_) => true,
        Expression::Factor {
            count: Spanned { node: count, .. },
            primary: box primary,
        } => {
            if *count == 0 {
                return true;
            }
            is_failing(primary, rules, trace)
        }
        Expression::Exception {
            subject: box subject,
            restriction: box restriction,
        } => {
            is_failing(subject, rules, trace) && is_failing(restriction, rules, trace)
        }
        Expression::Nonterminal(identifier) => {
            if !trace.contains(identifier) {
                if let Some(expression) = rules.get(identifier) {
                    trace.push(identifier.clone());
                    let result = is_failing(&expression, rules, trace);
                    trace.pop().unwrap();

                    return result;
                }
            }

            false
        }
        Expression::Terminal(_) => false,
        Expression::Special(_) => false,
        Expression::Empty => true,
    }
}

fn check_expr(
    expression: &Spanned<Expression>,
    rules: &HashMap<String, &Spanned<Expression>>,
    trace: &mut Vec<String>,
) -> Result<(), Spanned<Error>> {
    match &expression.node {
        Expression::Alternative {
            first: box first,
            second: box second,
            rest,
        } => {
            check_expr(first, rules, trace)?;
            check_expr(second, rules, trace)?;
            for expression in rest.iter() {
                check_expr(expression, rules, trace)?;
            }
            Ok(())
        }
        Expression::Sequence {
            first: box first,
            second: box second,
            rest,
        } => {
            if !is_failing(first, rules, &mut vec![trace.last().unwrap().clone()]) {
                return check_expr(first, rules, trace);
            }
            if rest.is_empty() {
                check_expr(second, rules, trace)
            } else {
                if !is_failing(second, rules, &mut vec![trace.last().unwrap().clone()]) {
                    return check_expr(second, rules, trace);
                }
                for expression in rest[..rest.len() - 1].iter() {
                    if !is_failing(expression, rules, &mut vec![trace.last().unwrap().clone()]) {
                        return check_expr(expression, rules, trace);
                    }
                }
                return check_expr(&rest.last().unwrap(), rules, trace);
            }
        }
        Expression::Optional(box inner) => {
            check_expr(inner, rules, trace)
        }
        Expression::Repeated(box inner) => {
            check_expr(inner, rules, trace)
        }
        Expression::Factor {
            count: Spanned { node: count, .. },
            primary: box primary,
        } => {
            if *count == 0 {
                return Ok(());
            }
            check_expr(primary, rules, trace)
        }
        Expression::Exception {
            subject: box subject,
            restriction: box restriction,
        } => {
            check_expr(subject, rules, trace)?;
            check_expr(restriction, rules, trace)?;
            Ok(())
        }
        Expression::Nonterminal(identifier) => {
            if &trace[0] == identifier {
                trace.push(identifier.clone());
                return Err(Error::LeftRecursion(trace.clone()).spanning(expression.span));
            }

            if !trace.contains(identifier) {
                if let Some(node) = rules.get(identifier) {
                    trace.push(identifier.clone());
                    let result = check_expr(&node, rules, trace);
                    trace.pop().unwrap();

                    return result;
                }
            }

            Ok(())
        }
        Expression::Terminal(_) => Ok(()),
        Expression::Special(_) => Ok(()),
        Expression::Empty => Ok(()),
    }
}

fn get_rule_hash_map(rules: &[Spanned<Production>]) -> HashMap<String, &Spanned<Expression>> {
    rules
        .iter()
        .map(
            |Spanned { node: rule, .. }| -> (String, &Spanned<Expression>) {
                (rule.lhs.node.clone(), &rule.rhs)
            },
        )
        .collect()
}

fn validate_left_recursion(
    Spanned { node: grammar, .. }: &Spanned<Grammar>,
) -> Result<(), Spanned<Error>> {
    let rules = get_rule_hash_map(&grammar.productions);

    for (name, expression) in &rules {
        let name = name.clone();
        check_expr(expression, &rules, &mut vec![name])?;
    }

    Ok(())
}

fn check_nonterminals(
    expression: &Spanned<Expression>,
    rules: &[String],
) -> Result<(), Spanned<Error>> {
    match &expression.node {
        Expression::Alternative {
            first: box first,
            second: box second,
            rest,
        } => {
            check_nonterminals(first, rules)?;
            check_nonterminals(second, rules)?;
            for expression in rest.iter() {
                check_nonterminals(expression, rules)?;
            }
            Ok(())
        }
        Expression::Sequence {
            first: box first,
            second: box second,
            rest,
        } => {
            check_nonterminals(first, rules)?;
            check_nonterminals(second, rules)?;
            for expression in rest.iter() {
                check_nonterminals(expression, rules)?;
            }
            Ok(())
        }
        Expression::Optional(box inner) => {
            check_nonterminals(inner, rules)
        }
        Expression::Repeated(box inner) => {
            check_nonterminals(inner, rules)
        }
        Expression::Factor {
            primary: box primary,
            ..
        } => {
            check_nonterminals(primary, rules)
        }
        Expression::Exception {
            subject: box subject,
            restriction: box restriction,
        } => {
            check_nonterminals(subject, rules)?;
            check_nonterminals(restriction, rules)?;
            Ok(())
        }
        Expression::Nonterminal(identifier) => {
            if !rules.contains(identifier) {
                Err(Error::UndefinedRule(identifier.clone()).spanning(expression.span))
            } else {
                Ok(())
            }
        }
        Expression::Terminal(_) => Ok(()),
        Expression::Special(_) => Ok(()),
        Expression::Empty => Ok(()),
    }
}

fn get_rule_identifiers(rules: &[Spanned<Production>]) -> Vec<String> {
    rules
        .iter()
        .map(|Spanned { node: rule, .. }| -> String { rule.lhs.node.clone() })
        .collect()
}

fn validate_nonterminals(
    Spanned { node: grammar, .. }: &Spanned<Grammar>,
) -> Result<(), Spanned<Error>> {
    let rules = get_rule_identifiers(&grammar.productions);

    for Spanned {
        node: Production {
            rhs: expression, ..
        },
        ..
    } in grammar.productions.iter()
    {
        check_nonterminals(&expression, &rules)?;
    }

    Ok(())
}

fn validate_repetitions(
    Spanned { node: grammar, .. }: &Spanned<Grammar>,
) -> Result<(), Spanned<Error>> {
    let rules = get_rule_identifiers(&grammar.productions);

    for spanned_production in grammar.productions.iter() {
        if rules.iter().filter(|&n| *n == spanned_production.node.lhs.node).count() != 1 {
            return Err(Error::MultipleDefinitions(spanned_production.node.lhs.node.to_owned()).spanning(spanned_production.node.lhs.span))
        }
    }

    for Spanned {
        node: Production {
            rhs: expression, ..
        },
        ..
    } in grammar.productions.iter()
    {
        check_nonterminals(&expression, &rules)?;
    }

    Ok(())
}

pub(super) fn preprocess(
    spanned_grammar: Spanned<Grammar>,
) -> Result<Spanned<Grammar>, Spanned<Error>> {
    validate_repetitions(&spanned_grammar)?;
    validate_nonterminals(&spanned_grammar)?;
    validate_left_recursion(&spanned_grammar)?;
    Ok(spanned_grammar)
}
