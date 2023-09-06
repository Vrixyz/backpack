use backpack_client_bevy_egui::*;
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_egui::*;

fn main() {
    App::new().add_plugins(GamePlugin).run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        drop(dotenvy::dotenv());
        let email = dotenvy::var("BACKPACK_GAME_EXAMPLE_USERNAME").unwrap();
        let password = dotenvy::var("BACKPACK_GAME_EXAMPLE_PASSWORD").unwrap();
        let host = dotenvy::var("BACKPACK_SERVER_BASE_URL").unwrap();
        app.add_plugins(DefaultPlugins.set(LogPlugin {
            level: Level::DEBUG,
            filter: "wgpu=info,bevy_render=info,bevy_ecs=info,wgpu_core=warn".to_string(),
        }))
        .add_plugins(EguiPlugin)
        .insert_resource(AuthInput {
            email: email.to_string(),
            password: password.to_string(),
            sign_in: password.is_empty(),
        })
        .add_plugins(AuthPlugin {
            host: dbg!(host.to_string()),
        })
        .add_systems(
            Update,
            (
                ui_backpack,
                handle_get_items_result,
                handle_modify_item_result,
            ),
        );
    }
}

fn ui_backpack(
    mut commands: Commands,
    time: Res<Time>,
    mut ctxs: EguiContexts,
    authentication: Res<BackpackClientAuthRefresh>,
    items: Res<BackpackItems>,
    backpack: Res<BackpackCom>,
    get_items_tasks: Query<Entity, With<GetItemsTask>>,
) {
    let Some(current_user_id) = authentication.get_current_user_id() else {
        return;
    };
    egui::Window::new("My Backpack")
        .auto_sized()
        .show(ctxs.ctx_mut(), |ui| {
            if get_items_tasks.is_empty() {
                let get_items_button = ui.button("Get items");
                if get_items_button.clicked() {
                    bevy_get_items(
                        &mut commands,
                        &*time,
                        &backpack.client,
                        &authentication,
                        &current_user_id,
                    );
                }
            } else {
                ui.label("Getting items...");
            }
            if !items.items.is_empty() {
                ui.group(|ui| {
                    for item in (*items.items).iter() {
                        if item.item.id.0 == 1 {
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "{}({}): {}",
                                    item.item.name, item.item.id.0, item.amount
                                ));
                                ui.vertical(|ui| {
                                    if ui.button("+1").clicked() {
                                        let _ = bevy_modify_item(
                                            &mut commands,
                                            &*time,
                                            &backpack.client,
                                            &authentication,
                                            &item.item.id,
                                            1,
                                            &current_user_id,
                                        );
                                    }
                                });
                            });
                        }
                    }
                });
            }
        });
}

fn handle_get_items_result(
    mut events: EventReader<GetItemsTaskResultEvent>,
    mut resource_items: ResMut<BackpackItems>,
) {
    for res in events.iter() {
        if let Ok(items) = &res.0 {
            dbg!(items);
            resource_items.items = items.clone();
            if !resource_items.items.iter().any(|item| item.item.id.0 == 1) {
                resource_items.items.push(shared::ItemAmount {
                    item: shared::ItemWithName {
                        id: shared::ItemId(1),
                        name: "currency".to_string(),
                    },
                    amount: 0,
                })
            }
        } else {
            dbg!("get items failed.");
        }
    }
}

fn handle_modify_item_result(
    mut events: EventReader<ModifyItemTaskResultEvent>,
    mut resource_items: ResMut<BackpackItems>,
) {
    for res in events.iter() {
        if let Ok(item) = &res.0 {
            dbg!(item);
            if let Some(saved_item) = resource_items
                .items
                .iter_mut()
                .find(|i| i.item.id == item.0)
            {
                saved_item.amount = item.2;
            }
        } else {
            dbg!("get items failed.");
        }
    }
}
