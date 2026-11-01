states! {
    mod ajar {
        start Open { shut -> Shut }
        state Ajar { shut -> Shut }
        terminal Shut { reopen -> Open }
    }
}
states! {
    mod stuck {
        start Open { swing -> Ajar, shut -> Shut }
        state Ajar { stick -> Stuck }
        state Stuck { stick -> Ajar }
        terminal Shut {}
    }
}
states! {
    mod unknown {
        begin Open {}
    }
}
