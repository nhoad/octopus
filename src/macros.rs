#[macro_export]
macro_rules! fatal {
    ( $( $x:expr ),* ) => {
        {
            use std::io::{stderr, Write};
            use std::process;
            let message = format!( $($x,)* );
            writeln!(&mut stderr(), "FATAL: {}", message).unwrap();
            process::exit(1);
        }
    }
}
