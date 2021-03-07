use crate::web::templates::{LeaderboardItem, SvcInfo, TeamInfo};
use sqlx::{Acquire, Postgres};
use tide_sqlx::ConnectionWrapInner;

pub async fn get_team_info(
	conn: &mut ConnectionWrapInner<Postgres>,
) -> Vec<TeamInfo> {
	sqlx::query_as!(
		TeamInfo,
		r#"
			SELECT	services.team_id, services.vm_id, services.svc_id,
						services.check_count, services.uptime_score, services.sla_count,
						services.latest_uptime_status
						FROM	teams
			INNER 	JOIN	services ON services.team_id = teams.team_id
			ORDER BY	services.team_id ASC, services.svc_id DESC;
		"#
	)
	.fetch_all(conn.acquire().await.unwrap())
	.await
	.unwrap()
}

pub async fn get_all_services(
	conn: &mut ConnectionWrapInner<Postgres>,
) -> Vec<SvcInfo> {
	sqlx::query_as!(
		SvcInfo,
		r#"
			SELECT DISTINCT vm_id, svc_id FROM services;
		"#
	)
	.fetch_all(conn.acquire().await.unwrap())
	.await
	.unwrap()
}

pub async fn get_leaderboard(
	conn: &mut ConnectionWrapInner<Postgres>,
) -> Vec<LeaderboardItem> {
	sqlx::query_as!(
		LeaderboardItem,
		r#"
			SELECT team_id, SUM(uptime_score - (sla_count * 1))
				FROM services
				GROUP BY team_id
				ORDER BY sum DESC;
		"#
	)
	.fetch_all(conn.acquire().await.unwrap())
	.await
	.unwrap()
}
