use lint_layout::{PathTail, Word, words};
use lint_paths::Use;

use super::called::Called;
use super::effect::Effect;
use super::effect_site::EffectSite;

#[derive(Debug)]
struct Tail {
    effect: Effect,
    words: &'static [Word<'static>],
}

impl Tail {
    const fn new(effect: Effect, words: &'static [Word<'static>]) -> Tail {
        Tail { effect, words }
    }

    fn spelled_in(&self, used: &Use<'_>) -> Option<&Tail> {
        used.spelling(PathTail::new(self.words)).map(|_| self)
    }

    fn site_of<'file>(&self, used: Use<'file>) -> Option<EffectSite<'file>> {
        let place = used.place();
        let ground = used.ground();
        used.spelling(PathTail::new(self.words))
            .cloned()
            .map(|path| EffectSite::new(place, self.effect, Called::Path(path), ground))
    }
}

#[derive(Debug)]
pub(super) struct Vocabulary;

impl Vocabulary {
    pub(super) const FINALS: [Word<'static>; 43] = words![
        "read",
        "read_to_string",
        "write",
        "create_dir",
        "create_dir_all",
        "remove_file",
        "remove_dir",
        "remove_dir_all",
        "copy",
        "rename",
        "metadata",
        "read_dir",
        "canonicalize",
        "hard_link",
        "set_permissions",
        "read_link",
        "open",
        "create",
        "new",
        "var",
        "var_os",
        "vars",
        "vars_os",
        "set_var",
        "remove_var",
        "args",
        "args_os",
        "current_dir",
        "set_current_dir",
        "temp_dir",
        "current_exe",
        "now",
        "sleep",
        "connect",
        "bind",
        "exit",
        "abort",
        "random",
        "thread_rng",
        "rng",
        "stdin",
        "stdout",
        "stderr",
    ];
    pub(super) const MACROS: [Word<'static>; 5] = words!["print", "println", "eprint", "eprintln", "dbg"];
    const SHORT: [Tail; 39] = [
        Tail::new(Effect::Files, &words!["fs", "read"]),
        Tail::new(Effect::Files, &words!["fs", "read_to_string"]),
        Tail::new(Effect::Files, &words!["fs", "write"]),
        Tail::new(Effect::Files, &words!["fs", "create_dir"]),
        Tail::new(Effect::Files, &words!["fs", "create_dir_all"]),
        Tail::new(Effect::Files, &words!["fs", "remove_file"]),
        Tail::new(Effect::Files, &words!["fs", "remove_dir"]),
        Tail::new(Effect::Files, &words!["fs", "remove_dir_all"]),
        Tail::new(Effect::Files, &words!["fs", "copy"]),
        Tail::new(Effect::Files, &words!["fs", "rename"]),
        Tail::new(Effect::Files, &words!["fs", "metadata"]),
        Tail::new(Effect::Files, &words!["fs", "read_dir"]),
        Tail::new(Effect::Files, &words!["fs", "canonicalize"]),
        Tail::new(Effect::Files, &words!["fs", "hard_link"]),
        Tail::new(Effect::Files, &words!["fs", "set_permissions"]),
        Tail::new(Effect::Files, &words!["fs", "read_link"]),
        Tail::new(Effect::Environment, &words!["env", "var"]),
        Tail::new(Effect::Environment, &words!["env", "var_os"]),
        Tail::new(Effect::Environment, &words!["env", "vars"]),
        Tail::new(Effect::Environment, &words!["env", "vars_os"]),
        Tail::new(Effect::Environment, &words!["env", "set_var"]),
        Tail::new(Effect::Environment, &words!["env", "remove_var"]),
        Tail::new(Effect::Environment, &words!["env", "args"]),
        Tail::new(Effect::Environment, &words!["env", "args_os"]),
        Tail::new(Effect::Environment, &words!["env", "current_dir"]),
        Tail::new(Effect::Environment, &words!["env", "set_current_dir"]),
        Tail::new(Effect::Environment, &words!["env", "temp_dir"]),
        Tail::new(Effect::Environment, &words!["env", "current_exe"]),
        Tail::new(Effect::Clock, &words!["SystemTime", "now"]),
        Tail::new(Effect::Clock, &words!["Instant", "now"]),
        Tail::new(Effect::Clock, &words!["thread", "sleep"]),
        Tail::new(Effect::Processes, &words!["process", "exit"]),
        Tail::new(Effect::Processes, &words!["process", "abort"]),
        Tail::new(Effect::Randomness, &words!["rand", "random"]),
        Tail::new(Effect::Randomness, &words!["rand", "thread_rng"]),
        Tail::new(Effect::Randomness, &words!["rand", "rng"]),
        Tail::new(Effect::Stdio, &words!["io", "stdin"]),
        Tail::new(Effect::Stdio, &words!["io", "stdout"]),
        Tail::new(Effect::Stdio, &words!["io", "stderr"]),
    ];
    const LONG: [Tail; 9] = [
        Tail::new(Effect::Files, &words!["fs", "File", "open"]),
        Tail::new(Effect::Files, &words!["fs", "File", "create"]),
        Tail::new(Effect::Files, &words!["fs", "OpenOptions", "new"]),
        Tail::new(Effect::Network, &words!["net", "TcpStream", "connect"]),
        Tail::new(Effect::Network, &words!["net", "TcpListener", "bind"]),
        Tail::new(Effect::Network, &words!["net", "UdpSocket", "bind"]),
        Tail::new(Effect::Processes, &words!["process", "Command", "new"]),
        Tail::new(Effect::Randomness, &words!["hash", "RandomState", "new"]),
        Tail::new(Effect::Clock, &words!["chrono", "Utc", "now"]),
    ];

    pub(super) fn site_of(used: Use<'_>) -> Option<EffectSite<'_>> {
        Self::LONG
            .iter()
            .find_map(|tail| tail.spelled_in(&used))
            .or_else(|| Self::SHORT.iter().find_map(|tail| tail.spelled_in(&used)))
            .and_then(|tail| tail.site_of(used))
    }
}
