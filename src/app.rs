use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use std::time::Instant;
use thruster::context::hyper_request::HyperRequest;
use thruster::errors::ThrusterError as Error;
use thruster::App;
use thruster::{async_middleware, map_try, middleware_fn};
use thruster::{MiddlewareNext, MiddlewareResult};
use tokio::sync::RwLock;

use crate::context::Ctx;
use crate::errors::ErrorSet;
use crate::pokemon::Pokemon;
use crate::pokemon_api::get_pokemon;
use crate::storage::{Storage, StorageDestination, StorageError};

// -- Util-ish stuff
fn generate_context(request: HyperRequest, state: &Arc<RwLock<Storage>>, _path: &str) -> Ctx {
    Ctx::new(request, state.clone())
}

#[middleware_fn]
async fn profiling(mut context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let start_time = Instant::now();

    let method = context
        .hyper_request
        .as_ref()
        .unwrap()
        .request
        .method()
        .clone();
    let path_and_query = context
        .hyper_request
        .as_ref()
        .unwrap()
        .request
        .uri()
        .path_and_query()
        .unwrap()
        .clone();

    context = match next(context).await {
        Ok(context) => context,
        Err(e) => e.context,
    };

    let elapsed_time = start_time.elapsed();
    info!(
        "{}μs\t\t{}\t{}",
        elapsed_time.as_micros(),
        method,
        path_and_query
    );

    Ok(context)
}

#[middleware_fn]
pub async fn log_storage(context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let mut default_context = Ctx::new(HyperRequest::default(), context.extra.clone());
    // let hyper_request = context.hyper_request.unwrap().request;
    let storage = context.extra.read().await;

    info!("Storage: {:#?}", storage);

    default_context.body("Check your logs, friend.");

    Ok(default_context)
}

// -- Actual middleware
#[derive(Serialize)]
struct CreateBoxResponse {
    box_id: usize,
}
#[middleware_fn]
pub async fn create_box(context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let mut default_context = Ctx::new(HyperRequest::default(), context.extra.clone());
    // let hyper_request = context.hyper_request.unwrap().request;
    let mut storage = context.extra.write().await;

    let box_id = (*storage).add_box().unwrap();

    let body = serde_json::to_string(&CreateBoxResponse { box_id }).unwrap();

    default_context.body(&body);

    Ok(default_context)
}

#[derive(Serialize)]
struct GetBoxResponse<'a> {
    pokemon: Vec<&'a Pokemon>,
}
#[middleware_fn]
pub async fn get_box(context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let mut default_context = Ctx::new(HyperRequest::default(), context.extra.clone());
    // let hyper_request = context.hyper_request.unwrap().request;
    let storage = context.extra.read().await;

    let id = map_try!(match context.params.unwrap().get("id") {
        Some(val) => val.parse::<usize>(),
        None => {
            return Err(Error::parsing_error(default_context, "Must include an id"));
        }
    }, Err(_e) => {
        Error::parsing_error(default_context, "Must include an id")
    });

    let pokemon = map_try!(storage.get_box(id), Err(_e) => {
        Error::not_found_error(default_context)
    });

    let body = serde_json::to_string(&GetBoxResponse { pokemon }).unwrap();

    default_context.body(&body);

    Ok(default_context)
}

#[middleware_fn]
pub async fn get_party(context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let mut default_context = Ctx::new(HyperRequest::default(), context.extra.clone());
    // let hyper_request = context.hyper_request.unwrap().request;
    let storage = context.extra.read().await;

    let pokemon = map_try!(storage.get_party(), Err(_e) => {
        Error::not_found_error(default_context)
    });

    let body = serde_json::to_string(&GetBoxResponse { pokemon }).unwrap();

    default_context.body(&body);

    Ok(default_context)
}

