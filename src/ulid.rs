use ulid::Ulid as ActualUlid;

/// Wrapper around UUID that implements the borsh traits
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ulid(ActualUlid);

impl Ulid {
    pub fn new() -> Self {
        Ulid(ActualUlid::new())
    }
}

impl std::ops::Deref for Ulid {
    type Target = ActualUlid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Ulid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for Ulid {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let id: u128 = self.0.into();
        borsh::BorshSerialize::serialize(&id, writer)
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for Ulid {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let id: u128 = borsh::BorshDeserialize::deserialize(buf)?;
        Ok(Ulid(ActualUlid::from(id)))
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshSchema for Ulid {
    fn add_definitions_recursively(
        definitions: &mut std::collections::HashMap<
            borsh::schema::Declaration,
            borsh::schema::Definition,
        >,
    ) {
        <u128 as borsh::BorshSchema>::add_definitions_recursively(definitions)
    }

    fn declaration() -> borsh::schema::Declaration {
        <u128 as borsh::BorshSchema>::declaration()
    }
}
