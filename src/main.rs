extern crate structopt;
extern crate glfw;
extern crate midir;
extern crate gl;

mod keycodes;
mod points;

use structopt::StructOpt;
use structopt::clap::ArgGroup;
use points::{BasisVector, Interval, point_to_interval};
use keycodes::key_to_point;
use glfw::{Action, Context, Key};
use midir::{MidiOutput, MidiOutputConnection};
use midir::os::unix::VirtualOutput;

// Bosanquet diagram for 19tet [3, 2, 0] "whole step" to right, smallest step up.
// Janko [-1, 1, -90]
// Inverted Gerhard [3, 4, -90] note i also liked this mapped on the keyboard rotated -60 [-1, 3, -90]
// Gerhard [4, 3, 0]
// Tonnetz [4, -3, 0]
// Guitar A [4, 5, -90]
// Guitar B [5, 6, -90]
// Park [5, 2, 0]
// Wesley [7, -5, 0]
// Fernandez [6, 7, -90]
// Wicki-Hayden [5, 7, -90]
// B-system [-2, 1, -90]
// C-system [-1, 2, -90]

#[derive(StructOpt)]
struct Basis {
    #[structopt(short = "a", long, default_value = "4")]
    upper: i8,
    #[structopt(short = "b", long, default_value = "-3")]
    lower: i8,
    #[structopt(short, long, default_value = "-90")]
    rotation: i8
}

const JANKO: Basis            = Basis { upper: -1, lower: 1,  rotation: -90 };
const INVERTED_GERHARD: Basis = Basis { upper: 3,  lower: 4,  rotation: -90 };
const GERHARD: Basis          = Basis { upper: 4,  lower: 3,  rotation: 0 };
const TONNETZ: Basis          = Basis { upper: 4,  lower: -3, rotation:  0 };
const GUITAR_A: Basis         = Basis { upper: 4,  lower: 5,  rotation: -90 };
const GUITAR_B: Basis         = Basis { upper: 5,  lower: 6,  rotation: -90 };
const PARK: Basis             = Basis { upper: 5,  lower: 2,  rotation: -90 };
const WESLEY: Basis           = Basis { upper: 7,  lower: -5, rotation: 0};
const FERNANDEZ: Basis        = Basis { upper: 6,  lower: 7,  rotation: -90 };
const WICKI_HAYDEN: Basis     = Basis { upper: 5,  lower: 7,  rotation: -90 };
const C_SYSTEM: Basis         = Basis { upper: -2, lower: 1,  rotation: -90 };
const B_SYSTEM: Basis         = Basis { upper: -1, lower: 2,  rotation: -90 };
const NAMES: &'static [&'static str] = &["C", "C#/Db", "D", "D#/Eb", "E", "F", "F#/Gb", "G", "G#/Ab", "A", "A#/Bb", "B"];
const PRESETS: &'static [&'static str] = &["janko", "inverted_gerhard", "gerhard", "tonnetz", "guitar_a", "guitar_b", "park", "wesley", "fernandez", "wicki-hayden", "c_system", "b_system"];

#[derive(StructOpt)]
struct BasisPreset {
    #[structopt(short, long, possible_values = PRESETS)]
    preset: Option<String>,
    #[structopt(flatten)]
    specified: Basis,
}

#[derive(StructOpt)]
#[structopt(group = ArgGroup::with_name("midi").required(true))]
struct MidiConfig {
    #[structopt(short = "v", long = "virtual", group = "midi")]
    use_virtual: bool,
    #[structopt(short = "l", long = "list", group = "midi")]
    list_outputs: bool,
    #[structopt(short, long, group = "midi")]
    output: Option<String>
}

#[derive(StructOpt)]
#[structopt(name = "Isomorphic Keyboard", version = "1.0", author = "Ashton Snelgrove")]
struct Opts {
    #[structopt(flatten)]
    basis: BasisPreset,
    #[structopt(flatten)]
    midi: MidiConfig,
    #[structopt(short, long, default_value = "0")]
    transpose: i8,
}

fn get_basis(basis: &BasisPreset) -> BasisVector {
    let basis_vector = match basis.preset.as_ref().map(String::as_ref) {
        Some("janko") =>  &JANKO,
        Some("inverted_gerhard") =>  &INVERTED_GERHARD,
        Some("gerhard") =>  &GERHARD,
        Some("tonnetz") =>  &TONNETZ,
        Some("guitar_a") =>  &GUITAR_A,
        Some("guitar_b") =>  &GUITAR_B,
        Some("park") =>  &PARK,
        Some("wesley") =>  &WESLEY,
        Some("fernandez") =>  &FERNANDEZ,
        Some("wicki_hayden") =>  &WICKI_HAYDEN,
        Some("b_system") =>  &B_SYSTEM,
        Some("c_system") =>  &C_SYSTEM,
        _ => &basis.specified,
    };

    if basis_vector.rotation == -90 {
        println!("vertical {} {}", basis_vector.upper, basis_vector.lower);
        BasisVector(basis_vector.upper, basis_vector.lower)
    } else {
        // Rotate -60 degrees to align with vertical
        println!("horizontal {} {}", basis_vector.upper, basis_vector.lower);
        BasisVector(basis_vector.upper - basis_vector.lower , basis_vector.upper)
    }
}

