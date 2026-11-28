use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait Decode: DeserializeOwned {}

pub trait Encode: Serialize {}

impl<Read: DeserializeOwned> Decode for Read {}

impl<Written: Serialize + ?Sized> Encode for Written {}
