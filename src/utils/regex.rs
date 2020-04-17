macro_rules! regexes {
    ($($name:ident: $value:expr,)*) => {
        use regex::{Error, Regex};

        #[derive(Clone)]
        pub struct Regexes {
            $(
                pub $name: Regex,
            )*
        }

        impl Regexes {
            pub fn load() -> Result<Regexes, Error> {
                Ok(Regexes {
                    $(
                        $name: Regex::new($value)?,
                    )*
                })
            }
        }
    }
}

regexes! {
    email: r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$",
    password: r"^.{8,32}$",
    username: r"^[a-zA-Z0-9]{3,20}$",
}
