#include "cdoom_rust.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct
{
    int type;
    int data1, data2, data3, data4, data5, data6;
} probe_event_t;

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
    probe_event_t event;
    probe_event_t *popped;

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

    printf("cdoom-rust probe OK: %s\n", version);
    return 0;
}
