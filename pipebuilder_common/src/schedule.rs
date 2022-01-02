pub trait ScheduleHash<T> {
    fn schedule_hash(&self) -> T;
}

// (namespace, id)
pub struct ScheduleDescriptor<'a>(pub &'a str, pub &'a str);

impl<'a> ScheduleHash<String> for ScheduleDescriptor<'a> {
    fn schedule_hash(&self) -> String {
        format!("{}/{}", self.0, self.1)
    }
}
