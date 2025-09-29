use crate::Column;

pub trait Selectable {
    type Row;
    fn collect(&self) -> Vec<&'static str>;
}

impl<T> Selectable for Column<T> {
    type Row = T;
    fn collect(&self) -> Vec<&'static str> {
        vec![self.name]
    }
}

macro_rules! impl_selectable_for_tuples {
    ( $( $Type:ident : $var:ident ),+ ) => {
        impl<$( $Type ),+> Selectable for ( $( $Type, )+ )
        where
            $( $Type: Selectable ),+
        {
            type Row = ( $( <$Type as Selectable>::Row, )+ );

            fn collect(&self) -> Vec<&'static str> {
                let ( $( $var, )+ ) = self;
                let mut out = Vec::new();
                $(
                    out.extend($var.collect());
                )+
                out
            }
        }
    };
}

impl_selectable_for_tuples!(A:a);
impl_selectable_for_tuples!(A:a, B:b);
impl_selectable_for_tuples!(A:a, B:b, C:c);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d, E:e);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d, E:e, F:f);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d, E:e, F:f, G:g);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k, L:l);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k, L:l, M:m);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k, L:l, M:m, N:n);
impl_selectable_for_tuples!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k, L:l, M:m, N:n, O:o);
