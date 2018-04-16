use std::fmt::Display;

pub trait Join: Iterator {
    fn join(self, seperator: &str) -> String
        where
            Self::Item: Display,
            Self: Sized,
    {
        let mut result = String::new();
        let mut sep = "";

        for t in self {
            result = format!("{}{}{}", result, sep, t);
            sep = seperator;
        }

        result
    }

    fn join_quoted(self, seperator: &str) -> String
        where
            Self::Item: Display,
            Self: Sized,
    {
        let mut result = String::new();
        let mut sep = "";

        for t in self {
            result = format!("{}{}'{}'", result, sep, t);
            sep = seperator;
        }

        result
    }
}

impl<I> Join for I
    where
        I: Iterator,
{}