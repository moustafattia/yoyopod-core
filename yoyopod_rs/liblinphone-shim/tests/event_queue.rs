use yoyopod_liblinphone_shim::event::{EventQueue, YoyopodLiblinphoneEvent};

#[test]
fn event_queue_polls_fifo_events() {
    let queue = EventQueue::default();
    let mut first = YoyopodLiblinphoneEvent {
        event_type: 1,
        ..Default::default()
    };
    let second = YoyopodLiblinphoneEvent {
        event_type: 2,
        ..Default::default()
    };
    first.registration_state = 3;

    queue.push(first);
    queue.push(second);

    let first = queue.pop().expect("first event");
    assert_eq!(first.event_type, 1);
    assert_eq!(first.registration_state, 3);
    assert_eq!(queue.pop().expect("second event").event_type, 2);
    assert!(queue.pop().is_none());
}
