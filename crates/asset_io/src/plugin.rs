use bevy::prelude::*;
use bevy::tasks::IoTaskPool;

#[derive(Default)]
pub struct InMemoryAssetPlugin;

impl Plugin for InMemoryAssetPlugin {
    fn build(&self, app: &mut App) {
        let task_pool = app
            .world
            .get_resource::<IoTaskPool>()
            .expect("`IoTaskPool` resource not found.")
            .0
            .clone();

        app.insert_resource(AssetServer::new(
            crate::InMemoryAssetIo::preloaded(),
            task_pool,
        ));
    }
}
