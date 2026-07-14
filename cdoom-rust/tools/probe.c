#include "cdoom_rust.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct
{
    int type;
    int data1, data2, data3, data4, data5, data6;
} probe_event_t;

typedef struct
{
    signed char forwardmove;
    signed char sidemove;
    short angleturn;
    unsigned char chatchar;
    unsigned char buttons;
    unsigned char consistancy;
    unsigned char buttons2;
    int inventory;
    unsigned char lookfly;
    unsigned char arti;
} probe_ticcmd_t;

static int dedicated_steps[3];
static int dedicated_step_count;

static void dedicated_print_banner(void)
{
    dedicated_steps[dedicated_step_count++] = 1;
}

static void dedicated_init_zone(void)
{
    dedicated_steps[dedicated_step_count++] = 2;
}

static void dedicated_run_server(void)
{
    dedicated_steps[dedicated_step_count++] = 3;
}

static void require_int(const char *name, int actual, int expected)
{
    if (actual != expected) {
        fprintf(stderr, "%s: got %d, expected %d\n", name, actual, expected);
        exit(1);
    }
}

static void require_string(const char *name, const char *actual, const char *expected)
{
    if (actual == NULL || strcmp(actual, expected) != 0) {
        fprintf(stderr, "%s: got %s, expected %s\n",
                name, actual == NULL ? "(null)" : actual, expected);
        exit(1);
    }
}

int main(void)
{
    const char *version;
    const char *server_argv[] = { "chocolate-server", "-WARP", "1", "-skill", "4" };
    const char *server_option;
    probe_event_t event;
    probe_event_t *popped;
    probe_ticcmd_t cmds[3];
    int ingame[4] = { 1, 1, 1, 0 };

    if (cdoom_rust_init() != 0) {
        fprintf(stderr, "cdoom_rust_init failed\n");
        return 1;
    }

    version = cdoom_rust_version();
    if (version == NULL) {
        fprintf(stderr, "cdoom_rust_version returned NULL\n");
        return 1;
    }

    require_int("valid doom retail", cdoom_rust_valid_game_mode(0, 3), 1);
    require_int("invalid doom commercial", cdoom_rust_valid_game_mode(0, 2), 0);
    require_int("heretic secret episode", cdoom_rust_valid_episode_map(6, 3, 6, 3), 1);
    require_int("doom2 version alias", cdoom_rust_valid_game_version(1, 8), 1);
    require_int("doom episode map", cdoom_rust_is_episode_map(0), 1);
    require_string("mission string", cdoom_rust_game_mission_string(3), "plutonia");
    require_string("mode string", cdoom_rust_game_mode_string(4), "unknown");

    event = (probe_event_t) { 0, 42, 43, 44, 45, 46, 47 };
    cdoom_rust_post_event(&event);
    popped = (probe_event_t *) cdoom_rust_pop_event();
    if (popped == NULL) {
        fprintf(stderr, "event queue unexpectedly empty\n");
        return 1;
    }
    require_int("event data1", popped->data1, 42);
    if (cdoom_rust_pop_event() != NULL) {
        fprintf(stderr, "event queue expected to be empty\n");
        return 1;
    }

    require_int("iwad name", cdoom_rust_is_iwad_name("DOOM2.WAD"), 1);
    require_int("not iwad name", cdoom_rust_is_iwad_name("doom2.wadx"), 0);
    require_string("savegame iwad", cdoom_rust_save_game_iwad_name(1, 1), "freedoom2.wad");
    require_string("suggest iwad", cdoom_rust_suggest_iwad_name(1, 2), "doom2.wad");
    require_string("suggest game", cdoom_rust_suggest_game_name(1, 4), "Doom II");

    require_int("nonvanilla record", cdoom_rust_d_loop_nonvanilla_record_allowed(1, 0), 1);
    require_int("strict nonvanilla record", cdoom_rust_d_loop_nonvanilla_record_allowed(1, 1), 0);
    require_int("nonvanilla playback lmp",
                cdoom_rust_d_loop_nonvanilla_playback_decision(1, 0, 1), 1);
    require_int("nonvanilla playback wad",
                cdoom_rust_d_loop_nonvanilla_playback_decision(1, 0, 0), 2);
    require_int("lowtic disconnected", cdoom_rust_d_loop_get_low_tic(12, 9, 0, 0), 12);
    require_int("lowtic connected", cdoom_rust_d_loop_get_low_tic(12, 9, 1, 0), 9);

    memset(cmds, 0, sizeof(cmds));
    cmds[0].chatchar = 7;
    cmds[0].buttons = 128;
    cmds[1].chatchar = 8;
    cmds[1].buttons = 129;
    cmds[2].chatchar = 9;
    cmds[2].buttons = 2;
    cdoom_rust_d_loop_ticdup_squash(cmds, 3);
    require_int("ticdup chatchar", cmds[0].chatchar, 0);
    require_int("ticdup special", cmds[0].buttons, 0);
    require_int("ticdup special attack", cmds[1].buttons, 0);
    require_int("ticdup normal button", cmds[2].buttons, 2);

    cdoom_rust_d_loop_single_player_clear(ingame, 4, 1);
    require_int("single player clear p0", ingame[0], 0);
    require_int("single player clear local", ingame[1], 1);
    require_int("players in game local",
                cdoom_rust_d_loop_players_in_game(0, 0, ingame, 4), 1);
    require_int("players in game drone",
                cdoom_rust_d_loop_players_in_game(1, 1, ingame, 4), 1);
    ingame[1] = 0;
    require_int("players in game empty drone",
                cdoom_rust_d_loop_players_in_game(1, 1, ingame, 4), 0);

    cdoom_rust_dedicated_net_client_run();
    cdoom_rust_dedicated_main(dedicated_print_banner, dedicated_init_zone,
                              dedicated_run_server);
    require_int("dedicated step count", dedicated_step_count, 3);
    require_int("dedicated step print", dedicated_steps[0], 1);
    require_int("dedicated step init", dedicated_steps[1], 2);
    require_int("dedicated step run", dedicated_steps[2], 3);
    server_option = cdoom_rust_dedicated_rejected_option(5, server_argv);
    require_string("dedicated rejected option", server_option, "-skill");
    require_int("dedicated no rejected option",
                cdoom_rust_dedicated_rejected_option(1, server_argv) == NULL, 1);

    printf("cdoom-rust probe OK: %s\n", version);
    return 0;
}
