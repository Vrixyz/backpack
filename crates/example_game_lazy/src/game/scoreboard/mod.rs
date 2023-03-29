use std::{cmp::Ordering, time::Duration};

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContext, EguiContexts};
use bevy_jornet::{JornetPlugin, Leaderboard};
use dotenvy_macro::dotenv;

use super::GameState;

pub struct ScoreboardPlugin;

#[derive(Clone, Hash, PartialEq, Eq, Debug, States)]
pub enum LeaderboardScreen {
    Hidden,
    Show,
}

impl Default for LeaderboardScreen {
    fn default() -> Self {
        LeaderboardScreen::Hidden
    }
}

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        let jornet_id = dotenv!("JORNET_ID");
        let jornet_secret = dotenv!("JORNET_SECRET");

        app.add_state::<LeaderboardScreen>();
        app.add_plugin(JornetPlugin::with_leaderboard(jornet_id, jornet_secret));
        app.add_startup_system(leaderboard_setup);
        app.add_systems(
            (ui_leaderboard, refresh_leaderboard).in_set(OnUpdate(LeaderboardScreen::Show)),
        );
        app.add_system(show_leaderboard.in_schedule(OnEnter(GameState::EndScreen)));
        app.add_system(hide_leaderboard.in_schedule(OnExit(GameState::EndScreen)));
    }
}

fn leaderboard_setup(mut leaderboard: ResMut<Leaderboard>) {
    // `None` will create a new user with a random name
    leaderboard.create_player(None);
    leaderboard.refresh_leaderboard();
}

fn show_leaderboard(
    mut leaderboard_screen: ResMut<NextState<LeaderboardScreen>>,
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

fn hide_leaderboard(mut leaderboard_screen: ResMut<NextState<LeaderboardScreen>>) {
    leaderboard_screen.set(LeaderboardScreen::Hidden);
}

fn ui_leaderboard(mut ctxs: EguiContexts, leaderboard: Res<Leaderboard>) {
    egui::Window::new("leaderboard").show(ctxs.ctx_mut(), |ui| {
        let mut scores: Vec<_> = leaderboard
            .get_leaderboard()
            .into_iter()
            .filter(|p| p.score > 0f32)
            .collect();

        scores.sort_by(|s1, s2| s2.score.partial_cmp(&s1.score).unwrap_or(Ordering::Equal));
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
