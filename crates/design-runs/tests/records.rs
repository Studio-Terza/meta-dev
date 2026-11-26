use design_model::JobId;
use design_runs::{Release, ReleaseNew, Succession, SuccessionNew};
use design_store::StreamSchema;
use laws::proptest::prelude::any;
use laws::proptest::strategy::Strategy;
use port_core::Port;
use sqlite_ports::SqliteOpener;
use store_ports::{Column, IntoArgument, Prepared, Store, StoreFailure};

type Sqlite = <SqliteOpener as store_ports::Opens>::Store;

struct Failed;

impl Failed {
    fn job(store: &Port<'_, Sqlite>, job: JobId) -> Result<(), StoreFailure> {
        store.write(Prepared::written(
            "INSERT OR IGNORE INTO jobs (id, state) VALUES (?1, 'flailed')",
            vec![job.raw().into_argument()],
        ))?;
        store.write(Prepared::written(
            "INSERT OR IGNORE INTO failures (job, halted, account, delivery) VALUES (?1, 'flailed', 'told', 'unreported')",
            vec![job.raw().into_argument()],
        ))
    }

    fn released(store: &Port<'_, Sqlite>, fresh: &SuccessionNew) -> Result<(), StoreFailure> {
        Failed::job(store, fresh.released)?;
        store.write(Prepared::written(
            "INSERT OR IGNORE INTO releases (job) VALUES (?1)",
            vec![fresh.released.raw().into_argument()],
        ))?;
        Failed::job(store, fresh.successor)
    }
}

laws::crud!(
    a_release_keeps_the_crud_laws,
    Release,
    any::<JobId>().prop_map(|job| ReleaseNew { job }),
    Sqlite::in_memory::<StreamSchema>(),
    given |store: &Port<'_, Sqlite>, fresh: &ReleaseNew| Failed::job(store, fresh.job)
);

laws::crud!(
    a_succession_keeps_the_crud_laws,
    Succession,
    (any::<JobId>(), any::<JobId>()).prop_map(|(released, successor)| SuccessionNew { released, successor }),
    Sqlite::in_memory::<StreamSchema>(),
    given Failed::released
);
