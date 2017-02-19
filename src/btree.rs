use std::marker::PhantomData;
use std::path::Path;
use std::convert::AsRef;
use errors::*;
use wabl::Wabl;

pub struct PageTreeMaster {
    wabl: Wabl,
}

pub struct PageTree;

pub struct TypeMapMaster;

//pub struct TypeMapRead<'a>;

//pub struct TypeMapWrite<'a>;

pub struct TypeMap<K, V, D=()>(PhantomData<(K, V, D)>);

impl TypeMapMaster {
    fn new<P: AsRef<Path>>(p: &P) -> Result<TypeMapMaster> {
        panic!()
    }

    fn map<K, V, D>() -> TypeMap<K, V, D> {
        panic!()
    }
}
