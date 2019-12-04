extern crate structopt;
extern crate glfw;
extern crate midir;

mod keycodes;
mod points;

use structopt::StructOpt;
use points::{BasisVector, Interval, point_to_interval};
use keycodes::key_to_point;
use glfw::{Action, Context, Key};
use midir::{MidiOutput, MidiOutputPort};
use std::io::stdin;

// something else interesting [1, 3, -90] haha reinvented b-system in a different orientation
// Bosanquet diagram [3, 2, 0]
// Janko [-1, 1, -90]
// Inverted Gerhard [3, 4, -90] note i also liked this mapped on the keyboard rotated -60 [-1, 3, -90]
// Gerhard [4, 3, 0]
// Tonnetz [4, -3, 0]
// Guitar A [4, 5, -90]
// Guitar B [5, 6, -90]
// Park [5, 2, 0]
// Wesley [7, -5, 0]
// Fernandez [6, 7, -90]
// Wicki-Haydn [5, 7, -90]
// B-system [-2, 1, -90]
// C-system [-1, 2, -90]

#[derive(StructOpt)]
struct Basis {
    #[structopt(short, long, default_value = "4")]
    upper: i8,
    #[structopt(short, long, default_value = "-3")]
    lower: i8,
    #[structopt(short, long, default_value = "-90")]
    rotation: i8
}

const JANKO: Basis            = Basis { upper: -1, lower: 1,  rotation: -90 };
const INVERTED_GERHARD: Basis = Basis { upper: 3,  lower: 4,  rotation: -90 };
const GERHARD: Basis          = Basis { upper: 4,  lower: 3,  rotation: 0 };
const TONNETZ: Basis          = Basis { upper: 4,  lower: -3, rotation:  0 };
const GUITAR_A: Basis         = Basis { upper: -1, lower: 1,  rotation: -90 };
const GUITAR_B: Basis         = Basis { upper: -1, lower: 1,  rotation: -90 };
const PARK: Basis             = Basis { upper: 5,  lower: 2,  rotation: -90 };
const WESLEY: Basis           = Basis { upper: 7,  lower: -5, rotation: 0};
const FERNANDEZ: Basis        = Basis { upper: 6,  lower: 7,  rotation: -90 };
const WICKI_HAYDN: Basis      = Basis { upper: 5,  lower: 7,  rotation: -90 };
const B_SYSTEM: Basis         = Basis { upper: -2, lower: 1,  rotation: -90 };
const C_SYSTEM: Basis         = Basis { upper: -1, lower: 2,  rotation: -90 };
const NAMES: &'static [&'static str] = &["C", "C#/Db", "D", "D#/Eb", "E", "F", "F#/Gb", "G", "G#/Ab", "A", "A#/Bb", "B"];

#[derive(StructOpt)]
struct BasisPreset {
    #[structopt(short, long)]
    preset: Option<String>,
    #[structopt(flatten)]
    specified: Basis,
}

#[derive(StructOpt)]
#[structopt(name = "Isomorphic Keyboard", version = "1.0", author = "Ashton Snelgrove")]
struct Opts {
    #[structopt(flatten)]
    basis: BasisPreset,
    #[structopt(short, long, default_value = "0")]
    transpose: i8,
}

fn main() {
    let opts: Opts = Opts::from_args();

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

    let basis_vector = match opts.basis.preset.as_ref().map(String::as_ref) {
        Some("janko") =>  JANKO,
        Some("inverted_gerhard") =>  INVERTED_GERHARD,
        Some("gerhard") =>  GERHARD,
        Some("tonnetz") =>  TONNETZ,
        Some("guitar_a") =>  GUITAR_A,
        Some("guitar_b") =>  GUITAR_B,
        Some("park") =>  PARK,
        Some("wesley") =>  WESLEY,
        Some("fernandez") =>  FERNANDEZ,
        Some("wicki_haydn") =>  WICKI_HAYDN,
        Some("b_system") =>  B_SYSTEM,
        Some("c_system") =>  C_SYSTEM,
        _ => opts.basis.specified,
    };

    let basis = if basis_vector.rotation == -90 {
        println!("vertical");
        BasisVector(basis_vector.upper, basis_vector.lower)
    } else {
        // Rotate -60 degrees to align with vertical
        println!("horizontal");
        BasisVector(basis_vector.upper - basis_vector.lower , basis_vector.upper)
    };

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw.create_window(300, 300, "hello world!", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);

    // Loop until the user closes the window
    while !window.should_close() {
        // Swap front and back buffers
        //window.swap_buffers();

        // Poll for and process events
        glfw.wait_events();
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
                            let note = (i + 60 + opts.transpose) as u8;
                            let name = note % 12;
                            let octave = note / 12 - 2;
                            match a {
                                Action::Press => {
                                    let _ = conn_out.send(&[0x90, note, 64]);
                                    println!("{}{} {} on",  NAMES[name as usize], octave, note);
                                },
                                Action::Release => {
                                    let _ = conn_out.send(&[0x80, note, 0]);
                                    println!("{}{} {} off",  NAMES[name as usize], octave, note);
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
