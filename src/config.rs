use envy;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BNIConfig {
    pub ibank_url: String,
    pub username: String,
    pub password: String,
}

impl BNIConfig {
    pub fn init() -> BNIConfig {
        envy::prefixed("BNI_")
            .from_env::<BNIConfig>()
            .expect("error loading env vars")
    }
}
