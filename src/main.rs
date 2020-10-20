mod macros;
mod graphics_init;
mod basic_graphics_data;
mod loaders;
mod states;
mod graphics_utils;
mod input_tracker;
mod collision;
mod containers;
mod collision_models;

use std::fs::File;
use std::time::{ Duration, Instant };
use std::error::Error as StdError;

use log::trace;
use simplelog::*;

use input_tracker::InputTracker;
use graphics_init::GraphicsContext;
use states::{ UPDATE_FPS, GameState };

fn init_logging() -> Result<(), Box<dyn StdError>> {
    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)] {
            println!("Initializing debug logging facade");
            match TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed) {
                Some(logger) =>
                    CombinedLogger::init(
                        vec![
                            logger,
                            WriteLogger::new(LevelFilter::Error, Config::default(), File::create("err.log")?),
                        ]
                    )?
                ,
                None => WriteLogger::init(LevelFilter::Error, Config::default(), File::create("err.log")?)?,
            }
        } else {
            println!("Initializing release logging facade");
            WriteLogger::init(LevelFilter::Error, Config::default(), File::create("err.log")?)?
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn StdError>> {
    if let Err(err) = init_logging() { eprintln!("Failed to init logging"); return Err(err.into()) }

    let (mut ctx, event_loop, mut renders) = GraphicsContext::new()?;
    {
        use glutin::dpi::LogicalSize;

        //ctx.set_window_size(LogicalSize::new(1480.0f64, 720.0f64));
    }

    // Time of the previous frame
    let mut last_frame = Instant::now();

    let mut transition_request = None;
    let mut state = GameState::boot_state(&mut ctx);

    let mut input_tracker = InputTracker::new(ctx.display.gl_window().window());

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
                    let current_frame = Instant::now();
                    let eclapsed = current_frame.duration_since(last_frame);

                    // Check if we need updating
                    // Updating is ignored if a state change is requested
                    if transition_request.is_none() && eclapsed.as_secs_f32() >= 1.0f32 / UPDATE_FPS {
                        // Do the updating
                        transition_request = state.update(&mut ctx, &input_tracker);

                        // Zero the timing variables
                        last_frame = current_frame;

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