#[cfg(not(unix))]
fn create_virtual(_midi_out: MidiOutput) -> Result<MidiOutputConnection, &'static str> {
    Err("Virtual not available")
}

#[cfg(unix)]
fn create_virtual(midi_out: MidiOutput) -> Result<MidiOutputConnection, &'static str> {
    midi_out.create_virtual("Isomorphic Keyboard")
        .map_err(|_| "Unable to create virtual midi port")
}

fn setup_midi(opts: &Opts) -> Result<MidiOutputConnection, &'static str> {
    let midi_out  = MidiOutput::new("Isomorphic Keyboard Output").unwrap();
    if let Some(o) = &opts.midi.output {
        midi_out.ports().get(o.trim().parse::<usize>().unwrap())
            .ok_or_else(|| "Invalid port")
            .and_then(|p| {
                println!("Connecting to port {}", midi_out.port_name(p).unwrap());
                midi_out.connect(p, "Isomorphic Keyboard").map_err(|_| "Unable to connect")
            })
    } else if opts.midi.use_virtual {
        create_virtual(midi_out)
    } else if opts.midi.list_outputs {
        println!("Available MIDI outputs");
        for (i, p) in midi_out.ports().iter().enumerate() {
            println!("{}: {}", i, midi_out.port_name(p).unwrap());
        }
        Err("")
    } else {
        println!("Available MIDI outputs");
        for (i, p) in midi_out.ports().iter().enumerate() {
            println!("{}: {}", i, midi_out.port_name(p).unwrap());
        }
        Err("No output selected")
    }
}

fn main() {
    let opts: Opts = Opts::from_args();
    let basis = get_basis(&opts.basis);

    /******************************** MIDI ********************************/
    let out = setup_midi(&opts);
    if out.is_err() {
        println!("{}", out.err().unwrap());
        return;
    }
    let mut conn_out = out.unwrap();

    /******************************** GLFW Setup ********************************/
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw.create_window(300, 300, "Isomorphic MIDI Input", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader_program, vao) = create_program();

    /******************************** Main loop ********************************/
    while !window.should_close() {
        render(shader_program, vao);
        // Swap front and back buffers
        window.swap_buffers();

        // Poll for and process events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            //println!("{:?}", event);
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    // make sure the viewport matches the new window dimensions; note that width and
                    // height will be significantly larger than specified on retina displays.
                    unsafe { gl::Viewport(0, 0, width, height) }
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                },
                glfw::WindowEvent::Key(_, _, Action::Repeat, _) => {},
                glfw::WindowEvent::Key(key, _, action, _) => {
                    handle_keypress(key, action, &basis, opts.transpose, &mut conn_out)
                },
                _ => {}
            }
        }
    }
}

fn handle_keypress(key: Key, action: Action, basis: &BasisVector, transpose: i8, conn_out: &mut MidiOutputConnection) {
    match (key, action) {
        (Key::Space, Action::Press) => {
            println!("Sustain on");
            conn_out.send(&[0xB0, 0x40, 0x7F]).unwrap();
        },
        (Key::Space, Action::Release) => {
            println!("Sustain off");
            conn_out.send(&[0xB0, 0x40, 0x00]).unwrap();
        },
        (k, a) => {
            match key_to_point(k) {
                Some(k) => {
                    let Interval(i) = point_to_interval(basis, &k);
                    let note = (i + 60 + transpose) as u8;
                    let name = note % 12;
                    let octave = note / 12 - 2;
                    match a {
                        Action::Press => {
                            conn_out.send(&[0x90, note, 64]).unwrap();
                            println!("{}{} {} on",  NAMES[name as usize], octave, note);
                        },
                        Action::Release => {
                            conn_out.send(&[0x80, note, 0]).unwrap();
                            println!("{}{} {} off",  NAMES[name as usize], octave, note);
                        },
                        _ => {}
                    }
                },
                _ => println!("unmapped key {:?}", k),
            }
        },
    }
}

fn create_program() -> (u32, u32) {
    unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        (0, 0)
        //(shader_program, vao)
    }
}

fn render(_shader_program: u32, _vao: u32) {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}
