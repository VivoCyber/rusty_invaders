use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use invaders::{
    frame::{new_frame, Drawable},
    invaders::Invaders,
    player::Player,
    render::render,
};
use rusty_audio::Audio;
use std::{
    error::Error,
    io,
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn Error>> {
    // audio
    let mut audio = Audio::new();
    audio.add("start", "./sounds/start.ogg");
    audio.add("lasser", "./sounds/lasser.ogg");
    audio.add("lose", "./sounds/lose.ogg");
    audio.add("move", "./sounds/move.ogg");
    audio.add("win", "./sounds/win.ogg");
    audio.add("explode", "./sounds/explode.ogg");
    audio.play("start");

    //terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    //render loop in a separate thread
    let (render_tx, render_rx) = channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = new_frame();
        let mut stdout = io::stdout();
        render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    //game loop
    let mut instant = Instant::now();
    let mut player = Player::new();
    let mut invaders = Invaders::new();
    'gameloop: loop {
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("lasser");
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }

        // update
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        if player.hit_invader(&mut invaders) {
            audio.play("explode");
        }

        //draw & render
        player.draw(&mut curr_frame);
        invaders.draw(&mut curr_frame);
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(7));

        if invaders.all_kill() {
            audio.play("win");
            break 'gameloop;
        }

        if invaders.reached_bottom() {
            audio.play("lose");
            break 'gameloop;
        }
    }

    //clean up
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
