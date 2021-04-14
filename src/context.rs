use std::sync::Arc;
use thruster::context::typed_hyper_context::TypedHyperContext;
use tokio::sync::RwLock;

use crate::storage::Storage;

pub type Ctx = TypedHyperContext<Arc<RwLock<Storage>>>;
