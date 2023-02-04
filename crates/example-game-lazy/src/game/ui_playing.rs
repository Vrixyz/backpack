use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContext,
};

use crate::utils::mouse::GameCamera;

use super::{scoring::ScoreNear, GameState};

pub(super) fn ui_playing(
    mut egui_context: ResMut<EguiContext>,
    mut game_state: ResMut<State<GameState>>,
) {
    egui::Area::new("Game UI")
        .fixed_pos(egui::pos2(0.0, -150f32))
        .anchor(Align2::CENTER_TOP, egui::Vec2::ZERO)
        .show(egui_context.ctx_mut(), |ui| {
            if ui.button("Forfeit").clicked() {
                game_state.set(GameState::Warmup);
            }
        });
}
pub(super) fn ui_scoring(
    mut egui_context: ResMut<EguiContext>,
    mut camera: Query<(&GlobalTransform, &Camera), With<GameCamera>>,
    mut q_scores: Query<(&ScoreNear, &Transform)>,
    mut game_state: ResMut<State<GameState>>,
) {
    let camera = camera.single();
    for (i, (score, transform)) in q_scores.iter().enumerate() {
        let Some(position) = camera.1.world_to_viewport(camera.0, transform.translation) else {
            continue;
        };
        egui::Area::new(format!("score {i}"))
            .fixed_pos(egui::pos2(0f32, 0f32))
            .anchor(
                Align2::LEFT_BOTTOM,
                egui::Vec2::new(position.x, -position.y),
            )
            .show(egui_context.ctx_mut(), |ui| {
                ui.label("enemy");
            });
    }
}
