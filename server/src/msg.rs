// Standard
use std::mem;

// Library
use specs::prelude::*;
use vek::*;

// Project
use common::{ecs::phys::Pos, util::manager::Manager};

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
            // Send the help information to the player
            srv.send_chat_msg(player, "Available commands:");
            srv.send_chat_msg(player, "/players - View all online players");
            srv.send_chat_msg(player, "/tp <alias> - Teleport to a player");
            srv.send_chat_msg(player, "/pos - Display your current position");
            srv.send_chat_msg(player, "/alias <alias> - Change your alias");
            srv.send_chat_msg(player, "/warp <dx> <dy> <dz> - Offset your position");
			srv.send_chat_msg(player, "/goto <dx> <dy> <dz> - Teleport to specified position");
        }),
        Some("players") => srv.do_for(|srv| {
            // Find a list of player names and format them
            let player_names = srv
                .world
                .read_storage::<Player>()
                .join()
                .map(|p| p.alias.clone())
                .collect::<Vec<_>>()
                .join(", ");

            // Send them back to the player
            srv.send_chat_msg(player, &format!("Online Players: {}", player_names));
        }),
        Some("tp") => 'tp: {
            // Find the alias the player typed (i.e: '/tp zesterer')
            let tgt_alias = if let Some(s) = cmd.nth(0) {
                s
            } else {
                srv.do_for(|srv| srv.send_chat_msg(player, "A second argument is needed: /tp <alias>"));
                break 'tp;
            };

            // Find the position of the player with the given alias, if possible
            let tgt_pos = if let Some(p) = srv.do_for(|srv| {
                (&srv.world.read_storage::<Pos>(), &srv.world.read_storage::<Player>())
                    .join()
                    .find(|(_, player)| player.alias == tgt_alias) // This is the important bit
                    .map(|(pos, _)| pos.0)
            }) {
                p
            } else {
                srv.do_for(|srv| srv.send_chat_msg(player, &format!("Could not locate {}!", tgt_alias)));
                break 'tp;
            };

            // Set the position of the current player accordingly
            srv.do_for_mut(|srv| {
                if srv.update_comp(player, Pos(tgt_pos)) {
                    srv.force_comp::<Pos>(player); // Force clients to update
                    srv.send_chat_msg(player, &format!("Teleported to {}!", tgt_alias));
                } else {
                    srv.send_chat_msg(player, "You don't have a position!");
                }
            });
        },
        Some("pos") => srv.do_for(|srv| {
            if let Some(pos_comp) = srv.world.read_storage::<Pos>().get(player) {
                srv.send_chat_msg(player, &format!("Current position: {}", pos_comp.0));
            } else {
                srv.send_chat_msg(player, "You don't have a position!");
            }
        }),
        Some("alias") => srv.do_for_mut(|srv| 'nick: {
            let alias = match cmd.nth(0) {
                Some(alias) => alias,
                _ => {
                    srv.send_chat_msg(player, "A second argument is needed: /alias <alias>");
                    break 'nick;
                },
            };

            // Check if the alias is already used by another player.
            for p in (&srv.world.read_storage::<Player>()).join() {
                if p.alias == alias {
                    srv.send_chat_msg(player, "This alias is already in use");
                    break 'nick;
                }
            }

            if !srv.is_valid_alias(&alias) {
                srv.send_chat_msg(player, "The provided alias is invalid");
                break 'nick;
            }

            // Give the player their new alias, hold on to the old one temporarily
            if let Some(old_alias) = srv.do_for_comp_mut::<Player, _, _>(player, |player_comp| {
                let mut alias = alias.to_string();
                mem::swap(&mut player_comp.alias, &mut alias);
                alias
            }) {
                srv.force_comp::<Pos>(player); // Force clients to update
                srv.broadcast_chat_msg(&format!("[{} changed their alias to {}]", old_alias, alias));
            } else {
                srv.send_chat_msg(player, "Could not change alias");
                break 'nick;
            }
        }),
        Some("warp") => srv.do_for_mut(|srv| 'warp: {
            let mut tensor = [0.0; 3];
            for i in 0..3 {
                let arg = if let Some(a) = cmd.next() {
                    a
                } else {
                    srv.send_chat_msg(player, "3 numbers are needed: /warp <dx> <dy> <dz>");
                    break 'warp;
                };

                if let Ok(v) = arg.parse() {
                    tensor[i] = v;
                } else {
                    srv.send_chat_msg(
                        player,
                        &format!("Invalid value for {}: /warp <x> <y> <z>", ['x', 'y', 'z'][i]),
                    );
                    break 'warp;
                }
            }

            if let Some(pos) = srv.do_for_comp_mut::<Pos, _, _>(player, |pos_comp| {
                pos_comp.0 += Vec3::from(tensor);
                pos_comp.0
            }) {
                srv.force_comp::<Pos>(player); // Force clients to update
                srv.send_chat_msg(player, &format!("Warped to: {}!", pos));
            } else {
                srv.send_chat_msg(player, "You don't have a position!");
                break 'warp;
            }
        }),
		Some("goto") => srv.do_for_mut(|srv| 'goto: {
            let mut tensor = [0.0; 3];
            for i in 0..3 {
                let arg = if let Some(a) = cmd.next() {
                    a
                } else {
                    srv.send_chat_msg(player, "3 numbers are needed: /goto <dx> <dy> <dz>");
                    break 'goto;
                };

                if let Ok(v) = arg.parse() {
                    tensor[i] = v;
                } else {
                    srv.send_chat_msg(
                        player,
                        &format!("Invalid value for {}: /goto <x> <y> <z>", ['x', 'y', 'z'][i]),
                    );
                    break 'goto;
                }
            }

            if let Some(pos) = srv.do_for_comp_mut::<Pos, _, _>(player, |pos_comp| {
                pos_comp.0 = Vec3::from(tensor);
                pos_comp.0
            }) {
                srv.force_comp::<Pos>(player); // Force clients to update
                srv.send_chat_msg(player, &format!("teleported to: {}!", pos));
            } else {
                srv.send_chat_msg(player, "You don't have a position!");
                break 'goto;
            }
        }),
        _ => srv.do_for(|srv| srv.send_chat_msg(player, "Unrecognised command!")),
    }
}
