use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, FontId, RichText},
    EguiContext,
};

use crate::utils::mouse::GameCamera;

use super::{
    scoring::{Score, ScoreNear, ScoreNearDef},
    GameState,
};

pub(super) fn ui_playing(
    mut egui_context: ResMut<EguiContext>,
    mut game_state: ResMut<NextState<GameState>>,
    windows: Query<Entity, With<Window>>,
) {
    egui::Area::new("Game UI")
        .fixed_pos(egui::pos2(0.0, -150f32))
        .anchor(Align2::CENTER_TOP, egui::Vec2::ZERO)
        .show(
            egui_context.ctx_for_window_mut(windows.iter().next().unwrap()),
            |ui| {
                if ui.button("Forfeit").clicked() {
                    game_state.set(GameState::Warmup);
                }
            },
        );
}
pub(super) fn ui_scoring(
    mut egui_context: ResMut<EguiContext>,
    mut camera: Query<(&GlobalTransform, &Camera), With<GameCamera>>,
    mut q_scores: Query<(&ScoreNear, &Transform, &ScoreNearDef)>,
    mut game_state: ResMut<State<GameState>>,
    windows: Query<Entity, With<Window>>,
) {
    let camera = camera.single();
    for (i, (score, transform, def)) in q_scores.iter().enumerate() {
        let Some(position) = camera.1.world_to_viewport(camera.0, transform.translation) else {
            continue;
        };
        egui::Area::new(format!("score {i}"))
            .fixed_pos(egui::pos2(0f32, 0f32))
            .anchor(
                Align2::LEFT_BOTTOM,
                egui::Vec2::new(position.x, -position.y),
            )
            .show(
                egui_context.ctx_for_window_mut(windows.iter().next().unwrap()),
                |ui| match score {
                    ScoreNear::Scoring(_) => {
                        ui.label("scoring");
                    }
                    ScoreNear::NotNear => {
                        ui.label(format!("enemy ({})", def.score));
                    }
                    ScoreNear::Gained => {
                        ui.label("enemy");
                    }
                },
            );
    }
}

pub(super) fn ui_score(
    mut egui_context: ResMut<EguiContext>,
    mut score: ResMut<Score>,
    windows: Query<Entity, With<Window>>,
) {
    egui::Area::new(format!("score for player"))
        .fixed_pos(egui::pos2(0f32, 0f32))
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(
            egui_context.ctx_for_window_mut(windows.iter().next().unwrap()),
            |ui| {
                ui.label(RichText::new(score.score.to_string()).font(FontId::proportional(40.0)));
            },
        );
}
