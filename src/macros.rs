
#[macro_export]
macro_rules! fatal {
    ( $( $x:expr ),* ) => {
        {
            use std::io::stderr;
            let message = format!( $($x,)* );
            writeln!(&mut stderr(), "FATAL: {}", message).unwrap();
            std::process::exit(1);
        }
    }
}
