pub trait Beautify {
    fn beautiful_out(&self) -> String;
}

// todo
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadableRequest(pub String);
