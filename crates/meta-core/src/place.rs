/// Where a log lives: the default data dir, `$HOME/.local/share/meta-dev` with XDG_DATA_HOME unset,
/// which holds the lead's log; or anywhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Place {
    Default,
    #[default]
    Elsewhere,
}
