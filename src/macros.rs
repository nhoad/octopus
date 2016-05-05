
#[macro_export]
macro_rules! fatal {
    ( $( $x:expr ),* ) => {
        {
            use std::io::stderr;
            writeln!(&mut stderr(), $($x,)*);
            std::process::exit(1);
        }
    }
}
