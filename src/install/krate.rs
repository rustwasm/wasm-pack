use crate::install::Tool;

#[derive(Debug, Deserialize)]
pub struct Krate {
    pub max_version: String,
}

#[derive(Debug, Deserialize)]
pub struct KrateResponse {
    #[serde(rename = "crate")]
    pub krate: Krate,
}

impl Krate {
    pub fn new(name: &Tool) -> Result<Krate, failure::Error> {
        let krate_address = format!("https://crates.io/api/v1/crates/{}", name);
        let res = reqwest::blocking::get(&krate_address)?;

        let kr: KrateResponse = serde_json::from_str(&res.text()?)?;
        Ok(kr.krate)
    }
}
