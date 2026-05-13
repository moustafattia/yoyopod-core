#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockSource {
    SceneTime,
    GlobalTime,
    EventTime(EventId),
}
