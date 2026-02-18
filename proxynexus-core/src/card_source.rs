pub trait CardSource {
    fn get_codes(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;
}

pub struct Cardlist(pub String);
pub struct SetName(pub String);
pub struct NrdbUrl(pub String);
