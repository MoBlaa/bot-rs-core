use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

/// Create a new random string with `size` Alphanumeric characters.
pub fn rand_alphanumeric(size: usize) -> String {
    thread_rng().sample_iter(&Alphanumeric).take(size).collect()
}
