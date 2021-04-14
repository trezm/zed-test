use anyhow::Error;
use serde::Deserialize;

use crate::pokemon::Pokemon;

#[derive(Debug, Deserialize)]
pub struct PokemonFromApi {
    pub id: u32,
    pub name: String,
    pub height: u32,
    pub weight: u32,
    pub base_happiness: u32,
}

#[derive(Debug, Deserialize)]
pub struct PokemonSepeciesFromApi {
    pub id: u32,
    pub base_happiness: u32,
}

#[cfg(test)]
pub async fn get_pokemon(_id: u32) -> Result<Pokemon, Error> {
    Ok(Pokemon {
        pokeAPI_id: 141,
        name: "kabuptops".to_string(),
        height: 13,
        weight: 405,
        base_happiness: 0,
    })
}

#[cfg(not(test))]
pub async fn get_pokemon(id: u32) -> Result<Pokemon, Error> {
    let pokemon = get_pokemon_from_api(id).await?;
    let species = get_pokemon_species_from_api(id).await?;

    Ok(Pokemon {
        pokeAPI_id: pokemon.id,
        name: pokemon.name,
        height: pokemon.height,
        weight: pokemon.weight,
        base_happiness: species.base_happiness,
    })
}

#[cfg(not(test))]
async fn get_pokemon_from_api(id: u32) -> Result<PokemonFromApi, Error> {
    use log::error;
    use reqwest::header::CONTENT_TYPE;

    let client = reqwest::Client::new();
    let req = client
        .get(&format!("https://pokeapi.co/api/v2/pokemon/{}", id))
        .header(CONTENT_TYPE, "application/json; charset=utf-8");

    let res = req.send().await;

    match res {
        Ok(body) => {
            let body = body.text().await.unwrap();
            let parsed = serde_json::from_str::<PokemonFromApi>(&body);

            match parsed {
                Ok(resp) => Ok(resp),
                Err(e) => {
                    error!("pokemon api parsing error: {:#?}", e);
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            error!("pokemon api error: {:#?}", e);
            Err(e.into())
        }
    }
}

#[cfg(not(test))]
async fn get_pokemon_species_from_api(id: u32) -> Result<PokemonSepeciesFromApi, Error> {
    use log::error;
    use reqwest::header::CONTENT_TYPE;

    let client = reqwest::Client::new();
    let req = client
        .get(&format!("https://pokeapi.co/api/v2/pokemon-species/{}", id))
        .header(CONTENT_TYPE, "application/json; charset=utf-8");

    let res = req.send().await;

    match res {
        Ok(body) => {
            let body = body.text().await.unwrap();
            let parsed = serde_json::from_str::<PokemonSepeciesFromApi>(&body);

            match parsed {
                Ok(resp) => Ok(resp),
                Err(e) => {
                    error!("pokemon api parsing error: {:#?}", e);
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            error!("pokemon api error: {:#?}", e);
            Err(e.into())
        }
    }
}
