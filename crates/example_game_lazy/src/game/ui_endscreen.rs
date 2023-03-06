use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, FontId, RichText},
    EguiContext,
};

use crate::utils::mouse::GameCamera;

use super::{
    scoring::{Score, ScoreNear, ScoreNearDef},
    GameState,
};

pub(super) fn ui_endscreen(
    mut egui_context: ResMut<EguiContext>,
    mut game_state: ResMut<NextState<GameState>>,
    windows: Query<Entity, With<Window>>,
) {
    egui::Area::new("Game Endscreen")
        .fixed_pos(egui::pos2(0.0, -150f32))
        .anchor(Align2::CENTER_TOP, egui::Vec2::ZERO)
        .show(
            egui_context.ctx_for_window_mut(windows.iter().next().unwrap()),
            |ui| {
                if ui.button("New Game").clicked() {
                    game_state.set(GameState::Warmup);
                }
            },
        );
}
pub(super) fn ui_end_title_and_score(
    mut egui_context: ResMut<EguiContext>,
    mut score: ResMut<Score>,
    windows: Query<Entity, With<Window>>,
) {
    egui::Area::new(format!("end juice"))
        .fixed_pos(egui::pos2(0f32, 0f32))
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(
            egui_context.ctx_for_window_mut(windows.iter().next().unwrap()),
            |ui| {
                ui.colored_label(
                    Color32::RED,
                    RichText::new(format!("GAME OVER ({})", score.score))
                        .font(FontId::proportional(80.0)),
                );
            },
        );
}
