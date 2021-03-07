use crate::config::{Cfg, Inject};
use askama::Template;
use chrono::{Duration, Utc};
use core_extensions::slices::ValSliceExt;

#[derive(Template)]
#[template(path = "scores.html")]
pub struct Scores<'a> {
	pub round: &'a str,
	pub info: Vec<(String, Vec<TeamInfo>)>,
	pub services: Vec<SvcInfo>,
	pub injects: Vec<(Inject, String)>,
	pub mode: TplMode,
}

#[derive(Template)]
#[template(path = "patch.html")]
pub struct PatchServer<'a> {
	pub round: &'a str,
	pub files: Vec<String>,
}

pub enum TplMode {
	Scores,
	Uptime,
	SLAs,
}

#[derive(Clone)]
pub struct TeamInfo {
	pub team_id: String,
	pub vm_id: String,
	pub svc_id: String,
	pub check_count: i32,
	pub uptime_score: i32,
	pub sla_count: i32,
	pub latest_uptime_status: bool,
}

impl TeamInfo {
	pub fn is_positive(&self) -> bool { self.get_percentage() > 50. }

	pub fn get_percentage(&self) -> f64 {
		((self.uptime_score as f64 / self.check_count as f64) * 100.).round()
	}
}

pub struct SvcInfo {
	pub svc_id: String,
	pub vm_id: String,
}

#[derive(Template)]
#[template(path = "leaderboard.html")]
pub struct Leaderboard<'a> {
	pub round: &'a str,
	pub teams: Vec<LeaderboardItem>,
}

pub struct LeaderboardItem {
	pub team_id: String,
	pub sum: Option<i64>,
}

pub fn info_to_scores(
	cfg: &Cfg,
	teams: Vec<TeamInfo>,
	services: Vec<SvcInfo>,
	mode: TplMode,
) -> Scores {
	Scores {
		round: &cfg.round,
		info: teams
			.split_while(|x| &x.team_id)
			.map(|x| (x.key.to_owned(), x.slice.to_vec()))
			.collect::<Vec<(String, Vec<TeamInfo>)>>(),
		services,
		injects: cfg
			.injects
			.iter()
			.filter(|i| {
				(cfg.start + Duration::minutes(i.offset as i64)) < Utc::now()
			})
			.map(|i| {
				(
					i,
					(cfg.start
						+ Duration::minutes((i.offset + i.duration) as i64))
					.to_string(),
				)
			})
			.map(|(i, s)| (i.to_owned(), s))
			.collect::<Vec<_>>(),
		mode,
	}
}
