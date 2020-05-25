use crate::db;
use warp_ws_server::*;
use diesel_utils::{PgConn,};
use crate::WSConnections_;
use uuid::Uuid;
use itertools::Itertools;
use crate::subscriptions::*;
use crate::schema;
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros
use crate::diesel::ExpressionMethods;
use crate::types::{competitions::*, series::*, teams::*, matches::*, results::*, players::*};
use crate::messages::*;


pub async fn insert_competitions(method: &str, message_id: Uuid, data: Vec<ApiCompetition>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
    // It's possible to just borrow it in db-insertion,
    // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
    ApiCompetition::insert(conn, data.clone()).await?;
    // TODO ideally would return response before awaiting publishing going out
    if let Err(e) = publish::<SubType, ApiCompetition>(
        ws_conns, &data, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    println!("{:?}", &data);
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_competitions(method: &str, message_id: Uuid, data: Vec<CompetitionUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let data: Vec<Competition> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, competitions, competition_id, c)
    }).collect::<Result<Vec<Competition>, _>>()})?;
    // TODO ideally would return response before awaiting publishing going out
    if let Err(e) = publish::<SubType, Competition>(
        ws_conns, &data, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_series(method: &str, message_id: Uuid, data: Vec<ApiSeriesNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
        // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
    // It's possible to just borrow it in db-insertion,
    // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
    let inserted_ser = ApiSeriesNew::insert(&conn, data.clone())?;
    let to_publish_rows = db::get_publishable_series(&conn, inserted_ser)?;
    let to_publish = ApiCompetition::from_rows(to_publish_rows);
    if let Err(e) = publish::<SubType, ApiCompetition>(
        ws_conns, &to_publish, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_series(method: &str, message_id: Uuid, data: Vec<SeriesUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Series> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, series, series_id, c)
    }).collect::<Result<Vec<Series>, _>>()})?;
    let add_empty_shit = out.into_iter().map(|m| (m, vec![], vec![])).collect();
    let to_publish_rows = db::get_publishable_series(&conn, add_empty_shit)?;
    let to_publish = ApiCompetition::from_rows(to_publish_rows);
    // TODO ideally would return response before awaiting publishing going out
    if let Err(e) = publish::<SubType, ApiCompetition>(
        ws_conns, &to_publish, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_matches(method: &str, message_id: Uuid, data: Vec<ApiMatchNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
    // It's possible to just borrow it in db-insertion,
    // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
    let new_matches = ApiMatchNew::insert(&conn, data.clone())?;
   // let comps: Vec<Competition> = insert!(&conn, schema::competitions::table, data)?;
    // assume anything upserted the user wants to subscribe to
    // TODO ideally would return response before awaiting publishing going out
    let to_publish_rows = db::get_publishable_matches(&conn, new_matches)?;
    let to_publish = ApiCompetition::from_match_rows(to_publish_rows);
    if let Err(e) = publish::<SubType, ApiCompetition>(
        ws_conns, &to_publish, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_matches(method: &str, message_id: Uuid, data: Vec<MatchUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Match> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, matches, match_id, c)
    }).collect()})?;
    // TODO a bit ugly hacking in the empty vecs. improve
    let to_publish_rows = db::get_publishable_matches(&conn, out.into_iter().map(|m| (m, vec![], vec![])).collect())?;
    let to_publish = ApiCompetition::from_match_rows(to_publish_rows);
    if let Err(e) = publish::<SubType, ApiCompetition>(
        ws_conns, &to_publish, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_series_results(method: &str, message_id: Uuid, data: Vec<TeamSeriesResult>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let new: Vec<TeamSeriesResult> = insert!(&conn, schema::team_series_results::table, &data)?;
    let to_publish_rows = db::get_publishable_team_series_results(&conn, new)?;
    let to_publish = ApiCompetition::from_rows(to_publish_rows);
    if let Err(e) = publish::<SubType, ApiCompetition>(
        ws_conns, &to_publish, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    //publish::<SubType, TeamSeriesResult>(ws_conns, &data, SubType::Competition, Some(comp_to_series_ids)).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_team_series_results(method: &str, message_id: Uuid, data: Vec<TeamSeriesResultUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<TeamSeriesResult> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update_2pkey!(&conn, team_series_results, series_id, team_id, c)
    }).collect::<Result<Vec<TeamSeriesResult>, _>>()})?;
    let to_publish_rows = db::get_publishable_team_series_results(&conn, out)?;
    let to_publish = ApiCompetition::from_rows(to_publish_rows);
    if let Err(e) = publish::<SubType, ApiCompetition>(
        ws_conns, &to_publish, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_match_results(method: &str, message_id: Uuid, data: Vec<TeamMatchResult>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let new: Vec<TeamMatchResult> = insert!(&conn, schema::team_match_results::table, &data)?;
    let to_publish_rows = db::get_publishable_team_match_results(&conn, new)?;
    let to_publish = ApiCompetition::from_opty_rows(to_publish_rows);
    if let Err(e) = publish::<SubType, ApiCompetition>(
        ws_conns, &to_publish, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_team_match_results(method: &str, message_id: Uuid, data: Vec<TeamMatchResultUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<TeamMatchResult> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update_2pkey!(&conn, team_match_results, match_id, team_id, c)
    }).collect::<Result<Vec<TeamMatchResult>, _>>()})?;
    let to_publish_rows = db::get_publishable_team_match_results(&conn, out)?;
    let to_publish = ApiCompetition::from_opty_rows(to_publish_rows);
    if let Err(e) = publish::<SubType, ApiCompetition>(
        ws_conns, &to_publish, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_results(method: &str, message_id: Uuid, data: Vec<PlayerResult>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let new: Vec<PlayerResult> = insert!(&conn, schema::player_results::table, &data)?;
    let to_publish_rows = db::get_publishable_player_results(&conn, new)?;
    let to_publish = ApiCompetition::from_opty_rows(to_publish_rows);
    if let Err(e) = publish::<SubType, ApiCompetition>(
        ws_conns, &to_publish, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_player_results(method: &str, message_id: Uuid, data: Vec<PlayerResultUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<PlayerResult> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update_2pkey!(&conn, player_results, match_id, player_id, c)
    }).collect()})?;
    let to_publish_rows = db::get_publishable_player_results(&conn, out)?;
    let to_publish = ApiCompetition::from_opty_rows(to_publish_rows);
    if let Err(e) = publish::<SubType, ApiCompetition>(
        ws_conns, &to_publish, SubType::Competition, None
    ).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_teams(method: &str, message_id: Uuid, data: Vec<ApiTeam>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    ApiTeam::insert(conn, data.clone())?;
    let to_publish = ApiTeamWithPlayersHierarchy::from_api_team(data);
    if let Err(e) = publish::<SubType, ApiTeamWithPlayersHierarchy>(ws_conns, &to_publish, SubType::Team, None).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_teams(method: &str, message_id: Uuid, data: Vec<TeamUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Team> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update!(&conn, teams, team_id, c)
    }).collect()})?;
    let to_publish = ApiTeamWithPlayersHierarchy::from_team(out);
    if let Err(e) = publish::<SubType, ApiTeamWithPlayersHierarchy>(ws_conns, &to_publish, SubType::Team, None).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_players(method: &str, message_id: Uuid, data: Vec<ApiPlayer>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    ApiPlayer::insert(&conn, data.clone())?;
    let to_publish = db::get_teams_from_players(&conn, Some(data.iter().map(|p| p.player_id).collect_vec()))?;
    if let Err(e) = publish::<SubType, ApiTeamWithPlayersHierarchy>(ws_conns, &to_publish, SubType::Team, None).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_players(method: &str, message_id: Uuid, data: Vec<PlayerUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Player> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update!(&conn, players, player_id, c)
    }).collect::<Result<Vec<Player>, _>>()})?;
    let to_publish = db::get_teams_from_players(&conn, Some(out.iter().map(|p| p.player_id).collect_vec()))?;
    if let Err(e) = publish::<SubType, ApiTeamWithPlayersHierarchy>(ws_conns, &to_publish, SubType::Team, None).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_players(method: &str, message_id: Uuid, data: Vec<ApiTeamPlayer>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_team_players(&conn, &data)?;
    let to_publish = db::get_teams_from_players(&conn, Some(out.iter().map(|p| p.player_id).collect_vec()))?;
    if let Err(e) = publish::<SubType, ApiTeamWithPlayersHierarchy>(ws_conns, &to_publish, SubType::Team, None).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_names(method: &str, message_id: Uuid, data: Vec<ApiTeamNameNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_team_names(&conn, data)?;
    let to_publish = db::get_teams_names(&conn, out.into_iter().map(|t|t.team_id).collect_vec())?;
    if let Err(e) = publish::<SubType, ApiTeamWithPlayersHierarchy>(ws_conns, &to_publish, SubType::Team, None).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_names(method: &str, message_id: Uuid, data: Vec<ApiPlayerNameNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_player_names(&conn, &data)?;
    //let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    let to_publish = db::get_teams_from_players(&conn, Some(out.iter().map(|p| p.player_id).collect_vec()))?;
    if let Err(e) = publish::<SubType, ApiTeamWithPlayersHierarchy>(ws_conns, &to_publish, SubType::Team, None).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_positions(method: &str, message_id: Uuid, data: Vec<ApiPlayerPositionNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_player_positions(&conn, &data)?;
    let to_publish = db::get_teams_from_players(&conn, Some(out.iter().map(|p| p.player_id).collect_vec()))?;
    if let Err(e) = publish::<SubType, ApiTeamWithPlayersHierarchy>(ws_conns, &to_publish, SubType::Team, None).await{
        println!("Error publishing: {:?}", e);
    };
    let resp_msg = WSMsgOut::resp(message_id, method, to_publish);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}
// TODO Prob need some deletions

pub async fn sub_competitions(method: &str, message_id: Uuid, data: SubCompetition, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
        // let ws_user = ws_conns.lock().await.get_mut(&user_ws_id).ok_or("Webscoket gone away")?;
    // why does this need splitting into two lines?
    // ANd is it holding the lock for this whole scope? doesnt need to
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if let Some(toggle) = data.all{
        sub_all(&SubType::Competition, ws_user, toggle).await;
    }
    if let Some(competition_ids) = data.sub_competition_ids{
        sub(&SubType::Competition, ws_user, competition_ids.iter()).await;
    }
    if let Some(competition_ids) = data.unsub_competition_ids{
        unsub(&SubType::Competition, ws_user, competition_ids.iter()).await;
    }
    let subscription = ws_user.subscriptions.get_ez(&SubType::Competition);
    let rows = match subscription.all{
        true => {
            db::get_full_competitions(&conn, None)
        },
        false => {
            db::get_full_competitions(&conn, Some(subscription.ids.iter().collect()))
        }
    }?;
    let data = ApiCompetition::from_rows(rows);
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn sub_teams(method: &str, message_id: Uuid, data: SubTeam, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if data.toggle{
        sub_all(&SubType::Team, ws_user, data.toggle).await;
    }
    // Not supporting subbing to "some" teams yet
    // if let Some(competition_ids) = data.sub_competition_ids{
    //     sub(&SubType::Team, ws_user, competition_ids.iter()).await;
    // }
    // if let Some(competition_ids) = data.unsub_competition_ids{
    //     unsub(&SubType::Team, ws_user, competition_ids.iter()).await;
    // }
    let subscription = ws_user.subscriptions.get_ez(&SubType::Competition);
    let data = match subscription.all{
        true => {
            db::get_teams_from_players(&conn, None)
        },
        false => Ok(vec![])
    }?;
    println!("resp_msg sub_teams: {:?}", data);
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}