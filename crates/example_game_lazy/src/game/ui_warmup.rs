use crate::{BackpackCom, BackpackItems};
use backpack_client_bevy::{
    bevy_get_items, bevy_modify_item, BackpackClientAuthRefresh, GetItemsTask,
    GetItemsTaskResultEvent, ModifyItemTaskResultEvent,
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    egui::{self, Align2},
    EguiContext, EguiContexts,
};

use super::{mouse::MousePos, CollisionState, GameDef, GameDefBorder, GameState};

pub(super) fn ui_tuto_start(auth_cache: Res<BackpackClientAuthRefresh>, mut ctxs: EguiContexts) {
    egui::Area::new("my_area")
        .fixed_pos(egui::pos2(0.0, 0.0))
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctxs.ctx_mut(), |ui| {
            ui.colored_label(
                egui::Color32::BLUE,
                "TAP\nin Game Area\nTo START!\n\nAvoid little bevies.",
            );
            if auth_cache.current_authentication_token.is_none() {
                ui.colored_label(
                    egui::Color32::RED,
                    "\n\nYou are not connected,\nYou won't gain any items.",
                );
            }
        });
}

pub(super) fn handle_tap_to_start(
    borders: Res<GameDefBorder>,
    mut game_state: ResMut<NextState<GameState>>,
    mut mouse_input: ResMut<Input<MouseButton>>,
    mouse_pos: Res<MousePos>,
    q_collisions: Query<&CollisionState>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if !Rect::from_center_half_size(Vec2::ZERO, borders.borders).contains(mouse_pos.0) {
            return;
        }
        for collision in q_collisions.iter() {
            if *collision == CollisionState::Colliding {
                return;
            }
        }
        dbg!("tap detected");
        game_state.set(GameState::LoadingPlay);
        mouse_input.clear_just_pressed(MouseButton::Left);
    }
}

pub(super) fn ui_warmup(
    mut commands: Commands,
    time: Res<Time>,
    mut ctxs: EguiContexts,
    authentication: Res<BackpackClientAuthRefresh>,
    items: Res<BackpackItems>,
    mut game_def: ResMut<GameDef>,
    backpack: Res<BackpackCom>,
    get_items_tasks: Query<Entity, With<GetItemsTask>>,
) {
    let Some(current_user_id) = authentication.get_current_user_id() else {
        return;
    };
    egui::Window::new("Warmup")
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
                                    item.item.name,
                                    item.item.id.0,
                                    item.amount - game_def.enemy_count as i32
                                ));
                                ui.vertical(|ui| {
                                    if item.amount > game_def.enemy_count as i32 {
                                        if ui.button("+1 enemy").clicked() {
                                            game_def.enemy_count += 1;
                                        }
                                    } else {
                                        // Not enough item amount
                                        if ui.button("Not enough enemy in stock").clicked() {
                                            dbg!("not enough item amount");
                                        }
                                    }
                                    if game_def.enemy_count > 0 {
                                        // Can remove enemies
                                        if ui.button("-1 enemy").clicked() {
                                            game_def.enemy_count -= 1;
                                        }
                                    }
                                    if std::env::var("CHEAT").unwrap_or("false".into()) == "true" {
                                        // Cheat
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
                                    }
                                });
                            });
                        }
                    }
                });
            }
        });
}

pub(super) fn handle_get_items_result(
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

pub(super) fn handle_modify_item_result(
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
