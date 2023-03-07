use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use bevy_jornet::{JornetPlugin, Leaderboard};

use super::GameState;

pub struct ScoreboardPlugin;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum LeaderboardScreen {
    Hidden,
    Show,
}

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(LeaderboardScreen::Hidden);
        app.add_plugin(JornetPlugin::with_leaderboard(
            &std::env::var("JORNET_ID").expect("No jornet id provided."),
            &std::env::var("JORNET_SECRET").expect("No jornet secret provided."),
        ));
        app.add_startup_system(leaderboard_setup);
        app.add_system_set(
            SystemSet::on_update(LeaderboardScreen::Show)
                .with_system(ui_leaderboard)
                .with_system(refresh_leaderboard),
        );
        app.add_system_set(SystemSet::on_enter(GameState::EndScreen).with_system(show_leaderboard));
        app.add_system_set(SystemSet::on_exit(GameState::EndScreen).with_system(hide_leaderboard));
    }
}

fn leaderboard_setup(mut leaderboard: ResMut<Leaderboard>) {
    // `None` will create a new user with a random name
    leaderboard.create_player(None);
    leaderboard.refresh_leaderboard();
}

fn show_leaderboard(
    mut leaderboard_screen: ResMut<State<LeaderboardScreen>>,
    leaderboard: Res<Leaderboard>,
) {
    leaderboard.refresh_leaderboard();
    leaderboard_screen.set(LeaderboardScreen::Show);
}
fn refresh_leaderboard(
    mut local_timer: Local<Timer>,
    time: Res<Time>,
    leaderboard: Res<Leaderboard>,
) {
    if local_timer.duration() == Duration::default() {
        local_timer.set_duration(Duration::from_secs(5));
        local_timer.set_mode(TimerMode::Repeating);
    }
    local_timer.tick(time.delta());
    if local_timer.just_finished() {
        leaderboard.refresh_leaderboard();
    }
}

fn hide_leaderboard(mut leaderboard_screen: ResMut<State<LeaderboardScreen>>) {
    leaderboard_screen.set(LeaderboardScreen::Hidden);
}

fn ui_leaderboard(mut egui_context: ResMut<EguiContext>, leaderboard: Res<Leaderboard>) {
    egui::Window::new("leaderboard").show(egui_context.ctx_mut(), |ui| {
        let scores = leaderboard.get_leaderboard();
        let local_player = leaderboard.get_player().map(|player| &player.name);
        egui::ScrollArea::vertical().show(ui, |ui| {
            for score in scores {
                match local_player {
                    Some(name) if name == &score.player => {
                        ui.colored_label(
                            egui::Color32::LIGHT_BLUE,
                            format!("{}: {}", score.player, score.score),
                        );
                    }
                    _ => {
                        ui.label(format!("{}: {}", score.player, score.score));
                    }
                }
            }
        });
    });
}
