use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, FontId, RichText},
    EguiContexts,
};

use crate::utils::mouse::GameCamera;

use super::{
    scoring::{Score, ScoreNear, ScoreNearDef},
    GameState,
};

pub(super) fn ui_playing(mut ctxs: EguiContexts, mut game_state: ResMut<NextState<GameState>>) {
    egui::Area::new("Game UI")
        .fixed_pos(egui::pos2(0.0, -150f32))
        .anchor(Align2::CENTER_TOP, egui::Vec2::ZERO)
        .show(ctxs.ctx_mut(), |ui| {
            if ui.button("Forfeit").clicked() {
                game_state.set(GameState::Warmup);
            }
        });
}
pub(super) fn ui_scoring(
    mut ctxs: EguiContexts,
    camera: Query<(&GlobalTransform, &Camera), With<GameCamera>>,
    q_scores: Query<(&ScoreNear, &Transform, &ScoreNearDef)>,
) {
    let camera = camera.single();
    for (i, (score, transform, def)) in q_scores.iter().enumerate() {
        let Some(position) = camera.1.world_to_viewport(camera.0, transform.translation) else {
            continue;
        };
        egui::Area::new(format!("score {i}"))
            .fixed_pos(egui::pos2(0f32, 0f32))
            .anchor(Align2::LEFT_TOP, egui::Vec2::new(position.x, position.y))
            .show(ctxs.ctx_mut(), |ui| match score {
                ScoreNear::Scoring(_) => {
                    ui.label("scoring");
                }
                ScoreNear::NotNear => {
                    ui.label(format!("enemy ({})", def.score));
                }
                ScoreNear::Gained => {
                    ui.label("enemy");
                }
            });
    }
}

pub(super) fn ui_score(mut ctxs: EguiContexts, score: ResMut<Score>) {
    egui::Area::new("score for player")
        .fixed_pos(egui::pos2(0f32, 0f32))
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctxs.ctx_mut(), |ui| {
            ui.label(RichText::new(score.score.to_string()).font(FontId::proportional(40.0)));
        });
}
