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
