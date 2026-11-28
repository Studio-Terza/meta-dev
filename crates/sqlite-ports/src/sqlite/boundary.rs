use port_core::Pass;
use query_cache::{DescribeFiller, Entry, Undescribed, describe_ops};
use store_ports::StoreFailure;

use super::Sqlite;

impl DescribeFiller for Sqlite {
    fn describe(&self, _pass: Pass<'_, describe_ops::Describe>, query: &str) -> Result<Entry<String>, Undescribed> {
        let opened = self.shared.opened(|opened| Ok::<_, StoreFailure>(opened.described(query)));
        opened.map_err(|failed| Undescribed::told(query, &failed)).and_then(|described| described)
    }
}
