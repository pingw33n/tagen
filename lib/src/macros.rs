macro_rules! impl_as_into {
    ($enum:ident: $($into_name:ident, $as_name:ident, $as_mut_name:ident <= $vari:ident ( $type:ty ),)+) => {
        $(
            impl $enum {
                pub fn $into_name(self) -> Option<$type> {
                    if let $enum :: $vari ( v ) = self {
                        Some(v)
                    } else {
                        None
                    }
                }

                pub fn $as_name(&self) -> Option<&$type> {
                    if let $enum :: $vari ( ref v ) = self {
                        Some(v)
                    } else {
                        None
                    }
                }

                pub fn $as_mut_name(&mut self) -> Option<&mut $type> {
                    if let $enum :: $vari ( ref mut v ) = self {
                        Some(v)
                    } else {
                        None
                    }
                }
            }
        )+
    };
}