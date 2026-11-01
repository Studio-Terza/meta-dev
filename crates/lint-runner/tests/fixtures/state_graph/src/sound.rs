states! {
    pub mod pr {
        start Working { review -> Reviewing, abandon -> Abandoned }
        state Reviewing { approve -> Merged, rework -> Working }
        terminal Merged {}
        terminal Abandoned {}
    }
}
