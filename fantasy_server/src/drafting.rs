use crate::db;
use crate::diesel::RunQueryDsl; // imported here so that can run db macros
use crate::schema;
use crate::types::{drafts::*, fantasy_teams::*, leagues::*};
use diesel;
use diesel_utils::*;
use std::collections::{HashSet, HashMap};
use chrono::{Utc, self};
use uuid::Uuid;
use tokio::sync::Notify;
use tokio::time::delay_for;
use tokio::sync::Mutex;
use std::sync::Arc;
use futures::join;
use rand::seq::SliceRandom;
use rand;
use warp_ws_server::{publish, BoxError};
use std::fmt;
use crate::WSConnections_;
use crate::subscriptions::SubType;
use crate::errors;

#[derive(Debug, Clone)]
struct NoValidPicksError {
}

impl fmt::Display for NoValidPicksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No valid picks to choose from")
    }
}

impl std::error::Error for NoValidPicksError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub async fn draft_builder(pg_pool: PgPool, mut ws_conns: WSConnections_) {
    // https://docs.rs/tokio-core/0.1.17/tokio_core/reactor/struct.Timeout.html
    // https://doc.rust-lang.org/stable/rust-by-example/std_misc/channels.html
    //https://medium.com/@polyglot_factotum/rust-concurrency-patterns-communicate-by-sharing-your-sender-11a496ce7791
    // https://docs.rs/tokio/0.2.20/tokio/sync/struct.Notify.html
    println!("In draft builder");
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();
    // TODO handle error
    let mut timeout: Option<chrono::Duration> = None;

    'outer: loop {
        println!("timeout: {:?}", timeout);
        //let conn = pg_pool.clone().get().unwrap();
        match db::get_undrafted_periods(pg_pool.get().unwrap()){
            // Want this to be resilient/not bring down whole service.....
            // maybe it should just be a separate service/binary.
            // separate binary makes sense
            Err(e) => {
                println!("db::get_undrafted_periods(&conn) went wrong");
                continue
            },
            Ok(all_undrafted) => {
                'inner: for undrafted in all_undrafted.into_iter(){
                    let time_to_draft_gen = undrafted.draft_lockdown - Utc::now();
                    if (time_to_draft_gen) < chrono::Duration::zero(){
                        match generate_drafts(pg_pool.get().unwrap(), undrafted){
                            Ok(drafts) => {
                                // TODO publkish league as well
                                match publish::<SubType, ApiDraft>(&mut ws_conns, &drafts, SubType::Draft, None).await{
                                    Err(e) => println!("Error publishing drafts: {:?}", e),
                                    _ => {}
                                }
                            },
                            Err(e) => println!("{:?}", e)
                        };
                    } else{
                        timeout = Some(time_to_draft_gen);
                        continue 'outer;  // If we didnt use up all drafts, dont want to re-set timeout to None below.
                    }
                };
                timeout = None;
            }
        }
        // If we get an external notify in this wait time,
        // the timeout will still trigger, so we'll do an extra pass.
        // I think that's ok though, as it'll just be one, and it'll be the time when we should we processing the next draft anyway.
        // Overall logic handles accidental wakeups fine, it just realises too early, sets the timeout, and waits next loop.
        let wait_task = match timeout{
            // std::time::Duration
            Some(t) => {
                let notify3 = notify.clone();
                tokio::task::spawn_local(async move {
                    println!("delay_for");
                    delay_for(t.to_std().expect("Time conversion borkled!")).await;
                    notify3.notify();
                })
                //Timeout::new(t.to_std().expect("Wrong timeline!"), &||notify2.notify());
            },
            None => {
                // TODO dumb placeholder. Really want to `cancel` this task if other thing notified.
                tokio::spawn(async move {
                    println!("in dummy");
                    delay_for(std::time::Duration::from_millis(1)).await;
                })
            }
        };
        let waity_waity = || notify2.notified();
        println!("pre join!(waity_waity(), wait_task)");
        join!(waity_waity(), wait_task);
        println!("post join!(waity_waity(), wait_task)");
    }
}

