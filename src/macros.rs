#[macro_export]
macro_rules! verbose_try {
    ($e : expr, $loc : ident) => {
        match $e {
            Ok(x) => x,
            Err(err) => {
                use log::error;

                error!(target: stringify!($loc), "An error has been encoutered at {}:{}\n{}", file!(), line!(), err);

                return Err(err.into())
            },
        }
    }
}
