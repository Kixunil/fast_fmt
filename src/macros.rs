#[macro_export]
macro_rules! fast_fmt_instantiate {
    ($arg:expr) => {
        $crate::Instantiated::new($arg, &$crate::consts::DISPLAY)
    };
    ($arg:expr=>?) => {
        $crate::Instantiated::new($arg, &$crate::consts::DEBUG)
    };
    ($arg:expr=>{$strategy:expr}) => {
        $crate::Instantiated::new($arg, $strategy)
    };
}

#[macro_export]
macro_rules! fwrite {
    ($writer:expr, $($args:expr),*) => {
        {
            use $crate::Fmt;
            let chain = $crate::Empty;
            $( let chain = chain.chain(fast_fmt_instantiate!($args)); )*

            if $crate::Write::uses_size_hint($writer) {
                $crate::Write::size_hint($writer, chain.size_hint(&$crate::consts::DISPLAY));
            }

            chain.fmt($writer, &$crate::consts::DISPLAY)
        }
    };
}
