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
use itertools::Itertools;

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

pub async fn draft_builder(pg_pool: PgPool, mut ws_conns: WSConnections_, new_draft_notifier: Arc<Notify>, draft_choices_notifier: Arc<Notify>) {
    // https://docs.rs/tokio-core/0.1.17/tokio_core/reactor/struct.Timeout.html
    // https://doc.rust-lang.org/stable/rust-by-example/std_misc/channels.html
    //https://medium.com/@polyglot_factotum/rust-concurrency-patterns-communicate-by-sharing-your-sender-11a496ce7791
    // https://docs.rs/tokio/0.2.20/tokio/sync/struct.Notify.html
    println!("In draft builder");
    // TODO handle error
    let mut timeout: Option<chrono::Duration> = None;

    loop {
        //let conn = pg_pool.clone().get().unwrap();
        match db::get_undrafted_periods(pg_pool.get().unwrap()){
            // Want this to be resilient/not bring down whole service.....
            // maybe it should just be a separate service/binary.
            // separate binary makes sense
            Err(e) => {
                println!("db::get_undrafted_periods(&conn) went wrong: {:?}", e);
                continue
            },
            Ok(all_undrafted) => {
                for undrafted in all_undrafted.into_iter(){
                    let time_to_draft_gen = undrafted.draft_lockdown - Utc::now();
                    if (time_to_draft_gen) < chrono::Duration::zero(){
                        match generate_drafts(pg_pool.get().unwrap(), undrafted){
                            Ok(drafts) => {
                                // TODO publkish league as well
                                match publish::<SubType, ApiDraft>(&mut ws_conns, &drafts, SubType::Draft, None).await{
                                    Err(e) => println!("Error publishing drafts: {:?}", e),
                                    _ => {}
                                }
                                draft_choices_notifier.notify();
                            },
                            Err(e) => println!("{:?}", e)
                        };
                    } else{
                        timeout = Some(time_to_draft_gen);
                        break;
                        //continue 'outer;  // If we didnt use up all drafts, dont want to re-set timeout to None below.
                    }
                };
                //timeout = None;
            }
        }
        // If we get an external notify in this wait time,
        // the timeout will still trigger, so we'll do an extra pass.
        // I think that's ok though, as it'll just be one, and it'll be the time when we should we processing the next draft anyway.
        // Overall logic handles accidental wakeups fine, it just realises too early, sets the timeout, and waits next loop.
        println!("Draft: generator timeout: {:?}", timeout);
        let wait_task = match timeout{
            // std::time::Duration
            Some(t) => {
                let new_draft_notifier = new_draft_notifier.clone();
                tokio::task::spawn(async move {
                    println!("delay_for");
                    delay_for(t.to_std().expect("Time conversion borkled!")).await;
                    new_draft_notifier.notify();
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
        let waity_waity = || new_draft_notifier.notified();
        //println!("Draft: generator pre join!(waity_waity(), wait_task)");
        println!("Draft: generator pre join!(waity_waity(), wait_task)");
        join!(waity_waity());
        // let (_, err) = join!(waity_waity(), wait_task);
        // if let Err(_) = err{
        //     println!("Unexpected task join error in draft builder")
        // };
        println!("Draft: generator post join!(waity_waity(), wait_task)");
        timeout = None;
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
        let num_drafts = ((num_teams - 1 ) / period.teams_per_draft as usize) + 1;
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
        let drafts_r: Result<Vec<Draft>, diesel::result::Error> = draft_map
            .into_iter()
            .map(|(_, teams)| {
                let drafts: Vec<Draft> = insert!(
                    &conn,
                    schema::drafts::table,
                    vec![Draft::new(period.period_id)]
                )?;
                let draft = drafts.into_iter().nth(0).unwrap();
                let team_drafts: Vec<TeamDraft> = teams
                    .iter()
                    .map(|team| TeamDraft::new(draft.draft_id, team.fantasy_team_id))
                    .collect();
                let _: Vec<TeamDraft> = insert!(&conn, schema::team_drafts::table, &team_drafts)?;
                //let reversed_team_drafts: Vec<TeamDraft> = team_drafts.reverse().collect();
                let mut choices: Vec<ApiDraftChoice> = Vec::with_capacity(squad_size * num_teams);
                let mut j = 0i64;
                for round in 0..squad_size {
                    let make_choices = |(i, t): (usize, &TeamDraft)| {
                        let start = period.draft_start
                            + chrono::Duration::seconds(
                                j * period.draft_interval_secs as i64,
                            );
                        let end = start + chrono::Duration::seconds(period.draft_interval_secs as i64);
                        let timespan = new_dieseltimespan(start, end);
                        choices.push(ApiDraftChoice::new(
                            t.fantasy_team_id,
                            t.team_draft_id,
                            timespan,
                        ));
                        j += 1;
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
                Ok(draft)
            })
            .collect();
            let drafts = drafts_r?;
            db::get_full_drafts(&conn, Some(drafts.iter().map(|d|&d.draft_id).collect_vec()))
    })
}

pub async fn publish_updated_pick(ws_conns: &mut WSConnections_, pick: Pick, draft_id: Uuid) -> Result<bool, BoxError>{
    let to_publish: Vec<ApiPick> = vec![ApiPick{
        pick_id: pick.pick_id, player_id: pick.player_id, timespan: pick.timespan, fantasy_team_id: Some(pick.fantasy_team_id), draft_id: Some(draft_id)
    }];
    publish::<SubType, ApiPick>(ws_conns, &to_publish, SubType::Draft, None).await
}


pub async fn draft_handler(
    pg_pool: PgPool, 
    player_position_cache_mut: Arc<Mutex<Option<HashMap<Uuid, String>>>>, 
    player_team_cache_mut: Arc<Mutex<Option<HashMap<Uuid, Uuid>>>>, 
    mut ws_conns: WSConnections_,
    draft_choices_notifier: Arc<Notify>
) {
    println!("In draft handler");
    // TODO handle error
    let mut timeout: Option<chrono::Duration> = None;

    loop {
        // TODO do i need both unwraps? and two pools
        let conn = pg_pool.get().unwrap();
        let unchosen = db::get_unchosen_draft_choices(&conn);
        println!("Draft: length unchosen picks: {:?}", &unchosen.as_ref().map(|x|x.iter().map(|y|y.0.team_draft_id).collect_vec().len()));
        //TODO if there's an error, the timeout gets fucked.
        match unchosen{
            // Want this to be resilient/not bring down whole service.....
            // maybe it should just be a separate service/binary.
            // separate binary makes sense
            Err(e) => {
                println!("db::get_unchosen_draft_choices(&conn) went wrong: {:?}", e);
                continue
            },
            Ok(all_unchosen) => {
                // TODO handle this less terribly
                let mut league_to_max_per_position: HashMap<Uuid, HashMap<String, i32>> = HashMap::new();
                all_unchosen.iter().map(|t| t.3.league_id).dedup().for_each(|lid|{
                    let max_pos_vec = db::get_max_per_position(&conn, lid).unwrap();
                    league_to_max_per_position.insert(lid, max_pos_vec.into_iter().map(|x| (x.position, x.squad_max)).collect());
                });
                let mut i = 0; 
                for unchosen in all_unchosen.into_iter(){
                    println!("{}", i);
                    i += 1;
                    let (draft_choice, period, team_draft, league) = unchosen;
                    let raw_time = DieselTimespan::upper(draft_choice.timespan);
                    let time_to_unchosen = raw_time - Utc::now();
                    println!("Time for next choice: {:?}", time_to_unchosen);
                    if (time_to_unchosen) < chrono::Duration::zero(){
                        let pick_straight_into_team = league.team_size == league.squad_size;
                        let player_position_cache_opt = player_position_cache_mut.lock().await;
                        let player_team_cache_opt = player_team_cache_mut.lock().await;
                        match (player_position_cache_opt.as_ref(), player_team_cache_opt.as_ref()){
                            (Some(ref player_position_cache), Some(ref player_team_cache)) => {
                                println!("Draft: Picking from queue or random for team_id {}", &team_draft.fantasy_team_id);
                                let out: Result<(Pick, Option<ActivePick>), BoxError> = pick_from_queue_or_random(
                                    // TODO safetify these unwraps
                                    pg_pool.get().unwrap(), team_draft.fantasy_team_id, draft_choice, period.timespan, &team_draft.draft_id, period.period_id,
                                    &league.max_squad_players_same_team, league_to_max_per_position.get(&league.league_id).unwrap(),
                                    pick_straight_into_team,
                                    player_position_cache, player_team_cache
                                );
                                match out
                                {
                                    Ok((p, _)) => {
                                        println!("{} PICKED {} for draft {}", team_draft.fantasy_team_id, p.player_id, team_draft.draft_id);
                                        // TODO no way should be doing this on every pick
                                        // Well maybe the publish part, but not the huge db query to get full hierarchy
                                        match publish_updated_pick(&mut ws_conns, p, team_draft.draft_id).await{
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
                        break;
                        //continue 'outer;  // If we didnt use up all drafts, dont want to re-set timeout to None below.
                    }
                };
            }
        }
        // If we get an external notify in this wait time,
        // the timeout will still trigger, so we'll do an extra pass.
        // I think that's ok though, as it'll just be one, and it'll be the time when we should we processing the next draft anyway.
        // Overall logic handles accidental wakeups fine, it just realises too early, sets the timeout, and waits next loop.
        println!("Draft: handler timeout: {:?}", timeout);
        let wait_task = match timeout{
            // std::time::Duration
            Some(t) => {
                let draft_choices_notifier = draft_choices_notifier.clone();
                tokio::task::spawn(async move {
                    println!("delay_for");
                    delay_for(t.to_std().expect("Time conversion borkled!")).await;
                    draft_choices_notifier.notify();
                })
                //Timeout::new(t.to_std().expect("Wrong timeline!"), &||notify2.notify());
            },
            None => {
                // TODO dumb placeholder. Really want to `cancel` above task if other thing notified.
                tokio::spawn(async move {
                    println!("Draft: in dummy (i.e. timeout None) sleep");
                    delay_for(std::time::Duration::from_millis(1)).await;
                })
            }
        };
        let waity_waity = || draft_choices_notifier.notified();
        //println!("Draft: handler pre join!(waity_waity(), wait_task)");
        println!("Draft: handler pre join!");
        // TODO get back errors from wait_task, without blocking.
        join!(waity_waity());
        println!("Draft: handler post join!(waity_waity())");
        timeout = None;
    }
    
}

pub fn pick_from_queue_or_random(
    conn: PgConn,
    fantasy_team_id: Uuid,
    unchosen: DraftChoice,
    belongs_to_team_for: DieselTimespan,
    draft_id: &Uuid,
    period_id: Uuid,
    max_squad_players_same_team: &i32,
    max_squad_players_same_position: &HashMap<String, i32>,
    pick_straight_into_team: bool,
    player_position_cache: &HashMap<Uuid, String>,
    player_team_cache: &HashMap<Uuid, Uuid>
) -> Result<(Pick, Option<ActivePick>), BoxError>{
    conn.build_transaction().run(||{
        // TODO deal with squads not just teams
        let draft_choice_id = unchosen.draft_choice_id;
        let draft_queue_opt = db::get_draft_queue_for_choice(&conn, unchosen)?;
        // If its a hashmap rather than set, can include player-info grabbed from other api.
        // use vec and set, because want fast-lookup when looping through draft queue,
        // but also need access random element if !picked
        //teams_and_players_mut
        let valid_remaining_players: Vec<Uuid> = db::get_valid_picks(&conn, draft_id, &period_id)?;
        let (positions, teams) = (player_position_cache, player_team_cache);
        let current_squad = db::get_current_players(&conn, fantasy_team_id, period_id)?;
        // TODO rust defaultdict?
        let (position_counts, team_counts) = position_team_counts(
            current_squad, player_position_cache, player_team_cache
        );
        let banned_teams: HashSet<Uuid> = team_counts.into_iter().filter(|(_, count)| count > max_squad_players_same_team).map(|(team, _)| team).collect();
        // TODO properly handle a missed position, not just assume a high max
        let banned_positions: HashSet<String> = position_counts.into_iter()
            .filter(|(pos, count)| count > max_squad_players_same_position.get(pos).unwrap_or(&99i32)).map(|(pos, _)| pos).collect();
        let valid_remaining_players_hash: HashSet<&Uuid> = valid_remaining_players.iter().collect();
        println!("banned_teams start");
        banned_teams.iter().for_each(|v| println!("{}", v));
        println!("banned_teams end");
        println!("banned positions: {:?}", banned_positions);
        println!("valid-remaining start");
        valid_remaining_players_hash.iter().for_each(|v| println!("{}", v));
        println!("valid-remaining end");
        // TODO do dumb then tidy
        let mut new_pick: Option<Pick> = None;
        if let Some(draft_queue) = draft_queue_opt{
            for pick_id in draft_queue{
                if let Some(_) = valid_remaining_players_hash.get(&pick_id) 
                {
                    let (position, team) = match (positions.get(&pick_id), teams.get(&pick_id)){
                        (None, _) => {
                            println!("Draft: Could not find player {:?} in player_position_cache", pick_id);
                            dbg!(player_position_cache);
                            continue;
                        },
                        (_, None) => {
                            println!("Draft: Could not find player {:?} in player_team_cache", pick_id);
                            dbg!(player_team_cache);
                            continue;
                        },
                        (Some(pos), Some(team)) => (pos, team)
                    };
                    if (banned_positions.get(position).is_none())
                    && (banned_teams.get(team).is_none())
                    {
                        new_pick = Some(Pick{
                            pick_id: Uuid::new_v4(), fantasy_team_id: fantasy_team_id, draft_choice_id,
                            player_id: pick_id, timespan: belongs_to_team_for
                        });
                        break
                    }
                    
                }
            }
        }
        if new_pick.is_none(){
            println!("random picking");
            if let Some(random_choice) = valid_remaining_players.choose(&mut rand::thread_rng()){
                println!("random picked");
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

fn position_team_counts(
    player_ids: Vec<Uuid>, player_position_cache: &HashMap<Uuid, String>, player_team_cache: &HashMap<Uuid, Uuid>
) -> (HashMap<String, i32>, HashMap<Uuid, i32>){
    let (mut position_counts, mut team_counts): (HashMap<String, i32>, HashMap<Uuid, i32>) = (HashMap::new(), HashMap::new());
    // TODO rust defaultdict?
    //https://github.com/rust-lang/rust/issues/42505
    // https://github.com/rust-lang/rust/issues/35463
    let dummya = "".to_string();
    let dummyb = Uuid::new_v4();
    player_ids.into_iter().for_each(|pid|{
        // pretty sure these unwraps are super-safe as we've built the maps ourselves with these pick-ids.
        let (position, team) = (
            player_position_cache.get(&pid).unwrap_or_else(||
            {
                println!("failed unwrap in position_team_counts player_position_cache, pid: {}", pid);
                player_position_cache.iter().for_each(|x| println!("{}", x.0));
                &dummya
            }), 
            player_team_cache.get(&pid).unwrap_or_else(||
                {
                    println!("failed unwrap in position_team_counts player_team_cache, pid: {}", pid);
                    player_team_cache.iter().for_each(|x| println!("{}", x.0));
                    &dummyb
                })
        );
        match position_counts.get_mut(position) {
            Some(v) => {
                *v = *v + 1;
            }
            None => {
                // TODO get red of clone, be smarter?
                // issue with using reference in hashmap value is this is in the "Caches" type, seems to force me to put a specific lifetime on so many things,
                // cos it gets passed through everything
                // Ooh does this work, or is it different
                position_counts.insert((*position).clone(), 1);
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
    player_position_cache: &HashMap<Uuid, String>,
    player_team_cache: &HashMap<Uuid, Uuid>,
    max_team_players_same_team: &i32,
    max_team_players_same_position: &HashMap<String, i32>,
    max_team_size: &i32
) -> Result<bool, BoxError>{
    println!("all teams");
    all_teams.iter().nth(0).map(|t|t.inner.iter().for_each(|t|println!("{}", t)));
    for team in all_teams{
        let team_len = team.inner.len() as i32;
        println!("team_len: {}, max_team_size: {}", team_len, max_team_size);
        let (position_counts, team_counts) = position_team_counts(
            team.inner, player_position_cache, player_team_cache
        );
        println!("position_counts: {:?}", position_counts);
        if &team_len > max_team_size {
            return Err(Box::new(errors::InvalidTeamError{description: format!("Team cannot be larger than {}", max_team_size)}) as BoxError)
        }
        for (pos, count) in position_counts{
            let max = max_team_players_same_position.get(&pos).unwrap_or(&99i32);
            if &count > max{
                return Err(Box::new(errors::InvalidTeamError{description: format!("Team cannot have more than {} players for position {}", max, pos)}) as BoxError)
            }
        }
        if team_counts.values().max().unwrap_or(&0i32) > max_team_players_same_team {
            return Err(Box::new(errors::InvalidTeamError{description: format!("Team cannot have more than {} players from same team", max_team_players_same_team)}) as BoxError)
        }
    }
    Ok(true)
}