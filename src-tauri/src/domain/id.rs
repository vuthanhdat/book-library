use std::fmt;
use uuid::Uuid;

macro_rules! domain_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub(crate) struct $name(Uuid);

        impl $name {
            pub(crate) fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub(crate) fn parse(value: &str) -> Result<Self, uuid::Error> {
                Uuid::parse_str(value).map(Self)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

domain_id!(LibraryId);
domain_id!(BookId);
