
extern crate pest;

#[macro_use]
extern crate pest_derive;

use std::io::{
    self,
    stdin,
    stdout,
    BufRead,
    Write,
};

use pest::{
    Parser,
    prec_climber::{PrecClimber, Operator, Assoc},
    iterators::{
        Pair,
        Pairs,
    },
};

use rand::{
    thread_rng,
    distributions::{
        Distribution,
        Uniform,
    },
};

use once_cell::sync::Lazy;

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
    let input = stdin();
    let input = input.lock();
    print!("> ");
    let _ = stdout().flush();
    for line in input.lines() {
        let line = line?;
        match DiceParser::parse(Rule::calculation, &line) {
            Err(e) => {
                println!("{}", e);
                print!("> ");
                let _ = stdout().flush();
            }
            Ok(parsed) => {
                let val = eval(parsed);
                print!("{}\n> ", val);
                let _ = stdout().flush();
            }
        }
    }
    Ok(())
}

fn eval(expression: Pairs<Rule>) -> u32 {
    PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::die => {
                let mut s = pair.as_str().split('d');
                let first = s.next().unwrap().parse::<f64>().unwrap() as u32;
                match s.next() {
                    None => first,
                    Some(second) => {
                        let second = second.parse::<f64>().unwrap() as u32;
                        let mut sum = 0;
                        for _ in 0..first {
                            sum += Uniform::from(0..second).sample(&mut thread_rng())
                        }
                        sum
                    }
                }
            }
            Rule::num => {
                pair.as_str().parse::<f64>().unwrap() as u32
            }
            Rule::expr => eval(pair.into_inner()),
            _ => unreachable!(),
        },
        |lhs: u32, op: Pair<Rule>, rhs: u32| match op.as_rule() {
            Rule::add      => lhs + rhs,
            Rule::subtract => lhs - rhs,
            Rule::multiply => lhs * rhs,
            Rule::divide   => lhs / rhs,
            Rule::power    => (lhs as f64).powf(rhs as f64) as u32,
            _ => unreachable!(),
        },
    )
}
