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
enum Register {
    R8(u8),
    R16(u16),
    R32(u32),
    R64(u64),
}

impl Register {
    fn read(&self) -> u64 {
        match self {
            Register::R8(v) => *v as u64,
            Register::R16(v) => *v as u64,
            Register::R32(v) => *v as u64,
            Register::R64(v) => *v,
        }
    }

    fn write(&mut self, value: u64) {
        match self {
            Register::R8(v) => *v = value as u8,
            Register::R16(v) => *v = value as u16,
            Register::R32(v) => *v = value as u32,
            Register::R64(v) => *v = value,
        }
    }
}

#[derive(serde_derive::Deserialize, Debug)]
struct Arch {
    pub groups: HashMap<String, Group>,
    pub registers: HashMap<String, Register>,
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

    println!("Arch: {:?}", arch);
    let instructions_map = HashMap::new();

    let mut mem = [0xA0u8, 0x0, 0x0, 0x0];
    let emulator = Emulator::new(arch, instructions_map, &mut mem);
    emulator.emulate();

    Ok(())
}
