use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContext,
};

use crate::utils::mouse::GameCamera;

use super::{
    scoring::{ScoreNear, ScoreNearDef},
    GameState,
};

pub(super) fn ui_endscreen(
    mut egui_context: ResMut<EguiContext>,
    mut game_state: ResMut<State<GameState>>,
) {
    egui::Area::new("Game Endscreen")
        .fixed_pos(egui::pos2(0.0, -150f32))
        .anchor(Align2::CENTER_TOP, egui::Vec2::ZERO)
        .show(egui_context.ctx_mut(), |ui| {
            if ui.button("New Game").clicked() {
                game_state.set(GameState::Warmup);
            }
        });
}
