extern crate ctrlc;
extern crate libc;
use crate::term;
use crate::old::game;
use std::f64::consts::PI;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::vec::Vec;

pub fn old_main() -> std::io::Result<()> {
    let should_go = Arc::new(AtomicBool::new(true));
    let should_go_2 = should_go.clone();
    ctrlc::set_handler(move || {
        should_go.store(false, Ordering::Relaxed)
    }).expect("Error setting ctrl-c handler");

    let term = term::Term::new()?;
    curse_main(&term, should_go_2)
}

fn curse_main(term: &term::Term, should_go: Arc<AtomicBool>) -> std::io::Result<()> {
    // change size to accomodate header
    let term_width = term.width;
    let term_height = term.height - 2;
    let mut frame_num: u64 = 0;
    let mut prev_time = std::time::Instant::now();
    let mut pause = false;
    let mut last_input = std::vec::Vec::new();

    let speed_factor = 0.4;

    let mut field = game::Field {
        blocks: vec![ vec![b'#', b'#', b'#', b'#', b'#']
                    , vec![b'#', b' ', b' ', b' ', b'#']
                    , vec![b'#', b' ', b'#', b' ', b'#']
                    , vec![b'#', b' ', b' ', b' ', b'#']
                    , vec![b'#', b' ', b'#', b'#', b'#']
                    ],
        player: game::Player {
            x: 1.5,
            y: 1.5,
            angle: PI/2.0,
            vel_x: 0.0,
            vel_y: 0.0,
        }
    };
    let mut buffer = Vec::new();
    buffer.resize(term_width * term_height, 'a');

    while should_go.load(Ordering::Relaxed) {
        // process input
        let input = term.get_input_buffer()?;
        for c in &input {
            if *c == b'q' {
                should_go.store(false, Ordering::Relaxed);
            } else if *c == b'p' {
                pause = !pause;
            } else if *c == b'h' || *c == b'j' {
                field.player.angle += PI / 90.0;
            } else if *c == b'l' || *c == b'k' {
                field.player.angle -= PI / 90.0;
            } else if *c == b'w' {
                let delta_x = field.player.angle.cos() * speed_factor;
                let delta_y = field.player.angle.sin() * speed_factor;
                field.player.vel_x = delta_x;
                field.player.vel_y = delta_y;
            } else if *c == b's' {
                let delta_x = field.player.angle.cos() * speed_factor;
                let delta_y = field.player.angle.sin() * speed_factor;
                field.player.vel_x = delta_x;
                field.player.vel_y = delta_y;
            } else if *c == b'd' {
                let delta_x = field.player.angle.sin() * speed_factor;
                let delta_y = - field.player.angle.cos() * speed_factor;
                field.player.vel_x = delta_x;
                field.player.vel_y = delta_y;
            } else if *c == b'a' {
                let delta_x = - field.player.angle.sin() * speed_factor;
                let delta_y = field.player.angle.cos() * speed_factor;
                field.player.vel_x = delta_x;
                field.player.vel_y = delta_y;
            } else if *c == b' ' {
                field.player.vel_x = 0.0;
                field.player.vel_y = 0.0;
            }
        }
        if !input.is_empty() {
            last_input = input.clone();
        }

        if pause {
            continue;
        }

        // time counting

        let duration = prev_time.elapsed().as_nanos() as f64;
        let freq = (1_000_000_000f64 / duration) as i64; // fps
        prev_time = std::time::Instant::now();

        // physics routines

        field.player.x += field.player.vel_x * duration / 1_000_000_000f64;
        field.player.y += field.player.vel_y * duration / 1_000_000_000f64;
        if player_in_wall(&field) {
            field.player.x += field.player.vel_x * duration / 1_000_000_000f64;
            field.player.y += field.player.vel_y * duration / 1_000_000_000f64;
            field.player.vel_x = 0.0;
            field.player.vel_y = 0.0;
        }
        // check if velocity too low to lower further
        let eps_x = (field.player.vel_x - 0.0001).max(0.0);
        let eps_y = (field.player.vel_y - 0.0001).max(0.0);
        if eps_x == 0.0 && eps_y == 0.0 {
            // ensure zero speed
            field.player.vel_x = 0.0;
            field.player.vel_y = 0.0;
        } else {
            // accelerate backwards
            let acc_x = field.player.angle.cos() * speed_factor;
            let acc_y = field.player.angle.sin() * speed_factor;
            field.player.vel_x -= acc_x * duration / 1_000_000_000f64;
            field.player.vel_y -= acc_y * duration / 1_000_000_000f64;
        }

        // render routines

        game::draw_scene(&field, (term_width, term_height), &mut buffer);

        // drawing routines

        // header with screen info
        let header = format!("Size: {} x {}, frame: {}, fps: {}", term_width, term_height, frame_num, freq);
        let mut remains = Vec::new();
        remains.resize(term_width as usize - header.len(), b' ');
        term.put_buffer(header.as_bytes())?;
        term.put_partial_buffer(remains.as_slice())?;

        // header with keyboard info
        let header = format!("last input: {:?}, current input: {:?}", last_input, input);
        let mut remains = Vec::new();
        remains.resize(term_width as usize - header.len(), b' ');
        term.put_partial_buffer(header.as_bytes())?;
        term.put_partial_buffer(remains.as_slice())?;

        // main game view
        term.put_partial_utf8_buffer(buffer.as_slice())?;

        frame_num += 1;
    }

    Ok(())
}

fn player_in_wall(field: &game::Field) -> bool {
    field.blocks[field.player.x.floor() as usize][field.player.y.floor() as usize] == b'#'
}
