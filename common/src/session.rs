use net::Message;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SessionKind {
    PingPong,
    // TODO: Implement other session type
}
impl Message for SessionKind {}
