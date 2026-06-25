use std::sync::Arc;
use axum::extract::FromRef;
use necko3_core::core::NeckoCore;
use necko3_core::prelude::db::DatabaseExt;
use necko3_core::types::NeckoEvent;
use tokio::sync::broadcast;
use crate::auth::keystore::KeyStore;
use crate::db::DatabaseAdapter as BackendDatabaseAdapter;
use crate::http::ws::event_loop::EventForwarder;
use crate::http::ws::WsEvent;

pub struct AppState<D> {
    pub app_db: Arc<dyn BackendDatabaseAdapter>,
    pub core: NeckoCore<D, NeckoEvent>,

    pub key_store: Arc<KeyStore>,
    pub ws_sender: broadcast::Sender<WsEvent>,
}

impl<D> Clone for AppState<D> {
    fn clone(&self) -> Self {
        Self {
            app_db: Arc::clone(&self.app_db),
            core: self.core.clone(),
            key_store: Arc::clone(&self.key_store),
            ws_sender: self.ws_sender.clone(),
        }
    }
}

impl<D: DatabaseExt> FromRef<AppState<D>> for Arc<KeyStore> {
    fn from_ref(state: &AppState<D>) -> Self {
        state.key_store.clone()
    }
}

impl<D> AppState<D>
where
    D: DatabaseExt + 'static,
{
    pub fn new<B>(app_db: B, core: NeckoCore<D, NeckoEvent>) -> Self
    where
        B: BackendDatabaseAdapter + 'static,
    {
        let app_db = Arc::new(app_db);

        let (ws_sender, _) = broadcast::channel(1024);

        let key_store = Arc::new(KeyStore::new(app_db.clone()));

        let state = AppState { app_db, core, key_store, ws_sender };

        let event_forwarder = EventForwarder::new(state.clone());
        tokio::spawn(event_forwarder.run());

        state
    }
}