#[derive(Deserialize)]
struct MovePokemonRequest {
    pokeAPI_id: u32,
}
#[derive(Serialize)]
struct MovePokemonResponse<'a> {
    pokemon: &'a Pokemon,
}
#[middleware_fn]
pub async fn move_pokemon_to_box(
    context: Ctx,
    _next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let mut default_context = Ctx::new(HyperRequest::default(), context.extra.clone());
    let (content, context) = context.get_body().await.unwrap();
    let mut storage = context.extra.write().await;

    let pokemon_id = map_try!(serde_json::from_str::<MovePokemonRequest>(&content), Err(_e) => {
        Error::parsing_error(default_context, "Must include a pokeAPI_id in your request")
    })
    .pokeAPI_id;

    let id = map_try!(match context.params.unwrap().get("id") {
        Some(val) => val.parse::<usize>(),
        None => {
            return Err(Error::parsing_error(default_context, "Must include an id"));
        }
    }, Err(_e) => {
        Error::parsing_error(default_context, "Must include an id")
    });

    match storage.move_pokemon(pokemon_id, StorageDestination::Box(id)) {
        Ok(pokemon) => {
            let body = serde_json::to_string(&MovePokemonResponse { pokemon }).unwrap();

            default_context.body(&body);
        }
        Err(e) => match e {
            StorageError::BoxDoesNotExist => {
                return Err(Error::not_found_error(default_context));
            }
            StorageError::ContainerIsFull => {
                return Err(Error::container_is_full(default_context));
            }
            StorageError::PokemonNotFound => {
                let pokemon = map_try!(get_pokemon(pokemon_id).await, Err(_e) => {
                    Error::generic_error(default_context)
                });

                match storage.add_pokemon(pokemon, StorageDestination::Box(id)) {
                    Ok(pokemon) => {
                        let body = serde_json::to_string(&MovePokemonResponse { pokemon }).unwrap();

                        default_context.body(&body);
                    }
                    Err(e) => match e {
                        StorageError::BoxDoesNotExist => {
                            return Err(Error::not_found_error(default_context));
                        }
                        StorageError::ContainerIsFull => {
                            return Err(Error::container_is_full(default_context));
                        }
                        StorageError::PokemonNotFound => {
                            error!("Yikes, we tried to make a pokemon, inserted it, but then still couldn't find it");
                            return Err(Error::not_found_error(default_context));
                        }
                    },
                }
            }
        },
    };

    Ok(default_context)
}

#[middleware_fn]
pub async fn move_pokemon_to_party(
    context: Ctx,
    _next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let mut default_context = Ctx::new(HyperRequest::default(), context.extra.clone());
    let (content, context) = context.get_body().await.unwrap();
    let mut storage = context.extra.write().await;

    let pokemon_id = map_try!(serde_json::from_str::<MovePokemonRequest>(&content), Err(_e) => {
        Error::parsing_error(default_context, "Must include a pokeAPI_id in your request")
    })
    .pokeAPI_id;

    match storage.move_pokemon(pokemon_id, StorageDestination::Party) {
        Ok(pokemon) => {
            let body = serde_json::to_string(&MovePokemonResponse { pokemon }).unwrap();

            default_context.body(&body);
        }
        Err(e) => match e {
            StorageError::BoxDoesNotExist => {
                error!("Yikes, we tried to make a pokemon, and somehow got box doesn't exist");
                return Err(Error::generic_error(default_context));
            }
            StorageError::ContainerIsFull => {
                return Err(Error::container_is_full(default_context));
            }
            StorageError::PokemonNotFound => {
                let pokemon = map_try!(get_pokemon(pokemon_id).await, Err(_e) => {
                    Error::generic_error(default_context)
                });

                match storage.add_pokemon(pokemon, StorageDestination::Party) {
                    Ok(pokemon) => {
                        let body = serde_json::to_string(&MovePokemonResponse { pokemon }).unwrap();

                        default_context.body(&body);
                    }
                    Err(e) => match e {
                        StorageError::BoxDoesNotExist => {
                            return Err(Error::not_found_error(default_context));
                        }
                        StorageError::ContainerIsFull => {
                            return Err(Error::container_is_full(default_context));
                        }
                        StorageError::PokemonNotFound => {
                            error!("Yikes, we tried to make a pokemon, inserted it, but then still couldn't find it");
                            return Err(Error::not_found_error(default_context));
                        }
                    },
                }
            }
        },
    };

    Ok(default_context)
}

pub async fn create() -> App<HyperRequest, Ctx, Arc<RwLock<Storage>>> {
    let storage = Storage::default();

    let mut app = App::<HyperRequest, Ctx, Arc<RwLock<Storage>>>::create(
        generate_context,
        Arc::new(RwLock::new(storage)),
    );

    app.use_middleware("/", async_middleware!(Ctx, [profiling]));
    app.post("/boxes", async_middleware!(Ctx, [create_box]));
    app.get("/boxes/:id", async_middleware!(Ctx, [get_box]));
    app.get("/parties", async_middleware!(Ctx, [get_party]));
    app.post(
        "/boxes/:id/pokemon",
        async_middleware!(Ctx, [move_pokemon_to_box]),
    );
    app.post(
        "/parties/pokemon",
        async_middleware!(Ctx, [move_pokemon_to_party]),
    );
    app.get("/info", async_middleware!(Ctx, [log_storage]));

    app
}

#[cfg(not(test))]
pub async fn init() {
    use dotenv::dotenv;
    use std::env;
    use thruster::HyperServer;
    use thruster::ThrusterServer;

    let _ = dotenv();
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    info!("Starting server at {}:{}", host, port);

    let app = create().await;

    let server = HyperServer::new(app);
    server.build(&host, port.parse::<u16>().unwrap()).await
}
