states::states! {
    pub stored RunState in run {
        start Running as Ended {
            replied -> Replied,
            unreadable -> Unreadable,
            errored -> Errored,
            overdue -> Overdue,
            killed -> Killed,
            gone -> Gone,
            failed -> Failed,
        }
        terminal Replied {}
        terminal Unreadable {}
        terminal Errored {}
        terminal Overdue {}
        terminal Killed {}
        terminal Gone {}
        terminal Failed {}
    }
}
states::states! {
    pub stored Delivery in delivery {
        start Unreported { reported -> Reported, withdrawn -> Withdrawn }
        state Reported { unreported -> Unreported }
        terminal Withdrawn {}
    }
}
