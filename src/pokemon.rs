use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Pokemon {
    pub pokeAPI_id: u32,
    pub name: String,
    pub height: u32,
    pub weight: u32,
    pub base_happiness: u32,
}

impl Pokemon {
    pub fn id(&self) -> u32 {
        self.pokeAPI_id
    }
}
