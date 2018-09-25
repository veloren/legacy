// Library
use specs::prelude::*;

// Project
use common::manager::Manager;
use region::ecs::phys::Pos;

// Local
use api::Api;
use player::Player;
use Payloads;
use Server;
use Wrapper;

pub(crate) fn process_chat_msg<P: Payloads>(
    srv: &Wrapper<Server<P>>,
    text: String,
    player: Entity,
    mgr: &Manager<Wrapper<Server<P>>>,
) {
    if text.starts_with('/') {
        let cmd = text[1..].split(' ');
        process_cmd(srv, cmd, player, mgr);
    } else if let Some(text) = srv.do_for(|srv| srv.payload.on_chat_msg(srv, player, &text)) {
        // Run the message past the payload interface
        srv.do_for(|srv| srv.broadcast_chat_msg(&text));
    }
}

pub(crate) fn process_cmd<'a, P: Payloads>(
    srv: &Wrapper<Server<P>>,
    mut cmd: impl Iterator<Item = &'a str> + 'a,
    player: Entity,
    _mgr: &Manager<Wrapper<Server<P>>>,
) {
    match cmd.next() {
        Some("help") => srv.do_for(|srv| {
            srv.send_chat_msg(player, "Available commands:");
            srv.send_chat_msg(player, "/players - View all online players");
            srv.send_chat_msg(player, "/tp <alias> - Teleport to a player");
        }),
        Some("players") => srv.do_for(|srv| {
            // Find a list of player names
            let player_names = srv
                .world
                .read_storage::<Player>()
                .join()
                .map(|p| p.alias.clone())
                .collect::<Vec<_>>()
                .join(", ");
            srv.send_chat_msg(player, &format!("Online Players: {}", player_names));
        }),
        Some("tp") => {
            // TODO: Simplify this? Put it somewhere else?
            // Find the name the player typed (i.e: '/tp zesterer')

            'tp: {
                let tgt_alias = if let Some(s) = cmd.nth(0) {
                    s
                } else {
                    srv.do_for(|srv| srv.send_chat_msg(player, "Usage: /tp <alias>"));
                    break 'tp;
                };

                let tgt_pos = srv.do_for(|srv| {
                    // Find the position of that player
                    let pos_storage = srv.world.read_storage::<Pos>();
                    let player_storage = srv.world.read_storage::<Player>();
                    (&pos_storage, &player_storage).join().find_map(|(pos, player)| {
                        if player.alias == tgt_alias {
                            Some(pos.0)
                        } else {
                            None
                        }
                    })
                });

                let tgt_pos = if let Some(p) = tgt_pos {
                    p
                } else {
                    srv.do_for(|srv| srv.send_chat_msg(player, &format!("Could not locate {}!", tgt_alias)));
                    break 'tp;
                };

                srv.do_for_mut(|srv| {
                    if let Some(pos) = srv.world.write_storage::<Pos>().get_mut(player) {
                        pos.0 = tgt_pos;
                        srv.send_chat_msg(player, &format!("Teleported to {}!", tgt_alias));
                    } else {
                        srv.send_chat_msg(player, "You don't have a position!");
                    }
                });
            }
        },
        _ => srv.do_for(|srv| srv.send_chat_msg(player, "Unrecognised command!")),
    }
}
