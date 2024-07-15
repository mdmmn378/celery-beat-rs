use chrono::prelude::*;
use cron_parser;
use rand;
use rand::Rng;

pub fn get_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let characters: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    let mut result = String::new();
    for _ in 0..length {
        let index = rng.gen_range(0..characters.len());
        result.push(characters[index]);
    }
    result
}

pub fn get_random_origin() -> String {
    format!("{}@localhost", get_random_string(16))
}

pub fn next_call(previouse: DateTime<Utc>, expr: String) -> DateTime<Utc> {
    let next = cron_parser::parse(&expr, &previouse).unwrap();
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_call() {
        let expr = "*/5 * * * *".to_string();
        let now = Utc::now();
        let next = next_call(now, expr);
        assert!(next > now);
    }
}
