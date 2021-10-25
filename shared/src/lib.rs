use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum Command {
    Restart,
    Dummy
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
