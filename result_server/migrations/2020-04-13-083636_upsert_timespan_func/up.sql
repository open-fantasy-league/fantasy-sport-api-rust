-- Your SQL goes here
-- technically possible to dynamically have one func for all required tables
-- however not sure it's worth the faff
-- https://stackoverflow.com/questions/10705616/table-name-as-a-postgresql-function-parameter
CREATE FUNCTION public.trim_team_name_timespans(new_team_id uuid, new_timespan tstzrange) 
	RETURNS SETOF record
	LANGUAGE plpgsql
AS $function$
DECLARE
	existing_row RECORD;
	r RECORD;
BEGIN
	FOR existing_row IN
		SELECT team_name_id, timespan FROM team_names
		WHERE team_id = new_team_id AND timespan && new_timespan
	LOOP
		IF new_timespan @> existing_row.timespan THEN
			DELETE FROM team_names WHERE team_name_id = existing_row.team_name_id;
		ELSIF new_timespan @> lower(existing_row.timespan) THEN
			FOR r IN UPDATE team_names
			SET timespan = tstzrange(upper(new_timespan), upper(existing_row.timespan), "[)")
			WHERE team_name_id = existing_row.team_name_id
			RETURNING * LOOP RETURN NEXT r; END LOOP;
		ELSIF new_timespan @> upper(existing_row.timespan) THEN
			FOR r IN UPDATE team_names
			SET timespan = tstzrange(lower(existing_row.timespan), lower(new_timespan), "[)")
			WHERE team_name_id = existing_row.team_name_id
			RETURNING * LOOP RETURN NEXT r; END LOOP;
		END IF;
	END LOOP;
	RETURN;
END $function$;

CREATE FUNCTION public.trim_player_name_timespans(new_player_id uuid, new_timespan tstzrange) 
	RETURNS SETOF record
	LANGUAGE plpgsql
AS $function$
DECLARE
	existing_row RECORD;
	r RECORD;
BEGIN
	FOR existing_row IN
		SELECT player_name_id, timespan FROM player_names
		WHERE player_id = new_player_id AND timespan && new_timespan
	LOOP
		IF new_timespan @> existing_row.timespan THEN
			DELETE FROM player_names WHERE player_name_id = existing_row.player_name_id;
		ELSIF new_timespan @> lower(existing_row.timespan) THEN
			FOR r IN UPDATE player_names
			SET timespan = tstzrange(upper(new_timespan), upper(existing_row.timespan), "[)")
			WHERE player_name_id = existing_row.player_name_id
			RETURNING * LOOP RETURN NEXT r; END LOOP;
		ELSIF new_timespan @> upper(existing_row.timespan) THEN
			FOR r IN UPDATE player_names
			SET timespan = tstzrange(lower(existing_row.timespan), lower(new_timespan), "[)")
			WHERE player_name_id = existing_row.player_name_id
			RETURNING * LOOP RETURN NEXT r; END LOOP;
		END IF;
	END LOOP;
	RETURN;
END $function$;

CREATE FUNCTION public.trim_team_player_timespans(new_player_id uuid, new_timespan tstzrange) 
	RETURNS SETOF record
	LANGUAGE plpgsql
AS $function$
DECLARE
	existing_row RECORD;
	r RECORD;
BEGIN
	FOR existing_row IN
		SELECT player_id, timespan FROM team_players
		WHERE player_id = new_player_id AND timespan && new_timespan
	LOOP
		IF new_timespan @> existing_row.timespan THEN
			DELETE FROM team_players WHERE team_player_id = existing_row.team_player_id;
		ELSIF new_timespan @> lower(existing_row.timespan) THEN
			FOR r IN UPDATE team_players
			SET timespan = tstzrange(upper(new_timespan), upper(existing_row.timespan), "[)")
			WHERE team_player_id = existing_row.team_player_id
			RETURNING * LOOP RETURN NEXT r; END LOOP;
		ELSIF new_timespan @> upper(existing_row.timespan) THEN
			FOR r IN UPDATE team_players
			SET timespan = tstzrange(lower(existing_row.timespan), lower(new_timespan), "[)")
			WHERE team_player_id = existing_row.team_player_id
			RETURNING * LOOP RETURN NEXT r; END LOOP;
		END IF;
	END LOOP;
	RETURN;
END $function$;

CREATE FUNCTION public.trim_player_position_timespans(new_player_id uuid, new_timespan tstzrange) 
	RETURNS SETOF record
	LANGUAGE plpgsql
AS $function$
DECLARE
	existing_row RECORD;
	r RECORD;
BEGIN
	FOR existing_row IN
		SELECT player_position_id, timespan FROM player_positions
		WHERE player_id = new_player_id AND timespan && new_timespan
	LOOP
		IF new_timespan @> existing_row.timespan THEN
			DELETE FROM player_positions WHERE player_position_id = existing_row.player_position_id;
		ELSIF new_timespan @> lower(existing_row.timespan) THEN
			FOR r IN UPDATE player_positions
			SET timespan = tstzrange(upper(new_timespan), upper(existing_row.timespan), "[)")
			WHERE player_position_id = existing_row.player_position_id
			RETURNING * LOOP RETURN NEXT r; END LOOP;
		ELSIF new_timespan @> upper(existing_row.timespan) THEN
			FOR r IN UPDATE player_positions
			SET timespan = tstzrange(lower(existing_row.timespan), lower(new_timespan), "[)")
			WHERE player_position_id = existing_row.player_position_id
			RETURNING * LOOP RETURN NEXT r; END LOOP;
		END IF;
	END LOOP;
	RETURN;
END $function$;