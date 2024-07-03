extern crate glob;

use std::{env, error::Error, fs, io, process::exit};
use soloud::*;
use std::fs::metadata;
use glob::glob;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("The command must has 2 arguments. Right version is <rust-player example1.mp3> or <rust-player /path>");
        exit(-1);
    }

    let buffer: String = args[1].clone();

    let mut is_running: bool = true;
    let mut current_track: usize = 0;
    let mut track_list: Vec<String> = open_file_or_directory_recursive(&buffer).expect("Error while opening a directory or file.");

    let stdin = io::stdin();
    let mut sl = Soloud::default().expect("Can not create a soloud");

    while is_running{
        let path: String = track_list[current_track%track_list.len()].clone();
        let file_name: String = path.split("/").last().unwrap().to_string();
    
        let mut music = audio::Wav::default();
        let mut is_loaded = true;

        music.load(&std::path::Path::new(&path)).unwrap_or_else(|e| {
            println!("Can not load a file due to{:?}", e);
            track_list.remove(current_track);
            is_loaded = false;
        });

        let h = sl.play(&music);

        sl.set_volume(h, 0.5);

        if is_loaded && is_running {
            print_sys_info(&file_name).unwrap_or_else(|e| {
                println!("Error while printing an info. {:?}", e)
            });
        }

        'mainloop: while sl.voice_count() > 0 && is_loaded {
            std::thread::sleep(std::time::Duration::from_millis(100));
            println!("Your command below: ");

            let mut command: String = String::new();
            stdin.read_line(&mut command).expect("Can not read a line");

            let command_query = command.split(" ").collect::<Vec<&str>>();

            match command_query[0] {
                "help\n" => {
                    show_help_page(&file_name).unwrap_or_else(|e|{
                        println!("Error with showing help page. {:?}", e);
                    });
                },
                "quit\n" | "q\n" => {
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                    is_running = false;
                    break 'mainloop;
                },
                "volume" | "vol" => {
                    set_current_track_volume(&file_name, &mut sl, &h, &command_query).unwrap_or_else(|e| {
                        println!("Error while changing the volume. {:?}", e);
                    });
                },
                "get_volume\n" | "gvol\n" => {
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                    println!("{} {}", sl.volume(h)*100.0, sl.global_volume()*100.0);
                },
                "global_volume" | "glvol" => {
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                    sl.set_global_volume(command_query[1][0..command_query[1].len() - 1]
                        .parse::<f32>()
                        .unwrap_or_else(|e| {
                            println!("Wrong value. The value must be between -100 and 100. etc 0.1. {:?}", e);
                            sl.global_volume()*100.0
                        })/100.0);
                },
                "pause\n" | "p\n" => {
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                    sl.set_pause(h, true);
                },
                "continue\n" | "c\n" => {
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                    sl.set_pause(h, false);
                },
                "next\n" | "n\n" => {
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                    current_track += 1;
                    sl.stop(h);
                },
                "next_by" | "nb" => {
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                    current_track += command_query[1][0..command_query[1].len() - 1]
                        .parse::<usize>()
                        .unwrap_or_else(|e| {
                            println!("Unexpected error!!! {:?}\n Try again.", e);
                            current_track
                        });
                    sl.stop(h);
                },
                "go_to" | "gt" => {
                    go_to_track(&file_name, &command_query, &h, &mut current_track, &track_list, &sl).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                },
                "list\n" | "l\n" => {
                    for index in 0..track_list.len() {
                        println!("{index}/{:}", track_list[index].split("/").last().unwrap().to_string());
                    }
                },
                "previous\n" | "prev\n" | "back\n" | "b\n" => {
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                    if current_track >= 1 {
                        current_track -= 1;
                        sl.stop(h);
                    } else {
                        println!("It is a first song.");
                    }
                },
                "clear\n" | "cl\n" => {
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                },
                "rename" | "rn" => {
                    let track_path: Vec<&str> = track_list[current_track].split("/").into_iter().collect();
                    let new_path: String = track_path[0..track_path.len() - 1].join("/") + "/" + command_query[1] + ".mp3";
                    print!("{:?} => {}", track_path, new_path);
                    fs::rename(&track_list[current_track], new_path).unwrap_or_else(|e| {
                        println!("Error with renaming. {}", e);
                    });
                },
                "copy" | "cp" => {
                    fs::copy(&track_list[current_track], command_query[1]).unwrap_or_else(|e| {
                        println!("Error while coping. {:?}", e);
                        0
                    });
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                },
                "delete\n" |  "del\n" => {
                    fs::remove_file(&track_list[current_track]).unwrap_or_else(|e| {
                        println!("Error while deleting the file. {:?}", e);
                    });
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                }
                _ => {
                    print_sys_info(&file_name).unwrap_or_else(|e| {
                        println!("Error while printing an info. {:?}", e);
                    });
                    println!("Wrong command! Type help to open a help page.");
                },
                
            }
        }
    }
    Ok(())    
}

