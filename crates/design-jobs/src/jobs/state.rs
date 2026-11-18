states::states! {
    pub stored JobState in job {
        start Open { applied -> Applied, repeated -> Repeated, flailed -> Flailed, stalled -> Stalled, interrupted -> Interrupted }
        terminal Applied {}
        terminal Repeated {}
        terminal Flailed {}
        terminal Stalled {}
        terminal Interrupted {}
    }
}
