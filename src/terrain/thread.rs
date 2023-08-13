use std::thread;

use bevy::prelude::{error, Commands, Plugin, Resource, Startup};
use crossbeam_channel::{RecvError, SendError};

use crate::atom_physics::{self, element::Element};

use super::{storage::Atoms, AtomWorld};

pub(super) struct ThreadPlugin;

impl Plugin for ThreadPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, spawn_terrain_thread_system);
    }
}

fn spawn_terrain_thread_system(mut commands: Commands) {
    let (outside_sender, reciever) = crossbeam_channel::unbounded();
    let (outside_reciever, sender) = crossbeam_channel::unbounded();

    thread::Builder::new()
        .name("Terrain Thread".to_owned())
        .spawn(|| {
            let mut channel = Channel { sender, reciever };

            let mut world = AtomWorld {
                atoms: Atoms::default(),
                elements: Element::create_map(),
            };

            loop {
                match main_loop(&mut world, &mut channel) {
                    Ok(()) => continue,
                    Err(e) => match e {
                        CommunicationError::Send => error!("Main event loop dropped it's reciever"),
                        CommunicationError::Recv => error!("Main event loop dropped it's sender"),
                    },
                }
            }
        })
        .expect("Unable to create terrain thread!");

    commands.insert_resource(TerrainThread {
        sender: outside_sender,
        reciever: outside_reciever,
    });
}

#[derive(Debug, Clone, Resource)]
pub struct TerrainThread {
    sender: crossbeam_channel::Sender<Message>,
    reciever: crossbeam_channel::Receiver<MeshUpdate>,
}

#[derive(Debug)]
enum Message {
    LoadSet(atom_physics::io::SetHandle),
    UpdateMeshes,
}

#[derive(Debug)]
struct MeshUpdate {}

impl TerrainThread {
    pub fn load_set(&self, set: atom_physics::io::SetHandle) {
        Self::handle_communication_error(self.sender.send(Message::LoadSet(set)));
    }

    fn handle_communication_error<T: Into<CommunicationError>>(res: Result<(), T>) {
        match res {
            Ok(()) => {}
            Err(e) => match e.into() {
                CommunicationError::Send => error!("Terrain thread dropped it's reciever"),
                CommunicationError::Recv => error!("Terrain thread dropper it's sender"),
            },
        }
    }
}

struct Channel {
    sender: crossbeam_channel::Sender<MeshUpdate>,
    reciever: crossbeam_channel::Receiver<Message>,
}

enum CommunicationError {
    Send,
    Recv,
}

impl<T> From<SendError<T>> for CommunicationError {
    fn from(value: SendError<T>) -> Self {
        CommunicationError::Send
    }
}

impl From<RecvError> for CommunicationError {
    fn from(value: RecvError) -> Self {
        CommunicationError::Recv
    }
}

fn main_loop(world: &mut AtomWorld, channel: Channel) -> Result<(), CommunicationError> {
    let mut update_meshes = false;

    let first_message = channel.reciever.recv()?;
    process_message(first_message, &mut update_meshes, world);

    for message in channel.reciever.try_iter() {
        process_message(message, &mut update_meshes, world);
    }

    Ok(())
}

fn process_message(message: Message, update_meshes: &mut bool, world: &mut AtomWorld) {
    match message {
        Message::LoadSet(set) => atom_physics::io::load_and_reload_set(set, world),
        Message::UpdateMeshes => *update_meshes = true,
    }
}