fn print_sys_info (song_name: &String) -> Result<(), Box<dyn std::error::Error>> {
    let console_term_stdout = console::Term::stdout();
    console_term_stdout.clear_screen().unwrap_or_else(|e| {
        println!("Can not clear the screen. {:?}", e);
    });
    println!("Now playing <{:?}>", song_name);
    Ok(())
}

fn open_file_or_directory_recursive (path: &String) -> Result<Vec<String>, Box<dyn std::error::Error>>  {
    let mut res: Vec<String> = vec![];
    if metadata(&path).expect("Can not open your path").is_dir() {
        for entry in glob(&(path.to_string() + "/**/*.mp3")).expect("Can not open the directory.") {
            res.push(entry?.display().to_string());
        }
    } else if metadata(&path).expect("Can not open your path").is_file() {
        res.push(path.to_string());
    }
    Ok(res)
}

fn show_help_page (file_name: &String) -> Result<(), Box<dyn Error>> {
    print_sys_info(file_name).unwrap_or_else(|e| {
        println!("Error while printing an info. {:?}", e);
    });
    println!("Help page.\n
                pause or p => pause the song.\n
                continue or c => continue the song.\n
                help => Show this page.
                volume or vol => set the volume. New value msut be between -100 and 100.\n
                global_volume or glvol => set a global volume. New value msut be between -100 and 100.\n
                next or n => select a next song.\n
                next_by or nb => go to current + i track.\n
                prev or previous => select a previous song. Only on second or highder song.\n
                get_volume or gvol => print a current volume.\n
                go_to or gt => play the track by index in track list.\n
                list or l => show all tracks.\n
                clear or cl => clear the terminal.\n
                quit or q => exit from application.\n
            ");
    Ok(())
}

fn set_current_track_volume (file_name: &String, sl: &mut Soloud, h: &Handle, command_query: &Vec<&str>) -> Result<(), Box<dyn Error>> {
    print_sys_info(file_name).unwrap_or_else(|e| {
        println!("Error while printing an info. {:?}", e);
    });
    sl.set_volume(*h, command_query[1][0..command_query[1].len() - 1]
                        .parse::<f32>()
                        .unwrap_or_else(|e| {
                            println!("Wrong value. The value must be between -100 and 100. etc 0.1. {:?}", e);
                            sl.volume(*h)*100.0
                        })/100.0);
    Ok(())
}

fn go_to_track (file_name: &String, command_query: &Vec<&str>, h: &Handle,
                current_track: &mut usize, track_list: &Vec<String>, sl: &Soloud) -> Result<(), Box<dyn Error>> {
    print_sys_info(&file_name).unwrap_or_else(|e| {
        println!("Error while printing an info. {:?}", e);
    });
    let song_number: usize = command_query[1][0..command_query[1].len() - 1]
        .parse::<usize>()
        .unwrap_or_else(|e| {
            println!("Unexpected error!!! {:?}\n Try again.", e);
            *current_track
        });
    if song_number < track_list.len() {
        *current_track = song_number;
        sl.stop(*h);
    } else {
        println!("Value must be positive and less than {}", track_list.len());
    }
    Ok(())
}
