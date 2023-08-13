use std::{
    fs::{self, File},
    io::{self, Read},
    path::PathBuf,
};

use bevy::prelude::*;

use crate::terrain::{thread::TerrainThread, AtomWorld};

use self::diagnostics::{Diagnostic, Diagnostics};

use super::{
    element::Element,
    id::{IdMap, MappedToId},
};

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
    terrain_thread: Res<TerrainThread>,
) {
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
        if let Some(set) = avalible_sets.iter().find(|set| &set.name == set_name) {
            terrain_thread.load_set(set.clone());
        } else {
            // Uses Bevy diagnostic because end users should never encounter
            // this error.
            error!("Request to load set {set_name}, which does not exist");
        };
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

pub fn load_and_reload_set(set: SetHandle, world: &mut AtomWorld) {
    let mut diagnostics = Diagnostics::init();
    if let Some(new_elements) = load_set(&set, &mut diagnostics) {
        if !diagnostics.has_errored() {
            hot_reload_set(world, new_elements);
        }
    }
}

fn load_set(set: &SetHandle, diagnostics: &mut Diagnostics) -> Option<IdMap<Element>> {
    let files = read_files(set, diagnostics);
    let ret = (!diagnostics.has_errored()).then(|| {
        let mut elements = Element::create_map();
        for (id, _name, FileContents(file)) in files.iter() {
            parsing::parse_file(file, id, diagnostics, &mut elements);
        }
        elements
    });
    diagnostics.print_to_console(&files);
    ret
}

#[derive(Debug, Clone)]
struct FileContents(String);

impl MappedToId for FileContents {
    type Id = FileId;
}

type FileId = u16;

#[derive(Debug)]
pub enum ReadFilesError {
    ReadDirectory(std::io::Error),
    OpenFile { name: String, e: std::io::Error },
    ReadFile { name: String, e: std::io::Error },
}

impl Diagnostic for ReadFilesError {
    fn level(&self) -> diagnostics::Level {
        match self {
            ReadFilesError::OpenFile { .. } | ReadFilesError::ReadFile { .. } => {
                diagnostics::Level::Warn
            }
            ReadFilesError::ReadDirectory(_) => diagnostics::Level::Error,
        }
    }

    fn description(&self) -> String {
        match self {
            ReadFilesError::ReadDirectory(e) => {
                format!("Unable to read set directiory: {e}")
            }
            ReadFilesError::OpenFile { name, e } => {
                format!("Unable to open file {name}: {e}; skipping")
            }
            ReadFilesError::ReadFile { name, e } => {
                format!("Unable to read file {name}: {e}; skipping")
            }
        }
    }
}

#[must_use]
fn read_files(set: &SetHandle, diagnostics: &mut Diagnostics) -> IdMap<FileContents> {
    let mut files = FileContents::create_map();

    let entries = match fs::read_dir(&set.path) {
        Ok(entries) => entries,
        Err(e) => {
            diagnostics.add_unpositioned(ReadFilesError::ReadDirectory(e));
            return files;
        }
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if entry.file_type().is_ok_and(|ty| ty.is_file())
            && path.extension().and_then(|s| s.to_str()) == Some("splang")
        {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy().into_owned();
            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(e) => {
                    diagnostics.add_unpositioned(ReadFilesError::OpenFile { name: file_name, e });
                    continue;
                }
            };

            let mut buf = String::new();
            match file.read_to_string(&mut buf) {
                Ok(_) => {
                    files
                        .insert(file_name, FileContents(buf))
                        .expect("Files can't have the same name");
                }
                Err(e) => {
                    diagnostics.add_unpositioned(ReadFilesError::ReadFile { name: file_name, e });
                    continue;
                }
            }
        }
    }

    files
}

fn hot_reload_set(world: &mut AtomWorld, new_elements: IdMap<Element>) {
    world.atoms.modify_all(|mut atom| {
        if let Some((name, old_element)) = world.elements.get_full(atom.element) {
            if let Some((id, element)) = new_elements.get_full_by_name(name) {
                atom.element = id;

                macro_rules! change_if_default {
                    ($( $field:ident ),+) => {
                        $(
                            if atom.$field == old_element.$field
                                && atom.$field != element.$field {
                                atom.$field = element.$field
                            }
                        )+
                    };
                }

                change_if_default!(color);
                change_if_default!(join_face);
            } else {
                *atom = new_elements.air();
            }
        }
    });
    world.elements = new_elements;
}
