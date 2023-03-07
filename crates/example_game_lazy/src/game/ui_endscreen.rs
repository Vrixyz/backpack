use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    egui::{self, Align2, Color32, FontId, RichText},
    EguiContext,
};

use super::{scoring::Score, GameState};

pub(super) fn ui_endscreen(
    egui_ctx: Query<&EguiContext, With<PrimaryWindow>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    egui::Area::new("Game Endscreen")
        .fixed_pos(egui::pos2(0.0, -150f32))
        .anchor(Align2::CENTER_TOP, egui::Vec2::ZERO)
        .show(egui_ctx.single(), |ui| {
            if ui.button("New Game").clicked() {
                game_state.set(GameState::Warmup);
            }
        });
}
pub(super) fn ui_end_title_and_score(
    egui_ctx: Query<&EguiContext, With<PrimaryWindow>>,
    score: ResMut<Score>,
) {
    egui::Area::new("end juice")
        .fixed_pos(egui::pos2(0f32, 0f32))
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(egui_ctx.single(), |ui| {
            ui.colored_label(
                Color32::RED,
                RichText::new(format!("GAME OVER ({})", score.score))
                    .font(FontId::proportional(80.0)),
            );
        });
}
