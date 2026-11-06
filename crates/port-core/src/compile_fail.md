Compile checks for `Entry`. The first block compiles; each later block differs from it by one defect
and must not compile. Stable rustdoc does not check the error codes, so they are recorded for readers.

A port method receives its entry from the meter:

```
use port_core::{Entry, Metered, Op, OpName};

struct Disk;

struct Read;

impl Op for Read {
    type Port = Disk;
    const NAME: OpName = OpName::new("read");
}

fn read(_entry: Entry<Read>) {}

fn main() {
    Metered::new(Disk).call(|_, entry| read(entry));
}
```

An entry cannot be built by calling its constructor:

```compile_fail,E0423
use port_core::{Entry, Op, OpName};

struct Disk;

struct Read;

impl Op for Read {
    type Port = Disk;
    const NAME: OpName = OpName::new("read");
}

fn read(_entry: Entry<Read>) {}

fn main() {
    read(Entry(std::marker::PhantomData));
}
```

An entry cannot be built from a struct literal:

```compile_fail
use port_core::{Entry, Op, OpName};

struct Disk;

struct Read;

impl Op for Read {
    type Port = Disk;
    const NAME: OpName = OpName::new("read");
}

fn read(_entry: Entry<Read>) {}

fn main() {
    read(Entry { 0: std::marker::PhantomData });
}
```

An entry cannot be admitted from outside the crate:

```compile_fail,E0624
use port_core::{Entry, Op, OpName};

struct Disk;

struct Read;

impl Op for Read {
    type Port = Disk;
    const NAME: OpName = OpName::new("read");
}

fn read(_entry: Entry<Read>) {}

fn main() {
    read(Entry::admitted());
}
```

An entry for one port's op cannot be minted by the meter of another port:

```compile_fail,E0271
use port_core::{Entry, Metered, Op, OpName};

struct Disk;

struct Read;

impl Op for Read {
    type Port = Disk;
    const NAME: OpName = OpName::new("read");
}

fn read(_entry: Entry<Read>) {}

fn main() {
    Metered::new(()).call(|_, entry| read(entry));
}
```

A held resource cannot be built from an entry of another port's op:

```compile_fail,E0271
use std::rc::Rc;

use port_core::{Entry, Held, Metered, Op, OpName, ReleaseLog, Resource};

struct Disk;

struct Vault;

struct Read;

impl Op for Read {
    type Port = Disk;
    const NAME: OpName = OpName::new("read");
}

struct Key;

impl Resource for Key {
    type Port = Vault;
    type Failure = ();

    fn release(&mut self) -> Result<(), ()> {
        Ok(())
    }
}

fn main() {
    let log = Rc::new(ReleaseLog::default());
    let _held: Held<Key> = Metered::new(Disk).call(|_, entry: Entry<Read>| Held::new(entry, Key, &log));
}
```
