use std::{
    fs::{self, File},
    io::{self, Read},
    path::PathBuf,
};

use bevy::prelude::*;

use self::diagnostics::Diagnostics;

mod diagnostics;
mod parsing;

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadSet>()
            .init_resource::<AvalibleSets>()
            .add_systems(Startup, load_set_system)
            .add_systems(Update, load_set_system.run_if(on_event::<LoadSet>()));
    }
}

#[derive(Debug, Clone, Event)]
pub struct LoadSet {
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SetHandle {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Default, Deref, Resource)]
pub struct AvalibleSets(Vec<SetHandle>);

fn load_set_system(
    mut event_reader: EventReader<LoadSet>,
    mut avalible_sets: ResMut<AvalibleSets>,
) {
    let mut diagnostics = Diagnostics::init();
    let set_name = event_reader
        .iter()
        .next()
        .and_then(|event| event.name.as_ref());

    avalible_sets.0.clear();
    if let Err(e) = load_avalible_sets(&mut avalible_sets.0) {
        // Uses Bevy diagnostic because end users should never encounter this
        // error.
        error!("Unable to load builtin sets: {e}");
        return;
    }
    avalible_sets.0.sort();

    if let Some(set_name) = set_name {
        let files = read_files(set_name, &avalible_sets, &mut diagnostics);
        if !diagnostics.has_errored() {
            let mut ast = Vec::new();
            for ((_, file), id) in files.iter().zip(0..) {
                ast.push(parsing::parse_file(file, id, &mut diagnostics));
            }
            dbg!(ast);
        }
        diagnostics.print_to_console(&files);
    } else {
        diagnostics.print_to_console(&[]);
    }
}

fn load_avalible_sets(avalible_sets: &mut Vec<SetHandle>) -> io::Result<()> {
    for entry in fs::read_dir("assets/sets/")? {
        match entry {
            Ok(entry) => {
                if entry
                    .file_type()
                    .is_ok_and(|ty| ty.is_dir() || ty.is_symlink())
                {
                    avalible_sets.push(SetHandle {
                        name: entry.file_name().to_string_lossy().into_owned(),
                        path: entry.path(),
                    })
                }
            }
            Err(e) => warn!("Unable to read entry in assets/sets/: {e}"),
        }
    }
    Ok(())
}

#[must_use]
fn read_files(
    set_name: &str,
    avalible_sets: &AvalibleSets,
    diagnostics: &mut Diagnostics,
) -> Vec<(String, Vec<u8>)> {
    let Some(set) = avalible_sets.iter().find(|set| set.name == set_name) else {
            // Uses Bevy diagnostic because end users should never encounter
            // this error.
            error!("Request to load set {set_name}, which does not exist");
            return Vec::new();
        };

    let mut files = Vec::new();

    let entries = match fs::read_dir(&set.path) {
        Ok(entries) => entries,
        Err(e) => {
            diagnostics
                .error("Unable to read set directiory")
                .context(e);
            return Vec::new();
        }
    };
    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if entry.file_type().is_ok_and(|ty| ty.is_file())
                    && path.extension().and_then(|s| s.to_str()) == Some("splang")
                {
                    let file_name = entry.file_name().to_string_lossy().into_owned();
                    let mut file = match File::open(path) {
                        Ok(file) => file,
                        Err(e) => {
                            diagnostics
                                .warn(format!("Unable to open file {file_name}; skipping"))
                                .context(e);
                            continue;
                        }
                    };

                    let mut buf = Vec::new();
                    buf.clear();
                    match file.read_to_end(&mut buf) {
                        Ok(_) => {
                            files.push((file_name, buf));
                        }
                        Err(e) => {
                            diagnostics
                                .warn(format!("Unable to read file {file_name}; skipping"))
                                .context(e);
                            continue;
                        }
                    }
                }
            }
            Err(e) => diagnostics
                .error("Unable to read entry in set directory")
                .context(e)
                .w(),
        }
    }

    files
}