pub fn generate_drafts(
    conn: PgConn,
    period: Period,
) -> Result<Vec<ApiDraft>, diesel::result::Error> {
    println!("In generate_drafts");
    conn.build_transaction().run(||{
        // TODO can build in more efficient way, rather than adding entries 1-by-1
        let teams = db::get_randomised_teams_for_league(&conn, period.league_id)?;
        let squad_size = db::get_league_squad_size(&conn, period.league_id)? as usize;
        let num_teams = teams.len();
        let num_drafts = period.teams_per_draft as usize / teams.len() as usize + 1;
        let draft_map: HashMap<usize, Vec<FantasyTeam>> =
            teams
                .into_iter()
                .enumerate()
                .fold(HashMap::new(), |mut hm, (i, team)| {
                    let draft_num = i % num_drafts;
                    match hm.get_mut(&draft_num) {
                        Some(v) => {
                            v.push(team);
                        }
                        None => {
                            hm.insert(draft_num, vec![team]);
                        }
                    };
                    hm
                });
        // maybe need to fold results
        let drafts: Result<Vec<_>, _> = draft_map
            .into_iter()
            .map(|(_, teams)| {
                let drafts: Vec<Draft> = insert!(
                    &conn,
                    schema::drafts::table,
                    vec![Draft::new(period.period_id)]
                )?;
                let draft = drafts.first().unwrap();
                let team_drafts: Vec<TeamDraft> = teams
                    .iter()
                    .map(|team| TeamDraft::new(draft.draft_id, team.fantasy_team_id))
                    .collect();
                let _: Vec<TeamDraft> = insert!(&conn, schema::team_drafts::table, &team_drafts)?;
                //let reversed_team_drafts: Vec<TeamDraft> = team_drafts.reverse().collect();
                let mut choices: Vec<ApiDraftChoice> = Vec::with_capacity(squad_size * num_teams);
                for round in 0..squad_size {
                    let make_choices = |(i, t): (usize, &TeamDraft)| {
                        let start = period.draft_start
                            + chrono::Duration::seconds(
                                (round * num_teams + i) as i64 * period.draft_interval_secs as i64,
                            );
                        let end = start + chrono::Duration::seconds(period.draft_interval_secs as i64);
                        let timespan = new_dieseltimespan(start, end);
                        choices.push(ApiDraftChoice::new(
                            t.fantasy_team_id,
                            t.team_draft_id,
                            timespan,
                        ));
                    };
                    match round % 2 {
                        0 => team_drafts.iter().enumerate().for_each(make_choices),
                        1 => team_drafts.iter().rev().enumerate().for_each(make_choices),
                        _ => panic!("Maths is fucked"),
                    };
                }
                // could use frunk rather than .into()
                // was just removing potential errors when i accidentally made infinite recrusion
                // it was due to putting the to_insert expression straight into the macro
                let to_insert: Vec<DraftChoice> = choices.iter().map(|c| c.clone().into()).collect();
                let _: Vec<DraftChoice> = insert!(&conn, schema::draft_choices::table, to_insert)?;
                let out = ApiDraft {
                    league_id: period.league_id,
                    draft_id: draft.draft_id,
                    period_id: period.period_id,
                    meta: draft.meta.clone(),
                    choices: choices,
                };
                Ok(out)
            })
            .collect();
            drafts
    })
}


