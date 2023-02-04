use std::ops::Bound;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContext,
};

use crate::{
    backpack_client_bevy::{bevy_get_items, GetItemsTask, GetItemsTaskResultEvent, LoginTask},
    AuthData, BackpackCom, BackpackItems,
};

use super::{mouse::MousePos, CollisionState, GameDef, GameState};

pub(super) fn ui_tuto_start(mut egui_context: ResMut<EguiContext>) {
    egui::Area::new("my_area")
        .fixed_pos(egui::pos2(0.0, 0.0))
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(egui_context.ctx_mut(), |ui| {
            ui.colored_label(
                egui::Color32::BLUE,
                "TAP\nin Game Area\nTo START!\n\nAvoid little bevies.",
            );
        });
}

pub(super) fn handle_tap_to_start(
    mut game_state: ResMut<State<GameState>>,
    mouse_input: Res<Input<MouseButton>>,
    mouse_pos: Res<MousePos>,
    q_collisions: Query<&CollisionState>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if !Rect::from_center_half_size(Vec2::ZERO, Vec2::splat(1000f32)).contains(mouse_pos.0) {
            return;
        }
        for collision in q_collisions.iter() {
            if *collision == CollisionState::Colliding {
                return;
            }
        }
        game_state.set(GameState::Playing);
    }
}

pub(super) fn ui_warmup(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    auth_data: Res<AuthData>,
    items: Res<BackpackItems>,
    mut game_def: ResMut<GameDef>,
    backpack: Res<BackpackCom>,
    get_items_tasks: Query<Entity, With<GetItemsTask>>,
) {
    if auth_data.data.is_none() {
        return;
    }
    egui::Window::new("Warmup")
        .auto_sized()
        .show(egui_context.ctx_mut(), |ui| {
            if let Some(auth) = &auth_data.data {
                if get_items_tasks.is_empty() {
                    let mut get_items_button = ui.button("Get items");
                    if get_items_button.clicked() {
                        bevy_get_items(&mut commands, &backpack.client, &auth.0, &auth.1.user_id);
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
                                                // TODO: pay 1 item
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
                                                // TODO: pay 1 item
                                            }
                                        }
                                    });
                                });
                            }
                        }
                    });
                }
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
            resource_items.items = (*items).clone();
        } else {
            dbg!("get items failed.");
        }
    }
}
