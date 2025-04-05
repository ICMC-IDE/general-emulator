use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(serde_derive::Deserialize, Debug)]
struct Group {
    pub size: u8,
    pub mask: u64,
    #[serde(deserialize_with = "deserialize_invert_hashmap")]
    pub subgroups: HashMap<u64, String>,
    #[serde(deserialize_with = "deserialize_invert_hashmap")]
    pub instructions: HashMap<u64, String>,
}

fn deserialize_invert_hashmap<'de, D>(deserializer: D) -> Result<HashMap<u64, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let original_map = HashMap::<String, u64>::deserialize(deserializer)?;
    let inverted_map = original_map
        .iter()
        .map(|(k, v)| (v.clone(), k.clone()))
        .collect::<HashMap<u64, String>>();
    Ok(inverted_map)
}

#[derive(serde_derive::Deserialize, Debug)]
struct Arch {
    pub groups: HashMap<String, Group>,
}

struct Emulator<'mem> {
    arch: Arch,
    instructions: HashMap<String, &'mem dyn Fn() -> ()>,
    memory: &'mem mut [u8],
}

impl<'mem> Emulator<'mem> {
    fn new(
        arch: Arch,
        instructions: HashMap<String, &'mem dyn Fn()>,
        memory: &'mem mut [u8],
    ) -> Self {
        Self {
            arch,
            instructions,
            memory,
        }
    }
    fn parse_group(&self, group: &str) -> &String {
        let group = self.arch.groups.get(group).unwrap();
        let mut buffer = [0u8; 8];
        self.memory
            .take((group.size / 8) as u64)
            .read(&mut buffer[(8 - (group.size / 8)) as usize..])
            .unwrap();

        let instruction = u64::from_be_bytes(buffer) & group.mask;

        println!("keys: {:?}", group.instructions.keys());
        group
            .instructions
            .get(&instruction)
            .unwrap_or_else(|| self.parse_group(group.subgroups.get(&instruction).unwrap()))
    }
    fn emulate(&self) {
        let instruction = self.parse_group("main");
        self.instructions.get(instruction).unwrap()();
    }
}

fn main() -> std::io::Result<()> {
    let mut file = File::open("main.toml")?;
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let arch = toml::from_str::<Arch>(&content).unwrap();

    let instructions_map = HashMap::new();

    let mut mem = [0xA0u8, 0x0, 0x0, 0x0];
    let emulator = Emulator::new(arch, instructions_map, &mut mem);
    emulator.emulate();

    Ok(())
}
