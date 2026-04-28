#[macro_export]
macro_rules! define_ro_struct {
    {
        [no_brw]
        struct $name:ident {
            $($field:ident: $field_type:ty,)*
        }
    } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            $($field:$field_type,)*
        }

        $crate::define_ro_struct!(@impl_methods $name { $($field: $field_type,)* });
    };

    {
        struct $name:ident {
            $($field:ident: $field_type:ty,)*
        }
    } => {
        #[derive(::binrw::BinRead, ::binrw::BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            $($field:$field_type,)*
        }

        $crate::define_ro_struct!(@impl_methods $name { $($field: $field_type,)* });
    };

    {
        @impl_methods $name:ident { $($field:ident: $field_type:ty,)* }
    } => {
        impl $name {
            pub const fn new($($field: $field_type,)*) -> Self {
                Self { $($field,)* }
            }

            $(
                pub const fn $field(&self) -> $field_type {
                    self.$field
                }
            )*
        }
    };
}
