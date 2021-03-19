#![cfg_attr(feature = "bench", feature(test))]

#[cfg(feature = "bench")]
extern crate test;

use std::fs::File;
use std::time::{ Duration, Instant };
use std::error::Error as StdError;

use log::trace;
use glium::glutin;
use simplelog::*;

use sys_api::input_tracker::InputTracker;
use sys_api::graphics_init::GraphicsContext;
use game::states::{ FRAMES_PER_SECOND, TICKS_PER_SECOND, GameState };

fn init_logging() -> Result<(), Box<dyn StdError>> {
    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)] {
            println!("Initializing debug logging facade");
            CombinedLogger::init(
                vec![
                    TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed),
                    WriteLogger::new(LevelFilter::Error, Config::default(), File::create("err.log")?),
                ]
            )?
        } else {
            println!("Initializing release logging facade");
            WriteLogger::init(LevelFilter::Error, Config::default(), File::create("err.log")?)?
        }
    };

    Ok(())
}

fn main() -> Result<(), Box<dyn StdError>> {
    if let Err(err) = init_logging() { eprintln!("Failed to init logging"); return Err(err.into()) }

    let (mut ctx, event_loop, mut renders) = GraphicsContext::new()?;
    let mut transition_request = None;
    let mut state = GameState::boot_state(&mut ctx);
    let mut input_tracker = InputTracker::new(ctx.display.gl_window().window());

    let mut last_tick = Instant::now();
    let mut last_frame = Instant::now();
    let mut tick_time_acc = Duration::new(0, 0);
    let tick_duration = Duration::new(1, 0) / TICKS_PER_SECOND;
    let frame_duration = Duration::new(1, 0) / FRAMES_PER_SECOND;

    trace!("Starting loop");
    event_loop.run(move |event, _, control_flow| {
        match &event {
            glutin::event::Event::LoopDestroyed => {
                trace!("Loop destroyed. Terminal can be shutdown");
                return;
            },
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    trace!("Received close event");
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => (),
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::Init => (),
                _ => {
                    // Timing code
                    let instant = Instant::now();
                    let eclapsed_frame = instant.duration_since(last_frame);
                    let eclapsed_tick = instant.duration_since(last_tick);

                    last_tick = instant;
                    tick_time_acc += eclapsed_tick;
                    // Check if we need updating
                    // Updating is ignored if a state change is requested
                    while transition_request.is_none() && tick_time_acc >= tick_duration {
                        // Do the updating
                        transition_request = state.update(&mut ctx, &input_tracker, tick_duration);

                        tick_time_acc -= tick_duration;
                    }
                    
                    // Check if we need rendering
                    if eclapsed_frame >= frame_duration {
                        // Zero the timing variables
                        last_frame = instant;

                        // Rendering
                        state.render(&mut ctx, &mut renders, &input_tracker);
                    }
                },
            },
            glutin::event::Event::MainEventsCleared => {
                use std::mem::replace;

                if transition_request.is_some() {
                    // Retrieve the transition request making the current one a `None`
                    let f = replace(&mut transition_request, None).unwrap();
                    // Retrieve the state, making the current on an `Empty`
                    let old_state = replace(&mut state, GameState::Empty);
                    // Execute it!
                    state = f(&mut ctx, old_state);

                    // Check if the state transition requested a `Quit`
                    if let GameState::Quit = &state {
                        trace!("Exit was requested (entered a quit state)");
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                }

                return;
            },
            _ => (),
        }

        // Forward the event to the state
        // Event processing is ignored if a state change was requested
        if transition_request.is_none() {
            input_tracker.process_event(&event);
            transition_request = state.process_event(&mut ctx, &input_tracker, &event)
        }
    });
}