pub async fn draft_handler(
    pg_pool: PgPool, 
    player_position_cache_mut: Arc<Mutex<Option<HashMap<Uuid, &String>>>>, 
    player_team_cache_mut: Arc<Mutex<Option<HashMap<Uuid, Uuid>>>>, 
    mut ws_conns: WSConnections_
) {
    println!("In draft builder");
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();
    // TODO handle error
    let mut timeout: Option<chrono::Duration> = None;

    'outer: loop {
        println!("timeout: {:?}", timeout);
        match db::get_unchosen_draft_choices(pg_pool.get().unwrap()){
            // Want this to be resilient/not bring down whole service.....
            // maybe it should just be a separate service/binary.
            // separate binary makes sense
            Err(e) => {
                println!("db::get_unchosen_draft_choices(&conn) went wrong");
                continue
            },
            Ok(all_unchosen) => {
                'inner: for unchosen in all_unchosen.into_iter(){
                    let (draft_choice, period, team_draft, league) = unchosen;
                    let raw_time = DieselTimespan::upper(draft_choice.timespan);
                    let time_to_unchosen = raw_time - Utc::now();
                    if (time_to_unchosen) < chrono::Duration::zero(){
                        let pick_straight_into_team = league.team_size == league.squad_size;
                        let player_position_cache_opt = player_position_cache_mut.lock().await;
                        let player_team_cache_opt = player_team_cache_mut.lock().await;
                        match (*player_position_cache_opt, *player_team_cache_opt){
                            (Some(ref player_position_cache), Some(ref player_team_cache)) => {
                                let out: Result<(Pick, Option<ActivePick>), BoxError> = pick_from_queue_or_random(
                                    pg_pool.get().unwrap(), team_draft.fantasy_team_id, draft_choice, period.timespan, period.period_id,
                                    &league.max_squad_players_same_team, &league.max_squad_players_same_position,
                                    pick_straight_into_team,
                                    player_position_cache, player_team_cache
                                );
                                match out
                                {
                                    Ok((pick, None)) => {
                                        match publish::<SubType, Pick>(&mut ws_conns, &vec![pick], SubType::Draft, None).await{
                                            Err(e) => println!("Error publishing draft picks: {:?}", e),
                                            _ => {}
                                        }
                                    },
                                    Ok((pick, Some(active_pick))) => {
                                        match publish::<SubType, Pick>(&mut ws_conns, &vec![pick], SubType::Draft, None).await{
                                            Err(e) => println!("Error publishing draft picks: {:?}", e),
                                            _ => {}
                                        };
                                        match publish::<SubType, ActivePick>(&mut ws_conns, &vec![active_pick], SubType::Draft, None).await{
                                            Err(e) => println!("Error publishing draft picks: {:?}", e),
                                            _ => {}
                                        }
                                    },
                                    Err(e) => println!("{:?}", e)
                                };
                            },
                            // will this fuck up? aka not retry the choice?
                            // I think it will retry the choice as will still be classes as unchosen, 
                            // only minor problem might be that whilst iterating the unchosen, this gets populated, and so a later pick gets priority, before we outer-loop again,
                            // and come back to this....thats a good point actually, it just queries once for unchosen and then goes through, 
                            // its not like this is getting thrown back on rear of queue, it will wait until the next round of processing (could be long time), prob needs a rethink.
                            _ => {println!("teams_and_players still empty")}
                        }
                    } else{
                        timeout = Some(time_to_unchosen);
                        continue 'outer;  // If we didnt use up all drafts, dont want to re-set timeout to None below.
                    }
                };
                timeout = None;
            }
        }
        // If we get an external notify in this wait time,
        // the timeout will still trigger, so we'll do an extra pass.
        // I think that's ok though, as it'll just be one, and it'll be the time when we should we processing the next draft anyway.
        // Overall logic handles accidental wakeups fine, it just realises too early, sets the timeout, and waits next loop.
        let wait_task = match timeout{
            // std::time::Duration
            Some(t) => {
                let notify3 = notify.clone();
                tokio::task::spawn_local(async move {
                    println!("delay_for");
                    delay_for(t.to_std().expect("Time conversion borkled!")).await;
                    notify3.notify();
                })
                //Timeout::new(t.to_std().expect("Wrong timeline!"), &||notify2.notify());
            },
            None => {
                // TODO dumb placeholder. Really want to `cancel` this task if other thing notified.
                tokio::spawn(async move {
                    println!("in dummy");
                    delay_for(std::time::Duration::from_millis(1)).await;
                })
            }
        };
        let waity_waity = || notify2.notified();
        println!("pre join!(waity_waity(), wait_task)");
        join!(waity_waity(), wait_task);
        println!("post join!(waity_waity(), wait_task)");
    }
    
}

