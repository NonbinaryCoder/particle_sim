use std::{
    fs::{self, File},
    io::{self, Read},
    path::PathBuf,
};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::terrain::storage::Atoms;

use self::diagnostics::Diagnostics;

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
    hot_reload_params: HotReloadParams,
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
        if let Some(set) = avalible_sets.iter().find(|set| &set.name == set_name) {
            if let Some(elements) = load_set(set, &mut diagnostics) {
                if !diagnostics.has_errored() {
                    hot_reload_set(hot_reload_params, elements);
                }
            }
        } else {
            // Uses Bevy diagnostic because end users should never encounter
            // this error.
            error!("Request to load set {set_name}, which does not exist");
            diagnostics.print_to_console(&[]);
        };
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

fn load_set(set: &SetHandle, diagnostics: &mut Diagnostics) -> Option<IdMap<Element>> {
    let files = read_files(set, diagnostics);
    let ret = (!diagnostics.has_errored()).then(|| {
        let mut elements = Element::create_map();
        for ((_, file), id) in files.iter().zip(0..) {
            parsing::parse_file(file, id, diagnostics, &mut elements);
        }
        elements
    });
    diagnostics.print_to_console(&files);
    ret
}

#[must_use]
fn read_files(set: &SetHandle, diagnostics: &mut Diagnostics) -> Vec<(String, Vec<u8>)> {
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

#[derive(Debug, SystemParam)]
struct HotReloadParams<'w> {
    world: ResMut<'w, Atoms>,
    elements: ResMut<'w, IdMap<Element>>,
}

fn hot_reload_set(mut old: HotReloadParams, elements: IdMap<Element>) {
    old.world.modify_all(|mut atom| {
        if let Some((name, old_element)) = old.elements.get_full(atom.element) {
            if let Some((id, element)) = elements.get_full_by_name(name) {
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
            }
        }
    });
    *old.elements = elements;
}
