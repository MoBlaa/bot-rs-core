use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn rand_alphanumeric(size: usize) -> String {
    thread_rng().sample_iter(&Alphanumeric)
        .take(size).collect()
}
