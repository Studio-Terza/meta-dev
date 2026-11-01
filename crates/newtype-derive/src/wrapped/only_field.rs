use syn::{Data, DeriveInput, Error, Fields, Member, Type};

pub struct OnlyField {
    member: Member,
    inner: Type,
}

impl OnlyField {
    const COUNT: &str = "usize";
    const REFUSED: &str = "a newtype is a struct with exactly one field";

    pub fn found_in(input: &DeriveInput) -> syn::Result<OnlyField> {
        match &input.data {
            Data::Struct(data) => Self::among(&data.fields),
            Data::Enum(_) | Data::Union(_) => None,
        }
        .ok_or_else(|| Error::new_spanned(input, Self::REFUSED))
    }

    pub fn member(&self) -> &Member {
        &self.member
    }

    pub fn inner(&self) -> &Type {
        &self.inner
    }

    pub fn counts(&self) -> bool {
        matches!(&self.inner, Type::Path(path) if path.qself.is_none() && path.path.is_ident(Self::COUNT))
    }

    fn among(fields: &Fields) -> Option<OnlyField> {
        match fields.iter().zip(fields.members()).collect::<Vec<_>>().as_slice() {
            [(field, member)] => Some(OnlyField { member: member.clone(), inner: field.ty.clone() }),
            _ => None,
        }
    }
}
