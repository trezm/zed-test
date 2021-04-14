use std::collections::HashMap;

use crate::pokemon::Pokemon;

const DEFAULT_MAX_PARTY_SIZE: usize = 6;
const DEFAULT_MAX_BOX_SIZE: usize = 30;

pub enum StorageDestination {
    Party,
    Box(usize),
}

#[derive(Debug)]
pub enum StorageError {
    ContainerIsFull,
    BoxDoesNotExist,
    PokemonNotFound,
}

#[derive(Debug)]
enum ContainerLocation {
    Party,
    Box(usize),
}

#[derive(Debug)]
pub struct Storage {
    party: Container,
    boxes: Vec<Container>,
    max_party_size: usize,
    max_box_size: usize,
    pokemon_locations: HashMap<u32, ContainerLocation>,
}

impl Default for Storage {
    fn default() -> Self {
        Storage {
            party: Container::new(DEFAULT_MAX_PARTY_SIZE),
            boxes: vec![],
            max_party_size: DEFAULT_MAX_PARTY_SIZE,
            max_box_size: DEFAULT_MAX_BOX_SIZE,
            pokemon_locations: HashMap::new(),
        }
    }
}

impl Storage {
    pub fn add_box(&mut self) -> Result<usize, ()> {
        self.boxes.push(Container::new(self.max_box_size));

        Ok(self.boxes.len() - 1)
    }

    pub fn get_box(&self, id: usize) -> Result<Vec<&Pokemon>, StorageError> {
        Ok(self
            .boxes
            .get(id)
            .ok_or(StorageError::BoxDoesNotExist)?
            .get_pokemon())
    }

    pub fn get_party(&self) -> Result<Vec<&Pokemon>, StorageError> {
        Ok(self.party.get_pokemon())
    }

    pub fn add_pokemon(
        &mut self,
        pokemon: Pokemon,
        destination: StorageDestination,
    ) -> Result<(), StorageError> {
        match destination {
            StorageDestination::Party => {
                let id = pokemon.id();
                self.party.push(pokemon)?;
                self.pokemon_locations.insert(id, ContainerLocation::Party);
                Ok(())
            }
            StorageDestination::Box(i) => match self.boxes.get_mut(i) {
                Some(bx) => {
                    let id = pokemon.id();
                    bx.push(pokemon)?;
                    self.pokemon_locations
                        .insert(id, ContainerLocation::Box(self.boxes.len() - 1));
                    Ok(())
                }
                None => Err(StorageError::BoxDoesNotExist),
            },
        }
    }

    pub fn move_pokemon(
        &mut self,
        pokemon_id: u32,
        destination: StorageDestination,
    ) -> Result<&Pokemon, StorageError> {
        match destination {
            StorageDestination::Party => {
                if !self.party.has_space() {
                    Err(StorageError::ContainerIsFull)
                } else {
                    let storage_location = self
                        .pokemon_locations
                        .remove(&pokemon_id)
                        .ok_or(StorageError::PokemonNotFound)?;
                    let pokemon = match storage_location {
                        ContainerLocation::Party => self.party.remove(pokemon_id),
                        ContainerLocation::Box(i) => {
                            self.boxes.get_mut(i).unwrap().remove(pokemon_id)
                        }
                    }?;

                    self.party.push(pokemon)?;

                    self.pokemon_locations
                        .insert(pokemon_id, ContainerLocation::Party);

                    Ok(&pokemon)
                }
            }
            StorageDestination::Box(i) => match self.boxes.get_mut(i) {
                Some(bx) => {
                    if !bx.has_space() {
                        Err(StorageError::ContainerIsFull)
                    } else {
                        let storage_location = self
                            .pokemon_locations
                            .remove(&pokemon_id)
                            .ok_or(StorageError::PokemonNotFound)?;
                        let pokemon = match storage_location {
                            ContainerLocation::Party => self.party.remove(pokemon_id),
                            ContainerLocation::Box(i) => {
                                self.boxes.get_mut(i).unwrap().remove(pokemon_id)
                            }
                        }?;

                        self.party.push(pokemon)?;

                        self.pokemon_locations
                            .insert(pokemon_id, ContainerLocation::Party);

                        Ok(&pokemon)
                    }
                }
                None => Err(StorageError::BoxDoesNotExist),
            },
        }
    }
}

#[derive(Debug)]
pub struct Container {
    pokemon: HashMap<u32, Pokemon>,
    max_size: usize,
}

impl Container {
    pub fn new(max_size: usize) -> Self {
        Container {
            pokemon: HashMap::new(),
            max_size,
        }
    }

    pub fn push(&mut self, pokemon: Pokemon) -> Result<(), StorageError> {
        if !self.has_space() {
            Err(StorageError::ContainerIsFull)
        } else {
            self.pokemon.insert(pokemon.id(), pokemon);
            Ok(())
        }
    }

    pub fn remove(&mut self, id: u32) -> Result<Pokemon, StorageError> {
        self.pokemon
            .remove(&id)
            .ok_or(StorageError::PokemonNotFound)
    }

    pub fn has_space(&self) -> bool {
        self.pokemon.len() < self.max_size
    }

    pub fn get_pokemon(&self) -> Vec<&Pokemon> {
        self.pokemon.values().collect::<Vec<&Pokemon>>()
    }
}
