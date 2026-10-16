//! Headless stress test: two large warships (thousands of blocks each)
//! slugging it out. Reports the average and worst simulation tick time.
//!
//! `cargo run -p gf_bench --release -- [half_size] [ticks]`

use gf_sim::{Command, Control, PIRATE_FACTION, PLAYER_FACTION, Vec2, World, scenario};
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let mut args = std::env::args().skip(1);
    let half: i32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(27);
    let ticks: u32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(600);
    let content = Arc::new(gf_content::ContentDb::load_dir(gf_content::default_content_dir()).expect("content"));
    let mut world = World::new(content, 1);
    let a = scenario::battleship(&world, half, Some(PLAYER_FACTION), Vec2::new(0.0, 0.0), 0.0);
    let a = world.add_body(a);
    let b = scenario::battleship(&world, half, Some(PIRATE_FACTION), Vec2::new(0.0, half as f32 * 2.0 + 30.0), std::f32::consts::PI);
    let b = world.add_body(b);
    println!(
        "ships: {} + {} blocks",
        world.bodies[&a].block_count(),
        world.bodies[&b].block_count()
    );
    let mut worst = 0.0f64;
    let start = Instant::now();
    for _ in 0..ticks {
        for (me, them) in [(a, b), (b, a)] {
            let aim = world.bodies.get(&them).map(|t| t.pos);
            if world.bodies.contains_key(&me) {
                let control = Control { thrust: Vec2::new(0.0, 0.3), fire: true, aim, auto_suppress: true, ..Default::default() };
                world.queue(Command::Control { body: me, control });
            }
        }
        let t0 = Instant::now();
        world.step();
        worst = worst.max(t0.elapsed().as_secs_f64());
    }
    let total = start.elapsed().as_secs_f64();
    println!(
        "{ticks} ticks: avg {:.2} ms/tick, worst {:.2} ms, projectiles alive {}, bodies {}",
        total * 1000.0 / ticks as f64,
        worst * 1000.0,
        world.projectiles.len(),
        world.bodies.len()
    );
    println!("budget at 30 Hz: 33.3 ms/tick");
}
