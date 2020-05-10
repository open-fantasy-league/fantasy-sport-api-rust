use crate::db;
use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl; // imported here so that can run db macros
use crate::schema;
use crate::types::{drafts::*, fantasy_teams::*, leagues::*};
use diesel;
use diesel_utils::*;
use std::collections::{HashSet, HashMap};
use chrono::{DateTime, Utc, self};
use uuid::Uuid;
use tokio::sync::Notify;
use tokio::time::delay_for;
use std::sync::Arc;
use futures::join;
use std::ops::Bound::*;
use rand::seq::SliceRandom;
use rand;
use warp_ws_server::BoxError;
use std::fmt;

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

pub async fn draft_builder(pg_pool: PgPool) {
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
        let conn = pg_pool.clone().get().unwrap();
        match db::get_undrafted_periods(&conn){
            // Want this to be resilient/not bring down whole service.....
            // maybe it should just be a separate service/binary.
            // separate binary makes sense
            Err(e) => {
                println!("db::get_undrafted_periods(&conn) went wrong");
                continue
            },
            Ok(all_undrafted) => {
                'inner: for undrafted in all_undrafted.into_iter(){
                    let time_to_draft_gen = undrafted.draft_start - Utc::now();
                    if (time_to_draft_gen) < chrono::Duration::zero(){
                        match conn.build_transaction().run(||{generate_drafts(&conn, undrafted)}){
                            Ok(drafts) => {println!("TODO publish them")},
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
    conn: &PgConn,
    period: Period,
) -> Result<Vec<ApiDraft>, diesel::result::Error> {
    println!("In generate_drafts");
    // TODO can build in more efficient way, rather than adding entries 1-by-1
    let teams = db::get_randomised_teams_for_league(conn, period.league_id)?;
    let squad_size = db::get_league_squad_size(conn, period.league_id)? as usize;
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
    //Result<Vec<ApiDraft>, diesel::result::Error>
    //Result<Vec<_>, _>
    let drafts: Result<Vec<_>, _> = draft_map
        .into_iter()
        .map(|(_, teams)| {
            let drafts: Vec<Draft> = insert!(
                conn,
                schema::drafts::table,
                vec![Draft::new(period.period_id)]
            )?;
            let draft = drafts.first().unwrap();
            let team_drafts: Vec<TeamDraft> = teams
                .iter()
                .map(|team| TeamDraft::new(draft.draft_id, team.fantasy_team_id))
                .collect();
            let _: Vec<TeamDraft> = insert!(conn, schema::team_drafts::table, &team_drafts)?;
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
            let _: Vec<DraftChoice> = insert!(conn, schema::draft_choices::table, to_insert)?;
            let out = ApiDraft {
                draft_id: draft.draft_id,
                period_id: period.period_id,
                meta: draft.meta.clone(),
                choices: choices,
            };
            Ok(out)
        })
        .collect();
        drafts
}


pub async fn draft_handler(pg_pool: PgPool) {
    println!("In draft builder");
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();
    // TODO handle error
    let mut timeout: Option<chrono::Duration> = None;

    'outer: loop {
        println!("timeout: {:?}", timeout);
        let conn = pg_pool.clone().get().unwrap();
        match db::get_unchosen_draft_choices(&conn){
            // Want this to be resilient/not bring down whole service.....
            // maybe it should just be a separate service/binary.
            // separate binary makes sense
            Err(e) => {
                println!("db::get_unchosen_draft_choices(&conn) went wrong");
                continue
            },
            Ok(all_unchosen) => {
                'inner: for unchosen in all_unchosen.into_iter(){
                    let (draft_choice, period, team_draft) = unchosen;
                    let raw_time = match draft_choice.timespan.1{
                        Included(x) => x,
                        Excluded(x) => x,
                        Unbounded => panic!("Why the flying fudge is there an unbounded timestamp IN MY GOD DAMN DATABASE!!")
                    };
                    let time_to_unchosen = raw_time - Utc::now();
                    if (time_to_unchosen) < chrono::Duration::zero(){
                        match conn.build_transaction().run(||{pick_from_queue_or_random(&conn, team_draft.fantasy_team_id, draft_choice, period.timespan)}){
                            Ok(drafts) => {println!("TODO publish them....or could publish in the func")},
                            Err(e) => println!("{:?}", e)
                        };
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
    conn: &PgConn,
    fantasy_team_id: Uuid,
    unchosen: DraftChoice,
    belongs_to_team_for: DieselTimespan
) -> Result<Pick, BoxError>{
    let draft_choice_id = unchosen.draft_choice_id;
    let draft_queue = db::get_draft_queue_for_choice(conn, unchosen)?;
    // If its a hashmap rather than set, can include player-info grabbed from other api.
    // use vec and set, because want fast-lookup when looping through draft queue,
    // but also need access random element if !picked
    let valid_remaining_picks: Vec<Uuid> = vec![];
    let valid_remaining_picks_hash: HashSet<&Uuid> = valid_remaining_picks.iter().collect();
    // TODO do dumb then tidy
    let mut picked = false;
    let mut new_pick: Option<Pick> = None;
    for pick_id in draft_queue{
        if let Some(valid_pick_id) = valid_remaining_picks_hash.get(&pick_id){
            let new_pick = Some(Pick{
                pick_id: Uuid::new_v4(), fantasy_team_id: fantasy_team_id, draft_choice_id,
                player_id: pick_id, timespan: belongs_to_team_for, active: false
            });
            let a = vec![new_pick.unwrap()];
            let out: Vec<Pick> = insert!(conn, schema::picks::table, a)?;
            break
        }
    }
    if new_pick.is_none(){
        if let Some(random_choice) = valid_remaining_picks.choose(&mut rand::thread_rng()){
            new_pick = Some(Pick{
                pick_id: Uuid::new_v4(), fantasy_team_id: fantasy_team_id, draft_choice_id,
                player_id: *random_choice, timespan: belongs_to_team_for, active: false
            });
        };
    }
    new_pick.ok_or(Box::new(NoValidPicksError{}))
    // we dont mutate the draft-queue. because maybe they want the same queue for future drafts

}