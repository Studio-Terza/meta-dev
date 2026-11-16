use design_model::JobId;
use design_runs::{Release, ReleaseNew, Succession, SuccessionNew};
use design_store::StreamSchema;
use laws::proptest::prelude::any;
use laws::proptest::strategy::Strategy;
use newtype::Counter;
use port_core::Port;
use sqlite_ports::SqliteOpener;
use store_ports::{Column, IntoArgument, Prepared, Store, StoreFailure, Written};

type Sqlite = <SqliteOpener as store_ports::Opens>::Store;

struct Failed;

impl Failed {
    fn job(store: &Port<'_, Sqlite>, job: JobId) -> Result<Written, StoreFailure> {
        let opened = store.write(Prepared::written(
            "INSERT OR IGNORE INTO jobs (id, state) VALUES (?1, 'flailed')",
            vec![job.raw().into_argument()],
        ))?;
        let failed = store.write(Prepared::written(
            "INSERT OR IGNORE INTO failures (job, halted, account, delivery) VALUES (?1, 'flailed', 'told', 'unreported')",
            vec![job.raw().into_argument()],
        ))?;
        Ok(Written::total([opened, failed].into_iter()))
    }

    fn released(store: &Port<'_, Sqlite>, fresh: &SuccessionNew) -> Result<Written, StoreFailure> {
        let failed = Failed::job(store, fresh.released)?;
        let released = store.write(Prepared::written(
            "INSERT OR IGNORE INTO releases (job) VALUES (?1)",
            vec![fresh.released.raw().into_argument()],
        ))?;
        let succeeding = Failed::job(store, fresh.successor)?;
        Ok(Written::total([failed, released, succeeding].into_iter()))
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