pub fn pick_from_queue_or_random(
    conn: PgConn,
    fantasy_team_id: Uuid,
    unchosen: DraftChoice,
    belongs_to_team_for: DieselTimespan,
    period_id: Uuid,
    max_squad_players_same_team: &i32,
    max_squad_players_same_position: &i32,
    pick_straight_into_team: bool,
    player_position_cache: &HashMap<Uuid, &String>,
    player_team_cache: &HashMap<Uuid, Uuid>
) -> Result<(Pick, Option<ActivePick>), BoxError>{
    conn.build_transaction().run(||{
        // TODO deal with squads not just teams
        let draft_choice_id = unchosen.draft_choice_id;
        let draft_queue = db::get_draft_queue_for_choice(&conn, unchosen)?;
        // If its a hashmap rather than set, can include player-info grabbed from other api.
        // use vec and set, because want fast-lookup when looping through draft queue,
        // but also need access random element if !picked
        //teams_and_players_mut
        let valid_remaining_picks: Vec<Uuid> = db::get_valid_picks(&conn, period_id)?;
        let (positions, teams) = (player_position_cache, player_team_cache);
        let current_squad = db::get_current_picks(&conn, fantasy_team_id, period_id)?;
        let (mut position_counts, mut team_counts): (HashMap<&String, i32>, HashMap<Uuid, i32>) = (HashMap::new(), HashMap::new());
        // TODO rust defaultdict?
        let (position_counts, team_counts) = position_team_counts(
            current_squad, player_position_cache, player_team_cache
        );
        let banned_teams: HashSet<Uuid> = team_counts.into_iter().filter(|(_, count)| count > max_squad_players_same_team).map(|(team, _)| team).collect();
        let banned_positions: HashSet<&String> = position_counts.into_iter().filter(|(_, count)| count > max_squad_players_same_position).map(|(pos, _)| pos).collect();
        let valid_remaining_picks_hash: HashSet<&Uuid> = valid_remaining_picks.iter().collect();
        // TODO do dumb then tidy
        let mut new_pick: Option<Pick> = None;
        for pick_id in draft_queue{
            if let Some(valid_pick_id) = valid_remaining_picks_hash.get(&pick_id) 
            {
                if (banned_positions.get(positions.get(&pick_id).unwrap()).is_none())
                && (banned_teams.get(teams.get(&pick_id).unwrap()).is_none())
                {
                    let new_pick = Some(Pick{
                        pick_id: Uuid::new_v4(), fantasy_team_id: fantasy_team_id, draft_choice_id,
                        player_id: pick_id, timespan: belongs_to_team_for
                    });
                    break
                }
                
            }
        }
        if new_pick.is_none(){
            if let Some(random_choice) = valid_remaining_picks.choose(&mut rand::thread_rng()){
                new_pick = Some(Pick{
                    pick_id: Uuid::new_v4(), fantasy_team_id: fantasy_team_id, draft_choice_id,
                    player_id: *random_choice, timespan: belongs_to_team_for
                });
            };
        }
        //pick_straight_into_team
        match new_pick{
            Some(np) =>{
                let _: Vec<Pick> = insert!(&conn, schema::picks::table, vec![&np])?;
                if pick_straight_into_team{
                    let active_pick = ActivePick{active_pick_id: Uuid::new_v4(), pick_id: np.pick_id, timespan: belongs_to_team_for};
                    let _: Vec<ActivePick> = insert!(&conn, schema::active_picks::table, vec![&active_pick])?;
                    Ok((np, Some(active_pick)))
                } else{
                    Ok((np, None))
                }
            },
            None => Err(Box::new(NoValidPicksError{}) as BoxError)
        }
    })
    // we dont mutate the draft-queue. because maybe they want the same queue for future drafts
}

fn position_team_counts<'a>(
    player_ids: Vec<Uuid>, player_position_cache: &HashMap<Uuid, &'a String>, player_team_cache: &HashMap<Uuid, Uuid>
) -> (HashMap<&'a String, i32>, HashMap<Uuid, i32>){
    let (mut position_counts, mut team_counts): (HashMap<&String, i32>, HashMap<Uuid, i32>) = (HashMap::new(), HashMap::new());
    // TODO rust defaultdict?
    player_ids.iter().for_each(|pick_id|{
        // pretty sure these unwraps are super-safe as we've built the maps ourselves with these pick-ids.
        let (position, team) = (player_position_cache.get(&pick_id).unwrap(), player_team_cache.get(&pick_id).unwrap());
        match position_counts.get_mut(position) {
            Some(v) => {
                *v = *v + 1;
            }
            None => {
                position_counts.insert(position, 1);
            }
        };
        match team_counts.get_mut(team) {
            Some(v) => {
                *v = *v + 1;
            }
            None => {
                team_counts.insert(*team, 1);
            }
        };
    });
    (position_counts, team_counts)
}

pub fn verify_teams(
    all_teams: Vec<db::VecUuid>, 
    player_position_cache: &HashMap<Uuid, &String>,
    player_team_cache: &HashMap<Uuid, Uuid>,
    max_team_players_same_team: &i32,
    max_team_players_same_position: &i32,
    max_team_size: &i32
) -> Result<bool, BoxError>{
    for team in all_teams{
        let (position_counts, team_counts) = position_team_counts(
            team.inner, player_position_cache, player_team_cache
        );
        if team.length() > max_team_size {
            Err(errors::InvalidTeamError{description: format!("Team cannot be larger than {}", max_team_size)} as BoxError)
        }
        if position_counts.values().max() > max_team_players_same_position{
            Err(errors::InvalidTeamError{description: format!("Team cannot have more than {} players from same position", max_team_players_same_position)} as BoxError)
        }
        if team_counts.values().max() > max_team_players_same_team {
            Err(errors::InvalidTeamError{description: format!("Team cannot have more than {} players from same team", max_team_players_same_team)} as BoxError)
        }
    }
    Ok(true)
}