use std::{fs, io, path::PathBuf};

use bevy::prelude::*;

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
    let set_name = event_reader
        .iter()
        .next()
        .and_then(|event| event.name.as_ref());

    avalible_sets.0.clear();
    if let Err(e) = load_avalible_sets(&mut avalible_sets.0) {
        error!("Unable to load builtin sets: {e}");
        return;
    }
    avalible_sets.0.sort();
}

fn load_avalible_sets(avalible_sets: &mut Vec<SetHandle>) -> io::Result<()> {
    for entry in fs::read_dir("assets/sets/")? {
        let entry = entry?;
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
    Ok(())
}
