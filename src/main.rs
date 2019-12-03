extern crate glfw;
extern crate midir;

mod keycodes;
mod points;

use points::{BasisVector, Interval, point_to_interval};
use keycodes::key_to_point;
use glfw::{Action, Context, Key};
use midir::{MidiOutput, MidiOutputPort};
use std::io::stdin;

fn main() {
    let midi_out = MidiOutput::new("My Test Output").unwrap();
    let out_ports = midi_out.ports();
    let out_port: &MidiOutputPort = match out_ports.len() {
        0 => return,
        1 => {
            println!("Choosing the only available output port: {}", midi_out.port_name(&out_ports[0]).unwrap());
            &out_ports[0]
        },
        _ => {
            println!("\nAvailable output ports:");
            for (i, p) in out_ports.iter().enumerate() {
                println!("{}: {}", i, midi_out.port_name(p).unwrap());
            }
            print!("Please select output port: ");
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            out_ports.get(input.trim().parse::<usize>().unwrap())
                     .ok_or("invalid output port selected").unwrap()
        }
    };

    let mut conn_out = midi_out.connect(out_port, "midir-test").unwrap();

    let names = vec!["C", "C#/Db", "D", "D#/Eb", "E", "F", "F#/Gb", "G", "G#/Ab", "A", "A#/Bb", "B"];
    let basis = BasisVector(4, -3);

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw.create_window(300, 300, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);

    // Loop until the user closes the window
    while !window.should_close() {
        // Swap front and back buffers
        //        window.swap_buffers();

        // Poll for and process events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            //println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                },
                glfw::WindowEvent::Key(_, _, Action::Repeat, _) => {},
                glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    println!("Sustain on");
                    let _ = conn_out.send(&[0xB0, 0x40, 0x7F]);
                },
                glfw::WindowEvent::Key(Key::Space, _, Action::Release, _) => {
                    println!("Sustain off");
                    let _ = conn_out.send(&[0xB0, 0x40, 0x00]);
                },
                glfw::WindowEvent::Key(k, _, a, _) => {
                    match key_to_point(k) {
                        Some(k) => {
                            let Interval(i) = point_to_interval(&basis, &k);
                            let note = (i + 60) as u8;
                            let name = note % 12;
                            let octave = note / 12 - 2;
                            match a {
                                Action::Press => {
                                    let _ = conn_out.send(&[0x90, note, 64]);
                                    println!("{}{} {} on",  names[name as usize], octave, note);
                                },
                                Action::Release => {
                                    let _ = conn_out.send(&[0x80, note, 0]);
                                    println!("{}{} {} off",  names[name as usize], octave, note);
                                },
                                _ => {}
                            }
                        },
                        _ => println!("unknown {:?}", k),
                    }
                },
                _ => {}
            }
        }
    }
}
