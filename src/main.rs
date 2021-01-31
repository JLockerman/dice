
extern crate pest;

#[macro_use]
extern crate pest_derive;

use std::io;

use pest::{
    Parser,
    prec_climber::{PrecClimber, Operator, Assoc},
    iterators::{
        Pair,
        Pairs,
    },
};

use once_cell::sync::Lazy;

use rand::{
    thread_rng,
    distributions::{
        Distribution,
        Uniform,
    },
};

use rustyline::{
    Editor,
    error::ReadlineError,
};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct DiceParser;

static PREC_CLIMBER: Lazy<PrecClimber<Rule>> = Lazy::new(||{
    use Rule::*;
    use Assoc::*;

    PrecClimber::new(vec![
        Operator::new(add, Left) | Operator::new(subtract, Left),
        Operator::new(multiply, Left) | Operator::new(divide, Left),
        Operator::new(power, Right)
    ])
});

fn main() -> io::Result<()> {
    let mut ed = Editor::<()>::new();
    loop {
        let line = ed.readline("> ");
        let line = match line {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(ReadlineError::Io(err)) => return Err(err),
            Err(ReadlineError::Errno(err)) => {
                println!("Error: {}", err);
                break
            }
            Err(e) => panic!(e),
        };
        match DiceParser::parse(Rule::calculation, &line) {
            Err(e) => println!("{}", e),
            Ok(parsed) => {
                let val = eval(parsed);
                ed.add_history_entry(&line);
                println!("{}", val);
            }
        }
    }
    Ok(())
}

fn eval(expression: Pairs<Rule>) -> f64 {
    PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::die => {
                let mut s = pair.as_str().split('d');
                let first = s.next().unwrap().parse::<u64>().unwrap();
                match s.next() {
                    None => first as f64,
                    Some(second) => {
                        let second: u64 = second.parse().unwrap();
                        let mut sum = 0u64;
                        for _ in 0..first {
                            sum += Uniform::from(1..=second).sample(&mut thread_rng())
                        }
                        sum as f64
                    }
                }
            }
            Rule::num => {
                pair.as_str().parse::<f64>().unwrap()
            }
            Rule::expr => eval(pair.into_inner()),
            _ => unreachable!(),
        },
        |lhs: f64, op: Pair<Rule>, rhs: f64| match op.as_rule() {
            Rule::add      => lhs + rhs,
            Rule::subtract => lhs - rhs,
            Rule::multiply => lhs * rhs,
            Rule::divide   => lhs / rhs,
            Rule::power    => lhs.powf(rhs),
            _ => unreachable!(),
        },
    )
}

