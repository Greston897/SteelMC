use super::*;

#[test]
fn slot_selector_keeps_wildcard_and_leaves_following_argument() {
    let mut dispatcher = TestDispatcher::new();
    assert!(
        dispatcher
            .register(literal("slots").then(
                argument("slots", SteelArgumentType::item_slots()).then(literal("next").executes(
                    |context| {
                        assert_eq!(context.string("slots")?, "container.*");
                        Ok(1)
                    }
                ),),
            ))
            .is_ok()
    );
    let parsed = dispatcher.parse("slots container.* next", TestSource::new());
    let chain = dispatcher
        .context_chain(parsed)
        .expect("wildcard must not leave trailing input");
    assert_eq!(chain.top_context().string("slots"), Ok("container.*"));
}

#[test]
fn slot_selector_has_minecraft_wire_type() {
    assert!(matches!(
        SteelArgumentType::item_slots().protocol_argument().0,
        ProtocolArgumentType::ItemSlots
    ));
}
