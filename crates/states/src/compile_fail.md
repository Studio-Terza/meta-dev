Compile checks for `states!`. The first block compiles; each later block differs from it by one defect
and must not compile. Stable rustdoc does not check the error codes, so they are recorded for readers.

A lifecycle that reaches its terminal state compiles:

```
states::states! {
    mod door {
        start Open { shut -> Shut }
        terminal Shut {}
    }
}
use door::{Finished, Open};

fn run<'id>(door: Open<'id>) -> Finished<'id> {
    door.shut().finish()
}

fn main() {
    Open::scope(run);
}
```

A path that drops the state has no witness to return:

```compile_fail,E0308
states::states! {
    mod door {
        start Open { shut -> Shut }
        terminal Shut {}
    }
}
use door::{Finished, Open};

fn run<'id>(door: Open<'id>, ready: bool) -> Finished<'id> {
    if ready {
        return door.shut().finish();
    }
    drop(door);
}

fn main() {}
```

A witness cannot be built by hand, so a dropped state cannot be covered up:

```compile_fail,E0423
states::states! {
    mod door {
        start Open { shut -> Shut }
        terminal Shut {}
    }
}
use door::{Finished, Open};

fn run<'id>(door: Open<'id>) -> Finished<'id> {
    drop(door);
    Finished(std::marker::PhantomData)
}

fn main() {}
```

A witness from another brand does not count, even when one lifetime outlives the other:

```compile_fail
states::states! {
    mod door {
        start Open { shut -> Shut }
        terminal Shut {}
    }
}
use door::{Finished, Open};

fn run<'id, 'other: 'id>(door: Open<'id>, other: Open<'other>) -> Finished<'id> {
    drop(door);
    other.shut().finish()
}

fn main() {}
```

A state cannot be copied, so one state cannot finish while its copy is dropped:

```compile_fail,E0599
states::states! {
    mod door {
        start Open { shut -> Shut }
        terminal Shut {}
    }
}
use door::{Finished, Open};

fn run<'id>(door: Open<'id>) -> Finished<'id> {
    let copy = door.clone();
    drop(door);
    copy.shut().finish()
}

fn main() {}
```

A state is not built by hand, so no second brand appears outside `scope`:

```compile_fail,E0423
states::states! {
    mod door {
        start Open { shut -> Shut }
        terminal Shut {}
    }
}
use door::Open;

fn main() {
    drop(Open(std::marker::PhantomData));
}
```

Only a terminal state finishes:

```compile_fail,E0599
states::states! {
    mod door {
        start Open { shut -> Shut }
        terminal Shut {}
    }
}
use door::{Finished, Open};

fn run<'id>(door: Open<'id>) -> Finished<'id> {
    door.finish()
}

fn main() {}
```

A transition consumes its state:

```compile_fail,E0382
states::states! {
    mod door {
        start Open { shut -> Shut }
        terminal Shut {}
    }
}
use door::{Finished, Open};

fn run<'id>(door: Open<'id>) -> Finished<'id> {
    drop(door.shut());
    door.shut().finish()
}

fn main() {}
```

A scope that drops its state does not compile:

```compile_fail,E0308
states::states! {
    mod door {
        start Open { shut -> Shut }
        terminal Shut {}
    }
}
use door::Open;

fn main() {
    Open::scope(|door| drop(door));
}
```

An unused state is reported, because states are `#[must_use]`:

```compile_fail
#![deny(unused_must_use)]

states::states! {
    mod door {
        start Open { shut -> Shut }
        terminal Shut {}
    }
}
use door::{Finished, Open};

fn run<'id>(door: Open<'id>, other: Open<'id>) -> Finished<'id> {
    other.shut();
    door.shut().finish()
}

fn main() {}
```

An unknown kind of state does not compile:

```compile_fail
states::states! {
    mod door {
        begin Open {}
    }
}

fn main() {}
```
